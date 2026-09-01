//! wabou-runtime: QuickJS + SolidJS host driving the GPUI wabou-shell.
//!
//! SolidJS (via `solid-js/universal`) emits binary DOM-mutation ops into a
//! `Writer`; one `requestAnimationFrame` tick per frame flushes them to Rust;
//! [`gpui_controller::GpuiController`] decodes and applies them to the retained GPUI
//! projection. The retired Winit/Vello implementation lives in separate,
//! unpublished `wabou-legacy-*` crates as a migration oracle; it is not part of
//! this runtime and cannot be selected by applications.

#![warn(missing_docs)]

mod actor;
mod bundle;
mod clock;
mod effect_bridge;
mod effect_trace;
mod error;
mod gpui_controller;
#[cfg(feature = "headless")]
mod gpui_headless;
mod gpui_motion;
mod gpui_performance_hud;
mod gpui_projection_boundary;
mod gpui_view;
mod gpui_widgets;
mod gpui_windows;
mod host;
#[cfg(test)]
pub(crate) mod host_abi;
mod host_ffi;
mod host_frame;
mod host_message;
mod image_resource;
mod json_capability;
mod jsrt;
mod kv;
mod native_capability;
mod persistent_cache;
mod protocol;
mod reload;
pub mod resource;
mod runtime_session;
mod serial_worker;
mod source_map;
use wabou_style::stylesheet as style_ir;
mod test_driver;
mod test_report;
mod ui_inbox;

#[cfg(test)]
mod css_support_matrix_test;
#[cfg(feature = "vite")]
mod vite;

pub use error::{Error, Result};
#[cfg(feature = "headless")]
pub use gpui_headless::{
    GpuiHeadlessHarness, GpuiHeadlessOptions, GpuiHeadlessOutput, GpuiHeadlessScreenshot,
};
pub use host::{
    HostBuilder, HostService, HostServiceContext, HostServiceHandle, ManagedHostService,
    managed_host_service,
};
pub use host_message::{
    HostMessage, HostMessageContext, HostMessageError, HostMessageHandle, HostMessagePayload,
    HostMessageRouter, RevisionedHostPublication, RevisionedHostPublisher,
    RevisionedHostSnapshot,
};
pub use image_resource::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use jsrt::{DEFAULT_QUICKJS_STACK_SIZE, JsRuntime, JsRuntimeOptions};
pub use native_capability::NativeCapability;
pub use persistent_cache::PersistentJsonCache;
/// Generated event codes shared with native widget adapters.
pub use protocol::event;
pub use rquickjs;
pub use serial_worker::SerialWorker;
pub use wabou_shell::{
    AppDirectories, AppDirectoryConfig, GpuiLayoutNode, NativeWidgetContext, NativeWidgetEventSink,
    NativeWidgetFactory, NativeWidgetMount, ProjectedNodeKind, RgbaColor, WindowBackground,
    WindowInputMode, WindowLevel, WindowMetrics, WindowOptions, WindowResourceKey, gpui,
    initial_window_resource_key,
};

mod intl;
