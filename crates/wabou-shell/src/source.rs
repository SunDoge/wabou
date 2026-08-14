//! The frame-producer contract a host implements to drive a [`crate::Shell`].
//!
//! Each frame, the app calls `build_frame` to get the flattened layout list,
//! then renders+presents. `has_anim` gates the continuous-redraw loop: a
//! reactive source (e.g. the SolidJS applier) returns true while rAF callbacks
//! are queued; a static source returns false so the loop idles until a resize.

use vello::peniko::Color;

pub use wabou_accessibility::{SemanticAction, SemanticNode, SemanticRole, SemanticSnapshot};

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use crate::layout::PlacedNode;
use crate::text::TextContext;
use vello::Scene;

/// Logical pixel delta emitted for one discrete mouse-wheel line.
///
/// Consumers with line-based scrolling should accumulate pixel deltas against
/// this value so high-resolution trackpads and discrete wheels share one unit.
pub const WHEEL_LINE_DELTA: f64 = 40.0;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowOptions {
    pub title: String,
    pub initial_inner_size: (u32, u32),
    pub min_inner_size: Option<(u32, u32)>,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
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
        }
    }
}

impl WindowOptions {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }
    pub fn initial_inner_size(mut self, width: u32, height: u32) -> Self {
        self.initial_inner_size = (width.max(1), height.max(1));
        self
    }
    pub fn min_inner_size(mut self, width: u32, height: u32) -> Self {
        self.min_inner_size = Some((width.max(1), height.max(1)));
        self
    }
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
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WindowMetrics {
    pub window_id: u64,
    pub logical_width: u32,
    pub logical_height: u32,
    pub physical_width: u32,
    pub physical_height: u32,
    pub scale_factor: f64,
    pub maximized: bool,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowCommand {
    Close,
    Minimize,
    SetMaximized(bool),
    SetTitle(String),
    StartDragging,
}

/// Wabou's renderer-independent input model. Web-style events are an adapter
/// concern of individual frame sources, not part of the shell contract.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

bitflags::bitflags! {
    /// Keyboard modifiers active for an input event.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const SHIFT = 1 << 0;
        const CONTROL = 1 << 1;
        const ALT = 1 << 2;
        const META = 1 << 3;
    }
}

impl Modifiers {
    pub const fn shift(self) -> bool {
        self.contains(Self::SHIFT)
    }

    pub const fn control(self) -> bool {
        self.contains(Self::CONTROL)
    }

    pub const fn alt(self) -> bool {
        self.contains(Self::ALT)
    }

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
pub enum PointerButton {
    Primary,
    Auxiliary,
    Secondary,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerPhase {
    Move,
    Down,
    Up,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointerEvent {
    pub phase: PointerPhase,
    pub position: Point,
    pub button: Option<PointerButton>,
    /// DOM-compatible bit layout is deliberately not required here. This is a
    /// native set owned by the shell; adapters translate it at their boundary.
    pub buttons: u32,
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WheelEvent {
    pub position: Point,
    pub delta_x: f64,
    pub delta_y: f64,
    pub modifiers: Modifiers,
}

/// Platform input-method lifecycle delivered to the focused native widget.
/// Cursor offsets and delete ranges are UTF-8 byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeEvent {
    Enabled,
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    Commit(String),
    DeleteSurrounding {
        before_bytes: usize,
        after_bytes: usize,
    },
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPhase {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyLocation {
    #[default]
    Standard,
    Left,
    Right,
    Numpad,
}

impl KeyLocation {
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
pub struct KeyEvent {
    pub phase: KeyPhase,
    pub key: String,
    pub key_without_modifiers: String,
    pub code: String,
    pub text: Option<String>,
    pub text_with_all_modifiers: Option<String>,
    pub location: KeyLocation,
    pub modifiers: Modifiers,
    pub repeat: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiEvent {
    Pointer(PointerEvent),
    Wheel(WheelEvent),
    Key(KeyEvent),
    TextInput(String),
    Ime(ImeEvent),
    Paste(String),
    Focus(bool),
    WindowMetrics(WindowMetrics),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventResponse {
    pub handled: bool,
    pub request_redraw: bool,
    /// Suppress the committed text paired with the current physical key.
    /// This is intentionally independent from `handled`: JS key listeners
    /// must not accidentally swallow text input.
    pub consume_key_text: bool,
    /// `Some(true)` enables platform text input/IME; `Some(false)` disables it.
    /// `None` leaves the current window setting unchanged.
    pub text_input: Option<bool>,
    pub clipboard: Option<ClipboardRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardRequest {
    Write(String),
    Read,
}

/// An asynchronous request from a frame source to its native window host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostAction {
    /// Open a URL with the platform's registered handler.
    OpenUrl(String),
    SetClipboard(String),
    WriteClipboard {
        request_id: u64,
        text: String,
    },
    ReadClipboard {
        request_id: u64,
    },
    /// `None` restores the host's default title.
    SetWindowTitle(Option<String>),
    RequestAttention,
    CreateWindow {
        window_id: u64,
        options: WindowOptions,
    },
    ControlWindow {
        window_id: u64,
        command: WindowCommand,
    },
}

/// Completion for a [`HostAction`] which requires data from the native host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HostActionResult {
    Clipboard {
        request_id: u64,
        text: Option<String>,
    },
    ClipboardWrite {
        request_id: u64,
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
    pub build_frame_ms: f64,
    pub js_tick_ms: f64,
    pub scene_ms: f64,
    pub present_ms: f64,
    pub node_count: usize,
    pub viewport_w: u32,
    pub viewport_h: u32,
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
    pub const IGNORED: Self = Self {
        handled: false,
        request_redraw: false,
        consume_key_text: false,
        text_input: None,
        clipboard: None,
    };

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

pub trait FrameSource {
    /// Inform the source of the physical-pixels-per-logical-pixel ratio before
    /// it builds widget scene fragments for this frame.
    fn set_device_scale(&mut self, _scale: f64) {}

    /// Lay out for `width x height` and return the paint-ordered node list.
    /// Borrowed `tcx` is used for text measurement (parley).
    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode>;

    /// Latest retained accessibility snapshot, in logical window coordinates.
    /// Enable semantic snapshot production while a platform accessibility
    /// client is active. Sources should avoid accessibility tree work when
    /// this is false.
    fn set_semantics_enabled(&mut self, _enabled: bool) {}

    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        None
    }

    fn handle_semantic_action(&mut self, _action: SemanticAction) -> bool {
        false
    }

    /// Focused editor exclusion area for the platform IME candidate window,
    /// expressed in window-logical coordinates.
    fn ime_cursor_area(&self) -> Option<[f64; 4]> {
        None
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

    fn complete_host_action(&mut self, _result: HostActionResult) {}

    /// Drain one typed desktop effect. Unlike render ops, effects represent
    /// OS interaction and may complete asynchronously at a later frame boundary.
    fn take_effect(&mut self) -> Option<crate::EffectRequest> {
        None
    }

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
    fn take_screenshot_request(&mut self) -> Option<PathBuf> {
        None
    }

    fn complete_screenshot(&mut self, _result: Result<PathBuf, String>) {}
}

#[cfg(test)]
mod tests {
    use super::{Modifiers, WindowOptions};

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
            .transparent(true);
        assert_eq!(options.title, "Inspector");
        assert_eq!(options.initial_inner_size, (1440, 900));
        assert_eq!(options.min_inner_size, Some((960, 600)));
        assert!(!options.resizable);
        assert!(!options.decorations);
        assert!(options.transparent);
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
