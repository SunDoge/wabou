//! Test-only access to the legacy widget contract.

pub use legacy_shell::{
    MeasureContext, Widget, WidgetChanges, WidgetEventResult, WidgetNodeEvent, WidgetStyle,
};
pub(crate) use wabou_legacy_widgets::{PasswordInput, SecretStore};
