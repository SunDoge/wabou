//! Static visual benchmarks based on the gpui-shell-benchs applications.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window(
            WindowOptions::new()
                .title("GPUI shell visual benchmarks · Wabou")
                .initial_inner_size(1440, 900)
                .min_inner_size(960, 640),
        )
        .run()
        .whatever_context("failed to run the GPUI shell visual benchmarks")
}
