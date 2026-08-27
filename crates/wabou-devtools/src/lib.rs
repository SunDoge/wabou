//! Agent-friendly development protocol for a running Wabou application.
//!
//! The runtime publishes immutable snapshots into [`DebugState`]. A local
//! newline-delimited JSON socket serves those snapshots to the CLI and MCP
//! adapter without ever touching UI state from a background thread.

#![warn(missing_docs)]

use std::collections::VecDeque;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use wabou_host_api::DebugOverlayPaintStats;
pub use wabou_host_api::NodeKey;

mod validation;
pub use validation::{DebugValidationIssue, DebugValidationReport};

/// Current newline-delimited JSON protocol version.
pub const PROTOCOL_VERSION: u16 = 3;
/// Default number of recent host frames retained in memory.
pub const DEFAULT_TRACE_CAPACITY: usize = 128;
/// Default number of completed screenshot artifacts retained per runtime.
pub const DEFAULT_CAPTURE_RETENTION: usize = 16;
/// Maximum accepted JSON request line size.
pub const MAX_REQUEST_BYTES: usize = 1024 * 1024;
/// Maximum validation findings serialized in one protocol response.
pub const MAX_VALIDATION_ISSUES: usize = 256;

fn identity_transform() -> [f64; 6] {
    [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Axis-aligned rectangle in logical window coordinates.
pub struct Rect {
    /// Left edge.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub x: f32,
    /// Top edge.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub y: f32,
    /// Non-negative width.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub width: f32,
    /// Non-negative height.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
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
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// One rounded clip and its coordinate transform.
pub struct DebugClip {
    /// Node establishing the clip.
    pub node_id: NodeKey,
    /// Clip origin such as overflow or native widget.
    pub kind: String,
    /// Coordinate space in which [`Self::rect`] is expressed.
    pub coordinate_space: String,
    /// Clip rectangle.
    pub rect: Rect,
    /// Uniform corner radius.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub radius: f32,
    /// Affine coefficients mapping the clip into window coordinates.
    #[serde(default = "identity_transform")]
    #[cfg_attr(
        feature = "bindings",
        specta(type = [specta_typescript::Number; 6])
    )]
    pub transform: [f64; 6],
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
#[serde(default)]
/// Complete clipping and transform diagnostics for one retained node.
pub struct DebugClipInfo {
    /// Content-local clip applied inside a native widget fragment.
    pub widget_local: Option<DebugClip>,
    /// Ordered ancestor clip chain.
    pub chain: Vec<DebugClip>,
    /// Final intersected clip used for hit testing.
    pub effective: Option<DebugClip>,
    /// Authored CSS transform.
    #[cfg_attr(
        feature = "bindings",
        specta(type = [specta_typescript::Number; 6])
    )]
    pub static_transform: [f64; 6],
    /// Host-driven transform composed after static CSS.
    #[cfg_attr(
        feature = "bindings",
        specta(type = Option<[specta_typescript::Number; 6]>)
    )]
    pub runtime_transform: Option<[f64; 6]>,
    /// Transform from border-local to logical window coordinates.
    #[cfg_attr(
        feature = "bindings",
        specta(type = [specta_typescript::Number; 6])
    )]
    pub border_transform: [f64; 6],
    /// Transform used when appending content to the final scene.
    #[cfg_attr(
        feature = "bindings",
        specta(type = [specta_typescript::Number; 6])
    )]
    pub scene_transform: [f64; 6],
    /// Physical pixels per logical pixel.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub device_scale: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Compact, serializable view of a node's resolved style.
