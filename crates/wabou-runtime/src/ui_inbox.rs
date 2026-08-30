//! Background-producer queues drained at safe points on the UI thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use arc_swap::ArcSwapOption;
use gpui_shell::WakeCallback;

struct StoredWakeCallback(WakeCallback);

struct WakeState {
    pending: AtomicBool,
    callback: ArcSwapOption<StoredWakeCallback>,
}

/// Cloneable producer for work owned and applied by the UI thread.
pub(crate) struct UiInboxSender<T> {
    sender: flume::Sender<T>,
    wake: Arc<WakeState>,
}

impl<T> Clone for UiInboxSender<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            wake: self.wake.clone(),
        }
    }
}

impl<T> UiInboxSender<T> {
    /// Send without blocking and wake the native event loop after publication.
    pub(crate) fn try_send(&self, message: T) -> Result<(), flume::TrySendError<T>> {
        self.sender.try_send(message)?;
        self.notify();
        Ok(())
    }

    /// Wait asynchronously for bounded capacity, then wake the UI thread.
    pub(crate) async fn send_async(&self, message: T) -> Result<(), flume::SendError<T>> {
        self.sender.send_async(message).await?;
        self.notify();
        Ok(())
    }

    /// Wait synchronously up to `timeout` for bounded capacity.
    pub(crate) fn send_timeout(
        &self,
        message: T,
        timeout: Duration,
    ) -> Result<(), flume::SendTimeoutError<T>> {
        self.sender.send_timeout(message, timeout)?;
        self.notify();
        Ok(())
    }

    fn notify(&self) {
        self.wake.pending.store(true, Ordering::Release);
        let callback = self.wake.callback.load();
        if let Some(callback) = callback.as_ref() {
            (callback.0)();
        }
    }
}

/// Consumer owned and drained exclusively by the UI thread.
pub(crate) struct UiInbox<T> {
    receiver: flume::Receiver<T>,
    wake: Arc<WakeState>,
}

impl<T> UiInbox<T> {
    /// Whether a producer has published work since the previous drain began.
    pub(crate) fn has_pending(&self) -> bool {
        self.wake.pending.load(Ordering::Acquire)
    }

    /// Install or replace the event-loop callback used by producers.
    pub(crate) fn set_wake(&self, callback: WakeCallback) {
        self.wake
            .callback
            .store(Some(Arc::new(StoredWakeCallback(callback))));
    }

    /// Drain every message that is immediately available.
    pub(crate) fn drain(&self) -> Vec<T> {
        self.begin_drain();
        self.receiver.try_iter().collect()
    }

    /// Drain at most `limit` messages and retain pending state for the rest.
    pub(crate) fn drain_up_to(&self, limit: usize) -> Vec<T> {
        self.begin_drain();
        let batch: Vec<_> = self.receiver.try_iter().take(limit).collect();
        if !self.receiver.is_empty() {
            self.wake.pending.store(true, Ordering::Release);
        }
        batch
    }

    fn begin_drain(&self) {
        // Clear first: a producer racing with this drain restores the bit and
        // wakes the event loop after publishing its message.
        self.wake.pending.store(false, Ordering::Release);
    }
}

fn channel<T>(
    sender: flume::Sender<T>,
    receiver: flume::Receiver<T>,
) -> (UiInboxSender<T>, UiInbox<T>) {
    let wake = Arc::new(WakeState {
        pending: AtomicBool::new(false),
        callback: ArcSwapOption::empty(),
    });
    (
        UiInboxSender {
            sender,
            wake: wake.clone(),
        },
        UiInbox { receiver, wake },
    )
}

/// Create a non-blocking producer with explicit backpressure.
pub(crate) fn bounded<T>(capacity: usize) -> (UiInboxSender<T>, UiInbox<T>) {
    let (sender, receiver) = flume::bounded(capacity.max(1));
    channel(sender, receiver)
}

/// Create a producer for infrequent control messages that must not be lost.
pub(crate) fn unbounded<T>() -> (UiInboxSender<T>, UiInbox<T>) {
    let (sender, receiver) = flume::unbounded();
    channel(sender, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_marks_pending_and_wakes() {
        let (sender, inbox) = bounded(1);
        let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callback_wakes = wakes.clone();
        inbox.set_wake(Arc::new(move || {
            callback_wakes.fetch_add(1, Ordering::Relaxed);
        }));

        sender.try_send(7).unwrap();

        assert!(inbox.has_pending());
        assert_eq!(wakes.load(Ordering::Relaxed), 1);
        assert_eq!(inbox.drain(), [7]);
        assert!(!inbox.has_pending());
    }

    #[test]
    fn replacing_wake_callback_changes_the_next_notification() {
        let (sender, inbox) = unbounded();
        let first = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let first_wakes = first.clone();
        inbox.set_wake(Arc::new(move || {
            first_wakes.fetch_add(1, Ordering::Relaxed);
        }));
        sender.try_send(1).unwrap();

        let second = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let second_wakes = second.clone();
        inbox.set_wake(Arc::new(move || {
            second_wakes.fetch_add(1, Ordering::Relaxed);
        }));
        sender.try_send(2).unwrap();

        assert_eq!(first.load(Ordering::Relaxed), 1);
        assert_eq!(second.load(Ordering::Relaxed), 1);
        assert_eq!(inbox.drain(), [1, 2]);
    }

    #[test]
    fn bounded_inbox_retains_pending_work_across_batches() {
        let (sender, inbox) = bounded(3);
        sender.try_send(1).unwrap();
        sender.try_send(2).unwrap();
        sender.try_send(3).unwrap();

        assert_eq!(inbox.drain_up_to(2), [1, 2]);
        assert!(inbox.has_pending());
        assert_eq!(inbox.drain_up_to(2), [3]);
        assert!(!inbox.has_pending());
    }

    #[test]
    fn bounded_sender_reports_backpressure_without_waking() {
        let (sender, inbox) = bounded(1);
        let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let callback_wakes = wakes.clone();
        inbox.set_wake(Arc::new(move || {
            callback_wakes.fetch_add(1, Ordering::Relaxed);
        }));
        sender.try_send(1).unwrap();
        assert!(matches!(
            sender.try_send(2),
            Err(flume::TrySendError::Full(2))
        ));
        assert_eq!(wakes.load(Ordering::Relaxed), 1);
        assert_eq!(inbox.drain(), [1]);
    }

    #[test]
    fn publication_after_drain_begins_restores_pending() {
        let (sender, inbox) = unbounded();

        inbox.begin_drain();
        sender.try_send(9).unwrap();

        assert!(inbox.has_pending());
        assert_eq!(inbox.drain(), [9]);
    }

    #[test]
    fn async_producer_interoperates_with_sync_ui_drain() {
        let (sender, inbox) = bounded(1);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(sender.send_async(11)).unwrap();

        assert!(inbox.has_pending());
        assert_eq!(inbox.drain(), [11]);
    }
}
