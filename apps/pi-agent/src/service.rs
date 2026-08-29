use std::{
    collections::{BTreeSet, HashMap},
    env,
    io::{BufRead as _, BufReader, Write as _},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{Arc, Mutex},
};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use gix::bstr::ByteSlice as _;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wabou::{
    AppDirectories, AppDirectoryConfig, CapabilityContract, HostMessage, HostMessageContext,
    HostMethod, JsonMethod, NativeCapability, rquickjs,
};

pub const CAPABILITY: CapabilityContract = CapabilityContract::new("piAgent", 1);
const EVENT_TOPIC: &str = "pi.event";

const GET_STATUS: JsonMethod<AgentRequest, PiStatus> = JsonMethod::new("getStatus");
const START: JsonMethod<StartRequest, PiStatus> = JsonMethod::new("start");
const PROMPT: JsonMethod<PromptRequest, ()> = JsonMethod::new("prompt");
const STEER: JsonMethod<PromptRequest, ()> = JsonMethod::new("steer");
const FOLLOW_UP: JsonMethod<PromptRequest, ()> = JsonMethod::new("followUp");
const ABORT: JsonMethod<AgentRequest, ()> = JsonMethod::new("abort");
const STOP: JsonMethod<AgentRequest, ()> = JsonMethod::new("stop");
const NEW_SESSION: JsonMethod<AgentRequest, ()> = JsonMethod::new("newSession");
const RENAME_SESSION: JsonMethod<RenameSessionRequest, ()> = JsonMethod::new("renameSession");
const CYCLE_MODEL: JsonMethod<AgentRequest, ()> = JsonMethod::new("cycleModel");
const CYCLE_THINKING: JsonMethod<AgentRequest, ()> = JsonMethod::new("cycleThinking");
const SET_MODEL: JsonMethod<SetModelRequest, ()> = JsonMethod::new("setModel");
const GET_MODEL_OPTIONS: JsonMethod<AgentRequest, ()> = JsonMethod::new("getModelOptions");
const SET_THINKING: JsonMethod<SetThinkingRequest, ()> = JsonMethod::new("setThinking");
const SET_AUTO_COMPACTION: JsonMethod<ToggleRequest, ()> = JsonMethod::new("setAutoCompaction");
const SET_STEERING_MODE: JsonMethod<QueueModeRequest, ()> = JsonMethod::new("setSteeringMode");
const SET_FOLLOW_UP_MODE: JsonMethod<QueueModeRequest, ()> = JsonMethod::new("setFollowUpMode");
const LIST_SESSIONS: JsonMethod<AgentRequest, Vec<PiSession>> = JsonMethod::new("listSessions");
const GET_MESSAGES: JsonMethod<AgentRequest, ()> = JsonMethod::new("getMessages");
const GET_SESSION_STATS: JsonMethod<AgentRequest, ()> = JsonMethod::new("getSessionStats");
const GET_COMMANDS: JsonMethod<AgentRequest, ()> = JsonMethod::new("getCommands");
const GET_FORK_MESSAGES: JsonMethod<AgentRequest, ()> = JsonMethod::new("getForkMessages");
const FORK: JsonMethod<ForkRequest, ()> = JsonMethod::new("fork");
const CLONE_SESSION: JsonMethod<AgentRequest, ()> = JsonMethod::new("cloneSession");
const COMPACT_SESSION: JsonMethod<AgentRequest, ()> = JsonMethod::new("compactSession");
const EXPORT_SESSION: JsonMethod<ExportSessionRequest, ()> = JsonMethod::new("exportSession");
const LIST_WORKSPACE_FILES: HostMethod<WorkspaceFilesRequest, Vec<String>> =
    HostMethod::new("listWorkspaceFiles");
const WORKSPACE_INFO: HostMethod<WorkspaceFilesRequest, WorkspaceInfo> =
    HostMethod::new("workspaceInfo");
const READ_WORKSPACE_FILE: HostMethod<ReadWorkspaceFileRequest, WorkspaceFilePreview> =
    HostMethod::new("readWorkspaceFile");
const WORKSPACE_CHANGES: HostMethod<WorkspaceFilesRequest, WorkspaceChanges> =
    HostMethod::new("workspaceChanges");
const RESPOND_EXTENSION_UI: JsonMethod<ExtensionUiResponseRequest, ()> =
    JsonMethod::new("respondExtensionUi");
const LIST_AGENTS: JsonMethod<(), Vec<AgentProfile>> = JsonMethod::no_request("listAgents");
const SAVE_AGENTS: JsonMethod<Vec<AgentProfile>, ()> = JsonMethod::new("saveAgents");
const GET_APP_SETTINGS: JsonMethod<(), AppSettings> = JsonMethod::no_request("getAppSettings");
const SAVE_APP_SETTINGS: JsonMethod<AppSettings, ()> = JsonMethod::new("saveAppSettings");
const DELETE_AGENT: JsonMethod<AgentRequest, ()> = JsonMethod::new("deleteAgent");
const DEFAULT_WORKSPACE: JsonMethod<AgentRequest, String> = JsonMethod::new("defaultWorkspace");

#[derive(Clone)]
pub struct PiService {
    state: Arc<Mutex<HashMap<String, PiProcess>>>,
    events_tx: flume::Sender<Value>,
    events_rx: flume::Receiver<Value>,
    sessions: Arc<Mutex<SessionCatalog>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SessionCatalog {
    #[serde(default)]
    sessions: Vec<PiSession>,
    #[serde(default)]
    agents: Vec<AgentProfile>,
    #[serde(default)]
    settings: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    #[serde(default)]
    locale: AppLocale,
    #[serde(default)]
    proxy: String,
    #[serde(default = "default_no_proxy")]
    no_proxy: String,
    #[serde(default)]
    provider: String,
    #[serde(default)]
    model: String,
    #[serde(default = "default_enabled")]
    subagents_enabled: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum AppLocale {
    #[default]
    En,
    Zh,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            locale: AppLocale::En,
            proxy: String::new(),
            no_proxy: default_no_proxy(),
            provider: String::new(),
            model: String::new(),
            subagents_enabled: true,
        }
    }
}

fn default_no_proxy() -> String {
    "127.0.0.1,localhost".to_owned()
}

