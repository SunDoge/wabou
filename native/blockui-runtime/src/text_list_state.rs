//! Host-owned UTF-8 collections; only synchronous borrowed descriptors cross ABI.
use crate::{Context, Dependency, Value, copy_bytes};
use blockui_abi::{Bytes, ERROR, MAX_LIST_ITEMS, MAX_TEXT_LIST_BYTES, OK};
use std::ffi::c_void;
pub(crate) unsafe fn copy(ptr: *const Bytes, len: usize) -> Result<Vec<String>, String> {
    if len > MAX_LIST_ITEMS
        || (len != 0
            && (ptr.is_null() || !(ptr as usize).is_multiple_of(std::mem::align_of::<Bytes>())))
    {
        return Err("invalid text list descriptors or item limit exceeded".into());
    }
    if len == 0 {
        return Ok(Vec::new());
    }
    let descriptors = unsafe { std::slice::from_raw_parts(ptr, len) };
    let mut total = 0_usize;
    // Validate the aggregate bound before copying any item allocations.
    for bytes in descriptors {
        total = total
            .checked_add(bytes.len)
            .ok_or("text list size overflow")?;
        if total > MAX_TEXT_LIST_BYTES {
            return Err("text list exceeds 1 MiB".into());
        }
    }
    descriptors
        .iter()
        .map(|bytes| unsafe { copy_bytes(*bytes) })
        .collect()
}
pub(crate) unsafe fn output(
    value: &[String],
    index: usize,
    out: *mut u8,
    capacity: usize,
) -> isize {
    if index == usize::MAX {
        return if out.is_null() && capacity == 0 {
            value.len() as isize
        } else {
            ERROR as isize
        };
    }
    let Some(text) = value.get(index) else {
        return ERROR as isize;
    };
    if out.is_null() {
        return text.len() as isize;
    }
    if capacity < text.len() {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(text.as_ptr(), out, text.len());
    }
    text.len() as isize
}
unsafe extern "C" fn access(
    ctx: *mut c_void,
    key: u64,
    index: usize,
    out: *mut u8,
    capacity: usize,
    previous: bool,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    let storage = if previous {
        if turn.read_only {
            return ERROR as isize;
        }
        let Some(storage) = turn.previous.as_ref() else {
            return ERROR as isize;
        };
        storage
    } else {
        turn.reads.insert(Dependency::State(key));
        &turn.storage
    };
    match storage.slots.get(&key).map(Box::as_ref) {
        Some(Value::TextList(value)) => unsafe { output(value, index, out, capacity) },
        _ => ERROR as isize,
    }
}
pub(crate) unsafe extern "C" fn read(
    ctx: *mut c_void,
    key: u64,
    index: usize,
    out: *mut u8,
    capacity: usize,
) -> isize {
    unsafe { access(ctx, key, index, out, capacity, false) }
}
pub(crate) unsafe extern "C" fn previous(
    ctx: *mut c_void,
    key: u64,
    index: usize,
    out: *mut u8,
    capacity: usize,
) -> isize {
    unsafe { access(ctx, key, index, out, capacity, true) }
}
pub(crate) unsafe extern "C" fn write(
    ctx: *mut c_void,
    key: u64,
    ptr: *const Bytes,
    len: usize,
) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only {
        return ERROR;
    }
    let Ok(value) = (unsafe { copy(ptr, len) }) else {
        return ERROR;
    };
    match turn.storage.slots.get_mut(&key).map(Box::as_mut) {
        Some(Value::TextList(old)) => {
            turn.conversions.remove(&key);
            if *old != value {
                let Ok(entries) = turn.graph.map_reads.refresh(key, &value) else {
                    return ERROR;
                };
                for dependency in entries {
                    turn.graph.invalidate(dependency);
                }
                let previous = std::mem::replace(old, value);
                if let Some(journal) = &mut turn.journal {
                    journal.entry(key).or_insert(Value::TextList(previous));
                }
                turn.changed.insert(key);
                *turn.revisions.entry(key).or_default() += 1;
                turn.graph.invalidate(Dependency::State(key));
            }
            OK
        }
        _ => ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn read_only_context_rejects_writes_and_previous_reads_without_touching_values() {
        let mut turn = crate::Turn::default();
        turn.storage
            .slots
            .insert(1, Box::new(Value::TextList(vec!["keep".into()])));
        turn.previous = Some(turn.storage.clone());
        turn.read_only = true;
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = std::ptr::from_ref(&context).cast_mut().cast();
        assert_eq!(
            unsafe { write(ctx, 1, std::ptr::null(), usize::MAX) },
            ERROR
        );
        assert_eq!(
            unsafe { previous(ctx, 1, usize::MAX, std::ptr::null_mut(), 0) },
            ERROR as isize
        );
        assert_eq!(
            unsafe { read(ctx, 1, usize::MAX, std::ptr::null_mut(), 0) },
            1
        );
        assert_eq!(
            turn.storage.slots[&1].as_ref(),
            &Value::TextList(vec!["keep".into()])
        );
        assert!(turn.changed.is_empty());
        assert_eq!(
            turn.reads,
            std::collections::BTreeSet::from([Dependency::State(1)])
        );
    }
    #[test]
    fn descriptors_validate_utf8_item_and_aggregate_limits_before_copying() {
        let descriptors = [Bytes::new(""), Bytes::new("a\0b"), Bytes::new("标题😀")];
        assert_eq!(
            unsafe { copy(descriptors.as_ptr(), 3) }.unwrap(),
            ["", "a\0b", "标题😀"]
        );
        assert!(unsafe { copy(std::ptr::null(), 0) }.unwrap().is_empty());
        assert!(unsafe { copy(std::ptr::null(), 1) }.is_err());
        assert!(unsafe { copy(descriptors.as_ptr(), MAX_LIST_ITEMS + 1) }.is_err());
        assert!(
            unsafe { copy(descriptors.as_ptr().cast::<u8>().wrapping_add(1).cast(), 1) }.is_err()
        );
        let invalid = [Bytes {
            ptr: [255_u8].as_ptr(),
            len: 1,
        }];
        assert!(unsafe { copy(invalid.as_ptr(), 1) }.is_err());
        let invalid = [Bytes {
            ptr: std::ptr::null(),
            len: 1,
        }];
        assert!(unsafe { copy(invalid.as_ptr(), 1) }.is_err());
        let too_large = [
            Bytes {
                ptr: std::ptr::null(),
                len: MAX_TEXT_LIST_BYTES,
            },
            Bytes {
                ptr: std::ptr::null(),
                len: 1,
            },
        ];
        assert!(unsafe { copy(too_large.as_ptr(), 2) }.is_err());
        let overflow = [Bytes {
            ptr: std::ptr::null(),
            len: usize::MAX,
        }];
        assert!(unsafe { copy(overflow.as_ptr(), 1) }.is_err());
        let empty = vec![Bytes::new(""); MAX_LIST_ITEMS];
        assert_eq!(
            unsafe { copy(empty.as_ptr(), empty.len()) }.unwrap().len(),
            MAX_LIST_ITEMS
        );
        let max = "x".repeat(MAX_TEXT_LIST_BYTES);
        let max = [Bytes::new(&max)];
        assert_eq!(
            unsafe { copy(max.as_ptr(), 1) }.unwrap()[0].len(),
            MAX_TEXT_LIST_BYTES
        );
    }
    #[test]
    fn indexed_reads_check_capacity_and_preserve_output_on_rejection() {
        let values = vec![String::new(), "a\0b".into(), "😀".into()];
        assert_eq!(
            unsafe { output(&values, usize::MAX, std::ptr::null_mut(), 0) },
            3
        );
        assert_eq!(unsafe { output(&values, 2, std::ptr::null_mut(), 0) }, 4);
        let mut bytes = [99; 4];
        assert_eq!(
            unsafe { output(&values, 2, bytes.as_mut_ptr(), 3) },
            ERROR as isize
        );
        assert_eq!(bytes, [99; 4]);
        assert_eq!(
            unsafe { output(&values, 3, bytes.as_mut_ptr(), 4) },
            ERROR as isize
        );
        assert_eq!(
            unsafe { output(&values, usize::MAX, bytes.as_mut_ptr(), 4) },
            ERROR as isize
        );
        assert_eq!(unsafe { output(&values, 2, bytes.as_mut_ptr(), 4) }, 4);
        assert_eq!(bytes, "😀".as_bytes());
    }
}
