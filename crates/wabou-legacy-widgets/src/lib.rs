//! Legacy native widgets implemented exclusively through
//! `wabou-vello-shell`'s public [`wabou_shell::Widget`] contract.

#![warn(missing_docs)]

extern crate wabou_vello_shell as wabou_shell;

use std::collections::HashMap;
use std::sync::Arc;

use wabou_shell::WidgetFactory;

mod canvas;
mod code_editor;
mod controls;
mod image;
mod password_input;
mod text_input;

pub use canvas::Canvas;
pub use code_editor::CodeEditor;
pub use controls::{IndeterminateProgress, Slider, Spinner};
pub use image::ImageWidget;
pub use password_input::{PasswordInput, SecretStore};
pub use text_input::TextInput;

/// Factories installed by the standard QuickJS host.
pub fn builtin_factories() -> HashMap<String, WidgetFactory> {
    let mut factories: HashMap<String, WidgetFactory> = HashMap::new();
    factories.insert("canvas".into(), Arc::new(|| Box::new(Canvas)));
    factories.insert("editor".into(), Arc::new(|| Box::new(CodeEditor::new())));
    factories.insert("img".into(), Arc::new(|| Box::new(ImageWidget::new())));
    factories.insert("input".into(), Arc::new(|| Box::new(TextInput::new())));
    factories.insert("spinner".into(), Arc::new(|| Box::new(Spinner::new())));
    factories.insert("slider".into(), Arc::new(|| Box::new(Slider::new())));
    factories.insert(
        "progress-indeterminate".into(),
        Arc::new(|| Box::new(IndeterminateProgress::new())),
    );
    factories.insert(
        "textarea".into(),
        Arc::new(|| Box::new(TextInput::multiline())),
    );
    factories
}

/// Construct a factory for secure password inputs sharing one Rust-only store.
pub fn password_input_factory(secrets: SecretStore) -> WidgetFactory {
    Arc::new(move || Box::new(PasswordInput::new(secrets.clone())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_factories_cover_framework_native_components() {
        let factories = builtin_factories();
        let textarea = factories["textarea"]();
        assert_eq!(textarea.intrinsic_size(), Some([240.0, 96.0]));
        assert!(textarea.accepts_focus());
        assert!(factories.contains_key("spinner"));
        assert!(factories.contains_key("slider"));
        assert!(factories.contains_key("progress-indeterminate"));
        assert!(factories.contains_key("editor"));
        assert!(!factories.contains_key("code-editor"));
    }
}
