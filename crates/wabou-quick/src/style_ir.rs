use bon::Builder;
use serde::Deserialize;
use wabou_shell::style::IrValue;

pub const VERSION: u16 = 3;

#[derive(Clone, Deserialize)]
#[serde(untagged)]
pub(crate) enum StylesheetUpdate {
    Ir(StyleSheet),
}

#[derive(Builder, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StyleSheet {
    #[builder(default = VERSION)]
    pub version: u16,
    #[serde(default)]
    #[builder(default)]
    pub diagnostics: Vec<String>,
    #[serde(default)]
    #[builder(default)]
    pub rules: Vec<StyleRule>,
}

#[derive(Builder, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StyleRule {
    #[builder(into)]
    pub class_name: String,
    pub declarations: Vec<StyleDeclaration>,
    #[builder(default = 10)]
    pub specificity: u16,
    #[builder(default)]
    pub source_order: u32,
}

#[derive(Builder, Clone, Deserialize)]
pub(crate) struct StyleDeclaration {
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
        Ok(())
    }
}

pub(crate) fn utility_value(value: &wabou_style::Value) -> IrValue {
    match value {
        wabou_style::Value::Keyword { value } => IrValue::Keyword {
            value: value.clone(),
        },
        wabou_style::Value::Boolean { value } => IrValue::Boolean { value: *value },
        wabou_style::Value::Number { value } => IrValue::Number { value: *value },
        wabou_style::Value::Length { value } => IrValue::Length {
            value: match value {
                wabou_style::Length::Px { value } => {
                    wabou_shell::style::IrLength::Px { value: *value }
                }
                wabou_style::Length::Percent { value } => {
                    wabou_shell::style::IrLength::Percent { value: *value }
                }
                wabou_style::Length::Auto => wabou_shell::style::IrLength::Auto,
            },
        },
        wabou_style::Value::Color { value } => IrValue::Color {
            value: match value {
                wabou_style::Color::Literal { rgba } => {
                    wabou_shell::style::IrColor::Literal { rgba: *rgba }
                }
            },
        },
        wabou_style::Value::List { values } => IrValue::List {
            values: values.iter().map(utility_value).collect(),
        },
        wabou_style::Value::Record { fields } => IrValue::Record {
            fields: fields
                .iter()
                .map(|(key, value)| (key.clone(), utility_value(value)))
                .collect(),
        },
    }
}

#[cfg(test)]
pub(crate) mod fixture {
    use std::collections::HashMap;

    use wabou_shell::style::{IrColor, IrLength};

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
    fn deserializes_versioned_style_ir() {
        let json = r#"{"version":3,"rules":[]}"#;
        assert!(matches!(
            serde_json::from_str::<StylesheetUpdate>(json).unwrap(),
            StylesheetUpdate::Ir(_)
        ));
    }
}
