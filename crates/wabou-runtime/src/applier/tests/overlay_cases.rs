use super::*;

#[test]
fn overflow_container_supports_wheel_and_selection_autoscroll() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (div, width, height, overflow_y, flex_shrink) = {
        let mut atoms = applier.atoms.borrow_mut();
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
            id,
            tag: div,
            attrs: Vec::new(),
        });
        applier.apply_op(&Op::SetStyle {
            id,
            prop: width,
            value: "100px",
        });
    }
    applier.apply_op(&Op::SetStyle {
        id: 2,
        prop: height,
        value: "100px",
    });
    applier.apply_op(&Op::SetStyle {
        id: 2,
        prop: overflow_y,
        value: "auto",
    });
    applier.apply_op(&Op::SetStyle {
        id: 3,
        prop: height,
        value: "300px",
    });
    applier.apply_op(&Op::SetStyle {
        id: 3,
        prop: flex_shrink,
        value: "0",
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 2,
        child: 3,
    });

    applier.apply_op(&Op::CreateText {
        id: 4,
        text: "scroll selectable",
    });
    applier.apply_op(&Op::AppendChild {
        parent: 3,
        child: 4,
    });
    applier.rebuild_layout_boxes();
    applier.inherit();

    let mut root_style = applier
        .node_store
        .tree
        .style(applier.node_store.root)
        .expect("root style")
        .clone();
    root_style.size.width = taffy::Dimension::length(800.0);
    root_style.size.height = taffy::Dimension::length(600.0);
    applier
        .node_store
        .tree
        .set_style(applier.node_store.root, root_style)
        .expect("viewport style");
    applier
        .node_store
        .tree
        .compute_layout(
            applier.node_store.root,
            taffy::geometry::Size {
                width: taffy::AvailableSpace::Definite(800.0),
                height: taffy::AvailableSpace::Definite(600.0),
            },
        )
        .expect("layout");
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.placed_rects = placed
        .iter()
        .map(|placed| (placed.node_id, placed.rect))
        .collect();
    applier.rebuild_hit_geometry(&placed);

    let container = applier.node_store.solid_to_node[&2];
    applier.invalidation.remove(InvalidationFlags::LAYOUT);
    assert_eq!(
        applier.node_store.tree.style(container).unwrap().overflow.y,
        taffy::Overflow::Scroll
    );
    assert_ne!(
        applier.input.hit_test(10.0, 150.0),
        Some(3),
        "overflow must clip hits"
    );
    applier.apply_op(&Op::SetTransform2D {
        id: 2,
        matrix: [1.0, 0.0, 0.0, 1.0, 200.0, 0.0],
    });
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.rebuild_hit_geometry(&placed);
    assert_eq!(applier.input.hit_test(210.0, 50.0), Some(3));
    assert_ne!(applier.input.hit_test(210.0, 150.0), Some(3));
    applier.apply_op(&Op::SetTransform2D {
        id: 2,
        matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    });
    let placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.rebuild_hit_geometry(&placed);
    assert!(applier.scroll_into_view(3));
    assert_eq!(applier.scroll_offsets[&container], [0.0, 100.0]);
    applier.scroll_offsets.insert(container, [0.0, 0.0]);
    let response = applier.handle_event(UiEvent::Wheel(wabou_shell::WheelEvent {
        position: Point { x: 10.0, y: 10.0 },
        delta_x: 0.0,
        delta_y: 50.0,
        modifiers: Modifiers::default(),
    }));
    assert!(response.handled);
    assert_eq!(applier.scroll_offsets[&container], [0.0, 50.0]);
    assert!(
        !applier.invalidation.contains(InvalidationFlags::LAYOUT),
        "scroll offsets must not invalidate intrinsic layout"
    );

    applier.apply_op(&Op::ScrollTo {
        id: 2,
        x: f32::NAN,
        y: 120.0,
    });
    assert_eq!(applier.scroll_offsets[&container], [0.0, 120.0]);
    applier.apply_op(&Op::ScrollBy {
        id: 2,
        x: 0.0,
        y: -20.0,
    });
    assert_eq!(applier.scroll_offsets[&container], [0.0, 100.0]);
    applier.apply_op(&Op::ScrollTo {
        id: 2,
        x: f32::NAN,
        y: -100.0,
    });
    assert_eq!(applier.scroll_offsets[&container], [0.0, 0.0]);

    let mut placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.scrollbar_activity.clear();
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
            .scrollbar_hits
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
        (applier.scroll_offsets[&container][1] - 102.0).abs() < 1.0,
        "34 thumb pixels should map through the shared geometry ratio"
    );
    let up = applier.handle_event(pointer(PointerPhase::Up, 95.0, 50.0, 0));
    assert!(up.handled);
    assert!(applier.scrollbar_drag.is_none());

    applier.scroll_offsets.insert(container, [0.0, 0.0]);
    let mut placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.update_scrollbar_visuals(&mut placed);
    applier.rebuild_hit_geometry(&placed);
    let track = applier.handle_event(pointer(PointerPhase::Down, 95.0, 80.0, 1));
    assert!(track.handled);
    assert_eq!(applier.scroll_offsets[&container][1], 100.0);

    applier.scroll_offsets.insert(container, [0.0, 0.0]);
    let mut tcx = TextContext::new();
    let mut placed = layout::flatten_with_scroll(
        &applier.node_store.tree,
        applier.node_store.root,
        &applier.scroll_offsets,
    );
    applier.placed_rects = placed
        .iter()
        .map(|placed| (placed.node_id, placed.rect))
        .collect();
    applier.prepare_text_selection(&mut placed, &mut tcx);
    let origin = applier.text_selection.selectable[&3].origin;
    applier.begin_text_selection(
        3,
        f64::from(origin[0] + 1.0),
        f64::from(origin[1] + 5.0),
        Modifiers::empty(),
    );
    applier.input.pointer_buttons = 1;
    applier.input.pointer_position = (200.0, 140.0);
    applier.extend_text_selection(None, 200.0, 140.0);
    // Model a cross-panel drag: the stable anchor is outside this
    // overflow container while the focus endpoint remains inside it.
    applier
        .text_selection
        .active
        .as_mut()
        .unwrap()
        .anchor_target = 1;
    applier.arm_text_selection_autoscroll();
    assert!(applier.animation_deadline().is_some());
    applier.text_selection.next_scroll = Some(Instant::now() - Duration::from_millis(1));
    assert!(applier.tick_text_selection_autoscroll());
    let first_scroll = applier.scroll_offsets[&container][1];
    assert!(first_scroll > 0.0);
    applier.text_selection.next_scroll = Some(Instant::now() - Duration::from_millis(1));
    assert!(applier.tick_text_selection_autoscroll());
    assert!(applier.scroll_offsets[&container][1] > first_scroll);
    applier.input.pointer_buttons = 0;
    applier.text_selection.next_scroll = Some(Instant::now() - Duration::from_millis(1));
    assert!(!applier.tick_text_selection_autoscroll());
    assert!(applier.text_selection.next_scroll.is_none());
    applier
        .text_selection
        .active
        .as_mut()
        .unwrap()
        .anchor_target = 3;
    assert_eq!(
        applier.selected_text().as_deref(),
        Some("scroll selectable")
    );
}

