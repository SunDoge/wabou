//! Actual libraries preserve nested host allocations and explicit migrations.
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(paths.len(), 9);
    let mut map = Actor::new(133, Path::new(".runtime"))?;
    map.reload(Path::new(&paths[0]))?;
    for value in [2, 1] {
        map.dispatch(Message {
            action: blockui_abi::key("map.insert"),
            value,
        })?;
    }
    assert_eq!(
        map.text_list("labels.entries"),
        ["1", "old 😀", "2", "old 😀"]
    );
    let row = blockui_abi::key("labels.map.1");
    assert_eq!(map.view.bindings[&row].text, "old 😀");
    assert_eq!(
        map.view.bindings[&blockui_abi::key("labels.map.first")].text,
        "old 😀"
    );
    let row_parent = map.view.bindings[&row].node.parent;
    let buffer = map.text_list("labels.entries").as_ptr();
    let slot = map.slot_address("labels.entries");
    map.reload(Path::new(&paths[1]))?;
    assert_eq!(map.text_list("labels.entries").as_ptr(), buffer);
    assert_eq!(map.slot_address("labels.entries"), slot);
    assert_eq!(map.view.bindings[&row].node.parent, row_parent);
    assert_eq!(map.view.bindings[&row].text, "old 😀");
    assert_eq!(
        map.view.bindings[&blockui_abi::key("labels.map.first")].text,
        "old 😀"
    );
    assert!(
        map.dispatch(Message {
            action: blockui_abi::key("map.insert"),
            value: -1
        })
        .is_err()
    );
    assert_eq!(
        map.text_list("labels.entries"),
        ["1", "old 😀", "2", "old 😀"]
    );
    map.dispatch(Message {
        action: blockui_abi::key("map.insert"),
        value: 1,
    })?;
    assert_eq!(
        map.text_list("labels.entries"),
        ["1", "new 🦀", "2", "old 😀"]
    );
    assert_eq!(
        map.view.bindings[&blockui_abi::key("labels.map.first")].text,
        "new 🦀"
    );
    map.dispatch(Message {
        action: blockui_abi::key("map.remove"),
        value: 2,
    })?;
    assert_eq!(map.text_list("labels.entries"), ["1", "new 🦀"]);
    assert_eq!(map.view.bindings[&row].text, "new 🦀");
    assert_eq!(map.view.bindings[&row].node.parent, row_parent);
    assert!(
        !map.view
            .bindings
            .contains_key(&blockui_abi::key("labels.map.2"))
    );
    for value in [-1, 0] {
        assert!(
            map.dispatch(Message {
                action: blockui_abi::key("map.rename"),
                value
            })
            .is_err()
        );
        assert_eq!(map.text_list("labels.entries"), ["1", "new 🦀"]);
        assert!(map.view.bindings.contains_key(&row));
        assert_eq!(
            map.view.bindings[&blockui_abi::key("labels.map.first")].text,
            "new 🦀"
        );
    }
    map.dispatch(Message {
        action: blockui_abi::key("map.rename"),
        value: 1,
    })?;
    assert_eq!(
        map.text_list("labels.entries"),
        ["3", "new 🦀", "4", "标题"]
    );
    assert!(!map.view.bindings.contains_key(&row));
    assert_eq!(
        map.view.bindings[&blockui_abi::key("labels.map.first")].text,
        "No first title"
    );
    assert_eq!(
        map.view.bindings[&blockui_abi::key("labels.map.3")].text,
        "new 🦀"
    );
    let map_version = map.implementation_version();
    let map_before = map.text_list("labels.entries").to_vec();
    assert!(map.reload(Path::new(&paths[7])).is_err());
    assert_eq!(map.implementation_version(), map_version);
    assert_eq!(map.text_list("labels.entries"), map_before);
    map.reload(Path::new(&paths[8]))?;
    assert_eq!(
        map.text_list("labels.entries"),
        ["document:3", "new 🦀", "document:4", "标题"]
    );
    assert_eq!(
        map.view.bindings[&blockui_abi::key("labels.map.document:3")].text,
        "new 🦀"
    );
    println!(
        "PASS runtime text map preserves storage across reload, current-code replacement and rejected edit rollback"
    );
    let mut full_map = Actor::new(134, Path::new(".runtime"))?;
    full_map.reload(Path::new(&paths[0]))?;
    full_map.dispatch(message_for_map("map.fill", 0))?;
    assert_eq!(full_map.text_list("labels.entries").len(), 4096);
    let full_buffer = full_map.text_list("labels.entries").as_ptr();
    full_map.reload(Path::new(&paths[1]))?;
    assert_eq!(full_map.text_list("labels.entries").as_ptr(), full_buffer);
    assert!(full_map.dispatch(message_for_map("map.fill", 1)).is_err());
    assert_eq!(full_map.text_list("labels.entries").len(), 4096);
    assert_eq!(
        full_map.view.bindings[&blockui_abi::key("labels.map.entry:0000")].text,
        "title"
    );
    full_map.dispatch(message_for_map("map.recover", 0))?;
    assert_eq!(full_map.text_list("labels.entries").len(), 4096);
    assert_eq!(
        full_map.view.bindings[&blockui_abi::key("labels.map.entry:0000")].text,
        "recovered"
    );
    println!(
        "PASS full text map survives reload; capacity overflow rolls back and handled batch errors permit independent edits"
    );
    let mut records = Actor::new(136, Path::new(".runtime"))?;
    records.reload(Path::new(&paths[0]))?;
    records.dispatch(message_for_map("records.put", 42))?;
    let record_buffer = records.text_list("labels.documents").as_ptr();
    records.reload(Path::new(&paths[1]))?;
    assert_eq!(
        records.text_list("labels.documents").as_ptr(),
        record_buffer
    );
    let record_row = blockui_abi::key("labels.document.42");
    assert_eq!(records.view.bindings[&record_row].text, "1: Document 🦀");
    assert!(
        records
            .dispatch(message_for_map("records.put", -42))
            .is_err()
    );
    assert_eq!(records.view.bindings[&record_row].text, "1: Document 🦀");
    records.dispatch(message_for_map("records.put", 42))?;
    assert_eq!(records.view.bindings[&record_row].text, "2: Document 🦀");
    assert!(
        records
            .dispatch(message_for_map("records.batch", 1))
            .is_err()
    );
    assert_eq!(records.view.bindings[&record_row].text, "2: Document 🦀");
    assert!(
        !records
            .view
            .bindings
            .contains_key(&blockui_abi::key("labels.document.73"))
    );
    records.dispatch(message_for_map("records.batch", 0))?;
    assert_eq!(
        records.view.bindings[&record_row].text,
        "3: Document 🦀 edited"
    );
    assert_eq!(
        records.view.bindings[&blockui_abi::key("labels.document.73")].text,
        "3: Second document"
    );
    println!(
        "PASS typed dynamic records retain host storage through reload and use current codec/action with rollback"
    );
    let mut actor = Actor::new(132, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let message = |name, value| Message {
        action: blockui_abi::key(name),
        value,
    };
    assert_eq!(actor.text_list("labels.items"), ["default", "标题😀"]);
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "2"
    );
    let default_schema = actor.schema_version;
    let default_implementation = actor.implementation_version();
    let default_storage = actor.state_address();
    let default_slot = actor.slot_address("labels.items");
    let default_buffer = actor.text_list("labels.items").as_ptr();
    let default_text = actor.text_list("labels.items")[1].as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.schema_version, default_schema);
    assert_ne!(actor.implementation_version(), default_implementation);
    assert_eq!(actor.state_address(), default_storage);
    assert_eq!(actor.slot_address("labels.items"), default_slot);
    let mut fresh = Actor::new(137, Path::new(".runtime"))?;
    fresh.reload(Path::new(&paths[1]))?;
    assert_eq!(fresh.text_list("labels.items"), ["changed default", "new"]);
    assert_eq!(fresh.schema_version, default_schema);
    drop(fresh);
    assert_eq!(actor.text_list("labels.items"), ["default", "标题😀"]);
    assert_eq!(actor.text_list("labels.items").as_ptr(), default_buffer);
    assert_eq!(actor.text_list("labels.items")[1].as_ptr(), default_text);
    // This migration callback expects previous integer-list state and would fail
    // if called for this compatible text-list schema. Compatible reload skips it.
    actor.reload(Path::new(&paths[6]))?;
    assert_eq!(actor.schema_version, default_schema);
    assert_eq!(actor.text_list("labels.items").as_ptr(), default_buffer);
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(message("seed", 0))?;
    assert_eq!(actor.text_list("labels.items"), ["", "a\0b", "标题😀"]);
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "3"
    );
    let state = actor.state_address();
    let slot = actor.slot_address("labels.items");
    let buffer = actor.text_list("labels.items").as_ptr();
    let string = actor.text_list("labels.items")[2].as_ptr();
    let version = actor.implementation_version();
    assert!(actor.dispatch(message("append", -1)).is_err());
    assert_eq!(actor.text_list("labels.items").as_ptr(), buffer);
    assert_eq!(actor.text_list("labels.items")[2].as_ptr(), string);
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.text_list("labels.items").as_ptr(), buffer);
    for round in 0..40 {
        actor.reload(Path::new(&paths[round % 2]))?;
        assert_eq!(
            actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
            "3"
        );
        assert_eq!(actor.state_address(), state);
        assert_eq!(actor.slot_address("labels.items"), slot);
        assert_eq!(actor.text_list("labels.items").as_ptr(), buffer);
        assert_eq!(actor.text_list("labels.items")[2].as_ptr(), string);
        assert_eq!(
            actor.view.bindings[&blockui_abi::key("labels.summary")].text,
            actor
                .text_list("labels.items")
                .join(if round % 2 == 0 { " | " } else { " / " })
        );
        assert_eq!(actor.artifact_count(), 1);
        assert_eq!(actor.mapped_implementation_count()?, 1);
    }
    actor.enqueue(message("append", 9))?;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("labels.items"), slot);
    assert_eq!(actor.text_list("labels.items").as_ptr(), buffer);
    assert_eq!(actor.text_list("labels.items")[2].as_ptr(), string);
    actor.reload(Path::new(&paths[3]))?;
    assert_ne!(actor.schema_version, default_schema);
    assert_eq!(actor.integer("labels.bonus"), 7);
    assert_eq!(actor.text_list("labels.tags"), ["priority", "资料😀"]);
    assert_eq!(actor.slot_address("labels.items"), slot);
    assert_eq!(actor.text_list("labels.items").as_ptr(), buffer);
    assert_eq!(actor.text_list("labels.items")[2].as_ptr(), string);
    actor.tick()?;
    assert_eq!(actor.text_list("labels.items").last().unwrap(), "new 9 😀");
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "4"
    );
    assert!(
        actor.view.bindings[&blockui_abi::key("labels.summary")]
            .text
            .contains("new 9 😀")
    );
    actor.dispatch(message("delayed-append", 11))?;
    let task_id = blockui_abi::key("labels.pending_append");
    let task_generation = actor.tasks[&task_id].generation();
    let task_worker = actor.tasks[&task_id].worker_id();
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.tasks[&task_id].generation(), task_generation);
    assert_eq!(actor.tasks[&task_id].worker_id(), task_worker);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while actor.text_list("labels.items").last().map(String::as_str) != Some("old 11 😀")
        && std::time::Instant::now() < deadline
    {
        actor.tick()?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(actor.text_list("labels.items").last().unwrap(), "old 11 😀");
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "5"
    );
    assert_eq!(actor.slot_address("labels.items"), slot);
    actor.dispatch(message("delayed-append", 12))?;
    let task_id = blockui_abi::key("labels.pending_append");
    let task_generation = actor.tasks[&task_id].generation();
    let task_worker = actor.tasks[&task_id].worker_id();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.tasks[&task_id].generation(), task_generation);
    assert_eq!(actor.tasks[&task_id].worker_id(), task_worker);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while actor.text_list("labels.items").last().map(String::as_str) != Some("new 12 😀")
        && std::time::Instant::now() < deadline
    {
        actor.tick()?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(actor.text_list("labels.items").last().unwrap(), "new 12 😀");
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "6"
    );
    assert_eq!(actor.slot_address("labels.items"), slot);
    let version = actor.implementation_version();
    let buffer = actor.text_list("labels.items").as_ptr();
    assert!(actor.reload(Path::new(&paths[5])).is_err());
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.text_list("labels.items").as_ptr(), buffer);
    drop(actor);
    let mut actor = Actor::new(132, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[4]))?;
    actor.dispatch(message("seed", 0))?;
    assert_eq!(actor.list("labels.items"), [7, 8]);
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "2"
    );
    assert!(actor.reload(Path::new(&paths[0])).is_err());
    actor.reload(Path::new(&paths[6]))?;
    assert_eq!(actor.text_list("labels.items"), ["7", "8"]);
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "2"
    );
    actor.reload(Path::new(&paths[5]))?;
    assert_eq!(actor.list("labels.items"), [7, 8]);
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("labels.count.text")].text,
        "2"
    );
    println!(
        "PASS host-owned text lists: Unicode/NUL/empty values, action/candidate rollback, nested allocation identity across code/schema replacement, queued current-code actions, task field rename with stable explicit key in both directions, reactive output, explicit integer/text list migrations and failed parse retention"
    );
    Ok(())
}

fn message_for_map(name: &str, value: i64) -> Message {
    Message {
        action: blockui_abi::key(name),
        value,
    }
}
