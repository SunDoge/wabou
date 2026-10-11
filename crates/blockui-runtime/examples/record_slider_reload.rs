use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::path::Path;
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 4 {
        return Err("record_slider_reload base replacement invalid-range evolved".into());
    }
    let mut actor = Actor::new(213, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let slider = key("volume");
    actor.edit_slider(slider, 40)?;
    let state = actor.state_address();
    let slot = actor.slot_address("slider.fields.settings");
    let buffer = actor.text("slider.fields.settings").as_ptr();
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("slider.fields.settings"), slot);
    assert_eq!(actor.text("slider.fields.settings").as_ptr(), buffer);
    actor.edit_slider(slider, 20)?;
    assert_eq!(actor.integer_control_value(slider), Some(30));
    let version = actor.implementation_version();
    let buffer = actor.text("slider.fields.settings").as_ptr();
    assert!(
        actor
            .reload(Path::new(&paths[2]))
            .unwrap_err()
            .contains("within its range")
    );
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.text("slider.fields.settings").as_ptr(), buffer);
    let schema = actor.schema_version;
    let binding = actor.view.bindings[&slider].node.binding;
    actor.reload(Path::new(&paths[3]))?;
    assert_ne!(actor.schema_version, schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("slider.fields.settings"), slot);
    assert_eq!(actor.view.bindings[&slider].node.binding, binding);
    assert_eq!(actor.integer_control_value(slider), Some(30));
    assert_eq!(actor.view.bindings[&key("title")].text, "Sound");
    let evaluations = actor.view.bindings[&slider].evaluations;
    actor.dispatch(Message {
        action: key("slider.fields.metadata"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&slider].evaluations, evaluations);
    let buffer = actor.text("slider.fields.settings").as_ptr();
    for value in [80, 100] {
        assert!(actor.edit_slider(slider, value).is_err());
        assert_eq!(actor.text("slider.fields.settings").as_ptr(), buffer);
        assert_eq!(actor.integer_control_value(slider), Some(30));
    }
    actor.edit_slider(slider, 40)?;
    assert_eq!(actor.integer_control_value(slider), Some(50));
    println!(
        "PASS record slider DLL: compatible replacement retains state/buffer, new action executes, invalid range rejects, schema shifts selector while retaining identity; unrelated metadata skips slider and failed/out-of-range writes roll back"
    );
    Ok(())
}
