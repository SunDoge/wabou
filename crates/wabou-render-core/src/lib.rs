//! Shared layout, text, paint and widget contracts for native and browser hosts.
#![warn(missing_docs)]
pub mod image;
pub mod layout;
/// Renderer and value animation algorithms shared by native and web hosts.
pub use wabou_motion as motion;
pub mod paint;
pub mod scene;
pub mod scrollbar;
pub mod shader;
pub mod shortcut;
pub mod source;
pub mod style;
pub mod svg;
pub mod text;
pub mod vector_path;
#[cfg(target_arch = "wasm32")]
pub mod web;
pub mod widget;
pub use paint::{Glyph, NormalizedCoord, PaintCommand, PaintScene, Scene, SvgDocument};
pub use shader::{ShaderEffect, ShaderEffectId, ShaderEffectSource};
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
pub use wabou_shell_api::{EffectCompletion, EffectRequest};
pub use widget::{
    MeasureContext, PaintContext, Widget, WidgetAccessibility, WidgetAvailableSpace, WidgetChanges,
    WidgetEventResult, WidgetFactory, WidgetGeometry, WidgetHarness, WidgetImePurpose,
    WidgetImeState, WidgetNodeEvent, WidgetRasterImage, WidgetRasterImageError, WidgetStyle,
    WidgetTextSelection, WidgetTextSelectionKind, decode_widget_config,
};
