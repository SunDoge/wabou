//! Native utility-class parser for Wabou.
//!
//! The Rust parser is authoritative. [`manifest`] exports the same scales,
//! colors and dynamic rule families for editor/build tooling.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use winnow::ascii::float;
use winnow::combinator::{alt, delimited, repeat, terminated};
use winnow::error::ContextError;
use winnow::token::{rest, take_till, take_while};
use winnow::{ModalResult, Parser};

pub const MANIFEST_VERSION: u16 = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "unit", rename_all = "kebab-case")]
pub enum Length {
    Px { value: f32 },
    Percent { value: f32 },
    Auto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Color {
    Literal { rgba: u32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Value {
    Keyword { value: String },
    Boolean { value: bool },
    Number { value: f32 },
    Length { value: Length },
    Color { value: Color },
    List { values: Vec<Value> },
    Record { fields: BTreeMap<String, Value> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Declaration {
    pub property: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedUtility {
    pub class_name: String,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnknownUtility(String),
    InvalidValue {
        utility: String,
        expected: &'static str,
    },
    InvalidCandidate {
        utility: String,
        reason: &'static str,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownUtility(value) => write!(f, "unsupported Wabou utility `{value}`"),
            Self::InvalidValue { utility, expected } => {
                write!(
                    f,
                    "invalid value in Wabou utility `{utility}`; expected {expected}"
                )
            }
            Self::InvalidCandidate { utility, reason } => {
                write!(f, "invalid Wabou utility `{utility}`: {reason}")
            }
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRule {
    pub prefixes: Vec<&'static str>,
    pub resolver: &'static str,
    pub properties: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: u16,
    pub spacing: BTreeMap<String, f32>,
    pub colors: BTreeMap<String, u32>,
    pub static_utilities: BTreeMap<&'static str, Vec<Declaration>>,
    pub dynamic_rules: Vec<DynamicRule>,
    pub conformance: Vec<ParsedUtility>,
}

const SPACING: &[(&str, f32)] = &[
    ("0", 0.0),
    ("0.5", 2.0),
    ("1", 4.0),
    ("1.5", 6.0),
    ("2", 8.0),
    ("2.5", 10.0),
    ("3", 12.0),
    ("3.5", 14.0),
    ("4", 16.0),
    ("5", 20.0),
    ("6", 24.0),
    ("7", 28.0),
    ("8", 32.0),
    ("9", 36.0),
    ("10", 40.0),
    ("11", 44.0),
    ("12", 48.0),
    ("14", 56.0),
    ("16", 64.0),
    ("18", 72.0),
    ("20", 80.0),
    ("24", 96.0),
    ("28", 112.0),
    ("32", 128.0),
    ("36", 144.0),
    ("40", 160.0),
    ("44", 176.0),
    ("48", 192.0),
    ("52", 208.0),
    ("56", 224.0),
    ("60", 240.0),
    ("64", 256.0),
    ("72", 288.0),
    ("80", 320.0),
    ("96", 384.0),
];

const COLORS: &[(&str, u32)] = &[
    ("transparent", 0x00000000),
    ("black", 0x000000ff),
    ("white", 0xffffffff),
    ("slate-50", 0xf8fafcff),
    ("slate-100", 0xf1f5f9ff),
    ("slate-200", 0xe2e8f0ff),
    ("slate-300", 0xcbd5e1ff),
    ("slate-400", 0x94a3b8ff),
    ("slate-500", 0x64748bff),
    ("slate-600", 0x475569ff),
    ("slate-700", 0x334155ff),
    ("slate-800", 0x1e293bff),
    ("slate-900", 0x0f172aff),
    ("slate-950", 0x020617ff),
    ("red-50", 0xfef2f2ff),
    ("red-100", 0xfee2e2ff),
    ("red-200", 0xfecacaff),
    ("red-300", 0xfca5a5ff),
    ("red-500", 0xef4444ff),
    ("red-600", 0xdc2626ff),
    ("red-700", 0xb91c1cff),
    ("red-800", 0x991b1bff),
    ("red-900", 0x7f1d1dff),
    ("red-950", 0x450a0aff),
    ("sky-400", 0x38bdf8ff),
    ("sky-500", 0x0ea5e9ff),
    ("sky-600", 0x0284c7ff),
    ("sky-700", 0x0369a1ff),
    ("emerald-50", 0xecfdf5ff),
    ("emerald-100", 0xd1fae5ff),
    ("emerald-200", 0xa7f3d0ff),
    ("emerald-300", 0x6ee7b7ff),
    ("emerald-400", 0x34d399ff),
    ("emerald-600", 0x059669ff),
    ("emerald-700", 0x047857ff),
    ("emerald-800", 0x065f46ff),
    ("emerald-950", 0x022c22ff),
    ("blue-600", 0x2563ebff),
    ("cyan-400", 0x22d3eeff),
    ("purple-300", 0xd8b4feff),
    ("purple-900", 0x581c87ff),
    ("violet-400", 0xa78bfaff),
    ("violet-950", 0x2e1065ff),
    ("amber-300", 0xfcd34dff),
];

fn keyword(property: &str, value: &str) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Keyword {
            value: value.into(),
        },
    }
}

fn number(property: &str, value: f32) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Number { value },
    }
}

fn length(property: &str, value: Length) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Length { value },
    }
}

fn transform(kind: &str, value: Value) -> Declaration {
    Declaration {
        property: "transform".to_string(),
        value: Value::List {
            values: vec![Value::Record {
                fields: BTreeMap::from([
                    ("kind".to_string(), Value::Keyword { value: kind.into() }),
                    ("value".to_string(), value),
                ]),
            }],
        },
    }
}

fn color(property: &str, rgba: u32) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Color {
            value: Color::Literal { rgba },
        },
    }
}

