//! Versioned capability boundary. Only C-compatible values cross libraries.
//!
//! Plugins are trusted native code. Every function must contain its panics or
//! abort; unwinding through this boundary is forbidden. Inputs, output sinks,
//! and HostApi contexts are borrowed only for the duration of each ABI call.
//! Long-lived objects belong to the host and are identified by application
//! resource IDs. A task pointer is opaque and may only be freed by its module.
#![no_std]
use core::ffi::c_void;

/// Initial module ABI revision, independent of application contract versions.
pub const ABI_VERSION: u32 = 1;
/// Symbol exported by capability libraries.
pub const ENTRY_SYMBOL: &[u8] = b"wabou_capability_v1\0";
/// A call completed successfully.
pub const READY: i32 = 0;
/// A task needs a subsequent host poll.
pub const PENDING: i32 = 1;
/// A call failed; its output contains a UTF-8 error message.
pub const FAILED: i32 = -1;

/// A borrowed byte buffer, never an owned Rust allocation.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bytes {
    /// Buffer address; may be null only when length is zero.
    pub data: *const u8,
    /// Buffer length in bytes.
    pub len: usize,
}
impl Bytes {
    /// Borrow a slice for an ABI call.
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}

/// Output is copied by the receiver during each write callback.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Output {
    /// Receiver-owned context, never dereferenced by the writer.
    pub context: *mut c_void,
    /// Return READY on acceptance, FAILED on rejection. No retained pointers.
    pub write: unsafe extern "C" fn(*mut c_void, Bytes) -> i32,
}

/// Access to stable host operations during a start/poll/drop call.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HostApi {
    /// Borrowed host context. It must not escape an ABI call.
    pub context: *mut c_void,
    /// Invoke a named stable operation with encoded data and resource IDs.
    pub call: unsafe extern "C" fn(*mut c_void, Bytes, Bytes, Output) -> i32,
}

/// A replaceable module's function table; contains no plugin-owned state.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ModuleApi {
    /// Must equal ABI_VERSION.
    pub abi_version: u32,
    /// Must equal the host's size_of::<ModuleApi>().
    pub size: usize,
    /// Application contract revision; mismatches reject replacement.
    pub contract_version: u32,
    /// Begin an invocation. Copy borrowed arguments before returning. Null
    /// indicates failure, with an error written to output.
    pub start: unsafe extern "C" fn(HostApi, Bytes, Bytes, Output) -> *mut c_void,
    /// Poll an opaque task. Write a result on READY or an error on FAILED.
    /// Pending tasks are polled by the stable host; no Rust Future crosses ABI.
    pub poll: unsafe extern "C" fn(*mut c_void, HostApi, Output) -> i32,
    /// Destroy a task exactly once, including cancellation, before unloading.
    /// Native threads/callbacks must not outlive this call. Host-owned tasks may.
    pub drop_task: unsafe extern "C" fn(*mut c_void, HostApi),
}
