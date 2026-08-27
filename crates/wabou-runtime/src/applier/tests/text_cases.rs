use super::*;

#[test]
fn password_input_keeps_secret_out_of_attrs_and_js_events() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let secrets = crate::SecretStore::default();
    let mut factories = builtin_factories();
    let factory_secrets = secrets.clone();
    factories.insert(
        "password-input".into(),
        Arc::new(move || Box::new(crate::PasswordInput::new(factory_secrets.clone()))),
    );
    let mut applier = Applier::from_runtime_with_factories(js, factories, Color::BLACK);
    let (tag, secret, value, aria_value_text) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("password-input"),
            atoms.intern("secret"),
            atoms.intern("value"),
            atoms.intern("aria-valuetext"),
        )
    };
    create_element_with_attrs(
        &mut applier,
        2,
        tag,
        &[
            (secret, "master-password"),
            (aria_value_text, "must-not-leak"),
        ],
    );
    set_focus_order(&mut applier, 2, 0);
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::INPUT,
    });
    let mut tcx = TextContext::new();
    applier.build_frame(&mut tcx, 800, 600);
    applier.handle_event(pointer(PointerPhase::Down, 10.0, 10.0, 1));
    assert!(
        applier
            .handle_event(UiEvent::TextInput("hunter2".into()))
            .handled
    );
    applier.build_frame(&mut tcx, 800, 600);

    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert!(
        applier.document.widget_manager.widgets[&node]
            .current_value()
            .is_none()
    );
    assert!(
        !applier.document.node_store.declared[&node]
            .attrs
            .contains_key(&value)
    );
    assert_eq!(
        applier
            .runtime
            .js
            .with(|ctx| ctx.eval::<usize, _>("globalThis.dispatched.length"))
            .unwrap(),
        0
    );
    assert_eq!(secrets.take("master-password").as_str(), "hunter2");
    assert_eq!(
        applier
            .frame
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .find(|node| node.id == sk(2))
            .and_then(|node| node.value.as_deref()),
        None
    );
}

#[test]
fn text_input_updates_value_paints_and_dispatches_input() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            r#"
            // No app bundle is booted in this test; stub the rAF entry
            // points so build_frame's `js.tick()` runs without error.
            globalThis.__wabou_tick = () => false;
            globalThis.__wabou_has_raf = () => false;
            "#,
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (input, value, width) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("input"),
            atoms.intern("value"),
            atoms.intern("width"),
        )
    };
    create_element_with_attrs(&mut applier, 2, input, &[(value, "")]);
    set_focus_order(&mut applier, 2, 0);
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: width,
        value: "200px",
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::INPUT,
    });
    let mut tcx = TextContext::new();
    // build_frame computes layout + paints widgets + drains value sync.
    applier.build_frame(&mut tcx, 800, 600);
    let focus = applier.handle_event(pointer(PointerPhase::Down, 10.0, 10.0, 1));
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert!(
        applier
            .document
            .node_store
            .tree
            .layout(node)
            .unwrap()
            .size
            .height
            > 0.0
    );
    assert_eq!(focus.text_input, Some(true));
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(2, 1))
    );

    // Widgets receive the complete captured pointer stream, including
    // moves/releases outside their hit-test bounds. Text selection relies
    // on this just like native controls do.
    assert!(
        applier
            .handle_event(pointer(PointerPhase::Move, 400.0, 10.0, 1))
            .handled
    );
    assert!(
        applier
            .handle_event(pointer(PointerPhase::Up, 400.0, 10.0, 0))
            .handled
    );
    assert!(applier.interaction.input.pointer_down_target.is_none());

    // Text edits resolve at paint (pending-edit pattern: handle_event has
    // no FontContext), so the value sync + INPUT dispatch are deferred to
    // build_frame. Drive one frame to apply + dispatch.
    assert!(
        applier
            .handle_event(UiEvent::TextInput("ab".into()))
            .handled
    );
    applier.build_frame(&mut tcx, 800, 600);
    assert_eq!(
        applier.document.widget_manager.widgets[&node].current_value(),
        Some("ab")
    );
    assert_eq!(
        applier.document.node_store.declared[&node].attrs[&value].as_ref(),
        "ab"
    );
    let payload = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<String, _>("globalThis.dispatched[0][2]"))
        .unwrap();
    assert_eq!(payload, r#"{"value":"ab"}"#);

    applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "Backspace".into(),
        key_without_modifiers: "Backspace".into(),
        code: "Backspace".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Modifiers::default(),
        repeat: false,
        synthetic: false,
    }));
    applier.build_frame(&mut tcx, 800, 600);
    assert_eq!(
        applier.document.widget_manager.widgets[&node].current_value(),
        Some("a")
    );
    assert_eq!(
        applier.document.node_store.declared[&node].attrs[&value].as_ref(),
        "a"
    );

    for _ in 0..2 {
        applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
            phase: KeyPhase::Down,
            key: "Backspace".into(),
            key_without_modifiers: "Backspace".into(),
            code: "Backspace".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Modifiers::default(),
            repeat: false,
            synthetic: false,
        }));
    }
    applier.build_frame(&mut tcx, 800, 600);
    assert_eq!(
        applier.document.widget_manager.widgets[&node].current_value(),
        Some("")
    );
    assert_eq!(
        applier.document.node_store.declared[&node].attrs[&value].as_ref(),
        ""
    );
    // A control char (backspace as text) is filtered out → IGNORED, no
    // value sync, handled stays false.
    assert!(
        !applier
            .handle_event(UiEvent::TextInput("\u{8}".into()))
            .handled
    );
    applier.build_frame(&mut tcx, 800, 600);
    assert_eq!(
        applier.document.widget_manager.widgets[&node].current_value(),
        Some("")
    );
}

