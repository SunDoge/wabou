use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "unit", rename_all = "kebab-case")]
/// Length value shared by the Rust parser and generated tooling manifest.
pub enum Length {
    /// Logical-pixel length.
    Px {
        /// Logical-pixel magnitude.
        value: f32,
    },
    /// Percentage expressed in CSS units, where `100` means full size.
    Percent {
        /// Percentage in CSS units, where `100` means full size.
        value: f32,
    },
    /// Layout-engine-defined automatic size.
    Auto,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
/// Serializable color representation used by style tooling.
pub enum Color {
    /// Packed RGBA channels in network byte order (`0xRRGGBBAA`).
    Literal {
        /// Packed RGBA channels in network byte order (`0xRRGGBBAA`).
        rgba: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
/// Typed value emitted by a supported utility rule.
pub enum Value {
    /// Closed vocabulary interpreted by the named property.
    Keyword {
        /// Canonical keyword spelling.
        value: String,
    },
    /// Boolean property value.
    Boolean {
        /// Boolean value.
        value: bool,
    },
    /// Unitless numeric value.
    Number {
        /// Finite unitless number.
        value: f32,
    },
    /// Typed layout length.
    Length {
        /// Typed length.
        value: Length,
    },
    /// Typed color value.
    Color {
        /// Typed color.
        value: Color,
    },
    /// Ordered composite value.
    List {
        /// Ordered child values.
        values: Vec<Value>,
    },
    /// Named composite value.
    Record {
        /// Named child values in deterministic key order.
        fields: BTreeMap<String, Value>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// One normalized style property assignment.
pub struct Declaration {
    /// Canonical Style IR property name.
    pub property: String,
    /// Typed property value.
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Result of parsing one supported utility class.
pub struct ParsedUtility {
    /// Original normalized utility candidate.
    pub class_name: String,
    /// Declarations applied by the candidate.
    pub declarations: Vec<Declaration>,
}

/// Static design tokens used while resolving utility candidates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Theme {
    /// Named logical-pixel spacing tokens.
    #[serde(default)]
    pub spacing: BTreeMap<String, f32>,
    /// Named packed RGBA color tokens.
    #[serde(default)]
    pub colors: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// Prefix handled by a named dynamic resolver.
pub struct DynamicPrefix {
    /// Utility prefix before the dynamic value.
    pub name: &'static str,
    /// Style IR properties potentially emitted by the resolver.
    pub properties: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// Serializable description of one family of dynamic utilities.
pub struct DynamicRule {
    /// Stable resolver identifier understood by build tooling.
    pub resolver: &'static str,
    /// Prefixes accepted by this resolver.
    pub prefixes: Vec<DynamicPrefix>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
/// Versioned parser capability manifest consumed by JS build tooling.
pub struct Manifest {
    /// Manifest schema version.
    pub version: u16,
    /// Built-in spacing tokens.
    pub spacing: BTreeMap<String, f32>,
    /// Built-in packed RGBA color tokens.
    pub colors: BTreeMap<String, u32>,
    /// Utilities that require no dynamic parsing.
    pub static_utilities: BTreeMap<&'static str, Vec<Declaration>>,
    /// Supported dynamic utility families.
    pub dynamic_rules: Vec<DynamicRule>,
    /// Representative candidates used to verify Rust/TypeScript conformance.
    pub conformance: Vec<ParsedUtility>,
}
