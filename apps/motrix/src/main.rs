mod activity;
mod aria2;
mod config;
mod nat;
mod task_archive;
mod torrent;

use snafu::{OptionExt, ResultExt, Whatever};
use wabou::{
    AppDirectories, AppDirectoryConfig, HostBuilder, WindowOptions, initial_window_resource_key,
};
use wabou_tray::{SystemTray, TrayImage};

fn tray_icon() -> TrayImage {
    const SIZE: u32 = 32;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    let set = |rgba: &mut [u8], x: u32, y: u32, color: [u8; 4]| {
        let offset = ((y * SIZE + x) * 4) as usize;
        rgba[offset..offset + 4].copy_from_slice(&color);
    };
    for y in 2..30 {
        for x in 2..30 {
            let corner_x = if x < 7 {
                7 - x
            } else if x > 24 {
                x - 24
            } else {
                0
            };
            let corner_y = if y < 7 {
                7 - y
            } else if y > 24 {
                y - 24
            } else {
                0
            };
            if corner_x * corner_x + corner_y * corner_y <= 25 {
                set(&mut rgba, x, y, [47, 134, 246, 255]);
            }
        }
    }
    for y in 7..19 {
        for x in 15..18 {
            set(&mut rgba, x, y, [255, 255, 255, 255]);
        }
    }
    for offset in 0..6 {
        set(&mut rgba, 16 - offset, 17 + offset, [255, 255, 255, 255]);
        set(&mut rgba, 17 + offset, 17 + offset, [255, 255, 255, 255]);
    }
    for y in 24..26 {
        for x in 9..24 {
            set(&mut rgba, x, y, [255, 255, 255, 255]);
        }
    }
    TrayImage::from_rgba(rgba, SIZE, SIZE).expect("valid built-in Motrix tray icon")
}

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
    let tray_service = aria2.clone();
    let main_window = initial_window_resource_key(0);
    let tray = SystemTray::new(tray_icon())
        .tooltip("Motrix")
        .item("motrix.show", "Open Motrix", move |context| {
            context.show_window(main_window);
        })
        .separator()
        .item("motrix.quit", "Quit Motrix", move |context| {
            context.show_window(main_window);
            tray_service.request_quit();
        })
        .hide_window_on_close(main_window);
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
        .extension(tray)
        .host_message_producer(move |context| {
            aria2::stream_snapshots(context, stream_service.clone())
        });
    if let Some(service) = managed_aria2 {
        host = host.service(service);
    }
    host.run()
        .whatever_context("failed to run Motrix Wabou application")
}
