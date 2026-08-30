//! Host → JS application messages.
//!
//! Background tasks push [`HostMessage`] into a bounded queue without touching
//! QuickJS. The applier drains on the UI thread and encodes each item as an
//! application record in the unified HostEventFrame.

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use wabou_shell_gpui::WakeCallback;

use crate::ui_inbox::{UiInbox, UiInboxSender};

#[derive(Default)]
pub(crate) struct HostTaskTracker {
    active: Mutex<usize>,
    idle: Condvar,
}

impl HostTaskTracker {
    fn started(&self) {
        *self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner()) += 1;
    }

    fn finished(&self) {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        *active = active.saturating_sub(1);
        if *active == 0 {
            self.idle.notify_all();
        }
    }

    pub(crate) fn wait_for_idle(&self, timeout: Duration) -> bool {
        let active = self
            .active
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (active, _) = self
            .idle
            .wait_timeout_while(active, timeout, |active| *active != 0)
            .unwrap_or_else(|error| error.into_inner());
        *active == 0
    }
}

struct HostTaskGuard(Arc<HostTaskTracker>);

impl Drop for HostTaskGuard {
    fn drop(&mut self) {
        self.0.finished();
    }
}

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
    /// A value could not be serialized as JSON.
    Serialization,
    /// The requested window does not currently own a JavaScript runtime.
    WindowUnavailable,
}

/// Cloneable, thread-safe handle for enqueueing host messages.
#[derive(Clone)]
pub struct HostMessageHandle {
    tx: UiInboxSender<HostMessage>,
}

/// Thread-safe application router for sending events to a specific window.
///
/// Register it with [`crate::HostBuilder::host_message_router`], then retain a
/// clone in tray callbacks, services, or other native event sources. Routes
/// are installed when each JavaScript runtime boots and removed when it drops.
#[derive(Clone, Default)]
pub struct HostMessageRouter {
    inner: Arc<HostMessageRouterInner>,
}

/// Snapshot value carrying a monotonically increasing publication revision.
pub trait RevisionedHostSnapshot {
    /// Return the exact revision represented by this snapshot.
    fn revision(&self) -> u64;
}

/// Whether a revisioned publication used a complete snapshot or a patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevisionedHostPublication {
    /// A complete snapshot was queued.
    Snapshot,
    /// A patch from the preceding revision was queued.
    Patch,
    /// An equal or older value was ignored without changing the baseline.
    IgnoredStale,
}

/// Stateful encoder for a host-owned snapshot plus contiguous patches.
///
/// The previous value advances only after a message enters the bounded host
/// queue. A revision gap therefore automatically repairs dropped/coalesced
/// notifications with a complete snapshot.
pub struct RevisionedHostPublisher<T> {
    snapshot_topic: String,
    patch_topic: String,
    previous: Option<T>,
}

impl<T> RevisionedHostPublisher<T>
where
    T: RevisionedHostSnapshot + serde::Serialize,
{
    /// Create a publisher for the full-snapshot and patch topics.
    pub fn new(snapshot_topic: impl Into<String>, patch_topic: impl Into<String>) -> Self {
        Self {
            snapshot_topic: snapshot_topic.into(),
            patch_topic: patch_topic.into(),
            previous: None,
        }
    }

    /// Publish `next`, using `make_patch` only for a contiguous revision.
    pub fn publish<P>(
        &mut self,
        messages: &HostMessageHandle,
        next: T,
        make_patch: impl FnOnce(&T, &T) -> P,
    ) -> Result<RevisionedHostPublication, HostMessageError>
    where
        P: serde::Serialize,
    {
        let publication = if let Some(previous) = &self.previous {
            if next.revision() <= previous.revision() {
                return Ok(RevisionedHostPublication::IgnoredStale);
            }
            if previous.revision().checked_add(1) == Some(next.revision()) {
                messages.emit_json(&self.patch_topic, &make_patch(previous, &next))?;
                RevisionedHostPublication::Patch
            } else {
                messages.emit_json(&self.snapshot_topic, &next)?;
                RevisionedHostPublication::Snapshot
            }
        } else {
            messages.emit_json(&self.snapshot_topic, &next)?;
            RevisionedHostPublication::Snapshot
        };
        self.previous = Some(next);
        Ok(publication)
    }

    /// Forget the last successfully published value, forcing a full snapshot.
    pub fn reset(&mut self) {
        self.previous = None;
    }
}

