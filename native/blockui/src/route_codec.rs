//! Canonical path-segment framing used by generated route implementations.

fn unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

pub fn encode_segment(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::new();
    for byte in value.bytes() {
        if unreserved(byte) {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    encoded
}

pub fn decode_segment(encoded: &str) -> Option<String> {
    if encoded.is_empty() || encoded.len() > 4096 {
        return None;
    }
    fn hex(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    }
    let mut decoded = Vec::with_capacity(encoded.len());
    let mut bytes = encoded.bytes();
    while let Some(byte) = bytes.next() {
        if byte == b'%' {
            decoded.push(hex(bytes.next()?)? * 16 + hex(bytes.next()?)?);
        } else if unreserved(byte) {
            decoded.push(byte);
        } else {
            return None;
        }
    }
    let value = String::from_utf8(decoded).ok()?;
    if value.chars().any(char::is_control)
        || matches!(value.as_str(), "." | "..")
        || encode_segment(&value) != encoded
    {
        return None;
    }
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn segments_have_one_encoding_and_decode_once() {
        for (text, path) in [
            ("name-._~123", "name-._~123"),
            ("a b/c?d#e%f+g", "a%20b%2Fc%3Fd%23e%25f%2Bg"),
            ("文档 🦀", "%E6%96%87%E6%A1%A3%20%F0%9F%A6%80"),
            ("%2F", "%252F"),
        ] {
            assert_eq!(encode_segment(text), path);
            assert_eq!(decode_segment(path).as_deref(), Some(text));
        }
        for invalid in [
            "",
            ".",
            "..",
            "%",
            "%2",
            "%GG",
            "%2f",
            "%41",
            "%7E",
            "%00",
            "%0A",
            "%FF",
            "%C0%AF",
            "%ED%A0%80",
            "%E6%96",
            "文档",
            "x y",
            "a/b",
        ] {
            assert!(decode_segment(invalid).is_none(), "{invalid}");
        }
        assert!(decode_segment(&"x".repeat(4096)).is_some());
        assert!(decode_segment(&"x".repeat(4097)).is_none());
    }
}
