//! Download-engine boundary used by the Motrix reference application.
//!
//! The UI contract deliberately does not expose `gosh-dl` types. This keeps
//! the application model stable while the embedded engine is still young and
//! lets tests replace the engine without starting a native window.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, mpsc},
    thread,
};

use gosh_dl::{
    DownloadEngine, DownloadEvent as EngineEvent, DownloadId, DownloadKind, DownloadOptions,
    DownloadState, EngineConfig,
};
use serde::Serialize;
use tokio::sync::{broadcast, mpsc as tokio_mpsc, oneshot};

use crate::config::{AppConfig, parse_byte_size};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: String,
    pub name: String,
    pub state: TaskState,
    pub total_size: Option<u64>,
    pub completed_size: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub connections: u32,
    pub save_dir: PathBuf,
    pub source: Option<String>,
    pub bittorrent: bool,
    pub seeders: Option<u64>,
    pub error: Option<String>,
    pub file_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    Waiting,
    Active,
    Paused,
    Complete,
    Error,
}

/// Coarse application-facing invalidation emitted by the embedded engine.
///
/// Progress is sampled on the UI's steady cadence, while structural and state
/// changes should make the UI refresh immediately. Keeping this distinction at
/// the backend boundary avoids leaking gosh-dl's event model into the app.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DownloadEvent {
    Progress,
    Changed,
}

impl From<&DownloadState> for TaskState {
    fn from(value: &DownloadState) -> Self {
        match value {
            DownloadState::Queued => Self::Waiting,
            DownloadState::Connecting | DownloadState::Downloading | DownloadState::Seeding => {
                Self::Active
            }
            DownloadState::Paused => Self::Paused,
            DownloadState::Completed => Self::Complete,
            DownloadState::Error { .. } => Self::Error,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HttpDownloadRequest {
    pub url: String,
    pub save_dir: Option<PathBuf>,
    pub filename: Option<String>,
    pub headers: Vec<(String, String)>,
    pub max_connections: Option<usize>,
}

impl HttpDownloadRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            save_dir: None,
            filename: None,
            headers: Vec::new(),
            max_connections: None,
        }
    }
}

enum Command {
    AddHttp {
        request: HttpDownloadRequest,
        reply: oneshot::Sender<Result<String, String>>,
    },
    AddTorrent {
        data: Vec<u8>,
        save_dir: Option<PathBuf>,
        selected_files: Option<Vec<usize>>,
        reply: oneshot::Sender<Result<String, String>>,
    },
    AddMagnet {
        uri: String,
        save_dir: Option<PathBuf>,
        reply: oneshot::Sender<Result<String, String>>,
    },
    List {
        reply: oneshot::Sender<Vec<DownloadTask>>,
    },
    Pause {
        id: String,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Resume {
        id: String,
        reply: oneshot::Sender<Result<(), String>>,
    },
    Cancel {
        id: String,
        delete_files: bool,
        reply: oneshot::Sender<Result<(), String>>,
    },
    PauseAll {
        reply: oneshot::Sender<Result<(), String>>,
    },
    ResumeAll {
        reply: oneshot::Sender<Result<(), String>>,
    },
    Shutdown {
        reply: oneshot::Sender<Result<(), String>>,
    },
}

/// Thread-safe handle to the Tokio-owned embedded download engine.
#[derive(Clone)]
pub struct GoshBackend {
    commands: tokio_mpsc::UnboundedSender<Command>,
    events: broadcast::Sender<DownloadEvent>,
    default_connections: usize,
}

impl GoshBackend {
    pub fn start(config: &AppConfig, data_dir: &Path) -> Result<Self, String> {
        let engine_config = engine_config(config, data_dir);
        let default_connections = engine_config.max_connections_per_download;
        let (commands, receiver) = tokio_mpsc::unbounded_channel();
        let (events, _) = broadcast::channel(512);
        let event_output = events.clone();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);

        thread::Builder::new()
            .name("motrix-gosh-dl".to_owned())
            .spawn(move || run_engine(engine_config, receiver, event_output, ready_tx))
            .map_err(|error| format!("cannot start download engine thread: {error}"))?;

