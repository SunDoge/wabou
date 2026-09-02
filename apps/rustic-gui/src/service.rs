use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use rustic_backend::BackendOptions;
use rustic_core::{
    BackupOptions, ConfigOptions, Credentials, KeyOptions, LsOptions, PathList, Repository,
    RepositoryOptions, SnapshotOptions,
};
use serde::{Deserialize, Serialize};
use wabou::{CapabilityContract, HostMethod, NativeCapability, rquickjs};

pub const CAPABILITY: CapabilityContract = CapabilityContract::new("rustic", 1);

const STATUS: HostMethod<(), AppStatus> = HostMethod::no_request("status");
const CREATE_REPOSITORY: HostMethod<RepositoryRequest, AppStatus> =
    HostMethod::new("createRepository");
const OPEN_REPOSITORY: HostMethod<RepositoryRequest, AppStatus> = HostMethod::new("openRepository");
const SET_SOURCES: HostMethod<SetSourcesRequest, AppStatus> = HostMethod::new("setSources");
const RUN_BACKUP: HostMethod<(), BackupResult> = HostMethod::no_request("runBackup");
const LIST_SNAPSHOTS: HostMethod<(), Vec<SnapshotEntry>> = HostMethod::no_request("listSnapshots");
const LIST_FILES: HostMethod<ListFilesRequest, Vec<FileEntry>> = HostMethod::new("listFiles");

#[derive(Clone, Default)]
pub struct RusticService {
    state: Arc<RwLock<ServiceState>>,
}

