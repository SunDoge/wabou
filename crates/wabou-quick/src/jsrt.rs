//! QuickJS host (via rquickjs).
//!
//! Owns the runtime + context, injects the host functions the JS side needs
//! (`__wabou_flush`, `__wabou_log`, codec, `performance.now`),
//! loads `core-prelude.js` + the app bundle, and exposes `tick()` which runs
//! the requestAnimationFrame queue for one frame and returns the flushed
//! binary frame bytes + whether more rAF callbacks remain queued.
//!
//! Ported from blitz-js's `jsrt.rs`; fetch is backed by rquickjs's native async
//! bridge and Tokio, with its scheduler waker connected to the shell event loop.

use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context as TaskContext, Poll, Wake, Waker};

use rquickjs::{AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Function, Object, TypedArray};
type JsResult<T> = rquickjs::Result<T>;
pub(crate) type ResizeTargets = Rc<RefCell<HashMap<u32, Option<(f32, f32)>>>>;

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
pub(crate) struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LayoutMetric {
    pub rect: LayoutRect,
    pub clip: LayoutRect,
}

#[derive(Debug, Default)]
pub(crate) struct LayoutMetricsSnapshot {
    pub revision: u64,
    pub viewport: LayoutRect,
    pub nodes: HashMap<u32, LayoutMetric>,
}

#[derive(serde::Serialize)]
struct LayoutMetricResponse {
    id: u32,
    rect: LayoutRect,
    clip: LayoutRect,
}

#[derive(serde::Serialize)]
struct LayoutSnapshotResponse {
    revision: u64,
    viewport: LayoutRect,
    nodes: Vec<LayoutMetricResponse>,
}

use crate::atom::AtomPool;
use crate::host_frame::{HostEvent, encode_host_frame};
use crate::style_ir::StylesheetUpdate;
use wabou_shell::FrameStats;

const CORE_PRELUDE: &str = include_str!("gen/core-prelude.js");

#[derive(serde::Deserialize, Default)]
struct FetchInit {
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

struct RuntimeWake {
    callback: Mutex<Option<wabou_shell::WakeCallback>>,
    pending: AtomicBool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HostFrameDisposition {
    pub prevented_event_ids: Vec<u32>,
    pub needs_tick: bool,
}

impl HostFrameDisposition {
    pub fn is_prevented(&self, event_id: u32) -> bool {
        self.prevented_event_ids.contains(&event_id)
    }
}

impl RuntimeWake {
    fn notify(&self) {
        self.pending.store(true, Ordering::Release);
        if let Some(callback) = self.callback.lock().ok().and_then(|slot| slot.clone()) {
            callback();
        }
    }
}

impl Wake for RuntimeWake {
    fn wake(self: Arc<Self>) {
        self.notify();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.notify();
    }
}

pub struct JsRuntime {
    clock: Arc<dyn crate::Clock>,
    /// Bytes flushed by the most recent `__wabou_flush` call.
    out: Rc<RefCell<Vec<u8>>>,
    /// True once the app's initial render has been evaluated.
    booted: bool,
    /// Stylesheet pushed through `__wabou_set_stylesheet` (JSON parsed by host
    /// into the atomic-CSS dict). Drained by the Applier in build_frame.
    pending_css: Rc<RefCell<Option<StylesheetUpdate>>>,
    /// Font bytes pushed through the typed Host API. Drained by the
    /// Applier in build_frame and registered into the text `FontContext`.
    pending_fonts: Rc<RefCell<Vec<Vec<u8>>>>,
    /// Latest per-frame render-stage timings, written by the Applier each
    /// frame and exposed by `useHost().diagnostics.frameStats()`.
    frame_stats: Rc<RefCell<Option<FrameStats>>>,
    /// Latest completed native layout, exposed as one coherent snapshot to JS.
    layout_metrics: Rc<RefCell<LayoutMetricsSnapshot>>,
    /// Runtime-scoped structural strings. Rust owns ID allocation; JS caches
    /// returned IDs and sends them through the bridge on subsequent uses.
    atoms: Rc<RefCell<AtomPool>>,
    host_frame_sequence: u64,
    host_frame_epoch: std::time::Instant,
    debug_state: Option<wabou_devtools::SharedDebugState>,
    resize_targets: ResizeTargets,
    runtime_wake: Arc<RuntimeWake>,
    ctx: AsyncContext,
    rt: AsyncRuntime,
    /// Tokio runtime + LocalSet that drives rquickjs async jobs. rquickjs runtime
    /// is !Send, so multi-thread tokio + LocalSet; `with()` enters the tokio
    /// context and `block_on`s so async host functions could `.await`.
    _tokio: tokio::runtime::Runtime,
    _local: tokio::task::LocalSet,
    #[cfg(feature = "vite")]
    vite: Option<crate::vite::ViteState>,
}

impl JsRuntime {
    pub fn new() -> JsResult<Self> {
        Self::new_with_clock(Arc::new(crate::SystemClock::new()))
    }

    pub fn new_with_clock(clock: Arc<dyn crate::Clock>) -> JsResult<Self> {
        let rt = AsyncRuntime::new()?;
        futures_lite::future::block_on(async {
            rt.set_max_stack_size(2048 * 1024).await;
        });
        Self::build_inner(rt, clock)
    }

