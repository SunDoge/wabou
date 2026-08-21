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
use vello::kurbo::{Affine, Point};
#[cfg(any(feature = "devtools", test))]
use vello::kurbo::{Rect, Stroke};
use vello::peniko::Color;
#[cfg(any(feature = "devtools", test))]
use vello::peniko::Fill;
use wabou_shell::layout::{self, PlacedNode, SubtreeEvent, subtree_events};
use wabou_shell::scrollbar::{
    ScrollAxis, ScrollbarPart, ScrollbarTarget, drag_ratio as scrollbar_drag_ratio,
    hit as scrollbar_hit,
};
use wabou_shell::style::{
    self, DeclaredPaint, HostPaint, InheritedPaint, IrValue, OverlayPlane, Paint, PaintTransform,
    ScrollbarStyle, ScrollbarVisibility, TextAlign,
};
use wabou_shell::text::TextContext;
#[cfg(any(feature = "devtools", test))]
use wabou_shell::text::layout_text_styled;
use wabou_shell::{
    EventResponse, FrameSource, FrameStats, KeyPhase, Modifiers, PointerButton, PointerPhase,
    SemanticAction, SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole, SemanticSnapshot,
    SemanticStates, SemanticToggleState, UiEvent, WakeCallback,
};

use crate::asset_cache::ResourceCache;
use crate::host_frame::{HostEvent, HostNodeEvent, NodeEventPayload, ResizeObservation};
use crate::protocol::NodeKey;

#[cfg(any(feature = "devtools", test))]
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
use crate::inline_context::{InlineFormattingContext, NodeFacts};
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
use scroll::{ScrollState, ScrollbarDrag, ScrollbarHit};
use style_resolution::StyleState;
use text_selection::TextSelectionState;
#[cfg(test)]
use text_selection::{SelectableText, TextSelectionGranularity};
use wabou_widgets::builtin_factories;
use widget_manager::WidgetManager;

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

fn subtree_blocks_interaction(node_store: &NodeStore, mut node: NodeId) -> bool {
    loop {
        if node_store
            .declared
            .get(&node)
            .is_some_and(|declared| declared.interaction_blocked)
        {
            return true;
        }
        let Some(parent) = node_store.logical_parent.get(&node).copied() else {
            return false;
        };
        node = parent;
    }
}

// Widget actions retain their tagged 32-bit namespace. Native effect request
// ids use a process-wide sequence so completions remain unambiguous across
// runtimes; persistent resources use independent generational keys.
const JS_HOST_ACTION_NAMESPACE: u64 = 1 << 31;
const HOST_ACTION_SEQUENCE_MASK: u64 = JS_HOST_ACTION_NAMESPACE - 1;
const CLASS_RESOLUTION_CACHE_CAPACITY: usize = 1024;

#[derive(Clone)]
struct ResolvedClassDeclaration {
    property: String,
    value: wabou_shell::style::IrValue,
    #[cfg(any(feature = "devtools", test))]
    source: Option<Atom>,
}

#[derive(Clone)]
struct CachedClassResolution {
    declarations: Vec<ResolvedClassDeclaration>,
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
    /// Typed host behavior authored by the JS Text primitive.
    text_behavior: u8,
    /// Maximum rendered text lines; zero means unlimited.
    text_max_lines: u32,
    /// Explicit native focus order; `None` means this node cannot receive focus.
    focus_order: Option<i32>,
    /// Excludes this node and its logical subtree from input and semantics.
    interaction_blocked: bool,
    /// Contains sequential focus within this logical subtree while present.
    focus_contained: bool,
    /// Trusted inline SVG source authored through the typed graphic contract.
    svg_source: Option<Arc<str>>,
    /// Network raster URL authored through the typed graphic contract.
    network_image_url: Option<Arc<str>>,
    /// Decoded local-coordinate vector path.
    vector_path: Option<Arc<wabou_shell::style::VectorPath>>,
}

impl Declared {
    fn attribute(&self, atoms: &AtomPool, wanted: &str) -> Option<Arc<str>> {
        atoms
            .get(wanted)
            .and_then(|name| self.attrs.get(&name))
            .cloned()
    }
}

