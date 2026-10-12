use super::*;
use blockui::{Action, Column, Cx, IntoView, StateSpec, Toast, ToastVariant, Toasts, View};
const POSITION: blockui::State<i64> = blockui::State::<i64>::new("native.toast.position", 0);
const SET_POSITION: Action =
    Action::new("native.toast.position", |cx, value| POSITION.set(cx, value));
const TOASTS: Toasts = Toasts::new("native.toasts");
const EXPIRE: Action = Action::new("native.toast.expire", |cx, revision| {
    TOASTS.expire(cx, revision, EXPIRE)
});
const SHOW: Action = Action::new("native.toast.show", |cx, _| {
    TOASTS.show(
        cx,
        Toast::new(
            "Changes saved",
            "Your draft is safe. Use the controls below to continue.",
        )
        .variant(ToastVariant::Success)
        .action("Undo", 1),
        EXPIRE,
    )?;
    Ok(())
});
const TIMED: Action = Action::new("native.toast.timed", |cx, _| {
    TOASTS.show(
        cx,
        Toast::new("Timed message", "Read before closing").duration(Duration::from_millis(100)),
        EXPIRE,
    )?;
    Ok(())
});
fn action(_: &Cx<'_>, _: i64) -> blockui::Result<()> {
    Ok(())
}
struct Notifications;
impl blockui::Actor for Notifications {
    const STATE: &'static [StateSpec] = &[
        TOASTS.state()[0],
        TOASTS.state()[1],
        TOASTS.state()[2],
        TOASTS.state()[3],
        POSITION.spec(),
    ];
    const ACTIONS: &'static [Action] = &[SHOW, EXPIRE, TIMED, SET_POSITION];
    fn view(cx: &Cx<'_>) -> blockui::Result<View> {
        Ok(blockui::ScrollView::default()
            .key("page.scroll")
            .children(vec![
                Column {
                    padding: 24.0,
                    gap: 12.0,
                    ..Default::default()
                }
                .key("page")
                .children(vec![
                    blockui::Button::new("Save changes", SHOW)
                        .key("save")
                        .into_view(),
                    blockui::Text::new("Page body").key("body").into_view(),
                    blockui::Column {
                        key: blockui::Key::named("spacer"),
                        ..Default::default()
                    }
                    .children(vec![
                        blockui::Text::new("Scroll content\n".repeat(80))
                            .key("bottom")
                            .into_view(),
                    ])
                    .style(blockui::ViewStyle::default()),
                    TOASTS
                        .position(blockui::ToastPosition::try_from(POSITION.get(cx)?)?)
                        .view(cx, EXPIRE, action)?,
                ]),
            ]))
    }
}
fn fixture() -> Result<(NativeSource, TextContext), String> {
    let actor = Actor::for_test::<Notifications>()?;
    actor.use_manual_timers_for_test()?;
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 600);
    Ok((source, tcx))
}
fn dismiss() -> u64 {
    blockui::Key::named("native.toasts")
        .scoped(1)
        .scoped(key("dismiss"))
        .0
}
#[test]
fn toast_is_fixed_nonmodal_and_returns_focus_after_dismissal() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    source.set_focus(Some(key("save")));
    let body = source.widget_layout_rect(key("body")).unwrap();
    source.actor.dispatch_widget(key("save"))?;
    source.build_frame(&mut tcx, 480, 600);
    assert_eq!(source.focus, Some(key("save")));
    assert_eq!(source.modal, None);
    assert_eq!(source.widget_layout_rect(key("body")).unwrap(), body);
    let region = source
        .placed
        .iter()
        .find(|n| n.node_id == source.ids[&key("native.toasts")])
        .unwrap();
    assert!((region.rect[2] - 464.0).abs() < 1.0, "{:?}", region.rect);
    assert!((region.rect[3] - 584.0).abs() < 1.0, "{:?}", region.rect);
    let rect = source.widget_layout_rect(dismiss()).unwrap();
    assert_eq!(
        source.hit(shell::Point {
            x: f64::from((rect[0] + rect[2]) * 0.5),
            y: f64::from((rect[1] + rect[3]) * 0.5)
        }),
        Some(dismiss())
    );
    let status = source.semantic_snapshot().unwrap();
    assert!(
        status
            .nodes
            .iter()
            .any(|n| n.role == shell::SemanticRole::Status
                && n.label.as_deref() == Some("Changes saved"))
    );
    source.set_focus(Some(dismiss()));
    source.actor.dispatch_widget(dismiss())?;
    source.build_frame(&mut tcx, 480, 600);
    assert_eq!(source.focus, Some(key("save")));
    assert!(!source.ids.contains_key(&dismiss()));
    Ok(())
}
#[test]
fn toast_tracks_window_width_and_ignores_page_scroll() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    source.actor.dispatch_widget(key("save"))?;
    source.build_frame(&mut tcx, 480, 600);
    let id = source.ids[&dismiss()];
    source
        .actor
        .view_mut()
        .renderer
        .get_mut(&key("page.scroll"))
        .unwrap()
        .scroll = 150.0;
    source.build_frame(&mut tcx, 240, 400);
    let region = source
        .placed
        .iter()
        .find(|n| n.node_id == source.ids[&key("native.toasts")])
        .unwrap();
    assert_eq!(region.rect[0], 16.0);
    assert_eq!(region.rect[2], 224.0);
    assert_eq!(region.rect[3], 384.0);
    assert_eq!(source.ids[&dismiss()], id);
    let rect = source.widget_layout_rect(dismiss()).unwrap();
    assert!(rect[0] >= 16.0 && rect[2] <= 224.0);
    Ok(())
}

