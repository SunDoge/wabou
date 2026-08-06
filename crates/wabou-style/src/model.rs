use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

/// Static design tokens used while resolving utility candidates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Theme {
    #[serde(default)]
    pub spacing: BTreeMap<String, f32>,
    #[serde(default)]
    pub colors: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicPrefix {
    pub name: &'static str,
    pub properties: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicRule {
    pub resolver: &'static str,
    pub prefixes: Vec<DynamicPrefix>,
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
