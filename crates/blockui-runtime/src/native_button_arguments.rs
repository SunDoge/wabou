use super::*;

/// Real DLL replacement and native event routing; no OS input synthesis.
pub fn proof(paths: &[String]) -> Result<(), String> {
    if !matches!(paths.len(), 5 | 7) {
        return Err(
            "button arguments proof needs base/replacement/disabled/invalid/domain-error DLLs and optional invalid-variant/non-button-variant DLLs"
                .into(),
        );
    }
    let mut actor = Actor::new(913, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let storage = actor.state_address();
    let slot = actor.slot_address("amount");
    let widget = blockui_abi::key("amount.button");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.handle_event(UiEvent::Focus(true));
    source.build_frame(&mut tcx, 520, 240);
    let node = source.ids[&widget];
    let key = |name: &str| {
        UiEvent::Key(shell::KeyEvent {
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            phase: shell::KeyPhase::Down,
            modifiers: Default::default(),
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(key("Tab"));
    assert_eq!(source.focus, Some(widget));
    source.handle_event(key("Enter"));
    assert_eq!(source.actor.float("amount"), 0.25);
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 520, 240);
    assert_eq!(source.ids[&widget], node);
    assert_eq!(source.focus, Some(widget));
    assert_eq!(source.actor.state_address(), storage);
    assert_eq!(source.actor.slot_address("amount"), slot);
    assert_eq!(source.actor.float("amount"), 0.25);
    source.handle_event(key("Enter"));
    assert_eq!(source.actor.float("amount"), 10.75);
    source.build_frame(&mut tcx, 520, 240);
    let version = source.actor.implementation_version();
    let snapshot = source.semantic_snapshot().unwrap();
    assert!(
        source
            .replace(Path::new(&paths[3]))
            .unwrap_err()
            .contains("action payload requires a button")
    );
    assert_eq!(source.actor.implementation_version(), version);
    assert_eq!(source.actor.float("amount"), 10.75);
    assert_eq!(source.actor.state_address(), storage);
    assert_eq!(source.actor.slot_address("amount"), slot);
    assert_eq!(source.ids[&widget], node);
    assert_eq!(source.focus, Some(widget));
    assert!(Arc::ptr_eq(&snapshot, &source.semantic_snapshot().unwrap()));
    assert_eq!(source.actor.artifact_count(), 1);
    for rejected in paths.iter().skip(5) {
        let snapshot = source.semantic_snapshot().unwrap();
        assert!(
            source
                .replace(Path::new(rejected))
                .unwrap_err()
                .contains("invalid button appearance")
        );
        assert_eq!(source.actor.implementation_version(), version);
        assert_eq!(source.actor.state_address(), storage);
        assert_eq!(source.actor.slot_address("amount"), slot);
        assert_eq!(source.actor.float("amount"), 10.75);
        assert_eq!(source.ids[&widget], node);
        assert_eq!(source.focus, Some(widget));
        assert!(Arc::ptr_eq(&snapshot, &source.semantic_snapshot().unwrap()));
        assert_eq!(source.actor.artifact_count(), 1);
        // Rejection leaves the previous implementation callable.
        source.handle_event(key("Enter"));
        assert_eq!(source.actor.float("amount"), 10.75);
    }
    let placed = source.placed.iter().find(|p| p.node_id == node).unwrap();
    let position = shell::Point {
        x: (placed.rect[0] + 10.0) as f64,
        y: (placed.rect[1] + 10.0) as f64,
    };
    source.actor.dispatch(Message {
        action: source.actor.view.bindings[&widget].node.action,
        value: 0.0_f64.to_bits() as i64,
    })?;
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
    assert_eq!(source.actor.float("amount"), 10.75);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: blockui_abi::key("item.button")
    }));
    assert_eq!(source.actor.integer("item"), -42);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: blockui_abi::key("enable.button")
    }));
    assert!(source.actor.boolean("enabled"));
    source.replace(Path::new(&paths[4]))?;
    source.build_frame(&mut tcx, 520, 240);
    let failed_snapshot = source.semantic_snapshot().unwrap();
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: widget }));
    assert_eq!(
        source.actor.float("amount"),
        10.75,
        "domain rejection must undo the attempted 0.25 write"
    );
    assert_eq!(source.actor.slot_address("amount"), slot);
    assert!(Arc::ptr_eq(
        &failed_snapshot,
        &source.semantic_snapshot().unwrap()
    ));
    let owned_error = source.actor.dispatch_widget(widget).unwrap_err();
    assert_eq!(owned_error, "Preset rejected by application policy");
    source.replace(Path::new(&paths[2]))?;
    assert_eq!(
        owned_error, "Preset rejected by application policy",
        "diagnostic must outlive its implementation library"
    );
    source.build_frame(&mut tcx, 520, 240);
    assert_eq!(source.ids[&widget], node);
    assert_eq!(source.focus, None);
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: widget }));
    source.actor.dispatch_widget(widget)?;
    assert_eq!(source.actor.float("amount"), 10.75);
    // Dynamic rows retain identity through reordering and actual implementation swaps.
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 520, 640);
    let row = source
        .actor
        .view
        .bindings
        .values()
        .find(|binding| binding.node.kind == BUTTON && binding.node.action_payload == 7)
        .unwrap()
        .node
        .key;
    let row_node = source.ids[&row];
    let select_action = source.actor.view.bindings[&row].node.action;
    let initial_y = source
        .placed
        .iter()
        .find(|placed| placed.node_id == row_node)
        .unwrap()
        .rect[1];
    let items_slot = source.actor.slot_address("items");
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: row }));
    source.handle_event(key("Enter"));
    assert_eq!(source.actor.integer("selected"), 7);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: blockui_abi::key("reverse.button")
    }));
    source.build_frame(&mut tcx, 520, 640);
    assert_eq!(source.actor.list("items"), &[99, 42, 7]);
    assert_eq!(source.ids[&row], row_node);
    let reordered_y = source
        .placed
        .iter()
        .find(|placed| placed.node_id == row_node)
        .unwrap()
        .rect[1];
    assert!(
        reordered_y > initial_y,
        "selected row must actually change position"
    );
    // Semantic clicks do not move focus; the selected row stays focused.
    assert_eq!(source.focus, Some(row));
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 520, 640);
    assert_eq!(source.actor.slot_address("items"), items_slot);
    assert_eq!(source.actor.integer("selected"), 7);
    assert_eq!(source.actor.list("items"), &[99, 42, 7]);
    assert_eq!(source.ids[&row], row_node);
    assert_eq!(source.focus, Some(row));
    source.handle_event(key("Enter"));
    assert_eq!(source.actor.integer("selected"), 7);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: blockui_abi::key("remove.button")
    }));
    source.build_frame(&mut tcx, 520, 640);
    assert_eq!(source.actor.list("items"), &[99, 42]);
    assert!(!source.ids.contains_key(&row));
    assert_eq!(source.focus, None);
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: row }));
    assert_eq!(source.actor.integer("selected"), 0);
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Click {
            target: blockui_abi::key("remove.button")
        })
    );
    // A queued data message for a removed row is ignored by the current model.
    source.actor.dispatch(Message {
        action: select_action,
        value: 7,
    })?;
    assert_eq!(source.actor.integer("selected"), 0);
    for value in [99, 42] {
        let row = source
            .actor
            .view
            .bindings
            .values()
            .find(|binding| binding.node.kind == BUTTON && binding.node.action_payload == value)
            .unwrap()
            .node
            .key;
        assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: row }));
        assert_eq!(source.actor.integer("selected"), value);
        assert!(source.handle_semantic_action(shell::SemanticAction::Click {
            target: blockui_abi::key("remove.button")
        }));
        source.build_frame(&mut tcx, 520, 640);
    }
    assert!(source.actor.list("items").is_empty());
    assert_eq!(source.actor.integer("selected"), 0);
    assert_eq!(
        source.actor.view.bindings[&blockui_abi::key("items.empty")].text,
        "No items"
    );
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Click {
            target: blockui_abi::key("remove.button")
        })
    );
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 520, 640);
    assert!(
        source.actor.list("items").is_empty(),
        "reload must not restore defaults over an empty list"
    );
    assert_eq!(source.actor.integer("selected"), 0);
    assert_eq!(source.actor.slot_address("items"), items_slot);
    // Recover from the empty state without resetting the identity allocator.
    let next_slot = source.actor.slot_address("next_id");
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: blockui_abi::key("add.button")
    }));
    source.build_frame(&mut tcx, 520, 640);
    assert_eq!(source.actor.list("items"), &[100]);
    assert_eq!(source.actor.integer("selected"), 100);
    assert_eq!(source.actor.integer("next_id"), 101);
    assert!(
        !source
            .actor
            .view
            .bindings
            .contains_key(&blockui_abi::key("items.empty"))
    );
    let added = source
        .actor
        .view
        .bindings
        .values()
        .find(|binding| binding.node.kind == BUTTON && binding.node.action_payload == 100)
        .unwrap()
        .node
        .key;
    let added_node = source.ids[&added];
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: added }));
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 520, 640);
    assert_eq!(source.focus, Some(added));
    assert_eq!(source.ids[&added], added_node);
    assert_eq!(source.actor.slot_address("next_id"), next_slot);
    assert_eq!(source.actor.integer("next_id"), 101);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: blockui_abi::key("remove.button")
    }));
    source.build_frame(&mut tcx, 520, 640);
    assert!(source.actor.list("items").is_empty());
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 520, 640);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: blockui_abi::key("add.button")
    }));
    source.build_frame(&mut tcx, 520, 640);
    assert_eq!(source.actor.list("items"), &[101]);
    assert_eq!(source.actor.integer("selected"), 101);
    assert_eq!(source.actor.integer("next_id"), 102);
    assert!(!source.ids.contains_key(&added));
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: added }));
    source.actor.dispatch(Message {
        action: select_action,
        value: 100,
    })?;
    assert_eq!(source.actor.integer("selected"), 101);
    // Growth stays inside the list viewport; toolbar controls remain fixed.
    let add = blockui_abi::key("add.button");
    let add_node = source.ids[&add];
    let toolbar_rect = source
        .placed
        .iter()
        .find(|placed| placed.node_id == add_node)
        .unwrap()
        .rect;
    for _ in 0..8 {
        assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: add }));
    }
    source.build_frame(&mut tcx, 520, 640);
    let scroll = blockui_abi::key("items.scroll");
    let scroll_node = source.ids[&scroll];
    let placed = source
        .placed
        .iter()
        .find(|placed| placed.node_id == scroll_node)
        .unwrap();
    assert!(placed.scroll.range[1] > 0.0);
    assert_eq!(
        source
            .placed
            .iter()
            .find(|placed| placed.node_id == add_node)
            .unwrap()
            .rect,
        toolbar_rect
    );
    let position = shell::Point {
        x: f64::from(placed.rect[0] + 10.0),
        y: f64::from(placed.rect[1] + 10.0),
    };
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position,
        delta_x: 0.0,
        delta_y: 80.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 520, 640);
    let offset = source.actor.view.renderer[&scroll].scroll;
    assert!(offset > 0.0);
    let selected = source.actor.integer("selected");
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 520, 640);
    assert_eq!(source.ids[&scroll], scroll_node);
    assert_eq!(source.actor.view.renderer[&scroll].scroll, offset);
    assert_eq!(source.actor.integer("selected"), selected);
    assert_eq!(
        source
            .placed
            .iter()
            .find(|placed| placed.node_id == add_node)
            .unwrap()
            .rect,
        toolbar_rect
    );
    source.build_frame(&mut tcx, 320, 480);
    let narrow_scroll = source
        .placed
        .iter()
        .find(|placed| placed.node_id == scroll_node)
        .unwrap();
    assert!(
        narrow_scroll.rect[3] <= 480.0,
        "list viewport must fit the declared small fixture"
    );
    assert_eq!(source.actor.view.renderer[&scroll].scroll, offset);
    assert_eq!(source.actor.artifact_count(), 1);
    println!(
        "PASS typed button DLL arguments: keyboard, pointer, semantics, current handler/payload, stable state/widget/focus, disabled, malformed payload/appearance candidate rollback, keyed row reorder/remove/reload, stale selection, empty-state reload, monotonic row IDs, scroll retention with fixed toolbar, application error rollback"
    );
    Ok(())
}
