//! Reject malformed reloadable shortcut declarations before publishing code/state.
use blockui_abi::key;
use blockui_runtime::Actor;
use std::path::Path;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err("shortcut_admission good duplicate invalid".into());
    }
    let mut actor = Actor::new(731, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[1]))?;
    actor.set_text(key("editor.draft"), "Unsaved before invalid reload 😀")?;
    let state = actor.state_address();
    let slot = actor.slot_address("editor.draft");
    let implementation = actor.implementation_version();
    let order = actor.view.order.clone();
    for (path, expected) in [
        (&args[2], "duplicate primary shortcut"),
        (&args[3], "ASCII letter"),
    ] {
        let error = actor.reload(Path::new(path)).unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(actor.state_address(), state);
        assert_eq!(actor.slot_address("editor.draft"), slot);
        assert_eq!(
            actor.text("editor.draft"),
            "Unsaved before invalid reload 😀"
        );
        assert_eq!(actor.view.order, order);
        assert_eq!(actor.artifact_count(), 1);
    }
    println!(
        "PASS duplicate/non-ASCII shortcut candidates reject atomically and retain running implementation, draft, slot, view and one artifact"
    );
    Ok(())
}
