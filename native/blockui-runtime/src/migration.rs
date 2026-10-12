//! Read-only access to the previous schema, scoped to candidate migration.
use super::{Context, Value};
use blockui_abi::{ERROR, OK};
use std::ffi::c_void;
pub(super) unsafe extern "C" fn integer(ctx: *mut c_void, key: u64, out: *mut i64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only || out.is_null() {
        return ERROR;
    }
    match turn
        .previous
        .as_ref()
        .and_then(|storage| storage.slots.get(&key))
        .map(Box::as_ref)
    {
        Some(Value::Boolean(value)) => {
            unsafe {
                *out = i64::from(*value);
            }
            OK
        }
        Some(Value::Integer(value)) => {
            unsafe {
                *out = *value;
            }
            OK
        }
        _ => ERROR,
    }
}
pub(super) unsafe extern "C" fn text(
    ctx: *mut c_void,
    key: u64,
    out: *mut u8,
    capacity: usize,
) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only {
        return ERROR as isize;
    }
    let Some(Value::Text(value)) = turn
        .previous
        .as_ref()
        .and_then(|storage| storage.slots.get(&key))
        .map(Box::as_ref)
    else {
        return ERROR as isize;
    };
    if out.is_null() {
        return value.len() as isize;
    }
    if capacity < value.len() {
        return ERROR as isize;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len());
    }
    value.len() as isize
}

pub(super) unsafe extern "C" fn float(ctx: *mut c_void, key: u64, out: *mut f64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only || out.is_null() {
        return ERROR;
    }
    match turn
        .previous
        .as_ref()
        .and_then(|storage| storage.slots.get(&key))
        .map(Box::as_ref)
    {
        Some(Value::Float(value)) => {
            unsafe {
                *out = *value;
            }
            OK
        }
        _ => ERROR,
    }
}
