//! Support for generated transient record codecs. Not a durable Rust layout.
use crate::{Error, Result};
use std::borrow::Cow;

fn invalid() -> Error {
    Error::Application("Invalid generated record payload".into())
}
pub trait Field: Sized {
    fn encode(&self) -> Cow<'_, str>;
    fn decode(text: &str) -> Result<Self>;
    fn append_to(&self, output: &mut String) -> Result<()> {
        append_parts(output, &[self.encode().as_ref()])
    }
}
impl<T: Field> Field for Option<T> {
    fn encode(&self) -> Cow<'_, str> {
        match self {
            None => Cow::Borrowed("0"),
            Some(value) => Cow::Owned(format!("1{}", value.encode())),
        }
    }
    fn decode(text: &str) -> Result<Self> {
        if text == "0" {
            return Ok(None);
        }
        let value = text.strip_prefix('1').ok_or_else(invalid)?;
        Ok(Some(T::decode(value)?))
    }
    fn append_to(&self, output: &mut String) -> Result<()> {
        match self {
            None => append_parts(output, &["0"]),
            Some(value) => append_parts(output, &["1", value.encode().as_ref()]),
        }
    }
}
impl Field for String {
    fn encode(&self) -> Cow<'_, str> {
        Cow::Borrowed(self)
    }
    fn decode(text: &str) -> Result<Self> {
        Ok(text.to_owned())
    }
}
impl Field for i64 {
    fn encode(&self) -> Cow<'_, str> {
        Cow::Owned(self.to_string())
    }
    fn decode(text: &str) -> Result<Self> {
        let value: Self = text.parse().map_err(|_| invalid())?;
        if value.to_string() != text {
            return Err(invalid());
        }
        Ok(value)
    }
}
impl Field for bool {
    fn encode(&self) -> Cow<'_, str> {
        Cow::Borrowed(if *self { "1" } else { "0" })
    }
    fn decode(text: &str) -> Result<Self> {
        match text {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(invalid()),
        }
    }
}
impl Field for u64 {
    fn encode(&self) -> Cow<'_, str> {
        Cow::Owned(self.to_string())
    }
    fn decode(text: &str) -> Result<Self> {
        let value: Self = text.parse().map_err(|_| invalid())?;
        if value.to_string() != text {
            return Err(invalid());
        }
        Ok(value)
    }
}
impl Field for f64 {
    fn encode(&self) -> Cow<'_, str> {
        Cow::Owned(format!("{:016x}", self.to_bits()))
    }
    fn decode(text: &str) -> Result<Self> {
        if text.len() != 16 {
            return Err(invalid());
        }
        let bits = u64::from_str_radix(text, 16).map_err(|_| invalid())?;
        if format!("{bits:016x}") != text {
            return Err(invalid());
        }
        Ok(Self::from_bits(bits))
    }
}

/// Generated nested record schema support. Lengths delimit child format identities.
#[doc(hidden)]
pub const fn nested_fingerprint(mut hash: u64, formats: &[&str]) -> [u8; 16] {
    let mut index = 0;
    while index < formats.len() {
        let bytes = formats[index].as_bytes();
        let length = (bytes.len() as u64).to_le_bytes();
        let mut offset = 0;
        while offset < 8 {
            hash = (hash ^ length[offset] as u64).wrapping_mul(0x100000001b3);
            offset += 1;
        }
        offset = 0;
        while offset < bytes.len() {
            hash = (hash ^ bytes[offset] as u64).wrapping_mul(0x100000001b3);
            offset += 1;
        }
        index += 1;
    }
    let mut hex = [0; 16];
    let digits = b"0123456789abcdef";
    index = 0;
    while index < 16 {
        hex[index] = digits[((hash >> ((15 - index) * 4)) & 15) as usize];
        index += 1;
    }
    hex
}
#[doc(hidden)]
pub const fn schema_name<const N: usize>(prefix: &str, fingerprint: [u8; 16]) -> [u8; N] {
    assert!(N == prefix.len() + 16);
    let mut output = [0; N];
    let mut index = 0;
    while index < prefix.len() {
        output[index] = prefix.as_bytes()[index];
        index += 1;
    }
    let mut digit = 0;
    while digit < 16 {
        output[index + digit] = fingerprint[digit];
        digit += 1;
    }
    output
}

