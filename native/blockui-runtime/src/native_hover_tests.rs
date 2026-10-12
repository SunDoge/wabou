use super::*;
use blockui::{
    Action, Button, ButtonVariant, Checkbox, Column, Cx, IntoView, State, StateSpec, Text,
    TextArea, TextInput, View,
};
const ENABLED: State<bool> = State::<bool>::new("hover.enabled", true);
const SHIFT: State<bool> = State::<bool>::new("hover.shift", false);
const SHOW: State<bool> = State::<bool>::new("hover.show", true);
const MODAL: State<bool> = State::<bool>::new("hover.modal", false);
const COUNT: State<i64> = State::<i64>::new("hover.count", 0);
const DRAFT: State<String> = State::<String>::new("hover.draft");
const LEVEL: State<i64> = State::<i64>::new("hover.level", 5);
const CHECK: State<bool> = State::<bool>::new("hover.checked", false);
const CLICK: Action = Action::new("hover.click", |cx, _| COUNT.update(cx, |n| Ok(n + 1)));
const DISABLE: Action = Action::new("hover.disable", |cx, _| ENABLED.set(cx, false));
const MOVE: Action = Action::new("hover.shift", |cx, _| SHIFT.set(cx, true));
const HIDE: Action = Action::new("hover.hide", |cx, _| SHOW.set(cx, false));
const OPEN: Action = Action::new("hover.open", |cx, _| MODAL.set(cx, true));
struct Controls;
impl blockui::Actor for Controls {
    const STATE: &'static [StateSpec] = &[
        ENABLED.spec(),
        SHIFT.spec(),
        SHOW.spec(),
        MODAL.spec(),
        COUNT.spec(),
        DRAFT.spec(),
        CHECK.spec(),
        LEVEL.spec(),
    ];
    const ACTIONS: &'static [Action] = &[CLICK, DISABLE, MOVE, HIDE, OPEN];
    fn view(cx: &Cx<'_>) -> blockui::Result<View> {
        let mut children = Vec::new();
        if SHIFT.get(cx)? {
            children.push(blockui::Spacer::new(100.0).key("hover.spacer").into_view());
        }
        if SHOW.get(cx)? {
            for (name, variant) in [
                ("primary", ButtonVariant::Primary),
                ("secondary", ButtonVariant::Secondary),
                ("danger", ButtonVariant::Danger),
            ] {
                children.push(
                    Button {
                        variant,
                        enabled: ENABLED.get(cx)?,
                        ..Button::new(name, CLICK)
                    }
                    .key(name)
                    .into_view(),
                );
            }
        }
        children.push(
            Checkbox::new("Checked", CHECK, |cx: &Cx<'_>, checked| {
                CHECK.set(cx, checked)
            })
            .into_view()
            .key("check"),
        );
        children.push(TextInput::new(DRAFT).key("input").into_view());
        children.push(TextArea::new(DRAFT).key("area").into_view());
        children.push(
            blockui::Slider::new(
                LEVEL,
                blockui::IntegerRange {
                    min: 0,
                    max: 10,
                    step: 1,
                },
                |cx: &Cx<'_>, value| LEVEL.set(cx, value),
            )
            .into_view()
            .key("slider"),
        );
        children.push(
            blockui::Select::new(
                "Level",
                LEVEL,
                [0, 5, 10]
                    .into_iter()
                    .map(|value| blockui::SelectOption {
                        value,
                        label: value.to_string(),
                        enabled: value != 10,
                    })
                    .collect(),
                |cx: &Cx<'_>, value| LEVEL.set(cx, value),
            )
            .into_view()
            .key("select"),
        );
        children.push(Text::new("Not interactive").key("plain").into_view());
        if MODAL.get(cx)? {
            children.push(
                blockui::Dialog::new("Dialog", CLICK)
                    .key("modal")
                    .children(vec![
                        Button::new("Close", CLICK).key("dialog.button").into_view(),
                    ]),
            );
        }
        Ok(Column::default().children(children))
    }
}
fn fixture() -> Result<(NativeSource, TextContext), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Controls>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 900);
    Ok((source, tcx))
}
fn point(source: &NativeSource, name: &str) -> shell::Point {
    let rect = source.widget_layout_rect(key(name)).unwrap();
    shell::Point {
        x: f64::from((rect[0] + rect[2]) / 2.0),
        y: f64::from((rect[1] + rect[3]) / 2.0),
    }
}
fn pointer(
    source: &mut NativeSource,
    position: shell::Point,
    phase: PointerPhase,
    buttons: u32,
) -> EventResponse {
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase,
        position,
        buttons,
        button: matches!(phase, PointerPhase::Down | PointerPhase::Up)
            .then_some(shell::PointerButton::Primary),
        modifiers: Default::default(),
        properties: Default::default(),
    }))
}
fn paint(source: &NativeSource, name: &str) -> Paint {
    source
        .tree
        .get_node_context(source.ids[&key(name)])
        .unwrap()
        .clone()
}
fn dispatch(source: &mut NativeSource, action: Action) -> Result<(), String> {
    source.actor.tell(action, 0)?;
    source.actor.tick()
}
#[test]
fn hover_changes_only_host_paint_and_coalesces_pointer_moves() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    let evaluations: Vec<_> = source
        .actor
        .view
        .bindings
        .iter()
        .map(|(key, b)| (*key, b.evaluations))
        .collect();
    for name in [
        "primary",
        "secondary",
        "danger",
        "check",
        "input",
        "area",
        "slider",
        "select",
    ] {
        let normal = paint(&source, name);
        let target = point(&source, name);
        let id = source.ids[&key(name)];
        let rect = source.widget_layout_rect(key(name)).unwrap();
        assert!(pointer(&mut source, target, PointerPhase::Move, 0).request_redraw);
        assert_eq!(source.hovered, Some(key(name)));
        source.sync()?;
        for id in source.ids.values() {
            assert!(
                !source.tree.dirty(*id).unwrap(),
                "hover entry invalidated layout for {name}"
            );
        }
        source.build_frame(&mut tcx, 480, 900);
        let hover = paint(&source, name);
        assert!(
            normal.background != hover.background
                || normal.border != hover.border
                || normal.outline_width != hover.outline_width
        );
        assert_eq!(source.ids[&key(name)], id);
        assert_eq!(source.widget_layout_rect(key(name)).unwrap(), rect);
        assert!(!pointer(&mut source, target, PointerPhase::Move, 0).request_redraw);
        pointer(&mut source, target, PointerPhase::Leave, 0);
        source.build_frame(&mut tcx, 480, 900);
        assert_eq!(source.hovered, None);
        assert_eq!(paint(&source, name).background, normal.background);
        assert_eq!(paint(&source, name).border, normal.border);
    }
    assert_eq!(source.actor.integer("hover.count"), 0);
    for (key, evaluations) in evaluations {
        assert_eq!(source.actor.view.bindings[&key].evaluations, evaluations);
    }
    let target = point(&source, "plain");
    assert!(!pointer(&mut source, target, PointerPhase::Move, 0).request_redraw);
    assert_eq!(source.hovered, None);
    Ok(())
}
#[test]
fn pressed_wins_focus_survives_and_cancel_blur_and_touch_clear_hover() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    let target = point(&source, "primary");
    pointer(&mut source, target, PointerPhase::Move, 0);
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(
        paint(&source, "primary").background,
        Some(Color::from_rgb8(27, 78, 140))
    );
    pointer(&mut source, target, PointerPhase::Down, 1);
    source.build_frame(&mut tcx, 480, 900);
    let pressed = paint(&source, "primary");
    assert_eq!(pressed.background, Some(Color::from_rgb8(21, 62, 112)));
    assert_eq!(pressed.outline_width, 2.0);
    pointer(&mut source, target, PointerPhase::Cancel, 0);
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.hovered, None);
    assert_eq!(source.pointer_pressed_visual, None);
    assert_eq!(source.actor.integer("hover.count"), 0);
    pointer(&mut source, target, PointerPhase::Move, 0);
    source.handle_event(UiEvent::Focus(false));
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.hovered, None);
    source.handle_event(UiEvent::Focus(true));
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(
        source.hovered, None,
        "focus restoration must not invent pointer presence"
    );
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: PointerPhase::Move,
        position: target,
        button: None,
        buttons: 0,
        modifiers: Default::default(),
        properties: shell::PointerProperties {
            pointer_type: shell::PointerType::Touch,
            ..Default::default()
        },
    }));
    assert_eq!(source.hovered, None);
    Ok(())
}
#[test]
fn hover_reconciles_stationary_pointer_disabled_removed_modal_and_replacement() -> Result<(), String>
{
    let (mut source, mut tcx) = fixture()?;
    let target = point(&source, "primary");
    pointer(&mut source, target, PointerPhase::Move, 0);
    source.build_frame(&mut tcx, 480, 900);
    let node = source.ids[&key("primary")];
    source.actor.replace_for_test::<Controls>()?;
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.hovered, Some(key("primary")));
    assert_eq!(source.ids[&key("primary")], node);
    dispatch(&mut source, MOVE)?;
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(
        source.hovered, None,
        "layout moved the button away from stationary pointer"
    );
    let target = point(&source, "primary");
    pointer(&mut source, target, PointerPhase::Move, 0);
    dispatch(&mut source, DISABLE)?;
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.hovered, None);
    assert_eq!(
        paint(&source, "primary").background,
        Some(Color::from_rgb8(240, 243, 247))
    );
    dispatch(&mut source, HIDE)?;
    source.build_frame(&mut tcx, 480, 900);
    assert!(!source.ids.contains_key(&key("primary")));
    let input = point(&source, "input");
    pointer(&mut source, input, PointerPhase::Move, 0);
    assert_eq!(source.hovered, Some(key("input")));
    dispatch(&mut source, OPEN)?;
    source.build_frame(&mut tcx, 480, 900);
    assert_ne!(
        source.hovered,
        Some(key("input")),
        "modal masks background controls"
    );
    Ok(())
}