fn static_utilities() -> BTreeMap<&'static str, Vec<Declaration>> {
    BTreeMap::from([
        ("block", vec![keyword("display", "block")]),
        ("flex", vec![keyword("display", "flex")]),
        ("inline-flex", vec![keyword("display", "flex")]),
        ("grid", vec![keyword("display", "grid")]),
        ("hidden", vec![keyword("display", "none")]),
        ("flex-row", vec![keyword("flex-direction", "row")]),
        ("flex-col", vec![keyword("flex-direction", "column")]),
        ("flex-wrap", vec![keyword("flex-wrap", "wrap")]),
        ("flex-nowrap", vec![keyword("flex-wrap", "nowrap")]),
        (
            "flex-1",
            vec![
                number("flex-grow", 1.0),
                number("flex-shrink", 1.0),
                length("flex-basis", Length::Percent { value: 0.0 }),
            ],
        ),
        (
            "flex-none",
            vec![
                number("flex-grow", 0.0),
                number("flex-shrink", 0.0),
                keyword("flex-basis", "auto"),
            ],
        ),
        ("grow", vec![number("flex-grow", 1.0)]),
        ("grow-0", vec![number("flex-grow", 0.0)]),
        ("shrink", vec![number("flex-shrink", 1.0)]),
        ("shrink-0", vec![number("flex-shrink", 0.0)]),
        ("items-start", vec![keyword("align-items", "flex-start")]),
        ("items-center", vec![keyword("align-items", "center")]),
        ("items-end", vec![keyword("align-items", "flex-end")]),
        ("items-stretch", vec![keyword("align-items", "stretch")]),
        (
            "justify-start",
            vec![keyword("justify-content", "flex-start")],
        ),
        ("justify-center", vec![keyword("justify-content", "center")]),
        ("justify-end", vec![keyword("justify-content", "flex-end")]),
        (
            "justify-between",
            vec![keyword("justify-content", "space-between")],
        ),
        (
            "justify-around",
            vec![keyword("justify-content", "space-around")],
        ),
        ("relative", vec![keyword("position", "relative")]),
        ("absolute", vec![keyword("position", "absolute")]),
        ("overflow-hidden", vec![keyword("overflow", "hidden")]),
        ("overflow-auto", vec![keyword("overflow", "auto")]),
        ("overflow-scroll", vec![keyword("overflow", "scroll")]),
        ("overflow-x-hidden", vec![keyword("overflow-x", "hidden")]),
        ("overflow-y-hidden", vec![keyword("overflow-y", "hidden")]),
        ("overflow-x-auto", vec![keyword("overflow-x", "auto")]),
        ("overflow-y-auto", vec![keyword("overflow-y", "auto")]),
        ("overflow-x-scroll", vec![keyword("overflow-x", "scroll")]),
        ("overflow-y-scroll", vec![keyword("overflow-y", "scroll")]),
        ("box-border", vec![keyword("box-sizing", "border-box")]),
        (
            "border",
            vec![length("border-width", Length::Px { value: 1.0 })],
        ),
        (
            "border-0",
            vec![length("border-width", Length::Px { value: 0.0 })],
        ),
        (
            "border-2",
            vec![length("border-width", Length::Px { value: 2.0 })],
        ),
        (
            "border-t",
            vec![length("border-top-width", Length::Px { value: 1.0 })],
        ),
        (
            "border-r",
            vec![length("border-right-width", Length::Px { value: 1.0 })],
        ),
        (
            "border-b",
            vec![length("border-bottom-width", Length::Px { value: 1.0 })],
        ),
        (
            "border-l",
            vec![length("border-left-width", Length::Px { value: 1.0 })],
        ),
        (
            "rounded",
            vec![length("border-radius", Length::Px { value: 4.0 })],
        ),
        (
            "rounded-sm",
            vec![length("border-radius", Length::Px { value: 2.0 })],
        ),
        (
            "rounded-md",
            vec![length("border-radius", Length::Px { value: 6.0 })],
        ),
        (
            "rounded-lg",
            vec![length("border-radius", Length::Px { value: 8.0 })],
        ),
        (
            "rounded-xl",
            vec![length("border-radius", Length::Px { value: 12.0 })],
        ),
        (
            "rounded-full",
            vec![length("border-radius", Length::Px { value: 9999.0 })],
        ),
        ("font-sans", vec![keyword("font-family", "sans-serif")]),
        ("font-mono", vec![keyword("font-family", "monospace")]),
        ("font-medium", vec![number("font-weight", 500.0)]),
        ("font-semibold", vec![number("font-weight", 600.0)]),
        ("font-bold", vec![number("font-weight", 700.0)]),
        (
            "text-xs",
            vec![
                length("font-size", Length::Px { value: 12.0 }),
                length("line-height", Length::Px { value: 16.0 }),
            ],
        ),
        (
            "text-sm",
            vec![
                length("font-size", Length::Px { value: 14.0 }),
                length("line-height", Length::Px { value: 20.0 }),
            ],
        ),
        (
            "text-base",
            vec![
                length("font-size", Length::Px { value: 16.0 }),
                length("line-height", Length::Px { value: 24.0 }),
            ],
        ),
        (
            "text-lg",
            vec![
                length("font-size", Length::Px { value: 18.0 }),
                length("line-height", Length::Px { value: 28.0 }),
            ],
        ),
        (
            "text-xl",
            vec![
                length("font-size", Length::Px { value: 20.0 }),
                length("line-height", Length::Px { value: 28.0 }),
            ],
        ),
        (
            "text-2xl",
            vec![
                length("font-size", Length::Px { value: 24.0 }),
                length("line-height", Length::Px { value: 32.0 }),
            ],
        ),
        (
            "text-3xl",
            vec![
                length("font-size", Length::Px { value: 30.0 }),
                length("line-height", Length::Px { value: 36.0 }),
            ],
        ),
        ("leading-tight", vec![number("line-height", 1.25)]),
        ("text-left", vec![keyword("text-align", "left")]),
        ("text-center", vec![keyword("text-align", "center")]),
        ("text-right", vec![keyword("text-align", "right")]),
        ("whitespace-nowrap", vec![keyword("white-space", "nowrap")]),
        (
            "pointer-events-none",
            vec![keyword("pointer-events", "none")],
        ),
        ("select-none", vec![keyword("user-select", "none")]),
        (
            "translate-x-4",
            vec![transform(
                "translateX",
                Value::Length {
                    value: Length::Px { value: 16.0 },
                },
            )],
        ),
        (
            "translate-y-4",
            vec![transform(
                "translateY",
                Value::Length {
                    value: Length::Px { value: 16.0 },
                },
            )],
        ),
        (
            "scale-50",
            vec![transform(
                "scale",
                Value::List {
                    values: vec![Value::Number { value: 0.5 }, Value::Number { value: 0.5 }],
                },
            )],
        ),
        (
            "scale-150",
            vec![transform(
                "scale",
                Value::List {
                    values: vec![Value::Number { value: 1.5 }, Value::Number { value: 1.5 }],
                },
            )],
        ),
        (
            "rotate-45",
            vec![transform(
                "rotate",
                Value::Number {
                    value: std::f32::consts::FRAC_PI_4,
                },
            )],
        ),
        (
            "max-w-xl",
            vec![length("max-width", Length::Px { value: 576.0 })],
        ),
        (
            "max-w-3xl",
            vec![length("max-width", Length::Px { value: 768.0 })],
        ),
        (
            "max-w-4xl",
            vec![length("max-width", Length::Px { value: 896.0 })],
        ),
    ])
}

