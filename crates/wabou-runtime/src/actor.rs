//! Small actor primitives for thread-owned state.

use std::any::type_name;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Instant;

use snafu::{OptionExt, Snafu, ensure};

type ReplySender<Reply> = tokio::sync::oneshot::Sender<Result<Reply, ActorError>>;
static NEXT_ACTOR_ID: AtomicU64 = AtomicU64::new(1);

enum Envelope<Message, Reply> {
    Tell {
        id: u64,
        name: &'static str,
        queued_at: Instant,
        message: Message,
    },
    Call {
        id: u64,
        name: &'static str,
        queued_at: Instant,
        message: Message,
        reply: ReplySender<Reply>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Snafu)]
pub(crate) enum ActorError {
    #[snafu(display("{actor} actor mailbox is full"))]
    Full { actor: Arc<str> },
    #[snafu(display("{actor} actor has stopped"))]
    Stopped { actor: Arc<str> },
    #[snafu(display("{actor} actor stopped before replying"))]
    ReplyDropped { actor: Arc<str> },
    #[snafu(display("{actor} actor {resource} is poisoned"))]
    Poisoned {
        actor: Arc<str>,
        resource: &'static str,
    },
    #[snafu(display("{actor} actor {phase} failed: {message}"))]
    Failed {
        actor: Arc<str>,
        phase: &'static str,
        message: String,
    },
}

