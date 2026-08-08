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

use std::cell::{Cell, RefCell};
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

mod semantics;
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

const SCROLLBAR_FADE_DELAY: Duration = Duration::from_millis(500);
const SCROLLBAR_FADE_DURATION: Duration = Duration::from_millis(200);
// Keep JS request IDs exactly representable as JavaScript numbers while
// separating them from widget-local host action IDs.
const JS_HOST_ACTION_NAMESPACE: u64 = 1 << 31;
const HOST_ACTION_SEQUENCE_MASK: u64 = JS_HOST_ACTION_NAMESPACE - 1;

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
struct HitClip {
    rect: [f32; 4],
    radius: f32,
    transform: Affine,
}

#[derive(Clone)]
struct HitNode {
    solid_id: u32,
    rect: [f32; 4],
    transform: Affine,
    clips: Vec<HitClip>,
    pointer_events: bool,
}

#[derive(Clone)]
struct ScrollbarHit {
    node: NodeId,
    placed: PlacedNode,
    transform: Affine,
}

#[derive(Clone)]
enum HitItem {
    Content(HitNode),
    Scrollbar(Box<ScrollbarHit>),
}

fn hit_contains(rect: [f32; 4], radius: f32, transform: Affine, point: Point) -> bool {
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let determinant = a * d - b * c;
    if !determinant.is_finite() || determinant.abs() <= f64::EPSILON {
        return false;
    }
    let local = transform.inverse() * point;
    let [x0, y0, x1, y1] = rect.map(f64::from);
    if local.x < x0 || local.y < y0 || local.x >= x1 || local.y >= y1 {
        return false;
    }
    let radius = f64::from(radius).min((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
    if radius <= 0.0
        || (local.x >= x0 + radius && local.x < x1 - radius)
        || (local.y >= y0 + radius && local.y < y1 - radius)
    {
        return true;
    }
    let center_x = if local.x < x0 + radius {
        x0 + radius
    } else {
        x1 - radius
    };
    let center_y = if local.y < y0 + radius {
        y0 + radius
    } else {
        y1 - radius
    };
    (local.x - center_x).powi(2) + (local.y - center_y).powi(2) <= radius.powi(2)
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
static NEXT_WINDOW_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1 << 32);

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct EventMask(u32);

impl EventMask {
    fn bit(code: u8) -> Option<u32> {
        code.checked_sub(1)
            .filter(|bit| u32::from(*bit) < u32::BITS)
            .map(|bit| 1 << bit)
    }

    fn insert(&mut self, code: u8) {
        if let Some(bit) = Self::bit(code) {
            self.0 |= bit;
        }
    }

    fn remove(&mut self, code: u8) {
        if let Some(bit) = Self::bit(code) {
            self.0 &= !bit;
        }
    }

    fn contains(self, code: u8) -> bool {
        Self::bit(code).is_some_and(|bit| self.0 & bit != 0)
    }

    fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn codes(self) -> impl Iterator<Item = u8> {
        (1..=u32::BITS as u8).filter(move |&code| self.contains(code))
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
    tree: TaffyTree<Paint>,
    root: NodeId,
    solid_to_node: HashMap<u32, NodeId>,
    node_to_solid: HashMap<NodeId, u32>,
    /// Logical child order per parent (Solid/DOM tree). Taffy children are a
    /// projection of this via [`InlineFormattingContext`].
    children: HashMap<NodeId, Vec<NodeId>>,
    logical_parent: HashMap<NodeId, NodeId>,
    declared: HashMap<NodeId, Declared>,
    /// Latest IFC build: which parents are Parley leaves and their plain text.
    /// Layout boxes are applied to Taffy; styled runs are filled in inherit.
    collapsed_text: HashMap<NodeId, Arc<str>>,
    inline_roots: HashSet<NodeId>,
    style_ir: Option<StyleSheet>,
    /// `class atom → indices` into `style_ir.rules`, built when the sheet
    /// arrives so per-node matching is O(C) (the node's classes) instead of
    /// O(R) (all rules). Universal (`*`) rules live in [`universal_rules`]
    /// since they match every node unconditionally.
    rule_index: HashMap<Atom, Vec<usize>>,
    universal_rules: Vec<usize>,
    /// Runtime utility fallback cache. Each interned class is parsed at most
    /// once; build-time stylesheet rules bypass this map entirely.
    utility_cache: HashMap<Atom, Result<wabou_style::ParsedUtility, String>>,
    warned_utility_classes: HashSet<Atom>,
    warned_ir_properties: HashSet<Atom>,
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
    /// Listeners keyed by solidId. Presence is also used to avoid crossing the
    /// JS bridge when neither the target nor an ancestor handles an event.
    listeners: HashMap<u32, EventMask>,
    pointer_position: (f64, f64),
    pointer_buttons: u32,
    pointer_down_target: Option<u32>,
    pointer_down_position: Option<(f64, f64)>,
    pointer_dragged: bool,
    next_host_event_id: u32,
    hovered_target: Option<u32>,
    focused_target: Option<u32>,
    window_focused: bool,
    /// Last tick's `has_raf` — gates the continuous-redraw loop.
    has_raf: bool,
    /// Receives Vite HMR signals from the background websocket client.
    reload_rx: Option<mpsc::Receiver<ReloadMsg>>,
    /// Set by [`ReloadHandle::send`] to wake the render loop for HMR drain.
    has_hmr_pending: Arc<AtomicBool>,
    /// Stylesheet pushed through the private host ABI;
    /// drained in build_frame → replaces `css` + re-resolves every node.
    pending_css: Option<Rc<RefCell<Option<StylesheetUpdate>>>>,
    /// Font file bytes pushed by the typed Host API (via
    /// `JsRuntime::pending_fonts_handle`); drained in build_frame → registered
    /// into the shared text `FontContext` (cache cleared).
    pending_fonts: Option<Rc<RefCell<Vec<Vec<u8>>>>>,
    /// Latest per-frame render-stage timings (EMA), written by
    /// `push_frame_stats` and read by the Host diagnostics API.
    frame_stats: Option<Rc<RefCell<Option<FrameStats>>>>,
    layout_metrics: Rc<RefCell<LayoutMetricsSnapshot>>,
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
    selectable_text: HashMap<u32, SelectableText>,
    selectable_text_order: Vec<u32>,
    active_text_selection: Option<ActiveTextSelection>,
    last_text_selection: TextSelectionSnapshot,
    text_selection_event_target: Option<u32>,
    last_text_click: Option<(Instant, u32, f64, f64, u8)>,
    next_text_selection_scroll: Option<Instant>,
    placed_rects: HashMap<NodeId, [f32; 4]>,
    hit_items: Vec<HitItem>,
    scrollbar_hits: Vec<ScrollbarHit>,
    scrollbar_drag: Option<ScrollbarDrag>,
    hovered_scrollbar: Option<(NodeId, ScrollAxis)>,
    scrollbar_activity: HashMap<NodeId, Instant>,
    semantics_enabled: bool,
    semantics_dirty: bool,
    semantic_snapshot: Arc<SemanticSnapshot>,
    /// Rust-side widgets (TextInput, Canvas, …) keyed by taffy NodeId.
    /// Painted every frame after layout; composited by `build_scene`.
    widgets: HashMap<NodeId, Box<dyn crate::widget::Widget>>,
    /// Last resolved content style delivered to each native widget.
    widget_styles: HashMap<NodeId, crate::widget::WidgetStyle>,
    pending_host_actions: Rc<RefCell<VecDeque<wabou_shell::HostAction>>>,
    pending_js_clipboard_requests: Rc<RefCell<HashSet<u64>>>,
    host_action_wake: Rc<RefCell<Option<WakeCallback>>>,
    next_host_action_id: u64,
    host_action_routes: HashMap<u64, (NodeId, u64)>,
    /// Widget factory registry: tag atom → factory. Populated by `HostBuilder`;
    /// keys are interned into [`atoms`](Self::atoms) at construction so lookup
    /// on `CreateElement` is a direct `Atom` hash with no `str::to_owned`.
    widget_factories: HashMap<Atom, crate::widget::WidgetFactory>,
    wake_callback: Option<WakeCallback>,
    scroll_offsets: HashMap<NodeId, [f32; 2]>,
    /// Solid IDs whose widget reported `value_changed` in `handle_event` but
    /// whose `current_value()` isn't fresh yet — pending edits are only applied
    /// in `paint_widgets`. Drained in `build_frame` after paint to read the
    /// updated value, sync the `value` attr, and dispatch `INPUT` to JS.
    pending_value_sync: HashSet<u32>,
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
    debug_state: Option<wabou_devtools::SharedDebugState>,
    debug_revision: u64,
}

fn install_window_functions(
    js: &JsRuntime,
    window_id: u64,
    actions: Rc<RefCell<VecDeque<wabou_shell::HostAction>>>,
    action_wake: Rc<RefCell<Option<WakeCallback>>>,
    clipboard_requests: Rc<RefCell<HashSet<u64>>>,
) {
    js.with(|ctx| -> rquickjs::Result<()> {
        let create_actions = actions.clone();
        let create_wake = action_wake.clone();
        ctx.globals().set(
            "__wabou_window_create",
            rquickjs::Function::new(ctx.clone(), move |json: String| -> u64 {
                let value: serde_json::Value = serde_json::from_str(&json).unwrap_or_default();
                let id = NEXT_WINDOW_ID.fetch_add(1, Ordering::Relaxed).max(2);
                let mut options = wabou_shell::WindowOptions::new();
                if let Some(title) = value.get("title").and_then(|value| value.as_str()) {
                    options = options.title(title);
                }
                let width = value
                    .get("width")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(800) as u32;
                let height = value
                    .get("height")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(600) as u32;
                options = options.initial_inner_size(width, height);
                if let Some(resizable) = value.get("resizable").and_then(|value| value.as_bool()) {
                    options = options.resizable(resizable);
                }
                if let (Some(width), Some(height)) = (
                    value.get("minWidth").and_then(|value| value.as_u64()),
                    value.get("minHeight").and_then(|value| value.as_u64()),
                ) {
                    options = options.min_inner_size(width as u32, height as u32);
                }
                create_actions
                    .borrow_mut()
                    .push_back(wabou_shell::HostAction::CreateWindow {
                        window_id: id,
                        options,
                    });
                if let Some(wake) = create_wake.borrow().as_ref() {
                    wake();
                }
                id
            })?,
        )?;
        let close_actions = actions.clone();
        let close_wake = action_wake.clone();
        ctx.globals().set(
            "__wabou_window_close",
            rquickjs::Function::new(ctx.clone(), move |target_window_id: u64| {
                close_actions
                    .borrow_mut()
                    .push_back(wabou_shell::HostAction::ControlWindow {
                        window_id: target_window_id,
                        command: wabou_shell::WindowCommand::Close,
                    });
                if let Some(wake) = close_wake.borrow().as_ref() {
                    wake();
                }
            })?,
        )?;
        let maximize_actions = actions.clone();
        let maximize_wake = action_wake.clone();
        ctx.globals().set(
            "__wabou_window_set_maximized",
            rquickjs::Function::new(ctx.clone(), move |target_window_id: u64, value: bool| {
                maximize_actions
                    .borrow_mut()
                    .push_back(wabou_shell::HostAction::ControlWindow {
                        window_id: target_window_id,
                        command: wabou_shell::WindowCommand::SetMaximized(value),
                    });
                if let Some(wake) = maximize_wake.borrow().as_ref() {
                    wake();
                }
            })?,
        )?;
        let title_actions = actions.clone();
        let title_wake = action_wake.clone();
        ctx.globals().set(
            "__wabou_window_set_title",
            rquickjs::Function::new(ctx.clone(), move |target_window_id: u64, title: String| {
                title_actions
                    .borrow_mut()
                    .push_back(wabou_shell::HostAction::ControlWindow {
                        window_id: target_window_id,
                        command: wabou_shell::WindowCommand::SetTitle(title),
                    });
                if let Some(wake) = title_wake.borrow().as_ref() {
                    wake();
                }
            })?,
        )?;
        ctx.globals().set("__wabou_window_id", window_id)?;

        let next_clipboard_request = Rc::new(Cell::new(1_u64));
        let write_actions = actions.clone();
        let write_wake = action_wake.clone();
        let write_requests = clipboard_requests.clone();
        let write_sequence = next_clipboard_request.clone();
        ctx.globals().set(
            "__wabou_clipboard_write",
            rquickjs::Function::new(ctx.clone(), move |text: String| -> u64 {
                let sequence = write_sequence.get();
                write_sequence.set((sequence.wrapping_add(1) & HOST_ACTION_SEQUENCE_MASK).max(1));
                let request_id = JS_HOST_ACTION_NAMESPACE | sequence;
                write_requests.borrow_mut().insert(request_id);
                write_actions
                    .borrow_mut()
                    .push_back(wabou_shell::HostAction::WriteClipboard { request_id, text });
                if let Some(wake) = write_wake.borrow().as_ref() {
                    wake();
                }
                request_id
            })?,
        )?;
        let read_actions = actions.clone();
        let read_wake = action_wake.clone();
        let read_requests = clipboard_requests.clone();
        ctx.globals().set(
            "__wabou_clipboard_read",
            rquickjs::Function::new(ctx.clone(), move || -> u64 {
                let sequence = next_clipboard_request.get();
                next_clipboard_request
                    .set((sequence.wrapping_add(1) & HOST_ACTION_SEQUENCE_MASK).max(1));
                let request_id = JS_HOST_ACTION_NAMESPACE | sequence;
                read_requests.borrow_mut().insert(request_id);
                read_actions
                    .borrow_mut()
                    .push_back(wabou_shell::HostAction::ReadClipboard { request_id });
                if let Some(wake) = read_wake.borrow().as_ref() {
                    wake();
                }
                request_id
            })?,
        )?;
        Ok(())
    })
    .expect("install window host functions");
}

fn complete_js_clipboard(js: &JsRuntime, request_id: u64, text: Option<String>, success: bool) {
    let result = js.with(|ctx| -> rquickjs::Result<()> {
        let callback: rquickjs::Function = ctx.globals().get("__wabou_clipboard_complete")?;
        callback.call::<_, ()>((request_id, text, success))
    });
    if let Err(error) = result {
        tracing::warn!(?error, request_id, "clipboard completion callback failed");
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
        let mut tree: TaffyTree<Paint> = TaffyTree::new();
        // Solid renderer's `mount` creates a #root handle with id 1; pre-create
        // the corresponding taffy node so AppendChild(1, …) has a parent. Size
        // it to 100% so it fills the viewport (the app's top div is w-full/h-full
        // of this root); without it, `100%` resolves against an auto-sized root.
        let root_style = taffy::Style {
            size: taffy::geometry::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0),
            },
            ..taffy::Style::default()
        };
        let root = tree.new_leaf(root_style).expect("root leaf");
        let _ = tree.set_node_context(root, Some(Paint::default()));

        let mut solid_to_node = HashMap::new();
        let mut node_to_solid = HashMap::new();
        let mut declared = HashMap::new();
        let mut children = HashMap::new();
        solid_to_node.insert(1, root);
        node_to_solid.insert(root, 1);
        declared.insert(root, Declared::default());
        children.insert(root, Vec::new());

        let (host_msg_handle, host_msg_inbox) = host_msg_channel(DEFAULT_HOST_MSG_CAPACITY);

        let pending_host_actions = Rc::new(RefCell::new(VecDeque::new()));
        let host_action_wake = Rc::new(RefCell::new(None));
        let pending_js_clipboard_requests = Rc::new(RefCell::new(HashSet::new()));
        install_window_functions(
            &js,
            window_id,
            pending_host_actions.clone(),
            host_action_wake.clone(),
            pending_js_clipboard_requests.clone(),
        );
        Self {
            js,
            tree,
            root,
            solid_to_node,
            node_to_solid,
            children,
            logical_parent: HashMap::new(),
            declared,
            collapsed_text: HashMap::new(),
            inline_roots: HashSet::new(),
            style_ir: None,
            rule_index: HashMap::new(),
            universal_rules: Vec::new(),
            utility_cache: HashMap::new(),
            warned_utility_classes: HashSet::new(),
            warned_ir_properties: HashSet::new(),
            svg_cache: HashMap::new(),
            runtime_transforms: HashMap::new(),
            overlay_planes: HashMap::new(),
            scrollbar_styles: HashMap::new(),
            base_color,
            atoms,
            listeners: HashMap::new(),
            pointer_position: (0.0, 0.0),
            pointer_buttons: 0,
            pointer_down_target: None,
            pointer_down_position: None,
            pointer_dragged: false,
            next_host_event_id: 0,
            hovered_target: None,
            focused_target: None,
            window_focused: true,
            has_raf: true,
            reload_rx: None,
            has_hmr_pending: Arc::new(AtomicBool::new(false)),
            pending_css: Some(pending_css),
            pending_fonts: Some(pending_fonts),
            frame_stats: Some(frame_stats),
            layout_metrics,
            resize_targets,
            invalidation: InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT,
            js_tick_ema: 0.0,
            last_viewport: (0, 0),
            device_scale: 1.0,
            selectable_text: HashMap::new(),
            selectable_text_order: Vec::new(),
            active_text_selection: None,
            last_text_selection: TextSelectionSnapshot::default(),
            text_selection_event_target: None,
            last_text_click: None,
            next_text_selection_scroll: None,
            placed_rects: HashMap::new(),
            hit_items: Vec::new(),
            scrollbar_hits: Vec::new(),
            scrollbar_drag: None,
            hovered_scrollbar: None,
            scrollbar_activity: HashMap::new(),
            semantics_enabled: false,
            semantics_dirty: true,
            semantic_snapshot: Arc::new(SemanticSnapshot::default()),
            widgets: HashMap::new(),
            widget_styles: HashMap::new(),
            pending_host_actions,
            pending_js_clipboard_requests,
            host_action_wake,
            next_host_action_id: 1,
            host_action_routes: HashMap::new(),
            widget_factories,
            wake_callback: None,
            scroll_offsets: HashMap::new(),
            pending_value_sync: HashSet::new(),
            batching_styles: false,
            dirty_styles: HashSet::new(),
            layout_viewport: None,
            vite_entry: None,
            last_hmr_result: HmrDrainResult::Idle,
            host_msg_inbox,
            host_msg_handle,
            debug_state: None,
            debug_revision: 0,
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

    pub fn set_debug_state(&mut self, state: wabou_devtools::SharedDebugState) {
        self.js.set_debug_state(state.clone());
        self.debug_state = Some(state);
    }

    /// Cloneable handle for background tasks / streams to push application
    /// messages toward JS (`host.subscribe` on the guest side).
    pub fn host_msg_handle(&self) -> HostMsgHandle {
        self.host_msg_handle.clone()
    }

    fn drain_host_messages(&mut self) {
        let batch = self.host_msg_inbox.drain_batch();
        if batch.is_empty() {
            return;
        }
        let events: Vec<_> = batch.into_iter().map(HostEvent::Application).collect();
        if let Err(e) = self.js.dispatch_host_frame(&events) {
            tracing::error!(target: "host_msg", error = ?e, count = events.len(), "dispatch Host application frame failed");
        }
    }

    /// Take a [`ReloadHandle`] the HMR client uses to push updates; the applier
    /// drains them in [`build_frame`] before the next tick.
    pub fn reload_handle(&mut self) -> ReloadHandle {
        let (tx, rx) = mpsc::channel();
        self.reload_rx = Some(rx);
        ReloadHandle {
            tx,
            pending: self.has_hmr_pending.clone(),
        }
    }

    /// Record the Vite entry path so declined HMR can re-import it in-process.
    pub fn set_vite_entry(&mut self, entry: impl Into<String>) {
        self.vite_entry = Some(entry.into());
    }

    /// Last HMR batch outcome (updated each `build_frame` that drains the queue).
    pub fn last_hmr_result(&self) -> &HmrDrainResult {
        &self.last_hmr_result
    }

    /// Drain every pending [`ReloadMsg`] into one batch and apply it.
    ///
    /// **Order:** native CSS updates are logged only (Style IR arrives via
    /// `pending_css` / virtual stylesheet in the same frame). JS updates run
    /// next; any reject/error or explicit full-reload payload resets the scene
    /// and re-imports the Vite entry when configured.
    fn drain_hmr_batch(&mut self) -> HmrDrainResult {
        let msgs = {
            let Some(rx) = &self.reload_rx else {
                return HmrDrainResult::Idle;
            };
            let mut msgs = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                msgs.push(msg);
            }
            msgs
        };
        if msgs.is_empty() {
            return HmrDrainResult::Idle;
        }
        let batch = plan_hmr_batch(msgs);
        self.apply_hmr_batch(batch)
    }

    fn apply_hmr_batch(&mut self, batch: HmrBatch) -> HmrDrainResult {
        // Native Vite CSS channel: styles that affect layout must go through
        // Style IR (`virtual:wabou-stylesheet` → `__wabou_set_stylesheet`), which
        // is already drained earlier in build_frame via `pending_css`.
        for path in &batch.css_paths {
            tracing::warn!(
                target: "hmr",
                %path,
                "ignoring native Vite css-update; layout styles use virtual:wabou-stylesheet → Style IR"
            );
        }

        if batch.full_reload {
            let reason = batch
                .full_reload_reason
                .unwrap_or_else(|| "vite full-reload".into());
            self.perform_full_reload(&reason);
            return HmrDrainResult::FullReload { reason };
        }

        #[cfg(feature = "vite")]
        let mut applied = 0usize;
        // Without the vite feature, count queued updates so diagnostics stay
        // useful even though the updates cannot be evaluated.
        #[cfg(not(feature = "vite"))]
        let applied = batch.js_updates.len();
        for update in batch.js_updates {
            #[cfg(feature = "vite")]
            {
                match self.js.apply_hmr_update(
                    &update.path,
                    &update.accepted_path,
                    update.timestamp,
                    update.source,
                ) {
                    Ok(true) => {
                        applied += 1;
                        tracing::debug!(
                            target: "hmr",
                            path = %update.path,
                            "HMR update accepted"
                        );
                    }
                    Ok(false) => {
                        let reason =
                            format!("module declined or missing hot context: {}", update.path);
                        tracing::warn!(target: "hmr", %reason);
                        self.perform_full_reload(&reason);
                        return HmrDrainResult::FullReload { reason };
                    }
                    Err(e) => {
                        let reason = format!("apply_hmr failed for {}: {e:?}", update.path);
                        tracing::error!(target: "hmr", %reason);
                        self.perform_full_reload(&reason);
                        return HmrDrainResult::FullReload { reason };
                    }
                }
            }
            #[cfg(not(feature = "vite"))]
            {
                let _ = update;
                tracing::warn!(
                    target: "hmr",
                    "received HMR update but binary built without `vite` feature"
                );
            }
        }
        if applied > 0 || !batch.css_paths.is_empty() {
            // CSS-only batches still report Applied (Style IR may have updated
            // via pending_css in the same frame).
            HmrDrainResult::Applied {
                js_updates: applied,
            }
        } else {
            HmrDrainResult::Idle
        }
    }

    /// Drop all non-root host nodes and re-import the Vite entry when possible.
    fn perform_full_reload(&mut self, reason: &str) {
        tracing::warn!(target: "hmr", %reason, "performing in-process full reload");
        self.reset_scene_tree();

        #[cfg(feature = "vite")]
        {
            if let Some(entry) = self.vite_entry.clone() {
                match self.js.reboot_vite_entry(&entry) {
                    Ok(()) => {
                        tracing::info!(target: "hmr", %entry, "vite entry re-imported after full reload");
                        self.invalidation
                            .insert(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
                        self.has_raf = true;
                    }
                    Err(e) => {
                        tracing::error!(
                            target: "hmr",
                            %entry,
                            error = ?e,
                            "full reload re-import failed — restart wabou-quick"
                        );
                    }
                }
                return;
            }
        }

        tracing::error!(
            target: "hmr",
            %reason,
            "full reload requested but no vite entry is configured — restart wabou-quick"
        );
    }

    /// Clear retained UI state down to the host root (solid id 1).
    fn reset_scene_tree(&mut self) {
        let doomed: Vec<NodeId> = self
            .solid_to_node
            .iter()
            .filter(|(solid, _)| **solid != 1)
            .map(|(_, node)| *node)
            .collect();
        for node in doomed {
            let _ = self.tree.remove(node);
        }
        self.solid_to_node.retain(|id, _| *id == 1);
        self.node_to_solid.retain(|_, id| *id == 1);
        self.declared.retain(|node, _| *node == self.root);
        self.children.clear();
        self.children.insert(self.root, Vec::new());
        let _ = self.tree.set_children(self.root, &[]);
        self.collapsed_text.clear();
        self.inline_roots.clear();
        self.svg_cache.clear();
        self.runtime_transforms.clear();
        self.overlay_planes.clear();
        self.scrollbar_styles.clear();
        self.widgets.clear();
        self.widget_styles.clear();
        self.listeners.clear();
        self.scroll_offsets.clear();
        self.scrollbar_hits.clear();
        self.scrollbar_drag = None;
        self.hovered_scrollbar = None;
        self.scrollbar_activity.clear();
        self.logical_parent.clear();
        self.semantic_snapshot = Arc::new(SemanticSnapshot::default());
        self.semantics_dirty = true;
        self.pending_value_sync.clear();
        self.dirty_styles.clear();
        self.pointer_down_target = None;
        self.pointer_down_position = None;
        self.pointer_dragged = false;
        self.hovered_target = None;
        self.focused_target = None;
        self.invalidation
            .insert(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
        if let Ok(mut targets) = self.resize_targets.try_borrow_mut() {
            targets.clear();
        }
    }

    /// Snapshot the currently resolved style for a Solid node id.
    ///
    /// Call this after the relevant op frame/build tick. It performs no style
    /// recomputation and cannot mutate renderer state.
    pub fn computed_node_snapshot(&self, solid_id: u32) -> Option<ComputedNodeSnapshot> {
        let &node = self.solid_to_node.get(&solid_id)?;
        let paint = self.tree.get_node_context(node)?;
        let declared = self.declared.get(&node)?;
        let atoms = self.atoms.borrow();
        Some(ComputedNodeSnapshot {
            solid_id,
            classes: declared
                .classes
                .iter()
                .filter_map(|class| atoms.resolve(*class).map(str::to_owned))
                .collect(),
            layout: self.tree.style(node).ok()?.clone(),
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

    /// Decode + apply one frame's ops in order.
    fn apply_frame(&mut self, frame: &Frame) {
        self.batching_styles = true;
        for op in &frame.ops {
            self.apply_op(op);
        }
        self.batching_styles = false;
        let dirty = std::mem::take(&mut self.dirty_styles);
        for node in dirty {
            self.recompute_node_now(node);
        }
        self.rebuild_layout_boxes();
    }

    fn apply_op(&mut self, op: &Op) {
        self.semantics_dirty = true;
        match op {
            Op::CreateElement { id, tag, attrs } => {
                let id = *id;
                let node = self
                    .tree
                    .new_leaf(taffy::Style::default())
                    .expect("new_leaf");
                let mut decl = Declared {
                    tag: Some(*tag),
                    ..Declared::default()
                };
                let class_value = {
                    let atoms = self.atoms.borrow();
                    if atoms.resolve(*tag).is_none() {
                        tracing::warn!(atom = tag.get(), "unknown tag atom");
                    }
                    attrs.iter().find_map(|(name, value)| {
                        matches!(atoms.resolve(*name), Some("class" | "className"))
                            .then_some(*value)
                    })
                };
                if let Some(value) = class_value {
                    // CreateElement attributes are retained for protocol
                    // compatibility; class tokens normally arrive through
                    // SetClassName and are already atoms there.
                    let mut atoms = self.atoms.borrow_mut();
                    decl.classes = value
                        .split_whitespace()
                        .map(|value| atoms.intern(value))
                        .collect();
                }
                for (name, value) in attrs {
                    decl.attrs.insert(*name, Arc::from(*value));
                }
                self.solid_to_node.insert(id, node);
                self.node_to_solid.insert(node, id);
                self.declared.insert(node, decl);
                self.children.insert(node, Vec::new());
                self.recompute_solid(id);
                // Rust-side widget creation: when the tag matches a known widget
                // type, create + store it. The widget paints custom content
                // (shapes, text+caret, …) that the standard renderer can't.
                if let Some(mut widget) = self.create_widget(*tag) {
                    // Feed initial attrs to the widget so it receives JS params.
                    let atoms = self.atoms.borrow();
                    for (name, value) in attrs {
                        if let Some(n) = atoms.resolve(*name) {
                            widget.attribute_changed(n, value);
                        }
                    }
                    drop(atoms);
                    self.widgets.insert(node, widget);
                    self.recompute_node(node);
                }
            }
            Op::CreateText { id, text } => {
                let id = *id;
                let node = self
                    .tree
                    .new_leaf(taffy::Style::default())
                    .expect("new_leaf");
                let decl = Declared {
                    text: Some(Arc::from(*text)),
                    ..Declared::default()
                };
                self.solid_to_node.insert(id, node);
                self.node_to_solid.insert(node, id);
                self.declared.insert(node, decl);
                self.children.insert(node, Vec::new());
                self.recompute_solid(id);
            }
            Op::AppendChild { parent, child } => {
                let (Some(&p), Some(&c)) = (
                    self.solid_to_node.get(parent),
                    self.solid_to_node.get(child),
                ) else {
                    return;
                };
                self.children.entry(p).or_default().push(c);
                self.logical_parent.insert(c, p);
                let kids = self.children[&p].clone();
                let _ = self.tree.set_children(p, &kids);
                // Nodes are styled when created, before they have a parent.
                self.recompute_subtree(c);
            }
            Op::InsertBefore {
                parent,
                child,
                ref_id,
            } => {
                let (Some(&p), Some(&c)) = (
                    self.solid_to_node.get(parent),
                    self.solid_to_node.get(child),
                ) else {
                    return;
                };
                // Compute the insertion index before mutably borrowing children.
                let idx = if *ref_id == 0 {
                    self.children.get(&p).map_or(0, Vec::len)
                } else {
                    self.solid_to_node
                        .get(ref_id)
                        .and_then(|r| {
                            self.children
                                .get(&p)
                                .and_then(|k| k.iter().position(|x| x == r))
                        })
                        .unwrap_or_else(|| self.children.get(&p).map_or(0, Vec::len))
                };
                let kids = self.children.entry(p).or_default();
                let idx = idx.min(kids.len());
                kids.insert(idx, c);
                self.logical_parent.insert(c, p);
                let kids = kids.clone();
                let _ = self.tree.set_children(p, &kids);
                self.recompute_subtree(c);
            }
            Op::RemoveChild { parent, child } => {
                let (Some(&p), Some(&c)) = (
                    self.solid_to_node.get(parent),
                    self.solid_to_node.get(child),
                ) else {
                    return;
                };
                if let Some(kids) = self.children.get_mut(&p) {
                    kids.retain(|x| *x != c);
                    let k = kids.clone();
                    let _ = self.tree.set_children(p, &k);
                    self.invalidation.insert(InvalidationFlags::LAYOUT);
                }
                if self.logical_parent.get(&c) == Some(&p) {
                    self.logical_parent.remove(&c);
                }
            }
            Op::ReplaceNode {
                parent,
                old_id,
                new_id,
            } => {
                let (Some(&p), Some(&old), Some(&new)) = (
                    self.solid_to_node.get(parent),
                    self.solid_to_node.get(old_id),
                    self.solid_to_node.get(new_id),
                ) else {
                    return;
                };
                if let Some(kids) = self.children.get_mut(&p) {
                    if let Some(i) = kids.iter().position(|x| *x == old) {
                        kids[i] = new;
                    }
                    let k = kids.clone();
                    let _ = self.tree.set_children(p, &k);
                }
                self.logical_parent.remove(&old);
                self.logical_parent.insert(new, p);
                self.recompute_subtree(new);
            }
            Op::SetText { id, text } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    if let Some(d) = self.declared.get_mut(&n) {
                        d.text = Some(Arc::from(*text));
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetClassName { id, classes } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    if let Some(d) = self.declared.get_mut(&n) {
                        d.classes.clone_from(classes);
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetStyle { id, prop, value } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    let Some(name) = self.atoms.borrow().resolve(*prop).map(str::to_owned) else {
                        tracing::warn!(atom = prop.get(), "unknown style-property atom");
                        return;
                    };
                    if let Some(d) = self.declared.get_mut(&n) {
                        d.inline.insert(*prop, InlineValue::Text(Arc::from(*value)));
                    }
                    // Fast path: a non-inherited inline property can be applied
                    // directly to the existing (post-inherit) ComputedStyle —
                    // the class rules haven't changed, so re-resolving them is
                    // wasted work, and skipping it also lets the layout branch
                    // skip the O(N) inherit pass. Inherited properties (color,
                    // font-*) still need the slow path to propagate to
                    // descendants. This is the hot path for per-frame animation
                    // (e.g. moving N nodes via top/left = 2N SetStyles/frame).
                    if INHERITED_PROPERTIES.contains(&name.as_str()) || name == "font" {
                        self.recompute_node(n);
                    } else if !self.apply_inline_fast(n, &name, value) {
                        if let Some(d) = self.declared.get_mut(&n) {
                            d.inline.remove(prop);
                        }
                        if self.warned_ir_properties.insert(*prop) {
                            tracing::warn!(property = %name, "unsupported inline style property or value");
                        }
                    }
                }
            }
            Op::SetStyleValue { id, prop, value } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    let Some(name) = self.atoms.borrow().resolve(*prop).map(str::to_owned) else {
                        tracing::warn!(atom = prop.get(), "unknown style-property atom");
                        return;
                    };
                    let ir = match value {
                        crate::protocol::StyleValue::Px(value) => IrValue::Length {
                            value: wabou_shell::style::IrLength::Px { value: *value },
                        },
                        crate::protocol::StyleValue::Percent(value) => IrValue::Length {
                            value: wabou_shell::style::IrLength::Percent { value: *value },
                        },
                        crate::protocol::StyleValue::Number(value) => {
                            IrValue::Number { value: *value }
                        }
                        crate::protocol::StyleValue::Boolean(value) => {
                            IrValue::Boolean { value: *value }
                        }
                        crate::protocol::StyleValue::Color(rgba) => IrValue::Color {
                            value: wabou_shell::style::IrColor::Literal { rgba: *rgba },
                        },
                        crate::protocol::StyleValue::Auto => IrValue::Length {
                            value: wabou_shell::style::IrLength::Auto,
                        },
                    };
                    if let Some(d) = self.declared.get_mut(&n) {
                        d.inline.insert(*prop, InlineValue::Typed(ir.clone()));
                    }
                    if INHERITED_PROPERTIES.contains(&name.as_str()) || name == "font" {
                        self.recompute_node(n);
                    } else if !self.apply_inline_ir_fast(n, &name, &ir) {
                        if let Some(d) = self.declared.get_mut(&n) {
                            d.inline.remove(prop);
                        }
                        if self.warned_ir_properties.insert(*prop) {
                            tracing::warn!(property = %name, "unsupported typed inline style property or value");
                        }
                    }
                }
            }
            Op::SetShadows { id, shadows } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    let prop = self.atoms.borrow_mut().intern("box-shadow");
                    let values = shadows
                        .iter()
                        .map(|shadow| {
                            let length = |value| IrValue::Length {
                                value: wabou_shell::style::IrLength::Px { value },
                            };
                            let mut fields = HashMap::from([
                                ("x".to_owned(), length(shadow.offset_x)),
                                ("y".to_owned(), length(shadow.offset_y)),
                                ("spread".to_owned(), length(shadow.spread)),
                                ("stdDev".to_owned(), length(shadow.std_dev)),
                                (
                                    "color".to_owned(),
                                    IrValue::Color {
                                        value: wabou_shell::style::IrColor::Literal {
                                            rgba: shadow.color,
                                        },
                                    },
                                ),
                            ]);
                            if let Some(radius) = shadow.radius {
                                fields.insert("radius".to_owned(), length(radius));
                            }
                            IrValue::Record { fields }
                        })
                        .collect();
                    let ir = IrValue::List { values };
                    if let Some(declared) = self.declared.get_mut(&n) {
                        declared.inline.insert(prop, InlineValue::Typed(ir.clone()));
                    }
                    if !self.apply_inline_ir_fast(n, "box-shadow", &ir) {
                        if let Some(declared) = self.declared.get_mut(&n) {
                            declared.inline.remove(&prop);
                        }
                        tracing::warn!("invalid Vello shadow list");
                    }
                }
            }
            Op::SetTransform2D { id, matrix } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    self.runtime_transforms.insert(n, *matrix);
                    if let Some(paint) = self.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.runtime_transform = Some(*matrix);
                        let _ = self.tree.set_node_context(n, Some(paint));
                    }
                }
            }
            Op::SetOverlayPlane { id, plane } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    let plane = match plane {
                        0 => OverlayPlane::Content,
                        1 => OverlayPlane::Floating,
                        2 => OverlayPlane::Modal,
                        // System and debug are intentionally host-reserved.
                        _ => OverlayPlane::Content,
                    };
                    self.overlay_planes.insert(n, plane);
                    if let Some(paint) = self.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.overlay_plane = plane;
                        let _ = self.tree.set_node_context(n, Some(paint));
                    }
                    self.invalidation.insert(InvalidationFlags::LAYOUT);
                }
            }
            Op::SetScrollbarStyle {
                id,
                visibility,
                thickness,
                margin,
                min_thumb_length,
                radius,
                colors,
            } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    let color = |rgba| {
                        Color::from_rgba8(
                            (rgba >> 24) as u8,
                            (rgba >> 16) as u8,
                            (rgba >> 8) as u8,
                            rgba as u8,
                        )
                    };
                    let style = ScrollbarStyle {
                        visibility: match visibility {
                            1 => ScrollbarVisibility::Always,
                            2 => ScrollbarVisibility::Hidden,
                            _ => ScrollbarVisibility::Auto,
                        },
                        thickness: *thickness,
                        margin: *margin,
                        min_thumb_length: *min_thumb_length,
                        radius: *radius,
                        track_color: color(colors[0]),
                        thumb_color: color(colors[1]),
                        hover_color: color(colors[2]),
                        active_color: color(colors[3]),
                    };
                    self.scrollbar_styles.insert(n, style);
                    if let Some(paint) = self.tree.get_node_context(n) {
                        let mut paint = paint.clone();
                        paint.scrollbar = style;
                        let _ = self.tree.set_node_context(n, Some(paint));
                    }
                }
            }
            Op::FocusNode { id } => {
                if self.solid_to_node.contains_key(id) {
                    self.set_focused_target(Some(*id));
                }
            }
            Op::ScrollTo { id, x, y } => {
                self.scroll_node(*id, *x, *y, false);
            }
            Op::ScrollBy { id, x, y } => {
                self.scroll_node(*id, *x, *y, true);
            }
            Op::RemoveStyle { id, prop } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    if let Some(d) = self.declared.get_mut(&n) {
                        d.inline.remove(prop);
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetAttribute { id, name, value }
                if matches!(
                    self.atoms.borrow().resolve(*name),
                    Some("class" | "className")
                ) =>
            {
                if let Some(&n) = self.solid_to_node.get(id) {
                    if let Some(d) = self.declared.get_mut(&n) {
                        let mut atoms = self.atoms.borrow_mut();
                        d.classes = value
                            .split_whitespace()
                            .map(|value| atoms.intern(value))
                            .collect();
                        d.attrs.insert(*name, Arc::from(*value));
                    }
                    self.recompute_node(n);
                }
            }
            Op::SetAttribute { id, name, value } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    if let Some(d) = self.declared.get_mut(&n) {
                        d.attrs.insert(*name, Arc::from(*value));
                    }
                    // Forward attribute changes to Rust-side widgets.
                    if let Some(widget) = self.widgets.get_mut(&n) {
                        let atoms = self.atoms.borrow();
                        if let Some(n_str) = atoms.resolve(*name) {
                            widget.attribute_changed(n_str, value);
                        }
                    }
                    self.recompute_node(n);
                }
            }
            Op::RemoveAttribute { id, name } => {
                if let Some(&n) = self.solid_to_node.get(id) {
                    let is_class = matches!(
                        self.atoms.borrow().resolve(*name),
                        Some("class" | "className")
                    );
                    if let Some(d) = self.declared.get_mut(&n) {
                        d.attrs.remove(name);
                        if is_class {
                            d.classes.clear();
                        }
                    }
                    if let Some(widget) = self.widgets.get_mut(&n)
                        && let Some(n_str) = self.atoms.borrow().resolve(*name)
                    {
                        widget.attribute_removed(n_str);
                    }
                    self.recompute_node(n);
                }
            }
            Op::AddEventListener { id, event_type } => {
                self.listeners.entry(*id).or_default().insert(*event_type);
            }
            Op::RemoveEventListener { id, event_type } => {
                if let Some(s) = self.listeners.get_mut(id) {
                    s.remove(*event_type);
                }
            }
            Op::DropNode { id } => {
                if self.pointer_down_target == Some(*id) {
                    self.cancel_active_pointer_gesture();
                }
                let node = self.solid_to_node.get(id).copied();
                let selection_dropped = self.active_text_selection.as_ref().is_some_and(|active| {
                    active.anchor_target == *id || active.focus_target == *id
                });
                if selection_dropped {
                    self.active_text_selection = None;
                    self.next_text_selection_scroll = None;
                    self.sync_text_selection_change();
                }
                if self.focused_target == Some(*id) {
                    if self.window_focused
                        && let Some(widget) = node.and_then(|node| self.widgets.get_mut(&node))
                    {
                        widget.focus_changed(false);
                    }
                    self.focused_target = None;
                }
                self.listeners.remove(id);
                self.resize_targets.borrow_mut().remove(id);
                // Keep the cached hover/focus targets from dangling at a solid
                // id whose node was just torn down — a stale hit would make
                // wheel/scroll (and keyboard delivery) silently no-op until a
                // pointer move re-establishes the hit.
                if self.hovered_target == Some(*id) {
                    self.hovered_target = None;
                }
                if let Some(n) = self.solid_to_node.remove(id) {
                    self.runtime_transforms.remove(&n);
                    self.overlay_planes.remove(&n);
                    self.scrollbar_styles.remove(&n);
                    self.node_to_solid.remove(&n);
                    self.scroll_offsets.remove(&n);
                    self.declared.remove(&n);
                    self.collapsed_text.remove(&n);
                    self.children.remove(&n);
                    self.logical_parent.remove(&n);
                    self.svg_cache.remove(&n);
                    if let Some(widget) = self.widgets.get_mut(&n) {
                        widget.unmount();
                    }
                    self.drain_widget_host_actions(n);
                    self.widgets.remove(&n);
                    self.widget_styles.remove(&n);
                    self.host_action_routes
                        .retain(|_, (widget_node, _)| *widget_node != n);
                    let _ = self.tree.remove(n);
                    self.invalidation.insert(InvalidationFlags::LAYOUT);
                }
            }
            Op::CreateComment { .. } | Op::FrameEnd => {}
        }
    }

    fn recompute_solid(&mut self, solid_id: u32) {
        if let Some(&n) = self.solid_to_node.get(&solid_id) {
            self.recompute_node(n);
        }
    }

    fn recompute_subtree(&mut self, node: NodeId) {
        self.recompute_node(node);
        let children = self.children.get(&node).cloned().unwrap_or_default();
        for child in children {
            self.recompute_subtree(child);
        }
    }

    /// Re-derive every node's `ComputedStyle` from the current `css` dict —
    /// called after a stylesheet host update.
    fn recompute_all(&mut self) {
        let nodes: Vec<NodeId> = self.solid_to_node.values().copied().collect();
        for n in nodes {
            self.recompute_node(n);
        }
        self.rebuild_layout_boxes();
    }

    /// Facts for [`InlineFormattingContext::build`] from the retained tree.
    fn node_facts(&self, node: NodeId) -> NodeFacts {
        let atoms = self.atoms.borrow();
        let decl = self.declared.get(&node);
        let tag = decl
            .and_then(|d| d.tag)
            .and_then(|t| atoms.resolve(t).map(str::to_owned));
        let text = decl.and_then(|d| d.text.clone());
        let display = self
            .tree
            .style(node)
            .map(|s| s.display)
            .unwrap_or(taffy::Display::DEFAULT);
        let is_svg = tag.as_deref() == Some("svg");
        let replaced = is_svg || self.widgets.contains_key(&node);
        let has_listeners = self
            .node_to_solid
            .get(&node)
            .and_then(|id| self.listeners.get(id))
            .is_some_and(|s| !s.is_empty());
        let independent_box = self.node_has_independent_box(node);
        NodeFacts {
            tag,
            text,
            display,
            display_explicit: decl.is_some_and(|d| d.display_explicit),
            replaced,
            has_listeners,
            independent_box,
        }
    }

    /// Principal-box signals: background, border, padding, margin, explicit size.
    /// Inline margin/padding is not modeled — any non-zero box edge keeps the node.
    fn node_has_independent_box(&self, node: NodeId) -> bool {
        if let Ok(style) = self.tree.style(node)
            && (rect_has_nonzero_lp(&style.padding)
                || rect_has_nonzero_lpa(&style.margin)
                || rect_has_nonzero_lp(&style.border)
                || size_is_explicit(&style.size))
        {
            return true;
        }
        if let Some(paint) = self.tree.get_node_context(node)
            && (paint.background.is_some()
                || paint.border.is_some()
                || paint.border_radius > 0.0
                || !paint.shadows.is_empty())
        {
            return true;
        }
        false
    }

    /// Project the logical tree into Taffy layout boxes via
    /// [`InlineFormattingContext`]. Only applies IFC output (children +
    /// collapsed text); does not re-implement formatting rules here.
    fn rebuild_layout_boxes(&mut self) {
        let ifc = InlineFormattingContext::build(&self.children, &|node| self.node_facts(node));

        let mut changed = Vec::new();
        self.inline_roots = ifc.roots;
        // Drop collapsed_text for parents that no longer collapse.
        let stale: Vec<NodeId> = self
            .collapsed_text
            .keys()
            .copied()
            .filter(|n| !ifc.collapsed_text.contains_key(n))
            .collect();
        for n in stale {
            self.collapsed_text.remove(&n);
            changed.push(n);
        }
        for (parent, text) in ifc.collapsed_text {
            if self.collapsed_text.get(&parent) != Some(&text) {
                self.collapsed_text.insert(parent, text);
                changed.push(parent);
            }
        }
        for (parent, kids) in ifc.layout_children {
            let _ = self.tree.set_children(parent, &kids);
        }
        for node in changed {
            self.recompute_node_now(node);
        }
    }

    /// Propagate inherited text styles (`color`, `font-size`) top-down so a
    /// Resolve every node's [`DeclaredPaint`] against its parent into a
    /// fully computed [`Paint`] on the taffy node. Mirrors CSS inheritance
    /// without a full CSS engine. Run after `apply_frame` and before layout
    /// (the measure callback reads the effective `font_size`).
    fn inherit(&mut self) {
        self.inherit_node(self.root, &InheritedPaint::default());
    }

    fn serialize_svg(&self, root: NodeId, color: Color) -> Option<String> {
        fn escape_text(out: &mut String, value: &str) {
            for ch in value.chars() {
                match ch {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    _ => out.push(ch),
                }
            }
        }

        fn escape_attr(out: &mut String, value: &str, current_color: &str) {
            let value = value.replace("currentColor", current_color);
            for ch in value.chars() {
                match ch {
                    '&' => out.push_str("&amp;"),
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '"' => out.push_str("&quot;"),
                    '\'' => out.push_str("&apos;"),
                    _ => out.push(ch),
                }
            }
        }

        fn write_node(
            this: &Applier,
            atoms: &AtomPool,
            node: NodeId,
            current_color: &str,
            root: bool,
            out: &mut String,
        ) -> Option<()> {
            let decl = this.declared.get(&node)?;
            if let Some(text) = &decl.text {
                escape_text(out, text);
                return Some(());
            }
            let tag = atoms.resolve(decl.tag?)?;
            out.push('<');
            out.push_str(tag);
            let has_xmlns = decl
                .attrs
                .keys()
                .any(|name| atoms.resolve(*name) == Some("xmlns"));
            if root && !has_xmlns {
                out.push_str(" xmlns=\"http://www.w3.org/2000/svg\"");
            }
            for (name, value) in &decl.attrs {
                let Some(name) = atoms.resolve(*name) else {
                    continue;
                };
                // Solid's keyed reconciliation marker is not an SVG
                // presentation attribute and may contain arbitrary data.
                if name == "key" || name.starts_with("on") {
                    continue;
                }
                out.push(' ');
                out.push_str(name);
                out.push_str("=\"");
                escape_attr(out, value, current_color);
                out.push('"');
            }
            let children = this.children.get(&node).map(Vec::as_slice).unwrap_or(&[]);
            if children.is_empty() {
                out.push_str("/>");
            } else {
                out.push('>');
                for child in children {
                    write_node(this, atoms, *child, current_color, false, out)?;
                }
                out.push_str("</");
                out.push_str(tag);
                out.push('>');
            }
            Some(())
        }

        let atoms = self.atoms.borrow();
        let decl = self.declared.get(&root)?;
        if decl.tag.and_then(|tag| atoms.resolve(tag)) != Some("svg") {
            return None;
        }
        let current_color = format!("{:x}", color.to_rgba8());
        let mut source = String::new();
        write_node(self, &atoms, root, &current_color, true, &mut source)?;
        Some(source)
    }

    fn inherit_node(&mut self, node: NodeId, parent: &InheritedPaint) {
        let Some(decl) = self.declared.get(&node) else {
            return;
        };
        let declared = decl.paint.clone();
        let inherited = declared.resolve_inherited(parent);

        // Preserve host-owned content from the previous computed paint (text,
        // widget scene, intrinsic size). Cascade never owns these.
        let prev = self.tree.get_node_context(node);
        let mut host = HostPaint {
            text: prev.and_then(|p| p.text.clone()),
            text_runs: prev
                .map(|p| p.text_runs.clone())
                .unwrap_or_else(|| Arc::from([])),
            selection_rects: prev
                .map(|p| p.selection_rects.clone())
                .unwrap_or_else(|| Arc::from([])),
            svg: None,
            widget: prev.and_then(|p| p.widget.clone()),
            intrinsic_size: prev.and_then(|p| p.intrinsic_size),
            runtime_transform: self.runtime_transforms.get(&node).copied(),
            overlay_plane: self.overlay_planes.get(&node).copied().unwrap_or_default(),
            scrollbar: self
                .scrollbar_styles
                .get(&node)
                .copied()
                .unwrap_or_default(),
        };
        if let Some(source) = self.serialize_svg(node, inherited.text_color) {
            let cached = self
                .svg_cache
                .get(&node)
                .filter(|(cached_source, _)| cached_source.as_ref() == source)
                .map(|(_, image)| image.clone());
            host.svg = if let Some(image) = cached {
                Some(image)
            } else {
                match wabou_shell::svg::SvgImage::parse(&source) {
                    Ok(image) => {
                        let image = Arc::new(image);
                        self.svg_cache
                            .insert(node, (Arc::from(source), image.clone()));
                        Some(image)
                    }
                    Err(error) => {
                        tracing::warn!(%error, "failed to parse inline SVG");
                        self.svg_cache.remove(&node);
                        None
                    }
                }
            };
        } else {
            self.svg_cache.remove(&node);
        }

        let paint = declared.resolve(parent, host);
        let _ = self.tree.set_node_context(node, Some(paint));

        let kids = self.children.get(&node).cloned().unwrap_or_default();
        for c in kids {
            self.inherit_node(c, &inherited);
        }

        if self.inline_roots.contains(&node) {
            let mut text = String::new();
            let mut runs = Vec::new();
            for child in self.children.get(&node).cloned().unwrap_or_default() {
                self.collect_styled_inline_runs(child, &mut text, &mut runs);
            }
            if let Some(mut paint) = self.tree.get_node_context(node).cloned() {
                paint.text = Some(Arc::from(text));
                paint.text_runs = Arc::from(runs);
                let _ = self.tree.set_node_context(node, Some(paint));
            }
        }
    }

    fn collect_styled_inline_runs(
        &self,
        node: NodeId,
        text: &mut String,
        runs: &mut Vec<wabou_shell::text::TextRun>,
    ) {
        let Some(decl) = self.declared.get(&node) else {
            return;
        };
        if let Some(value) = &decl.text {
            let start = text.len();
            text.push_str(value);
            let end = text.len();
            if start != end {
                let paint = self.tree.get_node_context(node);
                runs.push(wabou_shell::text::TextRun {
                    range: start..end,
                    font_size: paint.map(|p| p.font_size).unwrap_or(16.0),
                    font_weight: paint.map(|p| p.font_weight).unwrap_or(400.0),
                    line_height: paint.and_then(|p| p.line_height),
                    color: wabou_shell::text::brush_for_color(
                        paint.map(|p| p.text_color).unwrap_or(Color::BLACK),
                    ),
                });
            }
            return;
        }
        if let Some(children) = self.children.get(&node) {
            for child in children {
                self.collect_styled_inline_runs(*child, text, runs);
            }
        }
    }

    /// Re-derive `ComputedStyle` from declared state (classes via the css dict,
    /// inline applied on top so inline wins per-prop) and push to the taffy node.
    /// The root (#root, solid id 1) is skipped: its 100% viewport size + default
    /// Paint are host-provided and must not be overwritten by an empty
    /// `Declared` (which would reset the size to auto and collapse the tree).
    fn recompute_node(&mut self, node: NodeId) {
        if self.batching_styles {
            self.dirty_styles.insert(node);
            return;
        }
        self.recompute_node_now(node);
    }

    /// Fast-path apply for a non-inherited inline style: update the cascaded
    /// [`DeclaredPaint`] + patch the matching non-inherited fields on the
    /// existing computed [`Paint`], skipping full class re-resolution and the
    /// O(N) inherit pass. Correct because a non-inherited inline property
    /// doesn't propagate to descendants. Hot path for animation (moving N
    /// nodes via top/left = 2N SetStyles/frame).
    fn apply_inline_fast(&mut self, node: NodeId, prop: &str, value: &str) -> bool {
        let ir = style::parse_ir_value(value);
        self.apply_inline_ir_fast(node, prop, &ir)
    }

    fn apply_inline_ir_fast(&mut self, node: NodeId, prop: &str, ir: &IrValue) -> bool {
        let Ok(existing) = self.tree.style(node) else {
            return false;
        };
        let Some(decl) = self.declared.get_mut(&node) else {
            return false;
        };
        let mut layout = existing.clone();
        if prop == "display" {
            decl.display_explicit = true;
        }
        if !style::apply_ir(&mut layout, &mut decl.paint, prop, ir) {
            return false;
        }
        let declared = decl.paint.clone();
        let layout_changed = existing != &layout;
        if layout_changed {
            let _ = self.tree.set_style(node, layout);
            self.invalidation.insert(InvalidationFlags::LAYOUT);
        }
        // Patch only non-inherited computed fields; inherited fields stay at
        // their last resolved values (INHERIT is clear on this path).
        if let Some(mut paint) = self.tree.get_node_context(node).cloned() {
            paint.background = declared.background;
            paint.opacity = declared.opacity;
            paint.transform = declared.transform;
            paint.shadows = declared.shadows;
            paint.border_radius = declared.border_radius;
            paint.border = declared.border;
            paint.pointer_events = declared.pointer_events;
            paint.z_index = declared.z_index;
            let _ = self.tree.set_node_context(node, Some(paint));
        }
        true
    }

    fn recompute_node_now(&mut self, node: NodeId) {
        if node == self.root {
            return;
        }
        let Some(decl) = self.declared.get(&node) else {
            return;
        };
        let (layout, declared_paint, host_text, host_intrinsic, display_explicit) = {
            let atoms = self.atoms.borrow();
            let mut layout = taffy::Style::default();
            let mut paint = DeclaredPaint::default();
            let mut display_explicit = false;
            let tag_name = decl.tag.and_then(|tag| atoms.resolve(tag));
            if tag_name.is_some_and(NodeFacts::is_block_tag) {
                layout.display = taffy::Display::Block;
            }
            // Wabou's explicit Text primitive is a single-line layout leaf by
            // default, not an HTML inline formatting context. Authored class
            // and inline declarations below may still opt into wrapping and
            // flex shrinking explicitly.
            if tag_name == Some("text") {
                layout.flex_shrink = 0.0;
                paint.wrap_text = Some(false);
            }
            // Wabou has no CSS cascade: utility declarations are applied in
            // class-list order, with later classes overriding earlier ones.
            // Universal rules run first; inline style still runs last below.
            let mut declarations = Vec::new();
            if let Some(sheet) = &self.style_ir {
                for &idx in &self.universal_rules {
                    let rule = &sheet.rules[idx];
                    for (index, declaration) in rule.declarations.iter().enumerate() {
                        declarations.push((
                            declaration.important,
                            rule.specificity,
                            0usize,
                            rule.source_order,
                            index,
                            declaration.property.clone(),
                            declaration.value.clone(),
                        ));
                    }
                }
                for (class_position, class) in decl.classes.iter().enumerate() {
                    let Some(indices) = self.rule_index.get(class) else {
                        continue;
                    };
                    for &idx in indices {
                        let rule = &sheet.rules[idx];
                        for (index, declaration) in rule.declarations.iter().enumerate() {
                            declarations.push((
                                declaration.important,
                                rule.specificity,
                                class_position + 1,
                                rule.source_order,
                                index,
                                declaration.property.clone(),
                                declaration.value.clone(),
                            ));
                        }
                    }
                }
            }
            // Runtime-created class names use the same ordering and IR as
            // precompiled classes, rather than forming a higher-priority
            // fallback layer.
            for (class_position, class) in decl.classes.iter().enumerate() {
                if self.rule_index.contains_key(class) {
                    continue;
                }
                let utility = self.utility_cache.entry(*class).or_insert_with(|| {
                    atoms
                        .resolve(*class)
                        .ok_or_else(|| "unknown class atom".to_string())
                        .and_then(|name| {
                            wabou_style::parse_utility(name).map_err(|error| error.to_string())
                        })
                });
                let utility = match utility {
                    Ok(utility) => utility,
                    Err(diagnostic) => {
                        if self.warned_utility_classes.insert(*class) {
                            tracing::warn!(
                                class = atoms.resolve(*class).unwrap_or("<unknown>"),
                                    %diagnostic,
                                    "rejected runtime utility class"
                            );
                        }
                        continue;
                    }
                };
                for (index, declaration) in utility.declarations.iter().enumerate() {
                    declarations.push((
                        false,
                        10,
                        class_position + 1,
                        0,
                        index,
                        declaration.property.clone(),
                        style_ir::utility_value(&declaration.value),
                    ));
                }
            }
            declarations.sort_by_key(
                |(important, specificity, class_position, order, index, _, _)| {
                    (*important, *specificity, *class_position, *order, *index)
                },
            );
            for (_, _, _, _, _, property, value) in declarations {
                display_explicit |= property == "display";
                if !style::apply_ir(&mut layout, &mut paint, &property, &value)
                    && let Some(atom) = atoms.get(&property)
                    && self.warned_ir_properties.insert(atom)
                {
                    tracing::warn!(property, "unsupported Style IR property");
                }
            }
            for (property, value) in &decl.inline {
                if let Some(property) = atoms.resolve(*property) {
                    display_explicit |= property == "display";
                    let ir = value.ir();
                    style::apply_ir(&mut layout, &mut paint, property, &ir);
                }
            }
            let mut host_intrinsic = None;
            if decl.tag.and_then(|tag| atoms.resolve(tag)) == Some("svg") {
                let view_box_size = decl
                    .attrs
                    .iter()
                    .find_map(|(name, value)| {
                        (atoms.resolve(*name) == Some("viewBox")).then(|| {
                            let values: Vec<f32> = value
                                .split_whitespace()
                                .filter_map(|part| part.parse().ok())
                                .collect();
                            (values.len() == 4 && values[2] > 0.0 && values[3] > 0.0)
                                .then_some([values[2], values[3]])
                        })
                    })
                    .flatten();
                host_intrinsic = Some(view_box_size.unwrap_or([300.0, 150.0]));
                // Width/height are SVG presentation attributes and provide the
                // replaced element's intrinsic CSS size. Utility/inline CSS
                // still wins when it supplied an explicit dimension.
                for (name, value) in &decl.attrs {
                    let Some(px) = value
                        .strip_suffix("px")
                        .unwrap_or(value)
                        .parse::<f32>()
                        .ok()
                    else {
                        continue;
                    };
                    match atoms.resolve(*name) {
                        Some("width") if layout.size.width.is_auto() => {
                            layout.size.width = taffy::Dimension::length(px);
                        }
                        Some("height") if layout.size.height.is_auto() => {
                            layout.size.height = taffy::Dimension::length(px);
                        }
                        _ => {}
                    }
                }
            }
            // Replaced elements paint their own content. Standard text would
            // otherwise be drawn a second time underneath the widget scene.
            let host_text = (!self.widgets.contains_key(&node))
                .then(|| {
                    self.collapsed_text
                        .get(&node)
                        .cloned()
                        .or_else(|| decl.text.clone())
                })
                .flatten();
            if let Some(size) = self
                .widgets
                .get(&node)
                .and_then(|widget| widget.intrinsic_size())
            {
                host_intrinsic = Some(size);
            }
            (layout, paint, host_text, host_intrinsic, display_explicit)
        };
        // Persist cascade output for the inherit pass (and future resolves).
        if let Some(decl) = self.declared.get_mut(&node) {
            decl.paint = declared_paint.clone();
            decl.display_explicit = display_explicit;
        }
        let _ = self.tree.set_style(node, layout);

        // Resolve this node immediately against the parent so non-inherited
        // fields (background, …) and own inherited decls are correct without
        // waiting for the next build_frame inherit walk. Descendants still
        // need the full inherit pass when inherited props change.
        let parent_inherited = self
            .tree
            .parent(node)
            .and_then(|parent| self.tree.get_node_context(parent))
            .map(|p| InheritedPaint {
                text_color: p.text_color,
                font_size: p.font_size,
                font_weight: p.font_weight,
                line_height: p.line_height,
                wrap_text: p.wrap_text,
                text_selectable: p.text_selectable,
                text_select_all: p.text_select_all,
                text_align: p.text_align,
                font_family: p.font_family.clone(),
            })
            .unwrap_or_default();
        let prev = self.tree.get_node_context(node);
        let host = HostPaint {
            text: host_text,
            text_runs: prev
                .map(|p| p.text_runs.clone())
                .unwrap_or_else(|| Arc::from([])),
            selection_rects: prev
                .map(|p| p.selection_rects.clone())
                .unwrap_or_else(|| Arc::from([])),
            svg: prev.and_then(|p| p.svg.clone()),
            widget: prev.and_then(|p| p.widget.clone()),
            intrinsic_size: host_intrinsic,
            runtime_transform: self.runtime_transforms.get(&node).copied(),
            overlay_plane: self.overlay_planes.get(&node).copied().unwrap_or_default(),
            scrollbar: self
                .scrollbar_styles
                .get(&node)
                .copied()
                .unwrap_or_default(),
        };
        let paint = declared_paint.resolve(&parent_inherited, host);
        let _ = self.tree.set_node_context(node, Some(paint));
        self.invalidation.insert(InvalidationFlags::LAYOUT);
        // Cascade may have changed declared inherited props → re-resolve tree.
        self.invalidation.insert(InvalidationFlags::INHERIT);
    }

    fn solid_id_for_node(&self, node: NodeId) -> Option<u32> {
        self.node_to_solid.get(&node).copied()
    }

    fn is_logical_descendant(&self, node: NodeId, ancestor: NodeId) -> bool {
        let mut current = Some(node);
        while let Some(node) = current {
            if node == ancestor {
                return true;
            }
            current = self.logical_parent.get(&node).copied();
        }
        false
    }

    fn hit_test(&self, x: f64, y: f64) -> Option<u32> {
        let point = Point::new(x, y);
        for item in self.hit_items.iter().rev() {
            match item {
                HitItem::Content(node)
                    if node.pointer_events
                        && node.clips.iter().all(|clip| {
                            hit_contains(clip.rect, clip.radius, clip.transform, point)
                        })
                        && hit_contains(node.rect, 0.0, node.transform, point) =>
                {
                    return Some(node.solid_id);
                }
                HitItem::Scrollbar(hit)
                    if scrollbar_hit(&hit.placed, hit.transform.inverse() * point).is_some() =>
                {
                    return None;
                }
                _ => {}
            }
        }
        None
    }

    fn rebuild_hit_geometry(&mut self, placed: &[PlacedNode]) {
        self.hit_items.clear();
        self.scrollbar_hits.clear();
        let placed_by_id: HashMap<_, _> = placed.iter().map(|node| (node.node_id, node)).collect();
        let mut transforms = HashMap::with_capacity(placed.len());
        let mut clip_chains: HashMap<NodeId, Vec<HitClip>> = HashMap::with_capacity(placed.len());
        let mut content_hits = HashMap::new();
        let mut scrollbar_hits = HashMap::new();
        for node in placed {
            let parent_transform = node
                .parent_node_id
                .and_then(|parent| transforms.get(&parent).copied())
                .unwrap_or(Affine::IDENTITY);
            let transform = wabou_shell::scene::resolve_node_transform(node, parent_transform);
            let mut clips = node
                .parent_node_id
                .and_then(|parent| clip_chains.get(&parent).cloned())
                .unwrap_or_default();
            if let Some(parent) = node.parent_node_id
                && let Some(parent_node) = placed_by_id.get(&parent)
                && let Some(rect) = parent_node.own_clip
            {
                clips.push(HitClip {
                    rect,
                    radius: parent_node.own_clip_radius,
                    transform: transforms[&parent],
                });
            }
            if let Some(&solid_id) = self.node_to_solid.get(&node.node_id) {
                content_hits.insert(
                    node.node_id,
                    HitNode {
                        solid_id,
                        rect: node.rect,
                        transform,
                        clips: clips.clone(),
                        pointer_events: node.paint.pointer_events,
                    },
                );
            }
            if node.scroll.opacity > 0.0 && node.scroll.range.iter().any(|range| *range > 0.5) {
                let hit = ScrollbarHit {
                    node: node.node_id,
                    placed: node.clone(),
                    transform,
                };
                self.scrollbar_hits.push(hit.clone());
                scrollbar_hits.insert(node.node_id, hit);
            }
            transforms.insert(node.node_id, transform);
            clip_chains.insert(node.node_id, clips);
        }
        for event in subtree_events(placed) {
            match event {
                SubtreeEvent::Enter(node) => {
                    if let Some(hit) = content_hits.remove(&node.node_id) {
                        self.hit_items.push(HitItem::Content(hit));
                    }
                }
                SubtreeEvent::Exit(node) => {
                    if let Some(hit) = scrollbar_hits.remove(&node.node_id) {
                        self.hit_items.push(HitItem::Scrollbar(Box::new(hit)));
                    }
                }
            }
        }
    }

    fn update_scrollbar_visuals(&mut self, placed: &mut [PlacedNode]) {
        let now = Instant::now();
        self.scrollbar_activity.retain(|_, started| {
            now.duration_since(*started) < SCROLLBAR_FADE_DELAY + SCROLLBAR_FADE_DURATION
        });
        for node in placed {
            node.scroll.opacity = match node.paint.scrollbar.visibility {
                ScrollbarVisibility::Always => 1.0,
                ScrollbarVisibility::Hidden => 0.0,
                ScrollbarVisibility::Auto => self
                    .scrollbar_activity
                    .get(&node.node_id)
                    .map_or(0.0, |started| {
                        let elapsed = now.duration_since(*started);
                        if elapsed <= SCROLLBAR_FADE_DELAY {
                            1.0
                        } else {
                            1.0 - (elapsed - SCROLLBAR_FADE_DELAY).as_secs_f32()
                                / SCROLLBAR_FADE_DURATION.as_secs_f32()
                        }
                    })
                    .clamp(0.0, 1.0),
            };
            node.scroll.interaction = if self
                .scrollbar_drag
                .is_some_and(|drag| drag.node == node.node_id)
            {
                2
            } else if self
                .hovered_scrollbar
                .is_some_and(|(owner, _)| owner == node.node_id)
            {
                1
            } else {
                0
            };
        }
    }

    fn rebuild_semantic_snapshot(&mut self, placed: &[PlacedNode]) {
        semantics::rebuild(self, placed);
    }

    fn scrollbar_at(&self, x: f64, y: f64) -> Option<(NodeId, ScrollbarTarget)> {
        let point = Point::new(x, y);
        for item in self.hit_items.iter().rev() {
            match item {
                HitItem::Scrollbar(hit) => {
                    if let Some(target) =
                        scrollbar_hit(&hit.placed, hit.transform.inverse() * point)
                    {
                        return Some((hit.node, target));
                    }
                }
                HitItem::Content(node)
                    if node.pointer_events
                        && node.clips.iter().all(|clip| {
                            hit_contains(clip.rect, clip.radius, clip.transform, point)
                        })
                        && hit_contains(node.rect, 0.0, node.transform, point) =>
                {
                    return None;
                }
                _ => {}
            }
        }
        None
    }

    fn drag_scrollbar(&mut self, x: f64, y: f64) -> bool {
        let Some(mut drag) = self.scrollbar_drag else {
            return false;
        };
        let Some(hit) = self.scrollbar_hits.iter().find(|hit| hit.node == drag.node) else {
            self.scrollbar_drag = None;
            return false;
        };
        let local = hit.transform.inverse() * Point::new(x, y);
        let position = match drag.axis {
            ScrollAxis::Horizontal => local.x,
            ScrollAxis::Vertical => local.y,
        };
        let delta = (position - drag.last_position) * scrollbar_drag_ratio(&hit.placed, drag.axis);
        drag.last_position = position;
        self.scrollbar_drag = Some(drag);
        let offset = self.scroll_offsets.entry(drag.node).or_insert([0.0; 2]);
        let index = usize::from(drag.axis == ScrollAxis::Vertical);
        let old = offset[index];
        offset[index] = (offset[index] + delta as f32).clamp(0.0, hit.placed.scroll.range[index]);
        let changed = offset[index] != old;
        self.semantics_dirty |= changed;
        self.scrollbar_activity.insert(drag.node, Instant::now());
        changed
    }

    fn scroll_nearest(&mut self, target: u32, delta_x: f32, delta_y: f32) -> bool {
        let Some(mut node) = self.solid_to_node.get(&target).copied() else {
            return false;
        };
        loop {
            let scrollable = self.tree.style(node).ok().is_some_and(|style| {
                style.overflow.x == taffy::Overflow::Scroll
                    || style.overflow.y == taffy::Overflow::Scroll
            });
            if scrollable {
                let Ok(layout) = self.tree.layout(node) else {
                    return false;
                };
                let viewport_width =
                    (layout.size.width - layout.border.left - layout.border.right).max(0.0);
                let viewport_height =
                    (layout.size.height - layout.border.top - layout.border.bottom).max(0.0);
                let max_x = (layout.content_size.width - viewport_width).max(0.0);
                let max_y = (layout.content_size.height - viewport_height).max(0.0);
                let style = self.tree.style(node).expect("style checked above");
                let offset = self.scroll_offsets.entry(node).or_insert([0.0, 0.0]);
                let old = *offset;
                if style.overflow.x == taffy::Overflow::Scroll {
                    offset[0] = (offset[0] + delta_x).clamp(0.0, max_x);
                }
                if style.overflow.y == taffy::Overflow::Scroll {
                    offset[1] = (offset[1] + delta_y).clamp(0.0, max_y);
                }
                if *offset != old {
                    self.semantics_dirty = true;
                    self.scrollbar_activity.insert(node, Instant::now());
                    return true;
                }
            }
            let Some(parent) = self.tree.parent(node) else {
                return false;
            };
            node = parent;
        }
    }

    fn scroll_node(&mut self, target: u32, x: f32, y: f32, relative: bool) -> bool {
        let Some(&node) = self.solid_to_node.get(&target) else {
            return false;
        };
        let Ok(style) = self.tree.style(node) else {
            return false;
        };
        let scroll_x = style.overflow.x == taffy::Overflow::Scroll;
        let scroll_y = style.overflow.y == taffy::Overflow::Scroll;
        if !scroll_x && !scroll_y {
            return false;
        }
        let Ok(layout) = self.tree.layout(node) else {
            return false;
        };
        let viewport_width =
            (layout.size.width - layout.border.left - layout.border.right).max(0.0);
        let viewport_height =
            (layout.size.height - layout.border.top - layout.border.bottom).max(0.0);
        let max_x = (layout.content_size.width - viewport_width).max(0.0);
        let max_y = (layout.content_size.height - viewport_height).max(0.0);
        let offset = self.scroll_offsets.entry(node).or_insert([0.0, 0.0]);
        let old = *offset;
        if scroll_x && x.is_finite() {
            offset[0] = (if relative { offset[0] + x } else { x }).clamp(0.0, max_x);
        }
        if scroll_y && y.is_finite() {
            offset[1] = (if relative { offset[1] + y } else { y }).clamp(0.0, max_y);
        }
        let changed = *offset != old;
        if changed {
            self.scrollbar_activity.insert(node, Instant::now());
        }
        self.semantics_dirty |= changed;
        changed
    }

    fn text_selection_scroll_delta(&self) -> Option<(u32, f32, f32)> {
        if self.pointer_buttons & 1 == 0 {
            return None;
        }
        let active = self.active_text_selection.as_ref()?;
        // Autoscroll belongs to the endpoint currently following the pointer.
        // The stable anchor can live in a different scroll container during
        // a cross-panel selection.
        let target = active.focus_target;
        let mut node = *self.solid_to_node.get(&target)?;
        let pointer = [
            self.pointer_position.0 as f32,
            self.pointer_position.1 as f32,
        ];
        let axis_delta = |position: f32, start: f32, end: f32| {
            let outside = if position < start {
                position - start
            } else if position > end {
                position - end
            } else {
                0.0
            };
            if outside == 0.0 {
                0.0
            } else {
                outside.signum() * (outside.abs() * 0.35).clamp(4.0, 40.0)
            }
        };

        loop {
            let style = self.tree.style(node).ok()?;
            let scroll_x = style.overflow.x == taffy::Overflow::Scroll;
            let scroll_y = style.overflow.y == taffy::Overflow::Scroll;
            if (scroll_x || scroll_y)
                && let (Some(rect), Ok(layout)) =
                    (self.placed_rects.get(&node), self.tree.layout(node))
            {
                let x0 = rect[0] + layout.border.left;
                let y0 = rect[1] + layout.border.top;
                let x1 = rect[2] - layout.border.right;
                let y1 = rect[3] - layout.border.bottom;
                let dx = if scroll_x {
                    axis_delta(pointer[0], x0, x1)
                } else {
                    0.0
                };
                let dy = if scroll_y {
                    axis_delta(pointer[1], y0, y1)
                } else {
                    0.0
                };
                if dx != 0.0 || dy != 0.0 {
                    return Some((target, dx, dy));
                }
            }
            node = self.tree.parent(node)?;
        }
    }

    fn arm_text_selection_autoscroll(&mut self) {
        self.next_text_selection_scroll = self
            .text_selection_scroll_delta()
            .is_some()
            .then(Instant::now);
    }

    fn tick_text_selection_autoscroll(&mut self) -> bool {
        if !self
            .next_text_selection_scroll
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return false;
        }
        let Some((target, dx, dy)) = self.text_selection_scroll_delta() else {
            self.next_text_selection_scroll = None;
            return false;
        };
        let changed = self.scroll_nearest(target, dx, dy);
        self.next_text_selection_scroll =
            changed.then(|| Instant::now() + Duration::from_millis(50));
        changed
    }

    fn has_listener_in_chain(&self, mut solid_id: u32, code: u8) -> bool {
        loop {
            if self
                .listeners
                .get(&solid_id)
                .is_some_and(|events| events.contains(code))
            {
                return true;
            }
            let Some(&node) = self.solid_to_node.get(&solid_id) else {
                return false;
            };
            let Some(parent) = self.tree.parent(node) else {
                return false;
            };
            let Some(parent_id) = self.solid_id_for_node(parent) else {
                return false;
            };
            solid_id = parent_id;
        }
    }

    fn dispatch_pointer(
        &mut self,
        target: u32,
        code: u8,
        button: Option<PointerButton>,
        modifiers: Modifiers,
    ) -> bool {
        if !self.has_listener_in_chain(target, code) {
            return false;
        }
        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = self.pointer_position.0;
        data[event_data::CLIENT_Y as usize] = self.pointer_position.1;
        data[event_data::BUTTON as usize] = button.map_or(0, Self::web_button) as f64;
        data[event_data::BUTTONS as usize] = Self::web_buttons(self.pointer_buttons) as f64;
        data[event_data::MODS as usize] = modifiers.bits() as f64;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id: 0,
            cancellable: false,
            payload: NodeEventPayload::Numeric(data),
        });
        if let Err(error) = self.js.dispatch_host_frame(&[event]) {
            tracing::warn!(?error, target, code, "event dispatch failed");
            return false;
        }
        true
    }

    fn dispatch_cancellable_numeric(
        &mut self,
        target: u32,
        code: u8,
        data: [f64; event_data::LEN],
    ) -> (bool, bool) {
        if !self.has_listener_in_chain(target, code) {
            return (false, false);
        }
        self.next_host_event_id = self.next_host_event_id.wrapping_add(1).max(1);
        let event_id = self.next_host_event_id;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id,
            cancellable: true,
            payload: NodeEventPayload::Numeric(data),
        });
        match self.js.dispatch_host_frame(&[event]) {
            Ok(disposition) => (true, disposition.is_prevented(event_id)),
            Err(error) => {
                tracing::warn!(?error, target, code, "event dispatch failed");
                (false, false)
            }
        }
    }

    fn dispatch_cancellable_json(
        &mut self,
        target: u32,
        code: u8,
        payload: String,
    ) -> (bool, bool) {
        if !self.has_listener_in_chain(target, code) {
            return (false, false);
        }
        self.next_host_event_id = self.next_host_event_id.wrapping_add(1).max(1);
        let event_id = self.next_host_event_id;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id,
            cancellable: true,
            payload: NodeEventPayload::Json(payload),
        });
        match self.js.dispatch_host_frame(&[event]) {
            Ok(disposition) => (true, disposition.is_prevented(event_id)),
            Err(error) => {
                tracing::warn!(?error, target, code, "event dispatch failed");
                (false, false)
            }
        }
    }

    fn link_url(&self, mut target: u32) -> Option<String> {
        let atoms = self.atoms.borrow();
        loop {
            let node = *self.solid_to_node.get(&target)?;
            if let Some(declared) = self.declared.get(&node)
                && declared.tag.and_then(|tag| atoms.resolve(tag)) == Some("a")
                && let Some((_, href)) = declared
                    .attrs
                    .iter()
                    .find(|(name, _)| atoms.resolve(**name) == Some("href"))
            {
                return Some(href.to_string());
            }
            let parent = self.tree.parent(node)?;
            target = self.solid_id_for_node(parent)?;
        }
    }

    fn open_link_default(&mut self, target: u32) -> bool {
        let Some(raw) = self.link_url(target) else {
            return false;
        };
        self.pending_host_actions
            .borrow_mut()
            .push_back(wabou_shell::HostAction::OpenUrl(raw));
        true
    }

    /// Translate Wabou's compact native button representation only at the
    /// Solid/Web compatibility boundary.
    fn web_button(button: PointerButton) -> u8 {
        match button {
            PointerButton::Primary => 0,
            PointerButton::Auxiliary => 1,
            PointerButton::Secondary => 2,
            PointerButton::Other(index) => index.min(u8::MAX as u16) as u8,
        }
    }

    fn web_buttons(native: u32) -> u32 {
        (native & 1) | ((native & 2) << 1) | ((native & 4) >> 1) | (native & !7)
    }

    fn response(handled: bool) -> EventResponse {
        if handled {
            EventResponse::handled()
        } else {
            EventResponse::IGNORED
        }
    }

    fn dispatch_json(&mut self, target: u32, code: u8, payload: &str) -> bool {
        if !self.has_listener_in_chain(target, code) {
            return false;
        }
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id: 0,
            cancellable: false,
            payload: NodeEventPayload::Json(payload.to_owned()),
        });
        if let Err(error) = self.js.dispatch_host_frame(&[event]) {
            tracing::warn!(?error, target, code, "event dispatch failed");
            return false;
        }
        true
    }

    fn is_text_input_target(&self, target: u32) -> bool {
        let Some(&node) = self.solid_to_node.get(&target) else {
            return false;
        };
        if self
            .widgets
            .get(&node)
            .is_some_and(|widget| widget.accepts_focus())
        {
            return true;
        }
        let atoms = self.atoms.borrow();
        self.declared
            .get(&node)
            .and_then(|decl| decl.tag)
            .and_then(|tag| atoms.resolve(tag))
            == Some("input")
    }

    fn set_focused_target(&mut self, target: Option<u32>) -> bool {
        let old = self.focused_target;
        if old == target {
            return false;
        }
        self.semantics_dirty = true;
        self.focused_target = target;
        let mut changed = false;
        if let Some(old) = old {
            if self.window_focused
                && let Some(node) = self.solid_to_node.get(&old)
                && let Some(widget) = self.widgets.get_mut(node)
            {
                widget.focus_changed(false);
            }
            changed |= self.dispatch_json(old, event::BLUR, "");
            changed |= self.dispatch_json(old, event::FOCUSOUT, "");
        }
        if let Some(new) = target {
            if self.window_focused
                && let Some(node) = self.solid_to_node.get(&new)
                && let Some(widget) = self.widgets.get_mut(node)
            {
                widget.focus_changed(true);
            }
            changed |= self.dispatch_json(new, event::FOCUS, "");
            changed |= self.dispatch_json(new, event::FOCUSIN, "");
        }
        changed
    }

    fn set_window_focused(&mut self, focused: bool) -> bool {
        if self.window_focused == focused {
            return false;
        }
        self.window_focused = focused;
        let Some(target) = self.focused_target else {
            return false;
        };
        let mut changed = false;
        if let Some(node) = self.solid_to_node.get(&target)
            && let Some(widget) = self.widgets.get_mut(node)
        {
            widget.focus_changed(focused);
            changed = true;
        }
        let (focus, focus_within) = if focused {
            (event::FOCUS, event::FOCUSIN)
        } else {
            (event::BLUR, event::FOCUSOUT)
        };
        changed |= self.dispatch_json(target, focus, "");
        changed |= self.dispatch_json(target, focus_within, "");
        changed
    }

    fn handle_widget_event(&mut self, target: u32, input: &UiEvent) -> Option<EventResponse> {
        let node = *self.solid_to_node.get(&target)?;
        let result = {
            let widget = self.widgets.get_mut(&node)?;
            widget.handle_event(input)
        };
        self.drain_widget_host_actions(node);
        self.drain_widget_node_events(node);
        if !result.is_handled() {
            return None;
        }
        // Value sync is deferred to build_frame: `current_value()` reads
        // `cached_value`, which is only fresh after `paint_widgets` applies the
        // pending edits queued above. Reading + dispatching here would send a
        // stale value to JS. `flush_value_sync` drains this set after paint.
        if result.value_changed() {
            self.pending_value_sync.insert(target);
        }
        Some(EventResponse {
            handled: true,
            request_redraw: result.requests_redraw(),
            consume_key_text: result.consumes_key_text(),
            text_input: None,
            clipboard: result.clipboard_request().cloned(),
        })
    }

    fn enqueue_widget_host_action(&mut self, node: NodeId, action: wabou_shell::HostAction) {
        let action = match action {
            wabou_shell::HostAction::ReadClipboard { request_id } => {
                let host_request_id = self.next_host_action_id;
                self.next_host_action_id =
                    (self.next_host_action_id.wrapping_add(1) & HOST_ACTION_SEQUENCE_MASK).max(1);
                self.host_action_routes
                    .insert(host_request_id, (node, request_id));
                wabou_shell::HostAction::ReadClipboard {
                    request_id: host_request_id,
                }
            }
            action => action,
        };
        self.pending_host_actions.borrow_mut().push_back(action);
    }

    fn drain_widget_host_actions(&mut self, node: NodeId) {
        while let Some(action) = self
            .widgets
            .get_mut(&node)
            .and_then(|widget| widget.take_host_action())
        {
            self.enqueue_widget_host_action(node, action);
        }
    }

    fn drain_widget_node_events(&mut self, node: NodeId) -> bool {
        let Some(target) = self.solid_id_for_node(node) else {
            return false;
        };
        let mut events = Vec::new();
        while let Some(event) = self
            .widgets
            .get_mut(&node)
            .and_then(|widget| widget.take_node_event())
        {
            events.push(event);
        }
        #[allow(clippy::unnecessary_fold)] // Every queued event must dispatch.
        events.into_iter().fold(false, |dispatched, event| {
            self.dispatch_json(target, event.event_code, &event.json) || dispatched
        })
    }

    /// Drain [`pending_value_sync`]: after `paint_widgets` has applied pending
    /// edits, read each widget's now-fresh `current_value()`, sync the `value`
    /// attr, and dispatch `INPUT` to JS.
    fn flush_value_sync(&mut self) {
        if self.pending_value_sync.is_empty() {
            return;
        }
        let value_atom = self.atoms.borrow_mut().intern("value");
        for target in self.pending_value_sync.drain().collect::<Vec<_>>() {
            let Some(&node) = self.solid_to_node.get(&target) else {
                continue;
            };
            let Some(value) = self
                .widgets
                .get(&node)
                .and_then(|w| w.current_value().map(str::to_owned))
            else {
                continue;
            };
            if let Some(decl) = self.declared.get_mut(&node) {
                decl.attrs.insert(value_atom, Arc::from(value.as_str()));
            }
            let payload = serde_json::json!({ "value": value }).to_string();
            let _ = self.dispatch_json(target, event::INPUT, &payload);
        }
    }

    fn dispatch_resize_changes(&mut self) -> bool {
        let mut targets = self.resize_targets.borrow_mut();
        let mut changes = Vec::new();
        for (&solid_id, last) in targets.iter_mut() {
            let Some(&node) = self.solid_to_node.get(&solid_id) else {
                continue;
            };
            let Ok(layout) = self.tree.layout(node) else {
                continue;
            };
            let width = (layout.size.width
                - layout.border.left
                - layout.border.right
                - layout.padding.left
                - layout.padding.right)
                .max(0.0);
            let height = (layout.size.height
                - layout.border.top
                - layout.border.bottom
                - layout.padding.top
                - layout.padding.bottom)
                .max(0.0);
            if *last != Some((width, height)) {
                *last = Some((width, height));
                changes.push((solid_id, width, height));
            }
        }
        drop(targets);
        if changes.is_empty() {
            return false;
        }
        let events: Vec<_> = changes
            .into_iter()
            .map(|(target, width, height)| {
                HostEvent::Resize(ResizeObservation {
                    target,
                    width,
                    height,
                })
            })
            .collect();
        if let Err(error) = self.js.dispatch_host_frame(&events) {
            tracing::warn!(?error, "ResizeObserver dispatch failed");
            return false;
        }
        true
    }

    /// Create a Rust-side widget for a tag by looking up the factory registry.
    /// Built-in widgets (Canvas) are pre-registered by `HostBuilder::new()`;
    /// users add their own via `.widget("tag", || Box::new(MyWidget))`.
    fn create_widget(&self, tag: Atom) -> Option<Box<dyn crate::widget::Widget>> {
        self.widget_factories.get(&tag).map(|f| {
            let mut widget = f();
            if let Some(wake) = &self.wake_callback {
                widget.set_wake_callback(wake.clone());
            }
            widget
        })
    }

    /// Deliver resolved content styles before widget measurement.
    fn sync_widget_styles(&mut self) {
        for (&node, widget) in &mut self.widgets {
            let Some(paint) = self.tree.get_node_context(node) else {
                continue;
            };
            let style = crate::widget::WidgetStyle::from(paint);
            if self.widget_styles.get(&node) != Some(&style) {
                widget.style_changed(&style);
                self.widget_styles.insert(node, style);
            }
        }
    }

    /// After layout, call `Widget::paint` for each widget node + store the
    /// resulting Scene fragment in the matching PlacedNode's `paint.widget`.
    /// `build_scene` composites it at the node's content-box origin.
    fn paint_widgets(&mut self, placed: &mut [PlacedNode], tcx: &mut TextContext) {
        let mut transforms = HashMap::with_capacity(placed.len());
        for n in placed.iter_mut() {
            let parent_transform = n
                .parent_node_id
                .and_then(|parent| transforms.get(&parent).copied())
                .unwrap_or(Affine::IDENTITY);
            let transform = wabou_shell::scene::resolve_node_transform(n, parent_transform);
            transforms.insert(n.node_id, transform);
            if let Some(w) = self.widgets.get_mut(&n.node_id) {
                w.set_position(n.rect[0], n.rect[1]);
                let window_to_local =
                    Affine::translate((-f64::from(n.rect[0]), -f64::from(n.rect[1])))
                        * transform.inverse();
                w.set_window_to_local(window_to_local.as_coeffs());
                let [width, height] = n.content_size;
                if width > 0.0 && height > 0.0 {
                    let scene = w.paint_scaled(width, height, self.device_scale, tcx);
                    n.paint.widget = Some(std::sync::Arc::new(scene));
                }
            }
        }
    }

    fn measure_widgets(&mut self, tcx: &mut TextContext) {
        let changed: Vec<_> = self
            .widgets
            .iter_mut()
            .filter_map(|(&node, widget)| {
                let measured = widget.measure(tcx);
                let current = self
                    .tree
                    .get_node_context(node)
                    .and_then(|paint| paint.intrinsic_size);
                (measured != current).then_some((node, measured))
            })
            .collect();
        for (node, measured) in changed {
            if let Some(mut paint) = self.tree.get_node_context(node).cloned() {
                paint.intrinsic_size = measured;
                let _ = self.tree.set_node_context(node, Some(paint));
                self.invalidation.insert(InvalidationFlags::LAYOUT);
            }
        }
    }

    fn prepare_text_selection(&mut self, placed: &mut [PlacedNode], tcx: &mut TextContext) {
        self.selectable_text.clear();
        self.selectable_text_order.clear();
        for node in placed.iter_mut() {
            node.paint.selection_rects = Arc::from([]);
            let Some(text) = node.paint.text.clone() else {
                continue;
            };
            if !node.paint.text_selectable || self.widgets.contains_key(&node.node_id) {
                continue;
            }
            let Some(&target) = self.node_to_solid.get(&node.node_id) else {
                continue;
            };
            let layout = layout_text_styled(
                tcx,
                text.clone(),
                node.paint.font_size,
                node.paint.font_weight,
                node.paint.line_height,
                node.paint.text_align,
                wabou_shell::text::brush_for_color(node.paint.text_color),
                node.paint.text_runs.clone(),
                node.paint.font_family.as_ref(),
                node.paint
                    .wrap_text
                    .then_some((node.rect[2] - node.rect[0]).max(0.0)),
            );
            let selectable = SelectableText {
                text,
                visual_y: node.content_origin[1]..node.content_origin[1] + layout.height().max(0.0),
                layout,
                origin: node.content_origin,
                select_all: node.paint.text_select_all,
                order: self.selectable_text_order.len(),
            };
            self.selectable_text_order.push(target);
            self.selectable_text.insert(target, selectable);
        }

        let valid = self.active_text_selection.as_ref().is_none_or(|active| {
            self.selectable_text.contains_key(&active.anchor_target)
                && self.selectable_text.contains_key(&active.focus_target)
        });
        if !valid {
            self.active_text_selection = None;
            self.next_text_selection_scroll = None;
            return;
        }
        if let Some(active) = &mut self.active_text_selection {
            let anchor = &self.selectable_text[&active.anchor_target].layout;
            active.base_selection = active.base_selection.refresh(anchor);
            let focus = &self.selectable_text[&active.focus_target].layout;
            active.focus_selection = active.focus_selection.refresh(focus);
        }
        for node in placed.iter_mut() {
            let Some(&target) = self.node_to_solid.get(&node.node_id) else {
                continue;
            };
            let Some(range) = self.text_selection_range(target) else {
                continue;
            };
            let text = &self.selectable_text[&target];
            let selection = Selection::new(
                Cursor::from_byte_index(&text.layout, range.start, Affinity::Downstream),
                Cursor::from_byte_index(&text.layout, range.end, Affinity::Upstream),
            );
            node.paint.selection_rects = selection
                .geometry(&text.layout)
                .into_iter()
                .map(|(rect, _)| {
                    [
                        rect.x0 as f32,
                        rect.y0 as f32,
                        rect.x1 as f32,
                        rect.y1 as f32,
                    ]
                })
                .collect::<Vec<_>>()
                .into();
        }
    }

    fn selection_from_point(
        text: &SelectableText,
        granularity: TextSelectionGranularity,
        x: f32,
        y: f32,
    ) -> Selection {
        if text.select_all {
            return Selection::new(
                Cursor::from_byte_index(&text.layout, 0, Affinity::Downstream),
                Cursor::from_byte_index(&text.layout, text.text.len(), Affinity::Upstream),
            );
        }
        match granularity {
            TextSelectionGranularity::Cluster => Selection::from_point(&text.layout, x, y),
            TextSelectionGranularity::Word => Selection::word_from_point(&text.layout, x, y),
            TextSelectionGranularity::Line => Selection::line_from_point(&text.layout, x, y),
        }
    }

    fn begin_text_selection(&mut self, target: u32, x: f64, y: f64, modifiers: Modifiers) -> bool {
        if modifiers.shift() && self.active_text_selection.is_some() {
            self.last_text_click = None;
            return self.extend_text_selection(Some(target), x, y);
        }
        let Some(text) = self.selectable_text.get(&target) else {
            self.last_text_click = None;
            return self.active_text_selection.take().is_some();
        };
        let local_x = x as f32 - text.origin[0];
        let local_y = y as f32 - text.origin[1];
        let now = Instant::now();
        let clicks =
            self.last_text_click
                .map_or(1, |(time, last_target, last_x, last_y, count)| {
                    if last_target == target
                        && now.duration_since(time) <= Duration::from_millis(400)
                        && (x - last_x).abs() <= 4.0
                        && (y - last_y).abs() <= 4.0
                    {
                        count % 3 + 1
                    } else {
                        1
                    }
                });
        self.last_text_click = Some((now, target, x, y, clicks));
        let granularity = match clicks {
            2 => TextSelectionGranularity::Word,
            3 => TextSelectionGranularity::Line,
            _ => TextSelectionGranularity::Cluster,
        };
        let selection = Self::selection_from_point(text, granularity, local_x, local_y);
        self.active_text_selection = Some(ActiveTextSelection {
            anchor_target: target,
            focus_target: target,
            base_selection: selection,
            focus_selection: selection,
            granularity,
        });
        true
    }

    fn extend_text_selection(&mut self, hit_target: Option<u32>, x: f64, y: f64) -> bool {
        if self.active_text_selection.is_none() {
            return false;
        }
        let target = hit_target
            .filter(|target| self.selectable_text.contains_key(target))
            .or_else(|| {
                self.selectable_text_order
                    .iter()
                    .copied()
                    .min_by(|left, right| {
                        let distance = |target: u32| {
                            let text = &self.selectable_text[&target];
                            let dx = if x < f64::from(text.origin[0]) {
                                f64::from(text.origin[0]) - x
                            } else if x > f64::from(text.origin[0] + text.layout.width()) {
                                x - f64::from(text.origin[0] + text.layout.width())
                            } else {
                                0.0
                            };
                            let dy = if y < f64::from(text.origin[1]) {
                                f64::from(text.origin[1]) - y
                            } else if y > f64::from(text.origin[1] + text.layout.height()) {
                                y - f64::from(text.origin[1] + text.layout.height())
                            } else {
                                0.0
                            };
                            dx * dx + dy * dy
                        };
                        distance(*left).total_cmp(&distance(*right))
                    })
            });
        let Some(target) = target else {
            return false;
        };
        let text = &self.selectable_text[&target];
        let local_x = x as f32 - text.origin[0];
        let local_y = y as f32 - text.origin[1];
        let active = self.active_text_selection.as_mut().unwrap();
        active.focus_target = target;
        active.focus_selection = if target == active.anchor_target {
            active
                .base_selection
                .extend_to_point(&text.layout, local_x, local_y)
        } else {
            Self::selection_from_point(text, active.granularity, local_x, local_y)
        };
        true
    }

    fn text_selection_range(&self, target: u32) -> Option<std::ops::Range<usize>> {
        let active = self.active_text_selection.as_ref()?;
        let anchor_index = self.selectable_text.get(&active.anchor_target)?.order;
        let focus_index = self.selectable_text.get(&active.focus_target)?.order;
        let target_index = self.selectable_text.get(&target)?.order;
        if anchor_index == focus_index {
            return (target_index == anchor_index).then(|| active.focus_selection.text_range());
        }
        let anchor_range = active.base_selection.text_range();
        let focus_range = active.focus_selection.text_range();
        let text_len = self.selectable_text.get(&target)?.text.len();
        if anchor_index < focus_index {
            match target_index {
                index if index < anchor_index || index > focus_index => None,
                index if index == anchor_index => Some(anchor_range.start..text_len),
                index if index == focus_index => Some(0..focus_range.end),
                _ => Some(0..text_len),
            }
        } else {
            match target_index {
                index if index < focus_index || index > anchor_index => None,
                index if index == focus_index => Some(focus_range.start..text_len),
                index if index == anchor_index => Some(0..anchor_range.end),
                _ => Some(0..text_len),
            }
        }
        .filter(|range| !range.is_empty())
    }

    fn selected_text(&self) -> Option<String> {
        self.active_text_selection.as_ref()?;
        let mut selected = String::new();
        let mut previous_visual_y: Option<std::ops::Range<f32>> = None;
        for target in &self.selectable_text_order {
            let Some(range) = self.text_selection_range(*target) else {
                continue;
            };
            let text = &self.selectable_text[target];
            if previous_visual_y.as_ref().is_some_and(|previous| {
                previous.end <= text.visual_y.start || text.visual_y.end <= previous.start
            }) {
                selected.push('\n');
            }
            selected.push_str(&text.text[range]);
            previous_visual_y = Some(text.visual_y.clone());
        }
        (!selected.is_empty()).then_some(selected)
    }

    fn sync_text_selection_change(&mut self) -> bool {
        let text = self.selected_text();
        let kind = text.as_ref().and_then(|_| {
            self.active_text_selection
                .as_ref()
                .map(|selection| match selection.granularity {
                    TextSelectionGranularity::Cluster => "simple",
                    TextSelectionGranularity::Word => "word",
                    TextSelectionGranularity::Line => "line",
                })
        });
        let snapshot = TextSelectionSnapshot { text, kind };
        if snapshot == self.last_text_selection {
            return false;
        }
        if let Some(target) = self
            .active_text_selection
            .as_ref()
            .map(|selection| selection.anchor_target)
        {
            self.text_selection_event_target = Some(target);
        }
        self.last_text_selection = snapshot.clone();
        let Some(target) = self.text_selection_event_target else {
            return false;
        };
        self.dispatch_json(
            target,
            event::TEXTSELECTIONCHANGE,
            &serde_json::json!({ "text": snapshot.text, "kind": snapshot.kind }).to_string(),
        )
    }

    fn select_all_text(&mut self) -> bool {
        let Some((&anchor_target, &focus_target)) = self
            .selectable_text_order
            .first()
            .zip(self.selectable_text_order.last())
        else {
            return false;
        };
        let anchor = &self.selectable_text[&anchor_target];
        let focus = &self.selectable_text[&focus_target];
        let whole = |text: &SelectableText| {
            Selection::new(
                Cursor::from_byte_index(&text.layout, 0, Affinity::Downstream),
                Cursor::from_byte_index(&text.layout, text.text.len(), Affinity::Upstream),
            )
        };
        self.active_text_selection = Some(ActiveTextSelection {
            anchor_target,
            focus_target,
            base_selection: whole(anchor),
            focus_selection: whole(focus),
            granularity: TextSelectionGranularity::Cluster,
        });
        true
    }
    fn publish_layout_metrics(&self, placed: &[PlacedNode], width: u32, height: u32) {
        let viewport = LayoutRect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
        let mut snapshot = self.layout_metrics.borrow_mut();
        snapshot.revision = snapshot.revision.wrapping_add(1);
        snapshot.viewport = viewport;
        snapshot.nodes.clear();
        snapshot.nodes.reserve(placed.len());
        for placed_node in placed {
            let Some(&id) = self.node_to_solid.get(&placed_node.node_id) else {
                continue;
            };
            let rect = |value: [f32; 4]| LayoutRect {
                x: value[0],
                y: value[1],
                width: (value[2] - value[0]).max(0.0),
                height: (value[3] - value[1]).max(0.0),
            };
            snapshot.nodes.insert(
                id,
                LayoutMetric {
                    rect: rect(placed_node.rect),
                    clip: placed_node.clip.map_or(viewport, rect),
                },
            );
        }
    }

    fn publish_debug_snapshot(&mut self, placed: &[PlacedNode]) {
        let Some(state) = self.debug_state.clone() else {
            return;
        };
        self.debug_revision = self.debug_revision.wrapping_add(1);
        let atoms = self.atoms.borrow();
        let placed_by_id: HashMap<_, _> = placed.iter().map(|node| (node.node_id, node)).collect();
        let mut css_transforms = HashMap::with_capacity(placed.len());
        for node in placed {
            let parent_transform = node
                .parent_node_id
                .and_then(|parent| css_transforms.get(&parent).copied())
                .unwrap_or(Affine::IDENTITY);
            css_transforms.insert(
                node.node_id,
                wabou_shell::scene::resolve_node_transform(node, parent_transform),
            );
        }
        let mut nodes = Vec::with_capacity(placed.len());
        for placed_node in placed {
            let Some(&id) = self.node_to_solid.get(&placed_node.node_id) else {
                continue;
            };
            let declared = self.declared.get(&placed_node.node_id);
            let tag = declared
                .and_then(|declared| declared.tag)
                .and_then(|tag| atoms.resolve(tag))
                .unwrap_or(if id == 1 { "#root" } else { "#text" })
                .to_owned();
            let mut attrs: Vec<_> = declared
                .into_iter()
                .flat_map(|declared| declared.attrs.iter())
                .filter_map(|(name, value)| {
                    atoms.resolve(*name).map(|name| {
                        let lower = name.to_ascii_lowercase();
                        let value = if ["password", "token", "secret", "authorization"]
                            .iter()
                            .any(|needle| lower.contains(needle))
                        {
                            "[REDACTED]".to_owned()
                        } else {
                            value.chars().take(4096).collect()
                        };
                        (name.to_owned(), value)
                    })
                })
                .collect();
            attrs.sort_by(|left, right| left.0.cmp(&right.0));
            let mut listeners: Vec<_> = self
                .listeners
                .get(&id)
                .into_iter()
                .flat_map(|events| events.codes())
                .collect();
            listeners.sort_unstable();
            let classes = declared
                .into_iter()
                .flat_map(|declared| declared.classes.iter())
                .filter_map(|class| atoms.resolve(*class).map(str::to_owned))
                .collect();
            let matched_rules = declared
                .into_iter()
                .flat_map(|declared| {
                    std::iter::once(&self.universal_rules).chain(
                        declared
                            .classes
                            .iter()
                            .filter_map(|class| self.rule_index.get(class)),
                    )
                })
                .flatten()
                .filter_map(|index| self.style_ir.as_ref()?.rules.get(*index))
                .map(|rule| {
                    if rule.class_name == "*" {
                        "*".to_owned()
                    } else {
                        format!(".{}", rule.class_name)
                    }
                })
                .collect();
            let [x0, y0, x1, y1] = placed_node.rect;
            let [cx, cy] = placed_node.content_origin;
            let [cw, ch] = placed_node.content_size;
            let content_transform =
                css_transforms[&placed_node.node_id] * Affine::translate((cx as f64, cy as f64));
            let (static_transform, _) = wabou_shell::scene::resolve_local_transforms(placed_node);
            let layout = self.tree.style(placed_node.node_id).ok();
            let debug_rect = |[x0, y0, x1, y1]: [f32; 4]| wabou_devtools::Rect {
                x: x0,
                y: y0,
                width: (x1 - x0).max(0.0),
                height: (y1 - y0).max(0.0),
            };
            let intersect = |left: [f32; 4], right: [f32; 4]| {
                [
                    left[0].max(right[0]),
                    left[1].max(right[1]),
                    left[2].min(right[2]),
                    left[3].min(right[3]),
                ]
            };
            let mut clip_ancestors = Vec::new();
            let mut ancestor_id = placed_node.parent_node_id;
            while let Some(node_id) = ancestor_id {
                let Some(ancestor) = placed_by_id.get(&node_id).copied() else {
                    break;
                };
                if let Some(rect) = ancestor.own_clip {
                    clip_ancestors.push(wabou_devtools::DebugClip {
                        node_id: self.node_to_solid.get(&node_id).copied().unwrap_or(0),
                        kind: "ancestor-overflow".into(),
                        coordinate_space: "layout-window-logical".into(),
                        rect: debug_rect(rect),
                        radius: ancestor.own_clip_radius,
                        transform: css_transforms[&node_id].as_coeffs(),
                    });
                }
                ancestor_id = ancestor.parent_node_id;
            }
            clip_ancestors.reverse();
            if let Some(rect) = placed_node.own_clip {
                clip_ancestors.push(wabou_devtools::DebugClip {
                    node_id: id,
                    kind: "self-overflow".into(),
                    coordinate_space: "layout-window-logical".into(),
                    rect: debug_rect(rect),
                    radius: placed_node.own_clip_radius,
                    transform: css_transforms[&placed_node.node_id].as_coeffs(),
                });
            }
            let widget_local = self.widgets.contains_key(&placed_node.node_id).then(|| {
                let border_inset = placed_node.border_widths.into_iter().fold(0.0, f32::max);
                wabou_devtools::DebugClip {
                    node_id: id,
                    kind: "widget-content".into(),
                    coordinate_space: "content-local".into(),
                    rect: wabou_devtools::Rect {
                        x: 0.0,
                        y: 0.0,
                        width: cw,
                        height: ch,
                    },
                    radius: (placed_node.paint.border_radius - border_inset).max(0.0),
                    transform: content_transform.as_coeffs(),
                }
            });
            let widget_self_clip = widget_local.as_ref().and_then(|clip| {
                if clip.radius > 0.0 {
                    Some(([cx, cy, cx + cw, cy + ch], clip.radius))
                } else {
                    placed_node
                        .own_clip
                        .map(|rect| (rect, placed_node.own_clip_radius))
                }
            });
            let effective_rect = match (placed_node.clip, widget_self_clip) {
                (Some(inherited), Some((local, _))) => Some((intersect(inherited, local), 0.0)),
                (Some(inherited), None) => Some((inherited, placed_node.clip_radius)),
                (None, Some(local)) => Some(local),
                (None, None) => None,
            };
            let axis_aligned_clips = clip_ancestors
                .iter()
                .all(|clip| clip.transform == Affine::IDENTITY.as_coeffs())
                && (widget_self_clip.is_none()
                    || css_transforms[&placed_node.node_id] == Affine::IDENTITY);
            let effective =
                axis_aligned_clips
                    .then_some(effective_rect)
                    .flatten()
                    .map(|(rect, radius)| wabou_devtools::DebugClip {
                        node_id: id,
                        kind: "effective".into(),
                        coordinate_space: "window-logical".into(),
                        rect: debug_rect(rect),
                        radius,
                        transform: Affine::IDENTITY.as_coeffs(),
                    });
            nodes.push(wabou_devtools::DebugNode {
                id,
                parent_id: placed_node
                    .parent_node_id
                    .and_then(|parent| self.node_to_solid.get(&parent).copied()),
                tag,
                text: placed_node
                    .paint
                    .text
                    .as_deref()
                    .map(|text| text.chars().take(4096).collect()),
                classes,
                matched_rules,
                attrs,
                rect: wabou_devtools::Rect {
                    x: x0,
                    y: y0,
                    width: x1 - x0,
                    height: y1 - y0,
                },
                content_rect: wabou_devtools::Rect {
                    x: cx,
                    y: cy,
                    width: cw,
                    height: ch,
                },
                listeners,
                widget: self
                    .widgets
                    .contains_key(&placed_node.node_id)
                    .then(|| "native".into()),
                clip: wabou_devtools::DebugClipInfo {
                    widget_local,
                    chain: clip_ancestors,
                    effective,
                    static_transform: static_transform.as_coeffs(),
                    runtime_transform: placed_node
                        .paint
                        .runtime_transform
                        .map(|matrix| matrix.map(f64::from)),
                    border_transform: css_transforms[&placed_node.node_id].as_coeffs(),
                    scene_transform: content_transform.as_coeffs(),
                    device_scale: self.device_scale,
                },
                computed: wabou_devtools::DebugComputedStyle {
                    display: layout.map(|style| format!("{:?}", style.display)),
                    position: layout.map(|style| format!("{:?}", style.position)),
                    overflow_x: layout.map(|style| format!("{:?}", style.overflow.x)),
                    overflow_y: layout.map(|style| format!("{:?}", style.overflow.y)),
                    font_size: placed_node.paint.font_size,
                    font_weight: placed_node.paint.font_weight,
                    wrap_text: placed_node.paint.wrap_text,
                    opacity: placed_node.paint.opacity,
                    pointer_events: placed_node.paint.pointer_events,
                    z_index: placed_node.paint.z_index,
                    overlay_plane: format!("{:?}", placed_node.paint.overlay_plane),
                    scrollbar_opacity: placed_node.scroll.opacity,
                    text_color: format!("{:x}", placed_node.paint.text_color.to_rgba8()),
                    background: placed_node
                        .paint
                        .background
                        .map(|color| format!("{:x}", color.to_rgba8())),
                },
            });
        }
        let snapshot = wabou_devtools::DebugSnapshot {
            status: wabou_devtools::DebugStatus {
                protocol_version: wabou_devtools::PROTOCOL_VERSION,
                pid: std::process::id(),
                revision: self.debug_revision,
                viewport_width: self.last_viewport.0,
                viewport_height: self.last_viewport.1,
                device_scale: self.device_scale,
                node_count: nodes.len(),
                focused_node: self.focused_target,
                hovered_node: self.hovered_target,
            },
            nodes,
        };
        drop(atoms);
        if let Ok(mut state) = state.write() {
            state.publish(snapshot);
        }
    }
}

