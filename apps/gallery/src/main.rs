//! Native host executable for the Wabou component gallery.

use snafu::{ResultExt, Whatever};
use wabou::WindowOptions;

#[cfg(not(feature = "vello-hybrid"))]
use wabou::HostBuilder as SelectedHostBuilder;
#[cfg(feature = "vello-hybrid")]
use wabou::WinitHostBuilder as SelectedHostBuilder;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let builder = SelectedHostBuilder::new()
        .app_directories("dev", "Wabou", "Gallery")
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title(if cfg!(feature = "vello-hybrid") {
                    "Wabou Components · Vello Hybrid"
                } else {
                    "Wabou Components"
                })
                .initial_inner_size(1280, 840)
                .min_inner_size(900, 600),
        )
        .capability(gallery::bindings::CAPABILITY, |capability| {
            capability.json_hot_method(
                gallery::bindings::DESCRIBE_PALETTE,
                gallery::bindings::describe_palette,
            )
        });

    #[cfg(not(feature = "vello-hybrid"))]
    let builder = builder.native_widget("fractal", gallery::fractal::gpui_factory());

    builder
        .run()
        .whatever_context("failed to run component gallery")
}
