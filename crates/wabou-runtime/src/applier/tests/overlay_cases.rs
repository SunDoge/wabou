use super::*;

#[test]
fn overflow_container_supports_wheel_and_selection_autoscroll() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (div, width, height, overflow_y, flex_shrink) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("div"),
            atoms.intern("width"),
            atoms.intern("height"),
            atoms.intern("overflow-y"),
            atoms.intern("flex-shrink"),
        )
    };
    for id in [2, 3] {
        applier.apply_op(&Op::CreateElement {
            id: nk(id),
            tag: div,
        });
        if id == 3 {
            set_text_behavior(&mut applier, id);
        }
        applier.apply_op(&Op::SetStyle {
            id: nk(id),
            prop: width,
            value: "100px",
        });
    }
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: height,
        value: "100px",
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(2, 1),
        prop: overflow_y,
        value: "auto",
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::SCROLL,
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(3, 1),
        prop: height,
        value: "300px",
    });
    applier.apply_op(&Op::SetStyle {
        id: NodeKey::new(3, 1),
        prop: flex_shrink,
        value: "0",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });

    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(4, 1),
        text: "scroll selectable",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(3, 1),
        child: NodeKey::new(4, 1),
    });
    applier.rebuild_layout_boxes();
    applier.inherit();

    let mut root_style = applier
        .document
        .node_store
        .tree
        .style(applier.document.node_store.root)
        .expect("root style")
        .clone();
    root_style.size.width = taffy::Dimension::length(800.0);
    root_style.size.height = taffy::Dimension::length(600.0);
    applier
        .document
        .node_store
        .tree
        .set_style(applier.document.node_store.root, root_style)
        .expect("viewport style");
    applier
        .document
        .node_store
        .tree
        .compute_layout(
            applier.document.node_store.root,
            taffy::geometry::Size {
                width: taffy::AvailableSpace::Definite(800.0),
                height: taffy::AvailableSpace::Definite(600.0),
            },
        )
        .expect("layout");
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.interaction.scroll.placed_rects = placed
        .iter()
        .map(|placed| (placed.node_id, placed.rect))
        .collect();
    applier.rebuild_hit_geometry(&placed);

    let container = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    applier
        .interaction
        .scroll
        .offsets
        .insert(container, [0.0, 500.0]);
    assert!(applier.clamp_scroll_offsets(&placed));
    assert_eq!(
        applier.interaction.scroll.offsets[&container],
        [0.0, 200.0],
        "a shrinking virtual extent must not retain an unreachable offset"
    );
    assert_eq!(
        applier
            .interaction
            .scroll
            .pending_events
            .get(&NodeKey::new(2, 1)),
        Some(&[0.0, 200.0]),
        "the corrected offset must be observable by the JavaScript virtualizer"
    );
    applier
        .interaction
        .scroll
        .offsets
        .insert(container, [0.0, 0.0]);
    applier.interaction.scroll.pending_events.clear();
    applier
        .document
        .invalidation
        .remove(InvalidationFlags::LAYOUT);
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .style(container)
            .unwrap()
            .overflow
            .y,
        taffy::Overflow::Scroll
    );
    assert_ne!(
        applier.interaction.input.hit_test(10.0, 150.0),
        Some(NodeKey::new(3, 1)),
        "overflow must clip hits"
    );
    applier.apply_op(&Op::SetTransform2D {
        id: NodeKey::new(2, 1),
        matrix: [1.0, 0.0, 0.0, 1.0, 200.0, 0.0],
    });
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.rebuild_hit_geometry(&placed);
    assert_eq!(
        applier.interaction.input.hit_test(210.0, 50.0),
        Some(NodeKey::new(3, 1))
    );
    assert_ne!(
        applier.interaction.input.hit_test(210.0, 150.0),
        Some(NodeKey::new(3, 1))
    );
    applier.apply_op(&Op::SetTransform2D {
        id: NodeKey::new(2, 1),
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    });
    let placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.rebuild_hit_geometry(&placed);
    applier.apply_op(&Op::FocusNode { id: nk(3) });
    assert_eq!(applier.interaction.input.focused_target, Some(nk(3)));
    assert_eq!(applier.interaction.scroll.offsets[&container], [0.0, 100.0]);
    applier
        .interaction
        .scroll
        .offsets
        .insert(container, [0.0, 0.0]);
    let response = applier.handle_event(UiEvent::Wheel(wabou_shell::WheelEvent {
        position: Point { x: 10.0, y: 10.0 },
        delta_x: 0.0,
        delta_y: 50.0,
        modifiers: Modifiers::default(),
    }));
    assert!(response.handled);
    assert_eq!(applier.interaction.scroll.offsets[&container], [0.0, 50.0]);
    applier.publish_layout_metrics(&placed, 800, 600);
    let projected_scroll =
        applier.frame.projections.layout_metrics.borrow().nodes[&NodeKey::new(2, 1)].scroll;
    assert_eq!(projected_scroll.offset_x, 0.0);
    assert_eq!(projected_scroll.offset_y, 50.0);
    assert_eq!(projected_scroll.range_x, 0.0);
    assert_eq!(projected_scroll.range_y, 200.0);
    assert!(
        !applier
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT),
        "scroll offsets must not invalidate intrinsic layout"
    );

    applier.apply_op(&Op::ScrollTo {
        id: NodeKey::new(2, 1),
        x: f32::NAN,
        y: 120.0,
    });
    assert_eq!(applier.interaction.scroll.offsets[&container], [0.0, 120.0]);
    applier.apply_op(&Op::ScrollBy {
        id: NodeKey::new(2, 1),
        x: 0.0,
        y: -20.0,
    });
    assert_eq!(applier.interaction.scroll.offsets[&container], [0.0, 100.0]);
    applier.apply_op(&Op::ScrollTo {
        id: NodeKey::new(2, 1),
        x: f32::NAN,
        y: -100.0,
    });
    assert_eq!(applier.interaction.scroll.offsets[&container], [0.0, 0.0]);

    let mut placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.interaction.scroll.activity.clear();
    applier.update_scrollbar_visuals(&mut placed);
    applier.rebuild_hit_geometry(&placed);
    assert_eq!(applier.scrollbar_at(95.0, 16.0), None);
    let edge = applier.handle_event(pointer(PointerPhase::Move, 85.0, 50.0, 0));
    assert!(
        edge.request_redraw,
        "the scrollbar edge hot zone must wake auto visibility"
    );
    applier.update_scrollbar_visuals(&mut placed);
    applier.rebuild_hit_geometry(&placed);
    assert!(applier.scrollbar_at(95.0, 16.0).is_some());
    let down = applier.handle_event(pointer(PointerPhase::Down, 95.0, 16.0, 1));
    assert!(
        down.handled,
        "the native thumb must capture pointer down; hits={:?}",
        applier
            .interaction
            .scroll
            .hits
            .iter()
            .map(|hit| (
                hit.placed.rect,
                hit.placed.own_clip,
                hit.placed.scroll.range,
                wabou_shell::scrollbar::thumb(&hit.placed, ScrollAxis::Vertical)
            ))
            .collect::<Vec<_>>()
    );
    let moved = applier.handle_event(pointer(PointerPhase::Move, 95.0, 50.0, 1));
    assert!(moved.handled);
    assert!(
        (applier.interaction.scroll.offsets[&container][1] - 102.0).abs() < 1.0,
        "34 thumb pixels should map through the shared geometry ratio"
    );
    let up = applier.handle_event(pointer(PointerPhase::Up, 95.0, 50.0, 0));
    assert!(up.handled);
    assert!(applier.interaction.scroll.drag.is_none());

    applier
        .interaction
        .scroll
        .offsets
        .insert(container, [0.0, 0.0]);
    let mut placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.update_scrollbar_visuals(&mut placed);
    applier.rebuild_hit_geometry(&placed);
    let track = applier.handle_event(pointer(PointerPhase::Down, 95.0, 80.0, 1));
    assert!(track.handled);
    assert_eq!(applier.interaction.scroll.offsets[&container][1], 100.0);

    applier
        .interaction
        .scroll
        .offsets
        .insert(container, [0.0, 0.0]);
    let mut tcx = TextContext::new();
    let mut placed = layout::flatten_with_scroll(
        &applier.document.node_store.tree,
        applier.document.node_store.root,
        &applier.interaction.scroll.offsets,
    );
    applier.interaction.scroll.placed_rects = placed
        .iter()
        .map(|placed| (placed.node_id, placed.rect))
        .collect();
    applier.prepare_text_selection(&mut placed, &mut tcx);
    let origin = applier.interaction.text_selection.selectable[&NodeKey::new(3, 1)].origin;
    applier.begin_text_selection(
        nk(3),
        f64::from(origin[0] + 1.0),
        f64::from(origin[1] + 5.0),
        Modifiers::empty(),
    );
    applier.interaction.input.pointer_buttons = 1;
    applier.interaction.input.pointer_position = (200.0, 140.0);
    applier.extend_text_selection(None, 200.0, 140.0);
    // Model a cross-panel drag: the stable anchor is outside this
    // overflow container while the focus endpoint remains inside it.
    applier
        .interaction
        .text_selection
        .active
        .as_mut()
        .unwrap()
        .anchor_target = nk(1);
    applier.arm_text_selection_autoscroll();
    assert!(applier.animation_deadline().is_some());
    applier.interaction.text_selection.next_scroll =
        Some(Instant::now() - Duration::from_millis(1));
    assert!(applier.tick_text_selection_autoscroll());
    let first_scroll = applier.interaction.scroll.offsets[&container][1];
    assert!(first_scroll > 0.0);
    applier.interaction.text_selection.next_scroll =
        Some(Instant::now() - Duration::from_millis(1));
    assert!(applier.tick_text_selection_autoscroll());
    assert!(applier.interaction.scroll.offsets[&container][1] > first_scroll);
    applier.interaction.input.pointer_buttons = 0;
    applier.interaction.text_selection.next_scroll =
        Some(Instant::now() - Duration::from_millis(1));
    assert!(!applier.tick_text_selection_autoscroll());
    assert!(applier.interaction.text_selection.next_scroll.is_none());
    applier
        .interaction
        .text_selection
        .active
        .as_mut()
        .unwrap()
        .anchor_target = nk(3);
    assert_eq!(
        applier.selected_text().as_deref(),
        Some("scroll selectable")
    );
}

