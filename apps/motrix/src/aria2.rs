use std::{
    collections::HashSet,
    env,
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex as StdMutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::config::{AppConfig, ConfigStore, EngineMode};
use aria2_ws::{
    Client, TaskOptions,
    response::{BitTorrentFileMode, Status},
};
#[cfg(windows)]
use process_wrap::std::JobObject;
#[cfg(unix)]
use process_wrap::std::ProcessGroup;
use process_wrap::std::{ChildWrapper, CommandWrap};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use wabou::{
    HostMessageContext, HostService, JsonCapability, JsonCapabilityContract, JsonMethod, rquickjs,
};

pub const CAPABILITY: JsonCapabilityContract = JsonCapabilityContract::new("aria2", 1);
const SNAPSHOT: &str = "aria2.snapshot";

const GET_SNAPSHOT: JsonMethod<(), Snapshot> = JsonMethod::no_request("getSnapshot");
const ADD_URI: JsonMethod<AddUriRequest, Vec<String>> = JsonMethod::new("addUri");
const ADD_TORRENT: JsonMethod<AddTorrentRequest, String> = JsonMethod::new("addTorrent");
const TASK_ACTION: JsonMethod<TaskActionRequest, ()> = JsonMethod::new("taskAction");
const BATCH_TASK_ACTION: JsonMethod<BatchTaskActionRequest, Vec<String>> =
    JsonMethod::new("batchTaskAction");
const ENGINE_ACTION: JsonMethod<EngineActionRequest, ()> = JsonMethod::new("engineAction");
const GET_CONFIG: JsonMethod<(), AppConfig> = JsonMethod::no_request("getConfig");
const SET_CONFIG: JsonMethod<AppConfig, AppConfig> = JsonMethod::new("setConfig");
const OPEN_TASK_FOLDER: JsonMethod<OpenTaskFolderRequest, ()> = JsonMethod::new("openTaskFolder");
const OPEN_CONFIG_FOLDER: JsonMethod<(), ()> = JsonMethod::no_request("openConfigFolder");
const GLOBAL_TASK_ACTION: JsonMethod<GlobalTaskActionRequest, ()> =
    JsonMethod::new("globalTaskAction");
const GET_TASK_DETAILS: JsonMethod<GetTaskDetailsRequest, TaskDetails> =
    JsonMethod::new("getTaskDetails");
const SET_SELECTED_FILES: JsonMethod<SetSelectedFilesRequest, TaskDetails> =
    JsonMethod::new("setSelectedFiles");

#[derive(Clone)]
struct Connection {
    endpoint: Arc<str>,
    secret: Option<Arc<str>>,
    managed: bool,
}

#[derive(Clone)]
pub struct Aria2Service {
    connection: Arc<RwLock<Connection>>,
    client: Arc<Mutex<Option<Client>>>,
    managed: ManagedAria2,
    config: Arc<RwLock<AppConfig>>,
    config_store: ConfigStore,
}

#[derive(Clone)]
pub struct ManagedAria2 {
    port: u16,
    secret: Arc<str>,
    session_path: Arc<PathBuf>,
    child: Arc<StdMutex<Option<Box<dyn ChildWrapper>>>>,
    enabled: Arc<AtomicBool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddUriRequest {
    uris: Vec<String>,
    dir: Option<String>,
    out: Option<String>,
    split: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTorrentRequest {
    path: String,
    dir: Option<String>,
    split: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenTaskFolderRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTaskDetailsRequest {
    gid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSelectedFilesRequest {
    gid: String,
    indices: Vec<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalTaskActionRequest {
    action: GlobalTaskAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum GlobalTaskAction {
    PauseAll,
    ResumeAll,
    ClearCompleted,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskActionRequest {
    gid: String,
    action: TaskAction,
    #[serde(default)]
    remove_files: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTaskActionRequest {
    gids: Vec<String>,
    action: TaskAction,
    #[serde(default)]
    remove_files: bool,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TaskAction {
    Pause,
    Resume,
    Remove,
    Retry,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineActionRequest {
    action: EngineAction,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum EngineAction {
    Start,
    Stop,
    Restart,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    connected: bool,
    endpoint: String,
    version: Option<String>,
    error: Option<String>,
    download_speed: u64,
    upload_speed: u64,
    tasks: Vec<TaskSnapshot>,
    managed: bool,
    engine_running: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskSnapshot {
    gid: String,
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
    file_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskDetails {
    files: Vec<TaskFileDetails>,
    trackers: Vec<String>,
    peers: Vec<TaskPeerDetails>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskFileDetails {
    index: u64,
    path: String,
    length: u64,
    completed_length: u64,
    selected: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TaskPeerDetails {
    ip: String,
    port: u16,
    download_speed: u64,
    upload_speed: u64,
    seeder: bool,
}

impl Aria2Service {
    pub fn from_config(
        mut config: AppConfig,
        config_store: ConfigStore,
    ) -> Result<(Self, Option<ManagedAria2>), String> {
        if let Ok(endpoint) = env::var("WABOU_ARIA2_URL") {
            config.engine_mode = EngineMode::External;
            config.external_endpoint = endpoint;
            config.external_secret = env::var("WABOU_ARIA2_SECRET").unwrap_or_default();
        }
        let port = TcpListener::bind(("127.0.0.1", 0))
            .map_err(|error| error.to_string())?
            .local_addr()
            .map_err(|error| error.to_string())?
            .port();
        let secret: Arc<str> = uuid::Uuid::new_v4().simple().to_string().into();
        let session_path = config_store.directory()?.join("aria2.session");
        let managed = ManagedAria2 {
            port,
            secret: secret.clone(),
            session_path: Arc::new(session_path),
            child: Arc::new(StdMutex::new(None)),
            enabled: Arc::new(AtomicBool::new(config.engine_mode == EngineMode::Managed)),
        };
        let connection = if config.engine_mode == EngineMode::Managed {
            Connection {
                endpoint: format!("ws://127.0.0.1:{port}/jsonrpc").into(),
                secret: Some(secret),
                managed: true,
            }
        } else {
            Connection {
                endpoint: config.external_endpoint.clone().into(),
                secret: (!config.external_secret.is_empty())
                    .then(|| config.external_secret.clone().into()),
                managed: false,
            }
        };
        Ok((
            Self {
                connection: Arc::new(RwLock::new(connection)),
                client: Arc::new(Mutex::new(None)),
                managed: managed.clone(),
                config: Arc::new(RwLock::new(config)),
                config_store,
            },
            Some(managed),
        ))
    }

    #[cfg(test)]
    fn for_endpoint(endpoint: impl Into<Arc<str>>) -> Self {
        let config = AppConfig {
            engine_mode: EngineMode::External,
            external_endpoint: endpoint.into().to_string(),
            ..AppConfig::default()
        };
        let managed = ManagedAria2 {
            port: 0,
            secret: "test".into(),
            session_path: Arc::new(PathBuf::from("aria2.session")),
            child: Arc::new(StdMutex::new(None)),
            enabled: Arc::new(AtomicBool::new(false)),
        };
        Self {
            connection: Arc::new(RwLock::new(Connection {
                endpoint: config.external_endpoint.clone().into(),
                secret: None,
                managed: false,
            })),
            client: Arc::new(Mutex::new(None)),
            managed,
            config: Arc::new(RwLock::new(config)),
            config_store: ConfigStore::new(std::path::Path::new(".")),
        }
    }

    async fn client(&self) -> Result<Client, String> {
        let mut slot = self.client.lock().await;
        if let Some(client) = slot.as_ref() {
            return Ok(client.clone());
        }
        let connection = self
            .connection
            .read()
            .map_err(|_| "aria2 connection lock poisoned")?
            .clone();
        let client = Client::connect(&connection.endpoint, connection.secret.as_deref())
            .await
            .map_err(|error| format!("cannot connect to {}: {error}", connection.endpoint))?;
        *slot = Some(client.clone());
        Ok(client)
    }

    async fn snapshot(&self) -> Snapshot {
        match self.read_snapshot().await {
            Ok(snapshot) => snapshot,
            Err(error) => {
                *self.client.lock().await = None;
                let connection =
                    self.connection
                        .read()
                        .map(|value| value.clone())
                        .unwrap_or(Connection {
                            endpoint: "invalid".into(),
                            secret: None,
                            managed: false,
                        });
                Snapshot {
                    connected: false,
                    endpoint: connection.endpoint.to_string(),
                    version: None,
                    error: Some(error),
                    download_speed: 0,
                    upload_speed: 0,
                    tasks: Vec::new(),
                    managed: connection.managed,
                    engine_running: !connection.managed || self.managed.is_running(),
                }
            }
        }
    }

    async fn read_snapshot(&self) -> Result<Snapshot, String> {
        let client = self.client().await?;
        let (version, stat, active, waiting, stopped) = tokio::try_join!(
            client.get_version(),
            client.get_global_stat(),
            client.tell_active(),
            client.tell_waiting(0, 100),
            client.tell_stopped(0, 100),
        )
        .map_err(|error| error.to_string())?;
        let connection = self
            .connection
            .read()
            .map_err(|_| "aria2 connection lock poisoned")?
            .clone();
        Ok(Snapshot {
            connected: true,
            endpoint: connection.endpoint.to_string(),
            version: Some(version.version),
            error: None,
            download_speed: stat.download_speed,
            upload_speed: stat.upload_speed,
            tasks: active
                .into_iter()
                .chain(waiting)
                .chain(stopped)
                .map(task_snapshot)
                .collect(),
            managed: connection.managed,
            engine_running: !connection.managed || self.managed.is_running(),
        })
    }

    async fn engine_action(&self, action: EngineAction) -> Result<(), String> {
        let connection = self
            .connection
            .read()
            .map_err(|_| "aria2 connection lock poisoned")?
            .clone();
        if !connection.managed {
            return Err("external aria2 engines are controlled outside Motrix".to_owned());
        }
        let managed = &self.managed;
        match action {
            EngineAction::Start => managed.start(),
            EngineAction::Stop => {
                managed.shutdown();
                Ok(())
            }
            EngineAction::Restart => {
                managed.shutdown();
                managed.start()
            }
        }?;
        *self.client.lock().await = None;
        Ok(())
    }

    fn config(&self) -> Result<AppConfig, String> {
        self.config
            .read()
            .map(|value| value.clone())
            .map_err(|_| "configuration lock poisoned".to_owned())
    }

    async fn set_config(&self, config: AppConfig) -> Result<AppConfig, String> {
        if config.engine_mode == EngineMode::External
            && !(config.external_endpoint.starts_with("ws://")
                || config.external_endpoint.starts_with("wss://"))
        {
            return Err("external endpoint must start with ws:// or wss://".to_owned());
        }
        let previous = self.config()?;
        self.config_store.save(&config)?;
        let reconnect = previous.engine_mode != config.engine_mode
            || (config.engine_mode == EngineMode::External
                && (previous.external_endpoint != config.external_endpoint
                    || previous.external_secret != config.external_secret));
        if reconnect {
            self.managed.shutdown();
            self.managed
                .enabled
                .store(config.engine_mode == EngineMode::Managed, Ordering::Release);
            let connection = if config.engine_mode == EngineMode::Managed {
                Connection {
                    endpoint: format!("ws://127.0.0.1:{}/jsonrpc", self.managed.port).into(),
                    secret: Some(self.managed.secret.clone()),
                    managed: true,
                }
            } else {
                Connection {
                    endpoint: config.external_endpoint.clone().into(),
                    secret: (!config.external_secret.is_empty())
                        .then(|| config.external_secret.clone().into()),
                    managed: false,
                }
            };
            *self
                .connection
                .write()
                .map_err(|_| "aria2 connection lock poisoned")? = connection;
            *self.client.lock().await = None;
            if config.engine_mode == EngineMode::Managed {
                self.managed.start()?;
            }
        }
        *self
            .config
            .write()
            .map_err(|_| "configuration lock poisoned")? = config.clone();
        if let Ok(client) = self.client().await {
            let mut options = TaskOptions::default();
            options.extra_options.insert(
                "max-concurrent-downloads".to_owned(),
                serde_json::Value::String(config.max_concurrent_downloads.to_string()),
            );
            options.extra_options.insert(
                "bt-tracker".to_owned(),
                serde_json::Value::String(config.bt_trackers.join(",")),
            );
            options.extra_options.insert(
                "max-overall-download-limit".to_owned(),
                serde_json::Value::String(config.max_overall_download_limit.clone()),
            );
            options.extra_options.insert(
                "max-overall-upload-limit".to_owned(),
                serde_json::Value::String(config.max_overall_upload_limit.clone()),
            );
            options.extra_options.insert(
                "user-agent".to_owned(),
                serde_json::Value::String(config.user_agent.clone()),
            );
            change_global_options(&client, options).await?;
        }
        Ok(config)
    }
}

async fn change_global_options(client: &Client, options: TaskOptions) -> Result<(), String> {
    let value = serde_json::to_value(options).map_err(|error| error.to_string())?;
    let response: String = client
        .call_and_wait("changeGlobalOption", vec![value])
        .await
        .map_err(|error| error.to_string())?;
    if response == "OK" {
        Ok(())
    } else {
        Err(format!(
            "aria2 returned an unexpected changeGlobalOption result: {response}"
        ))
    }
}

impl ManagedAria2 {
    fn is_running(&self) -> bool {
        self.child.lock().is_ok_and(|child| child.is_some())
    }

    fn prepare_session_file(&self) -> Result<(), String> {
        let parent = self
            .session_path
            .parent()
            .ok_or_else(|| "aria2 session path has no parent directory".to_owned())?;
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        if !self.session_path.exists() {
            std::fs::write(self.session_path.as_ref(), b"").map_err(|error| {
                format!("cannot initialize {}: {error}", self.session_path.display())
            })?;
        }
        Ok(())
    }

    fn request_graceful_shutdown(&self) {
        let endpoint = format!("ws://127.0.0.1:{}/jsonrpc", self.port);
        let secret = self.secret.clone();
        let request = thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?;
            runtime.block_on(async move {
                let client = Client::connect(&endpoint, Some(secret.as_ref()))
                    .await
                    .map_err(|error| error.to_string())?;
                client
                    .save_session()
                    .await
                    .map_err(|error| error.to_string())?;
                client.shutdown().await.map_err(|error| error.to_string())
            })
        });
        if let Err(error) = request
            .join()
            .unwrap_or_else(|_| Err("aria2 shutdown task panicked".into()))
        {
            tracing::warn!(%error, "could not gracefully stop managed aria2; forcing process exit");
        }
    }
}

impl HostService for ManagedAria2 {
    fn name(&self) -> &'static str {
        "aria2c"
    }

    fn start(&self) -> Result<(), String> {
        if !self.enabled.load(Ordering::Acquire)
            || env::var("WABOU_ARIA2_DISABLE_MANAGED").is_ok_and(|value| value != "0")
        {
            return Ok(());
        }
        if self.is_running() {
            return Ok(());
        }
        self.prepare_session_file()?;
        let mut command =
            Command::new(env::var_os("WABOU_ARIA2_BIN").unwrap_or_else(|| "aria2c".into()));
        command.args([
            "--enable-rpc=true",
            "--rpc-listen-all=false",
            &format!("--rpc-listen-port={}", self.port),
            &format!("--rpc-secret={}", self.secret),
            &format!("--input-file={}", self.session_path.display()),
            &format!("--save-session={}", self.session_path.display()),
            "--save-session-interval=10",
            "--continue=true",
            "--console-log-level=warn",
        ]);
        let mut command = CommandWrap::from(command);
        #[cfg(unix)]
        command.wrap(ProcessGroup::leader());
        #[cfg(windows)]
        command.wrap(JobObject);
        let child = command.spawn().map_err(|error| error.to_string())?;
        *self
            .child
            .lock()
            .map_err(|_| "aria2 process lock poisoned")? = Some(child);
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if TcpStream::connect(("127.0.0.1", self.port)).is_ok() {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(20));
        }
        Err(format!("RPC port {} did not become ready", self.port))
    }

    fn shutdown(&self) {
        if let Ok(mut slot) = self.child.lock()
            && let Some(mut child) = slot.take()
        {
            self.request_graceful_shutdown();
            // aria2 acknowledges `shutdown` before its process has fully
            // exited. Give it a short grace period after the already-awaited
            // `saveSession` RPC, then retain the process-group fallback.
            let deadline = Instant::now() + Duration::from_millis(500);
            while Instant::now() < deadline {
                if child.try_wait().is_ok_and(|status| status.is_some()) {
                    return;
                }
                thread::sleep(Duration::from_millis(20));
            }
            let _ = child.kill();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires aria2c RPC on port 16800"]
    async fn reads_a_real_aria2_snapshot() {
        let snapshot = Aria2Service::for_endpoint("ws://127.0.0.1:16800/jsonrpc")
            .read_snapshot()
            .await
            .expect("connect to aria2c");
        assert!(snapshot.connected);
        assert_eq!(snapshot.version.as_deref(), Some("1.37.0"));
    }

    #[tokio::test]
    async fn managed_aria2_uses_its_generated_secret() {
        let root = std::env::temp_dir().join(format!("motrix-aria2-test-{}", uuid::Uuid::new_v4()));
        let (service, managed) =
            Aria2Service::from_config(AppConfig::default(), ConfigStore::new(&root))
                .expect("configuration");
        let managed = managed.expect("managed mode without WABOU_ARIA2_URL");
        managed.start().expect("start aria2c");
        let snapshot = service.read_snapshot().await.expect("authenticated RPC");
        assert!(snapshot.connected);
        managed.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn managed_aria2_restores_unfinished_tasks_from_its_session() {
        let root = std::env::temp_dir().join(format!(
            "motrix-aria2-session-test-{}",
            uuid::Uuid::new_v4()
        ));
        let (service, managed) =
            Aria2Service::from_config(AppConfig::default(), ConfigStore::new(&root))
                .expect("configuration");
        let managed = managed.expect("managed service");
        managed.start().expect("start aria2c");

        let client = service.client().await.expect("connect to aria2c");
        let mut options = TaskOptions::default();
        options.extra_options.insert(
            "pause".to_owned(),
            serde_json::Value::String("true".to_owned()),
        );
        let gid = client
            .add_uri(
                vec!["magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567".to_owned()],
                Some(options),
                None,
                None,
            )
            .await
            .expect("add paused task");

        managed.shutdown();
        assert!(
            std::fs::metadata(root.join("aria2.session")).is_ok_and(|metadata| metadata.len() > 0),
            "graceful shutdown should persist the aria2 session"
        );

        *service.client.lock().await = None;
        managed.start().expect("restart aria2c");
        let restored = service.read_snapshot().await.expect("restored snapshot");
        assert!(restored.tasks.iter().any(|task| task.gid == gid));

        managed.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn switching_engine_mode_stops_and_restarts_the_owned_process() {
        let root =
            std::env::temp_dir().join(format!("motrix-engine-mode-{}", uuid::Uuid::new_v4()));
        let store = ConfigStore::new(&root);
        let (service, managed) =
            Aria2Service::from_config(AppConfig::default(), store.clone()).expect("configuration");
        let managed = managed.expect("managed service");
        managed.start().expect("start managed aria2");
        assert!(managed.is_running());

        let external = AppConfig {
            engine_mode: EngineMode::External,
            external_endpoint: "ws://127.0.0.1:9/jsonrpc".to_owned(),
            ..AppConfig::default()
        };
        service
            .set_config(external.clone())
            .await
            .expect("switch external");
        assert!(!managed.is_running());
        assert_eq!(
            store.load().expect("persisted config").engine_mode,
            EngineMode::External
        );

        service
            .set_config(AppConfig::default())
            .await
            .expect("switch managed");
        assert!(managed.is_running());
        managed.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn single_file_removal_includes_payload_and_aria2_control_file() {
        assert_eq!(
            planned_removal_paths(
                Path::new("/downloads"),
                &[PathBuf::from("/downloads/archive.zip")],
                false,
            ),
            vec![
                PathBuf::from("/downloads/archive.zip"),
                PathBuf::from("/downloads/archive.zip.aria2"),
            ],
        );
    }

    #[test]
    fn multi_file_torrent_removes_only_its_common_root() {
        assert_eq!(
            planned_removal_paths(
                Path::new("/downloads"),
                &[
                    PathBuf::from("/downloads/release/video.mkv"),
                    PathBuf::from("/downloads/release/subtitles/en.srt"),
                ],
                true,
            ),
            vec![
                PathBuf::from("/downloads/release"),
                PathBuf::from("/downloads/release.aria2"),
            ],
        );
    }

    #[test]
    fn multi_file_task_never_selects_the_whole_download_directory() {
        assert_eq!(
            planned_removal_paths(
                Path::new("/downloads"),
                &[
                    PathBuf::from("/downloads/one.bin"),
                    PathBuf::from("/downloads/two.bin"),
                ],
                true,
            ),
            vec![
                PathBuf::from("/downloads/one.bin"),
                PathBuf::from("/downloads/one.bin.aria2"),
                PathBuf::from("/downloads/two.bin"),
                PathBuf::from("/downloads/two.bin.aria2"),
            ],
        );
    }

    #[test]
    fn task_action_accepts_the_camel_case_trash_flag() {
        let request: TaskActionRequest =
            serde_json::from_str(r#"{"gid":"42","action":"remove","removeFiles":true}"#)
                .expect("valid task action");
        assert_eq!(request.gid, "42");
        assert!(matches!(request.action, TaskAction::Remove));
        assert!(request.remove_files);

        let request: TaskActionRequest = serde_json::from_str(r#"{"gid":"42","action":"remove"}"#)
            .expect("backward-compatible task action");
        assert!(!request.remove_files);
    }

    #[test]
    fn file_selection_is_validated_sorted_and_encoded_for_aria2() {
        let available = HashSet::from([1, 2, 3, 4, 5, 8]);
        let selection =
            normalized_file_selection(vec![8, 4, 3, 1, 5, 3], &available).expect("valid selection");
        assert_eq!(selection, vec![1, 3, 4, 5, 8]);
        assert_eq!(format_file_selection(&selection), "1,3-5,8");
        assert!(normalized_file_selection(Vec::new(), &available).is_err());
        assert!(normalized_file_selection(vec![1, 9], &available).is_err());
    }
}

fn task_snapshot(status: Status) -> TaskSnapshot {
    let bittorrent = status.bittorrent.is_some();
    let file_count = status.files.len();
    let task_status = if status.status == aria2_ws::response::TaskStatus::Active
        && bittorrent
        && status.seeder == Some(true)
    {
        "seeding".to_owned()
    } else {
        format!("{:?}", status.status).to_lowercase()
    };
    let name = status
        .files
        .first()
        .and_then(|file| std::path::Path::new(&file.path).file_name())
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(&status.gid)
        .to_owned();
    TaskSnapshot {
        gid: status.gid,
        name,
        status: task_status,
        total_length: status.total_length,
        completed_length: status.completed_length,
        download_speed: status.download_speed,
        upload_speed: status.upload_speed,
        uploaded_length: status.upload_length,
        dir: status.dir,
        file_path: status
            .files
            .first()
            .map(|file| file.path.clone())
            .filter(|path| !path.is_empty()),
        uri: status
            .files
            .first()
            .and_then(|file| file.uris.first())
            .map(|uri| uri.uri.clone()),
        connections: status.connections,
        seeders: status.num_seeders,
        error_message: status.error_message,
        bittorrent,
        file_count,
    }
}

pub fn mount(capability: JsonCapability<'_>, service: Aria2Service) -> rquickjs::Result<()> {
    let snapshot_service = service.clone();
    capability.method(GET_SNAPSHOT, move |(): ()| {
        let service = snapshot_service.clone();
        async move { Ok::<_, String>(service.snapshot().await) }
    })?;
    let details_service = service.clone();
    capability.method(GET_TASK_DETAILS, move |request: GetTaskDetailsRequest| {
        let service = details_service.clone();
        async move {
            let client = service.client().await?;
            task_details(&client, &request.gid).await
        }
    })?;
    let selected_files_service = service.clone();
    capability.method(
        SET_SELECTED_FILES,
        move |request: SetSelectedFilesRequest| {
            let service = selected_files_service.clone();
            async move {
                let client = service.client().await?;
                set_selected_files(&client, &request.gid, request.indices).await?;
                task_details(&client, &request.gid).await
            }
        },
    )?;
    let get_config_service = service.clone();
    capability.method(GET_CONFIG, move |(): ()| {
        let service = get_config_service.clone();
        async move { service.config() }
    })?;
    let set_config_service = service.clone();
    capability.method(SET_CONFIG, move |config: AppConfig| {
        let service = set_config_service.clone();
        async move { service.set_config(config).await }
    })?;
    capability.method(
        OPEN_TASK_FOLDER,
        move |request: OpenTaskFolderRequest| async move {
            let path = std::path::PathBuf::from(request.path);
            let target = if path.is_dir() {
                path
            } else {
                path.parent()
                    .map(std::path::Path::to_path_buf)
                    .ok_or_else(|| "task path has no parent directory".to_owned())?
            };
            open::that(target).map_err(|error| error.to_string())
        },
    )?;
    let config_folder_service = service.clone();
    capability.method(OPEN_CONFIG_FOLDER, move |(): ()| {
        let service = config_folder_service.clone();
        async move {
            let directory = service.config_store.directory()?.to_path_buf();
            open::that(directory).map_err(|error| error.to_string())
        }
    })?;
    let global_service = service.clone();
    capability.method(
        GLOBAL_TASK_ACTION,
        move |request: GlobalTaskActionRequest| {
            let service = global_service.clone();
            async move {
                let client = service.client().await?;
                match request.action {
                    GlobalTaskAction::PauseAll => client.force_pause_all().await,
                    GlobalTaskAction::ResumeAll => client.unpause_all().await,
                    GlobalTaskAction::ClearCompleted => client.purge_download_result().await,
                }
                .map_err(|error| error.to_string())
            }
        },
    )?;
    let add_service = service.clone();
    capability.method(ADD_URI, move |request: AddUriRequest| {
        let service = add_service.clone();
        async move {
            if request.uris.is_empty() {
                return Err("at least one URI is required".to_owned());
            }
            let options = TaskOptions {
                dir: request.dir.filter(|value| !value.trim().is_empty()),
                out: request.out.filter(|value| !value.trim().is_empty()),
                split: request.split.filter(|value| *value > 0),
                ..TaskOptions::default()
            };
            let client = service.client().await?;
            let mut gids = Vec::with_capacity(request.uris.len());
            for uri in request.uris {
                let gid = client
                    .add_uri(vec![uri], Some(options.clone()), None, None)
                    .await
                    .map_err(|error| error.to_string())?;
                gids.push(gid);
            }
            Ok(gids)
        }
    })?;
    let torrent_service = service.clone();
    capability.method(ADD_TORRENT, move |request: AddTorrentRequest| {
        let service = torrent_service.clone();
        async move {
            let metadata = std::fs::metadata(&request.path)
                .map_err(|error| format!("cannot inspect torrent file: {error}"))?;
            if metadata.len() > 64 * 1024 * 1024 {
                return Err("torrent file exceeds the 64 MB safety limit".to_owned());
            }
            let torrent = std::fs::read(&request.path)
                .map_err(|error| format!("cannot read torrent file: {error}"))?;
            let options = TaskOptions {
                dir: request.dir.filter(|value| !value.trim().is_empty()),
                split: request.split.filter(|value| *value > 0),
                ..TaskOptions::default()
            };
            service
                .client()
                .await?
                .add_torrent(torrent, None, Some(options), None, None)
                .await
                .map_err(|error| error.to_string())
        }
    })?;
    let action_service = service.clone();
    capability.method(TASK_ACTION, move |request: TaskActionRequest| {
        let service = action_service.clone();
        async move {
            let client = service.client().await?;
            match request.action {
                TaskAction::Pause => client
                    .pause(&request.gid)
                    .await
                    .map_err(|error| error.to_string()),
                TaskAction::Resume => client
                    .unpause(&request.gid)
                    .await
                    .map_err(|error| error.to_string()),
                TaskAction::Remove => {
                    remove_task(&client, &request.gid, request.remove_files).await
                }
                TaskAction::Retry => retry_task(&client, &request.gid).await,
            }
        }
    })?;
    let batch_service = service.clone();
    capability.method(BATCH_TASK_ACTION, move |request: BatchTaskActionRequest| {
        let service = batch_service.clone();
        async move {
            let client = service.client().await?;
            let mut completed = Vec::with_capacity(request.gids.len());
            for gid in request.gids {
                let result = match request.action {
                    TaskAction::Pause => {
                        client.pause(&gid).await.map_err(|error| error.to_string())
                    }
                    TaskAction::Resume => client
                        .unpause(&gid)
                        .await
                        .map_err(|error| error.to_string()),
                    TaskAction::Remove => remove_task(&client, &gid, request.remove_files).await,
                    TaskAction::Retry => retry_task(&client, &gid).await,
                };
                result.map_err(|error| format!("task {gid}: {error}"))?;
                completed.push(gid);
            }
            Ok::<_, String>(completed)
        }
    })?;
    capability.method(ENGINE_ACTION, move |request: EngineActionRequest| {
        let service = service.clone();
        async move { service.engine_action(request.action).await }
    })
}

async fn task_details(client: &Client, gid: &str) -> Result<TaskDetails, String> {
    let status = client
        .tell_status(gid)
        .await
        .map_err(|error| error.to_string())?;
    let peers = if status.bittorrent.is_some() {
        client.get_peers(gid).await.unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut trackers = status
        .bittorrent
        .as_ref()
        .map(|torrent| {
            torrent
                .announce_list
                .iter()
                .flatten()
                .filter(|tracker| !tracker.is_empty())
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut seen = HashSet::new();
    trackers.retain(|tracker| seen.insert(tracker.clone()));
    Ok(TaskDetails {
        files: status
            .files
            .into_iter()
            .map(|file| TaskFileDetails {
                index: file.index,
                path: file.path,
                length: file.length,
                completed_length: file.completed_length,
                selected: file.selected,
            })
            .collect(),
        trackers,
        peers: peers
            .into_iter()
            .map(|peer| TaskPeerDetails {
                ip: peer.ip,
                port: peer.port,
                download_speed: peer.download_speed,
                upload_speed: peer.upload_speed,
                seeder: peer.seeder,
            })
            .collect(),
    })
}

async fn set_selected_files(client: &Client, gid: &str, indices: Vec<u64>) -> Result<(), String> {
    let status = client
        .tell_status(gid)
        .await
        .map_err(|error| error.to_string())?;
    if status.bittorrent.is_none() || status.files.len() <= 1 {
        return Err("file selection is only available for multi-file BitTorrent tasks".to_owned());
    }
    let available = status
        .files
        .iter()
        .map(|file| file.index)
        .collect::<HashSet<_>>();
    let selection = normalized_file_selection(indices, &available)?;
    let mut options = TaskOptions::default();
    options.extra_options.insert(
        "select-file".to_owned(),
        serde_json::Value::String(format_file_selection(&selection)),
    );
    client
        .change_option(gid, options)
        .await
        .map_err(|error| error.to_string())
}

fn normalized_file_selection(
    indices: Vec<u64>,
    available: &HashSet<u64>,
) -> Result<Vec<u64>, String> {
    let mut indices = indices;
    indices.sort_unstable();
    indices.dedup();
    if indices.is_empty() {
        return Err("at least one file must remain selected".to_owned());
    }
    if let Some(index) = indices.iter().find(|index| !available.contains(index)) {
        return Err(format!("file index {index} does not belong to this task"));
    }
    Ok(indices)
}

fn format_file_selection(indices: &[u64]) -> String {
    let mut parts = Vec::new();
    let mut start = indices[0];
    let mut previous = start;
    let flush = |parts: &mut Vec<String>, start: u64, previous: u64| {
        if previous - start >= 2 {
            parts.push(format!("{start}-{previous}"));
        } else {
            parts.extend((start..=previous).map(|index| index.to_string()));
        }
    };
    for &index in &indices[1..] {
        if index == previous + 1 {
            previous = index;
        } else {
            flush(&mut parts, start, previous);
            start = index;
            previous = index;
        }
    }
    flush(&mut parts, start, previous);
    parts.join(",")
}

async fn remove_task(client: &Client, gid: &str, remove_files: bool) -> Result<(), String> {
    // Capture authoritative paths before removing the aria2 row. The client
    // never accepts filesystem paths from JavaScript for destructive actions.
    let status = if remove_files {
        Some(
            client
                .tell_status(gid)
                .await
                .map_err(|error| format!("cannot inspect task files before removal: {error}"))?,
        )
    } else {
        None
    };

    match client.force_remove(gid).await {
        Ok(()) => {}
        Err(_) => client
            .remove_download_result(gid)
            .await
            .map_err(|error| error.to_string())?,
    }

    if let Some(status) = status {
        move_task_files_to_trash(&status).await?;
    }
    Ok(())
}

async fn move_task_files_to_trash(status: &Status) -> Result<(), String> {
    let paths = task_removal_paths(status)
        .into_iter()
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    if paths.is_empty() {
        return Ok(());
    }
    tokio::task::spawn_blocking(move || trash::delete_all(&paths))
        .await
        .map_err(|error| format!("trash worker failed: {error}"))?
        .map_err(|error| format!("could not move downloaded files to Trash: {error}"))
}

fn task_removal_paths(status: &Status) -> Vec<PathBuf> {
    let base = Path::new(&status.dir);
    let files = status
        .files
        .iter()
        .filter_map(|file| {
            let path = PathBuf::from(file.path.trim());
            (!path.as_os_str().is_empty()).then(|| {
                if path.is_absolute() {
                    path
                } else {
                    base.join(path)
                }
            })
        })
        .collect::<Vec<_>>();
    let multi_file_torrent = status
        .bittorrent
        .as_ref()
        .is_some_and(|torrent| torrent.mode == Some(BitTorrentFileMode::Multi));
    planned_removal_paths(base, &files, multi_file_torrent)
}

fn planned_removal_paths(base: &Path, files: &[PathBuf], multi_file_torrent: bool) -> Vec<PathBuf> {
    if files.is_empty() {
        return Vec::new();
    }

    let mut payloads = files.to_vec();
    if multi_file_torrent {
        let mut common = files[0].parent().map(Path::to_path_buf);
        for file in &files[1..] {
            while common
                .as_ref()
                .is_some_and(|candidate| !file.starts_with(candidate))
            {
                common = common.and_then(|candidate| candidate.parent().map(Path::to_path_buf));
            }
        }
        // Never infer that the user's whole download directory is one task.
        if let Some(root) = common.filter(|root| root != base && root.starts_with(base)) {
            payloads = vec![root];
        }
    }

    let mut paths = Vec::with_capacity(payloads.len() * 2);
    for payload in payloads {
        let mut control = payload.as_os_str().to_owned();
        control.push(".aria2");
        paths.push(payload);
        paths.push(PathBuf::from(control));
    }
    let mut seen = HashSet::new();
    paths.retain(|path| seen.insert(path.clone()));
    paths
}

async fn retry_task(client: &Client, gid: &str) -> Result<(), String> {
    let status = client
        .tell_status(gid)
        .await
        .map_err(|error| error.to_string())?;
    let uri = status
        .files
        .first()
        .and_then(|file| file.uris.first())
        .map(|uri| uri.uri.clone());
    let Some(uri) = uri else {
        // aria2 does not retain a reconstructable URI for most torrent tasks.
        // Do not silently create a different download recipe.
        return Err("task has no reconstructable source URI".to_owned());
    };
    let out = status
        .files
        .first()
        .and_then(|file| std::path::Path::new(&file.path).file_name())
        .and_then(|name| name.to_str())
        .map(str::to_owned);
    let options = TaskOptions {
        dir: (!status.dir.is_empty()).then_some(status.dir),
        out,
        ..TaskOptions::default()
    };
    client
        .remove_download_result(gid)
        .await
        .map_err(|error| error.to_string())?;
    client
        .add_uri(vec![uri], Some(options), None, None)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn stream_snapshots(context: HostMessageContext, service: Aria2Service) {
    let task_context = context.clone();
    context.spawn(async move {
        if service
            .config()
            .is_ok_and(|config| config.resume_all_when_app_launched)
            && let Ok(client) = service.client().await
        {
            let _ = client.unpause_all().await;
        }
        loop {
            let snapshot = service.snapshot().await;
            if let Ok(payload) = serde_json::to_string(&snapshot) {
                let _ = task_context.messages().emit_str(SNAPSHOT, payload);
            }
            tokio::select! {
                () = task_context.cancelled() => break,
                () = tokio::time::sleep(Duration::from_secs(1)) => {}
            }
        }
    });
}
