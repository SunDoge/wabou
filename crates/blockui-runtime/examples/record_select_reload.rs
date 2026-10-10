use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("record_select_reload base replacement invalid-options evolved".into());
    }
    let mut actor = Actor::new(217, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let select = key("choice");
    actor.edit_select(select, 1)?;
    let state = actor.state_address();
    let slot = actor.slot_address("select.fields.settings");
    let buffer = actor.text("select.fields.settings").as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.text("select.fields.settings").as_ptr(), buffer);
    actor.edit_select(select, 2)?;
    assert_eq!(actor.view.bindings[&key("edits")].text, "3");
    let buffer = actor.text("select.fields.settings").as_ptr();
    let version = actor.implementation_version();
    assert!(
        actor
            .reload(Path::new(&paths[2]))
            .unwrap_err()
            .contains("matching an option")
    );
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.text("select.fields.settings").as_ptr(), buffer);
    let binding = actor.view.bindings[&select].node.binding;
    actor.reload(Path::new(&paths[3]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("select.fields.settings"), slot);
    assert_eq!(actor.view.bindings[&select].node.binding, binding);
    assert_eq!(actor.integer_control_value(select), Some(2));
    assert_eq!(actor.view.bindings[&key("title")].text, "Appearance");
    let evaluations = actor.view.bindings[&select].evaluations;
    actor.dispatch(Message {
        action: key("select.fields.metadata"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&select].evaluations, evaluations);
    let buffer = actor.text("select.fields.settings").as_ptr();
    for value in [3, 4, 99] {
        assert!(actor.edit_select(select, value).is_err());
        assert_eq!(actor.text("select.fields.settings").as_ptr(), buffer);
        assert_eq!(actor.integer_control_value(select), Some(2));
    }
    actor.edit_select(select, 1)?;
    assert_eq!(actor.view.bindings[&key("edits")].text, "7");
    println!(
        "PASS record select DLL: compatible buffer retention/current action, invalid options reject atomically, schema shifts selector with retained state/slot/binding; metadata skips select and failed choices roll back"
    );
    Ok(())
}
