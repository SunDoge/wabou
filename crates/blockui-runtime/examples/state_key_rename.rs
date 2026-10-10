use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 3 {
        return Err("state_key_rename base renamed-field renamed-type".into());
    }
    let mut actor = Actor::new(96, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(Message {
        action: key("change"),
        value: 1,
    })?;
    let slot = actor.slot_address("checked");
    let storage = actor.state_address();
    let schema = actor.schema_version;
    let widget = *actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.slot == key("checked"))
        .unwrap()
        .0;
    assert_eq!(actor.integer("settings.visits"), 1);
    let visits = actor.slot_address("settings.visits");
    for (index, path) in paths[1..].iter().enumerate() {
        let value = i64::from(index % 2 != 0);
        let before = actor.view.bindings[&widget].text.clone();
        actor.enqueue(Message {
            action: key("change"),
            value,
        })?;
        actor.reload(Path::new(path))?;
        assert_eq!(actor.slot_address("checked"), slot);
        assert_eq!(actor.slot_address("settings.visits"), visits);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.view.bindings[&widget].text, before);
        actor.tick()?;
        assert_eq!(actor.integer("calls"), index as i64 + 2);
        assert_eq!(actor.integer("settings.visits"), index as i64 + 2);
        assert_eq!(actor.boolean("checked"), value != 0);
        assert_eq!(actor.artifact_count(), 1);
    }
    println!(
        "PASS Rust field/type rename retains explicit and namespace-derived state keys, storage/slots/schema, checkbox identity and queued action"
    );
    Ok(())
}
