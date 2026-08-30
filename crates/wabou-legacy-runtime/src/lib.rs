//! Retired Winit/Vello runtime retained only as a deterministic migration oracle.
//!
//! Applications must use `wabou-runtime`. This crate is not a renderer backend
//! and is intentionally excluded from Wabou's public facade.

#![allow(missing_docs)]

extern crate legacy_shell as wabou_shell;

#[path = "../../wabou-runtime/src/atom.rs"]
mod atom;
#[path = "../../wabou-runtime/src/clock.rs"]
mod clock;
#[path = "../../wabou-runtime/src/config.rs"]
mod config;
#[path = "../../wabou-runtime/src/effect_bridge.rs"]
mod effect_bridge;
#[path = "../../wabou-runtime/src/effect_trace.rs"]
mod effect_trace;
#[path = "../../wabou-runtime/src/error.rs"]
mod error;
#[path = "../../wabou-runtime/src/gpui_controller.rs"]
mod gpui_controller;
#[path = "../../wabou-runtime/src/host_ffi.rs"]
mod host_ffi;
#[path = "../../wabou-runtime/src/host_frame.rs"]
mod host_frame;
#[path = "../../wabou-runtime/src/host_message.rs"]
mod host_message;
#[path = "../../wabou-runtime/src/image_resource.rs"]
mod image_resource;
#[path = "../../wabou-runtime/src/inline_context.rs"]
mod inline_context;
#[path = "../../wabou-runtime/src/intl.rs"]
mod intl;
#[path = "../../wabou-runtime/src/jsrt.rs"]
mod jsrt;
#[path = "../../wabou-runtime/src/protocol.rs"]
mod protocol;
#[path = "../../wabou-runtime/src/reload.rs"]
mod reload;
#[path = "../../wabou-runtime/src/resource.rs"]
pub mod resource;
#[path = "../../wabou-runtime/src/runtime_session.rs"]
mod runtime_session;
#[path = "../../wabou-runtime/src/source_map.rs"]
mod source_map;
#[path = "../../wabou-runtime/src/ui_inbox.rs"]
mod ui_inbox;
#[path = "../../wabou-runtime/src/widget.rs"]
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

#[path = "../../wabou-runtime/src/applier.rs"]
mod applier;

pub use applier::{ComputedNodeSnapshot, LegacyRuntimeController as Applier};
