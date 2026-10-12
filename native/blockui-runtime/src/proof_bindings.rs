use blockui_abi::{CHECKBOX, key};
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn action(actor: &mut Actor, name: &str, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(name),
        value,
    })
}
fn views(actor: &mut Actor) -> Result<i64, String> {
    actor
        .dispatch_request(Message {
            action: key("views"),
            value: 0,
        })?
        .ok_or("missing view count".into())
}
pub fn run(paths: &[String]) -> Result<(), String> {
    if paths.len() != 4 {
        return Err("binding-proof base replacement invalid-view broken-binding".into());
    }
    let mut actor = Actor::new(933, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let storage = actor.state_address();
    let slot = actor.slot_address("count");
    assert_eq!(
        views(&mut actor)?,
        1,
        "initial view must build exactly once for all bindings"
    );
    action(&mut actor, "step", 0)?;
    assert_eq!(actor.view.bindings[&key("caption")].text, "Count: 1 old");
    actor.set_text(key("notes"), "a😀bc draft")?;
    let checkbox = *actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.node.kind == CHECKBOX)
        .unwrap()
        .0;
    actor.dispatch_widget(checkbox)?;
    assert!(actor.boolean("checked"));
    assert!(actor.view.bindings[&checkbox].text.starts_with("☑"));
    assert_eq!(
        views(&mut actor)?,
        1,
        "binding updates must not reconstruct the view"
    );
    action(&mut actor, "shape", 1)?;
    assert_eq!(views(&mut actor)?, 2);
    assert_eq!(
        actor.view.bindings[&key("caption")].text,
        "Other: 100",
        "changed callback must invalidate a same-named binding"
    );
    let evaluations = actor.view.bindings[&key("caption")].evaluations;
    action(&mut actor, "step", 0)?;
    assert_eq!(
        actor.view.bindings[&key("caption")].evaluations,
        evaluations
    );
    action(&mut actor, "other", 0)?;
    assert_eq!(actor.view.bindings[&key("caption")].text, "Other: 101");
    assert_eq!(views(&mut actor)?, 2);
    assert!(action(&mut actor, "shape", 2).is_err());
    assert_eq!(actor.integer("shape"), 1);
    action(&mut actor, "other", 0)?;
    assert_eq!(actor.view.bindings[&key("caption")].text, "Other: 102");
    assert_eq!(
        views(&mut actor)?,
        3,
        "failed structural projection must not publish its code registry"
    );
    action(&mut actor, "shape", 0)?;
    assert_eq!(views(&mut actor)?, 4);
    let version = actor.implementation_version();
    for path in &paths[2..] {
        assert!(actor.reload(Path::new(path)).is_err());
        assert_eq!(actor.implementation_version(), version);
        action(&mut actor, "step", 0)?;
        assert_eq!(views(&mut actor)?, 4);
        assert!(actor.view.bindings[&key("caption")].text.ends_with(" old"));
        assert_eq!(actor.artifact_count(), 1);
    }
    let count = actor.integer("count");
    actor.enqueue(Message {
        action: key("step"),
        value: 0,
    })?;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(
        views(&mut actor)?,
        1,
        "new implementation must rebuild its own registry"
    );
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("count"), slot);
    assert_eq!(actor.text("notes"), "a😀bc draft");
    assert!(actor.boolean("checked"));
    actor.tick()?;
    assert_eq!(actor.integer("count"), count + 10);
    assert_eq!(
        actor.view.bindings[&key("caption")].text,
        format!("Count: {} new", count + 10)
    );
    assert_eq!(views(&mut actor)?, 1);
    assert_eq!(actor.artifact_count(), 1);
    assert_eq!(actor.mapped_implementation_count()?, 1);
    // Exercise rollback and index rebuilding repeatedly through independently loaded DLLs.
    let notes = actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.kind == blockui_abi::ENTRY)
        .map(|(key, _)| *key)
        .unwrap();
    actor.view.renderer.get_mut(&notes).unwrap().selection = (1, 3);
    actor.view.renderer.get_mut(&notes).unwrap().focus = true;
    let snapshots = Path::new(".runtime").join(format!("actor-933-{}", std::process::id()));
    let mut max_snapshot_allocated = 0;
    for round in 0..24 {
        let replacement = round % 2;
        actor.reload(Path::new(&paths[replacement]))?;
        action(&mut actor, "shape", 1)?;
        let other = actor.integer("other");
        let caption = actor.view.bindings[&key("caption")].clone();
        assert!(action(&mut actor, "other", i64::MAX).is_err());
        assert_eq!(actor.integer("other"), other);
        assert_eq!(actor.view.bindings[&key("caption")].text, caption.text);
        assert_eq!(
            actor.view.bindings[&key("caption")].dependencies,
            caption.dependencies
        );
        let evaluations = actor.view.bindings[&key("caption")].evaluations;
        action(&mut actor, "step", 0)?;
        assert_eq!(
            actor.view.bindings[&key("caption")].evaluations,
            evaluations,
            "failed binding must not publish its temporary count dependency"
        );
        action(&mut actor, "other", 0)?;
        assert_eq!(
            actor.view.bindings[&key("caption")].text,
            format!("Other: {}", other + 1)
        );
        let version = actor.implementation_version();
        assert!(actor.reload(Path::new(&paths[2])).is_err());
        assert_eq!(actor.implementation_version(), version);
        action(&mut actor, "shape", 0)?;
        assert!(actor.reload(Path::new(&paths[3])).is_err());
        assert_eq!(actor.implementation_version(), version);
        let count = actor.integer("count");
        action(&mut actor, "step", 0)?;
        let increment = if replacement == 0 { 1 } else { 10 };
        let suffix = if replacement == 0 { "old" } else { "new" };
        assert_eq!(actor.integer("count"), count + increment);
        assert_eq!(
            actor.view.bindings[&key("caption")].text,
            format!("Count: {} {suffix}", count + increment)
        );
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.slot_address("count"), slot);
        assert_eq!(actor.text("notes"), "a😀bc draft");
        assert!(actor.boolean("checked"));
        assert_eq!(actor.view.renderer[&notes].selection, (1, 3));
        assert!(actor.view.renderer[&notes].focus);
        assert_eq!(actor.artifact_count(), 1);
        assert_eq!(actor.mapped_implementation_count()?, 1);
        let mut allocated = 0;
        for entry in std::fs::read_dir(&snapshots).map_err(|error| error.to_string())? {
            let metadata = entry
                .map_err(|error| error.to_string())?
                .metadata()
                .map_err(|error| error.to_string())?;
            assert!(metadata.len() <= 32 * 1024 * 1024);
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                allocated += metadata.blocks() * 512;
            }
            #[cfg(not(unix))]
            {
                allocated += metadata.len();
            }
        }
        max_snapshot_allocated = max_snapshot_allocated.max(allocated);
    }
    println!(
        "steady_snapshot_max_allocated_bytes={max_snapshot_allocated}; successful_replacements=24; rejected_candidates=48; failed_binding_turns=24"
    );
    println!(
        "PASS implementation-owned binding registry builds view once, directly updates reactive/text/boolean bindings, replaces same-name callbacks, prunes dynamic dependencies, rejects structural/candidate failures without publishing callbacks and rebuilds registry through real DLL replacement with stable state/slots/mailbox; 24 alternating replacements retain host renderer state and prune failed binding edges"
    );
    Ok(())
}

pub fn benchmark(path: &Path, rounds: usize) -> Result<(), String> {
    if !(1..=100_000).contains(&rounds) {
        return Err("binding-bench rounds must be 1..=100000".into());
    }
    let mut actor = Actor::new(934, Path::new(".runtime"))?;
    let start = std::time::Instant::now();
    actor.reload(path)?;
    let load_us = start.elapsed().as_micros();
    let initial_views = views(&mut actor)?;
    let storage = actor.state_address();
    let slot = actor.slot_address("count");
    for _ in 0..100 {
        action(&mut actor, "step", 0)?;
    }
    let start = std::time::Instant::now();
    for _ in 0..rounds {
        action(&mut actor, "step", 0)?;
    }
    let update_ns = start.elapsed().as_nanos();
    assert_eq!(actor.integer("count"), rounds as i64 + 100);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("count"), slot);
    assert_eq!(actor.artifact_count(), 1);
    println!(
        "{{\"load_us\":{load_us},\"initial_views\":{initial_views},\"rounds\":{rounds},\"update_ns\":{update_ns},\"views_after\":{},\"nodes\":{},\"stable_state\":true,\"artifacts\":1}}",
        views(&mut actor)?,
        actor.view.bindings.len()
    );
    Ok(())
}
