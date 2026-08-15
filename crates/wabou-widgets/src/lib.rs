//! Built-in native widgets implemented exclusively through `wabou-shell`'s
//! public [`wabou_shell::Widget`] contract.

use std::collections::HashMap;
use std::sync::Arc;

use wabou_shell::WidgetFactory;

mod canvas;
mod code_editor;
mod image;
mod password_input;
mod text_input;

pub use canvas::Canvas;
pub use code_editor::CodeEditor;
pub use image::ImageWidget;
pub use password_input::{PasswordInput, SecretStore};
pub use text_input::TextInput;

/// Factories installed by the standard QuickJS host.
pub fn builtin_factories() -> HashMap<String, WidgetFactory> {
    let mut factories: HashMap<String, WidgetFactory> = HashMap::new();
    factories.insert("canvas".into(), Arc::new(|| Box::new(Canvas)));
    factories.insert(
        "code-editor".into(),
        Arc::new(|| Box::new(CodeEditor::new())),
    );
    factories.insert("img".into(), Arc::new(|| Box::new(ImageWidget::new())));
    factories.insert("input".into(), Arc::new(|| Box::new(TextInput::new())));
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

/// Vertically position the contents of a single-line native input.
fn single_line_y_offset(container_height: f32, line_height: f32, font_size: f32) -> f64 {
    let available = (container_height - line_height).max(0.0);
    let optical_offset = font_size / 16.0;
    f64::from((available * 0.5 + optical_offset).min(available))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_factories_include_multiline_textarea() {
        let factories = builtin_factories();
        let textarea = factories["textarea"]();
        assert_eq!(textarea.intrinsic_size(), Some([240.0, 96.0]));
        assert!(textarea.accepts_focus());
    }

    #[test]
    fn single_line_inputs_share_a_scale_independent_optical_offset() {
        assert_eq!(single_line_y_offset(32.0, 24.0, 16.0), 5.0);
        assert_eq!(single_line_y_offset(32.0, 32.0, 16.0), 0.0);
        assert_eq!(single_line_y_offset(16.0, 24.0, 16.0), 0.0);
    }
}
