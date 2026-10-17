//! Compact, bounded wire decoder for compiler-produced Style IR.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::{IrColor, IrLength, IrValue, Theme};

use super::{Appearance, ColorTheme, ColorThemes, StyleDeclaration, StyleRule, StyleSheet};

const MAGIC: u32 = 0x4953_4257;
const WIRE_VERSION: u16 = 1;
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_ITEMS: usize = 1_000_000;
const MAX_STRING_BYTES: usize = 1024 * 1024;
const MAX_VALUE_DEPTH: usize = 32;

/// A malformed or unsupported binary stylesheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeError(&'static str);

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for DecodeError {}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
    strings: Vec<String>,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Result<Self, DecodeError> {
        if bytes.len() > MAX_BYTES {
            return Err(DecodeError("binary Style IR exceeds 16 MiB"));
        }
        Ok(Self {
            bytes,
            cursor: 0,
            strings: Vec::new(),
        })
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(DecodeError("binary Style IR offset overflow"))?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or(DecodeError("truncated binary Style IR"))?;
        self.cursor = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().expect("exact slice"),
        ))
    }

    fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("exact slice"),
        ))
    }

    fn f32(&mut self) -> Result<f32, DecodeError> {
        let value = f32::from_le_bytes(self.take(4)?.try_into().expect("exact slice"));
        if !value.is_finite() {
            return Err(DecodeError("binary Style IR contains a non-finite number"));
        }
        Ok(value)
    }

    fn count(&mut self) -> Result<usize, DecodeError> {
        let value = self.u32()? as usize;
        if value > MAX_ITEMS {
            return Err(DecodeError("binary Style IR collection is too large"));
        }
        Ok(value)
    }

    fn string(&mut self) -> Result<String, DecodeError> {
        let index = self.u32()? as usize;
        self.strings
            .get(index)
            .cloned()
            .ok_or(DecodeError("binary Style IR has an invalid string id"))
    }

    fn string_list(&mut self) -> Result<Vec<String>, DecodeError> {
        let count = self.count()?;
        (0..count).map(|_| self.string()).collect()
    }

    fn value(&mut self, depth: usize) -> Result<IrValue, DecodeError> {
        if depth > MAX_VALUE_DEPTH {
            return Err(DecodeError("binary Style IR value nesting exceeds 32"));
        }
        match self.u8()? {
            0 => Ok(IrValue::Keyword {
                value: self.string()?,
            }),
            1 => match self.u8()? {
                0 => Ok(IrValue::Boolean { value: false }),
                1 => Ok(IrValue::Boolean { value: true }),
                _ => Err(DecodeError("binary Style IR has an invalid boolean")),
            },
            2 => Ok(IrValue::Number { value: self.f32()? }),
            3 => Ok(IrValue::Length {
                value: IrLength::Px { value: self.f32()? },
            }),
            4 => Ok(IrValue::Length {
                value: IrLength::Percent { value: self.f32()? },
            }),
            5 => Ok(IrValue::Length {
                value: IrLength::Auto,
            }),
            6 => Ok(IrValue::Color {
                value: IrColor::Literal { rgba: self.u32()? },
            }),
            7 => Ok(IrValue::Color {
                value: IrColor::Token {
                    name: self.string()?,
                },
            }),
            8 => {
                let count = self.count()?;
                let mut values = Vec::with_capacity(count);
                for _ in 0..count {
                    values.push(self.value(depth + 1)?);
                }
                Ok(IrValue::List { values })
            }
            9 => {
                let count = self.count()?;
                let mut fields = HashMap::with_capacity(count);
                for _ in 0..count {
                    let name = self.string()?;
                    if fields.insert(name, self.value(depth + 1)?).is_some() {
                        return Err(DecodeError("binary Style IR has a duplicate record field"));
                    }
                }
                Ok(IrValue::Record { fields })
            }
            _ => Err(DecodeError("binary Style IR has an unknown value tag")),
        }
    }

    fn f32_map(&mut self) -> Result<BTreeMap<String, f32>, DecodeError> {
        let count = self.count()?;
        let mut values = BTreeMap::new();
        for _ in 0..count {
            let name = self.string()?;
            if values.insert(name, self.f32()?).is_some() {
                return Err(DecodeError("binary Style IR has a duplicate map key"));
            }
        }
        Ok(values)
    }

    fn u32_btree_map(&mut self) -> Result<BTreeMap<String, u32>, DecodeError> {
        let count = self.count()?;
        let mut values = BTreeMap::new();
        for _ in 0..count {
            let name = self.string()?;
            if values.insert(name, self.u32()?).is_some() {
                return Err(DecodeError("binary Style IR has a duplicate map key"));
            }
        }
        Ok(values)
    }

    fn u32_hash_map(&mut self) -> Result<HashMap<String, u32>, DecodeError> {
        Ok(self.u32_btree_map()?.into_iter().collect())
    }
}

