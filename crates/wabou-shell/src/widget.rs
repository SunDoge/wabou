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

use crate::style::{Paint, TextAlign};
use crate::text::TextContext;
use crate::{ClipboardRequest, HostAction, HostActionResult, UiEvent, WakeCallback};
use vello::Scene;
use vello::peniko::Color;

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
}

impl<'a> PaintContext<'a> {
    pub fn new(width: f32, height: f32, device_scale: f64, text: &'a mut TextContext) -> Self {
        Self {
            width,
            height,
            device_scale: device_scale.max(f64::EPSILON),
            text,
            scene: Scene::new(),
        }
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

    pub fn finish(self) -> Scene {
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
pub trait Widget {
    /// Measure content before layout. The default supports widgets whose
    /// intrinsic size is already known; text-backed widgets can update font
    /// metrics with `tcx` and then return their measured size.
    fn measure(&mut self, _tcx: &mut TextContext) -> Option<[f32; 2]> {
        self.intrinsic_size()
    }

    /// Paint inside the node's local content box. The framework composites
    /// the context's scene fragment at the content-box origin.
    fn paint(&mut self, cx: &mut PaintContext<'_>);

    /// Handle an event targeted at this widget's node.
    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::IGNORED
    }

    /// An attribute was set or changed. Called after `CreateElement` for each
    /// initial attribute, and on subsequent `SetAttribute` ops. This is how JS
    /// passes parameters to the widget — e.g. `<chart data={...} color="red" />`
    /// triggers `attribute_changed("data", ...)` + `attribute_changed("color", "red")`.
    fn attribute_changed(&mut self, _name: &str, _value: &str) {}

    /// An attribute was removed (via `RemoveAttribute`). Default: ignore.
    fn attribute_removed(&mut self, _name: &str) {}

    /// Release host-owned side effects before the node is removed.
    ///
    /// The applier drains host actions produced here while node routing still
    /// exists. Resource-only cleanup can continue to use `Drop`.
    fn unmount(&mut self) {}

    /// Resolved CSS content styles changed for the host element.
    ///
    /// Called before measurement and only when the resolved style differs
    /// from the value delivered previously.
    fn style_changed(&mut self, _style: &WidgetStyle) {}

    /// Return the widget's current value (for syncing to JS). Called by the
    /// applier after `handle_event` returns `true` on a text-edit event.
    /// The applier dispatches an `INPUT` event with `{"value": ...}` to JS.
    fn current_value(&self) -> Option<&str> {
        None
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
    fn focus_changed(&mut self, _focused: bool) {}

    /// Next time at which this widget needs a repaint for an animation.
    fn animation_deadline(&self) -> Option<Instant> {
        None
    }

    /// Local content-box area which the platform IME candidate window should
    /// avoid. Text editors should include the caret or active preedit run.
    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        None
    }

    /// Absolute screen-space position of the widget's border-box origin.
    /// Called by the applier during `paint_widgets` (from the PlacedNode rect)
    /// so the widget can convert absolute pointer coordinates to local.
    fn set_position(&mut self, _x: f32, _y: f32) {}

    /// Window-logical coordinates to the widget content-box coordinate space.
    fn set_window_to_local(&mut self, _transform: [f64; 6]) {}

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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(WidgetEventResult::VALUE_CHANGED.requests_redraw());
        assert!(WidgetEventResult::handled_consuming_key_text().consumes_key_text());
        assert!(WidgetEventResult::value_changed_consuming_key_text().value_changed());
        assert!(WidgetEventResult::value_changed_consuming_key_text().consumes_key_text());
        assert!(!WidgetEventResult::HANDLED.consumes_key_text());
    }
}
