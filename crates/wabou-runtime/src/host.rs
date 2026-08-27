//! HostBuilder — Tauri-like entry point for wabou apps.
//!
//! Users register custom widgets (Rust `Widget` impls keyed by tag) +
//! named capabilities (Rust fns callable from JS), then `.run()`. Built-in widgets
//! (Canvas) are pre-registered; users override or add their own.
//!
//! ```ignore
//! use serde::{Deserialize, Serialize};
//! use wabou_bindgen::{JsonCapabilityContract, JsonMethod};
//! use wabou_runtime::{HostBuilder, Widget};
//!
//! #[derive(Deserialize)]
//! struct ReadFileRequest {
//!     path: String,
//! }
//!
//! #[derive(Serialize)]
//! struct ReadFileResponse {
//!     contents: String,
//! }
//!
//! const WORKSPACE: JsonCapabilityContract = JsonCapabilityContract::new("workspace", 1);
//!
//! HostBuilder::new()
//!     .widget("chart", || Box::new(MyChart::new()))
//!     .json_capability(WORKSPACE, |capability| {
//!         capability.method(JsonMethod::new("readFile"), |request: ReadFileRequest| async move {
//!             let contents = std::fs::read_to_string(request.path)
//!                 .map_err(|error| error.to_string())?;
//!             Ok::<_, String>(ReadFileResponse { contents })
//!         })
//!     })
//!     .run()
//!     .unwrap();
//! ```

use snafu::ResultExt;
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use vello::peniko::Color;

use wabou_bindgen::JsonCapabilityContract;
use wabou_bindgen::JsonMethod;

use crate::applier::Applier;
use crate::asset_cache::ResourceCache;
use crate::bundle;
use crate::json_capability::JsonCapability;
use crate::jsrt::JsRuntime;
use crate::native_capability::NativeCapability;
use crate::test_report::finish_test_report;
use crate::{HostMessageContext, HostMessageRouter};
use crate::{ShellExtension, WindowOptions, run_windows_with_factory_and_extensions, style};
use wabou_shell::{Widget, WidgetFactory};
use wabou_widgets::{SecretStore, builtin_factories, password_input_factory};

type CapabilityInstaller = Arc<dyn Fn(&JsRuntime) -> rquickjs::Result<()>>;
type HostMessageProducer = Arc<dyn Fn(HostMessageContext) + Send + Sync>;
type WindowSource = (Box<dyn crate::FrameSource>, WindowOptions);

const IMAGE_RESOURCES: JsonCapabilityContract = JsonCapabilityContract::new("imageResources", 1);
const CREATE_FILE_IMAGE: JsonMethod<CreateFileImageRequest, ImageResourceDescriptor> =
    JsonMethod::new("createFile");
const CREATE_NETWORK_IMAGE: JsonMethod<CreateNetworkImageRequest, ImageResourceDescriptor> =
    JsonMethod::new("createNetwork");
const RELEASE_IMAGE: JsonMethod<crate::ImageResourceHandle, bool> = JsonMethod::new("release");

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateFileImageRequest {
    path: PathBuf,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateNetworkImageRequest {
    url: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageResourceDescriptor {
    handle: crate::ImageResourceHandle,
    width: u32,
    height: u32,
}

/// Application-owned resource that must surround every native window and JS runtime.
pub trait HostService: Send + Sync {
    /// Stable name used in diagnostics.
    fn name(&self) -> &'static str;
    /// Start the service before Wabou creates JavaScript runtimes.
    fn start(&self, context: &HostServiceContext) -> Result<(), String>;
    /// Stop the service after the native event loop finishes.
    fn shutdown(&self) -> Result<(), String>;
}

/// Read-only environment available while a host-owned service starts.
#[derive(Clone, Debug)]
pub struct HostServiceContext {
    app_directories: Option<wabou_shell::AppDirectories>,
    behavior_test: bool,
    headless: bool,
}

impl HostServiceContext {
    /// Return the application directories resolved by the host, when configured.
    pub fn app_directories(&self) -> Option<&wabou_shell::AppDirectories> {
        self.app_directories.as_ref()
    }

    /// Return whether `wabou test` controls this host run.
    pub fn is_behavior_test(&self) -> bool {
        self.behavior_test
    }

    /// Return whether the deterministic headless shell is active.
    pub fn is_headless(&self) -> bool {
        self.headless
    }
}

/// Cloneable application handle for a resource whose lifetime is owned by
/// [`HostBuilder`]. The handle exists before the resource starts, so it can be
/// captured by capabilities and message producers without starting native
/// work outside [`HostBuilder::run`].
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
    /// Clone the started resource, or report that the host has not started it.
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

/// A host-owned service paired with a stable handle for application code.
///
/// Startup is serialized with retry and shutdown. If the host exits during an
/// asynchronous retry, shutdown waits for the synchronous initializer to
/// settle and then closes the produced value exactly once. Initializer panics
/// become ordinary failed states so the service can still be retried or shut
/// down without leaving the lifecycle stuck in `Starting`.
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

/// Create a service that starts inside [`HostBuilder::run`] while exposing a
/// handle that can safely be captured by capabilities beforehand.
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
                HostServiceState::Stopped => None,
                HostServiceState::Failed { .. } => None,
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
    /// Retry a failed start with the same host environment. Returns an error
    /// when the service has not failed or is already running/starting.
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

    /// Retry without blocking the caller's async executor or JavaScript event
    /// loop while the synchronous service initializer runs.
    ///
    /// This requires a Tokio runtime because the initializer is dispatched to
    /// its blocking pool. Calling it from another executor returns an error
    /// instead of panicking; use [`Self::retry`] when the caller owns its own
    /// blocking-task mechanism.
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

struct HostServicesGuard(Vec<Arc<dyn HostService>>);

impl HostServicesGuard {
    fn shutdown_all(&mut self) -> Vec<(&'static str, String)> {
        let mut failures = Vec::new();
        while let Some(service) = self.0.pop() {
            let name = service.name();
            let started = Instant::now();
            let result = service.shutdown();
            let elapsed = started.elapsed();
            if elapsed >= Duration::from_secs(1) {
                tracing::warn!(
                    service = name,
                    elapsed_ms = elapsed.as_millis(),
                    "host service shutdown was slow"
                );
            } else {
                tracing::debug!(
                    service = name,
                    elapsed_ms = elapsed.as_millis(),
                    "host service shut down"
                );
            }
            if let Err(error) = result {
                failures.push((service.name(), error));
            }
        }
        failures
    }

    fn finish(mut self) -> crate::Result<()> {
        let failures = self.shutdown_all();
        if failures.is_empty() {
            return Ok(());
        }
        Err(crate::Error::HostServiceShutdown {
            message: failures
                .into_iter()
                .map(|(name, error)| format!("`{name}`: {error}"))
                .collect::<Vec<_>>()
                .join("; "),
        })
    }
}

fn start_host_services(
    services: &[(Arc<dyn HostService>, bool)],
    context: &HostServiceContext,
) -> crate::Result<HostServicesGuard> {
    let mut started = HostServicesGuard(Vec::with_capacity(services.len()));
    for (service, required) in services {
        let name = service.name();
        let start = Instant::now();
        let result = service.start(context);
        let elapsed = start.elapsed();
        if elapsed >= Duration::from_secs(1) {
            tracing::warn!(
                service = name,
                elapsed_ms = elapsed.as_millis(),
                "host service startup was slow"
            );
        } else {
            tracing::debug!(
                service = name,
                elapsed_ms = elapsed.as_millis(),
                "host service started"
            );
        }
        if let Err(message) = result {
            if *required {
                return Err(crate::Error::HostService {
                    name: service.name(),
                    message,
                });
            }
            tracing::warn!(service = name, %message, "recoverable host service failed to start");
            continue;
        }
        started.0.push(service.clone());
    }
    Ok(started)
}

impl Drop for HostServicesGuard {
    fn drop(&mut self) {
        for (service, error) in self.shutdown_all() {
            tracing::warn!(service, %error, "failed to shut down host service");
        }
    }
}

struct ResourceCacheShutdownGuard {
    cache: Arc<ResourceCache>,
}

impl ResourceCacheShutdownGuard {
    fn new(cache: Arc<ResourceCache>) -> Self {
        Self { cache }
    }
}

impl Drop for ResourceCacheShutdownGuard {
    fn drop(&mut self) {
        if let Err(error) = self.cache.shutdown() {
            tracing::warn!(%error, "failed to gracefully close persistent asset cache");
        }
    }
}

#[cfg(feature = "profiling")]
fn init_tracing() -> Option<tracing_chrome::FlushGuard> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let Some(path) = std::env::var_os("WABOU_PROFILE_TRACE") else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .try_init()
            .ok();
        return None;
    };
    let file = match std::fs::File::create(&path) {
        Ok(file) => file,
        Err(error) => {
            eprintln!(
                "failed to create Wabou profile trace {}: {error}",
                Path::new(&path).display()
            );
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .try_init()
                .ok();
            return None;
        }
    };
    init_profile_tracing(file, env_filter)
}

