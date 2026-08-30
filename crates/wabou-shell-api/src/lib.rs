//! Backend-neutral contracts shared by Wabou's GPUI shell and legacy backend.

#![warn(missing_docs)]

pub mod app_dirs;
pub mod effect;
#[path = "generated/effect_abi.rs"]
mod effect_abi;
pub mod window;

pub use app_dirs::{AppDirectories, AppDirectoryConfig};
pub use effect::{
    CapabilityId, ContextMenuItem, ContextMenuRequest, DialogFilter, EFFECT_ABI_VERSION,
    EffectCompletion, EffectDispatch, EffectErrorCode, EffectExecutor, EffectId, EffectOp,
    EffectPayload, EffectRequest, EffectResult, EffectScope, EffectTapeEntry, MenuPosition,
    MessageDialogButtons, MessageDialogLevel, MessageDialogRequest, MethodId, NotificationRequest,
    OpenDialogRequest, PickDirectoryRequest, RecordingEffectExecutor, ReplayEffectExecutor,
    SaveDialogRequest, WindowCreateRequest, WindowResourceKey, initial_window_resource_key,
};
pub use window::{RendererBackend, WindowCommand, WindowInputMode, WindowLevel, WindowOptions};