    fn build_inner(rt: AsyncRuntime, clock: Arc<dyn crate::Clock>) -> JsResult<Self> {
        let tokio_rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;
        let local = tokio::task::LocalSet::new();

        let ctx = futures_lite::future::block_on(AsyncContext::full(&rt))?;
        let out: Rc<RefCell<Vec<u8>>> = Rc::new(RefCell::new(Vec::new()));
        let runtime_wake = Arc::new(RuntimeWake {
            callback: Mutex::new(None),
            pending: AtomicBool::new(false),
        });
        let resize_targets = Rc::new(RefCell::new(HashMap::new()));
        futures_lite::future::block_on(ctx.with(|ctx| -> JsResult<()> {
            let globals = ctx.globals();
            globals.set("__wabou_capabilities", Object::new(ctx.clone())?)?;

            // __wabou_flush(Uint8Array) -> copy the frame bytes out.
            let out = out.clone();
            let f = Function::new(ctx.clone(), move |buf: TypedArray<u8>| -> JsResult<()> {
                if let Some(bytes) = buf.as_bytes() {
                    *out.borrow_mut() = bytes.to_vec();
                }
                Ok(())
            })?
            .with_name("__wabou_flush")?;
            globals.set("__wabou_flush", f)?;

            Ok(())
        }))?;

        let this = Self {
            clock,
            rt,
            ctx,
            out,
            booted: false,
            pending_css: Rc::new(RefCell::new(None)),
            pending_fonts: Rc::new(RefCell::new(Vec::new())),
            frame_stats: Rc::new(RefCell::new(None)),
            layout_metrics: Rc::new(RefCell::new(LayoutMetricsSnapshot::default())),
            atoms: Rc::new(RefCell::new(AtomPool::default())),
            host_frame_sequence: 0,
            host_frame_epoch: std::time::Instant::now(),
            debug_state: None,
            resize_targets,
            runtime_wake,
            _tokio: tokio_rt,
            _local: local,
            #[cfg(feature = "vite")]
            vite: None,
        };
        this.register_core_host_fns()?;
        this.register_fetch()?;
        this.register_sleep()?;
        this.install_core_prelude()?;
        Ok(this)
    }

    fn install_core_prelude(&self) -> JsResult<()> {
        self.with(|ctx| ctx.eval::<(), _>(CORE_PRELUDE))
    }