pub fn manifest() -> Manifest {
    let mut manifest = Manifest {
        version: MANIFEST_VERSION,
        spacing: SPACING.iter().map(|(k, v)| ((*k).into(), *v)).collect(),
        colors: COLORS.iter().map(|(k, v)| ((*k).into(), *v)).collect(),
        static_utilities: static_utilities(),
        dynamic_rules: vec![
            DynamicRule {
                prefixes: vec![
                    "p", "px", "py", "pt", "pr", "pb", "pl", "m", "mx", "my", "mt", "mr", "mb",
                    "ml", "gap",
                ],
                resolver: "spacing",
                properties: vec![],
            },
            DynamicRule {
                prefixes: vec![
                    "w", "h", "min-w", "min-h", "max-w", "max-h", "top", "right", "bottom", "left",
                    "inset",
                ],
                resolver: "dimension",
                properties: vec![],
            },
            DynamicRule {
                prefixes: vec!["bg", "text", "border"],
                resolver: "color",
                properties: vec![],
            },
            DynamicRule {
                prefixes: vec!["opacity"],
                resolver: "opacity",
                properties: vec!["opacity"],
            },
        ],
        conformance: Vec::new(),
    };
    manifest.conformance = [
        "flex",
        "flex-1",
        "px-3",
        "px-[13px]",
        "w-full",
        "w-38%",
        "min-h-0",
        "inset-[5%]",
        "bg-slate-900",
        "bg-slate-800/60",
        "border-red-500",
        "opacity-50",
        "rounded-xl",
        "text-sm",
        "translate-x-4",
        "scale-150",
        "rotate-45",
    ]
    .into_iter()
    .map(|candidate| parse_utility(candidate).expect("manifest conformance utility must parse"))
    .collect();
    manifest
}

