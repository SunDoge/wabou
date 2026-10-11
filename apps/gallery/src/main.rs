//! Native host executable for the Wabou component gallery.

use snafu::{ResultExt, Whatever};
use std::sync::Arc;
use wabou::{CapabilityDevelopment, CapabilityModule, HostBuilder, ModuleProject, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let host = HostBuilder::new()
        .app_directories("dev", "Wabou", "Gallery")
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title("Wabou Components")
                .initial_inner_size(1280, 840)
                .min_inner_size(900, 600),
        )
        .widget("fractal", || {
            Box::new(gallery::fractal::WinitFractal::default())
        });
    let host = if std::env::var_os("WABOU_RUST_HOT_RELOAD").is_some() {
        let module = CapabilityModule::new(
            gallery::bindings::CAPABILITY.version(),
            Arc::new(|operation, _| Err(format!("unknown gallery host operation: {operation}"))),
        );
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut project = ModuleProject::new(root.join("../gallery-functions/Cargo.toml"));
        project.watch.push(root.join("../gallery-api"));
        project
            .watch
            .push(root.join("../../crates/wabou-capability-sdk"));
        project
            .watch
            .push(root.join("../../crates/wabou-capability-abi"));
        // Only this repository's trusted application module is loaded.
        let development = unsafe { CapabilityDevelopment::new(module.clone(), project) };
        host.host_message_producer(development.diagnostics())
            .service(development)
            .capability(gallery::bindings::CAPABILITY, move |capability| {
                capability.json_module_method(gallery::bindings::DESCRIBE_PALETTE, module.clone())
            })
    } else {
        host.capability(gallery::bindings::CAPABILITY, |capability| {
            capability.json_method(gallery::bindings::DESCRIBE_PALETTE, |request| async move {
                gallery_functions::describe_palette(request)
            })
        })
    };
    host.run()
        .whatever_context("failed to run component gallery")
}
