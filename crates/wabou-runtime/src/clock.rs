//! Monotonic time source shared by `performance.now()` and animation frames.
//!
//! [`VirtualClock`] is the deterministic variant used by test hosts: time
//! advances only when the driver calls [`VirtualClock::advance_ms`], and
//! `__wabou_sleep` deadlines are evaluated against virtual time, so a
//! scenario's timer firings and frame timestamps never depend on wall-clock
//! scheduling.

use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

pub use wabou_host_api::clock::{Clock, SharedClock, SystemClock};

/// Host notification fired after timers expire during an advance.
type NotifyCallback = Arc<dyn Fn() + Send + Sync>;

struct VirtualClockState {
    now_ms: f64,
    next_id: u64,
    /// Registered sleeps as `(deadline, id, waker)`.
    pending: Vec<(f64, u64, Waker)>,
    /// Sleeps whose deadline has passed; their futures observe `Ready`.
    fired: HashSet<u64>,
}

/// Manually advanced clock for deterministic test hosts.
///
/// The clock starts at zero. [`VirtualClock::advance_ms`] moves time forward,
/// firing every sleep whose deadline has passed; timer continuations then run
/// on the next JavaScript job poll, which keeps deliveries on frame
/// boundaries. Nothing in the clock observes real time, so interleaved
/// wall-clock sleeps (chaotic scheduling, slow machines) cannot change a
/// scenario's timing trace.
#[derive(Clone)]
pub struct VirtualClock {
    state: Arc<Mutex<VirtualClockState>>,
    /// Host notification fired after timers expire during an advance.
    notify: Arc<Mutex<Option<NotifyCallback>>>,
    /// Real instant virtual zero maps to, so [`Clock::now_instant`] can
    /// derive a genuine `Instant`. Captured once in `new()`; every clone
    /// shares it.
    epoch: std::time::Instant,
}

impl Default for VirtualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualClock {
    /// Construct a clock frozen at zero milliseconds.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(VirtualClockState {
                now_ms: 0.0,
                next_id: 1,
                pending: Vec::new(),
                fired: HashSet::new(),
            })),
            notify: Arc::new(Mutex::new(None)),
            epoch: std::time::Instant::now(),
        }
    }

    /// Register a host callback invoked whenever an advance fires timers, so
    /// the host can schedule the job poll that runs the continuations.
    pub fn set_notify(&self, callback: NotifyCallback) {
        *self.notify.lock().expect("virtual clock mutex poisoned") = Some(callback);
    }

    /// Current virtual time in milliseconds.
    pub fn now_ms(&self) -> f64 {
        self.state
            .lock()
            .expect("virtual clock mutex poisoned")
            .now_ms
    }

    /// Advance virtual time, firing sleeps whose deadline has passed.
    ///
    /// Returns whether any timer fired. Wakers are invoked after the state
    /// lock is released so a re-entrant `advance_ms` cannot deadlock.
    pub fn advance_ms(&self, milliseconds: f64) -> bool {
        let woken = {
            let mut state = self.state.lock().expect("virtual clock mutex poisoned");
            state.now_ms += milliseconds.max(0.0);
            let now = state.now_ms;
            let expired: Vec<u64> = state
                .pending
                .iter()
                .filter(|(deadline, _, _)| *deadline <= now)
                .map(|(_, id, _)| *id)
                .collect();
            let woken: Vec<Waker> = state
                .pending
                .iter()
                .filter(|(_, id, _)| expired.contains(id))
                .map(|(_, _, waker)| waker.clone())
                .collect();
            state.pending.retain(|(_, id, _)| !expired.contains(id));
            for id in &expired {
                state.fired.insert(*id);
            }
            woken
        };
        let fired = !woken.is_empty();
        for waker in woken {
            waker.wake();
        }
        if fired
            && let Some(notify) = self
                .notify
                .lock()
                .expect("virtual clock mutex poisoned")
                .clone()
        {
            notify();
        }
        fired
    }

    /// Create a future that completes once virtual time passes `delay_ms`.
    ///
    /// The deadline is captured on the future's first poll, mirroring how a
    /// real `setTimeout` measures from when the timer starts, not from call
    /// construction.
    pub fn sleep_for(&self, delay_ms: f64) -> VirtualSleep {
        VirtualSleep {
            clock: self.clone(),
            delay_ms: delay_ms.max(0.0),
            id: None,
        }
    }

    /// Drop a sleep registration without firing it.
    fn cancel(&self, id: u64) {
        let mut state = self.state.lock().expect("virtual clock mutex poisoned");
        state.pending.retain(|(_, pending, _)| *pending != id);
        state.fired.remove(&id);
    }
}

