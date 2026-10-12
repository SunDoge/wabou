//! Ordinary Rust adapters for replaceable capability libraries.
//!
//! Durable state and long-lived asynchronous work belong to the stable host.
//! A module task can retain owned inputs and host resource IDs, but cannot retain
//! a borrowed invocation context. Rust futures and wakers never cross the ABI.
use abi::{Bytes, HostApi, ModuleApi, Output};
use std::{ffi::c_void, marker::PhantomData, panic::AssertUnwindSafe, task::Poll};
pub use wabou_capability_abi as abi;
pub use wabou_host_api::ConnectionKey;
pub use wabou_host_api::FileKey;
pub use wabou_host_api::ResourceKey;
pub use wabou_host_api::TaskKey;

/// Borrowed access to runtime-owned operations during one invocation step.
pub struct Host<'call> {
    api: HostApi,
    _borrowed: PhantomData<&'call mut ()>,
}
impl Host<'_> {
    /// Connect in resident code. Poll as ConnectionKey, then release the task.
    pub fn tcp_connect(&self, address: &str) -> Result<TaskKey, String> {
        self.call_json("tcp.connect", &address)
    }
    /// Read up to length bytes. Poll as Vec<u8>; empty is EOF unless length is zero.
    pub fn tcp_read(&self, connection: ConnectionKey, length: usize) -> Result<TaskKey, String> {
        self.resource_call("tcp.read", connection, &length)
    }
    /// Write a chunk. Poll as usize and handle partial writes using that count.
    pub fn tcp_write(&self, connection: ConnectionKey, bytes: &[u8]) -> Result<TaskKey, String> {
        self.resource_call("tcp.write", connection, &bytes)
    }
    /// Invalidate the connection and signal pending reads/writes to stop.
    pub fn tcp_close(&self, connection: ConnectionKey) -> Result<(), String> {
        self.call_json("tcp.close", &connection)
    }
    /// Open a resident file asynchronously. Poll the task as `FileKey`, release
    /// the task, and explicitly close the file when no longer needed.
    pub fn file_open(&self, path: &std::path::Path) -> Result<TaskKey, String> {
        self.call_json("file.open", &path)
    }
    /// Read a chunk at the resident cursor. Poll as Vec<u8>; each read may return
    /// fewer bytes than requested. Empty means EOF (unless length was zero).
    pub fn file_read(&self, file: FileKey, length: usize) -> Result<TaskKey, String> {
        self.resource_call("file.read", file, &length)
    }
    /// Prevent new reads. Already-started reads may still finish.
    pub fn file_close(&self, file: FileKey) -> Result<(), String> {
        self.call_json("file.close", &file)
    }
    /// Schedule a resident timer; its lifetime is independent of this module.
    /// Poll it as `task_poll::<()>` and release the handle when no longer needed.
    pub fn sleep(&self, duration: std::time::Duration) -> Result<TaskKey, String> {
        let milliseconds = u64::try_from(duration.as_millis()).map_err(|e| e.to_string())?;
        self.call_json("task.sleep", &milliseconds)
    }
    /// Query a runtime-owned job without transferring any future or waker.
    /// Completed results remain readable until explicitly released.
    pub fn task_poll<T: serde::de::DeserializeOwned>(
        &self,
        task: TaskKey,
    ) -> Poll<Result<T, String>> {
        let snapshot: serde_json::Value = match self.call_json("task.poll", &task) {
            Ok(value) => value,
            Err(error) => return Poll::Ready(Err(error)),
        };
        match snapshot["status"].as_str() {
            Some("pending") => Poll::Pending,
            Some("ready") => Poll::Ready(
                serde_json::from_value(snapshot["value"].clone()).map_err(|e| e.to_string()),
            ),
            Some("failed") => Poll::Ready(Err(snapshot["error"]
                .as_str()
                .unwrap_or("task failed")
                .into())),
            _ => Poll::Ready(Err("invalid task snapshot".into())),
        }
    }
    /// Release a job and invalidate its handle. Pending async work is aborted;
    /// this does not synchronously wait for the executor to drop its future.
    pub fn task_release(&self, task: TaskKey) -> Result<(), String> {
        self.call_json("task.release", &task)
    }
    /// Invoke a resource provider using a typed generational handle. The provider
    /// owns the native object; only its handle and copied data cross the boundary.
    /// Family branding is a Rust check; providers must also validate wire handles.
    pub fn resource_call<
        Family,
        Request: serde::Serialize,
        Response: serde::de::DeserializeOwned,
    >(
        &self,
        operation: &str,
        resource: ResourceKey<Family>,
        request: &Request,
    ) -> Result<Response, String> {
        self.call_json(
            operation,
            &serde_json::json!({"resource": resource, "request": request}),
        )
    }
    /// Invoke a stable operation. Both request and response are copied.
    pub fn call(&self, operation: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        let mut response = Vec::new();
        let status = unsafe {
            (self.api.call)(
                self.api.context,
                Bytes::new(operation.as_bytes()),
                Bytes::new(request),
                Output {
                    context: (&mut response as *mut Vec<u8>).cast(),
                    write: collect,
                },
            )
        };
        if status == abi::READY {
            Ok(response)
        } else {
            Err(String::from_utf8(response).unwrap_or_else(|_| "host operation failed".into()))
        }
    }
    /// Invoke a stable operation with typed JSON data.
    pub fn call_json<Request: serde::Serialize, Response: serde::de::DeserializeOwned>(
        &self,
        operation: &str,
        request: &Request,
    ) -> Result<Response, String> {
        let bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
        let response = self.call(operation, &bytes)?;
        serde_json::from_slice(&response).map_err(|e| e.to_string())
    }
}