#[test]
fn later_overlay_content_blocks_an_underlying_scrollbar_attachment() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let view = applier.document.atoms.borrow_mut().intern("view");
    for id in [2, 3] {
        applier.apply_op(&Op::CreateElement {
            id: nk(id),
            tag: view,
        });
    }
    let owner = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let overlay = applier.document.node_store.solid_to_node[&NodeKey::new(3, 1)];
    let root = applier.document.node_store.root;
    let placed = |node_id, scroll| PlacedNode {
        node_id,
        parent_node_id: Some(root),
        depth: 1,
        rect: [0.0, 0.0, 100.0, 100.0],
        content_origin: [0.0, 0.0],
        content_size: [100.0, 100.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [0.0; 4],
        scroll,
        paint: Paint::default(),
    };
    let owner_scroll = layout::ScrollMetrics {
        port: [0.0, 0.0, 100.0, 100.0],
        scrollable: [false, true],
        range: [0.0, 900.0],
        offset: [0.0, 0.0],
        opacity: 1.0,
        interaction: 0,
    };
    let owner_placed = placed(owner, owner_scroll);
    applier.rebuild_hit_geometry(std::slice::from_ref(&owner_placed));
    assert!(applier.scrollbar_at(95.0, 16.0).is_some());

    let overlay_placed = placed(overlay, layout::ScrollMetrics::default());
    applier.rebuild_hit_geometry(&[owner_placed, overlay_placed]);
    assert_eq!(applier.scrollbar_at(95.0, 16.0), None);
    assert_eq!(
        applier.interaction.input.hit_test(95.0, 16.0),
        Some(NodeKey::new(3, 1))
    );
}

