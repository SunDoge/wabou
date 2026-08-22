//! The frame-producer contract a host implements to drive a [`crate::Shell`].
//!
//! Each frame, the app calls `build_frame` to get the flattened layout list,
//! then renders+presents. `has_anim` gates the continuous-redraw loop: a
//! reactive source (e.g. the SolidJS applier) returns true while rAF callbacks
//! are queued; a static source returns false so the loop idles until a resize.

#![warn(missing_docs)]

use vello::peniko::Color;

pub use wabou_accessibility::{
    SemanticAction, SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole, SemanticSnapshot,
    SemanticStates, SemanticToggleState,
};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::layout::PlacedNode;
use crate::style::CursorStyle;
use crate::text::TextContext;
use vello::Scene;

/// Logical pixel delta emitted for one discrete mouse-wheel line.
///
/// Consumers with line-based scrolling should accumulate pixel deltas against
/// this value so high-resolution trackpads and discrete wheels share one unit.
pub const WHEEL_LINE_DELTA: f64 = 40.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// Initial stacking level requested for a native window.
pub enum WindowLevel {
    /// Keep the window below ordinary application windows where supported.
    AlwaysOnBottom,
    /// Use the platform's ordinary application-window level.
    #[default]
    Normal,
    /// Keep the window above ordinary application windows where supported.
    ///
    /// Wayland compositors generally do not expose this operation.
    AlwaysOnTop,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// Whether a native window participates in pointer hit testing.
pub enum WindowInputMode {
    /// Deliver pointer input to the window normally.
    #[default]
    Interactive,
    /// Let pointer input pass through to windows underneath where supported.
    Passthrough,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// Requested properties used when creating a native window.
///
/// These are initial requests rather than live state; use [`WindowMetrics`]
/// for the values the platform actually applied.
pub struct WindowOptions {
    /// Initial native window title.
    pub title: String,
    /// Requested logical client-area size.
    pub initial_inner_size: (u32, u32),
    /// Optional minimum logical client-area size.
    pub min_inner_size: Option<(u32, u32)>,
    /// Whether the user may resize the window.
    pub resizable: bool,
    /// Whether the OS draws its standard title bar and border.
    pub decorations: bool,
    /// Whether the native surface preserves alpha.
    pub transparent: bool,
    /// Requested initial stacking level.
    pub window_level: WindowLevel,
    /// Requested initial pointer hit-test behavior.
    pub input_mode: WindowInputMode,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "wabou".into(),
            initial_inner_size: (800, 600),
            min_inner_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            window_level: WindowLevel::Normal,
            input_mode: WindowInputMode::Interactive,
        }
    }
}