/// A typed state slot borrowed from the resident host. Values are copied across
/// the boundary; this handle never owns a module's Rust object in host memory.
pub struct State<'host, 'call, T> {
    host: &'host Host<'call>,
    key: &'static str,
    _value: PhantomData<T>,
}
impl<'call> Host<'call> {
    /// Access one state slot per application type, scoped to this capability.
    /// The key is Rust's fully qualified type name, stable across body edits.
    /// Renaming the type/crate changes identity; schema changes need an explicit
    /// compatible deserializer or migration. No silent reset occurs on decode errors.
    pub fn state<T>(&self) -> State<'_, 'call, T> {
        State {
            host: self,
            key: std::any::type_name::<T>(),
            _value: PhantomData,
        }
    }
}
impl<T: serde::de::DeserializeOwned + serde::Serialize> State<'_, '_, T> {
    /// Read a copied value; None means the slot has never been initialized.
    /// Stored JSON null remains distinct from absence for Option-valued slots.
    pub fn get(&self) -> Result<Option<T>, String> {
        let snapshot: serde_json::Value = self.host.call_json("state.snapshot", &self.key)?;
        let present = snapshot["present"]
            .as_bool()
            .ok_or("invalid state snapshot")?;
        if !present {
            return Ok(None);
        }
        let value = snapshot
            .get("value")
            .ok_or("state snapshot missing value")?
            .clone();
        serde_json::from_value(value)
            .map(Some)
            .map_err(|e| e.to_string())
    }
    /// Replace a slot value. Separate get/set calls are not an atomic transaction.
    pub fn set(&self, value: &T) -> Result<(), String> {
        let value = serde_json::to_value(value).map_err(|e| e.to_string())?;
        self.host.call_json(
            "state.write",
            &serde_json::json!({"key": self.key, "value": value}),
        )
    }
}