#[test]
fn overflowing_notification_stack_scrolls_and_clamps_after_removal() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    for _ in 0..3 {
        source.actor.dispatch_widget(key("save"))?;
    }
    source.build_frame(&mut tcx, 240, 300);
    let viewport = key("native.toasts");
    let placed = source
        .placed
        .iter()
        .find(|n| n.node_id == source.ids[&viewport])
        .unwrap();
    assert!(placed.scroll.range[1] > 0.0, "{:?}", placed.scroll);
    let last = blockui::Key::named("native.toasts")
        .scoped(3)
        .scoped(key("action"))
        .0;
    // A clipped button may have no PlacedNode. Retained geometry must still let
    // keyboard focus scroll it into the fixed viewport.
    source.set_focus(Some(last));
    assert!(source.reveal_widget(last));
    source.build_frame(&mut tcx, 240, 300);
    let button = source.widget_layout_rect(last).unwrap();
    let port = source.widget_layout_rect(viewport).unwrap();
    assert!(
        button[1] >= port[1] && button[3] <= port[3],
        "{button:?} outside {port:?}"
    );
    let page_scroll = source.actor.view().renderer[&key("page.scroll")].scroll;
    assert!(source.navigate_toast("F8"));
    source.build_frame(&mut tcx, 240, 300);
    let focused = source.widget_layout_rect(source.focus.unwrap()).unwrap();
    let port = source.widget_layout_rect(viewport).unwrap();
    assert!(focused[1] >= port[1] && focused[3] <= port[3]);
    assert_eq!(
        source.actor.view().renderer[&key("page.scroll")].scroll,
        page_scroll
    );
    source
        .actor
        .view_mut()
        .renderer
        .get_mut(&viewport)
        .unwrap()
        .scroll = 100000.0;
    source.build_frame(&mut tcx, 240, 300);
    let range = source
        .placed
        .iter()
        .find(|n| n.node_id == source.ids[&viewport])
        .unwrap()
        .scroll
        .range[1];
    assert_eq!(
        source.actor.view().renderer[&viewport].scroll,
        f64::from(range)
    );
    for id in [2, 3] {
        let target = blockui::Key::named("native.toasts")
            .scoped(id)
            .scoped(key("dismiss"))
            .0;
        source.actor.dispatch_widget(target)?;
    }
    source.build_frame(&mut tcx, 240, 500);
    assert_eq!(source.actor.view().renderer[&viewport].scroll, 0.0);
    assert!(source.widget_layout_rect(dismiss()).unwrap()[1] >= 0.0);
    Ok(())
}

