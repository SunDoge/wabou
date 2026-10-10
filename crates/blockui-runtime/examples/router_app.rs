//! Interactive router host using only the public application API.
use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let usage =
        "router_app <library.so> [--route <canonical-path>] [--watch <config.json>] [--capture]";
    let library = args.first().ok_or(usage)?;
    let mut capture = false;
    let mut watch = None;
    let mut route = None;
    let mut options = args.iter().skip(1);
    while let Some(option) = options.next() {
        match option.as_str() {
            "--capture" if !capture => capture = true,
            "--watch" if watch.is_none() => watch = Some(options.next().ok_or(usage)?),
            "--route" if route.is_none() => route = Some(options.next().ok_or(usage)?),
            _ => return Err(usage.into()),
        }
    }
    let mut app = App::new(library, ".runtime")
        .renderer_retention(32)?
        .window(
            WindowOptions::new()
                .title("Documents")
                .initial_inner_size(640, 480),
        );
    if let Some(route) = route {
        app = app.initial_request(blockui_abi::key("routes.navigate"), route.as_str())?;
    }
    if let Some(config) = watch {
        let repo = std::env::current_dir().map_err(|error| error.to_string())?;
        app = app.watch(ReloadConfig::from_file(
            std::path::Path::new(config),
            &repo,
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
