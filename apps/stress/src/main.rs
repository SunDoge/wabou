//! Render stress test — N nodes (static or restyled every frame) + a live
//! FPS / per-stage-ms overlay. Verifies the winit stack holds 60Hz+ across
//! node counts and pinpoints the stage that gives first.
//!
//! Build the JS bundle first (`bun run --filter @wabou/stress build`), then
//! Run with `wabou dev stress` or package with `wabou build stress`.

use snafu::{ResultExt, Whatever};
#[cfg(feature = "renderer-skia")]
use wabou::RendererBackend;
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window({
            let options = WindowOptions::new().initial_inner_size(800, 600);
            #[cfg(feature = "renderer-skia")]
            let options = options.renderer(RendererBackend::Skia);
            options
        })
        .run()
        .whatever_context("failed to run stress application")
}
