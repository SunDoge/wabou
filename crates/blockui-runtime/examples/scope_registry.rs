use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 3 {
        return Err("scope_registry base duplicate replacement".into());
    }
    let mut actor = Actor::new(99, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.scope_ids(), vec![key("document.panel")]);
    assert!(
        actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "Panel 2")
    );
    assert!(actor.view.bindings.contains_key(&key("document.panel")));
    assert_eq!(
        actor.view.scope_dependencies[&key("document.panel")].dependencies,
        std::collections::BTreeSet::from([blockui_runtime::Dependency::State(key(
            "editor.document.count"
        ))])
    );
    assert!(
        !actor
            .view
            .root_dependencies
            .contains(&blockui_runtime::Dependency::State(key(
                "editor.document.count"
            )))
    );
    let storage = actor.state_address();
    let slot = actor.slot_address("editor.document.count");
    let implementation = actor.implementation_version();
    assert!(actor.reload(Path::new(&paths[1])).is_err());
    assert_eq!(actor.implementation_version(), implementation);
    assert_eq!(actor.scope_ids(), vec![key("document.panel")]);
    assert_eq!(actor.integer("editor.document.count"), 2);
    let root_build = actor.view.bindings[&key("root.build")].text.clone();
    let anonymous_build = actor.view.bindings[&key("root.anonymous")].text.clone();
    actor.enqueue(Message {
        action: key("update"),
        value: 0,
    })?;
    actor.tick()?;
    assert_eq!(actor.integer("editor.document.count"), 3);
    assert!(
        actor
            .view
            .bindings
            .values()
            .any(|binding| binding.text == "Panel 3")
    );
    assert_eq!(actor.view.bindings[&key("root.build")].text, root_build);
    assert_eq!(
        actor.view.bindings[&key("root.anonymous")].text,
        anonymous_build
    );
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("editor.document.count"), slot);
    actor.enqueue(Message {
        action: key("update"),
        value: 0,
    })?;
    actor.reload(Path::new(&paths[2]))?;
    assert!(actor.scope_ids().is_empty());
    assert!(actor.view.scope_dependencies.is_empty());
    assert!(!actor.view.bindings.contains_key(&key("document.panel")));
    actor.tick()?;
    assert_eq!(actor.integer("editor.document.count"), 13);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("editor.document.count"), slot);
    assert_eq!(actor.artifact_count(), 1);
    #[cfg(target_os = "linux")]
    assert_eq!(actor.mapped_implementation_count()?, 1);
    println!(
        "PASS real scope updates retain root projection; candidates reject duplicate identity, remove old registry, retain state and dispatch queued action through current code"
    );
    Ok(())
}
