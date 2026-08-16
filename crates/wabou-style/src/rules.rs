//! Utility-class parsing rules and manifest generation.

use std::collections::BTreeMap;

use winnow::ascii::float;
use winnow::combinator::{alt, delimited, repeat, terminated};
use winnow::error::ContextError;
use winnow::token::{rest, take_till, take_while};
use winnow::{ModalResult, Parser};

use crate::model::{Color, Declaration, Length, ParsedUtility, Theme, Value};
use crate::theme::default_theme;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Diagnostic returned when a utility candidate is unsupported or malformed.
pub enum ParseError {
    /// No static or dynamic rule recognizes the candidate.
    UnknownUtility(String),
    /// A recognized rule received a value outside its accepted grammar.
    InvalidValue {
        /// Original utility candidate.
        utility: String,
        /// Human-readable description of accepted values.
        expected: &'static str,
    },
    /// The candidate itself violates Wabou's utility syntax.
    InvalidCandidate {
        /// Original utility candidate.
        utility: String,
        /// Human-readable rejection reason.
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

mod definitions;
pub(crate) use definitions::{BASE_COLORS, COLOR_SCALES, COLOR_STOPS, SPACING, static_utilities};
use definitions::{color, length, number, transform};

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

fn parse_length(token: &str, spacing: bool, theme: &Theme) -> Option<Length> {
    if token == "auto" {
        return Some(Length::Auto);
    }
    if token == "full" {
        return Some(Length::Percent { value: 1.0 });
    }
    if token == "px" && spacing {
        return Some(Length::Px { value: 1.0 });
    }
    if spacing && let Some(value) = theme.spacing.get(token) {
        return Some(Length::Px { value: *value });
    }
    arbitrary_length
        .parse(token)
        .or_else(|_| bare_percent.parse(token))
        .ok()
}

fn dimension_fraction(token: &str) -> Option<Length> {
    let (numerator, denominator) = token.split_once('/')?;
    let numerator: u32 = numerator.parse().ok()?;
    let denominator: u32 = denominator.parse().ok()?;
    if denominator == 0 {
        return None;
    }
    Some(Length::Percent {
        value: numerator as f32 / denominator as f32,
    })
}

fn parse_dimension_length(token: &str, theme: &Theme) -> Option<Length> {
    parse_length(token, false, theme)
        .or_else(|| dimension_fraction(token))
        .or_else(|| parse_length(token, true, theme))
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
        "ms" => ("margin", &["margin-inline-start"]),
        "me" => ("margin", &["margin-inline-end"]),
        "gap" => ("gap", &["row-gap", "column-gap"]),
        "gap-x" => ("gap", &["column-gap"]),
        "gap-y" => ("gap", &["row-gap"]),
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
            alt((
                "gap-x-".value("gap-x"),
                "gap-y-".value("gap-y"),
                "mt-".value("mt"),
                "mr-".value("mr"),
                "mb-".value("mb"),
                "ml-".value("ml"),
                "ms-".value("ms"),
                "me-".value("me"),
            )),
            alt(("gap-".value("gap"), "p-".value("p"), "m-".value("m"))),
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

fn theme_color(token: &str, theme: &Theme) -> Option<u32> {
    let (name, opacity) = token
        .split_once('/')
        .map_or((token, None), |(name, opacity)| (name, Some(opacity)));
    let rgba = theme
        .colors
        .get(name)
        .copied()
        .or_else(|| arbitrary_color(name))?;
    let Some(opacity) = opacity else {
        return Some(rgba);
    };
    let opacity: f32 = opacity.parse().ok()?;
    if !(0.0..=100.0).contains(&opacity) {
        return None;
    }
    Some((rgba & 0xffffff00) | (opacity * 2.55).round() as u32)
}

fn arbitrary_color(token: &str) -> Option<u32> {
    let hex = token.strip_prefix("[#")?.strip_suffix(']')?;
    match hex.len() {
        6 => u32::from_str_radix(hex, 16)
            .ok()
            .map(|rgb| (rgb << 8) | 0xff),
        8 => u32::from_str_radix(hex, 16).ok(),
        _ => None,
    }
}

fn arbitrary_number(token: &str) -> Option<f32> {
    let token = token
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))?;
    token.parse().ok().filter(|value: &f32| value.is_finite())
}

fn arbitrary_ratio(token: &str) -> Option<f32> {
    let token = token
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))?;
    let ratio = if let Some((numerator, denominator)) = token.split_once('/') {
        let numerator: f32 = numerator.parse().ok()?;
        let denominator: f32 = denominator.parse().ok()?;
        (denominator != 0.0).then_some(numerator / denominator)?
    } else {
        token.parse().ok()?
    };
    ratio.is_finite().then_some(ratio)
}

