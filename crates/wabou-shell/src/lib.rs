//! wabou-shell: render a UI tree with Winit, Taffy, Parley, and Vello GPU.
//!
//! The shell is a reusable host: a [`source::FrameSource`] produces a flattened
//! layout list each frame; [`shell::Shell`] owns the window and selected
//! Vello GPU backend. The `wabou-runtime` crate provides the SolidJS-driven
//! (op-protocol) source.

#![warn(missing_docs)]

extern crate self as wabou_shell;

#[cfg(feature = "widgets")]
pub mod widgets;

#[cfg(not(target_arch = "wasm32"))]
pub mod accessibility;
#[cfg(not(target_arch = "wasm32"))]
pub mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod app_dirs;
pub mod effect;
#[cfg(not(target_arch = "wasm32"))]
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod headless;
pub use wabou_renderer::{image, layout, motion, paint};
#[cfg(not(target_arch = "wasm32"))]
mod performance_hud;
#[cfg(not(target_arch = "wasm32"))]
pub mod renderer;
#[cfg(not(target_arch = "wasm32"))]
mod renderer_backend;
pub use wabou_renderer::{scene, scrollbar, shader};
#[cfg(not(target_arch = "wasm32"))]
pub mod shell;
pub use wabou_renderer::{shortcut, source, style, svg};
#[cfg(not(target_arch = "wasm32"))]
mod system;
pub use wabou_renderer::text;
#[cfg(target_arch = "wasm32")]
pub use wabou_renderer::web;
pub use wabou_renderer::widget;
pub use wabou_shell_api::window_lifecycle;
#[cfg(not(target_arch = "wasm32"))]
mod window_state;

#[cfg(not(target_arch = "wasm32"))]
pub use app::{
    ExtensionContext, FrameSourceFactory, RunOutcome, ShellExtension, run_window,
    run_window_with_options, run_window_with_size, run_windows, run_windows_with_factory,
    run_windows_with_factory_and_extensions,
};
#[cfg(not(target_arch = "wasm32"))]
pub use app_dirs::{AppDirectories, AppDirectoryConfig};
pub use effect::{
    CapabilityId, ContextMenuItem, ContextMenuRequest, DialogFilter, EFFECT_ABI_VERSION,
    EffectCompletion, EffectDispatch, EffectErrorCode, EffectExecutor, EffectId, EffectOp,
    EffectPayload, EffectRequest, EffectResult, EffectScope, EffectTapeEntry, MenuPosition,
    MessageDialogButtons, MessageDialogLevel, MessageDialogRequest, MethodId, NotificationRequest,
    OpenDialogRequest, PickDirectoryRequest, RecordingEffectExecutor, ReplayEffectExecutor,
    SaveDialogRequest, WindowResourceKey, initial_window_resource_key,
};
#[cfg(not(target_arch = "wasm32"))]
pub use error::{Error, Result};
pub use paint::{Glyph, NormalizedCoord, PaintCommand, PaintScene, Scene, SvgDocument};
pub use shader::{ShaderEffect, ShaderEffectId, ShaderEffectSource};
#[cfg(not(target_arch = "wasm32"))]
pub use shell::Shell;
pub use shortcut::StandardShortcut;
pub use source::{
    AppLifecycleEvent, ClipboardRequest, ColorScheme, EventResponse, FileDropEvent, FileDropPhase,
    FrameSource, FrameStats, GestureEvent, GesturePhase, HostAction, HostActionResult, ImeEvent,
    ImeState, KeyEvent, KeyLocation, KeyPhase, Modifiers, Point, PointerButton, PointerEvent,
    PointerId, PointerPhase, PointerProperties, PointerType, ScreenshotRequest, SemanticAction,
    SemanticCurrent, SemanticNode, SemanticPopup, SemanticRole, SemanticSnapshot,
    SemanticSortDirection, SemanticStates, SemanticToggleState, UiEvent, WHEEL_LINE_DELTA,
    WakeCallback, WheelDeltaMode, WheelEvent, WindowCommand, WindowInputMode, WindowLevel,
    WindowMetrics, WindowOptions,
};
pub use text::TextContext;
pub use widget::{
    MeasureContext, NativeWidgetPlugin, PaintContext, Widget, WidgetAccessibility,
    WidgetAvailableSpace, WidgetChanges, WidgetEventResult, WidgetFactory, WidgetGeometry,
    WidgetHarness, WidgetImePurpose, WidgetImeState, WidgetNodeEvent, WidgetRasterImage,
    WidgetRasterImageError, WidgetStyle, WidgetTextSelection, WidgetTextSelectionKind,
    decode_widget_config,
};
#[cfg(not(target_arch = "wasm32"))]
pub use window_state::WindowSizePersistence;
#[cfg(not(target_arch = "wasm32"))]
pub use winit::raw_window_handle;

/// Shared stable vector path decoder.
pub use wabou_renderer::vector_path;
