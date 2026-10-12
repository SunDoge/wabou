//! Start a parent actor after registering the implementation used by its children.
use blockui_runtime::{ActorId, App, app::WindowOptions, native::ReloadConfig};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let parent = args
        .get(1)
        .ok_or("declared_app <parent.so> <counter.so> [--verify]")?;
    let child = args.get(2).ok_or("missing counter implementation")?;
    let verify = args.get(3).map(String::as_str) == Some("--verify");
    if verify {
        let config =
            ReloadConfig::from_environment(&std::env::current_dir().map_err(|e| e.to_string())?);
        let rejected = App::new(parent, ".runtime")
            .watch_implementation("counter_typo", config)?
            .implementation("counter", child)?
            .into_window();
        assert!(
            matches!(rejected, Err(error) if error.contains("counter_typo") && error.contains("not registered"))
        );
        assert!(App::new(parent, ".runtime").into_window().is_err());
        let owned_root =
            std::path::Path::new(".runtime").join(format!("actor-1-{}", std::process::id()));
        assert!(
            !owned_root.exists(),
            "rejected startup leaked its root generations"
        );
    }
    let app = App::new(parent, ".runtime")
        .implementation("counter", child)?
        .window(WindowOptions::new().title("BlockUI — Declared Children"));
    if verify {
        let (source, options) = app.into_window()?;
        let runtime = source.actor.runtime();
        assert_eq!(runtime.len(), 3);
        let root = ActorId(1);
        let children: Vec<_> = runtime
            .declared_children()
            .into_iter()
            .filter(|(child, _)| runtime.parent(*child) == Some(root))
            .map(|(child, _)| child)
            .collect();
        assert_eq!(children.len(), 2);
        for child in children {
            assert_eq!(runtime.actor(child).unwrap().integer("count"), 0);
            assert!(
                source
                    .actor
                    .widget(child, blockui_abi::key("counter.notes"))
                    .is_some()
            );
        }
        println!(
            "PASS public App rejects unknown watcher and missing child implementation, then admits two declared children with mounted native identities"
        );
        wabou_shell_vello::run_window_with_options(Box::new(source.capture()), options)
            .map_err(|e| e.to_string())
    } else {
        app.run()
    }
}
