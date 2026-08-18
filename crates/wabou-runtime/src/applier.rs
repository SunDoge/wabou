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
use std::sync::Arc;
#[cfg(test)]
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;

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
    SemanticAction, SemanticNode, SemanticRole, SemanticSnapshot, SemanticStates,
    SemanticToggleState, UiEvent, WakeCallback,
};

use crate::asset_cache::ResourceCache;
use crate::host_frame::{HostEvent, HostNodeEvent, NodeEventPayload, ResizeObservation};

mod debug_projection;
mod effect_bridge;
mod focus;
mod frame_source;
mod input_router;
mod interaction;
mod node_store;
mod projections;
mod protocol_apply;
mod reload;
mod resources;
mod runtime_updates;
mod scroll;
mod semantics;
mod style_resolution;
mod text_selection;
mod widget_bridge;
mod widget_manager;
use crate::atom::{Atom, AtomPool};
use crate::host_message::{
    DEFAULT_HOST_MESSAGE_CAPACITY, HostMessageHandle, HostMessageInbox, host_message_channel,
};
use crate::inline_context::{
    InlineFormattingContext, NodeFacts, rect_has_nonzero_lp, rect_has_nonzero_lpa, size_is_explicit,
};
use crate::jsrt::{JsRuntime, LayoutMetric, LayoutMetricsSnapshot, LayoutRect, ResizeTargets};
use crate::protocol::{Frame, Op, decode_frame};
use crate::protocol::{event, event_data};
use crate::style_ir::{self, StyleSheet, StylesheetUpdate};
use effect_bridge::EffectBridge;
#[cfg(test)]
use input_router::EventMask;
use input_router::{HitClip, HitItem, HitNode, InputRouter, hit_contains};
use node_store::NodeStore;
use projections::FrameProjections;
#[cfg(test)]
use reload::plan_hmr_batch;
use reload::{HmrBatch, ReloadState};
pub use reload::{HmrDrainResult, ReloadHandle, ReloadMsg};
use resources::{ImageLoadResult, ResourceState};
use wabou_widgets::builtin_factories;
use widget_manager::WidgetManager;

#[derive(serde::Deserialize)]
struct NetworkImageSource {
    kind: String,
    url: String,
    format: String,
    cache: String,
}

fn remote_image_url(encoded: &str) -> Option<String> {
    let source: NetworkImageSource = serde_json::from_str(encoded).ok()?;
    (source.kind == "network" && source.format == "raster" && source.cache == "memory")
        .then_some(source.url)
}

fn declared_attribute_is(
    declared: &Declared,
    atoms: &AtomPool,
    name: &str,
    expected: Option<&str>,
) -> bool {
    declared.attrs.iter().any(|(atom, value)| {
        atoms.resolve(*atom) == Some(name)
            && expected.is_none_or(|expected| value.as_ref() == expected)
    })
}

fn subtree_has_attribute(
    node_store: &NodeStore,
    atoms: &AtomPool,
    mut node: NodeId,
    name: &str,
    expected: Option<&str>,
) -> bool {
    loop {
        if node_store
            .declared
            .get(&node)
            .is_some_and(|declared| declared_attribute_is(declared, atoms, name, expected))
        {
            return true;
        }
        let Some(parent) = node_store.logical_parent.get(&node).copied() else {
            return false;
        };
        node = parent;
    }
}

// Widget actions retain their tagged 32-bit namespace. Native effects use a
// process-wide sequence so window resource handles stay unique across runtimes.
const JS_HOST_ACTION_NAMESPACE: u64 = 1 << 31;
const HOST_ACTION_SEQUENCE_MASK: u64 = JS_HOST_ACTION_NAMESPACE - 1;
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
            Self::Typed(value) => value.clone(),
        }
    }
}