#[test]
fn focus_uses_explicit_wabou_contract_inside_modal_portals() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, button, lowercase_tabindex, disabled) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("button"),
            atoms.intern("tabindex"),
            atoms.intern("disabled"),
        )
    };
    for (id, tag) in [(2, button), (3, view), (4, button), (5, view), (6, button)] {
        applier.apply_op(&Op::CreateElement { id: nk(id), tag });
    }
    set_focus_contained(&mut applier, 3);
    for (parent, child) in [(1, 2), (1, 3), (3, 4), (1, 5), (5, 6)] {
        applier.apply_op(&Op::AppendChild {
            parent: nk(parent),
            child: nk(child),
        });
    }
    for id in [2, 4] {
        set_focus_order(&mut applier, id, 0);
    }
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(4, 1),
        name: disabled,
        value: "",
    });
    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(6, 1),
        name: lowercase_tabindex,
        value: "0",
    });

    let root = applier.document.node_store.root;
    let node = |solid: u32, parent_node_id, plane| PlacedNode {
        node_id: applier.document.node_store.solid_to_node[&nk(solid)],
        parent_node_id,
        depth: 1,
        rect: [0.0, 0.0, 100.0, 100.0],
        content_origin: [0.0, 0.0],
        content_size: [100.0, 100.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [0.0; 4],
        scroll: layout::ScrollMetrics::default(),
        paint: Paint {
            overlay_plane: plane,
            ..Paint::default()
        },
    };
    let modal = applier.document.node_store.solid_to_node[&NodeKey::new(3, 1)];
    let portal = applier.document.node_store.solid_to_node[&NodeKey::new(5, 1)];
    let placed = vec![
        node(2, Some(root), OverlayPlane::Content),
        node(3, Some(root), OverlayPlane::Modal),
        node(4, Some(modal), OverlayPlane::Content),
        node(5, Some(root), OverlayPlane::Modal),
        node(6, Some(portal), OverlayPlane::Content),
    ];

    applier.rebuild_focus_order(&placed);
    // Rust executes only the typed focus policy; it neither treats `disabled`
    // as focus policy nor accepts browser spelling aliases.
    assert_eq!(applier.interaction.input.focus_order, [nk(4)]);
    assert!(!applier.interaction.input.focusable_targets.contains(&nk(2)));
}

