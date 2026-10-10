//! Bounded, canonical option data copied from borrowed UTF-8 at admission.
//! These Rust values never cross the C boundary; only the wire bytes do.
use std::collections::BTreeSet;

pub const MAX_OPTIONS: usize = 256;
pub const MAX_LABEL_BYTES: usize = 4096;
pub const MAX_TOTAL_LABEL_BYTES: usize = 65536;
pub const MAX_WIRE_BYTES: usize = 80 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OptionItem {
    pub value: i64,
    pub label: String,
    pub enabled: bool,
}

pub fn validate(options: &[OptionItem]) -> Result<(), &'static str> {
    if options.is_empty() || options.len() > MAX_OPTIONS {
        return Err("select requires 1..=256 options");
    }
    let mut identities = BTreeSet::new();
    let mut bytes = 0usize;
    for option in options {
        bytes = bytes
            .checked_add(option.label.len())
            .ok_or("select label size overflow")?;
        if option.label.is_empty()
            || option.label.len() > MAX_LABEL_BYTES
            || bytes > MAX_TOTAL_LABEL_BYTES
            || !identities.insert(option.value)
        {
            return Err("invalid select labels or duplicate values");
        }
    }
    Ok(())
}

pub fn encode(options: &[OptionItem]) -> Result<String, &'static str> {
    validate(options)?;
    use std::fmt::Write;
    let mut wire = String::from("select1\n");
    for option in options {
        writeln!(
            wire,
            "{}\t{}\t{}",
            option.value,
            u8::from(option.enabled),
            option.label.len()
        )
        .unwrap();
        wire.push_str(&option.label);
    }
    if wire.len() > MAX_WIRE_BYTES {
        return Err("select wire exceeds limit");
    }
    Ok(wire)
}

pub fn decode(wire: &str) -> Result<Vec<OptionItem>, &'static str> {
    if wire.len() > MAX_WIRE_BYTES {
        return Err("select wire exceeds limit");
    }
    let mut remaining = wire
        .strip_prefix("select1\n")
        .ok_or("invalid select wire version")?;
    let mut options = Vec::new();
    while !remaining.is_empty() {
        if options.len() == MAX_OPTIONS {
            return Err("select option count exceeds limit");
        }
        let (header, rest) = remaining.split_once('\n').ok_or("invalid select header")?;
        let mut fields = header.split('\t');
        let value = fields
            .next()
            .ok_or("missing select value")?
            .parse::<i64>()
            .map_err(|_| "invalid select value")?;
        let enabled = match fields.next() {
            Some("0") => false,
            Some("1") => true,
            _ => return Err("invalid select enabled flag"),
        };
        let length = fields
            .next()
            .ok_or("missing select length")?
            .parse::<usize>()
            .map_err(|_| "invalid select length")?;
        if fields.next().is_some() || length > MAX_LABEL_BYTES {
            return Err("invalid select header fields");
        }
        let label = rest
            .get(..length)
            .ok_or("invalid select label length or UTF8 boundary")?;
        remaining = rest.get(length..).ok_or("invalid select label boundary")?;
        options.push(OptionItem {
            value,
            enabled,
            label: label.into(),
        });
    }
    validate(&options)?;
    if encode(&options)? != wire {
        return Err("noncanonical select wire");
    }
    Ok(options)
}

/// The control name is separate from option labels and never parsed by delimiters.
pub fn encode_control(label: &str, options: &[OptionItem]) -> Result<String, &'static str> {
    if label.is_empty() || label.len() > MAX_LABEL_BYTES {
        return Err("invalid select accessible name");
    }
    Ok(format!(
        "select-control1\n{}\n{}{}",
        label.len(),
        label,
        encode(options)?
    ))
}
pub fn decode_control(wire: &str) -> Result<(String, Vec<OptionItem>), &'static str> {
    if wire.len() > MAX_WIRE_BYTES + MAX_LABEL_BYTES + 64 {
        return Err("select control exceeds limit");
    }
    let remaining = wire
        .strip_prefix("select-control1\n")
        .ok_or("invalid select control version")?;
    let (length, remaining) = remaining
        .split_once('\n')
        .ok_or("invalid select name header")?;
    let count: usize = length.parse().map_err(|_| "invalid select name length")?;
    if count == 0 || count > MAX_LABEL_BYTES || count.to_string() != length {
        return Err("invalid select name length");
    }
    let label = remaining
        .get(..count)
        .ok_or("invalid select name boundary")?;
    let options = decode(
        remaining
            .get(count..)
            .ok_or("invalid select options boundary")?,
    )?;
    Ok((label.into(), options))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_delimiters_and_extreme_business_values_round_trip() {
        let options = vec![
            OptionItem {
                value: i64::MIN,
                label: "航程🦀\n\t\0".into(),
                enabled: true,
            },
            OptionItem {
                value: i64::MAX,
                label: "Return".into(),
                enabled: false,
            },
        ];
        assert_eq!(decode(&encode(&options).unwrap()).unwrap(), options);
    }
    #[test]
    fn admission_budgets_are_exact() {
        let options: Vec<_> = (0..MAX_OPTIONS)
            .map(|value| OptionItem {
                value: value as i64,
                label: "x".repeat(MAX_TOTAL_LABEL_BYTES / MAX_OPTIONS),
                enabled: true,
            })
            .collect();
        assert_eq!(decode(&encode(&options).unwrap()).unwrap(), options);
        let mut extra_bytes = options.clone();
        extra_bytes[0].label.push('x');
        assert!(encode(&extra_bytes).is_err());
        let mut extra_option = options;
        extra_option.push(OptionItem {
            value: 999,
            label: "x".into(),
            enabled: true,
        });
        assert!(encode(&extra_option).is_err());
    }
    #[test]
    fn malformed_and_noncanonical_data_are_rejected() {
        for wire in [
            "",
            "select1\n",
            "select2\n0\t1\t1\nx",
            "select1\n01\t1\t1\nx",
            "select1\n1\ttrue\t1\nx",
            "select1\n1\t1\t1\n🦀",
            "select1\n1\t1\t0\n",
            "select1\n1\t1\t1\nx1\t0\t1\ny",
            "select1\n1\t1\t9999999999999999999999999\nx",
        ] {
            assert!(decode(wire).is_err(), "{wire:?}");
        }
        let too_large = vec![OptionItem {
            value: 0,
            label: "x".repeat(MAX_LABEL_BYTES + 1),
            enabled: true,
        }];
        assert!(encode(&too_large).is_err());
    }
}