/// Encode a record with its schema identity. Used by maps and nested fields.
/// Both the envelope and the payload share the 1 MiB budget.
pub fn encode_record<T: crate::Record>(record: &T) -> Result<String> {
    validate_format::<T>()?;
    let payload = record.encode()?;
    if T::FORMAT
        .len()
        .saturating_add(1)
        .saturating_add(payload.len())
        > 1024 * 1024
    {
        return Err(Error::Application("Record payload exceeds 1 MiB".into()));
    }
    Ok(format!("{}\n{payload}", T::FORMAT))
}
/// Reject mismatched child schemas and noncanonical handwritten codecs.
pub fn decode_record<T: crate::Record>(encoded: &str) -> Result<T> {
    validate_format::<T>()?;
    if encoded.len() > 1024 * 1024 {
        return Err(Error::Application("Record payload exceeds 1 MiB".into()));
    }
    let (_, payload) = encoded
        .split_once('\n')
        .filter(|(format, _)| *format == T::FORMAT)
        .ok_or_else(|| Error::Application("Record format requires migration".into()))?;
    let value = T::decode(payload)?;
    if value.encode()? != payload {
        return Err(Error::Application("Noncanonical record encoding".into()));
    }
    Ok(value)
}
fn validate_format<T: crate::Record>() -> Result<()> {
    if T::FORMAT.is_empty() || T::FORMAT.len() > 128 || T::FORMAT.contains('\n') {
        return Err(Error::Application("Invalid record format identity".into()));
    }
    Ok(())
}
/// Append a nested record as one length-delimited field, including its schema.
/// A failure leaves the parent's output unchanged.
pub fn append_record<T: crate::Record>(output: &mut String, record: &T) -> Result<()> {
    let encoded = encode_record(record)?;
    append_parts(output, &[&encoded])
}

/// An optional nested record is tagged independently of its schema envelope.
/// None remains distinct from a present record with an empty payload.
pub fn append_optional_record<T: crate::Record>(
    output: &mut String,
    record: &Option<T>,
) -> Result<()> {
    validate_format::<T>()?;
    match record {
        None => append_parts(output, &["0"]),
        Some(record) => {
            let encoded = encode_record(record)?;
            append_parts(output, &["1", &encoded])
        }
    }
}

/// Maximum entries in one nested record list, in addition to the 1 MiB limit.
pub const MAX_RECORD_ITEMS: usize = 4096;
/// Append an ordered list atomically. Each element carries its child schema.
pub fn append_records<T: crate::Record>(output: &mut String, records: &[T]) -> Result<()> {
    validate_format::<T>()?;
    if records.len() > MAX_RECORD_ITEMS {
        return Err(Error::Application("Record list exceeds item budget".into()));
    }
    let mut encoded = String::new();
    append(&mut encoded, &(records.len() as u64))?;
    for record in records {
        append_record(&mut encoded, record)?;
    }
    append_parts(output, &[&encoded])
}

/// Append a bounded ordered scalar list atomically.
pub fn append_fields<T: Field>(output: &mut String, values: &[T]) -> Result<()> {
    if values.len() > MAX_RECORD_ITEMS {
        return Err(Error::Application("Record list exceeds item budget".into()));
    }
    let mut encoded = String::new();
    append(&mut encoded, &(values.len() as u64))?;
    for value in values {
        append(&mut encoded, value)?;
    }
    append_parts(output, &[&encoded])
}

