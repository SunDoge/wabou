//! Native host executable for the Wabou component gallery.

use snafu::{ResultExt, Whatever};
use wabou::WindowOptions;

#[cfg(feature = "gpui")]
use wabou::GpuiHostBuilder as SelectedHostBuilder;
#[cfg(not(feature = "gpui"))]
use wabou::HostBuilder as SelectedHostBuilder;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let builder = SelectedHostBuilder::new()
        .app_directories("dev", "Wabou", "Gallery")
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title(if cfg!(feature = "gpui") {
                    "Wabou Components"
                } else {
                    "Wabou Components · Vello Hybrid"
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

    #[cfg(feature = "gpui")]
    let builder = builder.native_widget("fractal", gallery::fractal::gpui_factory());
    #[cfg(not(feature = "gpui"))]
    let builder = builder.widget("fractal", || {
        Box::new(gallery::fractal::WinitFractal::default())
    });

    builder
        .run()
        .whatever_context("failed to run component gallery")
}