#[test]
#[ignore = "explicit Vello fixed notification plane at 1x and 2x"]
fn toast_visuals() -> Result<(), String> {
    for (scale, position) in [(1, 0), (2, 0), (1, 5), (2, 5)] {
        let (mut source, mut tcx) = fixture()?;
        source.scale = f64::from(scale);
        source.actor.dispatch_widget(key("save"))?;
        source.actor.tell(SET_POSITION, position)?;
        source.actor.tick()?;
        let nodes = source.build_frame(&mut tcx, 480, 600);
        let card = nodes
            .iter()
            .find(|n| n.node_id == source.ids[&blockui::Key::named("native.toasts").scoped(1).0])
            .unwrap();
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
        let pixel = image.get_pixel(
            ((card.rect[0] + 8.0) * scale as f32) as u32,
            ((card.rect[1] + 8.0) * scale as f32) as u32,
        );
        assert!(
            pixel.0[..3].iter().all(|c| *c >= 250),
            "toast surface: {pixel:?}"
        );
        let mut ink = 0;
        for y in ((card.rect[1] + 16.0) * scale as f32) as u32
            ..((card.rect[3] - 16.0) * scale as f32) as u32
        {
            for x in ((card.rect[0] + 16.0) * scale as f32) as u32
                ..((card.rect[2] - 16.0) * scale as f32) as u32
            {
                let p = image.get_pixel(x, y);
                if p[0] < 160 && p[1] < 160 && p[2] < 160 {
                    ink += 1;
                }
            }
        }
        assert!(ink > 100, "toast title, body and controls must paint");
        if scale == 1 {
            image
                .save(if position == 0 {
                    "../../reports/native-toast.png"
                } else {
                    "../../reports/native-toast-position.png"
                })
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[test]
fn toast_windows_keep_independent_state_focus_and_layout_caches() -> Result<(), String> {
    let (mut first, mut a) = fixture()?;
    let (mut second, mut b) = fixture()?;
    first.actor.dispatch_widget(key("save"))?;
    first.build_frame(&mut a, 480, 600);
    second.build_frame(&mut b, 480, 600);
    assert!(first.ids.contains_key(&dismiss()));
    assert!(!second.ids.contains_key(&dismiss()));
    let id = first.ids[&key("native.toasts")];
    assert!(!first.tree.dirty(id).unwrap());
    first.build_frame(&mut a, 480, 600);
    assert!(!first.tree.dirty(id).unwrap());
    second.actor.dispatch_widget(key("save"))?;
    second.build_frame(&mut b, 480, 600);
    first.actor.dispatch_widget(dismiss())?;
    first.build_frame(&mut a, 480, 600);
    assert!(second.ids.contains_key(&dismiss()));
    Ok(())
}

#[test]
fn toast_shortcuts_enter_return_and_respect_event_guards() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    let keyboard = |name: &str, phase, modifiers, repeat, synthetic| {
        UiEvent::Key(shell::KeyEvent {
            phase,
            key: name.into(),
            key_without_modifiers: name.into(),
            code: name.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers,
            repeat,
            synthetic,
        })
    };
    let plain = shell::Modifiers::default();
    source.set_focus(Some(key("save")));
    source.handle_event(keyboard("F8", shell::KeyPhase::Down, plain, false, false));
    assert_eq!(
        source.focus,
        Some(key("save")),
        "empty queue preserves focus"
    );
    source.actor.dispatch_widget(key("save"))?;
    source.build_frame(&mut tcx, 480, 600);
    for (phase, repeat, synthetic) in [
        (shell::KeyPhase::Up, false, false),
        (shell::KeyPhase::Down, true, false),
        (shell::KeyPhase::Down, false, true),
    ] {
        source.handle_event(keyboard("F8", phase, plain, repeat, synthetic));
        assert_eq!(source.focus, Some(key("save")));
    }
    source.handle_event(keyboard(
        "F8",
        shell::KeyPhase::Down,
        shell::Modifiers::SHIFT,
        false,
        false,
    ));
    assert_eq!(source.focus, Some(key("save")));
    source.handle_event(UiEvent::Focus(false));
    source.handle_event(keyboard("F8", shell::KeyPhase::Down, plain, false, false));
    assert_eq!(source.focus, Some(key("save")));
    source.handle_event(UiEvent::Focus(true));
    source.modal = Some(key("modal"));
    assert!(!source.navigate_toast("F8"));
    source.modal = None;
    source.handle_event(keyboard("F8", shell::KeyPhase::Down, plain, false, false));
    let undo = blockui::Key::named("native.toasts")
        .scoped(1)
        .scoped(key("action"))
        .0;
    assert_eq!(source.focus, Some(undo));
    assert_eq!(source.toast_return_focus, Some(key("save")));
    source.handle_event(keyboard(
        "Escape",
        shell::KeyPhase::Down,
        plain,
        false,
        false,
    ));
    assert_eq!(source.focus, Some(key("save")));
    assert!(
        source.actor.projected_view().bindings.contains_key(&undo),
        "Escape leaves the message available"
    );
    Ok(())
}

#[test]
fn toast_pauses_for_hover_focus_and_window_blur_without_resetting_remaining_time()
-> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    source.actor.set_monotonic_ms_for_test(0)?;
    source.actor.tell(TIMED, 0)?;
    source.actor.tick()?;
    source.build_frame(&mut tcx, 480, 600);
    let viewport = blockui::Key::named("native.toasts").0;
    let dismiss = blockui::Key::named("native.toasts")
        .scoped(1)
        .scoped(key("dismiss"))
        .0;
    let id = source.ids[&viewport];
    let rect = source
        .placed
        .iter()
        .find(|node| node.node_id == id)
        .unwrap()
        .rect;
    source.actor.set_monotonic_ms_for_test(30)?;
    source
        .actor
        .advance_timer_time_for_test(Duration::from_millis(30))?;
    let pointer = |phase, position| {
        UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            buttons: 0,
            button: None,
            modifiers: Default::default(),
            properties: Default::default(),
        })
    };
    source.handle_event(pointer(
        PointerPhase::Move,
        shell::Point {
            x: f64::from(rect[0] + 2.0),
            y: f64::from(rect[1] + 2.0),
        },
    ));
    assert_eq!(
        source.actor.view.bindings[&viewport].node.action_payload, 1,
        "card padding also pauses"
    );
    source.set_focus(Some(dismiss));
    source.handle_event(pointer(
        PointerPhase::Leave,
        shell::Point { x: 0.0, y: 0.0 },
    ));
    assert_eq!(
        source.actor.view.bindings[&viewport].node.action_payload, 1,
        "focus outlives hover"
    );
    source.handle_event(UiEvent::Focus(false));
    source.set_focus(Some(key("save")));
    assert_eq!(
        source.actor.view.bindings[&viewport].node.action_payload, 1,
        "window blur outlives control focus"
    );
    source.actor.set_monotonic_ms_for_test(1030)?;
    assert_eq!(
        source
            .actor
            .advance_timer_time_for_test(Duration::from_millis(1000))?,
        0
    );
    source.handle_event(UiEvent::Focus(true));
    assert_eq!(source.actor.view.bindings[&viewport].node.action_payload, 0);
    source.actor.set_monotonic_ms_for_test(1099)?;
    assert_eq!(
        source
            .actor
            .advance_timer_time_for_test(Duration::from_millis(69))?,
        0
    );
    source.actor.set_monotonic_ms_for_test(1100)?;
    assert_eq!(
        source
            .actor
            .advance_timer_time_for_test(Duration::from_millis(1))?,
        1
    );
    source.actor.tick()?;
    assert!(
        !source
            .actor
            .view
            .bindings
            .values()
            .any(|b| b.text == "Timed message")
    );
    assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
    Ok(())
}

