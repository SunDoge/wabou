use super::*;

pub fn proof(
    base: &Path,
    replacement: &Path,
    missing: &Path,
    evolved: &Path,
    window: bool,
) -> Result<(), String> {
    let mut actor = Actor::new(205, Path::new(".runtime"))?;
    actor.reload(base)?;
    let input = key("notepad.input");
    actor.edit_text(input, "a😀bc notes")?;
    let state = actor.state_address();
    let slot = actor.slot_address("notepad.document");
    let buffer = actor.text("notepad.document").as_ptr();
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.set_focus(Some(input));
    let mut tcx = TextContext::new();
    for (width, height) in [(640, 480), (480, 420)] {
        source.build_frame(&mut tcx, width, height);
        let mut bottom = 0.0;
        for name in [
            "notepad.heading",
            "notepad.input",
            "notepad.summary.text",
            "notepad.hint",
        ] {
            let rect = source.widget_layout_rect(key(name)).unwrap();
            assert!(rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01);
            assert!(rect[1] >= bottom && rect[3] <= height as f32 + 0.01);
            assert!(rect[2] > rect[0] && rect[3] > rect[1]);
            bottom = rect[3];
        }
    }
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 480, 420);
    let selection = source.editors[&input].text_selection();
    assert_eq!(
        selection.as_ref().map(|value| (value.anchor, value.head)),
        Some((1, 3))
    );
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 480, 420);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("notepad.document"), slot);
    assert_eq!(source.actor.text("notepad.document").as_ptr(), buffer);
    assert_eq!(source.actor.view.bindings[&input].text, "a😀bc notes");
    assert_eq!(
        source.actor.view.bindings[&key("notepad.summary.text")].text,
        "10 characters · 1 change"
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    let error = source.actor.edit_text(input, "bad\0notes").unwrap_err();
    assert!(error.contains("NUL character"), "{error}");
    source.build_frame(&mut tcx, 480, 420);
    assert_eq!(source.actor.view.bindings[&input].text, "a😀bc notes");
    assert_eq!(source.actor.text("notepad.document").as_ptr(), buffer);
    assert_eq!(source.editors[&input].text_selection(), selection);
    source.handle_event(UiEvent::TextInput("🦀".into()));
    source.build_frame(&mut tcx, 640, 480);
    assert_eq!(source.actor.view.bindings[&input].text, "a🦀bc notes");
    assert_eq!(
        source.actor.view.bindings[&key("notepad.summary.text")].text,
        "10 characters · 2 changes"
    );
    let schema = source.actor.schema_version;
    let current_buffer = source.actor.text("notepad.document").as_ptr();
    let selection = source.editors[&input].text_selection();
    let implementation = source.actor.implementation_version();
    let error = source.replace(missing).unwrap_err();
    assert!(error.contains("explicit migration is required"), "{error}");
    source.build_frame(&mut tcx, 640, 480);
    assert_eq!(source.actor.schema_version, schema);
    assert_eq!(
        source.actor.text("notepad.document").as_ptr(),
        current_buffer
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.actor.implementation_version(), implementation);
    source.handle_event(UiEvent::TextInput("?".into()));
    source.build_frame(&mut tcx, 640, 480);
    assert_eq!(source.actor.view.bindings[&input].text, "a🦀?bc notes");
    assert_eq!(
        source.actor.view.bindings[&key("notepad.summary.text")].text,
        "11 characters · 3 changes"
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.focus, Some(input));
    let selection = source.editors[&input].text_selection();
    source.replace(evolved)?;
    source.build_frame(&mut tcx, 480, 420);
    assert_ne!(source.actor.schema_version, schema);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("notepad.document"), slot);
    assert_eq!(source.actor.view.bindings[&input].text, "a🦀?bc notes");
    assert_eq!(
        source.actor.view.bindings[&key("notepad.summary.text")].text,
        "Untitled · 11 characters · 3 changes"
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    source.handle_event(UiEvent::TextInput("!".into()));
    source.build_frame(&mut tcx, 480, 420);
    assert_eq!(source.actor.view.bindings[&input].text, "a🦀?!bc notes");
    assert_eq!(
        source.actor.view.bindings[&key("notepad.summary.text")].text,
        "Untitled · 12 characters · 4 changes"
    );
    for (id, path) in [(206, evolved), (207, missing)] {
        let mut cold = Actor::new(id, Path::new(".runtime"))?;
        cold.reload(path)?;
        assert_eq!(cold.view.bindings[&input].text, "");
        assert_eq!(
            cold.view.bindings[&key("notepad.summary.text")].text,
            "Untitled · 0 characters · 0 changes"
        );
        let evaluations = cold.view.bindings[&input].evaluations;
        assert!(evaluations > 0);
        cold.edit_text(input, "")?;
        assert_eq!(cold.view.bindings[&input].evaluations, evaluations);
        assert_eq!(
            cold.view.bindings[&key("notepad.summary.text")].text,
            "Untitled · 0 characters · 1 change"
        );
    }
    let mut reactive = Actor::new(208, Path::new(".runtime"))?;
    reactive.reload(base)?;
    for path in [base, replacement] {
        reactive.reload(path)?;
        let evaluations = reactive.view.bindings[&input].evaluations;
        assert!(evaluations > 0);
        reactive.edit_text(input, "")?;
        assert_eq!(reactive.view.bindings[&input].evaluations, evaluations);
        assert_eq!(reactive.view.bindings[&input].text, "");
    }
    println!(
        "PASS record field reactivity: metadata-only edits do not reevaluate the direct field editor binding through real DLL replacement and evolved cold initialization"
    );
    println!(
        "PASS native record schema migration: missing upgrade rejected without disturbing the current editor; field-default upgrade retains actor/slot/editor/node/focus/Unicode selection and subsequent native edits use the new record; both new implementations initialize cold defaults"
    );
    println!(
        "PASS native record notepad: Unicode field editor writes its sole durable record atomically, real DLL replacement retains actor/slot/buffer/native editor/layout node/focus/selection, rejected edit rolls back the record and projection and current code continues"
    );
    if window {
        shell::run_window_with_options(
            Box::new(source.capture()),
            shell::WindowOptions::new()
                .title("Scratchpad")
                .initial_inner_size(640, 480),
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}