#[test]
fn popup_hover_masks_controls_and_skips_disabled_options() -> Result<(), String> {
    let (mut source, mut tcx) = fixture()?;
    source.set_focus(Some(key("select")));
    source
        .selects
        .get_mut(&key("select"))
        .unwrap()
        .key(crate::select_model::Key::Activate, false);
    source.build_frame(&mut tcx, 480, 900);
    assert!(source.select_popup.is_some());
    let background = point(&source, "primary");
    pointer(&mut source, background, PointerPhase::Move, 0);
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.hovered, None);
    let (_, layout, _) = source.select_popup.as_ref().unwrap();
    let rect = layout.row_rect(0).unwrap();
    let position = shell::Point {
        x: f64::from(rect[0] + 8.0),
        y: f64::from(rect[1] + 8.0),
    };
    pointer(&mut source, position, PointerPhase::Move, 0);
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.selects[&key("select")].active(), Some(0));
    assert!(!pointer(&mut source, position, PointerPhase::Move, 0).request_redraw);
    let rect = source.select_popup.as_ref().unwrap().1.row_rect(2).unwrap();
    pointer(
        &mut source,
        shell::Point {
            x: f64::from(rect[0] + 8.0),
            y: f64::from(rect[1] + 8.0),
        },
        PointerPhase::Move,
        0,
    );
    assert_eq!(source.selects[&key("select")].active(), Some(0));
    assert_eq!(source.actor.integer("hover.level"), 5);
    Ok(())
}

