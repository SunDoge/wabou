//! wabou-runtime: QuickJS + SolidJS host driving wabou-shell's renderer.
//!
//! SolidJS (via `solid-js/universal`) emits binary DOM-mutation ops into a
//! `Writer`; one `requestAnimationFrame` tick per frame flushes them to Rust;
//! [`Applier`] decodes + applies them to a retained taffy tree; wabou-shell
//! lays out + rasterises at vsync. Modeled on blitz-js.

#![warn(missing_docs)]

mod actor;
mod applier;
mod asset_cache;
mod atom;
mod bundle;
mod clock;
mod config;
mod effect_trace;
mod error;
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
mod native_capability;
mod persistent_cache;
mod protocol;
pub mod resource;
mod serial_worker;
mod source_map;
mod style_ir;
mod test_driver;
mod test_report;

#[cfg(test)]
mod css_support_matrix_test;
#[cfg(feature = "vite")]
pub mod vite;
mod widget;
pub mod widget_api;

pub use applier::{Applier, ComputedNodeSnapshot, HmrDrainResult, ReloadHandle, ReloadMsg};
pub use config::AppConfig;
pub use error::{Error, Result};
pub use host::{
    HostBuilder, HostService, HostServiceContext, HostServiceHandle, ManagedHostService,
    managed_host_service,
};
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, ResizeObservation,
};
pub use host_message::{
    DEFAULT_HOST_MESSAGE_CAPACITY, HostMessage, HostMessageContext, HostMessageError,
    HostMessageHandle, HostMessagePayload, HostMessageRouter, MAX_HOST_MESSAGES_PER_FRAME,
    RevisionedHostPublication, RevisionedHostPublisher, RevisionedHostSnapshot,
};
pub use image_resource::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use json_capability::JsonCapability;
pub use jsrt::{DEFAULT_QUICKJS_STACK_SIZE, JsRuntime, JsRuntimeOptions};
pub use native_capability::NativeCapability;
pub use persistent_cache::PersistentJsonCache;
/// Generated event codes shared with native widget adapters.
pub use protocol::event;
pub use rquickjs;
pub use serial_worker::SerialWorker;
pub use vello;
pub use wabou_shell::PaintScene;
pub use wabou_shell::anyrender;
pub use wabou_shell::{
    AppDirectories, AppDirectoryConfig, ExtensionContext, FrameSource, FrameSourceFactory,
    RendererBackend, RunOutcome, ShellExtension, TextContext, WindowInputMode, WindowLevel,
    WindowMetrics, WindowOptions, WindowResourceKey, initial_window_resource_key, run_window,
    run_window_with_options, run_window_with_size, run_windows, run_windows_with_factory,
    run_windows_with_factory_and_extensions, style,
};
pub use widget::WidgetFactory;
pub use widget::{
    Canvas, MeasureContext, PaintContext, PasswordInput, SecretStore, TextInput, Widget,
    WidgetAccessibility, WidgetAvailableSpace, WidgetChanges, WidgetEventResult, WidgetGeometry,
    WidgetHarness, WidgetNodeEvent, WidgetStyle, decode_widget_config,
};

#[cfg(feature = "vite")]
pub use vite::{HmrClient, ViteError, start_hmr_client, vite_url_from_env};
mod intl;
