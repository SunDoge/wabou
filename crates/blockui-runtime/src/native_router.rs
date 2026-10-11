use super::*;

pub fn proof(base: &Path, replacement: &Path) -> Result<(), String> {
    initial_route_proof(base, replacement)?;
    nested_outlet_proof(base, replacement)?;
    let mut actor = Actor::new(94, Path::new(".runtime"))?;
    actor.reload(base)?;
    if actor.slot_address("routes.scroll_rows") != 0 {
        scroll_restoration_proof(base, replacement)?;
    }
    actor.view.set_renderer_retention(32)?;
    actor.dispatch_text_request(key("routes.navigate"), "/documents/42".into())?;
    let input = key("routes.document.42.notes");
    actor.edit_text(input, "Unsaved workspace notes 🦀")?;
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.set_focus(Some(input));
    for (width, height) in [(640, 480), (320, 480)] {
        source.build_frame(&mut tcx, width, height);
        let mut previous_bottom = 0.0;
        for name in [
            "routes.home",
            "routes.open42",
            "routes.open73",
            "routes.back",
            "routes.forward",
            "routes.document.heading",
            "routes.document.42.notes",
        ] {
            let bounds = source.widget_layout_rect(key(name)).unwrap();
            assert!(
                bounds[2] > bounds[0] && bounds[3] > bounds[1],
                "empty {name}"
            );
            assert!(
                bounds[0] >= 0.0
                    && bounds[2] <= width as f32 + 0.01
                    && bounds[3] <= height as f32 + 0.01,
                "overflow {name}: {bounds:?}"
            );
            assert!(bounds[1] + 0.01 >= previous_bottom, "overlap {name}");
            previous_bottom = bounds[3];
        }
        let rect = source.widget_layout_rect(input).unwrap();
        assert!(
            rect[2] > rect[0] && rect[3] > rect[1],
            "empty input: {rect:?}"
        );
        assert!(
            rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01,
            "input overflow: {rect:?}"
        );
        let heading = source
            .widget_layout_rect(key("routes.document.heading"))
            .unwrap();
        assert!(heading[3] <= rect[1] + 0.01, "heading overlaps input");
    }
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    assert_eq!(
        source.actor.text("routes.notes"),
        "Unsaved workspace notes 🦀"
    );
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":26,"head":24}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 320, 480);
    let selection = source.editors[&input].text_selection();
    assert_eq!(
        selection.as_ref().map(|s| (s.anchor, s.head)),
        Some((26, 24))
    );
    assert!(
        source.handle_semantic_action(shell::SemanticAction::Click {
            target: key("routes.home")
        }),
        "{}",
        source.diagnostic
    );
    source.build_frame(&mut tcx, 320, 480);
    assert!(source.editors.contains_key(&input));
    assert_eq!(source.actor.text("routes.pending_navigation"), "/");
    source.replace(base)?;
    source.build_frame(&mut tcx, 320, 480);
    assert!(
        source
            .actor
            .view
            .bindings
            .contains_key(&key("routes.confirm.message"))
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: key("routes.confirm.cancel")
    }));
    assert!(source.actor.text("routes.pending_navigation").is_empty());
    assert_eq!(source.focus, Some(input), "cancel restores dialog opener");
    assert_eq!(
        source.actor.view.bindings[&key("routes.location")].text,
        "/documents/42"
    );
    assert!(
        source.handle_semantic_action(shell::SemanticAction::Click {
            target: key("routes.home")
        }),
        "{}",
        source.diagnostic
    );
    source.build_frame(&mut tcx, 320, 480);
    for name in [
        "routes.confirm.message",
        "routes.confirm.cancel",
        "routes.confirm.leave",
    ] {
        let bounds = source.widget_layout_rect(key(name)).unwrap();
        assert!(
            bounds[0] >= 0.0 && bounds[2] <= 320.01 && bounds[3] <= 480.01,
            "confirmation overflow {name}: {bounds:?}"
        );
    }
    source.replace(replacement)?;
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: key("routes.confirm.leave")
    }));
    source.build_frame(&mut tcx, 320, 480);
    assert!(source.actor.text("routes.pending_navigation").is_empty());
    assert!(!source.editors.contains_key(&input));
    assert!(!source.ids.contains_key(&input));
    assert_eq!(
        source.focus,
        Some(key("routes.home")),
        "removed opener falls back to current page navigation"
    );
    source.replace(base)?;
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: key("routes.back")
    }));
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.editors[&input].text_selection(), selection);
    // Modal confirmation owns focus. Restore editor focus explicitly, then
    // verify ordinary route unmount/remount still restores that page focus.
    source.set_focus(Some(input));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: key("routes.home")
    }));
    source.build_frame(&mut tcx, 320, 480);
    source.replace(replacement)?;
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: key("routes.back")
    }));
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    assert_eq!(
        source.actor.text("routes.notes"),
        "Unsaved workspace notes 🦀"
    );
    println!(
        "PASS router leave confirmation survives replacement; cancel stays and confirm navigates without discarding notes"
    );
    println!(
        "PASS native router outlet remount restores reversed Unicode selection and focus after hidden-page DLL replacement"
    );
    println!(
        "PASS native router layout at 640/320 widths; editor instance, node and draft survive DLL replacement"
    );
    let mut keyboard_actor = Actor::new(95, Path::new(".runtime"))?;
    keyboard_actor.reload(base)?;
    let mut keyboard = NativeSource::without_watch(keyboard_actor)?;
    keyboard.build_frame(&mut tcx, 320, 480);
    let key_event = |name: &str, repeat| {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: shell::Modifiers::empty(),
            repeat,
            synthetic: false,
        })
    };
    keyboard.set_focus(None);
    for name in [
        "routes.home",
        "routes.open42",
        "routes.open73",
        "routes.home",
        "routes.open42",
    ] {
        assert!(keyboard.handle_event(key_event("Tab", false)).handled);
        assert_eq!(keyboard.focus, Some(key(name)));
    }
    // Repeated Enter must not trigger navigation. Disabled history controls
    // must be absent from the Tab sequence before the first navigation.
    keyboard.handle_event(key_event("Enter", true));
    assert_eq!(
        keyboard.actor.view.bindings[&key("routes.location")].text,
        "/"
    );
    assert!(keyboard.handle_event(key_event("Enter", false)).handled);
    keyboard.build_frame(&mut tcx, 320, 480);
    assert_eq!(
        keyboard.actor.view.bindings[&key("routes.location")].text,
        "/documents/42"
    );
    keyboard.replace(replacement)?;
    keyboard.build_frame(&mut tcx, 320, 480);
    assert_eq!(keyboard.focus, Some(key("routes.open42")));
    println!(
        "PASS router keyboard Tab skips disabled history; Enter navigates once and focused navigation survives replacement"
    );
    Ok(())
}

