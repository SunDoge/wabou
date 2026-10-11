//! Bounded data-only keyed text messages. Rust strings stay on their own side.
const PREFIX: &str = "keyed-text-v1\n";
pub fn encode(key: &str, text: &str) -> Result<String, &'static str> {
    if key.is_empty() {
        return Err("keyed text key is empty");
    }
    let mut wire = format!("{PREFIX}{}\n", key.len());
    if wire
        .len()
        .saturating_add(key.len())
        .saturating_add(text.len())
        > crate::MAX_MESSAGE_TEXT_BYTES
    {
        return Err("keyed text message exceeds limit");
    }
    wire.push_str(key);
    wire.push_str(text);
    Ok(wire)
}
pub fn decode(wire: &str) -> Result<(&str, &str), &'static str> {
    if wire.len() > crate::MAX_MESSAGE_TEXT_BYTES {
        return Err("keyed text message exceeds limit");
    }
    let (length, tail) = wire
        .strip_prefix(PREFIX)
        .ok_or("invalid keyed text message")?
        .split_once('\n')
        .ok_or("missing keyed text length")?;
    let size: usize = length.parse().map_err(|_| "invalid keyed text length")?;
    if size == 0 || size.to_string() != length {
        return Err("noncanonical keyed text length");
    }
    let key = tail.get(..size).ok_or("invalid keyed text key bytes")?;
    let text = tail.get(size..).ok_or("invalid keyed text value bytes")?;
    Ok((key, text))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trips_empty_text_unicode_newlines_and_boundaries() {
        let key = "a\n🦀";
        let wire = encode(key, "\0\nβ").unwrap();
        assert_eq!(decode(&wire), Ok((key, "\0\nβ")));
        assert_eq!(decode(&encode(key, "").unwrap()), Ok((key, "")));
        let overhead = encode("x", "").unwrap().len();
        let maximum = "z".repeat(crate::MAX_MESSAGE_TEXT_BYTES - overhead);
        let wire = encode("x", &maximum).unwrap();
        assert_eq!(wire.len(), crate::MAX_MESSAGE_TEXT_BYTES);
        assert_eq!(decode(&wire), Ok(("x", maximum.as_str())));
        assert!(encode("x", &(maximum + "x")).is_err());
        assert!(encode("", "text").is_err());
    }
    #[test]
    fn rejects_ordinary_text_wrong_versions_noncanonical_and_split_utf8() {
        for wire in [
            "ordinary",
            "keyed-text-v2\n1\na",
            "keyed-text-v1\n0\n",
            "keyed-text-v1\n01\na",
            "keyed-text-v1\n+1\na",
            "keyed-text-v1\n3\n🦀",
            "keyed-text-v1\n8\na",
            "keyed-text-v1\n999999999999999999999999\na",
        ] {
            assert!(decode(wire).is_err(), "{wire}");
        }
        assert!(decode(&"x".repeat(crate::MAX_MESSAGE_TEXT_BYTES + 1)).is_err());
    }
}
