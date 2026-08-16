//! Native password input whose secret never crosses the QuickJS bridge.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};
use wabou_shell::style::TextAlign;
use wabou_shell::text::{brush_for_color, layout_text_styled};
use wabou_shell::{ImeEvent, KeyPhase, StandardShortcut, UiEvent};
use zeroize::{Zeroize, Zeroizing};

use wabou_shell::{PaintContext, Widget, WidgetEventResult, WidgetStyle};

use crate::single_line_y_offset;

const PLACEHOLDER: Color = Color::from_rgb8(0x64, 0x74, 0x8b);
const DEFAULT_SLOT: &str = "default";

#[derive(Clone, Default)]
pub struct SecretStore(Arc<Mutex<HashMap<String, Zeroizing<String>>>>);

impl SecretStore {
    pub fn take(&self, slot: &str) -> Zeroizing<String> {
        self.0
            .lock()
            .ok()
            .and_then(|mut secrets| secrets.remove(slot))
            .unwrap_or_default()
    }

    pub fn clear(&self, slot: &str) {
        if let Ok(mut secrets) = self.0.lock()
            && let Some(mut secret) = secrets.remove(slot)
        {
            secret.zeroize();
        }
    }

    fn edit(&self, slot: &str, edit: impl FnOnce(&mut String)) -> bool {
        let Ok(mut secrets) = self.0.lock() else {
            return false;
        };
        edit(secrets.entry(slot.to_owned()).or_default());
        true
    }

    fn character_count(&self, slot: &str) -> usize {
        self.0
            .lock()
            .ok()
            .and_then(|secrets| secrets.get(slot).map(|secret| secret.chars().count()))
            .unwrap_or(0)
    }
}

pub struct PasswordInput {
    secrets: SecretStore,
    slot: String,
    placeholder: String,
    focused: bool,
    disabled: bool,
    font_size: f32,
    font_weight: f32,
    line_height: Option<(f32, bool)>,
    font_family: Option<Arc<str>>,
    color: Color,
}

impl PasswordInput {
    pub fn new(secrets: SecretStore) -> Self {
        Self {
            secrets,
            slot: DEFAULT_SLOT.into(),
            placeholder: String::new(),
            focused: false,
            disabled: false,
            font_size: 16.0,
            font_weight: 400.0,
            line_height: None,
            font_family: None,
            color: Color::WHITE,
        }
    }

    fn insert(&self, text: &str) -> WidgetEventResult {
        let filtered: String = text
            .chars()
            .filter(|character| !character.is_control())
            .collect();
        if filtered.is_empty() {
            return WidgetEventResult::IGNORED;
        }
        if self
            .secrets
            .edit(&self.slot, |secret| secret.push_str(&filtered))
        {
            WidgetEventResult::HANDLED
        } else {
            WidgetEventResult::IGNORED
        }
    }
}

impl Widget for PasswordInput {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let height = cx.height();
        let device_scale = cx.device_scale();
        let tcx = cx.text();
        let count = self.secrets.character_count(&self.slot);
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
            self.line_height,
            TextAlign::Start,
            brush_for_color(color),
            Arc::from([]),
            self.font_family.as_ref(),
            None,
        );
        let y = single_line_y_offset(height, layout.height(), self.font_size);
        let glyphs = tcx.glyph_scene_scaled(&layout, device_scale);
        let mut scene = Scene::new();
        scene.append(
            &glyphs,
            Some(Affine::translate((0.0, y)) * Affine::scale(device_scale.recip())),
        );
        if self.focused && !self.disabled {
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
        cx.scene_mut().append(&scene, None);
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if self.disabled {
            return WidgetEventResult::IGNORED;
        }
        match event {
            UiEvent::TextInput(text) | UiEvent::Paste(text) => self.insert(text),
            UiEvent::Ime(ImeEvent::Commit(text)) => self.insert(text),
            UiEvent::Ime(ImeEvent::Preedit { .. } | ImeEvent::Enabled | ImeEvent::Disabled) => {
                WidgetEventResult::HANDLED
            }
            UiEvent::Key(key)
                if key.phase == KeyPhase::Down
                    && key.matches_standard_shortcut(StandardShortcut::Paste) =>
            {
                WidgetEventResult::paste()
            }
            UiEvent::Key(key) if key.phase == KeyPhase::Down && key.key == "Backspace" => {
                if self.secrets.edit(&self.slot, |secret| {
                    secret.pop();
                }) {
                    WidgetEventResult::HANDLED
                } else {
                    WidgetEventResult::IGNORED
                }
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> wabou_shell::WidgetChanges {
        match name {
            "placeholder" => self.placeholder = value.to_owned(),
            "secret" | "secret-slot" => self.slot = value.to_owned(),
            "disabled" => self.disabled = value != "false",
            _ => {}
        }
        match name {
            "placeholder" | "secret" | "secret-slot" | "disabled" => {
                wabou_shell::WidgetChanges::REDRAW | wabou_shell::WidgetChanges::SEMANTICS
            }
            _ => wabou_shell::WidgetChanges::empty(),
        }
    }

    fn attribute_removed(&mut self, name: &str) -> wabou_shell::WidgetChanges {
        match name {
            "placeholder" => self.placeholder.clear(),
            "secret" | "secret-slot" => self.slot = DEFAULT_SLOT.into(),
            "disabled" => self.disabled = false,
            _ => {}
        }
        match name {
            "placeholder" | "secret" | "secret-slot" | "disabled" => {
                wabou_shell::WidgetChanges::REDRAW | wabou_shell::WidgetChanges::SEMANTICS
            }
            _ => wabou_shell::WidgetChanges::empty(),
        }
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> wabou_shell::WidgetChanges {
        self.font_size = style.font_size;
        self.font_weight = style.font_weight;
        self.line_height = style.line_height;
        self.font_family = style.font_family.clone();
        self.color = style.color;
        wabou_shell::WidgetChanges::REDRAW
    }

    fn accepts_focus(&self) -> bool {
        !self.disabled
    }

    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        wabou_shell::WidgetAccessibility {
            role: Some(wabou_shell::SemanticRole::TextInput),
            disabled: Some(self.disabled),
            ..Default::default()
        }
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([160.0, 32.0])
    }

    fn focus_changed(&mut self, focused: bool) -> wabou_shell::WidgetChanges {
        self.focused = focused;
        wabou_shell::WidgetChanges::REDRAW
    }

    fn unmount(&mut self) {
        self.secrets.clear(&self.slot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_is_taken_once_without_becoming_a_widget_value() {
        let secrets = SecretStore::default();
        let mut input = PasswordInput::new(secrets.clone());
        input.attribute_changed("secret", "master-password");
        assert_eq!(input.accessibility().value, None);
        assert!(
            input
                .handle_event(&UiEvent::TextInput("sëcret🔑".into()))
                .is_handled()
        );
        assert!(input.current_value().is_none());
        assert_eq!(secrets.take("master-password").as_str(), "sëcret🔑");
        assert!(secrets.take("master-password").is_empty());
    }

    #[test]
    fn unmount_zeroizes_and_removes_the_secret() {
        let secrets = SecretStore::default();
        let mut input = PasswordInput::new(secrets.clone());
        input.handle_event(&UiEvent::TextInput("temporary".into()));
        input.unmount();
        assert!(secrets.take(DEFAULT_SLOT).is_empty());
    }
}
