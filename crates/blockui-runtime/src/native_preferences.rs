use super::*;
pub fn proof(base: &Path, replacement: &Path, evolved: &Path, window: bool) -> Result<(), String> {
    let mut actor = Actor::new(219, Path::new(".runtime"))?;
    actor.reload(base)?;
    let name = key("preferences.name");
    let channel = key("preferences.channel");
    let volume = key("preferences.volume");
    let notifications = key("preferences.notifications");
    actor.edit_text(name, "A😀bc")?;
    actor.dispatch_widget(notifications)?;
    actor.edit_slider(volume, 35)?;
    actor.edit_select(channel, 2)?;
    let state = actor.state_address();
    let slot = actor.slot_address("preferences.model");
    let buffer = actor.text("preferences.model").as_ptr();
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.set_focus(Some(name));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 320, 560);
    source
        .editors
        .get_mut(&name)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 320, 560);
    let selection = source.editors[&name].text_selection();
    let editor = (&**source.editors.get(&name).unwrap() as *const TextInput) as usize;
    let ids = [
        name,
        channel,
        volume,
        notifications,
        key("preferences.preview"),
    ]
    .map(|key| (key, source.ids[&key]));
    let old_name_parent = source.actor.view.bindings[&name].node.parent;
    let old_volume_parent = source.actor.view.bindings[&volume].node.parent;
    source.replace(replacement)?;
    assert_ne!(
        source.actor.view.bindings[&name].node.parent,
        old_name_parent
    );
    assert_ne!(
        source.actor.view.bindings[&volume].node.parent,
        old_volume_parent
    );
    source.build_frame(&mut tcx, 320, 560);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("preferences.model"), slot);
    assert_eq!(source.actor.text("preferences.model").as_ptr(), buffer);
    assert_eq!(source.focus, Some(name));
    assert_eq!(source.editors[&name].text_selection(), selection);
    assert_eq!(source.actor.widget_boolean(notifications), Some(false));
    assert_eq!(source.actor.widget_integer(channel), Some(2));
    assert_eq!(
        source.actor.view.bindings[&key("preferences.preview")].text,
        "0.35"
    );
    let evaluations = source.actor.view.bindings[&name].evaluations;
    source.actor.edit_slider(volume, 20)?;
    assert_eq!(source.actor.widget_integer(volume), Some(25));
    assert_eq!(
        source.actor.view.bindings[&key("preferences.preview")].text,
        "0.25"
    );
    assert_eq!(source.actor.view.bindings[&name].evaluations, evaluations);
    source.build_frame(&mut tcx, 320, 560);
    assert_eq!(source.editors[&name].text_selection(), selection);
    source.handle_event(UiEvent::TextInput("🦀".into()));
    source.build_frame(&mut tcx, 320, 560);
    assert_eq!(source.actor.view.bindings[&name].text, "A🦀bc");
    let buffer = source.actor.text("preferences.model").as_ptr();
    assert!(source.actor.edit_text(name, "bad\nname").is_err());
    assert_eq!(source.actor.text("preferences.model").as_ptr(), buffer);
    let key_event = |key_name: &str| {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: key_name.into(),
            key_without_modifiers: key_name.into(),
            code: key_name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Default::default(),
            repeat: false,
            synthetic: false,
        })
    };
    source.set_focus(Some(channel));
    source.handle_event(key_event("Enter"));
    source.handle_event(key_event("ArrowUp"));
    source.build_frame(&mut tcx, 320, 560);
    assert!(source.selects[&channel].is_open());
    let active = source.selects[&channel].active();
    source.replace(evolved)?;
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("preferences.model"), slot);
    for (width, height) in [(320, 560), (480, 560)] {
        source.build_frame(&mut tcx, width, height);
        let mut bottom = 0.0;
        for name in [
            "heading",
            "scope",
            "locale.label",
            "locale",
            "name.label",
            "name",
            "notifications",
            "volume.label",
            "volume",
            "volume.value",
            "channel",
            "preview",
            "hint",
        ] {
            let widget = if matches!(name, "name.label" | "volume.label") {
                let control = key(&format!(
                    "preferences.{}",
                    name.strip_suffix(".label").unwrap()
                ));
                let parent = source.actor.view.bindings[&control].node.parent;
                *source
                    .actor
                    .view
                    .bindings
                    .iter()
                    .find(|(_, binding)| {
                        binding.node.parent == parent && binding.node.kind == blockui_abi::LABEL
                    })
                    .unwrap()
                    .0
            } else {
                key(&format!("preferences.{name}"))
            };
            let rect = source
                .widget_layout_rect(widget)
                .ok_or("missing preferences layout")?;
            assert!(rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01, "{name}");
            assert!(
                rect[1] >= bottom && rect[3] <= height as f32 + 0.01,
                "{name}: {rect:?}"
            );
            assert!(rect[2] > rect[0] && rect[3] > rect[1]);
            bottom = rect[3];
        }
    }
    assert_eq!(source.focus, Some(channel));
    assert!(source.selects[&channel].is_open());
    assert_eq!(source.selects[&channel].active(), active);
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 480, 560);
    assert_eq!(source.actor.widget_integer(channel), Some(1));
    assert_eq!(source.actor.view.bindings[&name].text, "A🦀bc");
    assert_eq!(
        source.actor.view.bindings[&key("preferences.locale")].text,
        "English"
    );
    assert_eq!(
        (&**source.editors.get(&name).unwrap() as *const TextInput) as usize,
        editor
    );
    for (key, id) in ids {
        assert_eq!(source.ids[&key], id);
    }
    println!(
        "PASS mixed preferences form: direct record controls retain values/editor/Unicode selection/focus/menu/native nodes through real code and schema replacement; current action couples slider/progress, unrelated editor skips updates, rejection preserves model, layout fits 320/480 widths"
    );
    if window {
        source.set_focus(Some(name));
        shell::run_window_with_options(
            Box::new(source.capture()),
            shell::WindowOptions::new()
                .title("Preferences")
                .initial_inner_size(320, 560),
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}
