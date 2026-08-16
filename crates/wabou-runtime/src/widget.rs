//! Compatibility facade for the public widget contract and built-in widgets.
//!
//! New widget implementations should depend on `wabou-shell` for the trait;
//! the standard implementations live in the independent `wabou-widgets`
//! crate.

pub use wabou_shell::{
    MeasureContext, PaintContext, Widget, WidgetAccessibility, WidgetAvailableSpace, WidgetChanges,
    WidgetEventResult, WidgetFactory, WidgetGeometry, WidgetHarness, WidgetNodeEvent, WidgetStyle,
    decode_widget_config,
};
pub use wabou_widgets::{Canvas, PasswordInput, SecretStore, TextInput};
