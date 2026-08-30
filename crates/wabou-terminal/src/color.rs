use rio_vt::config::colors::term::{COUNT as TERMINAL_COLOR_COUNT, TermColors};
use rio_vt::config::colors::{AnsiColor, ColorRgb, NamedColor};

/// Renderer-independent RGBA8 terminal color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalColor(pub [u8; 4]);

impl TerminalColor {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 255])
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    pub const fn components(self) -> [u8; 4] {
        self.0
    }
}

pub(crate) fn parse_terminal_color(value: &str) -> Option<TerminalColor> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return match hex.len() {
            3 => Some(TerminalColor::rgb(
                u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?,
                u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?,
            )),
            6 => Some(TerminalColor::rgb(
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
            )),
            8 => Some(TerminalColor::rgba(
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
                u8::from_str_radix(&hex[6..8], 16).ok()?,
            )),
            _ => None,
        };
    }
    let rgb = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|value| value.strip_suffix(')'))?;
    let (components, alpha) = rgb
        .split_once('/')
        .map_or((rgb, None), |(rgb, alpha)| (rgb, Some(alpha.trim())));
    let values = components
        .split([',', ' '])
        .filter(|part| !part.trim().is_empty())
        .map(|part| part.trim().parse::<f32>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    let alpha = alpha
        .and_then(|alpha| alpha.parse::<f32>().ok())
        .or_else(|| values.get(3).copied())
        .unwrap_or(1.0);
    Some(TerminalColor::rgba(
        *values.first()? as u8,
        *values.get(1)? as u8,
        *values.get(2)? as u8,
        (alpha.clamp(0.0, 1.0) * 255.0) as u8,
    ))
}

pub(crate) fn terminal_named_color(color: NamedColor, foreground: bool) -> TerminalColor {
    let fallback = if foreground { 0xe2e8f0 } else { 0x0f172a };
    let rgb = match color {
        NamedColor::Black | NamedColor::DimBlack => 0x1e293b,
        NamedColor::Red | NamedColor::DimRed => 0xef4444,
        NamedColor::Green | NamedColor::DimGreen => 0x22c55e,
        NamedColor::Yellow | NamedColor::DimYellow => 0xeab308,
        NamedColor::Blue | NamedColor::DimBlue => 0x3b82f6,
        NamedColor::Magenta | NamedColor::DimMagenta => 0xd946ef,
        NamedColor::Cyan | NamedColor::DimCyan => 0x06b6d4,
        NamedColor::White | NamedColor::DimWhite => 0xcbd5e1,
        NamedColor::LightBlack => 0x64748b,
        NamedColor::LightRed => 0xf87171,
        NamedColor::LightGreen => 0x4ade80,
        NamedColor::LightYellow => 0xfacc15,
        NamedColor::LightBlue => 0x60a5fa,
        NamedColor::LightMagenta => 0xe879f9,
        NamedColor::LightCyan => 0x22d3ee,
        NamedColor::LightWhite => 0xf8fafc,
        NamedColor::Foreground | NamedColor::LightForeground | NamedColor::DimForeground => {
            fallback
        }
        NamedColor::Background => 0x0f172a,
        NamedColor::Cursor => 0xe2e8f0,
    };
    TerminalColor::rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

fn indexed_color(index: u8) -> TerminalColor {
    if index < 16 {
        const TABLE: [u32; 16] = [
            0x1e293b, 0xef4444, 0x22c55e, 0xeab308, 0x3b82f6, 0xd946ef, 0x06b6d4, 0xcbd5e1,
            0x64748b, 0xf87171, 0x4ade80, 0xfacc15, 0x60a5fa, 0xe879f9, 0x22d3ee, 0xf8fafc,
        ];
        let rgb = TABLE[index as usize];
        return TerminalColor::rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8);
    }
    if index < 232 {
        let value = index - 16;
        let component = |value: u8| if value == 0 { 0 } else { 55 + value * 40 };
        return TerminalColor::rgb(
            component(value / 36),
            component((value / 6) % 6),
            component(value % 6),
        );
    }
    let gray = 8 + (index - 232) * 10;
    TerminalColor::rgb(gray, gray, gray)
}

fn from_f32([r, g, b, a]: [f32; 4]) -> TerminalColor {
    TerminalColor::rgba(
        (r * 255.0).round().clamp(0.0, 255.0) as u8,
        (g * 255.0).round().clamp(0.0, 255.0) as u8,
        (b * 255.0).round().clamp(0.0, 255.0) as u8,
        (a * 255.0).round().clamp(0.0, 255.0) as u8,
    )
}

pub(crate) fn terminal_query_color(
    index: usize,
    colors: &TermColors,
    foreground: TerminalColor,
    background: TerminalColor,
) -> ColorRgb {
    let color = if let Some(color) = (index < TERMINAL_COLOR_COUNT)
        .then(|| colors[index])
        .flatten()
    {
        from_f32(color)
    } else {
        match index {
            index if index < 256 => indexed_color(index as u8),
            index
                if index == NamedColor::Foreground as usize
                    || index == NamedColor::LightForeground as usize
                    || index == NamedColor::DimForeground as usize =>
            {
                foreground
            }
            index if index == NamedColor::Background as usize => background,
            index if index == NamedColor::Cursor as usize => {
                terminal_named_color(NamedColor::Cursor, true)
            }
            _ => terminal_named_color(NamedColor::Foreground, true),
        }
    };
    let [r, g, b, _] = color.components();
    ColorRgb { r, g, b }
}

pub(crate) fn resolve_ansi_color(
    color: AnsiColor,
    foreground: bool,
    colors: &TermColors,
    theme_foreground: TerminalColor,
    theme_background: TerminalColor,
) -> TerminalColor {
    let override_index = match color {
        AnsiColor::Indexed(index) => Some(index as usize),
        AnsiColor::Named(name) => Some(name as usize),
        AnsiColor::Spec(_) => None,
    };
    override_index
        .and_then(|index| colors[index])
        .map(from_f32)
        .unwrap_or_else(|| match color {
            AnsiColor::Spec(ColorRgb { r, g, b }) => TerminalColor::rgb(r, g, b),
            AnsiColor::Indexed(index) => indexed_color(index),
            AnsiColor::Named(
                NamedColor::Foreground | NamedColor::LightForeground | NamedColor::DimForeground,
            ) => theme_foreground,
            AnsiColor::Named(NamedColor::Background) => theme_background,
            AnsiColor::Named(name) => terminal_named_color(name, foreground),
        })
}
