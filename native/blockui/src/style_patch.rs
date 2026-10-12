use crate::{
    Cx, Density, Error, Live, Result, StyleBinding, Surface, TextTone, ThemeMode, View, ViewStyle,
    abi,
};

/// Explicit presentation overrides. None preserves the preceding value;
/// Some(0.0), Some(false) and Some(Inherit) are real overrides.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StylePatch {
    pub width: Option<f32>,
    pub max_width: Option<f32>,
    pub font_size: Option<f32>,
    pub text_tone: Option<TextTone>,
    pub surface: Option<Surface>,
    pub radius: Option<f32>,
    pub theme: Option<ThemeMode>,
    pub density: Option<Density>,
    pub fill_height: Option<bool>,
}
impl StylePatch {
    fn apply(self, target: &mut abi::Presentation) {
        if let Some(value) = self.width {
            target.width = value;
        }
        if let Some(value) = self.max_width {
            target.max_width = value;
        }
        if let Some(value) = self.font_size {
            target.font_size = value;
        }
        if let Some(value) = self.radius {
            target.radius = value;
        }
        if let Some(value) = self.text_tone {
            target.text_tone = value as u32;
        }
        if let Some(value) = self.surface {
            target.surface = value as u32;
        }
        if let Some(value) = self.theme {
            target.theme = value as u32;
        }
        if let Some(value) = self.density {
            target.density = value as u32;
        }
        if let Some(value) = self.fill_height {
            target.fill_height = u32::from(value);
        }
    }
}
impl From<StylePatch> for StyleBinding {
    fn from(value: StylePatch) -> Self {
        Self::Patch(value)
    }
}
impl From<Live<StylePatch>> for StyleBinding {
    fn from(value: Live<StylePatch>) -> Self {
        Self::LivePatch(value)
    }
}
macro_rules! tuple_styles {
    ($($name:ident),+) => {
        impl<$($name: Into<StyleBinding>),+> From<($($name,)+)> for StyleBinding {
            #[allow(non_snake_case)]
            fn from(($($name,)+): ($($name,)+)) -> Self { Self::Layers(vec![$($name.into(),)+]) }
        }
    };
}
tuple_styles!(A, B);
tuple_styles!(A, B, C);
tuple_styles!(A, B, C, D);

impl StyleBinding {
    fn dynamic(&self) -> bool {
        match self {
            Self::Live(_) | Self::LivePatch(_) => true,
            Self::Layers(values) => values.iter().any(Self::dynamic),
            _ => false,
        }
    }
    fn apply_static(&self, target: &mut abi::Presentation) {
        match self {
            Self::Static(value) => *target = value.presentation(),
            Self::Patch(value) => value.apply(target),
            Self::Layers(values) => {
                for value in values {
                    value.apply_static(target);
                }
            }
            _ => unreachable!("dynamic style evaluated without context"),
        }
    }
    fn evaluate(&self, cx: &Cx<'_>, target: &mut abi::Presentation) -> Result<()> {
        match self {
            Self::Live(value) => *target = (value.compute)(cx)?.presentation(),
            Self::LivePatch(value) => (value.compute)(cx)?.apply(target),
            Self::Layers(values) => {
                for value in values {
                    value.evaluate(cx, target)?;
                }
            }
            _ => self.apply_static(target),
        }
        Ok(())
    }
    pub(crate) fn apply_to(self, mut view: View) -> View {
        // Preserve the existing fast entries and their observer allocation cost.
        match self {
            Self::Static(value) => {
                view.reactive_style = None;
                view.node.presentation = value.presentation();
                view
            }
            Self::Live(value) => view.with_reactive_style(value.compute),
            value if !value.dynamic() => {
                view.reactive_style = None;
                value.apply_static(&mut view.node.presentation);
                view
            }
            value => {
                let baseline = view.node.presentation;
                view.style_reactive(move |cx| {
                    let mut result = baseline;
                    value.evaluate(cx, &mut result)?;
                    decode(result)
                })
            }
        }
    }
}
fn decode(value: abi::Presentation) -> Result<ViewStyle> {
    let invalid = || Error::InvalidView("invalid composed presentation enum");
    Ok(ViewStyle {
        width: value.width,
        max_width: value.max_width,
        font_size: value.font_size,
        radius: value.radius,
        text_tone: match value.text_tone {
            0 => TextTone::Default,
            1 => TextTone::Muted,
            2 => TextTone::Accent,
            3 => TextTone::Error,
            _ => return Err(invalid()),
        },
        surface: match value.surface {
            0 => Surface::Default,
            1 => Surface::Canvas,
            2 => Surface::Panel,
            _ => return Err(invalid()),
        },
        theme: match value.theme {
            0 => ThemeMode::Inherit,
            1 => ThemeMode::Light,
            2 => ThemeMode::Dark,
            3 => ThemeMode::System,
            _ => return Err(invalid()),
        },
        density: match value.density {
            0 => Density::Inherit,
            1 => Density::Comfortable,
            2 => Density::Compact,
            _ => return Err(invalid()),
        },
        fill_height: match value.fill_height {
            0 => false,
            1 => true,
            _ => return Err(invalid()),
        },
    })
}
