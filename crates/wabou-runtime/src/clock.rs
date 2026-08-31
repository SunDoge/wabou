//! Monotonic time source shared by `performance.now()` and animation frames.

pub trait Clock: Send + Sync {
    fn now_ms(&self) -> f64;
}

/// Deterministic monotonic clock shared by headless GPUI and QuickJS.
#[cfg(feature = "headless")]
#[derive(Default)]
pub(crate) struct ManualClock {
    now_micros: std::sync::atomic::AtomicU64,
}

#[cfg(feature = "headless")]
impl ManualClock {
    pub(crate) fn advance(&self, duration: std::time::Duration) {
        let micros = duration.as_micros().min(u128::from(u64::MAX)) as u64;
        self.now_micros
            .fetch_add(micros, std::sync::atomic::Ordering::Relaxed);
    }
}

#[cfg(feature = "headless")]
impl Clock for ManualClock {
    fn now_ms(&self) -> f64 {
        self.now_micros.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1_000.0
    }
}

pub struct SystemClock {
    start: std::time::Instant,
}

impl SystemClock {
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
}
