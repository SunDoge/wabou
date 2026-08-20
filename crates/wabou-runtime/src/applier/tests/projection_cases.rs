use super::*;

#[test]
fn host_layout_snapshot_reports_completed_rects_and_viewport() {
    const CORE_FIXTURE: &str = include_str!("../../gen/test-runtime.js");
    let mut applier = interactive_applier();
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.publish_layout_metrics(&placed, 800, 600);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");

    let json = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "JSON.stringify(__wabou_test_host_api.host.layout.snapshot([{lo: 2, hi: 1}, {lo: 999999, hi: 1}]))",
            )
        })
        .expect("layout snapshot");
    let snapshot: serde_json::Value = serde_json::from_str(&json).expect("snapshot JSON");

    assert_eq!(snapshot["viewport"]["width"], 800.0);
    assert_eq!(snapshot["viewport"]["height"], 600.0);
    assert_eq!(snapshot["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(
        snapshot["nodes"][0]["id"],
        serde_json::json!({"lo": 2, "hi": 1})
    );
    assert_eq!(snapshot["nodes"][0]["rect"]["width"], 100.0);
    assert_eq!(snapshot["nodes"][0]["rect"]["height"], 50.0);
    assert_eq!(snapshot["nodes"][0]["clip"], snapshot["viewport"]);
    assert_eq!(
        snapshot["nodes"][0]["scroll"],
        serde_json::json!({
            "offsetX": 0,
            "offsetY": 0,
            "rangeX": 0,
            "rangeY": 0,
        })
    );
}

#[test]
fn stable_frames_do_not_republish_layout_or_empty_semantics() {
    let mut applier = interactive_applier();
    let empty_semantics = applier.frame.projections.semantic_snapshot.clone();
    applier.set_semantics_enabled(false);
    assert!(Arc::ptr_eq(
        &empty_semantics,
        &applier.frame.projections.semantic_snapshot
    ));
    applier
        .boot(
            "globalThis.__wabou_tick = () => false; \
             globalThis.__wabou_has_raf = () => false;",
        )
        .expect("boot stable-frame fixture");
    let debug = wabou_devtools::DebugState::shared();
    applier.set_debug_state(debug.clone());

    let mut text = TextContext::new();
    applier.build_frame(&mut text, 801, 600);
    let revision = applier.frame.projections.layout_metrics.borrow().revision;
    let debug_revision = debug.read().unwrap().snapshot().status.revision;
    assert!(revision > 0);
    assert!(debug_revision > 0);

    applier.build_frame(&mut text, 801, 600);
    assert_eq!(
        applier.frame.projections.layout_metrics.borrow().revision,
        revision
    );
    assert_eq!(
        debug.read().unwrap().snapshot().status.revision,
        debug_revision
    );

    let opacity = applier.document.atoms.borrow_mut().intern("opacity");
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: opacity,
        value: "0.5",
    });
    assert!(!applier.frame.projections.semantics_dirty);
    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::GEOMETRY)
    );
    applier.build_frame(&mut text, 801, 600);
    assert_eq!(
        applier.frame.projections.layout_metrics.borrow().revision,
        revision,
        "paint-only updates must not republish whole-tree layout projections"
    );
    assert!(
        debug.read().unwrap().snapshot().status.revision > debug_revision,
        "attached DevTools still receives the paint update"
    );
}

#[test]
fn hit_affecting_paint_still_invalidates_geometry() {
    let mut applier = interactive_applier();
    applier
        .document
        .invalidation
        .remove(InvalidationFlags::GEOMETRY);
    let pointer_events = applier.document.atoms.borrow_mut().intern("pointer-events");

    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: pointer_events,
        value: "none",
    });

    assert!(
        applier
            .document
            .invalidation
            .contains(InvalidationFlags::GEOMETRY)
    );
}

#[test]
fn semantic_attributes_do_not_invalidate_style_or_layout() {
    let mut applier = interactive_applier();
    applier
        .document
        .invalidation
        .remove(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
    applier.frame.projections.semantics_dirty = false;
    let expanded = applier.document.atoms.borrow_mut().intern("aria-expanded");

    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: expanded,
        value: "true",
    });

    assert!(applier.frame.projections.semantics_dirty);
    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
    );
    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::INHERIT)
    );

    let class = applier.document.atoms.borrow_mut().intern("class");
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: class,
        value: "opacity-50",
    });
    assert_eq!(
        applier
            .computed_node_snapshot(NodeKey::new(2, 1))
            .unwrap()
            .opacity,
        0.5
    );
    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
    );
    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::INHERIT)
    );

    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: class,
        value: "flex",
    });
    assert!(
        applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
    );

    applier
        .document
        .invalidation
        .remove(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: class,
        value: "text-xl",
    });
    assert!(
        applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
    );
    assert!(
        applier
            .document
            .invalidation
            .contains(InvalidationFlags::INHERIT)
    );
}

