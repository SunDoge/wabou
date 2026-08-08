use super::*;

#[test]
fn host_layout_snapshot_reports_completed_rects_and_viewport() {
    const CORE_FIXTURE: &str = include_str!("../../gen/test-runtime.js");
    let mut applier = interactive_applier();
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.publish_layout_metrics(&placed, 800, 600);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");

    let json = applier
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "JSON.stringify(__wabou_test_host_api.host.layout.snapshot([2, 999999]))",
            )
        })
        .expect("layout snapshot");
    let snapshot: serde_json::Value = serde_json::from_str(&json).expect("snapshot JSON");

    assert_eq!(snapshot["viewport"]["width"], 800.0);
    assert_eq!(snapshot["viewport"]["height"], 600.0);
    assert_eq!(snapshot["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["nodes"][0]["id"], 2);
    assert_eq!(snapshot["nodes"][0]["rect"]["width"], 100.0);
    assert_eq!(snapshot["nodes"][0]["rect"]["height"], 50.0);
    assert_eq!(snapshot["nodes"][0]["clip"], snapshot["viewport"]);
}

#[test]
fn stable_frames_do_not_republish_layout_or_empty_semantics() {
    let mut applier = interactive_applier();
    let empty_semantics = applier.projections.semantic_snapshot.clone();
    applier.set_semantics_enabled(false);
    assert!(Arc::ptr_eq(
        &empty_semantics,
        &applier.projections.semantic_snapshot
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
    let revision = applier.projections.layout_metrics.borrow().revision;
    let debug_revision = debug.read().unwrap().snapshot().status.revision;
    assert!(revision > 0);
    assert!(debug_revision > 0);

    applier.build_frame(&mut text, 801, 600);
    assert_eq!(
        applier.projections.layout_metrics.borrow().revision,
        revision
    );
    assert_eq!(
        debug.read().unwrap().snapshot().status.revision,
        debug_revision
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
    assert_eq!(applier.input.hit_test(20.0, 20.0), Some(2));
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
        .js
        .with(|ctx| ctx.eval::<Vec<u8>, _>("globalThis.dispatched.map((x) => x[1])"))
        .expect("read dispatched events");
    assert_eq!(
        codes,
        vec![event::POINTERDOWN, event::POINTERUP, event::CLICK]
    );
}

#[test]
fn dragging_inside_pressed_target_does_not_synthesize_a_click() {
    let mut applier = interactive_applier();
    applier.handle_event(pointer(PointerPhase::Down, 10.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Move, 80.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Up, 80.0, 20.0, 0));

    let codes = applier
        .js
        .with(|ctx| ctx.eval::<Vec<u8>, _>("globalThis.dispatched.map((x) => x[1])"))
        .expect("read dispatched events");
    assert!(codes.contains(&event::POINTERDOWN));
    assert!(codes.contains(&event::POINTERUP));
    assert!(!codes.contains(&event::CLICK));
    assert!(applier.input.pointer_down_target.is_none());
    assert!(applier.input.pointer_down_position.is_none());
    assert!(!applier.input.pointer_dragged);
}

#[test]
fn coalesced_release_distance_also_suppresses_click() {
    let mut applier = interactive_applier();
    applier.handle_event(pointer(PointerPhase::Down, 10.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Up, 80.0, 20.0, 0));

    let click_count = applier
        .js
        .with(|ctx| ctx.eval::<usize, _>("globalThis.dispatched.filter((x) => x[1] === 1).length"))
        .expect("read click count");
    assert_eq!(click_count, 0);
}

#[test]
fn devtools_snapshot_exposes_real_layout_and_event_trace() {
    let mut applier = interactive_applier();
    let state = wabou_devtools::DebugState::shared();
    applier.set_debug_state(state.clone());
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.last_viewport = (800, 600);
    applier.publish_debug_snapshot(&placed);
    applier.handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1));

    let state = state.read().unwrap();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.status.viewport_width, 800);
    let button = snapshot.nodes.iter().find(|node| node.id == 2).unwrap();
    assert_eq!(button.tag, "button");
    assert_eq!(button.rect.width, 100.0);
    assert_eq!(button.rect.height, 50.0);
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
    let widget_node = applier.node_store.solid_to_node[&2];
    applier
        .widget_manager
        .widgets
        .insert(widget_node, Box::new(MeasuringWidget([100.0, 50.0])));
    let mut placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    let root = placed
        .iter_mut()
        .find(|node| node.node_id == applier.node_store.root)
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

    applier.publish_debug_snapshot(&placed);

    let state = state.read().unwrap();
    let widget = state
        .snapshot()
        .nodes
        .iter()
        .find(|node| node.id == 2)
        .unwrap();
    assert_eq!(widget.clip.widget_local.as_ref().unwrap().radius, 12.0);
    assert_eq!(
        widget.clip.widget_local.as_ref().unwrap().coordinate_space,
        "content-local"
    );
    assert_eq!(widget.clip.chain.len(), 1);
    assert_eq!(widget.clip.chain[0].node_id, 1);
    assert_eq!(widget.clip.effective.as_ref().unwrap().rect.width, 80.0);
    assert_eq!(widget.clip.device_scale, 1.0);
}

#[test]
fn releasing_outside_the_pressed_target_does_not_click() {
    let mut applier = interactive_applier();
    applier.handle_event(pointer(PointerPhase::Down, 20.0, 20.0, 1));
    applier.handle_event(pointer(PointerPhase::Up, 200.0, 200.0, 0));

    let click_count = applier
        .js
        .with(|ctx| ctx.eval::<usize, _>("globalThis.dispatched.filter((x) => x[1] === 1).length"))
        .expect("read click count");
    assert_eq!(click_count, 0);
}

#[test]
fn resize_observer_reports_initial_content_box_once() {
    let mut applier = interactive_applier();
    applier
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                __wabou_resize_observe(2);
                "#,
            )
        })
        .expect("install resize hook");

    assert!(applier.dispatch_resize_changes());
    assert!(!applier.dispatch_resize_changes());
    let changes = applier
        .js
        .with(|ctx| ctx.eval::<Vec<Vec<f32>>, _>("globalThis.resizeChanges"))
        .expect("read resize changes");
    assert_eq!(changes, vec![vec![2.0, 100.0, 50.0]]);
}

