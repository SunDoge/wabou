use super::*;
use blockui_abi as abi;
#[path = "seven_identity_fixture.rs"]
mod identities;
use identities::{seven_action, seven_key as key, seven_state};

fn counter_controls_proof(paths: &[String]) -> Result<(), String> {
    let mut actor = Actor::new(719, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    let state = actor.state_address();
    let slot = actor.slot_address_by_key(seven_state("counter.count"));
    let increment = key("counter.increment.button");
    let reset = key("counter.reset.button");
    let value = key("counter.value");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 900, 720);
    let increment_node = source.ids[&increment];
    let reset_node = source.ids[&reset];
    assert_eq!(source.actor.view.bindings[&reset].node.disabled, 1);
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: reset }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: increment }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: increment }));
    assert_eq!(source.actor.integer_by_key(seven_state("counter.count")), 1);
    let rect = source.widget_layout_rect(increment).unwrap();
    let position = shell::Point {
        x: f64::from((rect[0] + rect[2]) / 2.0),
        y: f64::from((rect[1] + rect[3]) / 2.0),
    };
    source.handle_event(UiEvent::Pointer(shell::PointerEvent {
        phase: PointerPhase::Move,
        position,
        buttons: 0,
        button: None,
        modifiers: Default::default(),
        properties: Default::default(),
    }));
    source.build_frame(&mut tcx, 900, 720);
    assert_eq!(source.hovered, Some(increment));
    let hovered_paint = source
        .tree
        .get_node_context(increment_node)
        .unwrap()
        .background;
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 900, 720);
    assert_eq!(source.actor.integer_by_key(seven_state("counter.count")), 1);
    assert_eq!(source.hovered, Some(increment));
    assert_eq!(source.ids[&increment], increment_node);
    assert_eq!(
        source
            .tree
            .get_node_context(increment_node)
            .unwrap()
            .background,
        hovered_paint
    );
    println!(
        "PASS native hover: stationary pointer, button paint and identity survive actual DLL replacement"
    );

    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: increment }));
    assert_eq!(
        source.actor.integer_by_key(seven_state("counter.count")),
        3,
        "replacement must execute new increment code"
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: reset }));
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 900, 720);
    assert_eq!(source.actor.integer_by_key(seven_state("counter.count")), 0);
    assert_eq!(source.actor.view.bindings[&value].text, "0");
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(
        source
            .actor
            .slot_address_by_key(seven_state("counter.count")),
        slot
    );
    assert_eq!(source.focus, Some(increment));
    assert_eq!(source.ids[&increment], increment_node);
    assert_eq!(source.ids[&reset], reset_node);
    for count in [i64::MAX, i64::MIN, 0] {
        source.actor.dispatch(Message {
            action: key("counter.reset"),
            value: 0,
        })?;
        // Only the boundary display case uses direct host setup; actions above
        // prove actual native input and current implementation behavior.
        source.actor.transaction(|actor| {
            let table = actor.implementation.as_ref().unwrap().table;
            let code = crate::with_api(&mut actor.turn, table, |api| unsafe {
                (api.write_i64)(api.context, key("counter.count"), count)
            });
            assert_eq!(code, blockui_abi::OK);
            actor.flush()
        })?;
        for width in [320, 900] {
            source.build_frame(&mut tcx, width, 520);
            assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
            let mut bottom = 0.0;
            for widget in [key("counter.heading"), value, increment] {
                let rect = source
                    .widget_layout_rect(widget)
                    .ok_or("counter bounds missing")?;
                assert!(
                    rect[0] >= 0.0
                        && rect[1] >= bottom
                        && rect[2] <= width as f32
                        && rect[3] <= 520.0,
                    "{widget}: {rect:?}"
                );
                bottom = rect[3];
            }
            assert_eq!(source.actor.view.bindings[&value].text, count.to_string());
            let paint = source.tree.get_node_context(source.ids[&value]).unwrap();
            assert_eq!(paint.text.as_deref(), Some(count.to_string().as_str()));
            assert_eq!(paint.font_size, if count == 0 { 36.0 } else { 16.0 });
        }
    }
    println!(
        "PASS Counter controls: reset-to-zero and subsequent actual DLL replacement retain state/slot/button nodes/focus; new code changes increment step, disabled reset is ignored, large signed values and narrow/normal controls fit"
    );
    Ok(())
}

fn select_field_upgrade_proof(flat: &Path, grouped: &Path) -> Result<(), String> {
    let mut actor = Actor::new(721, Path::new(".runtime"))?;
    actor.reload(flat)?;
    actor.dispatch(Message {
        action: key("seven.select"),
        value: 2,
    })?;
    let departure = key("flight.departure.input");
    actor.edit_text(departure, "2026-")?;
    let draft_slot = actor.slot_address_by_key(seven_state("flight.departure"));
    let state = actor.state_address();
    let slot = actor.slot_address_by_key(seven_state("flight.trip_kind"));
    let trip = key("flight.trip.input");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 900, 720);
    source
        .editors
        .get_mut(&departure)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":0,"head":4}}"#)?;
    source.build_frame(&mut tcx, 900, 720);
    let editor = (&**source.editors.get(&departure).unwrap() as *const TextInput) as usize;
    let editor_node = source.ids[&departure];
    let editor_parent = source.actor.view.bindings[&departure].node.parent;
    let selection = source.editors[&departure].text_selection();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: trip }));
    source.handle_event(UiEvent::Key(shell::KeyEvent {
        phase: shell::KeyPhase::Down,
        key: "End".into(),
        key_without_modifiers: "End".into(),
        code: "End".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: Default::default(),
        repeat: false,
        synthetic: false,
    }));
    source.build_frame(&mut tcx, 900, 720);
    let node = source.ids[&trip];
    let parent = source.actor.view.bindings[&trip].node.parent;
    let options = source.selects[&trip].options().as_ptr();
    let option = source
        .semantic_snapshot()
        .unwrap()
        .nodes
        .iter()
        .find(|n| n.id == trip)
        .unwrap()
        .active_descendant
        .unwrap();
    assert_eq!(source.selects[&trip].active(), Some(1));
    source.replace(grouped)?;
    for width in [900, 320, 600] {
        source.build_frame(&mut tcx, width, 720);
        assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
        assert_eq!(source.actor.state_address(), state);
        assert_eq!(
            source
                .actor
                .slot_address_by_key(seven_state("flight.departure")),
            draft_slot
        );
        assert_eq!(
            source.actor.text_by_key(seven_state("flight.departure")),
            "2026-"
        );
        assert_ne!(
            source.actor.view.bindings[&departure].node.parent,
            editor_parent
        );
        assert_eq!(source.ids[&departure], editor_node);
        assert_eq!(
            (&**source.editors.get(&departure).unwrap() as *const TextInput) as usize,
            editor
        );
        assert_eq!(source.editors[&departure].text_selection(), selection);
        assert_eq!(
            source.actor.view.bindings[&key("flight.book.button")]
                .node
                .disabled,
            1
        );
        let feedback = source
            .actor
            .view
            .bindings
            .iter()
            .find(|(_, binding)| binding.text == "Invalid departure (YYYY-MM-DD).")
            .expect("invalid date needs inline feedback");
        assert_eq!(
            feedback.1.node.parent,
            source.actor.view.bindings[&departure].node.parent
        );
        let feedback_node = source.ids[feedback.0];
        assert!(source.tree.layout(feedback_node).unwrap().size.height > 0.0);
        let field = source.ids[&feedback.1.node.parent];
        let children = source.tree.children(field).unwrap();
        assert!(
            children.iter().position(|id| *id == editor_node).unwrap()
                < children.iter().position(|id| *id == feedback_node).unwrap()
        );

        assert_eq!(
            source
                .actor
                .slot_address_by_key(seven_state("flight.trip_kind")),
            slot
        );
        assert_ne!(source.actor.view.bindings[&trip].node.parent, parent);
        assert_eq!(source.ids[&trip], node);
        assert_eq!(source.focus, Some(trip));
        assert!(source.selects[&trip].is_open());
        assert_eq!(source.selects[&trip].active(), Some(1));
        assert_eq!(source.selects[&trip].options().as_ptr(), options);
        let snapshot = source.semantic_snapshot().unwrap();
        let combo = snapshot.nodes.iter().find(|n| n.id == trip).unwrap();
        assert_eq!(combo.label.as_deref(), Some("Flight type"));
        assert_eq!(combo.active_descendant, Some(option));
        let rect = source.widget_layout_rect(trip).unwrap();
        assert!(rect[0] >= 0.0 && rect[2] <= width as f32 && rect[3] <= 720.0);
    }
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: option }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.actor.integer_by_key(seven_state("flight.trip_kind")),
        1
    );
    assert!(!source.selects[&trip].is_open());
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: option }));
    println!(
        "PASS native Select Field structural upgrade: reparented open popup retains state slot/node/options/focus/highlight/semantic option through actual DLL replacement and responsive resize; current action commits and stale option retires"
    );
    Ok(())
}

fn field_upgrade_proof(flat: &Path, grouped: &Path) -> Result<(), String> {
    let mut actor = Actor::new(720, Path::new(".runtime"))?;
    actor.reload(flat)?;
    actor.dispatch(Message {
        action: key("seven.select"),
        value: 1,
    })?;
    actor.set_text(key("temperature.celsius"), "A😀bc")?;
    let state = actor.state_address();
    let slot = actor.slot_address_by_key(seven_state("temperature.celsius"));
    let input = key("temperature.celsius.input");
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 600, 520);
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)?;
    source.build_frame(&mut tcx, 600, 520);
    let selection = source.editors[&input].text_selection();
    let node = source.ids[&input];
    let allocation = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let old_parent = source.actor.view.bindings[&input].node.parent;
    source.replace(grouped)?;
    source.build_frame(&mut tcx, 900, 520);
    let heading = key("temperature.heading");
    let normal_title = source.widget_layout_rect(heading).unwrap();
    let normal_title_height = normal_title[3] - normal_title[1];
    for width in [600, 320, 900] {
        source.build_frame(&mut tcx, width, 520);
        assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
        assert_eq!(source.actor.state_address(), state);
        assert_eq!(
            source
                .actor
                .slot_address_by_key(seven_state("temperature.celsius")),
            slot
        );
        assert_eq!(
            source.actor.text_by_key(seven_state("temperature.celsius")),
            "A😀bc"
        );
        assert_eq!(source.ids[&input], node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            allocation
        );
        assert_eq!(source.focus, Some(input));
        assert_eq!(source.editors[&input].text_selection(), selection);
        let parent = source.actor.view.bindings[&input].node.parent;
        assert_ne!(
            parent, old_parent,
            "upgrade must actually reparent the editor"
        );
        let caption = source
            .actor
            .view
            .bindings
            .iter()
            .find(|(_, binding)| binding.node.parent == parent && binding.node.kind == LABEL)
            .unwrap()
            .0;
        let title_rect = source.widget_layout_rect(heading).unwrap();
        if width == 320 {
            assert!(
                title_rect[3] - title_rect[1] > normal_title_height * 1.5,
                "narrow title must grow for its second line: {title_rect:?}, normal {normal_title:?}"
            );
        }
        let caption_rect = source.widget_layout_rect(*caption).unwrap();
        assert!(
            caption_rect[1] >= title_rect[3],
            "wrapped title must not overlap its field"
        );
        let input_rect = source.widget_layout_rect(input).unwrap();
        assert!((input_rect[1] - caption_rect[3] - 6.0).abs() < 0.1);
        assert!(input_rect[2] <= width as f32 && input_rect[3] <= 520.0);
        let snapshot = source.semantic_snapshot().unwrap();
        assert!(snapshot.nodes.iter().any(|n| n.id == input
            && n.label.as_deref() == Some("Celsius")
            && n.value.as_deref() == Some("A😀bc")));
    }
    source.handle_event(UiEvent::TextInput("🦀".into()));
    source.build_frame(&mut tcx, 900, 520);
    assert_eq!(
        source.actor.text_by_key(seven_state("temperature.celsius")),
        "A🦀bc"
    );
    assert_eq!(
        source
            .actor
            .slot_address_by_key(seven_state("temperature.celsius")),
        slot
    );
    println!(
        "PASS native Field composition upgrade: flat-to-grouped actual DLL replacement and responsive resize retain state slot/editor allocation/Taffy node/Unicode draft/focus/UTF16 selection; caption/control spacing and accessible name are consistent, post-upgrade edits use current handler"
    );
    Ok(())
}

