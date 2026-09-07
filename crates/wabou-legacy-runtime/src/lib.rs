//! Winit + Taffy runtime used by Wabou's experimental Vello Hybrid backend.
//!
//! The crate retains its transitional name while backend parity is completed.
//! Application code reaches it through `wabou::WinitHostBuilder` rather than
//! depending on this implementation crate directly.

#![allow(missing_docs)]

extern crate legacy_shell as wabou_shell;

mod atom;
mod behavior_test;
mod bundle;
mod config;
mod effect_bridge;
#[allow(dead_code)] // Fixture-only trace mode is retained for the Winit behavior harness.
mod effect_trace;
mod error;
#[allow(dead_code)] // Transitional protocol-session adapter; Winit uses its shared subset.
mod gpui_controller;
mod host;
#[cfg(test)]
use runtime_api::host_abi;
mod host_frame {
    pub use runtime_api::{
        HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, NumericEventData,
        ResizeObservation,
    };
}
mod host_message {
    pub use runtime_api::{
        DEFAULT_HOST_MESSAGE_CAPACITY, HostMessage, HostMessageContext, HostMessageHandle,
        HostMessageInbox, HostMessagePayload, HostMessageRouter, HostTaskTracker,
        host_message_channel,
    };
}
mod inline_context;
mod jsrt {
    pub use runtime_api::{
        HostFrameDisposition, JsRuntime, JsRuntimeOptions, LayoutMetric, LayoutMetricsSnapshot,
        LayoutRect, ResizeTargets,
    };
}
mod protocol;
mod reload;
pub mod resource;
mod runtime_session;
use runtime_api::ui_inbox;
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
pub use jsrt::{JsRuntime, JsRuntimeOptions};
pub use legacy_shell::{
    EffectRequest as WinitEffectRequest, ExtensionContext as WinitExtensionContext,
    PaintContext as WinitPaintContext, Point as WinitPoint, PointerButton as WinitPointerButton,
    PointerPhase as WinitPointerPhase, ShellExtension as WinitShellExtension,
    TextContext as WinitTextContext, WakeCallback as WinitWakeCallback, Widget,
    WidgetChanges as WinitWidgetChanges, WidgetFactory, WidgetRasterImage as WinitRasterImage,
    WindowOptions,
};
pub use runtime_api::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use wabou_legacy_widgets::SecretStore as WinitSecretStore;

mod applier;

pub use applier::{ComputedNodeSnapshot, LegacyRuntimeController as Applier};
