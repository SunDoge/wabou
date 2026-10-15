mod shortcut;

use snafu::{ResultExt, Whatever};
use wabou::{HostBuilder, SystemTray, TrayImage, WindowBackground, WindowLevel, WindowOptions};
use wabou_launcher::service;

fn tray_icon() -> TrayImage {
    let size = 32_u32;
    let mut rgba = vec![0; (size * size * 4) as usize];
    for y in 4_u32..28 {
        for x in 4_u32..28 {
            let dx = i64::from(x) - 14;
            let dy = i64::from(y) - 14;
            let ring = dx * dx + dy * dy;
            let handle = x >= 20 && y >= 20 && x.abs_diff(y) <= 2;
            if (55..=115).contains(&ring) || handle {
                let offset = ((y * size + x) * 4) as usize;
                rgba[offset..offset + 4].copy_from_slice(&[104, 181, 255, 255]);
            }
        }
    }
    TrayImage::from_rgba(rgba, size, size).expect("valid built-in launcher icon")
}

#[snafu::report]
fn main() -> Result<(), Whatever> {
    let service = service::LauncherService::discover();
    let capability = service.clone();
    let tray = SystemTray::new(tray_icon())
        .tooltip("Wabou Launcher")
        .item("launcher.show", "Show Launcher", |context| {
            context.show_primary_window();
        })
        .separator()
        .item("launcher.quit", "Quit", |context| context.quit())
        .hide_window_on_close();

    HostBuilder::new()
        .app_directories("dev", "Wabou", "Launcher")
        .window(
            WindowOptions::new()
                .title("Wabou Launcher")
                .initial_inner_size(720, 500)
                .centered(true)
                .resizable(false)
                .decorations(false)
                .background(WindowBackground::Transparent)
                .window_level(WindowLevel::AlwaysOnTop),
        )
        .base_color([0, 0, 0, 0])
        .shell_extension(shortcut::LauncherShortcut::new())
        .shell_extension(tray)
        .capability(service::CAPABILITY, move |mut host| {
            service::mount(&mut host, capability.clone())
        })
        .run()
        .whatever_context("failed to run Wabou Launcher")
}
