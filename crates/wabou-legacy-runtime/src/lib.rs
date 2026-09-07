//! Winit + Taffy application backend rendered by Vello Hybrid.
//!
//! Application code normally reaches this backend through `wabou::HostBuilder`;
//! `wabou::VelloHybridHostBuilder` is available when the backend identity should
//! be explicit.

#![allow(missing_docs)]

extern crate vello_shell as wabou_shell;

mod atom;
mod behavior_test;
mod bundle {
    pub use runtime_api::{load_bundle as load, load_source_map, resource_directory};
}
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
mod headless_test;
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
    #[cfg(feature = "vite")]
    pub use runtime_api::is_vite_side_effect_update;
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
pub use host::VelloHybridHostBuilder;
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, NumericEventData, ResizeObservation,
};
pub use host_message::{
    HostMessage, HostMessageContext, HostMessageHandle, HostMessagePayload, HostMessageRouter,
};
pub use jsrt::{JsRuntime, JsRuntimeOptions};
pub use runtime_api::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use vello_shell::{
    EffectRequest as VelloHybridEffectRequest, ExtensionContext as VelloHybridExtensionContext,
    PaintContext as VelloHybridPaintContext, Point as VelloHybridPoint,
    PointerButton as VelloHybridPointerButton, PointerPhase as VelloHybridPointerPhase,
    ShellExtension as VelloHybridShellExtension, TextContext as VelloHybridTextContext,
    WakeCallback as VelloHybridWakeCallback, Widget, WidgetChanges as VelloHybridWidgetChanges,
    WidgetFactory, WidgetRasterImage as VelloHybridRasterImage, WindowOptions,
};
pub use wabou_legacy_widgets::SecretStore as VelloHybridSecretStore;

mod applier;

pub use applier::{ComputedNodeSnapshot, LegacyRuntimeController as Applier};