#[derive(Default)]
struct HostMessageRouterInner {
    next_generation: AtomicU64,
    routes: Mutex<HashMap<wabou_shell_gpui::WindowResourceKey, (u64, HostMessageHandle)>>,
}

impl HostMessageRouter {
    /// Create an empty router. Routes appear as registered windows boot.
    pub fn new() -> Self {
        Self::default()
    }

    /// Send a message to the current JavaScript runtime for `window_key`.
    pub fn send_to(
        &self,
        window_key: wabou_shell_gpui::WindowResourceKey,
        message: HostMessage,
    ) -> Result<(), HostMessageError> {
        let handle = self
            .inner
            .routes
            .lock()
            .map_err(|_| HostMessageError::Disconnected)?
            .get(&window_key)
            .map(|(_, handle)| handle.clone())
            .ok_or(HostMessageError::WindowUnavailable)?;
        handle.send(message)
    }

    pub(crate) fn attach(&self, context: HostMessageContext) {
        let generation = self
            .inner
            .next_generation
            .fetch_add(1, Ordering::Relaxed)
            .wrapping_add(1);
        let window_key = context.window_key();
        if let Ok(mut routes) = self.inner.routes.lock() {
            routes.insert(window_key, (generation, context.messages().clone()));
        }
        let router = self.clone();
        let cancellation = context.clone();
        context.spawn(async move {
            cancellation.cancelled().await;
            router.detach(window_key, generation);
        });
    }

    fn detach(&self, window_key: wabou_shell_gpui::WindowResourceKey, generation: u64) {
        if let Ok(mut routes) = self.inner.routes.lock()
            && routes
                .get(&window_key)
                .is_some_and(|(current, _)| *current == generation)
        {
            routes.remove(&window_key);
        }
    }
}

/// Per-window context for a long-running Rust → JavaScript message producer.
///
/// A producer owns a clone of this context. Wabou cancels it when the native
/// window and its JavaScript runtime are dropped, allowing background tasks to
/// stop even when they have no message ready to send.
#[derive(Clone)]
pub struct HostMessageContext {
    window_key: wabou_shell_gpui::WindowResourceKey,
    messages: HostMessageHandle,
    cancellation: CancellationToken,
    runtime: tokio::runtime::Handle,
    tasks: Arc<HostTaskTracker>,
}

impl HostMessageContext {
    pub(crate) fn new(
        window_key: wabou_shell_gpui::WindowResourceKey,
        messages: HostMessageHandle,
        cancellation: CancellationToken,
        runtime: tokio::runtime::Handle,
        tasks: Arc<HostTaskTracker>,
    ) -> Self {
        Self {
            window_key,
            messages,
            cancellation,
            runtime,
            tasks,
        }
    }

    /// Typed generational identity of the owning native window.
    pub fn window_key(&self) -> wabou_shell_gpui::WindowResourceKey {
        self.window_key
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
        self.tasks.started();
        let tasks = self.tasks.clone();
        self.runtime.spawn(async move {
            let _guard = HostTaskGuard(tasks);
            future.await
        })
    }
}

impl HostMessageHandle {
    /// Non-blocking send. Safe from any thread.
    pub fn send(&self, msg: HostMessage) -> Result<(), HostMessageError> {
        validate_message(&msg)?;
        match self.tx.try_send(msg) {
            Ok(()) => Ok(()),
            Err(flume::TrySendError::Full(_)) => Err(HostMessageError::Full),
            Err(flume::TrySendError::Disconnected(_)) => Err(HostMessageError::Disconnected),
        }
    }

