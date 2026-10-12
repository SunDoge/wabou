use super::*;
#[test]
fn select_popup_closes_when_resize_leaves_no_room() -> Result<(), String> {
    let actor = Actor::for_test::<crate::select_tests::Choices>()?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("choice.input");
    source.build_frame(&mut tcx, 320, 240);
    source.set_semantics_enabled(true);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: input }));
    source.build_frame(&mut tcx, 320, 240);
    let snapshot = source.semantic_snapshot().unwrap();
    let combo = snapshot.nodes.iter().find(|node| node.id == input).unwrap();
    let stale_option = combo.active_descendant.unwrap();
    source.select_pressed = Some(10);
    source.build_frame(&mut tcx, 320, 39);
    assert!(!source.selects[&input].is_open());
    assert!(source.select_popup.is_none());
    assert!(source.select_pressed.is_none());
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.actor.integer("choice.value"), 10);
    let snapshot = source.semantic_snapshot().unwrap();
    let combo = snapshot.nodes.iter().find(|node| node.id == input).unwrap();
    assert_eq!(combo.states.expanded, Some(false));
    assert!(combo.controls.is_empty());
    assert!(!snapshot.nodes.iter().any(|node| node.id == stale_option));
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Click {
            target: stale_option
        })
    );
    source.build_frame(&mut tcx, 320, 240);
    assert!(!source.selects[&input].is_open());
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: input }));
    source.build_frame(&mut tcx, 320, 240);
    assert!(source.selects[&input].is_open());
    Ok(())
}
#[test]
fn changed_select_declaration_rebuilds_options_and_uses_current_actions() -> Result<(), String> {
    let actor = Actor::for_test::<crate::select_tests::Choices>()?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("choice.input");
    source.build_frame(&mut tcx, 320, 240);
    source.set_focus(Some(input));
    source
        .selects
        .get_mut(&input)
        .unwrap()
        .key(crate::select_model::Key::Activate, false);
    source.selects.get_mut(&input).unwrap().highlight(40);
    let node = source.ids[&input];
    source
        .actor
        .replace_for_test::<crate::select_tests::RevisedChoices>()?;
    source.sync()?;
    source.build_frame(&mut tcx, 320, 240);
    let control = &source.selects[&input];
    assert_eq!(
        control
            .options()
            .iter()
            .map(|item| item.value)
            .collect::<Vec<_>>(),
        [40, 10, 30, 20]
    );
    assert_eq!(control.options()[0].label, "Updated 40");
    assert!(control.is_open());
    assert_eq!(control.active(), Some(40));
    assert_eq!(control.selected(), Some(10));
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.focus, Some(input));
    assert_eq!(
        source.tree.get_node_context(node).unwrap().text.as_deref(),
        Some("Updated 10")
    );
    assert!(source.actor.edit_select(input, 30).is_err());
    source.actor.edit_select(input, 20)?; // Newly enabled; current implementation accepts it.
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.selects[&input].selected(), Some(20));
    assert_eq!(
        source.tree.get_node_context(node).unwrap().text.as_deref(),
        Some("Updated 20")
    );
    Ok(())
}
#[test]
fn select_closed_projection_keyboard_and_rejection() -> Result<(), String> {
    let key_event = |name: &str| {
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
    };
    let actor = Actor::for_test::<crate::select_tests::Choices>()?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("choice.input");
    source.build_frame(&mut tcx, 320, 240);
    source.set_semantics_enabled(true);
    let semantic = source.semantic_snapshot().unwrap();
    let control = semantic.nodes.iter().find(|node| node.id == input).unwrap();
    assert_eq!(control.role, shell::SemanticRole::ComboBox);
    assert_eq!(control.label.as_deref(), Some("Flight type"));
    assert_eq!(control.value.as_deref(), Some("航程 10 🦀"));
    assert_eq!(control.states.expanded, Some(false));
    assert_eq!(control.states.popup, Some(shell::SemanticPopup::ListBox));
    let node = source.ids[&input];
    let options_buffer = source.selects[&input].options().as_ptr();
    let label_buffer = source.selects[&input].options()[0].label.as_ptr();
    let rect = source.widget_layout_rect(input).unwrap();
    assert!(rect[2] > rect[0] && rect[2] <= 320.01 && rect[3] <= 240.01);
    source.set_focus(Some(input));
    source.handle_event(key_event("ArrowDown"));
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.actor.integer("choice.value"), 10); // 20 disabled; 30 rejected
    assert_eq!(source.selects[&input].selected(), Some(10));
    assert_eq!(source.diagnostic, "choice rejected");
    source.handle_event(key_event("PageDown"));
    source.build_frame(&mut tcx, 640, 240);
    assert_eq!(source.actor.integer("choice.value"), 40);
    source.handle_event(key_event("End"));
    source.build_frame(&mut tcx, 640, 240);
    assert_eq!(source.actor.integer("choice.value"), 40);
    assert_eq!(source.selects[&input].selected(), Some(40));
    assert_eq!(source.selects[&input].options().as_ptr(), options_buffer);
    assert_eq!(
        source.selects[&input].options()[0].label.as_ptr(),
        label_buffer
    );
    assert_eq!(source.ids[&input], node);
    source
        .actor
        .replace_for_test::<crate::select_tests::Choices>()?;
    source.sync()?;
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.selects[&input].selected(), Some(40));
    assert_eq!(source.selects[&input].options().as_ptr(), options_buffer);
    assert_eq!(
        source.selects[&input].options()[0].label.as_ptr(),
        label_buffer
    );
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        source.tree.get_node_context(node).unwrap().text.as_deref(),
        Some("航程 40 🦀")
    );
    source.handle_event(key_event("Enter"));
    let frame = source.build_frame(&mut tcx, 320, 240);
    assert!(source.selects[&input].is_open());
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == input)
            .unwrap()
            .states
            .expanded,
        Some(true)
    );
    assert_eq!(frame.len(), source.placed.len() + 4);
    let popup_ids: std::collections::HashSet<_> = frame.iter().map(|node| node.node_id).collect();
    assert_eq!(popup_ids.len(), frame.len());
    source.handle_event(key_event("Home"));
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.actor.integer("choice.value"), 40);
    assert_eq!(source.selects[&input].active(), Some(10));
    source
        .actor
        .replace_for_test::<crate::select_tests::Choices>()?;
    source.sync()?;
    source.build_frame(&mut tcx, 320, 240);
    assert!(source.selects[&input].is_open());
    assert_eq!(source.selects[&input].active(), Some(10));
    let layout = source.select_popup.as_ref().unwrap().1;
    let first_row = layout.row_rect(0).unwrap();
    let position = shell::Point {
        x: f64::from(first_row[0] + 12.0),
        y: f64::from(first_row[1] + 12.0),
    };
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position,
        delta_x: 0.0,
        delta_y: 20.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.selects[&input].active(), Some(30));
    assert_eq!(source.actor.integer("choice.value"), 40);
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: PointerPhase::Move,
        position,
        button: None,
        buttons: 0,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.selects[&input].active(), Some(10));
    assert_eq!(source.actor.integer("choice.value"), 40);
    let rect = layout.row_rect(1).unwrap(); // disabled option 20
    let click = |source: &mut NativeSource, rect: [f32; 4]| {
        let position = shell::Point {
            x: f64::from((rect[0] + rect[2]) / 2.0),
            y: f64::from((rect[1] + rect[3]) / 2.0),
        };
        for phase in [PointerPhase::Down, PointerPhase::Up] {
            source.handle_event(UiEvent::Pointer(shell::PointerEvent {
                phase,
                position,
                button: Some(shell::PointerButton::Primary),
                buttons: if phase == PointerPhase::Down { 1 } else { 0 },
                modifiers: Default::default(),
                properties: Default::default(),
            }));
        }
    };
    click(&mut source, rect);
    assert!(source.selects[&input].is_open());
    assert_eq!(source.actor.integer("choice.value"), 40);
    click(&mut source, layout.row_rect(0).unwrap());
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.actor.integer("choice.value"), 10);
    assert!(!source.selects[&input].is_open());
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 320, 240);
    source.handle_event(key_event("End"));
    assert_eq!(source.selects[&input].active(), Some(40));
    source.select_pressed = Some(40);
    source.handle_event(key_event("Escape"));
    assert!(source.select_pressed.is_none());
    assert!(!source.selects[&input].is_open());
    assert_eq!(source.actor.integer("choice.value"), 10);
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.selects[&input].active(), Some(10));
    source.handle_event(key_event("Escape"));
    source.handle_event(key_event("Enter"));
    source.build_frame(&mut tcx, 320, 240);
    source.select_pressed = Some(40);
    source.handle_event(key_event("Home"));
    assert!(source.select_pressed.is_none());
    source.select_pressed = Some(40);
    source.handle_event(UiEvent::Focus(false));
    source.build_frame(&mut tcx, 320, 240);
    assert!(!source.selects[&input].is_open());
    assert!(source.select_pressed.is_none());
    source.handle_event(UiEvent::Focus(true));
    source.handle_event(UiEvent::TextInput("航程 40".into()));
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(source.actor.integer("choice.value"), 40);
    assert_eq!(source.selects[&input].selected(), Some(40));
    assert_eq!(source.selects[&input].options().as_ptr(), options_buffer);
    assert_eq!(
        source.selects[&input].options()[0].label.as_ptr(),
        label_buffer
    );
    let semantic = source.semantic_snapshot().unwrap();
    let control = semantic.nodes.iter().find(|node| node.id == input).unwrap();
    assert_eq!(control.value.as_deref(), Some("航程 40 🦀"));
    assert_eq!(control.states.expanded, Some(false));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: input }));
    source.build_frame(&mut tcx, 320, 240);
    let semantic = source.semantic_snapshot().unwrap();
    let control = semantic.nodes.iter().find(|node| node.id == input).unwrap();
    let list = semantic
        .nodes
        .iter()
        .find(|node| node.id == control.controls[0])
        .unwrap();
    assert_eq!(list.role, shell::SemanticRole::ListBox);
    assert_eq!(list.children.len(), 4);
    let option = semantic
        .nodes
        .iter()
        .find(|node| {
            node.role == shell::SemanticRole::Option && node.label.as_deref() == Some("航程 10 🦀")
        })
        .unwrap()
        .id;
    let disabled = semantic
        .nodes
        .iter()
        .find(|node| node.role == shell::SemanticRole::Option && node.disabled)
        .unwrap()
        .id;
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: disabled }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: option }));
    source.build_frame(&mut tcx, 320, 240);
    assert_eq!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .find(|node| node.id == input)
            .unwrap()
            .active_descendant,
        Some(option)
    );
    source
        .actor
        .replace_for_test::<crate::select_tests::Choices>()?;
    source.sync()?;
    source.build_frame(&mut tcx, 320, 240);
    assert!(
        source
            .semantic_snapshot()
            .unwrap()
            .nodes
            .iter()
            .any(|node| node.id == option)
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: option }));
    assert_eq!(source.actor.integer("choice.value"), 10);
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: option }));
    Ok(())
}

