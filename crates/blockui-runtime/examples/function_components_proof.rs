//! Owned fixture built by scripts/verify_function_components.py.
use blockui::abi::{BUTTON, LABEL};
use blockui_runtime::{Actor, Dependency};
use std::path::Path;

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("expected base, replacement, rejected and snapshots".into());
    }
    let mut actor = Actor::new(812, Path::new(&paths[3]))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.view.root_evaluations(), 1);
    let buttons: Vec<_> = actor
        .view
        .bindings
        .iter()
        .filter_map(|(key, binding)| (binding.node.kind == BUTTON).then_some(*key))
        .collect();
    assert_eq!(buttons.len(), 2);
    let labels: Vec<_> = buttons
        .iter()
        .map(|button| {
            let parent = actor.view.bindings[button].node.parent;
            *actor
                .view
                .bindings
                .iter()
                .find(|(_, binding)| binding.node.kind == LABEL && binding.node.parent == parent)
                .unwrap()
                .0
        })
        .collect();
    assert_ne!(labels[0], labels[1]);
    let slots: Vec<_> = labels
        .iter()
        .map(|label| {
            let dependencies = &actor.view.bindings[label].text_dependencies;
            assert_eq!(dependencies.len(), 1);
            let key = match dependencies.first().unwrap() {
                Dependency::State(key) => *key,
                Dependency::Computed(key) => {
                    let memo_dependencies = actor.computed_dependencies_by_key(*key).unwrap();
                    assert_eq!(memo_dependencies.len(), 1);
                    let Dependency::State(state) = memo_dependencies.first().unwrap() else {
                        panic!("expected memo to read state")
                    };
                    *state
                }
                Dependency::MapEntry(_) => panic!("expected scalar counter dependency"),
            };
            (key, actor.slot_address_by_key(key))
        })
        .collect();
    assert_ne!(slots[0].0, slots[1].0);
    assert!(slots.iter().all(|(_, address)| *address != 0));
    assert_ne!(slots[0].1, slots[1].1);
    let state = actor.state_address();
    let untouched = actor.view.bindings[&labels[1]].evaluations;
    actor.dispatch_widget(buttons[0])?;
    assert_eq!(actor.view.bindings[&labels[0]].text, "Count: 1");
    assert_eq!(actor.view.bindings[&labels[1]].text, "Count: 0");
    assert_eq!(actor.view.bindings[&labels[1]].evaluations, untouched);
    assert_eq!(actor.view.root_evaluations(), 1);
    let mut counts = [1, 0];
    for index in 0..20 {
        let replacement = index % 2 == 0;
        actor.reload(Path::new(&paths[usize::from(replacement)]))?;
        assert_eq!(actor.view.root_evaluations(), index as u64 + 2);
        assert_eq!(actor.state_address(), state);
        for (key, address) in &slots {
            assert_eq!(actor.slot_address_by_key(*key), *address);
        }
        let target = index % 2;
        let other = 1 - target;
        let before = actor.view.bindings[&labels[other]].evaluations;
        actor.dispatch_widget(buttons[target])?;
        assert_eq!(actor.view.root_evaluations(), index as u64 + 2);
        counts[target] += if replacement { 10 } else { 1 };
        for (label, count) in labels.iter().zip(counts) {
            assert_eq!(actor.view.bindings[label].text, format!("Count: {count}"));
        }
        assert_eq!(actor.view.bindings[&labels[other]].evaluations, before);
    }
    let version = actor.implementation_version();
    let error = actor.reload(Path::new(&paths[2])).unwrap_err();
    assert!(error.contains("reject function app"), "{error}");
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.view.root_evaluations(), 21);
    actor.dispatch_widget(buttons[0])?;
    assert_eq!(actor.view.root_evaluations(), 21);
    counts[0] += 1; // Failed candidate must not install its +10 callback.
    assert_eq!(
        actor.view.bindings[&labels[0]].text,
        format!("Count: {}", counts[0])
    );
    for (key, address) in slots {
        assert_eq!(actor.slot_address_by_key(key), address);
    }
    drop(actor);
    println!(
        "PASS components: two isolated instances, stable slots/widgets, fine-grained observers, 20 real replacements and rejected-view callback rollback"
    );
    Ok(())
}
