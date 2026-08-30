//! Versioned stylesheet IR shared by JavaScript tooling and native shells.

#![allow(missing_docs)]

use crate::{IrColor, IrLength, IrValue};
use bon::Builder;
use serde::Deserialize;

pub const VERSION: u16 = 6;

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub enum StylesheetUpdate {
    Ir(StyleSheet),
}

#[derive(Builder, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleSheet {
    #[builder(default = VERSION)]
    pub version: u16,
    #[serde(default)]
    #[builder(default)]
    pub theme: crate::Theme,
    #[serde(default, rename = "colorThemes")]
    pub color_themes: Option<ColorThemes>,
    #[serde(default)]
    #[builder(default)]
    pub diagnostics: Vec<String>,
    #[serde(default, rename = "ignoredClassPatterns")]
    #[builder(default)]
    pub ignored_class_patterns: Vec<String>,
    #[serde(default)]
    #[builder(default)]
    pub rules: Vec<StyleRule>,
}

#[derive(Clone, Deserialize)]
pub struct ColorThemes {
    pub default: String,
    pub themes: std::collections::HashMap<String, ColorTheme>,
}

#[derive(Clone, Deserialize)]
pub struct ColorTheme {
    #[serde(rename = "appearance")]
    pub _appearance: Appearance,
    pub colors: std::collections::HashMap<String, u32>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Appearance {
    Light,
    Dark,
}

#[derive(Builder, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleRule {
    #[builder(into)]
    pub class_name: String,
    pub declarations: Vec<StyleDeclaration>,
    #[builder(default = 10)]
    pub specificity: u16,
    #[builder(default)]
    pub source_order: u32,
}

#[derive(Builder, Clone, Deserialize)]
pub struct StyleDeclaration {
    #[builder(into)]
    pub property: String,
    pub value: IrValue,
    #[serde(default)]
    #[builder(default)]
    pub important: bool,
}

impl StyleSheet {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != VERSION {
            return Err("unsupported version");
        }
        if let Some(themes) = &self.color_themes {
            let Some(default) = themes.themes.get(&themes.default) else {
                return Err("missing default color theme");
            };
            if themes.themes.values().any(|theme| {
                theme.colors.len() != default.colors.len()
                    || default
                        .colors
                        .keys()
                        .any(|token| !theme.colors.contains_key(token))
            }) {
                return Err("inconsistent color theme tokens");
            }
        }
        Ok(())
    }

    pub fn ignores_class(&self, candidate: &str) -> bool {
        self.ignored_class_patterns
            .iter()
            .any(|pattern| glob_matches(pattern, candidate))
    }
}

fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let pattern = pattern.as_bytes();
    let candidate = candidate.as_bytes();
    let (mut p, mut c, mut star, mut retry) = (0, 0, None, 0);
    while c < candidate.len() {
        if p < pattern.len() && pattern[p] == candidate[c] {
            p += 1;
            c += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            retry = c;
        } else if let Some(star_index) = star {
            p = star_index + 1;
            retry += 1;
            c = retry;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

pub fn utility_value(value: &crate::Value) -> IrValue {
    match value {
        crate::Value::Keyword { value } => IrValue::Keyword {
            value: value.clone(),
        },
        crate::Value::Boolean { value } => IrValue::Boolean { value: *value },
        crate::Value::Number { value } => IrValue::Number { value: *value },
        crate::Value::Length { value } => IrValue::Length {
            value: match value {
                crate::Length::Px { value } => IrLength::Px { value: *value },
                crate::Length::Percent { value } => IrLength::Percent { value: *value },
                crate::Length::Auto => IrLength::Auto,
            },
        },
        crate::Value::Color { value } => IrValue::Color {
            value: match value {
                crate::Color::Literal { rgba } => IrColor::Literal { rgba: *rgba },
            },
        },
        crate::Value::List { values } => IrValue::List {
            values: values.iter().map(utility_value).collect(),
        },
        crate::Value::Record { fields } => IrValue::Record {
            fields: fields
                .iter()
                .map(|(key, value)| (key.clone(), utility_value(value)))
                .collect(),
        },
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedDeclaration {
    pub property: String,
    pub value: IrValue,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CascadeResult {
    pub declarations: Vec<ResolvedDeclaration>,
    pub diagnostics: Vec<String>,
}

/// Resolve universal rules, authored classes, and supported runtime utilities
/// using Wabou's deterministic class-list cascade.
pub fn resolve_classes(sheet: &StyleSheet, classes: &[&str]) -> CascadeResult {
    let active_colors = sheet
        .color_themes
        .as_ref()
        .and_then(|themes| themes.themes.get(&themes.default))
        .map(|theme| &theme.colors);
    let mut utility_theme = sheet.theme.clone();
    if let Some(colors) = active_colors {
        utility_theme
            .colors
            .extend(colors.iter().map(|(name, color)| (name.clone(), *color)));
    }

    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();
    for rule in &sheet.rules {
        if rule.class_name != "*" {
            continue;
        }
        append_rule(&mut declarations, rule, 0);
    }
    for (class_position, class_name) in classes.iter().enumerate() {
        let mut matched = false;
        for rule in &sheet.rules {
            if rule.class_name == *class_name {
                matched = true;
                append_rule(&mut declarations, rule, class_position + 1);
            }
        }
        if matched || sheet.ignores_class(class_name) {
            continue;
        }
        let semantic_color = ["bg-", "border-", "text-"]
            .iter()
            .find_map(|prefix| class_name.strip_prefix(prefix))
            .filter(|token| active_colors.is_some_and(|colors| colors.contains_key(*token)));
        match crate::parse_utility_with_theme(class_name, &utility_theme) {
            Ok(utility) => {
                for (declaration_index, declaration) in utility.declarations.into_iter().enumerate()
                {
                    let value = if let (Some(token), crate::Value::Color { .. }) =
                        (semantic_color, &declaration.value)
                    {
                        IrValue::Color {
                            value: IrColor::Token {
                                name: token.to_owned(),
                            },
                        }
                    } else {
                        utility_value(&declaration.value)
                    };
                    declarations.push((
                        false,
                        10,
                        class_position + 1,
                        0,
                        declaration_index,
                        declaration.property,
                        value,
                    ));
                }
            }
            Err(error) => diagnostics.push(format!(".{class_name}: {error}")),
        }
    }
    declarations.sort_by_key(
        |(important, specificity, class_position, source_order, index, _, _)| {
            (
                *important,
                *specificity,
                *class_position,
                *source_order,
                *index,
            )
        },
    );
    CascadeResult {
        declarations: declarations
            .into_iter()
            .map(|(_, _, _, _, _, property, value)| ResolvedDeclaration {
                property,
                value: resolve_color_tokens(value, active_colors),
            })
            .collect(),
        diagnostics,
    }
}

fn append_rule(
    declarations: &mut Vec<(bool, u16, usize, u32, usize, String, IrValue)>,
    rule: &StyleRule,
    class_position: usize,
) {
    for (declaration_index, declaration) in rule.declarations.iter().enumerate() {
        declarations.push((
            declaration.important,
            rule.specificity,
            class_position,
            rule.source_order,
            declaration_index,
            declaration.property.clone(),
            declaration.value.clone(),
        ));
    }
}

fn resolve_color_tokens(
    value: IrValue,
    colors: Option<&std::collections::HashMap<String, u32>>,
) -> IrValue {
    match value {
        IrValue::Color {
            value: IrColor::Token { name },
        } => colors.and_then(|colors| colors.get(&name)).map_or_else(
            || IrValue::Color {
                value: IrColor::Token { name: name.clone() },
            },
            |rgba| IrValue::Color {
                value: IrColor::Literal { rgba: *rgba },
            },
        ),
        IrValue::List { values } => IrValue::List {
            values: values
                .into_iter()
                .map(|value| resolve_color_tokens(value, colors))
                .collect(),
        },
        IrValue::Record { fields } => IrValue::Record {
            fields: fields
                .into_iter()
                .map(|(name, value)| (name, resolve_color_tokens(value, colors)))
                .collect(),
        },
        value => value,
    }
}

#[doc(hidden)]
pub mod fixture {
    use std::collections::HashMap;

    use super::*;

    pub fn sheet(rules: Vec<StyleRule>) -> StyleSheet {
        StyleSheet::builder().rules(rules).build()
    }

    pub fn rule(class_name: &str, declarations: Vec<StyleDeclaration>) -> StyleRule {
        StyleRule::builder()
            .class_name(class_name)
            .declarations(declarations)
            .build()
    }

    pub fn declaration(property: &str, value: IrValue) -> StyleDeclaration {
        StyleDeclaration::builder()
            .property(property)
            .value(value)
            .build()
    }

    pub fn keyword(value: &str) -> IrValue {
        IrValue::Keyword {
            value: value.to_string(),
        }
    }

    pub fn number(value: f32) -> IrValue {
        IrValue::Number { value }
    }

    pub fn px(value: f32) -> IrValue {
        IrValue::Length {
            value: IrLength::Px { value },
        }
    }

    pub fn percent(value: f32) -> IrValue {
        IrValue::Length {
            value: IrLength::Percent { value },
        }
    }

    pub fn auto() -> IrValue {
        IrValue::Length {
            value: IrLength::Auto,
        }
    }

    pub fn color(rgba: u32) -> IrValue {
        IrValue::Color {
            value: IrColor::Literal { rgba },
        }
    }

    pub fn color_token(name: &str) -> IrValue {
        IrValue::Color {
            value: IrColor::Token {
                name: name.to_owned(),
            },
        }
    }

    pub fn record(fields: impl IntoIterator<Item = (&'static str, IrValue)>) -> IrValue {
        IrValue::Record {
            fields: HashMap::from_iter(
                fields
                    .into_iter()
                    .map(|(name, value)| (name.to_string(), value)),
            ),
        }
    }

    pub fn edges(value: IrValue) -> IrValue {
        record([
            ("top", value.clone()),
            ("right", value.clone()),
            ("bottom", value.clone()),
            ("left", value),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_patterns_support_exact_and_glob_matches() {
        assert!(glob_matches("lucide", "lucide"));
        assert!(glob_matches("lucide-*", "lucide-sun"));
        assert!(glob_matches("icon-*-filled", "icon-home-filled"));
        assert!(!glob_matches("lucide-*", "text-lucide-sun"));
    }

    #[test]
    fn deserializes_versioned_style_ir() {
        let json = r#"{"version":4,"rules":[]}"#;
        assert!(matches!(
            serde_json::from_str::<StylesheetUpdate>(json).unwrap(),
            StylesheetUpdate::Ir(_)
        ));
    }

    #[test]
    fn class_cascade_is_backend_neutral_and_preserves_class_order() {
        let sheet = fixture::sheet(vec![
            fixture::rule(
                "*",
                vec![fixture::declaration("opacity", fixture::number(0.5))],
            ),
            fixture::rule(
                "narrow",
                vec![fixture::declaration("width", fixture::px(120.0))],
            ),
            fixture::rule(
                "wide",
                vec![fixture::declaration("width", fixture::px(320.0))],
            ),
        ]);

        let resolved = resolve_classes(&sheet, &["narrow", "wide", "flex"]);
        assert!(resolved.diagnostics.is_empty());
        assert_eq!(
            resolved
                .declarations
                .iter()
                .filter(|declaration| declaration.property == "width")
                .map(|declaration| declaration.value.clone())
                .collect::<Vec<_>>(),
            [fixture::px(120.0), fixture::px(320.0)]
        );
        assert!(
            resolved
                .declarations
                .iter()
                .any(|declaration| declaration.property == "display")
        );
    }

    #[test]
    fn class_cascade_resolves_active_theme_tokens_before_backend_projection() {
        let sheet = StyleSheet::builder()
            .color_themes(ColorThemes {
                default: "light".into(),
                themes: std::collections::HashMap::from([(
                    "light".into(),
                    ColorTheme {
                        _appearance: Appearance::Light,
                        colors: std::collections::HashMap::from([("accent".into(), 0x1234_56ff)]),
                    },
                )]),
            })
            .rules(vec![fixture::rule(
                "accent-text",
                vec![fixture::declaration(
                    "color",
                    fixture::color_token("accent"),
                )],
            )])
            .build();

        let resolved = resolve_classes(&sheet, &["accent-text"]);
        assert_eq!(
            resolved.declarations,
            [ResolvedDeclaration {
                property: "color".into(),
                value: fixture::color(0x1234_56ff),
            }]
        );
    }
}
