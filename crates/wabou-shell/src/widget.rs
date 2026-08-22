//! Rust-side widget trait — the Qt-Quick / Slint pattern.
//!
//! Performance-sensitive widgets (TextInput, Canvas, ListView, ScrollView, …)
//! are implemented in Rust. They paint a vello `Scene` fragment every frame;
//! `build_scene` composites it at the node's border-box origin. QuickJS +
//! SolidJS compose these widgets into apps via the binary protocol (tree
//! structure + property values + event handlers), while the Rust widget does
//! the actual rendering + interaction.
//!
//! The node's standard bg+border+clip (from classes/inline styles) still
//! render; the widget paints its **content** on top, inside the content box.

#![warn(missing_docs)]

use std::sync::Arc;
use std::time::Instant;

use crate::SemanticRole;
use crate::style::{Paint, TextAlign};
use crate::text::SingleLineTextMetrics;
use crate::text::TextContext;
use crate::{ClipboardRequest, HostAction, HostActionResult, UiEvent, WakeCallback};
use vello::{
    Scene,
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};

/// Authoritative geometry for one native widget after layout.
///
/// Pointer and wheel positions delivered to [`Widget::handle_event`] are
/// already expressed in this content-local coordinate space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidgetGeometry {
    /// Width and height of the CSS content box in logical pixels.
    pub content_size: [f32; 2],
    /// Physical pixels per logical pixel for the widget's window.
    pub device_scale: f64,
    /// Affine transform from content-local coordinates to window coordinates.
    pub local_to_window: [f64; 6],
    /// Inverse of [`Self::local_to_window`], used for event localization.
    pub window_to_local: [f64; 6],
}

/// Space offered by the layout engine for one measured axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidgetAvailableSpace {
    /// The containing layout constrains this axis to the supplied size.
    Definite(f32),
    /// Measure the smallest size that avoids optional overflow.
    MinContent,
    /// Measure the preferred size without optional wrapping.
    MaxContent,
}

/// Measurement state supplied from Taffy's active layout pass.
///
/// This context is valid only during [`Widget::measure`]. The text context
/// must not be retained by the widget.
pub struct MeasureContext<'a> {
    known_size: [Option<f32>; 2],
    available_space: [WidgetAvailableSpace; 2],
    device_scale: f64,
    text: &'a mut TextContext,
}

impl<'a> MeasureContext<'a> {
    /// Construct a context for one Taffy measurement callback.
    pub fn new(
        known_size: [Option<f32>; 2],
        available_space: [WidgetAvailableSpace; 2],
        device_scale: f64,
        text: &'a mut TextContext,
    ) -> Self {
        Self {
            known_size,
            available_space,
            device_scale: device_scale.max(f64::EPSILON),
            text,
        }
    }

    /// Dimensions already resolved by CSS, or `None` for intrinsic axes.
    pub fn known_size(&self) -> [Option<f32>; 2] {
        self.known_size
    }

    /// Constraints offered by the containing layout for width and height.
    pub fn available_space(&self) -> [WidgetAvailableSpace; 2] {
        self.available_space
    }

    /// Return the normalized physical-pixels-per-logical-pixel scale.
    pub fn device_scale(&self) -> f64 {
        self.device_scale
    }

    /// Borrow Wabou's shared font and shaping resources.
    pub fn text(&mut self) -> &mut TextContext {
        self.text
    }

    /// Keep CSS-resolved axes and fill intrinsic axes from `measured`.
    pub fn resolve_size(&self, measured: [f32; 2]) -> [f32; 2] {
        [
            self.known_size[0].unwrap_or(measured[0]),
            self.known_size[1].unwrap_or(measured[1]),
        ]
    }
}

impl Default for WidgetGeometry {
    fn default() -> Self {
        Self {
            content_size: [0.0, 0.0],
            device_scale: 1.0,
            local_to_window: Affine::IDENTITY.as_coeffs(),
            window_to_local: Affine::IDENTITY.as_coeffs(),
        }
    }
}

