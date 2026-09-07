//! Native host for the Wabou 7GUIs benchmark application.

use snafu::{ResultExt, Whatever};
use wabou::WindowOptions;

#[cfg(feature = "gpui")]
use wabou::GpuiHostBuilder as SelectedHostBuilder;
#[cfg(not(feature = "gpui"))]
use wabou::HostBuilder as SelectedHostBuilder;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    SelectedHostBuilder::new()
        .window(
            WindowOptions::new()
                .title(if cfg!(feature = "gpui") {
                    "7GUIs — Wabou"
                } else {
                    "7GUIs — Wabou · Vello Hybrid"
                })
                .initial_inner_size(1180, 780)
                .min_inner_size(820, 600),
        )
        .run()
        .whatever_context("failed to run the Wabou 7GUIs application")
}
