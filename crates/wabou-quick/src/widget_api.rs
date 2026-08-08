//! Public SDK surface for implementing native widgets outside `wabou-quick`.
//!
//! Importing this module is enough to implement [`Widget`] and register it on
//! a [`HostBuilder`], without depending directly on `wabou-shell` or matching
//! Wabou's `vello` version.

pub use crate::HostBuilder;
pub use crate::vello;
pub use crate::widget::{
    Widget, WidgetChanges, WidgetEventResult, WidgetFactory, WidgetNodeEvent, WidgetStyle,
};
pub use wabou_shell::{
    ClipboardRequest, HostAction, HostActionResult, KeyEvent, KeyLocation, KeyPhase, Modifiers,
    Point, PointerButton, PointerEvent, PointerPhase, TextContext, UiEvent, WakeCallback,
    WheelEvent, WindowCommand, WindowMetrics, WindowOptions,
};
