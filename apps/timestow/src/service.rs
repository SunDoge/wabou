use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};

use rustic_backend::BackendOptions;
use rustic_core::{
    BackupOptions, ConfigOptions, Credentials, KeyOptions, LocalDestination, LsOptions, PathList,
    Repository, RepositoryOptions, RestoreOptions, SnapshotOptions,
    repofile::{DeleteOption, StringList},
};
use serde::{Deserialize, Serialize};
use wabou::{CapabilityContract, HostMethod, NativeCapability, rquickjs};

pub const CAPABILITY: CapabilityContract = CapabilityContract::new("rustic", 5);

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
const SEARCH_FILES: HostMethod<SearchFilesRequest, Vec<FileEntry>> = HostMethod::new("searchFiles");
const DIFF_SNAPSHOTS: HostMethod<DiffSnapshotsRequest, SnapshotDiff> =
    HostMethod::new("diffSnapshots");
const UPDATE_SNAPSHOT: HostMethod<UpdateSnapshotRequest, SnapshotEntry> =
    HostMethod::new("updateSnapshot");
const PREVIEW_RESTORE: HostMethod<RestorePathRequest, RestorePlanSummary> =
    HostMethod::new("previewRestore");
const RESTORE_PATH: HostMethod<RestorePathRequest, RestoreResult> = HostMethod::new("restorePath");
const PREVIEW_PATH: HostMethod<PreviewPathRequest, RestoreResult> = HostMethod::new("previewPath");
const OPEN_PATH: HostMethod<OpenPathRequest, ()> = HostMethod::new("openPath");

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilesRequest {
    pub profile_id: String,
    pub snapshot_id: String,
    pub query: String,
    #[serde(default = "default_search_limit")]
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffSnapshotsRequest {
    pub profile_id: String,
    pub base_snapshot_id: String,
    pub snapshot_id: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSnapshotRequest {
    pub profile_id: String,
    pub snapshot_id: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub delete_protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePathRequest {
    pub profile_id: String,
    pub snapshot_id: String,
    pub path: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewPathRequest {
    pub profile_id: String,
    pub snapshot_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenPathRequest {
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
    pub parent_id: Option<String>,
    pub label: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub delete_protected: bool,
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
pub struct SnapshotDiffEntry {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub change: String,
    pub previous_size: Option<u64>,
    pub current_size: Option<u64>,
    pub previous_modified: Option<String>,
    pub current_modified: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDiffSummary {
    pub added: u64,
    pub removed: u64,
    pub modified: u64,
    pub metadata: u64,
    pub type_changed: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDiff {
    pub entries: Vec<SnapshotDiffEntry>,
    pub summary: SnapshotDiffSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub snapshot: SnapshotEntry,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestorePlanSummary {
    pub restore_size: u64,
    pub matched_size: u64,
    pub files_to_restore: u64,
    pub files_to_modify: u64,
    pub files_unchanged: u64,
    pub directories_to_restore: u64,
    pub directories_to_modify: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub destination: String,
    pub plan: RestorePlanSummary,
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
                Ok(file_entry(path, &node))
            })
            .collect::<Result<Vec<_>, String>>()?;
        files.sort_unstable_by(|left, right| {
            (left.kind != "directory", left.name.to_lowercase())
                .cmp(&(right.kind != "directory", right.name.to_lowercase()))
        });
        Ok(files)
    }

    fn search_files(&self, request: SearchFilesRequest) -> Result<Vec<FileEntry>, String> {
        let query = request.query.trim().to_lowercase();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let (path, password, _) = self.profile_config(&request.profile_id)?;
        let repo = open_repository(&path, &password)?
            .to_indexed_ids()
            .map_err(display_error)?;
        let snapshots = repo.get_all_snapshots().map_err(display_error)?;
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.id.to_string() == request.snapshot_id)
            .ok_or_else(|| format!("snapshot {} was not found", request.snapshot_id))?;
        let root = repo
            .node_from_snapshot_and_path(snapshot, "")
            .map_err(display_error)?;
        let limit = request.limit.clamp(1, 500);
        let mut matches = Vec::new();
        for entry in repo
            .ls(&root, &LsOptions::default().recursive(true))
            .map_err(display_error)?
        {
            let (relative_path, node) = entry.map_err(display_error)?;
            if relative_path
                .to_string_lossy()
                .to_lowercase()
                .contains(&query)
            {
                matches.push(file_entry(relative_path, &node));
                if matches.len() == limit {
                    break;
                }
            }
        }
        Ok(matches)
    }

    fn diff_snapshots(&self, request: DiffSnapshotsRequest) -> Result<SnapshotDiff, String> {
        if request.base_snapshot_id == request.snapshot_id {
            return Err("choose two different snapshots to compare".to_string());
        }
        let (path, password, _) = self.profile_config(&request.profile_id)?;
        let repo = open_repository(&path, &password)?
            .to_indexed_ids()
            .map_err(display_error)?;
        let snapshots = repo.get_all_snapshots().map_err(display_error)?;
        let base = snapshots
            .iter()
            .find(|snapshot| snapshot.id.to_string() == request.base_snapshot_id)
            .ok_or_else(|| format!("snapshot {} was not found", request.base_snapshot_id))?;
        let current = snapshots
            .iter()
            .find(|snapshot| snapshot.id.to_string() == request.snapshot_id)
            .ok_or_else(|| format!("snapshot {} was not found", request.snapshot_id))?;
        let base_node = repo
            .node_from_snapshot_and_path(base, &request.path)
            .map_err(display_error)?;
        let current_node = repo
            .node_from_snapshot_and_path(current, &request.path)
            .map_err(display_error)?;
        let options = LsOptions::default().recursive(true);
        let base_entries = repo
            .ls(&base_node, &options)
            .map_err(display_error)?
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(display_error)?;
        let current_entries = repo
            .ls(&current_node, &options)
            .map_err(display_error)?
            .collect::<Result<BTreeMap<_, _>, _>>()
            .map_err(display_error)?;
        let paths = base_entries
            .keys()
            .chain(current_entries.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut result = SnapshotDiff::default();
        for relative_path in paths {
            let previous = base_entries.get(&relative_path);
            let current = current_entries.get(&relative_path);
            let Some(change) = diff_change(previous, current, request.include_metadata) else {
                continue;
            };
            match change {
                "added" => result.summary.added += 1,
                "removed" => result.summary.removed += 1,
                "modified" => result.summary.modified += 1,
                "metadata" => result.summary.metadata += 1,
                "typeChanged" => result.summary.type_changed += 1,
                _ => {}
            }
            let node = current.or(previous).expect("a diff entry has one side");
            let full_path = if request.path.is_empty() {
                relative_path
            } else {
                PathBuf::from(&request.path).join(relative_path)
            };
            result.entries.push(SnapshotDiffEntry {
                name: node.name().to_string_lossy().into_owned(),
                path: full_path.to_string_lossy().into_owned(),
                kind: node_kind(node),
                change: change.to_string(),
                previous_size: previous.map(|node| node.meta.size),
                current_size: current.map(|node| node.meta.size),
                previous_modified: previous
                    .and_then(|node| node.meta.mtime.map(|time| time.to_string())),
                current_modified: current
                    .and_then(|node| node.meta.mtime.map(|time| time.to_string())),
            });
        }
        Ok(result)
    }

    fn update_snapshot(&self, request: UpdateSnapshotRequest) -> Result<SnapshotEntry, String> {
        let (path, password, _) = self.profile_config(&request.profile_id)?;
        let repo = open_repository(&path, &password)?;
        let snapshots = repo.get_all_snapshots().map_err(display_error)?;
        let previous_ids: BTreeSet<_> = snapshots.iter().map(|snapshot| snapshot.id).collect();
        let mut snapshot = snapshots
            .into_iter()
            .find(|snapshot| snapshot.id.to_string() == request.snapshot_id)
            .ok_or_else(|| format!("snapshot {} was not found", request.snapshot_id))?;
        let old_id = snapshot.id;
        snapshot.label = request.label.trim().to_string();
        snapshot.description = match request.description.trim() {
            "" => None,
            description => Some(description.to_string()),
        };
        let mut tags = StringList::default();
        for tag in request.tags {
            let tag = tag.trim();
            if !tag.is_empty() {
                tags.add(tag.to_string());
            }
        }
        snapshot.tags = tags;
        snapshot.delete = if request.delete_protected {
            DeleteOption::Never
        } else {
            DeleteOption::NotSet
        };
        repo.save_snapshots(vec![snapshot]).map_err(display_error)?;
        repo.delete_snapshots(&[old_id]).map_err(display_error)?;
        repo.get_all_snapshots()
            .map_err(display_error)?
            .iter()
            .find(|snapshot| !previous_ids.contains(&snapshot.id))
            .map(snapshot_entry)
            .ok_or_else(|| "updated snapshot could not be reloaded".to_string())
    }

    fn preview_restore(&self, request: RestorePathRequest) -> Result<RestorePlanSummary, String> {
        let (path, password, _) = self.profile_config(&request.profile_id)?;
        validate_restore_request(&request)?;
        let repo = open_repository(&path, &password)?
            .to_indexed()
            .map_err(display_error)?;
        let snapshots = repo.get_all_snapshots().map_err(display_error)?;
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.id.to_string() == request.snapshot_id)
            .ok_or_else(|| format!("snapshot {} was not found", request.snapshot_id))?;
        let node = repo
            .node_from_snapshot_and_path(snapshot, &request.path)
            .map_err(display_error)?;
        let destination = restore_destination(PathBuf::from(&request.destination).as_path(), &node);
        let destination = LocalDestination::new(&destination.to_string_lossy(), false, false)
            .map_err(display_error)?;
        let options = RestoreOptions::default();
        let plan = repo
            .prepare_restore(
                &options,
                repo.ls(&node, &LsOptions::default().recursive(true))
                    .map_err(display_error)?,
                &destination,
                true,
            )
            .map_err(display_error)?;
        Ok(restore_plan_summary(&plan))
    }

    fn restore_path(&self, request: RestorePathRequest) -> Result<RestoreResult, String> {
        validate_restore_request(&request)?;
        self.restore_to(
            &request.profile_id,
            &request.snapshot_id,
            &request.path,
            PathBuf::from(request.destination.trim()),
        )
    }

    fn preview_path(&self, request: PreviewPathRequest) -> Result<RestoreResult, String> {
        if request.path.trim().is_empty() {
            return Err("select a file or folder to preview".to_string());
        }
        let sequence = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(display_error)?
            .as_nanos();
        let destination = std::env::temp_dir()
            .join("wabou-rustic-preview")
            .join(format!("{}-{sequence}", std::process::id()));
        self.restore_to(
            &request.profile_id,
            &request.snapshot_id,
            &request.path,
            destination,
        )
    }

    fn restore_to(
        &self,
        profile_id: &str,
        snapshot_id: &str,
        path_in_snapshot: &str,
        destination_root: PathBuf,
    ) -> Result<RestoreResult, String> {
        let (path, password, _) = self.profile_config(profile_id)?;
        let repo = open_repository(&path, &password)?
            .to_indexed()
            .map_err(display_error)?;
        let snapshots = repo.get_all_snapshots().map_err(display_error)?;
        let snapshot = snapshots
            .iter()
            .find(|snapshot| snapshot.id.to_string() == snapshot_id)
            .ok_or_else(|| format!("snapshot {snapshot_id} was not found"))?;
        let node = repo
            .node_from_snapshot_and_path(snapshot, path_in_snapshot)
            .map_err(display_error)?;
        let destination_path = restore_destination(&destination_root, &node);
        let destination = LocalDestination::new(&destination_path.to_string_lossy(), true, false)
            .map_err(display_error)?;
        let options = RestoreOptions::default();
        let plan = repo
            .prepare_restore(
                &options,
                repo.ls(&node, &LsOptions::default().recursive(true))
                    .map_err(display_error)?,
                &destination,
                false,
            )
            .map_err(display_error)?;
        let summary = restore_plan_summary(&plan);
        repo.restore(
            plan,
            &options,
            repo.ls(&node, &LsOptions::default().recursive(true))
                .map_err(display_error)?,
            &destination,
        )
        .map_err(display_error)?;
        let restored_item = if node.is_dir() {
            destination_path
        } else {
            destination_path.join(node.name())
        };
        Ok(RestoreResult {
            destination: restored_item.to_string_lossy().into_owned(),
            plan: summary,
        })
    }
}

fn default_search_limit() -> usize {
    200
}

fn file_entry(path: PathBuf, node: &rustic_core::repofile::Node) -> FileEntry {
    FileEntry {
        name: node.name().to_string_lossy().into_owned(),
        path: path.to_string_lossy().into_owned(),
        kind: node_kind(node),
        size: node.meta.size,
        modified: node.meta.mtime.map(|time| time.to_string()),
    }
}

fn node_kind(node: &rustic_core::repofile::Node) -> String {
    if node.is_dir() {
        "directory"
    } else if node.is_file() {
        "file"
    } else if node.is_symlink() {
        "symlink"
    } else {
        "special"
    }
    .to_string()
}

fn diff_change(
    previous: Option<&rustic_core::repofile::Node>,
    current: Option<&rustic_core::repofile::Node>,
    include_metadata: bool,
) -> Option<&'static str> {
    match (previous, current) {
        (None, Some(_)) => Some("added"),
        (Some(_), None) => Some("removed"),
        (Some(previous), Some(current)) if previous.node_type != current.node_type => {
            Some("typeChanged")
        }
        (Some(previous), Some(current))
            if previous.is_file() && previous.content != current.content =>
        {
            Some("modified")
        }
        (Some(previous), Some(current))
            if previous.is_symlink()
                && previous.node_type.to_link() != current.node_type.to_link() =>
        {
            Some("modified")
        }
        (Some(previous), Some(current)) if include_metadata && previous.meta != current.meta => {
            Some("metadata")
        }
        _ => None,
    }
}

fn validate_restore_request(request: &RestorePathRequest) -> Result<(), String> {
    if request.path.trim().is_empty() {
        return Err("select a file or folder to extract".to_string());
    }
    let destination = PathBuf::from(request.destination.trim());
    if !destination.is_absolute() {
        return Err("restore destination must be an absolute path".to_string());
    }
    Ok(())
}

fn restore_destination(
    destination: &std::path::Path,
    node: &rustic_core::repofile::Node,
) -> PathBuf {
    if node.is_dir() {
        destination.join(node.name())
    } else {
        destination.to_path_buf()
    }
}

fn restore_plan_summary(plan: &rustic_core::RestorePlan) -> RestorePlanSummary {
    RestorePlanSummary {
        restore_size: plan.restore_size,
        matched_size: plan.matched_size,
        files_to_restore: plan.stats.files.restore,
        files_to_modify: plan.stats.files.modify,
        files_unchanged: plan.stats.files.unchanged + plan.stats.files.verified,
        directories_to_restore: plan.stats.dirs.restore,
        directories_to_modify: plan.stats.dirs.modify,
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
        parent_id: snapshot.parent.map(|id| id.to_string()),
        label: snapshot.label.clone(),
        description: snapshot.description.clone(),
        tags: snapshot.tags.iter().cloned().collect(),
        delete_protected: !matches!(&snapshot.delete, DeleteOption::NotSet),
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

    let files = service.clone();
    capability.method(LIST_FILES, move |request| {
        let service = files.clone();
        async move {
            tokio::task::spawn_blocking(move || service.list_files(request))
                .await
                .map_err(|error| format!("file listing task failed: {error}"))?
        }
    })?;

    let search = service.clone();
    capability.method(SEARCH_FILES, move |request| {
        let service = search.clone();
        async move {
            tokio::task::spawn_blocking(move || service.search_files(request))
                .await
                .map_err(|error| format!("file search task failed: {error}"))?
        }
    })?;

    let diff = service.clone();
    capability.method(DIFF_SNAPSHOTS, move |request| {
        let service = diff.clone();
        async move {
            tokio::task::spawn_blocking(move || service.diff_snapshots(request))
                .await
                .map_err(|error| format!("snapshot diff task failed: {error}"))?
        }
    })?;

    let update_snapshot = service.clone();
    capability.method(UPDATE_SNAPSHOT, move |request| {
        let service = update_snapshot.clone();
        async move {
            tokio::task::spawn_blocking(move || service.update_snapshot(request))
                .await
                .map_err(|error| format!("snapshot update task failed: {error}"))?
        }
    })?;

    let preview_restore = service.clone();
    capability.method(PREVIEW_RESTORE, move |request| {
        let service = preview_restore.clone();
        async move {
            tokio::task::spawn_blocking(move || service.preview_restore(request))
                .await
                .map_err(|error| format!("restore preview task failed: {error}"))?
        }
    })?;

    let restore = service.clone();
    capability.method(RESTORE_PATH, move |request| {
        let service = restore.clone();
        async move {
            tokio::task::spawn_blocking(move || service.restore_path(request))
                .await
                .map_err(|error| format!("restore task failed: {error}"))?
        }
    })?;

    capability.method(PREVIEW_PATH, move |request| {
        let service = service.clone();
        async move {
            tokio::task::spawn_blocking(move || service.preview_path(request))
                .await
                .map_err(|error| format!("file preview task failed: {error}"))?
        }
    })?;

    capability.method(OPEN_PATH, move |request: OpenPathRequest| async move {
        let path = PathBuf::from(request.path);
        if !path.exists() {
            return Err("path does not exist".to_string());
        }
        open::that_detached(path).map_err(display_error)
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
    fn local_repository_multiple_source_vertical_slice() {
        let root = std::env::var_os("WABOU_RUSTIC_TEST_ROOT").map_or_else(
            || tempfile::tempdir().expect("test root"),
            |parent| {
                tempfile::Builder::new()
                    .prefix("wabou-timestow-")
                    .tempdir_in(parent)
                    .expect("test root in configured directory")
            },
        );
        let repository = root.path().join("repository");
        let photos = root.path().join("photos");
        let documents = root.path().join("documents");
        let configs = root.path().join("configs");
        fs::create_dir_all(photos.join("chapter")).expect("photo directories");
        fs::create_dir_all(&documents).expect("document directory");
        fs::create_dir_all(configs.join("app")).expect("config directories");
        fs::write(photos.join("cover.txt"), "cover").expect("photo file");
        fs::write(photos.join("chapter/page.txt"), "page").expect("nested photo file");
        fs::write(documents.join("notes.md"), "backup notes").expect("document file");
        fs::write(configs.join("app/settings.toml"), "theme = 'light'").expect("config file");

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
                sources: vec![
                    photos.to_string_lossy().into_owned(),
                    documents.to_string_lossy().into_owned(),
                    configs.to_string_lossy().into_owned(),
                ],
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
        assert!(names.iter().any(|name| name == "notes.md"));
        assert!(names.iter().any(|name| name == "settings.toml"));

        let matches = service
            .search_files(SearchFilesRequest {
                profile_id: "photos".to_string(),
                snapshot_id: backup.snapshot.id.clone(),
                query: "SETTINGS".to_string(),
                limit: 20,
            })
            .expect("search snapshot");
        let settings = matches
            .iter()
            .find(|entry| entry.name == "settings.toml")
            .expect("settings search result");

        let restore_root = root.path().join("restored");
        let restore_request = RestorePathRequest {
            profile_id: "photos".to_string(),
            snapshot_id: backup.snapshot.id.clone(),
            path: settings.path.clone(),
            destination: restore_root.to_string_lossy().into_owned(),
        };
        let plan = service
            .preview_restore(restore_request.clone())
            .expect("preview restore");
        assert!(plan.files_to_restore >= 1);
        let restored = service.restore_path(restore_request).expect("restore file");
        assert_eq!(
            restored.destination,
            restore_root.join("settings.toml").to_string_lossy()
        );
        assert_eq!(
            fs::read_to_string(restore_root.join("settings.toml")).expect("restored settings"),
            "theme = 'light'"
        );

        let preview = service
            .preview_path(PreviewPathRequest {
                profile_id: "photos".to_string(),
                snapshot_id: backup.snapshot.id.clone(),
                path: settings.path.clone(),
            })
            .expect("restore temporary preview");
        let preview_path = PathBuf::from(&preview.destination);
        assert!(preview_path.starts_with(std::env::temp_dir().join("wabou-rustic-preview")));
        assert_eq!(
            fs::read_to_string(&preview_path).expect("preview settings"),
            "theme = 'light'"
        );

        fs::write(photos.join("cover.txt"), "updated cover").expect("modify photo file");
        fs::remove_file(documents.join("notes.md")).expect("remove document file");
        fs::write(documents.join("new.md"), "new document").expect("add document file");
        let next_backup = service
            .run_backup(ProfileIdRequest {
                profile_id: "photos".to_string(),
            })
            .expect("backup changed source");
        let updated_snapshot_id = next_backup.snapshot.id.clone();
        let diff = service
            .diff_snapshots(DiffSnapshotsRequest {
                profile_id: "photos".to_string(),
                base_snapshot_id: backup.snapshot.id,
                snapshot_id: updated_snapshot_id.clone(),
                path: String::new(),
                include_metadata: false,
            })
            .expect("compare snapshots");
        assert!(
            diff.entries
                .iter()
                .any(|entry| entry.name == "cover.txt" && entry.change == "modified")
        );
        assert!(
            diff.entries
                .iter()
                .any(|entry| entry.name == "notes.md" && entry.change == "removed")
        );
        assert!(
            diff.entries
                .iter()
                .any(|entry| entry.name == "new.md" && entry.change == "added")
        );
        assert_eq!(diff.summary.modified, 1);
        assert_eq!(diff.summary.removed, 1);
        assert_eq!(diff.summary.added, 1);

        let updated = service
            .update_snapshot(UpdateSnapshotRequest {
                profile_id: "photos".to_string(),
                snapshot_id: updated_snapshot_id.clone(),
                label: "After cleanup".to_string(),
                description: "Removed stale notes".to_string(),
                tags: vec!["release".to_string(), "docs".to_string()],
                delete_protected: true,
            })
            .expect("update snapshot metadata");
        assert_ne!(updated.id, updated_snapshot_id);
        assert_eq!(updated.label, "After cleanup");
        assert_eq!(updated.description.as_deref(), Some("Removed stale notes"));
        assert_eq!(updated.tags, ["docs", "release"]);
        assert!(updated.delete_protected);
        let updated_again = service
            .update_snapshot(UpdateSnapshotRequest {
                profile_id: "photos".to_string(),
                snapshot_id: updated.id.clone(),
                label: "Final archive".to_string(),
                description: String::new(),
                tags: vec!["archive".to_string()],
                delete_protected: false,
            })
            .expect("update snapshot metadata again");
        assert_ne!(updated_again.id, updated.id);
        assert_eq!(updated_again.label, "Final archive");
        assert_eq!(updated_again.description, None);
        assert_eq!(updated_again.tags, ["archive"]);
        assert!(!updated_again.delete_protected);
        let snapshots = service
            .list_snapshots(ProfileIdRequest {
                profile_id: "photos".to_string(),
            })
            .expect("reload updated snapshots");
        assert_eq!(snapshots.len(), 2);
        assert!(
            snapshots
                .iter()
                .any(|snapshot| snapshot.id == updated_again.id)
        );
        assert!(!snapshots.iter().any(|snapshot| snapshot.id == updated.id));
        assert!(
            !snapshots
                .iter()
                .any(|snapshot| snapshot.id == updated_snapshot_id)
        );
    }

    #[test]
    fn restore_rejects_relative_destinations_before_opening_the_repository() {
        let error = RusticService::default()
            .restore_path(RestorePathRequest {
                profile_id: "missing".to_string(),
                snapshot_id: "missing".to_string(),
                path: "file.txt".to_string(),
                destination: "relative".to_string(),
            })
            .expect_err("relative restore path");
        assert_eq!(error, "restore destination must be an absolute path");
    }
}
