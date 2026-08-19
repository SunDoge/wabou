//! Versioned, typed protocol for low-frequency native side effects.
//!
//! Render mutations deliberately do not use this channel. Effects are OS
//! interactions whose outcomes must be recorded to make a session replayable.

#![warn(missing_docs)]

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::AppDirectories;
use crate::{WindowCommand, WindowOptions};

/// Wire schema version for serialized effect requests and completions.
pub use crate::effect_abi::EFFECT_ABI_VERSION;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Request identifier unique within one runtime session.
pub struct EffectId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
/// Lifetime and routing domain of an effect.
pub enum EffectScope {
    /// Effect belongs to the complete application runtime.
    Runtime,
    /// Effect belongs to one Wabou window.
    Window(WindowResourceKey),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Stable numeric identifier for an effect capability family.
pub struct CapabilityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
/// Stable method identifier within a capability.
pub struct MethodId(pub u16);

/// Open operation key. Built-ins reserve low capability ids; third-party
/// crates receive stable ids through the application's generated registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectOp {
    /// Capability family.
    pub capability: CapabilityId,
    /// Method within the capability.
    pub method: MethodId,
}

impl EffectOp {
    /// Construct an operation key from its stable numeric components.
    pub const fn new(capability: u32, method: u16) -> Self {
        Self {
            capability: CapabilityId(capability),
            method: MethodId(method),
        }
    }
}

