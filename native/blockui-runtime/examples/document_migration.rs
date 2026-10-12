use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("document_migration legacy current".into());
    }
    let mut actor = Actor::new(98, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[0]))?;
    actor.edit_text(key("legacy.notes"), "Old unsaved draft 🦀")?;
    let state = actor.state_address();
    let slot = actor.slot_address("routes.notes");
    actor.reload(Path::new(&args[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("routes.notes"), slot);
    assert_eq!(
        actor.text_list("routes.document_notes"),
        ["42", "Old unsaved draft 🦀"]
    );
    assert_eq!(actor.list("routes.dirty_documents"), [42]);
    actor.dispatch(Message {
        action: key("routes.open"),
        value: 73,
    })?;
    assert_eq!(actor.text("routes.notes"), "");
    actor.edit_text(key("routes.document.73.notes"), "Second document")?;
    actor.dispatch(Message {
        action: key("routes.back"),
        value: 0,
    })?;
    assert_eq!(actor.text("routes.notes"), "Old unsaved draft 🦀");
    println!(
        "PASS shared draft migration retains state/slot and archives active document; independent edits and Back preserve both drafts"
    );
    let mut home = Actor::new(99, Path::new(".runtime"))?;
    home.reload(Path::new(&args[0]))?;
    home.dispatch(Message {
        action: key("legacy.home"),
        value: 0,
    })?;
    home.edit_text(key("legacy.notes"), "Unassigned legacy draft")?;
    home.reload(Path::new(&args[1]))?;
    assert_eq!(
        home.text("routes.recovered_notes"),
        "Unassigned legacy draft"
    );
    assert_eq!(
        home.view.bindings[&key("routes.home.recovered")].text,
        "Unassigned legacy draft"
    );
    assert!(home.text_list("routes.document_notes").is_empty());
    home.edit_text(key("routes.home.recovered"), "Edited recovered draft")?;
    home.dispatch(Message {
        action: key("routes.open"),
        value: 42,
    })?;
    assert_eq!(home.text("routes.notes"), "");
    home.dispatch(Message {
        action: key("routes.back"),
        value: 0,
    })?;
    assert_eq!(
        home.view.bindings[&key("routes.home.recovered")].text,
        "Edited recovered draft"
    );
    home.edit_text(key("routes.home.recovered"), "")?;
    assert!(
        home.view
            .bindings
            .contains_key(&key("routes.home.recovered"))
    );
    println!(
        "PASS unassigned legacy Home text remains an editable recovered draft across navigation; clearing keeps the editor mounted"
    );
    #[cfg(feature = "native")]
    blockui_runtime::native::document_recovery_layout_proof(home)?;
    Ok(())
}
