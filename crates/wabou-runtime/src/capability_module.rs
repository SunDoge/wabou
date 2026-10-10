//! Stable capability dispatch with separately loaded application implementations.
//!
//! The module interface is polling based: native Futures never cross the ABI.
//! An invocation pins its generation until completion or cancellation. Plugins
//! must not leave their own threads/callbacks running after drop_task; long-lived
//! work belongs to the stable host. This API accepts trusted local plugins only.
use std::{
    ffi::c_void,
    marker::PhantomData,
    path::Path,
    rc::Rc,
    sync::{Arc, RwLock},
    time::Duration,
};
use wabou_capability_abi::{self as abi, Bytes, HostApi, ModuleApi, Output};

#[cfg(test)]
#[path = "capability_module_tests.rs"]
mod tests;

/// Stable host operations, including application-defined resource handle access.
pub type HostOperations = dyn Fn(&str, &[u8]) -> Result<Vec<u8>, String> + Send + Sync;
const MAX_OUTPUT: usize = 1024 * 1024;

struct Generation {
    api: ModuleApi,
    _library: libloading::Library,
    // Library unload precedes removal of its private on-disk generation.
    _directory: tempfile::TempDir,
}

/// A stable dispatcher shared by all JS method wrappers of one capability.
#[derive(Clone)]
pub struct CapabilityModule {
    current: Arc<RwLock<Option<Arc<Generation>>>>,
    host: Arc<HostOperations>,
    contract_version: u32,
}
impl CapabilityModule {
    /// Create an unloaded dispatcher. The host closure owns retained Rust state.
    pub fn new(contract_version: u32, host: Arc<HostOperations>) -> Self {
        Self {
            current: Arc::new(RwLock::new(None)),
            host,
            contract_version,
        }
    }
    /// Load a candidate transactionally. An invalid library leaves current code
    /// untouched. Each live generation has one unique private library copy.
    ///
    /// # Safety
    /// The library must honor wabou-capability-abi's contract, including borrowed
    /// pointers, thread lifetimes and panic containment. The loader cannot
    /// sandbox native code or validate a malicious function table.
    pub unsafe fn reload(&self, path: &Path) -> Result<(), String> {
        let directory = tempfile::Builder::new()
            .prefix("wabou-capability-")
            .tempdir()
            .map_err(|e| e.to_string())?;
        let destination = directory
            .path()
            .join(path.file_name().ok_or("module path has no file name")?);
        std::fs::copy(path, &destination).map_err(|e| e.to_string())?;
        let library =
            unsafe { libloading::Library::new(&destination) }.map_err(|e| e.to_string())?;
        let entry =
            unsafe { library.get::<unsafe extern "C" fn() -> ModuleApi>(abi::ENTRY_SYMBOL) }
                .map_err(|e| e.to_string())?;
        let api = unsafe { entry() };
        if api.abi_version != abi::ABI_VERSION || api.size != std::mem::size_of::<ModuleApi>() {
            return Err("incompatible capability module ABI".into());
        }
        if api.contract_version != self.contract_version {
            return Err("incompatible capability contract version".into());
        }
        let next = Arc::new(Generation {
            api,
            _library: library,
            _directory: directory,
        });
        *self
            .current
            .write()
            .map_err(|_| "module dispatcher poisoned")? = Some(next);
        Ok(())
    }
    /// Start using the current generation, independent of later replacements.
    pub fn start(&self, method: &str, request: &[u8]) -> Result<ModuleInvocation, String> {
        let generation = self
            .current
            .read()
            .map_err(|_| "module dispatcher poisoned")?
            .clone()
            .ok_or("capability module is not loaded")?;
        let mut output = Buffer::default();
        let mut host = HostContext {
            operations: &self.host,
        };
        let task = unsafe {
            (generation.api.start)(
                host.api(),
                Bytes::new(method.as_bytes()),
                Bytes::new(request),
                output.sink(),
            )
        };
        if task.is_null() {
            return Err(output.error("module start failed"));
        }
        let mut invocation = ModuleInvocation {
            generation: Some(generation),
            host: Some(self.host.clone()),
            task,
            _local: PhantomData,
        };
        if let Some(error) = output.failure.take() {
            invocation.cancel();
            return Err(error);
        }
        Ok(invocation)
    }
    /// Invoke asynchronously; polling and scheduling belong to the stable host.
    /// The initial implementation polls at 2ms intervals rather than exposing
    /// plugin wakers. Host operations should start/query long-running work.
    pub async fn call(&self, method: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        let mut invocation = self.start(method, request)?;
        loop {
            if let Some(result) = invocation.poll()? {
                return Ok(result);
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
}

/// A host-owned invocation pin. Cancellation runs the old module's destructor
/// while its library and retained host state are still alive.
pub struct ModuleInvocation {
    generation: Option<Arc<Generation>>,
    host: Option<Arc<HostOperations>>,
    task: *mut c_void,
    // Tasks stay on their start thread. No unsafe Send/Sync assertion is needed.
    _local: PhantomData<Rc<()>>,
}
impl ModuleInvocation {
    /// Poll once. None means pending. Completion/error destroys the task.
    pub fn poll(&mut self) -> Result<Option<Vec<u8>>, String> {
        if self.task.is_null() {
            return Err("invocation already finished".into());
        }
        let mut output = Buffer::default();
        let mut host = HostContext {
            operations: self.host.as_ref().ok_or("invocation already finished")?,
        };
        let generation = self
            .generation
            .as_ref()
            .ok_or("invocation already finished")?;
        let status = unsafe { (generation.api.poll)(self.task, host.api(), output.sink()) };
        if status == abi::PENDING && output.bytes.is_empty() && output.failure.is_none() {
            return Ok(None);
        }
        self.cancel();
        if status == abi::READY && output.failure.is_none() {
            return Ok(Some(output.bytes));
        }
        Err(output.error("module poll failed"))
    }
    /// Cancel pending work and release plugin-owned allocations.
    pub fn cancel(&mut self) {
        if !self.task.is_null() {
            let task = std::mem::replace(&mut self.task, std::ptr::null_mut());
            let generation = self.generation.take().expect("live task has a generation");
            let operations = self.host.take().expect("live task has host operations");
            let mut host = HostContext {
                operations: &operations,
            };
            unsafe { (generation.api.drop_task)(task, host.api()) };
        }
    }
}
impl Drop for ModuleInvocation {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[derive(Default)]
struct Buffer {
    bytes: Vec<u8>,
    failure: Option<String>,
}
impl Buffer {
    fn sink(&mut self) -> Output {
        Output {
            context: (self as *mut Self).cast(),
            write,
        }
    }
    fn error(self, fallback: &str) -> String {
        self.failure.unwrap_or_else(|| {
            String::from_utf8(self.bytes)
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| fallback.into())
        })
    }
}
unsafe fn borrowed<'a>(bytes: Bytes) -> &'a [u8] {
    if bytes.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) }
    }
}
unsafe extern "C" fn write(context: *mut c_void, bytes: Bytes) -> i32 {
    let buffer = unsafe { &mut *context.cast::<Buffer>() };
    if bytes.len > MAX_OUTPUT.saturating_sub(buffer.bytes.len()) {
        buffer.failure = Some("capability output exceeds 1 MiB".into());
        return abi::FAILED;
    }
    buffer.bytes.extend_from_slice(unsafe { borrowed(bytes) });
    abi::READY
}
struct HostContext<'a> {
    operations: &'a Arc<HostOperations>,
}
impl HostContext<'_> {
    fn api(&mut self) -> HostApi {
        HostApi {
            context: (self as *mut Self).cast(),
            call: host_call,
        }
    }
}
unsafe extern "C" fn host_call(
    context: *mut c_void,
    method: Bytes,
    request: Bytes,
    output: Output,
) -> i32 {
    let host = unsafe { &*context.cast::<HostContext<'_>>() };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let method = std::str::from_utf8(unsafe { borrowed(method) }).map_err(|e| e.to_string())?;
        (host.operations)(method, unsafe { borrowed(request) })
    }))
    .unwrap_or_else(|_| Err("host operation panicked".into()));
    let (status, bytes) = match result {
        Ok(bytes) => (abi::READY, bytes),
        Err(error) => (abi::FAILED, error.into_bytes()),
    };
    let accepted = unsafe { (output.write)(output.context, Bytes::new(&bytes)) };
    if accepted == abi::READY {
        status
    } else {
        abi::FAILED
    }
}
