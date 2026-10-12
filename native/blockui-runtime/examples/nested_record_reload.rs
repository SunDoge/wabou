use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("nested_record_reload original compatible missing upgraded".into());
    }
    let mut actor = Actor::new(198, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(Message {
        action: key("put"),
        value: 0,
    })?;
    actor.dispatch(Message {
        action: key("clear_backup"),
        value: 0,
    })?;
    let state = actor.state_address();
    let slot = actor.slot_address("contacts");
    let buffer = actor.text_list("contacts").as_ptr();
    let schema = actor.schema_version;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text_list("contacts").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&key("status")].text, "Compatible");
    let implementation = actor.implementation_version();
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert_eq!(actor.implementation_version(), implementation);
    assert_eq!(actor.text_list("contacts").as_ptr(), buffer);
    actor.dispatch(Message {
        action: key("put"),
        value: 0,
    })?;
    let draft = actor.text("draft").as_ptr();
    actor.reload(Path::new(&paths[3]))?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("contacts"), slot);
    assert_eq!(actor.text("draft").as_ptr(), draft);
    assert_eq!(
        actor.view.bindings[&key("status")].text,
        "Ada 🦀 · 香港\n東京 · false"
    );
    assert_eq!(actor.artifact_count(), 1);
    println!(
        "PASS nested record schema propagates child changes through real DLLs; compatible reload retains encoded allocation, absent optional child and empty nested list still require migration and keeps old code on rejection, explicit nested Rust upgrade preserves identity, Unicode fields and ordered record lists"
    );
    Ok(())
}
