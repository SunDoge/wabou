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
mod effect_bridge {
    #[cfg(test)]
    pub use runtime_api::decode_effect_payload;
}
#[allow(dead_code)] // Fixture-only trace mode is retained for the Winit behavior harness.
mod effect_trace {
    pub use runtime_api::EffectTrace;
}
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
        HostMessage, HostMessageContext, HostMessageHandle, HostMessagePayload, HostMessageRouter,
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
mod reload {
    #[cfg(any(feature = "vite", test))]
    pub use runtime_api::ReloadMsg;
    #[cfg(test)]
    pub use runtime_api::plan_hmr_batch;
    pub use runtime_api::{HmrBatch, HmrDrainResult, ReloadHandle};
}
/// Generational resources shared by every Wabou backend.
pub mod resource {
    pub use runtime_api::{ResourceKey, ResourceRegistry};
}
mod runtime_session {
    pub use runtime_api::RuntimeSession;
}
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