#[test]
fn svg_descendant_attributes_still_refresh_the_svg_projection() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (svg, path, d) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (atoms.intern("svg"), atoms.intern("path"), atoms.intern("d"))
    };
    applier.apply_frame(&Frame {
        seq: 1,
        ops: vec![
            Op::CreateElement {
                id: NodeKey::new(2, 1),
                tag: svg,
            },
            Op::CreateElement {
                id: NodeKey::new(3, 1),
                tag: path,
            },
            Op::AppendChild {
                parent: NodeKey::new(2, 1),
                child: NodeKey::new(3, 1),
            },
            Op::AppendChild {
                parent: NodeKey::new(1, 1),
                child: NodeKey::new(2, 1),
            },
        ],
    });
    applier
        .document
        .invalidation
        .remove(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);

    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(3, 1),
        name: d,
        value: "M0 0L1 1",
    });

    assert!(
        applier
            .document
            .invalidation
            .contains(InvalidationFlags::INHERIT)
    );
    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
    );
}

#[test]
fn public_host_adapter_runs_in_embedded_quickjs() {
    const CORE_FIXTURE: &str = include_str!("../../gen/test-runtime.js");
    let mut applier = Applier::from_runtime(JsRuntime::new().expect("runtime"), Color::BLACK);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");

    let result = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                r#"JSON.stringify({
                  open: __wabou_test_host_api.host.system.openUrl("not a URL"),
                  font: __wabou_test_host_api.host.fonts.load("/wabou/does/not/exist.ttf"),
                  stats: __wabou_test_host_api.host.diagnostics.frameStats(),
                })"#,
            )
        })
        .expect("call public host adapter");

    assert_eq!(result, r#"{"open":false,"font":false,"stats":null}"#);
}

#[test]
fn pointer_sequence_hit_tests_and_synthesizes_one_click() {
    let mut applier = interactive_applier();
    assert_eq!(
        applier.interaction.input.hit_test(20.0, 20.0),
        Some(NodeKey::new(2, 1))
    );
    assert!(
        applier
            .handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1))
            .handled
    );
    assert!(
        applier
            .handle_event(pointer(PointerPhase::Up, 20.0, 20.0, 0))
            .handled
    );

    let codes = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<Vec<u8>, _>("globalThis.dispatched.map((x) => x[1])"))
        .expect("read dispatched events");
    assert_eq!(
        codes,
        vec![event::POINTERDOWN, event::POINTERUP, event::CLICK]
    );
}

#[test]
fn secondary_pointer_sequence_dispatches_context_menu_without_click() {
    let mut applier = interactive_applier();
    applier.handle_event(pointer_with_button(
        PointerPhase::Down,
        20.0,
        20.0,
        2,
        PointerButton::Secondary,
    ));
    applier.handle_event(pointer_with_button(
        PointerPhase::Up,
        20.0,
        20.0,
        0,
        PointerButton::Secondary,
    ));

    let codes = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<Vec<u8>, _>("globalThis.dispatched.map((x) => x[1])"))
        .expect("read dispatched events");
    assert_eq!(
        codes,
        vec![event::POINTERDOWN, event::POINTERUP, event::CONTEXTMENU]
    );
}

#[test]
fn dragging_outside_pressed_target_keeps_the_js_pointer_capture() {
    let mut applier = interactive_applier();
    applier.handle_event(pointer(PointerPhase::Down, 10.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Move, 180.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Up, 180.0, 20.0, 0));

    let codes = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<Vec<u8>, _>("globalThis.dispatched.map((x) => x[1])"))
        .expect("read dispatched events");
    assert!(codes.contains(&event::POINTERDOWN));
    assert!(codes.contains(&event::POINTERMOVE));
    assert!(codes.contains(&event::POINTERUP));
    assert!(!codes.contains(&event::CLICK));
    assert!(applier.interaction.input.pointer_down_target.is_none());
    assert!(applier.interaction.input.pointer_down_position.is_none());
    assert!(!applier.interaction.input.pointer_dragged);
}

#[test]
fn coalesced_release_distance_also_suppresses_click() {
    let mut applier = interactive_applier();
    applier.handle_event(pointer(PointerPhase::Down, 10.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Up, 80.0, 20.0, 0));

    let click_count = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<usize, _>("globalThis.dispatched.filter((x) => x[1] === 1).length"))
        .expect("read click count");
    assert_eq!(click_count, 0);
}