#[derive(Clone)]
struct InlineProperty {
    name: Arc<str>,
    inherited: bool,
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

/// Immutable view of a node's resolved layout and paint state.
///
/// This is intended for semantic renderer tests: assertions can stop at the
/// computed-style boundary without depending on Taffy's mutable tree or a GPU
/// pixel comparison. Native scene/widget handles are deliberately excluded.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputedNodeSnapshot {
    /// Solid-side retained node identifier.
    pub solid_id: u32,
    /// Resolved class names in authored order.
    pub classes: Vec<String>,
    /// Final Taffy layout style.
    pub layout: taffy::Style,
    /// Resolved background fill.
    pub background: Option<Color>,
    /// Resolved opacity.
    pub opacity: f32,
    /// Resolved static transforms.
    pub transforms: Vec<PaintTransform>,
    /// Resolved outer shadows.
    pub shadows: Vec<wabou_shell::style::Shadow>,
    /// Uniform border radius in logical pixels.
    pub border_radius: f32,
    /// Uniform border width and color.
    pub border: Option<(f32, Color)>,
    /// Non-layout outline width.
    pub outline_width: f32,
    /// Gap between border box and outline.
    pub outline_offset: f32,
    /// Outline color.
    pub outline_color: Option<Color>,
    /// Resolved platform cursor.
    pub cursor: wabou_shell::style::CursorStyle,
    /// Resolved text color.
    pub text_color: Color,
    /// Resolved font size in logical pixels.
    pub font_size: f32,
    /// Resolved numeric font weight.
    pub font_weight: f32,
    /// Resolved line height and whether it is font-relative.
    pub line_height: Option<(f32, bool)>,
    /// Whether normal inline wrapping is enabled.
    pub wrap_text: bool,
    /// Whether overflowing single-line text uses an ellipsis.
    pub text_ellipsis: bool,
    /// Whether pointer selection is enabled.
    pub text_selectable: bool,
    /// Whether one selection gesture selects all text.
    pub text_select_all: bool,
    /// Resolved text alignment.
    pub text_align: TextAlign,
    /// Whether the node itself participates in pointer hit testing.
    pub pointer_events: bool,
    /// Sibling-relative paint and hit-test order.
    pub z_index: i32,
    /// Resolved preferred font family.
    pub font_family: Option<Arc<str>>,
    /// Host-provided intrinsic content size.
    pub intrinsic_size: Option<[f32; 2]>,
    /// Host-driven transform composed after static transforms.
    pub runtime_transform: Option<[f32; 6]>,
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
        /// Paint-space geometry changed without changing layout or semantics.
        const GEOMETRY = 1 << 3;
    }
}

/// CSS properties that inherit to descendants. A SetStyle touching one of
/// these (or the `font` shorthand) must take the slow path — re-derive + run
/// the inherit pass — so children see the new value. Other inline properties
/// take [`Applier::apply_inline_ir_fast`].
const INHERITED_PROPERTIES: &[&str] = &[
    "color",
    "font-size",
    "font-weight",
    "font-family",
    "line-height",
    "text-align",
    "white-space",
    "user-select",
    "cursor",
];

/// Retained Solid/QuickJS frame source consumed by `wabou-shell`.
///
/// The applier owns the JavaScript runtime, decoded node tree, style cascade,
/// widget instances, input routing, and host projections for one logical
/// window. Mutation ops are applied only at frame boundaries.
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
    /// Property atoms are stable for the runtime lifetime. Resolve and classify
    /// each one once so dynamic inline updates do not allocate a property name
    /// or repeatedly scan the inherited-property table.
    inline_properties: HashMap<Atom, InlineProperty>,
    /// Rejections from the latest cascade pass, keyed by native node for
    /// DevTools inspection.
    style_diagnostics: HashMap<NodeId, Vec<String>>,
    resources: ResourceState,
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
    /// Number of non-empty JS protocol frames applied by this runtime.
    protocol_revision: u64,
    reload: ReloadState,
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
    #[cfg(feature = "profiling")]
    profile_class_cache_hits: u64,
    #[cfg(feature = "profiling")]
    profile_class_cache_misses: u64,
    #[cfg(feature = "profiling")]
    profile_runtime_utility_fallbacks: u64,
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
    scroll_metrics: HashMap<NodeId, wabou_shell::layout::ScrollMetrics>,
    scrollbar_drag: Option<ScrollbarDrag>,
    hovered_scrollbar: Option<(NodeId, ScrollAxis)>,
    scrollbar_activity: HashMap<NodeId, Instant>,
    widget_manager: WidgetManager,
    pending_host_actions: Rc<RefCell<VecDeque<wabou_shell::HostAction>>>,
    effect_bridge: EffectBridge,
    wake_callback: Option<WakeCallback>,
    scroll_offsets: HashMap<NodeId, [f32; 2]>,
    /// Native scroll changes coalesced by Solid target until the next JS tick.
    pending_scroll_events: HashMap<u32, [f32; 2]>,
    /// Protocol frames commonly create a node and then set several properties
    /// on it. Resolve style once at FrameEnd instead of after every operation.
    batching_styles: bool,
    dirty_styles: HashSet<NodeId>,
    /// Taffy layout and inherited paint are retained across scroll-only frames.
    layout_viewport: Option<(u32, u32)>,
    /// Bounded host→JS message inbox. Producers use [`HostMessageHandle`].
    host_message_inbox: HostMessageInbox,
    host_message_handle: HostMessageHandle,
    host_message_cancellation: CancellationToken,
}

impl Drop for Applier {
    fn drop(&mut self) {
        self.host_message_cancellation.cancel();
    }
}

impl Applier {
    /// Monotonically increasing count of non-empty JS-to-host protocol frames.
    ///
    /// Deterministic headless drivers can use this to wait for UI commits
    /// without inspecting private retained-tree state.
    pub fn protocol_revision(&self) -> u64 {
        self.protocol_revision
    }

    /// Build an applier over an already-booted [`JsRuntime`] (the host owns
    /// boot: `JsRuntime::new().boot(js)` for the static-bundle path, or
    /// `JsRuntime::new_vite(url).boot_vite(entry)` for dev mode).
    pub fn from_runtime(js: JsRuntime, base_color: Color) -> Self {
        Self::from_runtime_with_factories(js, builtin_factories(), base_color)
    }

