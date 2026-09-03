use super::*;

pub(crate) const SPACING: &[(&str, f32)] = &[
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
    ("22", 88.0),
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

pub(crate) const BASE_COLORS: &[(&str, u32)] = &[
    ("transparent", 0x00000000),
    ("black", 0x000000ff),
    ("white", 0xffffffff),
];

/// Predictable Tailwind-compatible color families. Every family has the same
/// 50..950 stops; keeping this as a matrix makes incomplete palettes
/// structurally impossible.
pub(crate) const COLOR_STOPS: &[u16; 11] = &[50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];

pub(crate) const COLOR_SCALES: &[(&str, [u32; 11])] = &[
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

pub(super) fn number(property: &str, value: f32) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Number { value },
    }
}

pub(super) fn length(property: &str, value: Length) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Length { value },
    }
}

pub(super) fn transform(kind: &str, value: Value) -> Declaration {
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

pub(super) fn color(property: &str, rgba: u32) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Color {
            value: Color::Literal { rgba },
        },
    }
}

fn shadow(x: f32, y: f32, std_dev: f32, spread: f32, rgba: u32) -> Value {
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
                "stdDev".into(),
                Value::Length {
                    value: Length::Px { value: std_dev },
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

fn grid_placement(property: &str, kind: &str, value: u16) -> Declaration {
    Declaration {
        property: property.into(),
        value: Value::Record {
            fields: BTreeMap::from([
                ("kind".into(), Value::Keyword { value: kind.into() }),
                (
                    "value".into(),
                    Value::Number {
                        value: value.into(),
                    },
                ),
            ]),
        },
    }
}

pub(crate) fn static_utilities() -> BTreeMap<&'static str, Vec<Declaration>> {
    let mut utilities = BTreeMap::from([
        ("block", vec![keyword("display", "block")]),
        ("flow-root", vec![keyword("display", "flow-root")]),
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
        ("cursor-auto", vec![keyword("cursor", "auto")]),
        ("cursor-default", vec![keyword("cursor", "default")]),
        ("cursor-pointer", vec![keyword("cursor", "pointer")]),
        ("cursor-text", vec![keyword("cursor", "text")]),
        ("cursor-crosshair", vec![keyword("cursor", "crosshair")]),
        ("cursor-move", vec![keyword("cursor", "move")]),
        ("cursor-not-allowed", vec![keyword("cursor", "not-allowed")]),
        ("cursor-ew-resize", vec![keyword("cursor", "ew-resize")]),
        ("cursor-ns-resize", vec![keyword("cursor", "ns-resize")]),
        (
            "outline-0",
            vec![length("outline-width", Length::Px { value: 0.0 })],
        ),
        (
            "outline",
            vec![
                length("outline-width", Length::Px { value: 1.0 }),
                keyword("outline-style", "solid"),
            ],
        ),
        (
            "outline-2",
            vec![
                length("outline-width", Length::Px { value: 2.0 }),
                keyword("outline-style", "solid"),
            ],
        ),
        (
            "outline-4",
            vec![
                length("outline-width", Length::Px { value: 4.0 }),
                keyword("outline-style", "solid"),
            ],
        ),
        (
            "outline-offset-0",
            vec![length("outline-offset", Length::Px { value: 0.0 })],
        ),
        (
            "outline-offset-1",
            vec![length("outline-offset", Length::Px { value: 1.0 })],
        ),
        (
            "outline-offset-2",
            vec![length("outline-offset", Length::Px { value: 2.0 })],
        ),
        (
            "outline-offset-4",
            vec![length("outline-offset", Length::Px { value: 4.0 })],
        ),
        ("flex-row", vec![keyword("flex-direction", "row")]),
        (
            "flex-row-reverse",
            vec![keyword("flex-direction", "row-reverse")],
        ),
        ("flex-col", vec![keyword("flex-direction", "column")]),
        (
            "flex-col-reverse",
            vec![keyword("flex-direction", "column-reverse")],
        ),
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
        (
            "content-evenly",
            vec![keyword("align-content", "space-evenly")],
        ),
        ("self-auto", vec![keyword("align-self", "auto")]),
        ("self-start", vec![keyword("align-self", "flex-start")]),
        ("self-center", vec![keyword("align-self", "center")]),
        ("self-end", vec![keyword("align-self", "flex-end")]),
        ("self-stretch", vec![keyword("align-self", "stretch")]),
        ("self-baseline", vec![keyword("align-self", "baseline")]),
        (
            "justify-items-start",
            vec![keyword("justify-items", "flex-start")],
        ),
        (
            "justify-items-center",
            vec![keyword("justify-items", "center")],
        ),
        (
            "justify-items-end",
            vec![keyword("justify-items", "flex-end")],
        ),
        (
            "justify-items-stretch",
            vec![keyword("justify-items", "stretch")],
        ),
        ("justify-self-auto", vec![keyword("justify-self", "auto")]),
        (
            "justify-self-start",
            vec![keyword("justify-self", "flex-start")],
        ),
        (
            "justify-self-center",
            vec![keyword("justify-self", "center")],
        ),
        (
            "justify-self-end",
            vec![keyword("justify-self", "flex-end")],
        ),
        (
            "justify-self-stretch",
            vec![keyword("justify-self", "stretch")],
        ),
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
        ("overflow-clip", vec![keyword("overflow", "clip")]),
        ("overflow-visible", vec![keyword("overflow", "visible")]),
        ("overflow-auto", vec![keyword("overflow", "auto")]),
        ("overflow-scroll", vec![keyword("overflow", "scroll")]),
        ("overflow-x-hidden", vec![keyword("overflow-x", "hidden")]),
        ("overflow-y-hidden", vec![keyword("overflow-y", "hidden")]),
        ("overflow-x-clip", vec![keyword("overflow-x", "clip")]),
        ("overflow-y-clip", vec![keyword("overflow-y", "clip")]),
        ("overflow-x-auto", vec![keyword("overflow-x", "auto")]),
        ("overflow-y-auto", vec![keyword("overflow-y", "auto")]),
        ("overflow-x-scroll", vec![keyword("overflow-x", "scroll")]),
        ("overflow-y-scroll", vec![keyword("overflow-y", "scroll")]),
        ("box-border", vec![keyword("box-sizing", "border-box")]),
        ("box-content", vec![keyword("box-sizing", "content-box")]),
        ("contain-none", vec![keyword("contain", "none")]),
        ("contain-layout", vec![keyword("contain", "layout")]),
        ("contain-paint", vec![keyword("contain", "paint")]),
        ("contain-content", vec![keyword("contain", "content")]),
        ("grid-flow-row", vec![keyword("grid-auto-flow", "row")]),
        ("grid-flow-col", vec![keyword("grid-auto-flow", "column")]),
        (
            "grid-flow-dense",
            vec![keyword("grid-auto-flow", "row-dense")],
        ),
        (
            "grid-flow-row-dense",
            vec![keyword("grid-auto-flow", "row-dense")],
        ),
        (
            "grid-flow-col-dense",
            vec![keyword("grid-auto-flow", "column-dense")],
        ),
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
            "rounded-l-lg",
            vec![
                length("border-top-left-radius", Length::Px { value: 8.0 }),
                length("border-bottom-left-radius", Length::Px { value: 8.0 }),
            ],
        ),
        (
            "rounded-r-lg",
            vec![
                length("border-top-right-radius", Length::Px { value: 8.0 }),
                length("border-bottom-right-radius", Length::Px { value: 8.0 }),
            ],
        ),
        (
            "rounded-r-none",
            vec![
                length("border-top-right-radius", Length::Px { value: 0.0 }),
                length("border-bottom-right-radius", Length::Px { value: 0.0 }),
            ],
        ),
        (
            "rounded-t-lg",
            vec![
                length("border-top-left-radius", Length::Px { value: 8.0 }),
                length("border-top-right-radius", Length::Px { value: 8.0 }),
            ],
        ),
        (
            "rounded-b-lg",
            vec![
                length("border-bottom-left-radius", Length::Px { value: 8.0 }),
                length("border-bottom-right-radius", Length::Px { value: 8.0 }),
            ],
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
        (
            "origin-center",
            vec![
                length("transform-origin-x", Length::Percent { value: 0.5 }),
                length("transform-origin-y", Length::Percent { value: 0.5 }),
            ],
        ),
        (
            "origin-top-left",
            vec![
                length("transform-origin-x", Length::Percent { value: 0.0 }),
                length("transform-origin-y", Length::Percent { value: 0.0 }),
            ],
        ),
        (
            "origin-top-right",
            vec![
                length("transform-origin-x", Length::Percent { value: 1.0 }),
                length("transform-origin-y", Length::Percent { value: 0.0 }),
            ],
        ),
        (
            "origin-bottom-left",
            vec![
                length("transform-origin-x", Length::Percent { value: 0.0 }),
                length("transform-origin-y", Length::Percent { value: 1.0 }),
            ],
        ),
        (
            "origin-bottom-right",
            vec![
                length("transform-origin-x", Length::Percent { value: 1.0 }),
                length("transform-origin-y", Length::Percent { value: 1.0 }),
            ],
        ),
        ("font-sans", vec![keyword("font-family", "sans-serif")]),
        ("font-mono", vec![keyword("font-family", "monospace")]),
        ("font-normal", vec![number("font-weight", 400.0)]),
        ("font-medium", vec![number("font-weight", 500.0)]),
        ("font-semibold", vec![number("font-weight", 600.0)]),
        ("font-bold", vec![number("font-weight", 700.0)]),
        ("font-extrabold", vec![number("font-weight", 800.0)]),
        ("italic", vec![keyword("font-style", "italic")]),
        ("not-italic", vec![keyword("font-style", "normal")]),
        (
            "underline",
            vec![keyword("text-decoration-line", "underline")],
        ),
        (
            "line-through",
            vec![keyword("text-decoration-line", "line-through")],
        ),
        (
            "no-underline",
            vec![keyword("text-decoration-line", "none")],
        ),
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
        (
            "text-4xl",
            vec![
                length("font-size", Length::Px { value: 36.0 }),
                length("line-height", Length::Px { value: 40.0 }),
            ],
        ),
        (
            "text-5xl",
            vec![
                length("font-size", Length::Px { value: 48.0 }),
                length("line-height", Length::Px { value: 48.0 }),
            ],
        ),
        (
            "text-6xl",
            vec![
                length("font-size", Length::Px { value: 60.0 }),
                length("line-height", Length::Px { value: 60.0 }),
            ],
        ),
        (
            "text-7xl",
            vec![
                length("font-size", Length::Px { value: 72.0 }),
                length("line-height", Length::Px { value: 72.0 }),
            ],
        ),
        (
            "text-8xl",
            vec![
                length("font-size", Length::Px { value: 96.0 }),
                length("line-height", Length::Px { value: 96.0 }),
            ],
        ),
        (
            "text-9xl",
            vec![
                length("font-size", Length::Px { value: 128.0 }),
                length("line-height", Length::Px { value: 128.0 }),
            ],
        ),
        ("leading-tight", vec![number("line-height", 1.25)]),
        ("leading-normal", vec![number("line-height", 1.5)]),
        ("leading-relaxed", vec![number("line-height", 1.625)]),
        (
            "tracking-tighter",
            vec![length("letter-spacing", Length::Px { value: -0.8 })],
        ),
        (
            "tracking-tight",
            vec![length("letter-spacing", Length::Px { value: -0.4 })],
        ),
        (
            "tracking-normal",
            vec![length("letter-spacing", Length::Px { value: 0.0 })],
        ),
        (
            "tracking-wide",
            vec![length("letter-spacing", Length::Px { value: 0.4 })],
        ),
        (
            "tracking-wider",
            vec![length("letter-spacing", Length::Px { value: 0.8 })],
        ),
        (
            "tracking-widest",
            vec![length("letter-spacing", Length::Px { value: 1.6 })],
        ),
        ("text-left", vec![keyword("text-align", "left")]),
        ("text-center", vec![keyword("text-align", "center")]),
        ("text-right", vec![keyword("text-align", "right")]),
        ("whitespace-nowrap", vec![keyword("white-space", "nowrap")]),
        ("whitespace-normal", vec![keyword("white-space", "normal")]),
        ("text-ellipsis", vec![keyword("text-overflow", "ellipsis")]),
        (
            "truncate",
            vec![
                keyword("overflow", "hidden"),
                keyword("white-space", "nowrap"),
                keyword("text-overflow", "ellipsis"),
            ],
        ),
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
        (
            "blur-none",
            vec![length("filter-blur", Length::Px { value: 0.0 })],
        ),
        (
            "blur-sm",
            vec![length("filter-blur", Length::Px { value: 4.0 })],
        ),
        (
            "blur",
            vec![length("filter-blur", Length::Px { value: 8.0 })],
        ),
        (
            "blur-md",
            vec![length("filter-blur", Length::Px { value: 12.0 })],
        ),
        (
            "blur-lg",
            vec![length("filter-blur", Length::Px { value: 16.0 })],
        ),
        (
            "blur-xl",
            vec![length("filter-blur", Length::Px { value: 24.0 })],
        ),
        (
            "backdrop-blur-none",
            vec![length("backdrop-blur", Length::Px { value: 0.0 })],
        ),
        (
            "backdrop-blur-sm",
            vec![length("backdrop-blur", Length::Px { value: 4.0 })],
        ),
        (
            "backdrop-blur",
            vec![length("backdrop-blur", Length::Px { value: 8.0 })],
        ),
        (
            "backdrop-blur-md",
            vec![length("backdrop-blur", Length::Px { value: 12.0 })],
        ),
        (
            "backdrop-blur-lg",
            vec![length("backdrop-blur", Length::Px { value: 16.0 })],
        ),
        (
            "backdrop-blur-xl",
            vec![length("backdrop-blur", Length::Px { value: 24.0 })],
        ),
        ("shadow-none", vec![shadows(vec![])]),
        (
            "shadow-xs",
            vec![shadows(vec![shadow(0.0, 1.0, 0.75, 0.0, 0x0f172a14)])],
        ),
        (
            "shadow-sm",
            vec![shadows(vec![shadow(0.0, 1.0, 1.5, 0.0, 0x0f172a1a)])],
        ),
        (
            "shadow",
            vec![shadows(vec![
                shadow(0.0, 1.0, 2.0, 0.0, 0x0f172a14),
                shadow(0.0, 2.0, 1.0, -1.0, 0x0f172a1f),
            ])],
        ),
        (
            "shadow-md",
            vec![shadows(vec![
                shadow(0.0, 3.0, 6.0, 0.0, 0x0f172a1a),
                shadow(0.0, 5.0, 2.5, -2.0, 0x0f172a24),
            ])],
        ),
        (
            "shadow-lg",
            vec![shadows(vec![
                shadow(0.0, 6.0, 10.0, 0.0, 0x0f172a1f),
                shadow(0.0, 12.0, 4.0, -4.0, 0x0f172a29),
            ])],
        ),
        (
            "shadow-xl",
            vec![shadows(vec![
                shadow(0.0, 10.0, 16.0, 0.0, 0x0f172a24),
                shadow(0.0, 20.0, 6.0, -6.0, 0x0f172a2e),
            ])],
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
            "max-w-xs",
            vec![length("max-width", Length::Px { value: 320.0 })],
        ),
        (
            "max-w-sm",
            vec![length("max-width", Length::Px { value: 384.0 })],
        ),
        (
            "max-w-md",
            vec![length("max-width", Length::Px { value: 448.0 })],
        ),
        (
            "max-w-lg",
            vec![length("max-width", Length::Px { value: 512.0 })],
        ),
        (
            "max-w-xl",
            vec![length("max-width", Length::Px { value: 576.0 })],
        ),
        (
            "max-w-2xl",
            vec![length("max-width", Length::Px { value: 672.0 })],
        ),
        (
            "max-w-3xl",
            vec![length("max-width", Length::Px { value: 768.0 })],
        ),
        (
            "max-w-4xl",
            vec![length("max-width", Length::Px { value: 896.0 })],
        ),
        (
            "max-w-5xl",
            vec![length("max-width", Length::Px { value: 1024.0 })],
        ),
        (
            "max-w-6xl",
            vec![length("max-width", Length::Px { value: 1152.0 })],
        ),
        (
            "max-w-7xl",
            vec![length("max-width", Length::Px { value: 1280.0 })],
        ),
    ]);
    const COL_START: [&str; 13] = [
        "col-start-1",
        "col-start-2",
        "col-start-3",
        "col-start-4",
        "col-start-5",
        "col-start-6",
        "col-start-7",
        "col-start-8",
        "col-start-9",
        "col-start-10",
        "col-start-11",
        "col-start-12",
        "col-start-13",
    ];
    const COL_END: [&str; 13] = [
        "col-end-1",
        "col-end-2",
        "col-end-3",
        "col-end-4",
        "col-end-5",
        "col-end-6",
        "col-end-7",
        "col-end-8",
        "col-end-9",
        "col-end-10",
        "col-end-11",
        "col-end-12",
        "col-end-13",
    ];
    const ROW_START: [&str; 13] = [
        "row-start-1",
        "row-start-2",
        "row-start-3",
        "row-start-4",
        "row-start-5",
        "row-start-6",
        "row-start-7",
        "row-start-8",
        "row-start-9",
        "row-start-10",
        "row-start-11",
        "row-start-12",
        "row-start-13",
    ];
    const ROW_END: [&str; 13] = [
        "row-end-1",
        "row-end-2",
        "row-end-3",
        "row-end-4",
        "row-end-5",
        "row-end-6",
        "row-end-7",
        "row-end-8",
        "row-end-9",
        "row-end-10",
        "row-end-11",
        "row-end-12",
        "row-end-13",
    ];
    const COL_SPAN: [&str; 12] = [
        "col-span-1",
        "col-span-2",
        "col-span-3",
        "col-span-4",
        "col-span-5",
        "col-span-6",
        "col-span-7",
        "col-span-8",
        "col-span-9",
        "col-span-10",
        "col-span-11",
        "col-span-12",
    ];
    const ROW_SPAN: [&str; 12] = [
        "row-span-1",
        "row-span-2",
        "row-span-3",
        "row-span-4",
        "row-span-5",
        "row-span-6",
        "row-span-7",
        "row-span-8",
        "row-span-9",
        "row-span-10",
        "row-span-11",
        "row-span-12",
    ];
    for index in 0..13 {
        let value = (index + 1) as u16;
        let col_start = COL_START[index];
        let col_end = COL_END[index];
        let row_start = ROW_START[index];
        let row_end = ROW_END[index];
        utilities.insert(
            col_start,
            vec![grid_placement("grid-column-start", "line", value)],
        );
        utilities.insert(
            col_end,
            vec![grid_placement("grid-column-end", "line", value)],
        );
        utilities.insert(
            row_start,
            vec![grid_placement("grid-row-start", "line", value)],
        );
        utilities.insert(row_end, vec![grid_placement("grid-row-end", "line", value)]);
        if index < 12 {
            utilities.insert(
                COL_SPAN[index],
                vec![grid_placement("grid-column-start", "span", value)],
            );
            utilities.insert(
                ROW_SPAN[index],
                vec![grid_placement("grid-row-start", "span", value)],
            );
        }
    }
    utilities
}
