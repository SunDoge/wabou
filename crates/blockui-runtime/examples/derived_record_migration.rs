use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;

fn send(actor: &mut Actor, action: &str, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(action),
        value,
    })
}
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 6 {
        return Err(
            "derived_record_migration base compatible missing rejected evolved bad-default".into(),
        );
    }
    let mut actor = Actor::new(140, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    send(&mut actor, "put", 42)?;
    let original = actor.text_list("records.documents").to_vec();
    let old_format = original[1].split_once('\n').unwrap().0.to_owned();
    assert!(old_format.starts_with("app.document@record1-"));
    let marker = actor.slot_address(&old_format);
    assert_ne!(marker, 0);
    let schema = actor.schema_version;
    let state = actor.state_address();
    let slot = actor.slot_address("records.documents");
    let buffer = actor.text_list("records.documents").as_ptr();
    let version = actor.implementation_version();
    actor.reload(Path::new(&paths[1]))?;
    assert_ne!(actor.implementation_version(), version);
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address(&old_format), marker);
    assert_eq!(actor.slot_address("records.documents"), slot);
    assert_eq!(actor.text_list("records.documents").as_ptr(), buffer);
    assert_eq!(actor.text_list("records.documents"), original);
    send(&mut actor, "edit", 1)?;
    assert!(
        actor.view.bindings[&key("document.42")]
            .text
            .contains("priority: 3")
    );
    let current = actor.text_list("records.documents").to_vec();
    let buffer = actor.text_list("records.documents").as_ptr();
    let version = actor.implementation_version();
    for path in [&paths[2], &paths[3], &paths[5]] {
        assert!(actor.reload(Path::new(path)).is_err());
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.slot_address(&old_format), marker);
        assert_eq!(actor.text_list("records.documents").as_ptr(), buffer);
        assert_eq!(actor.text_list("records.documents"), current);
    }
    // Old handler and codec remain usable after both rejected upgrades.
    send(&mut actor, "edit", 1)?;
    actor.reload(Path::new(&paths[4]))?;
    let new_format = actor.text_list("records.documents")[1]
        .split_once('\n')
        .unwrap()
        .0
        .to_owned();
    assert_ne!(old_format, new_format);
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("records.documents"), slot);
    assert_eq!(actor.slot_address(&old_format), 0);
    assert_ne!(actor.slot_address(&new_format), 0);
    let text = &actor.view.bindings[&key("document.42")].text;
    assert!(
        text.contains("Draft 🦀") && text.contains("priority: 5") && text.contains("Uncategorized")
    );
    send(&mut actor, "edit", 1)?;
    assert!(
        actor.view.bindings[&key("document.42")]
            .text
            .contains("priority: 15")
    );
    let records = actor.text_list("records.documents").to_vec();
    assert!(send(&mut actor, "upgrade.outside", 0).is_err());
    assert_eq!(actor.text_list("records.documents"), records);
    println!(
        "PASS stable record namespace survives actual DLL module/type refactor without schema migration or storage recreation"
    );
    println!(
        "PASS generated field upgrade defaults publish transactionally; failing default expression retains old implementation schema marker and record buffers"
    );
    println!(
        "PASS derived record codecs retain runtime buffers on compatible DLL replacement; automatic format marker changes require migration; missing/rejected upgrades roll back and explicit Rust migration preserves fields and current editing"
    );
    Ok(())
}
