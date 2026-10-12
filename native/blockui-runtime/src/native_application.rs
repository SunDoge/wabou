//! Native proof of the stable-host / replaceable-UI boundary.
use super::*;

/// Uses a disposable UI source file supplied by the verification harness.
pub fn watch_proof(mut actor: Actor, config: ReloadConfig, editable: &Path) -> Result<(), String> {
    let original = std::fs::read_to_string(editable).map_err(|e| e.to_string())?;
    let input = actor
        .view
        .bindings
        .values()
        .find(|b| b.node.kind == ENTRY)
        .ok_or("missing notebook editor")?
        .node
        .key;
    let button = actor
        .view
        .bindings
        .values()
        .find(|b| b.node.kind == BUTTON)
        .ok_or("missing notebook button")?
        .node
        .key;
    actor.edit_text(input, "Hello 世界")?;
    actor.dispatch_widget(button)?;
    let draft = key("Notes.draft");
    let summary = key("Notes.summary");
    assert_eq!(actor.text_by_key(summary), "2 words · request 1");
    let state = actor.state_address();
    let slot = actor.slot_address_by_key(draft);
    let version = actor.implementation_version();
    let mut source = NativeSource::new(actor, config)?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while !source
        .watcher
        .as_ref()
        .unwrap()
        .ready
        .load(Ordering::Acquire)
    {
        if Instant::now() > deadline {
            return Err("watcher startup timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    source.set_semantics_enabled(true);
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 720, 480);
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)?;
    source.build_frame(&mut tcx, 720, 480);
    let selection = source.editors[&input].text_selection();
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    std::fs::write(
        editable,
        format!("{original}\nfn broken() {{ let invalid: = ; }}\n"),
    )
    .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while source.diagnostic.is_empty() {
        source.poll_async();
        if Instant::now() > deadline {
            return Err("failed UI compilation timed out".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(source.actor.implementation_version(), version);
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    assert_eq!(source.actor.text_by_key(summary), "2 words · request 2");
    source.set_focus(Some(input));
    let repaired = original
        .replace("\"Notebook\"", "\"Notebook repaired\"")
        .replace("{prefix}{} words", "Repaired: {prefix}{} words");
    assert_ne!(repaired, original);
    let saved = Instant::now();
    std::fs::write(editable, repaired).map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(30);
    while source.actor.implementation_version() == version {
        source.poll_async();
        if Instant::now() > deadline {
            return Err(format!("repaired UI timed out: {}", source.diagnostic));
        }
        thread::sleep(Duration::from_millis(10));
    }
    source.build_frame(&mut tcx, 720, 480);
    let latency = saved.elapsed().as_millis();
    assert!(
        source
            .actor
            .view
            .bindings
            .values()
            .any(|b| b.text == "Notebook repaired")
    );
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address_by_key(draft), slot);
    assert_eq!(source.actor.text_by_key(draft), "Hello 世界");
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    assert_eq!(
        source.actor.text_by_key(summary),
        "Repaired: 2 words · request 3"
    );
    assert_eq!(source.actor.artifact_count(), 1);
    #[cfg(target_os = "linux")]
    assert_eq!(source.actor.mapped_implementation_count()?, 1);
    println!(
        "PASS notebook source watcher: failed compilation keeps old UI/service usable; repair retains state/editor/focus/selection and runs new events; save_to_native_layout_ms={latency}"
    );
    Ok(())
}

pub fn editor_proof(actor: Actor, replacement: &Path) -> Result<(), String> {
    let input = actor
        .view
        .bindings
        .values()
        .find(|binding| binding.node.kind == ENTRY)
        .ok_or("application proof needs an editor")?
        .node
        .key;
    let button = actor
        .view
        .bindings
        .values()
        .find(|binding| binding.node.kind == BUTTON)
        .ok_or("application proof needs a button")?
        .node
        .key;
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    let mut text = TextContext::new();
    source.build_frame(&mut text, 720, 480);
    source.focus = Some(input);
    source.editors.get_mut(&input).unwrap().focus_changed(true);
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":1,"head":4}}"#)
        .map_err(|e| e.to_string())?;
    source.build_frame(&mut text, 720, 480);
    let selection = source.editors[&input].text_selection();
    assert_eq!(
        selection.as_ref().map(|value| (value.anchor, value.head)),
        Some((1, 4))
    );
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    let value = source.editors[&input].current_value().unwrap().to_owned();
    source.replace(replacement)?;
    source.build_frame(&mut text, 720, 480);
    assert!(source.diagnostic.is_empty(), "{}", source.diagnostic);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.editors[&input].current_value(), Some(value.as_str()));
    assert!(source.handle_semantic_action(shell::SemanticAction::Click { target: button }));
    println!(
        "PASS native UI replacement retains editor/node/focus/selection; new event calls stable host service"
    );
    Ok(())
}