#[test]
fn toast_positions_anchor_inside_desktop_and_narrow_windows_and_retain_focus() -> Result<(), String>
{
    let (mut source, mut tcx) = fixture()?;
    source.actor.dispatch_widget(key("save"))?;
    source.build_frame(&mut tcx, 900, 600);
    source.set_focus(Some(dismiss()));
    let retained = source.ids[&dismiss()];
    let viewport = blockui::Key::named("native.toasts").0;
    for width in [900u32, 240] {
        for position in 0..6 {
            source.actor.tell(SET_POSITION, position)?;
            source.actor.tick()?;
            source.build_frame(&mut tcx, width, 600);
            let rect = source.widget_layout_rect(viewport).unwrap();
            let close = |a: f32, b: f32| {
                assert!(
                    (a - b).abs() < 1.0,
                    "{position}: {rect:?}, expected {b}, got {a}"
                )
            };
            match position % 3 {
                0 => close(rect[2], width as f32 - 16.0),
                1 => close(rect[0], 16.0),
                _ => close((rect[0] + rect[2]) * 0.5, width as f32 * 0.5),
            }
            if position >= 3 {
                close(rect[1], 16.0);
            } else {
                close(rect[3], 584.0);
            }
            assert!(
                rect[0] >= 0.0 && rect[2] <= width as f32 && rect[1] >= 0.0 && rect[3] <= 600.0
            );
            assert_eq!(source.focus, Some(dismiss()));
            assert_eq!(source.ids[&dismiss()], retained);
        }
    }
    assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
    Ok(())
}

