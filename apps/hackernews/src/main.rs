//! Hacker News app — multi-page Solid app with router + sidebar.
//!
//! Build the JS bundle first (`bun run --filter @wabou/hackernews build`),
//! then `cargo run -p hackernews`.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window(
            WindowOptions::new()
                .title("Wabou Hacker News")
                .initial_inner_size(1024, 768)
                .min_inner_size(720, 480),
        )
        .run()
        .whatever_context("failed to run Hacker News application")
}
