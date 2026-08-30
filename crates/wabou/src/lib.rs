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
pub use wabou_bindgen::{CapabilityContract, HostMethod, JsonCapabilityContract, JsonMethod};
pub use wabou_database::{
    AtomicCommit as KvAtomicCommit, KvCheck, KvEntry, KvKey, KvKeyPart, KvListOptions, KvMutation,
    KvStore, Versionstamp as KvVersionstamp,
};
pub use wabou_runtime::gpui;
pub use wabou_runtime::rquickjs;
pub use wabou_runtime::vello::peniko::Color;
pub use wabou_runtime::{
    AppDirectories, AppDirectoryConfig, Error, ExtensionContext, HostBuilder, HostMessage,
    HostMessageContext, HostMessageError, HostMessageHandle, HostMessagePayload, HostMessageRouter,
    HostService, HostServiceContext, HostServiceHandle, ImageResource, ImageResourceHandle,
    ImageResourceStore, JsonCapability, ManagedHostService, NativeCapability, NativeWidgetContext,
    NativeWidgetFactory, NativeWidgetMount, PersistentJsonCache, RendererBackend, Result,
    RevisionedHostPublication, RevisionedHostPublisher, RevisionedHostSnapshot, SerialWorker,
    ShellExtension, WindowInputMode, WindowLevel, WindowMetrics, WindowOptions, WindowResourceKey,
    initial_window_resource_key, managed_host_service, widget_api,
};
pub use wabou_runtime::{PaintScene, anyrender};
#[cfg(feature = "tray")]
pub use wabou_tray::{SystemTray, TrayContext, TrayImage};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_exposes_application_and_extension_entry_points() {
        let _builder = HostBuilder::new();
        let _window = WindowOptions::new().title("Facade test");
        let _transparent = Color::TRANSPARENT;
        let _: Option<&dyn widget_api::Widget> = None;
        let _: Option<widget_api::UiEvent> = None;
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

        let _ = Bindings::new()
            .capability(Capability::new(JsonCapabilityContract::new("workspace", 1)));
        let mut types = specta::Types::default();
        let _ = Payload::definition(&mut types);
    }
}