pub fn proof(paths: &[String], window: bool) -> Result<(), String> {
    if !(2..=4).contains(&paths.len()) {
        return Err(
            "seven GUIs needs base, replacement and optional tall-viewport/flat-field libraries"
                .into(),
        );
    }
    counter_controls_proof(paths)?;
    if let Some(flat) = paths.get(3) {
        field_upgrade_proof(Path::new(flat), Path::new(&paths[0]))?;
        select_field_upgrade_proof(Path::new(flat), Path::new(&paths[1]))?;
    }
    {
        let mut timer = crate::Actor::new(707, std::path::Path::new(".runtime"))?;
        timer.reload(std::path::Path::new(&paths[0]))?;
        let run = |actor: &mut crate::Actor, action, value| {
            actor.dispatch(crate::Message {
                action: seven_action(action),
                value,
            })
        };
        run(&mut timer, "timer.reset", 0)?;
        let baseline = timer.integer_by_key(seven_state("timer.baseline"));
        let elapsed_slot = timer.slot_address_by_key(seven_state("timer.elapsed"));
        let task = timer.tasks[&key("timer.clock")].generation;
        std::thread::sleep(std::time::Duration::from_millis(70));
        timer.reload(std::path::Path::new(&paths[1]))?;
        assert_eq!(
            timer.integer_by_key(seven_state("timer.baseline")),
            baseline
        );
        assert_eq!(
            timer.slot_address_by_key(seven_state("timer.elapsed")),
            elapsed_slot
        );
        assert_eq!(timer.tasks[&key("timer.clock")].generation, task);
        timer.tick()?;
        assert!(timer.integer_by_key(seven_state("timer.elapsed")) >= 50);
        run(&mut timer, "timer.duration.change", 0)?;
        assert_eq!(timer.integer_by_key(seven_state("timer.elapsed")), 0);
        assert!(timer.tasks.is_empty());
        assert!(run(&mut timer, "timer.duration.change", 60001).is_err());
        assert_eq!(timer.integer_by_key(seven_state("timer.duration")), 0);
        run(&mut timer, "timer.duration.change", 100)?;
        std::thread::sleep(std::time::Duration::from_millis(120));
        timer.tick()?;
        assert_eq!(timer.integer_by_key(seven_state("timer.elapsed")), 100);
        assert!(timer.tasks.is_empty());
    }
    {
        let mut drawing = crate::Actor::new(708, Path::new(".runtime"))?;
        drawing.reload(Path::new(&paths[0]))?;
        let run = |actor: &mut crate::Actor, action, value| {
            actor.dispatch(crate::Message {
                action: seven_action(action),
                value,
            })
        };
        let point = blockui_abi::LocalPoint { x: 10.5, y: 20.25 }
            .to_wire()
            .unwrap();
        run(&mut drawing, "circle.add", point)?;
        run(&mut drawing, "circle.edit", 0)?;
        run(&mut drawing, "circle.preview", 100)?;
        let slot = drawing.slot_address_by_key(seven_state("circle.model"));
        let retained = drawing.list_by_key(seven_state("circle.model")).to_vec();
        let allocation = drawing.list_by_key(seven_state("circle.model")).as_ptr();
        drawing.reload(Path::new(&paths[1]))?;
        assert_eq!(
            drawing.slot_address_by_key(seven_state("circle.model")),
            slot
        );
        assert_eq!(
            drawing.list_by_key(seven_state("circle.model")).as_ptr(),
            allocation
        );
        assert_eq!(drawing.list_by_key(seven_state("circle.model")), retained);
        assert!(run(&mut drawing, "circle.add", f32::NAN.to_bits() as i64).is_err());
        assert_eq!(drawing.list_by_key(seven_state("circle.model")), retained);
        assert!(run(&mut drawing, "circle.undo", 0).is_err());
        assert_eq!(drawing.list_by_key(seven_state("circle.model")), retained);
        run(&mut drawing, "circle.finish", 1)?;
        run(&mut drawing, "circle.undo", 0)?;
        assert_eq!(drawing.list_by_key(seven_state("circle.model"))[11], 52);
        drawing.reload(Path::new(&paths[0]))?;
        run(&mut drawing, "circle.redo", 0)?;
        assert_eq!(drawing.list_by_key(seven_state("circle.model"))[11], 100);
        run(&mut drawing, "circle.undo", 0)?;
        run(&mut drawing, "circle.undo", 0)?;
        drawing.reload(Path::new(&paths[1]))?;
        assert_eq!(drawing.list_by_key(seven_state("circle.model"))[5], 0);
        run(&mut drawing, "circle.add", point)?;
        assert_eq!(drawing.list_by_key(seven_state("circle.model"))[8], 2);
        assert_eq!(drawing.list_by_key(seven_state("circle.model"))[7], 0);
    }
    {
        let mut cells = crate::Actor::new(709, Path::new(".runtime"))?;
        cells.reload(Path::new(&paths[0]))?;
        let run = |actor: &mut crate::Actor, action, value| {
            actor.dispatch(crate::Message {
                action: seven_action(action),
                value,
            })
        };
        {
            let mut cells = crate::Actor::new(98, Path::new(".runtime"))?;
            cells.reload(Path::new(&paths[0]))?;
            cells.view.set_renderer_retention(256)?;
            run(&mut cells, "seven.select", 3)?;
            cells
                .view
                .renderer
                .get_mut(&key("crud.list"))
                .unwrap()
                .scroll = 77.0;
            run(&mut cells, "seven.select", 0)?;
            cells.reload(Path::new(&paths[1]))?;
            run(&mut cells, "seven.select", 3)?;
            assert_eq!(cells.view.renderer[&key("crud.list")].scroll, 77.0);
        }
        let draft = |actor: &mut crate::Actor, text: &str| {
            let mut envelope: crate::Envelope = crate::Message {
                action: seven_action("cells.draft"),
                value: 0,
            }
            .into();
            envelope.message_text = Some(text.to_owned());
            actor.mailbox.push_back(envelope);
            actor.tick()
        };
        run(&mut cells, "seven.select", 1)?;
        run(&mut cells, "seven.select", 2)?;
        run(&mut cells, "seven.back", 0)?;
        let history_allocation = cells
            .list_by_key(seven_state("seven.route.forward"))
            .as_ptr();
        draft(&mut cells, "2")?;
        run(&mut cells, "cells.commit", 0)?;
        run(&mut cells, "cells.choose", 1)?;
        draft(&mut cells, "=A1*3")?;
        run(&mut cells, "cells.commit", 0)?;
        draft(&mut cells, "=(unfinished")?;
        let slot = cells.slot_address_by_key(seven_state("cells.sources"));
        let allocation = cells
            .text_list_by_key(seven_state("cells.sources"))
            .as_ptr();
        let retained = cells
            .text_list_by_key(seven_state("cells.sources"))
            .to_vec();
        cells.reload(Path::new(&paths[1]))?;
        assert_eq!(cells.integer_by_key(seven_state("seven.page")), 1);
        assert_eq!(
            cells
                .list_by_key(seven_state("seven.route.forward"))
                .as_ptr(),
            history_allocation
        );
        run(&mut cells, "seven.forward", 0)?;
        assert_eq!(cells.integer_by_key(seven_state("seven.page")), 2);
        run(&mut cells, "seven.back", 0)?;
        run(&mut cells, "seven.select", 3)?;
        assert!(
            cells
                .list_by_key(seven_state("seven.route.forward"))
                .is_empty()
        );
        assert_eq!(
            cells.slot_address_by_key(seven_state("cells.sources")),
            slot
        );
        assert_eq!(
            cells
                .text_list_by_key(seven_state("cells.sources"))
                .as_ptr(),
            allocation
        );
        assert_eq!(
            cells.text_list_by_key(seven_state("cells.sources")),
            retained
        );
        assert_eq!(cells.integer_by_key(seven_state("cells.active")), 1);
        assert_eq!(
            cells.text_by_key(seven_state("cells.draft")),
            "=(unfinished"
        );
        assert!(run(&mut cells, "cells.choose", 2600).is_err());
        assert!(draft(&mut cells, &"x".repeat(1025)).is_err());
        assert_eq!(
            cells.text_by_key(seven_state("cells.draft")),
            "=(unfinished"
        );
        run(&mut cells, "cells.cancel", 0)?;
        assert_eq!(cells.text_by_key(seven_state("cells.draft")), "=A1*3");
        run(&mut cells, "cells.choose", 2599)?;
        draft(&mut cells, "=SUM(A1:B1)")?;
        run(&mut cells, "cells.commit", 0)?;
        assert_eq!(
            cells.text_list_by_key(seven_state("cells.sources")).len(),
            2600
        );
        cells.reload(Path::new(&paths[0]))?;
        assert_eq!(cells.integer_by_key(seven_state("cells.active")), 2599);
        assert_eq!(
            cells.text_list_by_key(seven_state("cells.sources"))[2599],
            "=SUM(A1:B1)"
        );
        draft(&mut cells, "")?;
        run(&mut cells, "cells.commit", 0)?;
        assert_eq!(
            cells.text_list_by_key(seven_state("cells.sources")).len(),
            2
        );
    }
    let root = ActorId(1);
    let (mut source, options) = crate::App::new(&paths[0], ".runtime")
        .window(
            crate::app::WindowOptions::new()
                .title("7GUIs")
                .initial_inner_size(900, 720),
        )
        .into_window()?;
    source.set_semantics_enabled(true);
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 600, 520);
    let increment = *source
        .actor
        .view()
        .bindings
        .iter()
        .find(|(_, b)| b.node.action == key("counter.increment"))
        .unwrap()
        .0;
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: increment }));
    let tab = source
        .actor
        .widget(root, key("seven.temperature.tab"))
        .unwrap();
    let tab_node = source.ids[&tab];
    let paint = source.tree.get_node_context(tab_node).unwrap();
    assert_eq!(paint.background, Some(Color::from_rgb8(240, 243, 247)));
    assert_eq!(paint.text_color, Color::from_rgb8(28, 35, 48));
    assert_eq!(paint.border, Some((1.0, Color::from_rgb8(210, 216, 225))));
    source.set_pointer_pressed_visual(Some(tab));
    source.build_frame(&mut tcx, 600, 520);
    assert_eq!(
        source.tree.get_node_context(tab_node).unwrap().background,
        Some(Color::from_rgb8(225, 235, 249))
    );
    source.set_pointer_pressed_visual(None);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: tab }));
    source.build_frame(&mut tcx, 600, 520);
    assert_eq!(source.ids[&tab], tab_node);
    assert_eq!(
        source.tree.get_node_context(tab_node).unwrap().background,
        Some(Color::from_rgb8(31, 88, 155))
    );
    assert!(
        source
            .tree
            .get_node_context(tab_node)
            .unwrap()
            .border
            .is_none()
    );
    let back = source.actor.widget(root, key("seven.back.button")).unwrap();
    let forward = source
        .actor
        .widget(root, key("seven.forward.button"))
        .unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: back }));
    source.build_frame(&mut tcx, 600, 520);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("seven.page")),
        0
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("counter.count")),
        1
    );
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: back }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: forward }));
    source.build_frame(&mut tcx, 600, 520);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("seven.page")),
        1
    );
    let input = source
        .actor
        .widget(root, key("temperature.celsius.input"))
        .unwrap();
    let error_widget = source
        .actor
        .root_widget(key("temperature.error.text"))
        .unwrap();
    let error_node = source.ids[&error_widget];
    assert_eq!(source.tree.layout(error_node).unwrap().size.height, 0.0);
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: input }));
    let select_all = || {
        UiEvent::Key(shell::KeyEvent {
            phase: shell::KeyPhase::Down,
            key: "a".into(),
            key_without_modifiers: "a".into(),
            code: "KeyA".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                shell::Modifiers::META
            } else {
                shell::Modifiers::CONTROL
            },
            repeat: false,
            synthetic: false,
        })
    };
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("100".into()));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.celsius")),
        "100"
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.fahrenheit")),
        "212"
    );
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("-".into()));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 520);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.celsius")),
        "-"
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.fahrenheit")),
        "212"
    );
    assert!(
        !source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.error"))
            .is_empty()
    );
    let error_widget = source
        .actor
        .root_widget(key("temperature.error.text"))
        .unwrap();
    let error_node = source.ids[&error_widget];
    assert_eq!(
        source.tree.get_node_context(error_node).unwrap().text_color,
        Color::from_rgb8(166, 32, 40)
    );
    assert!(source.tree.layout(error_node).unwrap().size.height > 0.0);
    let state = source.actor.root_actor().state_address();
    let slot = source
        .actor
        .root_actor()
        .slot_address_by_key(seven_state("temperature.celsius"));
    let node = source.ids[&input];
    let selection = source.editors[&input].text_selection();
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 520);
    assert_eq!(source.ids[&tab], tab_node);
    assert_eq!(
        source.tree.get_node_context(tab_node).unwrap().background,
        Some(Color::from_rgb8(31, 88, 155))
    );
    assert_eq!(source.actor.root_actor().state_address(), state);
    assert_eq!(
        source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("temperature.celsius")),
        slot
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("counter.count")),
        1
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.celsius")),
        "-"
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.editors[&input].text_selection(), selection);
    source.handle_event(UiEvent::TextInput("40".into()));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.celsius")),
        "-40"
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.fahrenheit")),
        "-40"
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("temperature.error")),
        ""
    );
    source.build_frame(&mut tcx, 600, 520);
    assert_eq!(source.ids[&error_widget], error_node);
    assert_eq!(source.tree.layout(error_node).unwrap().size.height, 0.0);
    let fahrenheit = source
        .actor
        .widget(root, key("temperature.fahrenheit.input"))
        .unwrap();
    let responsive_editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let responsive_selection = source.editors[&input].text_selection();
    for width in [600, 320, 900] {
        source.build_frame(&mut tcx, width, 520);
        assert_eq!(source.ids[&input], node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            responsive_editor
        );
        assert_eq!(
            source.editors[&input].text_selection(),
            responsive_selection
        );
        let navigation = if width < 560 {
            "seven.task.select"
        } else {
            "seven.temperature.tab"
        };
        assert!(source.actor.widget(root, key(navigation)).is_some());
        for widget in [input, fahrenheit] {
            let rect = source.widget_layout_rect(widget).unwrap();
            assert!(
                rect[0] >= 0.0 && rect[1] >= 0.0 && rect[2] <= width as f32 && rect[3] <= 520.0,
                "{widget}: {rect:?} at {width}"
            );
        }
    }
    source.build_frame(&mut tcx, 600, 520);
    let flight_tab = source.actor.widget(root, key("seven.flight.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: flight_tab }));
    source.build_frame(&mut tcx, 600, 720);
    let departure = source
        .actor
        .widget(root, key("flight.departure.input"))
        .unwrap();
    let return_date = source
        .actor
        .widget(root, key("flight.return.input"))
        .unwrap();
    let trip = source.actor.widget(root, key("flight.trip.input")).unwrap();
    let book = source
        .actor
        .widget(root, key("flight.book.button"))
        .unwrap();
    assert_eq!(
        source
            .tree
            .get_node_context(source.ids[&return_date])
            .unwrap()
            .background,
        Some(Color::from_rgb8(246, 247, 249))
    );
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Focus {
            target: return_date
        })
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: book }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("flight.confirmation")),
        "One-way flight booked for 2026-09-01."
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: trip }));
    source.build_frame(&mut tcx, 600, 720);
    let trip_node = source.ids[&trip];
    assert!(
        source
            .placed
            .iter()
            .find(|node| node.node_id == trip_node)
            .unwrap()
            .paint
            .widget
            .is_some()
    );
    assert_eq!(
        source.tree.style(trip_node).unwrap().padding.right,
        LengthPercentage::length(28.0)
    );
    assert_eq!(
        source.tree.style(trip_node).unwrap().border,
        Rect::length(1.0)
    );
    let trip_options = source.selects[&trip].options().as_ptr();
    let trip_labels: Vec<_> = source.selects[&trip]
        .options()
        .iter()
        .map(|option| option.label.as_ptr())
        .collect();
    assert!(source.selects[&trip].is_open());
    let select_key = |name: &str| {
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
    };
    source.handle_event(select_key("End"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.selects[&trip].active(), Some(1));
    let before = source.semantic_snapshot().unwrap();
    let combo = before.nodes.iter().find(|node| node.id == trip).unwrap();
    assert_eq!(combo.role, shell::SemanticRole::ComboBox);
    let return_option = combo.active_descendant.unwrap();
    let list = combo.controls[0];
    let option = before
        .nodes
        .iter()
        .find(|node| node.id == return_option)
        .unwrap();
    assert_eq!(option.label.as_deref(), Some("Return flight"));
    assert_eq!(option.states.selected, Some(false));
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.ids[&trip], trip_node);
    assert_eq!(source.selects[&trip].options().as_ptr(), trip_options);
    for (option, label) in source.selects[&trip].options().iter().zip(&trip_labels) {
        assert_eq!(option.label.as_ptr(), *label);
    }
    assert!(source.selects[&trip].is_open());
    assert_eq!(source.selects[&trip].active(), Some(1));
    let after = source.semantic_snapshot().unwrap();
    let combo = after.nodes.iter().find(|node| node.id == trip).unwrap();
    assert_eq!(combo.active_descendant, Some(return_option));
    assert_eq!(combo.controls, [list]);
    assert_eq!(combo.states.expanded, Some(true));
    let option_ids: std::collections::HashSet<_> = after.nodes.iter().map(|node| node.id).collect();
    assert_eq!(option_ids.len(), after.nodes.len());
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: return_option
    }));
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Click {
            target: return_option
        })
    );
    source.build_frame(&mut tcx, 600, 720);
    assert!(!source.selects[&trip].is_open());
    assert_eq!(source.selects[&trip].options().as_ptr(), trip_options);
    for (option, label) in source.selects[&trip].options().iter().zip(&trip_labels) {
        assert_eq!(option.label.as_ptr(), *label);
    }
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("flight.trip_kind")),
        1
    );
    let closed = source.semantic_snapshot().unwrap();
    let combo = closed.nodes.iter().find(|node| node.id == trip).unwrap();
    assert_eq!(combo.value.as_deref(), Some("Return flight"));
    assert_eq!(combo.states.expanded, Some(false));
    assert!(!closed.nodes.iter().any(|node| node.id == return_option));
    println!(
        "PASS native Flight Select semantic identities and active descendant survive real DLL replacement; accessible commitment updates value and retires stale option actions"
    );
    println!(
        "PASS native Flight Select popup retains node/focus/highlight across real DLL replacement and commits current action"
    );
    println!(
        "PASS native Flight Select option and label buffers survive real DLL replacement and selection commitment"
    );
    assert_eq!(
        source
            .tree
            .get_node_context(source.ids[&return_date])
            .unwrap()
            .background,
        Some(Color::WHITE)
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: return_date
    }));
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("2026-08-31".into()));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: book }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("flight.confirmation")),
        ""
    );
    let node = source.ids[&return_date];
    let selection = source.editors[&return_date].text_selection();
    let slot = source
        .actor
        .root_actor()
        .slot_address_by_key(seven_state("flight.return"));
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.ids[&return_date], node);
    assert_eq!(source.focus, Some(return_date));
    assert_eq!(source.editors[&return_date].text_selection(), selection);
    assert_eq!(
        source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("flight.return")),
        slot
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("flight.return")),
        "2026-08-31"
    );
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("2026-09-08".into()));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: book }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("flight.confirmation")),
        "Return flight booked: 2026-09-01 to 2026-09-08."
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: return_date
    }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: trip }));
    source.handle_event(select_key("Home"));
    source.handle_event(select_key("Enter"));
    source.build_frame(&mut tcx, 600, 720);
    assert_ne!(source.focus, Some(return_date));
    assert!(source.ime_state().is_none());
    assert!(
        !source.handle_semantic_action(shell::SemanticAction::Focus {
            target: return_date
        })
    );
    for width in [600, 320] {
        source.build_frame(&mut tcx, width, 720);
        for widget in [departure, return_date, trip, book] {
            let rect = source.widget_layout_rect(widget).unwrap();
            assert!(
                rect[0] >= 0.0 && rect[1] >= 0.0 && rect[2] <= width as f32 && rect[3] <= 720.0
            );
        }
    }
    source.build_frame(&mut tcx, 600, 720);
    let crud_tab = source.actor.widget(root, key("seven.crud.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: crud_tab }));
    source.build_frame(&mut tcx, 600, 720);
    let person = |id: u64| identities::seven_person(id);
    let second = source.actor.widget(root, person(2)).unwrap();
    let third = source.actor.widget(root, person(3)).unwrap();
    let third_node = source.ids[&third];
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: second }));
    let first = source.actor.widget(root, key("crud.first.input")).unwrap();
    let last = source.actor.widget(root, key("crud.last.input")).unwrap();
    let filter = source.actor.widget(root, key("crud.filter.input")).unwrap();
    let update = source
        .actor
        .widget(root, key("crud.update.button"))
        .unwrap();
    let create = source
        .actor
        .widget(root, key("crud.create.button"))
        .unwrap();
    let delete = source
        .actor
        .widget(root, key("crud.delete.button"))
        .unwrap();
    let list = source.actor.widget(root, key("crud.list")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: first }));
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("Maximilian".into()));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("crud.first")),
        "Maximilian"
    );
    assert!(
        source.actor.root_actor().view.bindings[&person(2)]
            .text
            .ends_with(", Max")
    );
    for width in [320, 900, 600] {
        source.build_frame(&mut tcx, width, 720);
        for input in [first, last, filter] {
            assert_eq!(
                source.tree.style(source.ids[&input]).unwrap().border,
                Rect::length(1.0)
            );
            let rect = source.widget_layout_rect(input).unwrap();
            assert!(
                rect[0] >= 0.0 && rect[2] <= width as f32 && rect[3] <= 720.0,
                "CRUD field {input}: {rect:?} at {width}"
            );
        }
        let snapshot = source.semantic_snapshot().unwrap();
        for (input, name) in [
            (first, "First name"),
            (last, "Last name"),
            (filter, "Filter people · surname prefix"),
        ] {
            assert_eq!(
                snapshot
                    .nodes
                    .iter()
                    .find(|node| node.id == input)
                    .unwrap()
                    .label
                    .as_deref(),
                Some(name)
            );
        }
        let actor = source.actor.root_actor();
        assert_eq!(actor.view.bindings[&person(2)].node.button_variant, 0);
        assert_eq!(actor.view.bindings[&person(3)].node.button_variant, 1);
        assert_eq!(
            actor.view.bindings[&key("crud.update.button")]
                .node
                .button_variant,
            1
        );
        assert_eq!(
            actor.view.bindings[&key("crud.delete.button")]
                .node
                .button_variant,
            2
        );
        assert_eq!(
            source
                .tree
                .get_node_context(source.ids[&delete])
                .unwrap()
                .text_color,
            Color::from_rgb8(166, 32, 40)
        );
    }
    // A layout fixture supplies overflow width until the SDK grid is available.
    let content = source.tree.children(source.ids[&list]).unwrap()[0];
    let original_style = source.tree.style(content).unwrap().clone();
    let mut wide_style = original_style.clone();
    wide_style.size.width = Dimension::length(900.0);
    wide_style.size.height = Dimension::length(20.0);
    source.tree.set_style(content, wide_style).unwrap();
    source.build_frame(&mut tcx, 600, 720);
    let horizontal_bounds = source.widget_layout_rect(list).unwrap();
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: shell::Point {
            x: f64::from(horizontal_bounds[0] + 8.0),
            y: f64::from(horizontal_bounds[1] + 8.0),
        },
        delta_x: 32.0,
        delta_y: 0.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.actor.view().renderer[&list].scroll_x, 32.0);
    let placed = source
        .placed
        .iter()
        .find(|p| p.node_id == source.ids[&list])
        .unwrap();
    assert_eq!(placed.scroll.offset[0], 32.0);
    source.replace(Path::new(&paths[1]))?;
    assert_eq!(source.actor.view().renderer[&list].scroll_x, 32.0);
    source.tree.set_style(content, original_style).unwrap();
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.actor.view().renderer[&list].scroll_x, 0.0);
    let bounds = source.widget_layout_rect(list).unwrap();
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: shell::Point {
            x: f64::from(bounds[0] + 8.0),
            y: f64::from(bounds[1] + 8.0),
        },
        delta_x: 0.0,
        delta_y: 30.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 600, 720);
    let offset = source.actor.view().renderer[&list].scroll;
    assert!(offset > 0.0);
    let first_node = source.ids[&first];
    let selection = source.editors[&first].text_selection();
    let slot = source
        .actor
        .root_actor()
        .slot_address_by_key(seven_state("crud.ids"));
    let records_slot = source
        .actor
        .root_actor()
        .slot_address_by_key(seven_state("crud.records"));
    let records_buffer = source
        .actor
        .root_actor()
        .text_list_by_key(seven_state("crud.records"))
        .as_ptr();
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("crud.ids")),
        slot
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("crud.records")),
        records_slot
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_list_by_key(seven_state("crud.records"))
            .as_ptr(),
        records_buffer
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("crud.selected")),
        2
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("crud.first")),
        "Maximilian"
    );
    assert_eq!(source.ids[&first], first_node);
    assert_eq!(source.ids[&third], third_node);
    assert_eq!(source.focus, Some(first));
    assert_eq!(source.editors[&first].text_selection(), selection);
    assert_eq!(source.actor.view().renderer[&list].scroll, offset);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: update }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.actor.root_actor().view.bindings[&person(2)]
            .text
            .as_str(),
        "✓ Mustermann, Maximilian"
    );
    assert_eq!(source.ids[&third], third_node);
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: filter }));
    source.handle_event(UiEvent::TextInput("ZZ".into()));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: second }));
    assert!(
        source
            .actor
            .view()
            .bindings
            .values()
            .any(|binding| binding.text == "No matching people")
    );
    source.handle_event(select_all());
    source.handle_event(UiEvent::Key(shell::KeyEvent {
        phase: shell::KeyPhase::Down,
        key: "Backspace".into(),
        key_without_modifiers: "Backspace".into(),
        code: "Backspace".into(),
        text: None,
        text_with_all_modifiers: None,
        location: Default::default(),
        modifiers: shell::Modifiers::empty(),
        repeat: false,
        synthetic: false,
    }));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert!(
        source
            .actor
            .view()
            .bindings
            .values()
            .any(|binding| binding.text == "✓ Mustermann, Maximilian")
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: first }));
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("Ada".into()));
    source.prepare_for_event(&mut tcx);
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: last }));
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("Lovelace".into()));
    source.prepare_for_event(&mut tcx);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: create }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("crud.ids")),
        &[1, 2, 3, 4]
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("crud.first")),
        ""
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("crud.selected")),
        0
    );
    let added = source.actor.widget(root, person(4)).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: added }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: delete }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("crud.ids")),
        &[1, 2, 3]
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("crud.next")),
        5
    );
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: added }));
    for width in [600, 320] {
        source.build_frame(&mut tcx, width, 720);
        for widget in [filter, list, first, last, create, update, delete] {
            let rect = source.widget_layout_rect(widget).unwrap();
            assert!(
                rect[0] >= 0.0 && rect[1] >= 0.0 && rect[2] <= width as f32 && rect[3] <= 720.0,
                "{widget}: {rect:?}"
            );
        }
    }
    source.build_frame(&mut tcx, 600, 720);
    let tab = source.actor.widget(root, key("seven.timer.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: tab }));
    source.build_frame(&mut tcx, 600, 720);
    let slider = source
        .actor
        .widget(root, key("timer.duration.slider"))
        .unwrap();
    let field = source.actor.view().bindings[&slider].node.parent;
    let caption = *source
        .actor
        .view()
        .bindings
        .iter()
        .find(|(_, binding)| {
            binding.node.parent == field && binding.node.kind == blockui_abi::LABEL
        })
        .unwrap()
        .0;
    let caption_node = source.ids[&caption];
    for width in [320, 600] {
        source.build_frame(&mut tcx, width, 720);
        let caption_layout = source.tree.layout(caption_node).unwrap();
        let slider_layout = source.tree.layout(source.ids[&slider]).unwrap();
        assert!(
            (slider_layout.location.y
                - caption_layout.location.y
                - caption_layout.size.height
                - 6.0)
                .abs()
                < 0.1
        );
    }
    let progress = source.actor.widget(root, key("timer.progress")).unwrap();
    let progress_node = source.ids[&progress];
    let native = (&**source.sliders.get(&slider).unwrap()
        as *const crate::native_slider::NativeSlider) as usize;
    let node_id = source.ids[&slider];
    let placed = source
        .placed
        .iter()
        .find(|node| node.node_id == node_id)
        .unwrap();
    let point = shell::Point {
        x: f64::from(placed.content_origin[0] + placed.content_size[0] * 0.5),
        y: f64::from(placed.content_origin[1] + placed.content_size[1] * 0.5),
    };
    let pointer = |phase, position| {
        UiEvent::Pointer(shell::PointerEvent {
            phase,
            position,
            button: Some(shell::PointerButton::Primary),
            buttons: 1,
            modifiers: shell::Modifiers::empty(),
            properties: Default::default(),
        })
    };
    source.handle_event(pointer(PointerPhase::Down, point));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.duration")),
        30000
    );
    assert_eq!(source.focus, Some(slider));
    let baseline = source
        .actor
        .root_actor()
        .integer_by_key(seven_state("timer.baseline"));
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.baseline")),
        baseline
    );
    assert_eq!(source.ids[&slider], node_id);
    assert_eq!(source.ids[&caption], caption_node);
    assert_eq!(
        source.actor.view().bindings[&caption].text,
        "Duration: 30.0 s"
    );
    let semantics = source.semantic_snapshot().unwrap();
    assert_eq!(
        semantics
            .nodes
            .iter()
            .find(|item| item.id == slider)
            .unwrap()
            .label
            .as_deref(),
        Some("Duration: 30.0 s")
    );

    assert_eq!(
        (&**source.sliders.get(&slider).unwrap() as *const crate::native_slider::NativeSlider)
            as usize,
        native
    );
    assert_eq!(source.focus, Some(slider));
    assert_eq!(source.ids[&progress], progress_node);
    let placed = source
        .placed
        .iter()
        .find(|node| node.node_id == node_id)
        .unwrap();
    let end = shell::Point {
        x: f64::from(placed.content_origin[0] + placed.content_size[0]),
        y: point.y,
    };
    source.handle_event(pointer(PointerPhase::Move, end));
    source.handle_event(pointer(PointerPhase::Up, end));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.duration")),
        60000
    );
    source.handle_event(pointer(PointerPhase::Down, point));
    source.prepare_for_event(&mut tcx);
    let before_blur = source
        .actor
        .root_actor()
        .integer_by_key(seven_state("timer.duration"));
    source.handle_event(UiEvent::Focus(false));
    source.handle_event(UiEvent::Focus(true));
    source.handle_event(pointer(PointerPhase::Move, end));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.duration")),
        before_blur
    );
    source.build_frame(&mut tcx, 600, 720);
    let semantic = source
        .semantics
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.id == slider)
        .unwrap();
    assert_eq!(semantic.role, shell::SemanticRole::Slider);
    assert_eq!(semantic.label.as_deref(), Some("Duration: 30.0 s"));
    assert_eq!(semantic.numeric_value, Some(before_blur as f64));
    assert_eq!(semantic.min_numeric_value, Some(0.0));
    assert_eq!(semantic.max_numeric_value, Some(60000.0));
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
    source.handle_event(key_event("Home"));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.duration")),
        0
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.elapsed")),
        0
    );
    source.handle_event(key_event("ArrowRight"));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.duration")),
        100
    );
    std::thread::sleep(std::time::Duration::from_millis(120));
    source.actor.tick()?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.elapsed")),
        100
    );
    let binding = &source.actor.view().bindings[&progress];
    assert_eq!(binding.text, "1");
    assert_eq!(binding.semantic_label.as_deref(), Some("Elapsed progress"));
    let semantic = source
        .semantics
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.id == progress)
        .unwrap();
    assert_eq!(semantic.role, shell::SemanticRole::ProgressBar);
    assert_eq!(semantic.numeric_value, Some(1.0));
    assert!(
        source
            .placed
            .iter()
            .find(|node| node.node_id == progress_node)
            .unwrap()
            .paint
            .widget
            .is_some()
    );
    assert!(!source.handle_semantic_action(shell::SemanticAction::Focus { target: progress }));
    let reset = source
        .actor
        .widget(root, key("timer.reset.button"))
        .unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: reset }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("timer.elapsed")),
        0
    );
    for width in [600, 320] {
        source.build_frame(&mut tcx, width, 720);
        for widget in [slider, reset, progress] {
            let rect = source.widget_layout_rect(widget).unwrap();
            assert!(rect[0] >= 0.0 && rect[2] <= width as f32 && rect[3] <= 720.0);
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(120));
    source.actor.tick()?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.actor.view().bindings[&progress].text, "1");
    let tab = source.actor.widget(root, key("seven.circle.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: tab }));
    source.build_frame(&mut tcx, 600, 720);
    let canvas = source.actor.widget(root, key("circle.canvas")).unwrap();
    assert_eq!(
        source.tree.style(source.ids[&canvas]).unwrap().border,
        Rect::length(1.0)
    );
    let canvas_node = source.ids[&canvas];
    let local_pointer =
        |source: &NativeSource<crate::native_model::NativeFamily>, point: LocalPoint, button| {
            let placed = source
                .placed
                .iter()
                .find(|node| node.node_id == source.ids[&canvas])
                .unwrap();
            UiEvent::Pointer(shell::PointerEvent {
                phase: PointerPhase::Down,
                position: shell::Point {
                    x: f64::from(placed.content_origin[0] + point.x),
                    y: f64::from(placed.content_origin[1] + point.y),
                },
                button: Some(button),
                buttons: 1,
                modifiers: shell::Modifiers::empty(),
                properties: Default::default(),
            })
        };
    let center = LocalPoint { x: 60.5, y: 70.25 };
    source.handle_event(local_pointer(
        &source,
        center,
        shell::PointerButton::Primary,
    ));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[5],
        1
    );
    let scene =
        blockui_abi::canvas::CanvasScene::from_wire(&source.actor.view().bindings[&canvas].text)
            .unwrap();
    assert_eq!(scene.circles[0].center, center);
    assert_eq!(scene.circles[0].radius, 26.0);
    assert!(
        source
            .placed
            .iter()
            .find(|node| node.node_id == canvas_node)
            .unwrap()
            .paint
            .widget
            .is_some()
    );
    source.handle_event(local_pointer(
        &source,
        center,
        shell::PointerButton::Primary,
    ));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[5],
        1
    );
    source.handle_event(local_pointer(
        &source,
        center,
        shell::PointerButton::Secondary,
    ));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    let diameter = source
        .actor
        .widget(root, key("circle.diameter.slider"))
        .unwrap();
    assert_eq!(source.focus, Some(diameter));
    assert!(source.modal.is_some());
    let background = source.actor.widget(root, key("seven.counter.tab")).unwrap();
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: background }));
    assert!(!source.handle_semantic_action(shell::SemanticAction::Focus { target: background }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("seven.page")),
        5
    );
    let focus = source.focus;
    source.handle_event(pointer(PointerPhase::Down, shell::Point { x: 1.0, y: 1.0 }));
    assert_eq!(source.focus, focus);
    for _ in 0..6 {
        source.handle_event(key_event("Tab"));
        assert!(source.in_modal(source.focus.unwrap()));
    }
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: diameter }));
    let native_diameter = (&**source.sliders.get(&diameter).unwrap()
        as *const crate::native_slider::NativeSlider) as usize;
    let placed = source
        .placed
        .iter()
        .find(|node| node.node_id == source.ids[&diameter])
        .unwrap();
    let middle = shell::Point {
        x: f64::from(placed.content_origin[0] + placed.content_size[0] * 0.5),
        y: f64::from(placed.content_origin[1] + placed.content_size[1] * 0.5),
    };
    source.handle_event(pointer(PointerPhase::Down, middle));
    source.handle_event(pointer(
        PointerPhase::Move,
        shell::Point { x: -100.0, y: 1.0 },
    ));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("circle.diameter")),
        20
    );
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    source.handle_event(pointer(
        PointerPhase::Move,
        shell::Point { x: 1000.0, y: 1.0 },
    ));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("circle.diameter")),
        160
    );
    source.handle_event(pointer(
        PointerPhase::Up,
        shell::Point { x: 1000.0, y: 1.0 },
    ));
    source.handle_event(pointer(
        PointerPhase::Move,
        shell::Point { x: -100.0, y: 1.0 },
    ));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("circle.diameter")),
        160
    );
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 600, 720);
    source.handle_event(pointer(PointerPhase::Down, middle));
    source.handle_event(pointer(PointerPhase::Up, middle));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("circle.diameter")),
        90
    );
    let state = source
        .actor
        .root_actor()
        .list_by_key(seven_state("circle.model"))
        .to_vec();
    let slot = source
        .actor
        .root_actor()
        .slot_address_by_key(seven_state("circle.model"));
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model")),
        state
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("circle.model")),
        slot
    );
    assert_eq!(source.ids[&canvas], canvas_node);
    assert_eq!(
        (&**source.sliders.get(&diameter).unwrap() as *const crate::native_slider::NativeSlider)
            as usize,
        native_diameter
    );
    assert_eq!(source.focus, Some(diameter));
    source.handle_event(key_event("ArrowRight"));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("circle.diameter")),
        91
    );
    let apply = source
        .actor
        .widget(root, key("circle.apply.button"))
        .unwrap();
    let apply_bounds = source.widget_layout_rect(apply).unwrap();
    let apply_point = shell::Point {
        x: f64::from((apply_bounds[0] + apply_bounds[2]) * 0.5),
        y: f64::from((apply_bounds[1] + apply_bounds[3]) * 0.5),
    };
    let before_cancel = source
        .actor
        .root_actor()
        .list_by_key(seven_state("circle.model"))
        .to_vec();
    for phase in [PointerPhase::Cancel, PointerPhase::Up] {
        source.handle_event(pointer(PointerPhase::Down, apply_point));
        assert_eq!(source.pressed, Some(apply));
        source.handle_event(pointer(phase, shell::Point { x: 1.0, y: 1.0 }));
        assert_eq!(source.pressed, None);
        source.handle_event(pointer(PointerPhase::Up, apply_point));
        source.build_frame(&mut tcx, 600, 720);
        assert!(source.modal.is_some());
        assert_eq!(
            source
                .actor
                .root_actor()
                .list_by_key(seven_state("circle.model")),
            before_cancel
        );
    }
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: apply }));
    source.build_frame(&mut tcx, 600, 720);
    assert!(!source.sliders.contains_key(&diameter));
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[11],
        91
    );
    let undo = source
        .actor
        .widget(root, key("circle.undo.button"))
        .unwrap();
    let redo = source
        .actor
        .widget(root, key("circle.redo.button"))
        .unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: undo }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[11],
        52
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: undo }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[5],
        0
    );
    source.replace(Path::new(&paths[0]))?;
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: redo }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: redo }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[11],
        91
    );
    source.handle_event(local_pointer(
        &source,
        center,
        shell::PointerButton::Secondary,
    ));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    let diameter = source
        .actor
        .widget(root, key("circle.diameter.slider"))
        .unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: diameter }));
    source.handle_event(key_event("Home"));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[11],
        20
    );
    let cancel = source
        .actor
        .widget(root, key("circle.cancel.button"))
        .unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: cancel }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[11],
        91
    );
    source.handle_event(local_pointer(
        &source,
        center,
        shell::PointerButton::Secondary,
    ));
    source.prepare_for_event(&mut tcx);
    for width in [600, 320] {
        source.build_frame(&mut tcx, width, 720);
        for local in [
            "circle.canvas",
            "circle.diameter.slider",
            "circle.apply.button",
            "circle.cancel.button",
        ] {
            let widget = source.actor.widget(root, key(local)).unwrap();
            let rect = source.widget_layout_rect(widget).unwrap();
            assert!(
                rect[0] >= 0.0 && rect[2] <= width as f32 && rect[3] <= 720.0,
                "{local}: {rect:?}"
            );
        }
    }
    source.build_frame(&mut tcx, 600, 720);
    let apply = source
        .actor
        .widget(root, key("circle.apply.button"))
        .unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: apply }));
    source.build_frame(&mut tcx, 600, 720);
    let edge = LocalPoint { x: 1.0, y: 1.0 };
    source.handle_event(local_pointer(&source, edge, shell::PointerButton::Primary));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .list_by_key(seven_state("circle.model"))[5],
        2
    );
    let placed = source
        .placed
        .iter()
        .find(|node| node.node_id == canvas_node)
        .unwrap();
    let clip = placed.own_clip.expect("canvas clips its native paint");
    assert!(
        clip[0] >= placed.rect[0]
            && clip[1] >= placed.rect[1]
            && clip[2] <= placed.rect[2]
            && clip[3] <= placed.rect[3]
    );
    source.handle_event(local_pointer(
        &source,
        edge,
        shell::PointerButton::Secondary,
    ));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    source.handle_event(key_event("Escape"));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert!(source.modal.is_none());
    let adjust = source
        .actor
        .widget(root, key("circle.adjust.button"))
        .unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: adjust }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: adjust }));
    source.build_frame(&mut tcx, 600, 720);
    let modal = source.modal.unwrap();
    let semantic = source
        .semantics
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|node| node.id == modal)
        .unwrap();
    assert_eq!(semantic.role, shell::SemanticRole::Dialog);
    assert_eq!(semantic.states.modal, Some(true));
    source.handle_event(key_event("Escape"));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.focus, Some(adjust));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: adjust }));
    source.build_frame(&mut tcx, 600, 720);
    for (width, height) in [(600, 720), (320, 480)] {
        source.build_frame(&mut tcx, width, height);
        let modal = source.modal.unwrap();
        let rect = source.widget_layout_rect(modal).unwrap();
        assert!(
            rect[0] >= 0.0 && rect[1] >= 0.0 && rect[2] <= width as f32 && rect[3] <= height as f32,
            "dialog: {rect:?}"
        );
        assert!(((rect[1] + rect[3]) * 0.5 - height as f32 * 0.5).abs() < 1.0);
    }
    source.handle_event(key_event("Escape"));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    let cells_tab = source.actor.widget(root, key("seven.cells.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: cells_tab }));
    source.build_frame(&mut tcx, 600, 720);
    let grid = source.actor.widget(root, key("cells.grid")).unwrap();
    let viewport_events = source
        .actor
        .root_actor()
        .integer_by_key(seven_state("cells.viewport.events"));
    assert!(viewport_events > 0);
    let port = source
        .placed
        .iter()
        .find(|node| node.node_id == source.ids[&grid])
        .unwrap()
        .scroll
        .port;
    assert_eq!(
        source
            .actor
            .root_actor()
            .float_by_key(seven_state("cells.viewport.width")),
        f64::from(port[2] - port[0])
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .float_by_key(seven_state("cells.viewport.height")),
        f64::from(port[3] - port[1])
    );
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.viewport.events")),
        viewport_events
    );
    source.build_frame(&mut tcx, 320, 720);
    assert!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.viewport.events"))
            > viewport_events
    );
    let narrow_width = source
        .actor
        .root_actor()
        .float_by_key(seven_state("cells.viewport.width"));
    assert!(narrow_width < f64::from(port[2] - port[0]));
    source.build_frame(&mut tcx, 600, 720);
    assert!(
        source.actor.view().bindings.len() < 400,
        "Cells must project a bounded viewport"
    );
    let sheet_range = source
        .tree
        .layout(source.ids[&grid])
        .unwrap()
        .scroll_height();
    assert!(
        (4190.0..=4210.0).contains(&sheet_range),
        "full sheet range: {sheet_range}"
    );
    let editor = source.actor.widget(root, key("cells.editor")).unwrap();
    let apply = source.actor.widget(root, key("cells.apply")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: editor }));
    source.handle_event(UiEvent::TextInput("2".into()));
    source.prepare_for_event(&mut tcx);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: apply }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_list_by_key(seven_state("cells.sources"))[0],
        "2"
    );
    let b1 = *source
        .actor
        .view()
        .bindings
        .iter()
        .find(|(_, binding)| {
            binding.node.action == key("cells.choose") && binding.node.action_payload == 1
        })
        .unwrap()
        .0;
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: b1 }));
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        1
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: editor }));
    source.handle_event(UiEvent::TextInput("=A1*3".into()));
    source.prepare_for_event(&mut tcx);
    source.handle_event(key_event("Enter"));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert!(source.actor.view().bindings[&b1].text.contains("6"));
    let semantics = source.semantic_snapshot().unwrap();
    let table = semantics.nodes.iter().find(|node| node.id == grid).unwrap();
    let cell = semantics.nodes.iter().find(|node| node.id == b1).unwrap();
    let row_key = source.actor.view().bindings[&b1].node.parent;
    let row = semantics
        .nodes
        .iter()
        .find(|node| node.id == row_key)
        .unwrap();
    assert_eq!(table.role, shell::SemanticRole::Grid);
    assert_eq!(row.role, shell::SemanticRole::Row);
    assert_eq!(cell.role, shell::SemanticRole::GridCell);
    assert!(table.children.contains(&row_key));
    assert!(row.children.contains(&b1));
    assert_eq!(cell.label.as_deref(), Some("B1 6"));
    assert!(!cell.disabled);

    let cell_paint = source.tree.get_node_context(source.ids[&b1]).unwrap();
    assert_eq!(cell_paint.background, Some(Color::WHITE));
    assert_eq!(cell_paint.text_color, Color::from_rgb8(28, 35, 48));
    assert_eq!(
        cell_paint.border,
        Some((1.0, Color::from_rgb8(210, 216, 225)))
    );
    assert_eq!(cell_paint.border_radii, CornerRadii::uniform(0.0));
    let placed_cell = source
        .placed
        .iter()
        .find(|node| node.node_id == source.ids[&b1])
        .unwrap();
    assert_eq!(placed_cell.border_widths, [1.0; 4]);
    assert_eq!(
        source
            .tree
            .get_node_context(source.ids[&apply])
            .unwrap()
            .background,
        Some(Color::from_rgb8(27, 78, 140))
    );

    // Changing only the ancestor's grid semantics must refresh unchanged
    // buttons without recreating their Taffy identities.
    let cell_node = source.ids[&b1];
    let grid_action = source.actor.view().bindings[&grid].node.navigate_action;
    source
        .actor
        .view_mut()
        .bindings
        .get_mut(&grid)
        .unwrap()
        .node
        .navigate_action = 0;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.ids[&b1], cell_node);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::from_rgb8(31, 88, 155))
    );
    source
        .actor
        .view_mut()
        .bindings
        .get_mut(&grid)
        .unwrap()
        .node
        .navigate_action = grid_action;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.ids[&b1], cell_node);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::WHITE)
    );
    let cell_bounds = source.widget_layout_rect(b1).unwrap();
    let cell_point = shell::Point {
        x: f64::from((cell_bounds[0] + cell_bounds[2]) * 0.5),
        y: f64::from((cell_bounds[1] + cell_bounds[3]) * 0.5),
    };
    source.handle_event(pointer(PointerPhase::Down, cell_point));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::from_rgb8(225, 235, 249))
    );
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.ids[&b1], cell_node);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::from_rgb8(225, 235, 249))
    );
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 600, 720);
    source.handle_event(pointer(PointerPhase::Move, shell::Point { x: 1.0, y: 1.0 }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::WHITE)
    );
    source.handle_event(pointer(PointerPhase::Move, cell_point));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::from_rgb8(225, 235, 249))
    );
    source.handle_event(pointer(PointerPhase::Up, cell_point));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::from_rgb8(241, 245, 251))
    );
    source.handle_event(pointer(PointerPhase::Down, cell_point));
    source.handle_event(pointer(PointerPhase::Cancel, cell_point));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::WHITE)
    );
    source.handle_event(pointer(PointerPhase::Down, cell_point));
    source.handle_event(UiEvent::Focus(false));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.tree.get_node_context(cell_node).unwrap().background,
        Some(Color::WHITE)
    );
    source.handle_event(UiEvent::Focus(true));
    let grid_bounds = source.widget_layout_rect(grid).unwrap();
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 4200.0 })?;
    assert!(
        source
            .actor
            .view()
            .order
            .iter()
            .any(|key| !source.ids.contains_key(key))
    );
    let interim_hit = source.hit(shell::Point {
        x: f64::from(grid_bounds[0] + 8.0),
        y: f64::from(grid_bounds[1] + 8.0),
    });
    assert!(interim_hit.is_none_or(|key| source.ids.contains_key(&key)));
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 0.0 })?;
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: shell::Point {
            x: f64::from(grid_bounds[0] + 8.0),
            y: f64::from(grid_bounds[1] + 8.0),
        },
        delta_x: 120.0,
        delta_y: 44.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.actor.view().renderer[&grid].scroll_x, 120.0);
    assert_eq!(source.actor.view().renderer[&grid].scroll, 44.0);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.viewport.row")),
        1
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: editor }));
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("=(unfinished".into()));
    source.prepare_for_event(&mut tcx);
    let editor_node = source.ids[&editor];
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.ids[&editor], editor_node);
    assert_eq!(source.focus, Some(editor));
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("cells.draft")),
        "=(unfinished"
    );
    assert_eq!(source.actor.view().renderer[&grid].scroll_x, 120.0);
    assert_eq!(source.actor.view().renderer[&grid].scroll, 44.0);
    source.handle_event(key_event("Escape"));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("cells.draft")),
        "=A1*3"
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_list_by_key(seven_state("cells.sources"))[1],
        "=A1*3"
    );
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("=A1*4".into()));
    source.prepare_for_event(&mut tcx);
    source.handle_event(key_event("Enter"));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_list_by_key(seven_state("cells.sources"))[1],
        "=A1*4"
    );
    assert!(source.actor.view().bindings[&b1].text.contains("8"));
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("=A1*5".into()));
    source.prepare_for_event(&mut tcx);
    let mut repeated = key_event("Enter");
    if let UiEvent::Key(event) = &mut repeated {
        event.repeat = true;
    }
    source.handle_event(repeated);
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_list_by_key(seven_state("cells.sources"))[1],
        "=A1*4"
    );
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Preedit {
        text: "拼".into(),
        cursor: None,
    }));
    source.prepare_for_event(&mut tcx);
    source.handle_event(key_event("Enter"));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_list_by_key(seven_state("cells.sources"))[1],
        "=A1*4"
    );
    source.handle_event(UiEvent::Ime(shell::ImeEvent::Disabled));
    source.prepare_for_event(&mut tcx);
    source.handle_event(key_event("Escape"));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("cells.draft")),
        "=A1*4"
    );
    source.build_frame(&mut tcx, 320, 720);
    for target in [editor, apply, grid] {
        let rect = source.widget_layout_rect(target).unwrap();
        assert!(
            rect[0] >= 0.0 && rect[2] <= 320.0 && rect[3] <= 720.0,
            "Cells: {rect:?}"
        );
    }
    // A colliding index in a nested grid must not steal the outer grid's target.
    let nested_cell = source.grid_target(grid, 26).unwrap();
    let nested_row = source.actor.view().bindings[&nested_cell].node.parent;
    let original_row = source.actor.view().bindings[&nested_row].node;
    let original_cell = source.actor.view().bindings[&nested_cell].node;
    let original_order = source.actor.view().order.clone();
    {
        let view = source.actor.view_mut();
        let row = &mut view.bindings.get_mut(&nested_row).unwrap().node;
        row.kind = SCROLL;
        row.navigate_action = original_row.key;
        view.bindings
            .get_mut(&nested_cell)
            .unwrap()
            .node
            .action_payload = 1;
        view.order.retain(|key| *key != nested_cell);
        view.order.insert(0, nested_cell);
    }
    assert_eq!(source.grid_owner(nested_cell), Some(nested_row));
    assert_eq!(source.grid_target(nested_row, 1), Some(nested_cell));
    assert_eq!(source.grid_target(grid, 1), Some(b1));
    {
        let view = source.actor.view_mut();
        view.bindings.get_mut(&nested_row).unwrap().node = original_row;
        view.bindings.get_mut(&nested_cell).unwrap().node = original_cell;
        view.order = original_order;
    }
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: b1 }));
    source.handle_event(key_event("ArrowRight"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        2
    );
    assert_eq!(
        source.actor.view().bindings[&source.focus.unwrap()]
            .node
            .action_payload,
        2
    );
    source.handle_event(key_event("End"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        25
    );
    assert!(source.actor.view().renderer[&grid].scroll_x > 2400.0);
    let focus_before = source.focus;
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.focus, focus_before);
    source.handle_event(key_event("ArrowDown"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        51
    );
    source.handle_event(key_event("Home"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        26
    );
    assert_eq!(source.actor.view().renderer[&grid].scroll_x, 0.0);
    source.handle_event(key_event("PageDown"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        130
    );
    source.handle_event(key_event("PageUp"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        26
    );
    let primary_key = |name: &str| {
        let mut event = key_event(name);
        if let UiEvent::Key(key) = &mut event {
            key.modifiers = if cfg!(target_os = "macos") {
                shell::Modifiers::META
            } else {
                shell::Modifiers::CONTROL
            };
        }
        event
    };
    source.handle_event(primary_key("End"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        2599
    );
    assert!(
        source.actor.view().bindings.len() < 400,
        "last-row projection must remain bounded"
    );
    assert_eq!(source.grid_target(grid, 2599), source.focus);
    assert!(source.actor.view().renderer[&grid].scroll > 4000.0);
    assert!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.viewport.row"))
            >= 95
    );
    let last_focus = source.focus;
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.focus, last_focus);
    source.handle_event(key_event("ArrowDown"));
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        2599
    );
    // A declared letter shortcut must not steal a named navigation key.
    source
        .actor
        .view_mut()
        .bindings
        .get_mut(&apply)
        .unwrap()
        .node
        .shortcut = u32::from(b'h');
    source.handle_event(primary_key("Home"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        0
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.viewport.row")),
        0
    );
    assert_eq!(source.actor.view().renderer[&grid].scroll, 0.0);
    assert_eq!(source.actor.view().renderer[&grid].scroll_x, 0.0);
    // Layout normalization of an excessive retained offset must update the
    // projection even without a wheel event or a keyboard reveal.
    source
        .actor
        .view_mut()
        .renderer
        .get_mut(&grid)
        .unwrap()
        .scroll = 10000.0;
    source.build_frame(&mut tcx, 600, 720);
    let normalized = source.actor.view().renderer[&grid].scroll;
    assert!((4190.0..=4210.0).contains(&normalized));
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.viewport.row")),
        (normalized / 44.0).floor() as i64
    );
    assert!(source.actor.view().bindings.len() < 400);
    assert!(
        source.grid_target(grid, 26 * 99).is_some(),
        "clamped viewport must materialize the last row"
    );
    source
        .actor
        .view_mut()
        .renderer
        .get_mut(&grid)
        .unwrap()
        .scroll = 0.0;
    source.notify_scroll(grid);
    source.build_frame(&mut tcx, 600, 720);
    let original_focus = source.focus.unwrap();
    let offscreen_focus = source.grid_target(grid, 26).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: offscreen_focus
    }));
    let draft_before_scroll = source
        .actor
        .root_actor()
        .text_by_key(seven_state("cells.draft"))
        .to_owned();
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 4200.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.focus, None);
    assert_eq!(source.suspended_grid_focus, Some((grid, 26)));
    source.replace(Path::new(&paths[1]))?;
    assert_eq!(source.suspended_grid_focus, Some((grid, 26)));
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 0.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.focus, source.grid_target(grid, 26));
    assert_eq!(source.suspended_grid_focus, None);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        0
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("cells.draft")),
        draft_before_scroll
    );
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 4200.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.suspended_grid_focus, Some((grid, 26)));
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: editor }));
    assert_eq!(source.suspended_grid_focus, None);
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 0.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source.focus,
        Some(editor),
        "returning row must not steal explicit focus"
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: original_focus
    }));
    for reverse in [false, true] {
        source
            .actor
            .observe_scroll(grid, LocalPoint { x: 0.0, y: 0.0 })?;
        source.build_frame(&mut tcx, 600, 720);
        let row = source.grid_target(grid, 26).unwrap();
        assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: row }));
        source
            .actor
            .observe_scroll(grid, LocalPoint { x: 0.0, y: 4200.0 })?;
        source.build_frame(&mut tcx, 600, 720);
        assert_eq!(source.suspended_grid_focus, Some((grid, 26)));
        source.replace(Path::new(&paths[1]))?;
        let mut tab = key_event("Tab");
        if reverse && let UiEvent::Key(event) = &mut tab {
            event.modifiers = shell::Modifiers::SHIFT;
        }
        source.handle_event(tab);
        source.build_frame(&mut tcx, 600, 720);
        let expected = source
            .actor
            .widget(
                root,
                key(if reverse {
                    "cells.cancel"
                } else {
                    "seven.counter.tab"
                }),
            )
            .unwrap();
        assert_eq!(source.focus, Some(expected));
        assert_eq!(source.suspended_grid_focus, None);
        assert_eq!(
            source
                .actor
                .root_actor()
                .integer_by_key(seven_state("cells.active")),
            0
        );
        assert_eq!(
            source
                .actor
                .root_actor()
                .text_by_key(seven_state("cells.draft")),
            draft_before_scroll
        );
        // Return from either neighbor without selecting a different edit cell.
        let mut returning = key_event("Tab");
        if !reverse && let UiEvent::Key(event) = &mut returning {
            event.modifiers = shell::Modifiers::SHIFT;
        }
        source.replace(Path::new(&paths[0]))?;
        source.handle_event(returning);
        source.build_frame(&mut tcx, 600, 720);
        assert_eq!(source.focus, source.grid_target(grid, 26));
        assert!(source.actor.view().renderer[&grid].scroll < 100.0);
        assert_eq!(
            source
                .actor
                .root_actor()
                .integer_by_key(seven_state("cells.active")),
            0
        );
        assert_eq!(
            source
                .actor
                .root_actor()
                .text_by_key(seven_state("cells.draft")),
            draft_before_scroll
        );
    }
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 0.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    let hidden_origin = source.grid_target(grid, 26).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: hidden_origin
    }));
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 4200.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    source.replace(Path::new(&paths[0]))?;
    assert_eq!(source.suspended_grid_focus, Some((grid, 26)));
    source.handle_event(key_event("ArrowRight"));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        27
    );
    assert_eq!(source.focus, source.grid_target(grid, 27));
    assert_eq!(source.suspended_grid_focus, None);
    assert!(source.actor.view().renderer[&grid].scroll < 100.0);
    source.handle_event(primary_key("Home"));
    source.build_frame(&mut tcx, 600, 720);
    let first_cell = source.focus.unwrap();
    source.handle_event(key_event("Tab"));
    source.build_frame(&mut tcx, 600, 720);
    assert_ne!(source.grid_owner(source.focus.unwrap()), Some(grid));
    let mut reverse_tab = key_event("Tab");
    if let UiEvent::Key(key) = &mut reverse_tab {
        key.modifiers = shell::Modifiers::SHIFT;
    }
    source.handle_event(reverse_tab.clone());
    assert_eq!(source.focus, Some(first_cell));
    source.handle_event(key_event("ArrowRight"));
    source.build_frame(&mut tcx, 600, 720);
    let retained_cell = source.focus;
    source.handle_event(key_event("Tab"));
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    source.handle_event(reverse_tab);
    assert_eq!(source.focus, retained_cell);
    assert_eq!(source.actor.view().renderer[&grid].grid_focus, Some(1));
    source.build_frame(&mut tcx, 600, 720);
    source
        .actor
        .root_actor_mut()
        .view
        .set_renderer_retention(512)?;
    let hidden_marker = source.grid_target(grid, 26).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: hidden_marker
    }));
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 4200.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    assert!(
        source
            .actor
            .root_actor()
            .view
            .retained_renderer
            .iter()
            .any(|(_, _, branch, state)| *branch == key("cells.viewport") && state.focus)
    );
    let current_grid_cell = source.grid_target(grid, 0).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: current_grid_cell
    }));
    let retained_editor = source.actor.widget(root, key("cells.editor")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: retained_editor
    }));
    assert!(
        source
            .actor
            .root_actor()
            .view
            .retained_renderer
            .iter()
            .all(|(_, _, branch, state)| *branch != key("cells.viewport") || !state.focus)
    );
    source
        .actor
        .observe_scroll(grid, LocalPoint { x: 0.0, y: 0.0 })?;
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.focus, Some(retained_editor));
    source.handle_event(select_all());
    source.handle_event(UiEvent::TextInput("🦀abc".into()));
    source.prepare_for_event(&mut tcx);
    source.build_frame(&mut tcx, 600, 720);
    source
        .editors
        .get_mut(&retained_editor)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":4,"head":2}}"#)?;
    source.prepare_for_event(&mut tcx);
    let retained_selection = source.editors[&retained_editor].text_selection();
    let retained_grid = source.actor.widget(root, key("cells.grid")).unwrap();
    let rect = source.widget_layout_rect(retained_grid).unwrap();
    source.handle_event(UiEvent::Wheel(shell::WheelEvent {
        position: shell::Point {
            x: f64::from(rect[0] + 8.0),
            y: f64::from(rect[1] + 8.0),
        },
        delta_x: 120.0,
        delta_y: 44.0,
        delta_mode: shell::WheelDeltaMode::Pixel,
        phase: shell::GesturePhase::Changed,
        modifiers: Default::default(),
    }));
    source.build_frame(&mut tcx, 600, 720);
    let retained_offsets = (
        source.actor.view().renderer[&retained_grid].scroll_x,
        source.actor.view().renderer[&retained_grid].scroll,
    );
    assert!(retained_offsets.0 > 0.0 && retained_offsets.1 > 0.0);
    let counter_tab = source.actor.widget(root, key("seven.counter.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: counter_tab
    }));
    source.build_frame(&mut tcx, 600, 720);
    assert!(!source.ids.contains_key(&retained_grid));
    source.replace(Path::new(&paths[0]))?;
    let cells_tab = source.actor.widget(root, key("seven.cells.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: cells_tab }));
    source.build_frame(&mut tcx, 600, 720);
    let restored_grid = source.actor.widget(root, key("cells.grid")).unwrap();
    assert_ne!(restored_grid, retained_grid);
    assert_eq!(
        (
            source.actor.view().renderer[&restored_grid].scroll_x,
            source.actor.view().renderer[&restored_grid].scroll
        ),
        retained_offsets
    );
    let placed = source
        .placed
        .iter()
        .find(|node| node.node_id == source.ids[&restored_grid])
        .unwrap();
    assert_eq!(
        placed.scroll.offset,
        [retained_offsets.0 as f32, retained_offsets.1 as f32]
    );
    let restored_editor = source.actor.widget(root, key("cells.editor")).unwrap();
    assert_ne!(restored_editor, retained_editor);
    assert_eq!(source.focus, Some(restored_editor));
    assert_eq!(
        source.editors[&restored_editor].current_value(),
        Some("🦀abc")
    );
    assert_eq!(
        source.editors[&restored_editor].text_selection(),
        retained_selection
    );
    let counter_tab = source.actor.widget(root, key("seven.counter.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: counter_tab
    }));
    source.build_frame(&mut tcx, 600, 720);
    source
        .actor
        .root_actor_mut()
        .dispatch_text_request(seven_action("cells.draft"), "a🦀".into())?;
    source.replace(Path::new(&paths[1]))?;
    let cells_tab = source.actor.widget(root, key("seven.cells.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: cells_tab }));
    source.build_frame(&mut tcx, 600, 720);
    let shortened_editor = source.actor.widget(root, key("cells.editor")).unwrap();
    let shortened_selection = source.editors[&shortened_editor].text_selection().unwrap();
    assert_eq!(source.focus, Some(shortened_editor));
    assert_eq!(
        source.editors[&shortened_editor].current_value(),
        Some("a🦀")
    );
    assert_eq!(
        (shortened_selection.anchor, shortened_selection.head),
        (3, 1)
    );
    source.replace(Path::new(&paths[0]))?;
    source.build_frame(&mut tcx, 600, 720);
    let nested_grid = source.actor.widget(root, key("cells.grid")).unwrap();
    let nested_editor = source.actor.widget(root, key("cells.editor")).unwrap();
    let nested_cell = source.grid_target(nested_grid, 26).unwrap();
    let page = source.actor.widget(root, key("cells.panel")).unwrap();
    let nested_scope = source.actor.widget(root, key("cells.viewport")).unwrap();
    assert_eq!(source.focus_branch(nested_cell), Some(nested_scope));
    assert_eq!(source.focus_branch(nested_editor), Some(page));
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: nested_cell
    }));
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus {
        target: nested_editor
    }));
    assert!(source.actor.view().renderer[&nested_cell].focus);
    assert!(source.actor.view().renderer[&nested_editor].focus);
    assert_eq!(source.focus, Some(nested_editor));
    for prefer_cell in [false, true] {
        if prefer_cell {
            let grid = source.actor.widget(root, key("cells.grid")).unwrap();
            let cell = source.grid_target(grid, 26).unwrap();
            assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: cell }));
        }
        let counter_tab = source.actor.widget(root, key("seven.counter.tab")).unwrap();
        assert!(source.handle_semantic_action(shell::SemanticAction::Click {
            target: counter_tab
        }));
        source.build_frame(&mut tcx, 600, 720);
        source.replace(Path::new(&paths[1]))?;
        let cells_tab = source.actor.widget(root, key("seven.cells.tab")).unwrap();
        assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: cells_tab }));
        source.build_frame(&mut tcx, 600, 720);
        let expected = if prefer_cell {
            let grid = source.actor.widget(root, key("cells.grid")).unwrap();
            source.grid_target(grid, 26).unwrap()
        } else {
            source.actor.widget(root, key("cells.editor")).unwrap()
        };
        assert_eq!(source.focus, Some(expected));
        assert_eq!(
            source
                .actor
                .root_actor()
                .text_by_key(seven_state("cells.draft")),
            "a🦀"
        );
    }
    // Route removal recreates widgets, but opt-in renderer retention must keep
    // the grid's roving Tab position independently of its new widget identity.
    let previous_grid = source.actor.widget(root, key("cells.grid")).unwrap();
    let previous_cell = source.grid_target(previous_grid, 26).unwrap();
    assert_eq!(source.focus, Some(previous_cell));
    assert_eq!(
        source.actor.view().renderer[&previous_grid].grid_focus,
        Some(26)
    );
    let sources_slot = source
        .actor
        .root_actor()
        .slot_address_by_key(seven_state("cells.sources"));
    let active_before_route = source
        .actor
        .root_actor()
        .integer_by_key(seven_state("cells.active"));
    let counter_tab = source.actor.widget(root, key("seven.counter.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: counter_tab
    }));
    source.build_frame(&mut tcx, 600, 720);
    assert!(!source.ids.contains_key(&previous_grid));
    source.replace(Path::new(&paths[0]))?;
    let cells_tab = source.actor.widget(root, key("seven.cells.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: cells_tab }));
    source.build_frame(&mut tcx, 600, 720);
    let returned_grid = source.actor.widget(root, key("cells.grid")).unwrap();
    let returned_cell = source.grid_target(returned_grid, 26).unwrap();
    let restored_semantics = source.semantic_snapshot().unwrap();
    for (id, role) in [
        (returned_grid, shell::SemanticRole::Grid),
        (returned_cell, shell::SemanticRole::GridCell),
    ] {
        assert_eq!(
            restored_semantics
                .nodes
                .iter()
                .find(|node| node.id == id)
                .unwrap()
                .role,
            role
        );
    }
    println!(
        "PASS native Cells grid/row/gridcell semantics and clean cell names survive route restoration across real DLL replacement"
    );
    assert_ne!(returned_grid, previous_grid);
    assert_ne!(returned_cell, previous_cell);
    assert_eq!(source.focus, Some(returned_cell));
    assert_eq!(
        source.actor.view().renderer[&returned_grid].grid_focus,
        Some(26)
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("cells.sources")),
        sources_slot
    );
    source.handle_event(key_event("Tab"));
    source.build_frame(&mut tcx, 600, 720);
    assert_ne!(
        source.grid_owner(source.focus.unwrap()),
        Some(returned_grid)
    );
    source.replace(Path::new(&paths[1]))?;
    source.build_frame(&mut tcx, 600, 720);
    let mut return_tab = key_event("Tab");
    if let UiEvent::Key(key) = &mut return_tab {
        key.modifiers = shell::Modifiers::SHIFT;
    }
    source.handle_event(return_tab);
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.focus, Some(returned_cell));
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        active_before_route
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("cells.draft")),
        "a🦀"
    );
    println!(
        "PASS native Cells route removal/restoration retains roving Tab memory across real DLL replacements with new widget identities and unchanged state slot"
    );
    let removed_editor = source.actor.widget(root, key("cells.editor")).unwrap();
    assert!(source.editors.contains_key(&removed_editor));
    source
        .actor
        .root_actor_mut()
        .view
        .set_renderer_retention(0)?;
    let counter_tab = source.actor.widget(root, key("seven.counter.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click {
        target: counter_tab
    }));
    source.build_frame(&mut tcx, 600, 720);
    assert_eq!(source.actor.root_actor().view.retained_renderer_count(), 0);
    assert_eq!(source.actor.root_actor().view.retained_renderer_bytes(), 0);
    assert!(!source.ids.contains_key(&returned_cell));
    assert!(!source.editors.contains_key(&removed_editor));
    source.replace(Path::new(&paths[0]))?;
    let cells_tab = source.actor.widget(root, key("seven.cells.tab")).unwrap();
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: cells_tab }));
    source.build_frame(&mut tcx, 600, 720);
    let uncached_grid = source.actor.widget(root, key("cells.grid")).unwrap();
    assert_eq!(
        source.actor.view().renderer[&uncached_grid].grid_focus,
        None
    );
    assert_ne!(source.focus, source.grid_target(uncached_grid, 26));
    assert_eq!(
        source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("cells.sources")),
        sources_slot
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.active")),
        active_before_route
    );
    assert_eq!(
        source
            .actor
            .root_actor()
            .text_by_key(seven_state("cells.draft")),
        "a🦀"
    );
    source
        .actor
        .root_actor_mut()
        .view
        .set_renderer_retention(512)?;
    println!(
        "PASS native Cells disabling retention removes cache/editor/stale focus across route removal and real DLL replacement while actor state remains intact"
    );
    if let Some(tall) = paths.get(2) {
        let editor = source.actor.widget(root, key("cells.editor")).unwrap();
        assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: editor }));
        let editor_instance = &*source.editors[&editor] as *const TextInput;
        let selection = source.editors[&editor].text_selection();
        let draft = source
            .actor
            .root_actor()
            .text_by_key(seven_state("cells.draft"))
            .to_owned();
        let schema = source.actor.root_actor().schema_version;
        let slot = source
            .actor
            .root_actor()
            .slot_address_by_key(seven_state("cells.sources"));
        let allocation = source
            .actor
            .root_actor()
            .text_list_by_key(seven_state("cells.sources"))
            .as_ptr();
        let grid = source.actor.widget(root, key("cells.grid")).unwrap();
        let grid_id = source.ids[&grid];
        let visible_cells = || {
            source
                .actor
                .view()
                .bindings
                .values()
                .filter(|binding| {
                    binding.node.kind == BUTTON && binding.node.action == key("cells.choose")
                })
                .count()
        };
        let before_count = visible_cells();
        let before_events = source
            .actor
            .root_actor()
            .integer_by_key(seven_state("cells.viewport.events"));
        let before_scroll = source.actor.view().renderer[&grid].scroll;
        source.replace(Path::new(tall))?;
        source.build_frame(&mut tcx, 600, 1000);
        let after_count = source
            .actor
            .view()
            .bindings
            .values()
            .filter(|binding| {
                binding.node.kind == BUTTON && binding.node.action == key("cells.choose")
            })
            .count();
        assert!(
            after_count > before_count,
            "{before_count} -> {after_count}"
        );
        assert_eq!(
            source
                .actor
                .root_actor()
                .float_by_key(seven_state("cells.viewport.height")),
            400.0
        );
        assert_eq!(
            source
                .actor
                .root_actor()
                .integer_by_key(seven_state("cells.viewport.events")),
            before_events + 1
        );
        assert_eq!(source.ids[&grid], grid_id);
        assert_eq!(source.actor.root_actor().schema_version, schema);
        assert_eq!(
            source
                .actor
                .root_actor()
                .slot_address_by_key(seven_state("cells.sources")),
            slot
        );
        assert_eq!(
            source
                .actor
                .root_actor()
                .text_list_by_key(seven_state("cells.sources"))
                .as_ptr(),
            allocation
        );
        assert_eq!(
            source
                .actor
                .root_actor()
                .text_by_key(seven_state("cells.draft")),
            draft
        );
        assert_eq!(
            &*source.editors[&editor] as *const TextInput,
            editor_instance
        );
        assert_eq!(source.editors[&editor].text_selection(), selection);
        assert_eq!(source.focus, Some(editor));
        assert_eq!(source.actor.view().renderer[&grid].scroll, before_scroll);
        let first = (before_scroll / 44.0).floor() as i64;
        let end = ((before_scroll + 400.0) / 44.0).ceil().min(100.0) as i64;
        for row in first..end {
            for column in 0..26 {
                let cell = source
                    .grid_target(grid, row * 26 + column)
                    .expect("visible row omitted");
                assert!(
                    source
                        .placed
                        .iter()
                        .any(|node| node.node_id == source.ids[&cell]),
                    "visible cell unmeasured"
                );
            }
        }
        assert!((3990.0..=4010.0).contains(&source.tree.layout(grid_id).unwrap().scroll_height()));
        source.build_frame(&mut tcx, 600, 1000);
        assert_eq!(
            source
                .actor
                .root_actor()
                .integer_by_key(seven_state("cells.viewport.events")),
            before_events + 1
        );
        source.replace(Path::new(&paths[0]))?;
        source.build_frame(&mut tcx, 600, 720);
        assert_eq!(
            source
                .actor
                .root_actor()
                .float_by_key(seven_state("cells.viewport.height")),
            200.0
        );
        assert_eq!(
            source
                .actor
                .root_actor()
                .integer_by_key(seven_state("cells.viewport.events")),
            before_events + 2
        );
        assert_eq!(source.focus, Some(editor));
        assert_eq!(
            source
                .actor
                .root_actor()
                .text_by_key(seven_state("cells.draft")),
            draft
        );
        let restored_count = source
            .actor
            .view()
            .bindings
            .values()
            .filter(|binding| {
                binding.node.kind == BUTTON && binding.node.action == key("cells.choose")
            })
            .count();
        assert_eq!(restored_count, before_count);
        println!(
            "PASS native viewport height replacement expands virtual rows before painting; reverse replacement shrinks the port and retains state/editor/selection/focus/scroll"
        );
    }
    source
        .actor
        .root_actor_mut()
        .view
        .set_renderer_retention(0)?;
    println!(
        "PASS 7GUIs initial tasks: Cells native roving Tab/re-entry after DLL replacement and arrows/Home/End/paging/primary-Home-End/focus/reveal after DLL replacement, Enter/Escape and full-sheet editing and two-axis reload-retained scroll/editor; typed Router history/branching retained across real DLL reload, native Back/Forward and disabled boundary, Cells source allocation/selection/uncommitted draft retained across real DLL reload; Counter, native bidirectional temperature edit callback, invalid input/error recovery, DLL reload retains state/slot/editor/focus/selection and narrow layout; Flight Booker date validation, disabled return/Book, current-code booking and retained draft/selection; CRUD native edit/filter/create/update/delete, stale row rejection and reload-retained selection/draft/editor/scroll; Circle native canvas create/select/resize/save/cancel/undo/redo and real-DLL edit retention, modal Tab/background/Escape/focus restoration; Timer determinate progress, native drag/keyboard/reset, retained control/focus/drag across DLL replacement"
    );
    if window {
        shell::run_window_with_options(Box::new(source.capture()), options)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
