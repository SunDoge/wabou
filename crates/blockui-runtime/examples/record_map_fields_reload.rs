use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 3 {
        return Err("record_map_fields_reload base replacement evolved".into());
    }
    let mut actor = Actor::new(220, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(Message {
        action: key("Model.initialize"),
        value: 0,
    })?;
    let state = actor.state_address();
    let slot = actor.slot_address("map.fields.rows");
    let change = |value| Message {
        action: key("Model.change"),
        value,
    };
    let name = actor.computed_evaluations("map.fields.name");
    let whole = actor.computed_evaluations("map.fields.whole");
    actor.dispatch(change(0))?;
    assert_eq!(actor.computed_evaluations("map.fields.name"), name);
    assert!(actor.computed_evaluations("map.fields.whole") > whole);
    let buffer = actor.text_list("map.fields.rows").as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("map.fields.rows"), slot);
    assert_eq!(actor.text_list("map.fields.rows").as_ptr(), buffer);
    actor.dispatch(change(1))?;
    assert_eq!(actor.view.bindings[&key("name")].text, "Updated");
    actor.reload(Path::new(&paths[2]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("map.fields.rows"), slot);
    assert_eq!(actor.view.bindings[&key("name")].text, "Updated");
    assert_eq!(actor.view.bindings[&key("metadata")].text, "true");
    let name = actor.computed_evaluations("map.fields.name");
    actor.dispatch(change(0))?;
    actor.dispatch(change(3))?;
    assert_eq!(actor.computed_evaluations("map.fields.name"), name);
    let buffer = actor.text_list("map.fields.rows").as_ptr();
    assert!(actor.dispatch(change(2)).is_err());
    assert!(actor.dispatch(change(7)).is_err());
    assert_eq!(actor.text_list("map.fields.rows").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&key("name")].text, "Updated");
    actor.dispatch(change(6))?;
    actor.dispatch(change(5))?;
    assert_eq!(actor.view.bindings[&key("missing")].text, "Inserted");
    actor.dispatch(change(4))?;
    assert_eq!(actor.view.bindings[&key("name")].text, "Absent");
    actor.dispatch(Message {
        action: key("Model.initialize"),
        value: 0,
    })?;
    actor.dispatch(change(1))?;
    assert_eq!(actor.view.bindings[&key("name")].text, "Updated");
    println!(
        "PASS record collection field dependencies across real DLL and schema replacement: compatible state/slot/buffer retained, new code runs, reordered ordinals read correct fields, sibling/other-row edits skip computations, whole-row and absent reads invalidate, removal/recreation and conflicts roll back"
    );
    Ok(())
}
