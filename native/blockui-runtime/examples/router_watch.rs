fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("router_watch <config.json> <disposable-source.rs>".into());
    }
    let repo = std::env::current_dir().map_err(|e| e.to_string())?;
    let config =
        blockui_runtime::native::ReloadConfig::from_file(std::path::Path::new(&args[0]), &repo)?;
    blockui_runtime::native::router_watch_proof(config, std::path::Path::new(&args[1]))
}