fn initial_route_proof(base: &Path, replacement: &Path) -> Result<(), String> {
    use crate::{ActorId, App};
    assert!(
        App::new(base, ".runtime")
            .initial_request(0, "invalid")
            .is_err()
    );
    assert!(
        App::new(base, ".runtime")
            .initial_request(key("routes.navigate"), "/documents/042")?
            .into_window()
            .is_err()
    );
    let (mut source, _) = App::new(base, ".runtime")
        .initial_request(key("routes.navigate"), "/documents/42")?
        .initial_request(key("routes.navigate"), "/documents/73")?
        .into_window()?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 320, 640);
    let root = ActorId(1);
    assert_eq!(
        source.actor.root_actor().view.bindings[&key("routes.location")].text,
        "/documents/73"
    );
    assert_eq!(
        source.actor.root_actor().text_list("routes.paths"),
        &["2", "/", "/documents/42", "/documents/73"]
    );
    let input = source
        .actor
        .widget(root, key("routes.document.73.notes"))
        .unwrap();
    assert!(source.editors.contains_key(&input));
    let state = source.actor.root_actor().state_address();
    let open = source.actor.widget(root, key("routes.open42")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: open }));
    source.build_frame(&mut tcx, 320, 640);
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 640, 640);
    assert_eq!(source.actor.root_actor().state_address(), state);
    assert_eq!(
        source.actor.root_actor().view.bindings[&key("routes.location")].text,
        "/documents/42"
    );
    println!(
        "PASS App initial route requests run before the first native frame in order; invalid routes reject assembly and hot replacement does not replay startup navigation"
    );
    Ok(())
}

