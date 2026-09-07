//! Backend-neutral lifecycle wrapper for host-owned application services.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Condvar, Mutex};

use wabou_shell_api::{HostService, HostServiceContext};

/// Cloneable handle for a resource whose lifetime is owned by a Wabou host.
///
/// The handle exists before the resource starts, so capabilities can capture
/// it without starting native work outside the host lifecycle.
pub struct HostServiceHandle<T> {
    name: &'static str,
    state: Arc<Mutex<HostServiceState<T>>>,
}

enum HostServiceState<T> {
    Stopped,
    Starting,
    Running(T),
    Failed {
        error: String,
        context: HostServiceContext,
    },
}

impl<T> Clone for HostServiceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            state: self.state.clone(),
        }
    }
}

impl<T: Clone> HostServiceHandle<T> {
    /// Clone the started resource, or report why it is unavailable.
    pub fn get(&self) -> Result<T, String> {
        match &*self
            .state
            .lock()
            .map_err(|_| format!("{} service state is poisoned", self.name))?
        {
            HostServiceState::Running(value) => Ok(value.clone()),
            HostServiceState::Starting => Err(format!("{} service is starting", self.name)),
            HostServiceState::Stopped => Err(format!("{} service is not running", self.name)),
            HostServiceState::Failed { error, .. } => {
                Err(format!("{} service failed to start: {error}", self.name))
            }
        }
    }
}

type HostServiceStart<T> = dyn Fn(&HostServiceContext) -> Result<T, String> + Send + Sync;
type HostServiceShutdown<T> = dyn Fn(T) -> Result<(), String> + Send + Sync;

fn panic_description(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&'static str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic payload")
}

/// Host-owned service paired with a stable handle for application code.
///
/// Startup is serialized with retry and shutdown. Initializer panics become
/// ordinary failed states, so the service can still be retried or shut down.
pub struct ManagedHostService<T> {
    name: &'static str,
    state: Arc<Mutex<HostServiceState<T>>>,
    settled: Arc<Condvar>,
    start: Arc<HostServiceStart<T>>,
    shutdown: Arc<HostServiceShutdown<T>>,
}

impl<T> Clone for ManagedHostService<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            state: self.state.clone(),
            settled: self.settled.clone(),
            start: self.start.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

/// Create a service that starts inside a Wabou host while exposing a handle
/// that capabilities can safely capture beforehand.
pub fn managed_host_service<T, Start, Shutdown>(
    name: &'static str,
    start: Start,
    shutdown: Shutdown,
) -> (HostServiceHandle<T>, ManagedHostService<T>)
where
    T: Clone + Send + Sync + 'static,
    Start: Fn(&HostServiceContext) -> Result<T, String> + Send + Sync + 'static,
    Shutdown: Fn(T) -> Result<(), String> + Send + Sync + 'static,
{
    let state = Arc::new(Mutex::new(HostServiceState::Stopped));
    (
        HostServiceHandle {
            name,
            state: state.clone(),
        },
        ManagedHostService {
            name,
            state,
            settled: Arc::new(Condvar::new()),
            start: Arc::new(start),
            shutdown: Arc::new(shutdown),
        },
    )
}

impl<T> HostService for ManagedHostService<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn start(&self, context: &HostServiceContext) -> Result<(), String> {
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| format!("{} service state is poisoned", self.name))?;
            match &*state {
                HostServiceState::Stopped | HostServiceState::Failed { .. } => {
                    *state = HostServiceState::Starting;
                }
                HostServiceState::Starting => {
                    return Err(format!("{} service is already starting", self.name));
                }
                HostServiceState::Running(_) => {
                    return Err(format!("{} service is already running", self.name));
                }
            }
        }

        let started = catch_unwind(AssertUnwindSafe(|| (self.start)(context))).map_err(|payload| {
            format!(
                "{} service initializer panicked: {}",
                self.name,
                panic_description(payload.as_ref())
            )
        });
        let value = match started.and_then(|result| result) {
            Ok(value) => value,
            Err(error) => {
                *self
                    .state
                    .lock()
                    .map_err(|_| format!("{} service state is poisoned", self.name))? =
                    HostServiceState::Failed {
                        error: error.clone(),
                        context: context.clone(),
                    };
                self.settled.notify_all();
                return Err(error);
            }
        };
        let mut state = self
            .state
            .lock()
            .map_err(|_| format!("{} service state is poisoned", self.name))?;
        if !matches!(*state, HostServiceState::Starting) {
            return Err(format!(
                "{} service state changed unexpectedly while starting",
                self.name
            ));
        }
        *state = HostServiceState::Running(value);
        self.settled.notify_all();
        Ok(())
    }

    fn shutdown(&self) -> Result<(), String> {
        let value = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| format!("{} service state is poisoned", self.name))?;
            while matches!(*state, HostServiceState::Starting) {
                state = self
                    .settled
                    .wait(state)
                    .map_err(|_| format!("{} service state is poisoned", self.name))?;
            }
            match std::mem::replace(&mut *state, HostServiceState::Stopped) {
                HostServiceState::Running(value) => Some(value),
                HostServiceState::Stopped | HostServiceState::Failed { .. } => None,
                HostServiceState::Starting => unreachable!("waited for service start to settle"),
            }
        };
        match value {
            Some(value) => (self.shutdown)(value),
            None => Ok(()),
        }
    }
}

impl<T> ManagedHostService<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Retry a failed start with the same host environment.
    pub fn retry(&self) -> Result<(), String> {
        let context = {
            let state = self
                .state
                .lock()
                .map_err(|_| format!("{} service state is poisoned", self.name))?;
            match &*state {
                HostServiceState::Failed { context, .. } => context.clone(),
                HostServiceState::Stopped => {
                    return Err(format!("{} service has not started", self.name));
                }
                HostServiceState::Starting => {
                    return Err(format!("{} service is already starting", self.name));
                }
                HostServiceState::Running(_) => {
                    return Err(format!("{} service is already running", self.name));
                }
            }
        };
        self.start(&context)
    }

    /// Retry on Tokio's blocking pool without blocking an async caller.
    pub async fn retry_async(&self) -> Result<(), String> {
        let runtime = tokio::runtime::Handle::try_current()
            .map_err(|_| format!("{} service async retry requires a Tokio runtime", self.name))?;
        let service = self.clone();
        runtime
            .spawn_blocking(move || service.retry())
            .await
            .map_err(|error| format!("{} service retry task failed: {error}", self.name))?
    }
}
