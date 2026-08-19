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
pub use wabou_bindgen::{Bindings, Capability, Type, specta};
pub use wabou_bindgen::{JsonCapabilityContract, JsonMethod};
pub use wabou_runtime::rquickjs;
pub use wabou_runtime::{
    AppDirectories, AppDirectoryConfig, Error, ExtensionContext, HostBuilder, HostMessage,
    HostMessageContext, HostMessageError, HostMessageHandle, HostMessagePayload, HostService,
    JsonCapability, Result, ShellExtension, WindowMetrics, WindowOptions, WindowResourceKey,
    initial_window_resource_key, widget_api,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_exposes_application_and_extension_entry_points() {
        let _builder = HostBuilder::new();
        let _window = WindowOptions::new().title("Facade test");
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
