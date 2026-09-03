mod service;

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let images = wabou::ImageResourceStore::default();
    let service = service::ReaderService::new(images.clone())
        .whatever_context("failed to initialize manga OCR service")?;
    let capability = service.clone();
    HostBuilder::with_image_resources(images)
        .app_directories("dev", "Wabou", "Manga OCR")
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title("Manga OCR · Wabou")
                .initial_inner_size(1440, 900)
                .min_inner_size(980, 640),
        )
        .capability(service::CAPABILITY, move |host| {
            service::mount(host, capability.clone())
        })
        .run()
        .whatever_context("failed to run Manga OCR")
}