#[test]
fn code_editor_drag_selection_survives_native_pointer_routing() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (tag, value, width, height) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("code-editor"),
            atoms.intern("value"),
            atoms.intern("width"),
            atoms.intern("height"),
        )
    };
    create_element_with_attrs(&mut applier, 2, tag, &[(value, "abcdef")]);
    set_focus_order(&mut applier, 2, 0);
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::TEXTSELECTIONCHANGE,
    });
    for (prop, value) in [(width, "320px"), (height, "120px")] {
        applier.apply_op(&Op::SetStyle {
            id: NodeKey::new(2, 1),
            prop,
            value,
        });
    }

    let mut tcx = TextContext::new();
    applier.build_frame(&mut tcx, 800, 600);
    // The editor gutter is 58px and text inset is 10px. Drag beyond the
    // visible end, as users naturally do when selecting a complete line.
    applier.handle_event(pointer(PointerPhase::Down, 68.0, 10.0, 1));
    applier.handle_event(pointer(PointerPhase::Move, 300.0, 10.0, 1));
    applier.handle_event(pointer(PointerPhase::Up, 300.0, 10.0, 0));
    assert!(
        !applier.handle_event(UiEvent::TextInput("X".into())).handled,
        "the controlled viewport must not edit outside CodeMirror"
    );
    applier.build_frame(&mut tcx, 800, 600);

    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert_eq!(
        applier.document.widget_manager.widgets[&node].current_value(),
        None
    );
    let selection_event = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(format!(
                "JSON.stringify(globalThis.dispatched.filter((event) => event[1] === {}).at(-1))",
                event::TEXTSELECTIONCHANGE
            ))
        })
        .expect("native widget selection event");
    let event: serde_json::Value = serde_json::from_str(&selection_event).unwrap();
    let payload: serde_json::Value = serde_json::from_str(event[2].as_str().unwrap()).unwrap();
    assert_eq!(payload["anchor"], 0);
    assert_eq!(payload["head"], 6);
    assert_eq!(payload["text"], "abcdef");
}