const fn default_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AgentProfile {
    id: String,
    name: String,
    cwd: String,
    provider: String,
    model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PiSession {
    agent_id: String,
    session_id: String,
    session_file: String,
    name: Option<String>,
    cwd: String,
    updated_at: u64,
}

struct PiProcess {
    child: Option<Child>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    cwd: Option<PathBuf>,
    last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PiStatus {
    running: bool,
    cwd: Option<String>,
    runtime: &'static str,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StartRequest {
    agent_id: String,
    cwd: Option<String>,
    proxy: Option<String>,
    no_proxy: Option<String>,
    provider: Option<String>,
    model: Option<String>,
    session_id: Option<String>,
    #[serde(default)]
    subagents_enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PromptRequest {
    agent_id: String,
    message: String,
    #[serde(default)]
    image_paths: Vec<PathBuf>,
    #[serde(default)]
    context_paths: Vec<PathBuf>,
}

const MAX_PROMPT_IMAGE_BYTES: u64 = 20 * 1024 * 1024;
const MAX_CONTEXT_FILES: usize = 8;
const MAX_CONTEXT_FILE_BYTES: u64 = 512 * 1024;
const MAX_CONTEXT_TOTAL_BYTES: usize = 2 * 1024 * 1024;

async fn prompt_images(paths: Vec<PathBuf>) -> Result<Vec<Value>, String> {
    let mut images = Vec::with_capacity(paths.len());
    for path in paths {
        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|error| format!("could not inspect {}: {error}", path.display()))?;
        if !metadata.is_file() {
            return Err(format!(
                "image attachment is not a file: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_PROMPT_IMAGE_BYTES {
            return Err(format!(
                "image attachment exceeds 20 MiB: {}",
                path.display()
            ));
        }
        let mime_type = match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("png") => "image/png",
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("webp") => "image/webp",
            Some("gif") => "image/gif",
            _ => return Err(format!("unsupported image attachment: {}", path.display())),
        };
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        images.push(json!({
            "type": "image",
            "data": BASE64.encode(bytes),
            "mimeType": mime_type,
        }));
    }
    Ok(images)
}

fn list_workspace_files(root: &std::path::Path) -> Result<Vec<String>, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not open workspace {}: {error}", root.display()))?;
    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false)
        .filter_entry(|entry| {
            !matches!(
                entry.file_name().to_str(),
                Some(".git" | "node_modules" | "target" | "dist")
            )
        });
    let mut files = builder
        .build()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .filter_map(|entry| {
            entry
                .path()
                .strip_prefix(&root)
                .ok()?
                .to_str()
                .map(str::to_owned)
        })
        .take(5_000)
        .collect::<Vec<_>>();
    files.sort_unstable();
    Ok(files)
}

fn read_workspace_file(root: &Path, relative: &Path) -> Result<WorkspaceFilePreview, String> {
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "file path must stay inside the workspace: {}",
            relative.display()
        ));
    }
    let root = root
        .canonicalize()
        .map_err(|error| format!("could not open workspace {}: {error}", root.display()))?;
    let path = root
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("could not open {}: {error}", relative.display()))?;
    if !path.starts_with(&root) {
        return Err(format!(
            "file path leaves the workspace: {}",
            relative.display()
        ));
    }
    let metadata = path
        .metadata()
        .map_err(|error| format!("could not inspect {}: {error}", relative.display()))?;
    if !metadata.is_file() || metadata.len() > MAX_CONTEXT_FILE_BYTES {
        return Err(format!(
            "preview requires a regular text file under 512 KiB: {}",
            relative.display()
        ));
    }
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("could not read {}: {error}", relative.display()))?;
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("file is not UTF-8 text: {}", relative.display()))?;
    Ok(WorkspaceFilePreview {
        path: relative.display().to_string(),
        text,
    })
}

