//! Stable Gallery window. Optional capture arguments operate only on owned state.
use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig, native_model::NativeModel};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let usage = "gallery LIBRARY [--watch CONFIG] [--capture] [--theme light|dark|system] [--page 0..14] [--dialog]";
    let library = args.first().ok_or(usage)?;
    let mut watch = None;
    let mut capture = false;
    let mut dialog = false;
    let mut theme = None;
    let mut page = None;
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--watch" if watch.is_none() => watch = Some(iter.next().ok_or(usage)?),
            "--capture" if !capture => capture = true,
            "--dialog" if !dialog => dialog = true,
            "--theme" if theme.is_none() => {
                theme = Some(match iter.next().map(String::as_str) {
                    Some("system") => 0,
                    Some("light") => 1,
                    Some("dark") => 2,
                    _ => return Err(usage.into()),
                })
            }
            "--page" if page.is_none() => {
                page = Some(
                    iter.next()
                        .ok_or(usage)?
                        .parse::<usize>()
                        .ok()
                        .filter(|v| *v < 15)
                        .ok_or(usage)?,
                )
            }
            _ => return Err(usage.into()),
        }
    }
    let mut app = App::new(library, ".runtime").window(
        WindowOptions::new()
            .title("BlockUI — Components")
            .initial_inner_size(1120, 820),
    );
    if let Some(config) = watch {
        app = app.watch(ReloadConfig::from_file(
            std::path::Path::new(config),
            &std::env::current_dir().map_err(|e| e.to_string())?,
        )?);
    }
    let (mut source, options) = app.into_window()?;
    if let Some(theme) = theme {
        let key = source
            .actor
            .root_widget(blockui_abi::key("gallery.appearance.select"))
            .ok_or("missing appearance selector")?;
        source.actor.set_widget_select(key, theme)?;
    }
    if let Some(page) = page {
        let key = source
            .actor
            .root_widget(blockui_abi::key(&format!("nav.{page}")))
            .ok_or("missing page")?;
        source.actor.dispatch_widget(key)?;
    }
    if dialog {
        let key = source
            .actor
            .root_widget(blockui_abi::key("dialog.open"))
            .ok_or("--dialog requires the Dialog page")?;
        source.actor.dispatch_widget(key)?;
    }
    if capture {
        blockui_runtime::app::run_capture(source, options)
    } else {
        wabou_shell_vello::run_window_with_options(Box::new(source), options)
            .map_err(|e| e.to_string())
    }
}
