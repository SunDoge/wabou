//! Generic stable executable for applications launched by the BlockUI CLI.
use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig};
fn main() -> Result<(), String> {
    let options = blockui_dev::HostOptions::from_env();
    let mut app =
        App::new(&options.library, ".runtime").window(WindowOptions::new().title("BlockUI"));
    if let Some(config) = options.watch {
        app = app.watch(ReloadConfig::from_file(
            &config,
            &std::env::current_dir().map_err(|e| e.to_string())?,
        )?);
    }
    if options.capture {
        {
            let (source, options) = app.into_window()?;
            blockui_runtime::app::run_capture(source, options)
        }
    } else {
        app.run()
    }
}
