use super::*;
use blockui::{Cx, IntoView, Popover, State, StateSpec};
const OPEN: State<bool> = State::<bool>::new("popover.open", false);
const PLACEMENT: State<i64> = State::<i64>::new("popover.placement", 0);
const SET_PLACEMENT: blockui::Action =
    blockui::Action::new("popover.placement", |cx, value| PLACEMENT.set(cx, value));
const DRAFT: State<String> = State::<String>::new("popover.draft");
struct Fixture;
impl blockui::Actor for Fixture {
    const STATE: &'static [StateSpec] = &[OPEN.spec(), DRAFT.spec(), PLACEMENT.spec()];
    const ACTIONS: &'static [blockui::Action] = &[SET_PLACEMENT];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        Ok(blockui::Column::default().children(vec![
            blockui::Spacer::new(160.0).into_view(),
            Popover::new(OPEN, "Settings")
                .key("settings")
                .placement(blockui::PopoverPlacement::try_from(PLACEMENT.get(cx)?)?)
                .children(
                    cx,
                    vec![
                        blockui::TextInput::new(DRAFT).key("draft").into_view(),
                        blockui::Button::label("Done")
                            .key("done")
                            .on_click(|cx| OPEN.set(cx, false)),
                    ],
                )?,
            blockui::Button::label("After")
                .key("after")
                .on_click(|_| Ok(())),
        ]))
    }
}
fn part(name: &str) -> u64 {
    blockui::Key::named("settings").scoped(key(name)).0
}
fn key_event(name: &str) -> UiEvent {
    UiEvent::Key(shell::KeyEvent {
        phase: shell::KeyPhase::Down,
        key: name.into(),
        key_without_modifiers: name.into(),
        code: name.into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Default::default(),
        repeat: false,
        synthetic: false,
    })
}
#[test]
fn popover_is_anchored_flips_and_restores_focus_without_moving_body() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 600);
    let body = source.widget_layout_rect(key("after")).unwrap();
    source.set_focus(Some(part("trigger")));
    source.actor.dispatch_widget(part("trigger"))?;
    source.build_frame(&mut tcx, 480, 600);
    assert_eq!(source.focus, Some(key("draft")));
    assert_eq!(source.modal, None);
    let done = source.widget_layout_rect(key("done")).unwrap();
    assert_eq!(
        source.hit(shell::Point {
            x: f64::from((done[0] + done[2]) * 0.5),
            y: f64::from((done[1] + done[3]) * 0.5)
        }),
        Some(key("done")),
        "overlay controls win over underlying page controls"
    );
    assert_eq!(source.widget_layout_rect(key("after")).unwrap(), body);
    let anchor = source.widget_layout_rect(part("trigger")).unwrap();
    let panel = source.widget_layout_rect(part("panel")).unwrap();
    assert!(
        (panel[1] - anchor[3] - 8.0).abs() < 1.0,
        "{anchor:?} {panel:?}"
    );
    let id = source.ids[&key("draft")];
    source.build_frame(&mut tcx, 240, 300);
    let panel = source.widget_layout_rect(part("panel")).unwrap();
    let anchor = source.widget_layout_rect(part("trigger")).unwrap();
    assert!(panel[3] <= anchor[1] - 7.0, "{anchor:?} {panel:?}");
    assert!(
        panel[0] >= 16.0 && panel[2] <= 224.0 && panel[1] >= 16.0 && panel[3] <= 284.0,
        "{panel:?}"
    );
    assert_eq!(source.ids[&key("draft")], id);
    let semantic = source.semantic_snapshot().unwrap();
    assert!(semantic.nodes.iter().any(|n| n.id == part("panel")
        && n.role == shell::SemanticRole::Dialog
        && n.states.modal == Some(false)));
    source.actor.set_widget_text(key("draft"), "保留草稿")?;
    source.handle_event(key_event("Escape"));
    source.build_frame(&mut tcx, 480, 600);
    assert_eq!(source.focus, Some(part("trigger")));
    assert!(!source.ids.contains_key(&key("draft")));
    source.actor.dispatch_widget(part("trigger"))?;
    source.build_frame(&mut tcx, 480, 600);
    assert_eq!(source.actor.view().bindings[&key("draft")].text, "保留草稿");
    source.actor.dispatch_widget(key("done"))?;
    source.build_frame(&mut tcx, 480, 600);
    assert_eq!(source.focus, Some(part("trigger")));
    Ok(())
}
#[test]
fn popover_tab_can_leave_and_dismiss_without_trapping_focus() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    let mut tcx = TextContext::new();
    source.actor.dispatch_widget(part("trigger"))?;
    source.build_frame(&mut tcx, 480, 600);
    source.set_focus(Some(key("done")));
    source.handle_event(key_event("Tab"));
    source.build_frame(&mut tcx, 480, 600);
    assert_eq!(source.focus, Some(key("after")));
    assert_eq!(
        source.actor.view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    Ok(())
}