struct ActorInner<Message, Reply> {
    id: u64,
    name: Arc<str>,
    next_id: AtomicU64,
    sender: Mutex<Option<flume::Sender<Envelope<Message, Reply>>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl<Message, Reply> ActorInner<Message, Reply> {
    fn shutdown(&self) -> Result<(), ActorError> {
        self.sender
            .lock()
            .map_err(|_| {
                PoisonedSnafu {
                    actor: self.name.clone(),
                    resource: "sender",
                }
                .build()
            })?
            .take();
        if let Some(thread) = self
            .thread
            .lock()
            .map_err(|_| {
                PoisonedSnafu {
                    actor: self.name.clone(),
                    resource: "thread",
                }
                .build()
            })?
            .take()
        {
            thread.join().map_err(|payload| {
                FailedSnafu {
                    actor: self.name.clone(),
                    phase: "thread",
                    message: panic_message(payload.as_ref()).to_owned(),
                }
                .build()
            })?;
        }
        Ok(())
    }
}

impl<Message, Reply> Drop for ActorInner<Message, Reply> {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

/// Cloneable reference to state exclusively owned by one named thread.
pub(crate) struct ThreadActor<Message, Reply> {
    inner: Arc<ActorInner<Message, Reply>>,
}

impl<Message, Reply> Clone for ThreadActor<Message, Reply> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<Message, Reply> ThreadActor<Message, Reply>
where
    Message: Send + 'static,
    Reply: Send + 'static,
{
    pub(crate) fn spawn<State, Initialize, Handle>(
        name: impl Into<String>,
        capacity: usize,
        initialize: Initialize,
        mut handle: Handle,
    ) -> Result<Self, ActorError>
    where
        State: 'static,
        Initialize: FnOnce() -> Result<State, String> + Send + 'static,
        Handle: FnMut(&mut State, Message) -> Result<Reply, String> + Send + 'static,
    {
        let name = Arc::<str>::from(name.into());
        let actor_id = NEXT_ACTOR_ID.fetch_add(1, Ordering::Relaxed);
        ensure!(
            capacity > 0,
            FailedSnafu {
                actor: name,
                phase: "spawn",
                message: "mailbox capacity must be greater than zero".to_owned(),
            }
        );
        let actor_name = name.clone();
        let (sender, receiver) = flume::bounded(capacity);
        let thread = std::thread::Builder::new()
            .name(name.to_string())
            .spawn(move || {
                let initialized = {
                    let span = tracing::info_span!(
                        "actor.initialize",
                        actor = %actor_name,
                        actor_id,
                    );
                    let _entered = span.enter();
                    catch_unwind(AssertUnwindSafe(initialize))
                        .map_err(|payload| panic_message(payload.as_ref()).to_owned())
                        .and_then(|result| result)
                };
                let mut state = match initialized {
                    Ok(state) => state,
                    Err(message) => {
                        let error = FailedSnafu {
                            actor: actor_name.clone(),
                            phase: "initialization",
                            message,
                        }
                        .build();
                        while let Ok(envelope) = receiver.recv() {
                            if let Envelope::Call { reply, .. } = envelope {
                                let _ = reply.send(Err(error.clone()));
                            }
                        }
                        return;
                    }
                };
                while let Ok(envelope) = receiver.recv() {
                    let (id, message_name, queued_at, message, reply) = match envelope {
                        Envelope::Tell {
                            id,
                            name,
                            queued_at,
                            message,
                        } => (id, name, queued_at, message, None),
                        Envelope::Call {
                            id,
                            name,
                            queued_at,
                            message,
                            reply,
                        } => (id, name, queued_at, message, Some(reply)),
                    };
                    let span = tracing::info_span!(
                        "actor.message",
                        actor = %actor_name,
                        actor_id,
                        message = message_name,
                        message_id = id,
                        queue_us = queued_at.elapsed().as_micros() as u64,
                        handle_us = tracing::field::Empty,
                    );
                    let _entered = span.enter();
                    let started = Instant::now();
                    let result = catch_unwind(AssertUnwindSafe(|| handle(&mut state, message)))
                        .map_err(|payload| panic_message(payload.as_ref()).to_owned())
                        .and_then(|result| result)
                        .map_err(|message| {
                            FailedSnafu {
                                actor: actor_name.clone(),
                                phase: "message",
                                message,
                            }
                            .build()
                        });
                    span.record("handle_us", started.elapsed().as_micros() as u64);
                    if let Some(reply) = reply {
                        let _ = reply.send(result);
                    } else if let Err(error) = result {
                        tracing::warn!(actor = %actor_name, %error, "actor tell failed");
                    }
                }
            })
            .map_err(|error| {
                FailedSnafu {
                    actor: name.clone(),
                    phase: "spawn",
                    message: error.to_string(),
                }
                .build()
            })?;
        Ok(Self {
            inner: Arc::new(ActorInner {
                id: actor_id,
                name,
                next_id: AtomicU64::new(1),
                sender: Mutex::new(Some(sender)),
                thread: Mutex::new(Some(thread)),
            }),
        })
    }

    fn sender(&self) -> Result<flume::Sender<Envelope<Message, Reply>>, ActorError> {
        self.inner
            .sender
            .lock()
            .map_err(|_| {
                PoisonedSnafu {
                    actor: self.inner.name.clone(),
                    resource: "sender",
                }
                .build()
            })?
            .clone()
            .context(StoppedSnafu {
                actor: self.inner.name.clone(),
            })
    }

    pub(crate) fn id(&self) -> u64 {
        self.inner.id
    }

    pub(crate) fn tell(&self, message: Message) -> Result<(), ActorError> {
        self.tell_named(type_name::<Message>(), message)
    }

    pub(crate) fn tell_named(
        &self,
        name: &'static str,
        message: Message,
    ) -> Result<(), ActorError> {
        let envelope = Envelope::Tell {
            id: self.inner.next_id.fetch_add(1, Ordering::Relaxed),
            name,
            queued_at: Instant::now(),
            message,
        };
        self.sender()?
            .try_send(envelope)
            .map_err(|error| match error {
                flume::TrySendError::Full(_) => FullSnafu {
                    actor: self.inner.name.clone(),
                }
                .build(),
                flume::TrySendError::Disconnected(_) => StoppedSnafu {
                    actor: self.inner.name.clone(),
                }
                .build(),
            })
    }

    pub(crate) async fn call(&self, message: Message) -> Result<Reply, ActorError> {
        self.call_named(type_name::<Message>(), message).await
    }

    pub(crate) async fn call_named(
        &self,
        name: &'static str,
        message: Message,
    ) -> Result<Reply, ActorError> {
        let (reply, result) = tokio::sync::oneshot::channel();
        let envelope = Envelope::Call {
            id: self.inner.next_id.fetch_add(1, Ordering::Relaxed),
            name,
            queued_at: Instant::now(),
            message,
            reply,
        };
        self.sender()?.send_async(envelope).await.map_err(|_| {
            StoppedSnafu {
                actor: self.inner.name.clone(),
            }
            .build()
        })?;
        result.await.map_err(|_| {
            ReplyDroppedSnafu {
                actor: self.inner.name.clone(),
            }
            .build()
        })?
    }

    pub(crate) fn shutdown(&self) -> Result<(), ActorError> {
        self.inner.shutdown()
    }
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> &str {
    payload
        .downcast_ref::<&'static str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("unknown panic payload")
}

#[cfg(test)]
mod tests {
    use super::ThreadActor;

    #[test]
    fn tell_and_call_share_one_exclusive_state() {
        let actor = ThreadActor::spawn(
            "counter",
            8,
            || Ok::<_, String>(0_u32),
            |state, add| {
                *state += add;
                Ok(*state)
            },
        )
        .unwrap();
        assert_ne!(actor.id(), 0);
        actor.tell(2).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        assert_eq!(runtime.block_on(actor.call(3)).unwrap(), 5);
        actor.shutdown().unwrap();
        assert!(actor.tell(1).is_err());
    }

    #[test]
    fn async_call_waits_for_bounded_mailbox_capacity() {
        let (entered_tx, entered_rx) = std::sync::mpsc::sync_channel(1);
        let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
        let actor = ThreadActor::spawn(
            "bounded-counter",
            1,
            move || Ok::<_, String>((0_u32, entered_tx, release_rx)),
            |state, add| {
                if add == 1 {
                    state.1.send(()).unwrap();
                    state.2.recv().unwrap();
                }
                state.0 += add;
                Ok(state.0)
            },
        )
        .unwrap();
        actor.tell(1).unwrap();
        entered_rx.recv().unwrap();
        actor.tell(2).unwrap();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let request = runtime.spawn({
            let actor = actor.clone();
            async move { actor.call(3).await }
        });
        runtime.block_on(tokio::task::yield_now());
        assert!(!request.is_finished());

        release_tx.send(()).unwrap();
        assert_eq!(runtime.block_on(request).unwrap().unwrap(), 6);
        actor.shutdown().unwrap();
    }

    #[test]
    fn owned_payload_moves_without_reallocating_its_buffer() {
        let actor = ThreadActor::spawn(
            "payload",
            2,
            || Ok::<_, String>(()),
            |_, bytes: Vec<u8>| Ok((bytes.as_ptr() as usize, bytes.len())),
        )
        .unwrap();
        let bytes = vec![7_u8; 1024 * 1024];
        let original_pointer = bytes.as_ptr() as usize;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let (received_pointer, length) = runtime.block_on(actor.call(bytes)).unwrap();
        assert_eq!(received_pointer, original_pointer);
        assert_eq!(length, 1024 * 1024);
    }
}
