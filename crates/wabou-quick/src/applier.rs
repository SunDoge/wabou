//! Apply decoded protocol ops to a retained taffy tree.
//!
//! Replaces blitz-dom: the applier owns a `TaffyTree<Paint>` + a `solidId →
//! NodeId` map + per-node declared state (class list + inline styles + text).
//! Each frame, `build_frame` runs one rAF tick (`js.tick()` → flushed bytes),
//! decodes + applies the ops, then reuses wabou-shell's layout pass.
//!
//! Keys are generational on both sides (solid u32 = JS-side gen<<20|slot;
//! taffy NodeId = slotmap DefaultKey), so `HashMap` is safe — slotmap would be
//! a redundant second generational layer. Reclamation is explicit on `DropNode`
//! (deterministic, sweep-driven) to keep frames recordable/replayable.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

use parley::{
    Affinity, Layout,
    editing::{Cursor, Selection},
};
#[cfg(test)]
use taffy::TraversePartialTree;
use taffy::{NodeId, TaffyTree};
use vello::Scene;
use vello::kurbo::{Affine, Point, Rect, Stroke};
use vello::peniko::{Color, Fill};
use wabou_shell::layout::{self, PlacedNode, SubtreeEvent, subtree_events};
use wabou_shell::scrollbar::{
    ScrollAxis, ScrollbarPart, ScrollbarTarget, drag_ratio as scrollbar_drag_ratio,
    hit as scrollbar_hit,
};
use wabou_shell::style::{
    self, DeclaredPaint, HostPaint, InheritedPaint, IrValue, OverlayPlane, Paint, PaintTransform,
    ScrollbarStyle, ScrollbarVisibility, TextAlign,
};
use wabou_shell::text::{TextContext, layout_text_styled};
use wabou_shell::{
    EventResponse, FrameSource, FrameStats, KeyPhase, Modifiers, PointerButton, PointerPhase,
    SemanticAction, SemanticNode, SemanticRole, SemanticSnapshot, UiEvent, WakeCallback,
};

use crate::host_frame::{HostEvent, HostNodeEvent, NodeEventPayload, ResizeObservation};

mod debug_projection;
mod focus;
mod frame_source;
mod input_router;
mod interaction;
mod node_store;
mod projections;
mod protocol_apply;
mod runtime_updates;
mod semantics;
mod style_resolution;
mod text_selection;
mod widget_bridge;
mod widget_manager;
use crate::host_msg::{DEFAULT_HOST_MSG_CAPACITY, HostMsgHandle, HostMsgInbox, host_msg_channel};
use crate::inline_context::{
    InlineFormattingContext, NodeFacts, rect_has_nonzero_lp, rect_has_nonzero_lpa, size_is_explicit,
};
use crate::jsrt::{JsRuntime, LayoutMetric, LayoutMetricsSnapshot, LayoutRect, ResizeTargets};
use crate::protocol::{Frame, Op, decode_frame};
use crate::protocol::{event, event_data};
use crate::style_ir::{self, StyleSheet, StylesheetUpdate};
use crate::widget::builtin_factories;
use crate::{Atom, AtomPool};
#[cfg(test)]
use input_router::EventMask;
use input_router::{HitClip, HitItem, HitNode, InputRouter, hit_contains};
use node_store::NodeStore;
use projections::FrameProjections;
use widget_manager::WidgetManager;

const SCROLLBAR_FADE_DELAY: Duration = Duration::from_millis(500);
const SCROLLBAR_FADE_DURATION: Duration = Duration::from_millis(200);
// Widget actions retain their tagged 32-bit namespace. Native effects use a
// process-wide sequence so window resource handles stay unique across runtimes.
const JS_HOST_ACTION_NAMESPACE: u64 = 1 << 31;
const HOST_ACTION_SEQUENCE_MASK: u64 = JS_HOST_ACTION_NAMESPACE - 1;
static NEXT_EFFECT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1 << 32);
const CLASS_RESOLUTION_CACHE_CAPACITY: usize = 1024;

#[derive(Clone)]
struct CachedClassResolution {
    declarations: Vec<(String, wabou_shell::style::IrValue)>,
    diagnostics: Vec<String>,
}