/// One module-owned invocation. It must not own threads or escaping callbacks.
/// Dropping it releases invocation-local state; host resources retain identity.
pub trait Task: 'static {
    /// Advance work using a fresh context. Pending work is polled by the host.
    fn poll(&mut self, host: &Host<'_>) -> Poll<Result<Vec<u8>, String>>;
    /// Clean up invocation-owned host jobs/handles on completion, failure or
    /// cancellation. The library is still pinned, and this context is borrowed
    /// only for cleanup. Leave deliberately persistent host resources alive.
    fn dispose(&mut self, _host: &Host<'_>) {}
}

/// Application module implementation, without a runtime or QuickJS dependency.
pub trait Module: 'static {
    /// DTO/operation contract version; independent of code compilation versions.
    const CONTRACT_VERSION: u32;
    /// Decode inputs and create invocation-local work. Host state stays in host.
    fn start(host: &Host<'_>, method: &str, request: &[u8]) -> Result<Box<dyn Task>, String>;
}

/// A completed value/error, useful for bounded synchronous application logic.
pub struct Ready(pub Result<Vec<u8>, String>);
impl Ready {
    /// Encode a typed response.
    pub fn json(value: &impl serde::Serialize) -> Result<Box<dyn Task>, String> {
        Ok(Box::new(Self(
            serde_json::to_vec(value).map_err(|e| e.to_string()),
        )))
    }
}
impl Task for Ready {
    fn poll(&mut self, _: &Host<'_>) -> Poll<Result<Vec<u8>, String>> {
        Poll::Ready(std::mem::replace(
            &mut self.0,
            Err("task already consumed".into()),
        ))
    }
}

/// Decode a request, run an ordinary Rust handler, and encode its result.
/// ABI buffers and task adapters remain inside the SDK. Use a custom Task only
/// for work that genuinely needs multiple host-driven steps.
pub fn json_handler<Request, Response, Error>(
    host: &Host<'_>,
    request: &[u8],
    handler: impl FnOnce(&Host<'_>, Request) -> Result<Response, Error>,
) -> Result<Box<dyn Task>, String>
where
    Request: serde::de::DeserializeOwned,
    Response: serde::Serialize,
    Error: std::fmt::Display,
{
    let request = serde_json::from_slice(request).map_err(|e| e.to_string())?;
    let response = handler(host, request).map_err(|e| e.to_string())?;
    Ready::json(&response)
}

/// Export typed handler registrations, leaving function bodies as normal Rust.
/// Each descriptor must provide name(), e.g. a generated HostMethod/JsonMethod.
/// Handlers receive an explicit borrowed Host followed by the typed request.
#[macro_export]
macro_rules! export_handlers {
    (version: $version:expr, $($method:expr => $handler:expr),+ $(,)?) => {
        struct __WabouHandlers;
        impl $crate::Module for __WabouHandlers {
            const CONTRACT_VERSION: u32 = $version;
            fn start(host: &$crate::Host<'_>, method: &str, request: &[u8])
                -> Result<Box<dyn $crate::Task>, String>
            {
                $(if method == $method.name() {
                    return $crate::json_handler(host, request, $handler);
                })+
                Err(format!("unknown application method: {method}"))
            }
        }
        $crate::export_module!(__WabouHandlers);
    };
}

/// Export one application module from a `cdylib` crate.
#[cfg(not(feature = "static-link"))]
#[macro_export]
macro_rules! export_module {
    ($module:ty) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn wabou_capability_v2() -> $crate::abi::ModuleApi {
            $crate::module_api::<$module>()
        }
        // Same checked-export protocol as stabby::export for the pinned stabby
        // version. Macro hygiene keeps applications from needing a direct
        // stabby dependency solely to expand this SDK macro.
        #[unsafe(no_mangle)]
        pub extern "C" fn wabou_capability_v2_stabbied_v3(
            report: &$crate::abi::stabby::abi::report::TypeReport,
        ) -> Option<$crate::abi::ModuleEntry> {
            $crate::checked_entry(wabou_capability_v2, report)
        }
        #[unsafe(no_mangle)]
        pub extern "C" fn wabou_capability_v2_stabbied_v3_report()
        -> &'static $crate::abi::stabby::abi::report::TypeReport {
            $crate::entry_report()
        }
    };
}

/// Register the same module as ordinary linked Rust code without a global export symbol.
#[cfg(feature = "static-link")]
#[macro_export]
macro_rules! export_module {
    ($module:ty) => {
        pub fn wabou_capability_v2() -> $crate::abi::ModuleApi {
            $crate::module_api::<$module>()
        }
    };
}

/// SDK implementation of stabby's checked export protocol.
#[doc(hidden)]
pub fn entry_report() -> &'static abi::stabby::abi::report::TypeReport {
    <abi::ModuleEntry as abi::stabby::abi::IStable>::REPORT
}
#[doc(hidden)]
pub fn checked_entry(
    entry: abi::ModuleEntry,
    report: &abi::stabby::abi::report::TypeReport,
) -> Option<abi::ModuleEntry> {
    entry_report().is_compatible(report).then_some(entry)
}

