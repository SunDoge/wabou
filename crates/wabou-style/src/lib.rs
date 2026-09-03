//! Native utility-class parser for Wabou.
//!
//! The Rust parser is authoritative. [`manifest`] exports the same scales,
//! colors and dynamic rule families for editor/build tooling.

#![warn(missing_docs)]

mod ir;
mod manifest;
mod model;
mod rules;
pub mod stylesheet;
mod theme;

pub use ir::{IrColor, IrLength, IrValue};
pub use manifest::{MANIFEST_VERSION, manifest, manifest_with_theme};
pub use model::{
    Color, Declaration, DynamicPrefix, DynamicRule, Length, Manifest, ParsedUtility, Theme, Value,
};
pub use rules::{ParseError, parse_utility, parse_utility_with_theme};

/// Parse a runtime inline-style string into backend-neutral Style IR.
///
/// Wabou intentionally accepts a small explicit grammar rather than CSS. The
/// parser is shared by every renderer so a backend cannot silently assign a
/// different meaning to an authored value.
#[must_use]
pub fn parse_inline_value(value: &str) -> IrValue {
    let value = value.trim();
    if let Some(rgba) = parse_inline_color(value) {
        return IrValue::Color {
            value: IrColor::Literal { rgba },
        };
    }
    if let Some(length) = parse_inline_length(value) {
        return IrValue::Length { value: length };
    }
    if let Ok(number) = value.parse::<f32>()
        && number.is_finite()
    {
        return IrValue::Number { value: number };
    }
    IrValue::Keyword {
        value: value.to_owned(),
    }
}

fn parse_inline_length(value: &str) -> Option<IrLength> {
    let parse = |number: &str| number.trim().parse::<f32>().ok().filter(|n| n.is_finite());
    if let Some(number) = value.strip_suffix("rem") {
        return parse(number).map(|value| IrLength::Px {
            value: value * 16.0,
        });
    }
    if let Some(number) = value.strip_suffix("em") {
        return parse(number).map(|value| IrLength::Px {
            value: value * 16.0,
        });
    }
    if let Some(number) = value.strip_suffix("px") {
        return parse(number).map(|value| IrLength::Px { value });
    }
    if let Some(number) = value.strip_suffix('%') {
        return parse(number).map(|value| IrLength::Percent {
            value: value / 100.0,
        });
    }
    None
}

fn parse_inline_color(value: &str) -> Option<u32> {
    if let Some(hex) = value.strip_prefix('#') {
        return match hex.len() {
            3 => {
                let nibble = |index| u8::from_str_radix(&hex[index..index + 1], 16).ok();
                Some(
                    (u32::from(nibble(0)?) * 0x1100_0000)
                        | (u32::from(nibble(1)?) * 0x0011_0000)
                        | (u32::from(nibble(2)?) * 0x0000_1100)
                        | 0xff,
                )
            }
            6 => u32::from_str_radix(hex, 16)
                .ok()
                .map(|rgb| (rgb << 8) | 0xff),
            8 => u32::from_str_radix(hex, 16).ok(),
            _ => None,
        };
    }
    let named = match value.to_ascii_lowercase().as_str() {
        "black" => 0x0000_00ff,
        "white" => 0xffff_ffff,
        "transparent" => 0x0000_0000,
        "red" => 0xff00_00ff,
        "green" => 0x0080_00ff,
        "blue" => 0x0000_ffff,
        "yellow" => 0xffff_00ff,
        "cyan" | "aqua" => 0x00ff_ffff,
        "magenta" | "fuchsia" => 0xff00_ffff,
        "gray" | "grey" => 0x8080_80ff,
        "orange" => 0xffa5_00ff,
        "purple" => 0x8000_80ff,
        "pink" => 0xffc0_cbff,
        "lime" => 0x00ff_00ff,
        "teal" => 0x0080_80ff,
        "navy" => 0x0000_80ff,
        "maroon" => 0x8000_00ff,
        "olive" => 0x8080_00ff,
        "silver" => 0xc0c0_c0ff,
        _ => return parse_rgb_color(value),
    };
    Some(named)
}

fn parse_rgb_color(value: &str) -> Option<u32> {
    let body = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))?
        .strip_suffix(')')?;
    let (rgb, explicit_alpha) = body
        .split_once('/')
        .map_or((body, None), |(rgb, alpha)| (rgb, Some(alpha.trim())));
    let values = rgb
        .split([',', ' '])
        .filter(|part| !part.trim().is_empty())
        .map(|part| part.trim().parse::<f32>().ok())
        .collect::<Option<Vec<_>>>()?;
    let byte = |value: f32| value.clamp(0.0, 255.0) as u8;
    let alpha = explicit_alpha
        .and_then(|value| value.parse::<f32>().ok())
        .or_else(|| values.get(3).copied())
        .unwrap_or(1.0)
        .clamp(0.0, 1.0);
    Some(
        (u32::from(byte(*values.first()?)) << 24)
            | (u32::from(byte(*values.get(1)?)) << 16)
            | (u32::from(byte(*values.get(2)?)) << 8)
            | u32::from((alpha * 255.0) as u8),
    )
}

#[cfg(test)]
mod inline_value_tests {
    use super::*;

    #[test]
    fn parses_the_renderer_independent_inline_grammar() {
        assert_eq!(
            parse_inline_value("1.5rem"),
            IrValue::Length {
                value: IrLength::Px { value: 24.0 }
            }
        );
        assert_eq!(
            parse_inline_value("25%"),
            IrValue::Length {
                value: IrLength::Percent { value: 0.25 }
            }
        );
        assert_eq!(
            parse_inline_value("#123"),
            IrValue::Color {
                value: IrColor::Literal { rgba: 0x1122_33ff }
            }
        );
        assert_eq!(
            parse_inline_value("rgb(1 2 3 / 0.5)"),
            IrValue::Color {
                value: IrColor::Literal { rgba: 0x0102_037f }
            }
        );
        assert_eq!(
            parse_inline_value("flex"),
            IrValue::Keyword {
                value: "flex".to_owned()
            }
        );
    }
}
