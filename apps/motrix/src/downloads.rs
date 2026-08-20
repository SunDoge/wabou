use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use wabou::{FunctionModule, NativeMethod};
use wabou::{
    HostMessageContext, HostMethod, HostService, HostServiceHandle, JsonCapability,
    JsonCapabilityContract, JsonMethod, ManagedHostService, NativeCapability,
    RevisionedHostPublisher, RevisionedHostSnapshot, managed_host_service, rquickjs,
};

use crate::{
    activity::ActivityLog,
    config::{AppConfig, ConfigStore},
    download_backend::{
        DownloadEvent, DownloadTask, GoshBackend, HttpDownloadRequest, TaskPriority, TaskState,
    },
    nat::{NatManager, NatStatus},
    torrent::{TorrentPreview, read_torrent},
};

pub const CAPABILITY: JsonCapabilityContract = JsonCapabilityContract::new("downloads", 1);
pub const NATIVE_CAPABILITY: JsonCapabilityContract =
    JsonCapabilityContract::new("downloadsNative", 1);
const SNAPSHOT: &str = "downloads.snapshot";
const SNAPSHOT_PATCH: &str = "downloads.snapshot.patch";
pub const QUIT_REQUESTED: &str = "motrix.quitRequested";

const GET_SNAPSHOT: JsonMethod<(), Snapshot> = JsonMethod::no_request("getSnapshot");
const ADD_URI: JsonMethod<AddUriRequest, Vec<String>> = JsonMethod::new("addUri");
const ADD_TORRENT: JsonMethod<AddTorrentRequest, String> = JsonMethod::new("addTorrent");
const INSPECT_TORRENT: HostMethod<InspectTorrentRequest, TorrentPreview> =
    HostMethod::new("inspectTorrent");
const TASK_ACTION: JsonMethod<TaskActionRequest, ()> = JsonMethod::new("taskAction");
const SET_TASK_PRIORITY: JsonMethod<SetTaskPriorityRequest, ()> =
    JsonMethod::new("setTaskPriority");
const BATCH_TASK_ACTION: JsonMethod<BatchTaskActionRequest, Vec<String>> =
    JsonMethod::new("batchTaskAction");
const GET_CONFIG: JsonMethod<(), AppConfig> = JsonMethod::no_request("getConfig");
const SET_CONFIG: JsonMethod<AppConfig, SetConfigResult> = JsonMethod::new("setConfig");
const OPEN_TASK_FOLDER: JsonMethod<OpenTaskFolderRequest, ()> = JsonMethod::new("openTaskFolder");
const OPEN_PATH: JsonMethod<OpenTaskFolderRequest, ()> = JsonMethod::new("openPath");
const OPEN_CONFIG_FOLDER: JsonMethod<(), ()> = JsonMethod::no_request("openConfigFolder");
const GLOBAL_TASK_ACTION: JsonMethod<GlobalTaskActionRequest, ()> =
    JsonMethod::new("globalTaskAction");
const GET_TASK_DETAILS: JsonMethod<GetTaskDetailsRequest, TaskDetails> =
    JsonMethod::new("getTaskDetails");
const RETRY_ENGINE: JsonMethod<(), ()> = JsonMethod::no_request("retryEngine");
static TEST_ENGINE_FAILURE_INJECTED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DownloadUriKind {
    Http,
    Magnet,
}