        ready_rx
            .recv()
            .map_err(|_| "download engine stopped during startup".to_owned())??;

        Ok(Self {
            commands,
            events,
            default_connections,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DownloadEvent> {
        self.events.subscribe()
    }

    pub async fn add_http(&self, mut request: HttpDownloadRequest) -> Result<String, String> {
        request
            .max_connections
            .get_or_insert(self.default_connections);
        self.request(|reply| Command::AddHttp { request, reply })
            .await?
    }

    pub async fn list(&self) -> Result<Vec<DownloadTask>, String> {
        self.request(|reply| Command::List { reply }).await
    }

    pub async fn task(&self, id: &str) -> Result<DownloadTask, String> {
        self.list()
            .await?
            .into_iter()
            .find(|task| task.id == id)
            .ok_or_else(|| format!("unknown download task `{id}`"))
    }

    /// Add a basic torrent task. Advanced peer and tracker controls stay an
    /// engine concern until Motrix has a concrete UI requirement for them.
    pub async fn add_torrent(
        &self,
        data: Vec<u8>,
        save_dir: Option<PathBuf>,
        selected_files: Option<Vec<usize>>,
    ) -> Result<String, String> {
        self.request(|reply| Command::AddTorrent {
            data,
            save_dir,
            selected_files,
            reply,
        })
        .await?
    }

    pub async fn add_magnet(
        &self,
        uri: impl Into<String>,
        save_dir: Option<PathBuf>,
    ) -> Result<String, String> {
        self.request(|reply| Command::AddMagnet {
            uri: uri.into(),
            save_dir,
            reply,
        })
        .await?
    }

    pub async fn pause(&self, id: impl Into<String>) -> Result<(), String> {
        self.request(|reply| Command::Pause {
            id: id.into(),
            reply,
        })
        .await?
    }

    pub async fn resume(&self, id: impl Into<String>) -> Result<(), String> {
        self.request(|reply| Command::Resume {
            id: id.into(),
            reply,
        })
        .await?
    }

    pub async fn cancel(&self, id: impl Into<String>, delete_files: bool) -> Result<(), String> {
        self.request(|reply| Command::Cancel {
            id: id.into(),
            delete_files,
            reply,
        })
        .await?
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        self.request(|reply| Command::Shutdown { reply }).await?
    }

    pub fn shutdown_blocking(&self) -> Result<(), String> {
        if tokio::runtime::Handle::try_current().is_ok() {
            let backend = self.clone();
            return thread::Builder::new()
                .name("motrix-gosh-shutdown".to_owned())
                .spawn(move || backend.shutdown_blocking())
                .map_err(|error| format!("cannot start shutdown thread: {error}"))?
                .join()
                .map_err(|_| "download shutdown thread panicked".to_owned())?;
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("cannot create shutdown runtime: {error}"))?;
        runtime.block_on(self.shutdown())
    }

    pub async fn pause_all(&self) -> Result<(), String> {
        self.request(|reply| Command::PauseAll { reply }).await?
    }

    pub async fn resume_all(&self) -> Result<(), String> {
        self.request(|reply| Command::ResumeAll { reply }).await?
    }

    async fn request<T>(
        &self,
        command: impl FnOnce(oneshot::Sender<T>) -> Command,
    ) -> Result<T, String> {
        let (reply, response) = oneshot::channel();
        self.commands
            .send(command(reply))
            .map_err(|_| "download engine is not running".to_owned())?;
        response
            .await
            .map_err(|_| "download engine dropped its response".to_owned())
    }
}

fn engine_config(config: &AppConfig, data_dir: &Path) -> EngineConfig {
    let mut engine = EngineConfig::default();
    if !config.download_dir.trim().is_empty() {
        engine.download_dir = PathBuf::from(&config.download_dir);
    }
    engine.database_path = Some(data_dir.join("downloads.sqlite3"));
    engine.max_concurrent_downloads = config.max_concurrent_downloads.max(1) as usize;
    engine.max_connections_per_download = config.max_connection_per_server.max(1) as usize;
    if let Some(bytes) = parse_byte_size(&config.min_split_size) {
        engine.min_segment_size = bytes.max(64 * 1024);
    }
    engine.global_download_limit = parse_nonzero_byte_size(&config.max_overall_download_limit);
    engine.global_upload_limit = parse_nonzero_byte_size(&config.max_overall_upload_limit);
    engine.user_agent = config.user_agent.clone();
    engine.enable_dht = config.dht_enabled;
    engine.enable_pex = config.pex_enabled;
    engine.max_peers = config.bt_max_peers.max(1) as usize;
    engine.seed_ratio = config.seed_ratio;
    engine.http.proxy_url = config.proxy.url();
    engine.torrent.listen_port_range = (config.listen_port, config.listen_port);
    engine
}

fn parse_nonzero_byte_size(value: &str) -> Option<u64> {
    parse_byte_size(value).filter(|value| *value > 0)
}

fn run_engine(
    config: EngineConfig,
    mut commands: tokio_mpsc::UnboundedReceiver<Command>,
    events: broadcast::Sender<DownloadEvent>,
    ready: mpsc::SyncSender<Result<(), String>>,
) {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("motrix-download-worker")
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = ready.send(Err(format!("cannot create download runtime: {error}")));
            return;
        }
    };

