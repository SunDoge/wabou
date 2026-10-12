use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let usage = "seven_guis <library.so> [--watch <config.json>] [--capture] [--task <name>]";
    let library = args.first().ok_or(usage)?;
    let mut watch = None;
    let mut capture = false;
    let mut task = None;
    let mut options = args.iter().skip(1);
    while let Some(option) = options.next() {
        match option.as_str() {
            "--watch" if watch.is_none() => watch = Some(options.next().ok_or(usage)?),
            "--capture" if !capture => capture = true,
            "--task" if task.is_none() => task = Some(options.next().ok_or(usage)?),
            _ => return Err(usage.into()),
        }
    }
    // Keep a bounded cache of removed task editor/scroll/focus records. The
    // runtime still disposes native widgets and owns the actor state separately.
    let mut app = App::new(library, ".runtime")
        .renderer_retention(512)?
        .window(
            WindowOptions::new()
                .title("7GUIs")
                .initial_inner_size(900, 720),
        );
    if let Some(task) = task {
        app = app.initial_request(blockui_abi::key("SevenGuis.open_task"), task)?;
    }
    if let Some(config) = watch {
        app = app.watch(ReloadConfig::from_file(
            std::path::Path::new(config),
            &std::env::current_dir().map_err(|error| error.to_string())?,
        )?);
    }
    if capture {
        let (source, options) = app.into_window()?;
        wabou_shell_vello::run_window_with_options(Box::new(source.capture()), options)
            .map_err(|error| error.to_string())
    } else {
        app.run()
    }
}
