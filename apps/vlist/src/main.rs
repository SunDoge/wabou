//! Virtual-list demo — 10,000 rows, only the visible slice is materialised.
//!
//! Build the JS bundle first (`bun run --filter @wabou/vlist build`), then
//! Run with `wabou dev vlist` or package with `wabou build vlist`.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window(WindowOptions::new().initial_inner_size(800, 600))
        .run()
        .whatever_context("failed to run vlist application")
}