    runtime.block_on(async move {
        let engine = match DownloadEngine::new(config).await {
            Ok(engine) => engine,
            Err(error) => {
                let _ = ready.send(Err(format!("cannot initialize download engine: {error}")));
                return;
            }
        };
        let _ = ready.send(Ok(()));
        let mut engine_events = engine.subscribe();

        loop {
            tokio::select! {
                event = engine_events.recv() => match event {
                    Ok(event) => {
                        let event = match event {
                            EngineEvent::Progress { .. } => DownloadEvent::Progress,
                            _ => DownloadEvent::Changed,
                        };
                        let _ = events.send(event);
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "Motrix download event bridge lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                },
                command = commands.recv() => {
                    let Some(command) = command else { break };
                    if handle_command(&engine, command).await { break; }
                }
            }
        }

        if let Err(error) = engine.shutdown().await {
            tracing::warn!(%error, "could not gracefully stop gosh-dl");
        }
    });
}

async fn handle_command(engine: &Arc<DownloadEngine>, command: Command) -> bool {
    match command {
        Command::AddHttp { request, reply } => {
            let mut options = DownloadOptions::new();
            options.save_dir = request.save_dir;
            options.filename = request.filename;
            options.headers = request.headers;
            options.max_connections = request.max_connections;
            let result = engine
                .add_http(&request.url, options)
                .await
                .map(|id| id.to_string())
                .map_err(|error| error.to_string());
            let _ = reply.send(result);
        }
        Command::AddTorrent {
            data,
            save_dir,
            selected_files,
            reply,
        } => {
            let mut options = DownloadOptions::new();
            options.save_dir = save_dir;
            options.selected_files = selected_files;
            let result = engine
                .add_torrent(&data, options)
                .await
                .map(|id| id.to_string())
                .map_err(|error| error.to_string());
            let _ = reply.send(result);
        }
        Command::AddMagnet {
            uri,
            save_dir,
            reply,
        } => {
            let mut options = DownloadOptions::new();
            options.save_dir = save_dir;
            let result = engine
                .add_magnet(&uri, options)
                .await
                .map(|id| id.to_string())
                .map_err(|error| error.to_string());
            let _ = reply.send(result);
        }
        Command::List { reply } => {
            let tasks = engine.list().iter().map(project_task).collect();
            let _ = reply.send(tasks);
        }
        Command::Pause { id, reply } => {
            let result = match find_id(engine, &id) {
                Ok(id) => async_result(engine.pause(id).await),
                Err(error) => Err(error),
            };
            let _ = reply.send(result);
        }
        Command::Resume { id, reply } => {
            let result = match find_id(engine, &id) {
                Ok(id) => async_result(engine.resume(id).await),
                Err(error) => Err(error),
            };
            let _ = reply.send(result);
        }
        Command::Cancel {
            id,
            delete_files,
            reply,
        } => {
            let result = match find_id(engine, &id) {
                Ok(id) => async_result(engine.cancel(id, delete_files).await),
                Err(error) => Err(error),
            };
            let _ = reply.send(result);
        }
        Command::PauseAll { reply } => {
            let result = batch_result(engine.pause_all().await);
            let _ = reply.send(result);
        }
        Command::ResumeAll { reply } => {
            let result = batch_result(engine.resume_all().await);
            let _ = reply.send(result);
        }
        Command::Shutdown { reply } => {
            let result = engine.shutdown().await.map_err(|error| error.to_string());
            let _ = reply.send(result);
            return true;
        }
    }
    false
}

fn async_result<T>(result: gosh_dl::Result<T>) -> Result<T, String> {
    result.map_err(|error| error.to_string())
}

fn batch_result(result: gosh_dl::BatchResult) -> Result<(), String> {
    if result.failed.is_empty() {
        Ok(())
    } else {
        Err(result
            .failed
            .into_iter()
            .map(|(id, error)| format!("{id}: {error}"))
            .collect::<Vec<_>>()
            .join("; "))
    }
}

fn find_id(engine: &DownloadEngine, id: &str) -> Result<DownloadId, String> {
    engine
        .list()
        .into_iter()
        .find(|status| status.id.matches_gid(id))
        .map(|status| status.id)
        .ok_or_else(|| format!("unknown download task `{id}`"))
}

fn project_task(status: &gosh_dl::DownloadStatus) -> DownloadTask {
    DownloadTask {
        id: status.id.to_string(),
        name: status.metadata.name.clone(),
        state: TaskState::from(&status.state),
        total_size: status.progress.total_size,
        completed_size: status.progress.completed_size,
        download_speed: status.progress.download_speed,
        upload_speed: status.progress.upload_speed,
        connections: status.progress.connections,
        save_dir: status.metadata.save_dir.clone(),
        source: status
            .metadata
            .url
            .clone()
            .or_else(|| status.metadata.magnet_uri.clone()),
        bittorrent: matches!(status.kind, DownloadKind::Torrent | DownloadKind::Magnet),
        seeders: matches!(status.kind, DownloadKind::Torrent | DownloadKind::Magnet)
            .then_some(status.progress.seeders as u64),
        error: match &status.state {
            DownloadState::Error { message, .. } => Some(message.clone()),
            _ => None,
        },
        file_count: status
            .torrent_info
            .as_ref()
            .map_or(1, |info| info.files.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
        time::Duration,
    };

    #[test]
    fn motrix_config_enables_parallel_http_downloads() {
        let config = AppConfig {
            max_concurrent_downloads: 7,
            max_connection_per_server: 12,
            ..AppConfig::default()
        };
        let engine = engine_config(&config, Path::new("/tmp/motrix-test"));
        assert_eq!(engine.max_concurrent_downloads, 7);
        assert_eq!(engine.max_connections_per_download, 12);
        assert_eq!(engine.min_segment_size, 20 * 1024 * 1024);
        assert_eq!(
            engine.database_path,
            Some(PathBuf::from("/tmp/motrix-test/downloads.sqlite3"))
        );
    }

    #[test]
    fn byte_sizes_map_to_gosh_limits() {
        assert_eq!(parse_byte_size("512K"), Some(512 * 1024));
        assert_eq!(parse_byte_size("20 MiB"), Some(20 * 1024 * 1024));
        assert_eq!(parse_nonzero_byte_size("0"), None);
        assert_eq!(parse_byte_size("1.5M"), None);
    }

    #[test]
    fn task_states_are_projected_without_leaking_engine_types() {
        assert_eq!(TaskState::from(&DownloadState::Queued), TaskState::Waiting);
        assert_eq!(
            TaskState::from(&DownloadState::Downloading),
            TaskState::Active
        );
        assert_eq!(
            TaskState::from(&DownloadState::Error {
                kind: "network".to_owned(),
                message: "offline".to_owned(),
                retryable: true,
            }),
            TaskState::Error
        );
    }

    #[tokio::test]
    async fn embedded_engine_starts_rejects_invalid_http_and_shuts_down() {
        let root = std::env::temp_dir().join(format!("motrix-gosh-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("create temporary engine directory");
        let config = AppConfig {
            download_dir: root.to_string_lossy().into_owned(),
            ..AppConfig::default()
        };
        let backend = GoshBackend::start(&config, &root).expect("start embedded engine");

        let error = backend
            .add_http(HttpDownloadRequest::new("file:///not-an-http-download"))
            .await
            .expect_err("reject unsupported URL scheme");
        assert!(error.contains("Unsupported scheme"), "{error}");
        backend.shutdown().await.expect("stop embedded engine");

        std::fs::remove_dir_all(root).expect("remove temporary engine directory");
    }

    #[tokio::test]
    async fn embedded_http_uses_multiple_range_requests() {
        let payload = Arc::new(vec![0x5a; 512 * 1024]);
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind range server");
        listener.set_nonblocking(true).expect("nonblocking server");
        let address = listener.local_addr().expect("range server address");
        let range_requests = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let server_payload = payload.clone();
        let server_ranges = range_requests.clone();
        let server_stop = stop.clone();
        let server = thread::spawn(move || {
            while !server_stop.load(Ordering::Acquire) {
                let Ok((mut stream, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(2));
                    continue;
                };
                let mut request = Vec::new();
                let mut byte = [0_u8; 1];
                while request.len() < 16 * 1024 && stream.read_exact(&mut byte).is_ok() {
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }
                let request = String::from_utf8_lossy(&request);
                let head = request.starts_with("HEAD ");
                let range = request.lines().find_map(|line| {
                    line.strip_prefix("Range: bytes=")
                        .or_else(|| line.strip_prefix("range: bytes="))
                });
                let (start, end, partial) = range
                    .and_then(|value| value.split_once('-'))
                    .and_then(|(start, end)| {
                        Some((
                            start.parse::<usize>().ok()?,
                            end.parse::<usize>().ok()?,
                            true,
                        ))
                    })
                    .unwrap_or((0, server_payload.len() - 1, false));
                if partial {
                    server_ranges.fetch_add(1, Ordering::AcqRel);
                }
                let length = end - start + 1;
                let status = if partial {
                    "206 Partial Content"
                } else {
                    "200 OK"
                };
                let content_range = if partial {
                    format!(
                        "Content-Range: bytes {start}-{end}/{}\r\n",
                        server_payload.len()
                    )
                } else {
                    String::new()
                };
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Length: {length}\r\nAccept-Ranges: bytes\r\n{content_range}Connection: close\r\n\r\n"
                )
                .expect("write response headers");
                if !head {
                    stream
                        .write_all(&server_payload[start..=end])
                        .expect("write response body");
                }
            }
        });

        let root = std::env::temp_dir().join(format!("motrix-range-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("create range download directory");
        let config = AppConfig {
            download_dir: root.to_string_lossy().into_owned(),
            min_split_size: "64K".to_owned(),
            max_connection_per_server: 4,
            ..AppConfig::default()
        };
        let backend = GoshBackend::start(&config, &root).expect("start embedded engine");
        let id = backend
            .add_http(HttpDownloadRequest::new(format!(
                "http://{address}/fixture.bin"
            )))
            .await
            .expect("add range download");
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            let task = backend.task(&id).await.expect("read range task");
            if task.state == TaskState::Complete {
                break;
            }
            assert!(
                task.state != TaskState::Error,
                "range download failed: {task:?}"
            );
            assert!(
                tokio::time::Instant::now() < deadline,
                "range download timed out"
            );
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        assert_eq!(std::fs::read(root.join("fixture.bin")).unwrap(), *payload);
        assert!(
            range_requests.load(Ordering::Acquire) > 1,
            "expected multiple HTTP range requests"
        );
        backend.shutdown().await.expect("stop embedded engine");
        stop.store(true, Ordering::Release);
        server.join().expect("join range server");
        std::fs::remove_dir_all(root).expect("remove range download directory");
    }
}
