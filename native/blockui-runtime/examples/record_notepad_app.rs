use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let usage = "record_notepad_app <library.so> [--watch <config.json>]";
    let library = args.next().ok_or(usage)?;
    let mut app = App::new(library, ".runtime").window(
        WindowOptions::new()
            .title("Scratchpad")
            .initial_inner_size(640, 480),
    );
    if let Some(option) = args.next() {
        if option != "--watch" {
            return Err(usage.into());
        }
        let configuration = args.next().ok_or(usage)?;
        app = app.watch(ReloadConfig::from_file(
            std::path::Path::new(&configuration),
            &std::env::current_dir().map_err(|error| error.to_string())?,
        )?);
    }
    if args.next().is_some() {
        return Err(usage.into());
    }
    app.run()
}
