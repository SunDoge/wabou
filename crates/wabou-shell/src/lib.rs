//! wabou-shell: render a UI tree with winit + taffy + vello + parley.
//!
//! The shell is a reusable host: a [`source::FrameSource`] produces a flattened
//! layout list each frame; [`shell::Shell`] owns the window + wgpu surface +
//! vello renderer and presents at vsync. The `wabou-quick` crate provides the
//! SolidJS-driven (op-protocol) source.

pub use wabou_accessibility as accessibility;
pub mod app;
pub mod effect;
pub mod error;
pub mod layout;
pub mod renderer;
pub mod scene;
pub mod scrollbar;
pub mod shell;
pub mod shortcut;
pub mod source;
pub mod style;
pub mod svg;
pub mod text;

pub use app::{
    ExtensionContext, FrameSourceFactory, ShellExtension, run_window, run_window_with_options,
    run_window_with_size, run_windows, run_windows_with_factory,
    run_windows_with_factory_and_extensions,
};
pub use effect::{
    CapabilityDescriptor, CapabilityId, CapabilityRegistry, CapabilityRegistryError,
    ContextMenuItem, ContextMenuRequest, EFFECT_ABI_VERSION, EffectCompletion, EffectDispatch,
    EffectErrorCode, EffectExecutor, EffectId, EffectOp, EffectPayload, EffectRequest,
    EffectResult, EffectScope, EffectTapeEntry, MenuPosition, MethodDescriptor, MethodId,
    RecordingEffectExecutor, ReplayEffectExecutor, ReplayPolicy, ThreadAffinity,
};
pub use error::{Error, Result};
pub use shell::Shell;
pub use shortcut::StandardShortcut;
pub use source::{
    ClipboardRequest, EventResponse, FrameSource, FrameStats, HostAction, HostActionResult,
    ImeEvent, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerPhase, SemanticAction, SemanticNode, SemanticRole, SemanticSnapshot, UiEvent,
    WHEEL_LINE_DELTA, WakeCallback, WheelEvent, WindowCommand, WindowMetrics, WindowOptions,
};
pub use text::TextContext;
pub use winit::raw_window_handle;
