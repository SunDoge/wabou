//! Monotonic time source for semantic timing.

use std::sync::Arc;

/// A monotonic clock supplying every semantic time read in the host:
/// `performance.now`, animation-frame timestamps, interaction timestamps,
/// motion advancement.
///
/// `SystemClock` is the production implementation; a test host supplies a
/// manually advanced variant. Mechanical time (watchdogs, diagnostics,
/// network timeouts) keeps reading `Instant::now()` directly.
pub trait Clock: Send + Sync {
    /// Milliseconds on the clock's timeline.
    fn now_ms(&self) -> f64;
    /// Monotonic instant on the same timeline as [`Clock::now_ms`].
    ///
    /// Native time state (`Instant` deadlines, elapsed-since timestamps)
    /// reads this instead of `Instant::now()` so an advanced test clock
    /// stays deterministic; a system clock returns the wall instant
    /// directly. `Instant` arithmetic in consumers is unchanged.
    fn now_instant(&self) -> std::time::Instant;
    /// Downcast hook: lets a runtime adopt a manually advanced clock handed
    /// to it as `Arc<dyn Clock>`. Returns `None` for every other
    /// implementation.
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

/// The production clock: wall-clock monotonic time.
pub struct SystemClock {
    start: std::time::Instant,
}

impl SystemClock {
    /// Start measuring from now.
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }

    fn now_instant(&self) -> std::time::Instant {
        std::time::Instant::now()
    }
}

/// Shared handle to a monotonic [`Clock`].
///
/// Newtype rather than a type alias: a clock slot in a widget or session is
/// an owned handle with intent (the host replaces it through the widget
/// contract's clock hook), and the wrapper keeps inherent constructors and
/// future helpers on the type itself instead of free functions.
#[derive(Clone)]
pub struct SharedClock(Arc<dyn Clock>);

impl SharedClock {
    /// A shared handle to a fresh [`SystemClock`].
    pub fn system() -> Self {
        Self(Arc::new(SystemClock::new()))
    }

    /// Wrap an existing clock handle.
    pub fn new(clock: Arc<dyn Clock>) -> Self {
        Self(clock)
    }

    /// Borrow the underlying clock trait object.
    pub fn as_dyn(&self) -> &dyn Clock {
        self.0.as_ref()
    }
}

impl std::ops::Deref for SharedClock {
    type Target = dyn Clock;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl From<Arc<dyn Clock>> for SharedClock {
    fn from(clock: Arc<dyn Clock>) -> Self {
        Self(clock)
    }
}

impl From<SystemClock> for SharedClock {
    fn from(clock: SystemClock) -> Self {
        Self(Arc::new(clock))
    }
}