    /// Wait asynchronously for queue capacity, then wake the UI thread.
    ///
    /// This is intended for background tasks that must not drop messages when
    /// the bounded queue is temporarily full. UI-thread callers should use
    /// [`Self::send`] and handle backpressure explicitly.
    pub async fn send_async(&self, msg: HostMessage) -> Result<(), HostMessageError> {
        validate_message(&msg)?;
        self.tx
            .send_async(msg)
            .await
            .map_err(|_| HostMessageError::Disconnected)
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

    /// Serialize a value as JSON, using a string payload when it fits and the
    /// larger binary payload otherwise.
    pub fn emit_json<T: serde::Serialize>(
        &self,
        topic: impl Into<String>,
        value: &T,
    ) -> Result<(), HostMessageError> {
        let topic = topic.into();
        let bytes = serde_json::to_vec(value).map_err(|_| HostMessageError::Serialization)?;
        if bytes.len() <= MAX_STR_PAYLOAD_BYTES {
            let text = String::from_utf8(bytes).map_err(|_| HostMessageError::Serialization)?;
            self.emit_str(topic, text)
        } else {
            self.emit_bytes(topic, bytes)
        }
    }

    /// Retry `try_send` until `timeout`.
    pub fn send_timeout(
        &self,
        msg: HostMessage,
        timeout: Duration,
    ) -> Result<(), HostMessageError> {
        validate_message(&msg)?;
        match self.tx.send_timeout(msg, timeout) {
            Ok(()) => Ok(()),
            Err(flume::SendTimeoutError::Timeout(_)) => Err(HostMessageError::Full),
            Err(flume::SendTimeoutError::Disconnected(_)) => Err(HostMessageError::Disconnected),
        }
    }
}

fn validate_message(msg: &HostMessage) -> Result<(), HostMessageError> {
    if msg.topic.len() > MAX_TOPIC_BYTES {
        return Err(HostMessageError::TooLarge);
    }
    match &msg.payload {
        HostMessagePayload::Str(value) if value.len() > MAX_STR_PAYLOAD_BYTES => {
            Err(HostMessageError::TooLarge)
        }
        HostMessagePayload::Bytes(value) if value.len() > MAX_BYTES_PAYLOAD => {
            Err(HostMessageError::TooLarge)
        }
        _ => Ok(()),
    }
}

/// Receiver half owned by the applier (UI thread).
pub(crate) struct HostMessageInbox {
    inbox: UiInbox<HostMessage>,
}

impl HostMessageInbox {
    pub(crate) fn set_wake(&self, wake: WakeCallback) {
        self.inbox.set_wake(wake);
    }

    pub(crate) fn has_pending(&self) -> bool {
        self.inbox.has_pending()
    }

