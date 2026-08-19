mod aria2;
mod config;

use snafu::{OptionExt, ResultExt, Whatever};
use wabou::{AppDirectories, AppDirectoryConfig, HostBuilder, WindowOptions};

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let directory_config = AppDirectoryConfig::new("dev", "Wabou", "Motrix");
    let executable =
        std::env::current_exe().whatever_context("failed to resolve executable path")?;
    let resource_dir = executable
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("resources");
    let directories = AppDirectories::resolve(&directory_config, resource_dir)
        .whatever_context("failed to resolve application directories")?;
    let config_store = config::ConfigStore::new(&directories.config_dir);
    let app_config = config_store
        .load()
        .whatever_context("failed to load Motrix configuration")?;
    let (aria2, managed_aria2) = aria2::Aria2Service::from_config(app_config, config_store)
        .whatever_context("failed to configure aria2")?;
    let capability_service = aria2.clone();
    let stream_service = aria2.clone();
    let mut host = HostBuilder::new()
        .app_directory_config(directory_config)
        .persist_window_size("main")
        .window(
            WindowOptions::new()
                .title("Motrix · Wabou")
                .initial_inner_size(1280, 820)
                .min_inner_size(900, 600),
        )
        .json_capability(aria2::CAPABILITY, move |capability| {
            aria2::mount(capability, capability_service.clone())
        })
        .host_message_producer(move |context| {
            aria2::stream_snapshots(context, stream_service.clone())
        });
    if let Some(service) = managed_aria2 {
        host = host.service(service);
    }
    host.run()
        .whatever_context("failed to run Motrix Wabou application")
}