/// Function table used by export_module; native consumers must honor the ABI.
#[doc(hidden)]
pub fn module_api<M: Module>() -> ModuleApi {
    ModuleApi {
        abi_version: abi::ABI_VERSION,
        contract_version: M::CONTRACT_VERSION,
        start: start::<M>,
        poll,
        drop_task,
    }
}
unsafe fn bytes<'a>(value: Bytes) -> &'a [u8] {
    if value.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(value.data, value.len) }
    }
}
unsafe extern "C" fn collect(context: *mut c_void, value: Bytes) -> i32 {
    let output = unsafe { &mut *context.cast::<Vec<u8>>() };
    if value.len > (1024usize * 1024).saturating_sub(output.len()) {
        return abi::FAILED;
    }
    output.extend_from_slice(unsafe { bytes(value) });
    abi::READY
}
fn context<'a>(api: HostApi) -> Host<'a> {
    Host {
        api,
        _borrowed: PhantomData,
    }
}
fn guarded<T>(f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(AssertUnwindSafe(f))
        .unwrap_or_else(|_| Err("capability module panicked".into()))
}
fn emit(output: Output, result: Result<Vec<u8>, String>) -> i32 {
    let (status, value) = match result {
        Ok(value) => (abi::READY, value),
        Err(e) => (abi::FAILED, e.into_bytes()),
    };
    if unsafe { (output.write)(output.context, Bytes::new(&value)) } == abi::READY {
        status
    } else {
        abi::FAILED
    }
}
unsafe extern "C" fn start<M: Module>(
    host: HostApi,
    method: Bytes,
    request: Bytes,
    output: Output,
) -> *mut c_void {
    let result = guarded(|| {
        let method = std::str::from_utf8(unsafe { bytes(method) }).map_err(|e| e.to_string())?;
        M::start(&context(host), method, unsafe { bytes(request) })
    });
    match result {
        // Double boxing keeps the trait object's fat pointer inside its library.
        Ok(task) => Box::into_raw(Box::new(task)).cast(),
        Err(error) => {
            emit(output, Err(error));
            std::ptr::null_mut()
        }
    }
}
unsafe extern "C" fn poll(task: *mut c_void, host: HostApi, output: Output) -> i32 {
    let result = guarded(|| {
        let task = unsafe { &mut *task.cast::<Box<dyn Task>>() };
        Ok(task.poll(&context(host)))
    });
    match result {
        Ok(Poll::Pending) => abi::PENDING,
        Ok(Poll::Ready(result)) => emit(output, result),
        Err(error) => emit(output, Err(error)),
    }
}
unsafe extern "C" fn drop_task(task: *mut c_void, host: HostApi) {
    // Destruction panics are contained too. With panic=abort the process aborts.
    let mut task = unsafe { Box::from_raw(task.cast::<Box<dyn Task>>()) };
    let _ = guarded(|| {
        task.dispose(&context(host));
        Ok(())
    });
    // Separate guards ensure a cleanup panic does not skip destruction or cause
    // a second destructor panic while already unwinding from cleanup.
    let _ = guarded(|| {
        drop(task);
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "C" fn state_call(
        context: *mut c_void,
        method: Bytes,
        request: Bytes,
        output: Output,
    ) -> i32 {
        let store =
            unsafe { &mut *context.cast::<std::collections::HashMap<String, serde_json::Value>>() };
        let request: serde_json::Value = serde_json::from_slice(unsafe { bytes(request) }).unwrap();
        let value = match unsafe { bytes(method) } {
            b"state.snapshot" => match store.get(request.as_str().unwrap()) {
                Some(value) => serde_json::json!({"present": true, "value": value}),
                None => serde_json::json!({"present": false}),
            },
            b"state.write" => {
                store.insert(
                    request["key"].as_str().unwrap().into(),
                    request["value"].clone(),
                );
                serde_json::Value::Null
            }
            _ => return emit(output, Err("unknown operation".into())),
        };
        emit(output, Ok(serde_json::to_vec(&value).unwrap()))
    }
    #[test]
    fn typed_state_survives_call_contexts_and_distinguishes_null_from_absence() {
        let mut store = std::collections::HashMap::<String, serde_json::Value>::new();
        let api = HostApi {
            context: (&mut store as *mut std::collections::HashMap<String, serde_json::Value>)
                .cast(),
            call: state_call,
        };
        {
            let host = context(api);
            let state = host.state::<Option<u32>>();
            assert_eq!(state.get().unwrap(), None);
            state.set(&None).unwrap();
        }
        let host = context(api);
        let state = host.state::<Option<u32>>();
        assert_eq!(state.get().unwrap(), Some(None));
        state.set(&Some(42)).unwrap();
        assert_eq!(
            context(api).state::<Option<u32>>().get().unwrap(),
            Some(Some(42))
        );
        store.insert(
            std::any::type_name::<Option<u32>>().into(),
            serde_json::json!("incompatible"),
        );
        assert!(state.get().is_err());
        assert_eq!(store[std::any::type_name::<Option<u32>>()], "incompatible");
    }
    struct CleanupTask(bool);
    static CLEANUP_DROPS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    impl Task for CleanupTask {
        fn poll(&mut self, _: &Host<'_>) -> Poll<Result<Vec<u8>, String>> {
            Poll::Pending
        }
        fn dispose(&mut self, host: &Host<'_>) {
            let _: u32 = host.call_json("increment", &1u32).unwrap();
            if self.0 {
                panic!("cleanup failure");
            }
        }
    }
    impl Drop for CleanupTask {
        fn drop(&mut self) {
            CLEANUP_DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    #[test]
    fn disposal_receives_host_context_and_frees_task_even_when_cleanup_panics() {
        let mut state = 0u32;
        let host = HostApi {
            context: (&mut state as *mut u32).cast(),
            call: host_call,
        };
        for fail in [false, true] {
            let task: Box<dyn Task> = Box::new(CleanupTask(fail));
            let pointer = Box::into_raw(Box::new(task)).cast();
            unsafe {
                drop_task(pointer, host);
            }
        }
        assert_eq!(state, 2);
        assert_eq!(CLEANUP_DROPS.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
    struct Example;
    impl Module for Example {
        const CONTRACT_VERSION: u32 = 3;
        fn start(host: &Host<'_>, method: &str, _: &[u8]) -> Result<Box<dyn Task>, String> {
            if method == "panic" {
                panic!("application failure");
            }
            let count: u32 = host.call_json("increment", &2u32)?;
            Ready::json(&count)
        }
    }
    unsafe extern "C" fn host_call(
        context: *mut c_void,
        _: Bytes,
        request: Bytes,
        output: Output,
    ) -> i32 {
        let state = unsafe { &mut *context.cast::<u32>() };
        *state += serde_json::from_slice::<u32>(unsafe { bytes(request) }).unwrap();
        emit(output, Ok(serde_json::to_vec(state).unwrap()))
    }
    #[test]
    fn adapter_keeps_state_in_host_and_contains_start_panics() {
        let api = module_api::<Example>();
        let mut state = 10u32;
        let host = HostApi {
            context: (&mut state as *mut u32).cast(),
            call: host_call,
        };
        let mut output = Vec::<u8>::new();
        let sink = Output {
            context: (&mut output as *mut Vec<u8>).cast(),
            write: collect,
        };
        let task =
            unsafe { (api.start)(host, Bytes::new(b"increment"), Bytes::new(b"null"), sink) };
        assert!(!task.is_null());
        assert_eq!(state, 12);
        assert_eq!(unsafe { (api.poll)(task, host, sink) }, abi::READY);
        assert_eq!(output, b"12");
        unsafe {
            (api.drop_task)(task, host);
        }
        output.clear();
        let task = unsafe { (api.start)(host, Bytes::new(b"panic"), Bytes::new(b"null"), sink) };
        assert!(task.is_null());
        assert_eq!(output, b"capability module panicked");
        assert_eq!(state, 12);
    }
}

/// Synchronous hot-reloadable native widget protocol.
pub mod widget;

/// One library exporting multiple capabilities and native widgets.
pub mod plugin;
