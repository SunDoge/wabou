//! Native multi-document workspace, using only the public application entry.
use blockui_runtime::{App, app::WindowOptions, native::ReloadConfig};
use std::path::{Path, PathBuf};
fn configuration(repo: &Path, package: &str, artifact: &str) -> ReloadConfig {
    ReloadConfig {
        manifest: repo.join("Cargo.toml"),
        package: package.into(),
        artifact: PathBuf::from(artifact),
        target: repo.join("target"),
        sources: vec![
            repo.join(format!("crates/{package}")),
            repo.join("native/blockui"),
            repo.join("native/blockui-abi"),
            repo.join("native/blockui-macros"),
            repo.join("Cargo.toml"),
            repo.join("Cargo.lock"),
        ],
        features: String::new(),
        snapshots: repo.join(".runtime"),
    }
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let parent = args
        .get(1)
        .ok_or("documents_app workspace.so editor.so [--watch] [--capture]")?;
    let editor = args.get(2).ok_or("missing editor implementation")?;
    let mut app = App::new(parent, ".runtime")
        .implementation("editor", editor)?
        .window(WindowOptions::new().title("Documents"));
    for flag in args.iter().skip(3) {
        if !matches!(flag.as_str(), "--watch" | "--capture") {
            return Err(format!("unknown option {flag}"));
        }
    }
    if args.iter().skip(3).any(|flag| flag == "--watch") {
        let repo = std::env::current_dir().map_err(|e| e.to_string())?;
        if !repo.join("crates/documents-demo/Cargo.toml").is_file() {
            return Err("--watch must run from the blockui workspace".into());
        }
        app = app
            .watch(configuration(&repo, "documents-demo", parent))
            .watch_implementation("editor", configuration(&repo, "editor-demo", editor))?;
    }
    if args.iter().skip(3).any(|flag| flag == "--capture") {
        let (source, options) = app.into_window()?;
        wabou_shell_vello::run_window_with_options(Box::new(source.capture()), options)
            .map_err(|e| e.to_string())
    } else {
        app.run()
    }
}
