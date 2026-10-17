//! Native host executable for the Wabou component gallery.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
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
        })
        .host_module_def::<gallery::bindings::js_bindings_demo>("bindingsDemo")
        .run()
        .whatever_context("failed to run component gallery")
}
