//! wabou-quick: QuickJS + SolidJS host driving wabou-shell's renderer.
//!
//! SolidJS (via `solid-js/universal`) emits binary DOM-mutation ops into a
//! `Writer`; one `requestAnimationFrame` tick per frame flushes them to Rust;
//! [`Applier`] decodes + applies them to a retained taffy tree; wabou-shell
//! lays out + rasterises at vsync. Modeled on blitz-js.

pub mod applier;
pub mod atom;
pub mod clock;
pub mod config;
pub mod error;
pub mod host;
#[cfg(test)]
pub(crate) mod host_abi;
pub mod host_ffi;
pub mod host_frame;
pub mod host_msg;
pub mod inline_context;
pub mod jsrt;
pub mod protocol;
mod style_ir;

#[cfg(test)]
mod css_support_matrix_test;
#[cfg(feature = "vite")]
pub mod vite;
pub mod widget;
pub mod widget_api;

pub use applier::{Applier, ComputedNodeSnapshot, HmrDrainResult, ReloadHandle, ReloadMsg};
pub use atom::{Atom, AtomPool};
pub use clock::{Clock, SystemClock};
pub use config::AppConfig;
pub use error::{Error, Result};
pub use host::HostBuilder;
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, ResizeObservation,
};
pub use host_msg::{
    DEFAULT_HOST_MSG_CAPACITY, HostMsg, HostMsgError, HostMsgHandle, HostPayload,
    MAX_HOST_MSG_PER_FRAME,
};
pub use jsrt::JsRuntime;
pub use rquickjs;
pub use vello;
pub use wabou_shell::{
    ExtensionContext, FrameSource, FrameSourceFactory, ShellExtension, TextContext, WindowMetrics,
    WindowOptions, run_window, run_window_with_options, run_window_with_size, run_windows,
    run_windows_with_factory, run_windows_with_factory_and_extensions, style,
};
pub use widget::WidgetFactory;
pub use widget::{
    Canvas, PasswordInput, SecretStore, TextInput, Widget, WidgetNodeEvent, WidgetStyle,
};

#[cfg(feature = "vite")]
pub use vite::{ViteError, start_hmr_client, vite_url_from_env};
