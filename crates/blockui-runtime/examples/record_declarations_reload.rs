use blockui_runtime::{Actor, Message};
use std::path::Path;

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 10 {
        return Err(
            "expected base, replacement, missing, noop, upgraded, additional, scoped, bad-default, checked-write, automatic DLLs".into(),
        );
    }
    let mut actor = Actor::new(202, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let state = actor.state_address();
    let slot = actor.slot_address("declared.document");
    let buffer = actor.text("declared.document").as_ptr();
    let schema = actor.schema_version;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text("declared.document").as_ptr(), buffer);
    for path in [&paths[2], &paths[3], &paths[7]] {
        assert!(actor.reload(Path::new(path)).is_err());
        assert_eq!(actor.text("declared.document").as_ptr(), buffer);
    }
    actor.dispatch(Message {
        action: blockui_abi::key("declared.edit"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("value")].text,
        "Draft 🦀: 17"
    );
    actor.reload(Path::new(&paths[0]))?;
    let before = actor.text("declared.document").to_owned();
    let before_buffer = actor.text("declared.document").as_ptr();
    for mode in [0, 1] {
        let error = actor
            .dispatch(Message {
                action: blockui_abi::key("declared.reentrant"),
                value: mode,
            })
            .unwrap_err();
        assert!(error.contains("Record changed during update"), "{error}");
        assert_eq!(actor.text("declared.document"), before);
        assert_eq!(actor.text("declared.document").as_ptr(), before_buffer);
    }
    actor.reload(Path::new(&paths[5]))?;
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("value")].text,
        "Draft 🦀: 17"
    );
    actor.reload(Path::new(&paths[4]))?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("declared.document"), slot);
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("value")].text,
        "Draft 🦀: 17"
    );
    let mut scoped = Actor::new(203, Path::new(".runtime"))?;
    scoped.reload(Path::new(&paths[6]))?;
    assert_eq!(
        scoped.view.bindings[&blockui_abi::key("value")].text,
        "Draft 🦀: 7"
    );
    let mut checked = Actor::new(204, Path::new(".runtime"))?;
    checked.reload(Path::new(&paths[8]))?;
    let before = checked.text("checked.document").to_owned();
    let buffer = checked.text("checked.document").as_ptr();
    for map in [0, 1] {
        let error = checked
            .dispatch(Message {
                action: blockui_abi::key("checked.invalid"),
                value: map,
            })
            .unwrap_err();
        assert!(error.contains("record payload rejected"), "{error}");
        assert_eq!(checked.integer("checked.count"), 0);
        assert_eq!(checked.text("checked.document"), before);
        assert_eq!(checked.text("checked.document").as_ptr(), buffer);
    }
    let mut automatic = Actor::new(208, Path::new(".runtime"))?;
    automatic.reload(Path::new(&paths[0]))?;
    let state = automatic.state_address();
    let slot = automatic.slot_address("declared.document");
    let schema = automatic.schema_version;
    automatic.reload(Path::new(&paths[9]))?;
    assert_ne!(automatic.schema_version, schema);
    assert_eq!(automatic.state_address(), state);
    assert_eq!(automatic.slot_address("declared.document"), slot);
    assert_eq!(
        automatic.view.bindings[&blockui_abi::key("value")].text,
        "Draft 🦀: 7"
    );
    let buffer = automatic.text("declared.document").as_ptr();
    let schema = automatic.schema_version;
    automatic.reload(Path::new(&paths[4]))?;
    assert_eq!(automatic.schema_version, schema);
    assert_eq!(automatic.text("declared.document").as_ptr(), buffer);

    println!(
        "PASS declared record state across ten real DLLs: cold and scoped defaults, compatible buffers, hidden migration and invalid default rejection, new field defaults and explicit upgrade preserve identity; unread malformed writes and reentrant record edits reject and roll back; declared previous records upgrade automatically"
    );
    Ok(())
}
