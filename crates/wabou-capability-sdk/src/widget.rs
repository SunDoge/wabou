//! Data-only widget boundary, carried by the existing stabby capability ABI.
//! Widget callbacks are synchronous. State, focus and renderer resources belong
//! to the resident host; no trait object, scene, future or borrowed ctx crosses.
use crate::{Host, Module, Ready, Task};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Current callback inputs. Sizes and positions are content-local logical pixels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inputs {
    /// Resident application state snapshot (never plugin-owned memory).
    pub state: Value,
    /// Configuration supplied by the UI.
    pub config: Value,
    /// Content box dimensions.
    pub size: [f32; 2],
    /// Resident native focus state.
    pub focused: bool,
}
/// Small initial event vocabulary; unsupported native events remain unhandled.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Event {
    /// Primary pointer button transition.
    Pointer { down: bool, position: [f64; 2] },
    /// Movement, entry, exit or cancellation; includes active buttons for dragging.
    PointerMotion {
        phase: PointerMotionPhase,
        position: [f64; 2],
        buttons: u32,
        modifiers: u8,
    },
    /// Auxiliary/context/extra button transition (primary uses `Pointer`).
    PointerButton {
        down: bool,
        button: u32,
        position: [f64; 2],
        buttons: u32,
        modifiers: u8,
    },
    /// Logical-pixel scroll deltas, already normalized by the host.
    Wheel {
        position: [f64; 2],
        delta: [f64; 2],
        modifiers: u8,
    },
    /// Physical/logical keyboard transition.
    Key {
        down: bool,
        key: String,
        code: String,
        repeat: bool,
        modifiers: u8,
    },
    /// Committed text outside IME composition.
    TextInput { text: String },
    /// Clipboard text delivered by the host.
    Paste { text: String },
    /// Input-method lifecycle; cursor and deletion offsets are UTF-8 bytes.
    Ime { event: Ime },
    /// Focus transition; focus itself remains host-owned.
    Focus { focused: bool },
}
/// Pointer motion lifecycle independent of the native backend.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PointerMotionPhase {
    Enter,
    Move,
    Cancel,
    Leave,
}
/// Input-method lifecycle independent of the native backend.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Ime {
    Enabled,
    Preedit {
        text: String,
        cursor: Option<(usize, usize)>,
    },
    Commit {
        text: String,
    },
    DeleteSurrounding {
        before_bytes: usize,
        after_bytes: usize,
    },
    Disabled,
}
/// Backend-independent drawing commands copied into the resident scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Draw {
    /// Solid logical rectangle: x, y, width, height and RGBA8.
    Rect { rect: [f64; 4], color: [u8; 4] },
    /// Unwrapped text at a logical origin, shaped using resident fonts.
    Text {
        origin: [f64; 2],
        text: String,
        font_size: f32,
        color: [u8; 4],
    },
}
/// Event transaction: commit state only after the complete response is validated.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Update {
    /// Replacement JSON value in the existing resident state slot.
    pub state: Value,
    /// Consume this event and request repaint/layout. Unhandled responses must
    /// leave state unchanged.
    pub handled: bool,
}
/// Implement ordinary Rust methods; export `WidgetModule<Self>` with export_module!.
/// Schema changes require a new version and are rejected while old widgets live.
pub trait WidgetImplementation: 'static {
    /// State/protocol contract revision, independent of code edits.
    const SCHEMA_VERSION: u32;
    /// Optional intrinsic size. CSS still constrains the final content box.
    fn measure(inputs: &Inputs) -> Result<Option<[f32; 2]>, String>;
    /// Pure projection; cannot mutate durable state.
    fn paint(inputs: &Inputs) -> Result<Vec<Draw>, String>;
    /// Enable platform text/IME input when focused. Defaults to a non-text control.
    fn accepts_text_input(_: &Inputs) -> Result<bool, String> {
        Ok(false)
    }
    /// Produce the next state without retaining pointers into the host.
    fn event(inputs: &Inputs, event: Event) -> Result<Update, String>;
}
/// SDK adapter using the existing versioned loader and panic containment.
pub struct WidgetModule<W>(std::marker::PhantomData<W>);
#[derive(Serialize, Deserialize)]
pub struct Request {
    /// Callback snapshot.
    pub inputs: Inputs,
    /// Present for event dispatch only.
    pub event: Option<Event>,
}
impl<W: WidgetImplementation> Module for WidgetModule<W> {
    const CONTRACT_VERSION: u32 = W::SCHEMA_VERSION;
    fn start(_: &Host<'_>, method: &str, request: &[u8]) -> Result<Box<dyn Task>, String> {
        let request: Request = serde_json::from_slice(request).map_err(|e| e.to_string())?;
        match method {
            "widget.measure" => Ready::json(&W::measure(&request.inputs)?),
            "widget.paint" => Ready::json(&W::paint(&request.inputs)?),
            "widget.textInput" => Ready::json(&W::accepts_text_input(&request.inputs)?),
            "widget.event" => Ready::json(&W::event(
                &request.inputs,
                request.event.ok_or("missing widget event")?,
            )?),
            _ => Err(format!("unknown widget callback: {method}")),
        }
    }
}
