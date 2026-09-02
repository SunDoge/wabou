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

pub const CAPABILITY: CapabilityContract = CapabilityContract::new("rustic", 2);

const STATUS: HostMethod<(), RuntimeStatus> = HostMethod::no_request("status");
const CREATE_PROFILE: HostMethod<ProfileRequest, RuntimeStatus> = HostMethod::new("createProfile");
const OPEN_PROFILE: HostMethod<ProfileRequest, RuntimeStatus> = HostMethod::new("openProfile");
const SELECT_PROFILE: HostMethod<SelectProfileRequest, RuntimeStatus> =
    HostMethod::new("selectProfile");
const SET_SOURCES: HostMethod<SetSourcesRequest, RuntimeStatus> = HostMethod::new("setSources");
const RUN_BACKUP: HostMethod<ProfileIdRequest, BackupResult> = HostMethod::new("runBackup");
const LIST_SNAPSHOTS: HostMethod<ProfileIdRequest, Vec<SnapshotEntry>> =
    HostMethod::new("listSnapshots");
const LIST_FILES: HostMethod<ListFilesRequest, Vec<FileEntry>> = HostMethod::new("listFiles");

#[derive(Clone, Default)]
pub struct RusticService {
    state: Arc<RwLock<ServiceState>>,
}

#[derive(Clone, Default)]
struct ServiceState {
    profiles: Vec<ProfileState>,
    active_profile_id: Option<String>,
}