#[test]
fn semantic_snapshot_promotes_modal_plane_and_keeps_focus_inside() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (button, view, role, aria_label, aria_modal) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("button"),
            atoms.intern("view"),
            atoms.intern("role"),
            atoms.intern("aria-label"),
            atoms.intern("aria-modal"),
        )
    };
    create_element_with_attrs(&mut applier, 2, button, &[(aria_label, "Background")]);
    create_element_with_attrs(
        &mut applier,
        3,
        view,
        &[
            (role, "dialog"),
            (aria_label, "Settings"),
            (aria_modal, "true"),
        ],
    );
    set_focus_contained(&mut applier, 3);
    create_element_with_attrs(&mut applier, 4, button, &[(aria_label, "Save")]);
    set_focus_order(&mut applier, 2, 0);
    set_focus_order(&mut applier, 4, 0);
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(3, 1),
        child: NodeKey::new(4, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(3, 1),
    });
    applier.rebuild_layout_boxes();
    applier.apply_op(&Op::SetOverlayPlane {
        id: NodeKey::new(3, 1),
        plane: 2,
    });
    applier.interaction.input.focused_target = Some(NodeKey::new(4, 1));

    let root = applier.document.node_store.root;
    let background = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let modal = applier.document.node_store.solid_to_node[&NodeKey::new(3, 1)];
    let save = applier.document.node_store.solid_to_node[&NodeKey::new(4, 1)];
    let paint = |plane| Paint {
        overlay_plane: plane,
        ..Paint::default()
    };
    let node = |node_id, parent_node_id, depth, paint| PlacedNode {
        node_id,
        parent_node_id,
        depth,
        rect: [0.0, 0.0, 100.0, 100.0],
        content_origin: [0.0, 0.0],
        content_size: [100.0, 100.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [0.0; 4],
        scroll: layout::ScrollMetrics::default(),
        paint,
    };
    let mut save_paint = paint(OverlayPlane::Content);
    save_paint.runtime_transform = Some([1.0, 0.0, 0.0, 1.0, 10.0, 5.0]);
    let placed = vec![
        node(background, Some(root), 1, paint(OverlayPlane::Content)),
        node(modal, Some(root), 1, paint(OverlayPlane::Modal)),
        node(save, Some(modal), 2, save_paint),
    ];
    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_focus_order(&placed);
    assert_eq!(applier.interaction.input.focus_order, [nk(4)]);
    assert!(!applier.interaction.input.focusable_targets.contains(&nk(2)));
    // Semantic source order must not inherit the paint list's z/plane order.
    // This keeps indexed locators stable when presentation order changes.
    let semantic_placed = vec![placed[1].clone(), placed[2].clone(), placed[0].clone()];
    applier.rebuild_semantic_snapshot(&semantic_placed);
    let snapshot = &applier.frame.projections.semantic_snapshot;
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [sk(2), sk(3), sk(4)]
    );
    assert_eq!(snapshot.root_children, vec![sk(2), sk(3)]);
    assert_eq!(snapshot.modal_root, Some(sk(3)));
    assert_eq!(snapshot.focus, Some(sk(4)));
    assert!(snapshot.nodes.iter().any(|node| {
        node.id == sk(3)
            && node.role == SemanticRole::Dialog
            && node.label.as_deref() == Some("Settings")
    }));
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == sk(4))
            .unwrap()
            .bounds,
        [10.0, 5.0, 110.0, 105.0]
    );
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(4, 1),
        event_type: event::CLICK,
    });
    assert!(!applier.handle_semantic_action(SemanticAction::Click { target: sk(2) }));
    assert!(applier.handle_semantic_action(SemanticAction::Click { target: sk(4) }));
    applier.interaction.input.focused_target = None;
    assert!(applier.handle_semantic_action(SemanticAction::Focus { target: sk(4) }));
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(4, 1))
    );

    create_element_with_attrs(
        &mut applier,
        5,
        view,
        &[
            // Semantic focus must use the explicit modal root rather than
            // guessing a focus target from descendant roles.
            (role, "group"),
            (aria_label, "Confirm"),
            (aria_modal, "true"),
        ],
    );
    create_element_with_attrs(&mut applier, 6, button, &[(aria_label, "Continue")]);
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(5, 1),
        child: NodeKey::new(6, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(5, 1),
    });
    applier.apply_op(&Op::SetOverlayPlane {
        id: NodeKey::new(5, 1),
        plane: 2,
    });
    let confirm = applier.document.node_store.solid_to_node[&NodeKey::new(5, 1)];
    let continue_button = applier.document.node_store.solid_to_node[&NodeKey::new(6, 1)];
    let mut continue_paint = paint(OverlayPlane::Content);
    continue_paint.runtime_transform = Some([1.0, 0.0, 0.0, 1.0, 20.0, 10.0]);
    let placed = vec![
        node(background, Some(root), 1, paint(OverlayPlane::Content)),
        node(modal, Some(root), 1, paint(OverlayPlane::Modal)),
        node(save, Some(modal), 2, Paint::default()),
        node(confirm, Some(root), 1, paint(OverlayPlane::Modal)),
        node(continue_button, Some(confirm), 2, continue_paint),
    ];
    applier.rebuild_hit_geometry(&placed);
    applier.rebuild_semantic_snapshot(&placed);
    assert_eq!(
        applier.frame.projections.semantic_snapshot.modal_root,
        Some(sk(5))
    );
    assert_eq!(
        applier.frame.projections.semantic_snapshot.focus,
        Some(sk(5))
    );
    assert!(
        !applier.handle_semantic_action(SemanticAction::Focus { target: sk(4) }),
        "an older modal must be inert while a newer modal is topmost"
    );
    assert!(applier.handle_semantic_action(SemanticAction::Focus { target: sk(6) }));
}

