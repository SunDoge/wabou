//! Transparent native-window experiment.

use snafu::{ResultExt, Whatever};
use wabou::{Color, HostBuilder, WindowLevel, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .base_color(Color::TRANSPARENT)
        .window(
            WindowOptions::new()
                .title("Wabou transparent window lab")
                .initial_inner_size(900, 600)
                .min_inner_size(640, 400)
                .decorations(false)
                .transparent(true)
                .window_level(WindowLevel::AlwaysOnTop),
        )
        .run()
        .whatever_context("failed to run transparent-window experiment")
}