pub struct DebugComputedStyle {
    /// Resolved display mode.
    pub display: Option<String>,
    /// Resolved positioning mode.
    pub position: Option<String>,
    /// Horizontal overflow mode.
    pub overflow_x: Option<String>,
    /// Vertical overflow mode.
    pub overflow_y: Option<String>,
    /// Font size in logical pixels.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub font_size: f32,
    /// Numeric font weight.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub font_weight: f32,
    /// Whether an italic or oblique face was requested.
    #[serde(default)]
    pub font_italic: bool,
    /// Requested font-family fallback stack, when explicitly configured.
    #[serde(default)]
    pub font_family: Option<String>,
    /// Whether resolved glyph runs require synthetic emboldening.
    #[serde(default)]
    pub synthetic_bold: bool,
    /// Whether resolved glyph runs require a synthetic italic skew.
    #[serde(default)]
    pub synthetic_italic: bool,
    /// Whether normal inline wrapping is enabled.
    pub wrap_text: bool,
    /// Resolved opacity.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub opacity: f32,
    /// Whether the node itself participates in hit testing.
    pub pointer_events: bool,
    /// Sibling-relative paint order.
    pub z_index: i32,
    /// Host-owned stacking plane.
    pub overlay_plane: String,
    /// Current host-owned scrollbar opacity.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub scrollbar_opacity: f32,
    /// Resolved text color in diagnostic notation.
    pub text_color: String,
    /// Resolved background color in diagnostic notation.
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
            font_italic: false,
            font_family: None,
            synthetic_bold: false,
            synthetic_italic: false,
            wrap_text: false,
            opacity: 1.0,
            pointer_events: true,
            z_index: 0,
            overlay_plane: "Content".into(),
            scrollbar_opacity: 0.0,
            text_color: String::new(),
            background: None,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Winning declaration and its lower-priority sources for one Style IR property.
pub struct DebugStyleCascade {
    /// Canonical Style IR property name.
    pub property: String,
    /// Source whose declaration won the cascade, such as `.font-normal` or `inline`.
    pub source: String,
    /// Lower-priority sources replaced by the winner, in cascade order.
    pub overridden_sources: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Final platform-neutral accessibility projection for one retained node.
pub struct DebugSemanticProjection {
    /// Canonical role after native-widget and authored-role resolution.
    pub role: String,
    /// Accessible name after descendant-label inference.
    pub label: Option<String>,
    /// Whether assistive technology exposes the node as disabled.
    pub disabled: bool,
    /// Whether the node is reachable from the current platform root.
    pub exposed: bool,
    /// Live nodes resolved from the authored `aria-controls` ID references.
    pub controls: Vec<NodeKey>,
    /// Live node resolved from the authored `aria-activedescendant` ID reference.
    pub active_descendant: Option<NodeKey>,
    /// Final role-specific states exported to the platform bridge.
    pub states: DebugSemanticStates,
    /// Final numeric range exported for sliders and progress indicators.
    pub range: DebugSemanticRange,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Canonical final values for role-specific accessibility states.
pub struct DebugSemanticStates {
    /// Final check state (`false`, `true`, or `mixed`).
    pub checked: Option<String>,
    /// Final toggle-button press state (`false`, `true`, or `mixed`).
    pub pressed: Option<String>,
    /// Final selection state.
    pub selected: Option<bool>,
    /// Final expansion state.
    pub expanded: Option<bool>,
    /// Final current-item category.
    pub current: Option<String>,
    /// Final popup kind.
    pub popup: Option<String>,
    /// Final explicit modal state.
    pub modal: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Canonical final numeric values for range-based accessibility controls.
pub struct DebugSemanticRange {
    /// Current numeric value.
    pub value: Option<f64>,
    /// Optional minimum numeric value.
    pub min: Option<f64>,
    /// Optional maximum numeric value.
    pub max: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Paint geometry for one single-line text layout.
pub struct DebugTextMetrics {
    /// `node` for ordinary text or `widget` for a native editor.
    pub source: String,
    /// Line box in logical window coordinates.
    pub line_box: Rect,
    /// Baseline in logical window coordinates.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub baseline: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Retained node projection published by the UI thread.
pub struct DebugNode {
    /// Solid-side node identifier.
    pub id: NodeKey,
    /// Logical parent identifier.
    pub parent_id: Option<NodeKey>,
    /// Intrinsic/JSX tag name.
    pub tag: String,
    /// Plain text content when present.
    pub text: Option<String>,
    /// Single-line paint geometry, including native widget text.
    #[serde(default)]
    pub text_metrics: Option<DebugTextMetrics>,
    /// Authored class names.
    pub classes: Vec<String>,
    /// Stylesheet selectors that contributed declarations.
    pub matched_rules: Vec<String>,
    /// Rejected utilities or invalid declarations associated with the node.
    #[serde(default)]
    pub style_diagnostics: Vec<String>,
    /// Final source and overridden sources for every declared Style IR property.
    #[serde(default)]
    pub style_cascade: Vec<DebugStyleCascade>,
    /// Retained string attributes.
    pub attrs: Vec<(String, String)>,
    /// Border box in logical window coordinates.
    pub rect: Rect,
    /// Content box in logical window coordinates.
    pub content_rect: Rect,
    /// Generated event codes registered on the node.
    pub listeners: Vec<u8>,
    /// Whether the final interaction projection admits this node as a focus target.
    #[serde(default)]
    pub focusable: bool,
    /// JS-authored focus order. Negative values are programmatically focusable but skipped by Tab.
    #[serde(default)]
    pub focus_order: Option<i32>,
    /// Final accessibility projection, absent when semantics were not built for this frame.
    #[serde(default)]
    pub semantic: Option<DebugSemanticProjection>,
    /// Native widget kind, if attached.
    pub widget: Option<String>,
    /// Clip and transform diagnostics.
    #[serde(default)]
    pub clip: DebugClipInfo,
    /// Resolved style projection.
    pub computed: DebugComputedStyle,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Runtime/window status returned by the DevTools `status` command.
pub struct DebugStatus {
    /// Wire protocol version.
    pub protocol_version: u16,
    /// Inspected native process identifier.
    pub pid: u32,
    /// Monotonic retained-tree revision.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub revision: u64,
    /// Logical viewport width.
    pub viewport_width: u32,
    /// Logical viewport height.
    pub viewport_height: u32,
    /// Physical pixels per logical pixel.
    #[serde(default = "default_device_scale")]
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub device_scale: f64,
    /// Number of retained nodes.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub node_count: usize,
    /// Active ordinary-text raster backend.
    #[serde(default)]
    pub text_backend: String,
    /// Platform policy used when ordinary raster text must fall back to outlines.
    #[serde(default)]
    pub text_outline_fallback: String,
    /// Focused Solid node identifier.
    pub focused_node: Option<NodeKey>,
    /// Hovered Solid node identifier.
    pub hovered_node: Option<NodeKey>,
    /// Native diagnostic layers currently requested by any DevTools client.
    #[serde(default)]
    pub overlay: DebugOverlay,
    /// Evidence from the most recently completed native overlay paint pass.
    #[serde(default)]
    pub overlay_paint: DebugOverlayPaintStats,
}

fn default_device_scale() -> f64 {
    1.0
}

impl Default for DebugStatus {
    fn default() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            pid: 0,
            revision: 0,
            viewport_width: 0,
            viewport_height: 0,
            device_scale: default_device_scale(),
            node_count: 0,
            text_backend: String::new(),
            text_outline_fallback: String::new(),
            focused_node: None,
            hovered_node: None,
            overlay: DebugOverlay::default(),
            overlay_paint: DebugOverlayPaintStats::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Metadata for one recent binary bridge frame.
pub struct DebugFrame {
    /// `jsToHost` or `hostToJs` direction.
    pub direction: String,
    /// Direction-local monotonic sequence.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub sequence: u64,
    /// Encoded frame length.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub byte_len: usize,
    /// Number of records in the frame.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub record_count: usize,
    /// Optional raw bytes when explicitly enabled for diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_hex: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Immutable retained-tree snapshot published by one application frame.
pub struct DebugSnapshot {
    /// Runtime/window status at publication time.
    pub status: DebugStatus,
    /// Retained nodes in paint/source order.
    pub nodes: Vec<DebugNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Hit-test result and ancestry at one logical window point.
pub struct DebugPointInspection {
    /// Queried horizontal coordinate.
    pub x: f32,
    /// Queried vertical coordinate.
    pub y: f32,
    /// Topmost hit-testable node.
    pub node: Option<DebugNode>,
    /// Root-to-parent ancestry of [`Self::node`].
    pub ancestors: Vec<DebugNode>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
/// Atomic diagnostic bundle combining pixels and retained state.
pub struct DebugCaptureCase {
    /// Secure temporary PNG path.
    pub screenshot_path: PathBuf,
    /// Snapshot corresponding to the capture request.
    pub snapshot: DebugSnapshot,
    /// Recent bridge frames retained with the case.
    pub frames: Vec<DebugFrame>,
    /// Optional point inspection requested with the capture.
    pub point: Option<DebugPointInspection>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
/// Parameters for a command that accepts no arguments.
pub struct EmptyParams {}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/// Logical point supplied to `inspectAtPoint`.
pub struct InspectPointParams {
    /// Horizontal logical coordinate.
    pub x: f32,
    /// Vertical logical coordinate.
    pub y: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
/// Optional logical point bundled with a capture case.
pub struct CaptureCaseParams {
    /// Horizontal coordinate; must be paired with [`Self::y`].
    pub x: Option<f32>,
    /// Vertical coordinate; must be paired with [`Self::x`].
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
/// Text query and bounded result count for `queryNodes`.
pub struct QueryNodesParams {
    /// Case-insensitive tag, text, or class substring.
    #[serde(default)]
    pub query: String,
    /// Maximum returned nodes, clamped by the server.
    #[serde(default = "default_query_limit")]
    pub limit: usize,
}

fn default_query_limit() -> usize {
    100
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/// Solid node identifier supplied to node-specific commands.
pub struct NodeIdParams {
    /// Solid-side node identifier.
    pub id: NodeKey,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
/// Bounded count supplied to `recentFrames`.
pub struct RecentFramesParams {
    /// Maximum returned frames.
    #[serde(default = "default_frame_limit")]
    pub limit: usize,
}

fn default_frame_limit() -> usize {
    20
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params")]
/// One request method supported by the local DevTools protocol.
pub enum DebugCommand {
    /// Return runtime/window status.
    #[serde(rename = "status")]
    Status(EmptyParams),
    /// Search retained nodes.
    #[serde(rename = "queryNodes")]
    QueryNodes(QueryNodesParams),
    /// Inspect one retained node.
    #[serde(rename = "inspectNode")]
    InspectNode(NodeIdParams),
    /// Hit-test and inspect one logical point.
    #[serde(rename = "inspectAtPoint")]
    InspectAtPoint(InspectPointParams),
    /// Return recent bridge-frame metadata.
    #[serde(rename = "recentFrames")]
    RecentFrames(RecentFramesParams),
    /// Validate retained node, geometry, clip and reference invariants.
    #[serde(rename = "validateSnapshot")]
    ValidateSnapshot(EmptyParams),
    /// Change the native debug overlay.
    #[serde(rename = "setOverlay")]
    SetOverlay(DebugOverlay),
    /// Request a secure offscreen screenshot.
    #[serde(rename = "captureScreenshot")]
    CaptureScreenshot(EmptyParams),
    /// Capture pixels, snapshot, frames, and optional point inspection atomically.
    #[serde(rename = "captureCase")]
    CaptureCase(CaptureCaseParams),
}

/// Runtime-controlled overlay painted by the inspected application.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DebugOverlay {
    /// Draw node layout bounds.
    pub layout: bool,
    /// Draw effective and owner-local clips.
    pub clips: bool,
    /// Draw the current hit target.
    pub hit_target: bool,
    /// Draw and retain one selected node identifier.
    pub selected_node: Option<NodeKey>,
}

impl DebugOverlay {
    /// Whether any overlay layer is active.
    pub fn is_enabled(self) -> bool {
        self.layout || self.clips || self.hit_target || self.selected_node.is_some()
    }
}

/// UI-thread-owned snapshots and asynchronous DevTools handshake state.
pub struct DebugState {
    snapshot: DebugSnapshot,
    frames: VecDeque<DebugFrame>,
    trace_capacity: usize,
    capture_directory: Option<tempfile::TempDir>,
    completed_captures: VecDeque<PathBuf>,
    capture_retention: usize,
    screenshot_request: Option<(PathBuf, fs::File)>,
    pending_screenshot: Option<PathBuf>,
    screenshot_sequence: u64,
    screenshot_result: Option<Result<PathBuf, String>>,
    capture_case_requested: bool,
    capture_case_point: Option<(f32, f32)>,
    capture_case_result: Option<Result<DebugCaptureCase, String>>,
    wake: Option<Arc<dyn Fn() + Send + Sync>>,
    raw_frames: bool,
    overlay: DebugOverlay,
    overlay_paint: DebugOverlayPaintStats,
    overlay_changed: bool,
}

/// Thread-safe handle shared with the read-only socket server.
pub type SharedDebugState = Arc<RwLock<DebugState>>;

/// RAII handle that stops the local server and removes its socket on drop.
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
            capture_directory: None,
            completed_captures: VecDeque::new(),
            capture_retention: DEFAULT_CAPTURE_RETENTION,
            screenshot_request: None,
            pending_screenshot: None,
            screenshot_sequence: 0,
            screenshot_result: None,
            capture_case_requested: false,
            capture_case_point: None,
            capture_case_result: None,
            wake: None,
            raw_frames: std::env::var_os("WABOU_DEVTOOLS_RAW_FRAMES").is_some(),
            overlay: DebugOverlay::default(),
            overlay_paint: DebugOverlayPaintStats::default(),
            overlay_changed: false,
        }
    }
}

impl DebugState {
    /// Construct a default state behind an `Arc<RwLock<_>>`.
    pub fn shared() -> SharedDebugState {
        Arc::new(RwLock::new(Self::default()))
    }