/// Per-frame painting state passed to a native widget.
///
/// This is intentionally a thin boundary around Vello rather than a second
/// drawing API. Widgets get the geometry and text resources they need while
/// direct scene access remains available for operations Wabou has not earned
/// an abstraction for yet.
pub struct PaintContext<'a> {
    width: f32,
    height: f32,
    device_scale: f64,
    text: &'a mut TextContext,
    local_to_window: Affine,
    scene: Scene,
    owns_clip: bool,
}

impl<'a> PaintContext<'a> {
    /// Create an unclipped content-local scene fragment.
    pub fn new(width: f32, height: f32, device_scale: f64, text: &'a mut TextContext) -> Self {
        Self {
            width,
            height,
            device_scale: device_scale.max(f64::EPSILON),
            text,
            local_to_window: Affine::IDENTITY,
            scene: Scene::new(),
            owns_clip: false,
        }
    }

    /// Create a context whose complete widget fragment is clipped in local
    /// coordinates. The clip is encoded before widget-owned drawing and scene
    /// appends, avoiding backend-dependent clipping at the parent append site.
    pub fn new_clipped(
        width: f32,
        height: f32,
        radius: f64,
        device_scale: f64,
        text: &'a mut TextContext,
    ) -> Self {
        let mut context = Self::new(width, height, device_scale, text);
        context.scene.push_clip_layer(
            Fill::NonZero,
            Affine::IDENTITY,
            &Rect::new(0.0, 0.0, f64::from(width), f64::from(height))
                .to_rounded_rect(radius.max(0.0)),
        );
        context.owns_clip = true;
        context
    }

    /// Create a clipped widget context with its logical window transform.
    pub fn new_clipped_at(
        width: f32,
        height: f32,
        radius: f64,
        device_scale: f64,
        local_to_window: [f64; 6],
        text: &'a mut TextContext,
    ) -> Self {
        let mut context = Self::new_clipped(width, height, radius, device_scale, text);
        context.local_to_window = Affine::new(local_to_window);
        context
    }

    /// Content-box width in logical pixels.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Content-box height in logical pixels.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Physical pixels per logical pixel for the target window.
    pub fn device_scale(&self) -> f64 {
        self.device_scale
    }

    /// Content-box `[width, height]` in logical pixels.
    pub fn size(&self) -> [f32; 2] {
        [self.width, self.height]
    }

    /// Borrow Wabou's shared font and shaping resources for this frame.
    pub fn text(&mut self) -> &mut TextContext {
        self.text
    }

    /// Direct access to the Vello scene while the painting API is evolving.
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    /// Paint shaped text at a content-local origin with final pixel alignment.
    pub fn draw_text_layout(
        &mut self,
        layout: &std::sync::Arc<parley::Layout<[u8; 4]>>,
        origin: [f64; 2],
    ) {
        let destination_to_output = Affine::scale(self.device_scale) * self.local_to_window;
        let text_to_output = destination_to_output * Affine::translate((origin[0], origin[1]));
        crate::scene::draw_text_layout_into(
            &mut self.scene,
            self.text,
            layout,
            destination_to_output,
            text_to_output,
            self.device_scale,
        );
    }

    /// Finish the fragment, balancing a context-owned clip if present.
    pub fn finish(mut self) -> Scene {
        if self.owns_clip {
            self.scene.pop_layer();
        }
        self.scene
    }
}

