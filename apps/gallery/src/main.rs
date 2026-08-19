//! Native host executable for the Wabou component gallery.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .widget("fractal", || Box::new(gallery::fractal::JuliaWidget::new()))
        .window(
            WindowOptions::new()
                .title("Wabou Components")
                .initial_inner_size(1080, 760)
                .min_inner_size(720, 520),
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