#[cfg(feature = "profiling")]
fn init_profile_tracing(
    file: std::fs::File,
    env_filter: tracing_subscriber::EnvFilter,
) -> Option<tracing_chrome::FlushGuard> {
    use tracing_subscriber::filter::filter_fn;
    use tracing_subscriber::prelude::*;

    let (chrome, guard) = tracing_chrome::ChromeLayerBuilder::new()
        .writer(file)
        .include_args(true)
        .include_locations(false)
        .build();
    let initialized = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(chrome.with_filter(filter_fn(|metadata| metadata.target() == "wabou::perf")))
        .try_init()
        .is_ok();
    initialized.then_some(guard)
}

enum EffectTraceConfig {
    Record { path: PathBuf, record_all: bool },
    Replay { path: PathBuf },
}

#[derive(Clone)]
enum ApplicationSource {
    Bundle {
        code: Arc<str>,
        source_map: Option<Arc<[u8]>>,
    },
    #[cfg(feature = "vite")]
    Vite { url: Arc<str>, entry: Arc<str> },
}

#[derive(Clone)]
struct RuntimeSourceConfig {
    source: ApplicationSource,
    js_runtime_options: crate::JsRuntimeOptions,
    capabilities: Vec<CapabilityInstaller>,
    host_message_producers: Vec<HostMessageProducer>,
    widget_factories: HashMap<String, WidgetFactory>,
    base_color: Color,
    #[cfg(feature = "devtools")]
    debug_state: Option<wabou_devtools::SharedDebugState>,
    effect_trace: Option<crate::effect_trace::EffectTrace>,
    app_directories: Option<wabou_shell::AppDirectories>,
    asset_cache: Arc<ResourceCache>,
    image_resources: crate::ImageResourceStore,
}

impl RuntimeSourceConfig {
    fn create(
        &self,
        window_key: wabou_shell::WindowResourceKey,
        window_options: &WindowOptions,
    ) -> crate::Result<Applier> {
        #[cfg(feature = "vite")]
        let js = match &self.source {
            ApplicationSource::Vite { url, .. } => {
                JsRuntime::new_vite_with_options(url, self.js_runtime_options).context(
                    crate::error::JavaScriptSnafu {
                        operation: "create Vite JavaScript runtime",
                    },
                )?
            }
            ApplicationSource::Bundle { .. } => JsRuntime::new_with_options(
                self.js_runtime_options,
            )
            .context(crate::error::JavaScriptSnafu {
                operation: "create JavaScript runtime",
            })?,
        };
        #[cfg(not(feature = "vite"))]
        let js = JsRuntime::new_with_options(self.js_runtime_options).context(
            crate::error::JavaScriptSnafu {
                operation: "create JavaScript runtime",
            },
        )?;

        for capability in &self.capabilities {
            capability(&js).context(crate::error::JavaScriptSnafu {
                operation: "mount JavaScript capability",
            })?;
        }
        let serialized_window_options =
            serde_json::to_string(window_options).expect("WindowOptions must remain serializable");
        js.with(|ctx| {
            ctx.globals()
                .set("__wabou_window_options_json", serialized_window_options)
        })
        .context(crate::error::JavaScriptSnafu {
            operation: "install native window creation options",
        })?;
        let mut applier = Applier::from_runtime_with_factories_and_window(
            js,
            self.widget_factories.clone(),
            if window_options.transparent {
                Color::TRANSPARENT
            } else {
                self.base_color
            },
            window_key,
        );
        install_host_message_producers(&self.host_message_producers, window_key, &applier);
        applier.set_asset_cache(self.asset_cache.clone());
        applier.set_image_resource_store(self.image_resources.clone());
        if let Some(directories) = &self.app_directories {
            applier.set_app_directories(directories.clone());
        }
        if let Some(trace) = &self.effect_trace {
            applier.set_effect_trace(trace.clone());
        }
        #[cfg(feature = "devtools")]
        if let Some(state) = &self.debug_state {
            // Install diagnostics before the guest boots so application-owned
            // debug controls can configure the first rendered frame.
            applier.set_debug_state(state.clone());
        }
        match &self.source {
            ApplicationSource::Bundle { code, source_map } => applier
                .boot_with_source_map(code, source_map.as_deref())
                .context(crate::error::JavaScriptSnafu {
                    operation: "boot JavaScript bundle",
                })?,
            #[cfg(feature = "vite")]
            ApplicationSource::Vite { entry, .. } => {
                applier
                    .boot_vite(entry)
                    .context(crate::error::JavaScriptSnafu {
                        operation: "boot Vite entry module",
                    })?
            }
        }
        Ok(applier)
    }

    #[cfg(feature = "vite")]
    fn start_hmr(&self, applier: &mut Applier) -> crate::Result<Option<crate::HmrClient>> {
        let ApplicationSource::Vite { url, entry } = &self.source else {
            return Ok(None);
        };
        applier.set_vite_entry(entry.as_ref());
        crate::start_hmr_client(url, applier.reload_handle())
            .map(Some)
            .context(crate::error::ViteSnafu)
    }
}

/// Application-facing builder for windows, widgets, capabilities, and tooling.
pub struct HostBuilder {
    base_color: Color,
    window: WindowOptions,
    additional_windows: Vec<WindowOptions>,
    widget_factories: HashMap<String, WidgetFactory>,
    capabilities: Vec<CapabilityInstaller>,
    host_message_producers: Vec<HostMessageProducer>,
    services: Vec<(Arc<dyn HostService>, bool)>,
    devtools: bool,
    extensions: Vec<Box<dyn ShellExtension>>,
    effect_trace: Option<EffectTraceConfig>,
    app_directory_config: Option<wabou_shell::AppDirectoryConfig>,
    persisted_window_size: Option<String>,
    image_resources: crate::ImageResourceStore,
    js_runtime_options: crate::JsRuntimeOptions,
}

impl Default for HostBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostBuilder {
    /// Create a builder with built-in widgets pre-registered (`"canvas"` →
    /// `Canvas`). In development the CLI supplies a Vite URL; packaged apps
    /// load `resources/bundle.js` next to the executable.
    pub fn new() -> Self {
        Self::with_image_resources(crate::ImageResourceStore::default())
    }