/// Resolved content styles exposed to native widgets.
///
/// Box-model and compositing properties remain framework-owned. Widgets paint
/// only inside their local content box and consume this smaller style surface
/// for content whose metrics or colors depend on CSS.
#[derive(Clone, Debug, PartialEq)]
pub struct WidgetStyle {
    /// Resolved background color, when one is present.
    pub background: Option<Color>,
    /// Resolved foreground/text color.
    pub color: Color,
    /// Font size in logical pixels.
    pub font_size: f32,
    /// Numeric CSS font weight.
    pub font_weight: f32,
    /// Resolved line height and whether it was explicitly specified.
    pub line_height: Option<(f32, bool)>,
    /// Horizontal text alignment.
    pub text_align: TextAlign,
    /// Preferred family, or the platform default when absent.
    pub font_family: Option<Arc<str>>,
}

/// Optional semantic information contributed by a native widget.
/// Explicit JS/ARIA attributes remain authoritative when both are present.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WidgetAccessibility {
    /// Native role used when the host element does not declare one.
    pub role: Option<SemanticRole>,
    /// Accessible name used when the host element does not declare one.
    pub label: Option<String>,
    /// Current textual value exposed to assistive technology.
    pub value: Option<String>,
    /// Whether every textual value associated with this widget is sensitive.
    ///
    /// The runtime suppresses both the widget value and any value authored by
    /// JavaScript when this is set. This lets custom secret controls opt into
    /// the same protection without relying on a reserved element tag.
    pub value_is_sensitive: bool,
    /// Disabled state when it cannot be inferred from the host element.
    pub disabled: Option<bool>,
}

impl From<&Paint> for WidgetStyle {
    fn from(paint: &Paint) -> Self {
        Self {
            background: paint.background,
            color: paint.text_color,
            font_size: paint.font_size,
            font_weight: paint.font_weight,
            line_height: paint.line_height,
            text_align: paint.text_align,
            font_family: paint.font_family.clone(),
        }
    }
}

/// An asynchronous event emitted by a Rust widget toward its Solid element.
///
/// Event codes come from the shared bridge protocol; `json` is merged into
/// the small event object received by JSX listeners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetNodeEvent {
    /// Event discriminator from the shared generated bridge protocol.
    pub event_code: u8,
    /// Serialized JSON object merged into the JS listener event.
    pub json: String,
}

impl WidgetNodeEvent {
    /// Construct an event with an already serialized JSON-object payload.
    pub fn json(event_code: u8, json: impl Into<String>) -> Self {
        Self {
            event_code,
            json: json.into(),
        }
    }
}

/// Factory stored by the host to construct one widget per matching node.
pub type WidgetFactory = Arc<dyn Fn() -> Box<dyn Widget>>;

bitflags::bitflags! {
    /// Framework work invalidated by a widget callback.
    ///
    /// Precise flags avoid performing layout when a change only requires
    /// repainting or accessibility publication.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct WidgetChanges: u8 {
        /// The widget consumed the input event.
        const HANDLED = 1 << 0;
        /// [`Widget::current_value`] changed and must be synchronized to JS.
        const VALUE = 1 << 1;
        /// The widget scene fragment must be painted again.
        const REDRAW = 1 << 2;
        /// Suppress the text event normally emitted after this key event.
        const CONSUME_KEY_TEXT = 1 << 3;
        /// Intrinsic measurement may have changed.
        const MEASURE = 1 << 4;
        /// Layout geometry must be recomputed.
        const LAYOUT = 1 << 5;
        /// Accessibility semantics must be republished.
        const SEMANTICS = 1 << 6;
    }
}

/// Result of dispatching one [`UiEvent`] to a native widget.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WidgetEventResult {
    changes: WidgetChanges,
    clipboard: Option<ClipboardRequest>,
}

impl WidgetEventResult {
    /// The widget ignored the event and requests no follow-up work.
    pub const IGNORED: Self = Self {
        changes: WidgetChanges::empty(),
        clipboard: None,
    };

    /// The widget consumed the event and needs repainting.
    pub const HANDLED: Self = Self {
        changes: WidgetChanges::HANDLED.union(WidgetChanges::REDRAW),
        clipboard: None,
    };

