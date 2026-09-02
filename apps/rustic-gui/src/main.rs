mod service;

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let service = service::RusticService::default();
    HostBuilder::new()
        .app_directories("dev", "Wabou", "Rustic GUI")
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title("Rustic GUI · Wabou")
                .initial_inner_size(1240, 780)
                .min_inner_size(900, 620),
        )
        .capability(service::CAPABILITY, move |host| {
            service::mount(host, service.clone())
        })
        .run()
        .whatever_context("failed to run Rustic GUI")
}
