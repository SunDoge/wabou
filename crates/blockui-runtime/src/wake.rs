//! Host notifications must not terminate task/executor threads.
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::Arc,
};
pub(crate) type Callback = Arc<dyn Fn() + Send + Sync>;

/// Disable a failed notification for this job; its data completion remains valid.
pub(crate) fn notify(callback: &mut Option<Callback>) {
    if let Some(wake) = callback.as_ref()
        && catch_unwind(AssertUnwindSafe(|| wake())).is_err()
    {
        *callback = None;
    }
}

/// Runtime-owned callback slot; jobs see later installation/replacement.
#[derive(Default)]
pub(crate) struct Slot(std::sync::Mutex<Option<Callback>>);
impl Slot {
    pub(crate) fn set(&self, callback: Callback) {
        let retired = self.0.lock().unwrap().replace(callback);
        // Captured host values may re-enter the notification slot on drop.
        drop(retired);
    }
    pub(crate) fn notify(&self) {
        let callback = self.0.lock().unwrap().clone();
        if let Some(callback) = callback
            && catch_unwind(AssertUnwindSafe(|| callback())).is_err()
        {
            let mut current = self.0.lock().unwrap();
            if current
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &callback))
            {
                *current = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[test]
    fn replacing_callback_releases_captures_outside_slot_lock() {
        struct Probe {
            slot: std::sync::Weak<Slot>,
            unlocked: Arc<std::sync::atomic::AtomicBool>,
        }
        impl Drop for Probe {
            fn drop(&mut self) {
                let slot = self.slot.upgrade().unwrap();
                self.unlocked
                    .store(slot.0.try_lock().is_ok(), Ordering::Release);
            }
        }
        let slot = Arc::new(Slot::default());
        let unlocked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let probe = Probe {
            slot: Arc::downgrade(&slot),
            unlocked: unlocked.clone(),
        };
        slot.set(Arc::new(move || {
            let _ = &probe;
        }));
        slot.set(Arc::new(|| {}));
        assert!(
            unlocked.load(Ordering::Acquire),
            "capture dropped under notification slot lock"
        );
    }

    #[test]
    fn failed_callback_capture_can_reinstall_notification_during_drop() {
        struct Reinstall {
            slot: std::sync::Weak<Slot>,
            unlocked: Arc<std::sync::atomic::AtomicBool>,
            healthy_calls: Arc<AtomicUsize>,
        }
        impl Drop for Reinstall {
            fn drop(&mut self) {
                let slot = self.slot.upgrade().unwrap();
                let unlocked = slot.0.try_lock().is_ok();
                self.unlocked.store(unlocked, Ordering::Release);
                // Avoid deadlocking a failing regression on a locked slot.
                if unlocked {
                    let calls = self.healthy_calls.clone();
                    slot.set(Arc::new(move || {
                        calls.fetch_add(1, Ordering::Relaxed);
                    }));
                }
            }
        }
        let slot = Arc::new(Slot::default());
        let unlocked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let healthy_calls = Arc::new(AtomicUsize::new(0));
        let capture = Reinstall {
            slot: Arc::downgrade(&slot),
            unlocked: unlocked.clone(),
            healthy_calls: healthy_calls.clone(),
        };
        slot.set(Arc::new(move || {
            let _ = &capture;
            panic!("failed host wake with reentrant capture drop");
        }));
        slot.notify();
        assert!(
            unlocked.load(Ordering::Acquire),
            "failed callback capture dropped under slot lock"
        );
        assert_eq!(healthy_calls.load(Ordering::Relaxed), 0);
        slot.notify();
        slot.notify();
        assert_eq!(healthy_calls.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn failed_old_callback_does_not_remove_reentrant_replacement() {
        let slot = Arc::new(Slot::default());
        let replacement_slot = slot.clone();
        let calls = Arc::new(AtomicUsize::new(0));
        let replacement_calls = calls.clone();
        slot.set(Arc::new(move || {
            let calls = replacement_calls.clone();
            replacement_slot.set(Arc::new(move || {
                calls.fetch_add(1, Ordering::Relaxed);
            }));
            panic!("old callback failed after replacement");
        }));
        slot.notify();
        slot.notify();
        assert_eq!(calls.load(Ordering::Relaxed), 1);
    }
    #[test]
    fn failed_notification_is_disabled_and_successful_one_is_retained() {
        let calls = Arc::new(AtomicUsize::new(0));
        let count = calls.clone();
        let mut failed: Option<Callback> = Some(Arc::new(move || {
            count.fetch_add(1, Ordering::Relaxed);
            panic!("notification failed");
        }));
        notify(&mut failed);
        notify(&mut failed);
        assert!(failed.is_none());
        assert_eq!(calls.load(Ordering::Relaxed), 1);
        let count = calls.clone();
        let mut healthy: Option<Callback> = Some(Arc::new(move || {
            count.fetch_add(1, Ordering::Relaxed);
        }));
        notify(&mut healthy);
        notify(&mut healthy);
        assert!(healthy.is_some());
        assert_eq!(calls.load(Ordering::Relaxed), 3);
    }
}