#[test]
fn later_overlay_content_blocks_an_underlying_scrollbar_attachment() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let view = applier.atoms.borrow_mut().intern("view");
    for id in [2, 3] {
        applier.apply_op(&Op::CreateElement {
            id,
            tag: view,
            attrs: vec![],
        });
    }
    let owner = applier.node_store.solid_to_node[&2];
    let overlay = applier.node_store.solid_to_node[&3];
    let root = applier.node_store.root;
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
    assert_eq!(applier.input.hit_test(95.0, 16.0), Some(3));
}

#[test]
fn modal_focus_scope_includes_later_portals_on_the_modal_plane() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, button, aria_modal) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("view"),
            atoms.intern("button"),
            atoms.intern("aria-modal"),
        )
    };
    for (id, tag, attrs) in [
        (2, button, vec![]),
        (3, view, vec![(aria_modal, "true")]),
        (4, button, vec![]),
        (5, view, vec![]),
        (6, button, vec![]),
    ] {
        applier.apply_op(&Op::CreateElement { id, tag, attrs });
    }
    for (parent, child) in [(1, 2), (1, 3), (3, 4), (1, 5), (5, 6)] {
        applier.apply_op(&Op::AppendChild { parent, child });
    }

    let root = applier.node_store.root;
    let node = |solid: u32, parent_node_id, plane| PlacedNode {
        node_id: applier.node_store.solid_to_node[&solid],
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
    let modal = applier.node_store.solid_to_node[&3];
    let portal = applier.node_store.solid_to_node[&5];
    let placed = vec![
        node(2, Some(root), OverlayPlane::Content),
        node(3, Some(root), OverlayPlane::Modal),
        node(4, Some(modal), OverlayPlane::Content),
        node(5, Some(root), OverlayPlane::Modal),
        node(6, Some(portal), OverlayPlane::Content),
    ];

    applier.rebuild_focus_order(&placed);
    assert_eq!(applier.input.focus_order, [4, 6]);
    assert!(!applier.input.focusable_targets.contains(&2));
}

