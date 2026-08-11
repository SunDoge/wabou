use crate::model::{DynamicPrefix, DynamicRule, Manifest, Theme};
use crate::rules::{parse_utility_with_theme, static_utilities};
use crate::theme::default_theme;

pub const MANIFEST_VERSION: u16 = 5;

pub fn manifest() -> Manifest {
    manifest_with_theme(default_theme())
}

pub fn manifest_with_theme(theme: &Theme) -> Manifest {
    Manifest {
        version: MANIFEST_VERSION,
        spacing: theme.spacing.clone(),
        colors: theme.colors.clone(),
        static_utilities: static_utilities(),
        dynamic_rules: dynamic_rules(),
        conformance: conformance_utilities(theme),
    }
}

fn dynamic_rules() -> Vec<DynamicRule> {
    vec![
        DynamicRule {
            resolver: "spacing",
            prefixes: vec![
                prefix(
                    "p",
                    &[
                        "padding-top",
                        "padding-right",
                        "padding-bottom",
                        "padding-left",
                    ],
                ),
                prefix("px", &["padding-left", "padding-right"]),
                prefix("py", &["padding-top", "padding-bottom"]),
                prefix("pt", &["padding-top"]),
                prefix("pr", &["padding-right"]),
                prefix("pb", &["padding-bottom"]),
                prefix("pl", &["padding-left"]),
                prefix(
                    "m",
                    &["margin-top", "margin-right", "margin-bottom", "margin-left"],
                ),
                prefix("mx", &["margin-left", "margin-right"]),
                prefix("my", &["margin-top", "margin-bottom"]),
                prefix("mt", &["margin-top"]),
                prefix("mr", &["margin-right"]),
                prefix("mb", &["margin-bottom"]),
                prefix("ml", &["margin-left"]),
                prefix("ms", &["margin-inline-start"]),
                prefix("me", &["margin-inline-end"]),
                prefix("gap", &["row-gap", "column-gap"]),
                prefix("gap-x", &["column-gap"]),
                prefix("gap-y", &["row-gap"]),
            ],
        },
        DynamicRule {
            resolver: "dimension",
            prefixes: vec![
                prefix("w", &["width"]),
                prefix("h", &["height"]),
                prefix("min-w", &["min-width"]),
                prefix("min-h", &["min-height"]),
                prefix("max-w", &["max-width"]),
                prefix("max-h", &["max-height"]),
                prefix("top", &["top"]),
                prefix("right", &["right"]),
                prefix("bottom", &["bottom"]),
                prefix("left", &["left"]),
                prefix("inset", &["top", "right", "bottom", "left"]),
            ],
        },
        DynamicRule {
            resolver: "color",
            prefixes: vec![
                prefix("bg", &["background-color"]),
                prefix("text", &["color"]),
                prefix("border", &["border-color"]),
            ],
        },
        DynamicRule {
            resolver: "length",
            prefixes: vec![
                prefix("rounded", &["border-radius"]),
                prefix("text", &["font-size"]),
                prefix("border", &["border-width"]),
            ],
        },
        DynamicRule {
            resolver: "opacity",
            prefixes: vec![prefix("opacity", &["opacity"])],
        },
        DynamicRule {
            resolver: "number",
            prefixes: vec![prefix("z", &["z-index"])],
        },
        DynamicRule {
            resolver: "ratio",
            prefixes: vec![prefix("aspect", &["aspect-ratio"])],
        },
        DynamicRule {
            resolver: "translate",
            prefixes: vec![
                prefix("translate-x", &["transform-translate-x"]),
                prefix("translate-y", &["transform-translate-y"]),
            ],
        },
        DynamicRule {
            resolver: "scale",
            prefixes: vec![prefix("scale", &["transform-scale"])],
        },
        DynamicRule {
            resolver: "rotate",
            prefixes: vec![prefix("rotate", &["transform-rotate"])],
        },
    ]
}

fn conformance_utilities(theme: &Theme) -> Vec<crate::model::ParsedUtility> {
    [
        "flex",
        "flex-1",
        "px-3",
        "px-[13px]",
        "gap-x-4",
        "w-full",
        "w-2/3",
        "max-w-md",
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
        "z-[42]",
        "aspect-[16/9]",
        "-mt-4",
        "mx-auto",
        "-inset-[5%]",
        "bg-[#336699cc]",
        "rounded-[10px]",
        "text-[18px]",
        "border-[3px]",
        "translate-y-6",
        "-translate-x-2",
        "scale-125",
        "rotate-30",
        "-rotate-30",
        "grid-cols-3",
        "shadow-md",
    ]
    .into_iter()
    .map(|candidate| {
        parse_utility_with_theme(candidate, theme)
            .expect("manifest conformance utility must parse with the selected theme")
    })
    .collect()
}

fn prefix(name: &'static str, properties: &[&'static str]) -> DynamicPrefix {
    DynamicPrefix {
        name,
        properties: properties.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_describes_every_dynamic_prefix_expansion() {
        let manifest = manifest();
        assert_eq!(manifest.version, MANIFEST_VERSION);
        assert!(manifest.dynamic_rules.iter().all(|rule| {
            !rule.prefixes.is_empty()
                && rule
                    .prefixes
                    .iter()
                    .all(|prefix| !prefix.properties.is_empty())
        }));
        for (class_name, declarations) in &manifest.static_utilities {
            assert_eq!(
                parse_utility_with_theme(class_name, default_theme())
                    .expect("every static manifest utility must parse")
                    .declarations,
                *declarations
            );
        }
        for rule in &manifest.dynamic_rules {
            let token = match rule.resolver {
                "spacing" | "dimension" | "translate" => "4",
                "color" => "slate-900",
                "opacity" => "50",
                "number" => "[2]",
                "ratio" => "[16/9]",
                "length" => "[2px]",
                "scale" => "125",
                "rotate" => "30",
                resolver => panic!("missing conformance sample for resolver {resolver}"),
            };
            for prefix in &rule.prefixes {
                let candidate = format!("{}-{token}", prefix.name);
                let parsed = parse_utility_with_theme(&candidate, default_theme())
                    .unwrap_or_else(|error| panic!("manifest candidate {candidate}: {error}"));
                assert_eq!(
                    parsed
                        .declarations
                        .iter()
                        .map(|declaration| declaration.property.as_str())
                        .collect::<Vec<_>>(),
                    prefix.properties,
                    "manifest property expansion drifted for {candidate}"
                );
            }
        }
    }

    #[test]
    fn custom_theme_is_exported_and_used_by_conformance_parser() {
        let mut theme = Theme::default();
        theme.spacing.insert("18.5".into(), 74.0);
        theme.colors.insert("brand".into(), 0x336699ff);
        let manifest = manifest_with_theme(&theme);
        assert_eq!(manifest.spacing["18.5"], 74.0);
        assert_eq!(manifest.colors["brand"], 0x336699ff);
    }
}
