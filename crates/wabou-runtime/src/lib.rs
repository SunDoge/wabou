//! wabou-runtime: QuickJS + SolidJS host driving the GPUI wabou-shell.
//!
//! SolidJS (via `solid-js/universal`) emits binary DOM-mutation ops into a
//! `Writer`; one `requestAnimationFrame` tick per frame flushes them to Rust;
//! [`RuntimeController`] decodes and applies them to the retained GPUI projection. The
//! old Winit/Vello applier remains temporarily available only as a migration
//! oracle and is never an application-selectable backend.

#![warn(missing_docs)]

// Kept only for the legacy debug projection while its in-flight extraction is
// completed. New runtime code must name `legacy_shell` explicitly.
extern crate legacy_shell as wabou_shell;

mod actor;
mod applier;
mod atom;
mod bundle;
mod clock;
mod config;
mod effect_trace;
mod error;
mod gpui_view;
mod headless_test;
mod host;
#[cfg(test)]
pub(crate) mod host_abi;
mod host_ffi;
mod host_frame;
mod host_message;
mod image_resource;
mod inline_context;
mod json_capability;
mod jsrt;
mod kv;
mod native_capability;
mod persistent_cache;
mod protocol;
pub mod resource;
mod serial_worker;
mod source_map;
mod style_ir;
mod test_driver;
mod test_report;
mod ui_inbox;

#[cfg(test)]
mod css_support_matrix_test;
#[cfg(feature = "vite")]
pub mod vite;
#[cfg(test)]
mod widget;

pub(crate) use applier::RuntimeController;
pub use applier::{HmrDrainResult, ReloadHandle, ReloadMsg};
pub use config::AppConfig;
pub use error::{Error, Result};
pub use gpui_shell::{
    AppDirectories, AppDirectoryConfig, NativeWidgetContext, NativeWidgetFactory,
    NativeWidgetMount, RgbaColor, WindowInputMode, WindowLevel, WindowMetrics, WindowOptions,
    WindowResourceKey, gpui, initial_window_resource_key,
};
pub use gpui_view::GpuiRuntimeView;
pub use host::{
    HostBuilder, HostService, HostServiceContext, HostServiceHandle, ManagedHostService,
    managed_host_service,
};
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, NumericEventData, ResizeObservation,
};
pub use host_message::{
    DEFAULT_HOST_MESSAGE_CAPACITY, HostMessage, HostMessageContext, HostMessageError,
    HostMessageHandle, HostMessagePayload, HostMessageRouter, MAX_HOST_MESSAGES_PER_FRAME,
    RevisionedHostPublication, RevisionedHostPublisher, RevisionedHostSnapshot,
};
pub use image_resource::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use json_capability::JsonCapability;
pub use jsrt::{DEFAULT_QUICKJS_STACK_SIZE, JsRuntime, JsRuntimeOptions};
use legacy_shell::{
    FrameSource, FrameSourceFactory, RunOutcome, TextContext,
    run_windows_with_factory_and_extensions, style,
};
pub use native_capability::NativeCapability;
pub use persistent_cache::PersistentJsonCache;
/// Generated event codes shared with native widget adapters.
pub use protocol::event;
pub use rquickjs;
pub use serial_worker::SerialWorker;

/// Transitional API used by the legacy Winit headless verification tools.
///
/// Applications must use [`HostBuilder`] and the GPUI shell instead. This
/// module will disappear when the layout oracle has moved out of the runtime.
#[doc(hidden)]
pub mod legacy_headless {
    pub use crate::applier::{ComputedNodeSnapshot, RuntimeController as Applier};
}

#[cfg(feature = "vite")]
pub use vite::{HmrClient, ViteError, start_hmr_client, vite_url_from_env};
mod intl;
