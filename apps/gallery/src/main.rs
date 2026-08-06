use snafu::{ResultExt, Whatever};
use wabou_quick::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .window(
            WindowOptions::new()
                .title("Wabou Components")
                .initial_inner_size(1080, 760)
                .min_inner_size(720, 520),
        )
        .run()
        .whatever_context("failed to run component gallery")
}
