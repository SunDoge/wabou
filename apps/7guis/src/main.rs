//! Native host for the Wabou 7GUIs benchmark application.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window(
            WindowOptions::new()
                .title("7GUIs — Wabou")
                .initial_inner_size(1180, 780)
                .min_inner_size(820, 600),
        )
        .run()
        .whatever_context("failed to run the Wabou 7GUIs application")
}