#[test]
fn devtools_snapshot_exposes_real_layout_and_event_trace() {
    let mut applier = interactive_applier();
    let (
        class,
        role,
        aria_label,
        aria_pressed,
        aria_current,
        aria_value_now,
        aria_value_min,
        aria_value_max,
    ) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("class"),
            atoms.intern("role"),
            atoms.intern("aria-label"),
            atoms.intern("aria-pressed"),
            atoms.intern("aria-current"),
            atoms.intern("aria-valuenow"),
            atoms.intern("aria-valuemin"),
            atoms.intern("aria-valuemax"),
        )
    };
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: class,
        value: "font-medium font-normal",
    });
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: role,
        value: "button",
    });
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: aria_label,
        value: "Save",
    });
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: aria_pressed,
        value: "mixed",
    });
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: aria_current,
        value: "page",
    });
    for (name, value) in [
        (aria_value_now, "64"),
        (aria_value_min, "0"),
        (aria_value_max, "100"),
    ] {
        applier.apply_op(&Op::SetAttribute {
            id: NodeKey::new(2, 1),
            name,
            value,
        });
    }
    let state = wabou_devtools::DebugState::shared();
    applier.set_debug_state(state.clone());
    set_focus_order(&mut applier, 2, 3);
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.rebuild_focus_order(&placed);
    applier.set_semantics_enabled(true);
    applier.rebuild_semantic_snapshot(&placed);
    applier.frame.last_viewport = (800, 600);
    applier.publish_debug_snapshot(&placed, &mut TextContext::new());
    applier.handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1));

    let state = state.read().unwrap();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.status.viewport_width, 800);
    assert_eq!(snapshot.status.text_backend, "swash");
    assert!(matches!(
        snapshot.status.text_outline_fallback.as_str(),
        "direct-native-weight" | "retained-synthetic-weight"
    ));
    let json = serde_json::to_value(snapshot).unwrap();
    assert_eq!(json["status"]["textBackend"], "swash");
    assert!(json["nodes"].as_array().unwrap().iter().any(|node| {
        node["id"]["lo"] == 2
            && node["computed"]["syntheticBold"] == false
            && node["computed"]["syntheticItalic"] == false
            && node["computed"].get("fontFamily").is_some()
    }));
    let button = snapshot.nodes.iter().find(|node| node.id == nk(2)).unwrap();
    assert_eq!(button.tag, "button");
    assert_eq!(button.rect.width, 100.0);
    assert_eq!(button.rect.height, 50.0);
    assert!(button.focusable);
    assert_eq!(button.focus_order, Some(3));
    let semantic = button.semantic.as_ref().expect("semantic projection");
    assert_eq!(semantic.role, "button");
    assert_eq!(semantic.label.as_deref(), Some("Save"));
    assert!(semantic.exposed);
    assert_eq!(semantic.states.pressed.as_deref(), Some("mixed"));
    assert_eq!(semantic.states.current.as_deref(), Some("page"));
    assert_eq!(semantic.range.value, Some(64.0));
    assert_eq!(semantic.range.min, Some(0.0));
    assert_eq!(semantic.range.max, Some(100.0));
    let button_json = json["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|node| node["id"]["lo"] == 2)
        .unwrap();
    assert_eq!(button_json["semantic"]["range"]["value"], 64.0);
    assert_eq!(button_json["semantic"]["range"]["min"], 0.0);
    assert_eq!(button_json["semantic"]["range"]["max"], 100.0);
    assert!(!button.computed.synthetic_bold);
    assert!(!button.computed.synthetic_italic);
    let font_weight = button
        .style_cascade
        .iter()
        .find(|entry| entry.property == "font-weight")
        .expect("font weight cascade entry");
    assert_eq!(font_weight.source, ".font-normal");
    assert_eq!(font_weight.overridden_sources, [".font-medium"]);
    assert!(
        state
            .frames()
            .iter()
            .any(|frame| { frame.direction == "hostToJs" && frame.record_count == 1 })
    );
}

#[test]
fn devtools_snapshot_exposes_widget_local_and_ancestor_clip_coordinates() {
    let mut applier = interactive_applier();
    let state = wabou_devtools::DebugState::shared();
    applier.set_debug_state(state.clone());
    let widget_node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    applier
        .document
        .widget_manager
        .widgets
        .insert(widget_node, Box::new(MeasuringWidget([100.0, 50.0])));
    let mut placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    let root = placed
        .iter_mut()
        .find(|node| node.node_id == applier.document.node_store.root)
        .unwrap();
    root.own_clip = Some([0.0, 0.0, 80.0, 40.0]);
    root.own_clip_radius = 6.0;
    let widget = placed
        .iter_mut()
        .find(|node| node.node_id == widget_node)
        .unwrap();
    widget.clip = Some([0.0, 0.0, 80.0, 40.0]);
    widget.clip_radius = 6.0;
    widget.paint.border_radius = 12.0;

    applier.publish_debug_snapshot(&placed, &mut TextContext::new());

    let state = state.read().unwrap();
    let widget = state
        .snapshot()
        .nodes
        .iter()
        .find(|node| node.id == nk(2))
        .unwrap();
    assert_eq!(widget.clip.widget_local.as_ref().unwrap().radius, 12.0);
    assert_eq!(
        widget.clip.widget_local.as_ref().unwrap().coordinate_space,
        "content-local"
    );
    assert_eq!(widget.clip.chain.len(), 1);
    assert_eq!(widget.clip.chain[0].node_id, nk(1));
    assert_eq!(widget.clip.effective.as_ref().unwrap().rect.width, 80.0);
    assert_eq!(widget.clip.device_scale, 1.0);
}