fn explicit_length(input: &mut &str) -> ModalResult<Length, ContextError> {
    alt((
        terminated(float, "px").map(|value| Length::Px { value }),
        terminated(float, "rem").map(|value: f32| Length::Px {
            value: value * 16.0,
        }),
        terminated(float, '%').map(|value: f32| Length::Percent {
            value: value / 100.0,
        }),
    ))
    .parse_next(input)
}

fn arbitrary_length(input: &mut &str) -> ModalResult<Length, ContextError> {
    delimited('[', explicit_length, ']').parse_next(input)
}

fn bare_percent(input: &mut &str) -> ModalResult<Length, ContextError> {
    terminated(float, '%')
        .map(|value: f32| Length::Percent {
            value: value / 100.0,
        })
        .parse_next(input)
}

fn parse_length(token: &str, spacing: bool) -> Option<Length> {
    if token == "auto" {
        return Some(Length::Auto);
    }
    if token == "full" {
        return Some(Length::Percent { value: 1.0 });
    }
    if token == "px" && spacing {
        return Some(Length::Px { value: 1.0 });
    }
    if spacing && let Some((_, value)) = SPACING.iter().find(|(name, _)| *name == token) {
        return Some(Length::Px { value: *value });
    }
    arbitrary_length
        .parse(token)
        .or_else(|_| bare_percent.parse(token))
        .ok()
}

fn candidate<'a>(input: &mut &'a str) -> ModalResult<&'a str, ContextError> {
    let utility = *input;
    let plain = take_while(1.., |character: char| {
        character != '[' && character != ']' && character != ':'
    });
    let arbitrary = delimited('[', take_till(0.., |character: char| character == ']'), ']');
    let _: Vec<&str> = repeat(1.., alt((plain, arbitrary))).parse_next(input)?;
    Ok(utility)
}

