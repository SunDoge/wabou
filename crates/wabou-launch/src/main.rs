//! Minimal launcher for a prebuilt runtime and JSON application configuration.
fn main() {
    if let Err(error) = launch() {
        eprintln!("wabou-launch: {error}");
        std::process::exit(1);
    }
}
fn launch() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let runtime = args
        .next()
        .ok_or("usage: wabou-launch RUNTIME_LIBRARY APP_CONFIG_JSON")?;
    let config = args
        .next()
        .ok_or("usage: wabou-launch RUNTIME_LIBRARY APP_CONFIG_JSON")?;
    if args.next().is_some() {
        return Err("unexpected extra argument".into());
    }
    let config_path = std::path::PathBuf::from(config)
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let config = std::fs::read(&config_path).map_err(|e| e.to_string())?;
    let mut config: wabou_launch::LaunchConfig =
        serde_json::from_slice(&config).map_err(|e| e.to_string())?;
    config.resolve_paths(
        config_path
            .parent()
            .ok_or("configuration has no directory")?,
    );
    // The command explicitly selects the trusted native runtime artifact.
    let runtime = unsafe { wabou_launch::RuntimeLibrary::load(std::path::Path::new(&runtime))? };
    runtime.run(&config)
}
