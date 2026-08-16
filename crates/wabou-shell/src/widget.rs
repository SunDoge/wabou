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

use std::sync::Arc;
use std::time::Instant;

use crate::SemanticRole;
use crate::style::{Paint, TextAlign};
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
    pub content_size: [f32; 2],
    pub device_scale: f64,
    pub local_to_window: [f64; 6],
    pub window_to_local: [f64; 6],
}

/// Space offered by the layout engine for one measured axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidgetAvailableSpace {
    Definite(f32),
    MinContent,
    MaxContent,
}

/// Measurement state supplied from Taffy's active layout pass.
pub struct MeasureContext<'a> {
    known_size: [Option<f32>; 2],
    available_space: [WidgetAvailableSpace; 2],
    device_scale: f64,
    text: &'a mut TextContext,
}

impl<'a> MeasureContext<'a> {
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

    pub fn known_size(&self) -> [Option<f32>; 2] {
        self.known_size
    }

    pub fn available_space(&self) -> [WidgetAvailableSpace; 2] {
        self.available_space
    }

    pub fn device_scale(&self) -> f64 {
        self.device_scale
    }

    pub fn text(&mut self) -> &mut TextContext {
        self.text
    }

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
    scene: Scene,
    owns_clip: bool,
}

impl<'a> PaintContext<'a> {
    pub fn new(width: f32, height: f32, device_scale: f64, text: &'a mut TextContext) -> Self {
        Self {
            width,
            height,
            device_scale: device_scale.max(f64::EPSILON),
            text,
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

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn device_scale(&self) -> f64 {
        self.device_scale
    }

    pub fn size(&self) -> [f32; 2] {
        [self.width, self.height]
    }

    pub fn text(&mut self) -> &mut TextContext {
        self.text
    }

    /// Direct access to the Vello scene while the painting API is evolving.
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

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
    pub background: Option<Color>,
    pub color: Color,
    pub font_size: f32,
    pub font_weight: f32,
    pub line_height: Option<(f32, bool)>,
    pub text_align: TextAlign,
    pub font_family: Option<Arc<str>>,
}

/// Optional semantic information contributed by a native widget.
/// Explicit JS/ARIA attributes remain authoritative when both are present.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WidgetAccessibility {
    pub role: Option<SemanticRole>,
    pub label: Option<String>,
    pub value: Option<String>,
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
    pub event_code: u8,
    pub json: String,
}

impl WidgetNodeEvent {
    pub fn json(event_code: u8, json: impl Into<String>) -> Self {
        Self {
            event_code,
            json: json.into(),
        }
    }
}

pub type WidgetFactory = Arc<dyn Fn() -> Box<dyn Widget>>;

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub struct WidgetChanges: u8 {
        const HANDLED = 1 << 0;
        const VALUE = 1 << 1;
        const REDRAW = 1 << 2;
        const CONSUME_KEY_TEXT = 1 << 3;
        const MEASURE = 1 << 4;
        const LAYOUT = 1 << 5;
        const SEMANTICS = 1 << 6;
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WidgetEventResult {
    changes: WidgetChanges,
    clipboard: Option<ClipboardRequest>,
}

impl WidgetEventResult {
    pub const IGNORED: Self = Self {
        changes: WidgetChanges::empty(),
        clipboard: None,
    };

    pub const HANDLED: Self = Self {
        changes: WidgetChanges::HANDLED.union(WidgetChanges::REDRAW),
        clipboard: None,
    };

    pub const VALUE_CHANGED: Self = Self {
        changes: WidgetChanges::HANDLED
            .union(WidgetChanges::VALUE)
            .union(WidgetChanges::REDRAW),
        clipboard: None,
    };

    pub const fn is_handled(&self) -> bool {
        self.changes.contains(WidgetChanges::HANDLED)
    }

    pub const fn value_changed(&self) -> bool {
        self.changes.contains(WidgetChanges::VALUE)
    }

    pub const fn requests_redraw(&self) -> bool {
        self.changes.contains(WidgetChanges::REDRAW)
    }

    pub const fn consumes_key_text(&self) -> bool {
        self.changes.contains(WidgetChanges::CONSUME_KEY_TEXT)
    }

    pub const fn handled_consuming_key_text() -> Self {
        Self {
            changes: WidgetChanges::HANDLED
                .union(WidgetChanges::REDRAW)
                .union(WidgetChanges::CONSUME_KEY_TEXT),
            clipboard: None,
        }
    }

    pub const fn value_changed_consuming_key_text() -> Self {
        Self {
            changes: WidgetChanges::HANDLED
                .union(WidgetChanges::VALUE)
                .union(WidgetChanges::REDRAW)
                .union(WidgetChanges::CONSUME_KEY_TEXT),
            clipboard: None,
        }
    }

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

    pub fn copy(text: String) -> Self {
        Self {
            changes: WidgetChanges::HANDLED,
            clipboard: Some(ClipboardRequest::Write(text)),
        }
    }

    pub fn copy_with_value_change(text: String) -> Self {
        Self {
            changes: WidgetChanges::HANDLED
                .union(WidgetChanges::VALUE)
                .union(WidgetChanges::REDRAW),
            clipboard: Some(ClipboardRequest::Write(text)),
        }
    }

    pub fn paste() -> Self {
        Self {
            changes: WidgetChanges::HANDLED,
            clipboard: Some(ClipboardRequest::Read),
        }
    }

    pub fn clipboard_request(&self) -> Option<&ClipboardRequest> {
        self.clipboard.as_ref()
    }
}

/// A Rust-side widget that paints custom content through a [`PaintContext`].
///
/// Ported from blitz's `Widget` trait
/// (`packages/blitz-dom/src/node/custom_widget.rs`), adapted to wabou's
/// protocol-based model.
/// Deserialize a `widgetConfig` payload into a widget-specific derived type.
pub fn decode_widget_config<T: serde::de::DeserializeOwned>(json: &str) -> Result<T, String> {
    serde_json::from_str(json).map_err(|error| error.to_string())
}

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

    /// Return the widget's current value (for syncing to JS). Called by the
    /// applier after `handle_event` returns `true` on a text-edit event.
    /// The applier dispatches an `INPUT` event with `{"value": ...}` to JS.
    fn current_value(&self) -> Option<&str> {
        None
    }

    /// Native semantics that cannot be inferred from the host element.
    fn accessibility(&self) -> WidgetAccessibility {
        WidgetAccessibility::default()
    }

    /// Whether this widget wants focus on pointer down. Default: false.
    fn accepts_focus(&self) -> bool {
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

    fn complete_host_action(&mut self, _result: HostActionResult) {}
}

/// Small deterministic driver for unit-testing a widget without a window.
pub struct WidgetHarness<W> {
    widget: W,
    geometry: WidgetGeometry,
    text: TextContext,
}

impl<W: Widget> WidgetHarness<W> {
    pub fn new(widget: W) -> Self {
        Self {
            widget,
            geometry: WidgetGeometry::default(),
            text: TextContext::new(),
        }
    }

    pub fn widget(&self) -> &W {
        &self.widget
    }

    pub fn widget_mut(&mut self) -> &mut W {
        &mut self.widget
    }

    pub fn mount(&mut self) -> WidgetChanges {
        self.widget.mounted()
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) -> WidgetChanges {
        self.widget.attribute_changed(name, value)
    }

    pub fn remove_attribute(&mut self, name: &str) -> WidgetChanges {
        self.widget.attribute_removed(name)
    }

    pub fn set_visible(&mut self, visible: bool) -> WidgetChanges {
        self.widget.visibility_changed(visible)
    }

    pub fn layout(&mut self, geometry: WidgetGeometry) {
        self.geometry = geometry;
        self.widget.layout_changed(geometry);
    }

    pub fn measure(
        &mut self,
        known_size: [Option<f32>; 2],
        available_space: [WidgetAvailableSpace; 2],
        device_scale: f64,
    ) -> Option<[f32; 2]> {
        let mut cx = MeasureContext::new(known_size, available_space, device_scale, &mut self.text);
        self.widget.measure(&mut cx)
    }

    pub fn event(&mut self, event: &UiEvent) -> WidgetEventResult {
        self.widget.handle_event(event)
    }

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
