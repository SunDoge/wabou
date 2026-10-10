use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let usage = "preferences_app <library.so> [--watch <config.json>]";
    let library = args.first().ok_or(usage)?;
    let mut app = App::new(library, ".runtime").window(
        WindowOptions::new()
            .title("Preferences")
            .initial_inner_size(480, 560),
    );
    match args.as_slice() {
        [_] => {}
        [_, flag, config] if flag == "--watch" => {
            app = app.watch(ReloadConfig::from_file(
                std::path::Path::new(config),
                &std::env::current_dir().map_err(|error| error.to_string())?,
            )?);
        }
        _ => return Err(usage.into()),
    }
    app.run()
}
