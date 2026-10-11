use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;

fn change(actor: &mut Actor, count: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key("change"),
        value: count,
    })
}
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 2 {
        return Err("root_scope_cache base replacement".into());
    }
    let mut actor = Actor::new(149, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(
        actor.view.bindings[&key("reuse.build")].text,
        "Scope build 1"
    );
    let title = key("reuse.title");
    let evaluations = actor.view.bindings[&title].evaluations;
    for count in [1, 3] {
        change(&mut actor, count)?;
        assert_eq!(
            actor.view.bindings[&key("reuse.build")].text,
            "Scope build 1"
        );
        assert_eq!(actor.view.bindings[&title].text, format!("Title {count}"));
    }
    assert_eq!(actor.view.bindings[&title].evaluations, evaluations + 2);
    assert_eq!(
        actor.view.bindings[&key("reuse.panel")].node.parent,
        key("reuse.wrapper")
    );
    change(&mut actor, -1)?;
    assert!(
        !actor
            .view
            .scope_dependencies
            .contains_key(&key("reuse.panel"))
    );
    assert!(!actor.view.bindings.contains_key(&title));
    change(&mut actor, 2)?;
    assert_eq!(
        actor.view.bindings[&key("reuse.build")].text,
        "Scope build 2"
    );
    assert_eq!(
        actor.view.bindings[&key("reuse.version")].text,
        "Version one"
    );
    let identity = actor.state_address();
    let slot = actor.slot_address("title");
    let buffer = actor.text("title").as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), identity);
    assert_eq!(actor.slot_address("title"), slot);
    assert_eq!(actor.text("title").as_ptr(), buffer);
    assert_eq!(
        actor.view.bindings[&key("reuse.version")].text,
        "Version two"
    );
    assert_eq!(
        actor.view.bindings[&key("reuse.build")].text,
        "Scope build 1"
    );
    assert_eq!(actor.view.bindings[&title].text, "Title 2");
    change(&mut actor, 3)?;
    assert_eq!(
        actor.view.bindings[&key("reuse.build")].text,
        "Scope build 1"
    );
    assert_eq!(actor.view.bindings[&title].text, "Title 3");
    println!(
        "PASS real DLL root updates retain unchanged scoped builders and refresh live bindings/reparent nodes; remount rebuilds and implementation replacement always projects new scope code with stable state allocations"
    );
    Ok(())
}
