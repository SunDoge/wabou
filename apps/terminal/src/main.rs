//! Native host executable for the terminal example.

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, HostShellBackend, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .shell_backend(HostShellBackend::LegacyWinit)
        .widget("terminal", wabou_terminal::terminal_widget)
        .window(
            WindowOptions::new()
                .title("Wabou Terminal")
                .initial_inner_size(1000, 680)
                .min_inner_size(640, 400),
        )
        .run()
        .whatever_context("failed to run terminal demo")
}