#[test]
#[ignore = "explicit Vello Hybrid offscreen pixel proof"]
fn select_popup_pixels() -> Result<(), String> {
    let actor = Actor::for_test::<crate::select_tests::Choices>()?;
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    let input = key("choice.input");
    source.build_frame(&mut tcx, 320, 240);
    source.set_focus(Some(input));
    source
        .selects
        .get_mut(&input)
        .unwrap()
        .key(crate::select_model::Key::Activate, false);
    for width in [320, 640] {
        let nodes = source.build_frame(&mut tcx, width, 240);
        let layout = source.select_popup.as_ref().unwrap().1;
        for scale in [1, 2] {
            let mut scene = shell::Scene::new();
            shell::scene::build_scene_scaled(
                &mut scene,
                &nodes,
                &mut tcx,
                width,
                240,
                Color::from_rgb8(240, 240, 240),
                f64::from(scale),
            );
            let image = shell::renderer::render_to_image(
                &scene,
                width * scale,
                240 * scale,
                Color::from_rgb8(240, 240, 240),
            )
            .map_err(|error| error.to_string())?;
            for (index, expected) in [(0, [225u8, 235, 249]), (1, [255u8, 255, 255])] {
                let row = layout.row_rect(index).unwrap();
                let x = ((row[2] - 4.0) * scale as f32) as u32;
                let y = ((row[1] + 16.0) * scale as f32) as u32;
                let pixel = image.get_pixel(x, y);
                for channel in 0..3 {
                    assert!(
                        pixel[channel].abs_diff(expected[channel]) <= 2,
                        "row {index}, scale {scale}: {pixel:?}"
                    );
                }
            }
            for index in 0..4 {
                let row = layout.row_rect(index).unwrap();
                let x = ((row[0] + 0.5) * scale as f32) as u32;
                let y = ((row[1] + 16.0) * scale as f32) as u32;
                let pixel = image.get_pixel(x, y);
                for (channel, expected) in [170u8, 180, 195].into_iter().enumerate() {
                    assert!(
                        pixel[channel].abs_diff(expected) <= 3,
                        "missing popup boundary row {index}, scale {scale}: {pixel:?}"
                    );
                }
            }
            for index in 0..4 {
                let row = layout.row_rect(index).unwrap();
                let mut ink = 0;
                for y in
                    ((row[1] + 6.0) * scale as f32) as u32..((row[3] - 4.0) * scale as f32) as u32
                {
                    for x in ((row[0] + 8.0) * scale as f32) as u32
                        ..((row[2] - 8.0) * scale as f32) as u32
                    {
                        let pixel = image.get_pixel(x, y);
                        if pixel[0] < 160 && pixel[1] < 160 && pixel[2] < 160 {
                            ink += 1;
                        }
                    }
                }
                assert!(
                    ink > 5,
                    "missing option label pixels: {index}, scale {scale}"
                );
            }
            if width == 320 && scale == 1 {
                let reports = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../reports");
                std::fs::create_dir_all(&reports).map_err(|error| error.to_string())?;
                image
                    .save(reports.join("select-popup.png"))
                    .map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}

#[test]
fn queued_control_values_request_redraw_without_declaration_changes() -> Result<(), String> {
    use blockui::{Action, Column, Cx, IntoView, State, StateSpec, View};
    const VALUE: State<i64> = State::<i64>::new("async.controls.value", 0);
    const SET: Action = Action::new("async.controls.set", |cx, value| VALUE.set(cx, value));
    struct Controls;
    impl blockui::Actor for Controls {
        const STATE: &'static [StateSpec] = &[VALUE.spec()];
        const ACTIONS: &'static [Action] = &[SET];
        fn view(_: &Cx<'_>) -> blockui::Result<View> {
            Ok(Column::default().children(vec![
                blockui::Select::new(
                    "Value",
                    VALUE,
                    [0, 1]
                        .into_iter()
                        .map(|value| blockui::SelectOption {
                            value,
                            label: value.to_string(),
                            enabled: true,
                        })
                        .collect(),
                    |cx: &Cx<'_>, value| VALUE.set(cx, value),
                )
                .into_view()
                .key("async.select"),
                blockui::Slider::new(
                    VALUE,
                    blockui::IntegerRange {
                        min: 0,
                        max: 10,
                        step: 1,
                    },
                    |cx: &Cx<'_>, value| VALUE.set(cx, value),
                )
                .into_view()
                .key("async.slider"),
            ]))
        }
    }
    let mut source = NativeSource::without_watch(Actor::for_test::<Controls>()?)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 640, 240);
    let keys = [key("async.select"), key("async.slider")];
    let declarations: Vec<_> = keys
        .iter()
        .map(|key| {
            let binding = &source.actor.view.bindings[key];
            (binding.node, binding.text.clone(), source.ids[key])
        })
        .collect();
    assert!(
        !source.poll_async(),
        "settled controls must not request redraw"
    );
    source.actor.tell(SET, 1)?;
    assert!(
        source.poll_async(),
        "queued value changes must request redraw"
    );
    for (key, (node, text, _)) in keys.iter().zip(&declarations) {
        let binding = &source.actor.view.bindings[key];
        assert_eq!(binding.node, *node);
        assert_eq!(binding.text, *text);
    }
    source.build_frame(&mut tcx, 640, 240);
    assert_eq!(source.selects[&keys[0]].selected(), Some(1));
    assert_eq!(source.sliders[&keys[1]].value(), 1);
    for (key, (_, _, id)) in keys.iter().zip(&declarations) {
        assert_eq!(source.ids[key], *id);
    }
    assert!(!source.poll_async());
    Ok(())
}
