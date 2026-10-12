use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("record_state_reload <base> <replacement> <missing> <migrated>".into());
    }
    let mut actor = Actor::new(201, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let state = actor.state_address();
    let slot = actor.slot_address("document");
    let buffer = actor.text("document").as_ptr();
    let schema = actor.schema_version;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text("document").as_ptr(), buffer);
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert_eq!(actor.text("document").as_ptr(), buffer);
    actor.dispatch(Message {
        action: blockui_abi::key("edit"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("value")].text,
        "Draft 🦀: 17"
    );
    actor.reload(Path::new(&paths[0]))?;
    let before = actor.text("document").to_owned();
    let before_buffer = actor.text("document").as_ptr();
    assert!(
        actor
            .dispatch(Message {
                action: blockui_abi::key("edit"),
                value: -1
            })
            .is_err()
    );
    assert_eq!(actor.text("document"), before);
    assert_eq!(actor.text("document").as_ptr(), before_buffer);
    actor.reload(Path::new(&paths[3]))?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("document"), slot);
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("value")].text,
        "Draft 🦀: 17"
    );
    println!(
        "PASS typed record state across four real DLLs: compatible buffers retained, current code executes, missing migration and failed edits roll back, explicit record upgrade retains actor and slot identity"
    );
    Ok(())
}