impl WindowOptions {
    /// Construct options with platform-friendly defaults.
    pub fn new() -> Self {
        Self::default()
    }
    /// Set the initial native window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    /// Set the requested logical client-area size, clamped to at least 1×1.
    pub fn initial_inner_size(mut self, width: u32, height: u32) -> Self {
        self.initial_inner_size = (width.max(1), height.max(1));
        self
    }
    /// Set the minimum logical client-area size, clamped to at least 1×1.
    pub fn min_inner_size(mut self, width: u32, height: u32) -> Self {
        self.min_inner_size = Some((width.max(1), height.max(1)));
        self
    }
    /// Control whether the user may resize the window.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
    /// Enable or disable native window-manager decorations.
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }
    /// Request a native window whose background preserves rendered alpha.
    ///
    /// The frame source must also use a transparent base color and avoid
    /// painting an opaque root background for transparency to be visible.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }
    /// Set the initial native stacking level.
    pub fn window_level(mut self, window_level: WindowLevel) -> Self {
        self.window_level = window_level;
        self
    }
    /// Set the initial pointer hit-test behavior.
    pub fn input_mode(mut self, input_mode: WindowInputMode) -> Self {
        self.input_mode = input_mode;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Current native window state delivered to a [`FrameSource`].
pub struct WindowMetrics {
    /// Stable Wabou window identifier, independent of platform handles.
    pub window_key: crate::WindowResourceKey,
    /// Logical client-area width.
    pub logical_width: u32,
    /// Logical client-area height.
    pub logical_height: u32,
    /// Physical surface width.
    pub physical_width: u32,
    /// Physical surface height.
    pub physical_height: u32,
    /// Physical pixels per logical pixel.
    pub scale_factor: f64,
    /// Whether the native window is maximized.
    pub maximized: bool,
    /// Whether the native window owns keyboard focus.
    pub focused: bool,
    /// Current native light/dark preference, when reported by the platform.
    pub color_scheme: Option<ColorScheme>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Platform color preference associated with a native window.
pub enum ColorScheme {
    /// Prefer a light application palette.
    Light,
    /// Prefer a dark application palette.
    Dark,
}

impl Default for WindowMetrics {
    fn default() -> Self {
        Self {
            window_key: crate::initial_window_resource_key(0),
            logical_width: 0,
            logical_height: 0,
            physical_width: 0,
            physical_height: 0,
            scale_factor: 1.0,
            maximized: false,
            focused: false,
            color_scheme: Some(ColorScheme::Light),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
/// Command addressed to a Wabou-managed native window.
pub enum WindowCommand {
    /// Request that the window close.
    Close,
    /// Minimize the window.
    Minimize,
    /// Set or clear the maximized state.
    SetMaximized(bool),
    /// Replace the native window title.
    SetTitle(String),
    /// Begin an OS-managed window drag from the current pointer gesture.
    StartDragging,
    /// Restore a hidden or surface-released logical window.
    Show,
}

/// Wabou's renderer-independent input model. Web-style events are an adapter
/// concern of individual frame sources, not part of the shell contract.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    /// Horizontal logical coordinate.
    pub x: f64,
    /// Vertical logical coordinate.
    pub y: f64,
}

bitflags::bitflags! {
    /// Keyboard modifiers active for an input event.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        /// Shift is held.
        const SHIFT = 1 << 0;
        /// Control is held.
        const CONTROL = 1 << 1;
        /// Alt/Option is held.
        const ALT = 1 << 2;
        /// Meta/Command/Windows is held.
        const META = 1 << 3;
    }
}

impl Modifiers {
    /// Whether Shift is held.
    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    /// Whether Control is held.
    pub const fn control(self) -> bool {
        self.contains(Self::CONTROL)
    }

    /// Whether Alt/Option is held.
    pub const fn alt(self) -> bool {
        self.contains(Self::ALT)
    }

    /// Whether Meta/Command/Windows is held.
    pub const fn meta(self) -> bool {
        self.contains(Self::META)
    }

    /// Platform primary shortcut modifier (Ctrl or Meta), excluding AltGr.
    pub const fn primary_shortcut(self) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.meta() && !self.control() && !self.alt()
        }
        #[cfg(not(target_os = "macos"))]
        {
            self.control() && !self.meta() && !self.alt()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Pointer button independent of winit and DOM numbering.
pub enum PointerButton {
    /// Main activation button, normally left.
    Primary,
    /// Auxiliary button, normally middle.
    Auxiliary,
    /// Context-menu button, normally right.
    Secondary,
    /// Platform button not covered by the standard variants.
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Lifecycle phase of a pointer interaction.
pub enum PointerPhase {
    /// Pointer moved without changing button state.
    Move,
    /// A button was pressed.
    Down,
    /// A button was released.
    Up,
    /// The platform cancelled the active pointer sequence.
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Pointer event in logical window or widget-local coordinates.
pub struct PointerEvent {
    /// Interaction phase.
    pub phase: PointerPhase,
    /// Pointer position in the receiver's documented coordinate space.
    pub position: Point,
    /// Button changed by this event, if any.
    pub button: Option<PointerButton>,
    /// DOM-compatible bit layout is deliberately not required here. This is a
    /// native set owned by the shell; adapters translate it at their boundary.
    pub buttons: u32,
    /// Keyboard modifiers active for the event.
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Pixel-normalized wheel or trackpad event.
pub struct WheelEvent {
    /// Pointer position in the receiver's documented coordinate space.
    pub position: Point,
    /// Horizontal logical-pixel delta.
    pub delta_x: f64,
    /// Vertical logical-pixel delta.
    pub delta_y: f64,
    /// Keyboard modifiers active for the event.
    pub modifiers: Modifiers,
}

/// Platform input-method lifecycle delivered to the focused native widget.
/// Cursor offsets and delete ranges are UTF-8 byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeEvent {
    /// Platform text input was enabled for the focused control.
    Enabled,
    /// Replace the active preedit text without committing it.
    Preedit {
        /// Current preedit string.
        text: String,
        /// Selected byte range within `text`, if supplied by the IME.
        cursor: Option<(usize, usize)>,
    },
    /// Commit text and finish the active composition.
    Commit(String),
    /// Delete UTF-8 bytes around the insertion point before the next update.
    DeleteSurrounding {
        /// Bytes immediately before the insertion point.
        before_bytes: usize,
        /// Bytes immediately after the insertion point.
        after_bytes: usize,
    },
    /// Platform text input was disabled for the control.
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Physical key transition phase.
pub enum KeyPhase {
    /// Key was pressed or auto-repeated.
    Down,
    /// Key was released.
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Physical location of a key with duplicated legends.
pub enum KeyLocation {
    #[default]
    /// Main section or a key without a meaningful side.
    Standard,
    /// Left-hand instance of a modifier key.
    Left,
    /// Right-hand instance of a modifier key.
    Right,
    /// Numeric keypad.
    Numpad,
}

impl KeyLocation {
    /// Return the DOM-compatible numeric `location` value.
    pub const fn dom_code(self) -> u8 {
        match self {
            Self::Standard => 0,
            Self::Left => 1,
            Self::Right => 2,
            Self::Numpad => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Keyboard event retaining both logical and physical-key information.
pub struct KeyEvent {
    /// Press/release phase.
    pub phase: KeyPhase,
    /// Logical key after applying active modifiers.
    pub key: String,
    /// Logical key with shortcut modifiers removed.
    pub key_without_modifiers: String,
    /// Physical-key code suitable for shortcut identity.
    pub code: String,
    /// Text the key would commit under normal text-input handling.
    pub text: Option<String>,
    /// Text produced when all modifiers are included.
    pub text_with_all_modifiers: Option<String>,
    /// Physical section containing the key.
    pub location: KeyLocation,
    /// Active keyboard modifiers.
    pub modifiers: Modifiers,
    /// Whether this is an automatic repeat rather than the initial press.
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Phase of a file drag entering, leaving, or being dropped on a window.
pub enum FileDropPhase {
    /// One or more native files entered the window.
    Entered,
    /// A native file drag moved within the window.
    Moved,
    /// The native file drag left the window without dropping.
    Left,
    /// A native file was dropped on the window.
    Dropped,
}

#[derive(Debug, Clone, PartialEq)]
/// Window-level native file drag-and-drop event.
pub struct FileDropEvent {
    /// Drag lifecycle phase reported by the window system.
    pub phase: FileDropPhase,
    /// Native paths supplied when the drag enters or drops.
    pub paths: Vec<std::path::PathBuf>,
    /// Pointer position in logical window pixels, when supplied by the platform.
    pub position: Option<Point>,
}

#[derive(Debug, Clone, PartialEq)]
/// Input and window-state events delivered to a [`FrameSource`] or widget.
pub enum UiEvent {
    /// Pointer movement or button transition.
    Pointer(PointerEvent),
    /// Wheel or trackpad scrolling.
    Wheel(WheelEvent),
    /// Physical/logical keyboard transition.
    Key(KeyEvent),
    /// Text committed outside an IME composition.
    TextInput(String),
    /// Input-method lifecycle event.
    Ime(ImeEvent),
    /// Text returned by an asynchronous clipboard read.
    Paste(String),
    /// Element-level focus transition.
    Focus(bool),
    /// A native file was dragged over, away from, or dropped on the window.
    FileDrop(FileDropEvent),
    /// Native window size, scale, or focus state changed.
    WindowMetrics(WindowMetrics),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// Synchronous effects requested while handling a [`UiEvent`].
pub struct EventResponse {
    /// Whether the source consumed the event.
    pub handled: bool,
    /// Whether the next frame may differ and should be scheduled.
    pub request_redraw: bool,
    /// Suppress the committed text paired with the current physical key.
    /// This is intentionally independent from `handled`: JS key listeners
    /// must not accidentally swallow text input.
    pub consume_key_text: bool,
    /// `Some(true)` enables platform text input/IME; `Some(false)` disables it.
    /// `None` leaves the current window setting unchanged.
    pub text_input: Option<bool>,
    /// Clipboard operation to execute after event dispatch.
    pub clipboard: Option<ClipboardRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Clipboard operation requested synchronously by event handling.
pub enum ClipboardRequest {
    /// Replace clipboard text.
    Write(String),
    /// Read text and later deliver it as [`UiEvent::Paste`].
    Read,
}

/// An asynchronous request from a frame source to its native window host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostAction {
    /// Open a URL with the platform's registered handler.
    OpenUrl(String),
    /// Replace clipboard text without requiring an acknowledgement.
    SetClipboard(String),
    /// Replace clipboard text and route completion to its producer.
    WriteClipboard {
        /// Producer-local request identifier.
        request_id: u64,
        /// Text to write.
        text: String,
    },
    /// Read clipboard text and route completion to its producer.
    ReadClipboard {
        /// Producer-local request identifier.
        request_id: u64,
    },
    /// `None` restores the host's default title.
    SetWindowTitle(Option<String>),
    /// Ask the window manager to draw the user's attention.
    RequestAttention,
}

/// Completion for a [`HostAction`] which requires data from the native host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostActionResult {
    /// Completion of [`HostAction::ReadClipboard`].
    Clipboard {
        /// Original producer-local request identifier.
        request_id: u64,
        /// Clipboard text, or `None` when unavailable or unsupported.
        text: Option<String>,
    },
    /// Completion of [`HostAction::WriteClipboard`].
    ClipboardWrite {
        /// Original producer-local request identifier.
        request_id: u64,
        /// Whether the platform accepted the write.
        success: bool,
    },
}

/// Thread-safe callback used by asynchronous frame sources to wake winit's
/// otherwise sleeping event loop.
pub type WakeCallback = Arc<dyn Fn() + Send + Sync>;

/// Per-frame render-stage timings, as an exponential moving average (so a
/// live overlay isn't dominated by a single slow frame). Reported to the host
/// each frame via [`FrameSource::push_frame_stats`] for a perf overlay
/// (`useHost().diagnostics.frameStats()`). All timing is captured at the app/shell seam.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameStats {
    /// Total frame-source construction time in milliseconds.
    pub build_frame_ms: f64,
    /// Guest JavaScript frame callback time in milliseconds.
    pub js_tick_ms: f64,
    /// Vello scene assembly time in milliseconds.
    pub scene_ms: f64,
    /// Renderer submission and presentation time in milliseconds.
    pub present_ms: f64,
    /// Number of retained nodes in the frame.
    pub node_count: usize,
    /// Logical viewport width.
    pub viewport_w: u32,
    /// Logical viewport height.
    pub viewport_h: u32,
}

/// An atomically reserved screenshot artifact fulfilled through its open file.
pub struct ScreenshotRequest {
    /// Stable identity and user-visible location of the capture.
    pub path: PathBuf,
    /// Reserved destination handle; renderers must not reopen `path`.
    pub file: std::fs::File,
}

impl FrameStats {
    const ALPHA: f64 = 0.1;

    /// Blend this frame's raw samples into the EMA and refresh the node count.
    pub fn update(
        &mut self,
        build_frame_ms: f64,
        scene_ms: f64,
        present_ms: f64,
        node_count: usize,
    ) {
        let prev = 1.0 - Self::ALPHA;
        self.build_frame_ms = self.build_frame_ms * prev + build_frame_ms * Self::ALPHA;
        self.scene_ms = self.scene_ms * prev + scene_ms * Self::ALPHA;
        self.present_ms = self.present_ms * prev + present_ms * Self::ALPHA;
        self.node_count = node_count;
    }
}

impl EventResponse {
    /// Response for an event the source did not consume.
    pub const IGNORED: Self = Self {
        handled: false,
        request_redraw: false,
        consume_key_text: false,
        text_input: None,
        clipboard: None,
    };

    /// Consume an event and request a new frame.
    pub const fn handled() -> Self {
        Self {
            handled: true,
            request_redraw: true,
            consume_key_text: false,
            text_input: None,
            clipboard: None,
        }
    }
}

/// Retained UI producer consumed by the native shell.
///
/// All methods run on the UI thread. Background work communicates through
/// [`WakeCallback`] plus the polling/drain methods; it must never call into
/// layout, widgets, or Vello directly.
pub trait FrameSource {
    /// Inform the source of the physical-pixels-per-logical-pixel ratio before
    /// it builds widget scene fragments for this frame.
    fn set_device_scale(&mut self, _scale: f64) {}

    /// Lay out for `width x height` and return the paint-ordered node list.
    /// Borrowed `tcx` is used for text measurement (parley).
    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode>;

    /// Enable semantic snapshot production while a platform accessibility
    /// client is active. Sources should avoid accessibility tree work when
    /// this is false.
    fn set_semantics_enabled(&mut self, _enabled: bool) {}

    /// Return the latest immutable accessibility snapshot.
    ///
    /// Bounds are expressed in logical window coordinates.
    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        None
    }

    /// Route a platform accessibility action back into retained UI state.
    fn handle_semantic_action(&mut self, _action: SemanticAction) -> bool {
        false
    }

    /// Focused editor exclusion area for the platform IME candidate window,
    /// expressed in window-logical coordinates.
    fn ime_cursor_area(&self) -> Option<[f64; 4]> {
        None
    }

    /// Cursor requested by the node currently under the pointer.
    fn pointer_cursor(&self) -> CursorStyle {
        CursorStyle::Default
    }

    /// Paint optional diagnostics after the application scene. Decorations are
    /// deliberately outside the retained tree, so they cannot affect layout,
    /// clipping, or hit testing.
    fn paint_debug_overlay(
        &mut self,
        _scene: &mut Scene,
        _nodes: &[PlacedNode],
        _tcx: &mut TextContext,
        _device_scale: f64,
    ) {
    }

    /// Viewport background color.
    fn base_color(&self) -> Color;

    /// Whether to keep redrawing every vsync. Default `false` (static); a
    /// reactive source overrides to return true while it has pending rAF work.
    fn has_anim(&self) -> bool {
        false
    }

    /// Earliest deferred animation repaint. Unlike `has_anim`, this lets the
    /// event loop sleep until the next frame is actually needed.
    fn animation_deadline(&self) -> Option<Instant> {
        None
    }

    /// Install the host event-loop wake callback. Sources should call this
    /// only after background work has made progress that must be observed on
    /// the UI thread.
    fn set_wake_callback(&mut self, _wake: WakeCallback) {}

    /// Drain asynchronous completions after the event loop was woken. Returns
    /// whether the completion can have changed the next rendered frame.
    fn poll_async(&mut self) -> bool {
        false
    }

    /// Drain one native host action produced by asynchronous work.
    fn take_host_action(&mut self) -> Option<HostAction> {
        None
    }

    /// Deliver the completion of a host action to its original producer.
    fn complete_host_action(&mut self, _result: HostActionResult) {}

    /// Drain one typed desktop effect. Unlike render ops, effects represent
    /// OS interaction and may complete asynchronously at a later frame boundary.
    fn take_effect(&mut self) -> Option<crate::EffectRequest> {
        None
    }

    /// Deliver completion of a typed desktop effect at a frame boundary.
    fn complete_effect(&mut self, _completion: crate::EffectCompletion) {}

    /// Deliver a native Wabou event to the source.
    fn handle_event(&mut self, _event: UiEvent) -> EventResponse {
        EventResponse::IGNORED
    }

    /// Receive the latest per-frame stage timings (EMA) for host-side perf
    /// tooling (e.g. a Host diagnostics overlay). Default: ignore.
    fn push_frame_stats(&mut self, _stats: &FrameStats) {}

    /// DevTools screenshot handshake. The shell renders its current scene to
    /// this path only when requested; normal frames pay no readback cost.
    fn take_screenshot_request(&mut self) -> Option<ScreenshotRequest> {
        None
    }

    /// Report completion of the last screenshot request.
    fn complete_screenshot(
        &mut self,
        _requested_path: &std::path::Path,
        _result: Result<PathBuf, String>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::{Modifiers, WindowInputMode, WindowLevel, WindowOptions};

    #[test]
    fn modifier_flags_match_host_protocol_bits() {
        let modifiers = Modifiers::SHIFT | Modifiers::ALT | Modifiers::META;
        assert_eq!(modifiers.bits(), 0b1101);
        assert!(modifiers.shift());
        assert!(!modifiers.control());
        assert!(modifiers.alt());
        assert!(modifiers.meta());
    }

    #[test]
    fn window_options_distinguish_requested_size_from_live_metrics() {
        let options = WindowOptions::new()
            .title("Inspector")
            .initial_inner_size(1440, 900)
            .min_inner_size(960, 600)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .window_level(WindowLevel::AlwaysOnTop)
            .input_mode(WindowInputMode::Passthrough);
        assert_eq!(options.title, "Inspector");
        assert_eq!(options.initial_inner_size, (1440, 900));
        assert_eq!(options.min_inner_size, Some((960, 600)));
        assert!(!options.resizable);
        assert!(!options.decorations);
        assert!(options.transparent);
        assert_eq!(options.window_level, WindowLevel::AlwaysOnTop);
        assert_eq!(options.input_mode, WindowInputMode::Passthrough);
    }

    #[test]
    fn primary_shortcut_uses_only_the_platform_modifier() {
        let expected = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        };
        let other = if cfg!(target_os = "macos") {
            Modifiers::CONTROL
        } else {
            Modifiers::META
        };

        assert!(expected.primary_shortcut());
        assert!((expected | Modifiers::SHIFT).primary_shortcut());
        assert!(!other.primary_shortcut());
        assert!(!(expected | other).primary_shortcut());
        assert!(!(expected | Modifiers::ALT).primary_shortcut());
    }
}
