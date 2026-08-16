//! Render stress test — N nodes (static or restyled every frame) + a live
//! FPS / per-stage-ms overlay. Verifies the winit stack holds 60Hz+ across
//! node counts and pinpoints the stage that gives first.
//!
//! Build the JS bundle first (`bun run --filter @wabou/stress build`), then
//! Run with `wabou dev stress` or package with `wabou build stress`.

use snafu::{ResultExt, Whatever};
use wabou_runtime::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window(WindowOptions::new().initial_inner_size(800, 600))
        .run()
        .whatever_context("failed to run stress application")
}
