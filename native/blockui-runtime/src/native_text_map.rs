use super::*;
pub fn migration_proof(base: &Path, rejected: &Path, evolved: &Path) -> Result<(), String> {
    let mut actor = Actor::new(141, Path::new(".runtime"))?;
    actor.reload(base)?;
    actor.dispatch(Message {
        action: key("records.put"),
        value: 42,
    })?;
    let input = key("labels.document.editor");
    actor.edit_text(input, "Draft 🦀")?;
    let state = actor.state_address();
    let slot = actor.slot_address("labels.documents");
    let schema = actor.schema_version;
    let version = actor.implementation_version();
    let stored = actor.text_list("labels.documents").to_vec();
    let buffer = actor.text_list("labels.documents").as_ptr();
    let old_format = stored[1].split_once('\n').unwrap().0.to_owned();
    let marker = actor.slot_address(&old_format);
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 320, 480);
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":8,"head":6}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 320, 480);
    let node = source.ids[&input];
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let selection = source.editors[&input].text_selection();
    assert!(source.replace(rejected).is_err());
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.actor.implementation_version(), version);
    assert_eq!(source.actor.schema_version, schema);
    assert_eq!(source.actor.text_list("labels.documents"), stored);
    assert_eq!(source.actor.text_list("labels.documents").as_ptr(), buffer);
    assert_eq!(source.actor.slot_address(&old_format), marker);
    assert_eq!(source.ids[&input], node);
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    source.replace(evolved)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_ne!(source.actor.schema_version, schema);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("labels.documents"), slot);
    assert_eq!(source.actor.slot_address(&old_format), 0);
    assert!(source.actor.text_list("labels.documents")[1].contains("Uncategorized"));
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.actor.text("labels.document_title"), "Draft 🦀");
    source.actor.edit_text(input, "After schema upgrade 🦀")?;
    source.sync()?;
    for width in [640, 320] {
        source.build_frame(&mut tcx, width, 480);
        assert_eq!(
            source.actor.view.bindings[&key("labels.document.42")].text,
            "1: After schema upgrade 🦀"
        );
        let rect = source.widget_layout_rect(input).unwrap();
        assert!(rect[0] >= 0.0 && rect[2] <= width as f32 + 0.01);
    }
    println!(
        "PASS native generated record schema migration preserves editor node/allocation draft Unicode selection and focus; rejected migration retains old code/records; accepted defaults and current field edits work"
    );
    Ok(())
}
pub fn proof(base: &Path, replacement: &Path) -> Result<(), String> {
    let mut actor = Actor::new(135, Path::new(".runtime"))?;
    actor.reload(base)?;
    for value in [2, 1] {
        actor.dispatch(Message {
            action: key("map.insert"),
            value,
        })?;
    }
    actor.dispatch(Message {
        action: key("records.put"),
        value: 42,
    })?;
    let input = key("labels.document.editor");
    actor.edit_text(input, "Draft 🦀")?;
    let document = key("labels.document.42");
    let row = key("labels.map.1");
    let removed = key("labels.map.2");
    let mut source = NativeSource::without_watch(actor)?;
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 320, 480);
    let node = source.ids[&row];
    let document_node = source.ids[&document];
    source.set_focus(Some(input));
    source
        .editors
        .get_mut(&input)
        .unwrap()
        .config_changed(r#"{"selection":{"anchor":8,"head":6}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 320, 480);
    let editor = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let documents_slot = source.actor.slot_address("labels.documents");
    let documents_buffer = source.actor.text_list("labels.documents").as_ptr();
    let selection = source.editors[&input].text_selection();
    assert_eq!(source.actor.view.bindings[&document].text, "1: Draft 🦀");
    let rect = source.widget_layout_rect(row).unwrap();
    assert!(rect[2] > rect[0] && rect[3] > rect[1] && rect[2] <= 320.01 && rect[3] <= 480.01);
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(
        source.actor.slot_address("labels.documents"),
        documents_slot
    );
    assert_eq!(
        source.actor.text_list("labels.documents").as_ptr(),
        documents_buffer
    );
    assert_eq!(source.ids[&row], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        editor
    );
    assert_eq!(source.actor.text("labels.document_title"), "Draft 🦀");
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.ids[&document], document_node);
    assert_eq!(source.actor.view.bindings[&document].text, "1: Draft 🦀");
    assert!(
        source
            .actor
            .dispatch(Message {
                action: key("records.batch"),
                value: 1
            })
            .is_err()
    );
    source.sync()?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.ids[&document], document_node);
    assert_eq!(source.actor.view.bindings[&document].text, "1: Draft 🦀");
    assert!(!source.ids.contains_key(&key("labels.document.73")));
    source.actor.dispatch(Message {
        action: key("records.batch"),
        value: 0,
    })?;
    source.sync()?;
    for width in [640, 320] {
        source.build_frame(&mut tcx, width, 480);
        assert_eq!(source.ids[&document], document_node);
        assert_eq!(
            source.actor.view.bindings[&document].text,
            "2: Draft 🦀 edited"
        );
        for id in [input, document, key("labels.document.73")] {
            let rect = source.widget_layout_rect(id).unwrap();
            assert!(rect[2] > rect[0] && rect[3] > rect[1]);
            assert!(
                rect[0] >= 0.0
                    && rect[1] >= 0.0
                    && rect[2] <= width as f32 + 0.01
                    && rect[3] <= 480.01
            );
        }
    }
    source.actor.dispatch(Message {
        action: key("map.insert"),
        value: 1,
    })?;
    source.actor.dispatch(Message {
        action: key("map.remove"),
        value: 2,
    })?;
    source.sync()?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.ids[&row], node);
    assert_eq!(source.actor.view.bindings[&row].text, "new 🦀");
    assert!(!source.ids.contains_key(&removed));
    assert!(
        source
            .actor
            .dispatch(Message {
                action: key("map.rename"),
                value: -1
            })
            .is_err()
    );
    source.sync()?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.ids[&row], node);
    assert_eq!(source.actor.view.bindings[&row].text, "new 🦀");
    source.actor.edit_text(input, "After reload 🦀")?;
    source.sync()?;
    source.build_frame(&mut tcx, 320, 480);
    assert_eq!(source.ids[&document], document_node);
    assert_eq!(
        source.actor.view.bindings[&document].text,
        "2: After reload 🦀"
    );
    assert_eq!(
        source.actor.text("labels.document_title"),
        "After reload 🦀"
    );
    println!(
        "PASS native record title input retains editor, focus, Unicode selection and draft through DLL replacement; current actions update typed fields"
    );
    println!(
        "PASS native keyed map rows retain Taffy identity through reload/update/other-row removal; rejected batch preserves row"
    );
    println!(
        "PASS native typed record rows retain identity and values through DLL replacement; failed field edits preserve rows and successful edits patch text at 640/320 widths"
    );
    Ok(())
}
