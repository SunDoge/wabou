use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("record_migration base missing rejected evolved".into());
    }
    let mut actor = Actor::new(137, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(Message {
        action: key("put"),
        value: 42,
    })?;
    actor.dispatch(Message {
        action: key("edit"),
        value: 0,
    })?;
    assert!(
        actor
            .dispatch(Message {
                action: key("edit"),
                value: -1
            })
            .is_err()
    );
    assert!(actor.text_list("records.documents")[1].starts_with("document.v1\n1\n"));
    let state = actor.state_address();
    let slot = actor.slot_address("records.documents");
    let buffer = actor.text_list("records.documents").as_ptr();
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    let marker = actor.slot_address("document.v1");
    assert_ne!(marker, 0);
    assert_eq!(actor.slot_address("document.v2"), 0);
    for candidate in &paths[1..3] {
        assert!(actor.reload(Path::new(candidate)).is_err());
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.slot_address("document.v1"), marker);
        assert_eq!(actor.slot_address("document.v2"), 0);
        assert_eq!(actor.text_list("records.documents").as_ptr(), buffer);
        assert!(actor.text_list("records.documents")[1].starts_with("document.v1\n"));
    }
    actor.reload(Path::new(&paths[3]))?;
    assert_eq!(actor.state_address(), state);
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.slot_address("document.v1"), 0);
    assert_ne!(actor.slot_address("document.v2"), 0);
    assert_eq!(actor.slot_address("records.documents"), slot);
    assert_eq!(
        actor.text_list("records.documents"),
        ["42", "document.v2\n1\nUncategorized\nDraft 🦀"]
    );
    assert!(
        actor.view.bindings[&key("document.42")]
            .text
            .contains("Uncategorized")
    );
    actor.dispatch(Message {
        action: key("edit"),
        value: 1,
    })?;
    assert!(actor.text_list("records.documents")[1].starts_with("document.v2\n11\n"));
    let edited = actor.text_list("records.documents").to_vec();
    assert!(
        actor
            .dispatch(Message {
                action: key("edit"),
                value: -1
            })
            .is_err()
    );
    assert_eq!(actor.text_list("records.documents"), edited);
    for mode in [0, 1] {
        actor.dispatch(Message {
            action: key("recover"),
            value: mode,
        })?;
        let records = actor.text_list("records.documents");
        assert_eq!(&records[..2], &edited[..]);
        assert_eq!(records.len(), 4);
        assert_eq!(records[2], "74");
        assert!(records[3].contains("Recovered"));
    }
    actor.dispatch(Message {
        action: key("put"),
        value: 73,
    })?;
    assert_eq!(actor.text_list("records.documents").len(), 6);
    let before_conflict = actor.text_list("records.documents").to_vec();
    assert!(
        actor
            .dispatch(Message {
                action: key("conflict"),
                value: 0
            })
            .is_err()
    );
    assert_eq!(actor.text_list("records.documents"), before_conflict);
    actor.dispatch(Message {
        action: key("conflict"),
        value: 1,
    })?;
    assert!(
        actor.text_list("records.documents")[1].starts_with("document.v2\n7\nInner\nInner edit")
    );
    actor.dispatch(Message {
        action: key("conflict"),
        value: 2,
    })?;
    assert!(actor.text_list("records.documents")[1].starts_with("document.v2\n999\n"));
    assert!(
        actor
            .text_list("records.documents")
            .iter()
            .any(|value| value == "75")
    );
    let before_batch = actor.text_list("records.documents").to_vec();
    assert!(
        actor
            .dispatch(Message {
                action: key("batch.conflict"),
                value: 0
            })
            .is_err()
    );
    assert_eq!(actor.text_list("records.documents"), before_batch);
    actor.dispatch(Message {
        action: key("batch.conflict"),
        value: 1,
    })?;
    assert_eq!(
        &actor.text_list("records.documents")[..before_batch.len()],
        &before_batch
    );
    assert!(
        actor
            .text_list("records.documents")
            .iter()
            .any(|value| value == "76")
    );
    println!(
        "PASS typed record field migration preserves IDs/values/state/slot; missing and rejected migrations retain old code/buffer; new actions remain usable; single-record edits skip missing keys, use current code and roll back errors; handled callback and capacity errors preserve records while later writes commit"
    );
    Ok(())
}
