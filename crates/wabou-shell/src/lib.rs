//! GPUI-CE platform shell for Wabou.
//!
//! Solid owns application reactivity. This crate owns the native window and
//! projects completed Solid flushes into GPUI. The projection boundary is
//! deliberately frame-oriented: individual property writes never notify GPUI.

mod application_extension;
mod element;
mod input;
mod native_widget;
mod projection;
mod protocol_projection;
mod style;
mod text_selection;
mod tree;
mod vector_path;
mod window_state;

pub use element::{
    ProjectedElement, ProjectedNativeElementFactory, ProjectedScrollHandle,
    ProjectedSubtreeElementFactory,
};
pub use gpui;
pub use input::{
    ProjectedImeEvent, ProjectedInputEvent, ProjectedInputHandler, ProjectedInputSink,
    ProjectedKeyEvent, ProjectedKeyPhase, ProjectedPointerButton, ProjectedPointerEvent,
    ProjectedPointerPhase, ProjectedScrollEvent, ProjectedTextInputState, ProjectedWheelEvent,
    ProjectedWheelPhase,
};
pub use native_widget::{
    NativeWidgetContext, NativeWidgetEventSink, NativeWidgetFactory, NativeWidgetInput,
    NativeWidgetInputHandler, NativeWidgetMount,
};
pub use projection::{
    DirtyKind, FrameBatch, GpuiNodeKeyExt, NodeKey, PendingNode, ProjectionBoundaryRevision,
    ProjectionInvalidationStats,
};
pub use protocol_projection::{
    GpuiCommand, GpuiComputedStyle, GpuiLayoutNode, GpuiNativeWidget, GpuiProjection,
    GpuiProjectionRenderSnapshot, GpuiSelectableText, GpuiTextCommand, GpuiTextControl,
    GpuiTextControlKind, GpuiTextControlStyle, GpuiThemeSnapshot, project_ir,
};
pub use style::{StyleDiagnostic, StyleProjection};
pub use text_selection::ProjectedTextSelection;
pub use tree::{
    ProjectedNode, ProjectedNodeKind, ProjectionError, ProjectionSnapshot, ProjectionTree,
    TextSelectionPolicy,
};
pub use wabou_shell_api::event::*;
pub use wabou_shell_api::{
    AppDirectories, AppDirectoryConfig, CapabilityId, ContextMenuItem, ContextMenuRequest,
    DialogFilter, EFFECT_ABI_VERSION, EffectCompletion, EffectDispatch, EffectErrorCode,
    EffectExecutor, EffectId, EffectOp, EffectPayload, EffectRequest, EffectResult, EffectScope,
    EffectTapeEntry, MenuPosition, MessageDialogButtons, MessageDialogLevel, MessageDialogRequest,
    MethodId, NotificationRequest, OpenDialogRequest, PickDirectoryRequest,
    RecordingEffectExecutor, ReplayEffectExecutor, RgbaColor, SaveDialogRequest,
    WindowCapabilities, WindowCommand, WindowCreateRequest, WindowEffect, WindowInputMode,
    WindowIntent, WindowLevel, WindowLifecycle, WindowOptions, WindowPresence, WindowResourceKey,
    initial_window_resource_key,
};
pub use wabou_shell_api::{app_dirs, effect, event, window, window_lifecycle};
pub use window_state::WindowSizePersistence;

/// Run a GPUI application using Wabou's selected platform implementation.
///
/// Keeping application construction here prevents downstream applications from
/// depending directly on `gpui_ce_platform` and gives Wabou one place to install
/// platform services.
pub fn application() -> gpui::Application {
    gpui_platform::application()
}
pub use application_extension::{
    ApplicationExtension, ApplicationExtensionContext, install_application_extensions,
};
