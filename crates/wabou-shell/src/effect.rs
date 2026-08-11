//! Versioned, typed protocol for low-frequency native side effects.
//!
//! Render mutations deliberately do not use this channel. Effects are OS
//! interactions whose outcomes must be recorded to make a session replayable.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::AppDirectories;
use crate::{WindowCommand, WindowOptions};

pub const EFFECT_ABI_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EffectId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum EffectScope {
    Runtime(u64),
    Window(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CapabilityId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MethodId(pub u16);

/// Open operation key. Built-ins reserve low capability ids; third-party
/// crates receive stable ids through the application's generated registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectOp {
    pub capability: CapabilityId,
    pub method: MethodId,
}

impl EffectOp {
    pub const fn new(capability: u32, method: u16) -> Self {
        Self {
            capability: CapabilityId(capability),
            method: MethodId(method),
        }
    }
}

pub mod builtin {
    use super::EffectOp;

    pub const CLIPBOARD_READ: EffectOp = EffectOp::new(1, 1);
    pub const CLIPBOARD_WRITE: EffectOp = EffectOp::new(1, 2);
    pub const WINDOW_CREATE: EffectOp = EffectOp::new(2, 1);
    pub const WINDOW_CLOSE: EffectOp = EffectOp::new(2, 2);
    pub const WINDOW_SET_MAXIMIZED: EffectOp = EffectOp::new(2, 3);
    pub const WINDOW_SET_TITLE: EffectOp = EffectOp::new(2, 4);
    pub const CONTEXT_MENU_SHOW: EffectOp = EffectOp::new(3, 1);
    pub const APP_DIRS_RESOLVE: EffectOp = EffectOp::new(4, 1);
    pub const DIALOG_OPEN: EffectOp = EffectOp::new(5, 1);
    pub const DIALOG_SAVE: EffectOp = EffectOp::new(5, 2);
    pub const DIALOG_PICK_DIRECTORY: EffectOp = EffectOp::new(5, 3);
    pub const DIALOG_MESSAGE: EffectOp = EffectOp::new(5, 4);
    pub const NOTIFICATION_SHOW: EffectOp = EffectOp::new(6, 1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadAffinity {
    Ui,
    Worker,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayPolicy {
    Execute,
    RecordCompletion,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub id: MethodId,
    pub name: &'static str,
    pub affinity: ThreadAffinity,
    pub replay: ReplayPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityDescriptor {
    pub id: CapabilityId,
    pub name: &'static str,
    pub version: u16,
    pub methods: &'static [MethodDescriptor],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityRegistryError {
    DuplicateId(CapabilityId),
    DuplicateName(&'static str),
    DuplicateMethod {
        capability: CapabilityId,
        method: MethodId,
    },
}

#[derive(Default)]
pub struct CapabilityRegistry {
    by_id: BTreeMap<CapabilityId, CapabilityDescriptor>,
    by_name: BTreeMap<&'static str, CapabilityId>,
}

impl CapabilityRegistry {
    pub fn register(
        &mut self,
        descriptor: CapabilityDescriptor,
    ) -> Result<(), CapabilityRegistryError> {
        if self.by_id.contains_key(&descriptor.id) {
            return Err(CapabilityRegistryError::DuplicateId(descriptor.id));
        }
        if self.by_name.contains_key(descriptor.name) {
            return Err(CapabilityRegistryError::DuplicateName(descriptor.name));
        }
        let mut methods = BTreeMap::new();
        for method in descriptor.methods {
            if methods.insert(method.id, method.name).is_some() {
                return Err(CapabilityRegistryError::DuplicateMethod {
                    capability: descriptor.id,
                    method: method.id,
                });
            }
        }
        self.by_name.insert(descriptor.name, descriptor.id);
        self.by_id.insert(descriptor.id, descriptor);
        Ok(())
    }

    pub fn get(&self, id: CapabilityId) -> Option<&CapabilityDescriptor> {
        self.by_id.get(&id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ContextMenuItem {
    Item {
        id: String,
        label: String,
        #[serde(default = "default_true")]
        enabled: bool,
        #[serde(default)]
        checked: bool,
    },
    Separator,
    Submenu {
        label: String,
        items: Vec<ContextMenuItem>,
    },
}

const fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextMenuRequest {
    pub window_id: u64,
    pub position: Option<MenuPosition>,
    pub items: Vec<ContextMenuItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowCreateRequest {
    pub window_id: u64,
    pub options: WindowOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub filters: Vec<DialogFilter>,
    #[serde(default)]
    pub multiple: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDialogRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub default_name: Option<String>,
    #[serde(default)]
    pub filters: Vec<DialogFilter>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickDirectoryRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub directory: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageDialogLevel {
    #[default]
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MessageDialogButtons {
    #[default]
    Ok,
    OkCancel,
    YesNo,
    YesNoCancel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageDialogRequest {
    #[serde(default)]
    pub title: Option<String>,
    pub message: String,
    #[serde(default)]
    pub level: MessageDialogLevel,
    #[serde(default)]
    pub buttons: MessageDialogButtons,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationRequest {
    pub title: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub silent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EffectPayload {
    ClipboardRead,
    ClipboardWrite {
        text: String,
    },
    WindowCreate(WindowCreateRequest),
    WindowControl {
        window_id: u64,
        command: WindowCommand,
    },
    ContextMenuShow(ContextMenuRequest),
    AppDirsResolve(AppDirectories),
    DialogOpen(OpenDialogRequest),
    DialogSave(SaveDialogRequest),
    DialogPickDirectory(PickDirectoryRequest),
    DialogMessage(MessageDialogRequest),
    NotificationShow(NotificationRequest),
    Extension {
        op: EffectOp,
        bytes: Vec<u8>,
    },
    Invalid {
        op: EffectOp,
        message: String,
    },
}

impl EffectPayload {
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
pub struct EffectRequest {
    pub id: EffectId,
    pub scope: EffectScope,
    pub payload: EffectPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EffectErrorCode {
    Unsupported,
    InvalidRequest,
    PlatformFailure,
    ReplayDiverged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum EffectResult {
    Unit,
    ClipboardText(Option<String>),
    ContextMenuSelection(Option<String>),
    AppDirectories(AppDirectories),
    DialogPaths(Option<Vec<String>>),
    DialogMessage(String),
    Cancelled,
    Error {
        code: EffectErrorCode,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectCompletion {
    pub id: EffectId,
    pub op: EffectOp,
    pub result: EffectResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectDispatch {
    Complete(EffectCompletion),
    Pending,
}

pub trait EffectExecutor {
    fn submit(&mut self, request: EffectRequest) -> EffectDispatch;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum EffectTapeEntry {
    Request(EffectRequest),
    Completion(EffectCompletion),
}

pub struct RecordingEffectExecutor<E> {
    inner: E,
    tape: Vec<EffectTapeEntry>,
}

impl<E> RecordingEffectExecutor<E> {
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            tape: Vec::new(),
        }
    }

    pub fn record_completion(&mut self, completion: EffectCompletion) {
        self.tape.push(EffectTapeEntry::Completion(completion));
    }

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

pub struct ReplayEffectExecutor {
    tape: VecDeque<EffectTapeEntry>,
}

impl ReplayEffectExecutor {
    pub fn new(tape: impl IntoIterator<Item = EffectTapeEntry>) -> Self {
        Self {
            tape: tape.into_iter().collect(),
        }
    }

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

    fn read(id: u64) -> EffectRequest {
        EffectRequest {
            id: EffectId(id),
            scope: EffectScope::Window(1),
            payload: EffectPayload::ClipboardRead,
        }
    }

    #[test]
    fn builtin_operation_ids_are_stable() {
        assert_eq!(builtin::CLIPBOARD_READ, EffectOp::new(1, 1));
        assert_eq!(builtin::CONTEXT_MENU_SHOW, EffectOp::new(3, 1));
        assert_eq!(builtin::APP_DIRS_RESOLVE, EffectOp::new(4, 1));
        assert_eq!(builtin::DIALOG_OPEN, EffectOp::new(5, 1));
        assert_eq!(builtin::DIALOG_MESSAGE, EffectOp::new(5, 4));
        assert_eq!(builtin::NOTIFICATION_SHOW, EffectOp::new(6, 1));
    }

    #[test]
    fn registry_accepts_external_capabilities_and_rejects_collisions() {
        static METHODS: &[MethodDescriptor] = &[MethodDescriptor {
            id: MethodId(1),
            name: "pickColor",
            affinity: ThreadAffinity::Ui,
            replay: ReplayPolicy::RecordCompletion,
        }];
        let descriptor = CapabilityDescriptor {
            id: CapabilityId(0x8000_0100),
            name: "color-picker",
            version: 1,
            methods: METHODS,
        };
        let mut registry = CapabilityRegistry::default();
        registry.register(descriptor.clone()).unwrap();
        assert_eq!(registry.get(descriptor.id), Some(&descriptor));
        assert_eq!(
            registry.register(descriptor),
            Err(CapabilityRegistryError::DuplicateId(CapabilityId(
                0x8000_0100
            )))
        );
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
