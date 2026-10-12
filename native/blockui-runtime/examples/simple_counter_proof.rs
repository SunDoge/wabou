use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().collect();
    if paths.len() != 7 {
        return Err(
            "simple_counter_proof base replacement migration rejected infallible bound-view".into(),
        );
    }
    let mut first = Actor::new(90, Path::new(".runtime"))?;
    let mut second = Actor::new(91, Path::new(".runtime"))?;
    first.reload(Path::new(&paths[1]))?;
    second.reload(Path::new(&paths[1]))?;
    let action = Message {
        action: key("Counter.increment"),
        value: 0,
    };
    first.dispatch(action)?;
    assert_eq!(first.integer("counter.count"), 1);
    assert_eq!(second.integer("counter.count"), 0);
    assert_eq!(first.view.bindings[&key("counter.count")].text, "Double: 2");
    let state = first.state_address();
    let slot = first.slot_address("counter.count");
    let schema = first.schema_version;
    first.enqueue(action)?;
    first.reload(Path::new(&paths[2]))?;
    assert_eq!(first.state_address(), state);
    assert_eq!(first.slot_address("counter.count"), slot);
    assert_eq!(first.schema_version, schema);
    assert_eq!(first.integer("counter.count"), 1);
    first.tick()?;
    assert_eq!(first.integer("counter.count"), 11);
    assert_eq!(
        first.view.bindings[&key("counter.count")].text,
        "Double: 22"
    );
    let version = first.implementation_version();
    assert!(
        first
            .dispatch(Message {
                value: -1,
                ..action
            })
            .is_err()
    );
    assert_eq!(first.integer("counter.count"), 11);
    assert_eq!(first.slot_address("counter.count"), slot);
    assert_eq!(
        first.view.bindings[&key("counter.count")].text,
        "Double: 22"
    );
    assert!(first.reload(Path::new(&paths[4])).is_err());
    assert_eq!(first.implementation_version(), version);
    assert_eq!(first.slot_address("counter.count"), slot);
    assert_eq!(first.integer("counter.count"), 11);
    first.reload(Path::new(&paths[3]))?;
    assert_ne!(first.schema_version, schema);
    assert_eq!(first.integer("counter.count"), 111);
    assert_eq!(first.integer("counter.bonus"), 7);
    assert_eq!(
        first.view.bindings[&key("counter.count")].text,
        "Double: 222"
    );
    first.reload(Path::new(&paths[5]))?;
    assert_eq!(first.integer("counter.count"), 111);
    assert_eq!(
        first.view.bindings[&key("counter.count")].text,
        "Static counter"
    );
    println!(
        "PASS explicit fallible views reject candidates and direct View methods preserve state through replacement"
    );
    first.reload(Path::new(&paths[6]))?;
    assert_eq!(first.integer("counter.count"), 111);
    assert_eq!(first.slot_address("counter.count"), slot);
    assert_eq!(
        first.view.bindings[&key("counter.count")].text,
        "Bound count: 111"
    );
    first.dispatch(action)?;
    assert_eq!(first.integer("counter.count"), 121);
    assert_eq!(
        first.view.bindings[&key("counter.count")].text,
        "Bound count: 121"
    );
    println!("PASS plain bound receiver renders live state without Result or context arguments");
    second.dispatch(action)?;
    assert_eq!(second.integer("counter.count"), 1);
    println!(
        "PASS receiver methods and generated actions preserve independent actor state/slot/schema and queued execution through real replacement"
    );
    Ok(())
}