fn key_event_payload(key: &wabou_shell::KeyEvent) -> String {
    serde_json::json!({
        "key": key.key,
        "keyWithoutModifiers": key.key_without_modifiers,
        "code": key.code,
        "location": key.location.dom_code(),
        "mods": key.modifiers.bits(),
        "primary": key.modifiers.primary_shortcut(),
        "repeat": key.repeat,
    })
    .to_string()
}

#[derive(Clone)]
enum InlineValue {
    Text(Arc<str>),
    Typed(IrValue),
}

#[derive(Clone)]
struct ScrollbarHit {
    node: NodeId,
    placed: PlacedNode,
    transform: Affine,
}

#[derive(Clone, Copy)]
struct ScrollbarDrag {
    node: NodeId,
    axis: ScrollAxis,
    last_position: f64,
}

impl InlineValue {
    fn ir(&self) -> IrValue {
        match self {
            Self::Text(value) => style::parse_ir_value(value),
            Self::Typed(value) => value.clone(),
        }
    }
}

const CLICK_DRAG_THRESHOLD_SQUARED: f64 = 16.0;

/// Per-node declared state (what the Solid reconciler wrote + cascaded CSS).
///
/// `paint` is the cascade output ([`DeclaredPaint`]): inherited fields stay
/// `Option` so inherit can distinguish "not declared" from an explicit value.
/// Fully resolved layout/render state lives on the taffy node as [`Paint`].
#[derive(Default, Clone)]
struct Declared {
    tag: Option<Atom>,
    classes: Vec<Atom>,
    inline: HashMap<Atom, InlineValue>,
    attrs: HashMap<Atom, Arc<str>>,
    text: Option<Arc<str>>,
    /// Cascaded paint declarations (pre-inherit).
    paint: DeclaredPaint,
    /// Distinguishes authored `display:flex` from Taffy's flex default.
    display_explicit: bool,
}

/// A Vite HMR signal forwarded from the background HMR client to the applier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReloadMsg {
    HmrUpdate {
        path: String,
        accepted_path: String,
        timestamp: u64,
        source: String,
    },
    /// Native Vite CSS channel. Wabou styles flow through
    /// `virtual:wabou-stylesheet` → `__wabou_set_stylesheet` (Style IR) instead;
    /// these messages are acknowledged and logged, not applied as CSSOM.
    CssUpdate {
        path: String,
        source: String,
    },
    FullReload,
}

/// Result of draining the HMR queue for one frame (for tests / diagnostics).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HmrDrainResult {
    Idle,
    /// One or more JS modules were accepted; Style IR may also have updated
    /// via `pending_css` in the same frame.
    Applied {
        js_updates: usize,
    },
    /// Entry was (or should be) fully re-imported.
    FullReload {
        reason: String,
    },
}

/// Coalesce a burst of websocket messages into one ordered batch.
///
/// Order within a frame:
/// 1. `FullReload` wins over partial updates
/// 2. Native `CssUpdate` is recorded only for diagnostics (Style IR is separate)
/// 3. JS `HmrUpdate` list preserves arrival order
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct HmrBatch {
    full_reload: bool,
    full_reload_reason: Option<String>,
    js_updates: Vec<HmrJsUpdate>,
    css_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HmrJsUpdate {
    path: String,
    accepted_path: String,
    timestamp: u64,
    source: String,
}

fn plan_hmr_batch(msgs: impl IntoIterator<Item = ReloadMsg>) -> HmrBatch {
    let mut batch = HmrBatch::default();
    for msg in msgs {
        match msg {
            ReloadMsg::FullReload => {
                batch.full_reload = true;
                batch.full_reload_reason = Some("vite full-reload payload".to_string());
            }
            ReloadMsg::HmrUpdate {
                path,
                accepted_path,
                timestamp,
                source,
            } => {
                batch.js_updates.push(HmrJsUpdate {
                    path,
                    accepted_path,
                    timestamp,
                    source,
                });
            }
            ReloadMsg::CssUpdate { path, .. } => {
                batch.css_paths.push(path);
            }
        }
    }
    batch
}

/// Immutable view of a node's resolved layout and paint state.
///
/// This is intended for semantic renderer tests: assertions can stop at the
/// computed-style boundary without depending on Taffy's mutable tree or a GPU
/// pixel comparison. Native scene/widget handles are deliberately excluded.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputedNodeSnapshot {
    pub solid_id: u32,
    pub classes: Vec<String>,
    pub layout: taffy::Style,
    pub background: Option<Color>,
    pub opacity: f32,
    pub transforms: Vec<PaintTransform>,
    pub shadows: Vec<wabou_shell::style::Shadow>,
    pub border_radius: f32,
    pub border: Option<(f32, Color)>,
    pub text_color: Color,
    pub font_size: f32,
    pub font_weight: f32,
    pub line_height: Option<(f32, bool)>,
    pub wrap_text: bool,
    pub text_ellipsis: bool,
    pub text_selectable: bool,
    pub text_select_all: bool,
    pub text_align: TextAlign,
    pub pointer_events: bool,
    pub z_index: i32,
    pub font_family: Option<Arc<str>>,
    pub intrinsic_size: Option<[f32; 2]>,
    pub runtime_transform: Option<[f32; 6]>,
}

