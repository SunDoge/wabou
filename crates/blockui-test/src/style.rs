use crate::{Result, abi};
use blockui::{Density, Surface, TextTone, ThemeMode, ViewStyle};

pub(super) fn snapshot(style: abi::Presentation) -> Result<ViewStyle> {
    let invalid = |field: &str, value: u32| format!("invalid projected style {field}: {value}");
    Ok(ViewStyle {
        width: style.width,
        max_width: style.max_width,
        font_size: style.font_size,
        radius: style.radius,
        text_tone: match style.text_tone {
            0 => TextTone::Default,
            1 => TextTone::Muted,
            2 => TextTone::Accent,
            3 => TextTone::Error,
            value => return Err(invalid("text_tone", value)),
        },
        surface: match style.surface {
            0 => Surface::Default,
            1 => Surface::Canvas,
            2 => Surface::Panel,
            value => return Err(invalid("surface", value)),
        },
        theme: match style.theme {
            0 => ThemeMode::Inherit,
            1 => ThemeMode::Light,
            2 => ThemeMode::Dark,
            3 => ThemeMode::System,
            value => return Err(invalid("theme", value)),
        },
        density: match style.density {
            0 => Density::Inherit,
            1 => Density::Comfortable,
            2 => Density::Compact,
            value => return Err(invalid("density", value)),
        },
        fill_height: match style.fill_height {
            0 => false,
            1 => true,
            value => return Err(invalid("fill_height", value)),
        },
    })
}