impl Applier {
    fn cancel_pointer_gesture(&mut self, pointer: wabou_shell::PointerEvent) -> bool {
        self.pointer_position = (pointer.position.x, pointer.position.y);
        self.pointer_buttons = pointer.buttons;
        self.next_text_selection_scroll = None;
        if self.pointer_down_target.is_some() {
            self.last_text_click = None;
        }
        let old_active = self.pointer_down_target.take();
        self.pointer_down_position = None;
        self.pointer_dragged = false;
        let target = old_active.or_else(|| self.hit_test(pointer.position.x, pointer.position.y));
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
        if self.pointer_down_target.is_none() {
            self.pointer_buttons = 0;
            return false;
        }
        self.cancel_pointer_gesture(wabou_shell::PointerEvent {
            phase: PointerPhase::Cancel,
            position: wabou_shell::Point {
                x: self.pointer_position.0,
                y: self.pointer_position.1,
            },
            button: None,
            buttons: 0,
            modifiers: Modifiers::empty(),
        })
    }
}

impl FrameSource for Applier {
    fn set_device_scale(&mut self, scale: f64) {
        self.device_scale = scale.max(f64::EPSILON);
    }

    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode> {
        self.invalidation.remove(InvalidationFlags::TICK);
        self.js.take_async_wake();
        self.js.poll_async_runtime();

        // Drain fonts queued by the Host API (typically once at boot):
        // register each blob into the shared text FontContext + clear the layout
        // cache so subsequent text measurement picks up the new family.
        if let Some(pf) = self.pending_fonts.clone() {
            let queued = std::mem::take(&mut *pf.borrow_mut());
            for bytes in queued {
                tcx.load_font(bytes);
            }
        }

        // Drain a stylesheet pushed through the private host ABI (the UnoCSS Vite
        // plugin's virtual module): replace the Style IR + re-resolve every node.
        // Clone the Rc out so the mutable self borrows below don't alias it.
        if let Some(p) = self.pending_css.clone()
            && let Some(update) = p.borrow_mut().take()
        {
            match update {
                StylesheetUpdate::Ir(sheet) if sheet.validate().is_ok() => {
                    for diagnostic in &sheet.diagnostics {
                        tracing::warn!(target: "stylesheet", %diagnostic);
                    }
                    // Build the class→rules index (interning each rule's
                    // class_name so node-side class atoms match by identity)
                    // so recompute_node_now matches in O(C) not O(R).
                    let (rule_index, universal_rules) = {
                        let mut atoms = self.atoms.borrow_mut();
                        let mut rule_index: HashMap<Atom, Vec<usize>> = HashMap::new();
                        let mut universal_rules = Vec::new();
                        for (idx, rule) in sheet.rules.iter().enumerate() {
                            for declaration in &rule.declarations {
                                atoms.intern(&declaration.property);
                            }
                            if rule.class_name == "*" {
                                universal_rules.push(idx);
                            } else {
                                rule_index
                                    .entry(atoms.intern(&rule.class_name))
                                    .or_default()
                                    .push(idx);
                            }
                        }
                        (rule_index, universal_rules)
                    };
                    self.style_ir = Some(sheet);
                    self.rule_index = rule_index;
                    self.universal_rules = universal_rules;
                    self.warned_utility_classes.clear();
                    self.warned_ir_properties.clear();
                }
                StylesheetUpdate::Ir(sheet) => {
                    tracing::error!(
                        version = sheet.version,
                        supported = style_ir::VERSION,
                        "invalid or unsupported Style IR"
                    );
                }
            }
            self.recompute_all();
        }

        // Drain Vite HMR updates (from the background websocket client) before
        // the tick so re-imported module effects land in this frame's flush.
        // Style IR is already applied above via pending_css (same frame as the
        // virtual:wabou-stylesheet JS update when both fire together).
        let hmr = self.drain_hmr_batch();
        if !matches!(hmr, HmrDrainResult::Idle) {
            self.last_hmr_result = hmr;
        }
        self.has_hmr_pending.store(false, Ordering::Release);

        // Host application messages before tick so subscribe
        // handlers can update signals before this frame's rAF flush.
        self.drain_host_messages();

        // One rAF round-trip: runs queued rAF callbacks (Solid effects re-emit
        // ops), flushes the writer → __wabou_flush lands bytes here. Timed so
        // the host overlay can show the QuickJS portion of build_frame.
        let js_t0 = std::time::Instant::now();
        let (bytes, has_raf) = match self.js.tick() {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(target: "bridge", "JS tick failed: {e:?}");
                self.has_raf = false;
                return Vec::new();
            }
        };
        let js_tick_ms = js_t0.elapsed().as_secs_f64() * 1000.0;
        self.js_tick_ema = self.js_tick_ema * 0.9 + js_tick_ms * 0.1;
        self.last_viewport = (width, height);
        self.has_raf = has_raf;
        if !bytes.is_empty() {
            match decode_frame(&bytes) {
                Ok(frame) => {
                    if let Some(state) = &self.debug_state
                        && let Ok(mut state) = state.write()
                    {
                        state.push_frame(wabou_devtools::DebugFrame {
                            direction: "jsToHost".into(),
                            sequence: u64::from(frame.seq),
                            byte_len: bytes.len(),
                            record_count: frame.ops.len(),
                            bytes_hex: Some(wabou_devtools::bytes_hex(&bytes, 4096)),
                        });
                    }
                    self.apply_frame(&frame)
                }
                Err(e) => tracing::error!(target: "bridge", "decode frame failed: {e}"),
            }
        }
        // Arm rquickjs's async scheduler after this tick may have started new
        // work. Pending IO keeps its waker and does not imply animation.
        self.js.poll_async_runtime();