#[test]
fn devtools_snapshot_exposes_layout_and_redacts_secrets() {
    let mut applier = interactive_applier();
    let password = applier.atoms.borrow_mut().intern("password");
    applier.apply_op(&Op::SetAttribute {
        id: 2,
        name: password,
        value: "do-not-leak",
    });
    let state = wabou_devtools::DebugState::shared();
    applier.set_debug_state(state.clone());
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.last_viewport = (800, 600);
    applier.publish_debug_snapshot(&placed);

    let state = state.read().unwrap();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.status.viewport_width, 800);
    let button = snapshot.nodes.iter().find(|node| node.id == 2).unwrap();
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
    applier.invalidation.remove(InvalidationFlags::LAYOUT);
    applier.apply_op(&Op::SetTransform2D {
        id: 2,
        matrix: [1.0, 0.0, 0.0, 1.0, 12.5, -3.25],
    });

    assert!(!applier.invalidation.contains(InvalidationFlags::LAYOUT));
    let node = applier.node_store.solid_to_node[&2];
    assert_eq!(
        applier
            .node_store
            .tree
            .get_node_context(node)
            .unwrap()
            .runtime_transform,
        Some([1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
    );
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.rebuild_hit_geometry(&placed);
    assert_ne!(applier.input.hit_test(5.0, 20.0), Some(2));
    assert_eq!(applier.input.hit_test(32.5, 16.75), Some(2));

    let transform = applier.atoms.borrow_mut().intern("transform");
    applier.apply_op(&Op::SetStyle {
        id: 2,
        prop: transform,
        value: "translate(2px, 3px)",
    });
    let paint = applier.node_store.tree.get_node_context(node).unwrap();
    assert_eq!(
        paint.runtime_transform,
        Some([1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
    );
    assert_eq!(
        applier.runtime_transforms.get(&node),
        Some(&[1.0, 0.0, 0.0, 1.0, 12.5, -3.25])
    );
}

#[test]
fn protocol_shadows_apply_vello_parameters_without_string_parsing() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: Vec::new(),
    });
    applier.apply_op(&Op::SetShadows {
        id: 2,
        shadows: vec![crate::protocol::ShadowValue {
            offset_x: 3.0,
            offset_y: 7.0,
            spread: -2.0,
            std_dev: 5.5,
            color: 0x336699cc,
            radius: Some(11.0),
        }],
    });

    let node = applier.node_store.solid_to_node[&2];
    let paint = applier.node_store.tree.get_node_context(node).unwrap();
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
