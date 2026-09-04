mod progress;
mod service;

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let service = service::RusticService::default();
    let progress_service = service.clone();
    HostBuilder::new()
        .app_directories("dev", "Wabou", "Timestow")
        .kv()
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title("Timestow · Wabou")
                .initial_inner_size(1240, 780)
                .min_inner_size(900, 620),
        )
        .host_message_producer(move |context| {
            progress_service.attach_progress_messages(context.messages().clone());
        })
        .capability(service::CAPABILITY, move |host| {
            service::mount(host, service.clone())
        })
        .run()
        .whatever_context("failed to run Timestow")
}
