//! A stable executable consumes the public framework API; behavior lives in a cdylib.
use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let library = args
        .get(1)
        .ok_or("native_app <actor.so> [--capture | --watch <repo>]")?;
    let mut app =
        App::new(library, ".runtime").window(WindowOptions::new().title("BlockUI — Public App"));
    if args.get(2).map(String::as_str).is_some_and(|arg| {
        arg == "--watch" || arg == "--watch-proof" || arg == "--scope-watch-proof"
    }) {
        let repo = args.get(3).ok_or("--watch needs the framework directory")?;
        app = app.watch(ReloadConfig::from_environment(std::path::Path::new(repo)));
    }
    if args.get(2).map(String::as_str) == Some("--scope-watch-proof") {
        let (source, options) = app.into_window()?;
        wabou_shell_vello::run_window_with_options(Box::new(source.scope_watch_smoke()), options)
            .map_err(|error| error.to_string())
    } else if args.get(2).map(String::as_str) == Some("--scope-proof") {
        let (source, options) = app.into_window()?;
        wabou_shell_vello::run_window_with_options(Box::new(source.scope_smoke()), options)
            .map_err(|error| error.to_string())
    } else if args.get(2).map(String::as_str) == Some("--capture") {
        let (source, options) = app.into_window()?;
        wabou_shell_vello::run_window_with_options(Box::new(source.capture()), options)
            .map_err(|error| error.to_string())
    } else if args.get(2).map(String::as_str) == Some("--watch-proof") {
        let (source, options) = app.into_window()?;
        wabou_shell_vello::run_window_with_options(Box::new(source.watch_smoke()?), options)
            .map_err(|error| error.to_string())
    } else {
        app.run()
    }
}
