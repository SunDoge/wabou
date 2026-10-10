use blockui::Record;
use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;

#[derive(Record)]
#[record(name = "seven.crud.person")]
struct Person {
    first: String,
    last: String,
}
fn person(actor: &Actor, id: &str) -> Person {
    let pair = actor
        .text_list("crud.records")
        .as_chunks::<2>()
        .0
        .iter()
        .find(|pair| pair[0] == id)
        .unwrap();
    let (format, payload) = pair[1].split_once('\n').unwrap();
    assert_eq!(format, Person::FORMAT);
    Person::decode(payload).unwrap()
}
fn send(actor: &mut Actor, action: &str, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(action),
        value,
    })
}
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("crud_migration legacy current compatible manual".into());
    }
    for previous in [&paths[0], &paths[3]] {
        let mut actor = Actor::new(142, Path::new(".runtime"))?;
        actor.reload(Path::new(previous))?;
        let identity = actor.state_address();
        let ids = actor.slot_address("crud.ids");
        actor.reload(Path::new(&paths[1]))?;
        assert_eq!(actor.state_address(), identity);
        assert_eq!(actor.slot_address("crud.ids"), ids);
        assert_eq!(actor.list("crud.ids"), &[7, 42]);
        assert_eq!(person(&actor, "42").first, "新");
        assert_eq!(person(&actor, "7").last, "Lovelace");
        assert_eq!(actor.text("crud.first"), "unsaved 🦀");
        assert_eq!(actor.text("crud.last"), "draft");
        assert_eq!(actor.text("crud.filter"), "张");
        assert_eq!(actor.integer("crud.selected"), 42);
        assert_eq!(actor.integer("crud.next"), 43);
        // Current implementation operates on the migrated records and old draft.
        send(&mut actor, "crud.update", 0)?;
        assert_eq!(person(&actor, "42").first, "unsaved 🦀");
        assert_eq!(person(&actor, "42").last, "draft");
        let records = actor.slot_address("crud.records");
        let buffer = actor.text_list("crud.records").as_ptr();
        let schema = actor.schema_version;
        let version = actor.implementation_version();
        actor.reload(Path::new(&paths[2]))?;
        assert_ne!(actor.implementation_version(), version);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.slot_address("crud.records"), records);
        assert_eq!(actor.text_list("crud.records").as_ptr(), buffer);
        assert_eq!(person(&actor, "42").first, "unsaved 🦀");
        send(&mut actor, "crud.delete", 0)?;
        assert_eq!(actor.list("crud.ids"), &[7]);
        if previous == &paths[0] {
            println!(
                "PASS real DLL legacy CRUD columns migrate to typed records retaining identity and drafts; current actions and subsequent compatible reload preserve encoded storage allocation"
            );
        } else {
            println!(
                "PASS real DLL existing CRUD records migrate handwritten marker to automatic marker retaining identity and drafts; current actions and compatible reload preserve encoded storage allocation"
            );
        }
    }
    for duplicate in [0, 1] {
        let mut old = Actor::new(143 + duplicate as u64, Path::new(".runtime"))?;
        old.reload(Path::new(&paths[0]))?;
        send(&mut old, "legacy.corrupt", duplicate)?;
        let identity = old.state_address();
        let version = old.implementation_version();
        let schema = old.schema_version;
        let ids = old.list("crud.ids").to_vec();
        let names = old.text_list("crud.firsts").to_vec();
        let buffer = old.text_list("crud.firsts").as_ptr();
        assert!(old.reload(Path::new(&paths[1])).is_err());
        assert_eq!(old.state_address(), identity);
        assert_eq!(old.implementation_version(), version);
        assert_eq!(old.schema_version, schema);
        assert_eq!(old.list("crud.ids"), ids);
        assert_eq!(old.text_list("crud.firsts"), names);
        assert_eq!(old.text_list("crud.firsts").as_ptr(), buffer);
        assert_eq!(old.text("crud.first"), "unsaved 🦀");
        // Execute the retained old function table after rejected replacement.
        send(&mut old, "legacy.corrupt", 0)?;
        assert_eq!(old.text_list("crud.lasts"), &["only one"]);
    }
    println!(
        "PASS real DLL CRUD corruption rejects migration and retains old implementation, schema, allocations and callable handlers"
    );
    for duplicate in [0, 1] {
        let mut old = Actor::new(146 + duplicate as u64, Path::new(".runtime"))?;
        old.reload(Path::new(&paths[3]))?;
        send(&mut old, "legacy.corrupt", duplicate)?;
        let stored = old.text_list("crud.records").to_vec();
        let buffer = old.text_list("crud.records").as_ptr();
        let version = old.implementation_version();
        assert!(old.reload(Path::new(&paths[1])).is_err());
        assert_eq!(old.text_list("crud.records"), stored);
        assert_eq!(old.text_list("crud.records").as_ptr(), buffer);
        assert_eq!(old.implementation_version(), version);
        assert_eq!(old.text("crud.first"), "unsaved 🦀");
        send(&mut old, "legacy.corrupt", 0)?;
    }
    println!(
        "PASS handwritten CRUD codec marker upgrades to automatic scoped marker across real DLLs; invalid record identities reject replacement with old storage and handler retained"
    );
    Ok(())
}