#[test]
#[ignore = "explicit Vello hover/pressed/disabled pixels at 1x and 2x"]
fn hover_pixels() -> Result<(), String> {
    for scale in [1, 2] {
        let (mut source, mut tcx) = fixture()?;
        source.use_manual_motion_for_test();
        let target = point(&source, "primary");
        for (stage, expected) in [
            ("normal", [31u8, 88, 155]),
            ("hover", [27, 78, 140]),
            ("pressed", [21, 62, 112]),
            ("disabled", [240, 243, 247]),
        ] {
            match stage {
                "hover" => {
                    pointer(&mut source, target, PointerPhase::Move, 0);
                }
                "pressed" => {
                    pointer(&mut source, target, PointerPhase::Down, 1);
                }
                "disabled" => dispatch(&mut source, DISABLE)?,
                _ => {}
            }
            source.build_frame(&mut tcx, 480, 900);
            source.advance_motion_for_test(Duration::from_millis(200));
            let nodes = source.build_frame(&mut tcx, 480, 900);
            let mut scene = shell::Scene::new();
            shell::scene::build_scene_scaled(
                &mut scene,
                &nodes,
                &mut tcx,
                480,
                900,
                source.base_color(),
                f64::from(scale),
            );
            let image = shell::renderer::render_to_image(
                &scene,
                480 * scale,
                900 * scale,
                source.base_color(),
            )
            .map_err(|e| e.to_string())?;
            let rect = source.widget_layout_rect(key("primary")).unwrap();
            let pixel = image.get_pixel(
                ((rect[2] - 8.0) * scale as f32) as u32,
                ((rect[1] + 8.0) * scale as f32) as u32,
            );
            for channel in 0..3 {
                assert!(
                    pixel[channel].abs_diff(expected[channel]) <= 2,
                    "{stage} scale={scale}: {pixel:?}"
                );
            }
            if scale == 1 {
                std::fs::create_dir_all("../../reports").map_err(|e| e.to_string())?;
                image
                    .save(format!("../../reports/hover-{stage}.png"))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[test]
fn hover_frames_interpolate_paint_without_actor_or_layout_work_and_retire_on_unmount()
-> Result<(), String> {
    use blockui_motion::{Key as MotionKey, Property};
    let (mut source, mut tcx) = fixture()?;
    source.use_manual_motion_for_test();
    let target = point(&source, "primary");
    let track = MotionKey::new(key("primary"), Property::Hover);
    let rect = source.widget_layout_rect(key("primary")).unwrap();
    let evaluations = source.actor.view().bindings[&key("primary")].evaluations;
    pointer(&mut source, target, PointerPhase::Move, 0);
    let nodes = source.build_frame(&mut tcx, 480, 900);
    let primary = nodes
        .iter()
        .find(|n| n.node_id == source.ids[&key("primary")])
        .unwrap();
    assert_eq!(
        primary.paint.background,
        Some(Color::from_rgb8(31, 88, 155))
    );
    source.advance_motion_for_test(Duration::from_millis(60));
    source.sync()?;
    assert_eq!(source.motion.value(track), Some(0.875));
    for id in source.ids.values() {
        assert!(!source.tree.dirty(*id).unwrap());
    }
    let nodes = source.build_frame(&mut tcx, 480, 900);
    let primary = nodes
        .iter()
        .find(|n| n.node_id == source.ids[&key("primary")])
        .unwrap();
    assert_ne!(
        primary.paint.background,
        Some(Color::from_rgb8(31, 88, 155))
    );
    assert_ne!(
        primary.paint.background,
        Some(Color::from_rgb8(27, 78, 140))
    );
    assert_eq!(source.widget_layout_rect(key("primary")).unwrap(), rect);
    assert_eq!(
        source.actor.view().bindings[&key("primary")].evaluations,
        evaluations
    );
    source.actor.replace_for_test::<Controls>()?;
    source.sync()?;
    assert_eq!(
        source.motion.value(track),
        Some(0.875),
        "stable tracks survive compatible implementation replacement"
    );
    pointer(&mut source, target, PointerPhase::Leave, 0);
    dispatch(&mut source, HIDE)?;
    source.build_frame(&mut tcx, 480, 900);
    assert_eq!(source.motion.value(track), None);
    assert!(!source.hover_paints.contains_key(&key("primary")));
    assert!(!source.has_anim());
    Ok(())
}