/// Decode a complete compiler-produced stylesheet. Trailing bytes and partial
/// values are rejected so an HMR update is installed atomically.
pub fn decode(bytes: &[u8]) -> Result<StyleSheet, DecodeError> {
    let mut reader = Reader::new(bytes)?;
    if reader.u32()? != MAGIC {
        return Err(DecodeError("invalid binary Style IR magic"));
    }
    if reader.u16()? != WIRE_VERSION {
        return Err(DecodeError("unsupported binary Style IR wire version"));
    }
    let version = reader.u16()?;
    let string_count = reader.count()?;
    reader.strings.reserve(string_count);
    for _ in 0..string_count {
        let length = reader.count()?;
        if length > MAX_STRING_BYTES {
            return Err(DecodeError("binary Style IR string exceeds 1 MiB"));
        }
        let value = std::str::from_utf8(reader.take(length)?)
            .map_err(|_| DecodeError("binary Style IR contains invalid UTF-8"))?;
        reader.strings.push(value.to_owned());
    }

    let theme = Theme {
        spacing: reader.f32_map()?,
        colors: reader.u32_btree_map()?,
    };
    let color_themes = match reader.u8()? {
        0 => None,
        1 => {
            let default = reader.string()?;
            let count = reader.count()?;
            let mut themes = HashMap::with_capacity(count);
            for _ in 0..count {
                let name = reader.string()?;
                let appearance = match reader.u8()? {
                    0 => Appearance::Light,
                    1 => Appearance::Dark,
                    _ => return Err(DecodeError("binary Style IR has an invalid appearance")),
                };
                let theme = ColorTheme {
                    _appearance: appearance,
                    colors: reader.u32_hash_map()?,
                };
                if themes.insert(name, theme).is_some() {
                    return Err(DecodeError("binary Style IR has a duplicate color theme"));
                }
            }
            Some(ColorThemes { default, themes })
        }
        _ => return Err(DecodeError("binary Style IR has an invalid optional flag")),
    };
    let diagnostics = reader.string_list()?;
    let ignored_class_patterns = reader.string_list()?;
    let rule_count = reader.count()?;
    let mut rules = Vec::with_capacity(rule_count);
    for _ in 0..rule_count {
        let class_name = reader.string()?;
        let specificity = reader.u16()?;
        let source_order = reader.u32()?;
        let declaration_count = reader.count()?;
        let mut declarations = Vec::with_capacity(declaration_count);
        for _ in 0..declaration_count {
            declarations.push(StyleDeclaration {
                property: reader.string()?,
                important: match reader.u8()? {
                    0 => false,
                    1 => true,
                    _ => return Err(DecodeError("binary Style IR has an invalid boolean")),
                },
                value: reader.value(0)?,
            });
        }
        rules.push(StyleRule {
            class_name,
            declarations,
            specificity,
            source_order,
        });
    }
    if reader.cursor != bytes.len() {
        return Err(DecodeError("binary Style IR has trailing bytes"));
    }
    let sheet = StyleSheet {
        version,
        theme,
        color_themes,
        diagnostics,
        ignored_class_patterns,
        rules,
    };
    sheet.validate().map_err(DecodeError)?;
    Ok(sheet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_the_typescript_encoder_fixture() {
        let sheet = decode(include_bytes!(
            "../../../../fixtures/style-ir/hmr-color.bin"
        ))
        .expect("shared fixture must decode");
        assert_eq!(sheet.version, super::super::VERSION);
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].class_name, "hmr-color");
        assert_eq!(sheet.rules[0].declarations[0].property, "color");
        assert_eq!(
            sheet.rules[0].declarations[0].value,
            IrValue::Color {
                value: IrColor::Literal { rgba: 0x3366_ccff }
            }
        );
    }

    #[test]
    fn rejects_truncated_and_trailing_frames() {
        assert_eq!(
            decode(b"WBSI").unwrap_err().to_string(),
            "truncated binary Style IR"
        );
        let mut empty = vec![b'W', b'B', b'S', b'I', 1, 0, super::super::VERSION as u8, 0];
        empty.extend_from_slice(&0_u32.to_le_bytes());
        empty.extend_from_slice(&0_u32.to_le_bytes());
        empty.extend_from_slice(&0_u32.to_le_bytes());
        empty.push(0);
        empty.extend_from_slice(&0_u32.to_le_bytes());
        empty.extend_from_slice(&0_u32.to_le_bytes());
        empty.extend_from_slice(&0_u32.to_le_bytes());
        assert!(decode(&empty).is_ok());
        empty.push(0);
        assert_eq!(
            decode(&empty).unwrap_err().to_string(),
            "binary Style IR has trailing bytes"
        );
    }
}