/// Immutable view of a node's resolved layout and paint state.
///
/// This is intended for semantic renderer tests: assertions can stop at the
/// computed-style boundary without depending on Taffy's mutable tree or a GPU
/// pixel comparison. Native scene/widget handles are deliberately excluded.
#[derive(Clone, Debug, PartialEq)]
pub struct ComputedNodeSnapshot {
    /// Solid-side retained node identifier.
    pub solid_id: NodeKey,
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
    /// Maximum rendered text lines; zero means unlimited.
    pub text_max_lines: u32,
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

/// QuickJS and host-bridge state with one shared cancellation lifetime.
struct RuntimeSession {
    js: JsRuntime,
    has_raf: bool,
    protocol_revision: u64,
    reload: ReloadState,
    pending_css: Option<Rc<RefCell<Option<StylesheetUpdate>>>>,
    pending_color_theme: Option<Rc<RefCell<Option<String>>>>,
    pending_color_palette: Option<Rc<RefCell<Option<Vec<u32>>>>>,
    pending_fonts: Option<Rc<RefCell<Vec<Vec<u8>>>>>,
    frame_stats: Option<Rc<RefCell<Option<FrameStats>>>>,
    pending_host_actions: Rc<RefCell<VecDeque<wabou_shell::HostAction>>>,
    effect_bridge: EffectBridge,
    wake_callback: Option<WakeCallback>,
    host_message_inbox: HostMessageInbox,
    host_message_handle: HostMessageHandle,
    host_message_cancellation: CancellationToken,
    host_tasks: Arc<crate::host_message::HostTaskTracker>,
}

impl Drop for RuntimeSession {
    fn drop(&mut self) {
        self.host_message_cancellation.cancel();
        if !self
            .host_tasks
            .wait_for_idle(std::time::Duration::from_secs(1))
        {
            tracing::warn!("host message producers did not stop before runtime shutdown");
        }
    }
}

impl RuntimeSession {
    fn new(js: JsRuntime, window_key: wabou_shell::WindowResourceKey) -> Self {
        let pending_css = js.pending_css_handle();
        let pending_color_theme = js.pending_color_theme_handle();
        let pending_color_palette = js.pending_color_palette_handle();
        let pending_fonts = js.pending_fonts_handle();
        let frame_stats = js.frame_stats_handle();
        let pending_host_actions = Rc::new(RefCell::new(VecDeque::new()));
        let effect_bridge = EffectBridge::install(&js, window_key);
        let (host_message_handle, host_message_inbox) =
            host_message_channel(DEFAULT_HOST_MESSAGE_CAPACITY);
        Self {
            js,
            has_raf: true,
            protocol_revision: 0,
            reload: ReloadState::default(),
            pending_css: Some(pending_css),
            pending_color_theme: Some(pending_color_theme),
            pending_color_palette: Some(pending_color_palette),
            pending_fonts: Some(pending_fonts),
            frame_stats: Some(frame_stats),
            pending_host_actions,
            effect_bridge,
            wake_callback: None,
            host_message_inbox,
            host_message_handle,
            host_message_cancellation: CancellationToken::new(),
            host_tasks: Arc::new(crate::host_message::HostTaskTracker::default()),
        }
    }
}

struct DocumentState {
    node_store: NodeStore,
    style: StyleState,
    resources: ResourceState,
    runtime_transforms: HashMap<NodeId, [f32; 6]>,
    overlay_planes: HashMap<NodeId, OverlayPlane>,
    base_color: Color,
    atoms: Rc<RefCell<AtomPool>>,
    invalidation: InvalidationFlags,
    widget_manager: WidgetManager,
    applying_frame: bool,
    dirty_styles: HashSet<NodeId>,
    ifc_dirty: bool,
    #[cfg(test)]
    ifc_projection_count: usize,
    layout_viewport: Option<(u32, u32)>,
}

impl DocumentState {
    fn new(
        atoms: Rc<RefCell<AtomPool>>,
        widget_factories: HashMap<Atom, wabou_shell::WidgetFactory>,
        base_color: Color,
    ) -> Self {
        Self {
            node_store: NodeStore::new(),
            style: StyleState::default(),
            resources: ResourceState::default(),
            runtime_transforms: HashMap::new(),
            overlay_planes: HashMap::new(),
            base_color,
            atoms,
            invalidation: InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT,
            widget_manager: WidgetManager::new(widget_factories),
            applying_frame: false,
            dirty_styles: HashSet::new(),
            ifc_dirty: false,
            #[cfg(test)]
            ifc_projection_count: 0,
            layout_viewport: None,
        }
    }