#[derive(Clone)]
struct ProfileState {
    id: String,
    name: String,
    repository_path: String,
    password: String,
    sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRequest {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetSourcesRequest {
    pub profile_id: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileIdRequest {
    pub profile_id: String,
}

pub type SelectProfileRequest = ProfileIdRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFilesRequest {
    pub profile_id: String,
    pub snapshot_id: String,
    #[serde(default)]
    pub path: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub unlocked_profile_ids: Vec<String>,
    pub active_profile_id: Option<String>,
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
    fn status(&self) -> Result<RuntimeStatus, String> {
        let state = self.state.read().map_err(|_| "service state is poisoned")?;
        Ok(status_from_state(&state))
    }

    fn remember_profile(&self, request: ProfileRequest) -> Result<RuntimeStatus, String> {
        let name = request.name.trim();
        if name.is_empty() {
            return Err("backup name is required".to_string());
        }
        let mut state = self
            .state
            .write()
            .map_err(|_| "service state is poisoned")?;
        if request.id.trim().is_empty() {
            return Err("backup profile id is required".to_string());
        }
        if let Some(profile) = state
            .profiles
            .iter_mut()
            .find(|profile| profile.id == request.id)
        {
            profile.name = name.to_string();
            profile.repository_path = request.path;
            profile.password = request.password;
            profile.sources = normalize_sources(request.sources);
        } else {
            state.profiles.push(ProfileState {
                id: request.id.clone(),
                name: name.to_string(),
                repository_path: request.path,
                password: request.password,
                sources: normalize_sources(request.sources),
            });
        }
        state.active_profile_id = Some(request.id);
        Ok(status_from_state(&state))
    }

    fn profile_config(&self, profile_id: &str) -> Result<(String, String, Vec<String>), String> {
        let state = self.state.read().map_err(|_| "service state is poisoned")?;
        let profile = state
            .profiles
            .iter()
            .find(|profile| profile.id == profile_id)
            .ok_or_else(|| format!("backup profile {profile_id} was not found"))?;
        Ok((
            profile.repository_path.clone(),
            profile.password.clone(),
            profile.sources.clone(),
        ))
    }

    fn create_profile(&self, request: ProfileRequest) -> Result<RuntimeStatus, String> {
        validate_profile_request(&request)?;
        create_repository(&request.path, &request.password)?;
        self.remember_profile(request)
    }

    fn open_profile(&self, request: ProfileRequest) -> Result<RuntimeStatus, String> {
        validate_profile_request(&request)?;
        open_repository(&request.path, &request.password).map(|_| ())?;
        self.remember_profile(request)
    }

    fn select_profile(&self, request: SelectProfileRequest) -> Result<RuntimeStatus, String> {
        let mut state = self
            .state
            .write()
            .map_err(|_| "service state is poisoned")?;
        if !state
            .profiles
            .iter()
            .any(|profile| profile.id == request.profile_id)
        {
            return Err(format!(
                "backup profile {} was not found",
                request.profile_id
            ));
        }
        state.active_profile_id = Some(request.profile_id);
        Ok(status_from_state(&state))
    }

    fn set_sources(&self, request: SetSourcesRequest) -> Result<RuntimeStatus, String> {
        let sources = normalize_sources(request.sources);
        let mut state = self
            .state
            .write()
            .map_err(|_| "service state is poisoned".to_string())?;
        let profile = state
            .profiles
            .iter_mut()
            .find(|profile| profile.id == request.profile_id)
            .ok_or_else(|| format!("backup profile {} was not found", request.profile_id))?;
        profile.sources = sources;
        Ok(status_from_state(&state))
    }

    fn run_backup(&self, request: ProfileIdRequest) -> Result<BackupResult, String> {
        let (path, password, sources) = self.profile_config(&request.profile_id)?;
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

    fn list_snapshots(&self, request: ProfileIdRequest) -> Result<Vec<SnapshotEntry>, String> {
        let (path, password, _) = self.profile_config(&request.profile_id)?;
        let repo = open_repository(&path, &password)?;
        let mut snapshots = repo.get_all_snapshots().map_err(display_error)?;
        snapshots.sort_unstable_by(|left, right| right.time.cmp(&left.time));
        Ok(snapshots.iter().map(snapshot_entry).collect())
    }

    fn list_files(&self, request: ListFilesRequest) -> Result<Vec<FileEntry>, String> {
        let (path, password, _) = self.profile_config(&request.profile_id)?;
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

fn normalize_sources(sources: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for source in sources {
        let source = source.trim();
        if !source.is_empty() && !normalized.iter().any(|current| current == source) {
            normalized.push(source.to_string());
        }
    }
    normalized
}

fn validate_profile_request(request: &ProfileRequest) -> Result<(), String> {
    if request.id.trim().is_empty() {
        return Err("backup profile id is required".to_string());
    }
    if request.name.trim().is_empty() {
        return Err("backup name is required".to_string());
    }
    if request.path.trim().is_empty() {
        return Err("repository path is required".to_string());
    }
    if request.password.is_empty() {
        return Err("repository password is required".to_string());
    }
    Ok(())
}

fn status_from_state(state: &ServiceState) -> RuntimeStatus {
    RuntimeStatus {
        unlocked_profile_ids: state
            .profiles
            .iter()
            .map(|profile| profile.id.clone())
            .collect(),
        active_profile_id: state.active_profile_id.clone(),
    }
}

pub fn mount(capability: NativeCapability<'_>, service: RusticService) -> rquickjs::Result<()> {
    let status = service.clone();
    capability.method(STATUS, move |(): ()| {
        let service = status.clone();
        async move { service.status() }
    })?;

    let create = service.clone();
    capability.method(CREATE_PROFILE, move |request| {
        let service = create.clone();
        async move {
            tokio::task::spawn_blocking(move || service.create_profile(request))
                .await
                .map_err(|error| format!("repository task failed: {error}"))?
        }
    })?;

    let open = service.clone();
    capability.method(OPEN_PROFILE, move |request| {
        let service = open.clone();
        async move {
            tokio::task::spawn_blocking(move || service.open_profile(request))
                .await
                .map_err(|error| format!("repository task failed: {error}"))?
        }
    })?;

    let select = service.clone();
    capability.method(SELECT_PROFILE, move |request| {
        let service = select.clone();
        async move { service.select_profile(request) }
    })?;

    let set_sources = service.clone();
    capability.method(SET_SOURCES, move |request| {
        let service = set_sources.clone();
        async move { service.set_sources(request) }
    })?;

    let backup = service.clone();
    capability.method(RUN_BACKUP, move |request| {
        let service = backup.clone();
        async move {
            tokio::task::spawn_blocking(move || service.run_backup(request))
                .await
                .map_err(|error| format!("backup task failed: {error}"))?
        }
    })?;

    let snapshots = service.clone();
    capability.method(LIST_SNAPSHOTS, move |request| {
        let service = snapshots.clone();
        async move {
            tokio::task::spawn_blocking(move || service.list_snapshots(request))
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
    fn profile_rejects_an_empty_repository_password() {
        let error = RusticService::default()
            .create_profile(ProfileRequest {
                id: "test".to_string(),
                name: "Test".to_string(),
                path: "/unused".to_string(),
                password: String::new(),
                sources: Vec::new(),
            })
            .expect_err("empty passwords must not create repositories");
        assert_eq!(error, "repository password is required");
    }

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
            .create_profile(ProfileRequest {
                id: "photos".to_string(),
                name: "Photos".to_string(),
                path: repository.to_string_lossy().into_owned(),
                password: "wabou-rustic-test".to_string(),
                sources: Vec::new(),
            })
            .expect("create repository");
        service
            .set_sources(SetSourcesRequest {
                profile_id: "photos".to_string(),
                sources: vec![source.to_string_lossy().into_owned()],
            })
            .expect("set source");
        let profile = ProfileIdRequest {
            profile_id: "photos".to_string(),
        };
        let backup = service.run_backup(profile.clone()).expect("backup source");
        let snapshots = service.list_snapshots(profile).expect("list snapshots");
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, backup.snapshot.id);

        let mut pending = vec![String::new()];
        let mut names = Vec::new();
        while let Some(path) = pending.pop() {
            for entry in service
                .list_files(ListFilesRequest {
                    profile_id: "photos".to_string(),
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