    pub(crate) fn drain_batch(&self) -> Vec<HostMessage> {
        self.inbox.drain_up_to(MAX_HOST_MESSAGES_PER_FRAME)
    }
}

pub(crate) fn host_message_channel(capacity: usize) -> (HostMessageHandle, HostMessageInbox) {
    let capacity = capacity.max(1);
    let (tx, inbox) = crate::ui_inbox::bounded(capacity);
    (HostMessageHandle { tx }, HostMessageInbox { inbox })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Debug, Serialize)]
    struct TestSnapshot {
        revision: u64,
        value: u64,
    }

    impl RevisionedHostSnapshot for TestSnapshot {
        fn revision(&self) -> u64 {
            self.revision
        }
    }

    #[derive(Serialize)]
    struct TestPatch {
        base_revision: u64,
        revision: u64,
        delta: u64,
    }

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
        assert_eq!(
            tx.send_timeout(HostMessage::i32("a", 4), Duration::ZERO),
            Err(HostMessageError::Full)
        );
    }

    #[tokio::test]
    async fn async_sender_waits_for_sync_ui_drain() {
        let (tx, rx) = host_message_channel(1);
        tx.emit_i32("a", 1).unwrap();
        let queued = tokio::spawn({
            let tx = tx.clone();
            async move { tx.send_async(HostMessage::i32("a", 2)).await }
        });
        tokio::task::yield_now().await;
        assert!(!queued.is_finished());

        let mut delivered = rx.drain_batch();
        queued.await.unwrap().unwrap();
        delivered.extend(rx.drain_batch());

        assert_eq!(
            delivered,
            [HostMessage::i32("a", 1), HostMessage::i32("a", 2)]
        );
        assert!(!rx.has_pending());
    }

    #[test]
    fn json_messages_switch_from_strings_to_bytes_without_changing_payload() {
        let (tx, rx) = host_message_channel(3);
        tx.emit_json("small", &serde_json::json!({ "ready": true }))
            .unwrap();
        let large = "x".repeat(MAX_STR_PAYLOAD_BYTES + 1);
        tx.emit_json("large", &large).unwrap();
        let messages = rx.drain_batch();
        assert!(matches!(messages[0].payload, HostMessagePayload::Str(_)));
        let HostMessagePayload::Bytes(bytes) = &messages[1].payload else {
            panic!("large JSON should use a byte payload");
        };
        assert_eq!(serde_json::from_slice::<String>(bytes).unwrap(), large);
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
            wabou_shell_gpui::WindowResourceKey::from_parts(9, 1).unwrap(),
            messages,
            cancellation.clone(),
            tokio::runtime::Handle::current(),
            Arc::new(HostTaskTracker::default()),
        );
        assert_eq!(context.window_key().into_parts(), (9, 1));
        assert!(!context.is_cancelled());

        cancellation.cancel();
        context.cancelled().await;
        assert!(context.is_cancelled());
    }

    #[tokio::test]
    async fn router_tracks_the_current_runtime_for_each_window() {
        let router = HostMessageRouter::new();
        let window_key = wabou_shell_gpui::WindowResourceKey::from_parts(9, 1).unwrap();
        assert_eq!(
            router.send_to(window_key, HostMessage::null("before")),
            Err(HostMessageError::WindowUnavailable)
        );

        let (messages, inbox) = host_message_channel(2);
        let cancellation = CancellationToken::new();
        router.attach(HostMessageContext::new(
            window_key,
            messages,
            cancellation.clone(),
            tokio::runtime::Handle::current(),
            Arc::new(HostTaskTracker::default()),
        ));
        router
            .send_to(window_key, HostMessage::str("ready", "yes"))
            .unwrap();
        assert_eq!(inbox.drain_batch()[0], HostMessage::str("ready", "yes"));

        cancellation.cancel();
        tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if router.send_to(window_key, HostMessage::null("after"))
                    == Err(HostMessageError::WindowUnavailable)
                {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn stale_runtime_cleanup_does_not_remove_a_replacement_route() {
        let router = HostMessageRouter::new();
        let window_key = wabou_shell_gpui::WindowResourceKey::from_parts(9, 1).unwrap();
        let (old_messages, _old_inbox) = host_message_channel(2);
        let old_cancellation = CancellationToken::new();
        router.attach(HostMessageContext::new(
            window_key,
            old_messages,
            old_cancellation.clone(),
            tokio::runtime::Handle::current(),
            Arc::new(HostTaskTracker::default()),
        ));

        let (new_messages, new_inbox) = host_message_channel(2);
        let new_cancellation = CancellationToken::new();
        router.attach(HostMessageContext::new(
            window_key,
            new_messages,
            new_cancellation.clone(),
            tokio::runtime::Handle::current(),
            Arc::new(HostTaskTracker::default()),
        ));
        old_cancellation.cancel();
        tokio::task::yield_now().await;

        router
            .send_to(window_key, HostMessage::str("current", "new"))
            .unwrap();
        assert_eq!(
            new_inbox.drain_batch()[0],
            HostMessage::str("current", "new")
        );
        new_cancellation.cancel();
    }

    #[test]
    fn revisioned_publisher_uses_patches_only_after_successful_contiguous_values() {
        let (messages, inbox) = host_message_channel(4);
        let mut publisher = RevisionedHostPublisher::new("snapshot", "patch");
        let patch = |old: &TestSnapshot, next: &TestSnapshot| TestPatch {
            base_revision: old.revision,
            revision: next.revision,
            delta: next.value - old.value,
        };

        assert_eq!(
            publisher
                .publish(
                    &messages,
                    TestSnapshot {
                        revision: 1,
                        value: 10,
                    },
                    patch,
                )
                .unwrap(),
            RevisionedHostPublication::Snapshot
        );
        assert_eq!(
            publisher
                .publish(
                    &messages,
                    TestSnapshot {
                        revision: 2,
                        value: 13,
                    },
                    patch,
                )
                .unwrap(),
            RevisionedHostPublication::Patch
        );
        assert_eq!(
            publisher
                .publish(
                    &messages,
                    TestSnapshot {
                        revision: 4,
                        value: 20,
                    },
                    patch,
                )
                .unwrap(),
            RevisionedHostPublication::Snapshot
        );
        let topics = inbox
            .drain_batch()
            .into_iter()
            .map(|message| message.topic)
            .collect::<Vec<_>>();
        assert_eq!(topics, ["snapshot", "patch", "snapshot"]);
    }

    #[test]
    fn revisioned_publisher_does_not_advance_when_the_queue_is_full() {
        let (messages, inbox) = host_message_channel(1);
        let mut publisher = RevisionedHostPublisher::new("snapshot", "patch");
        messages.emit_null("occupied").unwrap();
        assert_eq!(
            publisher.publish(
                &messages,
                TestSnapshot {
                    revision: 1,
                    value: 10,
                },
                |_, _| serde_json::Value::Null,
            ),
            Err(HostMessageError::Full)
        );
        inbox.drain_batch();
        assert_eq!(
            publisher
                .publish(
                    &messages,
                    TestSnapshot {
                        revision: 2,
                        value: 20,
                    },
                    |_, _| serde_json::Value::Null,
                )
                .unwrap(),
            RevisionedHostPublication::Snapshot
        );
    }

    #[test]
    fn revisioned_publisher_ignores_equal_and_regressing_values() {
        let (messages, inbox) = host_message_channel(4);
        let mut publisher = RevisionedHostPublisher::new("snapshot", "patch");
        let patch = |old: &TestSnapshot, next: &TestSnapshot| TestPatch {
            base_revision: old.revision,
            revision: next.revision,
            delta: next.value - old.value,
        };
        publisher
            .publish(
                &messages,
                TestSnapshot {
                    revision: 2,
                    value: 20,
                },
                patch,
            )
            .unwrap();
        assert_eq!(
            publisher
                .publish(
                    &messages,
                    TestSnapshot {
                        revision: 2,
                        value: 999,
                    },
                    patch,
                )
                .unwrap(),
            RevisionedHostPublication::IgnoredStale
        );
        assert_eq!(
            publisher
                .publish(
                    &messages,
                    TestSnapshot {
                        revision: 1,
                        value: 1,
                    },
                    patch,
                )
                .unwrap(),
            RevisionedHostPublication::IgnoredStale
        );
        assert_eq!(
            publisher
                .publish(
                    &messages,
                    TestSnapshot {
                        revision: 3,
                        value: 23,
                    },
                    patch,
                )
                .unwrap(),
            RevisionedHostPublication::Patch
        );
        let topics = inbox
            .drain_batch()
            .into_iter()
            .map(|message| message.topic)
            .collect::<Vec<_>>();
        assert_eq!(topics, ["snapshot", "patch"]);
    }
}
