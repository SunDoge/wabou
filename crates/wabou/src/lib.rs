//! Public Rust application API for Wabou.
//!
//! Applications should depend on this crate rather than Wabou's internal
//! implementation crates. The facade deliberately preserves one stable import
//! path while the renderer, QuickJS host, widgets, and platform crates evolve.
//!
//! ```no_run
//! use wabou::{HostBuilder, WindowOptions};
//!
//! # fn run() -> wabou::Result<()> {
//! HostBuilder::new()
//!     .window(WindowOptions::new().title("My Wabou app"))
//!     .run()
//! # }
//! ```

#[cfg(feature = "bindings")]
pub use wabou_bindgen::{Bindings, Capability, FunctionModule, NativeMethod, Type, specta};
pub use wabou_bindgen::{CapabilityContract, HostMethod, JsonMethod};
pub use wabou_database::{
    AtomicCommit as KvAtomicCommit, KvCheck, KvEntry, KvKey, KvKeyPart, KvListOptions, KvMutation,
    KvStore, Versionstamp as KvVersionstamp,
};
pub use wabou_runtime::RgbaColor as Color;
pub use wabou_runtime::rquickjs;
pub use wabou_runtime::{
    AppDirectories, AppDirectoryConfig, HostMessage, HostMessageContext, HostMessageError,
    HostMessageHandle, HostMessagePayload, HostMessageRouter, HostService, HostServiceContext,
    NativeCapability, PersistentJsonCache, RevisionedHostPublication, RevisionedHostPublisher,
    RevisionedHostSnapshot, SerialWorker, WindowBackground, WindowInputMode, WindowLevel,
    WindowOptions, WindowResourceKey, initial_window_resource_key,
};
#[cfg(feature = "gpui")]
pub use wabou_runtime::{
    Error as GpuiError, ImageResource as GpuiImageResource,
    ImageResourceHandle as GpuiImageResourceHandle, ImageResourceStore as GpuiImageResourceStore,
    Result as GpuiResult,
};
#[cfg(feature = "gpui")]
pub use wabou_runtime::{
    HostBuilder as GpuiHostBuilder, HostServiceHandle, ManagedHostService, NativeWidgetContext,
    NativeWidgetFactory, NativeWidgetMount, TextRenderingMode, gpui, managed_host_service,
};

#[cfg(feature = "vello-hybrid")]
pub use wabou_legacy_runtime::WinitHostBuilder as HostBuilder;
#[cfg(feature = "vello-hybrid")]
pub use wabou_legacy_runtime::{
    Error, ImageResource, ImageResourceHandle, ImageResourceStore, Result, Widget as WinitWidget,
    WinitEffectRequest, WinitExtensionContext, WinitHostBuilder, WinitPaintContext, WinitPoint,
    WinitPointerButton, WinitPointerPhase, WinitRasterImage, WinitSecretStore, WinitShellExtension,
    WinitTextContext, WinitWakeCallback, WinitWidgetChanges,
};
#[cfg(all(feature = "gpui", not(feature = "vello-hybrid")))]
pub use wabou_runtime::{
    Error, HostBuilder, ImageResource, ImageResourceHandle, ImageResourceStore, Result,
};
#[cfg(feature = "tray")]
pub use wabou_tray::{SystemTray, TrayContext, TrayImage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_exposes_default_application_entry_points() {
        let _builder = HostBuilder::new();
        #[cfg(feature = "gpui")]
        let _gpui_comparison = GpuiHostBuilder::new();
        let _window = WindowOptions::new().title("Facade test");
        let _transparent = Color::TRANSPARENT;
        let _: JsonMethod<(), bool> = JsonMethod::no_request("ready");
    }

    #[cfg(feature = "bindings")]
    #[test]
    fn bindings_feature_exposes_generation_entry_points() {
        #[allow(dead_code)]
        #[derive(Type)]
        struct Payload {
            ready: bool,
        }

        let _ =
            Bindings::new().capability(Capability::new(CapabilityContract::new("workspace", 1)));
        let mut types = specta::Types::default();
        let _ = Payload::definition(&mut types);
    }
}
