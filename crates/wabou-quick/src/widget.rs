//! Compatibility facade for the public widget contract and built-in widgets.
//!
//! New widget implementations should depend on `wabou-shell` for the trait;
//! the standard implementations live in the independent `wabou-widgets`
//! crate.

pub use wabou_shell::{
    PaintContext, Widget, WidgetChanges, WidgetEventResult, WidgetFactory, WidgetNodeEvent,
    WidgetStyle,
};
pub use wabou_widgets::{
    Canvas, ImageWidget, PasswordInput, SecretStore, TextInput, builtin_factories,
};
