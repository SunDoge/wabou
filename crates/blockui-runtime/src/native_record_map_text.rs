use super::*;
use crate::Envelope;
pub fn proof(
    base: &Path,
    replacement: &Path,
    missing: &Path,
    evolved: &Path,
) -> Result<(), String> {
    let mut actor = Actor::new(221, Path::new(".runtime"))?;
    actor.reload(base)?;
    actor.dispatch(Message {
        action: key("Model.initialize"),
        value: 0,
    })?;
    let input = *actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.kind == ENTRY)
        .ok_or("missing input")?
        .0;
    let area = *actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.kind == TEXTAREA)
        .ok_or("missing area")?
        .0;
    let state = actor.state_address();
    let slot = actor.slot_address("map.text.rows");
    let buffer = actor.text_list("map.text.rows").as_ptr();
    let mut source = NativeSource::without_watch(actor)?;
    source.set_semantics_enabled(true);
    source.set_focus(Some(input));
    let mut tcx = TextContext::new();
    source.build_frame(&mut tcx, 480, 480);
    source
        .editors
        .get_mut(&input)
        .ok_or("missing editor")?
        .config_changed(r#"{"selection":{"anchor":1,"head":3}}"#)
        .unwrap();
    source.build_frame(&mut tcx, 480, 480);
    let selection = source.editors[&input].text_selection();
    assert_eq!(
        selection.as_ref().map(|value| (value.anchor, value.head)),
        Some((1, 3))
    );
    let widget = (&**source.editors.get(&input).unwrap() as *const TextInput) as usize;
    let area_widget = (&**source.editors.get(&area).unwrap() as *const TextInput) as usize;
    let node = source.ids[&input];
    source.replace(replacement)?;
    source.build_frame(&mut tcx, 480, 480);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("map.text.rows"), slot);
    assert_eq!(source.actor.text_list("map.text.rows").as_ptr(), buffer);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    let evaluations = source.actor.view.bindings[&input].evaluations;
    source.actor.dispatch(Message {
        action: key("Model.metadata"),
        value: 0,
    })?;
    source.actor.dispatch(Message {
        action: key("Model.prepend"),
        value: 0,
    })?;
    assert_eq!(source.actor.view.bindings[&input].evaluations, evaluations);
    let dynamic = *source
        .actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.semantic_label.as_deref() == Some("Name aaa"))
        .ok_or("dynamic row missing")?
        .0;
    assert_eq!(
        source.actor.view.bindings[&dynamic].node.action,
        source.actor.view.bindings[&input].node.action
    );
    assert_eq!(
        source.actor.view.bindings[&area].node.action,
        source.actor.view.bindings[&input].node.action
    );
    source.actor.edit_text(dynamic, "  Dynamic  ")?;
    assert_eq!(source.actor.view.bindings[&dynamic].text, "Dynamic");
    assert_eq!(source.actor.view.bindings[&input].evaluations, evaluations);
    source.actor.edit_text(input, "  A😀bc  ")?;
    assert_eq!(source.actor.view.bindings[&input].text, "A😀bc");
    source.build_frame(&mut tcx, 480, 480);
    assert_eq!(source.editors[&input].text_selection(), selection);
    let buffer = source.actor.text_list("map.text.rows").as_ptr();
    assert!(source.actor.edit_text(input, "bad\0text").is_err());
    assert!(
        source
            .actor
            .dispatch(Message {
                action: key("Model.remove_visible"),
                value: 0
            })
            .is_err()
    );
    assert_eq!(source.actor.text_list("map.text.rows").as_ptr(), buffer);
    let version = source.actor.implementation_version();
    assert!(source.replace(missing).is_err());
    assert_eq!(source.actor.implementation_version(), version);
    assert_eq!(source.actor.text_list("map.text.rows").as_ptr(), buffer);
    source.replace(evolved)?;
    source.build_frame(&mut tcx, 480, 480);
    assert_eq!(source.actor.state_address(), state);
    assert_eq!(source.actor.slot_address("map.text.rows"), slot);
    assert_eq!(source.focus, Some(input));
    assert_eq!(source.editors[&input].text_selection(), selection);
    assert_eq!(source.ids[&input], node);
    assert_eq!(
        (&**source.editors.get(&input).unwrap() as *const TextInput) as usize,
        widget
    );
    assert_eq!(
        (&**source.editors.get(&area).unwrap() as *const TextInput) as usize,
        area_widget
    );
    source.handle_event(UiEvent::TextInput("🦀".into()));
    source.build_frame(&mut tcx, 480, 480);
    assert_eq!(source.actor.view.bindings[&input].text, "A🦀bc");
    assert_eq!(
        source.actor.view.bindings[&key("first.preview")].text,
        "A🦀bc"
    );
    let evaluations = source.actor.view.bindings[&input].evaluations;
    source.actor.edit_text(area, "Changed\n🦀")?;
    source.actor.dispatch(Message {
        action: key("Model.metadata"),
        value: 0,
    })?;
    assert_eq!(source.actor.view.bindings[&input].evaluations, evaluations);
    for width in [320, 640] {
        source.build_frame(&mut tcx, width, 480);
        let mut bottom = 0.0;
        for key in &source.actor.view.order {
            let Some(binding) = source.actor.view.bindings.get(key) else {
                continue;
            };
            if binding.node.kind == COLUMN {
                continue;
            }
            let rect = source
                .widget_layout_rect(*key)
                .ok_or("missing layout node")?;
            assert!(
                rect[0] >= 0.0
                    && rect[1] >= bottom
                    && rect[2] <= width as f32 + 0.5
                    && rect[3] <= 480.0
            );
            bottom = rect[3];
        }
    }
    let mut queued = Actor::new(222, Path::new(".runtime"))?;
    queued.reload(base)?;
    queued.dispatch(Message {
        action: key("Model.initialize"),
        value: 0,
    })?;
    let buffer = queued.text_list("map.text.rows").as_ptr();
    let mut envelope: Envelope = Message {
        action: key("Model.changed"),
        value: 0,
    }
    .into();
    envelope.message_text =
        Some(blockui_abi::keyed_text::encode("first\n🦀", "  Queued  ").map_err(str::to_owned)?);
    assert!(queued.enqueue_envelope(envelope).is_none());
    queued.reload(replacement)?;
    assert_eq!(queued.text_list("map.text.rows").as_ptr(), buffer);
    queued.tick()?;
    assert_eq!(queued.view.bindings[&key("first.preview")].text, "Queued");
    let buffer = queued.text_list("map.text.rows").as_ptr();
    let mut invalid: Envelope = Message {
        action: key("Model.changed"),
        value: 0,
    }
    .into();
    invalid.message_text = Some("ordinary unframed text".into());
    assert!(queued.enqueue_envelope(invalid).is_none());
    assert!(queued.tick().is_err());
    assert_eq!(queued.text_list("map.text.rows").as_ptr(), buffer);
    println!(
        "PASS shared keyed text actions: dynamic rows and multiline editor use one action, Unicode/newline keys route correctly, queued messages execute new code after real DLL replacement, invalid payloads preserve state"
    );
    println!(
        "PASS native record collection text: direct label/input/textarea retain row-keyed native editors/nodes/focus/Unicode selection through real code and reordered schema replacement; current normalization, rejection/missing migration rollback, sibling-field and other-row cache isolation, multiline editing and 320/640 layout verified"
    );
    Ok(())
}
