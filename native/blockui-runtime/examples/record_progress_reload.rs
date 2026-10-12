use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("record_progress_reload base replacement missing evolved".into());
    }
    let mut actor = Actor::new(215, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let send = |actor: &mut Actor, value: f64| {
        actor.dispatch(Message {
            action: key("progress.fields.change"),
            value: value.to_bits() as i64,
        })
    };
    send(&mut actor, 0.5)?;
    let state = actor.state_address();
    let slot = actor.slot_address("progress.fields.settings");
    let buffer = actor.text("progress.fields.settings").as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.text("progress.fields.settings").as_ptr(), buffer);
    send(&mut actor, 0.5)?;
    assert_eq!(actor.view.bindings[&key("progress")].text, "0.25");
    let buffer = actor.text("progress.fields.settings").as_ptr();
    let version = actor.implementation_version();
    assert!(
        actor
            .reload(Path::new(&paths[2]))
            .unwrap_err()
            .contains("explicit migration is required")
    );
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.text("progress.fields.settings").as_ptr(), buffer);
    let binding = actor.view.bindings[&key("progress")].node.binding;
    actor.reload(Path::new(&paths[3]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("progress.fields.settings"), slot);
    assert_eq!(actor.view.bindings[&key("progress")].node.binding, binding);
    assert_eq!(actor.view.bindings[&key("title")].text, "Transfer");
    assert_eq!(actor.view.bindings[&key("progress")].text, "0.25");
    let evaluations = actor.view.bindings[&key("progress")].evaluations;
    actor.dispatch(Message {
        action: key("progress.fields.metadata"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&key("progress")].evaluations,
        evaluations
    );
    let buffer = actor.text("progress.fields.settings").as_ptr();
    for value in [-1.0, 3.0, f64::NAN, f64::INFINITY, 0.75] {
        assert!(send(&mut actor, value).is_err());
        assert_eq!(actor.text("progress.fields.settings").as_ptr(), buffer);
        assert_eq!(actor.view.bindings[&key("progress")].text, "0.25");
    }
    send(&mut actor, 1.0)?;
    assert_eq!(actor.view.bindings[&key("progress")].text, "0.5");
    println!(
        "PASS record progress DLL: state/buffer survive code replacement, current action runs, schema shifts selector while keeping identity; missing migration and invalid fractions roll back, metadata skips progress"
    );
    Ok(())
}