#[test]
fn presentation_role_flattens_its_semantic_children_without_changing_paint() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, button, role, aria_label, aria_modal) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("button"),
            atoms.intern("role"),
            atoms.intern("aria-label"),
            atoms.intern("aria-modal"),
        )
    };
    for (id, tag, attrs) in [
        (2, view, vec![(role, "presentation"), (aria_modal, "true")]),
        (
            3,
            view,
            vec![
                (role, "dialog"),
                (aria_label, "Popover"),
                (aria_modal, "true"),
            ],
        ),
        (4, button, vec![(aria_label, "Close")]),
    ] {
        create_element_with_attrs(&mut applier, id, tag, &attrs);
    }
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(3, 1),
        child: NodeKey::new(4, 1),
    });
    applier.rebuild_layout_boxes();

    let root = applier.document.node_store.root;
    let presentation = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let dialog = applier.document.node_store.solid_to_node[&NodeKey::new(3, 1)];
    let close = applier.document.node_store.solid_to_node[&NodeKey::new(4, 1)];
    let placed = [
        (presentation, Some(root), 1),
        (dialog, Some(presentation), 2),
        (close, Some(dialog), 3),
    ]
    .map(|(node_id, parent_node_id, depth)| PlacedNode {
        node_id,
        parent_node_id,
        depth,
        rect: [0.0, 0.0, 100.0, 40.0],
        content_origin: [0.0, 0.0],
        content_size: [100.0, 40.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [0.0; 4],
        scroll: layout::ScrollMetrics::default(),
        paint: Paint {
            overlay_plane: if node_id == presentation {
                OverlayPlane::Modal
            } else {
                OverlayPlane::Content
            },
            ..Paint::default()
        },
    });

    applier.rebuild_semantic_snapshot(&placed);
    let snapshot = &applier.frame.projections.semantic_snapshot;
    assert_eq!(snapshot.root_children, [sk(3)]);
    assert_eq!(snapshot.modal_root, Some(sk(3)));
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [sk(3), sk(4)]
    );
    assert_eq!(snapshot.nodes[0].children, [sk(4)]);
    assert!(placed.iter().any(|node| node.node_id == presentation));

    applier.apply_op(&Op::SetAttribute {
        id: NodeKey::new(2, 1),
        name: role,
        value: "none",
    });
    applier.rebuild_semantic_snapshot(&placed);
    let snapshot = &applier.frame.projections.semantic_snapshot;
    assert_eq!(snapshot.root_children, [sk(3)]);
    assert_eq!(snapshot.modal_root, Some(sk(3)));
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [sk(3), sk(4)]
    );
}