    /// The widget consumed the event, changed value, and needs repainting.
    pub const VALUE_CHANGED: Self = Self {
        changes: WidgetChanges::HANDLED
            .union(WidgetChanges::VALUE)
            .union(WidgetChanges::REDRAW),
        clipboard: None,
    };

    /// Whether event propagation should stop at this widget.
    pub const fn is_handled(&self) -> bool {
        self.changes.contains(WidgetChanges::HANDLED)
    }

    /// Whether the host must synchronize [`Widget::current_value`] to JS.
    pub const fn value_changed(&self) -> bool {
        self.changes.contains(WidgetChanges::VALUE)
    }

    /// Whether the widget scene fragment needs repainting.
    pub const fn requests_redraw(&self) -> bool {
        self.changes.contains(WidgetChanges::REDRAW)
    }

    /// Whether the host should suppress the key's following text event.
    pub const fn consumes_key_text(&self) -> bool {
        self.changes.contains(WidgetChanges::CONSUME_KEY_TEXT)
    }

    /// Consume a key event and suppress its following text event.
    pub const fn handled_consuming_key_text() -> Self {
        Self {
            changes: WidgetChanges::HANDLED
                .union(WidgetChanges::REDRAW)
                .union(WidgetChanges::CONSUME_KEY_TEXT),
            clipboard: None,
        }
    }

    /// Report a value change while suppressing the following text event.
    pub const fn value_changed_consuming_key_text() -> Self {
        Self {
            changes: WidgetChanges::HANDLED
                .union(WidgetChanges::VALUE)
                .union(WidgetChanges::REDRAW)
                .union(WidgetChanges::CONSUME_KEY_TEXT),
            clipboard: None,
        }
    }

    /// Return the complete set of requested invalidations.
    pub const fn changes(&self) -> WidgetChanges {
        self.changes
    }

    /// Mark an event handled while requesting additional framework work.
    /// `HANDLED` is added automatically, so callers only need to specify
    /// effects such as `REDRAW`, `MEASURE`, or `LAYOUT`.
    pub const fn handled_with(changes: WidgetChanges) -> Self {
        Self {
            changes: changes.union(WidgetChanges::HANDLED),
            clipboard: None,
        }
    }

    /// Request that the host write `text` to the system clipboard.
    pub fn copy(text: String) -> Self {
        Self {
            changes: WidgetChanges::HANDLED,
            clipboard: Some(ClipboardRequest::Write(text)),
        }
    }

    /// Write `text` to the clipboard and synchronize a changed value.
    pub fn copy_with_value_change(text: String) -> Self {
        Self {
            changes: WidgetChanges::HANDLED
                .union(WidgetChanges::VALUE)
                .union(WidgetChanges::REDRAW),
            clipboard: Some(ClipboardRequest::Write(text)),
        }
    }

    /// Request clipboard text; the host returns it as a later paste event.
    pub fn paste() -> Self {
        Self {
            changes: WidgetChanges::HANDLED,
            clipboard: Some(ClipboardRequest::Read),
        }
    }

    /// Return the clipboard operation attached to this result, if any.
    pub fn clipboard_request(&self) -> Option<&ClipboardRequest> {
        self.clipboard.as_ref()
    }
}

/// Deserialize a `widgetConfig` payload into a widget-specific derived type.
///
/// Prefer a concrete type with `#[serde(deny_unknown_fields)]`: configuration
/// errors then stay at the JS/Rust boundary instead of becoming latent bugs.
pub fn decode_widget_config<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|error| error.to_string())
}

/// A Rust-side widget that measures, paints, and handles native interaction.
///
/// The host owns box layout, transforms, clipping, focus routing, and scene
/// composition. Implementations own content-local rendering and state. The
/// returned [`WidgetChanges`] flags form the invalidation contract between the
/// widget and those host-owned projections.
pub trait Widget {
    /// Measure content before layout. The default supports widgets whose
    /// intrinsic size is already known; text-backed widgets can update font
    /// metrics with `tcx` and then return their measured size.
    fn measure(&mut self, cx: &mut MeasureContext<'_>) -> Option<[f32; 2]> {
        self.intrinsic_size().map(|size| cx.resolve_size(size))
    }

