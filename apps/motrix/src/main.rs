use motrix_wabou::downloads;
use snafu::{ResultExt, Whatever};
use wabou::{
    AppDirectoryConfig, HostBuilder, HostMessage, HostMessageRouter, WindowOptions,
    initial_window_resource_key,
};
use wabou::{SystemTray, TrayImage};

fn tray_icon() -> TrayImage {
    const SIZE: u32 = 32;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    let set = |rgba: &mut [u8], x: u32, y: u32, color: [u8; 4]| {
        let offset = ((y * SIZE + x) * 4) as usize;
        rgba[offset..offset + 4].copy_from_slice(&color);
    };
    for y in 2_u32..30 {
        for x in 2_u32..30 {
            let corner_x = if x < 7 { 7 - x } else { x.saturating_sub(24) };
            let corner_y = if y < 7 { 7 - y } else { y.saturating_sub(24) };
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
    let (downloads, engine_service) = downloads::DownloadService::new();
    let capability_service = downloads.clone();
    let stream_service = downloads.clone();
    let main_window = initial_window_resource_key(0);
    let application_messages = HostMessageRouter::new();
    let tray_messages = application_messages.clone();
    let tray = SystemTray::new(tray_icon())
        .tooltip("Motrix")
        .item("motrix.show", "Open Motrix", move |context| {
            context.show_window(main_window);
        })
        .separator()
        .item("motrix.quit", "Quit Motrix", move |_context| {
            // Do not show the window first: close-to-tray (especially Wayland
            // surface recreation) would flash a frame before exit. JS exits
            // in place, or calls window.show() only when a confirm dialog is
            // needed.
            if let Err(error) =
                tray_messages.send_to(main_window, HostMessage::null(downloads::QUIT_REQUESTED))
            {
                tracing::warn!(?error, "could not enqueue quit request");
            }
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
        .json_capability(downloads::CAPABILITY, move |capability| {
            downloads::mount(capability, capability_service.clone())
        })
        .native_capability(downloads::NATIVE_CAPABILITY, downloads::mount_native)
        .extension(tray)
        .host_message_router(application_messages)
        .host_message_producer(move |context| {
            downloads::stream_snapshots(context, stream_service.clone())
        });
    host = host.recoverable_service(engine_service);
    host.run()
        .whatever_context("failed to run Motrix Wabou application")
}
