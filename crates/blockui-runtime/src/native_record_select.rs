use super::*;
pub fn proof(
    base: &Path,
    replacement: &Path,
    invalid: &Path,
    evolved: &Path,
) -> Result<(), String> {
    let mut actor = Actor::new(218, Path::new(".runtime"))?;
    actor.reload(base)?;
    let select = key("choice");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.set_focus(Some(select));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 320, 480);
    let node = source.ids[&select];
    let key_event = |name: &str| {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Default::default(),
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(key_event("Enter"));
    source.handle_event(key_event("ArrowDown"));
    source.build_frame(&mut tcx, 320, 480);
    assert!(source.selects[&select].is_open());
    assert_eq!(source.selects[&select].active(), Some(1));
    let descendant = source
        .semantic_snapshot()
        .unwrap()
        .nodes
        .iter()
        .find(|n| n.id == select)
        .unwrap()
        .active_descendant;
    let buffer = source.actor.text("select.fields.settings").as_ptr();
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.text("select.fields.settings").as_ptr(), buffer);
    assert_eq!(source.focus, Some(select));
    assert!(source.selects[&select].is_open());
    assert_eq!(source.selects[&select].active(), Some(1));
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|n| n.id == select)
            .unwrap()
            .active_descendant,
        descendant
    );
    source.handle_event(key_event("ArrowDown"));
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.widget_integer(select), Some(2));
    assert_eq!(source.actor.view.bindings[&key("edits")].text, "2");
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 320, 480);
    let options = source.selects[&select].options().as_ptr();
    let labels: Vec<_> = source.selects[&select]
        .options()
        .iter()
        .map(|o| o.label.as_ptr())
        .collect();
    assert!(source.replace(invalid).is_err());
    source.build_frame(&mut tcx, 320, 480);
    assert!(source.selects[&select].is_open());
    assert_eq!(source.selects[&select].active(), Some(2));
    source.replace(evolved)?;
    for (width, height) in [(320, 480), (640, 480)] {
        source.build_frame(&mut tcx, width, height);
        let mut bottom = 0.0;
        for name in ["title", "choice", "value", "edits"] {
            let rect = source
                .widget_layout_rect(key(name))
                .ok_or("missing select layout")?;
            assert!(rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01);
            assert!(rect[1] >= bottom && rect[3] <= height as f32 + 0.01);
            assert!(rect[2] > rect[0] && rect[3] > rect[1]);
            bottom = rect[3];
        }
    }
    assert!(source.selects[&select].is_open());
    assert_eq!(source.selects[&select].active(), Some(2));
    assert_eq!(source.selects[&select].options().as_ptr(), options);
    for (option, label) in source.selects[&select].options().iter().zip(labels) {
        assert_eq!(option.label.as_ptr(), label);
    }
    assert_eq!(source.ids[&select], node);
    source.handle_event(key_event("ArrowUp"));
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.widget_integer(select), Some(1));
    assert_eq!(source.actor.view.bindings[&key("edits")].text, "4");
    assert_eq!(source.focus, Some(select));
    let snapshot = source.semantic_snapshot().unwrap();
    let combo = snapshot.nodes.iter().find(|n| n.id == select).unwrap();
    assert_eq!(combo.label.as_deref(), Some("Theme"));
    assert_eq!(combo.value.as_deref(), Some("Light"));
    println!(
        "PASS native record select: open popup/highlight/focus/semantic identity survive actual DLL replacement and schema evolution; retained control/options/labels/node and 320/640 layout; commit uses current action"
    );
    Ok(())
}