    /// Paint inside the node's local content box. The framework composites
    /// the context's scene fragment at the content-box origin.
    fn paint(&mut self, cx: &mut PaintContext<'_>);

    /// Handle an event targeted at this widget's node.
    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::IGNORED
    }

    /// Layout geometry changed. This is delivered atomically after layout and
    /// before painting or dispatching subsequent input to the widget.
    fn layout_changed(&mut self, _geometry: WidgetGeometry) {}

    /// The widget has been attached to a live host node.
    fn mounted(&mut self) -> WidgetChanges {
        WidgetChanges::empty()
    }

    /// Whether the widget currently participates in visible layout.
    fn visibility_changed(&mut self, _visible: bool) -> WidgetChanges {
        WidgetChanges::empty()
    }

    /// An attribute was set or changed. Called after `CreateElement` for each
    /// initial attribute, and on subsequent `SetAttribute` ops. This is how JS
    /// passes parameters to the widget — e.g. `<chart data={...} color="red" />`
    /// triggers `attribute_changed("data", ...)` + `attribute_changed("color", "red")`.
    fn attribute_changed(&mut self, _name: &str, _value: &str) -> WidgetChanges {
        WidgetChanges::empty()
    }

    /// An attribute was removed (via `RemoveAttribute`). Default: ignore.
    fn attribute_removed(&mut self, _name: &str) -> WidgetChanges {
        WidgetChanges::empty()
    }

    /// The complete `widgetConfig` object changed. Implementations should
    /// deserialize it into a widget-specific type, normally with
    /// [`decode_widget_config`].
    fn config_changed(&mut self, _json: &str) -> Result<WidgetChanges, String> {
        Ok(WidgetChanges::empty())
    }

    /// `widgetConfig` was removed. Default: ignore.
    fn config_removed(&mut self) -> WidgetChanges {
        WidgetChanges::empty()
    }

    /// Release host-owned side effects before the node is removed.
    ///
    /// The applier drains host actions produced here while node routing still
    /// exists. Resource-only cleanup can continue to use `Drop`.
    fn unmount(&mut self) {}

    /// Resolved CSS content styles changed for the host element.
    ///
    /// Called before measurement and only when the resolved style differs
    /// from the value delivered previously.
    fn style_changed(&mut self, _style: &WidgetStyle) -> WidgetChanges {
        WidgetChanges::empty()
    }

    /// Return the widget's current value for synchronization to JS.
    ///
    /// The applier reads this after a callback returns [`WidgetChanges::VALUE`]
    /// and dispatches an input event with `{"value": ...}`. Returning `None`
    /// opts out even when the value flag was set.
    fn current_value(&self) -> Option<&str> {
        None
    }

    /// Native semantics that cannot be inferred from the host element.
    fn accessibility(&self) -> WidgetAccessibility {
        WidgetAccessibility::default()
    }

    /// Most recently painted single-line text geometry for diagnostics.
    ///
    /// Text-backed widgets should return metrics produced by
    /// [`crate::text::single_line_text_metrics`]. This lets headless tests
    /// compare native and ordinary text without inspecting pixels.
    fn text_metrics(&self) -> Option<SingleLineTextMetrics> {
        None
    }

    /// Whether this widget wants focus on pointer down. Default: false.
    fn accepts_focus(&self) -> bool {
        false
    }

    /// Whether focusing this widget should enable platform text/IME input.
    /// Focusability alone does not imply text entry (for example, a native
    /// canvas control may accept keyboard focus without accepting text).
    fn accepts_text_input(&self) -> bool {
        false
    }