/// Sendable handle the HMR client holds to push [`ReloadMsg`]s into the applier.
#[derive(Clone)]
pub struct ReloadHandle {
    tx: mpsc::Sender<ReloadMsg>,
    pending: Arc<AtomicBool>,
}

#[derive(Clone)]
struct SelectableText {
    text: Arc<str>,
    layout: Arc<Layout<[u8; 4]>>,
    origin: [f32; 2],
    visual_y: std::ops::Range<f32>,
    select_all: bool,
    order: usize,
}

#[derive(Clone)]
struct ActiveTextSelection {
    anchor_target: u32,
    focus_target: u32,
    base_selection: Selection,
    focus_selection: Selection,
    granularity: TextSelectionGranularity,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TextSelectionSnapshot {
    text: Option<String>,
    kind: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum TextSelectionGranularity {
    #[default]
    Cluster,
    Word,
    Line,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct InvalidationFlags: u8 {
        const LAYOUT = 1 << 0;
        const INHERIT = 1 << 1;
        const TICK = 1 << 2;
    }
}

impl ReloadHandle {
    pub fn send(&self, message: ReloadMsg) -> Result<(), mpsc::SendError<ReloadMsg>> {
        self.tx.send(message)?;
        // Wake the render loop: the applier's `has_anim` ORs this flag so
        // `about_to_wait` requests a redraw even when the app is otherwise idle.
        self.pending.store(true, Ordering::Release);
        Ok(())
    }
}

/// CSS properties that inherit to descendants. A SetStyle touching one of
/// these (or the `font` shorthand) must take the slow path — re-derive + run
/// the inherit pass — so children see the new value. Other inline properties
/// take [`Applier::apply_inline_fast`].
const INHERITED_PROPERTIES: &[&str] = &[
    "color",
    "font-size",
    "font-weight",
    "font-family",
    "line-height",
    "text-align",
    "white-space",
    "user-select",
];

pub struct Applier {
    js: JsRuntime,
    node_store: NodeStore,
    style_ir: Option<StyleSheet>,
    active_color_theme: Option<String>,
    active_theme_colors: Arc<HashMap<String, u32>>,
    /// Theme embedded in Style IR, shared by build-time resolution and the
    /// runtime fallback for classes created after compilation.
    style_theme: wabou_style::Theme,
    /// `class atom → indices` into `style_ir.rules`, built when the sheet
    /// arrives so per-node matching is O(C) (the node's classes) instead of
    /// O(R) (all rules). Universal (`*`) rules live in [`universal_rules`]
    /// since they match every node unconditionally.
    rule_index: HashMap<Atom, Vec<usize>>,
    universal_rules: Vec<usize>,
    /// Runtime utility fallback cache. Each interned class is parsed at most
    /// once; build-time stylesheet rules bypass this map entirely.
    utility_cache: HashMap<Atom, Result<wabou_style::ParsedUtility, String>>,
    /// Ordered class atoms → flattened, priority-sorted declarations.
    class_resolution_cache: HashMap<Vec<Atom>, Arc<CachedClassResolution>>,
    #[cfg(test)]
    class_resolution_cache_hits: usize,
    warned_utility_classes: HashSet<Atom>,
    warned_ir_properties: HashSet<Atom>,
    /// Rejections from the latest cascade pass, keyed by native node for
    /// DevTools inspection.
    style_diagnostics: HashMap<NodeId, Vec<String>>,
    /// Serialized source + parsed Vello fragment for each inline `<svg>` root.
    /// Source comparison makes attribute/child changes self-invalidating while
    /// keeping parsing out of the per-frame paint path.
    svg_cache: HashMap<NodeId, (Arc<str>, Arc<wabou_shell::svg::SvgImage>)>,
    /// Explicit host-driven transform state, independent of the CSS cascade.
    runtime_transforms: HashMap<NodeId, [f32; 6]>,
    /// Explicit host stacking planes, independent from CSS cascade/z-index.
    overlay_planes: HashMap<NodeId, OverlayPlane>,
    scrollbar_styles: HashMap<NodeId, ScrollbarStyle>,
    base_color: Color,
    atoms: Rc<RefCell<AtomPool>>,
    input: InputRouter,
    /// Last tick's `has_raf` — gates the continuous-redraw loop.
    has_raf: bool,
    /// Receives Vite HMR signals from the background websocket client.
    reload_rx: Option<mpsc::Receiver<ReloadMsg>>,
    /// Set by [`ReloadHandle::send`] to wake the render loop for HMR drain.
    has_hmr_pending: Arc<AtomicBool>,
    /// Stylesheet pushed through the private host ABI;
    /// drained in build_frame → replaces `css` + re-resolves every node.
    pending_css: Option<Rc<RefCell<Option<StylesheetUpdate>>>>,
    pending_color_theme: Option<Rc<RefCell<Option<String>>>>,
    pending_color_palette: Option<Rc<RefCell<Option<Vec<u32>>>>>,
    /// Font file bytes pushed by the typed Host API (via
    /// `JsRuntime::pending_fonts_handle`); drained in build_frame → registered
    /// into the shared text `FontContext` (cache cleared).
    pending_fonts: Option<Rc<RefCell<Vec<Vec<u8>>>>>,
    /// Latest per-frame render-stage timings (EMA), written by
    /// `push_frame_stats` and read by the Host diagnostics API.
    frame_stats: Option<Rc<RefCell<Option<FrameStats>>>>,
    projections: FrameProjections,
    resize_targets: ResizeTargets,
    /// Main-thread invalidation causes. `INHERIT` gates the O(N) cascade pass,
    /// while non-inherited animation can request only `LAYOUT`.
    invalidation: InvalidationFlags,
    /// EMA of `js.tick()` duration (the QuickJS portion of build_frame),
    /// folded into the FrameStats pushed to the host overlay.
    js_tick_ema: f64,
    /// Last frame's logical viewport (width, height) — exposed via
    /// Host diagnostics so the app can self-size / bounce within bounds.
    last_viewport: (u32, u32),
    device_scale: f64,
    ime_cursor_area: Option<[f64; 4]>,
    selectable_text: HashMap<u32, SelectableText>,
    selectable_text_order: Vec<u32>,
    active_text_selection: Option<ActiveTextSelection>,
    last_text_selection: TextSelectionSnapshot,
    text_selection_event_target: Option<u32>,
    last_text_click: Option<(Instant, u32, f64, f64, u8)>,
    next_text_selection_scroll: Option<Instant>,
    placed_rects: HashMap<NodeId, [f32; 4]>,
    scrollbar_hits: Vec<ScrollbarHit>,
    scrollbar_drag: Option<ScrollbarDrag>,
    hovered_scrollbar: Option<(NodeId, ScrollAxis)>,
    scrollbar_activity: HashMap<NodeId, Instant>,
    widget_manager: WidgetManager,
    pending_host_actions: Rc<RefCell<VecDeque<wabou_shell::HostAction>>>,
    pending_effects: Rc<RefCell<VecDeque<wabou_shell::EffectRequest>>>,
    pending_js_effects: Rc<RefCell<HashSet<u64>>>,
    effect_trace: Rc<RefCell<Option<crate::effect_trace::EffectTrace>>>,
    replay_completions: Rc<RefCell<VecDeque<wabou_shell::EffectCompletion>>>,
    host_action_wake: Rc<RefCell<Option<WakeCallback>>>,
    wake_callback: Option<WakeCallback>,
    scroll_offsets: HashMap<NodeId, [f32; 2]>,
    /// Protocol frames commonly create a node and then set several properties
    /// on it. Resolve style once at FrameEnd instead of after every operation.
    batching_styles: bool,
    dirty_styles: HashSet<NodeId>,
    /// Taffy layout and inherited paint are retained across scroll-only frames.
    layout_viewport: Option<(u32, u32)>,
    /// Vite entry module path (e.g. `packages/index.tsx`) for in-process full
    /// reload when solid-refresh declines an update. Empty outside vite mode.
    vite_entry: Option<String>,
    /// Last HMR drain outcome (diagnostics / tests).
    last_hmr_result: HmrDrainResult,
    /// Bounded host→JS message inbox. Producers use [`HostMsgHandle`].
    host_msg_inbox: HostMsgInbox,
    host_msg_handle: HostMsgHandle,
}

fn install_effect_functions(
    js: &JsRuntime,
    window_id: u64,
    effects: Rc<RefCell<VecDeque<wabou_shell::EffectRequest>>>,
    action_wake: Rc<RefCell<Option<WakeCallback>>>,
    pending: Rc<RefCell<HashSet<u64>>>,
    trace: Rc<RefCell<Option<crate::effect_trace::EffectTrace>>>,
    replay_completions: Rc<RefCell<VecDeque<wabou_shell::EffectCompletion>>>,
) {
    js.with(|ctx| -> rquickjs::Result<()> {
        let submit_effects = effects.clone();
        let submit_wake = action_wake.clone();
        let submit_pending = pending.clone();
        ctx.globals().set(
            "__wabou_effect_submit",
            rquickjs::Function::new(
                ctx.clone(),
                move |capability: u32, method: u16, payload_json: String| -> u64 {
                    let id = NEXT_EFFECT_ID.fetch_add(1, Ordering::Relaxed);
                    let op = wabou_shell::EffectOp::new(capability, method);
                    let parse =
                        |message: String| wabou_shell::EffectPayload::Invalid { op, message };
                    let payload = match op {
                        wabou_shell::effect::builtin::CLIPBOARD_READ => {
                            wabou_shell::EffectPayload::ClipboardRead
                        }
                        wabou_shell::effect::builtin::CLIPBOARD_WRITE => {
                            #[derive(serde::Deserialize)]
                            struct Request {
                                text: String,
                            }
                            serde_json::from_str::<Request>(&payload_json)
                                .map(|request| wabou_shell::EffectPayload::ClipboardWrite {
                                    text: request.text,
                                })
                                .unwrap_or_else(|error| parse(error.to_string()))
                        }
                        wabou_shell::effect::builtin::WINDOW_CREATE => {
                            let value: serde_json::Value =
                                serde_json::from_str(&payload_json).unwrap_or_default();
                            let mut options = wabou_shell::WindowOptions::new();
                            if let Some(title) = value.get("title").and_then(|value| value.as_str())
                            {
                                options = options.title(title);
                            }
                            options = options.initial_inner_size(
                                value
                                    .get("width")
                                    .and_then(|value| value.as_u64())
                                    .unwrap_or(800) as u32,
                                value
                                    .get("height")
                                    .and_then(|value| value.as_u64())
                                    .unwrap_or(600) as u32,
                            );
                            if let Some(resizable) =
                                value.get("resizable").and_then(|value| value.as_bool())
                            {
                                options = options.resizable(resizable);
                            }
                            if let Some(transparent) =
                                value.get("transparent").and_then(|value| value.as_bool())
                            {
                                options = options.transparent(transparent);
                            }
                            if let (Some(width), Some(height)) = (
                                value.get("minWidth").and_then(|value| value.as_u64()),
                                value.get("minHeight").and_then(|value| value.as_u64()),
                            ) {
                                options = options.min_inner_size(width as u32, height as u32);
                            }
                            wabou_shell::EffectPayload::WindowCreate(
                                wabou_shell::effect::WindowCreateRequest {
                                    window_id: id,
                                    options,
                                },
                            )
                        }
                        wabou_shell::effect::builtin::WINDOW_CLOSE
                        | wabou_shell::effect::builtin::WINDOW_SET_MAXIMIZED
                        | wabou_shell::effect::builtin::WINDOW_SET_TITLE => {
                            let value: serde_json::Value =
                                serde_json::from_str(&payload_json).unwrap_or_default();
                            let target = value
                                .get("windowId")
                                .and_then(|value| value.as_u64())
                                .unwrap_or(window_id);
                            let command = if op == wabou_shell::effect::builtin::WINDOW_CLOSE {
                                wabou_shell::WindowCommand::Close
                            } else if op == wabou_shell::effect::builtin::WINDOW_SET_MAXIMIZED {
                                wabou_shell::WindowCommand::SetMaximized(
                                    value
                                        .get("value")
                                        .and_then(|value| value.as_bool())
                                        .unwrap_or(false),
                                )
                            } else {
                                wabou_shell::WindowCommand::SetTitle(
                                    value
                                        .get("title")
                                        .and_then(|value| value.as_str())
                                        .unwrap_or_default()
                                        .to_owned(),
                                )
                            };
                            wabou_shell::EffectPayload::WindowControl {
                                window_id: target,
                                command,
                            }
                        }
                        wabou_shell::effect::builtin::CONTEXT_MENU_SHOW => {
                            serde_json::from_str::<wabou_shell::ContextMenuRequest>(&payload_json)
                                .map(wabou_shell::EffectPayload::ContextMenuShow)
                                .unwrap_or_else(|error| parse(error.to_string()))
                        }
                        _ => wabou_shell::EffectPayload::Extension {
                            op,
                            bytes: payload_json.into_bytes(),
                        },
                    };
                    submit_pending.borrow_mut().insert(id);
                    let request = wabou_shell::EffectRequest {
                        id: wabou_shell::EffectId(id),
                        scope: wabou_shell::EffectScope::Window(window_id),
                        payload,
                    };
                    let submission = trace.borrow().as_ref().map(|trace| trace.submit(&request));
                    match submission {
                        Some(crate::effect_trace::TraceSubmission::Replay(completions)) => {
                            replay_completions.borrow_mut().extend(completions);
                        }
                        Some(crate::effect_trace::TraceSubmission::Live) | None => {
                            submit_effects.borrow_mut().push_back(request);
                        }
                    }
                    if let Some(wake) = submit_wake.borrow().as_ref() {
                        wake();
                    }
                    id
                },
            )?,
        )?;
        ctx.globals()
            .set("__wabou_effect_abi", wabou_shell::EFFECT_ABI_VERSION)?;
        ctx.globals().set("__wabou_window_id", window_id)?;
        Ok(())
    })
    .expect("install effect host functions");
}

fn complete_js_effect(js: &JsRuntime, completion: &wabou_shell::EffectCompletion) {
    let (status, payload) = match &completion.result {
        wabou_shell::EffectResult::Unit => (0_u8, "null".to_owned()),
        wabou_shell::EffectResult::ClipboardText(text) => (
            0,
            serde_json::to_string(text).unwrap_or_else(|_| "null".into()),
        ),
        wabou_shell::EffectResult::ContextMenuSelection(selection) => (
            0,
            serde_json::to_string(selection).unwrap_or_else(|_| "null".into()),
        ),
        wabou_shell::EffectResult::Cancelled => (1, "null".to_owned()),
        wabou_shell::EffectResult::Error { code, message } => (
            2,
            serde_json::json!({ "code": code, "message": message }).to_string(),
        ),
    };
    let result = js.with(|ctx| -> rquickjs::Result<()> {
        let callback: rquickjs::Function = ctx.globals().get("__wabou_effect_complete")?;
        callback.call::<_, ()>((
            completion.id.0,
            completion.op.capability.0,
            completion.op.method.0,
            status,
            payload,
        ))
    });
    if let Err(error) = result {
        tracing::warn!(
            ?error,
            effect_id = completion.id.0,
            "effect completion callback failed"
        );
    }
}

impl Applier {
    /// Build an applier over an already-booted [`JsRuntime`] (the host owns
    /// boot: `JsRuntime::new().boot(js)` for the static-bundle path, or
    /// `JsRuntime::new_vite(url).boot_vite(entry)` for dev mode).
    pub fn from_runtime(js: JsRuntime, base_color: Color) -> Self {
        Self::from_runtime_with_factories(js, builtin_factories(), base_color)
    }

    /// Like `from_runtime` but with a widget factory registry (from `HostBuilder`).
    pub fn from_runtime_with_factories(
        js: JsRuntime,
        widget_factories: HashMap<String, crate::widget::WidgetFactory>,
        base_color: Color,
    ) -> Self {
        Self::from_runtime_with_factories_and_window(js, widget_factories, base_color, 1)
    }

    pub fn from_runtime_with_factories_and_window(
        js: JsRuntime,
        widget_factories: HashMap<String, crate::widget::WidgetFactory>,
        base_color: Color,
        window_id: u64,
    ) -> Self {
        let pending_css = js.pending_css_handle();
        let pending_color_theme = js.pending_color_theme_handle();
        let pending_color_palette = js.pending_color_palette_handle();
        let pending_fonts = js.pending_fonts_handle();
        let frame_stats = js.frame_stats_handle();
        let layout_metrics = js.layout_metrics_handle();
        let atoms = js.atom_pool_handle();
        let resize_targets = js.resize_targets_handle();
        // Intern the factory tag strings (from `HostBuilder`'s user-facing
        // `String` API) into the atom pool so `create_widget` can look up by
        // the `Atom` carried in `Op::CreateElement` with no per-op allocation.
        let widget_factories: HashMap<Atom, _> = widget_factories
            .into_iter()
            .map(|(k, v)| (atoms.borrow_mut().intern(&k), v))
            .collect();
        let (host_msg_handle, host_msg_inbox) = host_msg_channel(DEFAULT_HOST_MSG_CAPACITY);

        let pending_host_actions = Rc::new(RefCell::new(VecDeque::new()));
        let host_action_wake = Rc::new(RefCell::new(None));
        let pending_js_effects = Rc::new(RefCell::new(HashSet::new()));
        let pending_effects = Rc::new(RefCell::new(VecDeque::new()));
        let effect_trace = Rc::new(RefCell::new(None));
        let replay_completions = Rc::new(RefCell::new(VecDeque::new()));
        install_effect_functions(
            &js,
            window_id,
            pending_effects.clone(),
            host_action_wake.clone(),
            pending_js_effects.clone(),
            effect_trace.clone(),
            replay_completions.clone(),
        );
        Self {
            js,
            node_store: NodeStore::new(),
            style_ir: None,
            active_color_theme: None,
            active_theme_colors: Arc::new(HashMap::new()),
            style_theme: wabou_style::Theme::default(),
            rule_index: HashMap::new(),
            universal_rules: Vec::new(),
            utility_cache: HashMap::new(),
            class_resolution_cache: HashMap::new(),
            #[cfg(test)]
            class_resolution_cache_hits: 0,
            warned_utility_classes: HashSet::new(),
            warned_ir_properties: HashSet::new(),
            style_diagnostics: HashMap::new(),
            svg_cache: HashMap::new(),
            runtime_transforms: HashMap::new(),
            overlay_planes: HashMap::new(),
            scrollbar_styles: HashMap::new(),
            base_color,
            atoms,
            input: InputRouter::new(),
            has_raf: true,
            reload_rx: None,
            has_hmr_pending: Arc::new(AtomicBool::new(false)),
            pending_css: Some(pending_css),
            pending_color_theme: Some(pending_color_theme),
            pending_color_palette: Some(pending_color_palette),
            pending_fonts: Some(pending_fonts),
            frame_stats: Some(frame_stats),
            projections: FrameProjections::new(layout_metrics),
            resize_targets,
            invalidation: InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT,
            js_tick_ema: 0.0,
            last_viewport: (0, 0),
            device_scale: 1.0,
            ime_cursor_area: None,
            selectable_text: HashMap::new(),
            selectable_text_order: Vec::new(),
            active_text_selection: None,
            last_text_selection: TextSelectionSnapshot::default(),
            text_selection_event_target: None,
            last_text_click: None,
            next_text_selection_scroll: None,
            placed_rects: HashMap::new(),
            scrollbar_hits: Vec::new(),
            scrollbar_drag: None,
            hovered_scrollbar: None,
            scrollbar_activity: HashMap::new(),
            widget_manager: WidgetManager::new(widget_factories),
            pending_host_actions,
            pending_effects,
            pending_js_effects,
            effect_trace,
            replay_completions,
            host_action_wake,
            wake_callback: None,
            scroll_offsets: HashMap::new(),
            batching_styles: false,
            dirty_styles: HashSet::new(),
            layout_viewport: None,
            vite_entry: None,
            last_hmr_result: HmrDrainResult::Idle,
            host_msg_inbox,
            host_msg_handle,
        }
    }

    /// Boot the application after all Applier-owned host bridges have been
    /// installed. This ordering permits window APIs during initial render.
    pub fn boot(&mut self, source: &str) -> rquickjs::Result<()> {
        self.js.boot(source)
    }

    #[cfg(feature = "vite")]
    pub fn boot_vite(&mut self, entry: &str) -> rquickjs::Result<()> {
        self.js.boot_vite(entry)
    }

    pub(crate) fn set_effect_trace(&mut self, trace: crate::effect_trace::EffectTrace) {
        *self.effect_trace.borrow_mut() = Some(trace);
    }

    pub fn set_debug_state(&mut self, state: wabou_devtools::SharedDebugState) {
        self.js.set_debug_state(state.clone());
        self.projections.debug_state = Some(state);
        self.projections.debug_dirty = true;
    }

    /// Cloneable handle for background tasks / streams to push application
    /// messages toward JS (`host.subscribe` on the guest side).
    pub fn host_msg_handle(&self) -> HostMsgHandle {
        self.host_msg_handle.clone()
    }

    /// Snapshot the currently resolved style for a Solid node id.
    ///
    /// Call this after the relevant op frame/build tick. It performs no style
    /// recomputation and cannot mutate renderer state.
    pub fn computed_node_snapshot(&self, solid_id: u32) -> Option<ComputedNodeSnapshot> {
        let &node = self.node_store.solid_to_node.get(&solid_id)?;
        let paint = self.node_store.tree.get_node_context(node)?;
        let declared = self.node_store.declared.get(&node)?;
        let atoms = self.atoms.borrow();
        Some(ComputedNodeSnapshot {
            solid_id,
            classes: declared
                .classes
                .iter()
                .filter_map(|class| atoms.resolve(*class).map(str::to_owned))
                .collect(),
            layout: self.node_store.tree.style(node).ok()?.clone(),
            background: paint.background,
            opacity: paint.opacity,
            transforms: paint.transform.clone(),
            shadows: paint.shadows.clone(),
            border_radius: paint.border_radius,
            border: paint.border,
            text_color: paint.text_color,
            font_size: paint.font_size,
            font_weight: paint.font_weight,
            line_height: paint.line_height,
            wrap_text: paint.wrap_text,
            text_ellipsis: paint.text_ellipsis,
            text_selectable: paint.text_selectable,
            text_select_all: paint.text_select_all,
            text_align: paint.text_align,
            pointer_events: paint.pointer_events,
            z_index: paint.z_index,
            font_family: paint.font_family.clone(),
            intrinsic_size: paint.intrinsic_size,
            runtime_transform: self.runtime_transforms.get(&node).copied(),
        })
    }
}

impl Applier {
    fn cancel_pointer_gesture(&mut self, pointer: wabou_shell::PointerEvent) -> bool {
        self.input.pointer_position = (pointer.position.x, pointer.position.y);
        self.input.pointer_buttons = pointer.buttons;
        self.next_text_selection_scroll = None;
        if self.input.pointer_down_target.is_some() {
            self.last_text_click = None;
        }
        let old_active = self.input.pointer_down_target.take();
        self.input.pointer_down_position = None;
        self.input.pointer_dragged = false;
        let target =
            old_active.or_else(|| self.input.hit_test(pointer.position.x, pointer.position.y));
        let mut changed = old_active.is_some_and(|captured| {
            self.handle_widget_event(captured, &UiEvent::Pointer(pointer))
                .is_some_and(|response| response.handled || response.request_redraw)
        });
        changed |= target.is_some_and(|target| {
            self.dispatch_pointer(
                target,
                event::POINTERCANCEL,
                pointer.button,
                pointer.modifiers,
            )
        });
        changed |= self.sync_text_selection_change();
        changed
    }

    fn cancel_active_pointer_gesture(&mut self) -> bool {
        if self.input.pointer_down_target.is_none() {
            self.input.pointer_buttons = 0;
            return false;
        }
        self.cancel_pointer_gesture(wabou_shell::PointerEvent {
            phase: PointerPhase::Cancel,
            position: wabou_shell::Point {
                x: self.input.pointer_position.0,
                y: self.input.pointer_position.1,
            },
            button: None,
            buttons: 0,
            modifiers: Modifiers::empty(),
        })
    }
}

/// Layer 2 — cascade / inherit / priority (see `computed_style.rs`).
#[cfg(test)]
#[path = "computed_style.rs"]
mod computed_style;

/// Layer 3 — final layout geometry (see `layout_fixtures.rs`).
#[cfg(test)]
#[path = "layout_fixtures.rs"]
mod layout_fixtures;

#[cfg(test)]
mod tests;