    /// Create a builder using an image registry also owned by application Rust code.
    pub fn with_image_resources(image_resources: crate::ImageResourceStore) -> Self {
        let builder = Self {
            base_color: style::parse_color("#0f172a")
                .unwrap_or_else(|| Color::from_rgb8(0x0f, 0x17, 0x2a)),
            window: WindowOptions::default(),
            additional_windows: Vec::new(),
            widget_factories: builtin_factories(),
            capabilities: Vec::new(),
            host_message_producers: Vec::new(),
            services: Vec::new(),
            devtools: cfg!(all(debug_assertions, feature = "devtools")),
            extensions: Vec::new(),
            effect_trace: None,
            app_directory_config: None,
            persisted_window_size: None,
            image_resources: image_resources.clone(),
            js_runtime_options: crate::JsRuntimeOptions::default(),
        };
        let mounted = image_resources.clone();
        builder.json_capability(IMAGE_RESOURCES, move |capability| {
            let files = mounted.clone();
            capability.method(CREATE_FILE_IMAGE, move |request: CreateFileImageRequest| {
                let resources = files.clone();
                async move {
                    let loader = resources.clone();
                    let handle =
                        tokio::task::spawn_blocking(move || loader.create_file(request.path))
                            .await
                            .map_err(|error| error.to_string())??;
                    let (width, height) = resources
                        .get(handle)
                        .ok_or_else(|| "created image resource did not resolve".to_owned())?
                        .dimensions();
                    Ok::<_, String>(ImageResourceDescriptor {
                        handle,
                        width,
                        height,
                    })
                }
            })?;
            let network = mounted.clone();
            capability.method(
                CREATE_NETWORK_IMAGE,
                move |request: CreateNetworkImageRequest| {
                    let resources = network.clone();
                    async move {
                        let handle = resources.create_network(&request.url).await?;
                        let (width, height) = resources
                            .get(handle)
                            .ok_or_else(|| "created image resource did not resolve".to_owned())?
                            .dimensions();
                        Ok::<_, String>(ImageResourceDescriptor {
                            handle,
                            width,
                            height,
                        })
                    }
                },
            )?;
            let release = mounted.clone();
            capability.method(RELEASE_IMAGE, move |handle: crate::ImageResourceHandle| {
                let resources = release.clone();
                async move { Ok::<_, String>(resources.remove(handle)) }
            })
        })
    }

    /// Register or override a widget factory for `tag`. When the SolidJS app
    /// creates an element with this tag (`<chart />`), the factory is called
    /// to produce a fresh `Box<dyn Widget>`.
    pub fn widget(
        mut self,
        tag: impl Into<String>,
        factory: impl Fn() -> Box<dyn Widget> + 'static,
    ) -> Self {
        self.widget_factories.insert(tag.into(), Arc::new(factory));
        self
    }

    /// Register the framework password widget backed by a Rust-only secret store.
    pub fn password_inputs(mut self, secrets: SecretStore) -> Self {
        self.widget_factories
            .insert("password-input".into(), password_input_factory(secrets));
        self
    }

    fn mount_capability<F>(mut self, name: impl Into<String>, mount: F) -> Self
    where
        F: for<'js> Fn(rquickjs::Ctx<'js>, rquickjs::Object<'js>) -> rquickjs::Result<()>
            + rquickjs::markers::ParallelSend
            + Send
            + Sync
            + 'static,
    {
        let name = name.into();
        let mount = Arc::new(mount);
        self.capabilities.push(Arc::new(move |js| {
            let mount = mount.clone();
            js.mount_capability(&name, move |ctx, capability| mount(ctx, capability))
        }));
        self
    }

