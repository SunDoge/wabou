//! Visual parity experiment for the WebView-based meow-starter dashboard.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window(
            WindowOptions::new()
                .title("Meow Starter · Wabou")
                .initial_inner_size(1000, 680)
                .min_inner_size(820, 560)
                .decorations(false),
        )
        .run()
        .whatever_context("failed to run the Meow Starter Wabou experiment")
}