fn negate_length(value: Length) -> Length {
    match value {
        Length::Px { value } => Length::Px { value: -value },
        Length::Percent { value } => Length::Percent { value: -value },
        Length::Auto => Length::Auto,
    }
}

type RuleResult = Result<Vec<Declaration>, ParseError>;

fn parse_spacing_utility(utility: &str, class_name: &str, theme: &Theme) -> Option<RuleResult> {
    let (prefix, token, negative) = spacing_rule
        .parse(utility)
        .ok()
        .map(|(prefix, token)| (prefix, token, false))
        .or_else(|| {
            utility
                .strip_prefix('-')
                .and_then(|value| spacing_rule.parse(value).ok())
                .map(|(prefix, token)| (prefix, token, true))
        })?;
    Some((|| {
        if negative && !prefix.starts_with('m') {
            return Err(ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "negative values are only valid for margin utilities",
            });
        }
        let mut value =
            parse_length(token, true, theme).ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "a spacing token, px, rem, or percentage",
            })?;
        if value == Length::Auto && !prefix.starts_with('m') {
            return Err(ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "auto is only valid for margin utilities",
            });
        }
        if negative {
            if value == Length::Auto {
                return Err(ParseError::InvalidValue {
                    utility: class_name.into(),
                    expected: "a numeric negative margin",
                });
            }
            value = negate_length(value);
        }
        Ok(edge_properties(prefix)
            .expect("spacing parser only accepts known edge prefixes")
            .1
            .iter()
            .map(|property| length(property, value.clone()))
            .collect())
    })())
}

fn parse_dimension_utility(utility: &str, class_name: &str, theme: &Theme) -> Option<RuleResult> {
    let (prefix, token, negative) = dimension_rule
        .parse(utility)
        .ok()
        .map(|(prefix, token)| (prefix, token, false))
        .or_else(|| {
            utility
                .strip_prefix('-')
                .and_then(|value| dimension_rule.parse(value).ok())
                .map(|(prefix, token)| (prefix, token, true))
        })?;
    Some((|| {
        if negative && !matches!(prefix, "top" | "right" | "bottom" | "left" | "inset") {
            return Err(ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "negative values are only valid for inset and positioned edges",
            });
        }
        let mut value =
            parse_dimension_length(token, theme).ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "auto, full, a fraction, a spacing token, px, rem, or percentage",
            })?;
        if negative {
            if value == Length::Auto {
                return Err(ParseError::InvalidValue {
                    utility: class_name.into(),
                    expected: "a numeric negative positioned edge",
                });
            }
            value = negate_length(value);
        }
        let property = match prefix {
            "w" => "width",
            "h" => "height",
            "min-w" => "min-width",
            "min-h" => "min-height",
            "max-w" => "max-width",
            "max-h" => "max-height",
            other => other,
        };
        Ok(if prefix == "inset" {
            ["top", "right", "bottom", "left"]
                .iter()
                .map(|property| length(property, value.clone()))
                .collect()
        } else {
            vec![length(property, value)]
        })
    })())
}

