use blockui_abi::key;
use blockui_runtime::{Actor, Message, Value};
use std::path::Path;

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 3 {
        return Err("conditional_schema base enabled replacement".into());
    }
    let mut actor = Actor::new(97, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert!(actor.computed("conditional.double").is_none());
    actor.dispatch(Message {
        action: key("change"),
        value: 1,
    })?;
    let storage = actor.state_address();
    let checked = actor.slot_address("checked");
    let calls = actor.slot_address("calls");
    let base_schema = actor.schema_version;
    let widget = *actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.node.slot == key("checked"))
        .unwrap()
        .0;
    let check = |actor: &Actor| {
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.slot_address("checked"), checked);
        assert_eq!(actor.slot_address("calls"), calls);
        assert_eq!(actor.integer("calls"), 1);
        assert!(actor.boolean("checked"));
        assert!(actor.view.bindings.contains_key(&widget));
        assert_eq!(actor.artifact_count(), 1);
        #[cfg(target_os = "linux")]
        assert_eq!(actor.mapped_implementation_count().unwrap(), 1);
    };
    actor.reload(Path::new(&paths[1]))?;
    check(&actor);
    assert_ne!(actor.schema_version, base_schema);
    let enabled_schema = actor.schema_version;
    assert_eq!(actor.integer("conditional.extra"), 7);
    assert_eq!(
        actor.computed("conditional.double"),
        Some(&Value::Integer(14))
    );
    actor.dispatch(Message {
        action: key("conditional.bump"),
        value: 5,
    })?;
    assert_eq!(actor.integer("conditional.extra"), 12);
    assert_eq!(
        actor.computed("conditional.double"),
        Some(&Value::Integer(24))
    );
    let extra = actor.slot_address("conditional.extra");
    actor.enqueue(Message {
        action: key("conditional.bump"),
        value: 5,
    })?;
    actor.reload(Path::new(&paths[2]))?;
    check(&actor);
    assert_eq!(actor.schema_version, enabled_schema);
    assert_eq!(actor.slot_address("conditional.extra"), extra);
    assert_eq!(actor.integer("conditional.extra"), 12);
    assert_eq!(
        actor.computed("conditional.double"),
        Some(&Value::Integer(24))
    );
    actor.tick()?;
    assert_eq!(actor.integer("conditional.extra"), 22);
    assert_eq!(
        actor.computed("conditional.double"),
        Some(&Value::Integer(44))
    );
    actor.reload(Path::new(&paths[0]))?;
    check(&actor);
    assert_eq!(actor.schema_version, base_schema);
    assert!(actor.computed("conditional.double").is_none());
    assert!(
        actor
            .dispatch(Message {
                action: key("conditional.bump"),
                value: 1
            })
            .is_err()
    );
    actor.reload(Path::new(&paths[1]))?;
    check(&actor);
    assert_eq!(actor.integer("conditional.extra"), 7);
    assert_eq!(
        actor.computed("conditional.double"),
        Some(&Value::Integer(14))
    );
    assert_eq!(actor.schema_version, enabled_schema);
    check(&actor);
    println!(
        "PASS cfg_attr feature schemas: default addition/removal, retained stable slots/widget, compatible optional-slot address, queued new-code action, conditional computed rebuilding, absent action rejection and re-added default"
    );
    Ok(())
}