    /// Mount structured async methods without exposing QuickJS transport
    /// details to application code.
    pub fn json_capability<F>(self, contract: JsonCapabilityContract, mount: F) -> Self
    where
        F: for<'js> Fn(JsonCapability<'js>) -> rquickjs::Result<()>
            + rquickjs::markers::ParallelSend
            + Send
            + Sync
            + 'static,
    {
        self.mount_capability(contract.name(), move |ctx, object| {
            object.set("__wabouCapabilityVersion", contract.version())?;
            mount(JsonCapability { ctx, object })
        })
    }

    /// Mount typed asynchronous methods that exchange structured QuickJS
    /// values directly without JSON text encoding.
    pub fn native_capability<F>(self, contract: JsonCapabilityContract, mount: F) -> Self
    where
        F: for<'js> Fn(NativeCapability<'js>) -> rquickjs::Result<()>
            + rquickjs::markers::ParallelSend
            + Send
            + Sync
            + 'static,
    {
        self.mount_capability(contract.name(), move |ctx, object| {
            object.set("__wabouCapabilityVersion", contract.version())?;
            mount(NativeCapability { ctx, object })
        })
    }

    /// Register a producer for application-level Rust → JavaScript messages.
    ///
    /// The callback runs once for every native window after that window's
    /// bounded message queue is created and before its JavaScript bundle boots.
    /// Background tasks may retain the cloneable context and emit from any
    /// thread. JavaScript receives values through `hostMessages.subscribe()`.
    /// The context is cancelled when its window is dropped.
    ///
    /// Producers should use `context.window_key()` to avoid duplicate global
    /// streams when an application creates additional windows.
    pub fn host_message_producer<F>(mut self, producer: F) -> Self
    where
        F: Fn(HostMessageContext) + Send + Sync + 'static,
    {
        self.host_message_producers.push(Arc::new(producer));
        self
    }

    /// Connect a window-addressable message router to every JavaScript runtime.
    /// Native callbacks may retain a clone and send without polling or touching
    /// QuickJS from the callback thread.
    pub fn host_message_router(mut self, router: HostMessageRouter) -> Self {
        self.host_message_producers
            .push(Arc::new(move |context| router.attach(context)));
        self
    }

    /// Own a process, database, or background service for the full host lifetime.
    pub fn service(mut self, service: impl HostService + 'static) -> Self {
        self.services.push((Arc::new(service), true));
        self
    }

    /// Own a service whose startup failure leaves the application available in
    /// a degraded state. Capabilities using a managed handle receive the
    /// original startup diagnostic instead of a generic "not running" error.
    pub fn recoverable_service(mut self, service: impl HostService + 'static) -> Self {
        self.services.push((Arc::new(service), false));
        self
    }

    /// Configure the primary native window.
    pub fn window(mut self, options: WindowOptions) -> Self {
        self.window = options;
        self
    }

    /// Create another independent native window running the same application
    /// bundle, widget registry and host capabilities.
    pub fn additional_window(mut self, options: WindowOptions) -> Self {
        self.additional_windows.push(options);
        self
    }

    /// Set the viewport clear color behind the retained root.
    pub fn base_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self
    }

    /// Configure the maximum native stack available to each QuickJS runtime.
    pub fn quickjs_stack_size(mut self, bytes: usize) -> Self {
        self.js_runtime_options = self.js_runtime_options.max_stack_size(bytes);
        self
    }

    /// Configure the stable identity used by app-private storage directories.
    pub fn app_directories(
        self,
        qualifier: impl Into<String>,
        organization: impl Into<String>,
        application: impl Into<String>,
    ) -> Self {
        self.app_directory_config(wabou_shell::AppDirectoryConfig::new(
            qualifier,
            organization,
            application,
        ))
    }

    /// Set an already constructed stable application directory identity.
    pub fn app_directory_config(mut self, config: wabou_shell::AppDirectoryConfig) -> Self {
        self.app_directory_config = Some(config);
        self
    }

    /// Restore and persist the primary window's normal logical size.
    ///
    /// This requires [`Self::app_directories`] so the state has a stable,
    /// application-private location. `key` distinguishes independently sized
    /// window roles and may contain ASCII letters, numbers, `-`, or `_`.
    pub fn persist_window_size(mut self, key: impl Into<String>) -> Self {
        let key = key.into();
        assert!(
            !key.is_empty()
                && key
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
            "window persistence key must contain only ASCII letters, numbers, '-' or '_'"
        );
        self.persisted_window_size = Some(key);
        self
    }

    /// Enable or disable the local, read-only DevTools socket. It defaults to
    /// enabled in debug builds and is absent from release builds unless opted in.
    pub fn devtools(mut self, enabled: bool) -> Self {
        self.devtools = enabled;
        self
    }

    /// Install a native integration that shares Wabou's platform event loop.
    pub fn extension(mut self, extension: impl ShellExtension + 'static) -> Self {
        self.extensions.push(Box::new(extension));
        self
    }

    /// Record replay-safe native effects to `path` when the application exits.
    /// Clipboard and third-party payloads are excluded because they may contain secrets.
    pub fn record_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Record {
            path: path.into(),
            record_all: false,
        });
        self
    }

    /// Record every native effect. This can persist clipboard or extension secrets.
    pub fn record_all_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Record {
            path: path.into(),
            record_all: true,
        });
        self
    }

    /// Replay recorded effects without invoking their native implementations.
    /// Operations absent from the tape continue to execute live.
    pub fn replay_effects(mut self, path: impl Into<PathBuf>) -> Self {
        self.effect_trace = Some(EffectTraceConfig::Replay { path: path.into() });
        self
    }

    /// Build the JsRuntime + Applier + run the winit event loop.
    pub fn run(self) -> crate::Result<()> {
        let outcome = self.run_once()?;
        if outcome == crate::RunOutcome::Relaunch && std::env::var_os("WABOU_TEST_SCRIPT").is_none()
        {
            relaunch_current_process()?;
        }
        Ok(())
    }

    fn run_once(mut self) -> crate::Result<crate::RunOutcome> {
        #[cfg(feature = "profiling")]
        let _profile_guard = init_tracing();
        #[cfg(not(feature = "profiling"))]
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .try_init()
            .ok();

        let test_script = std::env::var_os("WABOU_TEST_SCRIPT")
            .map(PathBuf::from)
            .map(|path| {
                std::fs::read_to_string(&path).context(crate::error::ReadFileSnafu {
                    kind: "test scenario bundle",
                    path,
                })
            })
            .transpose()?;
        let trace_path = self.effect_trace.as_ref().map(|config| match config {
            EffectTraceConfig::Record { path, .. } | EffectTraceConfig::Replay { path } => {
                path.clone()
            }
        });
        let mut effect_trace = match &self.effect_trace {
            Some(EffectTraceConfig::Record { record_all, .. }) => {
                Some(crate::effect_trace::EffectTrace::record(*record_all))
            }
            Some(EffectTraceConfig::Replay { path }) => Some(
                crate::effect_trace::EffectTrace::replay(path)
                    .map_err(|message| crate::Error::EffectTrace { message })?,
            ),
            None => None,
        };
        let recording_effects = matches!(self.effect_trace, Some(EffectTraceConfig::Record { .. }));
        if test_script.is_some() && effect_trace.is_none() {
            effect_trace = Some(crate::effect_trace::EffectTrace::fixtures());
        }
        let test_controller = test_script.as_ref().map(|_| {
            crate::test_driver::TestController::new(
                effect_trace
                    .clone()
                    .expect("behavior tests always install an effect fixture bridge"),
            )
        });
        let headless_test = test_controller.is_some()
            && std::env::var("WABOU_TEST_HEADLESS").is_ok_and(|value| value != "0");
        if let Some(controller) = &test_controller {
            let capability_controller = controller.clone();
            self.capabilities
                .push(Arc::new(move |js| capability_controller.mount(js)));
            if !headless_test {
                self.extensions
                    .push(Box::new(crate::test_driver::TestDriver::new(
                        controller.clone(),
                    )));
            }
        }
        let app_directories = self
            .app_directory_config
            .as_ref()
            .map(|config| {
                let resource = bundle::resource_directory()?;
                wabou_shell::AppDirectories::resolve(config, resource).ok_or_else(|| {
                    crate::Error::AppDirectories {
                        application: "configured application".into(),
                    }
                })
            })
            .transpose()?;
        let service_context = HostServiceContext {
            app_directories: app_directories.clone(),
            behavior_test: test_controller.is_some(),
            headless: headless_test,
        };
        let services = start_host_services(&self.services, &service_context)?;
        if let Some(key) = &self.persisted_window_size {
            if let Some(directories) = &app_directories {
                let path = directories
                    .local_data_dir
                    .join("window-state")
                    .join(format!("{key}.json"));
                let persistence = wabou_shell::WindowSizePersistence::restore(
                    path,
                    wabou_shell::initial_window_resource_key(0),
                    &mut self.window,
                );
                // Observe close before a tray extension consumes the request.
                self.extensions.insert(0, Box::new(persistence));
            } else {
                tracing::warn!(
                    key,
                    "window size persistence requires HostBuilder::app_directories"
                );
            }
        }
        let windows = std::iter::once(self.window.clone())
            .chain(self.additional_windows.iter().cloned())
            .collect::<Vec<_>>();
        let asset_cache = Arc::new(if let Some(directories) = &app_directories {
            ResourceCache::with_disk(&directories.cache_dir).unwrap_or_else(|error| {
                tracing::warn!(%error, "failed to enable persistent asset cache");
                ResourceCache::memory_only()
            })
        } else {
            ResourceCache::memory_only()
        });
        // Declared immediately after the cache so it runs on successful exit
        // and on every later `?` path, before the cache-owned runtime is dropped.
        let _asset_cache_shutdown = ResourceCacheShutdownGuard::new(asset_cache.clone());
        self.image_resources.set_cache(asset_cache.clone());

        #[cfg(feature = "devtools")]
        let devtools_server = {
            let mut server = None;
            let debug_state = (self.devtools
                || std::env::var_os("WABOU_TEST_SNAPSHOT_PATH").is_some())
            .then(wabou_devtools::DebugState::shared);
            if self.devtools {
                let path = wabou_devtools::socket_path();
                server = Some(
                    wabou_devtools::serve(debug_state.as_ref().unwrap().clone(), path.clone())
                        .context(crate::error::DevtoolsSnafu)?,
                );
                tracing::info!(target: "devtools", socket = %path.display(), "Wabou DevTools listening");
            }
            (server, debug_state)
        };
        #[cfg(not(feature = "devtools"))]
        if self.devtools {
            tracing::warn!("HostBuilder::devtools(true) requires the `wabou/devtools` feature");
        }

        #[cfg(feature = "vite")]
        let source = if let Ok(url) = std::env::var("WABOU_VITE_URL") {
            ApplicationSource::Vite {
                url: url.into(),
                entry: std::env::var("WABOU_VITE_ENTRY")
                    .unwrap_or_else(|_| "src/index.tsx".to_string())
                    .into(),
            }
        } else {
            ApplicationSource::Bundle {
                code: bundle::load()?.into(),
                source_map: bundle::load_source_map()?.map(Into::into),
            }
        };
        #[cfg(not(feature = "vite"))]
        let source = ApplicationSource::Bundle {
            code: bundle::load()?.into(),
            source_map: bundle::load_source_map()?.map(Into::into),
        };
        let runtime_sources = RuntimeSourceConfig {
            source,
            js_runtime_options: self.js_runtime_options,
            capabilities: self.capabilities.clone(),
            host_message_producers: self.host_message_producers.clone(),
            widget_factories: self.widget_factories.clone(),
            base_color: self.base_color,
            #[cfg(feature = "devtools")]
            debug_state: devtools_server.1.clone(),
            effect_trace: effect_trace.clone(),
            app_directories: app_directories.clone(),
            asset_cache: asset_cache.clone(),
            image_resources: self.image_resources.clone(),
        };
        #[cfg(feature = "vite")]
        let mut hmr_clients = Vec::new();
        let mut sources = Vec::with_capacity(windows.len());
        for (index, options) in windows.into_iter().enumerate() {
            let window_key = wabou_shell::initial_window_resource_key(index);
            #[cfg_attr(not(feature = "vite"), allow(unused_mut))]
            let mut applier = runtime_sources.create(window_key, &options)?;
            if index == 0
                && let Some(script) = &test_script
            {
                applier
                    .eval_script(script)
                    .context(crate::error::JavaScriptSnafu {
                        operation: "evaluate test scenario",
                    })?;
            }
            #[cfg(feature = "vite")]
            if let Some(client) = runtime_sources.start_hmr(&mut applier)? {
                hmr_clients.push(client);
            }
            sources.push((Box::new(applier) as Box<dyn crate::FrameSource>, options));
        }

        if headless_test && let Some(controller) = &test_controller {
            run_headless_test(
                controller,
                &mut sources,
                self.base_color,
                #[cfg(feature = "devtools")]
                devtools_server.1.as_ref(),
            )?;
            services.finish()?;
            return Ok(crate::RunOutcome::Exit);
        }

        #[cfg(feature = "vite")]
        let child_hmr_clients = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        #[cfg(feature = "vite")]
        let child_hmr_store = child_hmr_clients.clone();
        let child_sources = runtime_sources.clone();
        #[allow(clippy::arc_with_non_send_sync)] // winit invokes this only on its event thread.
        let factory: crate::FrameSourceFactory = Arc::new(move |window_key, options| {
            #[cfg_attr(not(feature = "vite"), allow(unused_mut))]
            let mut applier = child_sources
                .create(window_key, options)
                .map_err(|error| error.to_string())?;
            #[cfg(feature = "vite")]
            if let Some(client) = child_sources
                .start_hmr(&mut applier)
                .map_err(|error| error.to_string())?
            {
                child_hmr_store.lock().unwrap().push(client);
            }
            Ok(Box::new(applier))
        });

        let run_result =
            run_windows_with_factory_and_extensions(sources, Some(factory), self.extensions)
                .context(crate::error::ShellSnafu);
        let trace_result =
            if recording_effects && let (Some(trace), Some(path)) = (&effect_trace, trace_path) {
                trace
                    .write(&path)
                    .map_err(|message| crate::Error::EffectTrace { message })
            } else {
                Ok(())
            };
        let outcome = run_result?;
        trace_result?;
        if let Some(controller) = test_controller {
            finish_test_report(controller)?;
        }
        #[cfg(feature = "vite")]
        drop(hmr_clients);
        #[cfg(feature = "vite")]
        drop(child_hmr_clients);
        #[cfg(feature = "devtools")]
        drop(devtools_server);
        services.finish()?;
        Ok(outcome)
    }
}

