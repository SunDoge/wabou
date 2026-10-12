use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 2 {
        return Err("record_reactivity_reload base replacement".into());
    }
    let mut actor = Actor::new(199, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    actor.dispatch(Message {
        action: key("Model.batch_initialize"),
        value: 0,
    })?;
    actor.dispatch(Message {
        action: key("Model.batch_edit"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&key("caption.first")].text,
        "Changed first"
    );
    let caption_slot = actor.slot_address("captions");
    let caption_buffer = actor.text_list("captions").as_ptr();
    actor.dispatch(Message {
        action: key("Model.initialize"),
        value: 0,
    })?;
    let state = actor.state_address();
    let slot = actor.slot_address("rows");
    let second = actor.view.bindings[&key("second")].evaluations;
    let memo = actor.computed_evaluations("second.memo");
    actor.dispatch(Message {
        action: key("Model.edit_first"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "Changed first");
    assert_eq!(actor.view.bindings[&key("second")].evaluations, second);
    assert_eq!(actor.computed_evaluations("second.memo"), memo);
    actor.dispatch(Message {
        action: key("Model.record_batch_update"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "Batch first");
    let buffer = actor.text_list("rows").as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.slot_address("captions"), caption_slot);
    assert_eq!(actor.text_list("captions").as_ptr(), caption_buffer);
    let caption_second = actor.view.bindings[&key("caption.second")].evaluations;
    let captions = actor.text_list("captions").to_vec();
    let failure = actor
        .dispatch(Message {
            action: key("Model.batch_conflict"),
            value: 0,
        })
        .unwrap_err();
    assert!(
        failure.contains("Text map changed during update"),
        "{failure}"
    );
    assert_eq!(actor.text_list("captions"), captions);
    assert_eq!(
        actor.view.bindings[&key("caption.first")].text,
        "Changed first"
    );
    assert_eq!(actor.view.bindings[&key("caption.second")].text, "Second");
    actor.dispatch(Message {
        action: key("Model.batch_edit"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&key("caption.first")].text,
        "Updated first"
    );
    assert_eq!(
        actor.view.bindings[&key("caption.second")].evaluations,
        caption_second
    );
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("rows"), slot);
    assert_eq!(actor.text_list("rows").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&key("first")].text, "Batch first");
    let second = actor.view.bindings[&key("second")].evaluations;
    let memo = actor.computed_evaluations("second.memo");
    actor.dispatch(Message {
        action: key("Model.edit_first"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "Updated first");
    actor.dispatch(Message {
        action: key("Model.prepend"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("second")].evaluations, second);
    assert_eq!(actor.computed_evaluations("second.memo"), memo);
    actor.dispatch(Message {
        action: key("Model.remove_first"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "Missing first");
    actor.dispatch(Message {
        action: key("Model.initialize"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "First");
    assert!(
        actor
            .dispatch(Message {
                action: key("Model.fail_edit"),
                value: 0
            })
            .is_err()
    );
    actor.dispatch(Message {
        action: key("Model.edit_first"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "Updated first");
    assert_eq!(actor.view.bindings[&key("second")].text, "Second");
    assert_eq!(actor.view.bindings[&key("second")].evaluations, second);
    assert_eq!(actor.computed_evaluations("second.memo"), memo);
    actor.dispatch(Message {
        action: key("Model.record_batch_update"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "Batch replacement");
    assert_eq!(actor.view.bindings[&key("second")].evaluations, second);
    let committed = actor.text_list("rows").to_vec();
    let first_evaluations = actor.view.bindings[&key("first")].evaluations;
    for (action, expected) in [
        ("Model.record_batch_fail", "batch callback failed"),
        (
            "Model.record_batch_conflict",
            "Text map changed during update",
        ),
        ("Model.record_batch_invalid", "Record payload exceeds 1 MiB"),
    ] {
        let error = actor
            .dispatch(Message {
                action: key(action),
                value: 0,
            })
            .unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert_eq!(actor.text_list("rows"), committed);
        assert_eq!(actor.view.bindings[&key("first")].text, "Batch replacement");
        assert_eq!(
            actor.view.bindings[&key("first")].evaluations,
            first_evaluations
        );
        assert_eq!(actor.view.bindings[&key("second")].evaluations, second);
    }
    actor.dispatch(Message {
        action: key("Model.record_batch_update"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&key("first")].text, "Batch replacement");
    println!(
        "PASS RecordMap batch update: replacement callback, retained batch state, unrelated row cache, callback/codec/conflict rollback and recovery"
    );
    println!(
        "PASS TextMap batch update through real DLL replacement: compatible slot/buffer retained, replacement mapper runs, unrelated key stays cached, conflicting update rolls back storage and view"
    );
    println!(
        "PASS keyed record dependencies through real DLL replacement: unrelated row stays cached across edits, key reordering, removal/recreation and rollback; new code runs with stable actor/slot/compatible encoded buffer"
    );
    Ok(())
}
