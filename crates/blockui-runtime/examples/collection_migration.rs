//! Real libraries verify conversion, rollback and durable mailbox semantics.
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 6 {
        return Err("collection_migration scalar list no-migration rejected extra".into());
    }
    let mut actor = Actor::new(48, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[1]))?;
    let scalar_slot = actor.slot_address("items");
    let count_slot = actor.slot_address("count");
    let version = actor.implementation_version();
    actor.dispatch(Message {
        action: blockui_abi::key("push"),
        value: 8,
    })?;
    actor.enqueue(Message {
        action: blockui_abi::key("push"),
        value: 9,
    })?;
    assert_eq!(actor.integer("items"), 15);
    let preview = actor.preview_reload(Path::new(&args[2]))?;
    assert!(preview.requires_migration());
    for path in &args[3..5] {
        assert!(actor.reload(Path::new(path)).is_err());
        assert_eq!(actor.integer("items"), 15);
        assert_eq!(actor.integer("count"), 1);
        assert_eq!(actor.slot_address("items"), scalar_slot);
        assert_eq!(actor.slot_address("count"), count_slot);
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.mailbox.len(), 1);
    }
    actor.reload(Path::new(&args[2]))?;
    assert_eq!(actor.list("items"), [15]);
    assert_eq!(actor.slot_address("count"), count_slot);
    actor.tick()?;
    assert_eq!(actor.list("items"), [15, 9]);
    actor.reload(Path::new(&args[1]))?;
    assert_eq!(actor.integer("items"), 24);
    assert_eq!(actor.integer("count"), 1);
    assert_eq!(actor.slot_address("count"), count_slot);
    actor.reload(Path::new(&args[2]))?;
    assert_eq!(actor.list("items"), [24]);
    actor.dispatch(Message {
        action: blockui_abi::key("push"),
        value: i64::MAX,
    })?;
    let list_slot = actor.slot_address("items");
    let buffer = actor.list("items").as_ptr();
    let version = actor.implementation_version();
    assert!(actor.reload(Path::new(&args[1])).is_err());
    assert_eq!(actor.list("items"), [24, i64::MAX]);
    assert_eq!(actor.list("items").as_ptr(), buffer);
    assert_eq!(actor.slot_address("items"), list_slot);
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.slot_address("count"), count_slot);
    actor.reload(Path::new(&args[5]))?;
    assert_eq!(actor.integer("bonus"), 11);
    assert_eq!(actor.list("items"), [24, i64::MAX]);
    assert_eq!(actor.list("items").as_ptr(), buffer);
    assert_eq!(actor.slot_address("items"), list_slot);
    assert_eq!(actor.slot_address("count"), count_slot);
    println!(
        "PASS scalar/list conversions use explicit previous reads, reject implicit and failing migrations, preserve queued messages and unrelated slots, and roll back overflowing list reduction; unrelated schema additions retain the original list buffer"
    );
    Ok(())
}