#[test]
fn text_only_element_uses_one_parley_leaf_instead_of_text_boxes() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    set_text_behavior(&mut applier, 2);
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(3, 1),
        text: "Hello ",
    });
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(4, 1),
        text: "world",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(4, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.rebuild_layout_boxes();

    let parent = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert_eq!(applier.document.node_store.tree.child_count(parent), 0);
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .get_node_context(parent)
            .unwrap()
            .text
            .as_deref(),
        Some("Hello world")
    );

    applier.apply_op(&Op::SetText {
        id: NodeKey::new(4, 1),
        text: "Wabou",
    });
    applier.rebuild_layout_boxes();
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .get_node_context(parent)
            .unwrap()
            .text
            .as_deref(),
        Some("Hello Wabou")
    );
}

#[test]
fn ordinary_text_drag_selects_highlights_and_copies() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    set_text_behavior(&mut applier, 2);
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(3, 1),
        text: "selectable text",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::TEXTSELECTIONCHANGE,
    });
    applier.rebuild_layout_boxes();
    applier.inherit();

    let mut tcx = TextContext::new();
    let mut placed = layout::compute_and_walk_with_scroll(
        &mut applier.document.node_store.tree,
        applier.document.node_store.root,
        400.0,
        100.0,
        &mut tcx,
        &HashMap::new(),
    );
    applier.rebuild_hit_geometry(&placed);
    applier.prepare_text_selection(&mut placed, &mut tcx);
    let origin = applier.interaction.text_selection.selectable[&NodeKey::new(2, 1)].origin;

    assert!(
        applier
            .handle_event(pointer(
                PointerPhase::Down,
                f64::from(origin[0] + 1.0),
                f64::from(origin[1] + 5.0),
                1,
            ))
            .handled
    );
    assert!(
        applier
            .handle_event(pointer(
                PointerPhase::Move,
                f64::from(origin[0] + 55.0),
                f64::from(origin[1] + 5.0),
                1,
            ))
            .handled
    );
    let during_drag = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<usize, _>(format!(
                "globalThis.dispatched.filter((event) => event[1] === {}).length",
                event::TEXTSELECTIONCHANGE
            ))
        })
        .expect("selection event count while dragging");
    assert_eq!(during_drag, 0, "dragging stays local to the renderer");
    applier.handle_event(pointer(
        PointerPhase::Up,
        f64::from(origin[0] + 55.0),
        f64::from(origin[1] + 5.0),
        0,
    ));
    applier.prepare_text_selection(&mut placed, &mut tcx);
    assert!(
        applier.interaction.text_selection.last_click.is_none(),
        "a drag must not seed the subsequent multi-click streak"
    );

    let selected = applier.selected_text().expect("non-empty selection");
    let selection_event = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(format!(
                "JSON.stringify(globalThis.dispatched.find((event) => event[1] === {}))",
                event::TEXTSELECTIONCHANGE
            ))
        })
        .expect("committed selection event");
    let selection_event: serde_json::Value = serde_json::from_str(&selection_event).unwrap();
    assert_eq!(selection_event[0], 2);
    assert_eq!(selection_event[1], event::TEXTSELECTIONCHANGE);
    let payload: serde_json::Value =
        serde_json::from_str(selection_event[2].as_str().unwrap()).unwrap();
    assert_eq!(payload["text"], selected);
    assert_eq!(payload["kind"], "simple");
    assert!("selectable text".starts_with(&selected));
    assert!(
        !placed
            .iter()
            .find(|node| node.node_id
                == applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)])
            .unwrap()
            .paint
            .selection_rects
            .is_empty()
    );

    let copied = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "c".into(),
        key_without_modifiers: "c".into(),
        code: "KeyC".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        },
        repeat: false,
        synthetic: false,
    }));
    assert_eq!(
        copied.clipboard,
        Some(wabou_shell::ClipboardRequest::Write(selected))
    );

    applier.interaction.text_selection.last_click = None;
    let click_x = f64::from(origin[0] + 10.0);
    let click_y = f64::from(origin[1] + 5.0);
    applier.handle_event(pointer(PointerPhase::Down, click_x, click_y, 1));
    let mut cancelled = match pointer(PointerPhase::Cancel, click_x, click_y, 0) {
        UiEvent::Pointer(pointer) => pointer,
        _ => unreachable!(),
    };
    // Platform pointer-cancel events commonly omit the triggering button.
    cancelled.button = None;
    applier.handle_event(UiEvent::Pointer(cancelled));
    applier.handle_event(pointer(PointerPhase::Down, click_x, click_y, 1));
    assert_eq!(
        applier
            .interaction
            .text_selection
            .active
            .as_ref()
            .unwrap()
            .granularity,
        TextSelectionGranularity::Cluster,
        "a cancelled click must not turn the next click into word selection"
    );
    applier.handle_event(pointer(PointerPhase::Cancel, click_x, click_y, 0));
    applier.interaction.text_selection.last_click = None;

    let word_x = f64::from(origin[0] + 10.0);
    for _ in 0..2 {
        applier.begin_text_selection(
            NodeKey::new(2, 1),
            word_x,
            f64::from(origin[1] + 5.0),
            Modifiers::empty(),
        );
    }
    assert_eq!(applier.selected_text().as_deref(), Some("selectable"));
    assert!(applier.sync_text_selection_change());
    let word_event = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(format!(
                "JSON.stringify(globalThis.dispatched.filter((event) => event[1] === {}).at(-1))",
                event::TEXTSELECTIONCHANGE
            ))
        })
        .expect("word selection event");
    let word_event: serde_json::Value = serde_json::from_str(&word_event).unwrap();
    let payload: serde_json::Value = serde_json::from_str(word_event[2].as_str().unwrap()).unwrap();
    assert_eq!(
        payload,
        serde_json::json!({ "text": "selectable", "kind": "word" })
    );
    let line_end = f64::from(
        origin[0]
            + applier.interaction.text_selection.selectable[&NodeKey::new(2, 1)]
                .layout
                .width()
            + 10.0,
    );
    applier.begin_text_selection(
        NodeKey::new(2, 1),
        line_end,
        f64::from(origin[1] + 5.0),
        Modifiers::SHIFT,
    );
    assert_eq!(
        applier.selected_text().as_deref(),
        Some("selectable text"),
        "the explicit single-line Text contract extends through the full run"
    );
    assert!(
        applier.interaction.text_selection.last_click.is_none(),
        "Shift extension must not seed a later double click"
    );

    let user_select = applier.document.atoms.borrow_mut().intern("user-select");
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: user_select,
        value: "all",
    });
    applier.inherit();
    let mut placed = layout::compute_and_walk_with_scroll(
        &mut applier.document.node_store.tree,
        applier.document.node_store.root,
        400.0,
        100.0,
        &mut tcx,
        &HashMap::new(),
    );
    applier.rebuild_hit_geometry(&placed);
    applier.prepare_text_selection(&mut placed, &mut tcx);
    applier.handle_event(pointer(
        PointerPhase::Down,
        f64::from(origin[0] + 2.0),
        f64::from(origin[1] + 5.0),
        1,
    ));
    assert_eq!(applier.selected_text().as_deref(), Some("selectable text"));
    applier.interaction.text_selection.next_scroll = Some(Instant::now());

    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: user_select,
        value: "none",
    });
    applier.inherit();
    let mut placed = layout::compute_and_walk_with_scroll(
        &mut applier.document.node_store.tree,
        applier.document.node_store.root,
        400.0,
        100.0,
        &mut tcx,
        &HashMap::new(),
    );
    applier.prepare_text_selection(&mut placed, &mut tcx);
    assert!(
        !applier
            .computed_node_snapshot(NodeKey::new(2, 1))
            .unwrap()
            .text_selectable
    );
    assert!(
        !applier
            .interaction
            .text_selection
            .selectable
            .contains_key(&NodeKey::new(2, 1))
    );
    assert!(applier.interaction.text_selection.active.is_none());
    assert!(applier.interaction.text_selection.next_scroll.is_none());
    assert!(applier.sync_text_selection_change());
    let cleared = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(format!(
                "JSON.stringify(globalThis.dispatched.filter((event) => event[1] === {}).at(-1))",
                event::TEXTSELECTIONCHANGE
            ))
        })
        .expect("selection clear event");
    let cleared: serde_json::Value = serde_json::from_str(&cleared).unwrap();
    let payload: serde_json::Value = serde_json::from_str(cleared[2].as_str().unwrap()).unwrap();
    assert_eq!(payload, serde_json::json!({ "text": null, "kind": null }));
}

