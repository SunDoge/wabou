//! Retired Winit/Vello runtime retained only as a deterministic migration oracle.
//!
//! Applications must use `wabou-runtime`. This crate is not a renderer backend
//! and is intentionally excluded from Wabou's public facade.
//!
//! Its runtime support and generated JavaScript are deliberately frozen inside
//! this crate. Do not source-include files from the formal GPUI runtime: the
//! oracle must keep compiling when GPUI ownership and APIs evolve.

#![allow(missing_docs)]

extern crate legacy_shell as wabou_shell;

mod atom;
mod bundle;
mod clock;
mod config;
mod effect_bridge;
#[allow(dead_code)] // Capability parity is restored incrementally on the Winit host.
mod effect_trace;
mod error;
#[allow(dead_code)] // Transitional protocol-session adapter; Winit uses its shared subset.
mod gpui_controller;
mod host;
#[cfg(test)]
mod host_abi;
mod host_ffi;
mod host_frame;
#[allow(dead_code)] // Public message producers are not mounted by WinitHostBuilder yet.
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
pub use legacy_shell::{Widget, WidgetFactory, WindowOptions};

mod applier;

pub use applier::{ComputedNodeSnapshot, LegacyRuntimeController as Applier};
