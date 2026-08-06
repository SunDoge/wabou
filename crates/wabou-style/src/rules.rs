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

pub(super) const SPACING: &[(&str, f32)] = &[
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

pub(super) const BASE_COLORS: &[(&str, u32)] = &[
    ("transparent", 0x00000000),
    ("black", 0x000000ff),
    ("white", 0xffffffff),
];

/// Predictable Tailwind-compatible color families. Every family has the same
/// 50..950 stops; keeping this as a matrix makes incomplete palettes
/// structurally impossible.
pub(super) const COLOR_STOPS: &[u16; 11] = &[50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

pub(super) const COLOR_SCALES: &[(&str, [u32; 11])] = &[
    (
        "rose",
        [
            0xfff1f2ff, 0xffe4e6ff, 0xfecdd3ff, 0xfda4afff, 0xfb7185ff, 0xf43f5eff, 0xe11d48ff,
            0xbe123cff, 0x9f1239ff, 0x881337ff, 0x4c0519ff,
        ],
    ),
    (
        "pink",
        [
            0xfdf2f8ff, 0xfce7f3ff, 0xfbcfe8ff, 0xf9a8d4ff, 0xf472b6ff, 0xec4899ff, 0xdb2777ff,
            0xbe185dff, 0x9d174dff, 0x831843ff, 0x500724ff,
        ],
    ),
    (
        "fuchsia",
        [
            0xfdf4ffff, 0xfae8ffff, 0xf5d0feff, 0xf0abfcff, 0xe879f9ff, 0xd946efff, 0xc026d3ff,
            0xa21cafff, 0x86198fff, 0x701a75ff, 0x4a044eff,
        ],
    ),
    (
        "purple",
        [
            0xfaf5ffff, 0xf3e8ffff, 0xe9d5ffff, 0xd8b4feff, 0xc084fcff, 0xa855f7ff, 0x9333eaff,
            0x7e22ceff, 0x6b21a8ff, 0x581c87ff, 0x3b0764ff,
        ],
    ),
    (
        "violet",
        [
            0xf5f3ffff, 0xede9feff, 0xddd6feff, 0xc4b5fdff, 0xa78bfaff, 0x8b5cf6ff, 0x7c3aedff,
            0x6d28d9ff, 0x5b21b6ff, 0x4c1d95ff, 0x2e1065ff,
        ],
    ),
    (
        "indigo",
        [
            0xeef2ffff, 0xe0e7ffff, 0xc7d2feff, 0xa5b4fcff, 0x818cf8ff, 0x6366f1ff, 0x4f46e5ff,
            0x4338caff, 0x3730a3ff, 0x312e81ff, 0x1e1b4bff,
        ],
    ),
    (
        "blue",
        [
            0xeff6ffff, 0xdbeafeff, 0xbfdbfeff, 0x93c5fdff, 0x60a5faff, 0x3b82f6ff, 0x2563ebff,
            0x1d4ed8ff, 0x1e40afff, 0x1e3a8aff, 0x172554ff,
        ],
    ),
    (
        "sky",
        [
            0xf0f9ffff, 0xe0f2feff, 0xbae6fdff, 0x7dd3fcff, 0x38bdf8ff, 0x0ea5e9ff, 0x0284c7ff,
            0x0369a1ff, 0x075985ff, 0x0c4a6eff, 0x082f49ff,
        ],
    ),
    (
        "cyan",
        [
            0xecfeffff, 0xcffafeff, 0xa5f3fcff, 0x67e8f9ff, 0x22d3eeff, 0x06b6d4ff, 0x0891b2ff,
            0x0e7490ff, 0x155e75ff, 0x164e63ff, 0x083344ff,
        ],
    ),
    (
        "teal",
        [
            0xf0fdfaff, 0xccfbf1ff, 0x99f6e4ff, 0x5eead4ff, 0x2dd4bfff, 0x14b8a6ff, 0x0d9488ff,
            0x0f766eff, 0x115e59ff, 0x134e4aff, 0x042f2eff,
        ],
    ),
    (
        "emerald",
        [
            0xecfdf5ff, 0xd1fae5ff, 0xa7f3d0ff, 0x6ee7b7ff, 0x34d399ff, 0x10b981ff, 0x059669ff,
            0x047857ff, 0x065f46ff, 0x064e3bff, 0x022c22ff,
        ],
    ),
    (
        "green",
        [
            0xf0fdf4ff, 0xdcfce7ff, 0xbbf7d0ff, 0x86efacff, 0x4ade80ff, 0x22c55eff, 0x16a34aff,
            0x15803dff, 0x166534ff, 0x14532dff, 0x052e16ff,
        ],
    ),
    (
        "lime",
        [
            0xf7fee7ff, 0xecfccbff, 0xd9f99dff, 0xbef264ff, 0xa3e635ff, 0x84cc16ff, 0x65a30dff,
            0x4d7c0fff, 0x3f6212ff, 0x365314ff, 0x1a2e05ff,
        ],
    ),
    (
        "yellow",
        [
            0xfefce8ff, 0xfef9c3ff, 0xfef08aff, 0xfde047ff, 0xfacc15ff, 0xeab308ff, 0xca8a04ff,
            0xa16207ff, 0x854d0eff, 0x713f12ff, 0x422006ff,
        ],
    ),
    (
        "amber",
        [
            0xfffbebff, 0xfef3c7ff, 0xfde68aff, 0xfcd34dff, 0xfbbf24ff, 0xf59e0bff, 0xd97706ff,
            0xb45309ff, 0x92400eff, 0x78350fff, 0x451a03ff,
        ],
    ),
    (
        "orange",
        [
            0xfff7edff, 0xffedd5ff, 0xfed7aaff, 0xfdba74ff, 0xfb923cff, 0xf97316ff, 0xea580cff,
            0xc2410cff, 0x9a3412ff, 0x7c2d12ff, 0x431407ff,
        ],
    ),
    (
        "red",
        [
            0xfef2f2ff, 0xfee2e2ff, 0xfecacaff, 0xfca5a5ff, 0xf87171ff, 0xef4444ff, 0xdc2626ff,
            0xb91c1cff, 0x991b1bff, 0x7f1d1dff, 0x450a0aff,
        ],
    ),
    (
        "gray",
        [
            0xf9fafbff, 0xf3f4f6ff, 0xe5e7ebff, 0xd1d5dbff, 0x9ca3afff, 0x6b7280ff, 0x4b5563ff,
            0x374151ff, 0x1f2937ff, 0x111827ff, 0x030712ff,
        ],
    ),
    (
        "slate",
        [
            0xf8fafcff, 0xf1f5f9ff, 0xe2e8f0ff, 0xcbd5e1ff, 0x94a3b8ff, 0x64748bff, 0x475569ff,
            0x334155ff, 0x1e293bff, 0x0f172aff, 0x020617ff,
        ],
    ),
    (
        "zinc",
        [
            0xfafafaff, 0xf4f4f5ff, 0xe4e4e7ff, 0xd4d4d8ff, 0xa1a1aaff, 0x71717aff, 0x52525bff,
            0x3f3f46ff, 0x27272aff, 0x18181bff, 0x09090bff,
        ],
    ),
    (
        "neutral",
        [
            0xfafafaff, 0xf5f5f5ff, 0xe5e5e5ff, 0xd4d4d4ff, 0xa3a3a3ff, 0x737373ff, 0x525252ff,
            0x404040ff, 0x262626ff, 0x171717ff, 0x0a0a0aff,
        ],
    ),
    (
        "stone",
        [
            0xfafaf9ff, 0xf5f5f4ff, 0xe7e5e4ff, 0xd6d3d1ff, 0xa8a29eff, 0x78716cff, 0x57534eff,
            0x44403cff, 0x292524ff, 0x1c1917ff, 0x0c0a09ff,
        ],
    ),
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
        // Utility transforms compose in class-list order. Inline `transform`
        // remains a replacing declaration at the host boundary.
        property: match kind {
            "translateX" => "transform-translate-x",
            "translateY" => "transform-translate-y",
            "scale" => "transform-scale",
            "rotate" => "transform-rotate",
            _ => "transform-component",
        }
        .to_string(),
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

fn shadow(x: f32, y: f32, blur: f32, spread: f32, rgba: u32) -> Value {
    Value::Record {
        fields: BTreeMap::from([
            (
                "x".into(),
                Value::Length {
                    value: Length::Px { value: x },
                },
            ),
            (
                "y".into(),
                Value::Length {
                    value: Length::Px { value: y },
                },
            ),
            (
                "blur".into(),
                Value::Length {
                    value: Length::Px { value: blur },
                },
            ),
            (
                "spread".into(),
                Value::Length {
                    value: Length::Px { value: spread },
                },
            ),
            (
                "color".into(),
                Value::Color {
                    value: Color::Literal { rgba },
                },
            ),
            ("inset".into(), Value::Boolean { value: false }),
        ]),
    }
}

fn shadows(values: Vec<Value>) -> Declaration {
    Declaration {
        property: "box-shadow".into(),
        value: Value::List { values },
    }
}

fn grid_repeat(property: &str, count: u8) -> Declaration {
    let track = Value::Record {
        fields: BTreeMap::from([
            (
                "kind".into(),
                Value::Keyword {
                    value: "breadth".into(),
                },
            ),
            (
                "value".into(),
                Value::Record {
                    fields: BTreeMap::from([
                        (
                            "kind".into(),
                            Value::Keyword {
                                value: "flex".into(),
                            },
                        ),
                        ("value".into(), Value::Number { value: 1.0 }),
                    ]),
                },
            ),
        ]),
    };
    Declaration {
        property: property.into(),
        value: Value::List {
            values: vec![Value::Record {
                fields: BTreeMap::from([
                    (
                        "kind".into(),
                        Value::Keyword {
                            value: "repeat".into(),
                        },
                    ),
                    (
                        "count".into(),
                        Value::Number {
                            value: count.into(),
                        },
                    ),
                    (
                        "values".into(),
                        Value::List {
                            values: vec![track],
                        },
                    ),
                ]),
            }],
        },
    }
}

pub(super) fn static_utilities() -> BTreeMap<&'static str, Vec<Declaration>> {
    BTreeMap::from([
        ("block", vec![keyword("display", "block")]),
        ("flex", vec![keyword("display", "flex")]),
        ("inline-flex", vec![keyword("display", "flex")]),
        ("grid", vec![keyword("display", "grid")]),
        ("grid-cols-1", vec![grid_repeat("grid-template-columns", 1)]),
        ("grid-cols-2", vec![grid_repeat("grid-template-columns", 2)]),
        ("grid-cols-3", vec![grid_repeat("grid-template-columns", 3)]),
        ("grid-cols-4", vec![grid_repeat("grid-template-columns", 4)]),
        ("grid-cols-6", vec![grid_repeat("grid-template-columns", 6)]),
        (
            "grid-cols-12",
            vec![grid_repeat("grid-template-columns", 12)],
        ),
        ("grid-rows-1", vec![grid_repeat("grid-template-rows", 1)]),
        ("grid-rows-2", vec![grid_repeat("grid-template-rows", 2)]),
        ("grid-rows-3", vec![grid_repeat("grid-template-rows", 3)]),
        ("grid-rows-4", vec![grid_repeat("grid-template-rows", 4)]),
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
        ("items-baseline", vec![keyword("align-items", "baseline")]),
        (
            "content-start",
            vec![keyword("align-content", "flex-start")],
        ),
        ("content-center", vec![keyword("align-content", "center")]),
        ("content-end", vec![keyword("align-content", "flex-end")]),
        (
            "content-between",
            vec![keyword("align-content", "space-between")],
        ),
        (
            "content-around",
            vec![keyword("align-content", "space-around")],
        ),
        ("self-auto", vec![keyword("align-self", "auto")]),
        ("self-start", vec![keyword("align-self", "flex-start")]),
        ("self-center", vec![keyword("align-self", "center")]),
        ("self-end", vec![keyword("align-self", "flex-end")]),
        ("self-stretch", vec![keyword("align-self", "stretch")]),
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
        (
            "justify-evenly",
            vec![keyword("justify-content", "space-evenly")],
        ),
        ("relative", vec![keyword("position", "relative")]),
        ("absolute", vec![keyword("position", "absolute")]),
        ("aspect-square", vec![number("aspect-ratio", 1.0)]),
        ("aspect-video", vec![number("aspect-ratio", 16.0 / 9.0)]),
        ("z-0", vec![number("z-index", 0.0)]),
        ("z-10", vec![number("z-index", 10.0)]),
        ("z-20", vec![number("z-index", 20.0)]),
        ("z-30", vec![number("z-index", 30.0)]),
        ("z-40", vec![number("z-index", 40.0)]),
        ("z-50", vec![number("z-index", 50.0)]),
        ("overflow-hidden", vec![keyword("overflow", "hidden")]),
        ("overflow-visible", vec![keyword("overflow", "visible")]),
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
            "border-4",
            vec![length("border-width", Length::Px { value: 4.0 })],
        ),
        (
            "border-8",
            vec![length("border-width", Length::Px { value: 8.0 })],
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
            "rounded-none",
            vec![length("border-radius", Length::Px { value: 0.0 })],
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
            "rounded-2xl",
            vec![length("border-radius", Length::Px { value: 16.0 })],
        ),
        (
            "rounded-3xl",
            vec![length("border-radius", Length::Px { value: 24.0 })],
        ),
        (
            "rounded-full",
            vec![length("border-radius", Length::Px { value: 9999.0 })],
        ),
        ("font-sans", vec![keyword("font-family", "sans-serif")]),
        ("font-mono", vec![keyword("font-family", "monospace")]),
        ("font-normal", vec![number("font-weight", 400.0)]),
        ("font-medium", vec![number("font-weight", 500.0)]),
        ("font-semibold", vec![number("font-weight", 600.0)]),
        ("font-bold", vec![number("font-weight", 700.0)]),
        ("font-extrabold", vec![number("font-weight", 800.0)]),
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
        ("leading-normal", vec![number("line-height", 1.5)]),
        ("leading-relaxed", vec![number("line-height", 1.625)]),
        ("text-left", vec![keyword("text-align", "left")]),
        ("text-center", vec![keyword("text-align", "center")]),
        ("text-right", vec![keyword("text-align", "right")]),
        ("whitespace-nowrap", vec![keyword("white-space", "nowrap")]),
        ("whitespace-normal", vec![keyword("white-space", "normal")]),
        (
            "pointer-events-none",
            vec![keyword("pointer-events", "none")],
        ),
        (
            "pointer-events-auto",
            vec![keyword("pointer-events", "auto")],
        ),
        ("select-none", vec![keyword("user-select", "none")]),
        ("select-text", vec![keyword("user-select", "text")]),
        ("select-all", vec![keyword("user-select", "all")]),
        ("shadow-none", vec![shadows(vec![])]),
        (
            "shadow-sm",
            vec![shadows(vec![shadow(0.0, 1.0, 2.0, 0.0, 0x0000000d)])],
        ),
        (
            "shadow",
            vec![shadows(vec![shadow(0.0, 1.0, 3.0, 0.0, 0x0000001a)])],
        ),
        (
            "shadow-md",
            vec![shadows(vec![shadow(0.0, 4.0, 6.0, -1.0, 0x0000001a)])],
        ),
        (
            "shadow-lg",
            vec![shadows(vec![shadow(0.0, 10.0, 15.0, -3.0, 0x0000001a)])],
        ),
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

pub fn parse_utility(class_name: &str) -> Result<ParsedUtility, ParseError> {
    parse_utility_with_theme(class_name, default_theme())
}

pub fn parse_utility_with_theme(
    class_name: &str,
    theme: &Theme,
) -> Result<ParsedUtility, ParseError> {
    let utility = candidate_parts(class_name)?;
    let declarations = if let Some(value) = static_utilities().get(utility) {
        value.clone()
    } else if let Some((prefix, token, negative)) = spacing_rule
        .parse(utility)
        .ok()
        .map(|(prefix, token)| (prefix, token, false))
        .or_else(|| {
            utility
                .strip_prefix('-')
                .and_then(|value| spacing_rule.parse(value).ok())
                .map(|(prefix, token)| (prefix, token, true))
        })
    {
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
        edge_properties(prefix)
            .unwrap()
            .1
            .iter()
            .map(|property| length(property, value.clone()))
            .collect()
    } else if let Some((prefix, token, negative)) = dimension_rule
        .parse(utility)
        .ok()
        .map(|(prefix, token)| (prefix, token, false))
        .or_else(|| {
            utility
                .strip_prefix('-')
                .and_then(|value| dimension_rule.parse(value).ok())
                .map(|(prefix, token)| (prefix, token, true))
        })
    {
        if negative && !matches!(prefix, "top" | "right" | "bottom" | "left" | "inset") {
            return Err(ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "negative values are only valid for inset and positioned edges",
            });
        }
        let mut value = parse_length(token, false, theme)
            .or_else(|| parse_length(token, true, theme))
            .ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "auto, full, a spacing token, px, rem, or percentage",
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
        if prefix == "inset" {
            ["top", "right", "bottom", "left"]
                .iter()
                .map(|p| length(p, value.clone()))
                .collect()
        } else {
            vec![length(property, value)]
        }
    } else if let Some((kind, token, negative)) = [
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
        let mut value =
            parse_length(token, true, theme).ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "a translate spacing token, px, rem, or percentage",
            })?;
        if negative {
            value = negate_length(value);
        }
        vec![transform(kind, Value::Length { value })]
    } else if let Some(token) = utility.strip_prefix("scale-") {
        let value = arbitrary_number(token)
            .or_else(|| token.parse::<f32>().ok().map(|value| value / 100.0))
            .filter(|value| value.is_finite())
            .ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "a finite scale percentage or arbitrary number",
            })?;
        vec![transform(
            "scale",
            Value::List {
                values: vec![Value::Number { value }, Value::Number { value }],
            },
        )]
    } else if let Some((token, negative)) = utility
        .strip_prefix("rotate-")
        .map(|token| (token, false))
        .or_else(|| utility.strip_prefix("-rotate-").map(|token| (token, true)))
    {
        let mut degrees = arbitrary_number(token)
            .or_else(|| token.parse().ok())
            .ok_or_else(|| ParseError::InvalidValue {
                utility: class_name.into(),
                expected: "a finite rotation in degrees",
            })?;
        if negative {
            degrees = -degrees;
        }
        vec![transform(
            "rotate",
            Value::Number {
                value: degrees * std::f32::consts::PI / 180.0,
            },
        )]
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
mod tests {
    use super::*;
    use crate::manifest;

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
            parse_length("[-1.5rem]", false, default_theme()),
            Some(Length::Px { value: -24.0 })
        );
        assert_eq!(
            parse_length("[12.5%]", false, default_theme()),
            Some(Length::Percent { value: 0.125 })
        );
        assert_eq!(parse_length("[12px]junk", false, default_theme()), None);
    }

    #[test]
    fn rejects_variants_and_invalid_arbitrary_values() {
        for candidate in [
            "hover:bg-slate-900",
            "focus:w-4",
            "active:scale-150",
            "disabled:opacity-50",
            "sm:flex",
            "dark:bg-black",
        ] {
            assert!(matches!(
                parse_utility(candidate),
                Err(ParseError::InvalidCandidate { .. })
            ));
        }
        assert!(matches!(
            parse_utility("hover:w-[12px"),
            Err(ParseError::InvalidCandidate { .. })
        ));
        assert!(matches!(
            parse_utility("transition"),
            Err(ParseError::UnknownUtility(_))
        ));
        assert!(matches!(
            parse_utility("animate-spin"),
            Err(ParseError::UnknownUtility(_))
        ));
    }

    #[test]
    fn manifest_is_generated_from_runtime_definitions() {
        let manifest = manifest();
        assert_eq!(manifest.spacing["3"], 12.0);
        assert_eq!(manifest.colors["slate-900"], 0x0f172aff);
        assert!(manifest.static_utilities.contains_key("flex-1"));
    }

    #[test]
    fn custom_theme_extends_typed_spacing_and_colors() {
        let mut theme = Theme::default();
        theme.spacing.insert("18.5".into(), 74.0);
        theme.colors.insert("brand".into(), 0x336699ff);
        assert_eq!(
            parse_utility_with_theme("mt-18.5", &theme)
                .unwrap()
                .declarations[0]
                .value,
            Value::Length {
                value: Length::Px { value: 74.0 }
            }
        );
        assert_eq!(
            parse_utility_with_theme("bg-brand", &theme)
                .unwrap()
                .declarations[0]
                .value,
            Value::Color {
                value: Color::Literal { rgba: 0x336699ff }
            }
        );
    }

    #[test]
    fn supports_typed_stateless_arbitrary_values_and_restricted_negatives() {
        assert!(parse_utility("-mt-4").is_ok());
        assert!(parse_utility("-inset-[5%]").is_ok());
        assert!(parse_utility("bg-[#336699cc]").is_ok());
        assert!(parse_utility("aspect-[16/9]").is_ok());
        assert!(parse_utility("z-[-2]").is_ok());
        assert!(matches!(
            parse_utility("-p-4"),
            Err(ParseError::InvalidValue { .. })
        ));
        assert!(matches!(
            parse_utility("p-auto"),
            Err(ParseError::InvalidValue { .. })
        ));
        assert!(matches!(
            parse_utility("-w-4"),
            Err(ParseError::InvalidValue { .. })
        ));
        assert!(matches!(
            parse_utility("aspect-[1/0]"),
            Err(ParseError::InvalidValue { .. })
        ));
    }
}
