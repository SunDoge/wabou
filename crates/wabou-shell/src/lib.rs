//! wabou-shell: render a UI tree with winit + taffy + vello + parley.
//!
//! The shell is a reusable host: a [`source::FrameSource`] produces a flattened
//! layout list each frame; [`shell::Shell`] owns the window + wgpu surface +
//! vello renderer and presents at vsync. The `wabou-runtime` crate provides the
//! SolidJS-driven (op-protocol) source.

#![warn(missing_docs)]

pub use wabou_accessibility as accessibility;
pub mod app;
pub mod app_dirs;
pub mod effect;
#[path = "generated/effect_abi.rs"]
mod effect_abi;
pub mod error;
pub mod image;
pub mod layout;
pub mod renderer;
pub mod scene;
pub mod scrollbar;
pub mod shell;
pub mod shortcut;
pub mod source;
pub mod style;
pub mod svg;
mod system;
pub mod text;
pub mod widget;
pub mod window_lifecycle;

pub use app::{
    ExtensionContext, FrameSourceFactory, ShellExtension, run_window, run_window_with_options,
    run_window_with_size, run_windows, run_windows_with_factory,
    run_windows_with_factory_and_extensions,
};
pub use app_dirs::{AppDirectories, AppDirectoryConfig};
pub use effect::{
    CapabilityId, ContextMenuItem, ContextMenuRequest, DialogFilter, EFFECT_ABI_VERSION,
    EffectCompletion, EffectDispatch, EffectErrorCode, EffectExecutor, EffectId, EffectOp,
    EffectPayload, EffectRequest, EffectResult, EffectScope, EffectTapeEntry, MenuPosition,
    MessageDialogButtons, MessageDialogLevel, MessageDialogRequest, MethodId, NotificationRequest,
    OpenDialogRequest, PickDirectoryRequest, RecordingEffectExecutor, ReplayEffectExecutor,
    SaveDialogRequest, WindowResourceKey, initial_window_resource_key,
};
pub use error::{Error, Result};
pub use shell::Shell;
pub use shortcut::StandardShortcut;
pub use source::{
    ClipboardRequest, EventResponse, FrameSource, FrameStats, HostAction, HostActionResult,
    ImeEvent, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerPhase, SemanticAction, SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole,
    SemanticSnapshot, SemanticStates, SemanticToggleState, UiEvent, WHEEL_LINE_DELTA, WakeCallback,
    WheelEvent, WindowCommand, WindowMetrics, WindowOptions,
};
pub use text::TextContext;
pub use widget::{
    MeasureContext, PaintContext, Widget, WidgetAccessibility, WidgetAvailableSpace, WidgetChanges,
    WidgetEventResult, WidgetFactory, WidgetGeometry, WidgetHarness, WidgetNodeEvent, WidgetStyle,
    decode_widget_config,
};
pub use winit::raw_window_handle;
