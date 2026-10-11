use super::*;

/// Routed native interaction and real DLL identity proof for the boolean input.
pub fn checkbox_proof(paths: &[String], capture: bool) -> Result<(), String> {
    if paths.len() != 5 {
        return Err(
            "checkbox proof needs base/replacement/disabled/reject-action/invalid-slot".into(),
        );
    }
    let mut actor = Actor::new(912, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let widget = actor
        .view
        .bindings
        .iter()
        .find(|(_, b)| b.node.kind == CHECKBOX)
        .unwrap()
        .0
        .to_owned();
    let storage = actor.state_address();
    let slot = actor.slot_address("checked");
    let version = actor.implementation_version();
    assert!(
        actor
            .reload(Path::new(&paths[4]))
            .unwrap_err()
            .contains("checkbox needs a boolean state slot")
    );
    assert_eq!(actor.implementation_version(), version);
    assert_eq!(actor.slot_address("checked"), slot);
    assert!(!actor.boolean("checked"));
    assert_eq!(actor.artifact_count(), 1);
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.handle_event(UiEvent::Focus(true));
    source.build_frame(&mut tcx, 520, 240);
    let node = source.ids[&widget];
    let key_event = |name: &str, phase| {
        UiEvent::Key(shell::KeyEvent {
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            phase,
            modifiers: Default::default(),
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(key_event("Tab", shell::KeyPhase::Down));
    assert_eq!(source.focus, Some(widget));
    source.handle_event(key_event("Space", shell::KeyPhase::Down));
    assert!(!source.actor.boolean("checked"));
    source.handle_event(key_event("Space", shell::KeyPhase::Up));
    assert!(source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 1);
    source.build_frame(&mut tcx, 520, 240);
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == widget)
            .unwrap()
            .label
            .as_deref(),
        Some("Enable notifications")
    );
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == widget)
            .unwrap()
            .states
            .checked,
        Some(shell::SemanticToggleState::On)
    );
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 520, 240);
    assert_eq!(source.ids[&widget], node);
    assert_eq!(source.actor.state_address(), storage);
    assert_eq!(source.actor.slot_address("checked"), slot);
    assert_eq!(source.focus, Some(widget));
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == widget)
            .unwrap()
            .label
            .as_deref(),
        Some("Enable notifications (new code)")
    );
    assert!(source.actor.view.bindings[&widget].text.starts_with("☑"));
    assert!(
        source.actor.view.bindings[&widget]
            .text
            .contains("new code")
    );
    source.handle_event(key_event("Enter", shell::KeyPhase::Down));
    assert!(!source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 11);
    source.build_frame(&mut tcx, 520, 240);
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == widget)
            .unwrap()
            .states
            .checked,
        Some(shell::SemanticToggleState::Off)
    );
    source.replace(Path::new(&paths[3]))?;
    source.build_frame(&mut tcx, 520, 240);
    let retained_semantics = source.semantic_snapshot().unwrap();
    let retained_focus = source.focus;
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: widget }));
    assert!(!source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 11);
    assert_eq!(source.focus, retained_focus);
    assert!(Arc::ptr_eq(
        &retained_semantics,
        &source.semantic_snapshot().unwrap()
    ));
    source.build_frame(&mut tcx, 520, 240);
    assert!(Arc::ptr_eq(
        &retained_semantics,
        &source.semantic_snapshot().unwrap()
    ));
    assert!(source.actor.dispatch_widget(widget).is_err());
    assert!(!source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 11);
    source.replace(Path::new(&paths[2]))?;
    source.build_frame(&mut tcx, 520, 240);
    assert_eq!(source.focus, None);
    let disabled_semantics = source.semantic_snapshot().unwrap();
    assert!(
        disabled_semantics
            .nodes
            .iter()
            .find(|node| node.id == widget)
            .unwrap()
            .disabled
    );
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: widget }));
    assert!(!source.handle_semantic_action(shell::SemanticAction::Focus { target: widget }));
    assert!(Arc::ptr_eq(
        &disabled_semantics,
        &source.semantic_snapshot().unwrap()
    ));
    source.actor.dispatch_widget(widget)?;
    source.handle_event(key_event("Tab", shell::KeyPhase::Down));
    assert_eq!(source.focus, None);
    assert!(!source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 11);
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 520, 240);
    let placed = source.placed.iter().find(|p| p.node_id == node).unwrap();
    assert_eq!(placed.rect[3] - placed.rect[1], 36.0);
    let position = shell::Point {
        x: (placed.rect[0] + 10.0) as f64,
        y: (placed.rect[1] + 10.0) as f64,
    };
    for button in [
        shell::PointerButton::Secondary,
        shell::PointerButton::Auxiliary,
    ] {
        let focus = source.focus;
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            source.handle_event(UiEvent::Pointer(shell::PointerEvent {
                phase,
                position,
                button: Some(button),
                buttons: if phase == PointerPhase::Down {
                    if button == shell::PointerButton::Secondary {
                        4
                    } else {
                        2
                    }
                } else {
                    0
                },
                modifiers: Default::default(),
                properties: Default::default(),
            }));
        }
        assert_eq!(
            source.actor.integer("calls"),
            11,
            "non-primary click must not toggle checkbox"
        );
        assert!(!source.actor.boolean("checked"));
        assert_eq!(source.focus, focus);
    }
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        source.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            modifiers: Default::default(),
            properties: Default::default(),
        }));
    }
    assert!(source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 21);
    // Secondary events must not consume a primary press; pointer cancel must.
    let pointer = |phase, button| {
        UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button,
            buttons: 0,
            modifiers: Default::default(),
            properties: Default::default(),
        })
    };
    source.handle_event(pointer(
        PointerPhase::Down,
        Some(shell::PointerButton::Primary),
    ));
    source.handle_event(pointer(
        PointerPhase::Down,
        Some(shell::PointerButton::Secondary),
    ));
    source.handle_event(pointer(
        PointerPhase::Up,
        Some(shell::PointerButton::Secondary),
    ));
    assert_eq!(source.pressed, Some(widget));
    assert_eq!(source.actor.integer("calls"), 21);
    source.handle_event(pointer(
        PointerPhase::Up,
        Some(shell::PointerButton::Primary),
    ));
    assert!(!source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 31);
    source.handle_event(pointer(
        PointerPhase::Down,
        Some(shell::PointerButton::Primary),
    ));
    source.handle_event(pointer(PointerPhase::Cancel, None));
    assert_eq!(source.pressed, None);
    source.handle_event(pointer(
        PointerPhase::Up,
        Some(shell::PointerButton::Primary),
    ));
    assert_eq!(source.actor.integer("calls"), 31);
    // Return to checked for the optional captured final frame.
    source.actor.dispatch_widget(widget)?;
    assert!(source.actor.boolean("checked"));
    assert_eq!(source.actor.integer("calls"), 41);

    source.build_frame(&mut tcx, 520, 240);
    assert_eq!(source.ids[&widget], node);
    assert_eq!(source.actor.slot_address("checked"), slot);
    assert_eq!(source.focus, Some(widget));
    println!(
        "PASS checkbox boolean action/rollback, Tab/Space/Enter/pointer routing, disabled focus/activation, non-primary rejection/interleaving/cancel, retained state slot and native node through real DLL replacement"
    );
    {
        let parent = crate::ActorId(913);
        let child = crate::ActorId(914);
        let mut runtime = crate::Runtime::new(".runtime");
        runtime.spawn(parent, Path::new(&paths[0]))?;
        runtime.spawn_child(parent, child, Path::new(&paths[0]))?;
        let snapshot = runtime.compose(parent)?;
        let child_widget = snapshot.widget(child, widget).unwrap();
        snapshot.dispatch(&mut runtime, child_widget)?;
        assert!(runtime.actor(child).unwrap().boolean("checked"));
        assert!(!runtime.actor(parent).unwrap().boolean("checked"));
        assert_eq!(runtime.actor(parent).unwrap().integer("calls"), 0);
        let child_slot = runtime.actor(child).unwrap().slot_address("checked");
        runtime
            .actor_mut(child)
            .unwrap()
            .reload(Path::new(&paths[1]))?;
        let snapshot = runtime.compose(parent)?;
        assert_eq!(snapshot.widget(child, widget), Some(child_widget));
        snapshot.dispatch(&mut runtime, child_widget)?;
        assert!(!runtime.actor(child).unwrap().boolean("checked"));
        assert_eq!(runtime.actor(child).unwrap().integer("calls"), 11);
        assert_eq!(
            runtime.actor(child).unwrap().slot_address("checked"),
            child_slot
        );
        assert_eq!(runtime.actor(parent).unwrap().integer("calls"), 0);
        let parent_widget = snapshot.widget(parent, widget).unwrap();
        let family = crate::native_model::NativeFamily::new(runtime, parent)?;
        let mut family_source = NativeSource::without_watch(family)?;
        family_source.set_semantics_enabled(true);
        family_source.build_frame(&mut tcx, 520, 480);
        assert!(
            family_source.handle_semantic_action(shell::SemanticAction::Click {
                target: child_widget
            })
        );
        let semantics = family_source.semantic_snapshot().unwrap();
        assert_eq!(
            semantics
                .nodes
                .iter()
                .find(|node| node.id == child_widget)
                .unwrap()
                .states
                .checked,
            Some(shell::SemanticToggleState::On)
        );
        assert_eq!(
            semantics
                .nodes
                .iter()
                .find(|node| node.id == parent_widget)
                .unwrap()
                .states
                .checked,
            Some(shell::SemanticToggleState::Off)
        );
        assert_eq!(family_source.actor.root_actor().integer("calls"), 0);
        println!(
            "PASS composed checkbox routes toggles to owning child actor and current implementation with scoped widget identity"
        );
    }
    if capture {
        shell::run_window_with_options(
            Box::new(source.capture()),
            shell::WindowOptions::new()
                .title("Checkbox reload")
                .initial_inner_size(520, 240),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