    /// Replace the immutable snapshot visible to readers.
    pub fn publish(&mut self, mut snapshot: DebugSnapshot) {
        snapshot.status.overlay = self.overlay;
        snapshot.status.overlay_paint = self.overlay_paint;
        self.snapshot = snapshot;
    }

    /// Append frame metadata, evicting the oldest entry at capacity.
    pub fn push_frame(&mut self, mut frame: DebugFrame) {
        if !self.raw_frames {
            frame.bytes_hex = None;
        }
        if self.frames.len() == self.trace_capacity {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);
    }

    /// Borrow the most recently published retained snapshot.
    pub fn snapshot(&self) -> &DebugSnapshot {
        &self.snapshot
    }

    /// Return the currently requested native overlay layers.
    pub fn overlay(&self) -> DebugOverlay {
        self.overlay
    }

    /// Return evidence from the most recent native overlay paint pass.
    pub fn overlay_paint(&self) -> DebugOverlayPaintStats {
        self.overlay_paint
    }

    /// Publish counts collected while appending the overlay to a native scene.
    pub fn record_overlay_paint(&mut self, mut stats: DebugOverlayPaintStats) {
        stats.sequence = self.overlay_paint.sequence.wrapping_add(1).max(1);
        self.overlay_paint = stats;
        self.snapshot.status.overlay_paint = stats;
    }

