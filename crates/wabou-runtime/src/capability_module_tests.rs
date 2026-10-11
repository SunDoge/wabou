use super::*;
use std::sync::Mutex;

fn build(directory: &Path, version: u32, contract: u32) -> std::path::PathBuf {
    build_gated(directory, version, contract, false)
}
fn build_gated(directory: &Path, version: u32, contract: u32, gated: bool) -> std::path::PathBuf {
    build_fixture(directory, version, contract, gated, false)
}
fn build_fixture(
    directory: &Path,
    version: u32,
    contract: u32,
    gated: bool,
    wrong_layout: bool,
) -> std::path::PathBuf {
    let mut source = format!(
        r#"use wabou_capability_abi as abi;
use abi::*;
use abi::stabby;
use std::ffi::c_void;
struct Task {{ pending: bool }}
unsafe extern "C" fn start(host: HostApi, _: Bytes, _: Bytes, _: Output) -> *mut c_void {{
    if {gated} {{ unsafe {{ (host.call)(host.context, Bytes::new(b"started"), Bytes::new(b"{version}"), Output {{ context: std::ptr::null_mut(), write: ignore }}); }} }}
    Box::into_raw(Box::new(Task {{ pending: true }})).cast()
}}
unsafe extern "C" fn poll(task: *mut c_void, host: HostApi, output: Output) -> i32 {{
    let task = unsafe {{ &mut *task.cast::<Task>() }};
    if task.pending {{ task.pending = false; return PENDING; }}
    if {gated} {{
        let mut ready = false;
        unsafe {{ (host.call)(host.context, Bytes::new(b"ready"), Bytes::new(b"{version}"), Output {{ context: (&mut ready as *mut bool).cast(), write: read_ready }}); }}
        if !ready {{ return PENDING; }}
    }}
    unsafe {{ (host.call)(host.context, Bytes::new(b"increment"), Bytes::new(b"{version}"), output) }}
}}
unsafe extern "C" fn drop_task(task: *mut c_void, host: HostApi) {{
    unsafe {{ drop(Box::from_raw(task.cast::<Task>())); }}
    unsafe {{ (host.call)(host.context, Bytes::new(b"drop"), Bytes::new(b"{version}"), Output {{ context: std::ptr::null_mut(), write: ignore }}); }}
}}
unsafe extern "C" fn ignore(_: *mut c_void, _: Bytes) -> i32 {{ READY }}
unsafe extern "C" fn read_ready(context: *mut c_void, bytes: Bytes) -> i32 {{
    unsafe {{ *context.cast::<bool>() = bytes.len == 1 && *bytes.data == b'1'; }}
    READY
}}
#[stabby::export]
pub extern "C" fn wabou_capability_v2() -> ModuleApi {{
    ModuleApi {{ abi_version: ABI_VERSION, contract_version: {contract}, start, poll, drop_task }}
}}
"#
    );
    if wrong_layout {
        source = source
            .replace(
                "fn wabou_capability_v2() -> ModuleApi",
                "fn wabou_capability_v2() -> DifferentApi",
            )
            .replace(
                "    ModuleApi { abi_version:",
                "    DifferentApi { abi_version:",
            );
        // Same size and field types, different semantic type identity. Size-only
        // checks would accept this; stabby's report must reject it before entry.
        source.push_str(
            r#"
#[stabby::stabby]
#[derive(Clone, Copy)]
pub struct DifferentApi {
    pub abi_version: u32,
    pub contract_version: u32,
    pub start: unsafe extern "C" fn(HostApi, Bytes, Bytes, Output) -> *mut c_void,
    pub poll: unsafe extern "C" fn(*mut c_void, HostApi, Output) -> i32,
    pub drop_task: unsafe extern "C" fn(*mut c_void, HostApi),
}
const _: () = assert!(std::mem::size_of::<DifferentApi>() == std::mem::size_of::<ModuleApi>());
"#,
        );
        source = source.replace("    DifferentApi { abi_version: ABI_VERSION, contract_version:", "    panic!(\"incompatible entry must never be called\");\n    #[allow(unreachable_code)] DifferentApi { abi_version: ABI_VERSION, contract_version:");
    }
    let name = format!("fixture_{version}_{contract}_{gated}_{wrong_layout}");
    let project = directory.join(&name);
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(project.join("src/lib.rs"), source).unwrap();
    let abi = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../wabou-capability-abi")
        .canonicalize()
        .unwrap();
    std::fs::write(project.join("Cargo.toml"), format!(
        "[package]\nname = {name:?}\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[lib]\ncrate-type = [\"cdylib\"]\n[dependencies]\nstabby = \"=72.1.16\"\nwabou-capability-abi = {{ path = {abi:?} }}\n[profile.dev]\ndebug = 0\n"
    )).unwrap();
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/capability-fixtures");
    let library = target.join("debug").join(format!(
        "{}{name}{}",
        std::env::consts::DLL_PREFIX,
        std::env::consts::DLL_SUFFIX
    ));
    let result = std::process::Command::new("cargo")
        .args(["build", "--offline", "--manifest-path"])
        .arg(project.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    library
}

#[test]
fn generations_preserve_host_state_and_pending_calls_and_clean_up() {
    let fixtures = tempfile::tempdir().unwrap();
    let first = build(fixtures.path(), 1, 7);
    let second = build(fixtures.path(), 10, 7);
    let incompatible = build(fixtures.path(), 100, 8);
    let state = Arc::new(Mutex::new((0u32, Vec::<u32>::new())));
    let captured = state.clone();
    let module = CapabilityModule::new(
        7,
        Arc::new(move |method, request| {
            let value: u32 = std::str::from_utf8(request).unwrap().parse().unwrap();
            let mut state = captured.lock().unwrap();
            match method {
                "increment" => {
                    state.0 += value;
                    Ok(state.0.to_string().into_bytes())
                }
                "drop" => {
                    state.1.push(value);
                    Ok(Vec::new())
                }
                _ => Err("unknown method".into()),
            }
        }),
    );
    let host_lifetime = Arc::downgrade(&module.host);
    assert!(module.start("tick", b"").is_err());
    unsafe {
        module.reload(&first).unwrap();
    }
    let old = Arc::downgrade(module.current.read().unwrap().as_ref().unwrap());
    let old_path = old
        .upgrade()
        .unwrap()
        ._directory
        .as_ref()
        .unwrap()
        .path()
        .to_owned();
    let mut pending = module.start("tick", b"").unwrap();
    assert_eq!(pending.poll().unwrap(), None);
    unsafe {
        module.reload(&second).unwrap();
    }
    assert!(old.upgrade().is_some());
    assert!(old_path.exists());
    assert_eq!(pending.poll().unwrap(), Some(b"1".to_vec()));
    assert!(old.upgrade().is_none());
    assert!(!old_path.exists());
    assert!(pending.poll().is_err());
    unsafe {
        assert!(module.reload(&incompatible).is_err());
        assert!(module.reload(&fixtures.path().join("missing")).is_err());
    }
    let mut current = module.start("tick", b"").unwrap();
    assert_eq!(current.poll().unwrap(), None);
    assert_eq!(current.poll().unwrap(), Some(b"11".to_vec()));
    assert!(current.poll().is_err());
    let mut cancelled = module.start("tick", b"").unwrap();
    assert_eq!(cancelled.poll().unwrap(), None);
    let cancelled_generation = Arc::downgrade(module.current.read().unwrap().as_ref().unwrap());
    let cancelled_path = cancelled_generation
        .upgrade()
        .unwrap()
        ._directory
        .as_ref()
        .unwrap()
        .path()
        .to_owned();
    unsafe {
        module.reload(&first).unwrap();
    }
    cancelled.cancel();
    cancelled.cancel();
    assert!(cancelled_generation.upgrade().is_none());
    assert!(!cancelled_path.exists());
    assert_eq!(*state.lock().unwrap(), (11, vec![1, 10, 10]));
    let current_path = module
        .current
        .read()
        .unwrap()
        .as_ref()
        .unwrap()
        ._directory
        .as_ref()
        .unwrap()
        .path()
        .to_owned();
    drop(module);
    assert!(!current_path.exists());
    assert!(
        host_lifetime.upgrade().is_none(),
        "finished handles must not retain the host namespace"
    );
    drop((pending, current, cancelled));
}

#[test]
fn quickjs_keeps_the_same_function_across_native_replacement() {
    use crate::{jsrt::JsRuntime, native_capability::NativeCapability};
    use wabou_bindgen::HostMethod;
    let fixtures = tempfile::tempdir().unwrap();
    let first = build_gated(fixtures.path(), 1, 7, true);
    let second = build(fixtures.path(), 10, 7);
    let state = Arc::new(Mutex::new(0u32));
    let captured = state.clone();
    use std::sync::atomic::{AtomicBool, Ordering};
    let started = Arc::new(AtomicBool::new(false));
    let release = Arc::new(AtomicBool::new(false));
    let observed = started.clone();
    let allowed = release.clone();
    let module = CapabilityModule::new(
        7,
        Arc::new(move |method, bytes| {
            if method == "started" {
                observed.store(true, Ordering::Release);
                return Ok(Vec::new());
            }
            if method == "ready" {
                return Ok(if allowed.load(Ordering::Acquire) {
                    b"1"
                } else {
                    b"0"
                }
                .to_vec());
            }
            if method == "drop" {
                return Ok(Vec::new());
            }
            let mut state = captured.lock().unwrap();
            *state += std::str::from_utf8(bytes).unwrap().parse::<u32>().unwrap();
            Ok(state.to_string().into_bytes())
        }),
    );
    unsafe {
        module.reload(&first).unwrap();
    }
    let mounted = module.clone();
    let runtime = JsRuntime::new().unwrap();
    runtime
        .mount_capability("replaceable", move |ctx, object| {
            NativeCapability::from_runtime_parts(ctx, object)
                .module_method(HostMethod::<(), u32>::no_request("tick"), mounted)
        })
        .unwrap();
    runtime.with(|ctx| ctx.eval::<(), _>(
        "globalThis.savedTick = __wabou_capabilities.replaceable.tick; globalThis.result = null; savedTick().then(x => result = x);"
    )).unwrap();
    // Drive the old invocation until it has actually selected its generation.
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !started.load(Ordering::Acquire) {
        assert!(
            std::time::Instant::now() < deadline,
            "old invocation never started"
        );
        runtime.poll_pending_jobs().unwrap();
    }
    unsafe {
        module.reload(&second).unwrap();
    }
    release.store(true, Ordering::Release);
    let read = || {
        runtime
            .with(|ctx| ctx.eval::<Option<u32>, _>("result"))
            .unwrap()
    };
    while read().is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "old invocation did not complete"
        );
        runtime.poll_pending_jobs().unwrap();
    }
    assert_eq!(read(), Some(1));
    assert!(
        runtime
            .with(|ctx| ctx.eval::<bool, _>("savedTick === __wabou_capabilities.replaceable.tick"))
            .unwrap()
    );
    runtime
        .with(|ctx| ctx.eval::<(), _>("result = null; savedTick().then(x => result = x);"))
        .unwrap();
    while read().is_none() {
        assert!(
            std::time::Instant::now() < deadline,
            "new invocation did not complete"
        );
        runtime.poll_pending_jobs().unwrap();
    }
    assert_eq!(read(), Some(11));
    drop(runtime);
    assert_eq!(*state.lock().unwrap(), 11);
}

