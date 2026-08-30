//! Compatibility facade for the public widget contract and built-in widgets.
//!
//! New widget implementations should depend on `wabou-shell` for the trait;
//! the legacy implementations live in the independent
//! `wabou-backend-winit-widgets` crate.
//! crate.

pub use wabou_backend_winit_widgets::{Canvas, PasswordInput, SecretStore, TextInput};
pub use wabou_shell::{
    MeasureContext, PaintContext, Widget, WidgetAccessibility, WidgetAvailableSpace, WidgetChanges,
    WidgetEventResult, WidgetFactory, WidgetGeometry, WidgetHarness, WidgetNodeEvent, WidgetStyle,
    decode_widget_config,
};