#[test]
fn text_selection_crosses_hosts_in_both_directions() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (div, height, flex_direction, user_select) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("height"),
            atoms.intern("flex-direction"),
            atoms.intern("user-select"),
        )
    };
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(1, 1),
        prop: flex_direction,
        value: "column",
    });
    for (host, text_node, text, selectable) in [
        (2, 3, "alpha", true),
        (4, 5, "secret", false),
        (6, 7, "beta", true),
    ] {
        applier.apply_op(&Op::CreateElement {
            id: nk(host),
            tag: div,
        });
        set_text_behavior(&mut applier, host);
        applier.apply_op(&Op::SetStyle {
            id: nk(host),
            prop: height,
            value: "30px",
        });
        if !selectable {
            applier.apply_op(&Op::SetStyle {
                id: nk(host),
                prop: user_select,
                value: "none",
            });
        }
        applier.apply_op(&Op::CreateText {
            id: nk(text_node),
            text,
        });
        applier.apply_op(&Op::AppendChild {
            parent: nk(host),
            child: nk(text_node),
        });
        applier.apply_op(&Op::AppendChild {
            parent: NodeKey::new(1, 1),
            child: nk(host),
        });
    }
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::TEXTSELECTIONCHANGE,
    });
    applier.rebuild_layout_boxes();
    applier.inherit();

    let mut tcx = TextContext::new();
    let mut placed = layout::compute_and_walk_with_scroll(
        &mut applier.document.node_store.tree,
        applier.document.node_store.root,
        300.0,
        100.0,
        &mut tcx,
        &HashMap::new(),
    );
    applier.rebuild_hit_geometry(&placed);
    applier.prepare_text_selection(&mut placed, &mut tcx);
    assert_eq!(applier.interaction.text_selection.order, vec![nk(2), nk(6)]);

    let select_all = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "a".into(),
        key_without_modifiers: "a".into(),
        code: "KeyA".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        },
        repeat: false,
        synthetic: false,
    }));
    assert!(select_all.handled);
    assert!(select_all.request_redraw);
    assert_eq!(applier.selected_text().as_deref(), Some("alpha\nbeta"));
    let copy_all = applier.handle_event(UiEvent::Key(wabou_shell::KeyEvent {
        phase: KeyPhase::Down,
        key: "c".into(),
        key_without_modifiers: "c".into(),
        code: "KeyC".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        },
        repeat: false,
        synthetic: false,
    }));
    assert_eq!(
        copy_all.clipboard,
        Some(wabou_shell::ClipboardRequest::Write("alpha\nbeta".into()))
    );

    let point = |text: &SelectableText, index: usize| {
        let cursor = Cursor::from_byte_index(&text.layout, index, Affinity::Downstream);
        let geometry = cursor.geometry(&text.layout, 0.0);
        (
            f64::from(text.origin[0]) + geometry.x0 + 0.1,
            f64::from(text.origin[1]) + geometry.y0 + 2.0,
        )
    };
    let first_start = point(
        &applier.interaction.text_selection.selectable[&NodeKey::new(2, 1)],
        0,
    );
    let second_end = point(
        &applier.interaction.text_selection.selectable[&NodeKey::new(6, 1)],
        4,
    );
    assert_eq!(
        applier
            .interaction
            .input
            .hit_test(first_start.0, first_start.1),
        Some(NodeKey::new(2, 1))
    );
    assert_eq!(
        applier
            .interaction
            .input
            .hit_test(second_end.0, second_end.1),
        Some(NodeKey::new(6, 1))
    );
    applier.handle_event(pointer(PointerPhase::Down, first_start.0, first_start.1, 1));
    applier.handle_event(pointer(PointerPhase::Move, second_end.0, second_end.1, 1));
    applier.handle_event(pointer(PointerPhase::Up, second_end.0, second_end.1, 0));
    assert_eq!(applier.selected_text().as_deref(), Some("alpha\nbeta"));
    assert!(applier.interaction.input.pointer_down_target.is_none());
    applier.prepare_text_selection(&mut placed, &mut tcx);
    for target in [2, 6] {
        let node = applier.document.node_store.solid_to_node[&nk(target)];
        assert!(
            !placed
                .iter()
                .find(|placed| placed.node_id == node)
                .unwrap()
                .paint
                .selection_rects
                .is_empty()
        );
    }

    applier.interaction.text_selection.last_click = None;
    applier.begin_text_selection(
        NodeKey::new(6, 1),
        second_end.0,
        second_end.1,
        Modifiers::empty(),
    );
    applier.extend_text_selection(Some(NodeKey::new(2, 1)), first_start.0, first_start.1);
    assert_eq!(applier.selected_text().as_deref(), Some("alpha\nbeta"));

    applier.apply_op(&Op::SetText {
        id: NodeKey::new(7, 1),
        // Shorter multibyte replacement forces both endpoints through
        // Parley's UTF-8 cluster-boundary refresh path.
        text: "你",
    });
    applier.inherit();
    let mut placed = layout::compute_and_walk_with_scroll(
        &mut applier.document.node_store.tree,
        applier.document.node_store.root,
        300.0,
        100.0,
        &mut tcx,
        &HashMap::new(),
    );
    applier.prepare_text_selection(&mut placed, &mut tcx);
    assert_eq!(applier.selected_text().as_deref(), Some("alpha\n你"));

    applier.apply_op(&Op::RemoveChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(6, 1),
    });
    applier.apply_op(&Op::DropNode {
        id: NodeKey::new(6, 1),
    });
    let mut placed = layout::compute_and_walk_with_scroll(
        &mut applier.document.node_store.tree,
        applier.document.node_store.root,
        300.0,
        100.0,
        &mut tcx,
        &HashMap::new(),
    );
    applier.prepare_text_selection(&mut placed, &mut tcx);
    assert!(applier.interaction.text_selection.active.is_none());
    assert!(applier.selected_text().is_none());
    let cleared = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(format!(
                "JSON.stringify(globalThis.dispatched.filter((event) => event[1] === {}).at(-1))",
                event::TEXTSELECTIONCHANGE
            ))
        })
        .expect("selection clear on dropped endpoint");
    let cleared: serde_json::Value = serde_json::from_str(&cleared).unwrap();
    let payload: serde_json::Value = serde_json::from_str(cleared[2].as_str().unwrap()).unwrap();
    assert_eq!(payload, serde_json::json!({ "text": null, "kind": null }));
}