#[derive(Clone, Default)]
struct ServiceState {
    repository_path: Option<String>,
    password: String,
    sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryRequest {
    pub path: String,
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSourcesRequest {
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFilesRequest {
    pub snapshot_id: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub connected: bool,
    pub repository_path: Option<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotEntry {
    pub id: String,
    pub time: String,
    pub hostname: String,
    pub paths: Vec<String>,
    pub files_new: u64,
    pub files_changed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub size: u64,
    pub modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub snapshot: SnapshotEntry,
}

impl RusticService {
    fn status(&self) -> Result<AppStatus, String> {
        let state = self.state.read().map_err(|_| "service state is poisoned")?;
        Ok(AppStatus {
            connected: state.repository_path.is_some(),
            repository_path: state.repository_path.clone(),
            sources: state.sources.clone(),
        })
    }

    fn remember_repository(&self, request: RepositoryRequest) -> Result<AppStatus, String> {
        let mut state = self
            .state
            .write()
            .map_err(|_| "service state is poisoned")?;
        state.repository_path = Some(request.path);
        state.password = request.password;
        drop(state);
        self.status()
    }

    fn repository_config(&self) -> Result<(String, String), String> {
        let state = self.state.read().map_err(|_| "service state is poisoned")?;
        let path = state
            .repository_path
            .clone()
            .ok_or_else(|| "no repository is open".to_string())?;
        Ok((path, state.password.clone()))
    }

    fn sources(&self) -> Result<Vec<String>, String> {
        self.state
            .read()
            .map(|state| state.sources.clone())
            .map_err(|_| "service state is poisoned".to_string())
    }

    fn create_repository(&self, request: RepositoryRequest) -> Result<AppStatus, String> {
        create_repository(&request.path, &request.password)?;
        self.remember_repository(request)
    }

    fn open_repository(&self, request: RepositoryRequest) -> Result<AppStatus, String> {
        open_repository(&request.path, &request.password).map(|_| ())?;
        self.remember_repository(request)
    }

    fn set_sources(&self, request: SetSourcesRequest) -> Result<AppStatus, String> {
        let sources = request
            .sources
            .into_iter()
            .filter_map(|source| {
                let source = source.trim();
                (!source.is_empty()).then(|| source.to_string())
            })
            .collect();
        self.state
            .write()
            .map_err(|_| "service state is poisoned".to_string())?
            .sources = sources;
        self.status()
    }

    fn run_backup(&self) -> Result<BackupResult, String> {
        let (path, password) = self.repository_config()?;
        let sources = self.sources()?;
        if sources.is_empty() {
            return Err("add at least one backup folder first".to_string());
        }
        let repo = open_repository(&path, &password)?
            .to_indexed_ids()
            .map_err(display_error)?;
        let source = PathList::from_iter(sources.iter().map(PathBuf::from))
            .sanitize()
            .map_err(display_error)?;
        let snapshot = SnapshotOptions::default()
            .to_snapshot()
            .map_err(display_error)?;
        let snapshot = repo
            .backup(&BackupOptions::default(), &source, snapshot)
            .map_err(display_error)?;
        Ok(BackupResult {
            snapshot: snapshot_entry(&snapshot),
        })
    }

    fn list_snapshots(&self) -> Result<Vec<SnapshotEntry>, String> {
        let (path, password) = self.repository_config()?;
        let repo = open_repository(&path, &password)?;
        let mut snapshots = repo.get_all_snapshots().map_err(display_error)?;
        snapshots.sort_unstable_by(|left, right| right.time.cmp(&left.time));
        Ok(snapshots.iter().map(snapshot_entry).collect())
    }

    fn list_files(&self, request: ListFilesRequest) -> Result<Vec<FileEntry>, String> {
        let (path, password) = self.repository_config()?;
        let repo = open_repository(&path, &password)?
            .to_indexed_ids()
            .map_err(display_error)?;
        let snapshots = repo.get_all_snapshots().map_err(display_error)?;
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.id.to_string() == request.snapshot_id)
            .ok_or_else(|| format!("snapshot {} was not found", request.snapshot_id))?;
        let node = repo
            .node_from_snapshot_and_path(snapshot, &request.path)
            .map_err(display_error)?;
        let options = LsOptions::default().recursive(false);
        let mut files = repo
            .ls(&node, &options)
            .map_err(display_error)?
            .map(|entry| {
                let (relative_path, node) = entry.map_err(display_error)?;
                let path = if request.path.is_empty() {
                    relative_path
                } else {
                    PathBuf::from(&request.path).join(relative_path)
                };
                Ok(FileEntry {
                    name: node.name().to_string_lossy().into_owned(),
                    path: path.to_string_lossy().into_owned(),
                    kind: if node.is_dir() {
                        "directory"
                    } else if node.is_file() {
                        "file"
                    } else if node.is_symlink() {
                        "symlink"
                    } else {
                        "special"
                    }
                    .to_string(),
                    size: node.meta.size,
                    modified: node.meta.mtime.map(|time| time.to_string()),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        files.sort_unstable_by(|left, right| {
            (left.kind != "directory", left.name.to_lowercase())
                .cmp(&(right.kind != "directory", right.name.to_lowercase()))
        });
        Ok(files)
    }
}

fn create_repository(path: &str, password: &str) -> Result<(), String> {
    let backends = BackendOptions::default()
        .repository(path)
        .to_backends()
        .map_err(display_error)?;
    Repository::new(&RepositoryOptions::default(), &backends)
        .map_err(display_error)?
        .init(
            &Credentials::password(password),
            &KeyOptions::default(),
            &ConfigOptions::default(),
        )
        .map(|_| ())
        .map_err(display_error)
}

fn open_repository(
    path: &str,
    password: &str,
) -> Result<rustic_core::Repository<rustic_core::OpenStatus>, String> {
    let backends = BackendOptions::default()
        .repository(path)
        .to_backends()
        .map_err(display_error)?;
    Repository::new(&RepositoryOptions::default(), &backends)
        .map_err(display_error)?
        .open(&Credentials::password(password))
        .map_err(display_error)
}

fn snapshot_entry(snapshot: &rustic_core::repofile::SnapshotFile) -> SnapshotEntry {
    let summary = snapshot.summary.as_ref();
    SnapshotEntry {
        id: snapshot.id.to_string(),
        time: snapshot.time.to_string(),
        hostname: snapshot.hostname.clone(),
        paths: snapshot.paths.iter().cloned().collect(),
        files_new: summary.map_or(0, |summary| summary.files_new),
        files_changed: summary.map_or(0, |summary| summary.files_changed),
    }
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

pub fn mount(capability: NativeCapability<'_>, service: RusticService) -> rquickjs::Result<()> {
    let status = service.clone();
    capability.method(STATUS, move |(): ()| {
        let service = status.clone();
        async move { service.status() }
    })?;

    let create = service.clone();
    capability.method(CREATE_REPOSITORY, move |request| {
        let service = create.clone();
        async move {
            tokio::task::spawn_blocking(move || service.create_repository(request))
                .await
                .map_err(|error| format!("repository task failed: {error}"))?
        }
    })?;

    let open = service.clone();
    capability.method(OPEN_REPOSITORY, move |request| {
        let service = open.clone();
        async move {
            tokio::task::spawn_blocking(move || service.open_repository(request))
                .await
                .map_err(|error| format!("repository task failed: {error}"))?
        }
    })?;

    let set_sources = service.clone();
    capability.method(SET_SOURCES, move |request| {
        let service = set_sources.clone();
        async move { service.set_sources(request) }
    })?;

    let backup = service.clone();
    capability.method(RUN_BACKUP, move |(): ()| {
        let service = backup.clone();
        async move {
            tokio::task::spawn_blocking(move || service.run_backup())
                .await
                .map_err(|error| format!("backup task failed: {error}"))?
        }
    })?;

    let snapshots = service.clone();
    capability.method(LIST_SNAPSHOTS, move |(): ()| {
        let service = snapshots.clone();
        async move {
            tokio::task::spawn_blocking(move || service.list_snapshots())
                .await
                .map_err(|error| format!("snapshot task failed: {error}"))?
        }
    })?;

    capability.method(LIST_FILES, move |request| {
        let service = service.clone();
        async move {
            tokio::task::spawn_blocking(move || service.list_files(request))
                .await
                .map_err(|error| format!("file listing task failed: {error}"))?
        }
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn local_repository_vertical_slice() {
        let root = std::env::var_os("WABOU_RUSTIC_TEST_ROOT").map_or_else(
            || tempfile::tempdir().expect("test root"),
            |parent| {
                tempfile::Builder::new()
                    .prefix("wabou-rustic-gui-")
                    .tempdir_in(parent)
                    .expect("test root in configured directory")
            },
        );
        let repository = root.path().join("repository");
        let source = root.path().join("source");
        fs::create_dir_all(source.join("chapter")).expect("source directories");
        fs::write(source.join("cover.txt"), "cover").expect("root file");
        fs::write(source.join("chapter/page.txt"), "page").expect("nested file");

        let service = RusticService::default();
        service
            .create_repository(RepositoryRequest {
                path: repository.to_string_lossy().into_owned(),
                password: "test-password".to_string(),
            })
            .expect("create repository");
        service
            .set_sources(SetSourcesRequest {
                sources: vec![source.to_string_lossy().into_owned()],
            })
            .expect("set source");
        let backup = service.run_backup().expect("backup source");
        let snapshots = service.list_snapshots().expect("list snapshots");
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, backup.snapshot.id);

        let mut pending = vec![String::new()];
        let mut names = Vec::new();
        while let Some(path) = pending.pop() {
            for entry in service
                .list_files(ListFilesRequest {
                    snapshot_id: backup.snapshot.id.clone(),
                    path,
                })
                .expect("list files")
            {
                names.push(entry.name.clone());
                if entry.kind == "directory" {
                    pending.push(entry.path);
                }
            }
        }
        assert!(names.iter().any(|name| name == "cover.txt"));
        assert!(names.iter().any(|name| name == "chapter"));
    }
}
