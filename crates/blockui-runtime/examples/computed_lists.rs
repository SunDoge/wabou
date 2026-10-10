use blockui_abi::key;
use blockui_runtime::{Actor, Message, Value};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 3 {
        return Err("computed_lists base replacement bad".into());
    }
    let mut actor = Actor::new(98, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(
        actor.computed("selected"),
        Some(&Value::IntegerList(vec![2, 4]))
    );
    assert_eq!(actor.computed("length"), Some(&Value::Integer(2)));
    assert_eq!(actor.computed("empty"), Some(&Value::Boolean(false)));
    let evaluations = actor.computed_evaluations("length");
    actor.dispatch(Message {
        action: key("append"),
        value: 5,
    })?;
    assert_eq!(actor.computed_evaluations("length"), evaluations);
    let slot = actor.slot_address("items");
    let storage = actor.state_address();
    let schema = actor.schema_version;
    actor.enqueue(Message {
        action: key("append"),
        value: 6,
    })?;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.list("items"), &[2, 3, 4, 5]);
    assert_eq!(
        actor.computed("selected"),
        Some(&Value::IntegerList(vec![4, 2]))
    );
    actor.tick()?;
    assert_eq!(actor.list("items"), &[2, 3, 4, 5, 60]);
    assert_eq!(
        actor.computed("selected"),
        Some(&Value::IntegerList(vec![60, 4, 2]))
    );
    let buffer = actor.list("items").as_ptr();
    let implementation = actor.implementation_version();
    let text = actor.view.bindings[&key("selected.text")].text.clone();
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert_eq!(actor.implementation_version(), implementation);
    assert_eq!(actor.view.bindings[&key("selected.text")].text, text);
    for round in 0..20 {
        actor.reload(Path::new(&paths[round % 2]))?;
        assert_eq!(actor.slot_address("items"), slot);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.list("items").as_ptr(), buffer);
        assert_eq!(actor.list("items"), &[2, 3, 4, 5, 60]);
        let expected = if round % 2 == 0 {
            vec![2, 4, 60]
        } else {
            vec![60, 4, 2]
        };
        assert_eq!(
            actor.computed("selected"),
            Some(&Value::IntegerList(expected.clone()))
        );
        assert_eq!(
            actor.view.bindings[&key("page.text")].text,
            format!("{:?}", &expected[1..])
        );
        assert_eq!(actor.artifact_count(), 1);
        #[cfg(target_os = "linux")]
        assert_eq!(actor.mapped_implementation_count()?, 1);
    }
    actor.dispatch(Message {
        action: key("clear"),
        value: 0,
    })?;
    assert_eq!(actor.computed("length"), Some(&Value::Integer(0)));
    assert_eq!(actor.computed("empty"), Some(&Value::Boolean(true)));
    assert_eq!(actor.view.bindings[&key("summary.text")].text, "0:true");
    assert_eq!(actor.view.bindings[&key("page.text")].text, "[]");
    assert_eq!(actor.slot_address("items"), slot);
    assert_eq!(actor.state_address(), storage);
    println!(
        "PASS real DLL length/empty memo queries track clearing after 20 implementation replacements without copying list items in the query"
    );
    println!(
        "PASS real computed-list DLLs: equality pruning, queued new-code action, rejected oversize candidate, stable state/list buffer and 20 bounded unload/reload cycles"
    );
    Ok(())
}
