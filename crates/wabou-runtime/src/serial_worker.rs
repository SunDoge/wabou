//! Typed request/reply workers for thread-affine native state.

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
};

struct RequestEnvelope<Request, Response> {
    request: Request,
    reply: tokio::sync::oneshot::Sender<Result<Response, String>>,
}

struct WorkerInner<Request, Response> {
    name: Arc<str>,
    sender: Mutex<Option<mpsc::Sender<RequestEnvelope<Request, Response>>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl<Request, Response> WorkerInner<Request, Response> {
    fn shutdown(&self) -> Result<(), String> {
        self.sender
            .lock()
            .map_err(|_| format!("{} worker sender is poisoned", self.name))?
            .take();
        let thread = self
            .thread
            .lock()
            .map_err(|_| format!("{} worker thread is poisoned", self.name))?
            .take();
        if let Some(thread) = thread {
            thread
                .join()
                .map_err(|_| format!("{} worker thread panicked", self.name))?;
        }
        Ok(())
    }
}

impl<Request, Response> Drop for WorkerInner<Request, Response> {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Cloneable request handle for a serial thread that exclusively owns native state.
///
/// State is initialized on the worker thread, which makes this suitable for
/// inference engines and other thread-affine resources. Requests are processed
/// in FIFO order. Dropping a request future does not cancel work already queued;
/// only delivery of its result is abandoned.
pub struct SerialWorker<Request, Response> {
    inner: Arc<WorkerInner<Request, Response>>,
}

impl<Request, Response> Clone for SerialWorker<Request, Response> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
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
        mut handle: Handle,
    ) -> Result<Self, String>
    where
        State: 'static,
        Initialize: FnOnce() -> Result<State, String> + Send + 'static,
        Handle: FnMut(&mut State, Request) -> Result<Response, String> + Send + 'static,
    {
        let name = Arc::<str>::from(name.into());
        let thread_name = name.to_string();
        let diagnostic_name = name.clone();
        let (sender, receiver) = mpsc::channel::<RequestEnvelope<Request, Response>>();
        let thread = std::thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                let initialized = catch_unwind(AssertUnwindSafe(initialize))
                    .map_err(|_| format!("{diagnostic_name} worker initializer panicked"))
                    .and_then(|result| result);
                let mut state = match initialized {
                    Ok(state) => state,
                    Err(error) => {
                        while let Ok(envelope) = receiver.recv() {
                            let _ = envelope.reply.send(Err(error.clone()));
                        }
                        return;
                    }
                };
                while let Ok(envelope) = receiver.recv() {
                    let result =
                        catch_unwind(AssertUnwindSafe(|| handle(&mut state, envelope.request)))
                            .unwrap_or_else(|_| {
                                Err(format!("{diagnostic_name} worker request panicked"))
                            });
                    let _ = envelope.reply.send(result);
                }
            })
            .map_err(|error| format!("failed to start {name} worker: {error}"))?;
        Ok(Self {
            inner: Arc::new(WorkerInner {
                name,
                sender: Mutex::new(Some(sender)),
                thread: Mutex::new(Some(thread)),
            }),
        })
    }

    /// Queue one request and asynchronously wait for its typed result.
    pub async fn request(&self, request: Request) -> Result<Response, String> {
        let sender = self
            .inner
            .sender
            .lock()
            .map_err(|_| format!("{} worker sender is poisoned", self.inner.name))?
            .clone()
            .ok_or_else(|| format!("{} worker has stopped", self.inner.name))?;
        let (reply, result) = tokio::sync::oneshot::channel();
        sender
            .send(RequestEnvelope { request, reply })
            .map_err(|_| format!("{} worker has stopped", self.inner.name))?;
        result
            .await
            .map_err(|_| format!("{} worker stopped before replying", self.inner.name))?
    }

    /// Stop accepting requests, drain queued work, and join the worker thread.
    pub fn shutdown(&self) -> Result<(), String> {
        self.inner.shutdown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initializes_and_serializes_state_on_the_named_thread() {
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
    fn reports_initializer_and_request_panics_as_errors() {
        let failed = SerialWorker::<(), ()>::spawn(
            "failed-worker",
            || -> Result<(), String> { panic!("init") },
            |_, _| Ok(()),
        )
        .unwrap();
        let panics = SerialWorker::spawn(
            "panic-worker",
            || Ok::<_, String>(()),
            |_, _: ()| -> Result<(), String> { panic!("request") },
        )
        .unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert!(runtime.block_on(failed.request(())).is_err());
        assert!(runtime.block_on(panics.request(())).is_err());
    }
}
