//! Stable capability dispatch with separately loaded application implementations.
//!
//! The module interface is polling based: native Futures never cross the ABI.
//! An invocation pins its generation until completion or cancellation. Plugins
//! must not leave their own threads/callbacks running after drop_task; long-lived
//! work belongs to the stable host. This API accepts trusted local plugins only.
use stabby::libloading::StabbyLibrary;
use std::collections::BTreeMap;
use std::{
    ffi::c_void,
    marker::PhantomData,
    path::Path,
    rc::Rc,
    sync::{
        Arc, Mutex, RwLock, Weak,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use wabou_capability_abi::{self as abi, Bytes, HostApi, ModuleApi, Output};
use wabou_host_api::plugin::{Export, ExportKind, MANIFEST_METHOD, PLUGIN_CONTRACT_VERSION};

#[cfg(test)]
#[path = "capability_module_tests.rs"]
mod tests;

/// Stable host operations, including application-defined resource handle access.
pub type HostOperations = dyn Fn(&str, &[u8]) -> Result<Vec<u8>, String> + Send + Sync;
type ReloadWake = dyn Fn() + Send + Sync;
const MAX_OUTPUT: usize = 1024 * 1024;

static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

struct Generation {
    id: u64,
    exports: Option<Vec<Export>>,
    api: ModuleApi,
    _library: Option<libloading::Library>,
    // Library unload precedes removal of its private on-disk generation.
    _directory: Option<tempfile::TempDir>,
}

/// A stable dispatcher shared by all JS method wrappers of one capability.
#[derive(Clone)]
pub struct CapabilityModule {
    current: Arc<RwLock<Option<Arc<Generation>>>>,
    host: Arc<HostOperations>,
    contract_version: u32,
    completion: Option<Arc<tokio::sync::Notify>>,
    reload_wakes: Arc<Mutex<Vec<Weak<ReloadWake>>>>,
    required_exports: Arc<RwLock<BTreeMap<(ExportKind, String), u32>>>,
    export: Option<(ExportKind, String)>,
}
impl CapabilityModule {
    /// Create an unloaded dispatcher. The host closure owns retained Rust state.
    pub fn new(contract_version: u32, host: Arc<HostOperations>) -> Self {
        Self {
            current: Arc::new(RwLock::new(None)),
            host,
            contract_version,
            completion: None,
            reload_wakes: Arc::default(),
            required_exports: Arc::default(),
            export: None,
        }
    }
    /// Bind a registry export while retaining the plugin's shared code generation.
    /// The host closure owns this capability's separate state/resources. Bind all
    /// consumers before starting its watcher; every candidate validates them all.
    pub fn select_export(
        &self,
        kind: ExportKind,
        name: &str,
        version: u32,
        host: Arc<HostOperations>,
    ) -> Result<Self, String> {
        if self.contract_version != PLUGIN_CONTRACT_VERSION
            || self.export.is_some()
            || name.trim().is_empty()
        {
            return Err("select_export requires a registry root and nonempty name".into());
        }
        let key = (kind, name.to_owned());
        let mut required = self
            .required_exports
            .write()
            .map_err(|_| "plugin registry poisoned")?;
        if let Some(previous) = required.get(&key)
            && *previous != version
        {
            return Err(format!(
                "conflicting plugin export versions: {kind:?}/{name}"
            ));
        }
        if let Some(generation) = self
            .current
            .read()
            .map_err(|_| "module dispatcher poisoned")?
            .as_ref()
        {
            validate_export(
                generation
                    .exports
                    .as_deref()
                    .ok_or("plugin manifest missing")?,
                &key,
                version,
            )?;
        }
        required.insert(key.clone(), version);
        let mut selected = self.clone();
        selected.export = Some(key);
        selected.host = host;
        Ok(selected)
    }
    fn validate_api(&self, api: ModuleApi) -> Result<Option<Vec<Export>>, String> {
        if api.abi_version != abi::ABI_VERSION {
            return Err("incompatible capability module ABI".into());
        }
        if api.contract_version != self.contract_version {
            return Err("incompatible capability contract version".into());
        }
        if self.contract_version != PLUGIN_CONTRACT_VERSION {
            return Ok(None);
        }
        let probe = Self::new(
            PLUGIN_CONTRACT_VERSION,
            Arc::new(|_, _| Err("plugin registration cannot access host operations".into())),
        );
        *probe
            .current
            .write()
            .map_err(|_| "module dispatcher poisoned")? = Some(Arc::new(Generation {
            id: 0,
            exports: None,
            api,
            _library: None,
            _directory: None,
        }));
        let mut call = probe.start(MANIFEST_METHOD, b"null")?;
        let bytes = call
            .poll()?
            .ok_or("plugin manifest query must be synchronous")?;
        let exports: Vec<Export> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        let mut names = std::collections::BTreeSet::new();
        for entry in &exports {
            if entry.name.trim().is_empty() || !names.insert((entry.kind, &entry.name)) {
                return Err("plugin manifest contains invalid or duplicate exports".into());
            }
        }
        Ok(Some(exports))
    }
    /// Current implementation identity; failed replacements leave it unchanged.
    pub fn generation(&self) -> Result<Option<u64>, String> {
        Ok(self
            .current
            .read()
            .map_err(|_| "module dispatcher poisoned")?
            .as_ref()
            .map(|g| g.id))
    }
    /// Pin the current implementation until this snapshot is dropped. This lets
    /// a resident renderer use one generation for measurement, events and paint.
    pub fn snapshot(&self) -> Result<Self, String> {
        let generation = self
            .current
            .read()
            .map_err(|_| "module dispatcher poisoned")?
            .clone();
        let mut snapshot = self.clone();
        snapshot.current = Arc::new(RwLock::new(generation));
        Ok(snapshot)
    }
    /// Wake resident consumers on replacement. The caller retains the callback;
    /// expired subscriptions are removed and do not keep widgets alive.
    pub fn subscribe_replacement(&self, wake: &Arc<dyn Fn() + Send + Sync>) {
        let mut wakes = self.reload_wakes.lock().unwrap_or_else(|e| e.into_inner());
        wakes.retain(|w| w.strong_count() > 0);
        let weak = Arc::downgrade(wake);
        if !wakes.iter().any(|existing| existing.ptr_eq(&weak)) {
            wakes.push(weak);
        }
    }
    fn notify_replacement(&self) {
        let callbacks: Vec<_> = {
            let mut wakes = self.reload_wakes.lock().unwrap_or_else(|e| e.into_inner());
            wakes.retain(|w| w.strong_count() > 0);
            wakes.iter().filter_map(Weak::upgrade).collect()
        };
        for callback in callbacks {
            callback();
        }
    }
    /// Wake host-task waits on resident completion instead of periodic polling.
    /// The notifier must belong to the scheduler handling this module's task.poll.
    pub fn with_completion_notification(mut self, completion: Arc<tokio::sync::Notify>) -> Self {
        self.completion = Some(completion);
        self
    }
    /// Load a candidate transactionally. An invalid library leaves current code
    /// untouched. Each live generation has one unique private library copy.
    ///
    /// # Safety
    /// The library must honor wabou-capability-abi's contract, including borrowed
    /// pointers, thread lifetimes and panic containment. The loader cannot
    /// sandbox native code or validate a malicious function table.
    pub unsafe fn reload(&self, path: &Path) -> Result<(), String> {
        let _measurement = wabou_telemetry::span("reload", "module.load-swap");
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
        let entry = unsafe { library.get_stabbied::<abi::ModuleEntry>(abi::ENTRY_SYMBOL) }
            .map_err(|e| e.to_string())?;
        let api = entry();
        let exports = self.validate_api(api)?;
        // Keep binding and publication mutually exclusive so a consumer cannot
        // appear between candidate validation and the registry swap.
        let required = self
            .required_exports
            .read()
            .map_err(|_| "plugin registry poisoned")?;
        if let Some(exports) = &exports {
            for (key, version) in required.iter() {
                validate_export(exports, key, *version)?;
            }
        }
        let next = Arc::new(Generation {
            id: NEXT_GENERATION.fetch_add(1, Ordering::Relaxed),
            exports,
            api,
            _library: Some(library),
            _directory: Some(directory),
        });
        *self
            .current
            .write()
            .map_err(|_| "module dispatcher poisoned")? = Some(next);
        drop(required);
        self.notify_replacement();
        Ok(())
    }
    /// Install a statically linked implementation whose code lives for the process lifetime.
    ///
    /// # Safety
    /// The table must honor the capability ABI and all function pointers must
    /// remain valid for the process lifetime.
    pub unsafe fn install_linked(&self, api: ModuleApi) -> Result<(), String> {
        let exports = self.validate_api(api)?;
        // Keep binding and publication mutually exclusive so a consumer cannot
        // appear between candidate validation and the registry swap.
        let required = self
            .required_exports
            .read()
            .map_err(|_| "plugin registry poisoned")?;
        if let Some(exports) = &exports {
            for (key, version) in required.iter() {
                validate_export(exports, key, *version)?;
            }
        }
        *self
            .current
            .write()
            .map_err(|_| "module dispatcher poisoned")? = Some(Arc::new(Generation {
            id: NEXT_GENERATION.fetch_add(1, Ordering::Relaxed),
            exports,
            api,
            _library: None,
            _directory: None,
        }));
        drop(required);
        self.notify_replacement();
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
        let routed;
        let method = if let Some((kind, name)) = &self.export {
            routed = serde_json::to_string(&(kind, name, method)).map_err(|e| e.to_string())?;
            &routed
        } else {
            method
        };
        let mut output = Buffer::default();
        let mut host = HostContext {
            operations: &self.host,
            waiting_for_host: false,
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
            waiting_for_host: false,
        };
        if let Some(error) = output.failure.take() {
            invocation.cancel();
            return Err(error);
        }
        Ok(invocation)
    }
    /// Invoke asynchronously; polling and scheduling belong to the stable host.
    /// Resident tasks use completion notifications when configured. Custom tasks
    /// without host waits retain a 2ms polling fallback; plugin wakers never cross the ABI.
    pub async fn call(&self, method: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        let mut measurement = wabou_telemetry::span("capability.module", method);
        measurement.attribute("requestBytes", request.len() as u64);
        let mut invocation = self.start(method, request)?;
        loop {
            if let Some(completion) = &self.completion {
                let notified = completion.notified();
                tokio::pin!(notified);
                // Register before polling so completion cannot race the pending check.
                notified.as_mut().enable();
                if let Some(result) = invocation.poll()? {
                    measurement.attribute("responseBytes", result.len() as u64);
                    return Ok(result);
                }
                if invocation.waiting_for_host {
                    notified.await;
                } else {
                    tokio::time::sleep(Duration::from_millis(2)).await;
                }
                continue;
            }
            if let Some(result) = invocation.poll()? {
                measurement.attribute("responseBytes", result.len() as u64);
                return Ok(result);
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
    }
}

fn validate_export(
    exports: &[Export],
    key: &(ExportKind, String),
    version: u32,
) -> Result<(), String> {
    if exports
        .iter()
        .any(|e| (e.kind, &e.name) == (key.0, &key.1) && e.version == version)
    {
        Ok(())
    } else {
        Err(format!(
            "missing or incompatible plugin export: {:?}/{} (expected version {version})",
            key.0, key.1
        ))
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
    waiting_for_host: bool,
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
            waiting_for_host: false,
        };
        let generation = self
            .generation
            .as_ref()
            .ok_or("invocation already finished")?;
        let status = unsafe { (generation.api.poll)(self.task, host.api(), output.sink()) };
        self.waiting_for_host = host.waiting_for_host;
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
                waiting_for_host: false,
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
    waiting_for_host: bool,
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
    let host = unsafe { &mut *context.cast::<HostContext<'_>>() };
    let result: Result<Vec<u8>, String> =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let method =
                std::str::from_utf8(unsafe { borrowed(method) }).map_err(|e| e.to_string())?;
            let result = (host.operations)(method, unsafe { borrowed(request) })?;
            if method == "task.poll"
                && serde_json::from_slice::<serde_json::Value>(&result)
                    .ok()
                    .is_some_and(|value| value["status"] == "pending")
            {
                host.waiting_for_host = true;
            }
            Ok(result)
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