    /// Intrinsic content size used when CSS leaves an axis automatic.
    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        None
    }

    /// Notify the widget when its element focus changes.
    fn focus_changed(&mut self, _focused: bool) -> WidgetChanges {
        WidgetChanges::empty()
    }

    /// Next time at which this widget needs a repaint for an animation.
    fn animation_deadline(&self) -> Option<Instant> {
        None
    }

    /// Local content-box area which the platform IME candidate window should
    /// avoid. Text editors should include the caret or active preedit run.
    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        None
    }

    /// Install the event-loop wake callback for a background producer.
    fn set_wake_callback(&mut self, _wake: WakeCallback) {}

    /// Drain background completions after the host event loop was woken.
    fn poll_async(&mut self) -> bool {
        false
    }

    /// Drain one asynchronous request for the native window host.
    fn take_host_action(&mut self) -> Option<HostAction> {
        None
    }

    /// Drain one event produced asynchronously by this widget for its Solid
    /// element. Node routing is preserved with multiple widget instances.
    fn take_node_event(&mut self) -> Option<WidgetNodeEvent> {
        None
    }

    /// Complete the oldest host action routed back to this widget.
    fn complete_host_action(&mut self, _result: HostActionResult) {}
}

/// Small deterministic driver for unit-testing a widget without a window.
pub struct WidgetHarness<W> {
    widget: W,
    geometry: WidgetGeometry,
    text: TextContext,
}

