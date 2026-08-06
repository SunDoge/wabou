//! Agent-friendly development protocol for a running Wabou application.
//!
//! The runtime publishes immutable snapshots into [`DebugState`]. A local
//! newline-delimited JSON socket serves those snapshots to the CLI and MCP
//! adapter without ever touching UI state from a background thread.

use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const PROTOCOL_VERSION: u16 = 1;
pub const DEFAULT_TRACE_CAPACITY: usize = 128;
pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;

fn identity_transform() -> [f64; 6] {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}

impl DebugClip {
    fn contains(&self, x: f32, y: f32) -> bool {
        if !self.rect.contains(x, y) {
            return false;
        }
        let radius = self
            .radius
            .max(0.0)
            .min(self.rect.width * 0.5)
            .min(self.rect.height * 0.5);
        if radius == 0.0
            || (x >= self.rect.x + radius && x < self.rect.x + self.rect.width - radius)
            || (y >= self.rect.y + radius && y < self.rect.y + self.rect.height - radius)
        {
            return true;
        }
        let center_x = if x < self.rect.x + radius {
            self.rect.x + radius
        } else {
            self.rect.x + self.rect.width - radius
        };
        let center_y = if y < self.rect.y + radius {
            self.rect.y + radius
        } else {
            self.rect.y + self.rect.height - radius
        };
        (x - center_x).powi(2) + (y - center_y).powi(2) <= radius.powi(2)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugClip {
    pub node_id: u32,
    pub kind: String,
    pub coordinate_space: String,
    pub rect: Rect,
    pub radius: f32,
    #[serde(default = "identity_transform")]
    pub transform: [f64; 6],
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct DebugClipInfo {
    pub widget_local: Option<DebugClip>,
    pub chain: Vec<DebugClip>,
    pub effective: Option<DebugClip>,
    pub static_transform: [f64; 6],
    pub runtime_transform: Option<[f64; 6]>,
    pub border_transform: [f64; 6],
    pub scene_transform: [f64; 6],
    pub device_scale: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugComputedStyle {
    pub display: Option<String>,
    pub position: Option<String>,
    pub overflow_x: Option<String>,
    pub overflow_y: Option<String>,
    pub font_size: f32,
    pub font_weight: f32,
    pub wrap_text: bool,
    pub opacity: f32,
    pub pointer_events: bool,
    pub z_index: i32,
    pub text_color: String,
    pub background: Option<String>,
}

impl Default for DebugComputedStyle {
    fn default() -> Self {
        Self {
            display: None,
            position: None,
            overflow_x: None,
            overflow_y: None,
            font_size: 0.0,
            font_weight: 0.0,
            wrap_text: false,
            opacity: 1.0,
            pointer_events: true,
            z_index: 0,
            text_color: String::new(),
            background: None,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugNode {
    pub id: u32,
    pub parent_id: Option<u32>,
    pub tag: String,
    pub text: Option<String>,
    pub classes: Vec<String>,
    pub matched_rules: Vec<String>,
    pub attrs: Vec<(String, String)>,
    pub rect: Rect,
    pub content_rect: Rect,
    pub listeners: Vec<u8>,
    pub widget: Option<String>,
    #[serde(default)]
    pub clip: DebugClipInfo,
    pub computed: DebugComputedStyle,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugStatus {
    pub protocol_version: u16,
    pub pid: u32,
    pub revision: u64,
    pub viewport_width: u32,
    pub viewport_height: u32,
    #[serde(default = "default_device_scale")]
    pub device_scale: f64,
    pub node_count: usize,
    pub focused_node: Option<u32>,
    pub hovered_node: Option<u32>,
}

fn default_device_scale() -> f64 {
    1.0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DebugFrame {
    pub direction: String,
    pub sequence: u64,
    pub byte_len: usize,
    pub record_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_hex: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugSnapshot {
    pub status: DebugStatus,
    pub nodes: Vec<DebugNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugPointInspection {
    pub x: f32,
    pub y: f32,
    pub node: Option<DebugNode>,
    pub ancestors: Vec<DebugNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DebugCaptureCase {
    pub screenshot_path: PathBuf,
    pub snapshot: DebugSnapshot,
    pub frames: Vec<DebugFrame>,
    pub point: Option<DebugPointInspection>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct EmptyParams {}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct InspectPointParams {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct CaptureCaseParams {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

impl CaptureCaseParams {
    fn point(self) -> Result<Option<(f32, f32)>, String> {
        match (self.x, self.y) {
            (Some(x), Some(y)) => Ok(Some((x, y))),
            (None, None) => Ok(None),
            _ => Err("captureCase params.x and params.y must be provided together".into()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryNodesParams {
    #[serde(default)]
    pub query: String,
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

fn default_query_limit() -> usize {
    100
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct NodeIdParams {
    pub id: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RecentFramesParams {
    #[serde(default = "default_frame_limit")]
    pub limit: usize,
}

fn default_frame_limit() -> usize {
    20
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
pub enum DebugCommand {
    #[serde(rename = "status")]
    Status(EmptyParams),
    #[serde(rename = "queryNodes")]
    QueryNodes(QueryNodesParams),
    #[serde(rename = "inspectNode")]
    InspectNode(NodeIdParams),
    #[serde(rename = "inspectAtPoint")]
    InspectAtPoint(InspectPointParams),
    #[serde(rename = "recentFrames")]
    RecentFrames(RecentFramesParams),
    #[serde(rename = "setOverlay")]
    SetOverlay(DebugOverlay),
    #[serde(rename = "captureScreenshot")]
    CaptureScreenshot(EmptyParams),
    #[serde(rename = "captureCase")]
    CaptureCase(CaptureCaseParams),
}

/// Runtime-controlled overlay painted by the inspected application.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DebugOverlay {
    pub layout: bool,
    pub clips: bool,
    pub hit_target: bool,
    pub selected_node: Option<u32>,
}

impl DebugOverlay {
    pub fn is_enabled(self) -> bool {
        self.layout || self.clips || self.hit_target || self.selected_node.is_some()
    }
}

pub struct DebugState {
    snapshot: DebugSnapshot,
    frames: VecDeque<DebugFrame>,
    trace_capacity: usize,
    screenshot_request: Option<PathBuf>,
    screenshot_result: Option<Result<PathBuf, String>>,
    capture_case_requested: bool,
    capture_case_point: Option<(f32, f32)>,
    capture_case_result: Option<Result<DebugCaptureCase, String>>,
    wake: Option<Arc<dyn Fn() + Send + Sync>>,
    raw_frames: bool,
    overlay: DebugOverlay,
    overlay_changed: bool,
}

pub type SharedDebugState = Arc<RwLock<DebugState>>;

pub struct ServerHandle {
    running: Arc<AtomicBool>,
    path: PathBuf,
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Release);
        #[cfg(unix)]
        {
            let _ = std::os::unix::net::UnixStream::connect(&self.path);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        let _ = fs::remove_file(&self.path);
    }
}

impl Default for DebugState {
    fn default() -> Self {
        Self {
            snapshot: DebugSnapshot::default(),
            frames: VecDeque::new(),
            trace_capacity: DEFAULT_TRACE_CAPACITY,
            screenshot_request: None,
            screenshot_result: None,
            capture_case_requested: false,
            capture_case_point: None,
            capture_case_result: None,
            wake: None,
            raw_frames: std::env::var_os("WABOU_DEVTOOLS_RAW_FRAMES").is_some(),
            overlay: DebugOverlay::default(),
            overlay_changed: false,
        }
    }
}

impl DebugState {
    pub fn shared() -> SharedDebugState {
        Arc::new(RwLock::new(Self::default()))
    }

    pub fn publish(&mut self, snapshot: DebugSnapshot) {
        self.snapshot = snapshot;
    }

    pub fn push_frame(&mut self, mut frame: DebugFrame) {
        if !self.raw_frames {
            frame.bytes_hex = None;
        }
        if self.frames.len() == self.trace_capacity {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);
    }

    pub fn snapshot(&self) -> &DebugSnapshot {
        &self.snapshot
    }

    pub fn overlay(&self) -> DebugOverlay {
        self.overlay
    }

    pub fn set_overlay(&mut self, overlay: DebugOverlay) {
        self.overlay = overlay;
        self.overlay_changed = true;
        if let Some(wake) = &self.wake {
            wake();
        }
    }

    pub fn take_overlay_change(&mut self) -> bool {
        std::mem::take(&mut self.overlay_changed)
    }

    pub fn frames(&self) -> &VecDeque<DebugFrame> {
        &self.frames
    }

    pub fn set_wake(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.wake = Some(wake);
    }

    pub fn request_screenshot(&mut self) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "wabou-screenshot-{}-{}.png",
            std::process::id(),
            self.snapshot.status.revision
        ));
        self.screenshot_result = None;
        self.screenshot_request = Some(path.clone());
        if let Some(wake) = &self.wake {
            wake();
        }
        path
    }

    pub fn request_capture_case(&mut self, point: Option<(f32, f32)>) -> PathBuf {
        let path = self.request_screenshot();
        self.capture_case_requested = true;
        self.capture_case_point = point;
        self.capture_case_result = None;
        path
    }

    pub fn take_screenshot_request(&mut self) -> Option<PathBuf> {
        self.screenshot_request.take()
    }

    pub fn has_screenshot_request(&self) -> bool {
        self.screenshot_request.is_some()
    }

    pub fn complete_screenshot(&mut self, result: Result<PathBuf, String>) {
        if self.capture_case_requested {
            self.capture_case_requested = false;
            self.capture_case_result = Some(result.clone().map(|screenshot_path| {
                let point = self
                    .capture_case_point
                    .take()
                    .map(|(x, y)| self.inspect_point(x, y));
                DebugCaptureCase {
                    screenshot_path,
                    snapshot: self.snapshot.clone(),
                    frames: self.frames.iter().cloned().collect(),
                    point,
                }
            }));
        }
        self.screenshot_result = Some(result);
    }

    pub fn screenshot_result(&self) -> Option<&Result<PathBuf, String>> {
        self.screenshot_result.as_ref()
    }

    pub fn capture_case_result(&self) -> Option<&Result<DebugCaptureCase, String>> {
        self.capture_case_result.as_ref()
    }

    fn inspect_point(&self, x: f32, y: f32) -> DebugPointInspection {
        let node = self
            .snapshot
            .nodes
            .iter()
            .rev()
            .find(|node| {
                node.rect.contains(x, y)
                    && node
                        .clip
                        .effective
                        .as_ref()
                        .is_none_or(|clip| clip.contains(x, y))
                    && node.computed.pointer_events
            })
            .cloned();
        let mut ancestors = Vec::new();
        let mut parent_id = node.as_ref().and_then(|node| node.parent_id);
        while let Some(id) = parent_id {
            let Some(parent) = self.snapshot.nodes.iter().find(|node| node.id == id) else {
                break;
            };
            ancestors.push(parent.clone());
            parent_id = parent.parent_id;
        }
        ancestors.reverse();
        DebugPointInspection {
            x,
            y,
            node,
            ancestors,
        }
    }

    fn execute(&mut self, command: &DebugCommand) -> Result<Value, String> {
        match command {
            DebugCommand::Status(_) => {
                serde_json::to_value(&self.snapshot.status).map_err(|e| e.to_string())
            }
            DebugCommand::QueryNodes(params) => {
                let query = params.query.to_lowercase();
                let limit = params.limit.min(1000);
                let nodes: Vec<_> = self
                    .snapshot
                    .nodes
                    .iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.tag.to_lowercase().contains(&query)
                            || node
                                .text
                                .as_deref()
                                .is_some_and(|text| text.to_lowercase().contains(&query))
                            || node
                                .classes
                                .iter()
                                .any(|class| class.to_lowercase().contains(&query))
                    })
                    .take(limit)
                    .collect();
                serde_json::to_value(nodes).map_err(|e| e.to_string())
            }
            DebugCommand::InspectNode(NodeIdParams { id }) => {
                let node = self
                    .snapshot
                    .nodes
                    .iter()
                    .find(|node| node.id == *id)
                    .ok_or_else(|| format!("node {id} not found"))?;
                serde_json::to_value(node).map_err(|e| e.to_string())
            }
            DebugCommand::InspectAtPoint(InspectPointParams { x, y }) => {
                serde_json::to_value(self.inspect_point(*x, *y)).map_err(|e| e.to_string())
            }
            DebugCommand::RecentFrames(params) => {
                let limit = params.limit.min(self.trace_capacity);
                let frames: Vec<_> = self.frames.iter().rev().take(limit).collect();
                serde_json::to_value(frames).map_err(|e| e.to_string())
            }
            DebugCommand::SetOverlay(overlay) => {
                self.set_overlay(*overlay);
                serde_json::to_value(overlay).map_err(|error| error.to_string())
            }
            DebugCommand::CaptureScreenshot(_) | DebugCommand::CaptureCase(_) => {
                Err("capture commands require the asynchronous server path".into())
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub id: u64,
    #[serde(flatten)]
    pub command: DebugCommand,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Response {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn socket_path() -> PathBuf {
    if let Some(path) = std::env::var_os("WABOU_DEVTOOLS_SOCKET") {
        return PathBuf::from(path);
    }
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join(format!("wabou-{}.sock", std::process::id()))
}

/// Find the most recently created live-looking Wabou socket. Explicit
/// `WABOU_DEVTOOLS_SOCKET` always wins, which is required when multiple apps
/// are under test concurrently.
pub fn discover_socket() -> Result<PathBuf, String> {
    #[cfg(unix)]
    use std::os::unix::fs::FileTypeExt;
    if let Some(path) = std::env::var_os("WABOU_DEVTOOLS_SOCKET") {
        return Ok(PathBuf::from(path));
    }
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let mut candidates: Vec<_> = fs::read_dir(&base)
        .map_err(|e| format!("cannot scan {}: {e}", base.display()))?
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("wabou-") && name.ends_with(".sock"))
        })
        .filter_map(|entry| {
            let metadata = entry.metadata().ok()?;
            #[cfg(unix)]
            if !metadata.file_type().is_socket() {
                return None;
            }
            let modified = metadata.modified().ok()?;
            Some((modified, entry.path()))
        })
        .collect();
    candidates.sort_by_key(|(modified, _)| *modified);
    while let Some((_, path)) = candidates.pop() {
        #[cfg(unix)]
        if std::os::unix::net::UnixStream::connect(&path).is_ok() {
            return Ok(path);
        } else {
            let _ = fs::remove_file(path);
        }
        #[cfg(not(unix))]
        return Ok(path);
    }
    Err(format!(
        "no live Wabou DevTools socket found in {}",
        base.display()
    ))
}

#[cfg(unix)]
pub fn serve(state: SharedDebugState, path: PathBuf) -> std::io::Result<ServerHandle> {
    use std::os::unix::fs::FileTypeExt;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::{UnixListener, UnixStream};

    if path.exists() {
        if !fs::symlink_metadata(&path)?.file_type().is_socket() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("refusing to replace non-socket {}", path.display()),
            ));
        }
        if UnixStream::connect(&path).is_ok() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!("DevTools socket is already live: {}", path.display()),
            ));
        }
        fs::remove_file(&path)?;
    }
    let listener = UnixListener::bind(&path)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    let running = Arc::new(AtomicBool::new(true));
    let thread_running = running.clone();
    let thread_path = path.clone();
    let handle = thread::Builder::new()
        .name("wabou-devtools".into())
        .spawn(move || {
            for stream in listener.incoming() {
                if !thread_running.load(Ordering::Acquire) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(1)));
                let Ok(reader_stream) = stream.try_clone() else {
                    continue;
                };
                let reader = BufReader::new(reader_stream);
                for line in reader.lines() {
                    let response = match line.map_err(|e| e.to_string()).and_then(|line| {
                        if line.len() > MAX_REQUEST_BYTES {
                            Err("DevTools request exceeds 1 MiB".to_string())
                        } else {
                            serde_json::from_str::<Request>(&line).map_err(|e| e.to_string())
                        }
                    }) {
                        Ok(request) => {
                            let outcome = if matches!(
                                &request.command,
                                DebugCommand::CaptureScreenshot(_) | DebugCommand::CaptureCase(_)
                            ) {
                                let point = match &request.command {
                                    DebugCommand::CaptureCase(params) => params.point(),
                                    DebugCommand::CaptureScreenshot(_) => Ok(None),
                                    _ => unreachable!(),
                                };
                                let capture_case =
                                    matches!(&request.command, DebugCommand::CaptureCase(_));
                                let path = state
                                    .write()
                                    .map_err(|_| "debug state poisoned".to_string())
                                    .and_then(|mut state| {
                                        let point = point?;
                                        if capture_case {
                                            Ok(state.request_capture_case(point))
                                        } else {
                                            Ok(state.request_screenshot())
                                        }
                                    });
                                path.and_then(|path| {
                                    let deadline = std::time::Instant::now()
                                        + std::time::Duration::from_secs(10);
                                    loop {
                                        let result = state
                                            .read()
                                            .map_err(|_| "debug state poisoned".to_string())?;
                                        let result = if capture_case {
                                            result.capture_case_result().cloned().map(|result| {
                                                result.and_then(|capture| {
                                                    serde_json::to_value(capture)
                                                        .map_err(|error| error.to_string())
                                                })
                                            })
                                        } else {
                                            result.screenshot_result().cloned().map(|result| {
                                                result.map(|path| json!({"path": path}))
                                            })
                                        };
                                        if let Some(result) = result {
                                            break result;
                                        }
                                        if std::time::Instant::now() >= deadline {
                                            break Err(format!(
                                                "screenshot timed out; requested {}",
                                                path.display()
                                            ));
                                        }
                                        std::thread::sleep(std::time::Duration::from_millis(10));
                                    }
                                })
                            } else {
                                state
                                    .write()
                                    .map_err(|_| "debug state poisoned".to_string())
                                    .and_then(|mut state| state.execute(&request.command))
                            };
                            match outcome {
                                Ok(result) => Response {
                                    id: request.id,
                                    result: Some(result),
                                    error: None,
                                },
                                Err(error) => Response {
                                    id: request.id,
                                    result: None,
                                    error: Some(error),
                                },
                            }
                        }
                        Err(error) => Response {
                            id: 0,
                            result: None,
                            error: Some(error),
                        },
                    };
                    if serde_json::to_writer(&mut stream, &response).is_err()
                        || stream.write_all(b"\n").is_err()
                    {
                        break;
                    }
                }
            }
            let _ = fs::remove_file(thread_path);
        })?;
    Ok(ServerHandle {
        running,
        path,
        thread: Some(handle),
    })
}

#[cfg(not(unix))]
pub fn serve(_state: SharedDebugState, _path: PathBuf) -> std::io::Result<ServerHandle> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "named pipes are not implemented yet",
    ))
}

#[cfg(unix)]
pub fn call(path: &Path, request: &Request) -> Result<Response, String> {
    use std::os::unix::net::UnixStream;
    let mut stream = UnixStream::connect(path).map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut stream, request).map_err(|e| e.to_string())?;
    stream.write_all(b"\n").map_err(|e| e.to_string())?;
    let mut line = String::new();
    BufReader::new(stream)
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    serde_json::from_str(&line).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
pub fn call(_path: &Path, _request: &Request) -> Result<Response, String> {
    Err("named pipes are not implemented yet".into())
}

pub fn bytes_hex(bytes: &[u8], max: usize) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len().min(max) * 2);
    for byte in bytes.iter().take(max) {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

pub fn request(id: u64, method: impl Into<String>, params: Value) -> Request {
    serde_json::from_value(json!({
        "id": id,
        "method": method.into(),
        "params": params,
    }))
    .expect("known Wabou DevTools method and valid params")
}

pub fn empty_params() -> Value {
    json!({})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn query_and_inspect_are_bounded_and_semantic() {
        let mut state = DebugState::default();
        state.publish(DebugSnapshot {
            status: DebugStatus {
                protocol_version: 1,
                node_count: 1,
                ..Default::default()
            },
            nodes: vec![DebugNode {
                id: 7,
                tag: "span".into(),
                text: Some("1 comments".into()),
                classes: vec!["metadata".into()],
                ..Default::default()
            }],
        });
        let result = state
            .execute(&request(1, "queryNodes", json!({"query":"comments"})).command)
            .unwrap();
        assert_eq!(result.as_array().unwrap().len(), 1);
        let result = state
            .execute(&request(2, "inspectNode", json!({"id":7})).command)
            .unwrap();
        assert_eq!(result["tag"], "span");
    }

    #[test]
    fn inspect_at_point_uses_paint_order_pointer_events_and_effective_clip() {
        let mut state = DebugState::default();
        let node = |id, pointer_events, clip: Option<Rect>| DebugNode {
            id,
            tag: "div".into(),
            rect: Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            clip: DebugClipInfo {
                effective: clip.map(|rect| DebugClip {
                    node_id: id,
                    kind: "effective".into(),
                    coordinate_space: "window-logical".into(),
                    rect,
                    radius: 0.0,
                    transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                }),
                ..Default::default()
            },
            computed: DebugComputedStyle {
                pointer_events,
                ..Default::default()
            },
            ..Default::default()
        };
        state.publish(DebugSnapshot {
            nodes: vec![
                node(1, true, None),
                node(2, false, None),
                node(
                    3,
                    true,
                    Some(Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 10.0,
                        height: 10.0,
                    }),
                ),
            ],
            ..Default::default()
        });

        let result = state
            .execute(&request(1, "inspectAtPoint", json!({"x": 50, "y": 50})).command)
            .unwrap();
        assert_eq!(result["node"]["id"], 1);
    }

    #[test]
    fn capture_case_freezes_snapshot_when_the_screenshot_completes() {
        let mut state = DebugState::default();
        state.request_capture_case(Some((12.0, 34.0)));
        state.publish(DebugSnapshot {
            status: DebugStatus {
                revision: 7,
                ..Default::default()
            },
            ..Default::default()
        });
        let path = std::env::temp_dir().join("wabou-capture-case-test.png");
        state.complete_screenshot(Ok(path.clone()));
        state.publish(DebugSnapshot {
            status: DebugStatus {
                revision: 8,
                ..Default::default()
            },
            ..Default::default()
        });

        let capture = state.capture_case_result().unwrap().as_ref().unwrap();
        assert_eq!(capture.snapshot.status.revision, 7);
        assert_eq!(capture.screenshot_path, path);
        assert_eq!(capture.point.as_ref().unwrap().x, 12.0);
    }

    #[test]
    fn overlay_command_updates_runtime_diagnostics() {
        let mut state = DebugState::default();
        let value = state
            .execute(
                &request(
                    1,
                    "setOverlay",
                    json!({
                        "layout": true,
                        "clips": true,
                        "hitTarget": true,
                        "selectedNode": 42
                    }),
                )
                .command,
            )
            .expect("set overlay");
        assert_eq!(value["selectedNode"], 42);
        assert_eq!(
            state.overlay(),
            DebugOverlay {
                layout: true,
                clips: true,
                hit_target: true,
                selected_node: Some(42),
            }
        );
        assert!(state.take_overlay_change());
        assert!(!state.take_overlay_change());
    }

    #[cfg(unix)]
    #[test]
    fn server_refuses_to_replace_a_regular_file() {
        let path =
            std::env::temp_dir().join(format!("wabou-regular-file-{}.sock", std::process::id()));
        fs::write(&path, b"keep me").unwrap();
        let error = match serve(DebugState::shared(), path.clone()) {
            Ok(_) => panic!("regular file must not be replaced"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(&path).unwrap(), b"keep me");
        fs::remove_file(path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn unix_socket_round_trip_uses_versioned_status() {
        let state = DebugState::shared();
        state.write().unwrap().publish(DebugSnapshot {
            status: DebugStatus {
                protocol_version: PROTOCOL_VERSION,
                pid: 42,
                revision: 9,
                ..Default::default()
            },
            nodes: Vec::new(),
        });
        let path = std::env::temp_dir().join(format!(
            "wabou-test-{}-{}.sock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _server = serve(state.clone(), path.clone()).unwrap();
        let response = call(&path, &request(1, "status", empty_params())).unwrap();
        assert_eq!(response.result.unwrap()["revision"], 9);
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let screenshot_state = state.clone();
        let worker = std::thread::spawn(move || {
            loop {
                let request = { screenshot_state.write().unwrap().take_screenshot_request() };
                if let Some(path) = request {
                    screenshot_state
                        .write()
                        .unwrap()
                        .complete_screenshot(Ok(path));
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
        let response = call(&path, &request(2, "captureScreenshot", empty_params())).unwrap();
        worker.join().unwrap();
        assert!(
            response.result.unwrap()["path"]
                .as_str()
                .unwrap()
                .ends_with(".png")
        );

        let capture_state = state.clone();
        let worker = std::thread::spawn(move || {
            loop {
                let request = { capture_state.write().unwrap().take_screenshot_request() };
                if let Some(path) = request {
                    capture_state.write().unwrap().complete_screenshot(Ok(path));
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        });
        let response = call(&path, &request(3, "captureCase", json!({"x": 10, "y": 20}))).unwrap();
        worker.join().unwrap();
        let capture = response.result.unwrap();
        assert_eq!(capture["snapshot"]["status"]["revision"], 9);
        assert_eq!(capture["point"]["x"], 10.0);
    }
}
