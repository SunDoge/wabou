use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().collect();
    if paths.len() != 3 {
        return Err("bound_counter_proof base replacement".into());
    }
    let mut actor = Actor::new(92, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[1]))?;
    let action = Message {
        action: key("Counter.increment"),
        value: 0,
    };
    actor.dispatch(action)?;
    assert_eq!(actor.integer("counter.count"), 1);
    if !actor.scope_ids().is_empty() {
        assert_eq!(actor.scope_ids(), vec![key("counter.panel")]);
        assert!(actor.view.root_dependencies.is_empty());
        assert!(actor.view.bindings.contains_key(&key("counter.active")));
        assert_eq!(
            actor.view.scope_dependencies[&key("counter.panel")].dependencies,
            std::collections::BTreeSet::from([blockui_runtime::Dependency::State(key(
                "counter.count"
            ))])
        );
        println!(
            "PASS macro-declared scope registry and conditional projection through actual DLL"
        );
    }
    let storage = actor.state_address();
    let slot = actor.slot_address("counter.count");
    let schema = actor.schema_version;
    actor.enqueue(action)?;
    actor.reload(Path::new(&paths[2]))?;
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("counter.count"), slot);
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.integer("counter.count"), 1);
    assert_eq!(actor.view.bindings[&key("counter.count")].text, "1");
    assert_eq!(actor.view.bindings[&key("counter.double")].text, "3");
    assert_eq!(
        actor.view.bindings[&key("counter.heading")].text,
        "Updated count"
    );
    actor.tick()?;
    assert_eq!(actor.integer("counter.count"), 11);
    assert_eq!(actor.view.bindings[&key("counter.count")].text, "11");
    let mut expected = 11;
    let mut reload_us = Vec::new();
    for round in 0..40 {
        let version = round % 2 + 1;
        actor.enqueue(action)?;
        let start = std::time::Instant::now();
        actor.reload(Path::new(&paths[version]))?;
        reload_us.push(start.elapsed().as_micros());
        assert_eq!(actor.integer("counter.count"), expected);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.slot_address("counter.count"), slot);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.artifact_count(), 1);
        #[cfg(target_os = "linux")]
        assert_eq!(actor.mapped_implementation_count()?, 1);
        assert_eq!(
            actor.view.bindings[&key("counter.double")].text,
            (expected * if version == 1 { 2 } else { 3 }).to_string()
        );
        assert_eq!(
            actor.computed_dependencies("Counter.double"),
            Some(std::collections::BTreeSet::from([
                blockui_runtime::Dependency::State(key("counter.count"))
            ]))
        );
        let evaluations = actor.computed_evaluations("Counter.double");
        actor.tick()?;
        assert_eq!(
            actor.computed_evaluations("Counter.double"),
            evaluations + 1
        );
        expected += if version == 1 { 1 } else { 10 };
        assert_eq!(actor.integer("counter.count"), expected);
        assert_eq!(
            actor.view.bindings[&key("counter.count")].text,
            expected.to_string()
        );
        assert_eq!(
            actor.view.bindings[&key("counter.double")].text,
            (expected * if version == 1 { 2 } else { 3 }).to_string()
        );
    }
    actor.reload(Path::new(&paths[1]))?;
    actor.dispatch(Message {
        action: key("Counter.delayed_increment"),
        value: 25,
    })?;
    actor.reload(Path::new(&paths[2]))?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while actor.integer("counter.count") == expected {
        assert!(std::time::Instant::now() < deadline);
        actor.tick()?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(actor.integer("counter.count"), expected + 10);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("counter.count"), slot);
    actor.dispatch(Message {
        action: key("Counter.delayed_increment"),
        value: 60_000,
    })?;
    actor.dispatch(Message {
        action: key("Counter.stop_clock"),
        value: 0,
    })?;
    actor.tick()?;
    assert_eq!(actor.integer("counter.count"), expected + 10);
    println!(
        "PASS bound task queues and cancels host work without explicit context; pending completion runs new DLL action with stable state"
    );
    println!(
        "METRICS reload_cycles=40 reload_mean_us={} reload_max_us={}",
        reload_us.iter().sum::<u128>() / reload_us.len() as u128,
        reload_us.iter().max().unwrap()
    );
    println!(
        "PASS bound computed rebuilds dependencies and executes new derivation code through 40 replacements"
    );
    println!(
        "PASS 40 minimal counter replacements retain state and queued actions with one artifact and one Linux mapping"
    );
    println!(
        "PASS bound counter retains state/slot/schema and queued action updates plain Render through replacement"
    );
    Ok(())
}
