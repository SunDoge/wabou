//! Bounded runtime-owned collection slots. No Vec allocation crosses the ABI.
use crate::{Context, Dependency, Value};
use blockui_abi::{ERROR, MAX_LIST_ITEMS, OK};
use std::ffi::c_void;
pub(crate) unsafe fn copy(ptr: *const i64, len: usize) -> Result<Vec<i64>, String> {
    if len > MAX_LIST_ITEMS
        || (len != 0
            && (ptr.is_null() || !(ptr as usize).is_multiple_of(std::mem::align_of::<i64>())))
    {
        return Err("invalid integer list buffer or item limit exceeded".into());
    }
    if len == 0 {
        return Ok(Vec::new());
    }
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec())
}
pub(crate) unsafe fn output(value: &[i64], out: *mut i64, capacity: usize) -> isize {
    if out.is_null() {
        return value.len() as isize;
    }
    if capacity < value.len() || !(out as usize).is_multiple_of(std::mem::align_of::<i64>()) {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len());
    }
    value.len() as isize
}
pub(crate) unsafe fn output_range(
    value: &[i64],
    start: usize,
    end: usize,
    out: *mut i64,
    capacity: usize,
) -> isize {
    if start > end {
        return ERROR as isize;
    }
    let clipped = &value[start.min(value.len())..end.min(value.len())];
    unsafe { output(clipped, out, capacity) }
}
pub(crate) unsafe extern "C" fn read_range(
    ctx: *mut c_void,
    key: u64,
    start: usize,
    end: usize,
    out: *mut i64,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    turn.reads.insert(Dependency::State(key));
    match turn.storage.slots.get(&key).map(Box::as_ref) {
        Some(Value::IntegerList(value)) => unsafe {
            output_range(value, start, end, out, capacity)
        },
        _ => ERROR as isize,
    }
}
pub(crate) unsafe extern "C" fn read(
    ctx: *mut c_void,
    key: u64,
    out: *mut i64,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    turn.reads.insert(Dependency::State(key));
    match turn.storage.slots.get(&key).map(Box::as_ref) {
        Some(Value::IntegerList(value)) => unsafe { output(value, out, capacity) },
        _ => ERROR as isize,
    }
}
pub(crate) unsafe extern "C" fn previous(
    ctx: *mut c_void,
    key: u64,
    out: *mut i64,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only {
        return ERROR as isize;
    }
    match turn
        .previous
        .as_ref()
        .and_then(|storage| storage.slots.get(&key))
        .map(Box::as_ref)
    {
        Some(Value::IntegerList(value)) => unsafe { output(value, out, capacity) },
        _ => ERROR as isize,
    }
}
pub(crate) unsafe extern "C" fn write(
    ctx: *mut c_void,
    key: u64,
    ptr: *const i64,
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
        Some(Value::IntegerList(old)) => {
            turn.conversions.remove(&key);
            if *old != value {
                let previous = std::mem::replace(old, value);
                if let Some(journal) = &mut turn.journal {
                    journal.entry(key).or_insert(Value::IntegerList(previous));
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
    fn list_buffers_validate_counts_alignment_and_capacity_before_access() {
        let values = [i64::MIN, -1, i64::MAX];
        assert_eq!(
            unsafe { copy(values.as_ptr(), values.len()) }.unwrap(),
            values
        );
        assert!(unsafe { copy(values.as_ptr(), MAX_LIST_ITEMS + 1) }.is_err());
        assert!(unsafe { copy(std::ptr::null(), 1) }.is_err());
        assert!(
            unsafe {
                copy(
                    values.as_ptr().cast::<u8>().wrapping_add(1).cast::<i64>(),
                    1,
                )
            }
            .is_err()
        );
        assert!(unsafe { copy(std::ptr::null(), 0) }.unwrap().is_empty());
        let mut out = [99; 3];
        assert_eq!(
            unsafe { output(&values, out.as_mut_ptr(), 2) },
            ERROR as isize
        );
        assert_eq!(out, [99; 3]);
        assert_eq!(unsafe { output(&values, out.as_mut_ptr(), 3) }, 3);
        assert_eq!(out, values);
    }
}

#[cfg(test)]
mod range_tests {
    use super::*;
    #[test]
    fn clipping_and_invalid_buffers_do_not_touch_unrequested_items() {
        let values = [i64::MIN, 20, i64::MAX];
        let mut out = [99_i64; 2];
        assert_eq!(
            unsafe { output_range(&values, 1, usize::MAX, out.as_mut_ptr(), 2) },
            2
        );
        assert_eq!(out, [20, i64::MAX]);
        out = [99; 2];
        assert_eq!(
            unsafe { output_range(&values, 1, 3, out.as_mut_ptr(), 1) },
            ERROR as isize
        );
        assert_eq!(out, [99; 2]);
        assert_eq!(
            unsafe { output_range(&values, usize::MAX, usize::MAX, out.as_mut_ptr(), 0) },
            0
        );
        assert_eq!(out, [99; 2]);
        assert_eq!(
            unsafe { output_range(&values, 2, 1, std::ptr::null_mut(), 0) },
            ERROR as isize
        );
        let misaligned = out.as_mut_ptr().cast::<u8>().wrapping_add(1).cast::<i64>();
        assert_eq!(
            unsafe { output_range(&values, 0, 1, misaligned, 1) },
            ERROR as isize
        );
    }
}