pub fn append<T: Field>(output: &mut String, value: &T) -> Result<()> {
    value.append_to(output)
}
fn append_parts(output: &mut String, parts: &[&str]) -> Result<()> {
    let size = parts
        .iter()
        .fold(0_usize, |size, part| size.saturating_add(part.len()));
    let length = size.to_string();
    if output
        .len()
        .saturating_add(length.len())
        .saturating_add(1)
        .saturating_add(size)
        > 1024 * 1024
    {
        return Err(Error::Application("Record payload exceeds 1 MiB".into()));
    }
    output.push_str(&length);
    output.push('\n');
    for part in parts {
        output.push_str(part);
    }
    Ok(())
}
pub struct Reader<'a> {
    remaining: &'a str,
}
impl<'a> Reader<'a> {
    pub fn new(payload: &'a str) -> Result<Self> {
        if payload.len() > 1024 * 1024 {
            return Err(Error::Application("Record payload exceeds 1 MiB".into()));
        }
        Ok(Self { remaining: payload })
    }
    pub fn field<T: Field>(&mut self) -> Result<T> {
        self.field_with(T::decode)
    }
    /// Decode one nested record while validating its schema and canonical bytes.
    pub fn record<T: crate::Record>(&mut self) -> Result<T> {
        self.field_with(decode_record::<T>)
    }
    /// Decode an optional child, validating presence tag and child schema.
    pub fn optional_record<T: crate::Record>(&mut self) -> Result<Option<T>> {
        validate_format::<T>()?;
        self.field_with(|text| {
            if text == "0" {
                return Ok(None);
            }
            let encoded = text.strip_prefix('1').ok_or_else(invalid)?;
            decode_record(encoded).map(Some)
        })
    }
    /// Decode a bounded ordered list. Malformed lists do not advance this reader.
    pub fn records<T: crate::Record>(&mut self) -> Result<Vec<T>> {
        validate_format::<T>()?;
        self.list(3, |reader| reader.record())
    }
    #[doc(hidden)]
    pub fn upgrade_records<T: crate::Record>(&mut self) -> Result<Vec<T>> {
        self.list(3, |reader| reader.field_with(T::upgrade_encoded))
    }
    /// Decode a bounded ordered scalar list, including empty strings.
    pub fn fields<T: Field>(&mut self) -> Result<Vec<T>> {
        self.list(2, |reader| reader.field())
    }
    fn list<T>(
        &mut self,
        minimum_field_bytes: usize,
        mut decode: impl FnMut(&mut Reader<'_>) -> Result<T>,
    ) -> Result<Vec<T>> {
        self.field_with(|text| {
            let mut reader = Reader::new(text)?;
            let count = reader.field::<u64>()?;
            if count > MAX_RECORD_ITEMS as u64
                || count > (reader.remaining.len() / minimum_field_bytes) as u64
            {
                return Err(Error::Application("Invalid record list count".into()));
            }
            let mut values = Vec::with_capacity(count as usize);
            for _ in 0..count {
                values.push(decode(&mut reader)?);
            }
            reader.finish()?;
            Ok(values)
        })
    }
    fn field_with<T>(&mut self, decode: impl FnOnce(&str) -> Result<T>) -> Result<T> {
        let (length, rest) = self.remaining.split_once('\n').ok_or_else(invalid)?;
        let count: usize = length.parse().map_err(|_| invalid())?;
        if count.to_string() != length {
            return Err(invalid());
        }
        let text = rest.get(..count).ok_or_else(invalid)?;
        let value = decode(text)?;
        self.remaining = &rest[count..];
        Ok(value)
    }
    pub fn finish(self) -> Result<()> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(invalid())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn string_encoding_borrows_and_capacity_rejection_preserves_output() {
        let text = "x".repeat(1024 * 1024);
        let encoded = text.encode();
        assert!(matches!(encoded, Cow::Borrowed(_)));
        assert_eq!(encoded.as_ptr(), text.as_ptr());
        let mut output = "existing".to_owned();
        let pointer = output.as_ptr();
        let capacity = output.capacity();
        assert!(append(&mut output, &text).is_err());
        assert_eq!(output, "existing");
        assert_eq!(output.as_ptr(), pointer);
        assert_eq!(output.capacity(), capacity);
        assert!(append(&mut output, &Some(text)).is_err());
        assert_eq!(output, "existing");
        assert_eq!(output.as_ptr(), pointer);
        assert_eq!(output.capacity(), capacity);
        assert!(matches!(true.encode(), Cow::Borrowed("1")));
    }
}