impl Clock for VirtualClock {
    fn now_ms(&self) -> f64 {
        VirtualClock::now_ms(self)
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn now_instant(&self) -> std::time::Instant {
        let ms = VirtualClock::now_ms(self);
        // Clamp at the epoch when virtual time exceeds what `Instant` can
        // represent (years of virtual milliseconds): the clock freezes
        // rather than panicking, and monotonicity is preserved. Integer
        // arithmetic throughout — `Duration::from_secs_f64` panics on
        // values past `Duration`'s internal nanosecond range, well below
        // its seconds range.
        let whole_ms = if ms.is_finite() && ms > 0.0 {
            ms as u64
        } else {
            0
        };
        let elapsed = std::time::Duration::from_millis(whole_ms);
        self.epoch.checked_add(elapsed).unwrap_or(self.epoch)
    }
}

/// Future returned by [`VirtualClock::sleep_for`].
pub struct VirtualSleep {
    clock: VirtualClock,
    delay_ms: f64,
    /// Registration id once the deadline has been captured.
    id: Option<u64>,
}

impl Future for VirtualSleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        let this = &mut *self;
        let mut state = this
            .clock
            .state
            .lock()
            .expect("virtual clock mutex poisoned");
        match this.id {
            Some(id) if state.fired.contains(&id) => Poll::Ready(()),
            Some(id) => {
                if let Some(entry) = state
                    .pending
                    .iter_mut()
                    .find(|(_, pending, _)| *pending == id)
                {
                    // Futures can be re-polled with a different waker (task
                    // migration); keep the freshest one.
                    entry.2 = context.waker().clone();
                }
                Poll::Pending
            }
            None => {
                let id = state.next_id;
                state.next_id += 1;
                let deadline = state.now_ms + this.delay_ms;
                state.pending.push((deadline, id, context.waker().clone()));
                this.id = Some(id);
                // A zero-delay sleep (or one created after time already passed
                // its deadline) still waits for the next advance, keeping every
                // firing on an explicit frame boundary.
                Poll::Pending
            }
        }
    }
}

impl Drop for VirtualSleep {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            self.clock.cancel(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::task::{RawWaker, RawWakerVTable};

    fn noop_waker() -> Waker {
        fn clone(_: *const ()) -> RawWaker {
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        fn noop(_: *const ()) {}
        static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
        // SAFETY: the vtable functions are the no-ops above.
        unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
    }

    #[test]
    fn sleeps_fire_on_advance_only_and_carry_their_deadline() {
        let clock = VirtualClock::new();
        let mut early = Box::pin(clock.sleep_for(10.0));
        let mut late = Box::pin(clock.sleep_for(50.0));
        let waker = noop_waker();
        let mut context = Context::from_waker(&waker);

        assert!(early.as_mut().poll(&mut context).is_pending());
        assert!(late.as_mut().poll(&mut context).is_pending());
        assert!(!clock.advance_ms(9.0));
        assert!(early.as_mut().poll(&mut context).is_pending());
        assert!(clock.advance_ms(1.0));
        assert!(early.as_mut().poll(&mut context).is_ready());
        assert!(late.as_mut().poll(&mut context).is_pending());
        clock.advance_ms(39.0);
        assert!(late.as_mut().poll(&mut context).is_pending());
        clock.advance_ms(1.0);
        assert!(late.as_mut().poll(&mut context).is_ready());
        assert_eq!(clock.now_ms(), 50.0);
    }

    #[test]
    fn dropped_sleeps_are_never_fired_and_do_not_block_others() {
        let clock = VirtualClock::new();
        let waker = noop_waker();
        let mut context = Context::from_waker(&waker);
        let cancelled = clock.sleep_for(10.0);
        let mut kept = Box::pin(clock.sleep_for(10.0));
        drop(cancelled);
        assert!(kept.as_mut().poll(&mut context).is_pending());
        assert!(clock.advance_ms(10.0));
        assert!(kept.as_mut().poll(&mut context).is_ready());
    }

    #[test]
    fn now_instant_is_monotone_and_tracks_virtual_elapsed() {
        let clock = VirtualClock::new();
        let epoch = clock.now_instant();
        assert_eq!(epoch, clock.now_instant());
        clock.advance_ms(50.0);
        let later = clock.now_instant();
        assert!(later > epoch);
        assert_eq!(
            later.duration_since(epoch).as_millis(),
            std::time::Duration::from_millis(50).as_millis()
        );
        clock.advance_ms(0.0);
        assert_eq!(
            clock.now_instant(),
            later,
            "zero advance must not move time"
        );
    }

    #[test]
    fn now_instant_clamps_instead_of_panicking() {
        let clock = VirtualClock::new();
        // Far beyond Instant's representable range; checked_add saturates.
        clock.advance_ms(f64::MAX / 1e6);
        let clamped = clock.now_instant();
        assert!(clamped >= clock.epoch);
    }

    #[test]
    fn advance_notifies_the_host_after_firing() {
        let clock = VirtualClock::new();
        let count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counter = count.clone();
        clock.set_notify(Arc::new(move || {
            counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }));
        let waker = noop_waker();
        let mut context = Context::from_waker(&waker);
        let mut sleep = Box::pin(clock.sleep_for(5.0));
        assert!(sleep.as_mut().poll(&mut context).is_pending());
        assert!(!clock.advance_ms(4.0));
        assert_eq!(count.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert!(clock.advance_ms(1.0));
        assert_eq!(count.load(std::sync::atomic::Ordering::Relaxed), 1);
    }
}
