use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    HostBuilder::new()
        .app_directories("dev", "Wabou", "Reader")
        .persist_window_size("main")
        .plugin(wabou_pdf::plugin())
        .window(
            WindowOptions::new()
                .title("Reader · Wabou")
                .initial_inner_size(1440, 900)
                .min_inner_size(980, 640),
        )
        .run()
        .whatever_context("failed to run Wabou Reader")
}