    /// Install the stateless host functions: logging, UTF-8 codec and
    /// `performance.now`. Names are assigned here for useful JS stack traces.
    pub fn register_core_host_fns(&self) -> JsResult<()> {
        self.with(|ctx| -> JsResult<()> {
            let globals = ctx.globals();
            globals.set(
                "__wabou_log",
                rquickjs::Function::new(ctx.clone(), crate::host_ffi::host_log)?
                    .with_name("__wabou_log")?,
            )?;
            globals.set(
                "__wabou_utf8_encode",
                rquickjs::Function::new(ctx.clone(), crate::host_ffi::host_utf8_encode)?
                    .with_name("__wabou_utf8_encode")?,
            )?;
            globals.set(
                "__wabou_utf8_decode",
                rquickjs::Function::new(ctx.clone(), crate::host_ffi::host_utf8_decode)?
                    .with_name("__wabou_utf8_decode")?,
            )?;

            let perf = rquickjs::Object::new(ctx.clone())?;
            let clock = Arc::clone(&self.clock);
            perf.set(
                "now",
                rquickjs::Function::new(ctx.clone(), move || clock.now_ms())?
                    .with_name("now")?,
            )?;
            globals.set("performance", perf)?;

            // Private native ABI. Application code reaches these capabilities
            // through @wabou/solid-renderer's typed Host context.
            let atoms = self.atoms.clone();
            globals.set(
                "__wabou_intern",
                rquickjs::Function::new(ctx.clone(), move |value: String| -> u32 {
                    atoms.borrow_mut().intern(&value).get()
                })?
                .with_name("__wabou_intern")?,
            )?;
            globals.set(
                "__wabou_open_url",
                rquickjs::Function::new(ctx.clone(), move |raw: String| -> bool {
                    let Ok(url) = url::Url::parse(&raw) else {
                        tracing::warn!(url = %raw, "refused invalid external URL");
                        return false;
                    };
                    if !matches!(url.scheme(), "http" | "https") {
                        tracing::warn!(scheme = url.scheme(), "refused external URL scheme");
                        return false;
                    }
                    open::that_detached(url.as_str()).is_ok()
                })?
                .with_name("__wabou_open_url")?,
            )?;

            // Read a raw font file (TTF/OTF) and queue
            // it for the Applier to register into the text FontContext on the
            // next frame. Call once at boot before first paint. Returns false
            // (and warns) if the file can't be read.
            let pf = self.pending_fonts.clone();
            globals.set(
                "__wabou_load_font",
                rquickjs::Function::new(ctx.clone(), move |path: String| -> bool {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            pf.borrow_mut().push(bytes);
                            true
                        }
                        Err(error) => {
                            tracing::warn!(path = %path, %error, "loadFont: failed to read font file");
                            false
                        }
                    }
                })?
                .with_name("__wabou_load_font")?,
            )?;

            // Latest per-frame render-stage timings (EMA)
            // as JSON, or null until the first frame. For a live perf overlay.
            // Built by hand (no serde on FrameStats) to avoid a wabou-shell
            // serde dependency just for diagnostics.
            let stats = self.frame_stats.clone();
            globals.set(
                "__wabou_frame_stats",
                rquickjs::Function::new(ctx.clone(), move || -> String {
                    let cell = stats.borrow();
                    match cell.as_ref() {
                        Some(f) => format!(
                            "{{\"build_frame_ms\":{:.3},\"js_tick_ms\":{:.3},\"scene_ms\":{:.3},\"present_ms\":{:.3},\"node_count\":{},\"viewport_w\":{},\"viewport_h\":{}}}",
                            f.build_frame_ms, f.js_tick_ms, f.scene_ms, f.present_ms, f.node_count, f.viewport_w, f.viewport_h
                        ),
                        None => "null".to_string(),
                    }
                })?
                .with_name("__wabou_frame_stats")?,
            )?;

            // Batch layout measurement. The request stays binary (u32 IDs),
            // while the low-frequency result uses JSON like diagnostics. All
            // records come from the same completed native layout pass.
            let metrics = self.layout_metrics.clone();
            globals.set(
                "__wabou_layout_snapshot",
                rquickjs::Function::new(
                    ctx.clone(),
                    move |ids: TypedArray<u32>| -> JsResult<String> {
                        let snapshot = metrics.borrow();
                        let requested = ids.as_bytes().map_or(&[][..], |bytes| bytes);
                        let ids = requested
                            .chunks_exact(std::mem::size_of::<u32>())
                            .map(|bytes| u32::from_ne_bytes(bytes.try_into().unwrap()));
                        let nodes = ids
                            .filter_map(|id| snapshot.nodes.get(&id).map(|node| (id, node)))
                            .map(|(id, node)| LayoutMetricResponse {
                                id,
                                rect: node.rect,
                                clip: node.clip,
                            })
                            .collect::<Vec<_>>();
                        serde_json::to_string(&LayoutSnapshotResponse {
                            revision: snapshot.revision,
                            viewport: snapshot.viewport,
                            nodes,
                        })
                        .map_err(|error| rquickjs::Error::new_from_js_message(
                            "layout snapshot",
                            "string",
                            error.to_string(),
                        ))
                    },
                )?
                .with_name("__wabou_layout_snapshot")?,
            )?;

            // The style compiler pushes its Style IR through this private ABI.
            // virtual module pushes the compiled Style IR here as a JSON string.
            // The host forwards it to the applier, which swaps the sheet +
            // re-resolves every node.
            let pc = self.pending_css.clone();
            globals.set(
                "__wabou_set_stylesheet",
                rquickjs::Function::new(ctx.clone(), move |s: String| -> JsResult<()> {
                    match serde_json::from_str::<StylesheetUpdate>(&s) {
                        Ok(m) => {
                            *pc.borrow_mut() = Some(m);
                            Ok(())
                        }
                        Err(e) => {
                            tracing::error!(target: "stylesheet", "setStylesheet parse failed: {e}");
                            Err(rquickjs::Error::Unknown)
                        }
                    }
                })?
                .with_name("__wabou_set_stylesheet")?,
            )?;

            let targets = self.resize_targets.clone();
            globals.set(
                "__wabou_resize_observe",
                rquickjs::Function::new(ctx.clone(), move |solid_id: u32| {
                    targets.borrow_mut().entry(solid_id).or_insert(None);
                })?
                .with_name("__wabou_resize_observe")?,
            )?;
            let targets = self.resize_targets.clone();
            globals.set(
                "__wabou_resize_unobserve",
                rquickjs::Function::new(ctx.clone(), move |solid_id: u32| {
                    targets.borrow_mut().remove(&solid_id);
                })?
                .with_name("__wabou_resize_unobserve")?,
            )?;
            Ok(())
        })
    }

    /// Register the rquickjs-native async `__wabou_fetch(url, initJson)`. rquickjs
    /// owns the Rust Future -> JavaScript Promise conversion; the shell only
    /// supplies a waker that reconnects its scheduler to winit.
    fn register_fetch(&self) -> JsResult<()> {
        let client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .map_err(|_| rquickjs::Error::Unknown)?;
        self.with(|ctx| {
            let function = Function::new(
                ctx.clone(),
                rquickjs::prelude::Async(move |url: String, init_json: String| {
                    let client = client.clone();
                    fetch_request(client, url, init_json)
                }),
            )?
            .with_name("__wabou_fetch")?;
            ctx.globals().set("__wabou_fetch", function)
        })
    }

    fn register_sleep(&self) -> JsResult<()> {
        self.with(|ctx| {
            let function = Function::new(
                ctx.clone(),
                rquickjs::prelude::Async(|delay_ms: f64| async move {
                    let delay_ms = if delay_ms.is_finite() {
                        delay_ms.max(0.0)
                    } else {
                        0.0
                    };
                    tokio::time::sleep(std::time::Duration::from_secs_f64(delay_ms / 1000.0)).await;
                    Ok::<(), rquickjs::Error>(())
                }),
            )?
            .with_name("__wabou_sleep")?;
            ctx.globals().set("__wabou_sleep", function)
        })
    }