fn parse_transform_utility(utility: &str, class_name: &str, theme: &Theme) -> Option<RuleResult> {
    if let Some((kind, token, negative)) = [
        ("translate-x-", "translateX"),
        ("translate-y-", "translateY"),
    ]
    .into_iter()
    .find_map(|(prefix, kind)| {
        utility
            .strip_prefix(prefix)
            .map(|token| (kind, token, false))
            .or_else(|| {
                utility
                    .strip_prefix('-')
                    .and_then(|value| value.strip_prefix(prefix))
                    .map(|token| (kind, token, true))
            })
    }) {
        let result = parse_length(token, true, theme)
            .ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "a translate spacing token, px, rem, or percentage",
            })
            .map(|value| {
                let value = if negative {
                    negate_length(value)
                } else {
                    value
                };
                vec![transform(kind, Value::Length { value })]
            });
        return Some(result);
    }
    if let Some(token) = utility.strip_prefix("scale-") {
        return Some(
            arbitrary_number(token)
                .or_else(|| token.parse::<f32>().ok().map(|value| value / 100.0))
                .filter(|value| value.is_finite())
                .ok_or_else(|| ParseError::InvalidValue {
                    utility: class_name.into(),
                    expected: "a finite scale percentage or arbitrary number",
                })
                .map(|value| {
                    vec![transform(
                        "scale",
                        Value::List {
                            values: vec![Value::Number { value }, Value::Number { value }],
                        },
                    )]
                }),
        );
    }
    let (token, negative) = utility
        .strip_prefix("rotate-")
        .map(|token| (token, false))
        .or_else(|| utility.strip_prefix("-rotate-").map(|token| (token, true)))?;
    Some(
        arbitrary_number(token)
            .or_else(|| token.parse().ok())
            .ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "a finite rotation in degrees",
            })
            .map(|degrees: f32| {
                let radians = if negative { -degrees } else { degrees }.to_radians();
                vec![transform("rotate", Value::Number { value: radians })]
            }),
    )
}

/// Parse one utility candidate using Wabou's default theme.
pub fn parse_utility(class_name: &str) -> Result<ParsedUtility, ParseError> {
    parse_utility_with_theme(class_name, default_theme())
}

/// Parse one utility candidate using application-supplied design tokens.
pub fn parse_utility_with_theme(
    class_name: &str,
    theme: &Theme,
) -> Result<ParsedUtility, ParseError> {
    let utility = candidate_parts(class_name)?;
    let declarations = if let Some(value) = static_utilities().get(utility) {
        value.clone()
    } else if let Some(declarations) = parse_spacing_utility(utility, class_name, theme) {
        declarations?
    } else if let Some(declarations) = parse_dimension_utility(utility, class_name, theme) {
        declarations?
    } else if let Some(declarations) = parse_transform_utility(utility, class_name, theme) {
        declarations?
    } else if let Some((property, value)) = [
        ("rounded-", "border-radius"),
        ("text-", "font-size"),
        ("border-", "border-width"),
    ]
    .into_iter()
    .find_map(|(prefix, property)| {
        utility
            .strip_prefix(prefix)
            .and_then(|token| parse_length(token, false, theme))
            .map(|value| (property, value))
    }) {
        vec![length(property, value)]
    } else if let Ok((prefix, token)) = color_rule.parse(utility) {
        let rgba = theme_color(token, theme).ok_or_else(|| ParseError::InvalidValue {
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
    } else if let Some(token) = utility.strip_prefix("z-") {
        let value = arbitrary_number(token).ok_or_else(|| ParseError::InvalidValue {
            utility: class_name.into(),
            expected: "an arbitrary finite number such as z-[42]",
        })?;
        vec![number("z-index", value)]
    } else if let Some(token) = utility.strip_prefix("aspect-") {
        let value = arbitrary_ratio(token).ok_or_else(|| ParseError::InvalidValue {
            utility: class_name.into(),
            expected: "an arbitrary positive ratio such as aspect-[16/9]",
        })?;
        if value <= 0.0 {
            return Err(ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "an arbitrary positive ratio such as aspect-[16/9]",
            });
        }
        vec![number("aspect-ratio", value)]
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
mod tests;
