//! App-local secure input.
//!
//! The actual master password never becomes a DOM value or a QuickJS string.
//! Solid receives ordinary input events without a `value`; the native widget
//! retains the secret in zeroizing Rust memory until the login capability
//! atomically takes it.

use std::sync::{Arc, Mutex};

use wabou_quick::vello::Scene;
use wabou_quick::vello::kurbo::{Affine, Rect};
use wabou_quick::vello::peniko::{Color, Fill};
use wabou_quick::widget_api::{KeyPhase, UiEvent, Widget, WidgetEventResult, WidgetStyle};
use wabou_shell::style::TextAlign;
use wabou_shell::text::{TextContext, brush_for_color, layout_text_styled};
use zeroize::{Zeroize, Zeroizing};

const PLACEHOLDER: Color = Color::from_rgb8(0x64, 0x74, 0x8b);

#[derive(Clone, Default)]
pub struct SharedSecret(Arc<Mutex<Zeroizing<String>>>);

pub fn take_secret(secret: &SharedSecret) -> Zeroizing<String> {
    secret
        .0
        .lock()
        .map(|mut value| std::mem::take(&mut *value))
        .unwrap_or_default()
}

pub struct SecureInput {
    secret: SharedSecret,
    placeholder: String,
    focused: bool,
    font_size: f32,
    font_weight: f32,
    color: Color,
}

impl SecureInput {
    pub fn new(secret: SharedSecret) -> Self {
        Self {
            secret,
            placeholder: "Master password".into(),
            focused: false,
            font_size: 16.0,
            font_weight: 400.0,
            color: Color::WHITE,
        }
    }

    fn edit(&self, edit: impl FnOnce(&mut String)) -> WidgetEventResult {
        let Ok(mut secret) = self.secret.0.lock() else {
            return WidgetEventResult::IGNORED;
        };
        edit(&mut secret);
        WidgetEventResult::VALUE_CHANGED
    }
}

impl Widget for SecureInput {
    fn paint(&mut self, _width: f32, height: f32, tcx: &mut TextContext) -> Scene {
        let count = self
            .secret
            .0
            .lock()
            .map(|secret| secret.chars().count())
            .unwrap_or(0);
        let (text, color) = if count == 0 {
            (self.placeholder.clone(), PLACEHOLDER)
        } else {
            ("•".repeat(count), self.color)
        };
        let layout = layout_text_styled(
            tcx,
            Arc::from(text),
            self.font_size,
            self.font_weight,
            None,
            TextAlign::Start,
            brush_for_color(color),
            Arc::from([]),
            None,
            None,
        );
        let y = ((f64::from(height) - f64::from(layout.height())) * 0.5).max(0.0);
        let glyphs = tcx.glyph_scene_scaled(&layout, 1.0);
        let mut scene = Scene::new();
        scene.append(&glyphs, Some(Affine::translate((0.0, y))));
        if self.focused {
            let x = if count == 0 {
                0.0
            } else {
                f64::from(layout.width()) + 1.0
            };
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                self.color,
                None,
                &Rect::new(x, y, x + 1.5, y + f64::from(layout.height())),
            );
        }
        scene
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        match event {
            UiEvent::TextInput(text) | UiEvent::Paste(text) => self.edit(|secret| {
                secret.extend(text.chars().filter(|character| !character.is_control()));
            }),
            UiEvent::Key(key)
                if key.phase == KeyPhase::Down
                    && key.matches_standard_shortcut(wabou_shell::StandardShortcut::Paste) =>
            {
                WidgetEventResult::paste()
            }
            UiEvent::Key(key) if key.phase == KeyPhase::Down && key.key == "Backspace" => self
                .edit(|secret| {
                    secret.pop();
                }),
            UiEvent::Key(key) if key.phase == KeyPhase::Down && key.key == "Escape" => {
                self.edit(String::zeroize)
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn attribute_changed(&mut self, name: &str, value: &str) {
        if name == "placeholder" {
            self.placeholder = value.to_owned();
        }
    }

    fn style_changed(&mut self, style: &WidgetStyle) {
        self.font_size = style.font_size;
        self.font_weight = style.font_weight;
        self.color = style.color;
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([160.0, 32.0])
    }

    fn focus_changed(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn unmount(&mut self) {
        if let Ok(mut secret) = self.secret.0.lock() {
            secret.zeroize();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_is_taken_without_becoming_a_widget_value() {
        let shared = SharedSecret::default();
        let mut input = SecureInput::new(shared.clone());
        assert!(
            input
                .handle_event(&UiEvent::TextInput("correct horse".into()))
                .value_changed()
        );
        assert!(input.current_value().is_none());
        assert_eq!(take_secret(&shared).as_str(), "correct horse");
        assert!(take_secret(&shared).is_empty());
    }
}
