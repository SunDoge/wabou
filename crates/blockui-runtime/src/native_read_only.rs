use super::*;

pub fn proof(paths: &[String], capture: bool) -> Result<(), String> {
    if paths.len() != 6 {
        return Err(
            "read-only proof needs entry base/readonly/editable and area base/readonly/editable"
                .into(),
        );
    }
    let mut captured = None;
    for (index, paths) in paths.chunks(3).enumerate() {
        let mut actor = Actor::new(920 + index as u64, Path::new(".runtime"))?;
        actor.reload(Path::new(&paths[0]))?;
        actor.set_text(key("notes"), "a😀bc draft")?;
        let slot = actor.slot_address("notes");
        let storage = actor.state_address();
        let mut source = NativeSource::without_watch(actor)?;
        let input = key("counter.notes");
        let mut tcx = TextContext::new();
        source.handle_event(UiEvent::Focus(true));
        source.set_focus(Some(input));
        source.build_frame(&mut tcx, 720, 620);
        source
            .editors
            .get_mut(&input)
            .unwrap()
            .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
            .unwrap();
        source.build_frame(&mut tcx, 720, 620);
        let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
        let node = source.ids[&input];
        let selection = source.editors[&input].text_selection();
        let key_event = |name: &str, modifiers| {
            UiEvent::Key(shell::KeyEvent {
                phase: shell::KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: name.into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers,
                repeat: false,
                synthetic: false,
            })
        };
        let primary = if cfg!(target_os = "macos") {
            shell::Modifiers::META
        } else {
            shell::Modifiers::CONTROL
        };
        let request = source.handle_event(key_event("v", primary));
        assert_eq!(request.clipboard, Some(shell::ClipboardRequest::Read));
        assert_eq!(source.paste_target, Some(input));
        source.replace(Path::new(&paths[1]))?;
        source.build_frame(&mut tcx, 720, 620);
        assert_eq!(source.actor.state_address(), storage);
        assert_eq!(source.actor.slot_address("notes"), slot);
        assert_eq!(source.ids[&input], node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            editor
        );
        assert_eq!(source.editors[&input].text_selection(), selection);
        assert_eq!(source.focus, Some(input));
        assert!(source.ime_state().is_none());
        assert_eq!(source.paste_target, None);
        let copied = source.handle_event(key_event("c", primary));
        assert_eq!(
            copied.clipboard,
            Some(shell::ClipboardRequest::Write("😀".into()))
        );
        assert!(
            source
                .handle_event(key_event("x", primary))
                .clipboard
                .is_none()
        );
        assert!(
            source
                .handle_event(key_event("v", primary))
                .clipboard
                .is_none()
        );
        for event in [
            UiEvent::TextInput("wrong".into()),
            UiEvent::Paste("wrong".into()),
            UiEvent::Ime(shell::ImeEvent::Commit("wrong".into())),
            UiEvent::Ime(shell::ImeEvent::DeleteSurrounding {
                before_bytes: 4,
                after_bytes: 3,
            }),
            key_event("Backspace", shell::Modifiers::empty()),
            key_event("Delete", shell::Modifiers::empty()),
            key_event("Enter", shell::Modifiers::empty()),
        ] {
            source.handle_event(event);
            source.build_frame(&mut tcx, 720, 620);
            assert_eq!(source.actor.text("notes"), "a😀bc draft");
            assert_eq!(source.editors[&input].current_value(), Some("a😀bc draft"));
        }
        source.handle_event(key_event("ArrowRight", shell::Modifiers::empty()));
        source.build_frame(&mut tcx, 720, 620);
        assert_ne!(source.editors[&input].text_selection(), selection);
        source.replace(Path::new(&paths[2]))?;
        source.build_frame(&mut tcx, 720, 620);
        assert!(source.ime_state().is_some());
        source.handle_event(UiEvent::Paste("stale clipboard result".into()));
        source.build_frame(&mut tcx, 720, 620);
        assert_eq!(source.actor.text("notes"), "a😀bc draft");
        source.handle_event(UiEvent::Ime(shell::ImeEvent::Preedit {
            text: "仮".into(),
            cursor: None,
        }));
        source.build_frame(&mut tcx, 720, 620);
        assert_eq!(source.composing_editor, Some(input));
        assert_eq!(source.actor.text("notes"), "a😀bc draft");
        source.replace(Path::new(&paths[1]))?;
        source.build_frame(&mut tcx, 720, 620);
        assert_eq!(source.composing_editor, None);
        assert!(source.ime_state().is_none());
        assert_eq!(source.editors[&input].current_value(), Some("a😀bc draft"));
        source.replace(Path::new(&paths[2]))?;
        source.build_frame(&mut tcx, 720, 620);
        source.handle_event(UiEvent::TextInput("!".into()));
        source.build_frame(&mut tcx, 720, 620);
        assert!(source.actor.text("notes").contains('!'));
        assert_eq!(source.actor.slot_address("notes"), slot);
        assert_eq!(source.ids[&input], node);
        assert_eq!(
            (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
            editor
        );
        // Both window blur and client focus changes terminate the IME session.
        // Replacement while blurred must not revive uncommitted composition.
        let durable = source.actor.text("notes").to_owned();
        for window_blur in [true, false] {
            source.handle_event(UiEvent::Ime(shell::ImeEvent::Preedit {
                text: "未確定😀".into(),
                cursor: None,
            }));
            source.build_frame(&mut tcx, 720, 620);
            assert_eq!(source.composing_editor, Some(input));
            assert!(source.ime_state().unwrap().marked_range_utf16.is_some());
            if window_blur {
                source.handle_event(UiEvent::Focus(false));
            } else {
                source.set_focus(None);
            }
            source.build_frame(&mut tcx, 720, 620);
            assert_eq!(source.composing_editor, None);
            assert!(source.ime_state().is_none());
            source.replace(Path::new(&paths[2]))?;
            source.build_frame(&mut tcx, 720, 620);
            if window_blur {
                source.handle_event(UiEvent::Focus(true));
            } else {
                source.set_focus(Some(input));
            }
            source.build_frame(&mut tcx, 720, 620);
            assert!(source.ime_state().unwrap().marked_range_utf16.is_none());
            assert_eq!(source.actor.text("notes"), durable);
            assert_eq!(
                source.editors[&input].current_value(),
                Some(durable.as_str())
            );
            assert_eq!(source.actor.slot_address("notes"), slot);
            assert_eq!(
                (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
                editor
            );
        }
        println!(
            "PASS native IME window/client blur cancels Unicode preedit through real DLL replacement without changing durable text or editor identity"
        );
        println!(
            "PASS read-only {} retains editor/slot/node/focus/selection across real DLL replacement, permits copy/navigation, blocks text/paste/cut/delete/IME, cancels preedit and resumes editing",
            if index == 0 { "TextInput" } else { "TextArea" }
        );
        if index == 1 {
            captured = Some(source);
        }
    }
    if capture {
        shell::run_window_with_options(
            Box::new(captured.unwrap().capture()),
            shell::WindowOptions::new()
                .title("Read-only reload")
                .initial_inner_size(720, 620),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
