//! Stable actor ownership, synchronous C ABI turns, keyed view bindings.
use blockui_abi::*;
mod admission;
mod bindings;
#[cfg(feature = "native-window")]
mod build_queue;
mod children;
pub mod interaction;
#[cfg(target_arch = "wasm32")]
#[path = "web/files.rs"]
mod web_files;
#[cfg(windows)]
#[path = "windows/files.rs"]
mod windows_files;
pub use admission::proof as admission_proof;
pub mod application;
mod artifact;
mod async_task;
mod composition;
mod graph;
mod io;
mod keyed_map;
mod list_state;
mod migration;
#[cfg_attr(target_arch = "wasm32", path = "web/network.rs")]
pub mod network;
mod network_task;
mod record_fields;
pub mod request;
mod resources;
#[cfg_attr(target_arch = "wasm32", path = "web/save.rs")]
#[cfg_attr(windows, path = "windows/save.rs")]
mod save;
mod sockets;
pub use sockets::SocketReloadPolicy;
mod scopes;
mod system;
mod text_list_state;
#[cfg_attr(target_arch = "wasm32", path = "web/timer.rs")]
mod timer;
mod wake;
pub use composition::ComposedView;
pub use graph::Dependency;
use graph::Graph;
pub use scopes::ScopeDependencies;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ffi::c_void,
    fs,
    net::TcpStream,
    path::{Path, PathBuf},
    sync::mpsc,
    thread,
    time::Duration,
};
pub use system::{ActorFault, ActorId, Runtime};

#[derive(Clone, Debug)]
pub enum Value {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    IntegerList(Vec<i64>),
    TextList(Vec<String>),
    Text(String),
}
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float(a), Self::Float(b)) => a.to_bits() == b.to_bits(),
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::Text(a), Self::Text(b)) => a == b,
            (Self::IntegerList(a), Self::IntegerList(b)) => a == b,
            (Self::TextList(a), Self::TextList(b)) => a == b,
            _ => false,
        }
    }
}
#[derive(Clone, Default)]
pub struct Storage {
    pub slots: BTreeMap<u64, Box<Value>>,
}
#[derive(Default)]
struct Turn {
    event_owner: u64,
    services: application::Services,
    #[cfg(feature = "testing")]
    test_monotonic_ms: Option<u64>,
    owner: Option<(ActorId, u64)>,
    request_reply: Option<Option<i64>>,
    request_text_reply: Option<Option<String>>,
    child_scope: Option<(u64, std::rc::Weak<std::cell::RefCell<children::Registry>>)>,
    router: Option<std::rc::Rc<std::cell::RefCell<system::Router>>>,
    outbox: Vec<system::Outgoing>,
    storage: Storage,
    changed: BTreeSet<u64>,
    reads: BTreeSet<Dependency>,
    revisions: BTreeMap<u64, u64>,
    graph: Graph,
    journal: Option<BTreeMap<u64, Value>>,
    read_only: bool,
    handler_error: Option<String>,
    request_error: Option<String>,
    message_text: Option<String>,
    task_keys: BTreeSet<u64>,
    task_commands: Vec<TaskCommand>,
    task_generation_budget: u64,
    files: BTreeMap<u64, resources::HostFile>,
    sockets: sockets::Sockets,
    socket_pending: BTreeMap<u64, u64>,
    network_permits: BTreeMap<u64, std::sync::Arc<network_task::Permit>>,
    io_results: BTreeMap<u64, Option<std::sync::Arc<io::ReadResult>>>,
    previous: Option<Storage>,
    conversions: BTreeSet<u64>,
}

impl Turn {
    fn reserve_task_generation(&mut self) -> bool {
        let Some(remaining) = self.task_generation_budget.checked_sub(1) else {
            return false;
        };
        self.task_generation_budget = remaining;
        true
    }
}

enum TaskCommand {
    Delay(u64, Duration, Message, u32),
    Interval(u64, Duration, Message, u32),
    PreparedDelay(u64, PreparedTimer),
    Cancel(u64),
    Read(u64, io::Recipe),
    Network(u64, network_task::Recipe),
    PreparedNetwork(u64, network_task::Prepared),
}

unsafe extern "C" fn request_delay(
    ctx: *mut c_void,
    id: u64,
    millis: u64,
    action: u64,
    value: i64,
    policy: u32,
) -> i32 {
    unsafe { request_timer(ctx, id, millis, action, value, policy, false) }
}
unsafe extern "C" fn request_interval(
    ctx: *mut c_void,
    id: u64,
    millis: u64,
    action: u64,
    value: i64,
    policy: u32,
) -> i32 {
    unsafe { request_timer(ctx, id, millis, action, value, policy, true) }
}
unsafe fn request_timer(
    ctx: *mut c_void,
    id: u64,
    millis: u64,
    action: u64,
    value: i64,
    policy: u32,
    repeating: bool,
) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only
        || turn.journal.is_none()
        || id == 0
        || action == 0
        || policy > 2
        || (repeating && millis == 0)
        || millis > 86_400_000
        || turn.task_keys.len() >= 64
        || turn.task_commands.len() >= 256
        || turn.task_keys.contains(&id)
    {
        return ERROR;
    }
    if !turn.reserve_task_generation() {
        return ERROR;
    }
    turn.task_keys.insert(id);
    let duration = Duration::from_millis(millis);
    let message = Message { action, value };
    turn.task_commands.push(if repeating {
        TaskCommand::Interval(id, duration, message, policy)
    } else {
        TaskCommand::Delay(id, duration, message, policy)
    });
    OK
}

unsafe extern "C" fn stage_reply(ctx: *mut c_void, value: i64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only || turn.journal.is_none() {
        return ERROR;
    }
    let Some(reply) = turn.request_reply.as_mut() else {
        return ERROR;
    };
    if reply.is_some() {
        return ERROR;
    }
    *reply = Some(value);
    OK
}

unsafe extern "C" fn monotonic_ms(ctx: *mut c_void, output: *mut u64) -> i32 {
    if ctx.is_null() || output.is_null() {
        return ERROR;
    }
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    if turn.read_only || turn.journal.is_none() {
        return ERROR;
    }
    #[cfg(feature = "testing")]
    if let Some(millis) = turn.test_monotonic_ms {
        unsafe { output.write(millis) };
        return OK;
    }
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::Instant;
    #[cfg(target_arch = "wasm32")]
    use web_time::Instant;
    static EPOCH: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    let millis = EPOCH.get_or_init(Instant::now).elapsed().as_millis();
    unsafe { output.write(millis.min(u64::MAX as u128) as u64) };
    OK
}

unsafe extern "C" fn stage_text_reply(ctx: *mut c_void, bytes: Bytes) -> i32 {
    if bytes.len > MAX_MESSAGE_TEXT_BYTES {
        return ERROR;
    }
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only || turn.journal.is_none() {
        return ERROR;
    }
    let Some(reply) = turn.request_text_reply.as_mut() else {
        return ERROR;
    };
    if reply.is_some() {
        return ERROR;
    }
    let Ok(text) = (unsafe { copy_bytes(bytes) }) else {
        return ERROR;
    };
    *reply = Some(text);
    OK
}

unsafe extern "C" fn request_cancel_task(ctx: *mut c_void, id: u64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only || turn.journal.is_none() || id == 0 || turn.task_commands.len() >= 256 {
        return ERROR;
    }
    turn.task_keys.remove(&id);
    turn.io_results.remove(&id);
    turn.network_permits.remove(&id);
    turn.socket_pending.remove(&id);
    turn.task_commands.push(TaskCommand::Cancel(id));
    OK
}

unsafe extern "C" fn validate_state(ctx: *mut c_void, key: u64, kind: u32) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    let actual = match turn.storage.slots.get(&key).map(Box::as_ref) {
        Some(Value::Integer(_)) => blockui_abi::I64,
        Some(Value::Boolean(_)) => blockui_abi::BOOLEAN,
        Some(Value::Float(_)) => blockui_abi::F64,
        Some(Value::Text(_)) => blockui_abi::TEXT,
        Some(Value::IntegerList(_)) => blockui_abi::I64_LIST,
        Some(Value::TextList(_)) => blockui_abi::TEXT_LIST,
        None => return ERROR,
    };
    if actual == kind { OK } else { ERROR }
}

#[cfg(test)]
mod state_validation_tests {
    use super::*;

    #[test]
    fn exact_kind_checks_do_not_subscribe_or_change_state() {
        let values = [
            Value::Integer(1),
            Value::Boolean(true),
            Value::Float(2.5),
            Value::Text("hello".into()),
            Value::IntegerList(vec![3]),
            Value::TextList(vec!["标题😀".into(), String::new()]),
        ];
        let kinds = [
            blockui_abi::I64,
            blockui_abi::BOOLEAN,
            blockui_abi::F64,
            blockui_abi::TEXT,
            blockui_abi::I64_LIST,
            blockui_abi::TEXT_LIST,
        ];
        let mut turn = Turn::default();
        for (index, value) in values.into_iter().enumerate() {
            turn.storage.slots.insert(index as u64 + 1, Box::new(value));
        }
        let before = turn.storage.clone();
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let api = context.api();
        for (index, expected) in kinds.into_iter().enumerate() {
            for supplied in kinds.into_iter().chain([0, u32::MAX]) {
                assert_eq!(
                    unsafe { (api.validate_state)(api.context, index as u64 + 1, supplied) },
                    if supplied == expected { OK } else { ERROR }
                );
            }
        }
        assert_eq!(
            unsafe { (api.validate_state)(api.context, 99, blockui_abi::I64) },
            ERROR
        );
        assert!(turn.reads.is_empty());
        assert!(turn.changed.is_empty());
        assert_eq!(turn.storage.slots, before.slots);
    }
}

unsafe extern "C" fn read_i64(ctx: *mut c_void, key: u64, out: *mut i64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    turn.reads.insert(Dependency::State(key));
    match turn.storage.slots.get(&key).map(Box::as_ref) {
        Some(Value::Boolean(value)) if !out.is_null() => {
            unsafe { *out = i64::from(*value) };
            OK
        }
        Some(Value::Integer(value)) if !out.is_null() => {
            unsafe { *out = *value };
            OK
        }
        _ => ERROR,
    }
}
unsafe extern "C" fn write_i64(ctx: *mut c_void, key: u64, value: i64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only {
        return ERROR;
    }
    match turn.storage.slots.get_mut(&key).map(Box::as_mut) {
        Some(Value::Boolean(old)) if value == 0 || value == 1 => {
            let value = value == 1;
            turn.conversions.remove(&key);
            if *old != value {
                if let Some(journal) = &mut turn.journal {
                    journal.entry(key).or_insert_with(|| Value::Boolean(*old));
                }
                *old = value;
                turn.changed.insert(key);
                *turn.revisions.entry(key).or_default() += 1;
                turn.graph.invalidate(Dependency::State(key));
            }
            OK
        }
        Some(Value::Integer(old)) => {
            turn.conversions.remove(&key);
            if *old != value {
                if let Some(journal) = &mut turn.journal {
                    journal.entry(key).or_insert_with(|| Value::Integer(*old));
                }
                *old = value;
                turn.changed.insert(key);
                *turn.revisions.entry(key).or_default() += 1;
                turn.graph.invalidate(Dependency::State(key));
            }
            OK
        }
        _ => ERROR,
    }
}
unsafe extern "C" fn read_f64(ctx: *mut c_void, key: u64, out: *mut f64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    match turn.storage.slots.get(&key).map(Box::as_ref) {
        Some(Value::Float(value)) if !out.is_null() => {
            unsafe {
                *out = *value;
            };
            turn.reads.insert(Dependency::State(key));
            OK
        }
        _ => ERROR,
    }
}
unsafe extern "C" fn write_f64(ctx: *mut c_void, key: u64, value: f64) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only {
        return ERROR;
    }
    let Some(Value::Float(old)) = turn.storage.slots.get_mut(&key).map(Box::as_mut) else {
        return ERROR;
    };
    turn.conversions.remove(&key);
    if old.to_bits() != value.to_bits() {
        if let Some(journal) = &mut turn.journal {
            journal.entry(key).or_insert(Value::Float(*old));
        }
        *old = value;
        turn.changed.insert(key);
        *turn.revisions.entry(key).or_default() += 1;
        turn.graph.invalidate(Dependency::State(key));
    }
    OK
}
unsafe extern "C" fn read_text(ctx: *mut c_void, key: u64, out: *mut u8, capacity: usize) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    turn.reads.insert(Dependency::State(key));
    match turn.storage.slots.get(&key).map(Box::as_ref) {
        Some(Value::Text(value)) => {
            if out.is_null() {
                return value.len() as isize;
            }
            if capacity < value.len() {
                return -1;
            }
            unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), out, value.len()) };
            value.len() as isize
        }
        _ => -1,
    }
}
unsafe fn copy_bytes(bytes: Bytes) -> Result<String, String> {
    if bytes.len > 1024 * 1024 || (bytes.ptr.is_null() && bytes.len != 0) {
        return Err("invalid byte buffer".into());
    }
    if bytes.len == 0 {
        return Ok(String::new());
    }
    String::from_utf8(unsafe { std::slice::from_raw_parts(bytes.ptr, bytes.len) }.to_vec())
        .map_err(|e| e.to_string())
}
unsafe extern "C" fn read_message_text(ctx: *mut c_void, out: *mut u8, capacity: usize) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    let Some(text) = turn.message_text.as_deref() else {
        return -1;
    };
    if out.is_null() {
        return text.len() as isize;
    }
    if capacity < text.len() {
        return -1;
    }
    unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), out, text.len()) };
    text.len() as isize
}
unsafe extern "C" fn read_request_error(ctx: *mut c_void, out: *mut u8, capacity: usize) -> isize {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &*context.turn };
    let text = turn.request_error.as_deref().unwrap_or("");
    if out.is_null() {
        return text.len() as isize;
    }
    if capacity < text.len() {
        return -1;
    }
    unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), out, text.len()) };
    text.len() as isize
}
unsafe extern "C" fn report_error(ctx: *mut c_void, bytes: Bytes) -> i32 {
    if bytes.len > 4096 {
        return ERROR;
    }
    let Ok(text) = (unsafe { copy_bytes(bytes) }) else {
        return ERROR;
    };
    let context = unsafe { &*(ctx as *const Context) };
    let turn = unsafe { &mut *context.turn };
    if turn.handler_error.is_none() {
        turn.handler_error = Some(text);
    }
    OK
}
unsafe extern "C" fn write_text(ctx: *mut c_void, key: u64, bytes: Bytes) -> i32 {
    let context = unsafe { &*ctx.cast::<Context>() };
    let turn = unsafe { &mut *context.turn };
    if turn.read_only {
        return ERROR;
    }
    let Ok(value) = (unsafe { copy_bytes(bytes) }) else {
        return ERROR;
    };
    match turn.storage.slots.get_mut(&key).map(Box::as_mut) {
        Some(Value::Text(old)) => {
            if *old != value {
                let Ok(dependencies) = turn.graph.map_reads.refresh_record(key, &value) else {
                    return ERROR;
                };
                for dependency in dependencies {
                    turn.graph.invalidate(dependency);
                }
                let previous = std::mem::replace(old, value);
                if let Some(journal) = &mut turn.journal {
                    journal.entry(key).or_insert(Value::Text(previous));
                }
                turn.changed.insert(key);
                *turn.revisions.entry(key).or_default() += 1;
                turn.graph.invalidate(Dependency::State(key));
            }
            turn.conversions.remove(&key);
            OK
        }
        _ => ERROR,
    }
}
// The table is borrowed only on the host call stack, never retained by graph/state.
struct Context {
    turn: *mut Turn,
    table: *const Implementation,
}
impl Context {
    fn api(&self) -> HostApi {
        HostApi {
            event_owner: unsafe { &*self.turn }.event_owner,
            call_service: application::call,
            read_text_list: text_list_state::read,
            read_text_map: keyed_map::read,
            write_text_list: text_list_state::write,
            previous_text_list: text_list_state::previous,
            report_error,
            read_request_error,
            send_actor_text: system::send_actor_text,
            read_message_text,
            reply_text: stage_text_reply,
            monotonic_ms,
            read_f64,
            write_f64,
            previous_f64: migration::float,
            computed_f64: graph::read_float,
            ask_actor: system::ask_actor,
            ask_actor_text: system::ask_actor_text,
            cancel_request: system::cancel_request,
            reply: stage_reply,
            child_actor: system::child_actor,
            send_actor: system::send_actor,
            context: (self as *const Self).cast_mut().cast(),
            read_i64,
            write_i64,
            read_text,
            write_text,
            computed_i64: graph::read_integer,
            computed_text: graph::read_string,
            computed_list: graph::read_list,
            computed_text_list: graph::read_text_list,
            delay: request_delay,
            interval: request_interval,
            cancel_task: request_cancel_task,
            open_file: resources::open,
            close_file: resources::close,
            read_file: resources::read,
            read_file_async: io::request,
            write_file_async: io::request_write,
            save_file_async: io::request_save,
            create_file_async: io::request_create,
            connect_socket_async: network_task::connect,
            read_socket_async: network_task::read,
            write_socket_async: network_task::write,
            close_socket: network_task::close,
            task_result: io::result,
            previous_i64: migration::integer,
            previous_text: migration::text,
            read_list: list_state::read,
            write_list: list_state::write,
            previous_list: list_state::previous,
            validate_state,
            read_list_range: list_state::read_range,
            computed_list_range: graph::read_list_range,
            read_record_field: record_fields::read,
            read_record_map_field: keyed_map::read_record_field,
        }
    }
}
fn with_api<R>(turn: &mut Turn, table: *const Implementation, f: impl FnOnce(&HostApi) -> R) -> R {
    turn.graph.fault = None;
    turn.handler_error = None;
    let context = Context { turn, table };
    f(&context.api())
}

// Preserve eager symbol resolution: unresolved imports reject the candidate
// before any actor turn. libloading::Library::new uses lazy binding on Unix.
#[cfg(not(target_arch = "wasm32"))]
unsafe fn open_library(path: &Path) -> Result<libloading::Library, libloading::Error> {
    #[cfg(unix)]
    {
        unsafe {
            libloading::os::unix::Library::open(
                Some(path),
                libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_LOCAL,
            )
        }
        .map(Into::into)
    }
    #[cfg(not(unix))]
    unsafe {
        libloading::Library::new(path)
    }
}
fn next_event_owner() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.try_update(
        std::sync::atomic::Ordering::Relaxed,
        std::sync::atomic::Ordering::Relaxed,
        |value| value.checked_add(1),
    )
    .expect("UI implementation identity exhausted")
}
struct Loaded {
    event_owner: u64,
    scopes: std::cell::RefCell<BTreeSet<u64>>,
    bindings: std::cell::RefCell<bindings::Registry>,
    #[cfg(not(target_arch = "wasm32"))]
    library: Option<libloading::Library>,
    table: *const Implementation,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    path: PathBuf,
    version: u64,
}
impl Loaded {
    #[cfg(target_arch = "wasm32")]
    fn open(_: &Path, _: &Path, _: u64) -> Result<Self, String> {
        Err("dynamic loading is unavailable in browser builds".into())
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn open(source: &Path, dir: &Path, generation: u64) -> Result<Self, String> {
        let bytes = artifact::read(source)?;
        let version = fingerprint(&bytes);
        let path = dir.join(format!(
            "impl-{generation}-{version:016x}{}",
            std::env::consts::DLL_SUFFIX
        ));
        artifact::write(&path, &bytes)?;
        let library = match unsafe { open_library(&path) } {
            Ok(library) => library,
            Err(error) => {
                let _ = fs::remove_file(path);
                return Err(error.to_string());
            }
        };
        let mut loaded = Self {
            event_owner: 0,
            scopes: Default::default(),
            bindings: Default::default(),
            library: Some(library),
            table: std::ptr::null(),
            path,
            version,
        };
        // Trusted plugin exports this exact entry signature, with a static table.
        // The symbol is borrowed only for this call; Loaded owns the library for
        // every subsequent table use, including candidate preparation.
        loaded.table = {
            let entry = unsafe {
                loaded
                    .library
                    .as_ref()
                    .unwrap()
                    .get::<unsafe extern "C" fn() -> *const Implementation>(b"actor_entry")
            }
            .map_err(|error| error.to_string())?;
            unsafe { entry() }
        };
        if loaded.table.is_null() {
            return Err("null implementation".into());
        }
        let header = unsafe { &*loaded.table.cast::<ImplementationHeader>() };
        if header.abi_version != ABI_VERSION
            || header.struct_size != std::mem::size_of::<Implementation>()
        {
            return Err(format!(
                "incompatible ABI: expected version {ABI_VERSION}, table size {}; found version {}, table size {}",
                std::mem::size_of::<Implementation>(),
                header.abi_version,
                header.struct_size,
            ));
        }
        let table = loaded.table();
        if !matches!(table.panic_strategy, PANIC_UNWIND | PANIC_ABORT) {
            return Err(format!(
                "unknown implementation panic strategy {}",
                table.panic_strategy
            ));
        }
        if table.slot_count > 256 || (table.slot_count != 0 && table.slots.is_null()) {
            return Err("invalid schema".into());
        }
        loaded.event_owner = next_event_owner();
        Ok(loaded)
    }
    fn events(&self, operation: u32) {
        if self.event_owner != 0 {
            assert_eq!(
                unsafe { (self.table().events)(self.event_owner, operation, std::ptr::null(), 0) },
                OK,
                "implementation event lifecycle failed"
            );
        }
    }
    fn table(&self) -> &Implementation {
        unsafe { &*self.table }
    }
}
impl Drop for Loaded {
    fn drop(&mut self) {
        self.events(4);
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(library) = self.library.take() {
            // Close before deleting the snapshot (required by Windows loaders).
            drop(library);
            let _ = fs::remove_file(&self.path);
        }
    }
}
fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |h, b| {
        (h ^ *b as u64).wrapping_mul(0x100000001b3)
    })
}

