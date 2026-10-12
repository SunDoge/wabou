use super::*;
use blockui::{
    ChoiceOption, Column, Cx, Heading, IntoView, Key, RadioGroup, State, StateSpec, Switch, Tab,
    Tabs, Text, TextInput, View,
};
const TAB: State<i64> = State::<i64>::new("choice.tab", 0);
const RADIO: State<i64> = State::<i64>::new("choice.radio", 0);
const ON: State<bool> = State::<bool>::new("choice.on", false);
const DRAFT: State<String> = State::<String>::new("choice.draft");
const SET: blockui::BoolAction =
    blockui::BoolAction::new("choice.set", |cx, value| ON.set(cx, value));
struct Choices;
impl blockui::Actor for Choices {
    const STATE: &'static [StateSpec] = &[TAB.spec(), RADIO.spec(), ON.spec(), DRAFT.spec()];
    const ACTIONS: &'static [blockui::Action] = &[SET.spec()];
    fn view(cx: &Cx<'_>) -> blockui::Result<View> {
        Ok(Column {
            padding: 24.0,
            gap: 16.0,
            ..Default::default()
        }
        .key("root")
        .children(vec![
            Tabs::new("Settings", TAB).key("tabs").children(
                cx,
                vec![
                    Tab::new(
                        0,
                        "Account",
                        Column {
                            padding: 0.0,
                            ..Default::default()
                        }
                        .key("account")
                        .children(vec![
                            Heading::new("Account").key("heading").into_view(),
                            TextInput::new(DRAFT).key("draft").into_view(),
                        ]),
                    ),
                    Tab::new(1, "Disabled", Text::new("Unavailable").key("unavailable")).disabled(),
                    Tab::new(2, "Security", Text::new("Security details").key("security")),
                ],
            )?,
            RadioGroup::new(
                "Delivery",
                RADIO,
                vec![
                    ChoiceOption::new(0, "Standard"),
                    ChoiceOption::new(1, "Disabled").disabled(),
                    ChoiceOption::new(2, "Pickup"),
                ],
            )
            .key("radio")
            .view(cx)?,
            Switch::new("Airplane mode", ON, SET)
                .key("switch")
                .into_view(),
            Switch::new("Managed", ON, SET)
                .key("managed")
                .enabled(false)
                .into_view(),
        ]))
    }
}
fn fixture() -> Result<(NativeSource, TextContext), String> {
    let mut source = NativeSource::without_watch(Actor::for_test::<Choices>()?)?;
    let mut text = TextContext::new();
    source.build_frame(&mut text, 480, 600);
    source.set_semantics_enabled(true);
    Ok((source, text))
}
fn item(group: &str, value: u64) -> u64 {
    Key::named(group).scoped(value).0
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
        modifiers: shell::Modifiers::empty(),
        repeat: false,
        synthetic: false,
    })
}
#[test]
fn tabs_roving_focus_skips_disabled_wraps_and_preserves_draft() -> Result<(), String> {
    let (mut source, mut text) = fixture()?;
    source.actor.edit_text(key("draft"), "retained 🦀")?;
    source.sync()?;
    source.set_focus(None);
    source.focus_next(false);
    assert_eq!(source.focus, Some(item("tabs", 0)));
    source.handle_event(key_event("ArrowRight"));
    assert_eq!(source.actor.integer("choice.tab"), 2);
    assert_eq!(source.focus, Some(item("tabs", 2)));
    source.handle_event(key_event("ArrowRight"));
    assert_eq!(source.actor.integer("choice.tab"), 0);
    source.handle_event(key_event("End"));
    assert_eq!(source.actor.integer("choice.tab"), 2);
    source.handle_event(key_event("Home"));
    assert_eq!(source.actor.integer("choice.tab"), 0);
    source.build_frame(&mut text, 480, 600);
    assert_eq!(
        source.editors[&key("draft")].current_value(),
        Some("retained 🦀")
    );
    source.focus_next(false);
    assert_eq!(source.focus, Some(panel("tabs", 0)));
    source.focus_next(false);
    assert_eq!(source.focus, Some(key("draft")));
    source.set_focus(Some(panel("tabs", 0)));
    source.actor.dispatch_widget(item("tabs", 2))?;
    source.sync()?;
    assert_eq!(source.focus, Some(item("tabs", 2)));
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Focus {
            target: panel("tabs", 0)
        })
    );
    Ok(())
}
fn panel(group: &str, value: u64) -> u64 {
    Key(item(group, value)).scoped(key("panel")).0
}
#[test]
fn radio_keyboard_and_semantics_report_exact_current_state() -> Result<(), String> {
    let (mut source, mut text) = fixture()?;
    source.set_focus(Some(item("radio", 0)));
    source.handle_event(key_event("ArrowDown"));
    assert_eq!(source.actor.integer("choice.radio"), 2);
    assert_eq!(source.focus, Some(item("radio", 2)));
    source.handle_event(key_event("ArrowDown"));
    assert_eq!(source.actor.integer("choice.radio"), 0);
    source.handle_event(key_event("End"));
    source.build_frame(&mut text, 480, 600);
    let snapshot = source.semantic_snapshot().unwrap();
    let radio = snapshot
        .nodes
        .iter()
        .find(|node| node.id == item("radio", 2))
        .unwrap();
    assert_eq!(radio.role, shell::SemanticRole::RadioButton);
    assert_eq!(radio.states.checked, Some(shell::SemanticToggleState::On));
    let tab = snapshot
        .nodes
        .iter()
        .find(|node| node.id == item("tabs", 0))
        .unwrap();
    assert_eq!(tab.role, shell::SemanticRole::Tab);
    assert_eq!(tab.states.selected, Some(true));
    assert_eq!(tab.controls, vec![panel("tabs", 0)]);
    assert!(
        !snapshot
            .nodes
            .iter()
            .any(|node| node.id == panel("tabs", 2))
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == key("switch"))
            .unwrap()
            .role,
        shell::SemanticRole::Switch
    );
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == key("heading"))
            .unwrap()
            .role,
        shell::SemanticRole::Heading
    );
    Ok(())
}
#[test]
fn switch_keyboard_toggles_once_and_disabled_control_is_inert() -> Result<(), String> {
    let (mut source, mut text) = fixture()?;
    source.set_focus(Some(key("switch")));
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut text, 480, 600);
    assert_eq!(source.actor.checkbox_value(key("switch")), Some(true));
    source.actor.dispatch_widget(key("managed"))?;
    assert_eq!(source.actor.checkbox_value(key("switch")), Some(true));
    let snapshot = source.semantic_snapshot().unwrap();
    let switch = snapshot
        .nodes
        .iter()
        .find(|node| node.id == key("switch"))
        .unwrap();
    assert_eq!(switch.label.as_deref(), Some("Airplane mode"));
    assert_eq!(switch.states.checked, Some(shell::SemanticToggleState::On));
    Ok(())
}
#[test]
#[ignore = "requires an available Vello GPU adapter"]
fn choice_visuals_render_at_one_and_two_device_scales() -> Result<(), String> {
    for scale in [1, 2] {
        let (mut source, mut text) = fixture()?;
        source.use_manual_motion_for_test();
        source.scale = scale as f64;
        for checked in [false, true] {
            if checked {
                source.actor.dispatch_widget(key("switch"))?;
                source.sync()?;
            }
            source.advance_motion_for_test(Duration::from_millis(200));
            let nodes = source.build_frame(&mut text, 480, 600);
            let mut scene = shell::Scene::new();
            shell::scene::build_scene_scaled(
                &mut scene,
                &nodes,
                &mut text,
                480,
                600,
                source.base_color(),
                scale as f64,
            );
            let image = shell::renderer::render_to_image(
                &scene,
                480 * scale,
                600 * scale,
                source.base_color(),
            )
            .map_err(|e| e.to_string())?;
            let rect = source.widget_layout_rect(key("switch")).unwrap();
            let pixel = image.get_pixel(
                ((rect[0] + 8.0) * scale as f32) as u32,
                ((rect[1] + 18.0) * scale as f32) as u32,
            );
            let expected = if checked {
                [31u8, 88, 155]
            } else {
                [210u8, 216, 225]
            };
            for channel in 0..3 {
                assert!(
                    pixel[channel].abs_diff(expected[channel]) <= 2,
                    "switch scale={scale} checked={checked}: {pixel:?}"
                );
            }
            if scale == 1 {
                std::fs::create_dir_all("../../reports").map_err(|e| e.to_string())?;
                image
                    .save(format!(
                        "../../reports/native-choices-{}.png",
                        if checked { "on" } else { "off" }
                    ))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

#[test]
fn switch_motion_retargets_without_evaluating_bindings_or_dirtying_layout() -> Result<(), String> {
    use blockui_motion::{Key as MotionKey, Property};
    let (mut source, mut text) = fixture()?;
    source.use_manual_motion_for_test();
    let track = MotionKey::new(key("switch"), Property::Switch);
    assert_eq!(source.motion.value(track), Some(0.0));
    assert!(!source.has_anim());
    source.actor.dispatch_widget(key("switch"))?;
    source.build_frame(&mut text, 480, 600);
    assert!(source.has_anim());
    let evaluations: Vec<_> = source
        .actor
        .view()
        .bindings
        .iter()
        .map(|(key, b)| (*key, b.evaluations, b.style_evaluations))
        .collect();
    let rect = source.widget_layout_rect(key("switch")).unwrap();
    source.advance_motion_for_test(Duration::from_millis(80));
    source.sync()?;
    assert_eq!(source.motion.value(track), Some(0.875));
    for id in source.ids.values() {
        assert!(!source.tree.dirty(*id).unwrap());
    }
    source.build_frame(&mut text, 480, 600);
    assert_eq!(source.widget_layout_rect(key("switch")).unwrap(), rect);
    for (key, text_count, style_count) in evaluations {
        assert_eq!(source.actor.view().bindings[&key].evaluations, text_count);
        assert_eq!(
            source.actor.view().bindings[&key].style_evaluations,
            style_count
        );
    }
    source.actor.dispatch_widget(key("switch"))?;
    source.sync()?;
    assert_eq!(
        source.motion.value(track),
        Some(0.875),
        "reverse starts at the displayed value"
    );
    source.advance_motion_for_test(Duration::from_millis(80));
    source.sync()?;
    assert_eq!(source.motion.value(track), Some(0.109375));
    source.set_reduced_motion(true);
    assert_eq!(source.motion.value(track), Some(0.0));
    assert!(!source.has_anim());
    Ok(())
}

#[test]
#[ignore = "explicit Vello animation midpoint and final pixels at 1x and 2x"]
fn switch_motion_pixels() -> Result<(), String> {
    use blockui_motion::{Key as MotionKey, Property};
    for scale in [1, 2] {
        let (mut source, mut text) = fixture()?;
        source.use_manual_motion_for_test();
        source.scale = scale as f64;
        source.actor.dispatch_widget(key("switch"))?;
        source.sync()?;
        for (name, step, amount) in [("mid", 80, 0.875), ("end", 80, 1.0)] {
            source.advance_motion_for_test(Duration::from_millis(step));
            let nodes = source.build_frame(&mut text, 480, 600);
            assert_eq!(
                source
                    .motion
                    .value(MotionKey::new(key("switch"), Property::Switch)),
                Some(amount)
            );
            let placed = nodes
                .iter()
                .find(|n| n.node_id == source.ids[&key("switch")])
                .unwrap();
            let mut scene = shell::Scene::new();
            shell::scene::build_scene_scaled(
                &mut scene,
                &nodes,
                &mut text,
                480,
                600,
                source.base_color(),
                scale as f64,
            );
            let image = shell::renderer::render_to_image(
                &scene,
                480 * scale,
                600 * scale,
                source.base_color(),
            )
            .map_err(|e| e.to_string())?;
            let cx = f64::from(placed.content_origin[0]) - 38.0 + 16.0 * amount;
            let cy = f64::from(placed.content_origin[1] + placed.content_size[1] * 0.5);
            let knob = image.get_pixel((cx * scale as f64) as u32, (cy * scale as f64) as u32);
            assert!(
                knob[0] > 250 && knob[1] > 250 && knob[2] > 250,
                "{name} knob at {cx},{cy}: {knob:?}"
            );
            // Top of the track, away from the knob, reflects the same interpolation.
            let pixel = image.get_pixel(
                ((f64::from(placed.content_origin[0]) - 30.0) * scale as f64) as u32,
                ((cy - 9.0) * scale as f64) as u32,
            );
            let expected = Color::from_rgb8(210, 216, 225)
                .lerp_rect(Color::from_rgb8(31, 88, 155), amount as f32)
                .to_rgba8()
                .to_u8_array();
            for channel in 0..3 {
                assert!(
                    pixel[channel].abs_diff(expected[channel]) <= 2,
                    "{name} track: {pixel:?}, expected {expected:?}"
                );
            }
            if scale == 1 {
                std::fs::create_dir_all("../../reports").map_err(|e| e.to_string())?;
                image
                    .save(format!("../../reports/native-motion-{name}.png"))
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}
