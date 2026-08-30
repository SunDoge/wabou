//! Native host executable for the Wabou component gallery.

use snafu::{ResultExt, Whatever};
#[cfg(feature = "renderer-skia")]
use wabou::RendererBackend;
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .app_directories("dev", "Wabou", "Gallery")
        .persist_window_size("main")
        .gpui_widget("fractal", gallery::fractal::gpui_factory())
        .window({
            let options = WindowOptions::new()
                .title("Wabou Components")
                .initial_inner_size(1280, 840)
                .min_inner_size(900, 600);
            #[cfg(feature = "renderer-skia")]
            let options = options.renderer(RendererBackend::Skia);
            options
        })
        .json_capability(gallery::bindings::CAPABILITY, |capability| {
            capability.hot_method(
                gallery::bindings::DESCRIBE_PALETTE,
                gallery::bindings::describe_palette,
            )
        })
        .run()
        .whatever_context("failed to run component gallery")
}
