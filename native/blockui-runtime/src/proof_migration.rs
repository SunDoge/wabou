use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn increment(actor: &mut Actor, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key("increment"),
        value,
    })
}
pub fn run(paths: &[String]) -> Result<(), String> {
    if paths.len() != 8 {
        return Err("migration-proof needs eight schema variants".into());
    }
    let mut actor = Actor::new(67, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    increment(&mut actor, 17)?;
    actor.set_text(key("notes"), "a😀bc document")?;
    let count = actor.slot_address("count");
    let notes = actor.slot_address("notes");
    let touch = actor.slot_address("touch");
    let storage = actor.state_address();
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    // Checking actual candidate execution must not commit staged migration or UI.
    for index in [0, 1, 2, 3, 4, 5, 6, 7] {
        let result = actor.check_reload(Path::new(&paths[index]));
        assert_eq!(result.is_ok(), matches!(index, 0 | 1 | 7), "{result:?}");
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.integer("count"), 17);
        assert_eq!(actor.text("notes"), "a😀bc document");
        assert_eq!(actor.integer("touch"), 0);
        assert_eq!(actor.slot_address("count"), count);
        assert_eq!(actor.slot_address("notes"), notes);
        assert_eq!(actor.slot_address("touch"), touch);
        assert_eq!(actor.slot_address("document.notes"), 0);
        assert_eq!(actor.view.bindings[&key("document.count")].text, "17");
        assert_eq!(actor.artifact_count(), 1);
        assert_eq!(actor.mapped_implementation_count()?, 1);
    }
    // The retained old implementation remains callable after successful checks.
    increment(&mut actor, 1)?;
    assert_eq!(actor.integer("count"), 18);
    increment(&mut actor, -1)?;
    assert_eq!(actor.integer("count"), 17);
    println!(
        "PASS executable candidate checks discard valid/failed migration/view without replacing state/slots/schema/UI/code or retaining candidate artifacts"
    );
    for index in [2, 3, 4, 5, 6] {
        let error = actor.reload(Path::new(&paths[index])).unwrap_err();
        if index == 2 {
            assert!(error.contains("explicit migration writes"), "{error}");
        }
        if index == 3 {
            assert!(error.contains("intentional migration rejection"), "{error}");
        }
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.integer("count"), 17);
        assert_eq!(actor.integer("touch"), 0);
        assert_eq!(actor.text("notes"), "a😀bc document");
        assert_eq!(actor.slot_address("count"), count);
        assert_eq!(actor.slot_address("notes"), notes);
        assert_eq!(actor.slot_address("touch"), touch);
        assert_eq!(actor.slot_address("document.notes"), 0);
        assert_eq!(actor.view.bindings[&key("document.count")].text, "17");
        assert_eq!(actor.artifact_count(), 1);
    }
    assert!(
        actor
            .dispatch(Message {
                action: key("read_old_outside_migration"),
                value: 0
            })
            .is_err()
    );
    actor.reload(Path::new(&paths[1]))?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.text("count"), "17");
    assert_eq!(actor.text("document.notes"), "a😀bc document");
    assert_eq!(actor.slot_address("notes"), 0);
    assert_eq!(actor.slot_address("obsolete"), 0);
    assert_eq!(actor.integer("selection"), 3);
    assert_eq!(actor.slot_address("touch"), touch); // unchanged typed slot keeps its allocation
    increment(&mut actor, 2)?;
    assert_eq!(actor.text("count"), "19");
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.integer("count"), 19);
    assert_eq!(actor.text("notes"), "a😀bc document");
    assert_eq!(actor.slot_address("document.notes"), 0);
    assert_eq!(actor.slot_address("selection"), 0);
    assert_eq!(actor.slot_address("touch"), touch);
    actor.reload(Path::new(&paths[7]))?; // explicitly writing the default still acknowledges conversion
    assert_eq!(actor.text("count"), "0");
    assert_eq!(actor.slot_address("touch"), touch);
    for _ in 0..20 {
        actor.reload(Path::new(&paths[0]))?;
        actor.reload(Path::new(&paths[1]))?;
        assert_eq!(actor.slot_address("touch"), touch);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.artifact_count(), 1);
    }
    assert_eq!(actor.mapped_implementation_count()?, 1);
    println!(
        "PASS explicit integer/text migration, removed/renamed old slot reads, default additions/deletions, unchanged slot allocation, conversion acknowledgement, migration panic/error/view rollback, old-schema scope rejection and repeated schema swaps"
    );
    Ok(())
}