    /// Like `from_runtime` but with a widget factory registry (from `HostBuilder`).
    pub fn from_runtime_with_factories(
        js: JsRuntime,
        widget_factories: HashMap<String, wabou_shell::WidgetFactory>,
        base_color: Color,
    ) -> Self {
        Self::from_runtime_with_factories_and_window(js, widget_factories, base_color, 1)
    }

    /// Build an applier with explicit widget factories and logical window id.
    pub fn from_runtime_with_factories_and_window(
        js: JsRuntime,
        widget_factories: HashMap<String, wabou_shell::WidgetFactory>,
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
        let (host_message_handle, host_message_inbox) =
            host_message_channel(DEFAULT_HOST_MESSAGE_CAPACITY);
        let host_message_cancellation = CancellationToken::new();

        let pending_host_actions = Rc::new(RefCell::new(VecDeque::new()));
        let effect_bridge = EffectBridge::install(&js, window_id);
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
            inline_properties: HashMap::new(),
            style_diagnostics: HashMap::new(),
            resources: ResourceState::default(),
            runtime_transforms: HashMap::new(),
            overlay_planes: HashMap::new(),
            scrollbar_styles: HashMap::new(),
            base_color,
            atoms,
            input: InputRouter::new(),
            has_raf: true,
            protocol_revision: 0,
            reload: ReloadState::default(),
            pending_css: Some(pending_css),
            pending_color_theme: Some(pending_color_theme),
            pending_color_palette: Some(pending_color_palette),
            pending_fonts: Some(pending_fonts),
            frame_stats: Some(frame_stats),
            projections: FrameProjections::new(layout_metrics),
            resize_targets,
            invalidation: InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT,
            js_tick_ema: 0.0,
            #[cfg(feature = "profiling")]
            profile_class_cache_hits: 0,
            #[cfg(feature = "profiling")]
            profile_class_cache_misses: 0,
            #[cfg(feature = "profiling")]
            profile_runtime_utility_fallbacks: 0,
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
            scroll_metrics: HashMap::new(),
            scrollbar_drag: None,
            hovered_scrollbar: None,
            scrollbar_activity: HashMap::new(),
            widget_manager: WidgetManager::new(widget_factories),
            pending_host_actions,
            effect_bridge,
            wake_callback: None,
            scroll_offsets: HashMap::new(),
            pending_scroll_events: HashMap::new(),
            batching_styles: false,
            dirty_styles: HashSet::new(),
            layout_viewport: None,
            host_message_inbox,
            host_message_handle,
            host_message_cancellation,
        }
    }

    /// Boot the application after all Applier-owned host bridges have been
    /// installed. This ordering permits window APIs during initial render.
    pub fn boot(&mut self, source: &str) -> rquickjs::Result<()> {
        self.js.boot(source)
    }

    pub(crate) fn boot_with_source_map(
        &mut self,
        source: &str,
        source_map: Option<&[u8]>,
    ) -> rquickjs::Result<()> {
        self.js.boot_with_source_map(source, source_map)
    }

    /// Evaluate an additional script in the booted application realm.
    pub fn eval_script(&self, source: &str) -> rquickjs::Result<()> {
        self.js.eval_script(source)
    }

    #[cfg(feature = "vite")]
    /// Boots an application entry module through the Vite development loader.
    ///
    /// Host bridges must already be installed, just as for [`Self::boot`].
    pub fn boot_vite(&mut self, entry: &str) -> rquickjs::Result<()> {
        self.js.boot_vite(entry)
    }

    pub(crate) fn set_effect_trace(&mut self, trace: crate::effect_trace::EffectTrace) {
        self.effect_bridge.set_trace(trace);
    }

    /// Publish resolved application-private directories to native effects.
    pub fn set_app_directories(&mut self, directories: wabou_shell::AppDirectories) {
        self.effect_bridge.set_app_directories(directories);
    }

    pub(crate) fn set_asset_cache(&mut self, cache: Arc<ResourceCache>) {
        self.resources.set_cache(cache);
    }

    /// Attach the immutable snapshot store published through DevTools.
    pub fn set_debug_state(&mut self, state: wabou_devtools::SharedDebugState) {
        self.js.set_debug_state(state.clone());
        self.projections.debug_state = Some(state);
        self.projections.debug_dirty = true;
    }

    /// Cloneable handle for background tasks / streams to push application
    /// messages toward JS (`host.subscribe` on the guest side).
    pub fn host_message_handle(&self) -> HostMessageHandle {
        self.host_message_handle.clone()
    }

    pub(crate) fn host_message_context(&self, window_id: u64) -> crate::HostMessageContext {
        crate::HostMessageContext::new(
            window_id,
            self.host_message_handle(),
            self.host_message_cancellation.clone(),
            self.js.tokio_handle(),
        )
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
            outline_width: paint.outline_width,
            outline_offset: paint.outline_offset,
            outline_color: paint.outline_color,
            cursor: paint.cursor,
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