fn candidate_parts(class_name: &str) -> Result<&str, ParseError> {
    candidate
        .parse(class_name)
        .map_err(|_| ParseError::InvalidCandidate {
            utility: class_name.into(),
            reason: "invalid utility or arbitrary-value syntax",
        })
}

fn edge_properties(prefix: &str) -> Option<(&'static str, &'static [&'static str])> {
    Some(match prefix {
        "p" => (
            "padding",
            &[
                "padding-top",
                "padding-right",
                "padding-bottom",
                "padding-left",
            ],
        ),
        "px" => ("padding", &["padding-left", "padding-right"]),
        "py" => ("padding", &["padding-top", "padding-bottom"]),
        "pt" => ("padding", &["padding-top"]),
        "pr" => ("padding", &["padding-right"]),
        "pb" => ("padding", &["padding-bottom"]),
        "pl" => ("padding", &["padding-left"]),
        "m" => (
            "margin",
            &["margin-top", "margin-right", "margin-bottom", "margin-left"],
        ),
        "mx" => ("margin", &["margin-left", "margin-right"]),
        "my" => ("margin", &["margin-top", "margin-bottom"]),
        "mt" => ("margin", &["margin-top"]),
        "mr" => ("margin", &["margin-right"]),
        "mb" => ("margin", &["margin-bottom"]),
        "ml" => ("margin", &["margin-left"]),
        "gap" => ("gap", &["row-gap", "column-gap"]),
        _ => return None,
    })
}

fn spacing_rule<'a>(input: &mut &'a str) -> ModalResult<(&'static str, &'a str), ContextError> {
    let prefix = alt((
        alt((
            "px-".value("px"),
            "py-".value("py"),
            "pt-".value("pt"),
            "pr-".value("pr"),
            "pb-".value("pb"),
            "pl-".value("pl"),
            "mx-".value("mx"),
            "my-".value("my"),
        )),
        alt((
            "mt-".value("mt"),
            "mr-".value("mr"),
            "mb-".value("mb"),
            "ml-".value("ml"),
            "gap-".value("gap"),
            "p-".value("p"),
            "m-".value("m"),
        )),
    ))
    .parse_next(input)?;
    Ok((prefix, rest.parse_next(input)?))
}

fn dimension_rule<'a>(input: &mut &'a str) -> ModalResult<(&'static str, &'a str), ContextError> {
    let prefix = alt((
        alt((
            "min-w-".value("min-w"),
            "min-h-".value("min-h"),
            "max-w-".value("max-w"),
            "max-h-".value("max-h"),
            "inset-".value("inset"),
            "right-".value("right"),
            "bottom-".value("bottom"),
            "left-".value("left"),
        )),
        alt(("top-".value("top"), "w-".value("w"), "h-".value("h"))),
    ))
    .parse_next(input)?;
    Ok((prefix, rest.parse_next(input)?))
}

fn color_rule<'a>(input: &mut &'a str) -> ModalResult<(&'static str, &'a str), ContextError> {
    let prefix = alt((
        "border-".value("border"),
        "text-".value("text"),
        "bg-".value("bg"),
    ))
    .parse_next(input)?;
    Ok((prefix, rest.parse_next(input)?))
}

fn theme_color(token: &str) -> Option<u32> {
    let (name, opacity) = token
        .split_once('/')
        .map_or((token, None), |(name, opacity)| (name, Some(opacity)));
    let rgba = COLORS
        .iter()
        .find_map(|(candidate, value)| (*candidate == name).then_some(*value))?;
    let Some(opacity) = opacity else {
        return Some(rgba);
    };
    let opacity: f32 = opacity.parse().ok()?;
    if !(0.0..=100.0).contains(&opacity) {
        return None;
    }
    Some((rgba & 0xffffff00) | (opacity * 2.55).round() as u32)
}

