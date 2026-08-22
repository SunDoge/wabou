mod service;

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let service =
        service::ReaderService::new().whatever_context("failed to initialize manga OCR service")?;
    let capability = service.clone();
    HostBuilder::new()
        .app_directories("dev", "Wabou", "Manga OCR")
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title("Manga OCR · Wabou")
                .initial_inner_size(1440, 900)
                .min_inner_size(980, 640),
        )
        .json_capability(service::CAPABILITY, move |host| {
            service::mount(host, capability.clone())
        })
        .run()
        .whatever_context("failed to run Manga OCR")
}
