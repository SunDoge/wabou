//! Public Rust application API for Wabou's Winit + Taffy + Vello runtime.

#![allow(missing_docs)]

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

use wabou_style::stylesheet as style_ir;

pub use config::AppConfig;
pub use error::{Error, Result};
pub use host::HostBuilder;
pub use host_frame::{
    HostEvent, HostFrameError, HostNodeEvent, NodeEventPayload, NumericEventData, ResizeObservation,
};
pub use host_message::{
    HostMessage, HostMessageContext, HostMessageHandle, HostMessagePayload, HostMessageRouter,
};
pub use jsrt::{JsRuntime, JsRuntimeOptions};
pub use runtime_api::HostModuleState;
pub use runtime_api::JsValue;
pub use runtime_api::{ImageResource, ImageResourceHandle, ImageResourceStore};
pub use wabou_shell::widgets::SecretStore;
pub use wabou_shell::{
    EffectRequest, ExtensionContext, NativeWidgetPlugin, PaintContext, Point, PointerButton,
    PointerPhase, ShellExtension, TextContext, WakeCallback, Widget, WidgetChanges, WidgetFactory,
    WidgetRasterImage, WindowOptions,
};

pub use runtime_api::RgbaColor as Color;
pub use runtime_api::rquickjs;
pub use runtime_api::{
    AppDirectories, AppDirectoryConfig, HostMessageError, HostService, HostServiceContext,
    HostServiceHandle, ManagedHostService, PersistentJsonCache, ResidentResources, ResourceKey,
    ResourceRegistry, RevisionedHostPublication, RevisionedHostPublisher, RevisionedHostSnapshot,
    SerialWorker, WindowBackground, WindowInputMode, WindowLevel, WindowResourceKey,
    initial_window_resource_key, managed_host_service,
};
pub use wabou_bindgen::{js, js_class, js_module, js_type};
pub use wabou_database::{
    AtomicCommit as KvAtomicCommit, KvCheck, KvEntry, KvKey, KvKeyPart, KvListOptions, KvMutation,
    KvStore, Versionstamp as KvVersionstamp,
};
#[cfg(feature = "tray")]
pub use wabou_tray::{SystemTray, TrayContext, TrayImage};

mod applier;

pub use applier::{Applier, ComputedNodeSnapshot};
