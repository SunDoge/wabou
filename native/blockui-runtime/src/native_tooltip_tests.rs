use super::*;
use blockui::ActionToken;
use blockui::{Action, Button, Column, Cx, IntoView, State, StateSpec, Text, Tooltip};
const TEXT: State<String> =
    State::<String>::with_default("hint.text", "Invite a teammate to this project.");
const SHOW: State<bool> = State::<bool>::new("hint.show", true);
const ENABLED: State<bool> = State::<bool>::new("hint.enabled", true);
const UPDATE: Action = Action::new("hint.update", |cx, _| TEXT.set(cx, "邀请团队成员 🦀"));
const LONG: Action = Action::new("hint.long", |cx, _| {
    TEXT.set(cx, &"Long hint 邀请 🦀\n".repeat(100))
});
const HIDE: Action = Action::new("hint.hide", |cx, _| SHOW.set(cx, false));
const DISABLE: Action = Action::new("hint.disable", |cx, _| ENABLED.set(cx, false));
struct Fixture;
impl blockui::Actor for Fixture {
    const STATE: &'static [StateSpec] = &[TEXT.spec(), SHOW.spec(), ENABLED.spec()];
    const ACTIONS: &'static [Action] = &[UPDATE, LONG, HIDE, DISABLE];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        let mut children = vec![Text::new("A little context").key("title").into_view()];
        if SHOW.get(cx)? {
            children.push(
                Tooltip::new(TEXT).key("hint").trigger(
                    Button {
                        enabled: ENABLED.get(cx)?,
                        ..Button::label("Invite").key("invite")
                    }
                    .on_click(|_| Ok(())),
                )?,
            );
        }
        children.push(
            Button::label("Continue")
                .key("continue")
                .on_click(|_| Ok(())),
        );
        Ok(Column {
            padding: 24.0,
            gap: 16.0,
            ..Default::default()
        }
        .key("root")
        .children(children))
    }
}
fn source() -> Result<NativeSource, String> {
    let mut s = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    s.motion_manual_time = Some(Duration::ZERO);
    Ok(s)
}
fn hint() -> u64 {
    blockui::Key::named("hint").scoped(key("hint")).0
}
#[test]
fn delayed_hover_and_content_transfer_do_not_evaluate_actor_bindings() -> Result<(), String> {
    let mut s = source()?;
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 600, 400);
    let before = s.actor.projected_view().bindings[&hint()].evaluations;
    let trigger = s.widget_layout_rect(key("invite")).unwrap();
    s.pointer_position = Some(shell::Point {
        x: f64::from(trigger[0] + 10.0),
        y: f64::from(trigger[1] + 10.0),
    });
    s.refresh_hover();
    assert!(s.animation_deadline().is_some());
    s.motion_manual_time = Some(Duration::from_millis(699));
    s.build_frame(&mut tcx, 600, 400);
    assert_eq!(s.tooltip.visible(), None);
    s.motion_manual_time = Some(Duration::from_millis(700));
    s.build_frame(&mut tcx, 600, 400);
    assert_eq!(s.tooltip.visible(), Some(hint()));
    let r = s
        .placed
        .iter()
        .find(|n| n.node_id == s.ids[&hint()])
        .unwrap()
        .rect;
    assert!(
        r[0] >= 16.0 && r[2] <= 584.0 && r[1] >= 16.0 && r[3] <= 384.0,
        "{r:?}"
    );
    s.pointer_position = Some(shell::Point {
        x: f64::from((r[0] + r[2]) * 0.5),
        y: f64::from((r[1] + r[3]) * 0.5),
    });
    s.refresh_hover();
    assert_eq!(s.tooltip.visible(), Some(hint()));
    assert_eq!(s.tooltip.deadline(), None);
    s.pointer_position = None;
    s.refresh_hover();
    s.motion_manual_time = Some(Duration::from_millis(800));
    s.build_frame(&mut tcx, 600, 400);
    assert_eq!(s.tooltip.visible(), None);
    assert_eq!(
        s.actor.projected_view().bindings[&hint()].evaluations,
        before
    );
    Ok(())
}
#[test]
fn focus_live_text_removal_and_disablement_reconcile() -> Result<(), String> {
    let mut s = source()?;
    s.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 360, 400);
    let closed = s.semantic_snapshot().unwrap();
    assert!(
        closed
            .nodes
            .iter()
            .find(|n| n.id == key("invite"))
            .unwrap()
            .described_by
            .is_empty()
    );
    assert!(!closed.nodes.iter().any(|n| n.id == hint()));
    s.set_focus(Some(key("invite")));
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(s.tooltip.visible(), Some(hint()));
    assert_eq!(s.focus, Some(key("invite")));
    let snapshot = s.semantic_snapshot().unwrap();
    let control = snapshot
        .nodes
        .iter()
        .find(|n| n.id == key("invite"))
        .unwrap();
    assert_eq!(control.label.as_deref(), Some("Invite"));
    assert_eq!(control.described_by, vec![hint()]);
    assert!(snapshot.revision > closed.revision);
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(
        s.semantic_snapshot().unwrap().revision,
        snapshot.revision,
        "stable frames do not republish descriptions"
    );
    assert!(
        s.semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|n| n.id == hint() && n.role == shell::SemanticRole::Tooltip)
    );
    s.actor.dispatch(Message {
        action: UPDATE.encode(0).0,
        value: 0,
    })?;
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(
        s.actor.projected_view().bindings[&hint()].text,
        "邀请团队成员 🦀"
    );
    let updated = s.semantic_snapshot().unwrap();
    assert_eq!(
        updated
            .nodes
            .iter()
            .find(|n| n.id == hint())
            .unwrap()
            .label
            .as_deref(),
        Some("邀请团队成员 🦀")
    );
    assert_eq!(
        updated
            .nodes
            .iter()
            .find(|n| n.id == key("invite"))
            .unwrap()
            .label
            .as_deref(),
        Some("Invite")
    );
    s.dismiss_tooltip();
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(s.tooltip.visible(), None);
    let dismissed = s.semantic_snapshot().unwrap();
    assert!(
        dismissed
            .nodes
            .iter()
            .find(|n| n.id == key("invite"))
            .unwrap()
            .described_by
            .is_empty()
    );
    assert!(!dismissed.nodes.iter().any(|n| n.id == hint()));
    s.set_focus(Some(key("continue")));
    s.set_focus(Some(key("invite")));
    assert_eq!(s.tooltip.visible(), Some(hint()));
    s.actor.dispatch(Message {
        action: DISABLE.encode(0).0,
        value: 0,
    })?;
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(s.tooltip.visible(), None);
    assert!(
        s.semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|n| n.id == key("invite"))
            .unwrap()
            .described_by
            .is_empty()
    );
    s.actor.dispatch(Message {
        action: HIDE.encode(0).0,
        value: 0,
    })?;
    s.build_frame(&mut tcx, 360, 400);
    assert_eq!(s.tooltip.deadline(), None);
    Ok(())
}
#[test]
fn touch_activation_focus_does_not_open_and_escape_keeps_keyboard_focus() -> Result<(), String> {
    let mut s = source()?;
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 600, 400);
    let r = s.widget_layout_rect(key("invite")).unwrap();
    let point = shell::Point {
        x: f64::from(r[0] + 10.0),
        y: f64::from(r[1] + 10.0),
    };
    for phase in [PointerPhase::Down, PointerPhase::Up] {
        s.handle_event(UiEvent::Pointer(shell::PointerEvent {
            phase,
            position: point,
            buttons: if phase == PointerPhase::Down { 1 } else { 0 },
            button: Some(shell::PointerButton::Primary),
            modifiers: Default::default(),
            properties: shell::PointerProperties {
                pointer_type: shell::PointerType::Touch,
                ..Default::default()
            },
        }));
    }
    s.build_frame(&mut tcx, 600, 400);
    assert_eq!(s.focus, Some(key("invite")));
    assert_eq!(s.tooltip.visible(), None);
    s.set_focus(Some(key("continue")));
    s.set_focus(Some(key("invite")));
    assert_eq!(s.tooltip.visible(), Some(hint()));
    s.handle_event(UiEvent::Key(shell::KeyEvent {
        key: "Escape".into(),
        key_without_modifiers: "Escape".into(),
        phase: shell::KeyPhase::Down,
        repeat: false,
        synthetic: false,
        modifiers: Default::default(),
        code: Default::default(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
    }));
    s.build_frame(&mut tcx, 600, 400);
    assert_eq!(s.focus, Some(key("invite")));
    assert_eq!(s.tooltip.visible(), None);
    Ok(())
}

