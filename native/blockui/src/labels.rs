//! Small noninteractive labels and semantic dividers.
use crate::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum BadgeVariant {
    #[default]
    Default,
    Secondary,
    Outline,
    Destructive,
}
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Badge {
    #[setters(generate, into)]
    pub key: Key,
    pub text: Binding,
    #[setters(generate)]
    pub variant: BadgeVariant,
}
impl Badge {
    pub fn new(text: impl Into<Binding>) -> Self {
        Self {
            key: Key::default(),
            text: text.into(),
            variant: BadgeVariant::Default,
        }
    }
}
impl IntoView for Badge {
    fn into_view(self) -> View {
        let mut view = Text::new(self.text).key(self.key).style(ViewStyle {
            font_size: 12.0,
            ..default()
        });
        view.node.semantics.role = abi::semantics::BADGE;
        view.node.button_variant = self.variant as u32;
        view.node.padding = 2.0;
        view.node.height = 0.0;
        view
    }
}

/// A nonfocusable semantic separator. Vertical separators have explicit height.
#[derive(derive_setters::Setters)]
#[setters(generate = false)]
#[must_use = "mount this widget with #[node], Ui::add, a parent, or return it from the view"]
pub struct Separator {
    #[setters(generate, into)]
    pub key: Key,
    height: f32,
    vertical: bool,
}
impl Default for Separator {
    fn default() -> Self {
        Self::horizontal()
    }
}
impl Separator {
    pub fn horizontal() -> Self {
        Self {
            key: Key::default(),
            height: 1.0,
            vertical: false,
        }
    }
    pub fn vertical(height: f32) -> Self {
        Self {
            key: Key::default(),
            height,
            vertical: true,
        }
    }
}
impl IntoView for Separator {
    fn into_view(self) -> View {
        let mut view = Column::default().key(self.key).into_view();
        view.node.padding = 0.0;
        view.node.gap = 0.0;
        view.node.height = self.height;
        view.node.semantics.role = if self.vertical {
            abi::semantics::VERTICAL_SEPARATOR
        } else {
            abi::semantics::HORIZONTAL_SEPARATOR
        };
        if self.vertical {
            view.node.presentation.width = 1.0;
        }
        view
    }
}