#[test]
fn semantic_idrefs_resolve_to_live_native_nodes() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, role, id, aria_label, aria_controls, aria_active) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("role"),
            atoms.intern("id"),
            atoms.intern("aria-label"),
            atoms.intern("aria-controls"),
            atoms.intern("aria-activedescendant"),
        )
    };
    for (node, attrs) in [
        (
            2,
            vec![
                (role, "combobox"),
                (aria_label, "Workspace"),
                (aria_controls, "workspace-options missing"),
                (aria_active, "workspace-beta"),
            ],
        ),
        (
            3,
            vec![
                (role, "listbox"),
                (id, "workspace-options"),
                (aria_label, "Workspace"),
            ],
        ),
        (
            4,
            vec![
                (role, "option"),
                (id, "workspace-beta"),
                (aria_label, "Beta"),
            ],
        ),
    ] {
        create_element_with_attrs(&mut applier, node, view, &attrs);
    }
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(3, 1),
        child: NodeKey::new(4, 1),
    });
    applier.rebuild_layout_boxes();

    let root = applier.document.node_store.root;
    let combo = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let listbox = applier.document.node_store.solid_to_node[&NodeKey::new(3, 1)];
    let option = applier.document.node_store.solid_to_node[&NodeKey::new(4, 1)];
    let placed = [
        (combo, Some(root), 1),
        (listbox, Some(root), 1),
        (option, Some(listbox), 2),
    ]
    .map(|(node_id, parent_node_id, depth)| PlacedNode {
        node_id,
        parent_node_id,
        depth,
        rect: [0.0, 0.0, 100.0, 40.0],
        content_origin: [0.0, 0.0],
        content_size: [100.0, 40.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [0.0; 4],
        scroll: layout::ScrollMetrics::default(),
        paint: Paint::default(),
    });

    applier.rebuild_semantic_snapshot(&placed);
    let combo = applier
        .frame
        .projections
        .semantic_snapshot
        .nodes
        .iter()
        .find(|node| node.id == sk(2))
        .expect("combobox semantic node");
    assert_eq!(combo.controls, [sk(3)]);
    assert_eq!(combo.active_descendant, Some(sk(4)));

    let state = wabou_devtools::DebugState::shared();
    applier.set_debug_state(state.clone());
    applier.publish_debug_snapshot(&placed, &mut TextContext::new());
    let state = state.read().unwrap();
    let combo = state
        .snapshot()
        .nodes
        .iter()
        .find(|node| node.id == nk(2))
        .expect("combobox debug node");
    let semantic = combo.semantic.as_ref().expect("combobox projection");
    assert_eq!(semantic.controls, [nk(3)]);
    assert_eq!(semantic.active_descendant, Some(nk(4)));
}