#[test]
fn composed_actors_scope_descriptions_to_the_current_hint() -> Result<(), String> {
    use crate::{ActorId, Runtime, native_model::NativeFamily};
    struct Parent;
    impl blockui::Actor for Parent {
        const STATE: &'static [StateSpec] = &[];
        const ACTIONS: &'static [Action] = &[];
        fn view(_: &Cx<'_>) -> blockui::Result<blockui::View> {
            Ok(Column::default().key("parent").children(
                ["first", "second"]
                    .into_iter()
                    .map(|name| {
                        blockui::ChildActor {
                            key: name.into(),
                            implementation: blockui::Implementation::named("hint-child"),
                        }
                        .into_view()
                    })
                    .collect(),
            ))
        }
    }
    let directory =
        std::env::temp_dir().join(format!("blockui-hint-family-{}", std::process::id()));
    let mut runtime = Runtime::new(&directory);
    runtime.register_for_test::<Fixture>("hint-child")?;
    let root = ActorId(872);
    runtime.spawn_for_test::<Parent>(root)?;
    let first = runtime.child(root, key("first")).unwrap();
    let second = runtime.child(root, key("second")).unwrap();
    let model = NativeFamily::new(runtime, root)?;
    let a = model.widget(first, key("invite")).unwrap();
    let b = model.widget(second, key("invite")).unwrap();
    let ha = model.widget(first, hint()).unwrap();
    let hb = model.widget(second, hint()).unwrap();
    assert_ne!(ha, hb);
    let mut s = NativeSource::without_watch(model)?;
    s.motion_manual_time = Some(Duration::ZERO);
    s.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 600, 800);
    for (trigger, hint, other) in [(a, ha, b), (b, hb, a)] {
        s.set_focus(Some(trigger));
        s.build_frame(&mut tcx, 600, 800);
        let snapshot = s.semantic_snapshot().unwrap();
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .find(|n| n.id == trigger)
                .unwrap()
                .described_by,
            vec![hint]
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .find(|n| n.id == other)
                .unwrap()
                .described_by
                .is_empty()
        );
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|n| n.role == shell::SemanticRole::Tooltip)
                .count(),
            1
        );
        assert_eq!(s.actor.view().bindings[&hint].node.tooltip_anchor, trigger);
    }
    drop(s);
    assert_eq!(
        std::fs::read_dir(&directory)
            .map_err(|e| e.to_string())?
            .count(),
        0
    );
    std::fs::remove_dir(directory).map_err(|e| e.to_string())?;
    Ok(())
}
#[test]
#[ignore = "explicit offscreen Vello capture"]
fn tooltip_visuals() -> Result<(), String> {
    let mut s = source()?;
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 600, 400);
    s.set_focus(Some(key("invite")));
    let nodes = s.build_frame(&mut tcx, 600, 400);
    let mut scene = shell::Scene::new();
    shell::scene::build_scene_scaled(&mut scene, &nodes, &mut tcx, 600, 400, s.base_color(), 1.0);
    let image = shell::renderer::render_to_image(&scene, 600, 400, s.base_color())
        .map_err(|e| e.to_string())?;
    image
        .save("../../reports/native-tooltip.png")
        .map_err(|e| e.to_string())?;
    s.actor.dispatch(Message {
        action: LONG.encode(0).0,
        value: 0,
    })?;
    for (name, offset) in [("long", 0.0), ("long-end", 100000.0)] {
        s.actor.view_mut().renderer.get_mut(&hint()).unwrap().scroll = offset;
        let nodes = s.build_frame(&mut tcx, 360, 240);
        let mut scene = shell::Scene::new();
        shell::scene::build_scene_scaled(
            &mut scene,
            &nodes,
            &mut tcx,
            360,
            240,
            s.base_color(),
            1.0,
        );
        let image = shell::renderer::render_to_image(&scene, 360, 240, s.base_color())
            .map_err(|e| e.to_string())?;
        image
            .save(format!("../../reports/native-tooltip-{name}.png"))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[test]
#[ignore = "requires a freshly built Gallery DLL; explicit real replacement proof"]
fn tooltip_survives_real_dll_replacement() -> Result<(), String> {
    let path = std::env::var_os("BLOCKUI_TOOLTIP_LIBRARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../../target/debug/libgallery.so"))
        .canonicalize()
        .map_err(|e| e.to_string())?;
    struct Root(PathBuf);
    impl Drop for Root {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let root = Root(std::env::temp_dir().join(format!(
            "blockui-tooltip-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    let mut actor = Actor::new(951, &root.0)?;
    actor.reload(&path)?;
    actor.dispatch_widget(key("nav.12"))?;
    let mut s = NativeSource::without_watch(actor)?;
    s.set_semantics_enabled(true);
    s.motion_manual_time = Some(Duration::ZERO);
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 1200, 1000);
    let invite = key("tooltip.invite");
    let hint = s.actor.projected_view().bindings[&invite]
        .node
        .semantics
        .described_by;
    let r = s.widget_layout_rect(invite).unwrap();
    s.pointer_position = Some(shell::Point {
        x: f64::from(r[0] + 10.0),
        y: f64::from(r[1] + 10.0),
    });
    s.refresh_hover();
    let deadline = s.tooltip.deadline();
    assert!(deadline.is_some());
    s.replace(&path)?;
    assert_eq!(s.tooltip.deadline(), deadline);
    s.motion_manual_time = Some(Duration::from_millis(700));
    s.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(s.tooltip.visible(), Some(hint));
    let address = s.actor.state_address();
    for i in 0..8 {
        let text = format!("Reloaded hint {i} 🦀");
        s.actor.edit_text(key("tooltip.text"), &text)?;
        s.replace(&path)?;
        s.build_frame(&mut tcx, 1200, 1000);
        assert_eq!(s.tooltip.visible(), Some(hint));
        assert_eq!(s.actor.projected_view().bindings[&hint].text, text);
        let semantic = s.semantic_snapshot().unwrap();
        assert_eq!(
            semantic
                .nodes
                .iter()
                .find(|n| n.id == invite)
                .unwrap()
                .described_by,
            vec![hint]
        );
        assert_eq!(
            semantic
                .nodes
                .iter()
                .find(|n| n.id == hint)
                .unwrap()
                .label
                .as_deref(),
            Some(text.as_str())
        );
        assert_eq!(s.actor.state_address(), address);
        assert_eq!(s.actor.artifact_count(), 1);
        #[cfg(any(windows, target_os = "linux"))]
        assert_eq!(s.actor.mapped_implementation_count()?, 1);
        s.actor.dispatch_widget(invite)?;
        assert_eq!(s.actor.text("gallery.saved"), "Invitation sent.");
    }
    assert!(s.replace(&root.0.join("missing-library")).is_err());
    s.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(s.tooltip.visible(), Some(hint));
    s.actor.dispatch_widget(key("nav.0"))?;
    s.replace(&path)?;
    s.build_frame(&mut tcx, 1200, 1000);
    assert_eq!(s.tooltip.visible(), None);
    assert_eq!(s.tooltip.deadline(), None);
    println!(
        "PASS pending/visible Tooltip across 10 real replacements, native description links/live Unicode text/current action, rejected load continuity, removal and one generation ({} allocated bytes)",
        s.actor.artifact_allocated_bytes()?
    );
    drop(s);
    assert_eq!(
        std::fs::read_dir(&root.0)
            .map_err(|e| e.to_string())?
            .count(),
        0
    );
    Ok(())
}

#[test]
fn long_hint_is_bounded_scrollable_and_keeps_full_description() -> Result<(), String> {
    let mut s = source()?;
    s.set_semantics_enabled(true);
    s.actor.dispatch(Message {
        action: LONG.encode(0).0,
        value: 0,
    })?;
    let mut tcx = TextContext::new();
    s.build_frame(&mut tcx, 360, 240);
    s.set_focus(Some(key("invite")));
    s.build_frame(&mut tcx, 360, 240);
    let id = s.ids[&hint()];
    let placed = s.placed.iter().find(|n| n.node_id == id).unwrap();
    assert!(
        placed.rect[1] >= 16.0 && placed.rect[3] <= 224.0,
        "{:?}",
        placed.rect
    );
    assert!(placed.scroll.range[1] > 0.0);
    assert_eq!(placed.own_clip, Some(placed.rect));
    let position = shell::Point {
        x: f64::from(placed.rect[0] + 12.0),
        y: f64::from(placed.rect[1] + 12.0),
    };
    let before = s.actor.projected_view().bindings[&hint()].evaluations;
    s.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position,
        delta_x: 0.0,
        delta_y: 100000.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    s.build_frame(&mut tcx, 360, 240);
    let placed = s.placed.iter().find(|n| n.node_id == id).unwrap();
    assert_eq!(placed.scroll.offset[1], placed.scroll.range[1]);
    assert!(placed.content_origin[1] < placed.rect[1]);
    assert_eq!(
        s.actor.projected_view().bindings[&hint()].evaluations,
        before
    );
    assert_eq!(s.focus, Some(key("invite")));
    let snapshot = s.semantic_snapshot().unwrap();
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == hint())
            .unwrap()
            .label
            .as_deref(),
        Some("Long hint 邀请 🦀\n".repeat(100).as_str())
    );
    s.actor.dispatch(Message {
        action: UPDATE.encode(0).0,
        value: 0,
    })?;
    s.build_frame(&mut tcx, 360, 600);
    let placed = s.placed.iter().find(|n| n.node_id == id).unwrap();
    assert_eq!(placed.scroll.offset[1], 0.0);
    assert_eq!(placed.scroll.range[1], 0.0);
    for (width, height) in [(1, 1), (32, 240), (360, 32)] {
        s.build_frame(&mut tcx, width, height);
        assert!(s.placed.iter().all(|node| node.node_id != id));
    }
    s.build_frame(&mut tcx, 360, 240);
    assert!(s.placed.iter().any(|node| node.node_id == id));
    Ok(())
}