#[test]
fn cargo_watcher_retains_working_code_after_broken_edit_and_recovers() {
    use crate::{
        HostService, HostServiceContext,
        capability_development::{CapabilityDevelopment, ModuleProject},
    };
    let project = tempfile::tempdir().unwrap();
    build(project.path(), 1, 7);
    let fixture = project.path().join("fixture_1_7_false_false");
    let source = fixture.join("src/lib.rs");
    let original = std::fs::read_to_string(&source).unwrap();
    let module = CapabilityModule::new(7, Arc::new(|_, bytes| Ok(bytes.to_vec())));
    let mut config = ModuleProject::new(fixture.join("Cargo.toml"));
    config.target_directory = Some(project.path().join("target"));
    // Restrict the test to authored input rather than rustc outputs/build logs.
    config.watch = vec![source.clone()];
    let service = unsafe { CapabilityDevelopment::new(module.clone(), config) };
    service
        .start(&HostServiceContext::for_alternate_host(None, false, true))
        .unwrap();
    let value = || {
        let mut call = module.start("tick", b"").unwrap();
        assert_eq!(call.poll().unwrap(), None);
        call.poll().unwrap().unwrap()
    };
    assert_eq!(value(), b"1");
    let previous = Arc::downgrade(module.current.read().unwrap().as_ref().unwrap());
    std::fs::write(&source, "this does not compile").unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while service.status().attempts < 2 {
        assert!(
            std::time::Instant::now() < deadline,
            "broken edit was not compiled"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(
        service
            .status()
            .error
            .as_deref()
            .unwrap()
            .contains("src/lib.rs"),
        "compiler source location must reach development status"
    );
    assert_eq!(value(), b"1");
    assert!(previous.upgrade().is_some());
    std::fs::write(&source, original.replace("b\"1\"", "b\"10\"")).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    while previous.upgrade().is_some() {
        assert!(
            std::time::Instant::now() < deadline,
            "watcher did not install corrected build"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert_eq!(value(), b"10");
    service.shutdown().unwrap();
    service.shutdown().unwrap();
}

#[test]
fn gallery_palette_dylib_updates_through_unchanged_json_capability() {
    use crate::{
        HostService, HostServiceContext,
        capability_development::{CapabilityDevelopment, ModuleProject},
        jsrt::JsRuntime,
        native_capability::NativeCapability,
    };
    use wabou_bindgen::JsonMethod;
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let project = tempfile::tempdir().unwrap();
    let source = project.path().join("lib.rs");
    let original = include_str!("../../../apps/gallery-functions/src/lib.rs");
    std::fs::write(&source, original).unwrap();
    // JSON quotes are also valid TOML basic strings for these absolute paths.
    let api = serde_json::to_string(&workspace.join("apps/gallery-api")).unwrap();
    let sdk = serde_json::to_string(&workspace.join("crates/wabou-capability-sdk")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"), format!(
        "[package]\nname = \"gallery-reload-test\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\ncrate-type = [\"cdylib\"]\npath = \"lib.rs\"\n[workspace]\n[dependencies]\ngallery-api = {{ path = {api} }}\nwabou-capability-sdk = {{ path = {sdk} }}\nserde_json = \"1\"\n"
    )).unwrap();
    let module = CapabilityModule::new(1, Arc::new(|_, _| Err("no host operation needed".into())));
    let mut config = ModuleProject::new(project.path().join("Cargo.toml"));
    config.target_directory = Some(project.path().join("target"));
    config.watch = vec![source.clone()];
    let service = unsafe { CapabilityDevelopment::new(module.clone(), config) };
    service
        .start(&HostServiceContext::for_alternate_host(None, false, true))
        .unwrap();
    let runtime = JsRuntime::new().unwrap();
    runtime
        .mount_capability("bindingsDemo", move |ctx, object| {
            NativeCapability::from_runtime_parts(ctx, object).json_module_method(
                JsonMethod::<serde_json::Value, serde_json::Value>::new("describePalette"),
                module.clone(),
            )
        })
        .unwrap();
    runtime
        .with(|ctx| {
            ctx.eval::<(), _>(
                "globalThis.savedPalette = __wabou_capabilities.bindingsDemo.describePalette;",
            )
        })
        .unwrap();
    let invoke = || {
        runtime.with(|ctx| ctx.eval::<(), _>(
            r#"globalThis.paletteResult = null; savedPalette(JSON.stringify({name: 'ocean', swatchCount: 3})).then(x => paletteResult = x);"#
        )).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            runtime.poll_pending_jobs().unwrap();
            let result = runtime
                .with(|ctx| ctx.eval::<Option<String>, _>("paletteResult"))
                .unwrap();
            if let Some(result) = result {
                break serde_json::from_str::<serde_json::Value>(&result).unwrap();
            }
            assert!(
                std::time::Instant::now() < deadline,
                "palette promise did not settle"
            );
        }
    };
    let first = invoke();
    assert_eq!(first["ok"], true);
    assert_eq!(first["value"]["title"], "ocean palette");
    assert_eq!(
        first["value"]["swatches"],
        serde_json::json!(["ocean-1", "ocean-2", "ocean-3"])
    );
    std::fs::write(&source, original.replace("{} palette", "{} palette v2")).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while service.status().attempts < 2 {
        assert!(
            std::time::Instant::now() < deadline,
            "palette source edit was not compiled"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(
        service.status().error.is_none(),
        "{:?}",
        service.status().error
    );
    assert!(
        runtime
            .with(|ctx| ctx.eval::<bool, _>(
                "savedPalette === __wabou_capabilities.bindingsDemo.describePalette"
            ))
            .unwrap()
    );
    let second = invoke();
    assert_eq!(second["value"]["title"], "ocean palette v2");
    assert_eq!(second["value"]["swatches"], first["value"]["swatches"]);
    drop(runtime);
    service.shutdown().unwrap();
}

#[test]
fn runtime_owned_background_task_and_resource_survive_module_replacement() {
    use crate::{
        HostService, HostServiceContext,
        capability_development::{CapabilityDevelopment, ModuleProject},
    };
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let project = tempfile::tempdir().unwrap();
    let source = project.path().join("lib.rs");
    let original = r#"
use wabou_capability_sdk::{Host, Module, Task, TaskKey, export_module};
use std::task::Poll;
struct Application;
struct Waiting { resource: TaskKey }
impl Module for Application {
    const CONTRACT_VERSION: u32 = 1;
    fn start(host: &Host<'_>, _: &str, _: &[u8]) -> Result<Box<dyn Task>, String> {
        let resource = host.call_json("job.start_or_get", &())?;
        Ok(Box::new(Waiting { resource }))
    }
}
impl Task for Waiting {
    fn poll(&mut self, host: &Host<'_>) -> Poll<Result<Vec<u8>, String>> {
        match host.task_poll::<String>(self.resource) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(error)) => Poll::Ready(Err(error)),
            Poll::Ready(Ok(value)) => Poll::Ready(Ok(format!("v1:{value}").into_bytes())),
        }
    }
}
export_module!(Application);
"#;
    std::fs::write(&source, original).unwrap();
    let sdk = serde_json::to_string(&workspace.join("crates/wabou-capability-sdk")).unwrap();
    std::fs::write(project.path().join("Cargo.toml"), format!(
        "[package]\nname = \"background-reload-test\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\ncrate-type = [\"cdylib\"]\npath = \"lib.rs\"\n[workspace]\n[dependencies]\nwabou-capability-sdk = {{ path = {sdk} }}\n"
    )).unwrap();
    let resource = Arc::new(Mutex::new(None::<String>));
    let saved_resource = resource.clone();
    let executor = tokio::runtime::Runtime::new().unwrap();
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = executor
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let address = listener.local_addr().unwrap();
    let accepted = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let accepted_server = accepted.clone();
    let (close_server, server_closed) = tokio::sync::oneshot::channel();
    let server = executor.spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        accepted_server.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        stream.write_all(b"connected").await.unwrap();
        server_closed.await.unwrap();
    });
    let native = Arc::new(Mutex::new(
        None::<Arc<tokio::sync::Mutex<tokio::net::TcpStream>>>,
    ));
    let native_job = native.clone();
    let scheduler = crate::capability_tasks::CapabilityTasks::new(executor.handle().clone());
    let (finish, gate) = tokio::sync::oneshot::channel();
    let (started, ready) = std::sync::mpsc::channel();
    // The scheduler future and captured resource are compiled into the host.
    let task = scheduler
        .spawn(async move {
            let stream = Arc::new(tokio::sync::Mutex::new(
                tokio::net::TcpStream::connect(address)
                    .await
                    .map_err(|e| e.to_string())?,
            ));
            *native_job.lock().unwrap() = Some(stream.clone());
            started.send(()).unwrap();
            gate.await.map_err(|e| e.to_string())?;
            let mut bytes = [0; 9];
            stream
                .lock()
                .await
                .read_exact(&mut bytes)
                .await
                .map_err(|e| e.to_string())?;
            let value = String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())?;
            *saved_resource.lock().unwrap() = Some(value.clone());
            Ok(value)
        })
        .unwrap();
    let retained = scheduler.clone();
    let module = CapabilityModule::new(
        1,
        Arc::new(move |method, request| match method {
            "job.start_or_get" => serde_json::to_vec(&task).map_err(|e| e.to_string()),
            _ => retained.call(method, request),
        }),
    );
    let mut config = ModuleProject::new(project.path().join("Cargo.toml"));
    config.target_directory = Some(project.path().join("target"));
    config.watch = vec![source.clone()];
    let service = unsafe { CapabilityDevelopment::new(module.clone(), config) };
    service
        .start(&HostServiceContext::for_alternate_host(None, false, true))
        .unwrap();
    let old = Arc::downgrade(module.current.read().unwrap().as_ref().unwrap());
    let mut pending = module.start("connect", b"null").unwrap();
    ready.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(pending.poll().unwrap(), None);
    std::fs::write(&source, original.replace("v1:", "v2:")).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while service.status().attempts < 2 {
        assert!(
            std::time::Instant::now() < deadline,
            "replacement not built"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(service.status().error.is_none());
    assert!(old.upgrade().is_some());
    let mut current = module.start("connect", b"null").unwrap();
    assert_eq!(current.poll().unwrap(), None);
    finish.send(()).unwrap();
    let old_result = loop {
        if let Some(value) = pending.poll().unwrap() {
            break value;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "host background task did not finish"
        );
        std::thread::yield_now();
    };
    assert_eq!(old_result, b"v1:connected");
    assert_eq!(current.poll().unwrap(), Some(b"v2:connected".to_vec()));
    drop(pending);
    assert!(old.upgrade().is_none());
    drop(current);
    service.shutdown().unwrap();
    // The persistent resource is still owned by the runtime after code retires.
    assert_eq!(resource.lock().unwrap().as_deref(), Some("connected"));
    assert_eq!(
        accepted.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "reload must not reconnect"
    );
    let stream = native.lock().unwrap().clone().unwrap();
    assert_eq!(
        executor.block_on(async { stream.lock().await.peer_addr().unwrap() }),
        address
    );
    close_server.send(()).unwrap();
    executor.block_on(async {
        tokio::time::timeout(Duration::from_secs(3), server)
            .await
            .unwrap()
            .unwrap()
    });
    drop(module);
    drop(service);
    drop(scheduler);
}

#[test]
fn typed_sdk_state_survives_actual_library_replacement() {
    use crate::{
        HostService, HostServiceContext,
        capability_development::{CapabilityDevelopment, ModuleProject},
        capability_state::CapabilityState,
    };
    let project = tempfile::tempdir().unwrap();
    let source = project.path().join("lib.rs");
    let original = r#"
use wabou_capability_sdk::{Host, Module, Task, ResourceKey, FileKey, TaskKey, export_module};
#[derive(Clone, Copy)]
enum Connection {}
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Counter { count: u32, text: String, connection: Option<ResourceKey<Connection>>, file: Option<FileKey>, chunk: String }
struct Reading { value: Counter, task: TaskKey }
impl Task for Reading {
    fn poll(&mut self, host: &Host<'_>) -> std::task::Poll<Result<Vec<u8>, String>> {
        match host.task_poll::<Vec<u8>>(self.task) {
            std::task::Poll::Pending => std::task::Poll::Pending,
            std::task::Poll::Ready(result) => std::task::Poll::Ready((|| {
                host.task_release(self.task)?;
                self.value.chunk = String::from_utf8(result?).map_err(|e| e.to_string())?;
                serde_json::to_vec(&self.value).map_err(|e| e.to_string())
            })()),
        }
    }
}
struct Application;
impl Module for Application {
    const CONTRACT_VERSION: u32 = 1;
    fn start(host: &Host<'_>, _: &str, _: &[u8]) -> Result<Box<dyn Task>, String> {
        let slot = host.state::<Counter>();
        let mut value = slot.get()?.unwrap_or_else(|| Counter { count: 0, text: "unsaved input".into(), connection: None, file: None, chunk: String::new() });
        if value.connection.is_none() {
            value.connection = Some(host.call_json("connection.get", &())?);
        }
        let status: String = host.resource_call("connection.read", value.connection.unwrap(), &())?;
        if status != "connected" { return Err("connection lost".into()); }
        if value.file.is_none() { value.file = Some(host.call_json("file.get", &())?); }
        value.count += 1;
        slot.set(&value)?;
        let task = host.file_read(value.file.unwrap(), 1)?;
        Ok(Box::new(Reading { value, task }))
    }
}
export_module!(Application);
"#;
    std::fs::write(&source, original).unwrap();
    let sdk = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("wabou-capability-sdk");
    std::fs::write(project.path().join("Cargo.toml"), format!(
        "[package]\nname = \"typed-state-reload-test\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[lib]\ncrate-type = [\"cdylib\"]\npath = \"lib.rs\"\n[workspace]\n[dependencies]\nwabou-capability-sdk = {{ path = {} }}\nserde = {{ version = \"1\", features = [\"derive\"] }}\nserde_json = \"1\"\n", serde_json::to_string(&sdk).unwrap()
    )).unwrap();
    let executor = tokio::runtime::Runtime::new().unwrap();
    let tasks = crate::capability_tasks::CapabilityTasks::new(executor.handle().clone());
    let files =
        crate::capability_files::CapabilityFiles::new(project.path().to_owned(), tasks.clone());
    std::fs::write(project.path().join("document.txt"), b"AB").unwrap();
    let open: wabou_host_api::TaskKey =
        serde_json::from_slice(&files.call("file.open", br#""document.txt""#).unwrap()).unwrap();
    let open_request = serde_json::to_vec(&open).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    let file: wabou_host_api::FileKey = loop {
        let snapshot: serde_json::Value =
            serde_json::from_slice(&tasks.call("task.poll", &open_request).unwrap()).unwrap();
        if snapshot["status"] == "ready" {
            break serde_json::from_value(snapshot["value"].clone()).unwrap();
        }
        assert!(
            std::time::Instant::now() < deadline,
            "open failed: {snapshot}"
        );
        std::thread::yield_now();
    };
    tasks.call("task.release", &open_request).unwrap();
    let retained_files = files.clone();
    let state = CapabilityState::default();
    #[derive(Clone, Copy)]
    enum Connection {}
    let resources =
        crate::capability_resources::CapabilityResources::<Connection, Arc<String>>::default();
    let connection = Arc::new("connected".to_owned());
    let key = resources.insert(connection.clone()).unwrap();
    let provider = resources.clone();
    let module = CapabilityModule::new(
        1,
        Arc::new(move |operation, request| match operation {
            "file.get" => serde_json::to_vec(&file).map_err(|e| e.to_string()),
            method if method.starts_with("file.") => retained_files.call(method, request),
            method if method.starts_with("task.") => tasks.call(method, request),
            "connection.get" => serde_json::to_vec(&key).map_err(|e| e.to_string()),
            "connection.read" => {
                let request: serde_json::Value =
                    serde_json::from_slice(request).map_err(|e| e.to_string())?;
                let key = serde_json::from_value(request["resource"].clone())
                    .map_err(|e| e.to_string())?;
                provider.with(key, |connection| {
                    serde_json::to_vec(&**connection).map_err(|e| e.to_string())
                })
            }
            _ => state.call(operation, request),
        }),
    );
    let mut config = ModuleProject::new(project.path().join("Cargo.toml"));
    config.target_directory = Some(project.path().join("target"));
    config.watch = vec![source.clone()];
    let service = unsafe { CapabilityDevelopment::new(module.clone(), config) };
    service
        .start(&HostServiceContext::for_alternate_host(None, false, true))
        .unwrap();
    fn wait_for_read(call: &mut ModuleInvocation) -> Vec<u8> {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            if let Some(value) = call.poll().unwrap() {
                return value;
            }
            assert!(std::time::Instant::now() < deadline, "read did not finish");
            std::thread::yield_now();
        }
    }
    let mut call = module.start("increment", b"null").unwrap();
    let value: serde_json::Value = serde_json::from_slice(&wait_for_read(&mut call)).unwrap();
    assert_eq!(value["count"], 1);
    assert_eq!(value["chunk"], "A");
    drop(call);
    let old = Arc::downgrade(module.current.read().unwrap().as_ref().unwrap());
    std::fs::write(
        &source,
        original.replace("value.count += 1", "value.count += 10"),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while service.status().attempts < 2 {
        assert!(
            std::time::Instant::now() < deadline,
            "replacement not built"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    assert!(service.status().error.is_none());
    assert!(old.upgrade().is_none());
    let mut call = module.start("increment", b"null").unwrap();
    let value: serde_json::Value = serde_json::from_slice(&wait_for_read(&mut call)).unwrap();
    assert_eq!(value["count"], 11);
    assert_eq!(value["chunk"], "B");
    assert_eq!(value["text"], "unsaved input");
    assert_eq!(Arc::strong_count(&connection), 2);
    files
        .call("file.close", &serde_json::to_vec(&file).unwrap())
        .unwrap();
    resources.remove(key).unwrap();
    assert_eq!(Arc::strong_count(&connection), 1);
    assert!(module.start("increment", b"null").is_err());
    service.shutdown().unwrap();
}

#[test]
fn linked_implementation_checks_contract_and_executes_without_a_library() {
    unsafe extern "C" fn start(_: HostApi, _: Bytes, _: Bytes, _: Output) -> *mut c_void {
        Box::into_raw(Box::new(())).cast()
    }
    unsafe extern "C" fn poll(_: *mut c_void, _: HostApi, output: Output) -> i32 {
        unsafe { (output.write)(output.context, Bytes::new(b"linked result")) }
    }
    unsafe extern "C" fn drop_task(task: *mut c_void, _: HostApi) {
        unsafe {
            drop(Box::from_raw(task.cast::<()>()));
        }
    }
    let api = ModuleApi {
        abi_version: abi::ABI_VERSION,
        contract_version: 7,
        start,
        poll,
        drop_task,
    };
    let module = CapabilityModule::new(7, Arc::new(|_, _| Ok(Vec::new())));
    unsafe {
        module.install_linked(api).unwrap();
    }
    let mut invocation = module.start("test", b"").unwrap();
    let mut invalid = api;
    invalid.contract_version = 8;
    assert!(unsafe { module.install_linked(invalid) }.is_err());
    assert_eq!(invocation.poll().unwrap(), Some(b"linked result".to_vec()));
    assert_eq!(
        module.start("test", b"").unwrap().poll().unwrap(),
        Some(b"linked result".to_vec())
    );
}

#[test]
fn stabby_rejects_same_size_different_type_before_entry_and_keeps_current_generation() {
    let fixtures = tempfile::tempdir().unwrap();
    let good = build(fixtures.path(), 1, 7);
    let wrong = build_fixture(fixtures.path(), 100, 7, false, true);
    let module = CapabilityModule::new(7, Arc::new(|_, _| Ok(b"current".to_vec())));
    unsafe {
        module.reload(&good).unwrap();
    }
    let error = unsafe { module.reload(&wrong) }.unwrap_err();
    assert!(error.contains("Report mismatch"), "{error}");
    let mut invocation = module.start("increment", b"").unwrap();
    assert!(invocation.poll().unwrap().is_none());
    assert_eq!(invocation.poll().unwrap(), Some(b"current".to_vec()));
}
