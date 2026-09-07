//! Backend-neutral QuickJS + SolidJS runtime with an optional GPUI host.
//!
//! SolidJS (via `solid-js/universal`) emits binary DOM-mutation ops into a
//! `Writer`; one `requestAnimationFrame` tick per frame flushes them to Rust;
//! A backend consumes the flushed mutations and projects them into its native
//! scene. The `gpui` feature provides the GPUI host; the Vello Hybrid backend
//! consumes the same runtime without activating GPUI.

#![warn(missing_docs)]

mod actor;
mod bundle;
#[doc(hidden)]
#[allow(missing_docs)]
pub mod clock;
mod effect_bridge;
mod effect_trace;
mod error;
#[cfg(feature = "gpui")]
mod gpui_controller;
#[cfg(feature = "gpui-headless")]
mod gpui_headless;
#[cfg(feature = "gpui")]
mod gpui_motion;
#[cfg(feature = "gpui")]
mod gpui_performance_hud;
#[cfg(feature = "gpui")]
mod gpui_projection_boundary;
#[cfg(feature = "gpui")]
mod gpui_view;
#[cfg(feature = "gpui")]
mod gpui_widgets;
#[cfg(feature = "gpui")]
mod gpui_windows;
#[cfg(feature = "gpui")]
mod host;
#[doc(hidden)]
#[allow(missing_docs)]
pub mod host_abi;
#[doc(hidden)]
#[allow(missing_docs)]
pub mod host_ffi;
mod host_frame;
mod host_message;
mod image_resource;
mod json_capability;
mod jsrt;
mod kv;
mod native_capability;
mod persistent_cache;
mod reload;
pub mod resource;
mod runtime_session;
mod serial_worker;
#[doc(hidden)]
#[allow(missing_docs)]
pub mod source_map;
use wabou_style::stylesheet as style_ir;
#[doc(hidden)]
pub mod test_driver;
#[doc(hidden)]
pub mod test_report;
#[doc(hidden)]
#[allow(missing_docs)]
pub mod ui_inbox;

#[cfg(test)]
mod css_support_matrix_test;
#[cfg(feature = "vite")]
mod vite;

#[doc(hidden)]
pub use bundle::{BundleError, load as load_bundle, load_source_map, resource_directory};
#[doc(hidden)]
pub use effect_bridge::{EffectBridge, decode_effect_payload};
#[doc(hidden)]
pub use effect_trace::EffectTrace;
pub use error::{Error, Result};
#[cfg(feature = "gpui-headless")]
pub use gpui_headless::{
    GpuiHeadlessHarness, GpuiHeadlessOptions, GpuiHeadlessOutput, GpuiHeadlessScreenshot,
    GpuiProjectionBoundaryCheckpoint, GpuiProjectionCheckpoint,
};
#[cfg(feature = "gpui")]
pub use host::{
    HostBuilder, HostServiceHandle, ManagedHostService, TextRenderingMode, managed_host_service,
};
#[doc(hidden)]
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, NumericEventData,
    ResizeObservation, encode_host_frame,
};
#[doc(hidden)]
pub use host_message::HostMessageRouteLease;
#[doc(hidden)]
pub use host_message::{
    DEFAULT_HOST_MESSAGE_CAPACITY, HostMessageInbox, HostTaskTracker, host_message_channel,
};
pub use host_message::{
    HostMessage, HostMessageContext, HostMessageError, HostMessageHandle, HostMessagePayload,
    HostMessageRouter, RevisionedHostPublication, RevisionedHostPublisher, RevisionedHostSnapshot,
};
pub use image_resource::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use jsrt::{DEFAULT_QUICKJS_STACK_SIZE, JsRuntime, JsRuntimeOptions};
#[doc(hidden)]
pub use jsrt::{
    HostFrameDisposition, LayoutMetric, LayoutMetricsSnapshot, LayoutRect, ResizeTargets,
};
#[doc(hidden)]
pub use kv::mount_kv_methods;
pub use native_capability::NativeCapability;
pub use persistent_cache::PersistentJsonCache;
#[cfg(feature = "vite")]
#[doc(hidden)]
pub use reload::is_vite_side_effect_update;
#[doc(hidden)]
pub use reload::{
    HmrBatch, HmrDrainResult, HmrJsUpdate, ReloadHandle, ReloadMsg, ReloadState, plan_hmr_batch,
};
pub use resource::{ResourceKey, ResourceRegistry};
pub use rquickjs;
#[doc(hidden)]
pub use runtime_session::RuntimeSession;
pub use serial_worker::SerialWorker;
#[cfg(feature = "vite")]
#[doc(hidden)]
pub use vite::{HmrClient, ViteError, ViteHmrEvent, ViteState, start_hmr_bridge};
#[cfg(feature = "gpui-headless")]
pub use wabou_shell::{GpuiLayoutNode, ProjectedNodeKind};
#[cfg(feature = "gpui")]
pub use wabou_shell::{NativeWidgetContext, NativeWidgetFactory, NativeWidgetMount, gpui};
pub use wabou_shell_api::{
    AppDirectories, AppDirectoryConfig, RgbaColor, WindowBackground, WindowInputMode, WindowLevel,
    WindowOptions, WindowResourceKey, initial_window_resource_key,
};
pub use wabou_shell_api::{HostService, HostServiceContext};

#[doc(hidden)]
#[allow(missing_docs)]
pub mod intl;
