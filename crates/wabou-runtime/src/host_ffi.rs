//! Stateless host functions exposed to QuickJS by the manual registration in
//! `jsrt.rs`. JavaScript names live at that registration boundary so stack
//! traces and the installed global always use the same source of truth.

use rquickjs::{Ctx, Result, TypedArray};

pub fn host_log(tag: String, msg: String) {
    match tag.as_str() {
        "error" => tracing::error!(target: "js", "{msg}"),
        "warn" => tracing::warn!(target: "js", "{msg}"),
        "info" | "log" => tracing::info!(target: "js", "{msg}"),
        _ => tracing::debug!(target: "js", "{msg}"),
    }
}

pub fn host_utf8_encode<'js>(ctx: Ctx<'js>, s: String) -> Result<TypedArray<'js, u8>> {
    TypedArray::new(ctx, s.into_bytes())
}

pub fn host_utf8_decode<'js>(bytes: TypedArray<'js, u8>) -> Result<String> {
    let bytes = bytes.as_bytes().ok_or(rquickjs::Error::Allocation)?;
    // TextDecoder defaults to non-fatal: replace invalid byte sequences with
    // U+FFFD rather than throwing. from_utf8_lossy matches that.
    Ok(String::from_utf8_lossy(bytes).into_owned())
}

/// Copy native values into an exactly-sized JavaScript typed array.
///
/// rquickjs intentionally exposes immutable Rust views for JavaScript-owned
/// buffers. Host functions nevertheless need an efficient output-buffer ABI,
/// so keep the raw-pointer write and all of its invariants in this one helper.
pub fn fill_typed_array<T: Copy>(output: &TypedArray<'_, T>, values: &[T]) -> Result<()> {
    if output.len() != values.len() {
        return Err(rquickjs::Error::Unknown);
    }
    let raw = output.as_raw().ok_or(rquickjs::Error::Unknown)?;
    let byte_len = std::mem::size_of_val(values);
    if raw.len != byte_len {
        return Err(rquickjs::Error::Unknown);
    }
    // QuickJS owns a live, writable ArrayBuffer for this synchronous call.
    // TypedArray validates the element type and exact byte length above.
    unsafe {
        std::ptr::copy_nonoverlapping(values.as_ptr().cast::<u8>(), raw.ptr.as_ptr(), byte_len);
    }
    Ok(())
}