#[test]
fn semantic_snapshot_promotes_modal_plane_and_keeps_focus_inside() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (button, view, role, aria_label, aria_modal) = {
        let mut atoms = applier.atoms.borrow_mut();
        (
            atoms.intern("button"),
            atoms.intern("view"),
            atoms.intern("role"),
            atoms.intern("aria-label"),
            atoms.intern("aria-modal"),
        )
    };
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: button,
        attrs: vec![(aria_label, "Background")],
    });
    applier.apply_op(&Op::CreateElement {
        id: 3,
        tag: view,
        attrs: vec![
            (role, "dialog"),
            (aria_label, "Settings"),
            (aria_modal, "true"),
        ],
    });
    applier.apply_op(&Op::CreateElement {
        id: 4,
        tag: button,
        attrs: vec![(aria_label, "Save")],
    });
    applier.apply_op(&Op::AppendChild {
        parent: 3,
        child: 4,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 3,
    });
    applier.rebuild_layout_boxes();
    applier.apply_op(&Op::SetOverlayPlane { id: 3, plane: 2 });
    applier.input.focused_target = Some(4);

    let root = applier.node_store.root;
    let background = applier.node_store.solid_to_node[&2];
    let modal = applier.node_store.solid_to_node[&3];
    let save = applier.node_store.solid_to_node[&4];
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
    assert_eq!(applier.input.focus_order, [4]);
    assert!(!applier.input.focusable_targets.contains(&2));
    // Semantic source order must not inherit the paint list's z/plane order.
    // This keeps indexed locators stable when presentation order changes.
    let semantic_placed = vec![placed[1].clone(), placed[2].clone(), placed[0].clone()];
    applier.rebuild_semantic_snapshot(&semantic_placed);
    let snapshot = &applier.projections.semantic_snapshot;
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [2, 3, 4]
    );
    assert_eq!(snapshot.root_children, vec![2, 3]);
    assert_eq!(snapshot.modal_root, Some(3));
    assert_eq!(snapshot.focus, Some(4));
    assert!(snapshot.nodes.iter().any(|node| {
        node.id == 3
            && node.role == SemanticRole::Dialog
            && node.label.as_deref() == Some("Settings")
    }));
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == 4)
            .unwrap()
            .bounds,
        [10.0, 5.0, 110.0, 105.0]
    );
    applier.apply_op(&Op::AddEventListener {
        id: 4,
        event_type: event::CLICK,
    });
    assert!(!applier.handle_semantic_action(SemanticAction::Click { target: 2 }));
    assert!(applier.handle_semantic_action(SemanticAction::Click { target: 4 }));
    applier.input.focused_target = None;
    assert!(applier.handle_semantic_action(SemanticAction::Focus { target: 4 }));
    assert_eq!(applier.input.focused_target, Some(4));

    applier.apply_op(&Op::CreateElement {
        id: 5,
        tag: view,
        attrs: vec![
            (role, "dialog"),
            (aria_label, "Confirm"),
            (aria_modal, "true"),
        ],
    });
    applier.apply_op(&Op::CreateElement {
        id: 6,
        tag: button,
        attrs: vec![(aria_label, "Continue")],
    });
    applier.apply_op(&Op::AppendChild {
        parent: 5,
        child: 6,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 5,
    });
    applier.apply_op(&Op::SetOverlayPlane { id: 5, plane: 2 });
    let confirm = applier.node_store.solid_to_node[&5];
    let continue_button = applier.node_store.solid_to_node[&6];
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
    assert_eq!(applier.projections.semantic_snapshot.modal_root, Some(5));
    assert_eq!(applier.projections.semantic_snapshot.focus, Some(5));
    assert!(
        !applier.handle_semantic_action(SemanticAction::Focus { target: 4 }),
        "an older modal must be inert while a newer modal is topmost"
    );
    assert!(applier.handle_semantic_action(SemanticAction::Focus { target: 6 }));
}

#[test]
fn presentation_role_flattens_its_semantic_children_without_changing_paint() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, button, role, aria_label, aria_modal) = {
        let mut atoms = applier.atoms.borrow_mut();
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
        applier.apply_op(&Op::CreateElement { id, tag, attrs });
    }
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 2,
        child: 3,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 3,
        child: 4,
    });
    applier.rebuild_layout_boxes();

    let root = applier.node_store.root;
    let presentation = applier.node_store.solid_to_node[&2];
    let dialog = applier.node_store.solid_to_node[&3];
    let close = applier.node_store.solid_to_node[&4];
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
    let snapshot = &applier.projections.semantic_snapshot;
    assert_eq!(snapshot.root_children, [3]);
    assert_eq!(snapshot.modal_root, Some(3));
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [3, 4]
    );
    assert_eq!(snapshot.nodes[0].children, [4]);
    assert!(placed.iter().any(|node| node.node_id == presentation));

    applier.apply_op(&Op::SetAttribute {
        id: 2,
        name: role,
        value: "none",
    });
    applier.rebuild_semantic_snapshot(&placed);
    let snapshot = &applier.projections.semantic_snapshot;
    assert_eq!(snapshot.root_children, [3]);
    assert_eq!(snapshot.modal_root, Some(3));
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        [3, 4]
    );
}

#[test]
fn semantic_idrefs_resolve_to_live_native_nodes() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (view, role, id, aria_label, aria_controls, aria_active) = {
        let mut atoms = applier.atoms.borrow_mut();
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
        applier.apply_op(&Op::CreateElement {
            id: node,
            tag: view,
            attrs,
        });
    }
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 3,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 3,
        child: 4,
    });
    applier.rebuild_layout_boxes();

    let root = applier.node_store.root;
    let combo = applier.node_store.solid_to_node[&2];
    let listbox = applier.node_store.solid_to_node[&3];
    let option = applier.node_store.solid_to_node[&4];
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
        .projections
        .semantic_snapshot
        .nodes
        .iter()
        .find(|node| node.id == 2)
        .expect("combobox semantic node");
    assert_eq!(combo.controls, [3]);
    assert_eq!(combo.active_descendant, Some(4));
}