fn relaunch_current_process() -> crate::Result<()> {
    let executable = std::env::current_exe().map_err(|error| crate::Error::HostService {
        name: "application relaunch",
        message: format!("cannot resolve current executable: {error}"),
    })?;
    let mut command = std::process::Command::new(executable);
    command.args(std::env::args_os().skip(1));
    command.spawn().map_err(|error| crate::Error::HostService {
        name: "application relaunch",
        message: format!("cannot launch replacement process: {error}"),
    })?;
    Ok(())
}

fn install_host_message_producers(
    producers: &[HostMessageProducer],
    window_key: wabou_shell::WindowResourceKey,
    applier: &Applier,
) {
    for producer in producers {
        producer(applier.host_message_context(window_key));
    }
}

fn run_headless_test(
    controller: &crate::test_driver::TestController,
    sources: &mut [WindowSource],
    base_color: Color,
    #[cfg(feature = "devtools")] debug_state: Option<&wabou_devtools::SharedDebugState>,
) -> crate::Result<()> {
    let viewport = HeadlessViewport::from_environment()?;

    controller.initialize_headless(
        (0..sources.len()).map(wabou_shell::initial_window_resource_key),
        viewport.width,
        viewport.height,
    );
    // JavaScript reports individual test timeouts with the test name. This is
    // only a final safety net for a broken runtime or runner that cannot
    // produce a report at all.
    // The longest JavaScript test/replay budget is 60 seconds. Keep the host
    // deadline later so JavaScript can serialize its named failure first.
    let deadline = Instant::now() + Duration::from_secs(65);
    let mut text = crate::TextContext::new();
    let mut last_nodes = vec![Vec::new(); sources.len()];
    let mut profilers = (0..sources.len())
        .map(|_| wabou_shell::headless::HeadlessFrameProfiler::default())
        .collect::<Vec<_>>();
    while !controller.has_report() && Instant::now() < deadline {
        for (index, (source, _)) in sources.iter_mut().enumerate() {
            let window_key = wabou_shell::initial_window_resource_key(index);
            let (width, height) = controller
                .headless_viewport(window_key)
                .unwrap_or((viewport.width, viewport.height));
            source.set_semantics_enabled(true);
            source.handle_event(wabou_shell::UiEvent::WindowMetrics(crate::WindowMetrics {
                window_key,
                logical_width: width,
                logical_height: height,
                physical_width: viewport.physical_width_for(width),
                physical_height: viewport.physical_height_for(height),
                scale_factor: viewport.scale_factor,
                maximized: false,
                focused: true,
                color_scheme: Some(wabou_shell::ColorScheme::Light),
            }));
            last_nodes[index] = profilers[index].build(
                source.as_mut(),
                &mut text,
                width,
                height,
                viewport.scale_factor,
                base_color,
            );
            controller.poll_headless_source(window_key, source.as_mut());
            drain_headless_effects(source.as_mut());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    // A test can finish while its final JS mutation still needs projection.
    // Capture only after two additional host frames have settled that work.
    for _ in 0..2 {
        for (index, (source, _)) in sources.iter_mut().enumerate() {
            let window_key = wabou_shell::initial_window_resource_key(index);
            let (width, height) = controller
                .headless_viewport(window_key)
                .unwrap_or((viewport.width, viewport.height));
            last_nodes[index] = source.build_frame(&mut text, width, height);
        }
    }
    let capture_window = wabou_shell::initial_window_resource_key(viewport.window_index);
    let capture_viewport = controller
        .headless_viewport(capture_window)
        .map(|(width, height)| viewport.with_logical_size(width, height))
        .unwrap_or(viewport);
    // Every source publishes into the shared DevTools state. Build the selected
    // window last so its tree and the PNG below describe the same final frame.
    if let Some((source, _)) = sources.get_mut(viewport.window_index) {
        last_nodes[viewport.window_index] =
            source.build_frame(&mut text, capture_viewport.width, capture_viewport.height);
    }
    let source_count = sources.len();
    let capture_source = sources
        .get_mut(viewport.window_index)
        .map(|(source, _)| source.as_mut())
        .ok_or_else(|| crate::Error::TestScenario {
            message: format!(
                "capture requested window {} but the application has {} window(s)",
                viewport.window_index + 1,
                source_count
            ),
        })?;
    if controller.report_passed() == Some(false) {
        render_headless_failure(
            capture_source,
            &last_nodes,
            &mut text,
            base_color,
            capture_viewport,
        )?;
    }
    if let Some(output) = std::env::var_os("WABOU_TEST_CAPTURE_PATH") {
        render_headless_capture(
            capture_source,
            &last_nodes,
            &mut text,
            base_color,
            capture_viewport,
            Path::new(&output),
        )?;
    }
    #[cfg(feature = "devtools")]
    if let (Some(output), Some(state)) = (std::env::var_os("WABOU_TEST_SNAPSHOT_PATH"), debug_state)
    {
        write_headless_snapshot(state, Path::new(&output))?;
    }
    finish_test_report(controller.clone())
}

#[cfg(feature = "devtools")]
fn write_headless_snapshot(
    state: &wabou_devtools::SharedDebugState,
    output: &Path,
) -> crate::Result<()> {
    let failure = |message: String| crate::error::Error::HeadlessSnapshot {
        path: output.to_owned(),
        message,
    };
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| failure(error.to_string()))?;
    }
    let snapshot = state
        .read()
        .map_err(|_| failure("DevTools snapshot lock was poisoned".to_owned()))?
        .snapshot()
        .clone();
    let bytes = serde_json::to_vec_pretty(&snapshot).map_err(|error| failure(error.to_string()))?;
    std::fs::write(output, bytes).map_err(|error| failure(error.to_string()))?;
    Ok(())
}

#[derive(Clone, Copy)]
struct HeadlessViewport {
    width: u32,
    height: u32,
    scale_factor: f64,
    window_index: usize,
}