async fn append_workspace_context(
    message: &str,
    root: &std::path::Path,
    paths: Vec<PathBuf>,
) -> Result<String, String> {
    if paths.is_empty() {
        return Ok(message.to_owned());
    }
    if paths.len() > MAX_CONTEXT_FILES {
        return Err(format!(
            "at most {MAX_CONTEXT_FILES} context files are allowed"
        ));
    }
    let root = tokio::fs::canonicalize(root)
        .await
        .map_err(|error| format!("could not open workspace {}: {error}", root.display()))?;
    let mut total = 0;
    let mut result = String::with_capacity(message.len() + 1024);
    result.push_str(message);
    result.push_str("\n\n<workspace_context>");
    for relative in paths {
        if relative.is_absolute()
            || relative.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir
                        | std::path::Component::RootDir
                        | std::path::Component::Prefix(_)
                )
            })
        {
            return Err(format!(
                "context path must stay inside the workspace: {}",
                relative.display()
            ));
        }
        let path = tokio::fs::canonicalize(root.join(&relative))
            .await
            .map_err(|error| format!("could not open {}: {error}", relative.display()))?;
        if !path.starts_with(&root) {
            return Err(format!(
                "context path leaves the workspace: {}",
                relative.display()
            ));
        }
        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|error| format!("could not inspect {}: {error}", relative.display()))?;
        if !metadata.is_file() || metadata.len() > MAX_CONTEXT_FILE_BYTES {
            return Err(format!(
                "context file is not a regular text file under 512 KiB: {}",
                relative.display()
            ));
        }
        let bytes = tokio::fs::read(&path)
            .await
            .map_err(|error| format!("could not read {}: {error}", relative.display()))?;
        total += bytes.len();
        if total > MAX_CONTEXT_TOTAL_BYTES {
            return Err("context files exceed the 2 MiB combined limit".to_owned());
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| format!("context file is not UTF-8 text: {}", relative.display()))?;
        result.push_str("\n<file path=");
        result.push_str(
            &serde_json::to_string(&relative.to_string_lossy())
                .map_err(|error| error.to_string())?,
        );
        result.push_str(">\n");
        result.push_str(&text);
        result.push_str("\n</file>");
    }
    result.push_str("\n</workspace_context>");
    Ok(result)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AgentRequest {
    agent_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SetModelRequest {
    agent_id: String,
    provider: String,
    model_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SetThinkingRequest {
    agent_id: String,
    level: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ToggleRequest {
    agent_id: String,
    enabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct QueueModeRequest {
    agent_id: String,
    mode: String,
}

fn validate_queue_mode(mode: &str) -> Result<(), String> {
    if matches!(mode, "all" | "one-at-a-time") {
        Ok(())
    } else {
        Err(format!("unsupported queue mode `{mode}`"))
    }
}

fn validate_thinking_level(level: &str) -> Result<(), String> {
    if matches!(
        level,
        "off" | "minimal" | "low" | "medium" | "high" | "xhigh" | "max"
    ) {
        Ok(())
    } else {
        Err(format!("unsupported thinking level `{level}`"))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RenameSessionRequest {
    agent_id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ForkRequest {
    agent_id: String,
    entry_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExportSessionRequest {
    agent_id: String,
    output_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkspaceFilesRequest {
    cwd: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReadWorkspaceFileRequest {
    cwd: PathBuf,
    path: PathBuf,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WorkspaceFilePreview {
    path: String,
    text: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WorkspaceChanges {
    files: Vec<WorkspaceFileChange>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WorkspaceFileChange {
    path: String,
    status: &'static str,
    additions: usize,
    deletions: usize,
    patch: String,
}

const MAX_DIFF_FILES: usize = 100;
const MAX_DIFF_FILE_BYTES: usize = 256 * 1024;
const MAX_DIFF_TOTAL_BYTES: usize = 2 * 1024 * 1024;

fn truncate_utf8(mut value: String, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value;
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value.truncate(end);
    value.push_str("\n… diff truncated …\n");
    value
}

fn repository_changed_paths(repository: &gix::Repository) -> Result<BTreeSet<PathBuf>, String> {
    let platform = repository
        .status(gix::progress::Discard)
        .map_err(|error| format!("could not inspect repository status: {error}"))?
        .untracked_files(gix::status::UntrackedFiles::Files);
    let changes = platform
        .into_iter(Vec::<gix::bstr::BString>::new())
        .map_err(|error| format!("could not prepare repository status: {error}"))?;
    let mut paths = BTreeSet::new();
    for change in changes {
        let change =
            change.map_err(|error| format!("could not read repository status: {error}"))?;
        let path = change
            .location()
            .to_str()
            .map_err(|_| "repository contains a changed path that is not UTF-8".to_owned())?;
        paths.insert(PathBuf::from(path));
    }
    Ok(paths)
}

fn head_file(repository: &gix::Repository, path: &Path) -> Result<Option<Vec<u8>>, String> {
    let Ok(tree_id) = repository.head_tree_id() else {
        return Ok(None);
    };
    let tree = repository
        .find_tree(tree_id)
        .map_err(|error| format!("could not read HEAD tree: {error}"))?;
    let Some(entry) = tree
        .lookup_entry_by_path(path)
        .map_err(|error| format!("could not find {} in HEAD: {error}", path.display()))?
    else {
        return Ok(None);
    };
    if !entry.mode().is_blob() {
        return Ok(None);
    }
    let object = entry
        .object()
        .map_err(|error| format!("could not read {} from HEAD: {error}", path.display()))?;
    let blob = object
        .try_into_blob()
        .map_err(|error| format!("{} in HEAD is not a blob: {error}", path.display()))?;
    Ok(Some(blob.data.clone()))
}

fn worktree_file(root: &Path, path: &Path) -> Result<Option<Vec<u8>>, String> {
    let absolute = root.join(path);
    if absolute.is_dir() {
        return Ok(None);
    }
    match std::fs::read(absolute) {
        Ok(data) => Ok(Some(data)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("could not read {}: {error}", path.display())),
    }
}

fn diff_file(path: &Path, old: Option<Vec<u8>>, new: Option<Vec<u8>>) -> WorkspaceFileChange {
    let status = match (&old, &new) {
        (None, Some(_)) => "added",
        (Some(_), None) => "deleted",
        _ => "modified",
    };
    let path = path.to_string_lossy().replace('\\', "/");
    let binary = old
        .as_deref()
        .into_iter()
        .chain(new.as_deref())
        .any(|data| std::str::from_utf8(data).is_err());
    let (additions, deletions, patch) = if binary {
        (0, 0, format!("Binary file {path} changed\n"))
    } else {
        let old = old
            .as_deref()
            .and_then(|data| std::str::from_utf8(data).ok())
            .unwrap_or_default();
        let new = new
            .as_deref()
            .and_then(|data| std::str::from_utf8(data).ok())
            .unwrap_or_default();
        let diff = similar::TextDiff::from_lines(old, new);
        let additions = diff
            .iter_all_changes()
            .filter(|change| change.tag() == similar::ChangeTag::Insert)
            .count();
        let deletions = diff
            .iter_all_changes()
            .filter(|change| change.tag() == similar::ChangeTag::Delete)
            .count();
        let old_name = if status == "added" {
            "/dev/null".to_owned()
        } else {
            format!("a/{path}")
        };
        let new_name = if status == "deleted" {
            "/dev/null".to_owned()
        } else {
            format!("b/{path}")
        };
        let body = diff
            .unified_diff()
            .context_radius(3)
            .header(&old_name, &new_name)
            .to_string();
        (
            additions,
            deletions,
            format!("diff --git a/{path} b/{path}\n{body}"),
        )
    };
    WorkspaceFileChange {
        path,
        status,
        additions,
        deletions,
        patch: truncate_utf8(patch, MAX_DIFF_FILE_BYTES),
    }
}

fn workspace_changes(cwd: &Path) -> Result<WorkspaceChanges, String> {
    let cwd = cwd
        .canonicalize()
        .map_err(|error| format!("could not open workspace {}: {error}", cwd.display()))?;
    let repository = gix::open(&cwd)
        .map_err(|error| format!("{} is not a Git repository: {error}", cwd.display()))?;
    let mut files = Vec::new();
    let mut total_patch_bytes = 0;
    for path in repository_changed_paths(&repository)? {
        if files.len() >= MAX_DIFF_FILES || total_patch_bytes >= MAX_DIFF_TOTAL_BYTES {
            break;
        }
        let old = head_file(&repository, &path)?;
        let new = worktree_file(&cwd, &path)?;
        if old.is_none() && new.is_none() {
            continue;
        }
        let change = diff_file(&path, old, new);
        total_patch_bytes += change.patch.len();
        files.push(change);
    }
    Ok(WorkspaceChanges { files })
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct WorkspaceInfo {
    repository: bool,
    branch: Option<String>,
    changed_files: usize,
}

fn workspace_info(cwd: &Path) -> WorkspaceInfo {
    let Ok(repository) = gix::open(cwd) else {
        return WorkspaceInfo {
            repository: false,
            branch: None,
            changed_files: 0,
        };
    };
    let branch = repository
        .head()
        .ok()
        .and_then(|head| head.referent_name().map(|name| name.shorten().to_string()));
    let changed_files = repository_changed_paths(&repository).map_or(0, |paths| paths.len());
    WorkspaceInfo {
        repository: true,
        branch,
        changed_files,
    }
}

#[cfg(test)]
fn parse_git_status(status: &str) -> WorkspaceInfo {
    let mut lines = status.lines();
    let branch = lines.next().and_then(|line| {
        line.strip_prefix("## ")
            .and_then(|value| {
                value
                    .split_once("...")
                    .map_or(Some(value), |(name, _)| Some(name))
            })
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    });
    WorkspaceInfo {
        repository: true,
        branch,
        changed_files: lines.filter(|line| !line.trim().is_empty()).count(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExtensionUiResponseRequest {
    agent_id: String,
    id: String,
    value: Option<String>,
    confirmed: Option<bool>,
    cancelled: Option<bool>,
}

fn extension_ui_response(request: ExtensionUiResponseRequest) -> (String, Value) {
    let has_value = request.value.is_some();
    let has_confirmation = request.confirmed.is_some();
    let mut response = serde_json::Map::from_iter([
        (
            "type".to_owned(),
            Value::String("extension_ui_response".to_owned()),
        ),
        ("id".to_owned(), Value::String(request.id)),
    ]);
    if let Some(value) = request.value {
        response.insert("value".to_owned(), Value::String(value));
    }
    if let Some(confirmed) = request.confirmed {
        response.insert("confirmed".to_owned(), Value::Bool(confirmed));
    }
    if request.cancelled == Some(true) || (!has_value && !has_confirmation) {
        response.insert("cancelled".to_owned(), Value::Bool(true));
    }
    (request.agent_id, Value::Object(response))
}

impl PiService {
    pub fn new() -> Self {
        let (events_tx, events_rx) = flume::bounded(1024);
        let sessions = load_session_catalog();
        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            events_tx,
            events_rx,
            sessions: Arc::new(Mutex::new(sessions)),
        }
    }

    fn status(&self, agent_id: &str) -> Result<PiStatus, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?;
        let running = state
            .get_mut(agent_id)
            .and_then(|process| process.child.as_mut())
            .is_some_and(|child| child.try_wait().ok().flatten().is_none());
        if !running {
            state.remove(agent_id);
        }
        let process = state.get(agent_id);
        Ok(PiStatus {
            running,
            cwd: process
                .and_then(|process| process.cwd.as_ref())
                .map(|path| path.display().to_string()),
            runtime: "bun",
            error: process.and_then(|process| process.last_error.clone()),
        })
    }

    fn workspace(&self, agent_id: &str) -> Result<PathBuf, String> {
        self.state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?
            .get(agent_id)
            .and_then(|process| process.cwd.clone())
            .ok_or_else(|| format!("agent `{agent_id}` is not running"))
    }

    fn start(&self, request: StartRequest) -> Result<PiStatus, String> {
        validate_agent_id(&request.agent_id)?;
        let session_file = request
            .session_id
            .as_deref()
            .filter(|session_id| !session_id.trim().is_empty())
            .map(|session_id| self.resolve_session_file(&request.agent_id, session_id))
            .transpose()?;
        self.stop(&request.agent_id)?;
        let cwd = request
            .cwd
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .unwrap_or(env::current_dir().map_err(|error| error.to_string())?);
        if cwd.exists() && !cwd.is_dir() {
            return Err(format!("workspace does not exist: {}", cwd.display()));
        }
        if !cwd.exists() {
            std::fs::create_dir_all(&cwd).map_err(|error| {
                format!("could not create workspace {}: {error}", cwd.display())
            })?;
        }
        let explicit_pi = env::var_os("WABOU_PI_BIN");
        let mut command = match explicit_pi {
            Some(executable) => {
                let mut command = Command::new(executable);
                command.args(["--mode", "rpc"]);
                command
            }
            None => {
                let mut command = Command::new("bun");
                command.args([
                    "x",
                    "--package",
                    "@earendil-works/pi-coding-agent@0.84.3",
                    "pi",
                    "--mode",
                    "rpc",
                ]);
                command
            }
        };
        configure_pi_command(&mut command, &request);
        if let Some(session_file) = session_file {
            command.arg("--session").arg(session_file);
        } else {
            command.arg("--continue");
        }
        command
            .current_dir(&cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
        }
        let mut child = command.spawn().map_err(|error| {
            format!("could not start Pi through Bun. Install Bun or set WABOU_PI_BIN: {error}")
        })?;
        let stdin = Arc::new(Mutex::new(
            child.stdin.take().ok_or("Pi stdin was not piped")?,
        ));
        let stdout = child.stdout.take().ok_or("Pi stdout was not piped")?;
        let stderr = child.stderr.take().ok_or("Pi stderr was not piped")?;

        let stdout_events = self.events_tx.clone();
        let sessions = self.sessions.clone();
        let settled_state_stdin = stdin.clone();
        let session_cwd = cwd.display().to_string();
        let stdout_agent_id = request.agent_id.clone();
        std::thread::Builder::new()
            .name("pi-rpc-stdout".to_owned())
            .spawn(move || {
                for line in BufReader::new(stdout).lines() {
                    match line {
                        Ok(line) if !line.is_empty() => match serde_json::from_str(&line) {
                            Ok(event) => {
                                let follow_up = follow_up_request(&event);
                                remember_session(&sessions, &stdout_agent_id, &session_cwd, &event);
                                let _ = stdout_events.send(tag_event(&stdout_agent_id, event));
                                if let Some(request) = follow_up
                                    && let Err(error) =
                                        write_rpc_request(&settled_state_stdin, &request)
                                {
                                    let _ = stdout_events.send(tag_event(
                                        &stdout_agent_id,
                                        json!({
                                            "type":"bridge_error",
                                            "message":format!(
                                                "could not refresh Pi state after the turn settled: {error}"
                                            )
                                        }),
                                    ));
                                }
                            }
                            Err(error) => {
                                let _ = stdout_events.send(tag_event(
                                    &stdout_agent_id,
                                    json!({
                                        "type":"bridge_error",
                                        "message":format!("invalid Pi RPC event: {error}")
                                    }),
                                ));
                            }
                        },
                        Ok(_) => {}
                        Err(error) => {
                            let _ = stdout_events.send(tag_event(
                                &stdout_agent_id,
                                json!({"type":"bridge_error","message":error.to_string()}),
                            ));
                            break;
                        }
                    }
                }
                let _ =
                    stdout_events.send(tag_event(&stdout_agent_id, json!({"type":"process_exit"})));
            })
            .map_err(|error| error.to_string())?;
        let stderr_events = self.events_tx.clone();
        let stderr_agent_id = request.agent_id.clone();
        std::thread::Builder::new()
            .name("pi-rpc-stderr".to_owned())
            .spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    let _ = stderr_events.send(tag_event(
                        &stderr_agent_id,
                        json!({"type":"process_log","message":line}),
                    ));
                }
            })
            .map_err(|error| error.to_string())?;

        let mut state = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?;
        state.insert(
            request.agent_id.clone(),
            PiProcess {
                child: Some(child),
                stdin: Some(stdin),
                cwd: Some(cwd),
                last_error: None,
            },
        );
        drop(state);
        self.events_tx
            .send(tag_event(
                &request.agent_id,
                json!({"type":"process_start"}),
            ))
            .map_err(|error| error.to_string())?;
        self.send(
            &request.agent_id,
            json!({"id":"wabou-bootstrap-state","type":"get_state"}),
        )?;
        self.status(&request.agent_id)
    }

    fn send(&self, agent_id: &str, value: Value) -> Result<(), String> {
        let stdin = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?
            .get(agent_id)
            .and_then(|process| process.stdin.clone())
            .ok_or_else(|| format!("Pi agent `{agent_id}` is not running"))?;
        write_rpc_request(&stdin, &value)
    }

    fn stop(&self, agent_id: &str) -> Result<(), String> {
        let process = self
            .state
            .lock()
            .map_err(|_| "Pi process lock poisoned".to_owned())?
            .remove(agent_id);
        if let Some(mut child) = process.and_then(|mut process| process.child.take()) {
            child.kill().map_err(|error| error.to_string())?;
            let _ = child.wait();
        }
        Ok(())
    }

    fn sessions(&self, agent_id: &str) -> Result<Vec<PiSession>, String> {
        let mut catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        if prune_missing_sessions(&mut catalog) {
            persist_catalog(&catalog)?;
        }
        let mut sessions = catalog
            .sessions
            .iter()
            .filter(|session| session.agent_id == agent_id)
            .cloned()
            .collect::<Vec<_>>();
        sessions.sort_by_key(|session| std::cmp::Reverse(session.updated_at));
        Ok(sessions)
    }

    fn resolve_session_file(&self, agent_id: &str, session_id: &str) -> Result<PathBuf, String> {
        let mut catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        if prune_missing_sessions(&mut catalog) {
            persist_catalog(&catalog)?;
        }
        catalog_session_file(&catalog, agent_id, session_id)
            .ok_or_else(|| format!("saved session `{session_id}` is no longer available"))
    }

    fn agents(&self) -> Result<Vec<AgentProfile>, String> {
        self.sessions
            .lock()
            .map(|catalog| catalog.agents.clone())
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())
    }

    fn save_agents(&self, agents: Vec<AgentProfile>) -> Result<(), String> {
        validate_agent_profiles(&agents)?;
        let mut catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        catalog.agents = agents;
        persist_catalog(&catalog)
    }

    fn app_settings(&self) -> Result<AppSettings, String> {
        self.sessions
            .lock()
            .map(|catalog| catalog.settings.clone())
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())
    }

    fn save_app_settings(&self, settings: AppSettings) -> Result<(), String> {
        let mut catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        catalog.settings = settings;
        persist_catalog(&catalog)
    }

    fn delete_agent(&self, agent_id: &str) -> Result<(), String> {
        validate_agent_id(agent_id)?;
        self.stop(agent_id)?;
        let mut catalog = self
            .sessions
            .lock()
            .map_err(|_| "Pi session catalog lock poisoned".to_owned())?;
        catalog.agents.retain(|agent| agent.id != agent_id);
        catalog
            .sessions
            .retain(|session| session.agent_id != agent_id);
        persist_catalog(&catalog)
    }
}

fn write_rpc_request(stdin: &Arc<Mutex<ChildStdin>>, value: &Value) -> Result<(), String> {
    let mut stdin = stdin.lock().map_err(|_| "Pi stdin lock poisoned")?;
    serde_json::to_writer(&mut *stdin, value).map_err(|error| error.to_string())?;
    stdin.write_all(b"\n").map_err(|error| error.to_string())?;
    stdin.flush().map_err(|error| error.to_string())
}

fn follow_up_request(event: &Value) -> Option<Value> {
    (event.get("type").and_then(Value::as_str) == Some("agent_settled")).then(|| {
        json!({
            "id": "wabou-settled-state",
            "type": "get_state"
        })
    })
}

fn configure_pi_command(command: &mut Command, request: &StartRequest) {
    if request.subagents_enabled {
        command.args(["--extension", "npm:pi-subagents@0.58.0"]);
    }
    if let Some(provider) = request
        .provider
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        command.args(["--provider", provider]);
    }
    if let Some(model) = request
        .model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        command.args(["--model", model]);
    }
    if let Some(proxy) = request
        .proxy
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        for name in [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
        ] {
            command.env(name, proxy);
        }
    }
    if let Some(no_proxy) = request
        .no_proxy
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        command.env("NO_PROXY", no_proxy).env("no_proxy", no_proxy);
    }
}

fn validate_agent_profiles(agents: &[AgentProfile]) -> Result<(), String> {
    if agents.len() > 32 {
        return Err("at most 32 agent workspaces may be saved".to_owned());
    }
    let mut ids = std::collections::HashSet::with_capacity(agents.len());
    for agent in agents {
        validate_agent_id(&agent.id)?;
        if agent.name.trim().is_empty() {
            return Err(format!("agent `{}` must have a name", agent.id));
        }
        if !ids.insert(agent.id.as_str()) {
            return Err(format!("duplicate agent id `{}`", agent.id));
        }
    }
    Ok(())
}

fn session_catalog_path() -> Option<PathBuf> {
    AppDirectories::resolve(&AppDirectoryConfig::new("dev", "Wabou", "Pi Agent"), ".")
        .map(|dirs| dirs.local_data_dir.join("sessions.json"))
}

fn load_session_catalog() -> SessionCatalog {
    session_catalog_path()
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn prune_missing_sessions(catalog: &mut SessionCatalog) -> bool {
    let previous_len = catalog.sessions.len();
    catalog
        .sessions
        .retain(|session| Path::new(&session.session_file).is_file());
    catalog.sessions.len() != previous_len
}

fn catalog_session_file(
    catalog: &SessionCatalog,
    agent_id: &str,
    session_id: &str,
) -> Option<PathBuf> {
    catalog
        .sessions
        .iter()
        .find(|session| session.agent_id == agent_id && session.session_id == session_id)
        .map(|session| PathBuf::from(&session.session_file))
}

fn persist_catalog(catalog: &SessionCatalog) -> Result<(), String> {
    let Some(path) = session_catalog_path() else {
        return Err("could not resolve Pi Agent data directory".to_owned());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(catalog).map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(|error| error.to_string())
}

fn remember_session(
    sessions: &Arc<Mutex<SessionCatalog>>,
    agent_id: &str,
    cwd: &str,
    event: &Value,
) {
    if event.get("type").and_then(Value::as_str) != Some("response")
        || event.get("command").and_then(Value::as_str) != Some("get_state")
        || event.get("success").and_then(Value::as_bool) != Some(true)
    {
        return;
    }
    let Some(data) = event.get("data") else {
        return;
    };
    let (Some(session_id), Some(session_file)) = (
        data.get("sessionId").and_then(Value::as_str),
        data.get("sessionFile").and_then(Value::as_str),
    ) else {
        return;
    };
    let updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    let Ok(mut catalog) = sessions.lock() else {
        return;
    };
    let entry = PiSession {
        agent_id: agent_id.to_owned(),
        session_id: session_id.to_owned(),
        session_file: session_file.to_owned(),
        name: data
            .get("sessionName")
            .and_then(Value::as_str)
            .map(str::to_owned),
        cwd: cwd.to_owned(),
        updated_at,
    };
    upsert_session(&mut catalog, entry);
    let _ = persist_catalog(&catalog);
}

fn upsert_session(catalog: &mut SessionCatalog, entry: PiSession) {
    if let Some(existing) = catalog.sessions.iter_mut().find(|existing| {
        existing.agent_id == entry.agent_id && existing.session_id == entry.session_id
    }) {
        *existing = entry;
    } else {
        catalog.sessions.push(entry);
    }
}

fn validate_agent_id(agent_id: &str) -> Result<(), String> {
    if agent_id.is_empty()
        || !agent_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("agentId must contain only letters, numbers, '-' or '_'".to_owned());
    }
    Ok(())
}

fn default_workspace(agent_id: &str) -> Result<String, String> {
    validate_agent_id(agent_id)?;
    let root = if std::env::var_os("WABOU_TEST_APP_DATA_ROOT").is_some() {
        AppDirectories::resolve(&AppDirectoryConfig::new("dev", "Wabou", "Pi Agent"), ".")
            .ok_or_else(|| "could not resolve isolated Pi Agent data directory".to_owned())?
            .data_dir
            .join("workspaces")
    } else {
        let user = directories::UserDirs::new()
            .ok_or_else(|| "could not resolve the user home directory".to_owned())?;
        user.document_dir()
            .unwrap_or_else(|| user.home_dir())
            .join("pi-agent")
    };
    let workspace = root.join(agent_id);
    std::fs::create_dir_all(&workspace).map_err(|error| {
        format!(
            "could not create default workspace {}: {error}",
            workspace.display()
        )
    })?;
    Ok(workspace.display().to_string())
}

fn tag_event(agent_id: &str, mut event: Value) -> Value {
    if let Some(object) = event.as_object_mut() {
        object.insert("agentId".to_owned(), Value::String(agent_id.to_owned()));
    }
    event
}

impl Drop for PiProcess {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn mount(capability: NativeCapability<'_>, service: PiService) -> rquickjs::Result<()> {
    capability.method(
        WORKSPACE_INFO,
        move |request: WorkspaceFilesRequest| async move {
            tokio::task::spawn_blocking(move || workspace_info(&request.cwd))
                .await
                .map_err(|error| format!("workspace status task failed: {error}"))
        },
    )?;
    capability.method(
        READ_WORKSPACE_FILE,
        move |request: ReadWorkspaceFileRequest| async move {
            tokio::task::spawn_blocking(move || read_workspace_file(&request.cwd, &request.path))
                .await
                .map_err(|error| format!("workspace file task failed: {error}"))?
        },
    )?;
    capability.method(
        WORKSPACE_CHANGES,
        move |request: WorkspaceFilesRequest| async move {
            tokio::task::spawn_blocking(move || workspace_changes(&request.cwd))
                .await
                .map_err(|error| format!("workspace changes task failed: {error}"))?
        },
    )?;
    capability.method(
        LIST_WORKSPACE_FILES,
        move |request: WorkspaceFilesRequest| async move {
            tokio::task::spawn_blocking(move || list_workspace_files(&request.cwd))
                .await
                .map_err(|error| format!("workspace scan task failed: {error}"))?
        },
    )?;
    let extension_ui = service.clone();
    capability.json_method(
        RESPOND_EXTENSION_UI,
        move |request: ExtensionUiResponseRequest| {
            let service = extension_ui.clone();
            async move {
                let (agent_id, response) = extension_ui_response(request);
                service.send(&agent_id, response)
            }
        },
    )?;
    capability.json_method(DEFAULT_WORKSPACE, |request: AgentRequest| async move {
        default_workspace(&request.agent_id)
    })?;
    let list_agents = service.clone();
    capability.json_method(LIST_AGENTS, move |(): ()| {
        let service = list_agents.clone();
        async move { service.agents() }
    })?;
    let save_agents = service.clone();
    capability.json_method(SAVE_AGENTS, move |agents: Vec<AgentProfile>| {
        let service = save_agents.clone();
        async move { service.save_agents(agents) }
    })?;
    let get_app_settings = service.clone();
    capability.json_method(GET_APP_SETTINGS, move |(): ()| {
        let service = get_app_settings.clone();
        async move { service.app_settings() }
    })?;
    let save_app_settings = service.clone();
    capability.json_method(SAVE_APP_SETTINGS, move |settings: AppSettings| {
        let service = save_app_settings.clone();
        async move { service.save_app_settings(settings) }
    })?;
    let delete_agent = service.clone();
    capability.json_method(DELETE_AGENT, move |request: AgentRequest| {
        let service = delete_agent.clone();
        async move { service.delete_agent(&request.agent_id) }
    })?;
    let status = service.clone();
    capability.json_method(GET_STATUS, move |request: AgentRequest| {
        let service = status.clone();
        async move { service.status(&request.agent_id) }
    })?;
    let start = service.clone();
    capability.json_method(START, move |request: StartRequest| {
        let service = start.clone();
        async move { service.start(request) }
    })?;
    let list_sessions = service.clone();
    capability.json_method(LIST_SESSIONS, move |request: AgentRequest| {
        let service = list_sessions.clone();
        async move { service.sessions(&request.agent_id) }
    })?;
    let get_messages = service.clone();
    capability.json_method(GET_MESSAGES, move |request: AgentRequest| {
        let service = get_messages.clone();
        async move { service.send(&request.agent_id, json!({"type":"get_messages"})) }
    })?;
    let get_session_stats = service.clone();
    capability.json_method(GET_SESSION_STATS, move |request: AgentRequest| {
        let service = get_session_stats.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"id":"wabou-session-stats","type":"get_session_stats"}),
            )
        }
    })?;
    let get_commands = service.clone();
    capability.json_method(GET_COMMANDS, move |request: AgentRequest| {
        let service = get_commands.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"id":"wabou-commands","type":"get_commands"}),
            )
        }
    })?;
    let get_fork_messages = service.clone();
    capability.json_method(GET_FORK_MESSAGES, move |request: AgentRequest| {
        let service = get_fork_messages.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"id":"wabou-fork-messages","type":"get_fork_messages"}),
            )
        }
    })?;
    let fork = service.clone();
    capability.json_method(FORK, move |request: ForkRequest| {
        let service = fork.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"id":"wabou-fork","type":"fork","entryId":request.entry_id}),
            )
        }
    })?;
    let clone_session = service.clone();
    capability.json_method(CLONE_SESSION, move |request: AgentRequest| {
        let service = clone_session.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"id":"wabou-clone","type":"clone"}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-clone-state","type":"get_state"}),
            )
        }
    })?;
    let compact_session = service.clone();
    capability.json_method(COMPACT_SESSION, move |request: AgentRequest| {
        let service = compact_session.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"id":"wabou-compact","type":"compact"}),
            )
        }
    })?;
    let export_session = service.clone();
    capability.json_method(EXPORT_SESSION, move |request: ExportSessionRequest| {
        let service = export_session.clone();
        async move {
            let output_path = request.output_path.trim();
            if output_path.is_empty() {
                return Err("export path cannot be empty".to_owned());
            }
            service.send(
                &request.agent_id,
                json!({"id":"wabou-export","type":"export_html","outputPath":output_path}),
            )
        }
    })?;
    let prompt = service.clone();
    capability.json_method(PROMPT, move |request: PromptRequest| {
        let service = prompt.clone();
        async move {
            let message = request.message.trim().to_owned();
            if message.is_empty() {
                return Err("prompt cannot be empty".to_owned());
            }
            let images = prompt_images(request.image_paths).await?;
            let workspace = service.workspace(&request.agent_id)?;
            let message =
                append_workspace_context(&message, &workspace, request.context_paths).await?;
            service.send(
                &request.agent_id,
                json!({
                    "type":"prompt",
                    "message":message,
                    "images":images,
                }),
            )
        }
    })?;
    let steer = service.clone();
    capability.json_method(STEER, move |request: PromptRequest| {
        let service = steer.clone();
        async move {
            let message = request.message.trim().to_owned();
            if message.is_empty() {
                return Err("steering message cannot be empty".to_owned());
            }
            let images = prompt_images(request.image_paths).await?;
            let workspace = service.workspace(&request.agent_id)?;
            let message =
                append_workspace_context(&message, &workspace, request.context_paths).await?;
            service.send(
                &request.agent_id,
                json!({
                    "type":"steer",
                    "message":message,
                    "images":images,
                }),
            )
        }
    })?;
    let follow_up = service.clone();
    capability.json_method(FOLLOW_UP, move |request: PromptRequest| {
        let service = follow_up.clone();
        async move {
            let message = request.message.trim().to_owned();
            if message.is_empty() {
                return Err("follow-up cannot be empty".to_owned());
            }
            let images = prompt_images(request.image_paths).await?;
            let workspace = service.workspace(&request.agent_id)?;
            let message =
                append_workspace_context(&message, &workspace, request.context_paths).await?;
            service.send(
                &request.agent_id,
                json!({
                    "type":"follow_up",
                    "message":message,
                    "images":images,
                }),
            )
        }
    })?;
    let abort = service.clone();
    capability.json_method(ABORT, move |request: AgentRequest| {
        let service = abort.clone();
        async move { service.send(&request.agent_id, json!({"type":"abort"})) }
    })?;
    let stop = service.clone();
    capability.json_method(STOP, move |request: AgentRequest| {
        let service = stop.clone();
        async move { service.stop(&request.agent_id) }
    })?;
    let new_session = service.clone();
    capability.json_method(NEW_SESSION, move |request: AgentRequest| {
        let service = new_session.clone();
        async move {
            service.send(&request.agent_id, json!({"type":"new_session"}))?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-new-session-state","type":"get_state"}),
            )
        }
    })?;
    let rename_session = service.clone();
    capability.json_method(RENAME_SESSION, move |request: RenameSessionRequest| {
        let service = rename_session.clone();
        async move {
            let name = request.name.trim();
            if name.is_empty() {
                return Err("session name cannot be empty".to_owned());
            }
            service.send(
                &request.agent_id,
                json!({"type":"set_session_name","name":name}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-renamed-session-state","type":"get_state"}),
            )
        }
    })?;
    let cycle_model = service.clone();
    capability.json_method(CYCLE_MODEL, move |request: AgentRequest| {
        let service = cycle_model.clone();
        async move { service.send(&request.agent_id, json!({"type":"cycle_model"})) }
    })?;
    let cycle_thinking = service.clone();
    capability.json_method(CYCLE_THINKING, move |request: AgentRequest| {
        let service = cycle_thinking.clone();
        async move { service.send(&request.agent_id, json!({"type":"cycle_thinking_level"})) }
    })?;
    let model_options = service.clone();
    capability.json_method(GET_MODEL_OPTIONS, move |request: AgentRequest| {
        let service = model_options.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"id":"wabou-models","type":"get_available_models"}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-thinking-levels","type":"get_available_thinking_levels"}),
            )
        }
    })?;
    let set_thinking = service.clone();
    capability.json_method(SET_THINKING, move |request: SetThinkingRequest| {
        let service = set_thinking.clone();
        async move {
            validate_thinking_level(&request.level)?;
            service.send(
                &request.agent_id,
                json!({"type":"set_thinking_level","level":request.level}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-thinking-state","type":"get_state"}),
            )
        }
    })?;
    let auto_compaction = service.clone();
    capability.json_method(SET_AUTO_COMPACTION, move |request: ToggleRequest| {
        let service = auto_compaction.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"type":"set_auto_compaction","enabled":request.enabled}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-auto-compaction-state","type":"get_state"}),
            )
        }
    })?;
    let steering_mode = service.clone();
    capability.json_method(SET_STEERING_MODE, move |request: QueueModeRequest| {
        let service = steering_mode.clone();
        async move {
            validate_queue_mode(&request.mode)?;
            service.send(
                &request.agent_id,
                json!({"type":"set_steering_mode","mode":request.mode}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-steering-mode-state","type":"get_state"}),
            )
        }
    })?;
    let follow_up_mode = service.clone();
    capability.json_method(SET_FOLLOW_UP_MODE, move |request: QueueModeRequest| {
        let service = follow_up_mode.clone();
        async move {
            validate_queue_mode(&request.mode)?;
            service.send(
                &request.agent_id,
                json!({"type":"set_follow_up_mode","mode":request.mode}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-follow-up-mode-state","type":"get_state"}),
            )
        }
    })?;
    capability.json_method(SET_MODEL, move |request: SetModelRequest| {
        let service = service.clone();
        async move {
            service.send(
                &request.agent_id,
                json!({"type":"set_model","provider":request.provider,"modelId":request.model_id}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-model-state","type":"get_state"}),
            )?;
            service.send(
                &request.agent_id,
                json!({"id":"wabou-thinking-levels","type":"get_available_thinking_levels"}),
            )
        }
    })
}

fn drain_event_batch(first: Value, receiver: &flume::Receiver<Value>) -> Vec<Value> {
    let mut events = Vec::with_capacity(32);
    events.push(first);
    while events.len() < 64 {
        match receiver.try_recv() {
            Ok(event) => events.push(event),
            Err(_) => break,
        }
    }
    events
}

pub fn stream_events(context: HostMessageContext, service: PiService) {
    let producer = context.clone();
    context.spawn(async move {
        loop {
            tokio::select! {
                () = producer.cancelled() => break,
                event = service.events_rx.recv_async() => match event {
                    Ok(event) => {
                        // Pi can emit a JSONL record for every streamed token. Give the reader a
                        // short coalescing window, then deliver one JS update for the whole batch.
                        tokio::time::sleep(std::time::Duration::from_millis(8)).await;
                        let events = drain_event_batch(event, &service.events_rx);
                        let payload = match serde_json::to_string(&events) {
                            Ok(payload) => payload,
                            Err(error) => {
                                tracing::warn!(?error, "could not encode Pi event batch");
                                continue;
                            }
                        };
                        if producer.messages().send_async(HostMessage::str(EVENT_TOPIC, payload)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "wabou-pi-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ))
    }

    #[test]
    fn status_is_stopped_before_start() {
        let status = PiService::new().status("default").expect("status");
        assert!(!status.running);
        assert_eq!(status.runtime, "bun");
    }

    #[test]
    fn thinking_levels_are_restricted_to_pi_rpc_values() {
        for level in ["off", "minimal", "low", "medium", "high", "xhigh", "max"] {
            assert!(validate_thinking_level(level).is_ok());
        }
        assert_eq!(
            validate_thinking_level("ultra").unwrap_err(),
            "unsupported thinking level `ultra`"
        );
    }

    #[test]
    fn queue_modes_are_restricted_to_pi_rpc_values() {
        assert!(validate_queue_mode("all").is_ok());
        assert!(validate_queue_mode("one-at-a-time").is_ok());
        assert!(validate_queue_mode("parallel").is_err());
    }

    #[test]
    fn prompt_request_uses_the_javascript_camel_case_contract() {
        let request: PromptRequest = serde_json::from_value(json!({
            "agentId": "agent-1",
            "message": "inspect these files",
            "imagePaths": ["page.png"],
            "contextPaths": ["src/main.rs"]
        }))
        .expect("camel-case prompt request");

        assert_eq!(request.agent_id, "agent-1");
        assert_eq!(request.image_paths, vec![PathBuf::from("page.png")]);
        assert_eq!(request.context_paths, vec![PathBuf::from("src/main.rs")]);
    }

    #[test]
    fn pi_command_receives_runtime_configuration() {
        let mut request = StartRequest {
            agent_id: "agent-1".to_owned(),
            cwd: None,
            proxy: Some("  http://127.0.0.1:7890  ".to_owned()),
            no_proxy: Some(" localhost,127.0.0.1 ".to_owned()),
            provider: Some(" openai ".to_owned()),
            model: Some(" gpt-5 ".to_owned()),
            session_id: None,
            subagents_enabled: true,
        };
        let mut command = Command::new("pi");
        configure_pi_command(&mut command, &request);

        let args = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            args,
            [
                "--extension",
                "npm:pi-subagents@0.58.0",
                "--provider",
                "openai",
                "--model",
                "gpt-5"
            ]
        );

        let environment = command
            .get_envs()
            .map(|(name, value)| {
                (
                    name.to_string_lossy().into_owned(),
                    value
                        .expect("configured environment value")
                        .to_string_lossy()
                        .into_owned(),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        for name in [
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "ALL_PROXY",
            "http_proxy",
            "https_proxy",
            "all_proxy",
        ] {
            assert_eq!(
                environment.get(name).map(String::as_str),
                Some("http://127.0.0.1:7890")
            );
        }
        for name in ["NO_PROXY", "no_proxy"] {
            assert_eq!(
                environment.get(name).map(String::as_str),
                Some("localhost,127.0.0.1")
            );
        }

        request.subagents_enabled = false;
        let mut command = Command::new("pi");
        configure_pi_command(&mut command, &request);
        let args = command
            .get_args()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(args, ["--provider", "openai", "--model", "gpt-5"]);
    }

    #[test]
    fn event_batch_is_bounded_and_leaves_backpressure_in_the_channel() {
        let (sender, receiver) = flume::bounded(128);
        for index in 1..=70 {
            sender.send(json!({"index": index})).expect("queued event");
        }
        let first = receiver.recv().expect("first event");
        let batch = drain_event_batch(first, &receiver);
        assert_eq!(batch.len(), 64);
        assert_eq!(receiver.len(), 6);
    }

    #[test]
    fn tags_events_with_their_agent_identity() {
        assert_eq!(
            tag_event("agent-2", json!({"type":"agent_start"}))["agentId"],
            "agent-2"
        );
    }

    #[test]
    fn settled_turns_request_fresh_session_identity() {
        assert_eq!(
            follow_up_request(&json!({"type":"agent_settled"})),
            Some(json!({
                "id": "wabou-settled-state",
                "type": "get_state"
            }))
        );
        assert_eq!(follow_up_request(&json!({"type":"agent_end"})), None);
    }

    #[test]
    fn extension_ui_responses_preserve_value_confirmation_and_cancellation() {
        let (_, value) = extension_ui_response(ExtensionUiResponseRequest {
            agent_id: "agent-1".to_owned(),
            id: "input-1".to_owned(),
            value: Some("typed value".to_owned()),
            confirmed: None,
            cancelled: None,
        });
        assert_eq!(
            value,
            json!({"type":"extension_ui_response","id":"input-1","value":"typed value"})
        );

        let (_, confirmation) = extension_ui_response(ExtensionUiResponseRequest {
            agent_id: "agent-1".to_owned(),
            id: "confirm-1".to_owned(),
            value: None,
            confirmed: Some(false),
            cancelled: None,
        });
        assert_eq!(
            confirmation,
            json!({"type":"extension_ui_response","id":"confirm-1","confirmed":false})
        );

        let (agent_id, cancelled) = extension_ui_response(ExtensionUiResponseRequest {
            agent_id: "agent-2".to_owned(),
            id: "select-1".to_owned(),
            value: None,
            confirmed: None,
            cancelled: None,
        });
        assert_eq!(agent_id, "agent-2");
        assert_eq!(
            cancelled,
            json!({"type":"extension_ui_response","id":"select-1","cancelled":true})
        );
    }

    #[test]
    fn older_session_catalogs_default_to_no_saved_agents() {
        let catalog: SessionCatalog =
            serde_json::from_str(r#"{"sessions":[]}"#).expect("legacy catalog");
        assert!(catalog.agents.is_empty());
        assert_eq!(catalog.settings, AppSettings::default());
    }

    #[test]
    fn legacy_project_proxy_fields_are_ignored_in_favor_of_app_settings() {
        let catalog: SessionCatalog = serde_json::from_str(
            r#"{
                "sessions": [],
                "agents": [{
                    "id": "agent-1",
                    "name": "Agent 1",
                    "cwd": "",
                    "proxy": "http://old-project-proxy:7890",
                    "noProxy": "localhost",
                    "provider": "",
                    "model": ""
                }],
                "settings": {
                    "proxy": "http://global-proxy:7890",
                    "noProxy": "localhost",
                    "provider": "openai",
                    "model": "gpt-5",
                    "subagentsEnabled": true
                }
            }"#,
        )
        .expect("legacy project fields remain readable");
        assert_eq!(catalog.settings.proxy, "http://global-proxy:7890");
        assert_eq!(catalog.settings.locale, AppLocale::En);
        assert!(catalog.settings.subagents_enabled);
        assert_eq!(catalog.agents.len(), 1);
    }

    #[test]
    fn application_locale_round_trips_and_legacy_catalogs_default_to_english() {
        let legacy: SessionCatalog =
            serde_json::from_str(r#"{"settings":{}}"#).expect("legacy catalog");
        assert_eq!(legacy.settings.locale, AppLocale::En);

        let mut catalog = SessionCatalog::default();
        catalog.settings.locale = AppLocale::Zh;
        let encoded = serde_json::to_string(&catalog).expect("encode catalog");
        let decoded: SessionCatalog = serde_json::from_str(&encoded).expect("decode catalog");
        assert_eq!(decoded.settings.locale, AppLocale::Zh);
    }

    #[test]
    fn saved_agent_profiles_require_unique_stable_ids() {
        let profile = AgentProfile {
            id: "agent-1".to_owned(),
            name: "Agent 1".to_owned(),
            cwd: String::new(),
            provider: String::new(),
            model: String::new(),
        };
        assert!(validate_agent_profiles(std::slice::from_ref(&profile)).is_ok());
        assert_eq!(
            validate_agent_profiles(&[profile.clone(), profile]).unwrap_err(),
            "duplicate agent id `agent-1`"
        );
    }

    #[test]
    fn default_workspace_is_scoped_to_a_valid_agent_directory() {
        let workspace = PathBuf::from(default_workspace("agent-2").expect("workspace"));
        assert_eq!(
            workspace.file_name().and_then(|name| name.to_str()),
            Some("agent-2")
        );
        assert_eq!(
            workspace
                .parent()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str()),
            Some("pi-agent")
        );
        assert!(default_workspace("../escape").is_err());
    }

    #[test]
    fn git_status_summary_keeps_branch_and_change_count() {
        assert_eq!(
            parse_git_status(
                "## feat/ui...origin/feat/ui [ahead 2]\n M src/main.rs\n?? notes.md\n"
            ),
            WorkspaceInfo {
                repository: true,
                branch: Some("feat/ui".to_owned()),
                changed_files: 2,
            }
        );
        assert_eq!(
            parse_git_status("## main\n"),
            WorkspaceInfo {
                repository: true,
                branch: Some("main".to_owned()),
                changed_files: 0,
            }
        );
    }

    #[test]
    fn workspace_file_preview_is_text_only_and_cannot_escape_the_workspace() {
        let directory = test_directory("workspace-preview");
        std::fs::create_dir_all(directory.join("src")).expect("fixture directory");
        std::fs::write(directory.join("src/main.rs"), "fn main() {}\n").expect("text fixture");

        assert_eq!(
            read_workspace_file(&directory, Path::new("src/main.rs")).expect("workspace file"),
            WorkspaceFilePreview {
                path: "src/main.rs".to_owned(),
                text: "fn main() {}\n".to_owned(),
            }
        );
        assert!(read_workspace_file(&directory, Path::new("../outside.txt")).is_err());
        std::fs::write(directory.join("binary"), [0xff, 0xfe]).expect("binary fixture");
        assert!(read_workspace_file(&directory, Path::new("binary")).is_err());

        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn workspace_changes_returns_bounded_structured_patches() {
        let directory = test_directory("workspace-changes");
        std::fs::create_dir_all(&directory).expect("fixture directory");
        let git = |args: &[&str]| {
            let status = Command::new("git")
                .args(args)
                .current_dir(&directory)
                .status()
                .expect("git fixture command");
            assert!(status.success());
        };
        git(&["init", "--quiet"]);
        git(&["config", "user.email", "wabou@example.invalid"]);
        git(&["config", "user.name", "Wabou Test"]);
        std::fs::write(directory.join("example.txt"), "before\n").expect("initial file");
        git(&["add", "example.txt"]);
        git(&["commit", "--quiet", "-m", "fixture"]);
        std::fs::write(directory.join("example.txt"), "before\nafter\n").expect("changed file");
        std::fs::write(directory.join("new.txt"), "new file\n").expect("untracked file");

        let changes = workspace_changes(&directory).expect("workspace changes");
        assert_eq!(changes.files.len(), 2);
        assert_eq!(changes.files[0].path, "example.txt");
        assert_eq!(changes.files[0].status, "modified");
        assert_eq!(changes.files[0].additions, 1);
        assert_eq!(changes.files[0].deletions, 0);
        assert!(changes.files[0].patch.contains("+after"));
        assert_eq!(changes.files[1].path, "new.txt");
        assert_eq!(changes.files[1].status, "added");
        assert!(changes.files[1].patch.contains("+new file"));

        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn binary_workspace_changes_are_not_misreported_as_text_deletions() {
        let change = diff_file(
            Path::new("asset.bin"),
            Some(vec![0, 0xff]),
            Some(vec![0, 0xfe]),
        );
        assert_eq!(change.status, "modified");
        assert_eq!(change.additions, 0);
        assert_eq!(change.deletions, 0);
        assert_eq!(change.patch, "Binary file asset.bin changed\n");
    }

    #[test]
    fn missing_session_files_are_removed_from_the_catalog() {
        let directory = test_directory("session-catalog");
        std::fs::create_dir_all(&directory).expect("fixture directory");
        let existing = directory.join("existing.jsonl");
        std::fs::write(&existing, "{}\n").expect("session fixture");
        let session = |id: &str, path: &Path| PiSession {
            agent_id: "agent-1".to_owned(),
            session_id: id.to_owned(),
            session_file: path.display().to_string(),
            name: None,
            cwd: directory.display().to_string(),
            updated_at: 1,
        };
        let mut catalog = SessionCatalog {
            sessions: vec![
                session("existing", &existing),
                session("missing", &directory.join("missing.jsonl")),
            ],
            agents: Vec::new(),
            settings: AppSettings::default(),
        };

        assert!(prune_missing_sessions(&mut catalog));
        assert_eq!(catalog.sessions.len(), 1);
        assert_eq!(catalog.sessions[0].session_id, "existing");
        assert_eq!(
            catalog_session_file(&catalog, "agent-1", "existing"),
            Some(existing)
        );
        assert!(!prune_missing_sessions(&mut catalog));
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn session_identity_is_scoped_to_its_agent() {
        let session = |agent_id: &str, name: &str| PiSession {
            agent_id: agent_id.to_owned(),
            session_id: "shared-session-id".to_owned(),
            session_file: format!("/{agent_id}/session.jsonl"),
            name: Some(name.to_owned()),
            cwd: format!("/{agent_id}"),
            updated_at: 1,
        };
        let mut catalog = SessionCatalog::default();

        upsert_session(&mut catalog, session("agent-1", "First"));
        upsert_session(&mut catalog, session("agent-2", "Second"));
        upsert_session(&mut catalog, session("agent-1", "Renamed"));

        assert_eq!(catalog.sessions.len(), 2);
        assert_eq!(
            catalog
                .sessions
                .iter()
                .find(|entry| entry.agent_id == "agent-1")
                .and_then(|entry| entry.name.as_deref()),
            Some("Renamed")
        );
        assert_eq!(
            catalog
                .sessions
                .iter()
                .find(|entry| entry.agent_id == "agent-2")
                .and_then(|entry| entry.name.as_deref()),
            Some("Second")
        );
    }

    #[tokio::test]
    async fn prompt_images_stay_in_rust_until_the_rpc_payload_is_built() {
        let path = test_directory("image").with_extension("png");
        tokio::fs::write(&path, [0x89, b'P', b'N', b'G'])
            .await
            .expect("write fixture");

        let images = prompt_images(vec![path.clone()]).await.expect("image");
        assert_eq!(images[0]["type"], "image");
        assert_eq!(images[0]["mimeType"], "image/png");
        assert_eq!(images[0]["data"], "iVBORw==");

        tokio::fs::remove_file(path).await.expect("remove fixture");
    }

    #[tokio::test]
    async fn workspace_context_is_gitignore_aware_and_cannot_escape() {
        let root = test_directory("context");
        tokio::fs::create_dir_all(root.join("src"))
            .await
            .expect("create fixture");
        tokio::fs::create_dir_all(root.join("target"))
            .await
            .expect("create ignored fixture");
        tokio::fs::write(root.join(".gitignore"), "secret.txt\n")
            .await
            .expect("write ignore file");
        tokio::fs::write(root.join("src/main.rs"), "fn main() {}")
            .await
            .expect("write source");
        tokio::fs::write(root.join("secret.txt"), "secret")
            .await
            .expect("write ignored source");
        tokio::fs::write(root.join("target/output"), "build")
            .await
            .expect("write build output");

        let files = list_workspace_files(&root).expect("workspace files");
        assert!(files.contains(&"src/main.rs".to_owned()));
        assert!(!files.contains(&"secret.txt".to_owned()));
        assert!(!files.iter().any(|path| path.starts_with("target/")));

        let prompt = append_workspace_context(
            "Explain this file",
            &root,
            vec![PathBuf::from("src/main.rs")],
        )
        .await
        .expect("context prompt");
        assert!(prompt.contains("<file path=\"src/main.rs\">\nfn main() {}"));
        assert!(
            append_workspace_context("No", &root, vec![PathBuf::from("../outside")])
                .await
                .unwrap_err()
                .contains("stay inside")
        );

        #[cfg(unix)]
        {
            let outside = test_directory("outside");
            tokio::fs::write(&outside, "outside")
                .await
                .expect("write outside file");
            std::os::unix::fs::symlink(&outside, root.join("outside-link"))
                .expect("create symlink");
            assert!(
                append_workspace_context("No", &root, vec![PathBuf::from("outside-link")])
                    .await
                    .unwrap_err()
                    .contains("leaves the workspace")
            );
            tokio::fs::remove_file(outside)
                .await
                .expect("remove outside file");
        }

        tokio::fs::remove_dir_all(root)
            .await
            .expect("remove fixture");
    }
}