#[test]
fn same_visual_line_with_different_font_metrics_copies_without_newline() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (div, flex_direction, align_items, height, font_size) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("flex-direction"),
            atoms.intern("align-items"),
            atoms.intern("height"),
            atoms.intern("font-size"),
        )
    };
    for (prop, value) in [
        (flex_direction, "row"),
        (align_items, "center"),
        (height, "60px"),
    ] {
        applier.apply_op(&Op::SetStyle {
            id: NodeKey::new(1, 1),
            prop,
            value,
        });
    }
    for (host, text_node, text, size) in [(2, 3, "small", "12px"), (4, 5, "BIG", "30px")] {
        applier.apply_op(&Op::CreateElement {
            id: nk(host),
            tag: div,
        });
        set_text_behavior(&mut applier, host);
        applier.apply_op(&Op::SetStyle {
            id: nk(host),
            prop: font_size,
            value: size,
        });
        applier.apply_op(&Op::CreateText {
            id: nk(text_node),
            text,
        });
        applier.apply_op(&Op::AppendChild {
            parent: nk(host),
            child: nk(text_node),
        });
        applier.apply_op(&Op::AppendChild {
            parent: NodeKey::new(1, 1),
            child: nk(host),
        });
    }
    applier.rebuild_layout_boxes();
    applier.inherit();
    let mut tcx = TextContext::new();
    let mut placed = layout::compute_and_walk_with_scroll(
        &mut applier.document.node_store.tree,
        applier.document.node_store.root,
        300.0,
        60.0,
        &mut tcx,
        &HashMap::new(),
    );
    applier.prepare_text_selection(&mut placed, &mut tcx);

    let small = &applier.interaction.text_selection.selectable[&NodeKey::new(2, 1)];
    let big = &applier.interaction.text_selection.selectable[&NodeKey::new(4, 1)];
    assert!((small.origin[1] - big.origin[1]).abs() > 1.0);
    assert!(small.visual_y.start < big.visual_y.end && big.visual_y.start < small.visual_y.end);
    assert!(applier.select_all_text());
    assert_eq!(applier.selected_text().as_deref(), Some("smallBIG"));
}