fn nested_outlet_proof(base: &Path, replacement: &Path) -> Result<(), String> {
    let mut actor = Actor::new(198, Path::new(".runtime"))?;
    actor.reload(base)?;
    actor.view.set_renderer_retention(64)?;
    actor.dispatch(Message {
        action: key("routes.open"),
        value: 42,
    })?;
    let state = actor.state_address();
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 320, 640);
    let shell = key("routes.documents.shell");
    let shell_node = source.ids[&shell];
    for (id, anchor, head) in [(42, 4, 2), (73, 7, 1)] {
        source.persist_native_renderer();
        source.actor.dispatch(Message {
            action: key("routes.open"),
            value: id,
        })?;
        source.build_frame(&mut tcx, 320, 640);
        let input = key(&format!("routes.document.{id}.notes"));
        source.actor.edit_text(input, &format!("Draft {id} 🦀"))?;
        source.build_frame(&mut tcx, 320, 640);
        source.set_focus(Some(input));
        source
            .editors
            .get_mut(&input)
            .unwrap()
            .config_changed(&format!(
                r#"{{"selection":{{"anchor":{anchor},"head":{head}}}}}"#
            ))
            .unwrap();
        source.build_frame(&mut tcx, 320, 640);
        assert_eq!(source.ids[&shell], shell_node);
    }
    source.replace(replacement)?;
    for (id, anchor, head) in [(42, 4, 2), (73, 7, 1)] {
        source.persist_native_renderer();
        source.actor.dispatch(Message {
            action: key("routes.open"),
            value: id,
        })?;
        source.build_frame(&mut tcx, 640, 640);
        let input = key(&format!("routes.document.{id}.notes"));
        assert_eq!(source.focus, Some(input));
        assert_eq!(
            source.editors[&input]
                .text_selection()
                .map(|s| (s.anchor, s.head)),
            Some((anchor, head))
        );
        assert_eq!(source.actor.text("routes.notes"), format!("Draft {id} 🦀"));
        assert_eq!(source.ids[&shell], shell_node);
        assert_eq!(source.actor.state_address(), state);
    }
    println!(
        "PASS nested router outlets preserve the parent node and independent child drafts, focus and selections across real DLL replacement"
    );
    Ok(())
}