    /// Replace overlay configuration and wake the UI loop when it changed.
    pub fn set_overlay(&mut self, overlay: DebugOverlay) -> bool {
        if self.overlay == overlay {
            return false;
        }
        self.overlay = overlay;
        self.overlay_paint = DebugOverlayPaintStats {
            sequence: self.overlay_paint.sequence,
            ..Default::default()
        };
        self.snapshot.status.overlay = overlay;
        self.snapshot.status.overlay_paint = self.overlay_paint;
        self.overlay_changed = true;
        if let Some(wake) = &self.wake {
            wake();
        }
        true
    }

    /// Consume the flag indicating overlay paint state changed.
    pub fn take_overlay_change(&mut self) -> bool {
        std::mem::take(&mut self.overlay_changed)
    }

    /// Borrow retained bridge-frame metadata from oldest to newest.
    pub fn frames(&self) -> &VecDeque<DebugFrame> {
        &self.frames
    }

    /// Install the callback used to wake the UI loop for async requests.
    pub fn set_wake(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.wake = Some(wake);
    }

    /// Request a screenshot and return its preallocated secure temporary path.
    pub fn request_screenshot(&mut self) -> Result<PathBuf, String> {
        self.begin_screenshot(false, None)
    }

    fn begin_screenshot(
        &mut self,
        capture_case: bool,
        point: Option<(f32, f32)>,
    ) -> Result<PathBuf, String> {
        if let Some(path) = &self.pending_screenshot {
            return Err(format!(
                "a DevTools capture is already in progress: {}",
                path.display()
            ));
        }
        self.screenshot_sequence = self.screenshot_sequence.wrapping_add(1);
        if self.capture_directory.is_none() {
            let directory = tempfile::Builder::new()
                .prefix(&format!("wabou-captures-{}-", std::process::id()))
                .tempdir()
                .map_err(|error| format!("cannot create private capture directory: {error}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt as _;
                fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700))
                    .map_err(|error| format!("cannot secure private capture directory: {error}"))?;
            }
            self.capture_directory = Some(directory);
        }
        let directory = self
            .capture_directory
            .as_ref()
            .expect("capture directory initialized above");
        let temporary = tempfile::Builder::new()
            .prefix(&format!(
                "wabou-screenshot-{}-{}-{}-",
                std::process::id(),
                self.snapshot.status.revision,
                self.screenshot_sequence,
            ))
            .suffix(".png")
            .tempfile_in(directory.path())
            .map_err(|error| format!("cannot reserve secure screenshot file: {error}"))?;
        let (file, path) = temporary
            .keep()
            .map_err(|error| format!("cannot retain secure screenshot file: {error}"))?;
        self.screenshot_result = None;
        self.capture_case_result = None;
        self.capture_case_requested = capture_case;
        self.capture_case_point = point;
        self.pending_screenshot = Some(path.clone());
        self.screenshot_request = Some((path.clone(), file));
        if let Some(wake) = &self.wake {
            wake();
        }
        Ok(path)
    }

