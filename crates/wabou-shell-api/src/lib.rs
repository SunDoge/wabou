//! Platform contracts shared by Wabou's GPUI shell, runtime, and extensions.

#![warn(missing_docs)]

pub mod app_dirs;
pub mod color;
pub mod effect;
#[path = "generated/effect_abi.rs"]
mod effect_abi;
pub mod event;
pub mod window;
pub mod window_lifecycle;

pub use app_dirs::{AppDirectories, AppDirectoryConfig};
pub use color::RgbaColor;
pub use effect::{
    CapabilityId, ContextMenuItem, ContextMenuRequest, DialogFilter, EFFECT_ABI_VERSION,
    EffectCompletion, EffectDispatch, EffectErrorCode, EffectExecutor, EffectId, EffectOp,
    EffectPayload, EffectRequest, EffectResult, EffectScope, EffectTapeEntry, MenuPosition,
    MessageDialogButtons, MessageDialogLevel, MessageDialogRequest, MethodId, NotificationRequest,
    OpenDialogRequest, PickDirectoryRequest, RecordingEffectExecutor, ReplayEffectExecutor,
    SaveDialogRequest, WindowCreateRequest, WindowResourceKey, initial_window_resource_key,
};
pub use event::*;
pub use window::{WindowBackground, WindowCommand, WindowInputMode, WindowLevel, WindowOptions};
pub use window_lifecycle::{
    WindowCapabilities, WindowEffect, WindowIntent, WindowLifecycle, WindowPresence,
};
