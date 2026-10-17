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

/// Convenience alias shared by clock consumers.
pub type SharedClock = Arc<dyn Clock>;