    pub(crate) fn resize_targets_handle(&self) -> ResizeTargets {
        self.resize_targets.clone()
    }

    pub fn set_wake_callback(&self, callback: wabou_shell::WakeCallback) {
        if let Ok(mut wake) = self.runtime_wake.callback.lock() {
            *wake = Some(callback);
        }
    }

    /// Poll rquickjs's scheduler once with a waker backed by winit. Pending
    /// network IO parks naturally; Tokio calls this waker when it can progress.
    /// Ready jobs are time-sliced so a burst of fetch completions cannot drain
    /// an entire Promise/Solid update graph inside one window callback.
    pub fn poll_async_runtime(&self) -> bool {
        self.runtime_wake.pending.store(false, Ordering::Release);
        let _guard = self._tokio.handle().enter();
        let waker = Waker::from(self.runtime_wake.clone());
        let mut task_context = TaskContext::from_waker(&waker);
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1);
        for _ in 0..32 {
            let mut next = Box::pin(self.rt.execute_pending_job());
            match Pin::new(&mut next).poll(&mut task_context) {
                Poll::Ready(Ok(true)) => {}
                Poll::Ready(Ok(false)) | Poll::Pending => return false,
                Poll::Ready(Err(error)) => {
                    tracing::warn!(?error, "async JavaScript job failed");
                    return false;
                }
            }
            if std::time::Instant::now() >= deadline {
                self.runtime_wake.notify();
                return true;
            }
        }

