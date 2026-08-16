//! Host → JS application messages.
//!
//! Background tasks push [`HostMessage`] into a bounded queue without touching
//! QuickJS. The applier drains on the UI thread and encodes each item as an
//! application record in the unified HostEventFrame.

use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use wabou_shell::WakeCallback;

/// Default bound: producers `try_send` and get [`HostMessageError::Full`] when the
/// UI thread is not draining fast enough.
pub const DEFAULT_HOST_MESSAGE_CAPACITY: usize = 1024;

/// Max messages forwarded to JS in a single frame.
pub const MAX_HOST_MESSAGES_PER_FRAME: usize = 128;

/// Max UTF-8 topic length (u16).
pub const MAX_TOPIC_BYTES: usize = 0xffff;

/// Max string payload length (u16).
pub const MAX_STR_PAYLOAD_BYTES: usize = 0xffff;

/// Max binary payload length.
pub const MAX_BYTES_PAYLOAD: usize = 1024 * 1024;

/// Typed payload — no JSON on the hot path.
#[derive(Debug, Clone, PartialEq)]
pub enum HostMessagePayload {
    /// No payload value.
    Null,
    /// Boolean payload.
    Bool(bool),
    /// Signed 32-bit integer payload.
    I32(i32),
    /// IEEE-754 double payload.
    F64(f64),
    /// UTF-8 string payload.
    Str(String),
    /// Opaque binary payload.
    Bytes(Vec<u8>),
}

/// One application-level notification from Rust to JS.
#[derive(Debug, Clone, PartialEq)]
pub struct HostMessage {
    /// Application-defined routing topic.
    pub topic: String,
    /// Typed payload encoded without JSON.
    pub payload: HostMessagePayload,
}

