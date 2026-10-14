//! Data-only contract for statically linked native widget implementations.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use wabou_host_api::ResourceKey;

pub use crate::widget::{
    WidgetAccessibility, WidgetImePurpose, WidgetImeState, WidgetTextSelection,
    WidgetTextSelectionKind,
};

/// Current callback inputs. Sizes and positions are content-local logical pixels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetInputs {
    /// Rust-owned application state snapshot.
    pub state: Value,
    /// Configuration supplied by the UI.
    pub config: Value,
    /// Content box dimensions.
    pub size: [f32; 2],
    /// Resident native focus state.
    pub focused: bool,
}

/// Native input delivered to a custom widget.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WidgetEvent {
    /// Primary pointer button transition.
    Pointer {
        /// Whether the button was pressed rather than released.
        down: bool,
        /// Content-local logical coordinates.
        position: [f64; 2],
    },
    /// Movement, entry, exit or cancellation.
    PointerMotion {
        /// Current pointer lifecycle phase.
        phase: PointerMotionPhase,
        /// Content-local logical coordinates.
        position: [f64; 2],
        /// Bitset of currently pressed pointer buttons.
        buttons: u32,
        /// Bitset of active keyboard modifiers.
        modifiers: u8,
    },
    /// Auxiliary/context/extra button transition.
    PointerButton {
        /// Whether the button was pressed rather than released.
        down: bool,
        /// Native button identifier.
        button: u32,
        /// Content-local logical coordinates.
        position: [f64; 2],
        /// Bitset of currently pressed pointer buttons.
        buttons: u32,
        /// Bitset of active keyboard modifiers.
        modifiers: u8,
    },
    /// Logical-pixel scroll deltas.
    Wheel {
        /// Content-local logical coordinates.
        position: [f64; 2],
        /// Logical-pixel horizontal and vertical deltas.
        delta: [f64; 2],
        /// Bitset of active keyboard modifiers.
        modifiers: u8,
    },
    /// Physical/logical keyboard transition.
    Key {
        /// Whether the key was pressed rather than released.
        down: bool,
        /// Logical key value.
        key: String,
        /// Physical key code.
        code: String,
        /// Whether this event is an auto-repeat.
        repeat: bool,
        /// Bitset of active keyboard modifiers.
        modifiers: u8,
    },
    /// Committed text outside IME composition.
    TextInput {
        /// Committed UTF-8 text.
        text: String,
    },
    /// Clipboard text delivered by the host.
    Paste {
        /// Clipboard UTF-8 text.
        text: String,
    },
    /// Input-method lifecycle.
    Ime {
        /// Input-method lifecycle event.
        event: WidgetIme,
    },
    /// Focus transition.
    Focus {
        /// Current native focus state.
        focused: bool,
    },
}

/// Pointer motion lifecycle independent of the native backend.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PointerMotionPhase {
    /// Pointer entered the widget.
    Enter,
    /// Pointer moved within the widget.
    Move,
    /// Active pointer interaction was cancelled.
    Cancel,
    /// Pointer left the widget.
    Leave,
}

/// Input-method lifecycle independent of the native backend.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WidgetIme {
    /// Platform input method was enabled.
    Enabled,
    /// Composition text changed.
    Preedit {
        /// Current composition text.
        text: String,
        /// Optional UTF-8 byte selection within the composition.
        cursor: Option<(usize, usize)>,
    },
    /// Composition text was committed.
    Commit {
        /// Committed UTF-8 text.
        text: String,
    },
    /// Delete bytes around the current cursor.
    DeleteSurrounding {
        /// Number of UTF-8 bytes before the cursor.
        before_bytes: usize,
        /// Number of UTF-8 bytes after the cursor.
        after_bytes: usize,
    },
    /// Platform input method was disabled.
    Disabled,
}

