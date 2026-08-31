//! Backend-neutral window, input, host-action, and frame diagnostic contracts.

use std::{path::PathBuf, sync::Arc};

pub use wabou_accessibility::{
    SemanticAction, SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole, SemanticSnapshot,
    SemanticStates, SemanticToggleState,
};

/// Logical pixel delta emitted for one discrete mouse-wheel line.
///
/// Consumers with line-based scrolling should accumulate pixel deltas against
/// this value so high-resolution trackpads and discrete wheels share one unit.
pub const WHEEL_LINE_DELTA: f64 = 40.0;
#[derive(Debug, Clone, Copy, PartialEq)]
/// Current native window state delivered to a frame source.
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
    /// Outer-window horizontal desktop coordinate, when the platform exposes it.
    pub outer_x: Option<i32>,
    /// Outer-window vertical desktop coordinate, when the platform exposes it.
    pub outer_y: Option<i32>,
    /// Whether the compositor reports the window as completely hidden.
    pub occluded: bool,
    /// Current native light/dark preference, when reported by the platform.
    pub color_scheme: Option<ColorScheme>,
    /// Whether the platform requests reduced non-essential motion.
    pub reduced_motion: bool,
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
            outer_x: None,
            outer_y: None,
            occluded: false,
            color_scheme: Some(ColorScheme::Light),
            reduced_motion: false,
        }
    }
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
    /// Pointer entered the native window.
    Enter,
    /// Pointer moved without changing button state.
    Move,
    /// A button was pressed.
    Down,
    /// A button was released.
    Up,
    /// The platform cancelled the active pointer sequence.
    Cancel,
    /// Pointer left the native window.
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// Stable identity of one active native pointer, represented as FFI-safe words.
pub struct PointerId {
    /// Low 32 bits.
    pub lo: u32,
    /// High 32 bits, including a source namespace.
    pub hi: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Native pointer category exposed without DOM compatibility aliases.
pub enum PointerType {
    /// Conventional mouse or mouse-compatible pointing device.
    Mouse,
    /// Direct touchscreen contact.
    Touch,
    /// Tablet pen, eraser, brush, or similar tool.
    Pen,
    /// Platform source which cannot be classified reliably.
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Device properties accompanying a pointer transition.
pub struct PointerProperties {
    /// Identity stable for the active pointer sequence.
    pub id: PointerId,
    /// Native pointer category.
    pub pointer_type: PointerType,
    /// Whether this is the primary pointer for its category.
    pub primary: bool,
    /// Normalized contact pressure, when reported by the platform.
    pub pressure: Option<f64>,
    /// Normalized tablet barrel pressure in the range -1 to 1.
    pub tangential_pressure: Option<f64>,
    /// Tablet tilt around the surface Y-Z plane, in degrees.
    pub tilt_x: Option<f64>,
    /// Tablet tilt around the surface X-Z plane, in degrees.
    pub tilt_y: Option<f64>,
    /// Clockwise tablet-tool rotation in degrees.
    pub twist: Option<f64>,
}

impl Default for PointerProperties {
    fn default() -> Self {
        Self {
            id: PointerId { lo: 1, hi: 0 },
            pointer_type: PointerType::Mouse,
            primary: true,
            pressure: None,
            tangential_pressure: None,
            tilt_x: None,
            tilt_y: None,
            twist: None,
        }
    }
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
    /// Pointer identity, category, and optional contact/tool measurements.
    pub properties: PointerProperties,
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
    /// Whether the native source reported discrete lines or precise pixels.
    pub delta_mode: WheelDeltaMode,
    /// Native scroll lifecycle phase.
    pub phase: GesturePhase,
    /// Keyboard modifiers active for the event.
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Native granularity of a wheel delta.
pub enum WheelDeltaMode {
    /// A discrete mouse-wheel notch, normalized to logical pixels by the shell.
    Line,
    /// A precise trackpad or high-resolution wheel delta in logical pixels.
    Pixel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Lifecycle phase shared by native continuous gestures.
pub enum GesturePhase {
    /// The gesture has started.
    Started,
    /// The gesture changed since its previous update.
    Changed,
    /// The gesture ended normally.
    Ended,
    /// The platform cancelled the gesture.
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Native trackpad or touchscreen gesture kept independent from DOM events.
pub enum GestureEvent {
    /// Relative magnification; positive values zoom in.
    Pinch {
        /// Relative magnification since the previous update.
        delta: f64,
        /// Current gesture lifecycle phase.
        phase: GesturePhase,
    },
    /// Relative translation in logical window pixels.
    Pan {
        /// Horizontal logical-pixel change.
        delta_x: f64,
        /// Vertical logical-pixel change.
        delta_y: f64,
        /// Current gesture lifecycle phase.
        phase: GesturePhase,
    },
    /// Relative counter-clockwise rotation in degrees.
    Rotation {
        /// Relative counter-clockwise rotation in degrees.
        delta: f64,
        /// Current gesture lifecycle phase.
        phase: GesturePhase,
    },
    /// Platform smart-zoom/double-tap gesture.
    DoubleTap,
    /// Force-touch pressure and platform click stage.
    Pressure {
        /// Normalized force in the inclusive range zero to one.
        pressure: f64,
        /// Platform click stage.
        stage: i64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Operating-system application lifecycle notification.
pub enum AppLifecycleEvent {
    /// The application became active and may resume foreground work.
    Resumed,
    /// The application should pause foreground work.
    Suspended,
    /// The operating system requested that caches be reduced promptly.
    MemoryWarning,
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
    /// Whether the platform synthesized this transition during focus recovery.
    pub synthetic: bool,
}

/// Editing shortcuts whose physical bindings follow platform conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardShortcut {
    /// Copy the current selection.
    Copy,
    /// Cut the current selection.
    Cut,
    /// Paste clipboard contents.
    Paste,
    /// Select all editable/selectable content.
    SelectAll,
}

impl KeyEvent {
    /// Match a platform-standard editing shortcut. Physical combinations such
    /// as terminal Control+C should continue to inspect `modifiers` directly.
    pub fn matches_standard_shortcut(&self, shortcut: StandardShortcut) -> bool {
        let key = match shortcut {
            StandardShortcut::Copy => "c",
            StandardShortcut::Cut => "x",
            StandardShortcut::Paste => "v",
            StandardShortcut::SelectAll => "a",
        };
        self.phase == KeyPhase::Down
            && self.modifiers.primary_shortcut()
            && !self.modifiers.shift()
            && (self.key_without_modifiers.eq_ignore_ascii_case(key)
                || self.key.eq_ignore_ascii_case(key))
    }
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
/// Input and window-state events delivered to a frame source or widget.
pub enum UiEvent {
    /// Operating-system application lifecycle notification.
    AppLifecycle(AppLifecycleEvent),
    /// Authoritative physical modifier-key state changed.
    ModifiersChanged(Modifiers),
    /// Pointer movement or button transition.
    Pointer(PointerEvent),
    /// Wheel or trackpad scrolling.
    Wheel(WheelEvent),
    /// Native trackpad or touchscreen gesture.
    Gesture(GestureEvent),
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
/// each frame for a perf overlay
/// (`useHost().diagnostics.frameStats()`). All timing is captured at the app/shell seam.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FrameStats {
    /// Total frame-source construction time in milliseconds.
    pub build_frame_ms: f64,
    /// Guest JavaScript frame callback time in milliseconds.
    pub js_tick_ms: f64,
    /// Backend-neutral scene assembly time in milliseconds.
    pub scene_ms: f64,
    /// Renderer submission and presentation time in milliseconds, or zero when
    /// the active native toolkit does not expose a reliable completion time.
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

    /// Blend a GPUI frame sample, including the JavaScript flush and viewport.
    pub fn update_gpui(
        &mut self,
        build_frame_ms: f64,
        js_tick_ms: f64,
        scene_ms: f64,
        node_count: usize,
        viewport: (u32, u32),
    ) {
        let prev = 1.0 - Self::ALPHA;
        self.build_frame_ms = self.build_frame_ms * prev + build_frame_ms * Self::ALPHA;
        self.js_tick_ms = self.js_tick_ms * prev + js_tick_ms * Self::ALPHA;
        self.scene_ms = self.scene_ms * prev + scene_ms * Self::ALPHA;
        // GPUI-CE does not currently expose a per-window present-completion
        // timestamp to application views. Zero means unavailable, not free.
        self.present_ms = 0.0;
        self.node_count = node_count;
        self.viewport_w = viewport.0;
        self.viewport_h = viewport.1;
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