pub fn parse_utility(class_name: &str) -> Result<ParsedUtility, ParseError> {
    let utility = candidate_parts(class_name)?;
    let declarations = if let Some(value) = static_utilities().get(utility) {
        value.clone()
    } else if let Ok((prefix, token)) = spacing_rule.parse(utility) {
        let value = parse_length(token, true).ok_or_else(|| ParseError::InvalidValue {
            utility: class_name.into(),
            expected: "a spacing token, px, rem, or percentage",
        })?;
        edge_properties(prefix)
            .unwrap()
            .1
            .iter()
            .map(|property| length(property, value.clone()))
            .collect()
    } else if let Ok((prefix, token)) = dimension_rule.parse(utility) {
        let value = parse_length(token, false)
            .or_else(|| parse_length(token, true))
            .ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "auto, full, a spacing token, px, rem, or percentage",
            })?;
        let property = match prefix {
            "w" => "width",
            "h" => "height",
            "min-w" => "min-width",
            "min-h" => "min-height",
            "max-w" => "max-width",
            "max-h" => "max-height",
            other => other,
        };
        if prefix == "inset" {
            ["top", "right", "bottom", "left"]
                .iter()
                .map(|p| length(p, value.clone()))
                .collect()
        } else {
            vec![length(property, value)]
        }
    } else if let Ok((prefix, token)) = color_rule.parse(utility) {
        let rgba = theme_color(token).ok_or_else(|| ParseError::InvalidValue {
            utility: class_name.into(),
            expected: "a Wabou theme color",
        })?;
        vec![color(
            match prefix {
                "bg" => "background-color",
                "text" => "color",
                _ => "border-color",
            },
            rgba,
        )]
    } else if let Some(token) = utility.strip_prefix("opacity-") {
        let value: f32 = token.parse().map_err(|_| ParseError::InvalidValue {
            utility: class_name.into(),
            expected: "an opacity from 0 to 100",
        })?;
        if !(0.0..=100.0).contains(&value) {
            return Err(ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "an opacity from 0 to 100",
            });
        }
        vec![number("opacity", value / 100.0)]
    } else {
        return Err(ParseError::UnknownUtility(class_name.into()));
    };
    Ok(ParsedUtility {
        class_name: class_name.into(),
        declarations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dynamic_spacing() {
        let parsed = parse_utility("px-[13px]").unwrap();
        assert_eq!(parsed.declarations.len(), 2);
        assert_eq!(
            parsed.declarations[0].value,
            Value::Length {
                value: Length::Px { value: 13.0 }
            }
        );
    }

    #[test]
    fn static_transform_utilities_emit_typed_nested_ir() {
        let parsed = parse_utility("translate-x-4").unwrap();
        assert!(matches!(
            &parsed.declarations[0].value,
            Value::List { values }
                if matches!(&values[0], Value::Record { fields }
                    if fields.get("kind") == Some(&Value::Keyword { value: "translateX".into() }))
        ));
    }

    #[test]
    fn rejects_css_expressions() {
        assert!(matches!(
            parse_utility("w-[calc(100%-2rem)]"),
            Err(ParseError::InvalidValue { .. })
        ));
        assert!(matches!(
            parse_utility("p-[var(--space)]"),
            Err(ParseError::InvalidValue { .. })
        ));
    }

    #[test]
    fn winnow_parses_typed_arbitrary_lengths() {
        assert_eq!(
            parse_length("[-1.5rem]", false),
            Some(Length::Px { value: -24.0 })
        );
        assert_eq!(
            parse_length("[12.5%]", false),
            Some(Length::Percent { value: 0.125 })
        );
        assert_eq!(parse_length("[12px]junk", false), None);
    }

    #[test]
    fn rejects_variants_and_invalid_arbitrary_values() {
        assert!(matches!(
            parse_utility("hover:w-[theme:size]"),
            Err(ParseError::InvalidCandidate { .. })
        ));
        assert!(matches!(
            parse_utility("hover:w-[12px"),
            Err(ParseError::InvalidCandidate { .. })
        ));
    }

    #[test]
    fn manifest_is_generated_from_runtime_definitions() {
        let manifest = manifest();
        assert_eq!(manifest.spacing["3"], 12.0);
        assert_eq!(manifest.colors["slate-900"], 0x0f172aff);
        assert!(manifest.static_utilities.contains_key("flex-1"));
    }
}