/// Backend-independent drawing commands copied into the resident scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WidgetDraw {
    /// Solid logical rectangle.
    Rect {
        /// X, y, width and height in logical pixels.
        rect: [f64; 4],
        /// RGBA8 fill color.
        color: [u8; 4],
    },
    /// Filled logical circle.
    Circle {
        /// Center in logical pixels.
        center: [f64; 2],
        /// Radius in logical pixels.
        radius: f64,
        /// RGBA8 fill color.
        color: [u8; 4],
    },
    /// Stroked logical line.
    Line {
        /// Start point in logical pixels.
        from: [f64; 2],
        /// End point in logical pixels.
        to: [f64; 2],
        /// Stroke width in logical pixels.
        width: f64,
        /// RGBA8 stroke color.
        color: [u8; 4],
    },
    /// Inline RGBA8 image data interned by the renderer.
    Image {
        /// Destination x, y, width and height.
        rect: [f64; 4],
        /// Source width in physical pixels.
        width: u32,
        /// Source height in physical pixels.
        height: u32,
        /// Source pixels in row-major RGBA8 form.
        rgba: Vec<u8>,
    },
    /// Image already owned by the resident resource pool.
    ImageRef {
        /// Destination x, y, width and height.
        rect: [f64; 4],
        /// Stable resident image identity.
        image: WidgetImageKey,
    },
    /// SVG document parsed and retained by the renderer.
    Svg {
        /// Destination x, y, width and height.
        rect: [f64; 4],
        /// SVG source document.
        source: String,
    },
    /// Full-widget WGSL effect retained by the renderer.
    Shader {
        /// WGSL source.
        source: String,
        /// Whether the source is a complete shader module.
        module: bool,
        /// Animation time supplied to the shader.
        time: f32,
        /// Additional scalar uniforms.
        values: Vec<f32>,
    },
    /// Shaped, unwrapped text.
    Text {
        /// Text origin in logical pixels.
        origin: [f64; 2],
        /// UTF-8 text content.
        text: String,
        /// Font size in logical pixels.
        font_size: f32,
        /// RGBA8 text color.
        color: [u8; 4],
    },
}

/// State update produced by one native widget event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WidgetUpdate {
    /// Replacement value for the resident state slot.
    pub state: Value,
    /// Whether the event was consumed.
    pub handled: bool,
}

/// Platform-facing projection of native widget state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WidgetPresentation {
    /// Whether pointer interaction may focus the widget.
    pub focusable: bool,
    /// Whether focus enables the platform text-input service.
    pub text_input: bool,
    /// Current value synchronized to the UI element.
    pub value: Option<String>,
    /// Current native text selection.
    pub selection: Option<WidgetTextSelection>,
    /// Assistive-technology projection.
    pub accessibility: WidgetAccessibility,
    /// Platform input-method snapshot.
    pub ime: Option<WidgetImeState>,
}

impl Default for WidgetPresentation {
    fn default() -> Self {
        Self {
            focusable: true,
            text_input: false,
            value: None,
            selection: None,
            accessibility: WidgetAccessibility::default(),
            ime: None,
        }
    }
}

/// Work required after a handled event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WidgetInvalidation {
    /// Repaint while preserving measurement and layout.
    Paint,
    /// Recompute measurement and layout before repainting.
    #[default]
    Layout,
}

/// Contract implemented by directly linked native widgets.
pub trait WidgetImplementation: 'static {
    /// Work requested after a handled event.
    const EVENT_INVALIDATION: WidgetInvalidation = WidgetInvalidation::Layout;
    /// Return an optional intrinsic content size.
    fn measure(inputs: &WidgetInputs) -> Result<Option<[f32; 2]>, String>;
    /// Project state into backend-independent drawing commands.
    fn paint(inputs: &WidgetInputs) -> Result<Vec<WidgetDraw>, String>;
    /// Return whether focus should enable platform text input.
    fn accepts_text_input(_: &WidgetInputs) -> Result<bool, String> {
        Ok(false)
    }
    /// Project state into platform-facing widget information.
    fn presentation(inputs: &WidgetInputs) -> Result<WidgetPresentation, String> {
        Ok(WidgetPresentation {
            text_input: Self::accepts_text_input(inputs)?,
            ..Default::default()
        })
    }
    /// Produce the next resident state from one native input event.
    fn event(inputs: &WidgetInputs, event: WidgetEvent) -> Result<WidgetUpdate, String>;
}

/// Internal validated event transaction.
#[doc(hidden)]
pub struct WidgetEventReply {
    /// Validated state update.
    pub update: WidgetUpdate,
    /// Updated native presentation when it was recomputed.
    pub presentation: Option<WidgetPresentation>,
    /// Work requested from the renderer.
    pub invalidation: WidgetInvalidation,
}

/// Type brand for host-owned widget image slots.
pub enum WidgetImageResource {}

/// Pool-scoped, generational image identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WidgetImageKey {
    scope: u32,
    key: ResourceKey<WidgetImageResource>,
}

impl WidgetImageKey {
    /// Construct an image key for a nonzero resource-pool scope.
    pub fn new(scope: u32, key: ResourceKey<WidgetImageResource>) -> Option<Self> {
        (scope != 0).then_some(Self { scope, key })
    }
    /// Return the resource-pool scope.
    pub fn scope(self) -> u32 {
        self.scope
    }
    /// Return the generational key within the scope.
    pub fn key(self) -> ResourceKey<WidgetImageResource> {
        self.key
    }
}