impl HeadlessViewport {
    fn with_logical_size(self, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            ..self
        }
    }

    fn from_environment() -> crate::Result<Self> {
        fn parse<T>(name: &'static str, default: T) -> crate::Result<T>
        where
            T: std::str::FromStr,
            T::Err: std::fmt::Display,
        {
            let Some(value) = std::env::var_os(name) else {
                return Ok(default);
            };
            value
                .to_string_lossy()
                .parse()
                .map_err(|error| crate::Error::TestScenario {
                    message: format!("invalid {name}: {error}"),
                })
        }

        let width = parse("WABOU_TEST_VIEWPORT_WIDTH", 1100_u32)?;
        let height = parse("WABOU_TEST_VIEWPORT_HEIGHT", 720_u32)?;
        let scale_factor = parse("WABOU_TEST_SCALE_FACTOR", 1.0_f64)?;
        let window_id = parse("WABOU_TEST_CAPTURE_WINDOW_ID", 1_u32)?;
        if width == 0 || height == 0 {
            return Err(crate::Error::TestScenario {
                message: "headless viewport dimensions must be greater than zero".into(),
            });
        }
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return Err(crate::Error::TestScenario {
                message: "headless scale factor must be finite and greater than zero".into(),
            });
        }
        let window_index = window_id
            .checked_sub(1)
            .ok_or_else(|| crate::Error::TestScenario {
                message: "headless capture window id must be greater than zero".into(),
            })? as usize;
        Ok(Self {
            width,
            height,
            scale_factor,
            window_index,
        })
    }

    fn physical_width(self) -> u32 {
        self.physical_width_for(self.width)
    }

    fn physical_width_for(self, width: u32) -> u32 {
        (f64::from(width) * self.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32
    }

    fn physical_height(self) -> u32 {
        self.physical_height_for(self.height)
    }

    fn physical_height_for(self, height: u32) -> u32 {
        (f64::from(height) * self.scale_factor)
            .round()
            .clamp(1.0, f64::from(u32::MAX)) as u32
    }
}

fn drain_headless_effects(source: &mut dyn crate::FrameSource) {
    while let Some(request) = source.take_effect() {
        source.complete_effect(wabou_shell::EffectCompletion {
            id: request.id,
            op: request.payload.op(),
            result: wabou_shell::EffectResult::Error {
                code: wabou_shell::EffectErrorCode::Unsupported,
                message: format!(
                    "native effect {:?} has no deterministic test fixture",
                    request.payload.op()
                ),
            },
        });
    }
}

fn render_headless_failure(
    source: &mut dyn crate::FrameSource,
    last_nodes: &[Vec<wabou_shell::layout::PlacedNode>],
    text: &mut crate::TextContext,
    base_color: Color,
    viewport: HeadlessViewport,
) -> crate::Result<()> {
    if !std::env::var("WABOU_TEST_FAILURE_SCREENSHOT").is_ok_and(|value| value != "0") {
        return Ok(());
    }
    let Some(directory) = std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from) else {
        return Ok(());
    };
    std::fs::create_dir_all(&directory).map_err(|error| crate::Error::TestScenario {
        message: format!("cannot create failure artifact directory: {error}"),
    })?;
    render_headless_capture(
        source,
        last_nodes,
        text,
        base_color,
        viewport,
        &directory.join("failure.png"),
    )
}