impl<W: Widget> WidgetHarness<W> {
    /// Wrap a widget with deterministic geometry and text resources.
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            geometry: WidgetGeometry::default(),
            text: TextContext::new(),
        }
    }

    /// Borrow the widget under test.
    pub fn widget(&self) -> &W {
        &self.widget
    }

    /// Mutably borrow the widget for setup or state assertions.
    pub fn widget_mut(&mut self) -> &mut W {
        &mut self.widget
    }

    /// Deliver the mount lifecycle callback.
    pub fn mount(&mut self) -> WidgetChanges {
        self.widget.mounted()
    }

    /// Set an attribute directly on the widget.
    pub fn set_attribute(&mut self, name: &str, value: &str) -> WidgetChanges {
        self.widget.attribute_changed(name, value)
    }

    /// Remove an attribute directly from the widget.
    pub fn remove_attribute(&mut self, name: &str) -> WidgetChanges {
        self.widget.attribute_removed(name)
    }

    /// Deliver a visibility transition.
    pub fn set_visible(&mut self, visible: bool) -> WidgetChanges {
        self.widget.visibility_changed(visible)
    }

    /// Store and deliver new content-local geometry.
    pub fn layout(&mut self, geometry: WidgetGeometry) {
        self.geometry = geometry;
        self.widget.layout_changed(geometry);
    }

    /// Run intrinsic measurement with explicit Taffy-compatible constraints.
    pub fn measure(
        &mut self,
        known_size: [Option<f32>; 2],
        available_space: [WidgetAvailableSpace; 2],
        device_scale: f64,
    ) -> Option<[f32; 2]> {
        let mut cx = MeasureContext::new(known_size, available_space, device_scale, &mut self.text);
        self.widget.measure(&mut cx)
    }

    /// Dispatch one already-localized UI event.
    pub fn event(&mut self, event: &UiEvent) -> WidgetEventResult {
        self.widget.handle_event(event)
    }

    /// Paint using the most recently supplied geometry.
    pub fn paint(&mut self) -> Scene {
        let [width, height] = self.geometry.content_size;
        let mut cx = PaintContext::new(width, height, self.geometry.device_scale, &mut self.text);
        self.widget.paint(&mut cx);
        cx.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, PartialEq, serde::Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct TestConfig {
        read_only: bool,
        tab_width: u8,
    }

    #[test]
    fn widget_config_deserializes_into_a_derived_type() {
        let config: TestConfig = decode_widget_config(r#"{"readOnly":true,"tabWidth":4}"#).unwrap();
        assert_eq!(
            config,
            TestConfig {
                read_only: true,
                tab_width: 4,
            }
        );
        assert!(decode_widget_config::<TestConfig>(r#"{"unknown":true}"#).is_err());
    }

    #[test]
    fn paint_context_carries_frame_geometry_and_normalizes_scale() {
        let mut text = TextContext::new();
        let paint = PaintContext::new(320.0, 180.0, 0.0, &mut text);

        assert_eq!(paint.size(), [320.0, 180.0]);
        assert_eq!(paint.width(), 320.0);
        assert_eq!(paint.height(), 180.0);
        assert_eq!(paint.device_scale(), f64::EPSILON);
    }

    #[test]
    fn widget_changes_are_composable_flags() {
        assert_eq!(WidgetEventResult::IGNORED.changes(), WidgetChanges::empty());
        assert!(
            WidgetEventResult::HANDLED
                .changes()
                .contains(WidgetChanges::HANDLED | WidgetChanges::REDRAW)
        );
        assert!(WidgetEventResult::VALUE_CHANGED.value_changed());
        let layout =
            WidgetEventResult::handled_with(WidgetChanges::MEASURE | WidgetChanges::LAYOUT);
        assert!(layout.is_handled());
        assert!(
            layout
                .changes()
                .contains(WidgetChanges::MEASURE | WidgetChanges::LAYOUT)
        );
        assert!(WidgetEventResult::VALUE_CHANGED.requests_redraw());
        assert!(WidgetEventResult::handled_consuming_key_text().consumes_key_text());
        assert!(WidgetEventResult::value_changed_consuming_key_text().value_changed());
        assert!(WidgetEventResult::value_changed_consuming_key_text().consumes_key_text());
        assert!(!WidgetEventResult::HANDLED.consumes_key_text());
    }

    struct RecordingWidget(Rc<RefCell<Vec<&'static str>>>);

    impl Widget for RecordingWidget {
        fn measure(&mut self, cx: &mut MeasureContext<'_>) -> Option<[f32; 2]> {
            self.0.borrow_mut().push("measure");
            assert_eq!(cx.known_size(), [Some(80.0), None]);
            assert_eq!(
                cx.available_space(),
                [
                    WidgetAvailableSpace::Definite(120.0),
                    WidgetAvailableSpace::MaxContent,
                ]
            );
            Some(cx.resolve_size([40.0, 20.0]))
        }

        fn paint(&mut self, _cx: &mut PaintContext<'_>) {
            self.0.borrow_mut().push("paint");
        }

        fn mounted(&mut self) -> WidgetChanges {
            self.0.borrow_mut().push("mount");
            WidgetChanges::REDRAW
        }

        fn visibility_changed(&mut self, visible: bool) -> WidgetChanges {
            self.0
                .borrow_mut()
                .push(if visible { "visible" } else { "hidden" });
            WidgetChanges::REDRAW
        }

        fn layout_changed(&mut self, _geometry: WidgetGeometry) {
            self.0.borrow_mut().push("layout");
        }
    }

    #[test]
    fn widget_harness_drives_measurement_and_lifecycle_in_order() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let mut harness = WidgetHarness::new(RecordingWidget(calls.clone()));
        assert!(harness.mount().contains(WidgetChanges::REDRAW));
        assert_eq!(
            harness.measure(
                [Some(80.0), None],
                [
                    WidgetAvailableSpace::Definite(120.0),
                    WidgetAvailableSpace::MaxContent,
                ],
                2.0,
            ),
            Some([80.0, 20.0])
        );
        harness.layout(WidgetGeometry {
            content_size: [80.0, 20.0],
            device_scale: 2.0,
            ..WidgetGeometry::default()
        });
        harness.set_visible(true);
        let _ = harness.paint();
        assert_eq!(
            calls.borrow().as_slice(),
            ["mount", "measure", "layout", "visible", "paint"]
        );
    }
}
