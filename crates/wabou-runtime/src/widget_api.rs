//! Public SDK surface for implementing native widgets outside `wabou-runtime`.
//!
//! Importing this module is enough to implement [`Widget`] and register it on
//! a [`HostBuilder`], without depending directly on `wabou-shell` or matching
//! Wabou's `vello` version.

pub use crate::HostBuilder;
pub use crate::vello;
pub use wabou_shell::{
    ClipboardRequest, HostAction, HostActionResult, ImeEvent, KeyEvent, KeyLocation, KeyPhase,
    MeasureContext, Modifiers, PaintContext, Point, PointerButton, PointerEvent, PointerPhase,
    TextContext, UiEvent, WakeCallback, WheelEvent, Widget, WidgetAccessibility,
    WidgetAvailableSpace, WidgetChanges, WidgetEventResult, WidgetFactory, WidgetGeometry,
    WidgetHarness, WidgetNodeEvent, WidgetStyle, WindowCommand, WindowMetrics, WindowOptions,
};
