//! Selected record bytes only; no application decoder is retained by the host.
use crate::{Context, Dependency, Value};
use blockui_abi::{Bytes, ERROR};
use std::ffi::c_void;

pub(super) fn lookup<'a>(encoded: &'a str, selector: &str) -> Result<&'a str, ()> {
    if encoded.len() > 1024 * 1024 {
        return Err(());
    }
    if let Some(selector) = selector.strip_prefix("\n@record-list1\n") {
        return lookup_list(encoded, selector);
    }
    let (format, index) = selector.split_once('\n').ok_or(())?;
    let index: usize = index.parse().map_err(|_| ())?;
    let (actual, mut payload) = encoded.split_once('\n').ok_or(())?;
    if actual != format {
        return Err(());
    }
    let mut selected = None;
    let mut ordinal = 0;
    while !payload.is_empty() {
        let (length, rest) = payload.split_once('\n').ok_or(())?;
        let size: usize = length.parse().map_err(|_| ())?;
        if size.to_string() != length {
            return Err(());
        }
        let field = rest.get(..size).ok_or(())?;
        if ordinal == index {
            selected = Some(field);
        }
        payload = rest.get(size..).ok_or(())?;
        ordinal += 1;
    }
    selected.ok_or(())
}

fn take_field<'a>(payload: &mut &'a str) -> Result<&'a str, ()> {
    let (length, rest) = payload.split_once('\n').ok_or(())?;
    let size: usize = length.parse().map_err(|_| ())?;
    if size.to_string() != length {
        return Err(());
    }
    let field = rest.get(..size).ok_or(())?;
    *payload = rest.get(size..).ok_or(())?;
    Ok(field)
}
fn lookup_list<'a>(encoded: &'a str, mut selector: &str) -> Result<&'a str, ()> {
    let list_format = take_field(&mut selector)?;
    let row_format = take_field(&mut selector)?;
    if list_format.is_empty()
        || row_format.is_empty()
        || list_format.contains('\n')
        || row_format.contains('\n')
    {
        return Err(());
    }
    let identity_index = take_field(&mut selector)?;
    let identity: usize = identity_index.parse().map_err(|_| ())?;
    if identity.to_string() != identity_index {
        return Err(());
    }
    let key = decode_integer(take_field(&mut selector)?)?;
    let target_index = selector.strip_prefix('\n').ok_or(())?;
    let target: usize = target_index.parse().map_err(|_| ())?;
    if target.to_string() != target_index {
        return Err(());
    }
    let (format, mut payload) = encoded.split_once('\n').ok_or(())?;
    if format != list_format {
        return Err(());
    }
    let mut records = take_field(&mut payload)?;
    if !payload.is_empty() {
        return Err(());
    }
    let count = decode_integer(take_field(&mut records)?)?;
    if !(0..=4096).contains(&count) {
        return Err(());
    }
    let identity_selector = format!("{row_format}\n{identity}");
    let target_selector = format!("{row_format}\n{target}");
    let mut selected = None;
    for _ in 0..count {
        let row = take_field(&mut records)?;
        let id = decode_integer(lookup(row, &identity_selector)?)?;
        if id == key {
            if selected.is_some() {
                return Err(());
            }
            selected = Some(lookup(row, &target_selector)?);
        }
    }
    if !records.is_empty() {
        return Err(());
    }
    selected.ok_or(())
}

pub(super) unsafe extern "C" fn read(
    ctx: *mut c_void,
    slot: u64,
    format: Bytes,
    index: usize,
    out: *mut u8,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if format.len == 0 || format.len > 1024 {
        return ERROR as isize;
    }
    let Ok(format) = (unsafe { crate::copy_bytes(format) }) else {
        return ERROR as isize;
    };
    if (!format.starts_with("\n@record-list1\n") && format.contains('\n'))
        || (!format.starts_with("\n@record-list1\n") && format.len() > 128)
    {
        return ERROR as isize;
    }
    let selector = format!("{format}\n{index}");
    let Ok(field) = selected(turn, slot, &selector) else {
        return ERROR as isize;
    };
    if out.is_null() {
        return field.len() as isize;
    }
    if capacity < field.len() {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(field.as_ptr(), out, field.len());
    }
    field.len() as isize
}