/// Reserved operation identifiers implemented by the standard desktop host.
pub mod builtin {
    pub use crate::effect_abi::*;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Optional logical-window position for a context menu.
pub struct MenuPosition {
    /// Horizontal position in logical pixels.
    pub x: i32,
    /// Vertical position in logical pixels.
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
/// Declarative item in a native context menu tree.
pub enum ContextMenuItem {
    /// Selectable command item.
    Item {
        /// Stable application identifier returned on selection.
        id: String,
        /// User-visible label.
        label: String,
        /// Whether the item accepts selection.
        #[serde(default = "default_true")]
        enabled: bool,
        /// Whether the platform draws a check mark.
        #[serde(default)]
        checked: bool,
    },
    /// Non-interactive visual separator.
    Separator,
    /// Nested submenu.
    Submenu {
        /// User-visible submenu label.
        label: String,
        /// Child menu items.
        items: Vec<ContextMenuItem>,
    },
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to show a native context menu.
pub struct ContextMenuRequest {
    /// Window that owns the menu.
    pub window_id: WindowResourceKey,
    /// Explicit logical position, or current pointer position when absent.
    pub position: Option<MenuPosition>,
    /// Root menu items.
    pub items: Vec<ContextMenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Payload for creating a native window.
pub struct WindowCreateRequest {
    /// Initial native window options.
    pub options: WindowOptions,
}

/// Type-level family marker for native window resources.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WindowResource {}

/// Full-width generational identity of one native window resource.
pub type WindowResourceKey = wabou_host_api::ResourceKey<WindowResource>;

/// Predict the initial SlotMap keys allocated by the shell before it starts
/// the already-constructed root frame sources.
pub const fn initial_window_resource_key(index: usize) -> WindowResourceKey {
    match WindowResourceKey::from_parts(index as u32 + 1, 1) {
        Some(key) => key,
        None => panic!("initial window resource index exceeds u32"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// One file-type filter shown by a platform dialog.
pub struct DialogFilter {
    /// User-visible filter name.
    pub name: String,
    /// Extensions without leading dots.
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Options for choosing one or more existing files.
pub struct OpenDialogRequest {
    /// Optional native dialog title.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional initial directory.
    #[serde(default)]
    pub directory: Option<String>,
    /// File-type filters shown to the user.
    #[serde(default)]
    pub filters: Vec<DialogFilter>,
    /// Whether more than one file may be selected.
    #[serde(default)]
    pub multiple: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Options for choosing a destination file.
pub struct SaveDialogRequest {
    /// Optional native dialog title.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional initial directory.
    #[serde(default)]
    pub directory: Option<String>,
    /// Suggested file name.
    #[serde(default)]
    pub default_name: Option<String>,
    /// File-type filters shown to the user.
    #[serde(default)]
    pub filters: Vec<DialogFilter>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Options for choosing an existing directory.
pub struct PickDirectoryRequest {
    /// Optional native dialog title.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional initial directory.
    #[serde(default)]
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Severity communicated by a native message dialog.
pub enum MessageDialogLevel {
    #[default]
    /// Informational message.
    Info,
    /// Warning that does not necessarily prevent continuation.
    Warning,
    /// Error requiring user attention.
    Error,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Standard platform button arrangement for a message dialog.
pub enum MessageDialogButtons {
    #[default]
    /// Acknowledge only.
    Ok,
    /// Acknowledge or cancel.
    OkCancel,
    /// Positive or negative answer.
    YesNo,
    /// Positive, negative, or cancelled answer.
    YesNoCancel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to show a native modal message dialog.
pub struct MessageDialogRequest {
    /// Optional native dialog title.
    #[serde(default)]
    pub title: Option<String>,
    /// User-visible message body.
    pub message: String,
    /// Visual severity.
    #[serde(default)]
    pub level: MessageDialogLevel,
    /// Standard platform button arrangement.
    #[serde(default)]
    pub buttons: MessageDialogButtons,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Request to publish a system notification.
pub struct NotificationRequest {
    /// Notification title.
    pub title: String,
    /// Optional notification body.
    #[serde(default)]
    pub body: Option<String>,
    /// Optional platform-specific icon name or path.
    #[serde(default)]
    pub icon: Option<String>,
    /// Whether notification sounds should be suppressed.
    #[serde(default)]
    pub silent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
/// Typed payload for a built-in or application-defined effect operation.
pub enum EffectPayload {
    /// Read system clipboard text.
    ClipboardRead,
    /// Write system clipboard text.
    ClipboardWrite {
        /// Text to write.
        text: String,
    },
    /// Create a native window.
    WindowCreate(WindowCreateRequest),
    /// Control an existing native window.
    WindowControl {
        /// Target Wabou window identifier.
        window_id: WindowResourceKey,
        /// Native operation to apply.
        command: WindowCommand,
    },
    /// Show a native context menu.
    ContextMenuShow(ContextMenuRequest),
    /// Return already-resolved application directories through the effect ABI.
    AppDirsResolve(AppDirectories),
    /// Show an open-file dialog.
    DialogOpen(OpenDialogRequest),
    /// Show a save-file dialog.
    DialogSave(SaveDialogRequest),
    /// Show a directory picker.
    DialogPickDirectory(PickDirectoryRequest),
    /// Show a native message dialog.
    DialogMessage(MessageDialogRequest),
    /// Publish a desktop notification.
    NotificationShow(NotificationRequest),
    /// Opaque application-defined payload interpreted by a registered executor.
    Extension {
        /// Registered extension operation.
        op: EffectOp,
        /// Capability-specific encoded payload.
        bytes: Vec<u8>,
    },
    /// Preserved invalid request that must complete with a typed error.
    Invalid {
        /// Operation the caller attempted.
        op: EffectOp,
        /// Boundary validation error.
        message: String,
    },
}

impl EffectPayload {
    /// Return the stable operation key represented by this payload.
    pub const fn op(&self) -> EffectOp {
        match self {
            Self::ClipboardRead => builtin::CLIPBOARD_READ,
            Self::ClipboardWrite { .. } => builtin::CLIPBOARD_WRITE,
            Self::WindowCreate(_) => builtin::WINDOW_CREATE,
            Self::WindowControl {
                command: WindowCommand::Close,
                ..
            } => builtin::WINDOW_CLOSE,
            Self::WindowControl {
                command: WindowCommand::SetMaximized(_),
                ..
            } => builtin::WINDOW_SET_MAXIMIZED,
            Self::WindowControl {
                command: WindowCommand::SetTitle(_),
                ..
            } => builtin::WINDOW_SET_TITLE,
            Self::WindowControl {
                command: WindowCommand::Minimize,
                ..
            } => builtin::WINDOW_MINIMIZE,
            Self::WindowControl {
                command: WindowCommand::StartDragging,
                ..
            } => builtin::WINDOW_START_DRAGGING,
            Self::ContextMenuShow(_) => builtin::CONTEXT_MENU_SHOW,
            Self::AppDirsResolve(_) => builtin::APP_DIRS_RESOLVE,
            Self::DialogOpen(_) => builtin::DIALOG_OPEN,
            Self::DialogSave(_) => builtin::DIALOG_SAVE,
            Self::DialogPickDirectory(_) => builtin::DIALOG_PICK_DIRECTORY,
            Self::DialogMessage(_) => builtin::DIALOG_MESSAGE,
            Self::NotificationShow(_) => builtin::NOTIFICATION_SHOW,
            Self::Extension { op, .. } | Self::Invalid { op, .. } => *op,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// One effect submitted by a frame source at a frame boundary.
pub struct EffectRequest {
    /// Session-unique request identifier.
    pub id: EffectId,
    /// Runtime or window lifetime owning the effect.
    pub scope: EffectScope,
    /// Typed operation payload.
    pub payload: EffectPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Stable machine-readable reason an effect did not succeed.
pub enum EffectErrorCode {
    /// Host does not implement the operation on this platform.
    Unsupported,
    /// Payload failed boundary validation.
    InvalidRequest,
    /// Platform API failed after accepting the request.
    PlatformFailure,
    /// Deterministic replay did not match its recorded tape.
    ReplayDiverged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
/// Typed completion value returned by an effect executor.
pub enum EffectResult {
    /// Successful operation with no value.
    Unit,
    /// Clipboard contents, or `None` when no text is available.
    ClipboardText(Option<String>),
    /// Selected context-menu item identifier, or `None` when dismissed.
    ContextMenuSelection(Option<String>),
    /// Resolved platform application directories.
    AppDirectories(AppDirectories),
    /// Newly-created native window resource.
    Window(WindowResourceKey),
    /// Selected paths, or `None` when the dialog was dismissed.
    DialogPaths(Option<Vec<String>>),
    /// Platform-independent identifier of the selected message button.
    DialogMessage(String),
    /// Operation was explicitly cancelled.
    Cancelled,
    /// Operation failed with a stable code and diagnostic message.
    Error {
        /// Machine-readable failure category.
        code: EffectErrorCode,
        /// Human-readable diagnostic, not intended for control flow.
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Completion routed back to the source that submitted an effect.
pub struct EffectCompletion {
    /// Original request identifier.
    pub id: EffectId,
    /// Operation key, retained to validate routing and replay.
    pub op: EffectOp,
    /// Typed result.
    pub result: EffectResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Immediate outcome of submitting an effect to an executor.
pub enum EffectDispatch {
    /// Executor completed synchronously.
    Complete(EffectCompletion),
    /// Completion will arrive asynchronously at a later frame boundary.
    Pending,
}

/// Host implementation capable of executing typed desktop effects.
pub trait EffectExecutor {
    /// Submit one request without blocking the UI thread.
    fn submit(&mut self, request: EffectRequest) -> EffectDispatch;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
/// Ordered request/completion item used for deterministic recording and replay.
pub enum EffectTapeEntry {
    /// Submitted effect request.
    Request(EffectRequest),
    /// Corresponding effect completion.
    Completion(EffectCompletion),
}

/// Executor decorator that records requests and synchronous completions.
///
/// Asynchronous completions must be appended explicitly with
/// [`Self::record_completion`] when they return to the host.
pub struct RecordingEffectExecutor<E> {
    inner: E,
    tape: Vec<EffectTapeEntry>,
}

impl<E> RecordingEffectExecutor<E> {
    /// Wrap an executor with an initially empty tape.
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            tape: Vec::new(),
        }
    }

    /// Append an asynchronous completion received from the wrapped executor.
    pub fn record_completion(&mut self, completion: EffectCompletion) {
        self.tape.push(EffectTapeEntry::Completion(completion));
    }

    /// Borrow the ordered recording accumulated so far.
    pub fn tape(&self) -> &[EffectTapeEntry] {
        &self.tape
    }
}

impl<E: EffectExecutor> EffectExecutor for RecordingEffectExecutor<E> {
    fn submit(&mut self, request: EffectRequest) -> EffectDispatch {
        self.tape.push(EffectTapeEntry::Request(request.clone()));
        let dispatch = self.inner.submit(request);
        if let EffectDispatch::Complete(completion) = &dispatch {
            self.tape
                .push(EffectTapeEntry::Completion(completion.clone()));
        }
        dispatch
    }
}

/// Deterministic executor that consumes a previously recorded effect tape.
pub struct ReplayEffectExecutor {
    tape: VecDeque<EffectTapeEntry>,
}

impl ReplayEffectExecutor {
    /// Construct a replay executor from ordered request/completion entries.
    pub fn new(tape: impl IntoIterator<Item = EffectTapeEntry>) -> Self {
        Self {
            tape: tape.into_iter().collect(),
        }
    }

    /// Whether every recorded entry has been consumed.
    pub fn is_finished(&self) -> bool {
        self.tape.is_empty()
    }

    fn divergence(request: &EffectRequest, message: impl Into<String>) -> EffectDispatch {
        EffectDispatch::Complete(EffectCompletion {
            id: request.id,
            op: request.payload.op(),
            result: EffectResult::Error {
                code: EffectErrorCode::ReplayDiverged,
                message: message.into(),
            },
        })
    }
}

impl EffectExecutor for ReplayEffectExecutor {
    fn submit(&mut self, request: EffectRequest) -> EffectDispatch {
        let Some(EffectTapeEntry::Request(expected)) = self.tape.pop_front() else {
            return Self::divergence(&request, "effect tape ended before request");
        };
        if expected != request {
            return Self::divergence(
                &request,
                format!("expected {expected:?}, received {request:?}"),
            );
        }
        match self.tape.pop_front() {
            Some(EffectTapeEntry::Completion(completion)) => EffectDispatch::Complete(completion),
            Some(entry) => Self::divergence(
                &request,
                format!("expected completion after request, found {entry:?}"),
            ),
            None => EffectDispatch::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(id: u32) -> EffectRequest {
        EffectRequest {
            id: EffectId(id),
            scope: EffectScope::Window(initial_window_resource_key(0)),
            payload: EffectPayload::ClipboardRead,
        }
    }

    #[test]
    fn builtin_operation_ids_are_stable() {
        assert_eq!(builtin::CLIPBOARD_READ, EffectOp::new(1, 1));
        assert_eq!(builtin::WINDOW_MINIMIZE, EffectOp::new(2, 5));
        assert_eq!(builtin::WINDOW_START_DRAGGING, EffectOp::new(2, 6));
        assert_eq!(builtin::CONTEXT_MENU_SHOW, EffectOp::new(3, 1));
        assert_eq!(builtin::APP_DIRS_RESOLVE, EffectOp::new(4, 1));
        assert_eq!(builtin::DIALOG_OPEN, EffectOp::new(5, 1));
        assert_eq!(builtin::DIALOG_MESSAGE, EffectOp::new(5, 4));
        assert_eq!(builtin::NOTIFICATION_SHOW, EffectOp::new(6, 1));
    }

    #[test]
    fn window_resource_keys_preserve_both_slotmap_halves() {
        let key = WindowResourceKey::from_parts(0xffff_fffe, 0xffff_ffff).unwrap();
        assert_eq!(WindowResourceKey::from_ffi(key.as_ffi()), Some(key));
        assert!(WindowResourceKey::from_parts(0, 1).is_none());
        assert!(WindowResourceKey::from_parts(1, 2).is_none());
    }

    #[test]
    fn replay_rejects_a_different_request() {
        let mut replay = ReplayEffectExecutor::new([EffectTapeEntry::Request(read(1))]);
        let EffectDispatch::Complete(completion) = replay.submit(read(2)) else {
            panic!("divergence must complete with an error")
        };
        assert!(matches!(
            completion.result,
            EffectResult::Error {
                code: EffectErrorCode::ReplayDiverged,
                ..
            }
        ));
    }
}
