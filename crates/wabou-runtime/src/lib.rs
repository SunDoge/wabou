//! wabou-runtime: QuickJS + SolidJS host driving wabou-shell's renderer.
//!
//! SolidJS (via `solid-js/universal`) emits binary DOM-mutation ops into a
//! `Writer`; one `requestAnimationFrame` tick per frame flushes them to Rust;
//! [`Applier`] decodes + applies them to a retained taffy tree; wabou-shell
//! lays out + rasterises at vsync. Modeled on blitz-js.

#![warn(missing_docs)]

mod applier;
mod asset_cache;
mod atom;
mod bundle;
mod clock;
mod config;
mod effect_trace;
mod error;
mod host;
#[cfg(test)]
pub(crate) mod host_abi;
mod host_ffi;
mod host_frame;
mod host_message;
mod inline_context;
mod json_capability;
mod jsrt;
mod protocol;
mod source_map;
mod style_ir;
mod test_driver;

#[cfg(test)]
mod css_support_matrix_test;
#[cfg(feature = "vite")]
pub mod vite;
mod widget;
pub mod widget_api;

pub use applier::{Applier, ComputedNodeSnapshot, HmrDrainResult, ReloadHandle, ReloadMsg};
pub use config::AppConfig;
pub use error::{Error, Result};
pub use host::HostBuilder;
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, ResizeObservation,
};
pub use host_message::{
    DEFAULT_HOST_MESSAGE_CAPACITY, HostMessage, HostMessageContext, HostMessageError,
    HostMessageHandle, HostMessagePayload, MAX_HOST_MESSAGES_PER_FRAME,
};
pub use jsrt::JsRuntime;
/// Generated event codes shared with native widget adapters.
pub use protocol::event;
pub use rquickjs;
pub use vello;
pub use wabou_shell::{
    AppDirectories, AppDirectoryConfig, ExtensionContext, FrameSource, FrameSourceFactory,
    ShellExtension, TextContext, WindowMetrics, WindowOptions, run_window, run_window_with_options,
    run_window_with_size, run_windows, run_windows_with_factory,
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