fn classify_download_uri(value: &str) -> Result<DownloadUriKind, String> {
    let parsed =
        url::Url::parse(value).map_err(|error| format!("invalid download URI: {error}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(DownloadUriKind::Http),
        "magnet" => Ok(DownloadUriKind::Magnet),
        scheme => Err(format!(
            "unsupported download URI scheme `{scheme}`; expected HTTP, HTTPS, or magnet"
        )),
    }
}

#[derive(Clone)]
pub struct DownloadService {
    runtime: HostServiceHandle<DownloadRuntime>,
    engine_service: ManagedHostService<DownloadRuntime>,
}

#[derive(Clone)]
struct DownloadRuntime {
    backend: GoshBackend,
    engine_config: AppConfig,
    config: Arc<RwLock<AppConfig>>,
    config_store: ConfigStore,
    activity: Arc<Mutex<ActivityLog>>,
    nat: Arc<NatManager>,
    revision: Arc<AtomicU64>,
    network_discovery_enabled: bool,
}

impl DownloadService {
    pub fn new() -> (Self, impl HostService) {
        let (runtime, host_service) = managed_host_service(
            "gosh-dl",
            move |context| {
                if context.is_behavior_test()
                    && std::env::var_os("WABOU_MOTRIX_TEST_ENGINE_FAILURE").is_some()
                {
                    if !TEST_ENGINE_FAILURE_INJECTED.swap(true, Ordering::AcqRel) {
                        return Err("injected download engine startup failure".to_owned());
                    }
                    std::thread::sleep(Duration::from_millis(1_500));
                }
                let directories = context.app_directories().ok_or_else(|| {
                    "Motrix download service requires HostBuilder app directories".to_owned()
                })?;
                let config_store = ConfigStore::new(&directories.config_dir);
                let config = config_store.load()?;
                let mut engine_config = config.clone();
                if context.is_behavior_test() {
                    engine_config.dht_enabled = false;
                    engine_config.pex_enabled = false;
                    engine_config.nat_enabled = false;
                }
                let backend = GoshBackend::start(&engine_config, &directories.data_dir)?;
                Ok(DownloadRuntime {
                    backend,
                    engine_config: config.clone(),
                    config: Arc::new(RwLock::new(config)),
                    activity: Arc::new(Mutex::new(ActivityLog::load(config_store.directory()?))),
                    config_store,
                    nat: Arc::new(NatManager::default()),
                    revision: Arc::new(AtomicU64::new(0)),
                    network_discovery_enabled: !context.is_behavior_test(),
                })
            },
            |runtime| runtime.backend.shutdown_blocking(),
        );
        (
            Self {
                runtime,
                engine_service: host_service.clone(),
            },
            host_service,
        )
    }

    fn config(&self) -> Result<AppConfig, String> {
        self.runtime()?
            .config
            .read()
            .map(|value| value.clone())
            .map_err(|_| "configuration lock poisoned".to_owned())
    }

    fn backend(&self) -> Result<GoshBackend, String> {
        Ok(self.runtime()?.backend)
    }

    fn runtime(&self) -> Result<DownloadRuntime, String> {
        self.runtime.get()
    }

    async fn save_config(&self, config: AppConfig) -> Result<SetConfigResult, String> {
        config.validate()?;
        let runtime = self.runtime()?;
        let restart_required = engine_restart_required(&runtime.engine_config, &config);
        let previous = runtime
            .config
            .read()
            .map_err(|_| "configuration lock poisoned".to_owned())?
            .clone();
        runtime
            .backend
            .set_runtime_limits(
                config.max_concurrent_downloads.max(1) as usize,
                parse_nonzero_limit(&config.max_overall_download_limit),
                parse_nonzero_limit(&config.max_overall_upload_limit),
            )
            .await?;
        if let Err(error) = runtime.config_store.save(&config) {
            let _ = runtime
                .backend
                .set_runtime_limits(
                    previous.max_concurrent_downloads.max(1) as usize,
                    parse_nonzero_limit(&previous.max_overall_download_limit),
                    parse_nonzero_limit(&previous.max_overall_upload_limit),
                )
                .await;
            return Err(error);
        }
        *runtime
            .config
            .write()
            .map_err(|_| "configuration lock poisoned".to_owned())? = config.clone();
        Ok(SetConfigResult {
            config,
            restart_required,
        })
    }

    async fn snapshot(&self) -> Snapshot {
        let runtime = match self.runtime() {
            Ok(runtime) => runtime,
            Err(error) => {
                return Snapshot::disconnected(0, error);
            }
        };
        let result = runtime.backend.list().await;
        // A revision identifies one observed snapshot, regardless of whether
        // it was requested by the capability or produced by the push loop. Allocate it
        // after the asynchronous read so completion order is publication order.
        let revision = runtime.revision.fetch_add(1, Ordering::AcqRel) + 1;
        match result {
            Ok(tasks) => self.snapshot_from_tasks(&runtime, revision, tasks),
            Err(error) => Snapshot::disconnected(revision, error),
        }
    }

    fn snapshot_from_tasks(
        &self,
        runtime: &DownloadRuntime,
        revision: u64,
        tasks: Vec<DownloadTask>,
    ) -> Snapshot {
        let tasks = tasks
            .into_iter()
            .map(TaskSnapshot::from)
            .collect::<Vec<_>>();
        let download_speed = tasks.iter().map(|task| task.download_speed).sum();
        let upload_speed = tasks.iter().map(|task| task.upload_speed).sum();
        let (activity, downloaded_today, downloaded_total, uploaded_total) =
            if let Ok(mut log) = runtime.activity.lock() {
                log.observe(tasks.iter().map(|task| {
                    (
                        task.id.as_str(),
                        task.completed_length,
                        task.uploaded_length,
                    )
                }));
                (
                    log.recent(),
                    log.downloaded_today(),
                    log.downloaded_total(),
                    log.uploaded_total(),
                )
            } else {
                (vec![0; 364], 0, 0, 0)
            };
        let nat = if runtime.network_discovery_enabled {
            self.config()
                .map(|config| {
                    runtime.nat.sync(&config);
                    runtime.nat.status(config.nat_enabled)
                })
                .unwrap_or_default()
        } else {
            NatStatus::default()
        };
        Snapshot {
            revision,
            status: ServiceStatus::Ready,
            version: Some("gosh-dl 0.5.0".to_owned()),
            error: None,
            download_speed,
            upload_speed,
            tasks,
            activity,
            downloaded_today,
            downloaded_total,
            uploaded_total,
            nat,
        }
    }

    async fn details(&self, id: &str) -> Result<TaskDetails, String> {
        let task = self.backend()?.task(id).await?;
        Ok(TaskDetails {
            files: vec![TaskFileDetails {
                index: 1,
                path: task
                    .save_dir
                    .join(&task.name)
                    .to_string_lossy()
                    .into_owned(),
                length: task.total_size.unwrap_or_default(),
                completed_length: task.completed_size,
                selected: true,
            }],
        })
    }

    async fn task_action(
        &self,
        id: &str,
        action: TaskAction,
        remove_files: bool,
    ) -> Result<(), String> {
        match action {
            TaskAction::Pause => self.backend()?.pause(id).await,
            TaskAction::Resume => self.backend()?.resume(id).await,
            TaskAction::Remove | TaskAction::StopSeeding => {
                self.backend()?.cancel(id, remove_files).await
            }
            TaskAction::Retry => self.backend()?.retry(id).await.map(|_| ()),
        }
    }
}

fn parse_nonzero_limit(value: &str) -> Option<u64> {
    crate::config::parse_byte_size(value).filter(|value| *value > 0)
}

fn engine_restart_required(previous: &AppConfig, next: &AppConfig) -> bool {
    previous.download_dir != next.download_dir
        || previous.max_connection_per_server != next.max_connection_per_server
        || previous.min_split_size != next.min_split_size
        || previous.dht_enabled != next.dht_enabled
        || previous.pex_enabled != next.pex_enabled
        || previous.bt_max_peers != next.bt_max_peers
        || previous.listen_port != next.listen_port
        || previous.seed_ratio != next.seed_ratio
        || previous.user_agent != next.user_agent
        || previous.proxy.enabled != next.proxy.enabled
        || previous.proxy.host != next.proxy.host
        || previous.proxy.port != next.proxy.port
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    revision: u64,
    status: ServiceStatus,
    version: Option<String>,
    error: Option<String>,
    download_speed: u64,
    upload_speed: u64,
    tasks: Vec<TaskSnapshot>,
    activity: Vec<u64>,
    downloaded_today: u64,
    downloaded_total: u64,
    uploaded_total: u64,
    nat: NatStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum ServiceStatus {
    Ready,
    Failed,
}

impl Snapshot {
    fn disconnected(revision: u64, error: String) -> Self {
        Self {
            revision,
            status: ServiceStatus::Failed,
            version: None,
            error: Some(error),
            download_speed: 0,
            upload_speed: 0,
            tasks: Vec::new(),
            activity: vec![0; 364],
            downloaded_today: 0,
            downloaded_total: 0,
            uploaded_total: 0,
            nat: NatStatus::default(),
        }
    }
}

impl RevisionedHostSnapshot for Snapshot {
    fn revision(&self) -> u64 {
        self.revision
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskSnapshot {
    id: String,
    name: String,
    status: String,
    total_length: u64,
    completed_length: u64,
    download_speed: u64,
    upload_speed: u64,
    uploaded_length: u64,
    dir: String,
    file_path: Option<String>,
    uri: Option<String>,
    connections: u64,
    seeders: Option<u64>,
    error_message: Option<String>,
    bittorrent: bool,
    retryable: bool,
    archived: bool,
    file_count: usize,
    priority: TaskPriority,
    created_at_ms: i64,
}

impl From<DownloadTask> for TaskSnapshot {
    fn from(task: DownloadTask) -> Self {
        let status = match task.state {
            TaskState::Waiting => "waiting",
            TaskState::Active => "active",
            TaskState::Paused => "paused",
            TaskState::Complete => "complete",
            TaskState::Error => "error",
        }
        .to_owned();
        Self {
            id: task.id,
            name: task.name.clone(),
            status,
            total_length: task.total_size.unwrap_or_default(),
            completed_length: task.completed_size,
            download_speed: task.download_speed,
            upload_speed: task.upload_speed,
            uploaded_length: 0,
            dir: task.save_dir.to_string_lossy().into_owned(),
            file_path: Some(
                task.save_dir
                    .join(&task.name)
                    .to_string_lossy()
                    .into_owned(),
            ),
            uri: task.source,
            connections: u64::from(task.connections),
            seeders: task.seeders,
            error_message: task.error,
            bittorrent: task.bittorrent,
            retryable: task.retryable,
            archived: false,
            file_count: task.file_count,
            priority: task.priority,
            created_at_ms: task.created_at_ms,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotPatch {
    base_revision: u64,
    revision: u64,
    status: ServiceStatus,
    version: Option<String>,
    error: Option<String>,
    download_speed: u64,
    upload_speed: u64,
    activity: Vec<u64>,
    downloaded_today: u64,
    downloaded_total: u64,
    uploaded_total: u64,
    nat: NatStatus,
    upserted_tasks: Vec<TaskSnapshot>,
    removed_ids: Vec<String>,
    task_order: Vec<String>,
}

impl SnapshotPatch {
    fn between(previous: &Snapshot, next: &Snapshot) -> Self {
        let old: HashMap<_, _> = previous
            .tasks
            .iter()
            .map(|task| (task.id.as_str(), task))
            .collect();
        let current: HashSet<_> = next.tasks.iter().map(|task| task.id.as_str()).collect();
        Self {
            base_revision: previous.revision,
            revision: next.revision,
            status: next.status,
            version: next.version.clone(),
            error: next.error.clone(),
            download_speed: next.download_speed,
            upload_speed: next.upload_speed,
            activity: next.activity.clone(),
            downloaded_today: next.downloaded_today,
            downloaded_total: next.downloaded_total,
            uploaded_total: next.uploaded_total,
            nat: next.nat.clone(),
            upserted_tasks: next
                .tasks
                .iter()
                .filter(|task| old.get(task.id.as_str()).copied() != Some(*task))
                .cloned()
                .collect(),
            removed_ids: previous
                .tasks
                .iter()
                .filter(|task| !current.contains(task.id.as_str()))
                .map(|task| task.id.clone())
                .collect(),
            task_order: next.tasks.iter().map(|task| task.id.clone()).collect(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddUriRequest {
    uris: Vec<String>,
    dir: Option<String>,
    out: Option<String>,
    split: Option<i32>,
    #[serde(default)]
    headers: Vec<String>,
    #[serde(default)]
    priority: TaskPriority,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddTorrentRequest {
    path: String,
    dir: Option<String>,
    selected_files: Option<Vec<u64>>,
    #[serde(default)]
    priority: TaskPriority,
}
#[derive(Deserialize)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub struct InspectTorrentRequest {
    pub path: String,
}
#[derive(Deserialize)]
struct GetTaskDetailsRequest {
    id: String,
}
#[derive(Deserialize)]
struct OpenTaskFolderRequest {
    path: String,
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TaskAction {
    Pause,
    Resume,
    Remove,
    Retry,
    StopSeeding,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TaskActionRequest {
    id: String,
    action: TaskAction,
    #[serde(default)]
    remove_files: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetTaskPriorityRequest {
    id: String,
    priority: TaskPriority,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchTaskActionRequest {
    ids: Vec<String>,
    action: TaskAction,
    #[serde(default)]
    remove_files: bool,
}
#[derive(Deserialize)]
struct GlobalTaskActionRequest {
    action: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SetConfigResult {
    config: AppConfig,
    restart_required: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskDetails {
    files: Vec<TaskFileDetails>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskFileDetails {
    index: u64,
    path: String,
    length: u64,
    completed_length: u64,
    selected: bool,
}
pub fn mount(capability: JsonCapability<'_>, service: DownloadService) -> rquickjs::Result<()> {
    let get = service.clone();
    capability.method(GET_SNAPSHOT, move |(): ()| {
        let service = get.clone();
        async move { Ok::<_, String>(service.snapshot().await) }
    })?;
    let get = service.clone();
    capability.method(GET_CONFIG, move |(): ()| {
        let service = get.clone();
        async move { service.config() }
    })?;
    let set = service.clone();
    capability.method(SET_CONFIG, move |config| {
        let service = set.clone();
        async move { service.save_config(config).await }
    })?;
    let details = service.clone();
    capability.method(GET_TASK_DETAILS, move |request: GetTaskDetailsRequest| {
        let service = details.clone();
        async move { service.details(&request.id).await }
    })?;
    let retry = service.clone();
    capability.method(RETRY_ENGINE, move |(): ()| {
        let service = retry.clone();
        async move { service.engine_service.retry_async().await }
    })?;
    let add = service.clone();
    capability.method(ADD_URI, move |request: AddUriRequest| {
        let service = add.clone();
        async move {
            let uris = request
                .uris
                .into_iter()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
                .map(|uri| classify_download_uri(&uri).map(|kind| (uri, kind)))
                .collect::<Result<Vec<_>, _>>()?;
            if uris.is_empty() {
                return Err("at least one URI is required".to_owned());
            }
            if uris.len() > 1
                && request
                    .out
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty())
            {
                return Err("an output filename can only be used with one URI".to_owned());
            }
            let mut ids = Vec::new();
            for (uri, kind) in uris {
                let dir = request
                    .dir
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .map(PathBuf::from);
                let id = match kind {
                    DownloadUriKind::Magnet => {
                        service
                            .backend()?
                            .add_magnet(uri, dir, request.priority)
                            .await?
                    }
                    DownloadUriKind::Http => {
                        let mut download = HttpDownloadRequest::new(uri);
                        download.save_dir = dir;
                        download.filename = request.out.clone();
                        download.max_connections =
                            request.split.map(|value| value.clamp(1, 64) as usize);
                        download.headers = request
                            .headers
                            .iter()
                            .filter_map(|header| header.split_once(':'))
                            .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
                            .collect();
                        download.priority = request.priority;
                        service.backend()?.add_http(download).await?
                    }
                };
                ids.push(id);
            }
            Ok(ids)
        }
    })?;
    let add = service.clone();
    capability.method(ADD_TORRENT, move |request: AddTorrentRequest| {
        let service = add.clone();
        async move {
            let (data, _) = read_torrent(Path::new(&request.path))?;
            let selected = request.selected_files.map(|values| {
                values
                    .into_iter()
                    .filter_map(|value| value.checked_sub(1))
                    .filter_map(|value| usize::try_from(value).ok())
                    .collect()
            });
            service
                .backend()?
                .add_torrent(
                    data,
                    request.dir.map(PathBuf::from),
                    selected,
                    request.priority,
                )
                .await
        }
    })?;
    let action = service.clone();
    capability.method(TASK_ACTION, move |request: TaskActionRequest| {
        let service = action.clone();
        async move {
            service
                .task_action(&request.id, request.action, request.remove_files)
                .await
        }
    })?;
    let priority = service.clone();
    capability.method(SET_TASK_PRIORITY, move |request: SetTaskPriorityRequest| {
        let service = priority.clone();
        async move {
            service
                .backend()?
                .set_priority(request.id, request.priority)
                .await
        }
    })?;
    let action = service.clone();
    capability.method(BATCH_TASK_ACTION, move |request: BatchTaskActionRequest| {
        let service = action.clone();
        async move {
            let mut done = Vec::new();
            for id in request.ids {
                service
                    .task_action(&id, request.action, request.remove_files)
                    .await?;
                done.push(id);
            }
            Ok::<_, String>(done)
        }
    })?;
    let global = service.clone();
    capability.method(
        GLOBAL_TASK_ACTION,
        move |request: GlobalTaskActionRequest| {
            let service = global.clone();
            async move {
                match request.action.as_str() {
                    "pauseAll" => service.backend()?.pause_all().await,
                    "resumeAll" => service.backend()?.resume_all().await,
                    _ => Err("unknown global task action".to_owned()),
                }
            }
        },
    )?;
    capability.method(
        OPEN_TASK_FOLDER,
        move |request: OpenTaskFolderRequest| async move {
            let path = PathBuf::from(request.path);
            let target = if path.is_dir() {
                path
            } else {
                path.parent()
                    .map(Path::to_path_buf)
                    .ok_or_else(|| "task path has no parent".to_owned())?
            };
            open::that(target).map_err(|error| error.to_string())
        },
    )?;
    capability.method(
        OPEN_PATH,
        move |request: OpenTaskFolderRequest| async move {
            open::that(PathBuf::from(request.path)).map_err(|error| error.to_string())
        },
    )?;
    let folder = service.clone();
    capability.method(OPEN_CONFIG_FOLDER, move |(): ()| {
        let service = folder.clone();
        async move {
            open::that(service.runtime()?.config_store.directory()?)
                .map_err(|error| error.to_string())
        }
    })
}

pub fn mount_native(capability: NativeCapability<'_>) -> rquickjs::Result<()> {
    capability.method(
        INSPECT_TORRENT,
        move |request: InspectTorrentRequest| async move {
            read_torrent(Path::new(&request.path)).map(|(_, preview)| preview)
        },
    )
}

#[cfg(feature = "bindings")]
pub fn native_bindings() -> FunctionModule {
    FunctionModule::new("NativeDownloadsApi")
        .request_dto::<InspectTorrentRequest>()
        .response_dto::<TorrentPreview>()
        .method(NativeMethod::asynchronous(
            "inspectTorrent",
            &[("request", "InspectTorrentRequest")],
            "TorrentPreview",
        ))
}

pub fn stream_snapshots(context: HostMessageContext, service: DownloadService) {
    let snapshot_context = context.clone();
    context.spawn(async move {
        if service
            .config()
            .is_ok_and(|config| config.resume_all_when_app_launched)
            && let Ok(backend) = service.backend()
        {
            let _ = backend.resume_all().await;
        }
        let mut publisher = RevisionedHostPublisher::new(SNAPSHOT, SNAPSHOT_PATCH);
        let mut events = service.backend().ok().map(|backend| backend.subscribe());
        let mut cadence = tokio::time::interval(Duration::from_secs(1));
        cadence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // The first snapshot below is immediate, so consume interval's
        // immediate first tick before waiting for the next sampling deadline.
        cadence.tick().await;
        loop {
            let next = service.snapshot().await;
            let service_available = service.runtime().is_ok();
            if service_available && events.is_none() {
                events = service.backend().ok().map(|backend| backend.subscribe());
            }
            // Capability reads share the revision allocator. The publisher repairs an
            // interleaved read revision or dropped queue entry with a full value.
            match publisher.publish(snapshot_context.messages(), next, SnapshotPatch::between) {
                Ok(_) => {}
                Err(wabou::HostMessageError::Full) => {}
                Err(wabou::HostMessageError::Disconnected) => break,
                Err(error) => tracing::warn!(?error, "could not enqueue download snapshot"),
            }
            if !service_available {
                tokio::select! {
                    () = snapshot_context.cancelled() => break,
                    () = tokio::time::sleep(Duration::from_millis(250)) => {}
                }
                continue;
            }
            let refresh_immediately = loop {
                tokio::select! {
                    () = snapshot_context.cancelled() => return,
                    _ = cadence.tick() => break false,
                    event = async {
                        match events.as_mut() {
                            Some(events) => Some(events.recv().await),
                            None => None,
                        }
                    }, if events.is_some() => match event {
                        Some(Ok(DownloadEvent::Progress)) => {}
                        Some(Ok(DownloadEvent::Changed)) => break true,
                        Some(Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped))) => {
                            tracing::debug!(skipped, "Motrix snapshot listener lagged; refreshing");
                            break true;
                        }
                        Some(Err(tokio::sync::broadcast::error::RecvError::Closed)) | None => {
                            events = None;
                        }
                    }
                }
            };
            if refresh_immediately {
                // Coalesce the burst commonly produced by Added + Started +
                // StateChanged into one frame-sized update.
                tokio::select! {
                    () = snapshot_context.cancelled() => break,
                    () = tokio::time::sleep(Duration::from_millis(16)) => {}
                }
                if let Some(events) = events.as_mut() {
                    while events.try_recv().is_ok() {}
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{DownloadUriKind, classify_download_uri, engine_restart_required};
    use crate::config::{AppConfig, ThemeMode};

    #[test]
    fn restart_requirement_only_tracks_engine_startup_configuration() {
        let original = AppConfig::default();
        let mut appearance = original.clone();
        appearance.theme = ThemeMode::Dark;
        appearance.notify_on_complete = !appearance.notify_on_complete;
        assert!(!engine_restart_required(&original, &appearance));

        let mut runtime_limits = original.clone();
        runtime_limits.max_concurrent_downloads += 1;
        runtime_limits.max_overall_download_limit = "10M".to_owned();
        runtime_limits.max_overall_upload_limit = "1M".to_owned();
        assert!(!engine_restart_required(&original, &runtime_limits));

        let mut engine = original.clone();
        engine.max_connection_per_server += 1;
        assert!(engine_restart_required(&original, &engine));

        let mut later_save = engine.clone();
        later_save.theme = ThemeMode::Dark;
        assert!(engine_restart_required(&original, &later_save));
        assert!(!engine_restart_required(&engine, &later_save));
    }

    #[test]
    fn download_uri_validation_rejects_local_and_unknown_schemes_before_dispatch() {
        assert_eq!(
            classify_download_uri("https://example.com/file.iso").unwrap(),
            DownloadUriKind::Http
        );
        assert_eq!(
            classify_download_uri("magnet:?xt=urn:btih:0123456789abcdef").unwrap(),
            DownloadUriKind::Magnet
        );
        assert!(classify_download_uri("file:///tmp/private").is_err());
        assert!(classify_download_uri("not a URI").is_err());
    }
}
