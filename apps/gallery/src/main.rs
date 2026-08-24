//! Native host executable for the Wabou component gallery.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .app_directories("dev", "Wabou", "Gallery")
        .persist_window_size("main")
        .widget("fractal", || Box::new(gallery::fractal::JuliaWidget::new()))
        .window(
            WindowOptions::new()
                .title("Wabou Components")
                .initial_inner_size(1280, 840)
                .min_inner_size(900, 600),
        )
        .json_capability(gallery::bindings::CAPABILITY, |capability| {
            capability.method(
                gallery::bindings::DESCRIBE_PALETTE,
                gallery::bindings::describe_palette,
            )
        })
        .run()
        .whatever_context("failed to run component gallery")
}