impl HostMessage {
    /// Construct a topic-only message.
    pub fn null(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            payload: HostMessagePayload::Null,
        }
    }

    /// Construct a Boolean message.
    pub fn bool(topic: impl Into<String>, value: bool) -> Self {
        Self {
            topic: topic.into(),
            payload: HostMessagePayload::Bool(value),
        }
    }

    /// Construct a signed 32-bit integer message.
    pub fn i32(topic: impl Into<String>, value: i32) -> Self {
        Self {
            topic: topic.into(),
            payload: HostMessagePayload::I32(value),
        }
    }

    /// Construct a double-precision floating-point message.
    pub fn f64(topic: impl Into<String>, value: f64) -> Self {
        Self {
            topic: topic.into(),
            payload: HostMessagePayload::F64(value),
        }
    }

    /// Construct a UTF-8 string message.
    pub fn str(topic: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            payload: HostMessagePayload::Str(value.into()),
        }
    }

    /// Construct an opaque binary message.
    pub fn bytes(topic: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            topic: topic.into(),
            payload: HostMessagePayload::Bytes(value.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Non-blocking host-message enqueue failure.
pub enum HostMessageError {
    /// Bounded queue has no remaining capacity.
    Full,
    /// Owning JavaScript runtime and receiver were dropped.
    Disconnected,
    /// Topic or string payload exceeds length limits.
    TooLarge,
}

/// Cloneable, thread-safe handle for enqueueing host messages.
#[derive(Clone)]
pub struct HostMessageHandle {
    tx: SyncSender<HostMessage>,
    pending: Arc<AtomicBool>,
    wake: Arc<Mutex<Option<WakeCallback>>>,
}

/// Per-window context for a long-running Rust → JavaScript message producer.
///
/// A producer owns a clone of this context. Wabou cancels it when the native
/// window and its JavaScript runtime are dropped, allowing background tasks to
/// stop even when they have no message ready to send.
#[derive(Clone)]
pub struct HostMessageContext {
    window_id: u64,
    messages: HostMessageHandle,
    cancellation: CancellationToken,
    runtime: tokio::runtime::Handle,
}

impl HostMessageContext {
    pub(crate) fn new(
        window_id: u64,
        messages: HostMessageHandle,
        cancellation: CancellationToken,
        runtime: tokio::runtime::Handle,
    ) -> Self {
        Self {
            window_id,
            messages,
            cancellation,
            runtime,
        }
    }

    /// Stable logical identifier of the owning native window.
    pub fn window_id(&self) -> u64 {
        self.window_id
    }

    /// Borrow the thread-safe producer handle.
    pub fn messages(&self) -> &HostMessageHandle {
        &self.messages
    }

    /// Whether the owning window/runtime has been dropped.
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Wait until the owning window/runtime is dropped.
    pub async fn cancelled(&self) {
        self.cancellation.cancelled().await;
    }

    /// Spawn a `Send` producer on the Tokio runtime owned by this window's
    /// JavaScript host. Calling this is valid even when the setup callback is
    /// entered from Wabou's synchronous application thread.
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }
}

impl HostMessageHandle {
    /// Non-blocking send. Safe from any thread.
    pub fn send(&self, msg: HostMessage) -> Result<(), HostMessageError> {
        if msg.topic.len() > MAX_TOPIC_BYTES {
            return Err(HostMessageError::TooLarge);
        }
        match &msg.payload {
            HostMessagePayload::Str(s) if s.len() > MAX_STR_PAYLOAD_BYTES => {
                return Err(HostMessageError::TooLarge);
            }
            HostMessagePayload::Bytes(b) if b.len() > MAX_BYTES_PAYLOAD => {
                return Err(HostMessageError::TooLarge);
            }
            _ => {}
        }
        match self.tx.try_send(msg) {
            Ok(()) => {
                self.pending.store(true, Ordering::Release);
                if let Ok(slot) = self.wake.lock()
                    && let Some(wake) = slot.as_ref()
                {
                    wake();
                }
                Ok(())
            }
            Err(TrySendError::Full(_)) => Err(HostMessageError::Full),
            Err(TrySendError::Disconnected(_)) => Err(HostMessageError::Disconnected),
        }
    }

    /// Enqueue a topic-only message.
    pub fn emit_null(&self, topic: impl Into<String>) -> Result<(), HostMessageError> {
        self.send(HostMessage::null(topic))
    }

    /// Enqueue a Boolean message.
    pub fn emit_bool(&self, topic: impl Into<String>, value: bool) -> Result<(), HostMessageError> {
        self.send(HostMessage::bool(topic, value))
    }

    /// Enqueue a signed 32-bit integer message.
    pub fn emit_i32(&self, topic: impl Into<String>, value: i32) -> Result<(), HostMessageError> {
        self.send(HostMessage::i32(topic, value))
    }

    /// Enqueue a double-precision floating-point message.
    pub fn emit_f64(&self, topic: impl Into<String>, value: f64) -> Result<(), HostMessageError> {
        self.send(HostMessage::f64(topic, value))
    }

    /// Enqueue a UTF-8 string message.
    pub fn emit_str(
        &self,
        topic: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<(), HostMessageError> {
        self.send(HostMessage::str(topic, value))
    }

    /// Enqueue an opaque binary message.
    pub fn emit_bytes(
        &self,
        topic: impl Into<String>,
        value: impl Into<Vec<u8>>,
    ) -> Result<(), HostMessageError> {
        self.send(HostMessage::bytes(topic, value))
    }

    /// Retry `try_send` until `timeout`.
    pub fn send_timeout(
        &self,
        msg: HostMessage,
        timeout: Duration,
    ) -> Result<(), HostMessageError> {
        let start = std::time::Instant::now();
        loop {
            match self.send(msg.clone()) {
                Ok(()) => return Ok(()),
                Err(HostMessageError::Full) if start.elapsed() < timeout => {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// Receiver half owned by the applier (UI thread).
pub(crate) struct HostMessageInbox {
    rx: Receiver<HostMessage>,
    pending: Arc<AtomicBool>,
    wake: Arc<Mutex<Option<WakeCallback>>>,
}

impl HostMessageInbox {
    pub(crate) fn set_wake(&self, wake: WakeCallback) {
        if let Ok(mut slot) = self.wake.lock() {
            *slot = Some(wake);
        }
    }

    pub(crate) fn has_pending(&self) -> bool {
        self.pending.load(Ordering::Acquire)
    }

    pub(crate) fn drain_batch(&self) -> Vec<HostMessage> {
        let mut batch = Vec::new();
        while batch.len() < MAX_HOST_MESSAGES_PER_FRAME {
            match self.rx.try_recv() {
                Ok(msg) => batch.push(msg),
                Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
            }
        }
        if batch.len() >= MAX_HOST_MESSAGES_PER_FRAME {
            self.pending.store(true, Ordering::Release);
        } else {
            self.pending.store(false, Ordering::Release);
        }
        batch
    }
}

pub(crate) fn host_message_channel(capacity: usize) -> (HostMessageHandle, HostMessageInbox) {
    let capacity = capacity.max(1);
    let (tx, rx) = mpsc::sync_channel(capacity);
    let pending = Arc::new(AtomicBool::new(false));
    let wake = Arc::new(Mutex::new(None));
    (
        HostMessageHandle {
            tx,
            pending: pending.clone(),
            wake: wake.clone(),
        },
        HostMessageInbox { rx, pending, wake },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_send_and_drain_respects_frame_cap() {
        let (tx, rx) = host_message_channel(8);
        for i in 0..5 {
            tx.emit_i32("t", i).unwrap();
        }
        assert!(rx.has_pending());
        let batch = rx.drain_batch();
        assert_eq!(batch.len(), 5);
        assert!(!rx.has_pending());
    }

    #[test]
    fn full_queue_returns_full() {
        let (tx, _rx) = host_message_channel(2);
        tx.emit_i32("a", 1).unwrap();
        tx.emit_i32("a", 2).unwrap();
        assert_eq!(tx.emit_i32("a", 3), Err(HostMessageError::Full));
    }

    #[test]
    fn per_frame_cap_keeps_pending() {
        let (tx, rx) = host_message_channel(MAX_HOST_MESSAGES_PER_FRAME + 10);
        for i in 0..(MAX_HOST_MESSAGES_PER_FRAME + 3) {
            tx.emit_i32("t", i as i32).unwrap();
        }
        let batch = rx.drain_batch();
        assert_eq!(batch.len(), MAX_HOST_MESSAGES_PER_FRAME);
        assert!(rx.has_pending());
        let rest = rx.drain_batch();
        assert_eq!(rest.len(), 3);
        assert!(!rx.has_pending());
    }

    #[tokio::test]
    async fn producer_context_observes_window_cancellation() {
        let (messages, _inbox) = host_message_channel(2);
        let cancellation = CancellationToken::new();
        let context = HostMessageContext::new(
            9,
            messages,
            cancellation.clone(),
            tokio::runtime::Handle::current(),
        );
        assert_eq!(context.window_id(), 9);
        assert!(!context.is_cancelled());

        cancellation.cancel();
        context.cancelled().await;
        assert!(context.is_cancelled());
    }
}