pub(super) fn selected<'a>(
    turn: &'a mut crate::Turn,
    slot: u64,
    selector: &str,
) -> Result<&'a str, ()> {
    let Some(Value::Text(encoded)) = turn.storage.slots.get(&slot).map(Box::as_ref) else {
        return Err(());
    };
    let Ok(field) = lookup(encoded, selector) else {
        return Err(());
    };
    let Ok(id) = turn.graph.map_reads.track(slot, selector, Some(field)) else {
        return Err(());
    };
    turn.reads.insert(Dependency::MapEntry(id));
    Ok(field)
}

/// Canonical i64 parsing shared by admission, controls and tracked projection.
pub(super) fn decode_integer(field: &str) -> Result<i64, ()> {
    let value: i64 = field.parse().map_err(|_| ())?;
    if value.to_string() != field {
        return Err(());
    }
    Ok(value)
}
pub(super) fn integer(turn: &crate::Turn, slot: u64, selector: &str) -> Option<i64> {
    let Value::Text(encoded) = turn.storage.slots.get(&slot)?.as_ref() else {
        return None;
    };
    decode_integer(lookup(encoded, selector).ok()?).ok()
}

pub(super) fn decode_float(field: &str) -> Result<f64, ()> {
    let bits = u64::from_str_radix(field, 16).map_err(|_| ())?;
    if format!("{bits:016x}") != field {
        return Err(());
    }
    Ok(f64::from_bits(bits))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyed_list_selection_validates_schema_count_duplicates_and_trailing_bytes() {
        let frame = |text: &str| format!("{}\n{text}", text.len());
        let row = |id: &str, title: &str| format!("row\n{}{}", frame(id), frame(title));
        let selector = format!(
            "\n@record-list1\n{}{}{}{}\n1",
            frame("list"),
            frame("row"),
            frame("0"),
            frame("7")
        );
        let records = format!(
            "{}{}{}",
            frame("2"),
            frame(&row("8", "other")),
            frame(&row("7", "🦀"))
        );
        assert_eq!(
            lookup(&format!("list\n{}", frame(&records)), &selector),
            Ok("🦀")
        );
        for records in [
            format!(
                "{}{}{}",
                frame("2"),
                frame(&row("7", "a")),
                frame(&row("7", "b"))
            ),
            format!("{}{}", frame("1"), frame(&row("8", "missing"))),
            format!("{}{}", frame("01"), frame(&row("7", "a"))),
            format!("{}{}", frame("4097"), frame(&row("7", "a"))),
            format!("{}{}junk", frame("1"), frame(&row("7", "a"))),
            format!("{}{}", frame("1"), frame("wrong\n1\n7")),
        ] {
            assert!(lookup(&format!("list\n{}", frame(&records)), &selector).is_err());
        }
        assert!(lookup(&format!("list\n{}junk", frame(&records)), &selector).is_err());
    }
    #[test]
    fn framing_validates_entire_record_and_utf8_boundaries() {
        assert_eq!(lookup("schema\n0\n4\n🦀1\n7", "schema\n0"), Ok(""));
        assert_eq!(lookup("schema\n0\n4\n🦀1\n7", "schema\n1"), Ok("🦀"));
        for encoded in [
            "other\n1\nx",
            "schema\n01\nx",
            "schema\n3\n🦀",
            "schema\n1\nxjunk",
            "schema\n9\nx",
            "schema\n+1\nx",
        ] {
            assert!(lookup(encoded, "schema\n0").is_err(), "{encoded}");
        }
        assert!(lookup("schema\n0\n", "schema\n1").is_err());
    }
}