#[test]
fn semantic_projection_separates_explicit_roles_from_text_content() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, role, value, aria_current, aria_haspopup, aria_modal) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("role"),
            atoms.intern("value"),
            atoms.intern("aria-current"),
            atoms.intern("aria-haspopup"),
            atoms.intern("aria-modal"),
        )
    };
    create_element_with_attrs(
        &mut applier,
        2,
        view,
        &[
            (role, "button"),
            (value, "browser-style fallback"),
            (aria_current, "date"),
            (aria_haspopup, "listbox"),
            (aria_modal, "true"),
        ],
    );
    applier.apply_op(&Op::CreateText {
        id: NodeKey::new(3, 1),
        text: "unowned text",
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.rebuild_layout_boxes();

    let root = applier.document.node_store.root;
    let input = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    let text = applier.document.node_store.solid_to_node[&NodeKey::new(3, 1)];
    let placed = [input, text].map(|node_id| PlacedNode {
        node_id,
        parent_node_id: Some(if node_id == text { input } else { root }),
        depth: if node_id == text { 2 } else { 1 },
        rect: [0.0, 0.0, 100.0, 40.0],
        content_origin: [0.0, 0.0],
        content_size: [100.0, 40.0],
        clip: None,
        clip_radius: 0.0,
        clip_depth: None,
        own_clip: None,
        own_clip_radius: 0.0,
        border_widths: [0.0; 4],
        scroll: layout::ScrollMetrics::default(),
        paint: Paint {
            // A role-bearing text surface gets an accessible name from its
            // content, but content must not also become a control value.
            text: (node_id == input).then(|| "unowned text".into()),
            ..Paint::default()
        },
    });

    applier.rebuild_semantic_snapshot(&placed);
    let snapshot = &applier.frame.projections.semantic_snapshot;
    let input = snapshot
        .nodes
        .iter()
        .find(|node| node.id == sk(2))
        .expect("explicit button");
    assert_eq!(input.role, SemanticRole::Button);
    assert_eq!(input.value, None);
    assert_eq!(input.label.as_deref(), Some("unowned text"));
    assert_eq!(input.states.current, Some(SemanticCurrent::Date));
    assert_eq!(input.states.popup, Some(SemanticPopup::ListBox));
    assert_eq!(input.states.modal, Some(true));
    let text = snapshot
        .nodes
        .iter()
        .find(|node| node.id == sk(3))
        .expect("unowned text node");
    assert_eq!(text.role, SemanticRole::Generic);
}
