use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("record_scalar_reload <base> <replacement> <missing> <evolved>".into());
    }
    let mut actor = Actor::new(209, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let send = |actor: &mut Actor, value| {
        actor.dispatch(Message {
            action: key("scalar.fields.change"),
            value,
        })
    };
    send(&mut actor, 0)?;
    let state = actor.state_address();
    let slot = actor.slot_address("scalar.fields.values");
    let buffer = actor.text("scalar.fields.values").as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("scalar.fields.values"), slot);
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    for (name, text) in [
        ("signed", i64::MIN.to_string()),
        ("unsigned", u64::MAX.to_string()),
        ("ratio", "-0".into()),
        ("enabled", "true".into()),
    ] {
        assert_eq!(actor.view.bindings[&key(name)].text, text);
    }
    let evaluations = actor.view.bindings[&key("ratio")].evaluations;
    send(&mut actor, 1)?;
    assert_eq!(actor.view.bindings[&key("signed")].text, "8");
    assert_eq!(actor.view.bindings[&key("ratio")].evaluations, evaluations);
    let buffer = actor.text("scalar.fields.values").as_ptr();
    assert!(send(&mut actor, 2).unwrap_err().contains("reject"));
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&key("signed")].text, "8");
    send(&mut actor, 3)?;
    assert_eq!(actor.view.bindings[&key("ratio")].text, "inf");
    println!(
        "PASS scalar record bindings through actual DLL replacement: state and allocation retained, new action executes, sibling skips reevaluation, rejected action rolls back"
    );
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    let buffer = actor.text("scalar.fields.values").as_ptr();
    let bindings: Vec<_> = ["signed", "unsigned", "ratio", "enabled", "toggle"]
        .map(|name| actor.view.bindings[&key(name)].node.binding)
        .into();
    assert!(
        actor
            .reload(Path::new(&paths[2]))
            .unwrap_err()
            .contains("explicit migration is required")
    );
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    assert_eq!(actor.checkbox_value(key("toggle")), Some(true));
    actor.reload(Path::new(&paths[3]))?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("scalar.fields.values"), slot);
    for (name, text) in [
        ("title", "Settings".to_owned()),
        ("signed", "8".to_owned()),
        ("unsigned", u64::MAX.to_string()),
        ("ratio", "inf".to_owned()),
        ("enabled", "true".to_owned()),
    ] {
        assert_eq!(actor.view.bindings[&key(name)].text, text);
    }
    for (name, binding) in ["signed", "unsigned", "ratio", "enabled", "toggle"]
        .into_iter()
        .zip(bindings)
    {
        assert_eq!(actor.view.bindings[&key(name)].node.binding, binding);
    }
    let evaluations = actor.view.bindings[&key("ratio")].evaluations;
    actor.dispatch_widget(key("toggle"))?;
    assert_eq!(actor.checkbox_value(key("toggle")), Some(false));
    assert_eq!(actor.view.bindings[&key("signed")].text, "10");
    assert_eq!(actor.view.bindings[&key("ratio")].evaluations, evaluations);
    let buffer = actor.text("scalar.fields.values").as_ptr();
    assert!(send(&mut actor, 2).is_err());
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&key("signed")].text, "10");
    for (id, path) in [(211, &paths[3]), (212, &paths[2])] {
        let mut cold = Actor::new(id, Path::new(".runtime"))?;
        cold.reload(Path::new(path))?;
        assert_eq!(cold.view.bindings[&key("title")].text, "Settings");
        assert_eq!(cold.checkbox_value(key("toggle")), Some(false));
    }
    println!(
        "PASS scalar record schema migration: missing migration rejected atomically, added default field shifts all selectors while preserving values and binding identity, toggles and rollback still work; cold defaults initialize"
    );
    Ok(())
}