const SECOND: Toasts = Toasts::new("native.second");
const SECOND_EXPIRE: Action = Action::new("native.second.expire", |cx, revision| {
    SECOND.expire(cx, revision, SECOND_EXPIRE)
});
const BOTH: Action = Action::new("native.both", |cx, _| {
    TOASTS.show(
        cx,
        Toast::new("First queue", "Long notification\n".repeat(100)).persistent(),
        EXPIRE,
    )?;
    SECOND.show(
        cx,
        Toast::new("Second queue", "Long notification\n".repeat(100)).persistent(),
        SECOND_EXPIRE,
    )?;
    Ok(())
});
struct TwoQueues;
impl blockui::Actor for TwoQueues {
    const STATE: &'static [StateSpec] = &[
        TOASTS.state()[0],
        TOASTS.state()[1],
        TOASTS.state()[2],
        TOASTS.state()[3],
        SECOND.state()[0],
        SECOND.state()[1],
        SECOND.state()[2],
        SECOND.state()[3],
    ];
    const ACTIONS: &'static [Action] = &[BOTH, EXPIRE, SECOND_EXPIRE];
    fn view(cx: &Cx<'_>) -> blockui::Result<View> {
        Ok(Column::default().children(vec![
            TOASTS
                .position(blockui::ToastPosition::TopLeft)
                .view(cx, EXPIRE, action)?,
            SECOND
                .position(blockui::ToastPosition::TopLeft)
                .view(cx, SECOND_EXPIRE, action)?,
        ]))
    }
}
#[test]
fn same_position_queues_share_window_height_without_overlap_and_keep_keyboard_access()
-> Result<(), String> {
    let mut actor = Actor::for_test::<TwoQueues>()?;
    actor.use_manual_timers_for_test()?;
    actor.tell(BOTH, 0)?;
    actor.tick()?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 400, 400);
    let first = source
        .widget_layout_rect(blockui::Key::named("native.toasts").0)
        .unwrap();
    let second = source
        .widget_layout_rect(blockui::Key::named("native.second").0)
        .unwrap();
    assert!((first[1] - 16.0).abs() < 1.0);
    assert!(second[1] >= first[3] + 11.0 && second[3] <= 384.0);
    assert!(source.navigate_toast("F8"));
    source.build_frame(&mut tcx, 400, 400);
    let control = source.widget_layout_rect(source.focus.unwrap()).unwrap();
    assert!(control[1] >= first[1] && control[3] <= first[3]);
    assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
    Ok(())
}