        let selection_scrolled = self.tick_text_selection_autoscroll();
        // Only re-inherit when a change can affect inherited content styles.
        // Per-frame non-inherited animation sets LAYOUT but not INHERIT, so
        // this O(N) pass remains skipped for those frames.
        if self.invalidation.contains(InvalidationFlags::INHERIT) {
            self.inherit();
            self.invalidation.remove(InvalidationFlags::INHERIT);
        }
        self.sync_widget_styles();
        self.measure_widgets(tcx);
        let viewport = (width, height);
        let viewport_changed = self.layout_viewport != Some(viewport);
        let semantic_layout_dirty =
            self.invalidation.contains(InvalidationFlags::LAYOUT) || viewport_changed;
        let mut placed =
            if self.invalidation.contains(InvalidationFlags::LAYOUT) || viewport_changed {
                // A root percentage has no containing block in taffy and resolves
                // to zero. Only update it when the viewport changes: set_style
                // invalidates Taffy's retained layout cache.
                if viewport_changed && let Ok(style) = self.tree.style(self.root) {
                    let mut style = style.clone();
                    style.size.width = taffy::Dimension::length(width as f32);
                    style.size.height = taffy::Dimension::length(height as f32);
                    let _ = self.tree.set_style(self.root, style);
                }
                let mut placed = layout::compute_and_walk_with_scroll(
                    &mut self.tree,
                    self.root,
                    width as f32,
                    height as f32,
                    tcx,
                    &self.scroll_offsets,
                );
                self.invalidation.remove(InvalidationFlags::LAYOUT);
                self.layout_viewport = Some(viewport);
                let resize_changed = self.dispatch_resize_changes();
                self.invalidation
                    .set(InvalidationFlags::TICK, resize_changed);
                self.paint_widgets(&mut placed, tcx);
                placed
            } else {
                let mut placed =
                    layout::flatten_with_scroll(&self.tree, self.root, &self.scroll_offsets);
                self.paint_widgets(&mut placed, tcx);
                placed
            };
        self.update_scrollbar_visuals(&mut placed);
        self.placed_rects.clear();
        self.placed_rects
            .extend(placed.iter().map(|placed| (placed.node_id, placed.rect)));
        self.rebuild_hit_geometry(&placed);
        self.publish_layout_metrics(&placed, width, height);
        self.prepare_text_selection(&mut placed, tcx);
        if selection_scrolled {
            let target = self.hit_test(self.pointer_position.0, self.pointer_position.1);
            self.extend_text_selection(target, self.pointer_position.0, self.pointer_position.1);
            self.prepare_text_selection(&mut placed, tcx);
        }
        if self.pointer_buttons & 1 == 0 {
            self.sync_text_selection_change();
        }
        if self.semantics_enabled && (self.semantics_dirty || semantic_layout_dirty) {
            self.rebuild_semantic_snapshot(&placed);
            self.semantics_dirty = false;
        }
        // After paint applied pending edits, sync widget values → JS.
        self.flush_value_sync();
        self.publish_debug_snapshot(&placed);
        placed
    }

    fn base_color(&self) -> Color {
        self.base_color
    }

    fn set_semantics_enabled(&mut self, enabled: bool) {
        if enabled && !self.semantics_enabled {
            self.semantics_dirty = true;
        }
        self.semantics_enabled = enabled;
        if !enabled {
            self.semantic_snapshot = Arc::new(SemanticSnapshot::default());
        }
    }

    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        self.semantics_enabled
            .then(|| self.semantic_snapshot.clone())
    }

    fn handle_semantic_action(&mut self, action: SemanticAction) -> bool {
        let target = match action {
            SemanticAction::Click { target }
            | SemanticAction::Focus { target }
            | SemanticAction::Blur { target } => u32::try_from(target).ok(),
        };
        let Some(target) = target.filter(|target| self.solid_to_node.contains_key(target)) else {
            return false;
        };
        if let Some(modal) = self.semantic_snapshot.modal_root {
            let Some(modal_node) = u32::try_from(modal)
                .ok()
                .and_then(|modal| self.solid_to_node.get(&modal).copied())
            else {
                return false;
            };
            if !self
                .solid_to_node
                .get(&target)
                .is_some_and(|node| self.is_logical_descendant(*node, modal_node))
            {
                return false;
            }
        }
        match action {
            SemanticAction::Click { .. } => {
                self.dispatch_pointer(target, event::CLICK, None, Modifiers::empty())
            }
            SemanticAction::Focus { .. } => {
                let changed = self.focused_target != Some(target);
                self.set_focused_target(Some(target));
                changed
            }
            SemanticAction::Blur { .. } => {
                let changed = self.focused_target == Some(target);
                if changed {
                    self.set_focused_target(None);
                }
                changed
            }
        }
    }

    fn paint_debug_overlay(
        &mut self,
        scene: &mut Scene,
        placed: &[PlacedNode],
        tcx: &mut TextContext,
        device_scale: f64,
    ) {
        let Some(state) = &self.debug_state else {
            return;
        };
        let Ok(state) = state.read() else { return };
        let overlay = state.overlay();
        if !overlay.is_enabled() {
            return;
        }
        let device = Affine::scale(device_scale);
        let hovered = self.hovered_target;
        let mut clips = HashSet::new();

        for node in placed {
            let Some(&solid_id) = self.node_to_solid.get(&node.node_id) else {
                continue;
            };
            let [x0, y0, x1, y1] = node.rect;
            let rect = Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64);
            if overlay.layout {
                scene.stroke(
                    &Stroke::new(1.0),
                    device,
                    Color::from_rgba8(56, 189, 248, 190),
                    None,
                    &rect,
                );
            }
            if overlay.clips
                && let Some(clip) = node.clip
                && clips.insert(clip.map(f32::to_bits))
            {
                scene.stroke(
                    &Stroke::new(1.5),
                    device,
                    Color::from_rgba8(251, 146, 60, 220),
                    None,
                    &Rect::new(
                        clip[0] as f64,
                        clip[1] as f64,
                        clip[2] as f64,
                        clip[3] as f64,
                    ),
                );
            }

            let is_hit = overlay.hit_target && hovered == Some(solid_id);
            let is_selected = overlay.selected_node == Some(solid_id);
            if !is_hit && !is_selected {
                continue;
            }
            let accent = if is_selected {
                Color::from_rgba8(168, 85, 247, 255)
            } else {
                Color::from_rgba8(244, 63, 94, 255)
            };
            scene.fill(
                Fill::NonZero,
                device,
                Color::from_rgba8(244, 63, 94, 25),
                None,
                &rect,
            );
            scene.stroke(&Stroke::new(2.0), device, accent, None, &rect);

            let atoms = self.atoms.borrow();
            let tag = self
                .declared
                .get(&node.node_id)
                .and_then(|declared| declared.tag)
                .and_then(|tag| atoms.resolve(tag))
                .unwrap_or("#text");
            let label: Arc<str> = format!("{tag}#{solid_id}").into();
            drop(atoms);
            let layout = layout_text_styled(
                tcx,
                label,
                11.0,
                600.0,
                None,
                TextAlign::Start,
                [255, 255, 255, 255],
                Arc::from([]),
                None,
                None,
            );
            let label_width = layout.width() as f64 + 8.0;
            let label_height = layout.height() as f64 + 4.0;
            let label_y = (y0 as f64 - label_height).max(0.0);
            let label_rect = Rect::new(
                x0 as f64,
                label_y,
                x0 as f64 + label_width,
                label_y + label_height,
            );
            scene.fill(Fill::NonZero, device, accent, None, &label_rect);
            let glyphs = tcx.glyph_scene_scaled(&layout, device_scale);
            scene.append(
                &glyphs,
                Some(
                    device
                        * Affine::translate((x0 as f64 + 4.0, label_y + 2.0))
                        * Affine::scale(device_scale.recip()),
                ),
            );
        }
    }

    fn push_frame_stats(&mut self, stats: &FrameStats) {
        if let Some(cell) = &self.frame_stats {
            // The app fills build_frame/scene/present/node_count; fold in the
            // QuickJS tick EMA + last viewport the applier measured.
            let mut s = *stats;
            s.js_tick_ms = self.js_tick_ema;
            s.viewport_w = self.last_viewport.0;
            s.viewport_h = self.last_viewport.1;
            *cell.borrow_mut() = Some(s);
        }
    }

    fn has_anim(&self) -> bool {
        self.has_raf
            || self.has_hmr_pending.load(Ordering::Acquire)
            || self.host_msg_inbox.has_pending()
            || self.js.has_async_wake()
            || self.invalidation.contains(InvalidationFlags::TICK)
    }

    fn animation_deadline(&self) -> Option<Instant> {
        let now = Instant::now();
        let scrollbar_deadline = self.scrollbar_activity.values().map(|started| {
            let fade_start = *started + SCROLLBAR_FADE_DELAY;
            if now < fade_start {
                fade_start
            } else {
                now + Duration::from_millis(16)
            }
        });
        self.widgets
            .values()
            .filter_map(|widget| widget.animation_deadline())
            .chain(self.next_text_selection_scroll)
            .chain(scrollbar_deadline)
            .min()
    }

    fn set_wake_callback(&mut self, wake: WakeCallback) {
        if let Some(state) = &self.debug_state
            && let Ok(mut state) = state.write()
        {
            state.set_wake(wake.clone());
        }
        self.js.set_wake_callback(wake.clone());
        for widget in self.widgets.values_mut() {
            widget.set_wake_callback(wake.clone());
        }
        self.host_msg_inbox.set_wake(wake.clone());
        *self.host_action_wake.borrow_mut() = Some(wake.clone());
        self.wake_callback = Some(wake);
    }

    fn poll_async(&mut self) -> bool {
        let was_woken = self.js.take_async_wake();
        self.js.poll_async_runtime();
        let mut widget_woken = false;
        let mut host_actions = Vec::new();
        let mut node_events = Vec::new();
        for (node, widget) in &mut self.widgets {
            widget_woken |= widget.poll_async();
            while let Some(action) = widget.take_host_action() {
                host_actions.push((*node, action));
            }
            while let Some(event) = widget.take_node_event() {
                node_events.push((*node, event));
            }
        }
        for (node, action) in host_actions {
            self.enqueue_widget_host_action(node, action);
        }
        for (node, event) in node_events {
            let Some(target) = self.solid_id_for_node(node) else {
                continue;
            };
            widget_woken |= self.dispatch_json(target, event.event_code, &event.json);
        }
        let screenshot_pending = self
            .debug_state
            .as_ref()
            .and_then(|state| state.read().ok())
            .is_some_and(|state| state.has_screenshot_request());
        let overlay_changed = self
            .debug_state
            .as_ref()
            .and_then(|state| state.write().ok())
            .is_some_and(|mut state| state.take_overlay_change());
        widget_woken || was_woken || screenshot_pending || overlay_changed
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.pending_host_actions.borrow_mut().pop_front()
    }

    fn complete_host_action(&mut self, result: wabou_shell::HostActionResult) {
        match result {
            wabou_shell::HostActionResult::Clipboard { request_id, text } => {
                if self
                    .pending_js_clipboard_requests
                    .borrow_mut()
                    .remove(&request_id)
                {
                    complete_js_clipboard(&self.js, request_id, text, true);
                    return;
                }
                let Some((node, widget_request_id)) = self.host_action_routes.remove(&request_id)
                else {
                    return;
                };
                if let Some(widget) = self.widgets.get_mut(&node) {
                    widget.complete_host_action(wabou_shell::HostActionResult::Clipboard {
                        request_id: widget_request_id,
                        text,
                    });
                }
            }
            wabou_shell::HostActionResult::ClipboardWrite {
                request_id,
                success,
            } => {
                if self
                    .pending_js_clipboard_requests
                    .borrow_mut()
                    .remove(&request_id)
                {
                    complete_js_clipboard(&self.js, request_id, None, success);
                }
            }
        }
    }

    fn take_screenshot_request(&mut self) -> Option<std::path::PathBuf> {
        self.debug_state
            .as_ref()?
            .write()
            .ok()?
            .take_screenshot_request()
    }

    fn complete_screenshot(&mut self, result: Result<std::path::PathBuf, String>) {
        if let Some(state) = &self.debug_state
            && let Ok(mut state) = state.write()
        {
            state.complete_screenshot(result);
        }
    }

    fn handle_event(&mut self, input: UiEvent) -> EventResponse {
        if let UiEvent::WindowMetrics(metrics) = &input {
            let payload = serde_json::json!({
                "windowId": metrics.window_id,
                "logicalWidth": metrics.logical_width,
                "logicalHeight": metrics.logical_height,
                "physicalWidth": metrics.physical_width,
                "physicalHeight": metrics.physical_height,
                "scaleFactor": metrics.scale_factor,
                "maximized": metrics.maximized,
                "focused": metrics.focused,
            })
            .to_string();
            let event = HostEvent::Application(crate::host_msg::HostMsg::str(
                "wabou:window-metrics",
                payload,
            ));
            let handled = self.js.dispatch_host_frame(&[event]).is_ok();
            return EventResponse {
                handled,
                request_redraw: handled,
                ..EventResponse::IGNORED
            };
        }
        if matches!(
            &input,
            UiEvent::Key(key) if key.phase == KeyPhase::Down
        ) || matches!(
            &input,
            UiEvent::TextInput(_) | UiEvent::Paste(_) | UiEvent::Wheel(_)
        ) || matches!(
            &input,
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Down
                    && pointer.button != Some(PointerButton::Primary)
        ) {
            self.last_text_click = None;
        }
        // Application shortcuts get first refusal before a focused Rust
        // widget consumes the key. This is the native equivalent of a
        // cancellable keydown boundary: preventDefault keeps shortcuts like
        // Cmd+T out of a terminal PTY while ordinary keys continue unchanged.
        let keydown_dispatched = if let UiEvent::Key(key) = &input
            && key.phase == KeyPhase::Down
            && let Some(target) = self.focused_target
        {
            let payload = key_event_payload(key);
            let (dispatched, prevented) =
                self.dispatch_cancellable_json(target, event::KEYDOWN, payload);
            if prevented {
                return EventResponse {
                    handled: true,
                    request_redraw: true,
                    consume_key_text: true,
                    text_input: None,
                    clipboard: None,
                };
            }
            dispatched
        } else {
            false
        };
        // Keyboard and committed text belong to the focused widget. Pointer
        // focus is established in the pointer-down branch before delivery.
        let widget_response = if matches!(
            input,
            UiEvent::Key(_) | UiEvent::TextInput(_) | UiEvent::Paste(_)
        ) && let Some(target) = self.focused_target
        {
            self.handle_widget_event(target, &input)
        } else {
            None
        };
        if widget_response.is_none()
            && let UiEvent::Key(key) = &input
            && key.matches_standard_shortcut(wabou_shell::StandardShortcut::Copy)
            && let Some(text) = self.selected_text()
        {
            return EventResponse {
                handled: true,
                request_redraw: false,
                consume_key_text: false,
                text_input: None,
                clipboard: Some(wabou_shell::ClipboardRequest::Write(text)),
            };
        }
        if widget_response.is_none()
            && let UiEvent::Key(key) = &input
            && key.matches_standard_shortcut(wabou_shell::StandardShortcut::SelectAll)
            && self.select_all_text()
        {
            self.sync_text_selection_change();
            return EventResponse {
                handled: true,
                request_redraw: true,
                consume_key_text: false,
                text_input: None,
                clipboard: None,
            };
        }

        let handled = match input {
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Move => {
                let (x, y) = (pointer.position.x, pointer.position.y);
                self.pointer_buttons = pointer.buttons;
                self.pointer_position = (x, y);
                let hovered_scrollbar = self
                    .scrollbar_at(x, y)
                    .map(|(node, target)| (node, target.axis));
                let scrollbar_hover_changed = hovered_scrollbar != self.hovered_scrollbar;
                self.hovered_scrollbar = hovered_scrollbar;
                if let Some((node, _)) = hovered_scrollbar {
                    self.scrollbar_activity.insert(node, Instant::now());
                }
                if self.scrollbar_drag.is_some() {
                    let changed = self.drag_scrollbar(x, y);
                    return EventResponse {
                        handled: true,
                        request_redraw: changed || scrollbar_hover_changed,
                        ..EventResponse::IGNORED
                    };
                }
                if pointer.buttons & 1 != 0
                    && let Some((down_x, down_y)) = self.pointer_down_position
                {
                    let dx = x - down_x;
                    let dy = y - down_y;
                    self.pointer_dragged |= dx * dx + dy * dy > CLICK_DRAG_THRESHOLD_SQUARED;
                }
                let target = self.hit_test(x, y);
                let mut changed = scrollbar_hover_changed;
                if let Some(captured) = self.pointer_down_target
                    && let Some(response) =
                        self.handle_widget_event(captured, &UiEvent::Pointer(pointer))
                {
                    changed |= response.handled || response.request_redraw;
                }
                if pointer.buttons & 1 != 0 {
                    changed |= self.extend_text_selection(target, x, y);
                    self.arm_text_selection_autoscroll();
                }
                if target != self.hovered_target {
                    if let Some(old) = self.hovered_target {
                        changed |= self.dispatch_pointer(
                            old,
                            event::POINTERLEAVE,
                            None,
                            pointer.modifiers,
                        );
                    }
                    if let Some(new) = target {
                        changed |= self.dispatch_pointer(
                            new,
                            event::POINTERENTER,
                            None,
                            pointer.modifiers,
                        );
                    }
                    self.hovered_target = target;
                }
                if let Some(target) = target {
                    changed |=
                        self.dispatch_pointer(target, event::POINTERMOVE, None, pointer.modifiers);
                }
                changed
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Down => {
                let (x, y) = (pointer.position.x, pointer.position.y);
                let button = pointer.button.unwrap_or(PointerButton::Primary);
                self.pointer_position = (x, y);
                self.pointer_buttons = pointer.buttons;
                if button == PointerButton::Primary
                    && let Some((node, target)) = self.scrollbar_at(x, y)
                    && let Some(hit) = self.scrollbar_hits.iter().find(|hit| hit.node == node)
                {
                    self.scrollbar_activity.insert(node, Instant::now());
                    if target.part != ScrollbarPart::Thumb {
                        let index = usize::from(target.axis == ScrollAxis::Vertical);
                        let viewport = match target.axis {
                            ScrollAxis::Horizontal => {
                                hit.placed.scroll.port[2] - hit.placed.scroll.port[0]
                            }
                            ScrollAxis::Vertical => {
                                hit.placed.scroll.port[3] - hit.placed.scroll.port[1]
                            }
                        };
                        let direction = if target.part == ScrollbarPart::TrackBefore {
                            -1.0
                        } else {
                            1.0
                        };
                        let offset = self.scroll_offsets.entry(node).or_insert([0.0; 2]);
                        offset[index] = (offset[index] + direction * viewport)
                            .clamp(0.0, hit.placed.scroll.range[index]);
                        self.semantics_dirty = true;
                        return Self::response(true);
                    }
                    let local = hit.transform.inverse() * Point::new(x, y);
                    self.scrollbar_drag = Some(ScrollbarDrag {
                        node,
                        axis: target.axis,
                        last_position: match target.axis {
                            ScrollAxis::Horizontal => local.x,
                            ScrollAxis::Vertical => local.y,
                        },
                    });
                    return Self::response(true);
                }
                let target = self.hit_test(x, y);
                self.pointer_down_target = target;
                self.pointer_down_position = Some((x, y));
                self.pointer_dragged = false;
                let mut changed = self.set_focused_target(target);
                if button == PointerButton::Primary {
                    self.next_text_selection_scroll = None;
                    changed |= target.is_some_and(|target| {
                        self.begin_text_selection(target, x, y, pointer.modifiers)
                    });
                }
                if let Some(target) = target
                    && let Some(mut response) =
                        self.handle_widget_event(target, &UiEvent::Pointer(pointer))
                {
                    response.text_input = Some(self.is_text_input_target(target));
                    return response;
                }
                let handled = changed
                    | target.is_some_and(|target| {
                        self.dispatch_pointer(
                            target,
                            event::POINTERDOWN,
                            Some(button),
                            pointer.modifiers,
                        )
                    });
                return EventResponse {
                    handled,
                    request_redraw: handled,
                    consume_key_text: false,
                    text_input: Some(
                        target.is_some_and(|target| self.is_text_input_target(target)),
                    ),
                    clipboard: None,
                };
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Up => {
                let (x, y) = (pointer.position.x, pointer.position.y);
                let button = pointer.button.unwrap_or(PointerButton::Primary);
                self.pointer_position = (x, y);
                self.pointer_buttons = pointer.buttons;
                if self.scrollbar_drag.take().is_some() {
                    return Self::response(true);
                }
                if button == PointerButton::Primary
                    && let Some((down_x, down_y)) = self.pointer_down_position
                {
                    let dx = x - down_x;
                    let dy = y - down_y;
                    self.pointer_dragged |= dx * dx + dy * dy > CLICK_DRAG_THRESHOLD_SQUARED;
                }
                let target = self.hit_test(x, y);
                let captured = self.pointer_down_target;
                let mut changed = captured.is_some_and(|captured| {
                    self.handle_widget_event(captured, &UiEvent::Pointer(pointer))
                        .is_some_and(|response| response.handled || response.request_redraw)
                });
                if button == PointerButton::Primary {
                    changed |= self.extend_text_selection(target, x, y);
                    self.next_text_selection_scroll = None;
                    if self.pointer_dragged {
                        self.last_text_click = None;
                    }
                }
                changed |= target.is_some_and(|target| {
                    self.dispatch_pointer(target, event::POINTERUP, Some(button), pointer.modifiers)
                });
                if let Some(target) = target
                    && button == PointerButton::Primary
                    && !self.pointer_dragged
                    && Some(target) == self.pointer_down_target
                {
                    let mut data = [0.0; event_data::LEN];
                    data[event_data::CLIENT_X as usize] = self.pointer_position.0;
                    data[event_data::CLIENT_Y as usize] = self.pointer_position.1;
                    data[event_data::BUTTON as usize] = Self::web_button(button) as f64;
                    data[event_data::BUTTONS as usize] =
                        Self::web_buttons(self.pointer_buttons) as f64;
                    data[event_data::MODS as usize] = pointer.modifiers.bits() as f64;
                    let (dispatched, prevented) =
                        self.dispatch_cancellable_numeric(target, event::CLICK, data);
                    changed |= dispatched;
                    if !prevented {
                        changed |= self.open_link_default(target);
                    }
                }
                self.pointer_down_target.take();
                self.pointer_down_position = None;
                self.pointer_dragged = false;
                changed |= self.sync_text_selection_change();
                changed
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Cancel => {
                if self.scrollbar_drag.take().is_some() {
                    return Self::response(true);
                }
                self.cancel_pointer_gesture(pointer)
            }
            UiEvent::Wheel(wheel) => {
                self.pointer_position = (wheel.position.x, wheel.position.y);
                // A wheel gesture remains targeted at the element under the
                // last pointer move. Re-running a full-tree hit test for every
                // high-frequency trackpad sample is both redundant and a
                // major source of input latency. BUT the cached hit can go
                // stale if the node under the pointer was dropped (e.g. a
                // virtualised slot recycled at a list edge) — without a
                // pointer move to refresh it, wheel/scroll would silently
                // no-op until the user wiggles the mouse. Lazily validate +
                // refresh the cache here so wheel keeps dispatching.
                if !self
                    .hovered_target
                    .is_some_and(|t| self.solid_to_node.contains_key(&t))
                {
                    self.hovered_target =
                        self.hit_test(self.pointer_position.0, self.pointer_position.1);
                }
                let Some(target) = self.hovered_target else {
                    return EventResponse::IGNORED;
                };
                // Route wheel to a Rust widget at the hit target first (e.g.
                // the terminal's scrollback) — before JS WHEEL listeners or
                // native overflow scroll. Non-widget targets have no widget,
                // so this returns None and falls through unchanged.
                if let Some(response) = self.handle_widget_event(target, &UiEvent::Wheel(wheel)) {
                    return response;
                }
                let mut data = [0.0; event_data::LEN];
                data[event_data::CLIENT_X as usize] = self.pointer_position.0;
                data[event_data::CLIENT_Y as usize] = self.pointer_position.1;
                data[event_data::MODS as usize] = wheel.modifiers.bits() as f64;
                data[event_data::DELTA_X as usize] = wheel.delta_x;
                data[event_data::DELTA_Y as usize] = wheel.delta_y;
                let (dispatched, prevented) =
                    self.dispatch_cancellable_numeric(target, event::WHEEL, data);
                let scrolled = !prevented
                    && self.scroll_nearest(target, wheel.delta_x as f32, wheel.delta_y as f32);
                dispatched || scrolled
            }
            UiEvent::Key(key) if key.phase == KeyPhase::Down => keydown_dispatched,
            UiEvent::Key(key) if key.phase == KeyPhase::Up => {
                self.focused_target.is_some_and(|target| {
                    let payload = key_event_payload(&key);
                    self.dispatch_json(target, event::KEYUP, &payload)
                })
            }
            UiEvent::TextInput(text) => self.focused_target.is_some_and(|target| {
                let payload = serde_json::json!({ "data": text }).to_string();
                self.dispatch_json(target, event::IMECOMMIT, &payload)
            }),
            UiEvent::Paste(text) => self.focused_target.is_some_and(|target| {
                let payload = serde_json::json!({ "data": text }).to_string();
                self.dispatch_json(target, event::IMECOMMIT, &payload)
            }),
            UiEvent::Focus(focused) => {
                let mut changed = if focused {
                    false
                } else {
                    self.last_text_click = None;
                    self.cancel_active_pointer_gesture()
                };
                changed |= self.set_window_focused(focused);
                return EventResponse {
                    handled: changed,
                    request_redraw: changed,
                    consume_key_text: false,
                    text_input: Some(
                        focused
                            && self
                                .focused_target
                                .is_some_and(|target| self.is_text_input_target(target)),
                    ),
                    clipboard: None,
                };
            }
            UiEvent::Pointer(_) | UiEvent::Key(_) | UiEvent::WindowMetrics(_) => false,
        };
        if let Some(widget) = widget_response {
            EventResponse {
                handled: widget.handled || handled,
                request_redraw: widget.request_redraw || handled,
                consume_key_text: widget.consume_key_text,
                text_input: widget.text_input,
                clipboard: widget.clipboard,
            }
        } else {
            Self::response(handled)
        }
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
mod tests {
    use super::*;

    #[test]
    fn key_payload_keeps_physical_modifiers_separate_from_primary() {
        let platform_primary = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        };
        let event = wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "t".into(),
            key_without_modifiers: "t".into(),
            code: "KeyT".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: platform_primary,
            repeat: false,
        };
        let payload: serde_json::Value = serde_json::from_str(&key_event_payload(&event)).unwrap();

        assert_eq!(payload["mods"], platform_primary.bits());
        assert_eq!(payload["primary"], true);

        let mut physical_control = event;
        physical_control.modifiers = if cfg!(target_os = "macos") {
            Modifiers::CONTROL
        } else {
            Modifiers::META
        };
        let payload: serde_json::Value =
            serde_json::from_str(&key_event_payload(&physical_control)).unwrap();
        assert_eq!(payload["mods"], physical_control.modifiers.bits());
        assert_eq!(payload["primary"], false);
    }

    struct HostActionWidget(Option<wabou_shell::HostAction>);

    struct EventHostActionWidget(Option<wabou_shell::HostAction>);

    struct UnmountActionWidget(Option<wabou_shell::HostAction>);

    struct LifecycleWidget(Arc<std::sync::Mutex<Vec<&'static str>>>);

    struct NodeEventWidget(Option<crate::widget::WidgetNodeEvent>);

    struct ClipboardReadWidget {
        action: Option<wabou_shell::HostAction>,
        completed: Arc<std::sync::Mutex<Vec<wabou_shell::HostActionResult>>>,
    }

    struct WheelCaptureWidget(Arc<std::sync::Mutex<Vec<Point>>>);

    struct KeyCaptureWidget(Arc<std::sync::Mutex<usize>>);

    struct MeasuringWidget([f32; 2]);

    struct StyleAwareMeasuringWidget(Arc<std::sync::Mutex<Vec<&'static str>>>);

    impl crate::widget::Widget for MeasuringWidget {
        fn measure(&mut self, _tcx: &mut TextContext) -> Option<[f32; 2]> {
            Some(self.0)
        }

        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }
    }

    impl crate::widget::Widget for StyleAwareMeasuringWidget {
        fn style_changed(&mut self, _style: &crate::widget::WidgetStyle) {
            self.0.lock().unwrap().push("style");
        }

        fn measure(&mut self, _tcx: &mut TextContext) -> Option<[f32; 2]> {
            self.0.lock().unwrap().push("measure");
            Some([100.0, 40.0])
        }

        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }
    }

    impl crate::widget::Widget for HostActionWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn poll_async(&mut self) -> bool {
            self.0.is_some()
        }

        fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
            self.0.take()
        }
    }

    impl crate::widget::Widget for EventHostActionWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn handle_event(&mut self, _event: &UiEvent) -> crate::widget::WidgetEventResult {
            crate::widget::WidgetEventResult::HANDLED
        }

        fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
            self.0.take()
        }
    }

    impl crate::widget::Widget for UnmountActionWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn unmount(&mut self) {
            self.0 = Some(wabou_shell::HostAction::SetWindowTitle(None));
        }

        fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
            self.0.take()
        }
    }

    impl crate::widget::Widget for LifecycleWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn focus_changed(&mut self, focused: bool) {
            self.0
                .lock()
                .unwrap()
                .push(if focused { "focus-in" } else { "focus-out" });
        }

        fn accepts_focus(&self) -> bool {
            true
        }

        fn handle_event(&mut self, event: &UiEvent) -> crate::widget::WidgetEventResult {
            if matches!(event, UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Cancel) {
                self.0.lock().unwrap().push("pointer-cancel");
                crate::widget::WidgetEventResult::HANDLED
            } else {
                crate::widget::WidgetEventResult::IGNORED
            }
        }

        fn unmount(&mut self) {
            self.0.lock().unwrap().push("unmount");
        }
    }

    impl crate::widget::Widget for NodeEventWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn poll_async(&mut self) -> bool {
            self.0.is_some()
        }

        fn take_node_event(&mut self) -> Option<crate::widget::WidgetNodeEvent> {
            self.0.take()
        }
    }

    impl crate::widget::Widget for ClipboardReadWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn poll_async(&mut self) -> bool {
            self.action.is_some()
        }

        fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
            self.action.take()
        }

        fn complete_host_action(&mut self, result: wabou_shell::HostActionResult) {
            self.completed.lock().unwrap().push(result);
        }
    }

    impl crate::widget::Widget for WheelCaptureWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn handle_event(&mut self, event: &UiEvent) -> crate::widget::WidgetEventResult {
            if let UiEvent::Wheel(wheel) = event {
                self.0.lock().unwrap().push(wheel.position);
                crate::widget::WidgetEventResult::HANDLED
            } else {
                crate::widget::WidgetEventResult::IGNORED
            }
        }
    }

    impl crate::widget::Widget for KeyCaptureWidget {
        fn paint(&mut self, _width: f32, _height: f32, _tcx: &mut TextContext) -> vello::Scene {
            vello::Scene::new()
        }

        fn handle_event(&mut self, event: &UiEvent) -> crate::widget::WidgetEventResult {
            if matches!(event, UiEvent::Key(_)) {
                *self.0.lock().unwrap() += 1;
                crate::widget::WidgetEventResult::handled_consuming_key_text()
            } else {
                crate::widget::WidgetEventResult::IGNORED
            }
        }

        fn accepts_focus(&self) -> bool {
            true
        }
    }

    #[test]
    fn prevented_keydown_never_reaches_the_focused_widget() {
        let js = JsRuntime::new().expect("runtime");
        js.with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.__wabou_dispatch_host_frame = (bytes) => {
                    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
                    const eventId = view.getUint32(48, true);
                    return { needsTick: true, preventedEventIds: new Uint32Array([eventId]) };
                };
                "#,
            )
        })
        .unwrap();
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        applier.apply_op(&Op::AddEventListener {
            id: 2,
            event_type: event::KEYDOWN,
        });
        let node = applier.solid_to_node[&2];
        let received = Arc::new(std::sync::Mutex::new(0));
        applier
            .widgets
            .insert(node, Box::new(KeyCaptureWidget(received.clone())));
        applier.focused_target = Some(2);

        let response = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "t".into(),
            key_without_modifiers: "t".into(),
            code: "KeyT".into(),
            text: Some("t".into()),
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL
            },
            repeat: false,
        }));

        assert!(response.handled);
        assert!(response.consume_key_text);
        assert_eq!(*received.lock().unwrap(), 0);

        applier
            .js
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"
                    globalThis.__wabou_dispatch_host_frame = () => ({
                        needsTick: true,
                        preventedEventIds: new Uint32Array(),
                    });
                    "#,
                )
            })
            .unwrap();
        applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "x".into(),
            key_without_modifiers: "x".into(),
            code: "KeyX".into(),
            text: Some("x".into()),
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Modifiers::default(),
            repeat: false,
        }));
        assert_eq!(
            *received.lock().unwrap(),
            1,
            "unprevented keydown must continue to the focused widget"
        );
    }

    #[test]
    fn imperative_focus_uses_the_same_host_focus_state_as_pointer_input() {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });

        applier.apply_op(&Op::FocusNode { id: 2 });
        assert_eq!(applier.focused_target, Some(2));

        applier.apply_op(&Op::FocusNode { id: 999 });
        assert_eq!(
            applier.focused_target,
            Some(2),
            "a stale JS handle must not clear valid native focus"
        );
    }

    #[test]
    fn widget_measurements_refresh_intrinsic_layout_before_paint() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        let node = applier.solid_to_node[&2];
        applier
            .widgets
            .insert(node, Box::new(MeasuringWidget([123.0, 45.0])));
        assert_eq!(
            applier.computed_node_snapshot(2).unwrap().intrinsic_size,
            None
        );

        let mut tcx = TextContext::new();
        applier.measure_widgets(&mut tcx);

        assert_eq!(
            applier.computed_node_snapshot(2).unwrap().intrinsic_size,
            Some([123.0, 45.0])
        );
        assert!(applier.invalidation.contains(InvalidationFlags::LAYOUT));
    }

    #[test]
    fn widget_styles_are_delivered_once_before_measurement() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
        applier.widgets.insert(
            applier.root,
            Box::new(StyleAwareMeasuringWidget(calls.clone())),
        );

        let mut tcx = TextContext::new();
        applier.sync_widget_styles();
        applier.measure_widgets(&mut tcx);
        applier.sync_widget_styles();

        assert_eq!(*calls.lock().unwrap(), ["style", "measure"]);

        let mut paint = applier.tree.get_node_context(applier.root).unwrap().clone();
        paint.font_size += 1.0;
        applier
            .tree
            .set_node_context(applier.root, Some(paint))
            .unwrap();
        applier.sync_widget_styles();

        assert_eq!(*calls.lock().unwrap(), ["style", "measure", "style"]);
    }

    #[test]
    fn wheel_routing_preserves_pointer_position_for_widgets() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let received = Arc::new(std::sync::Mutex::new(Vec::new()));
        applier
            .widgets
            .insert(applier.root, Box::new(WheelCaptureWidget(received.clone())));
        applier.hovered_target = Some(1);
        applier.pointer_position = (42.0, 73.0);

        let response = applier.handle_event(UiEvent::Wheel(wabou_shell::WheelEvent {
            position: Point { x: 42.0, y: 73.0 },
            delta_x: 0.0,
            delta_y: -40.0,
            modifiers: Modifiers::default(),
        }));

        assert!(response.handled);
        assert_eq!(
            received.lock().unwrap().as_slice(),
            [Point { x: 42.0, y: 73.0 }]
        );
    }

    #[test]
    fn event_mask_is_compact_and_preserves_protocol_codes() {
        assert_eq!(std::mem::size_of::<EventMask>(), 4);
        let mut mask = EventMask::default();
        mask.insert(event::CLICK);
        mask.insert(event::SCROLL);
        mask.insert(0);
        assert!(mask.contains(event::CLICK));
        assert!(mask.contains(event::SCROLL));
        assert_eq!(
            mask.codes().collect::<Vec<_>>(),
            vec![event::CLICK, event::SCROLL]
        );
        mask.remove(event::CLICK);
        assert!(!mask.contains(event::CLICK));
    }

    #[test]
    fn widget_host_actions_reach_the_frame_source() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        applier.widgets.insert(
            applier.root,
            Box::new(HostActionWidget(Some(
                wabou_shell::HostAction::SetWindowTitle(Some("terminal".into())),
            ))),
        );

        assert!(FrameSource::poll_async(&mut applier));
        assert_eq!(
            FrameSource::take_host_action(&mut applier),
            Some(wabou_shell::HostAction::SetWindowTitle(Some(
                "terminal".into()
            )))
        );
        assert_eq!(FrameSource::take_host_action(&mut applier), None);
    }

    #[test]
    fn asynchronous_widget_events_are_routed_to_the_owning_solid_node() {
        let mut applier = interactive_applier();
        applier.apply_op(&Op::AddEventListener {
            id: 2,
            event_type: event::TERMINALEXIT,
        });
        let node = applier.solid_to_node[&2];
        applier.widgets.insert(
            node,
            Box::new(NodeEventWidget(Some(crate::widget::WidgetNodeEvent::json(
                event::TERMINALEXIT,
                r#"{"reason":"exit"}"#,
            )))),
        );

        assert!(FrameSource::poll_async(&mut applier));
        let dispatched = applier
            .js
            .with(|ctx| {
                ctx.eval::<String, _>(
                    "JSON.stringify(globalThis.dispatched[globalThis.dispatched.length - 1])",
                )
            })
            .expect("read widget node event");
        assert_eq!(
            dispatched,
            format!(r#"[2,{},"{{\"reason\":\"exit\"}}"]"#, event::TERMINALEXIT)
        );
    }

    #[test]
    fn widget_event_host_actions_are_available_without_an_async_poll() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        applier.widgets.insert(
            applier.root,
            Box::new(EventHostActionWidget(Some(
                wabou_shell::HostAction::OpenUrl("https://example.com".into()),
            ))),
        );

        let response = applier
            .handle_widget_event(1, &UiEvent::Focus(true))
            .expect("widget handled event");
        assert!(response.handled);
        assert_eq!(
            FrameSource::take_host_action(&mut applier),
            Some(wabou_shell::HostAction::OpenUrl(
                "https://example.com".into()
            ))
        );
    }

    #[test]
    fn dropping_a_widget_drains_unmount_host_actions_before_routing_is_removed() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        let node = applier.solid_to_node[&2];
        applier
            .widgets
            .insert(node, Box::new(UnmountActionWidget(None)));

        applier.apply_op(&Op::DropNode { id: 2 });

        assert_eq!(
            FrameSource::take_host_action(&mut applier),
            Some(wabou_shell::HostAction::SetWindowTitle(None))
        );
        assert!(!applier.widgets.contains_key(&node));
    }

    #[test]
    fn dropping_a_focused_captured_widget_releases_input_before_unmount() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        let node = applier.solid_to_node[&2];
        let lifecycle = Arc::new(std::sync::Mutex::new(Vec::new()));
        applier
            .widgets
            .insert(node, Box::new(LifecycleWidget(lifecycle.clone())));
        applier.focused_target = Some(2);
        applier.pointer_down_target = Some(2);
        applier.pointer_down_position = Some((10.0, 20.0));
        applier.pointer_dragged = true;

        applier.apply_op(&Op::DropNode { id: 2 });

        assert_eq!(
            *lifecycle.lock().unwrap(),
            ["pointer-cancel", "focus-out", "unmount"]
        );
        assert_eq!(applier.focused_target, None);
        assert_eq!(applier.pointer_down_target, None);
        assert_eq!(applier.pointer_down_position, None);
        assert!(!applier.pointer_dragged);
    }

    #[test]
    fn window_focus_loss_cancels_the_captured_pointer_before_blur() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        let node = applier.solid_to_node[&2];
        let lifecycle = Arc::new(std::sync::Mutex::new(Vec::new()));
        applier
            .widgets
            .insert(node, Box::new(LifecycleWidget(lifecycle.clone())));
        applier.focused_target = Some(2);
        applier.pointer_down_target = Some(2);
        applier.pointer_down_position = Some((10.0, 20.0));
        applier.pointer_position = (15.0, 25.0);
        applier.pointer_buttons = 1;
        applier.pointer_dragged = true;
        applier.last_text_click = Some((Instant::now(), 2, 15.0, 25.0, 1));

        let blurred = applier.handle_event(UiEvent::Focus(false));

        assert_eq!(*lifecycle.lock().unwrap(), ["pointer-cancel", "focus-out"]);
        assert_eq!(blurred.text_input, Some(false));
        assert_eq!(applier.focused_target, Some(2));
        assert!(!applier.window_focused);
        assert_eq!(applier.pointer_down_target, None);
        assert_eq!(applier.pointer_down_position, None);
        assert_eq!(applier.pointer_buttons, 0);
        assert!(!applier.pointer_dragged);
        assert!(applier.last_text_click.is_none());

        let focused = applier.handle_event(UiEvent::Focus(true));
        assert_eq!(focused.text_input, Some(true));
        assert_eq!(
            *lifecycle.lock().unwrap(),
            ["pointer-cancel", "focus-out", "focus-in"]
        );
        assert_eq!(applier.focused_target, Some(2));
        assert!(applier.window_focused);

        applier.last_text_click = Some((Instant::now(), 2, 15.0, 25.0, 1));
        applier.handle_event(UiEvent::TextInput("x".into()));
        assert!(
            applier.last_text_click.is_none(),
            "text input must break a native text multi-click sequence"
        );
    }

    #[test]
    fn clipboard_read_completions_route_to_the_requesting_widget() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        let second_node = applier.solid_to_node[&2];
        let first_completed = Arc::new(std::sync::Mutex::new(Vec::new()));
        let second_completed = Arc::new(std::sync::Mutex::new(Vec::new()));
        applier.widgets.insert(
            applier.root,
            Box::new(ClipboardReadWidget {
                action: Some(wabou_shell::HostAction::ReadClipboard { request_id: 7 }),
                completed: first_completed.clone(),
            }),
        );
        applier.widgets.insert(
            second_node,
            Box::new(ClipboardReadWidget {
                action: Some(wabou_shell::HostAction::ReadClipboard { request_id: 7 }),
                completed: second_completed.clone(),
            }),
        );

        assert!(FrameSource::poll_async(&mut applier));
        let mut requests = Vec::new();
        while let Some(wabou_shell::HostAction::ReadClipboard { request_id }) =
            FrameSource::take_host_action(&mut applier)
        {
            requests.push(request_id);
        }
        assert_eq!(requests.len(), 2);
        assert_ne!(requests[0], requests[1]);
        for request_id in requests.into_iter().rev() {
            let (node, _) = applier.host_action_routes[&request_id];
            let text = if node == applier.root {
                "first"
            } else {
                "second"
            };
            FrameSource::complete_host_action(
                &mut applier,
                wabou_shell::HostActionResult::Clipboard {
                    request_id,
                    text: Some(text.into()),
                },
            );
        }
        assert_eq!(
            *first_completed.lock().unwrap(),
            vec![wabou_shell::HostActionResult::Clipboard {
                request_id: 7,
                text: Some("first".into()),
            }]
        );
        assert_eq!(
            *second_completed.lock().unwrap(),
            vec![wabou_shell::HostActionResult::Clipboard {
                request_id: 7,
                text: Some("second".into()),
            }]
        );
    }
    use wabou_shell::{Point, PointerEvent};

    fn pointer(phase: PointerPhase, x: f64, y: f64, buttons: u32) -> UiEvent {
        UiEvent::Pointer(PointerEvent {
            phase,
            position: Point { x, y },
            button: Some(PointerButton::Primary),
            buttons,
            modifiers: Modifiers::default(),
        })
    }

    fn install_host_frame_test_hook(js: &JsRuntime) {
        js.with(|ctx| ctx.eval::<(), _>(r#"
          globalThis.dispatched = [];
          globalThis.resizeChanges = [];
          globalThis.__host_got = [];
          globalThis.__wabou_dispatch_host_frame = (u8) => {
            const view = new DataView(u8.buffer, u8.byteOffset, u8.byteLength);
            const dec = new TextDecoder();
            let o = 32;
            const count = view.getUint32(24, true);
            for (let n = 0; n < count; n++) {
              const start = o, kind = view.getUint8(o), len = view.getUint32(o + 4, true);
              o += 8;
              if (kind === 1) {
                const id = view.getUint32(o, true), code = view.getUint8(o + 4);
                const payloadKind = view.getUint8(o + 5); o += 12;
                let payload = "";
                if (payloadKind === 2) {
                  const size = view.getUint32(o, true); o += 4;
                  payload = dec.decode(u8.subarray(o, o + size));
                }
                globalThis.dispatched.push([id, code, payload]);
              } else if (kind === 2) {
                globalThis.resizeChanges.push([
                  view.getUint32(o, true), view.getFloat32(o + 4, true), view.getFloat32(o + 8, true)
                ]);
              } else if (kind === 3) {
                const tl = view.getUint16(o, true); o += 2;
                const topic = dec.decode(u8.subarray(o, o + tl)); o += tl;
                const valueKind = u8[o++]; let payload;
                if (valueKind === 2) { payload = view.getInt32(o, true); }
                else if (valueKind === 4) {
                  const size = view.getUint16(o, true); o += 2;
                  payload = dec.decode(u8.subarray(o, o + size));
                }
                globalThis.__host_got.push({topic, payload});
              }
              o = start + len;
            }
            return { needsTick: true, preventedEventIds: new Uint32Array() };
          };
        "#)).expect("host-frame test hook");
    }

    #[test]
    fn window_metrics_reach_js_without_waiting_for_a_resize_frame() {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let response = applier.handle_event(UiEvent::WindowMetrics(wabou_shell::WindowMetrics {
            window_id: 1,
            logical_width: 800,
            logical_height: 600,
            physical_width: 1600,
            physical_height: 1200,
            scale_factor: 2.0,
            maximized: true,
            focused: true,
        }));
        assert!(response.request_redraw);
        let payload = applier
            .js
            .with(|ctx| {
                ctx.eval::<String, _>(
                    "globalThis.__host_got.find((x) => x.topic === 'wabou:window-metrics').payload",
                )
            })
            .unwrap();
        assert!(payload.contains("logicalWidth"));
        assert!(payload.contains("\"windowId\":1"));
        assert!(payload.contains("\"scaleFactor\":2.0"));
    }

    #[test]
    fn window_bridge_is_available_during_initial_boot_and_targets_ids() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime_with_factories_and_window(
            js,
            builtin_factories(),
            Color::BLACK,
            17,
        );
        applier
            .boot(
                r#"
                globalThis.created = __wabou_window_create(JSON.stringify({
                  title: "Child", width: 640, height: 480, resizable: false
                }));
                __wabou_window_set_title(globalThis.created, "Renamed");
                __wabou_window_set_maximized(globalThis.created, true);
                __wabou_window_close(globalThis.created);
                globalThis.currentWindowId = __wabou_window_id;
                "#,
            )
            .expect("boot with window bridge");

        assert_eq!(
            applier
                .js
                .with(|ctx| ctx.eval::<u64, _>("currentWindowId"))
                .expect("current window id"),
            17
        );
        let created = match applier.take_host_action() {
            Some(wabou_shell::HostAction::CreateWindow { window_id, options }) => {
                assert_eq!(options.title, "Child");
                assert_eq!(options.initial_inner_size, (640, 480));
                assert!(!options.resizable);
                window_id
            }
            action => panic!("unexpected action: {action:?}"),
        };
        for command in [
            wabou_shell::WindowCommand::SetTitle("Renamed".into()),
            wabou_shell::WindowCommand::SetMaximized(true),
            wabou_shell::WindowCommand::Close,
        ] {
            assert_eq!(
                applier.take_host_action(),
                Some(wabou_shell::HostAction::ControlWindow {
                    window_id: created,
                    command,
                })
            );
        }
    }

    #[test]
    fn clipboard_bridge_routes_native_completions_back_to_javascript() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        applier
            .boot(
                r#"
                globalThis.clipboardCompletions = [];
                globalThis.__wabou_clipboard_complete = (id, text, success) => {
                  clipboardCompletions.push([id, text, success]);
                };
                globalThis.writeRequest = __wabou_clipboard_write("hello");
                globalThis.readRequest = __wabou_clipboard_read();
                "#,
            )
            .expect("boot with clipboard bridge");

        let requests = applier
            .js
            .with(|ctx| ctx.eval::<Vec<u64>, _>("[writeRequest, readRequest]"))
            .expect("clipboard request ids");
        let [write_request, read_request] = requests.as_slice() else {
            panic!("expected two clipboard request ids");
        };
        let (write_request, read_request) = (*write_request, *read_request);
        assert!(write_request < (1_u64 << 32));
        assert_eq!(
            applier.take_host_action(),
            Some(wabou_shell::HostAction::WriteClipboard {
                request_id: write_request,
                text: "hello".into(),
            })
        );
        assert_eq!(
            applier.take_host_action(),
            Some(wabou_shell::HostAction::ReadClipboard {
                request_id: read_request,
            })
        );

        applier.complete_host_action(wabou_shell::HostActionResult::ClipboardWrite {
            request_id: write_request,
            success: true,
        });
        applier.complete_host_action(wabou_shell::HostActionResult::Clipboard {
            request_id: read_request,
            text: Some("world".into()),
        });
        let completions = applier
            .js
            .with(|ctx| ctx.eval::<String, _>("JSON.stringify(clipboardCompletions)"))
            .expect("clipboard completions");
        assert_eq!(
            completions,
            format!("[[{write_request},null,true],[{read_request},\"world\",true]]")
        );
    }

    #[test]
    fn window_runtimes_keep_globals_and_action_queues_isolated() {
        let make = |window_id| {
            Applier::from_runtime_with_factories_and_window(
                JsRuntime::new().expect("runtime"),
                builtin_factories(),
                Color::BLACK,
                window_id,
            )
        };
        let mut first = make(1);
        let mut second = make(2);
        first
            .boot("globalThis.localState = 'first'; __wabou_window_close(1)")
            .expect("boot first");
        second
            .boot("globalThis.localState = 'second'; __wabou_window_close(2)")
            .expect("boot second");

        assert_eq!(
            first
                .js
                .with(|ctx| ctx.eval::<String, _>("localState"))
                .unwrap(),
            "first"
        );
        assert_eq!(
            second
                .js
                .with(|ctx| ctx.eval::<String, _>("localState"))
                .unwrap(),
            "second"
        );
        assert_eq!(
            first.take_host_action(),
            Some(wabou_shell::HostAction::ControlWindow {
                window_id: 1,
                command: wabou_shell::WindowCommand::Close,
            })
        );
        assert_eq!(
            second.take_host_action(),
            Some(wabou_shell::HostAction::ControlWindow {
                window_id: 2,
                command: wabou_shell::WindowCommand::Close,
            })
        );
        assert_eq!(first.take_host_action(), None);
        assert_eq!(second.take_host_action(), None);
    }

    #[test]
    fn hmr_batch_coalesces_full_reload_over_partial_updates() {
        let batch = plan_hmr_batch([
            ReloadMsg::HmrUpdate {
                path: "/a.tsx".into(),
                accepted_path: "/a.tsx".into(),
                timestamp: 1,
                source: "export {}".into(),
            },
            ReloadMsg::CssUpdate {
                path: "/x.css".into(),
                source: "body{}".into(),
            },
            ReloadMsg::FullReload,
            ReloadMsg::HmrUpdate {
                path: "/b.tsx".into(),
                accepted_path: "/b.tsx".into(),
                timestamp: 2,
                source: "export {}".into(),
            },
        ]);
        assert!(batch.full_reload);
        assert_eq!(batch.js_updates.len(), 2);
        assert_eq!(batch.css_paths, vec!["/x.css".to_string()]);
    }

    #[test]
    fn hmr_batch_preserves_js_update_order() {
        let batch = plan_hmr_batch([
            ReloadMsg::HmrUpdate {
                path: "/a.tsx".into(),
                accepted_path: "/a.tsx".into(),
                timestamp: 1,
                source: "a".into(),
            },
            ReloadMsg::HmrUpdate {
                path: "/b.tsx".into(),
                accepted_path: "/b.tsx".into(),
                timestamp: 2,
                source: "b".into(),
            },
        ]);
        assert!(!batch.full_reload);
        assert_eq!(
            batch
                .js_updates
                .iter()
                .map(|u| u.path.as_str())
                .collect::<Vec<_>>(),
            vec!["/a.tsx", "/b.tsx"]
        );
    }

    #[test]
    fn full_reload_clears_non_root_scene_nodes() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        assert!(applier.solid_to_node.contains_key(&2));
        applier.perform_full_reload("test");
        assert!(!applier.solid_to_node.contains_key(&2));
        assert!(applier.solid_to_node.contains_key(&1));
        assert_eq!(applier.tree.child_count(applier.root), 0);
    }

    #[test]
    fn host_messages_are_delivered_to_js_before_tick() {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);
        js.with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.__wabou_tick = () => false;
                globalThis.__wabou_has_raf = () => false;
                "#,
            )
        })
        .unwrap();
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let handle = applier.host_msg_handle();
        handle.emit_str("logs", "hello").unwrap();
        handle.emit_i32("count", 7).unwrap();

        let mut text = TextContext::new();
        applier.build_frame(&mut text, 100, 100);

        let got: String = applier
            .js
            .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.__host_got)"))
            .unwrap();
        let v: serde_json::Value = serde_json::from_str(&got).unwrap();
        assert_eq!(v.as_array().unwrap().len(), 2);
        assert_eq!(v[0]["topic"], "logs");
        assert_eq!(v[0]["payload"], "hello");
        assert_eq!(v[1]["topic"], "count");
        assert_eq!(v[1]["payload"], 7);
    }

    #[test]
    fn hmr_queue_full_reload_is_drained_as_full_reload_result() {
        let js = JsRuntime::new().expect("runtime");
        js.with(|ctx| {
            ctx.eval::<(), _>(
                "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
            )
        })
        .unwrap();
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let handle = applier.reload_handle();
        handle.send(ReloadMsg::FullReload).unwrap();
        let mut text = TextContext::new();
        applier.build_frame(&mut text, 100, 100);
        assert!(matches!(
            applier.last_hmr_result(),
            HmrDrainResult::FullReload { .. }
        ));
    }

    #[test]
    fn inline_svg_cache_follows_node_lifetime() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (svg, path, view_box, width, height, fill, stroke, d) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("svg"),
                atoms.intern("path"),
                atoms.intern("viewBox"),
                atoms.intern("width"),
                atoms.intern("height"),
                atoms.intern("fill"),
                atoms.intern("stroke"),
                atoms.intern("d"),
            )
        };
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: svg,
            attrs: vec![
                (view_box, "0 0 24 24"),
                (width, "24"),
                (height, "24"),
                (fill, "none"),
                (stroke, "currentColor"),
            ],
        });
        applier.apply_op(&Op::CreateElement {
            id: 3,
            tag: path,
            attrs: vec![(d, "M3 12h18")],
        });
        applier.apply_op(&Op::AppendChild {
            parent: 2,
            child: 3,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        applier.rebuild_layout_boxes();
        applier.inherit();

        let svg_node = applier.solid_to_node[&2];
        assert_eq!(applier.tree.child_count(svg_node), 0);
        assert_eq!(
            applier
                .tree
                .get_node_context(svg_node)
                .unwrap()
                .intrinsic_size,
            Some([24.0, 24.0])
        );
        assert!(
            applier
                .tree
                .get_node_context(svg_node)
                .unwrap()
                .svg
                .is_some()
        );
        assert_eq!(applier.svg_cache.len(), 1);

        applier.apply_op(&Op::DropNode { id: 2 });
        assert!(applier.svg_cache.is_empty());
    }

    #[test]
    fn text_input_updates_value_paints_and_dispatches_input() {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);
        js.with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                // No app bundle is booted in this test; stub the rAF entry
                // points so build_frame's `js.tick()` runs without error.
                globalThis.__wabou_tick = () => false;
                globalThis.__wabou_has_raf = () => false;
                "#,
            )
        })
        .unwrap();
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (input, value, width) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("input"),
                atoms.intern("value"),
                atoms.intern("width"),
            )
        };
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: input,
            attrs: vec![(value, "")],
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: width,
            value: "200px",
        });
        applier.apply_op(&Op::AddEventListener {
            id: 2,
            event_type: event::INPUT,
        });
        let mut tcx = TextContext::new();
        // build_frame computes layout + paints widgets + drains value sync.
        applier.build_frame(&mut tcx, 800, 600);
        let focus = applier.handle_event(pointer(PointerPhase::Down, 10.0, 10.0, 1));
        let node = applier.solid_to_node[&2];
        assert!(applier.tree.layout(node).unwrap().size.height > 0.0);
        assert_eq!(focus.text_input, Some(true));
        assert_eq!(applier.focused_target, Some(2));

        // Widgets receive the complete captured pointer stream, including
        // moves/releases outside their hit-test bounds. Text selection relies
        // on this just like native controls do.
        assert!(
            applier
                .handle_event(pointer(PointerPhase::Move, 400.0, 10.0, 1))
                .handled
        );
        assert!(
            applier
                .handle_event(pointer(PointerPhase::Up, 400.0, 10.0, 0))
                .handled
        );
        assert!(applier.pointer_down_target.is_none());

        // Text edits resolve at paint (pending-edit pattern: handle_event has
        // no FontContext), so the value sync + INPUT dispatch are deferred to
        // build_frame. Drive one frame to apply + dispatch.
        assert!(
            applier
                .handle_event(UiEvent::TextInput("ab".into()))
                .handled
        );
        applier.build_frame(&mut tcx, 800, 600);
        assert_eq!(applier.widgets[&node].current_value(), Some("ab"));
        assert_eq!(applier.declared[&node].attrs[&value].as_ref(), "ab");
        let payload = applier
            .js
            .with(|ctx| ctx.eval::<String, _>("globalThis.dispatched[0][2]"))
            .unwrap();
        assert_eq!(payload, r#"{"value":"ab"}"#);

        applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "Backspace".into(),
            key_without_modifiers: "Backspace".into(),
            code: "Backspace".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Modifiers::default(),
            repeat: false,
        }));
        applier.build_frame(&mut tcx, 800, 600);
        assert_eq!(applier.widgets[&node].current_value(), Some("a"));
        assert_eq!(applier.declared[&node].attrs[&value].as_ref(), "a");

        for _ in 0..2 {
            applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
                phase: KeyPhase::Down,
                key: "Backspace".into(),
                key_without_modifiers: "Backspace".into(),
                code: "Backspace".into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers: Modifiers::default(),
                repeat: false,
            }));
        }
        applier.build_frame(&mut tcx, 800, 600);
        assert_eq!(applier.widgets[&node].current_value(), Some(""));
        assert_eq!(applier.declared[&node].attrs[&value].as_ref(), "");
        // A control char (backspace as text) is filtered out → IGNORED, no
        // value sync, handled stays false.
        assert!(
            !applier
                .handle_event(UiEvent::TextInput("\u{8}".into()))
                .handled
        );
        applier.build_frame(&mut tcx, 800, 600);
        assert_eq!(applier.widgets[&node].current_value(), Some(""));
    }

    #[test]
    fn text_only_element_uses_one_parley_leaf_instead_of_text_boxes() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        applier.apply_op(&Op::CreateText {
            id: 3,
            text: "Hello ",
        });
        applier.apply_op(&Op::CreateText {
            id: 4,
            text: "world",
        });
        applier.apply_op(&Op::AppendChild {
            parent: 2,
            child: 3,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 2,
            child: 4,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        applier.rebuild_layout_boxes();

        let parent = applier.solid_to_node[&2];
        assert_eq!(applier.tree.child_count(parent), 0);
        assert_eq!(
            applier
                .tree
                .get_node_context(parent)
                .unwrap()
                .text
                .as_deref(),
            Some("Hello world")
        );

        applier.apply_op(&Op::SetText {
            id: 4,
            text: "Wabou",
        });
        applier.rebuild_layout_boxes();
        assert_eq!(
            applier
                .tree
                .get_node_context(parent)
                .unwrap()
                .text
                .as_deref(),
            Some("Hello Wabou")
        );
    }

    #[test]
    fn ordinary_text_drag_selects_highlights_and_copies() {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        applier.apply_op(&Op::CreateText {
            id: 3,
            text: "selectable text",
        });
        applier.apply_op(&Op::AppendChild {
            parent: 2,
            child: 3,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        applier.apply_op(&Op::AddEventListener {
            id: 2,
            event_type: event::TEXTSELECTIONCHANGE,
        });
        applier.rebuild_layout_boxes();
        applier.inherit();

        let mut tcx = TextContext::new();
        let mut placed = layout::compute_and_walk_with_scroll(
            &mut applier.tree,
            applier.root,
            400.0,
            100.0,
            &mut tcx,
            &HashMap::new(),
        );
        applier.rebuild_hit_geometry(&placed);
        applier.prepare_text_selection(&mut placed, &mut tcx);
        let origin = applier.selectable_text[&2].origin;

        assert!(
            applier
                .handle_event(pointer(
                    PointerPhase::Down,
                    f64::from(origin[0] + 1.0),
                    f64::from(origin[1] + 5.0),
                    1,
                ))
                .handled
        );
        assert!(
            applier
                .handle_event(pointer(
                    PointerPhase::Move,
                    f64::from(origin[0] + 55.0),
                    f64::from(origin[1] + 5.0),
                    1,
                ))
                .handled
        );
        let during_drag = applier
            .js
            .with(|ctx| {
                ctx.eval::<usize, _>(format!(
                    "globalThis.dispatched.filter((event) => event[1] === {}).length",
                    event::TEXTSELECTIONCHANGE
                ))
            })
            .expect("selection event count while dragging");
        assert_eq!(during_drag, 0, "dragging stays local to the renderer");
        applier.handle_event(pointer(
            PointerPhase::Up,
            f64::from(origin[0] + 55.0),
            f64::from(origin[1] + 5.0),
            0,
        ));
        applier.prepare_text_selection(&mut placed, &mut tcx);
        assert!(
            applier.last_text_click.is_none(),
            "a drag must not seed the subsequent multi-click streak"
        );

        let selected = applier.selected_text().expect("non-empty selection");
        let selection_event = applier
            .js
            .with(|ctx| {
                ctx.eval::<String, _>(format!(
                    "JSON.stringify(globalThis.dispatched.find((event) => event[1] === {}))",
                    event::TEXTSELECTIONCHANGE
                ))
            })
            .expect("committed selection event");
        let selection_event: serde_json::Value = serde_json::from_str(&selection_event).unwrap();
        assert_eq!(selection_event[0], 2);
        assert_eq!(selection_event[1], event::TEXTSELECTIONCHANGE);
        let payload: serde_json::Value =
            serde_json::from_str(selection_event[2].as_str().unwrap()).unwrap();
        assert_eq!(payload["text"], selected);
        assert_eq!(payload["kind"], "simple");
        assert!("selectable text".starts_with(&selected));
        assert!(
            !placed
                .iter()
                .find(|node| node.node_id == applier.solid_to_node[&2])
                .unwrap()
                .paint
                .selection_rects
                .is_empty()
        );

        let copied = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "c".into(),
            key_without_modifiers: "c".into(),
            code: "KeyC".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL
            },
            repeat: false,
        }));
        assert_eq!(
            copied.clipboard,
            Some(wabou_shell::ClipboardRequest::Write(selected))
        );

        applier.last_text_click = None;
        let click_x = f64::from(origin[0] + 10.0);
        let click_y = f64::from(origin[1] + 5.0);
        applier.handle_event(pointer(PointerPhase::Down, click_x, click_y, 1));
        let mut cancelled = match pointer(PointerPhase::Cancel, click_x, click_y, 0) {
            UiEvent::Pointer(pointer) => pointer,
            _ => unreachable!(),
        };
        // Platform pointer-cancel events commonly omit the triggering button.
        cancelled.button = None;
        applier.handle_event(UiEvent::Pointer(cancelled));
        applier.handle_event(pointer(PointerPhase::Down, click_x, click_y, 1));
        assert_eq!(
            applier.active_text_selection.as_ref().unwrap().granularity,
            TextSelectionGranularity::Cluster,
            "a cancelled click must not turn the next click into word selection"
        );
        applier.handle_event(pointer(PointerPhase::Cancel, click_x, click_y, 0));
        applier.last_text_click = None;

        let word_x = f64::from(origin[0] + 10.0);
        for _ in 0..2 {
            applier.begin_text_selection(2, word_x, f64::from(origin[1] + 5.0), Modifiers::empty());
        }
        assert_eq!(applier.selected_text().as_deref(), Some("selectable"));
        assert!(applier.sync_text_selection_change());
        let word_event = applier
            .js
            .with(|ctx| {
                ctx.eval::<String, _>(format!(
                    "JSON.stringify(globalThis.dispatched.filter((event) => event[1] === {}).at(-1))",
                    event::TEXTSELECTIONCHANGE
                ))
            })
            .expect("word selection event");
        let word_event: serde_json::Value = serde_json::from_str(&word_event).unwrap();
        let payload: serde_json::Value =
            serde_json::from_str(word_event[2].as_str().unwrap()).unwrap();
        assert_eq!(
            payload,
            serde_json::json!({ "text": "selectable", "kind": "word" })
        );
        let line_end = f64::from(origin[0] + applier.selectable_text[&2].layout.width() + 10.0);
        applier.begin_text_selection(2, line_end, f64::from(origin[1] + 5.0), Modifiers::SHIFT);
        assert_eq!(applier.selected_text().as_deref(), Some("selectable "));
        assert!(
            applier.last_text_click.is_none(),
            "Shift extension must not seed a later double click"
        );

        let user_select = applier.atoms.borrow_mut().intern("user-select");
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: user_select,
            value: "all",
        });
        applier.inherit();
        let mut placed = layout::compute_and_walk_with_scroll(
            &mut applier.tree,
            applier.root,
            400.0,
            100.0,
            &mut tcx,
            &HashMap::new(),
        );
        applier.rebuild_hit_geometry(&placed);
        applier.prepare_text_selection(&mut placed, &mut tcx);
        applier.handle_event(pointer(
            PointerPhase::Down,
            f64::from(origin[0] + 2.0),
            f64::from(origin[1] + 5.0),
            1,
        ));
        assert_eq!(applier.selected_text().as_deref(), Some("selectable text"));
        applier.next_text_selection_scroll = Some(Instant::now());

        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: user_select,
            value: "none",
        });
        applier.inherit();
        let mut placed = layout::compute_and_walk_with_scroll(
            &mut applier.tree,
            applier.root,
            400.0,
            100.0,
            &mut tcx,
            &HashMap::new(),
        );
        applier.prepare_text_selection(&mut placed, &mut tcx);
        assert!(!applier.computed_node_snapshot(2).unwrap().text_selectable);
        assert!(!applier.selectable_text.contains_key(&2));
        assert!(applier.active_text_selection.is_none());
        assert!(applier.next_text_selection_scroll.is_none());
        assert!(applier.sync_text_selection_change());
        let cleared = applier
            .js
            .with(|ctx| {
                ctx.eval::<String, _>(format!(
                    "JSON.stringify(globalThis.dispatched.filter((event) => event[1] === {}).at(-1))",
                    event::TEXTSELECTIONCHANGE
                ))
            })
            .expect("selection clear event");
        let cleared: serde_json::Value = serde_json::from_str(&cleared).unwrap();
        let payload: serde_json::Value =
            serde_json::from_str(cleared[2].as_str().unwrap()).unwrap();
        assert_eq!(payload, serde_json::json!({ "text": null, "kind": null }));
    }

    #[test]
    fn text_selection_crosses_hosts_in_both_directions() {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (div, height, flex_direction, user_select) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("div"),
                atoms.intern("height"),
                atoms.intern("flex-direction"),
                atoms.intern("user-select"),
            )
        };
        applier.apply_op(&Op::SetStyle {
            id: 1,
            prop: flex_direction,
            value: "column",
        });
        for (host, text_node, text, selectable) in [
            (2, 3, "alpha", true),
            (4, 5, "secret", false),
            (6, 7, "beta", true),
        ] {
            applier.apply_op(&Op::CreateElement {
                id: host,
                tag: div,
                attrs: vec![],
            });
            applier.apply_op(&Op::SetStyle {
                id: host,
                prop: height,
                value: "30px",
            });
            if !selectable {
                applier.apply_op(&Op::SetStyle {
                    id: host,
                    prop: user_select,
                    value: "none",
                });
            }
            applier.apply_op(&Op::CreateText {
                id: text_node,
                text,
            });
            applier.apply_op(&Op::AppendChild {
                parent: host,
                child: text_node,
            });
            applier.apply_op(&Op::AppendChild {
                parent: 1,
                child: host,
            });
        }
        applier.apply_op(&Op::AddEventListener {
            id: 2,
            event_type: event::TEXTSELECTIONCHANGE,
        });
        applier.rebuild_layout_boxes();
        applier.inherit();

        let mut tcx = TextContext::new();
        let mut placed = layout::compute_and_walk_with_scroll(
            &mut applier.tree,
            applier.root,
            300.0,
            100.0,
            &mut tcx,
            &HashMap::new(),
        );
        applier.rebuild_hit_geometry(&placed);
        applier.prepare_text_selection(&mut placed, &mut tcx);
        assert_eq!(applier.selectable_text_order, vec![2, 6]);

        let select_all = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "a".into(),
            key_without_modifiers: "a".into(),
            code: "KeyA".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL
            },
            repeat: false,
        }));
        assert!(select_all.handled);
        assert!(select_all.request_redraw);
        assert_eq!(applier.selected_text().as_deref(), Some("alpha\nbeta"));
        let copy_all = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "c".into(),
            key_without_modifiers: "c".into(),
            code: "KeyC".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL
            },
            repeat: false,
        }));
        assert_eq!(
            copy_all.clipboard,
            Some(wabou_shell::ClipboardRequest::Write("alpha\nbeta".into()))
        );

        let point = |text: &SelectableText, index: usize| {
            let cursor = Cursor::from_byte_index(&text.layout, index, Affinity::Downstream);
            let geometry = cursor.geometry(&text.layout, 0.0);
            (
                f64::from(text.origin[0]) + geometry.x0 + 0.1,
                f64::from(text.origin[1]) + geometry.y0 + 2.0,
            )
        };
        let first_start = point(&applier.selectable_text[&2], 0);
        let second_end = point(&applier.selectable_text[&6], 4);
        assert_eq!(applier.hit_test(first_start.0, first_start.1), Some(2));
        assert_eq!(applier.hit_test(second_end.0, second_end.1), Some(6));
        applier.handle_event(pointer(PointerPhase::Down, first_start.0, first_start.1, 1));
        applier.handle_event(pointer(PointerPhase::Move, second_end.0, second_end.1, 1));
        applier.handle_event(pointer(PointerPhase::Up, second_end.0, second_end.1, 0));
        assert_eq!(applier.selected_text().as_deref(), Some("alpha\nbeta"));
        assert!(applier.pointer_down_target.is_none());
        applier.prepare_text_selection(&mut placed, &mut tcx);
        for target in [2, 6] {
            let node = applier.solid_to_node[&target];
            assert!(
                !placed
                    .iter()
                    .find(|placed| placed.node_id == node)
                    .unwrap()
                    .paint
                    .selection_rects
                    .is_empty()
            );
        }

        applier.last_text_click = None;
        applier.begin_text_selection(6, second_end.0, second_end.1, Modifiers::empty());
        applier.extend_text_selection(Some(2), first_start.0, first_start.1);
        assert_eq!(applier.selected_text().as_deref(), Some("alpha\nbeta"));

        applier.apply_op(&Op::SetText {
            id: 7,
            // Shorter multibyte replacement forces both endpoints through
            // Parley's UTF-8 cluster-boundary refresh path.
            text: "你",
        });
        applier.inherit();
        let mut placed = layout::compute_and_walk_with_scroll(
            &mut applier.tree,
            applier.root,
            300.0,
            100.0,
            &mut tcx,
            &HashMap::new(),
        );
        applier.prepare_text_selection(&mut placed, &mut tcx);
        assert_eq!(applier.selected_text().as_deref(), Some("alpha\n你"));

        applier.apply_op(&Op::RemoveChild {
            parent: 1,
            child: 6,
        });
        applier.apply_op(&Op::DropNode { id: 6 });
        let mut placed = layout::compute_and_walk_with_scroll(
            &mut applier.tree,
            applier.root,
            300.0,
            100.0,
            &mut tcx,
            &HashMap::new(),
        );
        applier.prepare_text_selection(&mut placed, &mut tcx);
        assert!(applier.active_text_selection.is_none());
        assert!(applier.selected_text().is_none());
        let cleared = applier
            .js
            .with(|ctx| {
                ctx.eval::<String, _>(format!(
                    "JSON.stringify(globalThis.dispatched.filter((event) => event[1] === {}).at(-1))",
                    event::TEXTSELECTIONCHANGE
                ))
            })
            .expect("selection clear on dropped endpoint");
        let cleared: serde_json::Value = serde_json::from_str(&cleared).unwrap();
        let payload: serde_json::Value =
            serde_json::from_str(cleared[2].as_str().unwrap()).unwrap();
        assert_eq!(payload, serde_json::json!({ "text": null, "kind": null }));
    }

    #[test]
    fn same_visual_line_with_different_font_metrics_copies_without_newline() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (div, flex_direction, align_items, height, font_size) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("div"),
                atoms.intern("flex-direction"),
                atoms.intern("align-items"),
                atoms.intern("height"),
                atoms.intern("font-size"),
            )
        };
        for (prop, value) in [
            (flex_direction, "row"),
            (align_items, "center"),
            (height, "60px"),
        ] {
            applier.apply_op(&Op::SetStyle { id: 1, prop, value });
        }
        for (host, text_node, text, size) in [(2, 3, "small", "12px"), (4, 5, "BIG", "30px")] {
            applier.apply_op(&Op::CreateElement {
                id: host,
                tag: div,
                attrs: vec![],
            });
            applier.apply_op(&Op::SetStyle {
                id: host,
                prop: font_size,
                value: size,
            });
            applier.apply_op(&Op::CreateText {
                id: text_node,
                text,
            });
            applier.apply_op(&Op::AppendChild {
                parent: host,
                child: text_node,
            });
            applier.apply_op(&Op::AppendChild {
                parent: 1,
                child: host,
            });
        }
        applier.rebuild_layout_boxes();
        applier.inherit();
        let mut tcx = TextContext::new();
        let mut placed = layout::compute_and_walk_with_scroll(
            &mut applier.tree,
            applier.root,
            300.0,
            60.0,
            &mut tcx,
            &HashMap::new(),
        );
        applier.prepare_text_selection(&mut placed, &mut tcx);

        let small = &applier.selectable_text[&2];
        let big = &applier.selectable_text[&4];
        assert!((small.origin[1] - big.origin[1]).abs() > 1.0);
        assert!(small.visual_y.start < big.visual_y.end && big.visual_y.start < small.visual_y.end);
        assert!(applier.select_all_text());
        assert_eq!(applier.selected_text().as_deref(), Some("smallBIG"));
    }

    #[test]
    fn mixed_inline_subtree_becomes_one_styled_parley_leaf() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (div, strong, font_weight, color) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("div"),
                atoms.intern("strong"),
                atoms.intern("font-weight"),
                atoms.intern("color"),
            )
        };
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: vec![],
        });
        applier.apply_op(&Op::CreateText {
            id: 3,
            text: "Hello ",
        });
        applier.apply_op(&Op::CreateElement {
            id: 4,
            tag: strong,
            attrs: vec![],
        });
        applier.apply_op(&Op::CreateText {
            id: 5,
            text: "world",
        });
        applier.apply_op(&Op::SetStyle {
            id: 4,
            prop: font_weight,
            value: "700",
        });
        applier.apply_op(&Op::SetStyle {
            id: 4,
            prop: color,
            value: "#ff0000",
        });
        applier.apply_op(&Op::AppendChild {
            parent: 4,
            child: 5,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 2,
            child: 3,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 2,
            child: 4,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });

        applier.rebuild_layout_boxes();
        applier.inherit();

        let parent = applier.solid_to_node[&2];
        let paint = applier.tree.get_node_context(parent).unwrap();
        assert_eq!(applier.tree.child_count(parent), 0);
        assert_eq!(paint.text.as_deref(), Some("Hello world"));
        assert_eq!(paint.text_runs.len(), 2);
        assert_eq!(paint.text_runs[0].range, 0..6);
        assert_eq!(paint.text_runs[1].range, 6..11);
        assert_eq!(paint.text_runs[1].font_weight, 700.0);
        assert_eq!(paint.text_runs[1].color, [255, 0, 0, 255]);
    }

    fn interactive_applier() -> Applier {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);

        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (button, width, height) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("button"),
                atoms.intern("width"),
                atoms.intern("height"),
            )
        };
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: button,
            attrs: Vec::new(),
        });
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: width,
            value: "100px",
        });
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: height,
            value: "50px",
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        for code in [event::POINTERDOWN, event::POINTERUP, event::CLICK] {
            applier.apply_op(&Op::AddEventListener {
                id: 2,
                event_type: code,
            });
        }
        let mut root_style = applier
            .tree
            .style(applier.root)
            .expect("root style")
            .clone();
        root_style.size.width = taffy::Dimension::length(800.0);
        root_style.size.height = taffy::Dimension::length(600.0);
        applier
            .tree
            .set_style(applier.root, root_style)
            .expect("viewport style");
        applier
            .tree
            .compute_layout(
                applier.root,
                taffy::geometry::Size {
                    width: taffy::AvailableSpace::Definite(800.0),
                    height: taffy::AvailableSpace::Definite(600.0),
                },
            )
            .expect("layout");
        let mut placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.update_scrollbar_visuals(&mut placed);
        applier.rebuild_hit_geometry(&placed);
        applier
    }

    #[test]
    fn host_layout_snapshot_reports_completed_rects_and_viewport() {
        let applier = interactive_applier();
        let placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.publish_layout_metrics(&placed, 800, 600);

        let json = applier
            .js
            .with(|ctx| {
                ctx.eval::<String, _>("__wabou_layout_snapshot(new Uint32Array([2, 999999]))")
            })
            .expect("layout snapshot");
        let snapshot: serde_json::Value = serde_json::from_str(&json).expect("snapshot JSON");

        assert_eq!(snapshot["viewport"]["width"], 800.0);
        assert_eq!(snapshot["viewport"]["height"], 600.0);
        assert_eq!(snapshot["nodes"].as_array().unwrap().len(), 1);
        assert_eq!(snapshot["nodes"][0]["id"], 2);
        assert_eq!(snapshot["nodes"][0]["rect"]["width"], 100.0);
        assert_eq!(snapshot["nodes"][0]["rect"]["height"], 50.0);
        assert_eq!(snapshot["nodes"][0]["clip"], snapshot["viewport"]);
    }

    #[test]
    fn pointer_sequence_hit_tests_and_synthesizes_one_click() {
        let mut applier = interactive_applier();
        assert_eq!(applier.hit_test(20.0, 20.0), Some(2));
        assert!(
            applier
                .handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1))
                .handled
        );
        assert!(
            applier
                .handle_event(pointer(PointerPhase::Up, 20.0, 20.0, 0))
                .handled
        );

        let codes = applier
            .js
            .with(|ctx| ctx.eval::<Vec<u8>, _>("globalThis.dispatched.map((x) => x[1])"))
            .expect("read dispatched events");
        assert_eq!(
            codes,
            vec![event::POINTERDOWN, event::POINTERUP, event::CLICK]
        );
    }

    #[test]
    fn dragging_inside_pressed_target_does_not_synthesize_a_click() {
        let mut applier = interactive_applier();
        applier.handle_event(pointer(PointerPhase::Down, 10.0, 20.0, 1));
        applier.handle_event(pointer(PointerPhase::Move, 80.0, 20.0, 1));
        applier.handle_event(pointer(PointerPhase::Up, 80.0, 20.0, 0));

        let codes = applier
            .js
            .with(|ctx| ctx.eval::<Vec<u8>, _>("globalThis.dispatched.map((x) => x[1])"))
            .expect("read dispatched events");
        assert!(codes.contains(&event::POINTERDOWN));
        assert!(codes.contains(&event::POINTERUP));
        assert!(!codes.contains(&event::CLICK));
        assert!(applier.pointer_down_target.is_none());
        assert!(applier.pointer_down_position.is_none());
        assert!(!applier.pointer_dragged);
    }

    #[test]
    fn coalesced_release_distance_also_suppresses_click() {
        let mut applier = interactive_applier();
        applier.handle_event(pointer(PointerPhase::Down, 10.0, 20.0, 1));
        applier.handle_event(pointer(PointerPhase::Up, 80.0, 20.0, 0));

        let click_count = applier
            .js
            .with(|ctx| {
                ctx.eval::<usize, _>("globalThis.dispatched.filter((x) => x[1] === 1).length")
            })
            .expect("read click count");
        assert_eq!(click_count, 0);
    }

    #[test]
    fn devtools_snapshot_exposes_real_layout_and_event_trace() {
        let mut applier = interactive_applier();
        let state = wabou_devtools::DebugState::shared();
        applier.set_debug_state(state.clone());
        let placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.last_viewport = (800, 600);
        applier.publish_debug_snapshot(&placed);
        applier.handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1));

        let state = state.read().unwrap();
        let snapshot = state.snapshot();
        assert_eq!(snapshot.status.viewport_width, 800);
        let button = snapshot.nodes.iter().find(|node| node.id == 2).unwrap();
        assert_eq!(button.tag, "button");
        assert_eq!(button.rect.width, 100.0);
        assert_eq!(button.rect.height, 50.0);
        assert!(
            state
                .frames()
                .iter()
                .any(|frame| { frame.direction == "hostToJs" && frame.record_count == 1 })
        );
    }

    #[test]
    fn devtools_snapshot_exposes_widget_local_and_ancestor_clip_coordinates() {
        let mut applier = interactive_applier();
        let state = wabou_devtools::DebugState::shared();
        applier.set_debug_state(state.clone());
        let widget_node = applier.solid_to_node[&2];
        applier
            .widgets
            .insert(widget_node, Box::new(MeasuringWidget([100.0, 50.0])));
        let mut placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        let root = placed
            .iter_mut()
            .find(|node| node.node_id == applier.root)
            .unwrap();
        root.own_clip = Some([0.0, 0.0, 80.0, 40.0]);
        root.own_clip_radius = 6.0;
        let widget = placed
            .iter_mut()
            .find(|node| node.node_id == widget_node)
            .unwrap();
        widget.clip = Some([0.0, 0.0, 80.0, 40.0]);
        widget.clip_radius = 6.0;
        widget.paint.border_radius = 12.0;

        applier.publish_debug_snapshot(&placed);

        let state = state.read().unwrap();
        let widget = state
            .snapshot()
            .nodes
            .iter()
            .find(|node| node.id == 2)
            .unwrap();
        assert_eq!(widget.clip.widget_local.as_ref().unwrap().radius, 12.0);
        assert_eq!(
            widget.clip.widget_local.as_ref().unwrap().coordinate_space,
            "content-local"
        );
        assert_eq!(widget.clip.chain.len(), 1);
        assert_eq!(widget.clip.chain[0].node_id, 1);
        assert_eq!(widget.clip.effective.as_ref().unwrap().rect.width, 80.0);
        assert_eq!(widget.clip.device_scale, 1.0);
    }

    #[test]
    fn releasing_outside_the_pressed_target_does_not_click() {
        let mut applier = interactive_applier();
        applier.handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1));
        applier.handle_event(pointer(PointerPhase::Up, 200.0, 200.0, 0));

        let click_count = applier
            .js
            .with(|ctx| {
                ctx.eval::<usize, _>("globalThis.dispatched.filter((x) => x[1] === 1).length")
            })
            .expect("read click count");
        assert_eq!(click_count, 0);
    }

    #[test]
    fn resize_observer_reports_initial_content_box_once() {
        let mut applier = interactive_applier();
        applier
            .js
            .with(|ctx| {
                ctx.eval::<(), _>(
                    r#"
                    __wabou_resize_observe(2);
                    "#,
                )
            })
            .expect("install resize hook");

        assert!(applier.dispatch_resize_changes());
        assert!(!applier.dispatch_resize_changes());
        let changes = applier
            .js
            .with(|ctx| ctx.eval::<Vec<Vec<f32>>, _>("globalThis.resizeChanges"))
            .expect("read resize changes");
        assert_eq!(changes, vec![vec![2.0, 100.0, 50.0]]);
    }

    #[test]
    fn devtools_snapshot_exposes_layout_and_redacts_secrets() {
        let mut applier = interactive_applier();
        let password = applier.atoms.borrow_mut().intern("password");
        applier.apply_op(&Op::SetAttribute {
            id: 2,
            name: password,
            value: "do-not-leak",
        });
        let state = wabou_devtools::DebugState::shared();
        applier.set_debug_state(state.clone());
        let placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.last_viewport = (800, 600);
        applier.publish_debug_snapshot(&placed);

        let state = state.read().unwrap();
        let snapshot = state.snapshot();
        assert_eq!(snapshot.status.viewport_width, 800);
        let button = snapshot.nodes.iter().find(|node| node.id == 2).unwrap();
        assert_eq!(button.tag, "button");
        assert_eq!(button.rect.width, 100.0);
        assert_eq!(
            button
                .attrs
                .iter()
                .find(|(name, _)| name == "password")
                .unwrap()
                .1,
            "[REDACTED]"
        );
    }

    #[test]
    fn runtime_transform_updates_paint_without_invalidating_layout() {
        let mut applier = interactive_applier();
        applier.invalidation.remove(InvalidationFlags::LAYOUT);
        applier.apply_op(&Op::SetTransform2D {
            id: 2,
            matrix: [1.0, 0.0, 0.0, 1.0, 12.5, -3.25],
        });

        assert!(!applier.invalidation.contains(InvalidationFlags::LAYOUT));
        let node = applier.solid_to_node[&2];
        assert_eq!(
            applier
                .tree
                .get_node_context(node)
                .unwrap()
                .runtime_transform,
            Some([1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
        );
        let placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.rebuild_hit_geometry(&placed);
        assert_ne!(applier.hit_test(5.0, 20.0), Some(2));
        assert_eq!(applier.hit_test(32.5, 16.75), Some(2));

        let transform = applier.atoms.borrow_mut().intern("transform");
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: transform,
            value: "translate(2px, 3px)",
        });
        let paint = applier.tree.get_node_context(node).unwrap();
        assert_eq!(
            paint.runtime_transform,
            Some([1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
        );
        assert_eq!(
            applier.runtime_transforms.get(&node),
            Some(&[1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
        );
    }

    #[test]
    fn protocol_shadows_apply_vello_parameters_without_string_parsing() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let div = applier.atoms.borrow_mut().intern("div");
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: div,
            attrs: Vec::new(),
        });
        applier.apply_op(&Op::SetShadows {
            id: 2,
            shadows: vec![crate::protocol::ShadowValue {
                offset_x: 3.0,
                offset_y: 7.0,
                spread: -2.0,
                std_dev: 5.5,
                color: 0x336699cc,
                radius: Some(11.0),
            }],
        });

        let node = applier.solid_to_node[&2];
        let paint = applier.tree.get_node_context(node).unwrap();
        assert_eq!(
            paint.shadows,
            vec![wabou_shell::style::Shadow {
                offset_x: 3.0,
                offset_y: 7.0,
                spread: -2.0,
                std_dev: 5.5,
                color: Color::from_rgba8(0x33, 0x66, 0x99, 0xcc),
                radius: Some(11.0),
            }]
        );
    }

    #[test]
    fn overflow_container_supports_wheel_and_selection_autoscroll() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (div, width, height, overflow_y, flex_shrink) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("div"),
                atoms.intern("width"),
                atoms.intern("height"),
                atoms.intern("overflow-y"),
                atoms.intern("flex-shrink"),
            )
        };
        for id in [2, 3] {
            applier.apply_op(&Op::CreateElement {
                id,
                tag: div,
                attrs: Vec::new(),
            });
            applier.apply_op(&Op::SetStyle {
                id,
                prop: width,
                value: "100px",
            });
        }
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: height,
            value: "100px",
        });
        applier.apply_op(&Op::SetStyle {
            id: 2,
            prop: overflow_y,
            value: "auto",
        });
        applier.apply_op(&Op::SetStyle {
            id: 3,
            prop: height,
            value: "300px",
        });
        applier.apply_op(&Op::SetStyle {
            id: 3,
            prop: flex_shrink,
            value: "0",
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 2,
            child: 3,
        });

        applier.apply_op(&Op::CreateText {
            id: 4,
            text: "scroll selectable",
        });
        applier.apply_op(&Op::AppendChild {
            parent: 3,
            child: 4,
        });
        applier.rebuild_layout_boxes();
        applier.inherit();

        let mut root_style = applier
            .tree
            .style(applier.root)
            .expect("root style")
            .clone();
        root_style.size.width = taffy::Dimension::length(800.0);
        root_style.size.height = taffy::Dimension::length(600.0);
        applier
            .tree
            .set_style(applier.root, root_style)
            .expect("viewport style");
        applier
            .tree
            .compute_layout(
                applier.root,
                taffy::geometry::Size {
                    width: taffy::AvailableSpace::Definite(800.0),
                    height: taffy::AvailableSpace::Definite(600.0),
                },
            )
            .expect("layout");
        let placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.rebuild_hit_geometry(&placed);

        let container = applier.solid_to_node[&2];
        applier.invalidation.remove(InvalidationFlags::LAYOUT);
        assert_eq!(
            applier.tree.style(container).unwrap().overflow.y,
            taffy::Overflow::Scroll
        );
        assert_ne!(
            applier.hit_test(10.0, 150.0),
            Some(3),
            "overflow must clip hits"
        );
        applier.apply_op(&Op::SetTransform2D {
            id: 2,
            matrix: [1.0, 0.0, 0.0, 1.0, 200.0, 0.0],
        });
        let placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.rebuild_hit_geometry(&placed);
        assert_eq!(applier.hit_test(210.0, 50.0), Some(3));
        assert_ne!(applier.hit_test(210.0, 150.0), Some(3));
        applier.apply_op(&Op::SetTransform2D {
            id: 2,
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        });
        let placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.rebuild_hit_geometry(&placed);
        let response = applier.handle_event(UiEvent::Wheel(wabou_shell::WheelEvent {
            position: Point { x: 10.0, y: 10.0 },
            delta_x: 0.0,
            delta_y: 50.0,
            modifiers: Modifiers::default(),
        }));
        assert!(response.handled);
        assert_eq!(applier.scroll_offsets[&container], [0.0, 50.0]);
        assert!(
            !applier.invalidation.contains(InvalidationFlags::LAYOUT),
            "scroll offsets must not invalidate intrinsic layout"
        );

        applier.apply_op(&Op::ScrollTo {
            id: 2,
            x: f32::NAN,
            y: 120.0,
        });
        assert_eq!(applier.scroll_offsets[&container], [0.0, 120.0]);
        applier.apply_op(&Op::ScrollBy {
            id: 2,
            x: 0.0,
            y: -20.0,
        });
        assert_eq!(applier.scroll_offsets[&container], [0.0, 100.0]);
        applier.apply_op(&Op::ScrollTo {
            id: 2,
            x: f32::NAN,
            y: -100.0,
        });
        assert_eq!(applier.scroll_offsets[&container], [0.0, 0.0]);

        let mut placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.update_scrollbar_visuals(&mut placed);
        applier.rebuild_hit_geometry(&placed);
        let down = applier.handle_event(pointer(PointerPhase::Down, 95.0, 16.0, 1));
        assert!(
            down.handled,
            "the native thumb must capture pointer down; hits={:?}",
            applier
                .scrollbar_hits
                .iter()
                .map(|hit| (
                    hit.placed.rect,
                    hit.placed.own_clip,
                    hit.placed.scroll.range,
                    wabou_shell::scrollbar::thumb(&hit.placed, ScrollAxis::Vertical)
                ))
                .collect::<Vec<_>>()
        );
        let moved = applier.handle_event(pointer(PointerPhase::Move, 95.0, 50.0, 1));
        assert!(moved.handled);
        assert!(
            (applier.scroll_offsets[&container][1] - 102.0).abs() < 1.0,
            "34 thumb pixels should map through the shared geometry ratio"
        );
        let up = applier.handle_event(pointer(PointerPhase::Up, 95.0, 50.0, 0));
        assert!(up.handled);
        assert!(applier.scrollbar_drag.is_none());

        applier.scroll_offsets.insert(container, [0.0, 0.0]);
        let mut placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.update_scrollbar_visuals(&mut placed);
        applier.rebuild_hit_geometry(&placed);
        let track = applier.handle_event(pointer(PointerPhase::Down, 95.0, 80.0, 1));
        assert!(track.handled);
        assert_eq!(applier.scroll_offsets[&container][1], 100.0);

        applier.scroll_offsets.insert(container, [0.0, 0.0]);
        let mut tcx = TextContext::new();
        let mut placed =
            layout::flatten_with_scroll(&applier.tree, applier.root, &applier.scroll_offsets);
        applier.placed_rects = placed
            .iter()
            .map(|placed| (placed.node_id, placed.rect))
            .collect();
        applier.prepare_text_selection(&mut placed, &mut tcx);
        let origin = applier.selectable_text[&3].origin;
        applier.begin_text_selection(
            3,
            f64::from(origin[0] + 1.0),
            f64::from(origin[1] + 5.0),
            Modifiers::empty(),
        );
        applier.pointer_buttons = 1;
        applier.pointer_position = (200.0, 140.0);
        applier.extend_text_selection(None, 200.0, 140.0);
        // Model a cross-panel drag: the stable anchor is outside this
        // overflow container while the focus endpoint remains inside it.
        applier
            .active_text_selection
            .as_mut()
            .unwrap()
            .anchor_target = 1;
        applier.arm_text_selection_autoscroll();
        assert!(applier.animation_deadline().is_some());
        applier.next_text_selection_scroll = Some(Instant::now() - Duration::from_millis(1));
        assert!(applier.tick_text_selection_autoscroll());
        let first_scroll = applier.scroll_offsets[&container][1];
        assert!(first_scroll > 0.0);
        applier.next_text_selection_scroll = Some(Instant::now() - Duration::from_millis(1));
        assert!(applier.tick_text_selection_autoscroll());
        assert!(applier.scroll_offsets[&container][1] > first_scroll);
        applier.pointer_buttons = 0;
        applier.next_text_selection_scroll = Some(Instant::now() - Duration::from_millis(1));
        assert!(!applier.tick_text_selection_autoscroll());
        assert!(applier.next_text_selection_scroll.is_none());
        applier
            .active_text_selection
            .as_mut()
            .unwrap()
            .anchor_target = 3;
        assert_eq!(
            applier.selected_text().as_deref(),
            Some("scroll selectable")
        );
    }

    #[test]
    fn later_overlay_content_blocks_an_underlying_scrollbar_attachment() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let view = applier.atoms.borrow_mut().intern("view");
        for id in [2, 3] {
            applier.apply_op(&Op::CreateElement {
                id,
                tag: view,
                attrs: vec![],
            });
        }
        let owner = applier.solid_to_node[&2];
        let overlay = applier.solid_to_node[&3];
        let root = applier.root;
        let placed = |node_id, scroll| PlacedNode {
            node_id,
            parent_node_id: Some(root),
            depth: 1,
            rect: [0.0, 0.0, 100.0, 100.0],
            content_origin: [0.0, 0.0],
            content_size: [100.0, 100.0],
            clip: None,
            clip_radius: 0.0,
            clip_depth: None,
            own_clip: None,
            own_clip_radius: 0.0,
            border_widths: [0.0; 4],
            scroll,
            paint: Paint::default(),
        };
        let owner_scroll = layout::ScrollMetrics {
            port: [0.0, 0.0, 100.0, 100.0],
            scrollable: [false, true],
            range: [0.0, 900.0],
            offset: [0.0, 0.0],
            opacity: 1.0,
            interaction: 0,
        };
        let owner_placed = placed(owner, owner_scroll);
        applier.rebuild_hit_geometry(std::slice::from_ref(&owner_placed));
        assert!(applier.scrollbar_at(95.0, 16.0).is_some());

        let overlay_placed = placed(overlay, layout::ScrollMetrics::default());
        applier.rebuild_hit_geometry(&[owner_placed, overlay_placed]);
        assert_eq!(applier.scrollbar_at(95.0, 16.0), None);
        assert_eq!(applier.hit_test(95.0, 16.0), Some(3));
    }

    #[test]
    fn semantic_snapshot_promotes_modal_plane_and_keeps_focus_inside() {
        let js = JsRuntime::new().expect("runtime");
        install_host_frame_test_hook(&js);
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (button, view, role, aria_label, aria_modal) = {
            let mut atoms = applier.atoms.borrow_mut();
            (
                atoms.intern("button"),
                atoms.intern("view"),
                atoms.intern("role"),
                atoms.intern("aria-label"),
                atoms.intern("aria-modal"),
            )
        };
        applier.apply_op(&Op::CreateElement {
            id: 2,
            tag: button,
            attrs: vec![(aria_label, "Background")],
        });
        applier.apply_op(&Op::CreateElement {
            id: 3,
            tag: view,
            attrs: vec![
                (role, "dialog"),
                (aria_label, "Settings"),
                (aria_modal, "true"),
            ],
        });
        applier.apply_op(&Op::CreateElement {
            id: 4,
            tag: button,
            attrs: vec![(aria_label, "Save")],
        });
        applier.apply_op(&Op::AppendChild {
            parent: 3,
            child: 4,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 2,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 3,
        });
        applier.rebuild_layout_boxes();
        applier.apply_op(&Op::SetOverlayPlane { id: 3, plane: 2 });
        applier.focused_target = Some(4);

        let root = applier.root;
        let background = applier.solid_to_node[&2];
        let modal = applier.solid_to_node[&3];
        let save = applier.solid_to_node[&4];
        let paint = |plane| Paint {
            overlay_plane: plane,
            ..Paint::default()
        };
        let node = |node_id, parent_node_id, depth, paint| PlacedNode {
            node_id,
            parent_node_id,
            depth,
            rect: [0.0, 0.0, 100.0, 100.0],
            content_origin: [0.0, 0.0],
            content_size: [100.0, 100.0],
            clip: None,
            clip_radius: 0.0,
            clip_depth: None,
            own_clip: None,
            own_clip_radius: 0.0,
            border_widths: [0.0; 4],
            scroll: layout::ScrollMetrics::default(),
            paint,
        };
        let mut save_paint = paint(OverlayPlane::Content);
        save_paint.runtime_transform = Some([1.0, 0.0, 0.0, 1.0, 10.0, 5.0]);
        let placed = vec![
            node(background, Some(root), 1, paint(OverlayPlane::Content)),
            node(modal, Some(root), 1, paint(OverlayPlane::Modal)),
            node(save, Some(modal), 2, save_paint),
        ];
        applier.rebuild_hit_geometry(&placed);
        applier.rebuild_semantic_snapshot(&placed);
        let snapshot = &applier.semantic_snapshot;
        assert_eq!(snapshot.root_children, vec![2, 3]);
        assert_eq!(snapshot.modal_root, Some(3));
        assert_eq!(snapshot.focus, Some(4));
        assert!(snapshot.nodes.iter().any(|node| {
            node.id == 3
                && node.role == SemanticRole::Dialog
                && node.label.as_deref() == Some("Settings")
        }));
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == 4)
                .unwrap()
                .bounds,
            [10.0, 5.0, 110.0, 105.0]
        );
        applier.apply_op(&Op::AddEventListener {
            id: 4,
            event_type: event::CLICK,
        });
        assert!(!applier.handle_semantic_action(SemanticAction::Click { target: 2 }));
        assert!(applier.handle_semantic_action(SemanticAction::Click { target: 4 }));
        applier.focused_target = None;
        assert!(applier.handle_semantic_action(SemanticAction::Focus { target: 4 }));
        assert_eq!(applier.focused_target, Some(4));

        applier.apply_op(&Op::CreateElement {
            id: 5,
            tag: view,
            attrs: vec![
                (role, "dialog"),
                (aria_label, "Confirm"),
                (aria_modal, "true"),
            ],
        });
        applier.apply_op(&Op::CreateElement {
            id: 6,
            tag: button,
            attrs: vec![(aria_label, "Continue")],
        });
        applier.apply_op(&Op::AppendChild {
            parent: 5,
            child: 6,
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: 5,
        });
        applier.apply_op(&Op::SetOverlayPlane { id: 5, plane: 2 });
        let confirm = applier.solid_to_node[&5];
        let continue_button = applier.solid_to_node[&6];
        let mut continue_paint = paint(OverlayPlane::Content);
        continue_paint.runtime_transform = Some([1.0, 0.0, 0.0, 1.0, 20.0, 10.0]);
        let placed = vec![
            node(background, Some(root), 1, paint(OverlayPlane::Content)),
            node(modal, Some(root), 1, paint(OverlayPlane::Modal)),
            node(save, Some(modal), 2, Paint::default()),
            node(confirm, Some(root), 1, paint(OverlayPlane::Modal)),
            node(continue_button, Some(confirm), 2, continue_paint),
        ];
        applier.rebuild_hit_geometry(&placed);
        applier.rebuild_semantic_snapshot(&placed);
        assert_eq!(applier.semantic_snapshot.modal_root, Some(5));
        assert_eq!(applier.semantic_snapshot.focus, Some(5));
        assert!(
            !applier.handle_semantic_action(SemanticAction::Focus { target: 4 }),
            "an older modal must be inert while a newer modal is topmost"
        );
        assert!(applier.handle_semantic_action(SemanticAction::Focus { target: 6 }));
    }
}
