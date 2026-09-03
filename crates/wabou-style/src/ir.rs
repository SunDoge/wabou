//! Backend-neutral values transported by the compiled Wabou style protocol.

use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "unit", rename_all = "kebab-case")]
/// Length representation accepted by the runtime Style IR.
pub enum IrLength {
    /// Logical-pixel length.
    Px {
        /// Logical-pixel magnitude.
        value: f32,
    },
    /// Parent-relative ratio in `0.0..=1.0` representation.
    Percent {
        /// Parent-relative ratio.
        value: f32,
    },
    /// Automatic value resolved by layout.
    Auto,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
/// Color representation accepted by the runtime Style IR.
pub enum IrColor {
    /// Packed RGBA channels in network byte order (`0xRRGGBBAA`).
    Literal {
        /// Packed channel value.
        rgba: u32,
    },
    /// Theme token resolved before paint application.
    Token {
        /// Theme color name.
        name: String,
    },
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
/// Typed value transported from the style compiler to a native backend.
pub enum IrValue {
    /// Property-specific closed-vocabulary keyword.
    Keyword {
        /// Canonical keyword spelling.
        value: String,
    },
    /// Boolean value.
    Boolean {
        /// Boolean payload.
        value: bool,
    },
    /// Finite unitless number.
    Number {
        /// Numeric payload.
        value: f32,
    },
    /// Typed layout length.
    Length {
        /// Length payload.
        value: IrLength,
    },
    /// Typed color.
    Color {
        /// Color payload.
        value: IrColor,
    },
    /// Ordered composite value.
    List {
        /// Child values.
        values: Vec<IrValue>,
    },
    /// Named composite value.
    Record {
        /// Child values keyed by schema field name.
        fields: HashMap<String, IrValue>,
    },
}

impl IrValue {
    /// Return the keyword payload when this is a keyword value.
    #[must_use]
    pub fn keyword(&self) -> Option<&str> {
        match self {
            Self::Keyword { value } => Some(value),
            _ => None,
        }
    }

    /// Return the numeric payload when this is a number value.
    #[must_use]
    pub fn number(&self) -> Option<f32> {
        match self {
            Self::Number { value } => Some(*value),
            _ => None,
        }
    }

    /// Return the length payload when this is a length value.
    #[must_use]
    pub fn length(&self) -> Option<&IrLength> {
        match self {
            Self::Length { value } => Some(value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn style_ir_deserializes_without_a_render_backend() {
        let value: IrValue =
            serde_json::from_str(r#"{"type":"length","value":{"unit":"percent","value":0.5}}"#)
                .expect("valid backend-neutral style value");
        assert_eq!(
            value,
            IrValue::Length {
                value: IrLength::Percent { value: 0.5 }
            }
        );
    }
}