    /// Request an atomic diagnostic case with an optional inspected point.
    pub fn request_capture_case(&mut self, point: Option<(f32, f32)>) -> Result<PathBuf, String> {
        self.begin_screenshot(true, point)
    }

    /// Drain the screenshot path the renderer should fulfill.
    pub fn take_screenshot_request(&mut self) -> Option<(PathBuf, fs::File)> {
        self.screenshot_request.take()
    }

    /// Whether a renderer screenshot request is waiting.
    pub fn has_screenshot_request(&self) -> bool {
        self.screenshot_request.is_some()
    }

    /// Publish renderer completion and finalize a pending capture case.
    pub fn complete_screenshot(
        &mut self,
        requested_path: &std::path::Path,
        result: Result<PathBuf, String>,
    ) -> Result<(), String> {
        if self.pending_screenshot.as_deref() != Some(requested_path) {
            let _ = fs::remove_file(requested_path);
            return Err(format!(
                "ignored completion for stale DevTools capture: {}",
                requested_path.display()
            ));
        }
        self.pending_screenshot = None;
        self.screenshot_request = None;
        let result = result.and_then(|path| {
            if path == requested_path {
                Ok(path)
            } else {
                Err(format!(
                    "renderer completed capture at unexpected path {}; reserved {}",
                    path.display(),
                    requested_path.display()
                ))
            }
        });
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
        if result.is_err() {
            let _ = fs::remove_file(requested_path);
        } else {
            self.completed_captures.push_back(requested_path.to_owned());
            while self.completed_captures.len() > self.capture_retention.max(1) {
                if let Some(expired) = self.completed_captures.pop_front() {
                    let _ = fs::remove_file(expired);
                }
            }
        }
        self.screenshot_result = Some(result);
        Ok(())
    }

