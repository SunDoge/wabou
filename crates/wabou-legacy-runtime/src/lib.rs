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
mod clock;
mod config;
mod effect_bridge;
mod effect_trace;
mod error;
mod gpui_controller;
#[cfg(test)]
mod host_abi;
mod host_ffi;
mod host_frame;
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
mod widget;

use wabou_style::stylesheet as style_ir;

pub use config::AppConfig;
pub use error::{Error, Result};
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, NumericEventData, ResizeObservation,
};
pub use host_message::{
    HostMessage, HostMessageContext, HostMessageHandle, HostMessagePayload, HostMessageRouter,
};
pub use image_resource::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use jsrt::{JsRuntime, JsRuntimeOptions};

mod applier;

pub use applier::{ComputedNodeSnapshot, LegacyRuntimeController as Applier};