/// The source path must belong to the verifier's disposable fixture crate.
pub fn watch_proof(config: ReloadConfig, editable: &Path) -> Result<(), String> {
    let original = std::fs::read_to_string(editable).map_err(|e| e.to_string())?;
    let mut actor = Actor::new(97, Path::new(".runtime"))?;
    actor.reload(&config.artifact)?;
    actor.dispatch_text_request(key("routes.navigate"), "/documents/42".into())?;
    let input = key("routes.document.42.notes");
    actor.edit_text(input, "Unsaved workspace notes 🦀")?;
    let state = actor.state_address();
    let slot = actor.slot_address("routes.notes");
    let version = actor.implementation_version();
    let mut source = NativeSource::new(actor, config)?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while !source
        .watcher
        .as_ref()
        .unwrap()
        .ready
        .load(Ordering::Acquire)
    {
        if Instant::now() > deadline {
            return Err("watcher startup timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 320, 480);
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":26,"head":24}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 320, 480);
    let selection = source.editors[&input].text_selection();
    assert_eq!(
        selection.as_ref().map(|s| (s.anchor, s.head)),
        Some((26, 24))
    );
    let node = source.ids[&input];
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    std::fs::write(
        editable,
        format!("{original}\nfn broken() {{ let invalid: = ; }}\n"),
    )
    .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while source.diagnostic.is_empty() {
        source.poll_async();
        if Instant::now() > deadline {
            return Err("failed build timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(source.actor.implementation_version(), version);
    // The new external entry point still runs through the old implementation
    // during a failed build. Same-page navigation must preserve dirty state.
    source
        .actor
        .dispatch_text_request(key("routes.request_path"), "/documents/42".into())?;
    source.sync()?;
    source.build_frame(&mut tcx, 320, 480);
    assert!(source.actor.boolean("routes.dirty"));
    assert!(source.actor.text("routes.pending_navigation").is_empty());
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    // Old implementation still processes a normal action after compilation fails.
    source.actor.dispatch(Message {
        action: key("routes.dirty"),
        value: 0,
    })?;
    source.sync()?;
    let updated = original.replace("Document {id}", "Updated document {id}");
    assert_ne!(updated, original);
    let saved = Instant::now();
    std::fs::write(editable, updated).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while source.actor.implementation_version() == version {
        source.poll_async();
        if Instant::now() > deadline {
            return Err(format!("replacement timed out: {}", source.diagnostic));
        }
        thread::sleep(Duration::from_millis(10));
    }
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("routes.notes"), slot);
    assert_eq!(
        source.actor.text("routes.notes"),
        "Unsaved workspace notes 🦀"
    );
    assert_eq!(
        source.actor.view.bindings[&key("routes.document.heading")].text,
        "Updated document 42"
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    println!(
        "PASS router real source watcher: failed compile keeps old actions usable; repaired source updates heading and retains state/editor/selection/focus; save_to_native_layout_ms={}",
        saved.elapsed().as_millis()
    );
    // Measure repeated real watcher builds through a completed native layout,
    // keeping the same dirty document and native editor throughout.
    let mut latency_samples = Vec::new();
    for sample in 1..=6 {
        let previous = source.actor.implementation_version();
        let changed = original.replace(
            "Document {id}",
            &format!("Revision {sample} document {{id}}"),
        );
        std::fs::write(editable, changed).map_err(|e| e.to_string())?;
        let saved = Instant::now();
        let deadline = saved + Duration::from_secs(30);
        while source.actor.implementation_version() == previous {
            source.poll_async();
            if Instant::now() > deadline {
                return Err(format!(
                    "latency sample {sample} timed out: {}",
                    source.diagnostic
                ));
            }
            thread::sleep(Duration::from_millis(2));
        }
        source.build_frame(&mut tcx, 320, 480);
        latency_samples.push(saved.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(source.actor.state_address(), state);
        assert_eq!(source.actor.slot_address("routes.notes"), slot);
        assert_eq!(
            source.actor.text("routes.notes"),
            "Unsaved workspace notes 🦀"
        );
        assert_eq!(source.ids[&input], node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            editor
        );
        assert_eq!(source.editors[&input].text_selection(), selection);
        assert_eq!(source.focus, Some(input));
        assert_eq!(
            source.actor.view.bindings[&key("routes.document.heading")].text,
            format!("Revision {sample} document 42")
        );
    }
    println!(
        "PASS repeated watcher save-to-native-layout samples_ms={}",
        latency_samples
            .iter()
            .map(|value| format!("{value:.3}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    source.set_semantics_enabled(true);
    source.actor.dispatch(Message {
        action: key("routes.dirty"),
        value: 1,
    })?;
    source.sync()?;
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: key("routes.home")
    }));
    source.build_frame(&mut tcx, 320, 480);
    let pending_slot = source.actor.slot_address("routes.pending_navigation");
    let dialog_node = source.ids[&key("routes.confirm.dialog")];
    let confirmation_version = source.actor.implementation_version();
    source.diagnostic.clear();
    let current_source = std::fs::read_to_string(editable).map_err(|e| e.to_string())?;
    std::fs::write(
        editable,
        format!("{current_source}\nfn broken_confirmation() {{ let invalid: = ; }}\n"),
    )
    .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while source.diagnostic.is_empty() {
        source.poll_async();
        if Instant::now() > deadline {
            return Err("confirmation compile failure timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(source.actor.implementation_version(), confirmation_version);
    assert_eq!(source.actor.text("routes.pending_navigation"), "/");
    let revised = current_source.replace("Leave this page?", "Leave updated page?");
    assert_ne!(current_source, revised);
    std::fs::write(editable, revised).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while source.actor.implementation_version() == confirmation_version {
        source.poll_async();
        if Instant::now() > deadline {
            return Err(format!(
                "confirmation reload timed out: {}",
                source.diagnostic
            ));
        }
        thread::sleep(Duration::from_millis(10));
    }
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(
        source.actor.slot_address("routes.pending_navigation"),
        pending_slot
    );
    assert_eq!(source.actor.text("routes.pending_navigation"), "/");
    assert_eq!(source.ids[&key("routes.confirm.dialog")], dialog_node);
    assert_eq!(
        source.actor.view.bindings[&key("routes.confirm.message")].text,
        "Leave updated page? Document drafts will be kept."
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: key("routes.confirm.cancel")
    }));
    source.build_frame(&mut tcx, 320, 480);
    assert!(source.actor.text("routes.pending_navigation").is_empty());
    assert_eq!(source.focus, Some(input));
    assert_eq!(
        source.actor.text("routes.notes"),
        "Unsaved workspace notes 🦀"
    );
    println!(
        "PASS pending confirmation survives actual failed/repaired source builds; dialog node and pending slot retained, current cancel restores editor focus"
    );
    Ok(())
}

pub fn recovery_layout_proof(actor: Actor) -> Result<(), String> {
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    for width in [640, 320] {
        source.build_frame(&mut tcx, width, 480);
        let rect = source
            .widget_layout_rect(key("routes.home.recovered"))
            .unwrap();
        assert!(rect[2] > rect[0] && rect[3] > rect[1]);
        assert!(
            rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01 && rect[3] <= 480.01,
            "recovered editor overflow: {rect:?}"
        );
    }
    println!("PASS recovered Home editor native layout at 640/320 widths");
    Ok(())
}

fn scroll_restoration_proof(base: &Path, replacement: &Path) -> Result<(), String> {
    let mut actor = Actor::new(197, Path::new(".runtime"))?;
    actor.reload(base)?;
    actor.view.set_renderer_retention(256)?;
    actor.dispatch_text_request(key("routes.navigate"), "/documents/42".into())?;
    let storage = actor.state_address();
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let first = key("routes.document.42.scroll");
    let second = key("routes.document.73.scroll");
    for (document, scroll, offset) in [(42, first, 137.0), (73, second, 61.0)] {
        source
            .actor
            .dispatch_text_request(key("routes.navigate"), format!("/documents/{document}"))?;
        source.build_frame(&mut tcx, 320, 640);
        let rect = source.widget_layout_rect(scroll).unwrap();
        assert!(
            rect[0] >= 0.0 && rect[2] <= 320.01 && rect[3] <= 640.01,
            "scroll port overflow: {rect:?}"
        );
        assert!(
            source
                .tree
                .layout(source.ids[&scroll])
                .unwrap()
                .scroll_height()
                > offset as f32
        );
        source.handle_event(UiEvent::Wheel(shell::WheelEvent {
            position: shell::Point {
                x: f64::from(rect[0] + 8.0),
                y: f64::from(rect[1] + 8.0),
            },
            delta_x: 0.0,
            delta_y: offset,
            delta_mode: shell::WheelDeltaMode::Pixel,
            phase: shell::GesturePhase::Changed,
            modifiers: Default::default(),
        }));
        source.build_frame(&mut tcx, 320, 640);
        assert_eq!(source.actor.view.renderer[&scroll].scroll, offset);
    }
    source.replace(replacement)?;
    for (document, scroll, offset) in [(42, first, 137.0), (73, second, 61.0)] {
        source
            .actor
            .dispatch_text_request(key("routes.navigate"), format!("/documents/{document}"))?;
        source.build_frame(&mut tcx, 640, 640);
        assert_eq!(source.actor.view.renderer[&scroll].scroll, offset);
        assert_eq!(source.actor.state_address(), storage);
    }
    // Shrink the hidden page before returning. Restored scroll must be admitted
    // against the new geometry, rather than displaying an empty overscrolled page.
    source.actor.dispatch(Message {
        action: key("routes.scroll_rows"),
        value: 1,
    })?;
    source
        .actor
        .dispatch_text_request(key("routes.navigate"), "/documents/42".into())?;
    source.build_frame(&mut tcx, 320, 640);
    assert_eq!(source.actor.view.renderer[&first].scroll, 0.0);
    source.actor.view.set_renderer_retention(0)?;
    source
        .actor
        .dispatch_text_request(key("routes.navigate"), "/documents/73".into())?;
    source.build_frame(&mut tcx, 320, 640);
    assert_eq!(source.actor.view.renderer[&second].scroll, 0.0);
    assert_eq!(source.actor.view.retained_renderer_count(), 0);
    println!(
        "PASS route scroll restoration retains independent wheel offsets through hidden-page DLL replacement and resize; shorter content clamps restored offsets and disabling retention clears cache"
    );
    Ok(())
}
