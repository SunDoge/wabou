//! Typed request/reply workers for thread-affine native state.

use crate::actor::ThreadActor;

const DEFAULT_QUEUE_CAPACITY: usize = 64;

/// Compatibility facade over Wabou's thread actor primitive.
///
/// State is initialized and exclusively owned by the actor thread. Requests
/// use the actor's bounded mailbox and typed call/reply path.
pub struct SerialWorker<Request, Response> {
    actor: ThreadActor<Request, Response>,
}

impl<Request, Response> Clone for SerialWorker<Request, Response> {
    fn clone(&self) -> Self {
        Self {
            actor: self.actor.clone(),
        }
    }
}

impl<Request, Response> SerialWorker<Request, Response>
where
    Request: Send + 'static,
    Response: Send + 'static,
{
    /// Start a named worker and initialize its state inside that thread.
    pub fn spawn<State, Initialize, Handle>(
        name: impl Into<String>,
        initialize: Initialize,
        handle: Handle,
    ) -> Result<Self, String>
    where
        State: 'static,
        Initialize: FnOnce() -> Result<State, String> + Send + 'static,
        Handle: FnMut(&mut State, Request) -> Result<Response, String> + Send + 'static,
    {
        Self::spawn_with_capacity(name, DEFAULT_QUEUE_CAPACITY, initialize, handle)
    }

    /// Start a named worker with an explicit bounded mailbox capacity.
    pub fn spawn_with_capacity<State, Initialize, Handle>(
        name: impl Into<String>,
        capacity: usize,
        initialize: Initialize,
        handle: Handle,
    ) -> Result<Self, String>
    where
        State: 'static,
        Initialize: FnOnce() -> Result<State, String> + Send + 'static,
        Handle: FnMut(&mut State, Request) -> Result<Response, String> + Send + 'static,
    {
        ThreadActor::spawn(name, capacity, initialize, handle)
            .map(|actor| Self { actor })
            .map_err(|error| error.to_string())
    }

    /// Queue one request and asynchronously wait for its typed result.
    pub async fn request(&self, request: Request) -> Result<Response, String> {
        self.actor
            .call(request)
            .await
            .map_err(|error| error.to_string())
    }

    /// Queue a fire-and-forget message without allocating a reply channel.
    pub fn send(&self, request: Request) -> Result<(), String> {
        self.actor.tell(request).map_err(|error| error.to_string())
    }

    /// Return the process-local actor identity used by diagnostics and tracing.
    pub fn actor_id(&self) -> u64 {
        self.actor.id()
    }

    /// Stop accepting requests, drain queued work, and join the actor thread.
    pub fn shutdown(&self) -> Result<(), String> {
        self.actor.shutdown().map_err(|error| error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::SerialWorker;

    #[test]
    fn preserves_serial_worker_api_and_thread_affinity() {
        let worker = SerialWorker::spawn(
            "counter-worker",
            || Ok::<_, String>(0_u32),
            |state, add| {
                assert_eq!(std::thread::current().name(), Some("counter-worker"));
                *state += add;
                Ok(*state)
            },
        )
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert_eq!(runtime.block_on(worker.request(2)).unwrap(), 2);
        assert_eq!(runtime.block_on(worker.request(3)).unwrap(), 5);
        worker.shutdown().unwrap();
        assert!(runtime.block_on(worker.request(1)).is_err());
    }

    #[test]
    fn preserves_initializer_and_handler_errors() {
        let failed = SerialWorker::<(), ()>::spawn(
            "failed-worker",
            || Err::<(), _>("cannot initialize".to_owned()),
            |_, _| Ok(()),
        )
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert!(
            runtime
                .block_on(failed.request(()))
                .unwrap_err()
                .contains("cannot initialize")
        );
    }

    #[test]
    fn rejects_zero_capacity() {
        assert!(
            SerialWorker::<(), ()>::spawn_with_capacity(
                "zero-worker",
                0,
                || Ok::<_, String>(()),
                |_, _| Ok(())
            )
            .is_err()
        );
    }
}
