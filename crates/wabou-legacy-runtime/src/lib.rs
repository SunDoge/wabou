//! Winit + Taffy runtime used by Wabou's experimental Vello Hybrid backend.
//!
//! The crate retains its transitional name while backend parity is completed.
//! Application code reaches it through `wabou::WinitHostBuilder` rather than
//! depending on this implementation crate directly.

#![allow(missing_docs)]

extern crate legacy_shell as wabou_shell;

mod atom;
mod bundle;
mod clock;
mod config;
mod effect_bridge;
#[allow(dead_code)] // Fixture-only trace mode is retained for the Winit behavior harness.
mod effect_trace;
mod error;
#[allow(dead_code)] // Transitional protocol-session adapter; Winit uses its shared subset.
mod gpui_controller;
mod host;
#[cfg(test)]
mod host_abi;
mod host_ffi;
mod host_frame;
#[allow(dead_code)]
// Message queue is active; duplicated public helpers await shared-core extraction.
mod host_message;
mod image_resource;
mod inline_context;
mod intl;
mod jsrt;
mod protocol;
mod reload;
pub mod resource;
mod runtime_session;
mod source_map;
mod ui_inbox;
#[cfg(test)]
mod widget;

use wabou_style::stylesheet as style_ir;

pub use config::AppConfig;
pub use error::{Error, Result};
pub use host::WinitHostBuilder;
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, NumericEventData, ResizeObservation,
};
pub use host_message::{
    HostMessage, HostMessageContext, HostMessageHandle, HostMessagePayload, HostMessageRouter,
};
pub use image_resource::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use jsrt::{JsRuntime, JsRuntimeOptions};
pub use legacy_shell::{
    EffectRequest as WinitEffectRequest, ExtensionContext as WinitExtensionContext,
    Point as WinitPoint, PointerButton as WinitPointerButton, PointerPhase as WinitPointerPhase,
    ShellExtension as WinitShellExtension, WakeCallback as WinitWakeCallback, Widget,
    WidgetFactory, WindowOptions,
};
pub use wabou_legacy_widgets::SecretStore as WinitSecretStore;

mod applier;

pub use applier::{ComputedNodeSnapshot, LegacyRuntimeController as Applier};
