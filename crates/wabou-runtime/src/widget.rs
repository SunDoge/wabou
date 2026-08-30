//! Test-only access to the legacy widget contract.

pub use legacy_shell::{
    MeasureContext, Widget, WidgetChanges, WidgetEventResult, WidgetNodeEvent, WidgetStyle,
};
pub(crate) use wabou_backend_winit_widgets::{PasswordInput, SecretStore};
