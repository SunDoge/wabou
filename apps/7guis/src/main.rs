//! Native host for the Wabou 7GUIs benchmark application.

use snafu::{ResultExt, Whatever};
use wabou::WindowOptions;

#[cfg(not(feature = "vello-hybrid"))]
use wabou::HostBuilder as SelectedHostBuilder;
#[cfg(feature = "vello-hybrid")]
use wabou::WinitHostBuilder as SelectedHostBuilder;

#[snafu::report]
fn main() -> Result<(), Whatever> {
    SelectedHostBuilder::new()
        .window(
            WindowOptions::new()
                .title(if cfg!(feature = "vello-hybrid") {
                    "7GUIs — Wabou · Vello Hybrid"
                } else {
                    "7GUIs — Wabou"
                })
                .initial_inner_size(1180, 780)
                .min_inner_size(820, 600),
        )
        .run()
        .whatever_context("failed to run the Wabou 7GUIs application")
}
