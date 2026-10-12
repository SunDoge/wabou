use super::*;

pub fn proof(paths: &[String], window: bool) -> Result<(), String> {
    if paths.len() != 4 {
        return Err("request feedback needs client/replacement/service/repaired DLLs".into());
    }
    let root = ActorId(1);
    let (mut source, options) = crate::App::new(&paths[0], ".runtime")
        .implementation("feedback.service", &paths[2])?
        .window(
            crate::app::WindowOptions::new()
                .title("Request feedback")
                .initial_inner_size(520, 480),
        )
        .into_window()?;
    source.set_semantics_enabled(true);
    let service = source
        .actor
        .runtime()
        .declared_children()
        .into_iter()
        .find(|(id, _)| source.actor.runtime().parent(*id) == Some(root))
        .unwrap()
        .0;
    let input = source
        .actor
        .widget(root, key("feedback.notes.input"))
        .unwrap();
    let button = source
        .actor
        .widget(root, key("feedback.load.button"))
        .unwrap();
    let status = source
        .actor
        .widget(root, key("feedback.status.text"))
        .unwrap();
    let storage = source.actor.runtime().actor(root).unwrap().state_address();
    let slot = source
        .actor
        .runtime()
        .actor(root)
        .unwrap()
        .slot_address("feedback.notes");
    let service_slot = source
        .actor
        .runtime()
        .actor(service)
        .unwrap()
        .slot_address("feedback.service.calls");
    source.handle_event(UiEvent::Focus(true));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 520, 480);
    let node = source.ids[&input];
    assert!(source.handle_semantic_action(shell::SemanticAction::Focus { target: input }));
    source
        .actor
        .root_actor_mut()
        .set_text(key("feedback.notes"), "Unsaved notes 😀")?;
    source.build_frame(&mut tcx, 520, 480);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    source.build_frame(&mut tcx, 520, 480);
    assert!(
        source
            .actor
            .runtime()
            .actor(root)
            .unwrap()
            .boolean("feedback.busy")
    );
    assert!(!source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    source.actor.tick()?;
    assert!(
        source
            .actor
            .tick()
            .unwrap_err()
            .contains("Service unavailable")
    );
    // Failure is queued for the caller; replace both plugins before delivering it.
    source.replace(Path::new(&paths[1]))?;
    source.actor.reload_actor(service, Path::new(&paths[3]))?;
    source.actor.tick()?;
    source.build_frame(&mut tcx, 520, 480);
    assert_eq!(
        source.actor.view().bindings[&status].text,
        "Please retry — Service unavailable"
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.ids[&input], node);
    let actor = source.actor.runtime().actor(root).unwrap();
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("feedback.notes"), slot);
    assert_eq!(actor.text("feedback.notes"), "Unsaved notes 😀");
    assert!(!actor.boolean("feedback.busy"));
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(service)
            .unwrap()
            .integer("feedback.service.calls"),
        0
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(service)
            .unwrap()
            .slot_address("feedback.service.calls"),
        service_slot
    );
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    // Later editing must not rewrite the owned in-flight request payload.
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":0,"head":16}}"#)?;
    source.build_frame(&mut tcx, 520, 480);
    source.handle_event(UiEvent::TextInput("New draft after send 🦀".into()));
    source.prepare_for_event(&mut tcx);
    assert_eq!(
        source.actor.root_actor().text("feedback.notes"),
        "New draft after send 🦀"
    );
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":23,"head":21}}"#)?;
    source.build_frame(&mut tcx, 520, 480);
    let selection = source.editors[&input].text_selection();
    assert_eq!(
        (
            selection.as_ref().unwrap().anchor,
            selection.as_ref().unwrap().head
        ),
        (23, 21)
    );
    let input_instance = &*source.editors[&input] as *const TextInput;
    source.replace(Path::new(&paths[0]))?;
    source.actor.reload_actor(service, Path::new(&paths[3]))?;

    for _ in 0..3 {
        source.actor.tick()?;
    }
    source.build_frame(&mut tcx, 520, 480);
    assert_eq!(
        source.actor.view().bindings[&status].text,
        "Received: Unsaved notes 😀"
    );
    assert_eq!(
        source
            .actor
            .runtime()
            .actor(service)
            .unwrap()
            .integer("feedback.service.calls"),
        1
    );
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        source.actor.root_actor().text("feedback.notes"),
        "New draft after send 🦀"
    );
    assert_eq!(&*source.editors[&input] as *const TextInput, input_instance);
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert!(!source.actor.root_actor().boolean("feedback.busy"));
    source.build_frame(&mut tcx, 320, 400);
    let button_rect = source.widget_layout_rect(button).unwrap();
    assert!(button_rect[0] >= 0.0 && button_rect[2] <= 320.0 && button_rect[3] <= 400.0);
    source.build_frame(&mut tcx, 520, 480);
    println!(
        "PASS public App request feedback: owned text requests, pending/disabled, queued error through both DLL swaps, current receiver callback, later native draft editing/reversed Unicode selection retained without reply overwrite, retained notes/state/widget/focus, rollback and repaired retry"
    );
    if window {
        shell::run_window_with_options(Box::new(source.capture()), options)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}