#[derive(Clone, Debug)]
struct Field {
    kind: u32,
    default: Value,
}
fn schema(loaded: &Loaded) -> Result<BTreeMap<u64, Field>, String> {
    let table = loaded.table();
    let specs = if table.slot_count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(table.slots, table.slot_count) }
    };
    let mut fields = BTreeMap::new();
    for spec in specs {
        let default = match spec.kind {
            F64 => Value::Float(f64::from_bits(spec.default_i64 as u64)),
            I64 => Value::Integer(spec.default_i64),
            BOOLEAN if matches!(spec.default_i64, 0 | 1) => Value::Boolean(spec.default_i64 == 1),
            I64_LIST if spec.default_text.len.is_multiple_of(8) => Value::IntegerList(unsafe {
                list_state::copy(spec.default_text.ptr.cast(), spec.default_text.len / 8)
            }?),
            TEXT => Value::Text(unsafe { copy_bytes(spec.default_text) }?),
            TEXT_LIST
                if spec.default_i64 == 0
                    && spec
                        .default_text
                        .len
                        .is_multiple_of(std::mem::size_of::<Bytes>()) =>
            {
                Value::TextList(unsafe {
                    text_list_state::copy(
                        spec.default_text.ptr.cast(),
                        spec.default_text.len / std::mem::size_of::<Bytes>(),
                    )
                }?)
            }
            _ => return Err("unsupported slot type".into()),
        };
        if spec.key == 0
            || fields
                .insert(
                    spec.key,
                    Field {
                        kind: spec.kind,
                        default,
                    },
                )
                .is_some()
        {
            return Err("duplicate/zero state key".into());
        }
    }
    Ok(fields)
}
fn schema_version(fields: &BTreeMap<u64, Field>) -> u64 {
    let bytes: Vec<u8> = fields
        .iter()
        .flat_map(|(key, value)| {
            key.to_le_bytes()
                .into_iter()
                .chain(value.kind.to_le_bytes())
        })
        .collect();
    fingerprint(&bytes)
}

/// Host-owned description of state schema changes; no plugin pointers are retained.
#[derive(Clone, Debug, PartialEq)]
pub enum StateChange {
    Retained {
        key: u64,
        kind: u32,
    },
    Added {
        key: u64,
        default: Value,
    },
    Removed {
        key: u64,
        kind: u32,
    },
    TypeChanged {
        key: u64,
        from: u32,
        to: u32,
        default: Value,
    },
}
/// Metadata-only preview. Migration/view execution and resource admission can still fail.
#[derive(Clone, Debug, PartialEq)]
pub struct ReloadPreview {
    pub implementation_version: u64,
    pub schema_version: u64,
    pub compatible: bool,
    pub has_migration: bool,
    pub changes: Vec<StateChange>,
}
impl ReloadPreview {
    /// Type conversions require explicit migration in this runtime.
    pub fn requires_migration(&self) -> bool {
        self.changes
            .iter()
            .any(|change| matches!(change, StateChange::TypeChanged { .. }))
    }
}

fn state_kind_name(kind: u32) -> &'static str {
    match kind {
        F64 => "f64",
        I64 => "i64",
        TEXT => "String",
        BOOLEAN => "bool",
        I64_LIST => "Vec<i64>",
        TEXT_LIST => "Vec<String>",
        _ => "unknown",
    }
}
fn state_value_kind_name(value: &Value) -> &'static str {
    match value {
        Value::Float(_) => "f64",
        Value::Integer(_) => "i64",
        Value::Text(_) => "String",
        Value::Boolean(_) => "bool",
        Value::IntegerList(_) => "Vec<i64>",
        Value::TextList(_) => "Vec<String>",
    }
}

impl std::fmt::Display for ReloadPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "preview implementation={:016x} schema={:016x} compatible={} has_migration={} requires_migration={}",
            self.implementation_version,
            self.schema_version,
            self.compatible,
            self.has_migration,
            self.requires_migration()
        )?;
        for change in &self.changes {
            match change {
                StateChange::Retained { key, kind } => {
                    writeln!(f, "retained key={key:016x} kind={}", state_kind_name(*kind))
                }
                StateChange::Added { key, default } => {
                    writeln!(
                        f,
                        "added key={key:016x} kind={} default={default:?}",
                        state_value_kind_name(default)
                    )
                }
                StateChange::Removed { key, kind } => {
                    writeln!(f, "removed key={key:016x} kind={}", state_kind_name(*kind))
                }
                StateChange::TypeChanged {
                    key,
                    from,
                    to,
                    default,
                } => writeln!(
                    f,
                    "converted key={key:016x} from={} to={} default={default:?} migration=explicit-required",
                    state_kind_name(*from),
                    state_kind_name(*to)
                ),
            }?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub node: Node,
    pub text: String,
    pub icon: Option<std::sync::Arc<blockui_abi::icon::IconData>>,
    pub semantic_label: Option<std::sync::Arc<str>>,
    pub dependencies: BTreeSet<Dependency>,
    pub versions: BTreeMap<Dependency, u64>,
    /// Text and style are independent reactive observers on this node.
    pub text_dependencies: BTreeSet<Dependency>,
    pub style_dependencies: BTreeSet<Dependency>,
    pub evaluations: u64,
    pub style_evaluations: u64,
}
// Debug printing keys should not expose borrowed plugin pointers; Node has no pointers.
pub const MAX_RENDERER_CACHE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RendererState {
    pub text: String,
    pub selection: (i32, i32),
    /// True after a native editor selection has been captured.
    pub selection_saved: bool,
    pub focus: bool,
    /// Stable-host focus sequence used to choose among nested remembered scopes.
    pub focus_epoch: u64,
    pub scroll: f64,
    /// Horizontal logical-pixel offset; retained independently of plugin code.
    pub scroll_x: f64,
    /// Last row-major cell focus for a grid's single Tab stop.
    pub grid_focus: Option<i64>,
}
pub(crate) struct PreparedReload {
    candidate: Loaded,
    fields: BTreeMap<u64, Field>,
    version: u64,
    graph: Graph,
    staged: Option<Turn>,
    bindings: Projection,
    admission: Option<Box<dyn admission::Admission>>,
    files: BTreeMap<u64, resources::HostFile>,
    restart_timers: BTreeMap<u64, PreparedTimer>,
}

struct ProjectedBinding {
    node: Node,
    // None defers moving an unchanged host binding until admission succeeds.
    value: Option<Binding>,
}
struct Projection {
    root_evaluated: bool,
    scope_dependencies: BTreeMap<u64, ScopeDependencies>,
    root_dependencies: BTreeSet<Dependency>,
    recipes: bindings::Registry,
    bindings: Vec<ProjectedBinding>,
    dependencies: BTreeSet<Dependency>,
    versions: BTreeMap<Dependency, u64>,
}
#[derive(Default)]
pub struct View {
    root_evaluations: u64,
    pub scope_dependencies: BTreeMap<u64, ScopeDependencies>,
    pub root_dependencies: BTreeSet<Dependency>,
    dependents: BTreeMap<Dependency, BTreeSet<u64>>,
    pub dependencies: BTreeSet<Dependency>,
    versions: BTreeMap<Dependency, u64>,
    pub bindings: BTreeMap<u64, Binding>,
    pub renderer: BTreeMap<u64, RendererState>,
    pub order: Vec<u64>,
    renderer_retention: usize,
    pub(crate) renderer_boundaries: BTreeSet<u64>,
    retained_renderer: std::collections::VecDeque<(u64, u32, u64, RendererState)>,
}
unsafe extern "C" fn emit(ctx: *mut c_void, node: Node) -> i32 {
    let collection = unsafe { &mut *ctx.cast::<bindings::Collection>() };
    let nodes = &mut collection.nodes;
    if nodes.len() >= blockui_abi::MAX_VIEW_NODES {
        collection.error = Some("view exceeds node budget".into());
        return ERROR;
    }
    nodes.push(node);
    OK
}
unsafe extern "C" fn write(ctx: *mut c_void, bytes: Bytes) -> i32 {
    let output = unsafe { &mut *ctx.cast::<String>() };
    match unsafe { copy_bytes(bytes) } {
        Ok(value) => {
            output.push_str(&value);
            OK
        }
        Err(_) => ERROR,
    }
}
unsafe extern "C" fn write_presentation(ctx: *mut c_void, bytes: blockui_abi::Bytes) -> i32 {
    let output = unsafe { &mut *ctx.cast::<Option<blockui_abi::Presentation>>() };
    if output.is_some() || bytes.len != blockui_abi::Presentation::WIRE_BYTES || bytes.ptr.is_null()
    {
        return ERROR;
    }
    *output = blockui_abi::Presentation::decode(unsafe {
        std::slice::from_raw_parts(bytes.ptr, bytes.len)
    });
    if output.is_some() { OK } else { ERROR }
}

struct Rendered {
    text: String,
    presentation: Option<blockui_abi::Presentation>,
    dependencies: BTreeSet<Dependency>,
    text_dependencies: BTreeSet<Dependency>,
    style_dependencies: BTreeSet<Dependency>,
    text_evaluated: bool,
    style_evaluated: bool,
}
fn render(
    loaded: &Loaded,
    turn: &mut Turn,
    key: u64,
    recipes: &bindings::Registry,
    previous: Option<&Binding>,
) -> Result<Rendered, String> {
    let recipe = recipes.get(&key).ok_or("missing registered binding")?;
    let changed = |deps: &BTreeSet<Dependency>, old: &Binding| {
        let current = graph::versions(turn, deps);
        deps.iter()
            .any(|dep| old.versions.get(dep) != current.get(dep))
    };
    let text_evaluated = previous.is_none_or(|old| changed(&old.text_dependencies, old));
    let style_evaluated = matches!(recipe, bindings::Recipe::Styled(..))
        && previous.is_none_or(|old| changed(&old.style_dependencies, old));
    turn.read_only = true;
    turn.reads.clear();
    turn.graph.fault = None;
    turn.handler_error = None;
    let mut output = String::new();
    let mut code = OK;
    let mut observer = "text";
    let text_dependencies = if text_evaluated {
        code = bindings::evaluate(recipe.text(), loaded, turn, &mut output);
        std::mem::take(&mut turn.reads)
    } else {
        let old = previous.unwrap();
        output.clone_from(&old.text);
        old.text_dependencies.clone()
    };
    let mut presentation = previous
        .filter(|_| matches!(recipe, bindings::Recipe::Styled(..)))
        .map(|old| old.node.presentation);
    let style_dependencies = if code == OK && style_evaluated {
        let bindings::Recipe::Styled(_, token) = recipe else {
            unreachable!()
        };
        presentation = None;
        observer = "style";
        code = with_api(turn, loaded.table, |api| unsafe {
            (loaded.table().render_binding)(
                api,
                blockui_abi::BIND_STYLE,
                0,
                *token,
                std::ptr::from_mut(&mut presentation).cast(),
                write_presentation,
            )
        });
        if presentation.is_none() {
            code = ERROR;
        }
        std::mem::take(&mut turn.reads)
    } else {
        previous
            .map(|old| old.style_dependencies.clone())
            .unwrap_or_default()
    };
    turn.read_only = false;
    if code != OK || turn.graph.fault.is_some() || output.len() > 1024 * 1024 {
        let reason = turn
            .graph
            .fault
            .clone()
            .or_else(|| turn.handler_error.clone());
        let reason = reason.unwrap_or_else(|| {
            if output.len() > 1024 * 1024 {
                observer = "text";
                "text output exceeds 1048576-byte limit".into()
            } else {
                "view computation failed".into()
            }
        });
        return Err(format!(
            "{observer} binding {key:#018x} evaluation failed: {reason}"
        ));
    }
    let dependencies = text_dependencies
        .union(&style_dependencies)
        .copied()
        .collect();
    Ok(Rendered {
        text: output,
        presentation,
        dependencies,
        text_dependencies,
        style_dependencies,
        text_evaluated,
        style_evaluated,
    })
}

fn project(
    loaded: &Loaded,
    turn: &mut Turn,
    previous: Option<&BTreeMap<u64, Binding>>,
) -> Result<Projection, String> {
    project_with_scopes(loaded, turn, previous, None)
}
fn project_with_scopes(
    loaded: &Loaded,
    turn: &mut Turn,
    previous: Option<&BTreeMap<u64, Binding>>,
    previous_scopes: Option<&View>,
) -> Result<Projection, String> {
    loaded.events(1);
    let owner = std::mem::replace(&mut turn.event_owner, loaded.event_owner);
    let result = project_with_scopes_inner(loaded, turn, previous, previous_scopes);
    turn.event_owner = owner;
    if result.is_err() {
        loaded.events(3);
    }
    result
}
fn project_with_scopes_inner(
    loaded: &Loaded,
    turn: &mut Turn,
    previous: Option<&BTreeMap<u64, Binding>>,
    previous_scopes: Option<&View>,
) -> Result<Projection, String> {
    let mut collection = bindings::Collection::default();
    turn.read_only = true;
    turn.reads.clear();
    let code = with_api(turn, loaded.table, |api| unsafe {
        (loaded.table().view)(
            api,
            (&mut collection as *mut bindings::Collection).cast(),
            emit,
            bindings::emit_recipe,
        )
    });
    let root_dependencies = turn.reads.clone();
    let expanded = if code == OK && turn.graph.fault.is_none() && collection.error.is_none() {
        if previous_scopes.is_some() {
            scopes::expand_cached(
                loaded,
                turn,
                &mut collection,
                &BTreeSet::new(),
                previous_scopes,
            )
        } else {
            scopes::expand(loaded, turn, &mut collection)
        }
    } else {
        Ok(BTreeMap::new())
    };
    turn.read_only = false;
    let scope_dependencies = expanded?;
    if code != OK || turn.graph.fault.is_some() || collection.error.is_some() {
        return Err(collection
            .error
            .or_else(|| turn.graph.fault.clone())
            .or_else(|| turn.handler_error.clone())
            .unwrap_or_else(|| "view failed".into()));
    }
    let dependencies = std::mem::take(&mut turn.reads);
    project_collection(
        loaded,
        turn,
        collection,
        dependencies,
        root_dependencies,
        scope_dependencies,
        previous.map(|bindings| ProjectionCache {
            bindings,
            retained: None,
        }),
    )
    .map(|mut projection| {
        projection.root_evaluated = true;
        projection
    })
}

struct ProjectionCache<'a> {
    bindings: &'a BTreeMap<u64, Binding>,
    // Nodes carried over without invoking their builder keep anonymous recipes too.
    // Re-emitted anonymous callbacks must still run, even if their token is unchanged.
    retained: Option<&'a BTreeSet<u64>>,
}

#[cold]
fn presentation_error(node: &Node, field: &str, requirement: std::fmt::Arguments<'_>) -> String {
    let kind = match node.kind {
        COLUMN => "Column",
        ROW => "Row",
        SCROLL => "Scroll",
        LABEL => "label",
        BUTTON => "Button",
        ICON => "Icon",
        ENTRY => "TextInput",
        TEXTAREA => "TextArea",
        CHECKBOX => "Checkbox",
        SLIDER => "Slider",
        SELECT => "Select",
        CANVAS => "Canvas",
        DIALOG => "Dialog",
        PROGRESS => "Progress",
        _ => "widget",
    };
    format!(
        "invalid widget presentation on {kind} node {:#018x}: {field} {requirement}",
        node.key
    )
}

fn validate_presentation(node: &Node) -> Result<(), String> {
    let appearance = node.presentation;
    for (field, value, limit) in [
        ("width", appearance.width, 10000.0),
        ("max_width", appearance.max_width, 10000.0),
        ("font_size", appearance.font_size, 96.0),
        ("radius", appearance.radius, 48.0),
    ] {
        if !value.is_finite() || value < 0.0 || value > limit {
            return Err(presentation_error(
                node,
                field,
                format_args!("must be finite and within 0..={limit}; got {value}"),
            ));
        }
    }
    for (field, value, limit) in [
        ("text_tone", appearance.text_tone, 3),
        ("surface", appearance.surface, 2),
        ("theme", appearance.theme, 3),
        ("density", appearance.density, 2),
        ("fill_height", appearance.fill_height, 1),
    ] {
        if value > limit {
            return Err(presentation_error(
                node,
                field,
                format_args!("must be within 0..={limit}; got {value}"),
            ));
        }
    }
    if appearance.fill_height != 0 && !matches!(node.kind, COLUMN | ROW | SCROLL) {
        return Err(presentation_error(
            node,
            "fill_height",
            format_args!("requires a Column, Row or Scroll"),
        ));
    }
    if appearance.font_size != 0.0 && !matches!(node.kind, LABEL | BUTTON) {
        return Err(presentation_error(
            node,
            "font_size",
            format_args!("requires a label or Button"),
        ));
    }
    if appearance.text_tone != 0 && !matches!(node.kind, LABEL | ICON) {
        return Err(presentation_error(
            node,
            "text_tone",
            format_args!("requires a label or Icon"),
        ));
    }
    if !matches!(node.kind, COLUMN | ROW) {
        if appearance.surface != 0 {
            return Err(presentation_error(
                node,
                "surface",
                format_args!("requires a Column or Row"),
            ));
        }
        if appearance.radius != 0.0 {
            return Err(presentation_error(
                node,
                "radius",
                format_args!("requires a Column or Row"),
            ));
        }
    }
    Ok(())
}