    /// Cancel a capture only when it still belongs to the given requester.
    pub fn cancel_screenshot(&mut self, requested_path: &std::path::Path) -> bool {
        if self.pending_screenshot.as_deref() != Some(requested_path) {
            return false;
        }
        self.pending_screenshot = None;
        self.screenshot_request = None;
        self.capture_case_requested = false;
        self.capture_case_point = None;
        let _ = fs::remove_file(requested_path);
        true
    }

    /// Borrow the most recent screenshot completion.
    pub fn screenshot_result(&self) -> Option<&Result<PathBuf, String>> {
        self.screenshot_result.as_ref()
    }

    /// Borrow the most recent atomic capture-case completion.
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

    /// Validate the most recently published snapshot without mutating it.
    pub fn validation_report(&self) -> DebugValidationReport {
        validation::validate(&self.snapshot)
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
            DebugCommand::ValidateSnapshot(_) => {
                serde_json::to_value(self.validation_report()).map_err(|e| e.to_string())
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
/// Newline-delimited JSON request envelope.
pub struct Request {
    /// Client-assigned correlation identifier.
    pub id: u64,
    /// Requested method and typed parameters.
    #[serde(flatten)]
    pub command: DebugCommand,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Newline-delimited JSON response envelope.
pub struct Response {
    /// Correlation identifier copied from [`Request::id`].
    pub id: u64,
    /// Successful JSON result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Human-readable failure diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

fn configured_socket_path() -> Option<PathBuf> {
    std::env::var_os("WABOU_DEVTOOLS_SOCKET").map(PathBuf::from)
}

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, PathBuf::from)
}

/// Return this process's configured or default local socket path.
pub fn socket_path() -> PathBuf {
    if let Some(path) = configured_socket_path() {
        return path;
    }
    runtime_dir().join(format!("wabou-{}.sock", std::process::id()))
}

/// Find the most recently created live-looking Wabou socket. Explicit
/// `WABOU_DEVTOOLS_SOCKET` always wins, which is required when multiple apps
/// are under test concurrently.
pub fn discover_socket() -> Result<PathBuf, String> {
    #[cfg(unix)]
    use std::os::unix::fs::FileTypeExt;
    if let Some(path) = configured_socket_path() {
        return Ok(path);
    }
    let base = runtime_dir();
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
        }
        #[cfg(unix)]
        let _ = fs::remove_file(path);
        #[cfg(not(unix))]
        return Ok(path);
    }
    Err(format!(
        "no live Wabou DevTools socket found in {}; start the application with `wabou dev` (direct Cargo runs must enable the `wabou/devtools` feature)",
        base.display()
    ))
}

#[cfg(unix)]
fn execute_capture(state: &SharedDebugState, command: &DebugCommand) -> Result<Value, String> {
    execute_capture_with_timeout(state, command, std::time::Duration::from_secs(10))
}

#[cfg(unix)]
fn execute_capture_with_timeout(
    state: &SharedDebugState,
    command: &DebugCommand,
    timeout: std::time::Duration,
) -> Result<Value, String> {
    let point = match command {
        DebugCommand::CaptureCase(params) => params.point(),
        DebugCommand::CaptureScreenshot(_) => Ok(None),
        _ => unreachable!("capture helper only accepts capture commands"),
    }?;
    let capture_case = matches!(command, DebugCommand::CaptureCase(_));
    let path = state
        .write()
        .map_err(|_| "debug state poisoned".to_string())
        .and_then(|mut state| {
            if capture_case {
                state.request_capture_case(point)
            } else {
                state.request_screenshot()
            }
        })?;
    let deadline = std::time::Instant::now() + timeout;
    loop {
        let result = {
            let state = state
                .read()
                .map_err(|_| "debug state poisoned".to_string())?;
            if capture_case {
                state.capture_case_result().cloned().map(|result| {
                    result.and_then(|capture| {
                        serde_json::to_value(capture).map_err(|error| error.to_string())
                    })
                })
            } else {
                state
                    .screenshot_result()
                    .cloned()
                    .map(|result| result.map(|path| json!({"path": path})))
            }
        };
        if let Some(result) = result {
            return result;
        }
        if std::time::Instant::now() >= deadline {
            if let Ok(mut state) = state.write() {
                state.cancel_screenshot(&path);
            }
            return Err(format!(
                "screenshot timed out; requested {}",
                path.display()
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[cfg(unix)]
fn execute_request(state: &SharedDebugState, request: Request) -> Response {
    let outcome = if matches!(
        &request.command,
        DebugCommand::CaptureScreenshot(_) | DebugCommand::CaptureCase(_)
    ) {
        execute_capture(state, &request.command)
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

#[cfg(unix)]
fn serve_stream(state: &SharedDebugState, mut stream: std::os::unix::net::UnixStream) {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(1)));
    let Ok(reader_stream) = stream.try_clone() else {
        return;
    };
    for line in BufReader::new(reader_stream).lines() {
        let response = match line.map_err(|error| error.to_string()).and_then(|line| {
            if line.len() > MAX_REQUEST_BYTES {
                Err("DevTools request exceeds 1 MiB".to_string())
            } else {
                serde_json::from_str::<Request>(&line).map_err(|error| error.to_string())
            }
        }) {
            Ok(request) => execute_request(state, request),
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

#[cfg(unix)]
/// Starts the local DevTools server on the Unix socket at `path`.
///
/// The socket is created with owner-only permissions. An existing live socket
/// or a non-socket filesystem entry is never replaced.
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
                let Ok(stream) = stream else { continue };
                serve_stream(&state, stream);
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
/// Reports that the DevTools transport is not implemented on this platform.
pub fn serve(_state: SharedDebugState, _path: PathBuf) -> std::io::Result<ServerHandle> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "named pipes are not implemented yet",
    ))
}

#[cfg(unix)]
/// Sends one DevTools request to `path` and waits for its response.
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
/// Reports that the DevTools transport is not implemented on this platform.
pub fn call(_path: &Path, _request: &Request) -> Result<Response, String> {
    Err("named pipes are not implemented yet".into())
}

/// Encodes at most `max` bytes as lowercase hexadecimal text.
pub fn bytes_hex(bytes: &[u8], max: usize) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len().min(max) * 2);
    for byte in bytes.iter().take(max) {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

/// Constructs a typed DevTools request from a method name and JSON parameters.
///
/// # Panics
///
/// Panics when `method` and `params` do not match a known DevTools command.
pub fn request(id: u64, method: impl Into<String>, params: Value) -> Request {
    serde_json::from_value(json!({
        "id": id,
        "method": method.into(),
        "params": params,
    }))
    .expect("known Wabou DevTools method and valid params")
}

/// Returns the empty JSON object used by parameterless DevTools commands.
pub fn empty_params() -> Value {
    json!({})
}

#[cfg(test)]
mod tests;