        // The fixed job budget was exhausted. Continue on a fresh event-loop
        // turn even if no IO waker fires between now and then.
        self.runtime_wake.notify();
        true
    }

    pub fn take_async_wake(&self) -> bool {
        self.runtime_wake.pending.swap(false, Ordering::AcqRel)
    }

    pub fn has_async_wake(&self) -> bool {
        self.runtime_wake.pending.load(Ordering::Acquire)
    }

    /// A handle to the pending-stylesheet cell; the Applier drains it in
    /// `build_frame` and, on update, replaces its css dict + re-resolves.
    pub(crate) fn pending_css_handle(&self) -> Rc<RefCell<Option<StylesheetUpdate>>> {
        self.pending_css.clone()
    }

    /// A handle to the pending-fonts queue; the Applier drains it in
    /// `build_frame` and registers each blob into the text `FontContext`.
    pub(crate) fn pending_fonts_handle(&self) -> Rc<RefCell<Vec<Vec<u8>>>> {
        self.pending_fonts.clone()
    }

    /// A handle to the frame-stats cell; the Applier writes the latest EMA
    /// per-stage timings each frame for the typed Host diagnostics API.
    pub(crate) fn frame_stats_handle(&self) -> Rc<RefCell<Option<FrameStats>>> {
        self.frame_stats.clone()
    }

    pub(crate) fn layout_metrics_handle(&self) -> Rc<RefCell<LayoutMetricsSnapshot>> {
        self.layout_metrics.clone()
    }

    pub fn atom_pool_handle(&self) -> Rc<RefCell<AtomPool>> {
        self.atoms.clone()
    }

    pub fn set_debug_state(&mut self, state: wabou_devtools::SharedDebugState) {
        self.debug_state = Some(state);
    }

    /// Run a synchronous closure while holding the async QuickJS context lock.
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: for<'js> FnOnce(Ctx<'js>) -> R + rquickjs::markers::ParallelSend,
        R: rquickjs::markers::ParallelSend,
    {
        let _guard = self._tokio.handle().enter();
        self._local.block_on(&self._tokio, self.ctx.with(f))
    }

    /// Mount one named Rust capability before the guest bundle boots.
    /// Capability members are deliberately not installed as globals.
    pub fn mount_capability<F>(&self, name: &str, mount: F) -> JsResult<()>
    where
        F: for<'js> FnOnce(Ctx<'js>, Object<'js>) -> JsResult<()> + rquickjs::markers::ParallelSend,
    {
        if name.is_empty()
            || !name
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
        {
            return Err(rquickjs::Error::Unknown);
        }
        let name = name.to_owned();
        self.with(move |ctx| {
            let root: Object = ctx.globals().get("__wabou_capabilities")?;
            if root.contains_key(name.as_str())? {
                return Err(rquickjs::Error::Unknown);
            }
            let capability = Object::new(ctx.clone())?;
            mount(ctx, capability.clone())?;
            root.set(name, capability)
        })
    }

    /// Evaluate the bundled app. The IIFE registers host glue and runs the app's
    /// initial render (emitting ops into the writer, flushed on first tick).
    /// Call once before ticking.
    pub fn boot(&mut self, source: &str) -> JsResult<()> {
        if self.booted {
            return Ok(());
        }
        let src = source.to_string();
        self.with(|ctx| -> JsResult<()> {
            ctx.eval::<(), _>(src.as_str())
                .catch(&ctx)
                .map_err(|caught| {
                    match caught {
                        rquickjs::CaughtError::Value(v) => {
                            let s = v
                                .as_string()
                                .map(|s| s.to_string().unwrap_or_default())
                                .unwrap_or_default();
                            eprintln!("boot app failed (value): {s}");
                        }
                        rquickjs::CaughtError::Exception(e) => {
                            eprintln!("boot app failed (exception): {e:?}");
                            if let Some(msg) = e.message() {
                                eprintln!("  Message: {msg}");
                            }
                            if let Some(stack) = e.stack() {
                                eprintln!("  Stack: {stack}");
                            }
                        }
                        rquickjs::CaughtError::Error(e) => {
                            eprintln!("boot app failed (error): {e:?}")
                        }
                    }
                    rquickjs::Error::Unknown
                })?;
            Ok(())
        })?;
        self.booted = true;
        Ok(())
    }

    /// Run one rAF tick: drains the JS requestAnimationFrame queue (which makes
    /// Solid reactive updates emit ops into the writer), then flushes the
    /// writer — which calls `__wabou_flush` and lands the bytes in `self.out`.
    /// Returns the frame bytes (empty if nothing changed this tick) and whether
    /// more rAF callbacks remain queued (so the host can keep redrawing).
    pub fn tick(&mut self) -> JsResult<(Vec<u8>, bool)> {
        self.out.borrow_mut().clear();
        let frame_time = self.clock.now_ms();
        let has_raf = self.with(|ctx| -> JsResult<bool> {
            let tick: Function = ctx.globals().get("__wabou_tick")?;
            tick.call::<(f64,), bool>((frame_time,))
        })?;
        // Promise continuations share the same fixed/time budget as fetch and
        // timers. Never drain an unbounded microtask graph in one UI callback.
        self.poll_async_runtime();
        let bytes = std::mem::take(&mut *self.out.borrow_mut());
        Ok((bytes, has_raf))
    }

    /// Whether any rAF callbacks are queued on the JS side. The host uses this
    /// to decide whether to keep redrawing (rAF-driven, vsync-aligned).
    pub fn has_raf(&self) -> bool {
        self.with(|ctx| -> bool {
            let f: Function = match ctx.globals().get("__wabou_has_raf") {
                Ok(f) => f,
                Err(_) => return false,
            };
            f.call::<(), bool>(()).unwrap_or(false)
        })
    }

    /// Deliver unsolicited Host facts through the single versioned binary
    /// entry point. Guest-initiated mounted functions return through their own
    /// value/Promise and never use this path.
    pub fn dispatch_host_frame(&mut self, events: &[HostEvent]) -> JsResult<HostFrameDisposition> {
        if events.is_empty() {
            return Ok(HostFrameDisposition::default());
        }
        self.host_frame_sequence = self.host_frame_sequence.wrapping_add(1).max(1);
        let bytes = encode_host_frame(
            self.host_frame_sequence,
            self.host_frame_epoch.elapsed(),
            events,
        )
        .map_err(|_| rquickjs::Error::Unknown)?;
        if let Some(state) = &self.debug_state
            && let Ok(mut state) = state.write()
        {
            state.push_frame(wabou_devtools::DebugFrame {
                direction: "hostToJs".into(),
                sequence: self.host_frame_sequence,
                byte_len: bytes.len(),
                record_count: events.len(),
                bytes_hex: Some(wabou_devtools::bytes_hex(&bytes, 4096)),
            });
        }
        self.with(move |ctx| -> JsResult<HostFrameDisposition> {
            let dispatch: Function = ctx.globals().get("__wabou_dispatch_host_frame")?;
            let arr = TypedArray::<u8>::new(ctx.clone(), bytes)?;
            let result: Object = dispatch.call((arr,))?;
            let needs_tick = result.get::<_, bool>("needsTick").unwrap_or(false);
            let prevented_event_ids = result
                .get::<_, TypedArray<u32>>("preventedEventIds")
                .ok()
                .map(|values| AsRef::<[u32]>::as_ref(&values).to_vec())
                .unwrap_or_default();
            Ok(HostFrameDisposition {
                prevented_event_ids,
                needs_tick,
            })
        })
    }

    /// Drive rquickjs async jobs; returns whether more async work remains.
    /// (Kept for future async host functions; rAF-only apps don't need it.)
    pub fn poll_pending_jobs(&self) -> Result<bool, String> {
        let _guard = self._tokio.handle().enter();
        let pending = self._local.block_on(&self._tokio, async {
            let _ = tokio::time::timeout(std::time::Duration::from_millis(1), self.rt.idle()).await;
            self.rt.is_job_pending().await
        });
        Ok(pending)
    }

    // --- Vite dev-server integration (feature `vite`) -----------------------

    /// Create a runtime whose `import`s fetch ESM from the Vite dev server at
    /// `server_url`. Installs the ViteResolver/ViteLoader + no-op
    /// `__wabou_vite_update_style` / `__wabou_vite_remove_style` stubs (Vite's
    /// client may call them; layout styles use Style IR via
    /// `__wabou_set_stylesheet` instead of CSSOM).
    #[cfg(feature = "vite")]
    pub fn new_vite(server_url: &str) -> JsResult<Self> {
        let origin = url::Url::parse(server_url).map_err(|_| rquickjs::Error::Unknown)?;
        let rt = AsyncRuntime::new()?;
        futures_lite::future::block_on(async {
            rt.set_max_stack_size(2048 * 1024).await;
        });
        let vite = crate::vite::ViteState::new(origin);
        vite.install_loader(&rt)?;

        let mut this = Self::build_inner(rt, Arc::new(crate::SystemClock::new()))?;
        this.with(|ctx| -> JsResult<()> {
            let g = ctx.globals();
            g.set(
                "__wabou_vite_update_style",
                rquickjs::Function::new(ctx.clone(), |_id: String, _css: String| {})?
                    .with_name("__wabou_vite_update_style")?,
            )?;
            g.set(
                "__wabou_vite_remove_style",
                rquickjs::Function::new(ctx.clone(), |_id: String| {})?
                    .with_name("__wabou_vite_remove_style")?,
            )?;
            Ok(())
        })?;
        this.vite = Some(vite);
        Ok(this)
    }

    /// `import` the Vite entry module (the app's `src/index.tsx`), which
    /// transitively loads `/@vite/client` (the HMR client) + the app graph.
    #[cfg(feature = "vite")]
    pub fn boot_vite(&mut self, entry: &str) -> JsResult<()> {
        if self.booted {
            return Ok(());
        }
        let vite = self.vite.as_ref().expect("vite runtime");
        self.with(|ctx| vite.boot(&ctx, entry))?;
        self.booted = true;
        Ok(())
    }

    /// Apply one Vite HMR update: cache the prefetched module source (keyed by
    /// `?t=<timestamp>`) and call the JS `__wabou_apply_hmr` hook to reload it.
    /// Returns whether the JS side accepted (true) or needs a full reload.
    #[cfg(feature = "vite")]
    pub fn apply_hmr_update(
        &mut self,
        path: &str,
        accepted_path: &str,
        timestamp: u64,
        source: String,
    ) -> JsResult<bool> {
        let vite = self.vite.as_ref().expect("vite runtime");
        self.with(|ctx| vite.apply_hmr(&ctx, path, accepted_path, timestamp, source))
    }

    /// In-process full reload: clear the Vite module cache and re-import `entry`
    /// with a cache-busting query. Call after the applier has reset its tree.
    #[cfg(feature = "vite")]
    pub fn reboot_vite_entry(&mut self, entry: &str) -> JsResult<()> {
        let vite = self.vite.as_ref().expect("vite runtime");
        self.with(|ctx| vite.boot_full_reload(&ctx, entry))
    }
}

