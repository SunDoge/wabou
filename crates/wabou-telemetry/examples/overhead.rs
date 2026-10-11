//! Small local overhead probe; not an application performance benchmark.
use std::{hint::black_box, time::Instant};
use wabou_telemetry::{Mode, Recorder};
fn main() {
    for mode in [Mode::Off, Mode::Counters, Mode::Trace] {
        let recorder = Recorder::new();
        recorder.configure(mode, 16384).unwrap();
        let count = 50000;
        let start = Instant::now();
        for _ in 0..count {
            let span = recorder.span("probe", "operation");
            black_box(&span);
            drop(span);
        }
        println!(
            "{mode:?}: {:.1} ns/measurement",
            start.elapsed().as_nanos() as f64 / count as f64
        );
    }
}
