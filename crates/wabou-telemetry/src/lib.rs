//! Bounded CPU timeline shared by the resident runtime and renderer.
//! Disabled instrumentation does not acquire locks or allocate event storage.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU8, AtomicU64, Ordering},
    },
    time::Instant,
};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    Off,
    Counters,
    Trace,
}
#[derive(Default)]
struct Aggregate {
    count: u64,
    total: u64,
    max: u64,
    samples: VecDeque<u64>,
}
#[derive(Clone, Serialize)]
struct Event {
    name: String,
    cat: String,
    ph: &'static str,
    ts: u64,
    dur: u64,
    pid: u32,
    tid: u64,
    args: BTreeMap<String, u64>,
}
struct State {
    mode: Mode,
    capacity: usize,
    generation: u64,
    dropped: u64,
    groups: BTreeMap<(String, String), Aggregate>,
    events: VecDeque<Event>,
}
pub struct Recorder {
    start: Instant,
    mode: AtomicU8,
    sequence: AtomicU64,
    overhead_ns: AtomicU64,
    state: Mutex<State>,
}
impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}
impl Recorder {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            mode: AtomicU8::new(0),
            sequence: AtomicU64::new(1),
            overhead_ns: AtomicU64::new(0),
            state: Mutex::new(State {
                mode: Mode::Off,
                capacity: 16384,
                generation: 0,
                dropped: 0,
                groups: BTreeMap::new(),
                events: VecDeque::new(),
            }),
        }
    }
    pub fn configure(&self, mode: Mode, capacity: usize) -> Result<(), String> {
        if !(1..=65536).contains(&capacity) {
            return Err("trace capacity must be between 1 and 65536".into());
        }
        let mut s = self.state.lock().unwrap();
        self.overhead_ns.store(0, Ordering::Relaxed);
        s.generation += 1;
        s.mode = mode;
        s.capacity = capacity;
        s.dropped = 0;
        s.groups.clear();
        s.events.clear();
        self.mode.store(
            match mode {
                Mode::Off => 0,
                Mode::Counters => 1,
                Mode::Trace => 2,
            },
            Ordering::Release,
        );
        Ok(())
    }
    pub fn span(&self, category: &str, name: &str) -> Span<'_> {
        if self.mode.load(Ordering::Acquire) == 0 {
            return Span {
                recorder: None,
                event: None,
                generation: 0,
            };
        }
        let setup = Instant::now();
        let generation = self.state.lock().unwrap().generation;
        let now = Instant::now();
        let id = self.sequence.fetch_add(1, Ordering::Relaxed);
        let span = Span {
            recorder: Some(self),
            generation,
            event: Some(Event {
                name: bounded(name),
                cat: bounded(category),
                ph: "X",
                ts: micros(now.duration_since(self.start)),
                dur: 0,
                pid: std::process::id(),
                tid: thread_id(),
                args: BTreeMap::from([("id".into(), id)]),
            }),
        };
        self.overhead_ns
            .fetch_add(nanos(setup.elapsed()), Ordering::Relaxed);
        span
    }
    fn record(&self, generation: u64, event: Event) {
        let mut s = self.state.lock().unwrap();
        if generation != s.generation || s.mode == Mode::Off {
            return;
        }
        let key = (event.cat.clone(), event.name.clone());
        if s.groups.len() >= 256 && !s.groups.contains_key(&key) {
            s.dropped += 1;
            return;
        }
        let a = s.groups.entry(key).or_default();
        a.count += 1;
        a.total = a.total.saturating_add(event.dur);
        a.max = a.max.max(event.dur);
        if a.samples.len() == 128 {
            a.samples.pop_front();
        }
        a.samples.push_back(event.dur);
        if s.mode == Mode::Trace {
            if s.events.len() == s.capacity {
                s.events.pop_front();
                s.dropped += 1;
            }
            s.events.push_back(event);
        }
    }
    pub fn enabled(&self) -> bool {
        self.mode.load(Ordering::Acquire) != 0
    }
    /// Accept at most 128 completed JS measurements, anchored to the native clock.
    pub fn batch(&self, raw: &str) -> Result<(), String> {
        if !self.enabled() {
            return Ok(());
        }
        if raw.len() > 65536 {
            return Err("performance batch exceeds 64 KiB".into());
        }
        let events: Vec<JsMeasurement> = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        if events.len() > 128 {
            return Err("performance batch exceeds 128 measurements".into());
        }
        let generation = self.state.lock().unwrap().generation;
        let now = micros(self.start.elapsed());
        for e in events {
            self.record(
                generation,
                Event {
                    name: bounded(&e.name),
                    cat: if e.asynchronous {
                        "js.async"
                    } else {
                        "js.measure"
                    }
                    .into(),
                    ph: "X",
                    ts: now.saturating_sub(e.age_us).saturating_sub(e.duration_us),
                    dur: e.duration_us,
                    pid: std::process::id(),
                    tid: thread_id(),
                    args: BTreeMap::from([
                        ("id".into(), self.sequence.fetch_add(1, Ordering::Relaxed)),
                        ("correlation".into(), e.correlation),
                    ]),
                },
            );
        }
        Ok(())
    }
    pub fn snapshot(&self) -> Value {
        let s = self.state.lock().unwrap();
        let summaries:Vec<_>=s.groups.iter().map(|((category,name),a)| {
            let mut samples:Vec<_>=a.samples.iter().copied().collect(); samples.sort_unstable();
            let percentile=|p:usize| samples[(samples.len()*p).div_ceil(100).saturating_sub(1)];
            json!({"category":category,"name":name,"count":a.count,"totalUs":a.total,"maxUs":a.max,"p50Us":percentile(50),"p95Us":percentile(95),"p99Us":percentile(99),"sampleCount":samples.len()})
        }).collect();
        let mut timeline = Vec::new();
        for event in &s.events {
            if matches!(
                event.cat.as_str(),
                "capability.async" | "capability.module" | "js.async"
            ) {
                let mut begin = serde_json::to_value(event).unwrap();
                begin["ph"] = json!("b");
                begin["id"] = json!(event.args["id"]);
                let mut end = begin.clone();
                end["ph"] = json!("e");
                end["ts"] = json!(event.ts.saturating_add(event.dur));
                timeline.push(begin);
                timeline.push(end);
            } else {
                timeline.push(serde_json::to_value(event).unwrap());
            }
        }
        json!({"mode":s.mode,"capacity":s.capacity,"droppedEvents":s.dropped,"summaryWindow":128,"summaries":summaries,"traceEvents":timeline,"displayTimeUnit":"ms","recorderOverheadUs":self.overhead_ns.load(Ordering::Relaxed)/1000})
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct JsMeasurement {
    name: String,
    duration_us: u64,
    age_us: u64,
    #[serde(default)]
    correlation: u64,
    #[serde(default)]
    asynchronous: bool,
}
fn bounded(value: &str) -> String {
    value.chars().take(128).collect()
}
fn nanos(value: std::time::Duration) -> u64 {
    value.as_nanos().min(u64::MAX as u128) as u64
}
fn micros(value: std::time::Duration) -> u64 {
    value.as_micros().min(u64::MAX as u128) as u64
}
static NEXT_THREAD: AtomicU64 = AtomicU64::new(1);
thread_local! { static THREAD_ID:u64=NEXT_THREAD.fetch_add(1,Ordering::Relaxed); }
fn thread_id() -> u64 {
    THREAD_ID.with(|id| *id)
}
pub struct Span<'a> {
    recorder: Option<&'a Recorder>,
    event: Option<Event>,
    generation: u64,
}
impl Span<'_> {
    pub fn id(&self) -> u64 {
        self.event
            .as_ref()
            .and_then(|e| e.args.get("id"))
            .copied()
            .unwrap_or(0)
    }
    pub fn attribute(&mut self, name: &str, value: u64) {
        if let Some(e) = self.event.as_mut()
            && e.args.len() < 8
        {
            e.args.insert(bounded(name), value);
        }
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        if let (Some(recorder), Some(mut event)) = (self.recorder, self.event.take()) {
            event.dur = micros(recorder.start.elapsed()).saturating_sub(event.ts);
            let overhead = Instant::now();
            tracing::trace!(target:"wabou::perf",operation=%event.name,category=%event.cat,duration_us=event.dur,id=event.args.get("id").copied().unwrap_or(0),"measurement.complete");
            recorder.record(self.generation, event);
            recorder
                .overhead_ns
                .fetch_add(nanos(overhead.elapsed()), Ordering::Relaxed);
        }
    }
}
static GLOBAL: OnceLock<Recorder> = OnceLock::new();
pub fn global() -> &'static Recorder {
    GLOBAL.get_or_init(|| {
        let r = Recorder::new();
        let mode = match std::env::var("WABOU_PROFILE").as_deref() {
            Ok("trace") => Mode::Trace,
            Ok("counters") => Mode::Counters,
            _ => Mode::Off,
        };
        let _ = r.configure(mode, 16384);
        r
    })
}
pub fn span(category: &str, name: &str) -> Span<'static> {
    global().span(category, name)
}
/// Flush an explicitly requested file at resident runtime shutdown (also on error).
pub struct ExportOnDrop;
impl Drop for ExportOnDrop {
    fn drop(&mut self) {
        if let Some(path) = std::env::var_os("WABOU_TRACE_FILE") {
            let result = serde_json::to_vec(&global().snapshot())
                .map_err(|e| e.to_string())
                .and_then(|bytes| std::fs::write(path, bytes).map_err(|e| e.to_string()));
            if let Err(error) = result {
                tracing::error!(%error,"performance trace export failed");
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_trace_and_summary() {
        let r = Recorder::new();
        r.configure(Mode::Trace, 2).unwrap();
        for _ in 0..5 {
            drop(r.span("frame", "tick"));
        }
        let s = r.snapshot();
        assert_eq!(s["traceEvents"].as_array().unwrap().len(), 2);
        assert_eq!(s["droppedEvents"], 3);
        assert_eq!(s["summaries"][0]["count"], 5);
        assert_eq!(s["traceEvents"][0]["ph"], "X");
    }
    #[test]
    fn modes_and_old_generation() {
        let r = Recorder::new();
        drop(r.span("a", "b"));
        assert!(r.snapshot()["summaries"].as_array().unwrap().is_empty());
        r.configure(Mode::Counters, 2).unwrap();
        let old = r.span("a", "old");
        r.configure(Mode::Counters, 2).unwrap();
        drop(old);
        drop(r.span("a", "new"));
        assert_eq!(r.snapshot()["summaries"].as_array().unwrap().len(), 1);
        assert!(r.snapshot()["traceEvents"].as_array().unwrap().is_empty());
        assert!(r.configure(Mode::Trace, 0).is_err());
    }
}

#[cfg(test)]
mod contracts {
    use super::*;
    #[test]
    fn async_measurements_use_async_tracks_and_batch_is_bounded() {
        let r = Recorder::new();
        r.configure(Mode::Trace, 8).unwrap();
        r.batch(
            r#"[{"name":"load","durationUs":10,"ageUs":0,"correlation":42,"asynchronous":true}]"#,
        )
        .unwrap();
        let s = r.snapshot();
        let events = s["traceEvents"].as_array().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["ph"], "b");
        assert_eq!(events[1]["ph"], "e");
        assert_eq!(events[0]["id"], events[1]["id"]);
        assert_eq!(events[0]["args"]["correlation"], 42);
        assert!(r.batch(&" ".repeat(65537)).is_err());
        let input = serde_json::to_string(&vec![json!({"name":"x","durationUs":1,"ageUs":0}); 129])
            .unwrap();
        assert!(r.batch(&input).is_err());
        assert!(
            r.batch(r#"[{"name":"x","durationUs":-1,"ageUs":0}]"#)
                .is_err()
        );
    }
    #[test]
    fn summary_labels_and_sample_storage_are_bounded() {
        let r = Recorder::new();
        r.configure(Mode::Counters, 8).unwrap();
        for _ in 0..200 {
            drop(r.span("a", "b"));
        }
        assert_eq!(r.snapshot()["summaries"][0]["sampleCount"], 128);
        for i in 0..300 {
            drop(r.span("a", &format!("label{i}")));
        }
        let s = r.snapshot();
        assert_eq!(s["summaries"].as_array().unwrap().len(), 256);
        assert_eq!(s["droppedEvents"], 45);
    }
}