#[test]
fn releasing_outside_the_pressed_target_does_not_click() {
    let mut applier = interactive_applier();
    applier.handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Up, 200.0, 200.0, 0));

    let click_count = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<usize, _>("globalThis.dispatched.filter((x) => x[1] === 1).length"))
        .expect("read click count");
    assert_eq!(click_count, 0);
}

#[test]
fn resize_observer_reports_initial_content_box_once() {
    let mut applier = interactive_applier();
    applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                __wabou_resize_observe(2, 1);
                "#,
            )
        })
        .expect("install resize hook");

    assert!(applier.dispatch_resize_changes());
    assert!(!applier.dispatch_resize_changes());
    let changes = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<Vec<Vec<f32>>, _>("globalThis.resizeChanges"))
        .expect("read resize changes");
    assert_eq!(changes, vec![vec![2.0, 100.0, 50.0]]);
}

#[test]
fn devtools_snapshot_exposes_layout_and_redacts_secrets() {
    let mut applier = interactive_applier();
    let password = applier.document.atoms.borrow_mut().intern("password");
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: password,
        value: "do-not-leak",
    });
    let state = wabou_devtools::DebugState::shared();
    applier.set_debug_state(state.clone());
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.frame.last_viewport = (800, 600);
    applier.publish_debug_snapshot(&placed, &mut TextContext::new());

    let state = state.read().unwrap();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.status.viewport_width, 800);
    let button = snapshot.nodes.iter().find(|node| node.id == nk(2)).unwrap();
    assert_eq!(button.tag, "button");
    assert_eq!(button.rect.width, 100.0);
    assert_eq!(
        button
            .attrs
            .iter()
            .find(|(name, _)| name == "password")
            .unwrap()
            .1,
        "[REDACTED]"
    );
}

#[test]
fn runtime_transform_updates_paint_without_invalidating_layout() {
    let mut applier = interactive_applier();
    applier
        .document
        .invalidation
        .remove(InvalidationFlags::LAYOUT);
    applier.apply_op(&Op::SetTransform2D {
        id: NodeKey::new(2, 1),
        matrix: [1.0, 0.0, 0.0, 1.0, 12.5, -3.25],
    });

    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
    );
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .get_node_context(node)
            .unwrap()
            .runtime_transform,
        Some([1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
    );
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.rebuild_hit_geometry(&placed);
    assert_ne!(
        applier.interaction.input.hit_test(5.0, 20.0),
        Some(NodeKey::new(2, 1))
    );
    assert_eq!(
        applier.interaction.input.hit_test(32.5, 16.75),
        Some(NodeKey::new(2, 1))
    );

    let transform = applier.document.atoms.borrow_mut().intern("transform");
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: transform,
        value: "translate(2px, 3px)",
    });
    let paint = applier
        .document
        .node_store
        .tree
        .get_node_context(node)
        .unwrap();
    assert_eq!(
        paint.runtime_transform,
        Some([1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
    );
    assert_eq!(
        applier.document.runtime_transforms.get(&node),
        Some(&[1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
    );
}

#[test]
fn protocol_shadows_apply_vello_parameters_without_string_parsing() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    applier.apply_op(&Op::SetShadows {
        id: NodeKey::new(2, 1),
        shadows: vec![crate::protocol::ShadowValue {
            offset_x: 3.0,
            offset_y: 7.0,
            spread: -2.0,
            std_dev: 5.5,
            color: 0x336699cc,
            radius: Some(11.0),
        }],
    });

    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let paint = applier
        .document
        .node_store
        .tree
        .get_node_context(node)
        .unwrap();
    assert_eq!(
        paint.shadows,
        vec![wabou_shell::style::Shadow {
            offset_x: 3.0,
            offset_y: 7.0,
            spread: -2.0,
            std_dev: 5.5,
            color: Color::from_rgba8(0x33, 0x66, 0x99, 0xcc),
            radius: Some(11.0),
        }]
    );
}
