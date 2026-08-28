//! wabou-shell: render a UI tree with winit + Taffy + Parley + AnyRender.
//!
//! The shell is a reusable host: a [`source::FrameSource`] produces a flattened
//! layout list each frame; [`shell::Shell`] owns the window and selected
//! AnyRender backend. The `wabou-runtime` crate provides the SolidJS-driven
//! (op-protocol) source.

#![warn(missing_docs)]

pub use anyrender;
pub use anyrender::PaintScene;
pub use wabou_accessibility as accessibility;
pub mod app;
pub mod app_dirs;
pub mod effect;
#[path = "generated/effect_abi.rs"]
mod effect_abi;
pub mod error;
pub mod headless;
pub mod image;
pub mod layout;
pub mod renderer;
mod renderer_backend;
pub mod scene;
pub mod scrollbar;
pub mod shell;
pub mod shortcut;
pub mod source;
pub mod style;
pub mod svg;
mod system;
pub mod text;
mod text_raster;
pub mod widget;
pub mod window_lifecycle;
mod window_state;

pub use app::{
    ExtensionContext, FrameSourceFactory, RunOutcome, ShellExtension, run_window,
    run_window_with_options, run_window_with_size, run_windows, run_windows_with_factory,
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
    AppLifecycleEvent, ClipboardRequest, ColorScheme, EventResponse, FileDropEvent, FileDropPhase,
    FrameSource, FrameStats, GestureEvent, GesturePhase, HostAction, HostActionResult, ImeEvent,
    KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent, PointerId,
    PointerPhase, PointerProperties, PointerType, RendererBackend, ScreenshotRequest,
    SemanticAction, SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole, SemanticSnapshot,
    SemanticStates, SemanticToggleState, UiEvent, WHEEL_LINE_DELTA, WakeCallback, WheelEvent,
    WindowCommand, WindowInputMode, WindowLevel, WindowMetrics, WindowOptions,
};
pub use text::TextContext;
pub use widget::{
    MeasureContext, PaintContext, Widget, WidgetAccessibility, WidgetAvailableSpace, WidgetChanges,
    WidgetEventResult, WidgetFactory, WidgetGeometry, WidgetHarness, WidgetNodeEvent, WidgetStyle,
    WidgetTextSelection, WidgetTextSelectionKind, decode_widget_config,
};
pub use window_state::WindowSizePersistence;
pub use winit::raw_window_handle;