struct LongContent;
impl blockui::Actor for LongContent {
    const STATE: &'static [StateSpec] = &[OPEN.spec()];
    const ACTIONS: &'static [blockui::Action] = &[];
    fn view(cx: &Cx<'_>) -> blockui::Result<blockui::View> {
        Popover::new(OPEN, "Settings").key("settings").children(
            cx,
            vec![
                blockui::Spacer::new(1200.0).key("long.content").into_view(),
                blockui::Button::label("Done")
                    .key("done")
                    .on_click(|cx| OPEN.set(cx, false)),
            ],
        )
    }
}
#[test]
fn clipped_popover_controls_can_be_revealed_without_scrolling_the_page() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<LongContent>()?)?;
    let mut tcx = TextContext::new();
    source.actor.dispatch_widget(part("trigger"))?;
    source.build_frame(&mut tcx, 240, 300);
    let panel = source.widget_layout_rect(part("panel")).unwrap();
    assert!(panel[1] >= 16.0 && panel[3] <= 284.0);
    source.set_focus(Some(key("done")));
    assert!(source.reveal_widget(key("done")));
    source.build_frame(&mut tcx, 240, 300);
    let done = source.widget_layout_rect(key("done")).unwrap();
    assert!(
        done[1] >= panel[1] && done[3] <= panel[3],
        "{done:?} outside {panel:?}"
    );
    assert!(source.actor.view().renderer[&part("panel")].scroll > 0.0);
    source.actor.dispatch_widget(key("done"))?;
    source.build_frame(&mut tcx, 240, 300);
    assert_eq!(source.focus, Some(part("trigger")));
    Ok(())
}

#[test]
fn popover_outside_pointer_dismisses_and_windows_are_independent() -> Result<(), String> {
    let mut first = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    let mut second = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    let mut a = TextContext::new();
    let mut b = TextContext::new();
    first.actor.dispatch_widget(part("trigger"))?;
    first.build_frame(&mut a, 480, 600);
    second.build_frame(&mut b, 480, 600);
    assert!(first.ids.contains_key(&key("draft")));
    assert!(!second.ids.contains_key(&key("draft")));
    let r = first.widget_layout_rect(key("after")).unwrap();
    first.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: PointerPhase::Down,
        position: shell::Point {
            x: f64::from(r[2] - 4.0),
            y: f64::from(r[3] - 4.0),
        },
        button: Some(shell::PointerButton::Primary),
        buttons: 1,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
    first.build_frame(&mut a, 480, 600);
    assert_eq!(
        first.actor.view().bindings[&part("panel")]
            .node
            .semantics
            .hidden,
        1
    );
    assert_eq!(first.focus, Some(key("after")));
    second.actor.dispatch_widget(part("trigger"))?;
    second.build_frame(&mut b, 480, 600);
    assert!(second.ids.contains_key(&key("draft")));
    assert!(!first.ids.contains_key(&key("draft")));
    Ok(())
}

#[test]
#[ignore = "explicit offscreen Vello capture"]
fn popover_visuals() -> Result<(), String> {
    for scale in [1, 2] {
        let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
        source.scale = f64::from(scale);
        source.actor.dispatch_widget(part("trigger"))?;
        let mut tcx = TextContext::new();
        let nodes = source.build_frame(&mut tcx, 480, 600);
        let mut scene = shell::Scene::new();
        shell::scene::build_scene_scaled(
            &mut scene,
            &nodes,
            &mut tcx,
            480,
            600,
            source.base_color(),
            f64::from(scale),
        );
        let image =
            shell::renderer::render_to_image(&scene, 480 * scale, 600 * scale, source.base_color())
                .map_err(|e| e.to_string())?;
        let panel = source.widget_layout_rect(part("panel")).unwrap();
        let p = image.get_pixel(
            ((panel[0] + 8.0) * scale as f32) as u32,
            ((panel[1] + 8.0) * scale as f32) as u32,
        );
        assert!(p.0[..3].iter().all(|c| *c >= 250), "popover surface: {p:?}");
        if scale == 1 {
            image
                .save("../../reports/native-popover.png")
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[test]
fn twelve_popover_placements_keep_editor_and_focus() -> Result<(), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Fixture>()?)?;
    let mut tcx = TextContext::new();
    source.actor.dispatch_widget(part("trigger"))?;
    source.build_frame(&mut tcx, 480, 600);
    source.actor.set_widget_text(key("draft"), "Keep 草稿")?;
    let id = source.ids[&key("draft")];
    for (w, h) in [(900, 700), (240, 300)] {
        for position in 0..12 {
            source.actor.tell(SET_PLACEMENT, position)?;
            source.actor.tick()?;
            source.build_frame(&mut tcx, w, h);
            assert_eq!(source.ids[&key("draft")], id);
            assert_eq!(source.focus, Some(key("draft")));
            assert_eq!(
                source.actor.view().bindings[&key("draft")].text,
                "Keep 草稿"
            );
            let r = source.widget_layout_rect(part("panel")).unwrap();
            assert!(
                r[0] >= 16.0 && r[1] >= 16.0 && r[2] <= w as f32 - 16.0 && r[3] <= h as f32 - 16.0,
                "position {position}: {r:?}"
            );
        }
    }
    Ok(())
}
#[test]
fn popover_side_alignment_and_collision_geometry() {
    use super::toast::popover_origin;
    let a = [400.0, 300.0, 500.0, 340.0];
    let expected = [
        [400.0, 348.0],
        [360.0, 348.0],
        [320.0, 348.0],
        [400.0, 192.0],
        [360.0, 192.0],
        [320.0, 192.0],
        [508.0, 300.0],
        [508.0, 270.0],
        [508.0, 240.0],
        [212.0, 300.0],
        [212.0, 270.0],
        [212.0, 240.0],
    ];
    for (position, expected) in expected.into_iter().enumerate() {
        assert_eq!(
            popover_origin(a, [180.0, 100.0], [1000.0, 700.0], position as u32),
            expected
        );
    }
    assert_eq!(
        popover_origin(
            [850.0, 300.0, 950.0, 340.0],
            [180.0, 100.0],
            [1000.0, 700.0],
            7
        ),
        [662.0, 270.0]
    );
    assert_eq!(
        popover_origin(
            [400.0, 620.0, 500.0, 660.0],
            [180.0, 100.0],
            [1000.0, 700.0],
            1
        ),
        [360.0, 512.0]
    );
}