fn render_headless_capture(
    source: &mut dyn crate::FrameSource,
    last_nodes: &[Vec<wabou_shell::layout::PlacedNode>],
    text: &mut crate::TextContext,
    base_color: Color,
    viewport: HeadlessViewport,
    output: &Path,
) -> crate::Result<()> {
    let Some(nodes) = last_nodes.get(viewport.window_index) else {
        return Err(crate::Error::TestScenario {
            message: format!(
                "capture requested window {} but the application has {} window(s)",
                viewport.window_index + 1,
                last_nodes.len()
            ),
        });
    };
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| crate::Error::TestScenario {
            message: format!(
                "cannot create capture directory {}: {error}",
                parent.display()
            ),
        })?;
    }
    let mut scene = anyrender::Scene::new();
    wabou_shell::scene::build_scene_scaled(
        &mut scene,
        nodes,
        text,
        viewport.width,
        viewport.height,
        base_color,
        viewport.scale_factor,
    );
    source.paint_debug_overlay(&mut scene, nodes, text, viewport.scale_factor);
    wabou_shell::renderer::render_to_png(
        &scene,
        viewport.physical_width(),
        viewport.physical_height(),
        base_color,
        output.to_string_lossy().as_ref(),
    )
    .map_err(|error| crate::Error::TestScenario {
        message: format!("cannot render headless screenshot: {error:?}"),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        HostBuilder, HostService, HostServiceContext, HostServiceHandle, HostServicesGuard,
        JsonCapabilityContract, install_host_message_producers, managed_host_service,
        start_host_services,
    };
    use crate::host_message::{HostMessagePayload, HostTaskTracker, host_message_channel};
    use crate::json_capability::{JsonCapability, invoke_json_method};
    use crate::{Applier, HostMessageContext, JsRuntime};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use wabou_bindgen::JsonMethod;

    fn service_context() -> HostServiceContext {
        HostServiceContext {
            app_directories: None,
            behavior_test: false,
            headless: false,
        }
    }

    #[derive(serde::Deserialize)]
    struct JsonRequest {
        value: u32,
    }

    #[derive(serde::Serialize)]
    struct JsonResponse {
        doubled: u32,
    }

    struct TestService {
        name: &'static str,
        shutdowns: Arc<Mutex<Vec<&'static str>>>,
        fail_start: bool,
        fail: bool,
    }

    impl HostService for TestService {
        fn name(&self) -> &'static str {
            self.name
        }

        fn start(&self, _context: &HostServiceContext) -> Result<(), String> {
            if self.fail_start {
                Err("expected start failure".to_owned())
            } else {
                Ok(())
            }
        }

        fn shutdown(&self) -> Result<(), String> {
            self.shutdowns.lock().unwrap().push(self.name);
            if self.fail {
                Err("expected test failure".to_owned())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn host_services_shutdown_in_reverse_order_and_continue_after_errors() {
        let shutdowns = Arc::new(Mutex::new(Vec::new()));
        let first: Arc<dyn HostService> = Arc::new(TestService {
            name: "first",
            shutdowns: shutdowns.clone(),
            fail_start: false,
            fail: false,
        });
        let second: Arc<dyn HostService> = Arc::new(TestService {
            name: "second",
            shutdowns: shutdowns.clone(),
            fail_start: false,
            fail: true,
        });
        let error = HostServicesGuard(vec![first, second])
            .finish()
            .unwrap_err()
            .to_string();
        assert_eq!(*shutdowns.lock().unwrap(), ["second", "first"]);
        assert!(error.contains("`second`: expected test failure"));
    }

    #[test]
    fn later_service_start_failure_shuts_down_already_started_services() {
        let shutdowns = Arc::new(Mutex::new(Vec::new()));
        let first: Arc<dyn HostService> = Arc::new(TestService {
            name: "first",
            shutdowns: shutdowns.clone(),
            fail_start: false,
            fail: false,
        });
        let second: Arc<dyn HostService> = Arc::new(TestService {
            name: "second",
            shutdowns: shutdowns.clone(),
            fail_start: true,
            fail: false,
        });

        let error = start_host_services(&[(first, true), (second, true)], &service_context())
            .err()
            .unwrap();
        assert!(error.to_string().contains("second"));
        assert_eq!(*shutdowns.lock().unwrap(), ["first"]);
    }

    #[test]
    fn recoverable_service_start_failure_does_not_abort_later_services() {
        let shutdowns = Arc::new(Mutex::new(Vec::new()));
        let failed: Arc<dyn HostService> = Arc::new(TestService {
            name: "optional",
            shutdowns: shutdowns.clone(),
            fail_start: true,
            fail: false,
        });
        let healthy: Arc<dyn HostService> = Arc::new(TestService {
            name: "healthy",
            shutdowns: shutdowns.clone(),
            fail_start: false,
            fail: false,
        });

        start_host_services(&[(failed, false), (healthy, true)], &service_context())
            .unwrap()
            .finish()
            .unwrap();
        assert_eq!(*shutdowns.lock().unwrap(), ["healthy"]);
    }

    #[test]
    fn managed_service_handle_is_available_only_while_service_runs() {
        let shutdowns = Arc::new(Mutex::new(Vec::new()));
        let shutdown_log = shutdowns.clone();
        let (handle, service) = managed_host_service(
            "database",
            |_| Ok::<_, String>(String::from("ready")),
            move |value| {
                shutdown_log.lock().unwrap().push(value);
                Ok(())
            },
        );

        assert_eq!(handle.get().unwrap_err(), "database service is not running");
        service.start(&service_context()).unwrap();
        assert_eq!(handle.get().unwrap(), "ready");
        assert!(
            service
                .start(&service_context())
                .unwrap_err()
                .contains("already running")
        );
        service.shutdown().unwrap();
        assert_eq!(handle.get().unwrap_err(), "database service is not running");
        assert_eq!(*shutdowns.lock().unwrap(), ["ready"]);
        service.shutdown().unwrap();
        assert_eq!(*shutdowns.lock().unwrap(), ["ready"]);
    }

    #[test]
    fn managed_service_start_runs_without_holding_the_state_lock() {
        let observed = Arc::new(Mutex::new(None::<HostServiceHandle<String>>));
        let observed_during_start = observed.clone();
        let (handle, service) = managed_host_service(
            "database",
            move |_| {
                let error = observed_during_start
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .get()
                    .unwrap_err();
                assert_eq!(error, "database service is starting");
                Ok::<_, String>(String::from("ready"))
            },
            |_| Ok(()),
        );
        *observed.lock().unwrap() = Some(handle.clone());

        service.start(&service_context()).unwrap();
        assert_eq!(handle.get().unwrap(), "ready");
        service.shutdown().unwrap();
    }

    #[test]
    fn managed_service_can_retry_after_start_failure() {
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let start_attempts = attempts.clone();
        let (handle, service) = managed_host_service(
            "database",
            move |_| {
                if start_attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
                    Err("database unavailable".to_owned())
                } else {
                    Ok(String::from("ready"))
                }
            },
            |_| Ok(()),
        );

        assert_eq!(
            service.start(&service_context()).unwrap_err(),
            "database unavailable"
        );
        assert_eq!(
            handle.get().unwrap_err(),
            "database service failed to start: database unavailable"
        );
        service.retry().unwrap();
        assert_eq!(handle.get().unwrap(), "ready");
        service.shutdown().unwrap();
    }

    #[test]
    fn managed_service_turns_initializer_panics_into_retryable_failures() {
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let start_attempts = attempts.clone();
        let (handle, service) = managed_host_service(
            "database",
            move |_| {
                if start_attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
                    panic!("broken database initializer");
                }
                Ok(String::from("ready"))
            },
            |_| Ok(()),
        );

        assert_eq!(
            service.start(&service_context()).unwrap_err(),
            "database service initializer panicked: broken database initializer"
        );
        assert_eq!(
            handle.get().unwrap_err(),
            "database service failed to start: database service initializer panicked: broken database initializer"
        );
        service.retry().unwrap();
        assert_eq!(handle.get().unwrap(), "ready");
        service.shutdown().unwrap();
    }

    #[tokio::test]
    async fn managed_service_async_retry_runs_initializer_off_executor_thread() {
        let first_thread = std::thread::current().id();
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let retry_thread = Arc::new(Mutex::new(None));
        let start_attempts = attempts.clone();
        let observed_thread = retry_thread.clone();
        let (_handle, service) = managed_host_service(
            "database",
            move |_| {
                if start_attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
                    return Err("database unavailable".to_owned());
                }
                *observed_thread.lock().unwrap() = Some(std::thread::current().id());
                Ok(String::from("ready"))
            },
            |_| Ok(()),
        );

        service.start(&service_context()).unwrap_err();
        service.retry_async().await.unwrap();
        assert_ne!(*retry_thread.lock().unwrap(), Some(first_thread));
        service.shutdown().unwrap();
    }

    #[test]
    fn managed_service_shutdown_waits_for_concurrent_retry_and_closes_the_result() {
        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let start_attempts = attempts.clone();
        let shutdowns = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let shutdown_count = shutdowns.clone();
        let (entered_tx, entered_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let release_rx = Arc::new(Mutex::new(release_rx));
        let start_release = release_rx.clone();
        let (handle, service) = managed_host_service(
            "database",
            move |_| {
                if start_attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed) == 0 {
                    return Err("database unavailable".to_owned());
                }
                entered_tx.send(()).unwrap();
                start_release.lock().unwrap().recv().unwrap();
                Ok(String::from("ready"))
            },
            move |_| {
                shutdown_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(())
            },
        );

        service.start(&service_context()).unwrap_err();
        let retry_service = service.clone();
        let retry = std::thread::spawn(move || retry_service.retry());
        entered_rx.recv().unwrap();

        let shutdown_service = service.clone();
        let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            shutdown_tx.send(shutdown_service.shutdown()).unwrap();
        });
        assert!(
            shutdown_rx.recv_timeout(Duration::from_millis(20)).is_err(),
            "shutdown returned before the service initializer settled"
        );

        release_tx.send(()).unwrap();
        retry.join().unwrap().unwrap();
        shutdown_rx.recv().unwrap().unwrap();
        assert_eq!(shutdowns.load(std::sync::atomic::Ordering::Relaxed), 1);
        assert_eq!(handle.get().unwrap_err(), "database service is not running");
    }

    #[test]
    fn managed_service_async_retry_without_tokio_returns_error() {
        let (_handle, service) = managed_host_service(
            "database",
            |_| Err::<String, _>("database unavailable".to_owned()),
            |_| Ok(()),
        );

        service.start(&service_context()).unwrap_err();
        let error = futures_lite::future::block_on(service.retry_async()).unwrap_err();
        assert_eq!(
            error,
            "database service async retry requires a Tokio runtime"
        );
    }

    #[test]
    fn json_capability_encodes_successful_results() {
        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":21}"#,
            |request: JsonRequest| async move {
                Ok::<_, String>(JsonResponse {
                    doubled: request.value * 2,
                })
            },
        ));

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&response).unwrap(),
            serde_json::json!({ "ok": true, "value": { "doubled": 42 } })
        );
    }

    #[test]
    fn json_capability_encodes_handler_errors() {
        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":7}"#,
            |_request: JsonRequest| async move {
                Err::<JsonResponse, _>("palette unavailable".to_owned())
            },
        ));

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&response).unwrap(),
            serde_json::json!({
                "ok": false,
                "error": {
                    "code": "handlerFailure",
                    "message": "palette unavailable",
                },
            })
        );
    }

    #[test]
    fn json_capability_rejects_malformed_requests_before_calling_the_handler() {
        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":"wrong"}"#,
            |_request: JsonRequest| async move {
                panic!("malformed requests must not reach the handler");
                #[allow(unreachable_code)]
                Ok::<_, String>(JsonResponse { doubled: 0 })
            },
        ));
        let response = serde_json::from_str::<serde_json::Value>(&response).unwrap();

        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "invalidRequest");
        assert!(
            response["error"]["message"]
                .as_str()
                .unwrap()
                .starts_with("invalid capability request:")
        );
    }

    #[test]
    fn json_capability_classifies_response_encoding_failures() {
        struct InvalidResponse;

        impl serde::Serialize for InvalidResponse {
            fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                Err(serde::ser::Error::custom("fixture cannot be encoded"))
            }
        }

        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":7}"#,
            |_request: JsonRequest| async move { Ok::<_, String>(InvalidResponse) },
        ));
        let response = serde_json::from_str::<serde_json::Value>(&response).unwrap();

        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "responseEncodingFailure");
        assert_eq!(
            response["error"]["message"],
            "cannot encode capability response: fixture cannot be encoded"
        );
    }

    #[test]
    fn json_capability_rejects_ignored_request_fields() {
        #[derive(serde::Deserialize)]
        struct NestedRequest {
            value: u32,
            nested: NestedValue,
        }
        #[derive(serde::Deserialize)]
        struct NestedValue {
            known: bool,
        }

        let called = Arc::new(Mutex::new(false));
        let handler_called = Arc::clone(&called);
        let response = futures_lite::future::block_on(invoke_json_method(
            r#"{"value":7,"nested":{"known":true,"typo":true}}"#,
            move |request: NestedRequest| {
                *handler_called.lock().unwrap() = true;
                async move {
                    Ok::<_, String>(JsonResponse {
                        doubled: request.value * u32::from(request.nested.known),
                    })
                }
            },
        ));
        let response = serde_json::from_str::<serde_json::Value>(&response).unwrap();

        assert_eq!(response["ok"], false);
        assert_eq!(response["error"]["code"], "invalidRequest");
        assert_eq!(
            response["error"]["message"],
            "unknown capability request field: nested.typo"
        );
        assert!(!*called.lock().unwrap());
    }

    #[test]
    fn json_capability_mounts_structured_and_empty_requests_in_quickjs() {
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("typed", |ctx, object| {
                let capability = JsonCapability { ctx, object };
                capability.method(
                    JsonMethod::new("double"),
                    |request: JsonRequest| async move {
                        Ok::<_, String>(JsonResponse {
                            doubled: request.value * 2,
                        })
                    },
                )?;
                capability.method(JsonMethod::no_request("ping"), |(): ()| async move {
                    Ok::<_, String>("pong")
                })
            })
            .expect("mount JSON capability");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"
                    globalThis.capabilityResult = undefined;
                    Promise.all([
                      __wabou_capabilities.typed.double(JSON.stringify({ value: 9 })),
                      __wabou_capabilities.typed.ping(),
                    ]).then(values => globalThis.capabilityResult = JSON.stringify(values.map(JSON.parse)));
                    "#,
                )
            })
            .expect("invoke JSON capability");
        // JSON handlers in this fixture are immediately ready, but resolving
        // both async functions and the surrounding Promise.all can cross the
        // runtime's intentional 1 ms/32-job scheduling slice under load.
        while runtime.poll_async_runtime() {}
        let result = runtime
            .with(|ctx| ctx.eval::<Option<String>, _>("globalThis.capabilityResult"))
            .expect("read JSON capability result")
            .expect("capability promise settled");

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&result).unwrap(),
            serde_json::json!([
                { "ok": true, "value": { "doubled": 18 } },
                { "ok": true, "value": "pong" },
            ])
        );
    }

    #[test]
    fn json_capability_publishes_the_shared_abi_version() {
        const CONTRACT: JsonCapabilityContract = JsonCapabilityContract::new("versioned", 7);
        let host = HostBuilder::new().json_capability(CONTRACT, |_capability| Ok(()));
        let runtime = JsRuntime::new().expect("runtime");

        for install in host.capabilities {
            install(&runtime).expect("install capability");
        }

        let version = runtime
            .with(|ctx| {
                ctx.eval::<u32, _>(
                    "globalThis.__wabou_capabilities.versioned.__wabouCapabilityVersion",
                )
            })
            .expect("read capability version");
        assert_eq!(version, CONTRACT.version());
    }

    #[test]
    fn json_capability_rejects_duplicate_method_registration() {
        let runtime = JsRuntime::new().expect("runtime");
        let result = runtime.mount_capability("typed", |ctx, object| {
            let capability = JsonCapability { ctx, object };
            capability.method(JsonMethod::no_request("ping"), |(): ()| async move {
                Ok::<_, String>("first")
            })?;
            capability.method(JsonMethod::no_request("ping"), |(): ()| async move {
                Ok::<_, String>("second")
            })
        });

        assert!(result.is_err());
    }

    #[test]
    fn json_capability_uses_the_shared_method_identifier_rules() {
        let runtime = JsRuntime::new().expect("runtime");
        let result = runtime.mount_capability("typed", |ctx, object| {
            JsonCapability { ctx, object }
                .method(JsonMethod::no_request("1ping"), |(): ()| async move {
                    Ok::<_, String>("pong")
                })
        });

        assert!(result.is_err());
    }

    #[cfg(feature = "profiling")]
    #[test]
    fn profiling_writes_only_opted_in_perf_spans_without_source_locations() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("trace.json");
        let file = std::fs::File::create(&path).unwrap();
        let guard = super::init_profile_tracing(file, tracing_subscriber::EnvFilter::new("info"))
            .expect("test owns the process-global tracing subscriber");
        {
            let span = tracing::trace_span!(
                target: "wabou::perf",
                "quick.test",
                nodes = 3_u64,
            );
            let _guard = span.enter();
            tracing::trace!(target: "unrelated", secret = "must-not-appear", "private");
        }
        drop(guard);
        let trace = std::fs::read_to_string(path).unwrap();
        assert!(trace.contains("quick.test"));
        assert!(trace.contains("nodes"));
        assert!(!trace.contains("must-not-appear"));
        assert!(!trace.contains("host.rs"));
    }

    #[test]
    fn application_message_producers_receive_each_window_handle() {
        let observed = Arc::new(Mutex::new(Vec::new()));
        let producer_observed = observed.clone();
        let builder = HostBuilder::new().host_message_producer(move |context| {
            producer_observed.lock().unwrap().push(context.window_key());
            context
                .messages()
                .emit_i32("ready", context.window_key().lo() as i32)
                .unwrap();
        });
        let (handle, inbox) = host_message_channel(4);
        let cancellation = tokio_util::sync::CancellationToken::new();
        let runtime = tokio::runtime::Runtime::new().unwrap();

        for producer in &builder.host_message_producers {
            producer(crate::HostMessageContext::new(
                wabou_shell::WindowResourceKey::from_parts(7, 1).unwrap(),
                handle.clone(),
                cancellation.clone(),
                runtime.handle().clone(),
                Arc::new(HostTaskTracker::default()),
            ));
        }

        assert_eq!(observed.lock().unwrap()[0].into_parts(), (7, 1));
        let messages = inbox.drain_batch();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].topic, "ready");
        assert_eq!(messages[0].payload, HostMessagePayload::I32(7));
    }

    #[test]
    fn producer_context_is_cancelled_when_window_source_drops() {
        let observed = Arc::new(Mutex::new(None::<HostMessageContext>));
        let producer_observed = observed.clone();
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (stopped_tx, stopped_rx) = std::sync::mpsc::channel();
        let builder = HostBuilder::new().host_message_producer(move |context| {
            *producer_observed.lock().unwrap() = Some(context.clone());
            let task_context = context.clone();
            let started_tx = started_tx.clone();
            let stopped_tx = stopped_tx.clone();
            context.spawn(async move {
                started_tx.send(()).unwrap();
                task_context.cancelled().await;
                stopped_tx.send(()).unwrap();
            });
        });
        let runtime = JsRuntime::new().unwrap();
        let applier =
            Applier::from_runtime(runtime, vello::peniko::Color::from_rgb8(0x00, 0x00, 0x00));
        install_host_message_producers(
            &builder.host_message_producers,
            wabou_shell::WindowResourceKey::from_parts(3, 1).unwrap(),
            &applier,
        );
        started_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        let context = observed.lock().unwrap().clone().unwrap();
        assert_eq!(context.window_key().into_parts(), (3, 1));
        assert!(!context.is_cancelled());

        drop(applier);

        stopped_rx
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        assert!(context.is_cancelled());
    }
}
