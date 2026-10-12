use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;

fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 8 {
        return Err(
            "record_schema_reload original missing upgraded added automatic noop partial bad"
                .into(),
        );
    }
    let mut actor = Actor::new(145, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(Message {
        action: key("put"),
        value: 0,
    })?;
    let identity = actor.state_address();
    let slot = actor.slot_address("documents");
    let buffer = actor.text_list("documents").as_ptr();
    let archive = actor.text_list("archive").as_ptr();
    let schema = actor.schema_version;
    let version = actor.implementation_version();
    assert!(actor.reload(Path::new(&paths[1])).is_err());
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    actor.dispatch(Message {
        action: key("put"),
        value: 0,
    })?;
    actor.reload(Path::new(&paths[2]))?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), identity);
    assert_eq!(actor.slot_address("documents"), slot);
    assert_eq!(actor.text_list("archive").as_ptr(), archive);
    assert!(actor.text_list("documents")[1].contains("Draft 🦀"));
    assert!(actor.text_list("documents")[1].ends_with("1\n0"));
    println!(
        "PASS automatic per-collection codec markers reject missing migration across real DLL replacement without view reads; explicit defaults retain state, collection slot and unaffected archive allocation"
    );
    let mut actor = Actor::new(147, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(Message {
        action: key("put"),
        value: 0,
    })?;
    let buffer = actor.text_list("documents").as_ptr();
    let archive = actor.text_list("archive").as_ptr();
    let identity = actor.state_address();
    actor.reload(Path::new(&paths[3]))?;
    assert_eq!(actor.state_address(), identity);
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert_eq!(actor.text_list("archive").as_ptr(), archive);
    assert!(actor.text_list("new.documents").is_empty());
    actor.dispatch(Message {
        action: key("add"),
        value: 0,
    })?;
    assert!(actor.text_list("new.documents")[1].contains("Optional 🦀"));
    let added = actor.text_list("new.documents").as_ptr();
    actor.reload(Path::new(&paths[3]))?;
    assert_eq!(actor.text_list("new.documents").as_ptr(), added);
    assert!(actor.reload(Path::new(&paths[1])).is_err());
    assert_eq!(actor.text_list("documents").as_ptr(), buffer);
    assert_eq!(actor.text_list("new.documents").as_ptr(), added);
    println!(
        "PASS adding a new record collection initializes only new slots across real DLL replacement without custom migration; existing buffers survive and later invalid codec replacement rolls back"
    );
    let mut automatic = Actor::new(148, Path::new(".runtime"))?;
    automatic.reload(Path::new(&paths[0]))?;
    automatic.dispatch(Message {
        action: key("put"),
        value: 0,
    })?;
    let state = automatic.state_address();
    let slot = automatic.slot_address("documents");
    let documents = automatic.text_list("documents").as_ptr();
    let archive = automatic.text_list("archive").as_ptr();
    for path in &paths[5..8] {
        assert!(automatic.reload(Path::new(path)).is_err());
        assert_eq!(automatic.text_list("documents").as_ptr(), documents);
        assert_eq!(automatic.text_list("archive").as_ptr(), archive);
    }
    automatic.reload(Path::new(&paths[4]))?;
    assert_eq!(automatic.state_address(), state);
    assert_eq!(automatic.slot_address("documents"), slot);
    assert_eq!(automatic.text_list("archive").as_ptr(), archive);
    assert_eq!(automatic.text_list("documents")[0], "42");
    assert_eq!(automatic.text_list("documents")[2], "99");
    assert!(automatic.text_list("documents")[1].contains("Draft 🦀"));
    assert!(automatic.text_list("documents")[3].contains("Second"));
    let buffer = automatic.text_list("documents").as_ptr();
    automatic.reload(Path::new(&paths[2]))?;
    assert_eq!(automatic.text_list("documents").as_ptr(), buffer);
    println!(
        "PASS automatic record collection upgrades across real DLLs preserve keys, defaults, actor/slot and unrelated archive; no-op and partial/per-record failures roll back, compatible upgraded code retains buffers"
    );
    Ok(())
}