async fn fetch_request(
    client: reqwest::Client,
    url: String,
    init_json: String,
) -> JsResult<String> {
    let init: FetchInit = serde_json::from_str(&init_json).unwrap_or_default();
    let method = init
        .method
        .as_deref()
        .unwrap_or("GET")
        .parse::<reqwest::Method>()
        .map_err(|_| rquickjs::Error::Unknown)?;
    let mut request = client.request(method, &url);
    if let Some(headers) = init.headers {
        for (name, value) in headers {
            request = request.header(name, value);
        }
    }
    if let Some(body) = init.body {
        request = request.body(body);
    }
    let response = request.send().await.map_err(|_| rquickjs::Error::Unknown)?;
    let status = response.status();
    let headers = response
        .headers()
        .iter()
        .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_owned()))
        .collect::<HashMap<_, _>>();
    let body = response
        .text()
        .await
        .map_err(|_| rquickjs::Error::Unknown)?;
    serde_json::to_string(&serde_json::json!({
        "status": status.as_u16(),
        "statusText": status.canonical_reason().unwrap_or(""),
        "headers": headers,
        "body": body,
    }))
    .map_err(|_| rquickjs::Error::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestClock(std::sync::atomic::AtomicU64);

    impl TestClock {
        fn advance_ms(&self, milliseconds: u64) {
            self.0.fetch_add(milliseconds, Ordering::Relaxed);
        }
    }

    impl crate::Clock for TestClock {
        fn now_ms(&self) -> f64 {
            self.0.load(Ordering::Relaxed) as f64
        }
    }

    #[test]
    fn animation_frame_uses_the_injected_clock_timestamp() {
        const CORE_FIXTURE: &str = include_str!("gen/test-runtime.js");
        let clock = Arc::new(TestClock::default());
        let mut runtime = JsRuntime::new_with_clock(clock.clone()).expect("runtime");
        runtime.boot(CORE_FIXTURE).expect("boot fixture");
        runtime
            .with(|ctx| ctx.eval::<(), _>("requestAnimationFrame(t => globalThis.frameTime = t)"))
            .expect("schedule animation frame");
        clock.advance_ms(125);
        runtime.tick().expect("tick fixture");
        let times = runtime
            .with(|ctx| ctx.eval::<Vec<f64>, _>("[globalThis.frameTime, performance.now()]"))
            .expect("read timestamps");
        assert_eq!(times, [125.0, 125.0]);
    }

    #[test]
    fn javascript_and_rust_share_runtime_atom_ids() {
        let runtime = JsRuntime::new().expect("runtime");
        let ids = runtime
            .with(|ctx| {
                ctx.eval::<Vec<u32>, _>(
                    r#"[__wabou_intern("width"), __wabou_intern("width"), __wabou_intern("height")]"#,
                )
            })
            .expect("intern from JavaScript");

        assert_eq!(ids[0], ids[1]);
        assert_ne!(ids[0], ids[2]);
        let atoms = runtime.atom_pool_handle();
        let atoms = atoms.borrow();
        assert_eq!(atoms.resolve(crate::Atom::from_raw(ids[0])), Some("width"));
        assert_eq!(atoms.resolve(crate::Atom::from_raw(ids[2])), Some("height"));
    }

    #[test]
    fn mounted_capabilities_are_namespaced_and_reject_duplicates() {
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .mount_capability("workspace", |ctx, capability| {
                capability.set(
                    "basename",
                    Function::new(ctx, |path: String| {
                        path.rsplit('/').next().unwrap_or_default().to_owned()
                    })?,
                )
            })
            .expect("mount workspace capability");
        let value = runtime
            .with(|ctx| {
                ctx.eval::<String, _>("__wabou_capabilities.workspace.basename('/tmp/readme.md')")
            })
            .expect("call mounted function");
        assert_eq!(value, "readme.md");
        let leaked = runtime
            .with(|ctx| ctx.eval::<bool, _>("typeof globalThis.basename !== 'undefined'"))
            .expect("inspect globals");
        assert!(!leaked);
        assert!(
            runtime
                .mount_capability("workspace", |_ctx, _capability| Ok(()))
                .is_err()
        );
    }

    #[test]
    fn sleep_uses_rquickjs_async_scheduler_and_wakes_host() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let runtime = JsRuntime::new().expect("runtime");
        let wake_count = Arc::new(AtomicUsize::new(0));
        let callback_count = wake_count.clone();
        runtime.set_wake_callback(Arc::new(move || {
            callback_count.fetch_add(1, Ordering::Release);
        }));
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    "globalThis.sleepDone = false; __wabou_sleep(10).then(() => globalThis.sleepDone = true);",
                )
            })
            .expect("start sleep");
        assert!(!runtime.poll_async_runtime(), "sleep should park");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if runtime.take_async_wake() {
                runtime.poll_async_runtime();
            }
            let done = runtime
                .with(|ctx| ctx.eval::<bool, _>("globalThis.sleepDone"))
                .expect("read sleep state");
            if done {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "sleep timed out");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        // `RuntimeWake::notify` publishes the pending bit before invoking the
        // callback. The test thread can therefore observe and drain the bit,
        // finish the Promise, and reach this assertion while the notifying
        // thread is still between those two operations under heavy load.
        let callback_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while wake_count.load(Ordering::Acquire) == 0
            && std::time::Instant::now() < callback_deadline
        {
            std::thread::yield_now();
        }
        assert!(wake_count.load(Ordering::Acquire) >= 1);
    }

    #[test]
    fn promise_jobs_are_time_sliced() {
        let runtime = JsRuntime::new().expect("runtime");
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"
                    globalThis.jobCount = 0;
                    function nextJob() {
                      globalThis.jobCount++;
                      if (globalThis.jobCount < 1000) Promise.resolve().then(nextJob);
                    }
                    Promise.resolve().then(nextJob);
                    "#,
                )
            })
            .expect("queue promise jobs");

        assert!(
            runtime.poll_async_runtime(),
            "the first slice should exhaust its budget"
        );
        let first = runtime
            .with(|ctx| ctx.eval::<u32, _>("globalThis.jobCount"))
            .expect("read first slice count");
        assert!(first > 0 && first < 1000, "one poll drained {first} jobs");

        while runtime.poll_async_runtime() {}
        let final_count = runtime
            .with(|ctx| ctx.eval::<u32, _>("globalThis.jobCount"))
            .expect("read final job count");
        assert_eq!(final_count, 1000);
    }

    #[test]
    fn fetch_wakes_host_and_resolves_on_js_thread() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let address = listener.local_addr().expect("server address");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0u8; 4096];
            let count = stream.read(&mut request).expect("read request");
            let request = String::from_utf8_lossy(&request[..count]);
            assert!(
                request.starts_with("GET /story HTTP/1.1"),
                "unexpected request: {request}"
            );
            stream
                .write_all(
                    b"HTTP/1.1 201 Created\r\ncontent-type: application/json\r\ncontent-length: 10\r\nconnection: close\r\n\r\n{\"id\": 42}",
                )
                .expect("write response");
        });

        let runtime = JsRuntime::new().expect("runtime");
        let wake_count = Arc::new(AtomicUsize::new(0));
        let wake_count_for_callback = wake_count.clone();
        runtime.set_wake_callback(Arc::new(move || {
            wake_count_for_callback.fetch_add(1, Ordering::Release);
        }));
        runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    format!(
                        "globalThis.fetchResult = null; __wabou_fetch('http://{address}/story', '{{}}').then(value => globalThis.fetchResult = value);"
                    ),
                )
            })
            .expect("start fetch");
        // The first scheduler slice may either reach the socket wait or
        // exhaust its ready-job budget while constructing the request. Both
        // are valid; only eventual wake + resolution is contractual.
        runtime.poll_async_runtime();

        let initial_result = runtime
            .with(|ctx| ctx.eval::<Option<String>, _>("globalThis.fetchResult"))
            .expect("inspect initial fetch result");
        let wake_required = initial_result.is_none();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let result = if let Some(result) = initial_result {
            result
        } else {
            loop {
                if runtime.take_async_wake() {
                    runtime.poll_async_runtime();
                }
                if let Some(result) = runtime
                    .with(|ctx| ctx.eval::<Option<String>, _>("globalThis.fetchResult"))
                    .expect("inspect fetch result")
                {
                    break result;
                }
                assert!(std::time::Instant::now() < deadline, "fetch timed out");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        };
        if wake_required {
            // The pending bit is published before the callback is invoked, so
            // Promise completion can win the race by a few instructions.
            let callback_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
            while wake_count.load(Ordering::Acquire) == 0
                && std::time::Instant::now() < callback_deadline
            {
                std::thread::yield_now();
            }
            assert!(wake_count.load(Ordering::Acquire) >= 1);
        }
        runtime.take_async_wake();
        assert!(!runtime.poll_async_runtime(), "runtime should be idle");

        let result: serde_json::Value = serde_json::from_str(&result).expect("response JSON");
        assert_eq!(result["status"], 201);
        assert_eq!(result["statusText"], "Created");
        assert_eq!(result["body"], "{\"id\": 42}");
        assert_eq!(result["headers"]["content-type"], "application/json");
        server.join().expect("test server");
    }

    #[test]
    fn host_ffi_surface_matches_contract() {
        // Lock the globals installed by a bare JsRuntime. Applier adds the
        // window, clipboard, and renderer-specific half of the host bridge;
        // its contract test compares that complete surface with host.ts.
        let runtime = JsRuntime::new().expect("runtime");
        let mut bridge: Vec<String> = runtime
            .with(|ctx| {
                ctx.eval::<Vec<String>, _>(
                    r#"Object.keys(globalThis).filter(k => k.startsWith("__wabou"))"#,
                )
            })
            .expect("enumerate __wabou_* globals");
        bridge.sort();
        let mut expected = crate::host_abi::HOST_ABI
            .iter()
            .filter(|entry| {
                entry.direction == crate::host_abi::Direction::Host
                    && entry.owner == "runtime"
                    && entry.feature.is_none()
            })
            .map(|entry| entry.name.to_owned())
            .collect::<Vec<_>>();
        expected.sort();
        assert_eq!(bridge, expected, "Rust-registered __wabou_* set drifted");

        let has_legacy_global = runtime
            .with(|ctx| ctx.eval::<bool, _>(r#"typeof globalThis.Wabou !== "undefined""#))
            .expect("check legacy Wabou global");
        assert!(!has_legacy_global, "legacy globalThis.Wabou must not exist");

        // Old names must be gone — a partial revert of the rename resurfaces here.
        let stale: Vec<String> = runtime
            .with(|ctx| {
                ctx.eval::<Vec<String>, _>(
                    r#"
                    [
                      "__host_log", "__host_utf8_encode", "__host_utf8_decode",
                      "__bridge_flush", "__fetch", "__sleep",
                      "__resize_observe", "__resize_unobserve",
                      "__vite_update_style", "__vite_remove_style",
                    ].filter(k => typeof globalThis[k] !== "undefined")
                    "#,
                )
            })
            .expect("check for stale FFI names");
        assert!(stale.is_empty(), "stale FFI names still defined: {stale:?}");
    }

    #[test]
    fn motion_value_animations_run_inside_quickjs() {
        const MOTION_FIXTURE: &str = include_str!("gen/motion-test-runtime.js");
        let clock = Arc::new(TestClock::default());
        let mut runtime = JsRuntime::new_with_clock(clock.clone()).expect("runtime");
        runtime.boot(MOTION_FIXTURE).expect("boot Motion fixture");
        for frame in 0..300 {
            clock.advance_ms(10);
            runtime.tick().expect("drive animation frame");
            runtime.poll_async_runtime();
            if frame == 0 {
                let value = runtime
                    .with(|ctx| ctx.eval::<f64, _>("globalThis.__wabou_motion_result.number"))
                    .expect("read in-flight Motion value");
                assert!(
                    value > 0.0 && value < 100.0,
                    "40ms animation must still be in flight after 10ms: {value}"
                );
            }
            let done = runtime
                .with(|ctx| ctx.eval::<bool, _>("globalThis.__wabou_motion_result.done"))
                .expect("read Motion completion");
            if done {
                break;
            }
            assert!(frame < 299, "Motion fixture timed out");
        }
        let json = runtime
            .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.__wabou_motion_result)"))
            .expect("serialize Motion result");
        let result: serde_json::Value = serde_json::from_str(&json).expect("Motion result JSON");
        assert_eq!(result["number"].as_f64(), Some(100.0));
        assert_eq!(result["keyframe"].as_f64(), Some(10.0));
        assert!(result["spring"].as_f64().is_some_and(|value| value > 0.99));
        assert_eq!(result["cancelled"].as_f64(), Some(0.0));
        assert!(result["color"].as_str().is_some_and(|color| {
            matches!(
                color,
                "#ffffff" | "rgb(255, 255, 255)" | "rgba(255, 255, 255, 1)"
            )
        }));
    }
}