#[test]
fn explicit_text_flow_does_not_absorb_a_nested_element() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (div, strong, font_weight, color) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("strong"),
            atoms.intern("font-weight"),
            atoms.intern("color"),
        )
    };
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    set_text_behavior(&mut applier, 2);
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(3, 1),
        text: "Hello ",
    });
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(4, 1),
        tag: strong,
    });
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(5, 1),
        text: "world",
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(4, 1),
        prop: font_weight,
        value: "700",
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(4, 1),
        prop: color,
        value: "#ff0000",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(4, 1),
        child: NodeKey::new(5, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(4, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });

    applier.rebuild_layout_boxes();
    applier.inherit();

    let parent = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert_eq!(applier.document.node_store.tree.child_count(parent), 2);
    assert!(!applier.document.node_store.inline_roots.contains(&parent));
    assert!(
        applier
            .document
            .node_store
            .tree
            .get_node_context(parent)
            .unwrap()
            .text
            .is_none()
    );
}

#[test]
fn explicit_rich_text_flow_absorbs_styled_text_descendants_as_runs() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (text_tag, span_tag, font_weight, color) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("text"),
            atoms.intern("text-span"),
            atoms.intern("font-weight"),
            atoms.intern("color"),
        )
    };
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: text_tag,
    });
    applier.apply_op(&Op::SetTextBehavior {
        id: NodeKey::new(2, 1),
        flags: crate::protocol::TEXT_BEHAVIOR_AGGREGATE_DIRECT
            | crate::protocol::TEXT_BEHAVIOR_AGGREGATE_STYLED,
    });
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(3, 1),
        text: "Hello ",
    });
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(4, 1),
        tag: span_tag,
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(4, 1),
        prop: font_weight,
        value: "700",
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(4, 1),
        prop: color,
        value: "#ff0000",
    });
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(5, 1),
        text: "world",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(4, 1),
        child: NodeKey::new(5, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(4, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });

    applier.rebuild_layout_boxes();
    applier.inherit();

    let parent = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert_eq!(applier.document.node_store.tree.child_count(parent), 0);
    assert!(applier.document.node_store.inline_roots.contains(&parent));
    let paint = applier
        .document
        .node_store
        .tree
        .get_node_context(parent)
        .expect("rich text paint");
    assert_eq!(paint.text.as_deref(), Some("Hello world"));
    assert_eq!(paint.text_runs.len(), 2);
    assert_eq!(paint.text_runs[0].range, 0..6);
    assert_eq!(paint.text_runs[1].range, 6..11);
    assert_eq!(paint.text_runs[1].font_weight, 700.0);
    assert_eq!(paint.text_runs[1].color, [255, 0, 0, 255]);
}