    fn computed_node_snapshot(&self, solid_id: NodeKey) -> Option<ComputedNodeSnapshot> {
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
            text_max_lines: paint.text_max_lines,
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

struct InteractionState {
    input: InputRouter,
    ime_cursor_area: Option<[f64; 4]>,
    text_selection: TextSelectionState,
    scroll: ScrollState,
}

impl InteractionState {
    fn new() -> Self {
        Self {
            input: InputRouter::new(),
            ime_cursor_area: None,
            text_selection: TextSelectionState::default(),
            scroll: ScrollState::default(),
        }
    }

    fn use_pointer_modality(&mut self) {
        self.input.focus_visible = false;
    }

    fn use_keyboard_modality(&mut self) {
        self.input.focus_visible = true;
    }

    fn focus_event_payload(&self) -> String {
        serde_json::json!({ "focusVisible": self.input.focus_visible }).to_string()
    }
}

struct FrameState {
    projections: FrameProjections,
    resize_targets: ResizeTargets,
    js_tick_ema: f64,
    #[cfg(feature = "profiling")]
    profile_class_cache_hits: u64,
    #[cfg(feature = "profiling")]
    profile_class_cache_misses: u64,
    #[cfg(feature = "profiling")]
    profile_runtime_utility_fallbacks: u64,
    last_viewport: (u32, u32),
    device_scale: f64,
}

impl FrameState {
    fn new(projections: FrameProjections, resize_targets: ResizeTargets) -> Self {
        Self {
            projections,
            resize_targets,
            js_tick_ema: 0.0,
            #[cfg(feature = "profiling")]
            profile_class_cache_hits: 0,
            #[cfg(feature = "profiling")]
            profile_class_cache_misses: 0,
            #[cfg(feature = "profiling")]
            profile_runtime_utility_fallbacks: 0,
            last_viewport: (0, 0),
            device_scale: 1.0,
        }
    }
}

/// Coordinates one transactional JS protocol consumer and its retained native
/// document. Subsystems own their state; `Applier` owns frame ordering.
pub struct Applier {
    runtime: RuntimeSession,
    document: DocumentState,
    interaction: InteractionState,
    frame: FrameState,
}

impl Applier {
    /// Monotonically increasing count of non-empty JS-to-host protocol frames.
    ///
    /// Deterministic headless drivers can use this to wait for UI commits
    /// without inspecting private retained-tree state.
    pub fn protocol_revision(&self) -> u64 {
        self.runtime.protocol_revision
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
        Self::from_runtime_with_factories_and_window(
            js,
            widget_factories,
            base_color,
            wabou_shell::initial_window_resource_key(0),
        )
    }

    /// Build an applier with explicit widget factories and a typed window key.
    pub fn from_runtime_with_factories_and_window(
        js: JsRuntime,
        widget_factories: HashMap<String, wabou_shell::WidgetFactory>,
        base_color: Color,
        window_key: wabou_shell::WindowResourceKey,
    ) -> Self {
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
        Self {
            runtime: RuntimeSession::new(js, window_key),
            document: DocumentState::new(atoms, widget_factories, base_color),
            interaction: InteractionState::new(),
            frame: FrameState::new(FrameProjections::new(layout_metrics), resize_targets),
        }
    }

    /// Boot the application after all Applier-owned host bridges have been
    /// installed. This ordering permits window APIs during initial render.
    pub fn boot(&mut self, source: &str) -> rquickjs::Result<()> {
        self.runtime.js.boot(source)
    }

    pub(crate) fn boot_with_source_map(
        &mut self,
        source: &str,
        source_map: Option<&[u8]>,
    ) -> rquickjs::Result<()> {
        self.runtime.js.boot_with_source_map(source, source_map)
    }

    /// Evaluate an additional script in the booted application realm.
    pub fn eval_script(&self, source: &str) -> rquickjs::Result<()> {
        self.runtime.js.eval_script(source)
    }

    /// Evaluate a test-harness script with mapped guest exception details.
    pub fn eval_script_diagnostic(&self, source: &str) -> Result<(), String> {
        self.runtime.js.eval_script_diagnostic(source)
    }

    /// Evaluate a test-harness expression and return its string value.
    pub fn eval_string(&self, source: &str) -> rquickjs::Result<String> {
        self.runtime.js.eval_string(source)
    }

    #[cfg(feature = "vite")]
    /// Boots an application entry module through the Vite development loader.
    ///
    /// Host bridges must already be installed, just as for [`Self::boot`].
    pub fn boot_vite(&mut self, entry: &str) -> rquickjs::Result<()> {
        self.runtime.js.boot_vite(entry)
    }

    pub(crate) fn set_effect_trace(&mut self, trace: crate::effect_trace::EffectTrace) {
        self.runtime.effect_bridge.set_trace(trace);
    }

    /// Publish resolved application-private directories to native effects.
    pub fn set_app_directories(&mut self, directories: wabou_shell::AppDirectories) {
        self.runtime.effect_bridge.set_app_directories(directories);
    }

    pub(crate) fn set_asset_cache(&mut self, cache: Arc<ResourceCache>) {
        self.document.resources.set_cache(cache);
    }

    /// Attach the immutable snapshot store published through DevTools.
    #[cfg(any(feature = "devtools", test))]
    pub fn set_debug_state(&mut self, state: wabou_devtools::SharedDebugState) {
        self.runtime.js.set_debug_state(state.clone());
        self.frame.projections.debug_state = Some(state);
        self.frame.projections.debug_dirty = true;
    }

    /// Cloneable handle for background tasks / streams to push application
    /// messages toward JS (`host.subscribe` on the guest side).
    pub fn host_message_handle(&self) -> HostMessageHandle {
        self.runtime.host_message_handle.clone()
    }

    pub(crate) fn host_message_context(
        &self,
        window_key: wabou_shell::WindowResourceKey,
    ) -> crate::HostMessageContext {
        crate::HostMessageContext::new(
            window_key,
            self.host_message_handle(),
            self.runtime.host_message_cancellation.clone(),
            self.runtime.js.tokio_handle(),
            self.runtime.host_tasks.clone(),
        )
    }

    /// Snapshot the currently resolved style for a Solid node id.
    ///
    /// Call this after the relevant op frame/build tick. It performs no style
    /// recomputation and cannot mutate renderer state.
    pub fn computed_node_snapshot(&self, solid_id: NodeKey) -> Option<ComputedNodeSnapshot> {
        self.document.computed_node_snapshot(solid_id)
    }
}

impl Applier {
    fn cancel_pointer_gesture(&mut self, pointer: wabou_shell::PointerEvent) -> bool {
        self.interaction.input.pointer_position = (pointer.position.x, pointer.position.y);
        self.interaction.input.pointer_buttons = pointer.buttons;
        self.interaction.text_selection.next_scroll = None;
        if self.interaction.input.pointer_down_target.is_some() {
            self.interaction.text_selection.last_click = None;
        }
        let old_active = self.interaction.input.pointer_down_target.take();
        self.interaction.input.pointer_down_position = None;
        self.interaction.input.pointer_dragged = false;
        let target = old_active.or_else(|| {
            self.interaction
                .input
                .hit_test(pointer.position.x, pointer.position.y)
        });
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
        if self.interaction.input.pointer_down_target.is_none() {
            self.interaction.input.pointer_buttons = 0;
            return false;
        }
        self.cancel_pointer_gesture(wabou_shell::PointerEvent {
            phase: PointerPhase::Cancel,
            position: wabou_shell::Point {
                x: self.interaction.input.pointer_position.0,
                y: self.interaction.input.pointer_position.1,
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
