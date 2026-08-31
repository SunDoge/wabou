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

use rquickjs::{
    AsyncContext, AsyncRuntime, CatchResultExt, Ctx, Function, Object, TypedArray,
    context::EvalOptions,
};
pub(crate) use wabou_host_api::LayoutRect;
use wabou_host_api::{FrameStats as HostFrameStats, LayoutScrollMetrics, NodeKey};
type JsResult<T> = rquickjs::Result<T>;
pub(crate) type ResizeTargets = Rc<RefCell<HashMap<NodeKey, Option<(f32, f32)>>>>;

fn checked_node_key(lo: u32, hi: u32, boundary: &'static str) -> JsResult<NodeKey> {
    let key = NodeKey::new(lo, hi);
    if key.is_valid() {
        Ok(key)
    } else {
        Err(rquickjs::Error::new_from_js_message(
            boundary,
            "NodeKey",
            format!("invalid node key {lo}v{hi}"),
        ))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LayoutMetric {
    pub rect: LayoutRect,
    pub clip: LayoutRect,
    pub scroll: LayoutScrollMetrics,
}

#[derive(Debug, Default)]
pub(crate) struct LayoutMetricsSnapshot {
    pub revision: u64,
    pub viewport: LayoutRect,
    pub nodes: HashMap<NodeKey, LayoutMetric>,
}

use crate::atom::AtomPool;
use crate::host_frame::{HostEvent, encode_host_frame};
use crate::style_ir::{ColorThemes, StylesheetUpdate};
use wabou_shell::FrameStats;

const CORE_PRELUDE: &str = include_str!("gen/core-prelude.js");
// Solid 2's universal renderer mounts nested JSX synchronously. A realistic
// desktop page can consume more than QuickJS's previous 2 MiB C-stack budget
// without containing a reactive cycle. Keep this below the usual 8 MiB native
// main-thread stack so QuickJS still raises a catchable RangeError first.
/// Default maximum native stack reserved for QuickJS evaluation.
pub const DEFAULT_QUICKJS_STACK_SIZE: usize = 6 * 1024 * 1024;
const LAYOUT_SNAPSHOT_HEADER_LEN: usize = 8;
const LAYOUT_SNAPSHOT_NODE_LEN: usize = 14;
const LAYOUT_SNAPSHOT_VERSION: f64 = 1.0;

#[derive(serde::Deserialize, Default)]
struct FetchInit {
    method: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
}

struct FetchResponse {
    status: u16,
    status_text: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl<'js> rquickjs::IntoJs<'js> for FetchResponse {
    fn into_js(self, ctx: &Ctx<'js>) -> JsResult<rquickjs::Value<'js>> {
        let value = Object::new(ctx.clone())?;
        value.set("status", self.status)?;
        value.set("statusText", self.status_text)?;
        let headers = Object::new(ctx.clone())?;
        for (name, value) in self.headers {
            headers.set(name, value)?;
        }
        value.set("headers", headers)?;
        value.set("body", crate::host_ffi::OwnedJsBytes(self.body))?;
        Ok(value.into_value())
    }
}

struct RuntimeWake {
    callback: Mutex<Option<wabou_shell::WakeCallback>>,
    pending: AtomicBool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HostFrameDisposition {
    pub prevented_event_ids: Vec<u32>,
    pub needs_tick: bool,
    pub protocol_frame: Vec<u8>,
}

impl HostFrameDisposition {
    pub fn is_prevented(&self, event_id: u32) -> bool {
        self.prevented_event_ids.contains(&event_id)
    }
}

impl RuntimeWake {
    fn mark_pending(&self) {
        self.pending.store(true, Ordering::Release);
    }

    fn notify(&self) {
        self.mark_pending();
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

/// Construction options shared by application and compatibility-test runtimes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JsRuntimeOptions {
    max_stack_size: usize,
}

impl Default for JsRuntimeOptions {
    fn default() -> Self {
        Self {
            max_stack_size: DEFAULT_QUICKJS_STACK_SIZE,
        }
    }
}

impl JsRuntimeOptions {
    /// Set the maximum native stack available to QuickJS, in bytes.
    ///
    /// Larger library bundles and deeply nested Solid trees may need more than
    /// the default. The value must be non-zero and should remain below the
    /// native thread stack so QuickJS can raise a catchable range error first.
    pub fn max_stack_size(mut self, bytes: usize) -> Self {
        assert!(bytes > 0, "QuickJS stack size must be greater than zero");
        self.max_stack_size = bytes;
        self
    }

    /// Return the configured maximum native stack size in bytes.
    pub const fn stack_size(&self) -> usize {
        self.max_stack_size
    }
}

/// Single-threaded QuickJS runtime configured with Wabou's host ABI.
///
/// The runtime may be created directly for bundle compatibility tests. Normal
/// applications should use [`crate::HostBuilder`] and let [`crate::Applier`]
/// own its frame/event lifecycle.
pub struct JsRuntime {
    clock: Arc<dyn crate::clock::Clock>,
    max_stack_size: usize,
    /// Bytes flushed by the most recent `__wabou_flush` call.
    out: Rc<RefCell<Vec<u8>>>,
    /// True once the app's initial render has been evaluated.
    booted: bool,
    source_map: Rc<RefCell<Option<crate::source_map::StackSourceMap>>>,
    /// Stylesheet pushed through `__wabou_set_stylesheet` (JSON parsed by host
    /// into the atomic-CSS dict). Drained by the Applier in build_frame.
    pending_css: Rc<RefCell<Option<StylesheetUpdate>>>,
    /// Latest explicit window color-theme request from JavaScript.
    pending_color_theme: Rc<RefCell<Option<String>>>,
    /// Compiled palettes used by the JS animation primitive as immutable
    /// endpoints. Token order is derived deterministically by the host.
    color_themes: Rc<RefCell<Option<ColorThemes>>>,
    /// Latest JS-interpolated palette. One complete palette is committed per
    /// frame so paint never observes a mixture of animation steps.
    pending_color_palette: Rc<RefCell<Option<Vec<u32>>>>,
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
    #[cfg(any(feature = "devtools", test))]
    debug_state: Rc<RefCell<Option<wabou_devtools::SharedDebugState>>>,
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
    /// Create an empty runtime with the system monotonic clock.
    pub fn new() -> JsResult<Self> {
        Self::new_with_options(JsRuntimeOptions::default())
    }

    /// Create an empty runtime with explicit resource limits.
    pub fn new_with_options(options: JsRuntimeOptions) -> JsResult<Self> {
        Self::new_with_clock_and_options(Arc::new(crate::clock::SystemClock::new()), options)
    }

    pub(crate) fn new_with_clock(clock: Arc<dyn crate::clock::Clock>) -> JsResult<Self> {
        Self::new_with_clock_and_options(clock, JsRuntimeOptions::default())
    }

    fn new_with_clock_and_options(
        clock: Arc<dyn crate::clock::Clock>,
        options: JsRuntimeOptions,
    ) -> JsResult<Self> {
        let rt = AsyncRuntime::new()?;
        futures_lite::future::block_on(async {
            rt.set_max_stack_size(options.stack_size()).await;
        });
        Self::build_inner(rt, clock, options.stack_size())
    }

    fn build_inner(
        rt: AsyncRuntime,
        clock: Arc<dyn crate::clock::Clock>,
        max_stack_size: usize,
    ) -> JsResult<Self> {
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
        let source_map = Rc::new(RefCell::new(None));
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
            max_stack_size,
            clock,
            rt,
            ctx,
            out,
            booted: false,
            source_map,
            pending_css: Rc::new(RefCell::new(None)),
            pending_color_theme: Rc::new(RefCell::new(None)),
            color_themes: Rc::new(RefCell::new(None)),
            pending_color_palette: Rc::new(RefCell::new(None)),
            pending_fonts: Rc::new(RefCell::new(Vec::new())),
            frame_stats: Rc::new(RefCell::new(None)),
            layout_metrics: Rc::new(RefCell::new(LayoutMetricsSnapshot::default())),
            atoms: Rc::new(RefCell::new(AtomPool::default())),
            host_frame_sequence: 0,
            host_frame_epoch: std::time::Instant::now(),
            #[cfg(any(feature = "devtools", test))]
            debug_state: Rc::new(RefCell::new(None)),
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
            let source_map = self.source_map.clone();
            globals.set(
                "__wabou_log",
                rquickjs::Function::new(ctx.clone(), move |tag: String, msg: String| {
                    let mapped = source_map
                        .borrow()
                        .as_ref()
                        .map_or_else(|| msg.clone(), |map| map.map_stack(&msg));
                    crate::host_ffi::host_log(tag, mapped);
                })?
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
            // through @wabou/core's typed Host context.
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

            self.register_intl_host_fns(ctx.clone(), &globals)?;

            // Read a raw font file (TTF/OTF) and queue it for the GPUI text
            // system on the next frame. Call once at boot before first paint.
            // Returns false (and warns) if the file can't be read.
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

            self.register_observability_host_fns(ctx.clone(), &globals)?;

            self.register_style_host_fns(ctx.clone(), &globals)?;

            let targets = self.resize_targets.clone();
            globals.set(
                "__wabou_resize_observe",
                rquickjs::Function::new(ctx.clone(), move |lo: u32, hi: u32| -> JsResult<()> {
                    let key = checked_node_key(lo, hi, "resize observer")?;
                    targets.borrow_mut().entry(key).or_insert(None);
                    Ok(())
                })?
                .with_name("__wabou_resize_observe")?,
            )?;
            let targets = self.resize_targets.clone();
            globals.set(
                "__wabou_resize_unobserve",
                rquickjs::Function::new(ctx.clone(), move |lo: u32, hi: u32| -> JsResult<()> {
                    let key = checked_node_key(lo, hi, "resize observer")?;
                    targets.borrow_mut().remove(&key);
                    Ok(())
                })?
                .with_name("__wabou_resize_unobserve")?,
            )?;
            Ok(())
        })
    }

    fn register_intl_host_fns<'js>(
        &self,
        ctx: rquickjs::Ctx<'js>,
        globals: &rquickjs::Object<'js>,
    ) -> JsResult<()> {
        globals.set(
            "__wabou_system_locale",
            rquickjs::Function::new(ctx.clone(), crate::intl::system_locale)?
                .with_name("__wabou_system_locale")?,
        )?;
        globals.set(
            "__wabou_system_time_zone",
            rquickjs::Function::new(ctx.clone(), crate::intl::system_time_zone)?
                .with_name("__wabou_system_time_zone")?,
        )?;
        globals.set(
            "__wabou_system_calendar_date",
            rquickjs::Function::new(ctx.clone(), || -> String {
                serde_json::to_string(&crate::intl::system_calendar_date())
                    .expect("calendar date serialization is infallible")
            })?
            .with_name("__wabou_system_calendar_date")?,
        )?;
        Ok(())
    }

    fn register_style_host_fns<'js>(
        &self,
        ctx: rquickjs::Ctx<'js>,
        globals: &rquickjs::Object<'js>,
    ) -> JsResult<()> {
        let pending_css = self.pending_css.clone();
        let available_themes = self.color_themes.clone();
        globals.set(
            "__wabou_set_stylesheet",
            rquickjs::Function::new(ctx.clone(), move |source: String| -> JsResult<()> {
                match serde_json::from_str::<StylesheetUpdate>(&source) {
                    Ok(update) => {
                        let StylesheetUpdate::Ir(sheet) = &update;
                        *available_themes.borrow_mut() = sheet.color_themes.clone();
                        *pending_css.borrow_mut() = Some(update);
                        Ok(())
                    }
                    Err(error) => {
                        tracing::error!(target: "stylesheet", "setStylesheet parse failed: {error}");
                        Err(rquickjs::Error::Unknown)
                    }
                }
            })?
            .with_name("__wabou_set_stylesheet")?,
        )?;

        let pending_theme = self.pending_color_theme.clone();
        let wake = self.runtime_wake.clone();
        globals.set(
            "__wabou_set_color_theme",
            rquickjs::Function::new(ctx.clone(), move |name: String| {
                *pending_theme.borrow_mut() = Some(name);
                wake.notify();
            })?
            .with_name("__wabou_set_color_theme")?,
        )?;

        let available_themes = self.color_themes.clone();
        globals.set(
            "__wabou_get_color_theme_palette",
            rquickjs::Function::new(
                ctx.clone(),
                move |name: String, output: Option<TypedArray<u32>>| -> JsResult<u32> {
                    let themes = available_themes.borrow();
                    let themes = themes.as_ref().ok_or(rquickjs::Error::Unknown)?;
                    let theme = themes.themes.get(&name).ok_or(rquickjs::Error::Unknown)?;
                    let mut tokens = theme.colors.keys().collect::<Vec<_>>();
                    tokens.sort_unstable();
                    let colors = tokens
                        .into_iter()
                        .map(|token| theme.colors[token])
                        .collect::<Vec<_>>();
                    let len = u32::try_from(colors.len()).map_err(|_| rquickjs::Error::Unknown)?;
                    if let Some(output) = output {
                        crate::host_ffi::fill_typed_array(&output, &colors)?;
                    }
                    Ok(len)
                },
            )?
            .with_name("__wabou_get_color_theme_palette")?,
        )?;

        let pending_palette = self.pending_color_palette.clone();
        let wake = self.runtime_wake.clone();
        globals.set(
            "__wabou_set_color_palette",
            rquickjs::Function::new(
                ctx.clone(),
                move |colors: TypedArray<u32>| -> JsResult<()> {
                    let bytes = colors.as_bytes().ok_or(rquickjs::Error::Unknown)?;
                    let (chunks, _) = bytes.as_chunks::<{ std::mem::size_of::<u32>() }>();
                    let values = chunks
                        .iter()
                        .map(|bytes| u32::from_ne_bytes(*bytes))
                        .collect();
                    *pending_palette.borrow_mut() = Some(values);
                    wake.notify();
                    Ok(())
                },
            )?
            .with_name("__wabou_set_color_palette")?,
        )?;
        Ok(())
    }

    fn register_observability_host_fns<'js>(
        &self,
        ctx: rquickjs::Ctx<'js>,
        globals: &rquickjs::Object<'js>,
    ) -> JsResult<()> {
        let stats = self.frame_stats.clone();
        globals.set(
            "__wabou_frame_stats",
            rquickjs::Function::new(ctx.clone(), move || -> String {
                let cell = stats.borrow();
                serde_json::to_string(&cell.as_ref().map(|frame| HostFrameStats {
                    build_frame_ms: frame.build_frame_ms,
                    js_tick_ms: frame.js_tick_ms,
                    scene_ms: frame.scene_ms,
                    present_ms: frame.present_ms,
                    node_count: frame.node_count,
                    viewport_w: frame.viewport_w,
                    viewport_h: frame.viewport_h,
                }))
                .expect("frame stats are serializable")
            })?
            .with_name("__wabou_frame_stats")?,
        )?;

        #[cfg(any(feature = "devtools", test))]
        let debug_state = self.debug_state.clone();
        globals.set(
            "__wabou_set_debug_overlay",
            rquickjs::Function::new(
                ctx.clone(),
                move |layout: bool, clips: bool, hit_target: bool| -> bool {
                    #[cfg(any(feature = "devtools", test))]
                    {
                        let Some(state) = debug_state.borrow().clone() else {
                            return false;
                        };
                        let Ok(mut state) = state.write() else {
                            return false;
                        };
                        let selected_node = state.overlay().selected_node;
                        state.set_overlay(wabou_devtools::DebugOverlay {
                            layout,
                            clips,
                            hit_target,
                            selected_node,
                        });
                        true
                    }
                    #[cfg(not(any(feature = "devtools", test)))]
                    {
                        let _ = (layout, clips, hit_target);
                        false
                    }
                },
            )?
            .with_name("__wabou_set_debug_overlay")?,
        )?;

        #[cfg(any(feature = "devtools", test))]
        let debug_state = self.debug_state.clone();
        globals.set(
            "__wabou_debug_overlay_paint_stats",
            rquickjs::Function::new(ctx.clone(), move || -> String {
                #[cfg(any(feature = "devtools", test))]
                let stats = debug_state
                    .borrow()
                    .as_ref()
                    .and_then(|state| state.read().ok().map(|state| state.overlay_paint()));
                #[cfg(not(any(feature = "devtools", test)))]
                let stats: Option<wabou_host_api::DebugOverlayPaintStats> = None;
                serde_json::to_string(&stats).expect("overlay paint stats are serializable")
            })?
            .with_name("__wabou_debug_overlay_paint_stats")?,
        )?;

        let metrics = self.layout_metrics.clone();
        globals.set(
            "__wabou_layout_snapshot",
            rquickjs::Function::new(
                ctx.clone(),
                move |ids: TypedArray<u32>, output: Option<TypedArray<f64>>| -> JsResult<u32> {
                    let snapshot = metrics.borrow();
                    let requested = ids.as_bytes().map_or(&[][..], |bytes| bytes);
                    let (chunks, remainder) =
                        requested.as_chunks::<{ std::mem::size_of::<u32>() * 2 }>();
                    if !remainder.is_empty() {
                        return Err(rquickjs::Error::new_from_js_message(
                            "layout snapshot",
                            "NodeKey[]",
                            "node key buffer must contain complete lo/hi pairs",
                        ));
                    }
                    let ids = chunks.iter().map(|bytes| {
                        checked_node_key(
                            u32::from_ne_bytes(bytes[0..4].try_into().unwrap()),
                            u32::from_ne_bytes(bytes[4..8].try_into().unwrap()),
                            "layout snapshot",
                        )
                    });
                    let nodes = ids
                        .collect::<JsResult<Vec<_>>>()?
                        .into_iter()
                        .filter_map(|id| snapshot.nodes.get(&id).map(|node| (id, node)))
                        .collect::<Vec<_>>();
                    let required = LAYOUT_SNAPSHOT_HEADER_LEN
                        .checked_add(
                            nodes
                                .len()
                                .checked_mul(LAYOUT_SNAPSHOT_NODE_LEN)
                                .ok_or(rquickjs::Error::Unknown)?,
                        )
                        .ok_or(rquickjs::Error::Unknown)?;
                    let required_u32 =
                        u32::try_from(required).map_err(|_| rquickjs::Error::Unknown)?;
                    let Some(output) = output else {
                        return Ok(required_u32);
                    };
                    if output.len() < required {
                        return Ok(required_u32);
                    }

                    crate::host_ffi::with_typed_array_prefix_mut(&output, required, |packed| {
                        packed[..LAYOUT_SNAPSHOT_HEADER_LEN].copy_from_slice(&[
                            LAYOUT_SNAPSHOT_VERSION,
                            snapshot.revision as u32 as f64,
                            (snapshot.revision >> 32) as u32 as f64,
                            snapshot.viewport.x as f64,
                            snapshot.viewport.y as f64,
                            snapshot.viewport.width as f64,
                            snapshot.viewport.height as f64,
                            nodes.len() as f64,
                        ]);
                        for (index, (id, node)) in nodes.into_iter().enumerate() {
                            let offset =
                                LAYOUT_SNAPSHOT_HEADER_LEN + index * LAYOUT_SNAPSHOT_NODE_LEN;
                            packed[offset..offset + LAYOUT_SNAPSHOT_NODE_LEN].copy_from_slice(&[
                                id.lo as f64,
                                id.hi as f64,
                                node.rect.x as f64,
                                node.rect.y as f64,
                                node.rect.width as f64,
                                node.rect.height as f64,
                                node.clip.x as f64,
                                node.clip.y as f64,
                                node.clip.width as f64,
                                node.clip.height as f64,
                                node.scroll.offset_x as f64,
                                node.scroll.offset_y as f64,
                                node.scroll.range_x as f64,
                                node.scroll.range_y as f64,
                            ]);
                        }
                    })?;
                    Ok(required_u32)
                },
            )?
            .with_name("__wabou_layout_snapshot")?,
        )?;
        globals.set(
            "__wabou_crypto_random",
            rquickjs::Function::new(ctx.clone(), |output: TypedArray<u8>| -> JsResult<()> {
                if output.len() > 65_536 {
                    return Err(rquickjs::Error::new_from_js_message(
                        "crypto.getRandomValues",
                        "integer TypedArray",
                        "requested byte length exceeds 65536",
                    ));
                }
                crate::host_ffi::with_typed_array_prefix_mut(&output, output.len(), |bytes| {
                    getrandom::fill(bytes)
                })?
                .map_err(|error| {
                    rquickjs::Error::new_from_js_message(
                        "system random source",
                        "random bytes",
                        error.to_string(),
                    )
                })
            })?
            .with_name("__wabou_crypto_random")?,
        )?;
        globals.set(
            "__wabou_crypto_digest",
            rquickjs::Function::new(
                ctx.clone(),
                rquickjs::prelude::Async(|algorithm: u8, input: TypedArray<u8>| -> _ {
                    let bytes = input.as_bytes().map(<[u8]>::to_vec);
                    async move {
                        let bytes = bytes.ok_or(rquickjs::Error::Unknown)?;
                        tokio::task::spawn_blocking(move || digest_bytes(algorithm, &bytes))
                            .await
                            .map_err(|_| rquickjs::Error::Unknown)?
                            .map(crate::host_ffi::OwnedJsBytes)
                    }
                }),
            )?
            .with_name("__wabou_crypto_digest")?,
        )?;
        Ok(())
    }

    /// Register the rquickjs-native async `__wabou_fetch(url, initJson)`. rquickjs
    /// owns the Rust Future -> JavaScript Promise conversion; the shell only
    /// supplies a waker that reconnects its scheduler to the native event loop.
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

    pub(crate) fn set_wake_callback(&self, callback: wabou_shell::WakeCallback) {
        if let Ok(mut wake) = self.runtime_wake.callback.lock() {
            *wake = Some(callback.clone());
        }
        // Bundle evaluation can enqueue QuickJS scheduler work before the
        // native view installs its wake channel. That internal queue does not
        // necessarily touch `RuntimeWake::pending`, so installing the callback
        // is itself an edge: schedule one coalesced initial pump unconditionally.
        callback();
    }

    /// Poll rquickjs's scheduler once with a waker backed by the native host. Pending
    /// network IO parks naturally; Tokio calls this waker when it can progress.
    /// Ready jobs are time-sliced so a burst of fetch completions cannot drain
    /// an entire Promise/Solid update graph inside one window callback. Returns
    /// whether JavaScript made observable progress or another slice is pending,
    /// so the shell can schedule a frame even when the queue became idle during
    /// this poll.
    pub fn poll_async_runtime(&self) -> bool {
        self.runtime_wake.pending.store(false, Ordering::Release);
        let _guard = self._tokio.handle().enter();
        let waker = Waker::from(self.runtime_wake.clone());
        let mut task_context = TaskContext::from_waker(&waker);
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1);
        let mut progressed = false;
        for _ in 0..32 {
            let mut next = Box::pin(self.rt.execute_pending_job());
            match Pin::new(&mut next).poll(&mut task_context) {
                Poll::Ready(Ok(true)) => progressed = true,
                Poll::Ready(Ok(false)) | Poll::Pending => {
                    return progressed || self.runtime_wake.pending.load(Ordering::Acquire);
                }
                Poll::Ready(Err(error)) => {
                    tracing::warn!(?error, "async JavaScript job failed");
                    return progressed || self.runtime_wake.pending.load(Ordering::Acquire);
                }
            }
            if std::time::Instant::now() >= deadline {
                // We are already on the UI thread. Calling the event-loop
                // proxy here can create a self-wake storm that starves macOS
                // RedrawRequested events. The FrameSource return value and
                // pending bit are sufficient to schedule the next frame.
                self.runtime_wake.mark_pending();
                return true;
            }
        }

        // The fixed job budget was exhausted. Continue on a fresh event-loop
        // turn even if no IO waker fires between now and then. Do not invoke
        // the proxy from inside its own callback; the shell observes this bit.
        self.runtime_wake.mark_pending();
        true
    }

    pub(crate) fn take_async_wake(&self) -> bool {
        self.runtime_wake.pending.swap(false, Ordering::AcqRel)
    }

    /// A handle to the pending-stylesheet cell; the Applier drains it in
    /// `build_frame` and, on update, replaces its css dict + re-resolves.
    pub(crate) fn pending_css_handle(&self) -> Rc<RefCell<Option<StylesheetUpdate>>> {
        self.pending_css.clone()
    }

    pub(crate) fn pending_color_theme_handle(&self) -> Rc<RefCell<Option<String>>> {
        self.pending_color_theme.clone()
    }

    pub(crate) fn pending_color_palette_handle(&self) -> Rc<RefCell<Option<Vec<u32>>>> {
        self.pending_color_palette.clone()
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

    pub(crate) fn resize_targets_handle(&self) -> ResizeTargets {
        self.resize_targets.clone()
    }

    pub(crate) fn atom_pool_handle(&self) -> Rc<RefCell<AtomPool>> {
        self.atoms.clone()
    }

    pub(crate) fn tokio_handle(&self) -> tokio::runtime::Handle {
        self._tokio.handle().clone()
    }

    #[cfg(any(feature = "devtools", test))]
    pub(crate) fn set_debug_state(&mut self, state: wabou_devtools::SharedDebugState) {
        *self.debug_state.borrow_mut() = Some(state);
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
        if !wabou_bindgen::is_contract_identifier(name) {
            return Err(rquickjs::Error::new_into_js_message(
                "Rust capability name",
                "JavaScript identifier",
                format!("invalid capability identifier `{name}`"),
            ));
        }
        let name = name.to_owned();
        self.with(move |ctx| {
            let root: Object = ctx.globals().get("__wabou_capabilities")?;
            if root.contains_key(name.as_str())? {
                return Err(rquickjs::Exception::throw_type(
                    &ctx,
                    &format!("duplicate capability namespace `{name}`"),
                ));
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
        self.boot_with_source_map(source, None)
    }

    pub(crate) fn boot_with_source_map(
        &mut self,
        source: &str,
        source_map: Option<&[u8]>,
    ) -> JsResult<()> {
        if self.booted {
            return Ok(());
        }
        *self.source_map.borrow_mut() =
            source_map.and_then(crate::source_map::StackSourceMap::parse);
        let src = source.to_string();
        let mapped_source_map = self.source_map.clone();
        let max_stack_size = self.max_stack_size;
        self.with(|ctx| -> JsResult<()> {
            let mut options = EvalOptions::default();
            options.filename = Some("bundle.js".to_owned());
            ctx.eval_with_options::<(), _>(src.as_str(), options)
                .catch(&ctx)
                .map_err(|caught| {
                    let mut diagnostic = match caught {
                        rquickjs::CaughtError::Value(v) => {
                            v
                                .as_string()
                                .map(|s| s.to_string().unwrap_or_default())
                                .unwrap_or_else(|| "JavaScript threw a non-string value".into())
                        }
                        rquickjs::CaughtError::Exception(e) => {
                            let message = e
                                .message()
                                .unwrap_or_else(|| "JavaScript exception".to_owned());
                            e.stack().map_or(message.clone(), |stack| {
                                let mapped = mapped_source_map
                                    .borrow()
                                    .as_ref()
                                    .map_or_else(|| stack.clone(), |map| map.map_stack(&stack));
                                format!("{message}\n{mapped}")
                            })
                        }
                        rquickjs::CaughtError::Error(e) => e.to_string(),
                    };
                    if diagnostic.to_ascii_lowercase().contains("stack") {
                        diagnostic.push_str(&format!(
                            "\nQuickJS stack limit: {} bytes. This limit does not enlarge the native thread stack; the native stack must be larger than this value.",
                            max_stack_size
                        ));
                    }
                    eprintln!("boot app failed:\n{diagnostic}");
                    rquickjs::Error::new_from_js_message(
                        "JavaScript bundle",
                        "Wabou runtime",
                        diagnostic,
                    )
                })?;
            Ok(())
        })?;
        self.booted = true;
        Ok(())
    }

    /// Evaluate an additional bundled script in the already-booted runtime.
    /// Used by the opt-in test host; production applications never call it.
    pub fn eval_script(&self, source: &str) -> JsResult<()> {
        let source = source.to_owned();
        self.with(move |ctx| ctx.eval::<(), _>(source.as_str()))
    }

    /// Evaluate an additional script and preserve guest exception details.
    pub fn eval_script_diagnostic(&self, source: &str) -> Result<(), String> {
        let source = source.to_owned();
        let source_map = self.source_map.clone();
        self.with(move |ctx| {
            ctx.eval::<(), _>(source.as_str())
                .catch(&ctx)
                .map_err(|caught| match caught {
                    rquickjs::CaughtError::Exception(exception) => {
                        let message = exception
                            .message()
                            .unwrap_or_else(|| "JavaScript exception".to_owned());
                        let stack = exception.stack().unwrap_or_default();
                        let stack = source_map
                            .borrow()
                            .as_ref()
                            .map_or_else(|| stack.clone(), |map| map.map_stack(&stack));
                        format!("{message}\n{stack}")
                    }
                    rquickjs::CaughtError::Value(value) => value
                        .as_string()
                        .and_then(|value| value.to_string().ok())
                        .unwrap_or_else(|| "JavaScript threw a non-string value".to_owned()),
                    rquickjs::CaughtError::Error(error) => error.to_string(),
                })
        })
    }

    /// Evaluate an expression and return its string result.
    ///
    /// Test harnesses use this narrow boundary for runtime-owned metadata;
    /// application capabilities should continue using typed FFI or JSON DTOs.
    pub fn eval_string(&self, source: &str) -> JsResult<String> {
        let source = source.to_owned();
        self.with(move |ctx| ctx.eval::<String, _>(source.as_str()))
    }

    /// Evaluate a promise-producing expression and return its JSON result.
    ///
    /// This is the lightweight compatibility-test boundary for pure JavaScript
    /// libraries. Call [`Self::boot`] with a browser-targeted bundle first,
    /// then evaluate the library operation without creating an [`crate::Applier`]
    /// or any layout/rendering state.
    pub fn eval_promise_json(
        &self,
        expression: &str,
        timeout: std::time::Duration,
    ) -> Result<String, String> {
        const STATE: &str = "__wabou_compat_state";
        const VALUE: &str = "__wabou_compat_value";
        let expression = format!(
            r#"
            globalThis.{STATE} = "pending";
            globalThis.{VALUE} = null;
            Promise.resolve(({expression})).then(
              value => {{
                globalThis.{VALUE} = value === undefined ? "null" : JSON.stringify(value);
                globalThis.{STATE} = "resolved";
              }},
              error => {{
                globalThis.{VALUE} = `${{String(error?.name ?? "Error")}}: ${{String(error?.message ?? error)}}\n${{String(error?.stack ?? "")}}`;
                globalThis.{STATE} = "rejected";
              }},
            );
            "#,
        );
        self.eval_script_diagnostic(&expression)?;
        let deadline = std::time::Instant::now() + timeout;
        loop {
            self.poll_async_runtime();
            let state = self
                .eval_string(&format!("globalThis.{STATE}"))
                .map_err(|error| error.to_string())?;
            match state.as_str() {
                "resolved" => {
                    return self
                        .eval_string(&format!("globalThis.{VALUE}"))
                        .map_err(|error| error.to_string());
                }
                "rejected" => {
                    let error = self
                        .eval_string(&format!("globalThis.{VALUE}"))
                        .unwrap_or_else(|_| "JavaScript promise rejected".into());
                    return Err(error);
                }
                _ if std::time::Instant::now() >= deadline => {
                    return Err(format!(
                        "JavaScript compatibility probe timed out after {} ms",
                        timeout.as_millis()
                    ));
                }
                _ => std::thread::yield_now(),
            }
        }
    }

    /// Run one rAF tick: drains the JS requestAnimationFrame queue (which makes
    /// Solid reactive updates emit ops into the writer), then flushes the
    /// writer — which calls `__wabou_flush` and lands the bytes in `self.out`.
    /// Returns the frame bytes (empty if nothing changed this tick) and whether
    /// more rAF callbacks remain queued (so the host can keep redrawing).
    pub fn tick(&mut self) -> JsResult<(Vec<u8>, bool)> {
        let frame_time = self.clock.now_ms();
        let has_raf = self.with(|ctx| -> JsResult<bool> {
            let tick: Function = ctx.globals().get("__wabou_tick")?;
            tick.call::<(f64,), bool>((frame_time,))
        })?;
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
        #[cfg(any(feature = "devtools", test))]
        if let Some(state) = self.debug_state.borrow().as_ref()
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
            let protocol_frame = result
                .get::<_, TypedArray<u8>>("protocolFrame")
                .ok()
                .map(|values| AsRef::<[u8]>::as_ref(&values).to_vec())
                .unwrap_or_default();
            Ok(HostFrameDisposition {
                prevented_event_ids,
                needs_tick,
                protocol_frame,
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
        Self::new_vite_with_options(server_url, JsRuntimeOptions::default())
    }

    /// Create a Vite-backed runtime with explicit resource limits.
    #[cfg(feature = "vite")]
    pub fn new_vite_with_options(server_url: &str, options: JsRuntimeOptions) -> JsResult<Self> {
        let origin = url::Url::parse(server_url).map_err(|_| rquickjs::Error::Unknown)?;
        let rt = AsyncRuntime::new()?;
        futures_lite::future::block_on(async {
            // Vite evaluates an ESM graph instead of one bundled module, so
            // linking can require at least as much native stack as production
            // evaluation. Keep both runtime creation paths on the same limit.
            rt.set_max_stack_size(options.stack_size()).await;
        });
        let vite = crate::vite::ViteState::new(origin);
        vite.install_loader(&rt)?;

        let mut this = Self::build_inner(
            rt,
            Arc::new(crate::clock::SystemClock::new()),
            options.stack_size(),
        )?;
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
) -> JsResult<FetchResponse> {
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
        .bytes()
        .await
        .map_err(|_| rquickjs::Error::Unknown)?
        .to_vec();
    Ok(FetchResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or("").to_owned(),
        headers,
        body,
    })
}

fn digest_bytes(algorithm: u8, bytes: &[u8]) -> JsResult<Vec<u8>> {
    use sha1::Digest as _;

    let digest = match algorithm {
        1 => sha1::Sha1::digest(bytes).to_vec(),
        2 => sha2::Sha256::digest(bytes).to_vec(),
        3 => sha2::Sha384::digest(bytes).to_vec(),
        4 => sha2::Sha512::digest(bytes).to_vec(),
        _ => {
            return Err(rquickjs::Error::new_from_js_message(
                "crypto.subtle.digest",
                "SHA-1, SHA-256, SHA-384, or SHA-512",
                format!("unknown digest algorithm id {algorithm}"),
            ));
        }
    };
    Ok(digest)
}

#[cfg(test)]
#[path = "jsrt/tests.rs"]
mod tests;