fn project_collection(
    loaded: &Loaded,
    turn: &mut Turn,
    collection: bindings::Collection,
    dependencies: BTreeSet<Dependency>,
    root_dependencies: BTreeSet<Dependency>,
    scope_dependencies: BTreeMap<u64, ScopeDependencies>,
    previous: Option<ProjectionCache<'_>>,
) -> Result<Projection, String> {
    let nodes = collection.nodes;
    let structure_versions = graph::versions(turn, &dependencies);
    let mut node_index: Vec<_> = nodes.iter().collect();
    node_index.sort_unstable_by_key(|node| node.key);
    if node_index.first().is_some_and(|node| node.key == 0) {
        return Err("missing view identity; provide an explicit key for this widget".into());
    }
    for pair in node_index.windows(2) {
        if pair[0].key == pair[1].key {
            return Err(format!(
                "duplicate view identity {:#x}; provide distinct explicit keys for repeated widgets",
                pair[0].key
            ));
        }
    }
    let parent_node = |key| {
        node_index
            .binary_search_by_key(&key, |node| node.key)
            .ok()
            .map(|index| node_index[index])
    };
    if nodes.iter().filter(|node| node.kind == DIALOG).count() > 1 {
        return Err("only one modal dialog per actor view is supported".into());
    }
    let mut shortcuts = BTreeSet::new();
    let parents: BTreeSet<_> = nodes.iter().map(|node| node.parent).collect();
    let mut choice_counts = BTreeMap::<(u64, u32), (u32, u32)>::new();
    for node in &nodes {
        if !matches!(
            node.kind,
            COLUMN
                | ROW
                | LABEL
                | BUTTON
                | CHECKBOX
                | SELECT
                | SLIDER
                | PROGRESS
                | CANVAS
                | ICON
                | DIALOG
                | ENTRY
                | TEXTAREA
                | SCROLL
                | CHILD
        ) {
            return Err("unsupported widget kind".into());
        }
        if [node.gap, node.padding, node.height, node.content_width]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0 || *v > 10000.0)
        {
            return Err("invalid widget dimensions".into());
        }
        if node.kind != SCROLL && node.content_width != 0.0 {
            return Err("content width requires a scroll container".into());
        }
        if node.parent != 0
            && parent_node(node.parent)
                .is_none_or(|p| !matches!(p.kind, COLUMN | ROW | DIALOG | SCROLL))
        {
            return Err("invalid parent".into());
        }
        let valid_position = match node.semantics.role {
            semantics::TOAST_VIEWPORT => node.overlay_position <= 5,
            semantics::POPOVER_PANEL | semantics::MENU_PANEL | semantics::TOOLTIP => {
                node.overlay_position <= 11
            }
            _ => node.overlay_position == 0,
        };
        if !valid_position {
            return Err("overlay position does not match its viewport role".into());
        }
        if node.kind != BUTTON
            && node.action_payload != 0
            && !(node.semantics.role == semantics::TOAST_VIEWPORT && node.action_payload == 1)
        {
            return Err("action payload requires a button or boolean toast activity".into());
        }
        validate_presentation(node)?;
        node.semantics.validate(node.kind)?;
        if node.semantics.role == semantics::TOAST_VIEWPORT {
            if node.parent == 0 {
                return Err("notification viewport needs an application root".into());
            }
            let mut ancestor = node.parent;
            let mut remaining = nodes.len();
            while let Some(parent) = parent_node(ancestor) {
                if remaining == 0 {
                    return Err("cyclic view tree".into());
                }
                remaining -= 1;
                if parent.kind == DIALOG || parent.semantics.role == semantics::TOAST_VIEWPORT {
                    return Err("notification viewport cannot nest in a dialog or viewport".into());
                }
                ancestor = parent.parent;
            }
        }
        let parent_role = parent_node(node.parent).map(|p| p.semantics.role);
        if parent_role == Some(semantics::MENUBAR) {
            let children: Vec<_> = nodes
                .iter()
                .filter(|child| child.parent == node.key)
                .collect();
            if node.kind != COLUMN
                || node.semantics.role != semantics::DEFAULT
                || children.len() != 2
                || children
                    .iter()
                    .filter(|child| child.semantics.role == semantics::MENU_TRIGGER)
                    .count()
                    != 1
                || children
                    .iter()
                    .filter(|child| child.semantics.role == semantics::MENU_PANEL)
                    .count()
                    != 1
            {
                return Err("menubar children must be root menu containers".into());
            }
        }

        if (node.semantics.role == semantics::TABLE_ROW && parent_role != Some(semantics::TABLE))
            || (matches!(
                node.semantics.role,
                semantics::COLUMN_HEADER | semantics::CELL
            ) && parent_role != Some(semantics::TABLE_ROW))
            || (parent_role == Some(semantics::TABLE_ROW)
                && !matches!(
                    node.semantics.role,
                    semantics::COLUMN_HEADER | semantics::CELL
                ))
            || (parent_role == Some(semantics::TABLE)
                && node.semantics.role != semantics::TABLE_ROW
                && parent_node(node.parent).is_none_or(|p| p.semantics.labelled_by != node.key))
        {
            return Err("invalid table row or cell parent".into());
        }
        if semantics::is_menu_item(node.semantics.role)
            && (!matches!(
                parent_role,
                Some(semantics::MENU_PANEL | semantics::MENU_GROUP | semantics::SUBMENU)
            ) || node.action == 0)
        {
            return Err("menu item needs a menu parent and current action".into());
        }
        if parent_role == Some(semantics::MENU_PANEL)
            && !matches!(
                node.semantics.role,
                semantics::MENU_ITEM
                    | semantics::MENU_CHECKBOX
                    | semantics::MENU_RADIO
                    | semantics::HORIZONTAL_SEPARATOR
                    | semantics::MENU_GROUP
                    | semantics::SUBMENU
            )
        {
            return Err("command menu contains a non-item child".into());
        }
        if node.semantics.role == semantics::SUBMENU && parent_role != Some(semantics::MENU_PANEL) {
            return Err("submenu needs a menu parent".into());
        }
        if node.semantics.role == semantics::SUBMENU_TRIGGER
            && parent_role != Some(semantics::SUBMENU)
        {
            return Err("submenu trigger needs its wrapper".into());
        }
        if parent_role == Some(semantics::SUBMENU)
            && !matches!(
                node.semantics.role,
                semantics::SUBMENU_TRIGGER | semantics::MENU_PANEL
            )
        {
            return Err("submenu contains an invalid child".into());
        }
        if node.semantics.role == semantics::MENU_GROUP
            && (parent_role != Some(semantics::MENU_PANEL)
                || !parent_node(node.semantics.labelled_by).is_some_and(|label| {
                    label.parent == node.key && label.semantics.role == semantics::MENU_LABEL
                }))
        {
            return Err("menu group needs a menu parent and owned title".into());
        }
        if node.semantics.role == semantics::MENU_LABEL
            && parent_role != Some(semantics::MENU_GROUP)
        {
            return Err("menu title needs a group parent".into());
        }
        if parent_role == Some(semantics::MENU_GROUP)
            && !semantics::is_menu_item(node.semantics.role)
            && node.semantics.role != semantics::MENU_LABEL
        {
            return Err("menu group contains a non-item child".into());
        }
        if node.semantics.hidden != 0 && parents.contains(&node.key) {
            return Err("hidden panel must have no children".into());
        }
        for related in [
            node.semantics.controls,
            node.semantics.labelled_by,
            node.semantics.described_by,
            node.tooltip_anchor,
        ] {
            if related != 0 && (related == node.key || parent_node(related).is_none()) {
                return Err("invalid semantic relationship".into());
            }
        }
        if matches!(node.semantics.role, semantics::TAB | semantics::RADIO) {
            let group = if node.semantics.role == semantics::TAB {
                semantics::TAB_LIST
            } else {
                semantics::RADIO_GROUP
            };
            if parent_node(node.parent).is_none_or(|parent| parent.semantics.role != group) {
                return Err("choice requires its matching group".into());
            }
            let counts = choice_counts
                .entry((node.parent, node.semantics.role))
                .or_default();
            counts.0 += node.semantics.selected;
            counts.1 += node.semantics.tab_stop;
            if counts.0 > 1 || counts.1 > 1 || (node.disabled != 0 && node.semantics.tab_stop != 0)
            {
                return Err("invalid choice selection or Tab entry".into());
            }
        }
        if node.semantics.role == semantics::TAB
            && parent_node(node.semantics.controls).is_none_or(|panel| {
                panel.semantics.role != semantics::TAB_PANEL
                    || panel.semantics.labelled_by != node.key
                    || (panel.semantics.hidden == 0) != (node.semantics.selected != 0)
            })
        {
            return Err("tab needs a matching panel".into());
        }
        if node.semantics.role == semantics::TAB_PANEL
            && parent_node(node.semantics.labelled_by).is_none_or(|tab| {
                tab.semantics.role != semantics::TAB || tab.semantics.controls != node.key
            })
        {
            return Err("panel needs its matching tab".into());
        }
        if node.semantics.expanded != 0
            && node.semantics.controls != 0
            && !semantics::is_anchored_trigger(node.semantics.role)
            && parent_node(node.semantics.controls).is_none_or(|panel| {
                panel.semantics.role != semantics::DISCLOSURE_PANEL
                    || panel.semantics.labelled_by != node.key
                    || (panel.semantics.hidden == 0) != (node.semantics.expanded == 2)
            })
        {
            return Err("disclosure needs its matching panel".into());
        }
        if node.semantics.role == semantics::DISCLOSURE_PANEL
            && parent_node(node.semantics.labelled_by).is_none_or(|trigger| {
                trigger.semantics.expanded == 0 || trigger.semantics.controls != node.key
            })
        {
            return Err("disclosure panel needs its matching trigger".into());
        }
        if node.semantics.role == semantics::TOOLTIP {
            if node.parent == 0
                || node.action != 0
                || node.slot != 0
                || parents.contains(&node.key)
                || node.tooltip_delay_ms > 10000
                || parent_node(node.tooltip_anchor).is_none_or(|trigger| {
                    trigger.semantics.described_by != node.key || trigger.parent != node.parent
                })
            {
                return Err(
                    "tooltip needs a matching sibling trigger and noninteractive label".into(),
                );
            }
        } else if node.tooltip_anchor != 0 || node.tooltip_delay_ms != 0 {
            return Err("tooltip timing and anchor require a tooltip".into());
        }
        if node.semantics.described_by != 0
            && parent_node(node.semantics.described_by).is_none_or(|hint| {
                hint.semantics.role != semantics::TOOLTIP || hint.tooltip_anchor != node.key
            })
        {
            return Err("description needs its matching tooltip".into());
        }
        if semantics::is_anchored_trigger(node.semantics.role)
            && parent_node(node.semantics.controls).is_none_or(|panel| {
                panel.semantics.role
                    != if matches!(
                        node.semantics.role,
                        semantics::MENU_TRIGGER
                            | semantics::SUBMENU_TRIGGER
                            | semantics::CONTEXT_TRIGGER
                            | semantics::CONTEXT_SURFACE
                    ) {
                        semantics::MENU_PANEL
                    } else {
                        semantics::POPOVER_PANEL
                    }
                    || panel.semantics.labelled_by != node.key
                    || (panel.semantics.hidden == 0) != (node.semantics.expanded == 2)
            })
        {
            return Err("popover needs its matching panel".into());
        }
        if semantics::is_anchored_panel(node.semantics.role) {
            if node.parent == 0 || node.action == 0 || node.slot != 0 {
                return Err("popover needs a parent, close action and no state slot".into());
            }
            if parent_node(node.semantics.labelled_by).is_none_or(|trigger| {
                let compatible = if node.semantics.role == semantics::MENU_PANEL {
                    matches!(
                        trigger.semantics.role,
                        semantics::MENU_TRIGGER
                            | semantics::SUBMENU_TRIGGER
                            | semantics::CONTEXT_TRIGGER
                            | semantics::CONTEXT_SURFACE
                    )
                } else {
                    trigger.semantics.role == semantics::POPOVER_TRIGGER
                };
                !compatible || trigger.semantics.controls != node.key
            }) {
                return Err("popover panel needs its matching trigger".into());
            }
            let mut anchor = node.semantics.labelled_by;
            for _ in 0..nodes.len() {
                if anchor == node.key {
                    return Err("popover anchor cannot be inside its panel".into());
                }
                let Some(parent) = parent_node(anchor) else {
                    break;
                };
                anchor = parent.parent;
            }
        }
        if node.semantics.role == semantics::ACCORDION_ITEM
            && parent_node(node.parent)
                .is_none_or(|parent| parent.semantics.role != semantics::ACCORDION)
        {
            return Err("accordion item requires its matching group".into());
        }
        if node.semantics.role == semantics::ACCORDION_TRIGGER {
            let heading = parent_node(node.parent).ok_or("accordion trigger needs a heading")?;
            let item = parent_node(heading.parent).ok_or("accordion trigger needs an item")?;
            if heading.semantics.role != semantics::HEADING
                || item.semantics.role != semantics::ACCORDION_ITEM
                || parent_node(node.semantics.controls).is_none_or(|panel| panel.parent != item.key)
            {
                return Err("accordion trigger needs a heading and matching item panel".into());
            }
        }
        let invalid_variant = if node.semantics.role == semantics::BADGE {
            node.button_variant > 3
        } else {
            node.button_variant > 3 || (node.kind != BUTTON && node.button_variant != 0)
        };
        if invalid_variant {
            return Err("invalid button appearance".into());
        }
        if matches!(
            node.semantics.role,
            semantics::HORIZONTAL_SEPARATOR | semantics::VERTICAL_SEPARATOR
        ) && (node.height <= 0.0
            || node.action != 0
            || node.slot != 0
            || parents.contains(&node.key))
        {
            return Err("separator needs positive height and no children, action or slot".into());
        }
        if node.semantics.role == semantics::BADGE && node.action != 0 {
            return Err("badge cannot have an action".into());
        }
        if node.read_only > 1 || (node.read_only != 0 && !matches!(node.kind, ENTRY | TEXTAREA)) {
            return Err("invalid read-only widget state".into());
        }
        if node.disabled > 1
            || (node.disabled != 0
                && node.semantics.role != semantics::CONTEXT_SURFACE
                && !matches!(
                    node.kind,
                    BUTTON | CHECKBOX | SELECT | SLIDER | ENTRY | TEXTAREA
                ))
        {
            return Err("invalid disabled widget state".into());
        }
        if node.shortcut != 0 {
            if node.kind != BUTTON || !(b'a' as u32..=b'z' as u32).contains(&node.shortcut) {
                return Err("primary shortcut requires a button and lowercase ASCII letter".into());
            }
            if !shortcuts.insert(node.shortcut) {
                return Err("duplicate primary shortcut in actor view".into());
            }
        }
        if node.kind == DIALOG && (node.action == 0 || node.slot != 0) {
            return Err("dialog needs a close action and no state slot".into());
        }
        if node.focus_boundary > 1
            || (node.focus_boundary != 0 && !matches!(node.kind, COLUMN | ROW | SCROLL | CHILD))
        {
            return Err("invalid renderer focus boundary".into());
        }
        if (node.scroll_action != 0 || node.viewport_action != 0) && node.kind != SCROLL {
            return Err("scroll observer requires scroll container".into());
        }
        if (node.navigate_action != 0 || node.grid_rows != 0 || node.grid_columns != 0)
            && (node.kind != SCROLL
                || node.navigate_action == 0
                || node.grid_rows == 0
                || node.grid_columns == 0
                || u64::from(node.grid_rows) * u64::from(node.grid_columns)
                    > blockui_abi::MAX_VIEW_NODES as u64)
        {
            return Err("invalid grid navigation dimensions".into());
        }
        if (node.submit_action != 0 || node.cancel_action != 0) && node.kind != ENTRY {
            return Err("editor command requires single-line input".into());
        }
        if node.secondary_action != 0 && node.kind != CANVAS {
            return Err("secondary action requires canvas".into());
        }
        if node.kind == CANVAS && node.slot != 0 {
            return Err("canvas does not own a state slot".into());
        }
        if node.kind == PROGRESS
            && (node.action != 0
                || (node.slot != 0
                    && !matches!(
                        turn.storage.slots.get(&node.slot).map(Box::as_ref),
                        Some(Value::Float(value)) if value.is_finite() && (0.0..=1.0).contains(value)
                    )))
        {
            return Err("progress needs a finite fraction state and no action".into());
        }
        if node.kind == SLIDER {
            if node.action == 0
                || !node.range.is_valid()
                || (node.slot != 0
                    && !matches!(
                        turn.storage.slots.get(&node.slot).map(Box::as_ref),
                        Some(Value::Integer(value)) if node.range.contains(*value)
                    ))
            {
                return Err("slider needs a valid range, aligned integer state and action".into());
            }
        } else if node.range != IntegerRange::default() {
            return Err("range requires a slider".into());
        }
        if node.kind == SELECT
            && (node.action == 0
                || (node.slot != 0
                    && !matches!(
                        turn.storage.slots.get(&node.slot).map(Box::as_ref),
                        Some(Value::Integer(_))
                    )))
        {
            return Err("select needs an integer state slot and action".into());
        }
        if node.kind == CHECKBOX
            && (node.action == 0
                || (node.slot != 0
                    && !matches!(
                        turn.storage.slots.get(&node.slot).map(Box::as_ref),
                        Some(Value::Boolean(_))
                    )))
        {
            return Err("checkbox needs a boolean state slot and action".into());
        }
        if node.kind == CHILD && node.action == 0 {
            return Err("child needs an implementation identity".into());
        }
        if matches!(node.kind, ENTRY | TEXTAREA)
            && node.slot != 0
            && !matches!(
                turn.storage.slots.get(&node.slot).map(Box::as_ref),
                Some(Value::Text(_))
            )
        {
            return Err("entry needs a text state slot".into());
        }
    }
    let roots: Vec<_> = nodes.iter().filter(|n| n.parent == 0).collect();
    if roots.len() != 1 || !matches!(roots[0].kind, COLUMN | ROW | SCROLL) {
        return Err("view needs one layout root".into());
    }
    // Every node must reach the one root; reject cycles and disconnected subtrees.
    // Share reachability across walks: each node is entered once, even for a
    // deep tree. Indices refer to the sorted lookup, not authored sibling order.
    let mut colors = vec![0_u8; node_index.len()];
    let mut path = Vec::new();
    for start in 0..node_index.len() {
        let mut index = start;
        loop {
            match colors[index] {
                2 => break,
                1 => return Err("cyclic view tree".into()),
                _ => {}
            }
            colors[index] = 1;
            path.push(index);
            let parent = node_index[index].parent;
            if parent == 0 {
                break;
            }
            index = node_index
                .binary_search_by_key(&parent, |node| node.key)
                .map_err(|_| "missing parent")?;
        }
        for index in path.drain(..) {
            colors[index] = 2;
        }
    }
    let recipes = bindings::registry(collection.recipes, &nodes)?;
    let old_recipes = loaded.bindings.borrow();
    let bindings = nodes
        .into_iter()
        .map(|mut node| {
            let cached = previous
                .as_ref()
                .and_then(|cache| cache.bindings.get(&node.key))
                .filter(|old| {
                    (node.binding != 0
                        || previous.as_ref().is_some_and(|cache| {
                            cache.retained.is_some_and(|keys| keys.contains(&node.key))
                        }))
                        && node.binding == old.node.binding
                        && node.kind == old.node.kind
                        && old_recipes.get(&node.key) == recipes.get(&node.key)
                });
            if let Some(old) = cached
                && graph::refreshed(loaded, turn, &old.dependencies)? == old.versions
            {
                // A cached style is resolved output, not the authored baseline.
                if matches!(recipes.get(&node.key), Some(bindings::Recipe::Styled(..))) {
                    node.presentation = old.node.presentation;
                }
                return Ok(ProjectedBinding { node, value: None });
            }
            #[cfg(feature = "native-window")]
            if let Some(icon) = recipes.get(&node.key).and_then(bindings::Recipe::icon)
                && previous
                    .as_ref()
                    .and_then(|cache| cache.bindings.get(&node.key))
                    .and_then(|old| old.icon.as_ref())
                    .is_none_or(|old| **old != *icon)
            {
                native::validate_icon(&icon)?;
            }
            // A retained node can have dirty style dependencies while a sibling
            // scope changes structure. Reuse its independent text/style observer
            // results just as in the nonstructural patch path.
            let rendered = render(loaded, turn, node.key, &recipes, cached)?;
            if node.kind == BUTTON
                && recipes
                    .get(&node.key)
                    .and_then(bindings::Recipe::icon)
                    .is_some_and(|icon| icon.only)
                && rendered.text.trim().is_empty()
            {
                return Err("icon button requires an accessible label".into());
            }
            if let Some(style) = rendered.presentation {
                node.presentation = style;
            }
            validate_presentation(&node)?;
            Ok(ProjectedBinding {
                node,
                value: Some(Binding {
                    node,
                    text: rendered.text,
                    icon: recipes.get(&node.key).and_then(bindings::Recipe::icon),
                    semantic_label: match recipes.get(&node.key).map(bindings::Recipe::text) {
                        Some(
                            bindings::Recipe::Select(_, wire)
                            | bindings::Recipe::RecordSelect(_, _, wire),
                        ) => Some(
                            blockui_abi::select::decode_control(wire)
                                .map_err(str::to_owned)?
                                .0
                                .into(),
                        ),
                        Some(
                            bindings::Recipe::Text(_, label)
                            | bindings::Recipe::ComputedEditor(_, label)
                            | bindings::Recipe::RecordText(_, _, label)
                            | bindings::Recipe::RecordMapText { label, .. },
                        ) => label.clone(),
                        Some(bindings::Recipe::Canvas(scene)) => {
                            (!scene.label.is_empty()).then(|| scene.label.clone().into())
                        }
                        Some(
                            bindings::Recipe::Progress(_, label)
                            | bindings::Recipe::RecordProgress(_, _, label),
                        ) => (!label.is_empty()).then(|| label.clone()),
                        _ => None,
                    },
                    versions: graph::versions(turn, &rendered.dependencies),
                    dependencies: rendered.dependencies,
                    text_dependencies: rendered.text_dependencies,
                    style_dependencies: rendered.style_dependencies,
                    evaluations: u64::from(rendered.text_evaluated),
                    style_evaluations: u64::from(rendered.style_evaluated),
                }),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let published = dependencies
        .iter()
        .copied()
        .chain(bindings.iter().flat_map(|binding| {
            binding
                .value
                .as_ref()
                .or_else(|| {
                    previous
                        .as_ref()
                        .and_then(|cache| cache.bindings.get(&binding.node.key))
                })
                .into_iter()
                .flat_map(|binding| binding.dependencies.iter().copied())
        }));
    turn.graph.check_map_reads(published)?;
    let actions: Vec<_> = bindings.iter().map(|binding| binding.node.action).collect();
    assert_eq!(
        unsafe { (loaded.table().events)(loaded.event_owner, 6, actions.as_ptr(), actions.len()) },
        OK
    );
    let mut captured = Vec::new();
    for recipe in recipes.values() {
        recipe.captured_tokens(&mut captured);
    }
    assert_eq!(
        unsafe {
            (loaded.table().events)(loaded.event_owner, 7, captured.as_ptr(), captured.len())
        },
        OK
    );
    Ok(Projection {
        root_evaluated: false,
        scope_dependencies,
        root_dependencies,
        recipes,
        bindings,
        dependencies,
        versions: structure_versions,
    })
}
impl View {
    /// Number of committed root projections, including mount and accepted reload.
    /// Rejected candidates/turns and independent text/style/scope updates do not count.
    pub fn root_evaluations(&self) -> u64 {
        self.root_evaluations
    }
    fn index_binding(&mut self, key: u64, dependencies: &BTreeSet<Dependency>) {
        for dependency in dependencies {
            self.dependents.entry(*dependency).or_default().insert(key);
        }
    }
    fn unindex_binding(&mut self, key: u64, dependencies: &BTreeSet<Dependency>) {
        for dependency in dependencies {
            if let Some(keys) = self.dependents.get_mut(dependency) {
                keys.remove(&key);
                if keys.is_empty() {
                    self.dependents.remove(dependency);
                }
            }
        }
    }
    fn affected_bindings(&self, dependencies: &BTreeSet<Dependency>) -> BTreeSet<u64> {
        dependencies
            .iter()
            .filter_map(|dependency| self.dependents.get(dependency))
            .flat_map(|keys| keys.iter().copied())
            .collect()
    }
    /// Opt in to bounded renderer state retention for removed keyed widgets.
    /// Native widgets are still disposed. Zero disables/clears retention.
    /// This is a host policy, independent of actor code and state migrations.
    pub fn set_renderer_retention(&mut self, capacity: usize) -> Result<(), String> {
        if capacity > blockui_abi::MAX_VIEW_NODES {
            return Err("renderer retention exceeds node budget".into());
        }
        self.renderer_retention = capacity;
        self.trim_renderer_cache();
        Ok(())
    }
    pub fn retained_renderer_count(&self) -> usize {
        self.retained_renderer.len()
    }
    /// Occupied records and string capacities, excluding allocator overhead and spare container capacity.
    pub fn retained_renderer_bytes(&self) -> usize {
        self.retained_renderer
            .iter()
            .fold(0usize, |bytes, (_, _, _, state)| {
                bytes
                    .saturating_add(std::mem::size_of::<(u64, u32, u64, RendererState)>())
                    .saturating_add(state.text.capacity())
            })
    }
    fn trim_renderer_cache(&mut self) {
        self.trim_renderer_cache_with_bytes(self.retained_renderer_bytes());
    }
    fn trim_renderer_cache_with_bytes(&mut self, mut bytes: usize) -> usize {
        while self.retained_renderer.len() > self.renderer_retention
            || bytes > MAX_RENDERER_CACHE_BYTES
        {
            let Some((_, _, _, state)) = self.retained_renderer.pop_front() else {
                break;
            };
            bytes = bytes.saturating_sub(
                std::mem::size_of::<(u64, u32, u64, RendererState)>()
                    .saturating_add(state.text.capacity()),
            );
        }
        if self.renderer_retention == 0 {
            self.retained_renderer.shrink_to_fit();
        }
        bytes
    }
    /// Declare stable actor-local page/container keys. The nearest enclosing
    /// boundary owns focus history. Changing policy clears cached records.
    pub fn set_renderer_boundaries(&mut self, keys: &[u64]) -> Result<(), String> {
        let boundaries: BTreeSet<_> = keys.iter().copied().collect();
        if keys.len() > blockui_abi::MAX_VIEW_NODES
            || boundaries.contains(&0)
            || boundaries.len() != keys.len()
        {
            return Err("invalid renderer boundary identities".into());
        }
        if self.renderer_boundaries != boundaries {
            self.renderer_boundaries = boundaries;
            self.retained_renderer.clear();
            self.retained_renderer.shrink_to_fit();
        }
        Ok(())
    }
    pub(crate) fn renderer_branch(&self, mut key: u64) -> Option<u64> {
        let mut fallback = None;
        loop {
            let node = self.bindings.get(&key)?.node;
            if self.renderer_boundaries.contains(&key) || node.focus_boundary == 1 {
                return Some(key);
            }
            let parent = node.parent;
            if parent == 0 {
                return Some(fallback.unwrap_or(key));
            }
            if self.bindings.get(&parent)?.node.parent == 0 {
                fallback = Some(key);
            }
            key = parent;
        }
    }
    #[cfg(feature = "native-window")]
    pub(crate) fn invalidate_cached_focus(&mut self) {
        if self.retained_renderer.is_empty() {
            return;
        }
        let branches: BTreeSet<_> = self
            .renderer
            .iter()
            .filter_map(|(key, state)| state.focus.then(|| self.renderer_branch(*key)).flatten())
            .collect();
        for (_, _, branch, state) in &mut self.retained_renderer {
            if branches.contains(branch) {
                state.focus = false;
            }
        }
    }
    fn install(&mut self, projection: Projection) -> bindings::Registry {
        if projection.root_evaluated {
            self.root_evaluations = self.root_evaluations.saturating_add(1);
        }
        if projection.bindings.iter().any(|staged| {
            self.bindings
                .get(&staged.node.key)
                .is_some_and(|previous| previous.node.focus_boundary != staged.node.focus_boundary)
        }) {
            self.retained_renderer.clear();
        }
        let old_branches: BTreeMap<_, _> = if self.renderer_retention == 0 {
            BTreeMap::new()
        } else {
            self.renderer
                .keys()
                .map(|key| (*key, self.renderer_branch(*key).unwrap_or(*key)))
                .collect()
        };
        self.scope_dependencies = projection.scope_dependencies;
        self.root_dependencies = projection.root_dependencies;
        self.dependencies = projection.dependencies;
        self.versions = projection.versions;
        let bindings = projection.bindings;
        let mut next = BTreeMap::new();
        let mut renderer = BTreeMap::new();
        self.order.clear();
        self.dependents.clear();
        for staged in bindings {
            let key = staged.node.key;
            self.order.push(key);
            let previous = self.bindings.remove(&key);
            let compatible = previous
                .as_ref()
                .is_some_and(|old| old.node.kind == staged.node.kind);
            let state = if compatible {
                self.renderer.remove(&key).unwrap_or_default()
            } else if let Some(index) = self
                .retained_renderer
                .iter()
                .position(|(old_key, _, _, _)| *old_key == key)
            {
                let (_, kind, _, state) = self.retained_renderer.remove(index).unwrap();
                if kind == staged.node.kind {
                    state
                } else {
                    RendererState::default()
                }
            } else {
                RendererState::default()
            };
            let binding = match staged.value {
                Some(mut binding) => {
                    if let Some(old) = previous {
                        binding.evaluations += old.evaluations;
                        binding.style_evaluations += old.style_evaluations;
                    }
                    binding
                }
                None => {
                    let mut binding = previous.expect("validated retained binding");
                    binding.node = staged.node;
                    binding
                }
            };
            renderer.insert(key, state);
            self.index_binding(key, &binding.dependencies);
            next.insert(key, binding);
        }
        if self.renderer_retention != 0 {
            let removed = std::mem::take(&mut self.renderer);
            // Deduplicate the batch once, and carry its byte total through
            // eviction. Repeated full-cache scans make large route removals
            // quadratic in the number of retained widgets.
            self.retained_renderer
                .retain(|(old_key, _, _, _)| !removed.contains_key(old_key));
            let mut bytes = self.retained_renderer_bytes();
            for (key, state) in removed {
                if let Some(binding) = self.bindings.get(&key) {
                    let record_bytes = std::mem::size_of::<(u64, u32, u64, RendererState)>()
                        .saturating_add(state.text.capacity());
                    if record_bytes > MAX_RENDERER_CACHE_BYTES {
                        continue;
                    }
                    self.retained_renderer.push_back((
                        key,
                        binding.node.kind,
                        old_branches[&key],
                        state,
                    ));
                    bytes = self.trim_renderer_cache_with_bytes(bytes.saturating_add(record_bytes));
                }
            }
        }
        self.bindings = next;
        self.renderer = renderer;
        projection.recipes
    }
}

/// Reload behavior for stable-host futures. Restart requires a fresh future
/// factory and is intentionally not representable by this policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AsyncReloadPolicy {
    #[default]
    Preserve,
    Cancel,
}

struct PreparedTimer {
    task: Task,
    started: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl PreparedTimer {
    fn activate(self) -> Task {
        self.task
            .timer
            .as_ref()
            .unwrap()
            .set_deadline(self.task.duration);
        self.started
            .store(true, std::sync::atomic::Ordering::Release);
        self.task.timer.as_ref().unwrap().unpark();
        self.task
    }
}

pub struct Task {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
    future: Option<async_task::Handle>,
    timer: Option<timer::Handle>,
    policy: u32,
    duration: Duration,
    repeating: bool,
    message: Message,
    io: Option<io::Recipe>,
    network: bool,
    generation: u64,
}
impl Task {
    /// Whether this timer/future has finished; active periodic timers stay unfinished.
    /// I/O jobs use a pool.
    pub fn is_finished(&self) -> bool {
        self.timer.as_ref().is_some_and(|timer| timer.is_finished())
            || self
                .future
                .as_ref()
                .is_some_and(|future| future.is_finished())
    }
    /// Completion identity changes on restart even when the scheduler is shared.
    pub fn generation(&self) -> u64 {
        self.generation
    }
    #[cfg(target_arch = "wasm32")]
    pub fn worker_id(&self) -> Option<thread::ThreadId> {
        None
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn worker_id(&self) -> Option<thread::ThreadId> {
        self.timer
            .as_ref()
            .map(|timer| timer.worker_id())
            .or_else(|| self.future.as_ref().map(|future| future.worker_id()))
    }
}
impl Drop for Task {
    fn drop(&mut self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Message {
    pub action: u64,
    pub value: i64,
}
/// An owned result of a future compiled into the stable executable.
/// Text is bounded at construction and routed to the current implementation.
#[derive(Clone, Debug)]
pub struct AsyncMessage {
    message: Message,
    text: Option<std::sync::Arc<str>>,
    network: Option<std::sync::Arc<network_task::Output>>,
}
impl From<Message> for AsyncMessage {
    fn from(message: Message) -> Self {
        Self {
            message,
            text: None,
            network: None,
        }
    }
}
impl AsyncMessage {
    pub fn text(action: u64, text: impl Into<String>) -> Result<Self, String> {
        let text = text.into();
        if action == 0 || text.len() > MAX_MESSAGE_TEXT_BYTES {
            return Err("invalid async text action or payload limit exceeded".into());
        }
        Ok(Self {
            message: Message { action, value: 0 },
            text: Some(text.into()),
            network: None,
        })
    }
}
#[derive(Clone)]
struct TimerCompletion {
    id: u64,
    generation: u64,
    message: Message,
    text: Option<std::sync::Arc<str>>,
    network: Option<std::sync::Arc<network_task::Output>>,
}
/// Host mailbox envelope. Request identity is separate from application payload.
pub struct Envelope {
    message: Message,
    request_error: Option<String>,
    message_text: Option<String>,
    text_request: bool,
    sdk_request: Option<(ActorId, u64, u64)>,
    timer_ack: Option<(u64, u64)>,
    io_result: Option<(u64, std::sync::Weak<io::ReadResult>)>,
    request: Option<(
        request::RequestId,
        std::rc::Rc<std::cell::RefCell<request::RequestTracker>>,
    )>,
}
impl From<Message> for Envelope {
    fn from(message: Message) -> Self {
        Self {
            message,
            request_error: None,
            message_text: None,
            text_request: false,
            request: None,
            sdk_request: None,
            timer_ack: None,
            io_result: None,
        }
    }
}
/// One stable actor, driven exclusively by the host event loop.
pub struct Actor {
    pub id: u64,
    lifecycle: u64,
    validator: Option<admission::Validator>,
    turn: Box<Turn>,
    implementation: Option<Loaded>,
    fields: BTreeMap<u64, Field>,
    pub schema_version: u64,
    pub view: View,
    pub mailbox: VecDeque<Envelope>,
    pub tasks: BTreeMap<u64, Task>,
    wake: Option<std::sync::Arc<dyn Fn() + Send + Sync>>,
    sender: mpsc::SyncSender<TimerCompletion>,
    receiver: mpsc::Receiver<TimerCompletion>,
    io_executor: io::Executor,
    io_receiver: mpsc::Receiver<io::Completion>,
    task_generation: u64,
    future_executor: async_task::Executor,
    network_pool: network_task::Pool,
    timer_executor: timer::Executor,
    task_wake: std::sync::Arc<wake::Slot>,
    directory: PathBuf,
    generation: u64,
}
impl Actor {
    /// Current owned projection, shared by platform renderers.
    pub fn projected_view(&self) -> &View {
        &self.view
    }

    /// Attach stable host services before mounting/replacing the UI implementation.
    /// Service handlers and their captured business objects belong to the executable.
    pub fn set_services(&mut self, services: application::Services) {
        self.turn.services = services;
    }

    pub fn new(id: u64, root: &Path) -> Result<Self, String> {
        Self::new_inner(id, Some(root))
    }
    /// Mount an application linked into the executable, without filesystem snapshots.
    #[cfg(feature = "static-app")]
    pub fn mount<A: blockui::Actor>(id: u64) -> Result<Self, String> {
        let mut actor = Self::new_inner(id, None)?;
        let prepared = actor.prepare_for_test::<A>()?;
        actor.commit_reload(prepared);
        Ok(actor)
    }
    /// Data-only state transfer for replacing a statically linked Web instance.
    /// Running resources are deliberately rejected rather than silently lost.
    #[cfg(feature = "static-app")]
    pub fn checkpoint(&self) -> Result<Storage, String> {
        if !self.tasks.is_empty()
            || !self.mailbox.is_empty()
            || !self.turn.files.is_empty()
            || self.turn.sockets.len() != 0
            || !self.turn.outbox.is_empty()
            || self.turn.io_results.values().any(Option::is_some)
            || !self.turn.socket_pending.is_empty()
            || !self.turn.network_permits.is_empty()
        {
            return Err(
                "Web hot reload waits for active tasks, resources and messages to finish".into(),
            );
        }
        Ok(self.turn.storage.clone())
    }

    /// Construct and validate a candidate independently; failure cannot mutate
    /// the exporting actor. Reuses native replacement's schema/migration path.
    #[cfg(feature = "static-app")]
    pub fn mount_checkpoint<A: blockui::Actor>(id: u64, storage: Storage) -> Result<Self, String> {
        if storage.slots.len() > 256 || storage.slots.contains_key(&0) {
            return Err("invalid checkpoint state keys".into());
        }
        let mut actor = Self::new_inner(id, None)?;
        for (key, value) in &storage.slots {
            let kind = match &**value {
                Value::Float(_) => F64,
                Value::Integer(_) => I64,
                Value::Boolean(_) => BOOLEAN,
                Value::Text(value) if value.len() <= MAX_TEXT_LIST_BYTES => TEXT,
                Value::IntegerList(value) if value.len() <= MAX_LIST_ITEMS => I64_LIST,
                Value::TextList(value)
                    if value.len() <= MAX_LIST_ITEMS
                        && value.iter().map(String::len).sum::<usize>() <= MAX_TEXT_LIST_BYTES =>
                {
                    TEXT_LIST
                }
                _ => return Err("checkpoint value exceeds state limits".into()),
            };
            actor.fields.insert(
                *key,
                Field {
                    kind,
                    default: (**value).clone(),
                },
            );
        }
        actor.schema_version = schema_version(&actor.fields);
        actor.turn.storage = storage;
        let prepared = actor.prepare_for_test::<A>()?;
        actor.commit_reload(prepared);
        Ok(actor)
    }

    fn new_inner(id: u64, root: Option<&Path>) -> Result<Self, String> {
        static NEXT_LIFECYCLE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let lifecycle = NEXT_LIFECYCLE
            .try_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |value| value.checked_add(1),
            )
            .map_err(|_| "actor lifecycle identity exhausted")?;
        let directory = if let Some(root) = root {
            let directory = root.join(format!("actor-{id}-{}", std::process::id()));
            if directory.exists() {
                return Err("actor artifact directory already exists".into());
            }
            fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
            directory
        } else {
            PathBuf::new()
        };
        let (sender, receiver) = mpsc::sync_channel(256);
        let (io_sender, io_receiver) = mpsc::sync_channel(64);
        let task_wake = std::sync::Arc::new(wake::Slot::default());
        Ok(Self {
            id,
            lifecycle,
            validator: None,
            turn: Box::new(Turn {
                owner: Some((ActorId(id), lifecycle)),
                ..Turn::default()
            }),
            implementation: None,
            fields: BTreeMap::new(),
            schema_version: 0,
            view: View::default(),
            mailbox: VecDeque::new(),
            tasks: BTreeMap::new(),
            wake: None,
            sender,
            receiver,
            io_executor: io::Executor::new(io_sender),
            io_receiver,
            task_generation: 0,
            future_executor: async_task::Executor::with_wake(task_wake.clone()),
            network_pool: network_task::Pool::default(),
            timer_executor: timer::Executor::default(),
            task_wake,
            directory,
            generation: 0,
        })
    }
    /// Construct a test actor from an SDK definition compiled into this executable.
    /// Uses the production state/transaction/view path; creates no DLL snapshots.
    #[cfg(feature = "testing")]
    pub fn for_test<A: blockui::Actor>() -> Result<Self, String> {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);
        let id = NEXT_ID
            .try_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |value| value.checked_sub(1),
            )
            .map_err(|_| "test actor identity exhausted")?;
        let mut actor = Self::new(id, &std::env::temp_dir())?;
        actor.replace_for_test::<A>()?;
        Ok(actor)
    }
    /// Disable automatic timer deadlines for this test actor. Configure before
    /// any timer admission; completions require `complete_timer_for_test`.
    /// Shared runtime schedulers reject this actor-local configuration.
    /// Does not change application time, IO or network deadlines.
    #[cfg(feature = "testing")]
    pub fn use_manual_timers_for_test(&self) -> Result<(), String> {
        self.timer_executor.use_manual_timers_for_test()
    }
    /// Advance the isolated manual timer clock and admit due completions in task
    /// identity order. Call `tick` separately. Application time stays separate.
    /// A full completion queue returns an error; already admitted messages remain
    /// queued and retrying with zero duration admits remaining due timers.
    #[cfg(feature = "testing")]
    pub fn advance_timer_time_for_test(&self, elapsed: Duration) -> Result<usize, String> {
        let now = self.timer_executor.advance_for_test(elapsed)?;
        let mut admitted = 0;
        for task in self.tasks.values() {
            if let Some(timer) = &task.timer {
                admitted += usize::from(timer.complete_if_due_for_test(now)?);
            }
        }
        Ok(admitted)
    }
    /// Set the time observed by `Cx::monotonic_ms` in application tests.
    /// Configure before invoking actions; subsequent values cannot go backwards.
    /// This changes application time only, not timer/IO/network deadlines.
    #[cfg(feature = "testing")]
    pub fn set_monotonic_ms_for_test(&mut self, millis: u64) -> Result<(), String> {
        if self
            .turn
            .test_monotonic_ms
            .is_some_and(|previous| millis < previous)
        {
            return Err("test monotonic time cannot go backwards".into());
        }
        self.turn.test_monotonic_ms = Some(millis);
        Ok(())
    }
    /// Admit a committed timer's next completion without waiting for its deadline.
    /// Call `tick` explicitly to run the current implementation's action. This
    /// does not virtualize time: real timers remain active, so tests should use
    /// long delays. Pending/full-channel admission leaves the timer unchanged.
    #[cfg(feature = "testing")]
    pub fn complete_timer_for_test(&self, id: u64) -> Result<(), String> {
        self.tasks
            .get(&id)
            .and_then(|task| task.timer.as_ref())
            .ok_or("no timer with this identity")?
            .complete_for_test()
    }
    /// Queue a typed SDK action for explicit processing by `tick`.
    ///
    /// ```compile_fail
    /// use blockui::BoolAction;
    /// fn wrong(actor: &mut blockui_runtime::Actor, action: BoolAction) {
    ///     actor.tell(action, 1_i64).unwrap();
    /// }
    /// ```
    #[cfg(feature = "testing")]
    pub fn tell<T: blockui::TellToken>(
        &mut self,
        action: T,
        value: T::Payload,
    ) -> Result<(), String> {
        let (action, payload) = action
            .encode_message(value)
            .map_err(|error| error.to_string())?;
        let mut envelope: Envelope = Message { action, value: 0 }.into();
        match payload {
            blockui::MessagePayload::Scalar(value) => envelope.message.value = value,
            blockui::MessagePayload::Text(value) => envelope.message_text = Some(value),
        }
        if self.enqueue_envelope(envelope).is_some() {
            Err("mailbox full".into())
        } else {
            Ok(())
        }
    }

    /// Replace a compiled-in test definition through normal migration and rollback.
    /// This tests runtime behavior, not dynamic loader or unload safety.
    #[cfg(feature = "testing")]
    pub fn replace_for_test<A: blockui::Actor>(&mut self) -> Result<(), String> {
        let prepared = self.prepare_for_test::<A>()?;
        self.commit_reload(prepared);
        Ok(())
    }
    #[cfg(any(feature = "testing", feature = "static-app"))]
    pub(crate) fn prepare_for_test<A: blockui::Actor>(&mut self) -> Result<PreparedReload, String> {
        if A::STATE.len() > 256 {
            return Err("invalid schema".into());
        }
        self.prepare_implementation(|_, generation| {
            Ok(Loaded {
                event_owner: next_event_owner(),
                scopes: Default::default(),
                bindings: Default::default(),
                #[cfg(not(target_arch = "wasm32"))]
                library: None,
                table: &blockui::Export::<A>::TABLE,
                path: PathBuf::new(),
                version: generation,
            })
        })
    }

    /// Process-local instance identity; independent of implementation replacement.
    pub fn lifecycle(&self) -> u64 {
        self.lifecycle
    }
    pub fn implementation_version(&self) -> u64 {
        self.implementation.as_ref().map_or(0, |v| v.version)
    }
    pub fn state_address(&self) -> usize {
        (&self.turn.storage as *const Storage) as usize
    }
    pub fn slot_address(&self, name: &str) -> usize {
        self.slot_address_by_key(key(name))
    }
    pub fn slot_address_by_key(&self, identity: u64) -> usize {
        self.turn
            .storage
            .slots
            .get(&identity)
            .map_or(0, |v| (&**v as *const Value) as usize)
    }
    pub fn float(&self, name: &str) -> f64 {
        self.float_by_key(key(name))
    }
    pub fn float_by_key(&self, identity: u64) -> f64 {
        match self.turn.storage.slots.get(&identity).map(Box::as_ref) {
            Some(Value::Float(value)) => *value,
            _ => panic!("expected float state"),
        }
    }
    pub fn integer(&self, name: &str) -> i64 {
        self.integer_by_key(key(name))
    }
    pub fn integer_by_key(&self, identity: u64) -> i64 {
        match self.turn.storage.slots[&identity].as_ref() {
            Value::Integer(v) => *v,
            _ => panic!("wrong slot type"),
        }
    }
    pub fn text_list(&self, name: &str) -> &[String] {
        self.text_list_by_key(key(name))
    }
    pub fn text_list_by_key(&self, identity: u64) -> &[String] {
        match self.turn.storage.slots[&identity].as_ref() {
            Value::TextList(values) => values,
            _ => panic!("wrong slot type"),
        }
    }
    pub fn list(&self, name: &str) -> &[i64] {
        self.list_by_key(key(name))
    }
    pub fn list_by_key(&self, identity: u64) -> &[i64] {
        match self.turn.storage.slots[&identity].as_ref() {
            Value::IntegerList(values) => values,
            _ => panic!("wrong slot type"),
        }
    }
    pub fn text(&self, name: &str) -> &str {
        self.text_by_key(key(name))
    }
    pub fn text_by_key(&self, identity: u64) -> &str {
        match self.turn.storage.slots[&identity].as_ref() {
            Value::Text(v) => v,
            _ => panic!("wrong slot type"),
        }
    }
    pub fn set_text(&mut self, slot: u64, value: &str) -> Result<(), String> {
        self.write_editor_text(slot, 0, value)
    }
    /// Apply a user edit through the current editor projection in one actor transaction.
    /// Programmatic set_text does not invoke this callback or create feedback loops.
    pub fn edit_text(&mut self, widget: u64, value: &str) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed widget")?
            .node;
        if !matches!(node.kind, ENTRY | TEXTAREA) {
            return Err("widget is not a text input".into());
        }
        if node.disabled != 0 {
            return Err("text input is disabled".into());
        }
        if node.read_only != 0 {
            return Err("text input is read-only".into());
        }
        let keyed_wire = if let Some(loaded) = &self.implementation {
            let recipes = loaded.bindings.borrow();
            match recipes.get(&widget) {
                Some(bindings::Recipe::RecordMapText {
                    keyed: true, key, ..
                }) => Some(blockui_abi::keyed_text::encode(key, value).map_err(str::to_owned)?),
                _ => None,
            }
        } else {
            None
        };
        self.write_editor_text(
            node.slot,
            node.action,
            keyed_wire.as_deref().unwrap_or(value),
        )
    }
    fn write_editor_text(&mut self, slot: u64, action: u64, value: &str) -> Result<(), String> {
        if action != 0 && value.len() > MAX_MESSAGE_TEXT_BYTES {
            return Err("text edit callback payload exceeds limit".into());
        }
        if self.turn.message_text.is_some() {
            return Err("nested text edit".into());
        }
        let result = self.transaction(|actor| {
            let table = actor
                .implementation
                .as_ref()
                .ok_or("actor has no implementation")?
                .table;
            if slot != 0 {
                let code = with_api(&mut actor.turn, table, |api| unsafe {
                    (api.write_text)(api.context, slot, Bytes::new(value))
                });
                if code != OK {
                    return Err("invalid text slot".into());
                }
            } else if action == 0 {
                return Err("computed editor requires a text action".into());
            }
            if action != 0 {
                actor.turn.message_text = Some(value.to_owned());
                let code = with_api(&mut actor.turn, table, |api| unsafe {
                    ((*table).handle)(api, action, 0)
                });
                actor.turn.message_text = None;
                if code != OK || actor.turn.graph.fault.is_some() {
                    return Err(actor
                        .turn
                        .graph
                        .fault
                        .clone()
                        .or_else(|| actor.turn.handler_error.clone())
                        .unwrap_or_else(|| "text edit handler failed".into()));
                }
            }
            actor.flush()
        });
        self.turn.message_text = None;
        result
    }
    fn transaction(
        &mut self,
        f: impl FnOnce(&mut Self) -> Result<Option<Box<dyn admission::Admission>>, String>,
    ) -> Result<(), String> {
        let graph = self.turn.graph.clone();
        let revisions = self.turn.revisions.clone();
        let changed = self.turn.changed.clone();
        let files = self.turn.files.clone();
        let sockets = self.turn.sockets.clone();
        let socket_pending = self.turn.socket_pending.clone();
        let network_permits = self.turn.network_permits.clone();
        let io_results = self.turn.io_results.clone();
        self.turn.journal = Some(BTreeMap::new());
        self.turn.task_keys = self.tasks.keys().copied().collect();
        self.turn.task_commands.clear();
        self.turn.task_generation_budget = u64::MAX - self.task_generation;
        self.turn.outbox.clear();
        let (result, admission) = match f(self) {
            Ok(admission) => (Ok(()), admission),
            Err(error) => (Err(error), None),
        };
        if let Some(implementation) = &self.implementation {
            implementation.events(if result.is_ok() { 2 } else { 3 });
        }
        let undo = self.turn.journal.take().unwrap();
        let commands = std::mem::take(&mut self.turn.task_commands);
        let outbox = std::mem::take(&mut self.turn.outbox);
        let routed = !outbox.is_empty();
        if result.is_err() {
            for (key, value) in undo {
                **self.turn.storage.slots.get_mut(&key).unwrap() = value;
            }
            self.turn.graph = graph;
            self.turn.revisions = revisions;
            self.turn.changed = changed;
            self.turn.files = files;
            self.turn.sockets = sockets;
            self.turn.socket_pending = socket_pending;
            self.turn.network_permits = network_permits;
            self.turn.io_results = io_results;
        } else {
            if let Some(router) = &self.turn.router {
                router.borrow_mut().publish(outbox);
            }
            // Requests contain only host-owned data. Start work only after the turn commits.
            for command in commands {
                match command {
                    TaskCommand::PreparedDelay(id, prepared) => {
                        self.task_generation = prepared.task.generation;
                        self.tasks.insert(id, prepared.activate());
                    }
                    TaskCommand::Delay(..) | TaskCommand::Interval(..) => {
                        unreachable!("timer workers prepared before view publication")
                    }
                    TaskCommand::Cancel(id) => {
                        self.tasks.remove(&id);
                    }
                    TaskCommand::Read(id, recipe) => {
                        self.start_read(id, recipe);
                    }
                    TaskCommand::PreparedNetwork(id, prepared) => {
                        self.task_generation = prepared.task.generation;
                        self.tasks.insert(id, prepared.activate());
                    }
                    TaskCommand::Network(..) => {
                        unreachable!("network future prepared before publication")
                    }
                }
            }
            if let Some(admission) = admission {
                admission.commit();
            }
            if routed {
                wake::notify(&mut self.wake);
            }
            let published = self.view.dependencies.iter().copied().chain(
                self.view
                    .bindings
                    .values()
                    .flat_map(|binding| binding.dependencies.iter().copied()),
            );
            self.turn.graph.retain_map_reads(published);
        }
        result
    }
    pub fn computed(&self, name: &str) -> Option<&Value> {
        self.turn.graph.value(key(name))
    }
    /// Registered structural identities in the current implementation.
    /// Mounted scope builders invalidate independently unless their root projection changes.
    pub fn scope_ids(&self) -> Vec<u64> {
        self.implementation
            .as_ref()
            .map(|loaded| loaded.scopes.borrow().iter().copied().collect())
            .unwrap_or_default()
    }
    pub fn computed_evaluations(&self, name: &str) -> u64 {
        self.computed_evaluations_by_key(key(name))
    }
    pub fn computed_evaluations_by_key(&self, identity: u64) -> u64 {
        self.turn.graph.evaluations(identity)
    }
    pub fn computed_count(&self) -> usize {
        self.turn.graph.count()
    }
    pub fn computed_dependencies(&self, name: &str) -> Option<BTreeSet<Dependency>> {
        self.computed_dependencies_by_key(key(name))
    }
    pub fn computed_dependencies_by_key(&self, identity: u64) -> Option<BTreeSet<Dependency>> {
        self.turn.graph.dependencies(identity)
    }
    /// Inspect validated candidate metadata without running migration, view, actions,
    /// or admitting resources/children. Loading trusted code executes its entry point
    /// (and loader constructors); this is not a sandbox or a successful-reload promise.
    /// A fresh snapshot generation is consumed and released before returning.
    pub fn preview_reload(&mut self, path: &Path) -> Result<ReloadPreview, String> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or("snapshot generation exhausted")?;
        let candidate = Loaded::open(path, &self.directory, self.generation)?;
        *candidate.scopes.borrow_mut() = scopes::registry(&candidate)?;
        let fields = schema(&candidate)?;
        let keys: BTreeSet<_> = self.fields.keys().chain(fields.keys()).copied().collect();
        let changes: Vec<_> = keys
            .into_iter()
            .map(|key| match (self.fields.get(&key), fields.get(&key)) {
                (Some(old), Some(new)) if old.kind == new.kind => StateChange::Retained {
                    key,
                    kind: old.kind,
                },
                (Some(old), Some(new)) => StateChange::TypeChanged {
                    key,
                    from: old.kind,
                    to: new.kind,
                    default: new.default.clone(),
                },
                (Some(old), None) => StateChange::Removed {
                    key,
                    kind: old.kind,
                },
                (None, Some(new)) => StateChange::Added {
                    key,
                    default: new.default.clone(),
                },
                (None, None) => unreachable!(),
            })
            .collect();
        Ok(ReloadPreview {
            implementation_version: candidate.version,
            schema_version: schema_version(&fields),
            compatible: changes
                .iter()
                .all(|change| matches!(change, StateChange::Retained { .. })),
            has_migration: candidate.table().migrate.is_some(),
            changes,
        })
    }
    /// Execute candidate preparation and discard it without replacing this actor.
    /// Unlike preview_reload, this executes migration/computations/view and admission,
    /// may reopen resources or allocate dormant task workers, and consumes a snapshot
    /// generation. Trusted plugin code can have external side effects. Success proves
    /// preparation against current state only; it does not reserve a later reload.
    pub fn check_reload(&mut self, path: &Path) -> Result<(), String> {
        let prepared = self.prepare_reload(path)?;
        drop(prepared);
        Ok(())
    }
    pub fn reload(&mut self, path: &Path) -> Result<(), String> {
        let prepared = self.prepare_reload(path)?;
        self.commit_reload(prepared);
        Ok(())
    }
    pub(crate) fn prepare_reload(&mut self, path: &Path) -> Result<PreparedReload, String> {
        self.prepare_implementation(|directory, generation| {
            Loaded::open(path, directory, generation)
        })
    }
    fn prepare_implementation(
        &mut self,
        load: impl FnOnce(&Path, u64) -> Result<Loaded, String>,
    ) -> Result<PreparedReload, String> {
        let restarts = self.tasks.values().filter(|task| task.policy == 2).count() as u64;
        self.task_generation
            .checked_add(restarts)
            .ok_or("task generation exhausted; reload rejected")?;
        if self
            .tasks
            .values()
            .any(|task| task.policy == 2 && task.io.is_some())
        {
            self.io_executor.ensure_workers()?;
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or("snapshot generation exhausted; reload rejected")?;
        let mut restart_timers = BTreeMap::new();
        let mut task_generation = self.task_generation;
        for (id, task) in &self.tasks {
            if task.policy == 2 {
                task_generation += 1; // total restart budget validated above
                if task.io.is_none() {
                    restart_timers.insert(
                        *id,
                        self.prepare_timer_with_repeat(
                            *id,
                            task.duration,
                            task.message,
                            task.policy,
                            task_generation,
                            task.repeating,
                        )?,
                    );
                }
            }
        }
        let candidate = load(&self.directory, self.generation)?;
        *candidate.scopes.borrow_mut() = scopes::registry(&candidate)?;
        let fields = schema(&candidate)?;
        let version = schema_version(&fields);
        let compatible = self.fields.len() == fields.len()
            && self
                .fields
                .iter()
                .all(|(key, old)| fields.get(key).is_some_and(|v| v.kind == old.kind));
        let new_graph = Graph::load(&candidate)?;
        let next_files = resources::replacement(&self.turn.files)?;
        // No state clone or serialization on an implementation-only replacement.
        let (bindings, admission, graph, staged) = if compatible {
            let old_graph = std::mem::replace(&mut self.turn.graph, new_graph);
            let result = graph::eager(&candidate, &mut self.turn)
                .and_then(|_| project(&candidate, &mut self.turn, None))
                .and_then(|bindings| {
                    let admission = admission::validate(
                        &mut self.validator,
                        bindings.bindings.iter().map(|binding| binding.node),
                    )?;
                    Ok((bindings, admission))
                });
            let graph = std::mem::replace(&mut self.turn.graph, old_graph);
            let (bindings, admission) = result?;
            (bindings, admission, graph, None)
        } else {
            let mut staged = Turn {
                #[cfg(feature = "testing")]
                test_monotonic_ms: self.turn.test_monotonic_ms,
                previous: Some(self.turn.storage.clone()),
                revisions: self.turn.revisions.clone(),
                graph: new_graph,
                ..Turn::default()
            };
            for (key, field) in &fields {
                if self
                    .fields
                    .get(key)
                    .is_some_and(|old| old.kind == field.kind)
                {
                    staged
                        .storage
                        .slots
                        .insert(*key, self.turn.storage.slots[key].clone());
                } else {
                    if self.fields.contains_key(key) {
                        staged.conversions.insert(*key);
                    }
                    staged
                        .storage
                        .slots
                        .insert(*key, Box::new(field.default.clone()));
                }
            }
            if let Some(migrate) = candidate.table().migrate
                && with_api(&mut staged, candidate.table, |api| unsafe {
                    migrate(api, self.schema_version)
                }) != OK
            {
                return Err(staged
                    .handler_error
                    .clone()
                    .unwrap_or_else(|| "migration rejected; old actor preserved".into()));
            }
            if !staged.conversions.is_empty() {
                return Err(
                    "type changes require explicit migration writes to every changed slot".into(),
                );
            }
            staged.previous = None;
            staged.revisions.retain(|key, _| fields.contains_key(key));
            graph::eager(&candidate, &mut staged)?;
            let bindings = project(&candidate, &mut staged, None)?;
            let admission = admission::validate(
                &mut self.validator,
                bindings.bindings.iter().map(|binding| binding.node),
            )?;
            (bindings, admission, Graph::default(), Some(staged))
        };
        Ok(PreparedReload {
            candidate,
            fields,
            version,
            graph,
            staged,
            bindings,
            admission,
            files: next_files,
            restart_timers,
        })
    }
    pub(crate) fn commit_reload(&mut self, prepared: PreparedReload) {
        let PreparedReload {
            candidate,
            fields,
            version,
            graph,
            staged,
            bindings,
            admission,
            files: next_files,
            mut restart_timers,
        } = prepared;
        if let Some(mut staged) = staged {
            let mut old_slots = std::mem::take(&mut self.turn.storage.slots);
            for (key, value) in &mut staged.storage.slots {
                if self
                    .fields
                    .get(key)
                    .is_some_and(|old| fields[key].kind == old.kind)
                {
                    let original = old_slots.remove(key).unwrap();
                    let migrated = std::mem::replace(value, original);
                    // A schema change may clone staged values without changing them.
                    // Keep live string/list buffers when their contents are unchanged.
                    if **value != *migrated {
                        **value = *migrated;
                    }
                }
            }
            self.turn.storage = staged.storage;
            self.turn.graph = staged.graph;
            self.turn.revisions = staged.revisions;
        } else {
            self.turn.graph = graph;
        }
        self.fields = fields;
        self.schema_version = version;
        *candidate.bindings.borrow_mut() = self.view.install(bindings);
        // Binding code tokens live only in this implementation owner; old code can close.
        candidate.events(2);
        self.turn.event_owner = candidate.event_owner;
        self.implementation = Some(candidate);
        self.turn.files = next_files;
        self.turn.sockets.commit_reload();
        // Apply policies only after validation; failed reload leaves all work running.
        let replace: Vec<_> = self
            .tasks
            .iter()
            .filter(|(_, task)| task.policy != 0)
            .map(|(id, task)| (*id, task.policy, task.io.clone(), task.network))
            .collect();
        for (id, policy, recipe, network) in replace {
            self.tasks.remove(&id);
            if policy == 1 && (recipe.is_some() || network) {
                self.turn.socket_pending.remove(&id);
                self.turn.io_results.remove(&id);
                self.turn.network_permits.remove(&id);
            }
            if policy == 2 {
                if let Some(recipe) = recipe {
                    self.turn.io_results.insert(id, None);
                    self.start_read(id, recipe);
                } else {
                    let prepared = restart_timers
                        .remove(&id)
                        .expect("restart timer prepared before publication");
                    self.task_generation = prepared.task.generation;
                    self.tasks.insert(id, prepared.activate());
                }
            }
        }
        if let Some(admission) = admission {
            admission.commit();
        }
    }
    /// Host request-envelope entry. Returns a reply only after a successful turn.
    /// This does not allocate or route a request; Runtime ask wiring is separate.
    pub fn dispatch_request(&mut self, message: Message) -> Result<Option<i64>, String> {
        if self.turn.request_reply.is_some() || self.turn.request_text_reply.is_some() {
            return Err("nested request turn".into());
        }
        self.turn.request_reply = Some(None);
        let result = self.dispatch(message);
        let reply = self.turn.request_reply.take().flatten();
        result.map(|()| reply)
    }
    /// Process a host-owned text request transaction. Reply bytes publish only on success.
    /// Queueing and pending identity are managed separately by Runtime::ask_text.
    pub fn dispatch_text_request(
        &mut self,
        action: u64,
        text: String,
    ) -> Result<Option<String>, String> {
        if text.len() > MAX_MESSAGE_TEXT_BYTES || action == 0 {
            return Err("invalid text request payload or action".into());
        }
        if self.turn.request_reply.is_some()
            || self.turn.request_text_reply.is_some()
            || self.turn.message_text.is_some()
        {
            return Err("nested request turn".into());
        }
        self.turn.request_text_reply = Some(None);
        self.turn.message_text = Some(text);
        let result = self.dispatch(Message { action, value: 0 });
        self.turn.message_text = None;
        let reply = self.turn.request_text_reply.take().flatten();
        result.map(|()| reply)
    }
    /// Read a current native numeric control value from its host-owned source.
    pub fn integer_control_value(&self, widget: u64) -> Option<i64> {
        let node = self.view.bindings.get(&widget)?.node;
        if !matches!(node.kind, SLIDER | SELECT) {
            return None;
        }
        let recipes = self.implementation.as_ref()?.bindings.borrow();
        match recipes.get(&widget)? {
            bindings::Recipe::Slider(slot, _, _) | bindings::Recipe::Select(slot, _) => {
                match self.turn.storage.slots.get(slot)?.as_ref() {
                    Value::Integer(value) => Some(*value),
                    _ => None,
                }
            }
            bindings::Recipe::RecordSlider(slot, selector, _, _)
            | bindings::Recipe::RecordSelect(slot, selector, _) => {
                record_fields::integer(&self.turn, *slot, selector)
            }
            _ => None,
        }
    }
    /// Read a current checkbox value from host-owned state or a record selector.
    pub fn checkbox_value(&self, widget: u64) -> Option<bool> {
        let node = self.view.bindings.get(&widget)?.node;
        if node.kind != CHECKBOX {
            return None;
        }
        let recipes = self.implementation.as_ref()?.bindings.borrow();
        match recipes.get(&widget)? {
            bindings::Recipe::Checkbox(slot, _) => {
                match self.turn.storage.slots.get(slot)?.as_ref() {
                    Value::Boolean(value) => Some(*value),
                    _ => None,
                }
            }
            bindings::Recipe::RecordCheckbox(slot, selector, _) => {
                let Value::Text(encoded) = self.turn.storage.slots.get(slot)?.as_ref() else {
                    return None;
                };
                match record_fields::lookup(encoded, selector).ok()? {
                    "0" => Some(false),
                    "1" => Some(true),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    /// Host-owned aggregate notification activity, routed through current UI code.
    pub fn observe_toast_activity(&mut self, widget: u64, paused: bool) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed toast viewport")?
            .node;
        if node.kind != SCROLL
            || node.semantics.role != semantics::TOAST_VIEWPORT
            || node.action == 0
        {
            return Err("invalid toast activity observer".into());
        }
        if node.action_payload == i64::from(paused) {
            return Ok(());
        }
        self.dispatch(Message {
            action: node.action,
            value: i64::from(paused),
        })
    }
    /// Activate a current button/checkbox projection through its current action.
    pub fn dispatch_widget(&mut self, widget: u64) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed widget")?
            .node;
        if !matches!(node.kind, BUTTON | CHECKBOX)
            && node.semantics.role != semantics::CONTEXT_SURFACE
        {
            return Err("widget is not an action control".into());
        }
        if node.disabled != 0 {
            return Ok(());
        }
        let value = if node.kind == CHECKBOX {
            i64::from(
                !self
                    .checkbox_value(widget)
                    .ok_or("checkbox needs a boolean source")?,
            )
        } else {
            node.action_payload
        };
        self.dispatch(Message {
            action: node.action,
            value,
        })
    }
    /// Dispatch a user slider value through the current projection/action.
    /// Programmatic state writes never synthesize this event.
    pub fn edit_slider(&mut self, widget: u64, value: i64) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed slider")?
            .node;
        if node.kind != SLIDER || node.disabled != 0 || !node.range.contains(value) {
            return Err("slider input is disabled or outside its aligned range".into());
        }
        self.dispatch(Message {
            action: node.action,
            value,
        })
    }
    /// Validate against the current projection before dispatching its current action.
    pub fn edit_select(&mut self, widget: u64, value: i64) -> Result<(), String> {
        let binding = self.view.bindings.get(&widget).ok_or("removed select")?;
        if binding.node.kind != SELECT || binding.node.disabled != 0 {
            return Err("select is disabled or invalid".into());
        }
        let (_, options) =
            blockui_abi::select::decode_control(&binding.text).map_err(str::to_owned)?;
        if !options
            .iter()
            .any(|option| option.value == value && option.enabled)
        {
            return Err("select option is absent or disabled".into());
        }
        let action = binding.node.action;
        self.dispatch(Message { action, value })
    }
    /// Resolve current Canvas coordinates and action; no plugin callback is retained.
    pub fn edit_canvas(
        &mut self,
        widget: u64,
        point: LocalPoint,
        secondary: bool,
    ) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed canvas")?
            .node;
        if node.kind != CANVAS {
            return Err("widget is not a canvas".into());
        }
        let action = if secondary {
            node.secondary_action
        } else {
            node.action
        };
        if action == 0 {
            return Err("canvas has no pointer action".into());
        }
        let value = point.to_wire().ok_or("nonfinite canvas coordinates")?;
        self.dispatch(Message { action, value })
    }
    pub fn observe_scroll(
        &mut self,
        widget: u64,
        point: blockui_abi::LocalPoint,
    ) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed scroll container")?
            .node;
        if node.kind != SCROLL || node.scroll_action == 0 || point.x < 0.0 || point.y < 0.0 {
            return Err("invalid scroll observer".into());
        }
        let value = point.to_wire().ok_or("nonfinite scroll offset")?;
        self.dispatch(Message {
            action: node.scroll_action,
            value,
        })
    }
    pub fn observe_viewport(
        &mut self,
        widget: u64,
        size: blockui_abi::LocalPoint,
    ) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed scroll container")?
            .node;
        if node.kind != SCROLL || node.viewport_action == 0 || size.x < 0.0 || size.y < 0.0 {
            return Err("invalid viewport observer".into());
        }
        let value = size.to_wire().ok_or("nonfinite viewport size")?;
        self.dispatch(Message {
            action: node.viewport_action,
            value,
        })
    }
    pub fn navigate_grid(&mut self, widget: u64, index: i64) -> Result<(), String> {
        let node = self.view.bindings.get(&widget).ok_or("removed grid")?.node;
        if node.kind != SCROLL
            || node.navigate_action == 0
            || index < 0
            || index as u64 >= u64::from(node.grid_rows) * u64::from(node.grid_columns)
        {
            return Err("invalid grid navigation".into());
        }
        self.dispatch(Message {
            action: node.navigate_action,
            value: index,
        })
    }
    pub fn editor_command(&mut self, widget: u64, cancel: bool) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed editor")?
            .node;
        if node.kind != ENTRY || node.disabled != 0 || node.read_only != 0 {
            return Err("editor command requires editable input".into());
        }
        let action = if cancel {
            node.cancel_action
        } else {
            node.submit_action
        };
        if action == 0 {
            return Err("editor command is not configured".into());
        }
        self.dispatch(Message { action, value: 0 })
    }
    pub fn dismiss_dialog(&mut self, widget: u64) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed dialog")?
            .node;
        if node.kind != DIALOG {
            return Err("widget is not a dialog".into());
        }
        self.dispatch(Message {
            action: node.action,
            value: 0,
        })
    }
    /// Dismiss only a currently visible nonmodal panel, through current UI code.
    pub fn dismiss_popover(&mut self, widget: u64) -> Result<(), String> {
        let node = self
            .view
            .bindings
            .get(&widget)
            .ok_or("removed popover")?
            .node;
        if !semantics::is_anchored_panel(node.semantics.role) || node.semantics.hidden != 0 {
            return Err("widget is not an open popover".into());
        }
        self.dispatch(Message {
            action: node.action,
            value: 0,
        })
    }
    pub fn dispatch(&mut self, message: Message) -> Result<(), String> {
        self.transaction(|actor| {
            let table = actor
                .implementation
                .as_ref()
                .ok_or("actor has no implementation")?
                .table;
            let code = with_api(&mut actor.turn, table, |api| unsafe {
                ((*table).handle)(api, message.action, message.value)
            });
            if code != OK || actor.turn.graph.fault.is_some() {
                return Err(actor
                    .turn
                    .graph
                    .fault
                    .clone()
                    .or_else(|| actor.turn.handler_error.clone())
                    .unwrap_or_else(|| "message handler failed".into()));
            }
            actor.flush()
        })
    }
    fn prepare_task_commands(&mut self) -> Result<(), String> {
        let commands = std::mem::take(&mut self.turn.task_commands);
        let mut generation = self.task_generation;
        for command in commands {
            let repeating = matches!(command, TaskCommand::Interval(..));
            let prepared = match command {
                TaskCommand::Delay(id, duration, message, policy)
                | TaskCommand::Interval(id, duration, message, policy) => {
                    generation = generation
                        .checked_add(1)
                        .ok_or("task generation exhausted")?;
                    TaskCommand::PreparedDelay(
                        id,
                        self.prepare_timer_with_repeat(
                            id, duration, message, policy, generation, repeating,
                        )?,
                    )
                }
                TaskCommand::Read(id, recipe) => {
                    self.io_executor.ensure_workers()?;
                    generation = generation
                        .checked_add(1)
                        .ok_or("task generation exhausted")?;
                    TaskCommand::Read(id, recipe)
                }
                TaskCommand::Network(id, recipe) => {
                    generation = generation
                        .checked_add(1)
                        .ok_or("task generation exhausted")?;
                    TaskCommand::PreparedNetwork(
                        id,
                        network_task::prepare(self, id, generation, recipe)?,
                    )
                }
                command => command,
            };
            self.turn.task_commands.push(prepared);
        }
        Ok(())
    }
    fn flush(&mut self) -> Result<Option<Box<dyn admission::Admission>>, String> {
        self.prepare_task_commands()?;
        let dirty = std::mem::take(&mut self.turn.changed);
        if dirty.is_empty() {
            let published = self.view.dependencies.iter().copied().chain(
                self.view
                    .bindings
                    .values()
                    .flat_map(|binding| binding.dependencies.iter().copied()),
            );
            self.turn.graph.check_map_reads(published)?;
            return Ok(None);
        }
        let Some(loaded) = &self.implementation else {
            return Ok(None);
        };
        // Snapshot reachability before refreshing structure: evaluation may replace
        // dynamic edges, but existing bindings still describe the previous turn.
        let mut sources: Vec<_> = dirty.iter().copied().map(Dependency::State).collect();
        sources.extend(self.turn.graph.map_reads.sources(&dirty));
        let affected = self.turn.graph.affected(sources);
        let candidates = self.view.affected_bindings(&affected);
        let structure_versions = graph::refreshed(loaded, &mut self.turn, &self.view.dependencies)?;
        if structure_versions != self.view.versions {
            let root_changed = self.view.root_dependencies.iter().any(|dependency| {
                structure_versions.get(dependency) != self.view.versions.get(dependency)
            });
            let projection = if root_changed {
                project_with_scopes(
                    loaded,
                    &mut self.turn,
                    Some(&self.view.bindings),
                    Some(&self.view),
                )?
            } else {
                let scopes = self
                    .view
                    .scope_dependencies
                    .iter()
                    .filter_map(|(id, scope)| {
                        if graph::versions(&self.turn, &scope.dependencies) != scope.versions {
                            Some(*id)
                        } else {
                            None
                        }
                    })
                    .collect();
                scopes::refresh(loaded, &mut self.turn, &self.view, &scopes)?
            };
            let admission = admission::validate(
                &mut self.validator,
                projection.bindings.iter().map(|binding| binding.node),
            )?;
            *loaded.bindings.borrow_mut() = self.view.install(projection);
            return Ok(admission);
        }
        // Stage every binding before applying; a late error cannot partially patch UI.
        let mut patches = Vec::new();
        for key in candidates {
            let binding = &self.view.bindings[&key];
            let current = graph::refreshed(loaded, &mut self.turn, &binding.dependencies)?;
            if current == binding.versions {
                continue;
            }
            let rendered = render(
                loaded,
                &mut self.turn,
                key,
                &loaded.bindings.borrow(),
                Some(binding),
            )?;
            if binding.node.kind == BUTTON
                && binding.icon.as_ref().is_some_and(|icon| icon.only)
                && rendered.text.trim().is_empty()
            {
                return Err("icon button requires an accessible label".into());
            }
            let versions = graph::versions(&self.turn, &rendered.dependencies);
            patches.push((key, rendered, versions));
        }
        let replacements: BTreeMap<_, _> = patches
            .iter()
            .map(|(key, rendered, _)| (*key, &rendered.dependencies))
            .collect();
        let published =
            self.view
                .dependencies
                .iter()
                .copied()
                .chain(self.view.bindings.iter().flat_map(|(key, binding)| {
                    replacements
                        .get(key)
                        .copied()
                        .unwrap_or(&binding.dependencies)
                        .iter()
                        .copied()
                }));
        self.turn.graph.check_map_reads(published)?;
        let styles: BTreeMap<_, _> = patches
            .iter()
            .filter_map(|(key, rendered, _)| rendered.presentation.map(|style| (*key, style)))
            .collect();
        // This path preserves topology and node metadata. Existing presentations
        // were validated at projection admission; validate only changed patches.
        for (key, style) in &styles {
            let mut node = self.view.bindings[key].node;
            if node.presentation != *style {
                node.presentation = *style;
                validate_presentation(&node)?;
            }
        }
        // Host hooks can stage resources or reject a turn even for text-only
        // updates. Preserve their complete candidate contract. Without a hook,
        // no child declarations can be added by a presentation/text patch.
        let admission = if self.validator.is_some() {
            let nodes = self.view.order.iter().map(|key| {
                let mut node = self.view.bindings[key].node;
                if let Some(style) = styles.get(key) {
                    node.presentation = *style;
                }
                node
            });
            admission::validate(&mut self.validator, nodes)?
        } else {
            None
        };
        for (key, rendered, versions) in patches {
            let dependencies = rendered.dependencies;
            let old_dependencies = self.view.bindings[&key].dependencies.clone();
            self.view.unindex_binding(key, &old_dependencies);
            self.view.index_binding(key, &dependencies);
            let binding = self.view.bindings.get_mut(&key).unwrap();
            binding.text = rendered.text;
            if let Some(style) = rendered.presentation {
                binding.node.presentation = style;
            }
            binding.text_dependencies = rendered.text_dependencies;
            binding.style_dependencies = rendered.style_dependencies;
            binding.versions = versions;
            binding.dependencies = dependencies;
            binding.evaluations += u64::from(rendered.text_evaluated);
            binding.style_evaluations += u64::from(rendered.style_evaluated);
        }
        Ok(admission)
    }
    pub fn enqueue(&mut self, message: Message) -> Result<(), String> {
        if self.mailbox.len() >= 256 {
            return Err("mailbox full".into());
        }
        self.mailbox.push_back(message.into());
        Ok(())
    }
    /// Transfer host-owned payloads without copying; backpressure returns ownership.
    pub(crate) fn enqueue_envelope(&mut self, envelope: Envelope) -> Option<Envelope> {
        if self.mailbox.len() >= 256 {
            return Some(envelope);
        }
        self.mailbox.push_back(envelope);
        None
    }
    /// Deliver due browser timers as owned messages between actor turns.
    #[cfg(target_arch = "wasm32")]
    pub fn poll_browser(&mut self) -> Result<bool, String> {
        let delivered = self.timer_executor.poll();
        if delivered {
            self.tick()?;
        }
        Ok(delivered)
    }
    pub fn tick(&mut self) -> Result<(), String> {
        let mut io_budget_exhausted = false;
        let result = self.tick_turn(&mut io_budget_exhausted);
        // Wake providers may coalesce notifications. Yield between batches, but
        // never depend on a new producer event to drain already admitted work.
        // Keep this outside tick_turn so an action error also rearms remaining work.
        if !self.mailbox.is_empty() || io_budget_exhausted {
            self.task_wake.notify();
        }
        result
    }
    fn tick_turn(&mut self, io_budget_exhausted: &mut bool) -> Result<(), String> {
        for index in 0..32 {
            if self.mailbox.len() >= 256 {
                break;
            }
            let Ok(completion) = self.io_receiver.try_recv() else {
                break;
            };
            if index == 31 {
                // The receiver has no non-consuming length query. One extra
                // probe turn after an exact full batch is bounded and goes idle.
                *io_budget_exhausted = true;
            }
            if !self
                .tasks
                .get(&completion.id)
                .is_some_and(|task| task.generation == completion.generation)
            {
                continue;
            }
            self.tasks.remove(&completion.id);
            let result = std::sync::Arc::new(completion.result);
            self.turn
                .io_results
                .insert(completion.id, Some(result.clone()));
            let mut envelope: Envelope = completion.message.into();
            envelope.io_result = Some((completion.id, std::sync::Arc::downgrade(&result)));
            self.mailbox.push_back(envelope);
        }
        while self.mailbox.len() < 256 {
            let Ok(completion) = self.receiver.try_recv() else {
                break;
            };
            if !self
                .tasks
                .get(&completion.id)
                .is_some_and(|task| task.io.is_none() && task.generation == completion.generation)
            {
                continue;
            }
            let io_result = if self.tasks[&completion.id].network {
                let result =
                    network_task::complete(&mut self.turn, completion.id, completion.network);
                Some((completion.id, std::sync::Arc::downgrade(&result)))
            } else {
                None
            };
            let repeating = self.tasks[&completion.id].repeating;
            let mut envelope: Envelope = completion.message.into();
            envelope.message_text = completion.text.map(|text| text.to_string());
            envelope.io_result = io_result;
            if repeating {
                envelope.timer_ack = Some((completion.id, completion.generation));
            } else {
                self.tasks.remove(&completion.id);
            }
            self.mailbox.push_back(envelope);
        }
        // A bounded turn permits reload even while a producer remains active.
        for _ in 0..32 {
            let Some(message) = self.mailbox.pop_front() else {
                break;
            };
            // A result callback can only observe its own immutable result. Cancel
            // or task reuse invalidates this marker without retaining old buffers.
            if let Some((id, result)) = &message.io_result
                && !result.upgrade().is_some_and(|expected| {
                    self.turn
                        .io_results
                        .get(id)
                        .and_then(Option::as_ref)
                        .is_some_and(|current| std::sync::Arc::ptr_eq(current, &expected))
                })
            {
                continue;
            }
            if let Some(token) = message.sdk_request
                && self
                    .turn
                    .router
                    .as_ref()
                    .is_some_and(|router| router.borrow().cancelled(token))
            {
                continue;
            }
            if let Some((id, tracker)) = message.request {
                let target = ActorId(self.id);
                if !tracker.borrow().is_pending(
                    id,
                    target,
                    self.lifecycle,
                    std::time::Instant::now(),
                ) {
                    continue;
                }
                if message.text_request {
                    let result = self.dispatch_text_request(
                        message.message.action,
                        message.message_text.unwrap_or_default(),
                    );
                    match result {
                        Ok(Some(text)) => {
                            tracker.borrow_mut().resolve_text(
                                id,
                                target,
                                self.lifecycle,
                                text,
                                std::time::Instant::now(),
                            );
                        }
                        Ok(None) => {}
                        Err(error) => {
                            tracker.borrow_mut().handler_failed(id, &error);
                            return Err(error);
                        }
                    }
                    continue;
                }
                match self.dispatch_request(message.message) {
                    Ok(Some(value)) => {
                        tracker.borrow_mut().resolve(
                            id,
                            target,
                            self.lifecycle,
                            value,
                            std::time::Instant::now(),
                        );
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracker.borrow_mut().handler_failed(id, &error);
                        return Err(error);
                    }
                }
            } else {
                self.turn.request_error = message.request_error;
                self.turn.message_text = message.message_text;
                let result = self.dispatch(message.message);
                self.turn.request_error = None;
                self.turn.message_text = None;
                if let Some((id, generation)) = message.timer_ack
                    && let Some(task) = self.tasks.get(&id)
                    && task.generation == generation
                    && let Some(timer) = &task.timer
                {
                    timer.acknowledge();
                }
                result?;
            }
        }
        Ok(())
    }
    pub fn set_wake(&mut self, wake: std::sync::Arc<dyn Fn() + Send + Sync>) {
        self.future_executor.set_wake(wake.clone());
        self.wake = Some(wake);
        // Work may have completed before this callback existed. One initial
        // notification drains that backlog without consuming completion data here.
        self.task_wake.notify();
    }
    pub fn delay(&mut self, id: u64, duration: Duration, message: Message) -> Result<(), String> {
        self.delay_with_policy(id, duration, message, 0)
    }
    /// A host-owned fixed-delay interval. At most one undelivered callback is queued.
    /// Its period begins again after delivery, so missed time never causes catch-up bursts.
    pub fn interval(&mut self, id: u64, period: Duration, message: Message) -> Result<(), String> {
        self.start_timer(id, period, message, 0, true)
    }
    fn delay_with_policy(
        &mut self,
        id: u64,
        duration: Duration,
        message: Message,
        policy: u32,
    ) -> Result<(), String> {
        self.start_timer(id, duration, message, policy, false)
    }
    fn start_timer(
        &mut self,
        id: u64,
        duration: Duration,
        message: Message,
        policy: u32,
        repeating: bool,
    ) -> Result<(), String> {
        if (repeating && duration < Duration::from_millis(1))
            || id == 0
            || message.action == 0
            || policy > 2
            || duration > Duration::from_secs(86_400)
        {
            return Err("invalid timer identity, action, policy or delay (maximum one day)".into());
        }
        if self.tasks.len() >= 64 {
            return Err("task limit reached".into());
        }
        if self.tasks.contains_key(&id) {
            return Err("duplicate task identity".into());
        }
        let generation = self
            .task_generation
            .checked_add(1)
            .ok_or("task generation exhausted")?;
        let prepared =
            self.prepare_timer_with_repeat(id, duration, message, policy, generation, repeating)?;
        self.task_generation = generation;
        self.tasks.insert(id, prepared.activate());
        Ok(())
    }
    #[cfg(test)]
    fn prepare_timer(
        &self,
        id: u64,
        duration: Duration,
        message: Message,
        policy: u32,
        generation: u64,
    ) -> Result<PreparedTimer, String> {
        self.prepare_timer_with_repeat(id, duration, message, policy, generation, false)
    }
    fn prepare_timer_with_repeat(
        &self,
        id: u64,
        duration: Duration,
        message: Message,
        policy: u32,
        generation: u64,
        repeating: bool,
    ) -> Result<PreparedTimer, String> {
        timer::validate_deadline(duration)?;
        let started = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = self.timer_executor.prepare(
            self.id,
            timer::Spec {
                completion: TimerCompletion {
                    text: None,
                    network: None,
                    id,
                    generation,
                    message,
                },
                duration,
                repeating,
                cancelled: cancelled.clone(),
                started: started.clone(),
            },
            self.sender.clone(),
            self.task_wake.clone(),
        )?;
        Ok(PreparedTimer {
            started,
            task: Task {
                cancelled,
                future: None,
                timer: Some(handle),
                policy,
                duration,
                repeating,
                message,
                io: None,
                network: false,
                generation,
            },
        })
    }
    /// Register a connection owned by the stable executable (maximum 64).
    /// IDs must be nonzero and unique; an error drops the supplied connection.
    /// Connect asynchronously outside the GUI thread before admission. This does
    /// not retain plugin code, reconnect protocols, or change socket flags.
    pub fn register_socket(
        &mut self,
        id: u64,
        stream: TcpStream,
        policy: SocketReloadPolicy,
    ) -> Result<(), String> {
        if self
            .turn
            .socket_pending
            .values()
            .any(|resource| *resource == id)
            || self.turn.sockets.len() + self.turn.socket_pending.len() >= 64
        {
            return Err("socket identity reserved or resource capacity reached".into());
        }
        self.turn.sockets.insert(id, stream, policy)
    }
    /// Borrow a host connection. Perform blocking I/O on host workers, never
    /// the GUI thread. Clones outlive removal and have their own explicit owners.
    pub fn socket(&self, id: u64) -> Option<&TcpStream> {
        self.turn.sockets.get(id)
    }
    /// Transfer the connection back to the stable host. Removing it does not
    /// shut down independently cloned descriptors or undo network effects.
    pub fn remove_socket(&mut self, id: u64) -> Option<std::sync::Arc<TcpStream>> {
        self.turn.sockets.remove(id)
    }
    pub fn socket_count(&self) -> usize {
        self.turn.sockets.len()
    }

    /// Spawn a Rust future compiled into the stable host executable.
    /// Its result is delivered to the current actor implementation after reload.
    /// Never use this to retain a future or callback originating in a dylib.
    /// Poll and Drop must not block: cancellation waits for worker-side destruction. Panics during
    /// polling produce `on_panic`. These tasks preserve across compatible reloads.
    pub fn spawn_async<F>(&mut self, id: u64, future: F, on_panic: Message) -> Result<(), String>
    where
        F: std::future::Future<Output = Message> + Send + 'static,
    {
        self.spawn_async_with_policy(id, future, on_panic, AsyncReloadPolicy::Preserve)
    }
    /// As `spawn_async`, with cancellation after a successful reload commit.
    /// Failed candidate preparation does not apply the cancellation policy.
    pub fn spawn_async_with_policy<F>(
        &mut self,
        id: u64,
        future: F,
        on_panic: Message,
        policy: AsyncReloadPolicy,
    ) -> Result<(), String>
    where
        F: std::future::Future<Output = Message> + Send + 'static,
    {
        self.spawn_async_message(id, async move { future.await.into() }, on_panic, policy)
    }
    /// Spawn a stable-host future returning a bounded owned scalar or text message.
    /// Text reaches the current SDK TextAction through the ordinary mailbox. All
    /// future polling/destruction must remain nonblocking and compiled into the host.
    pub fn spawn_async_message<F>(
        &mut self,
        id: u64,
        future: F,
        on_panic: Message,
        policy: AsyncReloadPolicy,
    ) -> Result<(), String>
    where
        F: std::future::Future<Output = AsyncMessage> + Send + 'static,
    {
        if id == 0 || on_panic.action == 0 {
            return Err("invalid task identity or panic action".into());
        }
        if self.tasks.len() >= 64 {
            return Err("task limit reached".into());
        }
        if self.tasks.contains_key(&id) {
            return Err("duplicate task identity".into());
        }
        let generation = self
            .task_generation
            .checked_add(1)
            .ok_or("task generation exhausted")?;
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let handle = self.future_executor.spawn_message(
            self.id,
            TimerCompletion {
                text: None,
                network: None,
                id,
                generation,
                message: on_panic,
            },
            future,
            self.sender.clone(),
            self.wake.clone(),
        )?;
        self.task_generation = generation;
        self.tasks.insert(
            id,
            Task {
                cancelled,
                future: Some(handle),
                timer: None,
                policy: match policy {
                    AsyncReloadPolicy::Preserve => 0,
                    AsyncReloadPolicy::Cancel => 1,
                },
                duration: Duration::ZERO,
                repeating: false,
                message: on_panic,
                io: None,
                network: false,
                generation,
            },
        );
        Ok(())
    }
    /// Cancel host work and discard an unread I/O result. Returns whether a
    /// running task or retained result existed. Ordinary admitted mailbox messages
    /// are independent. I/O-result callbacks are discarded when their result
    /// is cancelled/replaced, preventing them from observing a later task result.
    pub fn cancel_task(&mut self, id: u64) -> bool {
        let running = self.tasks.remove(&id).is_some();
        self.turn.socket_pending.remove(&id);
        self.turn.network_permits.remove(&id);
        let result = self.turn.io_results.remove(&id).is_some();
        running || result
    }
    pub fn artifact_count(&self) -> usize {
        fs::read_dir(&self.directory).unwrap().count()
    }
    /// Disk allocation occupied by this actor's copied implementation generations.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn artifact_allocated_bytes(&self) -> Result<u64, String> {
        fs::read_dir(&self.directory)
            .map_err(|e| e.to_string())?
            .try_fold(0u64, |total, entry| {
                let entry = entry.map_err(|e| e.to_string())?;
                total
                    .checked_add(artifact::allocated(&entry.path())?)
                    .ok_or_else(|| "artifact allocation overflow".into())
            })
    }
    pub fn mapped_implementation_count(&self) -> Result<usize, String> {
        #[cfg(windows)]
        {
            artifact::mapped(&self.directory)
        }
        #[cfg(not(windows))]
        {
            let marker = format!(
                "/{}/impl-",
                self.directory.file_name().unwrap().to_string_lossy()
            );
            let maps = fs::read_to_string("/proc/self/maps").map_err(|error| error.to_string())?;
            Ok(maps
                .lines()
                .filter(|line| line.contains(&marker))
                .map(|line| {
                    line.split_whitespace()
                        .skip(5)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<BTreeSet<_>>()
                .len())
        }
    }
    pub fn boolean(&self, name: &str) -> bool {
        self.boolean_by_key(key(name))
    }
    pub fn boolean_by_key(&self, identity: u64) -> bool {
        match self.turn.storage.slots.get(&identity).map(Box::as_ref) {
            Some(Value::Boolean(value)) => *value,
            _ => panic!("expected boolean state"),
        }
    }
    #[cfg(unix)]
    pub fn file_descriptor(&self, name: &str) -> Option<i32> {
        use std::os::fd::AsRawFd;
        self.turn
            .files
            .get(&key(name))
            .map(|resource| resource.file.as_raw_fd())
    }
    fn start_read(&mut self, id: u64, recipe: io::Recipe) {
        self.task_generation = self
            .task_generation
            .checked_add(1)
            .expect("read generation reserved before turn or reload commit");
        let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let message = Message {
            action: recipe.action,
            value: id as i64,
        };
        let task = Task {
            cancelled: cancelled.clone(),
            future: None,
            timer: None,
            policy: recipe.policy,
            duration: Duration::ZERO,
            repeating: false,
            message,
            io: Some(recipe.clone()),
            network: false,
            generation: self.task_generation,
        };
        self.io_executor.submit(io::Job {
            sender: None,
            id,
            generation: task.generation,
            recipe,
            cancelled,
            wake: self.task_wake.clone(),
            message,
        });
        self.tasks.insert(id, task);
    }
    /// Active/queued/cached SDK network payload reservation across the shared pool.
    pub fn network_buffer_bytes(&self) -> usize {
        self.network_pool.buffer_bytes()
    }
    /// Shared reactor count; no reactor is started until a network request is prepared.
    pub fn network_worker_count(&self) -> usize {
        self.network_pool.worker_count()
    }
    pub fn network_waiting_count(&self) -> usize {
        self.network_pool.waiting_count()
    }
    /// Worker count of the attached pool, shared by actors in one Runtime.
    pub fn io_worker_count(&self) -> usize {
        self.io_executor.worker_count()
    }
    /// Queued jobs across the attached pool, including sibling actors.
    pub fn io_queue_depth(&self) -> usize {
        self.io_executor.queue_depth()
    }
    pub fn io_result_count(&self) -> usize {
        self.turn.io_results.len()
    }
    pub fn io_result_bytes(&self) -> usize {
        self.turn
            .io_results
            .values()
            .filter_map(Option::as_ref)
            .map(|result| match &**result {
                Ok(bytes) => bytes.len(),
                Err(error) => error.len(),
            })
            .sum()
    }
}
impl Drop for Actor {
    fn drop(&mut self) {
        self.implementation.take();
        let _ = fs::remove_dir(&self.directory);
    }
}

#[cfg(feature = "native-window")]
mod dev_process;
#[cfg(feature = "native-window")]
pub mod native;

#[cfg(feature = "native-window")]
pub mod native_model;
#[cfg(feature = "native-window")]
pub mod native_slider;

#[cfg(test)]
mod timer_input_tests {
    use super::*;
    #[test]
    fn periodic_mailbox_backlog_does_not_rearm_until_its_callback_is_processed() {
        let root =
            std::env::temp_dir().join(format!("identity-periodic-backlog-{}", std::process::id()));
        let mut actor = Actor::new(549, &root).unwrap();
        let (tx, rx) = mpsc::channel();
        actor.set_wake(std::sync::Arc::new(move || {
            let _ = tx.send(());
        }));
        while rx.try_recv().is_ok() {}
        let message = Message {
            action: 7,
            value: 0,
        };
        assert!(actor.interval(1, Duration::ZERO, message).is_err());
        assert!(actor.interval(1, Duration::from_nanos(1), message).is_err());
        assert!(
            actor
                .interval(1, Duration::from_secs(86_401), message)
                .is_err()
        );
        actor
            .interval(1, Duration::from_millis(5), message)
            .unwrap();
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        actor.mailbox.push_back(message.into());
        // This actor intentionally has no implementation: the first ordinary
        // message fails, leaving the admitted interval envelope behind it.
        assert!(actor.tick().is_err());
        assert_eq!(actor.mailbox.len(), 1);
        assert!(actor.mailbox.front().unwrap().timer_ack.is_some());
        assert!(
            actor
                .receiver
                .recv_timeout(Duration::from_millis(20))
                .is_err()
        );
        assert!(actor.tick().is_err()); // periodic callback fails but is acknowledged
        assert!(actor.receiver.recv_timeout(Duration::from_secs(2)).is_ok());
        assert!(actor.cancel_task(1));
        drop(actor);
        fs::remove_dir(root).unwrap();
    }

    #[test]
    fn installing_wake_notifies_already_queued_completions() {
        let root =
            std::env::temp_dir().join(format!("identity-queued-late-wake-{}", std::process::id()));
        let mut actor = Actor::new(993, &root).unwrap();
        let message = Message {
            action: 7,
            value: 42,
        };
        actor.delay(1, Duration::ZERO, message).unwrap();
        actor
            .spawn_async(2, std::future::ready(message), message)
            .unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while !actor.tasks.values().all(Task::is_finished) {
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        }
        let (tx, rx) = mpsc::channel();
        actor.set_wake(std::sync::Arc::new(move || {
            tx.send(()).unwrap();
        }));
        let notified = rx.try_recv().is_ok();
        assert_eq!(actor.receiver.try_iter().count(), 2);
        drop(actor);
        std::fs::remove_dir(root).unwrap();
        assert!(
            notified,
            "completed work needs an initial wake when its callback is installed"
        );
    }
    #[test]
    fn prepared_timer_uses_later_installed_host_callback() {
        let root =
            std::env::temp_dir().join(format!("identity-timer-late-wake-{}", std::process::id()));
        let mut actor = Actor::new(995, &root).unwrap();
        let message = Message {
            action: 7,
            value: 42,
        };
        let prepared = actor
            .prepare_timer(1, Duration::ZERO, message, 0, 1)
            .unwrap();
        let (tx, rx) = mpsc::channel();
        actor.set_wake(std::sync::Arc::new(move || {
            tx.send(()).unwrap();
        }));
        rx.try_recv().unwrap(); // callback installation's initial backlog notification
        let task = prepared.activate();
        assert_eq!(
            actor
                .receiver
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .message
                .value,
            42
        );
        rx.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(task);
        drop(actor);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built Counter DLL; run scripts/verify_requests.py"]
    fn failed_restart_preparation_retains_old_code_state_and_timer() {
        let library = std::env::var("ACTOR_TEST_LIBRARY").unwrap();
        let replacement = std::env::var("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!(
            "identity-restart-preparation-{}",
            std::process::id()
        ));
        let mut actor = Actor::new(996, &root).unwrap();
        actor.reload(Path::new(&library)).unwrap();
        let version = actor.implementation_version();
        let slot = actor.slot_address("count");
        actor
            .delay_with_policy(
                1,
                Duration::from_secs(86_400),
                Message {
                    action: key("increment"),
                    value: 0,
                },
                2,
            )
            .unwrap();
        let worker = actor.tasks[&1].worker_id();
        actor.tasks.get_mut(&1).unwrap().duration = Duration::MAX;
        assert!(
            actor
                .reload(Path::new(&replacement))
                .unwrap_err()
                .contains("deadline out of range")
        );
        assert_eq!(actor.tasks[&1].worker_id(), worker);
        assert!(
            !actor.tasks[&1]
                .cancelled
                .load(std::sync::atomic::Ordering::Relaxed)
        );
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.slot_address("count"), slot);
        actor
            .dispatch(Message {
                action: key("increment"),
                value: 0,
            })
            .unwrap();
        assert_eq!(actor.integer("count"), 1);
        actor.tasks.get_mut(&1).unwrap().duration = Duration::ZERO;
        actor.reload(Path::new(&replacement)).unwrap();
        let completion = actor.receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(actor.tasks[&1].generation, completion.generation);
        actor.dispatch(completion.message).unwrap();
        assert_eq!(actor.integer("count"), 11);
        assert_eq!(actor.slot_address("count"), slot);
        drop(actor);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn prepared_timer_cannot_publish_before_activation_or_after_rollback() {
        let root =
            std::env::temp_dir().join(format!("identity-timer-staging-{}", std::process::id()));
        let mut actor = Actor::new(997, &root).unwrap();
        let message = Message {
            action: 7,
            value: 19,
        };
        let prepared = actor
            .prepare_timer(1, Duration::ZERO, message, 0, 1)
            .unwrap();
        assert!(!prepared.started.load(std::sync::atomic::Ordering::Acquire));
        drop(prepared); // joins the dormant worker; absence of completion is definitive
        assert!(actor.receiver.try_recv().is_err());
        actor
            .turn
            .storage
            .slots
            .insert(1, Box::new(Value::Integer(5)));
        let result = actor.transaction(|actor| {
            actor
                .turn
                .journal
                .as_mut()
                .unwrap()
                .insert(1, Value::Integer(5));
            **actor.turn.storage.slots.get_mut(&1).unwrap() = Value::Integer(6);
            actor
                .turn
                .task_commands
                .push(TaskCommand::Delay(1, Duration::ZERO, message, 0));
            actor
                .turn
                .task_commands
                .push(TaskCommand::Delay(2, Duration::MAX, message, 0));
            actor.flush()
        });
        assert!(result.unwrap_err().contains("deadline out of range"));
        assert_eq!(
            **actor.turn.storage.slots.get(&1).unwrap(),
            Value::Integer(5)
        );
        assert!(actor.tasks.is_empty());
        assert_eq!(actor.task_generation, 0);
        assert!(actor.receiver.try_recv().is_err());
        let task = actor
            .prepare_timer(1, Duration::ZERO, message, 0, 1)
            .unwrap()
            .activate();
        let completion = actor.receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(completion.message.value, 19);
        drop(task);
        drop(actor);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    #[ignore = "requires built Counter DLL; run scripts/verify_requests.py"]
    fn snapshot_exhaustion_keeps_loaded_code_and_state() {
        let library = std::env::var("ACTOR_TEST_LIBRARY").unwrap();
        let replacement = std::env::var("ACTOR_TEST_REPLACEMENT").unwrap();
        let root = std::env::temp_dir().join(format!(
            "identity-snapshot-exhaustion-{}",
            std::process::id()
        ));
        let mut actor = Actor::new(999, &root).unwrap();
        actor.reload(Path::new(&library)).unwrap();
        let slot = actor.slot_address("count");
        let version = actor.implementation_version();
        let artifacts = std::fs::read_dir(&actor.directory).unwrap().count();
        actor.generation = u64::MAX;
        assert!(
            actor
                .reload(Path::new(&replacement))
                .unwrap_err()
                .contains("snapshot generation exhausted")
        );
        assert!(
            actor
                .preview_reload(Path::new(&replacement))
                .unwrap_err()
                .contains("snapshot generation exhausted")
        );
        assert_eq!(actor.generation, u64::MAX);
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.slot_address("count"), slot);
        assert_eq!(
            std::fs::read_dir(&actor.directory).unwrap().count(),
            artifacts
        );
        actor
            .dispatch(Message {
                action: key("increment"),
                value: 0,
            })
            .unwrap();
        assert_eq!(actor.integer("count"), 1);
        drop(actor);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn exhausted_sdk_admission_does_not_stage_work() {
        let mut turn = Turn {
            journal: Some(BTreeMap::new()),
            ..Turn::default()
        };
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let status = unsafe {
            request_delay(
                (&context as *const Context).cast_mut().cast(),
                1,
                0,
                7,
                0,
                0,
            )
        };
        assert_eq!(status, ERROR);
        assert!(turn.task_commands.is_empty());
        assert!(turn.task_keys.is_empty());
    }
    #[test]
    fn exhausted_reload_restarts_keep_existing_worker() {
        let root = std::env::temp_dir().join(format!(
            "identity-restart-exhaustion-{}",
            std::process::id()
        ));
        let mut actor = Actor::new(998, &root).unwrap();
        actor
            .delay_with_policy(
                1,
                Duration::from_secs(86_400),
                Message {
                    action: 7,
                    value: 0,
                },
                2,
            )
            .unwrap();
        let generation = actor.tasks[&1].generation;
        actor.task_generation = u64::MAX;
        assert!(
            actor
                .reload(Path::new("not-loaded-at-exhaustion.so"))
                .unwrap_err()
                .contains("generation exhausted")
        );
        assert_eq!(actor.tasks[&1].generation, generation);
        assert!(
            !actor.tasks[&1]
                .cancelled
                .load(std::sync::atomic::Ordering::Relaxed)
        );
        drop(actor);
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn task_generation_reservations_are_bounded_and_not_recycled() {
        let mut turn = Turn {
            task_generation_budget: 1,
            ..Turn::default()
        };
        assert!(turn.reserve_task_generation());
        // Cancel/reuse within one turn still consumes another completion identity.
        assert!(!turn.reserve_task_generation());
        assert_eq!(turn.task_generation_budget, 0);
    }
    #[test]
    fn cancellation_releases_future_worker_with_full_completion_queue() {
        let root =
            std::env::temp_dir().join(format!("identity-async-pressure-{}", std::process::id()));
        let mut actor = Actor::new(993, &root).unwrap();
        let message = Message {
            action: 7,
            value: 19,
        };
        for generation in 0..256 {
            actor
                .sender
                .try_send(TimerCompletion {
                    text: None,
                    network: None,
                    id: 999,
                    generation,
                    message,
                })
                .unwrap();
        }
        let (polled_tx, polled_rx) = mpsc::channel();
        actor
            .spawn_async(
                1,
                async move {
                    polled_tx.send(()).unwrap();
                    message
                },
                message,
            )
            .unwrap();
        polled_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(actor.cancel_task(1));
        assert!(!actor.cancel_task(1));
        assert_eq!(actor.receiver.try_iter().count(), 256);
        drop(actor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejected_future_is_not_polled_and_does_not_reuse_generation() {
        let root =
            std::env::temp_dir().join(format!("identity-async-admission-{}", std::process::id()));
        let mut actor = Actor::new(994, &root).unwrap();
        let message = Message {
            action: 7,
            value: 0,
        };
        actor.task_generation = u64::MAX;
        let polled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = polled.clone();
        assert!(
            actor
                .spawn_async(
                    1,
                    async move {
                        flag.store(true, std::sync::atomic::Ordering::Relaxed);
                        message
                    },
                    message
                )
                .unwrap_err()
                .contains("generation exhausted")
        );
        assert!(!polled.load(std::sync::atomic::Ordering::Relaxed));
        assert!(
            actor
                .delay(2, Duration::ZERO, message)
                .unwrap_err()
                .contains("generation exhausted")
        );
        assert!(actor.receiver.try_recv().is_err());
        assert!(actor.tasks.is_empty());
        assert_eq!(actor.task_generation, u64::MAX);
        drop(actor);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn host_future_panic_becomes_owned_completion() {
        let root =
            std::env::temp_dir().join(format!("identity-async-panic-{}", std::process::id()));
        let mut actor = Actor::new(992, &root).unwrap();
        actor
            .spawn_async(
                1,
                async { panic!("host future failed") },
                Message {
                    action: 7,
                    value: 19,
                },
            )
            .unwrap();
        let completion = actor.receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(completion.id, 1);
        assert_eq!(completion.message.action, 7);
        assert_eq!(completion.message.value, 19);
        assert_eq!(actor.tasks[&1].generation, completion.generation);
        drop(actor);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn host_timer_rejects_invalid_inputs_before_allocating_identity_or_thread() {
        let root =
            std::env::temp_dir().join(format!("identity-timer-input-{}", std::process::id()));
        let mut actor = Actor::new(991, &root).unwrap();
        let valid = Message {
            action: 1,
            value: 0,
        };
        for (id, duration, message) in [
            (0, Duration::ZERO, valid),
            (
                1,
                Duration::ZERO,
                Message {
                    action: 0,
                    value: 0,
                },
            ),
            (1, Duration::MAX, valid),
            (1, Duration::from_secs(86_401), valid),
        ] {
            assert!(actor.delay(id, duration, message).is_err());
            assert!(actor.tasks.is_empty());
            assert_eq!(actor.task_generation, 0);
        }
        actor.delay(1, Duration::from_secs(86_400), valid).unwrap();
        assert_eq!(actor.tasks.len(), 1);
        assert!(actor.delay(1, Duration::ZERO, valid).is_err());
        drop(actor); // cancellation must unpark the one-day timer promptly
        std::fs::remove_dir(root).unwrap();
    }
}

#[cfg(test)]
mod binding_index_tests {
    use super::*;

    #[test]
    fn shared_and_dynamic_readers_have_exact_candidate_sets() {
        let mut view = View::default();
        let a = BTreeSet::from([Dependency::State(1)]);
        let b = BTreeSet::from([Dependency::Computed(2)]);
        view.index_binding(10, &a);
        view.index_binding(20, &a);
        view.index_binding(30, &b);
        view.index_binding(40, &BTreeSet::new());
        assert_eq!(view.affected_bindings(&a), BTreeSet::from([10, 20]));
        view.unindex_binding(10, &a);
        view.index_binding(10, &b);
        assert_eq!(view.affected_bindings(&a), BTreeSet::from([20]));
        assert_eq!(view.affected_bindings(&b), BTreeSet::from([10, 30]));
        view.unindex_binding(20, &a);
        assert!(!view.dependents.contains_key(&Dependency::State(1)));
        assert!(view.affected_bindings(&a).is_empty());
    }
}

#[cfg(feature = "native-window")]
pub mod app;
#[cfg(feature = "native-window")]
pub use app::App;

#[cfg(all(test, feature = "testing"))]
mod projection_tree_tests {
    use super::*;
    struct Empty;
    impl blockui::Actor for Empty {
        const STATE: &'static [blockui::StateSpec] = &[];
        const ACTIONS: &'static [blockui::Action] = &[];
        fn view(_: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
            use blockui::IntoView;
            Ok(blockui::Column::default().into_view())
        }
    }
    struct Scoped;
    impl blockui::Actor for Scoped {
        const STATE: &'static [blockui::StateSpec] = &[];
        const ACTIONS: &'static [blockui::Action] = &[];
        const SCOPES: &'static [blockui::Scope] = &[blockui::Scope::new("raw.scope", |_| {
            use blockui::IntoView;
            Ok(blockui::Column::default().into_view())
        })];
        fn view(_: &blockui::Cx<'_>) -> blockui::Result<blockui::View> {
            use blockui::IntoView;
            Ok(Self::SCOPES[0].into_view())
        }
    }
    // Model a malformed plugin bypassing the SDK's root normalization.
    unsafe extern "C" fn malformed_scope(
        api: *const HostApi,
        id: u64,
        context: *mut c_void,
        emit: Emit,
        emit_binding: EmitBinding,
    ) -> i32 {
        struct Forward {
            context: *mut c_void,
            node: Emit,
            binding: EmitBinding,
        }
        unsafe extern "C" fn node(context: *mut c_void, mut node: Node) -> i32 {
            let forward = unsafe { &*context.cast::<Forward>() };
            if node.parent == 0 {
                node.kind = LABEL;
            }
            unsafe { (forward.node)(forward.context, node) }
        }
        unsafe extern "C" fn binding(context: *mut c_void, binding: BindingSpec) -> i32 {
            let forward = unsafe { &*context.cast::<Forward>() };
            unsafe { (forward.binding)(forward.context, binding) }
        }
        let mut forward = Forward {
            context,
            node: emit,
            binding: emit_binding,
        };
        unsafe {
            (blockui::Export::<Scoped>::TABLE.scope_view)(
                api,
                id,
                std::ptr::from_mut(&mut forward).cast(),
                node,
                binding,
            )
        }
    }
    static MALFORMED_SCOPE: Implementation = Implementation {
        scope_view: malformed_scope,
        ..blockui::Export::<Scoped>::TABLE
    };
    #[test]
    fn malformed_raw_scope_root_preserves_live_projection() {
        let mut actor = Actor::for_test::<Empty>().unwrap();
        let version = actor.implementation_version();
        let nodes: Vec<_> = actor
            .view
            .bindings
            .iter()
            .map(|(key, binding)| (*key, binding.node.kind, binding.node.parent))
            .collect();
        let error = actor
            .prepare_implementation(|_, generation| {
                Ok(Loaded {
                    event_owner: next_event_owner(),
                    scopes: Default::default(),
                    bindings: Default::default(),
                    #[cfg(not(target_arch = "wasm32"))]
                    library: None,
                    table: &MALFORMED_SCOPE,
                    path: PathBuf::new(),
                    version: generation,
                })
            })
            .err()
            .expect("malformed scope must be rejected");
        assert!(error.contains("not a container"), "{error}");
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(
            actor
                .view
                .bindings
                .iter()
                .map(|(key, binding)| (*key, binding.node.kind, binding.node.parent))
                .collect::<Vec<_>>(),
            nodes
        );
        actor.replace_for_test::<Scoped>().unwrap();
    }
    #[test]
    fn indexed_validation_accepts_forward_parents_and_rejects_invalid_graphs() {
        let mut actor = Actor::for_test::<Empty>().unwrap();
        let mut root = actor.view.bindings.values().next().unwrap().node;
        root.key = 1;
        let mut child = root;
        child.key = 2;
        child.parent = 1;
        let mut third = child;
        third.key = 3;
        let mut cases = vec![(vec![child, root], None)];
        // Reverse emission and long chains exercise reuse of completed walks;
        // every ancestor may sort after its children.
        let deep: Vec<_> = (1..=256)
            .map(|key| Node {
                key,
                parent: if key == 256 { 0 } else { key + 1 },
                ..root
            })
            .collect();
        cases.push((deep.clone(), None));
        let mut disconnected_cycle = deep;
        disconnected_cycle[254].parent = 128;
        cases.push((disconnected_cycle, Some("cyclic view tree")));
        let shared: Vec<_> = (2..=128)
            .map(|key| Node {
                key,
                parent: 1,
                ..root
            })
            .chain(std::iter::once(root))
            .collect();
        cases.push((shared, None));
        cases.push((vec![root, root], Some("duplicate view identity")));
        let mut missing = child;
        missing.parent = 99;
        cases.push((vec![root, missing], Some("invalid parent")));
        let mut nonlayout = root;
        nonlayout.kind = LABEL;
        cases.push((vec![root, nonlayout], Some("duplicate view identity")));
        nonlayout.key = 3;
        nonlayout.parent = 1;
        let mut invalid_child = child;
        invalid_child.parent = 3;
        cases.push((vec![root, invalid_child, nonlayout], Some("invalid parent")));
        let mut cycle = child;
        cycle.parent = 3;
        third.parent = 2;
        cases.push((vec![root, cycle, third], Some("cyclic view tree")));
        cycle.parent = 2;
        cases.push((vec![root, cycle], Some("cyclic view tree")));
        third.parent = 0;
        cases.push((vec![root, third], Some("view needs one layout root")));
        let mut zero = root;
        zero.key = 0;
        cases.push((vec![zero], Some("missing view identity")));
        let mut scroll_root = root;
        scroll_root.kind = SCROLL;
        cases.push((vec![scroll_root], None));
        let mut viewport = child;
        viewport.kind = SCROLL;
        viewport.semantics.role = semantics::TOAST_VIEWPORT;
        cases.push((vec![root, viewport], None));
        let mut positioned = viewport;
        positioned.overlay_position = 5;
        cases.push((vec![root, positioned], None));
        positioned.overlay_position = 6;
        cases.push((
            vec![root, positioned],
            Some("overlay position does not match its viewport role"),
        ));
        let mut misplaced = child;
        misplaced.overlay_position = 1;
        cases.push((
            vec![root, misplaced],
            Some("overlay position does not match its viewport role"),
        ));
        let mut only_viewport = viewport;
        only_viewport.parent = 0;
        cases.push((
            vec![only_viewport],
            Some("notification viewport needs an application root"),
        ));
        let mut nested = viewport;
        nested.key = 3;
        nested.parent = 2;
        cases.push((
            vec![root, viewport, nested],
            Some("notification viewport cannot nest in a dialog or viewport"),
        ));
        let mut cyclic_a = child;
        cyclic_a.parent = 3;
        let mut cyclic_b = third;
        cyclic_b.parent = 2;
        let mut cyclic_viewport = viewport;
        cyclic_viewport.key = 4;
        cyclic_viewport.parent = 2;
        cases.push((
            vec![root, cyclic_a, cyclic_b, cyclic_viewport],
            Some("cyclic view tree"),
        ));
        let mut dialog = root;
        dialog.kind = DIALOG;
        dialog.action = 7;
        cases.push((
            vec![dialog, viewport],
            Some("notification viewport cannot nest in a dialog or viewport"),
        ));
        for presentation in [
            blockui_abi::Presentation {
                width: f32::NAN,
                ..Default::default()
            },
            blockui_abi::Presentation {
                width: -1.0,
                ..Default::default()
            },
            blockui_abi::Presentation {
                font_size: 24.0,
                ..Default::default()
            },
            blockui_abi::Presentation {
                surface: 3,
                ..Default::default()
            },
            blockui_abi::Presentation {
                radius: 49.0,
                ..Default::default()
            },
            blockui_abi::Presentation {
                theme: 4,
                ..Default::default()
            },
            blockui_abi::Presentation {
                density: 3,
                ..Default::default()
            },
        ] {
            cases.push((
                vec![Node {
                    presentation,
                    ..root
                }],
                Some("invalid widget presentation"),
            ));
        }
        let mut trigger = child;
        trigger.kind = BUTTON;
        trigger.semantics = semantics::Semantics {
            expanded: 1,
            controls: 3,
            ..Default::default()
        };
        let mut panel = third;
        panel.parent = 1;
        panel.semantics = semantics::Semantics {
            role: semantics::DISCLOSURE_PANEL,
            hidden: 1,
            labelled_by: 2,
            ..Default::default()
        };
        cases.push((vec![root, trigger, panel], None));
        let mut mismatched = panel;
        mismatched.semantics.hidden = 0;
        cases.push((
            vec![root, trigger, mismatched],
            Some("disclosure needs its matching panel"),
        ));
        mismatched = panel;
        mismatched.semantics.labelled_by = 1;
        cases.push((
            vec![root, trigger, mismatched],
            Some("disclosure needs its matching panel"),
        ));
        let mut unlinked = trigger;
        unlinked.semantics.controls = 0;
        cases.push((
            vec![root, unlinked, panel],
            Some("disclosure panel needs its matching trigger"),
        ));
        let mut hidden_child = child;
        hidden_child.key = 4;
        hidden_child.parent = 3;
        cases.push((
            vec![root, trigger, panel, hidden_child],
            Some("hidden panel must have no children"),
        ));
        let mut orphan_item = child;
        orphan_item.semantics.role = semantics::ACCORDION_ITEM;
        cases.push((
            vec![root, orphan_item],
            Some("accordion item requires its matching group"),
        ));
        let popover_trigger = Node {
            semantics: semantics::Semantics {
                role: semantics::POPOVER_TRIGGER,
                expanded: 2,
                controls: 3,
                ..Default::default()
            },
            ..trigger
        };
        let popover_panel = Node {
            kind: SCROLL,
            action: 1,
            semantics: semantics::Semantics {
                role: semantics::POPOVER_PANEL,
                labelled_by: 2,
                ..Default::default()
            },
            ..panel
        };
        cases.push((vec![root, popover_trigger, popover_panel], None));
        let mut placed_popover = popover_panel;
        placed_popover.overlay_position = 11;
        cases.push((vec![root, popover_trigger, placed_popover], None));
        placed_popover.overlay_position = 12;
        cases.push((
            vec![root, popover_trigger, placed_popover],
            Some("overlay position does not match its viewport role"),
        ));
        let tooltip_trigger = Node {
            semantics: semantics::Semantics {
                described_by: 3,
                ..Default::default()
            },
            ..trigger
        };
        let tooltip = Node {
            kind: LABEL,
            action: 0,
            tooltip_anchor: 2,
            tooltip_delay_ms: 700,
            semantics: semantics::Semantics {
                role: semantics::TOOLTIP,
                ..Default::default()
            },
            ..panel
        };
        cases.push((vec![root, tooltip_trigger, tooltip], None));
        let mut bad_tooltip = tooltip;
        bad_tooltip.tooltip_delay_ms = 10001;
        cases.push((
            vec![root, tooltip_trigger, bad_tooltip],
            Some("tooltip needs a matching sibling trigger and noninteractive label"),
        ));
        bad_tooltip = tooltip;
        bad_tooltip.action = 1;
        cases.push((
            vec![root, tooltip_trigger, bad_tooltip],
            Some("tooltip needs a matching sibling trigger and noninteractive label"),
        ));
        bad_tooltip = tooltip;
        bad_tooltip.tooltip_anchor = 1;
        cases.push((
            vec![root, tooltip_trigger, bad_tooltip],
            Some("description needs its matching tooltip"),
        ));
        let mut stray = child;
        stray.tooltip_delay_ms = 1;
        cases.push((
            vec![root, stray],
            Some("tooltip timing and anchor require a tooltip"),
        ));
        let mut closed_trigger = popover_trigger;
        closed_trigger.semantics.expanded = 1;
        let mut closed_panel = popover_panel;
        closed_panel.semantics.hidden = 1;
        cases.push((vec![root, closed_trigger, closed_panel], None));
        cases.push((
            vec![root, popover_trigger, closed_panel],
            Some("popover needs its matching panel"),
        ));
        cases.push((
            vec![root, closed_trigger, popover_panel],
            Some("popover needs its matching panel"),
        ));
        cases.push((
            vec![root, popover_trigger, panel],
            Some("popover needs its matching panel"),
        ));
        let mut no_close = popover_panel;
        no_close.action = 0;
        cases.push((
            vec![root, popover_trigger, no_close],
            Some("popover needs a parent, close action and no state slot"),
        ));
        let mut recursive_anchor = popover_trigger;
        recursive_anchor.parent = popover_panel.key;
        cases.push((
            vec![root, recursive_anchor, popover_panel],
            Some("popover anchor cannot be inside its panel"),
        ));
        cases.push((
            vec![root, closed_trigger, closed_panel, hidden_child],
            Some("hidden panel must have no children"),
        ));
        let mut badge = child;
        badge.kind = LABEL;
        badge.semantics.role = semantics::BADGE;
        badge.button_variant = 3;
        cases.push((vec![root, badge], None));
        badge.button_variant = 4;
        cases.push((vec![root, badge], Some("invalid button appearance")));
        badge.button_variant = 0;
        badge.action = 1;
        cases.push((vec![root, badge], Some("badge cannot have an action")));
        let mut divider = child;
        divider.kind = COLUMN;
        divider.height = 1.0;
        divider.semantics.role = semantics::HORIZONTAL_SEPARATOR;
        cases.push((vec![root, divider], None));
        divider.height = 0.0;
        cases.push((
            vec![root, divider],
            Some("separator needs positive height and no children, action or slot"),
        ));
        divider.height = 1.0;
        divider.action = 1;
        cases.push((
            vec![root, divider],
            Some("separator needs positive height and no children, action or slot"),
        ));
        divider.action = 0;
        let mut divider_child = third;
        divider_child.parent = divider.key;
        cases.push((
            vec![root, divider, divider_child],
            Some("separator needs positive height and no children, action or slot"),
        ));
        let mut table = root;
        table.semantics.role = semantics::TABLE;
        let mut row = child;
        row.kind = ROW;
        row.semantics.role = semantics::TABLE_ROW;
        let mut cell = third;
        cell.parent = row.key;
        cell.semantics.role = semantics::CELL;
        cases.push((vec![table, row, cell], None));
        cases.push((
            vec![root, row, cell],
            Some("invalid table row or cell parent"),
        ));
        let mut invalid_cell = cell;
        invalid_cell.parent = table.key;
        cases.push((
            vec![table, row, invalid_cell],
            Some("invalid table row or cell parent"),
        ));
        invalid_cell.parent = row.key;
        invalid_cell.semantics.role = semantics::DEFAULT;
        cases.push((
            vec![table, row, invalid_cell],
            Some("invalid table row or cell parent"),
        ));
        let order = actor.view.order.clone();
        for (nodes, error) in cases {
            let authored: Vec<_> = nodes.iter().map(|node| node.key).collect();
            let collection = bindings::Collection {
                recipes: nodes
                    .iter()
                    .map(|node| (node.key, bindings::Recipe::Static("test".into())))
                    .collect(),
                nodes,
                error: None,
            };
            let result = project_collection(
                actor.implementation.as_ref().unwrap(),
                &mut actor.turn,
                collection,
                BTreeSet::new(),
                BTreeSet::new(),
                BTreeMap::new(),
                None,
            );
            match error {
                Some(expected) => {
                    let actual = result.err().expect("expected projection rejection");
                    assert!(
                        actual.contains(expected),
                        "expected {expected:?}, got {actual:?}; nodes {authored:?}"
                    );
                }
                None => assert_eq!(
                    result
                        .unwrap()
                        .bindings
                        .iter()
                        .map(|binding| binding.node.key)
                        .collect::<Vec<_>>(),
                    authored
                ),
            }
            assert_eq!(actor.view.order, order);
        }
    }
}

#[cfg(test)]
mod handler_diagnostic_tests {
    use super::*;
    #[test]
    fn clock_requires_action_scope_and_rejects_null_outputs() {
        let mut turn = Turn::default();
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = (&context as *const Context).cast_mut().cast();
        let mut first = u64::MAX;
        assert_eq!(unsafe { monotonic_ms(ctx, &mut first) }, ERROR);
        assert_eq!(first, u64::MAX);
        turn.journal = Some(BTreeMap::new());
        assert_eq!(unsafe { monotonic_ms(ctx, &mut first) }, OK);
        let mut second = 0;
        assert_eq!(unsafe { monotonic_ms(ctx, &mut second) }, OK);
        assert!(second >= first);
        assert_eq!(unsafe { monotonic_ms(ctx, std::ptr::null_mut()) }, ERROR);
        assert_eq!(
            unsafe { monotonic_ms(std::ptr::null_mut(), &mut second) },
            ERROR
        );
        turn.read_only = true;
        assert_eq!(unsafe { monotonic_ms(ctx, &mut second) }, ERROR);
    }
    #[test]
    fn staged_text_reply_is_bounded_owned_and_requires_one_writable_text_scope() {
        let mut turn = Turn {
            journal: Some(BTreeMap::new()),
            ..Turn::default()
        };
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = (&context as *const Context).cast_mut().cast();
        assert_eq!(
            unsafe { stage_text_reply(ctx, Bytes::new("absent")) },
            ERROR
        );
        turn.request_reply = Some(None);
        assert_eq!(
            unsafe { stage_text_reply(ctx, Bytes::new("scalar")) },
            ERROR
        );
        turn.request_reply = None;
        turn.request_text_reply = Some(None);
        for bytes in [
            Bytes {
                ptr: std::ptr::null(),
                len: MAX_MESSAGE_TEXT_BYTES + 1,
            },
            Bytes {
                ptr: std::ptr::null(),
                len: 1,
            },
            Bytes {
                ptr: [0xff].as_ptr(),
                len: 1,
            },
        ] {
            assert_eq!(unsafe { stage_text_reply(ctx, bytes) }, ERROR);
        }
        turn.read_only = true;
        assert_eq!(
            unsafe { stage_text_reply(ctx, Bytes::new("read-only")) },
            ERROR
        );
        turn.read_only = false;
        turn.journal = None;
        assert_eq!(
            unsafe { stage_text_reply(ctx, Bytes::new("no turn")) },
            ERROR
        );
        turn.journal = Some(BTreeMap::new());
        let mut text = String::from("資料 😀");
        assert_eq!(unsafe { stage_text_reply(ctx, Bytes::new(&text)) }, OK);
        text.clear();
        assert_eq!(
            turn.request_text_reply.as_ref().unwrap().as_deref(),
            Some("資料 😀")
        );
        assert_eq!(
            unsafe { stage_text_reply(ctx, Bytes::new("duplicate")) },
            ERROR
        );
        assert_eq!(unsafe { stage_reply(ctx, 42) }, ERROR);
        assert!(turn.changed.is_empty());
    }
    #[test]
    fn text_message_buffers_validate_and_copy_without_state_reads() {
        let router = std::rc::Rc::new(std::cell::RefCell::new(system::Router::default()));
        let mut turn = Turn {
            router: Some(router),
            journal: Some(BTreeMap::new()),
            ..Turn::default()
        };
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = (&context as *const Context).cast_mut().cast();
        assert_eq!(
            unsafe { read_message_text(ctx, std::ptr::null_mut(), 0) },
            -1
        );
        turn.message_text = Some(String::new());
        assert_eq!(
            unsafe { read_message_text(ctx, std::ptr::null_mut(), 0) },
            0
        );
        turn.message_text = Some("资料😀".into());
        let mut short = [7; 9];
        assert_eq!(
            unsafe { read_message_text(ctx, short.as_mut_ptr(), short.len()) },
            -1
        );
        assert_eq!(short, [7; 9]);
        let mut bytes = [0; 10];
        assert_eq!(
            unsafe { read_message_text(ctx, bytes.as_mut_ptr(), bytes.len()) },
            10
        );
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "资料😀");
        assert!(turn.reads.is_empty());
        assert!(turn.changed.is_empty());
    }

    #[test]
    fn request_detail_reads_use_borrowed_buffers_without_publishing_state() {
        let mut turn = Turn {
            request_error: Some("资料😀".into()),
            ..Turn::default()
        };
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = (&context as *const Context).cast_mut().cast();
        assert_eq!(
            unsafe { read_request_error(ctx, std::ptr::null_mut(), 0) },
            10
        );
        let mut short = [7; 9];
        assert_eq!(
            unsafe { read_request_error(ctx, short.as_mut_ptr(), short.len()) },
            -1
        );
        assert_eq!(short, [7; 9]);
        let mut bytes = [0; 10];
        assert_eq!(
            unsafe { read_request_error(ctx, bytes.as_mut_ptr(), bytes.len()) },
            10
        );
        assert_eq!(std::str::from_utf8(&bytes).unwrap(), "资料😀");
        assert!(turn.changed.is_empty());
        turn.request_error = None;
        assert_eq!(
            unsafe { read_request_error(ctx, std::ptr::null_mut(), 0) },
            0
        );
    }
    #[test]
    fn borrowed_diagnostics_are_copied_bounded_utf8_and_first_failure_wins() {
        let mut turn = Turn::default();
        let context = Context {
            turn: &mut turn,
            table: std::ptr::null(),
        };
        let ctx = (&context as *const Context).cast_mut().cast();
        let oversized = Bytes {
            ptr: std::ptr::null(),
            len: 4097,
        };
        assert_eq!(unsafe { report_error(ctx, oversized) }, ERROR);
        assert_eq!(
            unsafe {
                report_error(
                    ctx,
                    Bytes {
                        ptr: std::ptr::null(),
                        len: 1,
                    },
                )
            },
            ERROR
        );
        let invalid = [0xff];
        assert_eq!(
            unsafe {
                report_error(
                    ctx,
                    Bytes {
                        ptr: invalid.as_ptr(),
                        len: 1,
                    },
                )
            },
            ERROR
        );
        assert!(turn.handler_error.is_none());
        let owned = "Validation: 资料😀".to_string();
        assert_eq!(unsafe { report_error(ctx, Bytes::new(&owned)) }, OK);
        drop(owned);
        assert_eq!(unsafe { report_error(ctx, Bytes::new("later")) }, OK);
        assert_eq!(turn.handler_error.as_deref(), Some("Validation: 资料😀"));
    }
}

/// Host-owned selection interaction model; native rendering integration is pending.
pub mod select_model;

#[cfg(all(test, feature = "testing"))]
mod select_tests;

/// Shared logical geometry for Select popup paint and hit testing.
pub mod select_popup;
