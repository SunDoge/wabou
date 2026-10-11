use super::*;

fn assert_checkbox_semantics(source: &NativeSource, widget: u64, checked: bool) {
    let snapshot = source.semantic_snapshot().unwrap();
    let node = snapshot
        .nodes
        .iter()
        .find(|node| node.id == widget)
        .unwrap();
    assert_eq!(node.label.as_deref(), Some("Enabled"));
    assert_eq!(
        node.states.checked,
        Some(if checked {
            shell::SemanticToggleState::On
        } else {
            shell::SemanticToggleState::Off
        })
    );
    assert_eq!(snapshot.focus, Some(widget));
}

pub fn proof(
    base: &Path,
    replacement: &Path,
    missing: &Path,
    evolved: &Path,
) -> Result<(), String> {
    let mut actor = Actor::new(210, Path::new(".runtime"))?;
    actor.reload(base)?;
    let toggle = key("toggle");
    let state = actor.state_address();
    let slot = actor.slot_address("scalar.fields.values");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.set_focus(Some(toggle));
    let mut tcx = TextContext::new();
    for (width, height) in [(640, 480), (320, 480)] {
        source.build_frame(&mut tcx, width, height);
        let mut bottom = 0.0;
        for name in ["signed", "unsigned", "ratio", "enabled", "toggle"] {
            let rect = source
                .widget_layout_rect(key(name))
                .ok_or("missing field control layout")?;
            assert!(rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01);
            assert!(rect[1] >= bottom && rect[3] <= height as f32 + 0.01);
            assert!(rect[2] > rect[0] && rect[3] > rect[1]);
            bottom = rect[3];
        }
    }
    assert_checkbox_semantics(&source, toggle, false);
    let node = source.ids[&toggle];
    let key_event = || {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: "Enter".into(),
            key_without_modifiers: "Enter".into(),
            code: "Enter".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: shell::Modifiers::empty(),
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.widget_boolean(toggle), Some(true));
    assert_checkbox_semantics(&source, toggle, true);
    assert_eq!(source.actor.view.bindings[&key("signed")].text, "1");
    let buffer = source.actor.text("scalar.fields.values").as_ptr();
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("scalar.fields.values"), slot);
    assert_eq!(source.actor.text("scalar.fields.values").as_ptr(), buffer);
    assert_eq!(source.ids[&toggle], node);
    assert_eq!(source.focus, Some(toggle));
    assert_eq!(source.actor.widget_boolean(toggle), Some(true));
    assert_checkbox_semantics(&source, toggle, true);
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 640, 480);
    assert_eq!(source.actor.widget_boolean(toggle), Some(false));
    assert_checkbox_semantics(&source, toggle, false);
    assert_eq!(source.actor.view.bindings[&key("signed")].text, "3");
    assert_eq!(source.ids[&toggle], node);
    println!(
        "PASS native record checkbox: keyboard toggles the host record, replacement retains focus/node/state/buffer, new action executes; field layouts fit at 320/640 widths"
    );
    let schema = source.actor.schema_version;
    let buffer = source.actor.text("scalar.fields.values").as_ptr();
    let binding = source.actor.view.bindings[&toggle].node.binding;
    assert!(
        source
            .replace(missing)
            .unwrap_err()
            .contains("explicit migration is required")
    );
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.schema_version, schema);
    assert_checkbox_semantics(&source, toggle, false);
    assert_eq!(source.actor.text("scalar.fields.values").as_ptr(), buffer);
    assert_eq!(source.ids[&toggle], node);
    assert_eq!(source.focus, Some(toggle));
    source.replace(evolved)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_ne!(source.actor.schema_version, schema);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("scalar.fields.values"), slot);
    assert_eq!(source.ids[&toggle], node);
    assert_eq!(source.actor.view.bindings[&toggle].node.binding, binding);
    assert_eq!(source.focus, Some(toggle));
    assert_eq!(source.actor.widget_boolean(toggle), Some(false));
    assert_checkbox_semantics(&source, toggle, false);
    assert_eq!(source.actor.view.bindings[&key("title")].text, "Settings");
    let mut bottom = 0.0;
    for name in ["title", "signed", "unsigned", "ratio", "enabled", "toggle"] {
        let rect = source
            .widget_layout_rect(key(name))
            .ok_or("missing migrated field layout")?;
        assert!(rect[0] >= 0.0 && rect[2] <= 320.01);
        assert!(rect[1] >= bottom && rect[3] <= 480.01);
        bottom = rect[3];
    }
    source.handle_event(key_event());
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.widget_boolean(toggle), Some(true));
    assert_checkbox_semantics(&source, toggle, true);
    assert_eq!(source.actor.view.bindings[&key("signed")].text, "5");
    assert_eq!(source.ids[&toggle], node);
    assert_eq!(source.focus, Some(toggle));
    println!(
        "PASS native checkbox schema migration: missing upgrade preserves current frame, default-field upgrade preserves focus/node/binding identity and shifted selector toggles through current code"
    );
    Ok(())
}
