#![cfg(feature = "testing")]
use blockui::*;
#[derive(Record, Default)]
#[record(name = "map.text.row")]
struct Row {
    name: String,
    edits: i64,
}
#[actor(name = "map.text")]
struct Model {
    #[state(codec = Row)]
    rows: State<Vec<String>>,
    visible: State<bool>,
}
#[actor_impl]
impl Model {
    #[action]
    fn initialize(&self, cx: &Cx<'_>) -> Result<()> {
        let rows = RecordMap::<Row>::new(self.rows);
        rows.insert(
            cx,
            "first\n🦀",
            &Row {
                name: "A😀bc".into(),
                ..Default::default()
            },
        )?;
        rows.insert(
            cx,
            "second",
            &Row {
                name: "Line\nβ".into(),
                ..Default::default()
            },
        )?;
        self.visible.set(cx, true)
    }
    fn normalize(text: String) -> String {
        text
    }
    fn rename(&self, cx: &Cx<'_>, key: &str, text: String) -> Result<()> {
        if text.contains('\0') {
            return Err(Error::Application("NUL rejected".into()));
        }
        let text = Self::normalize(text);
        if !RecordMap::<Row>::new(self.rows).edit(cx, key, |row| {
            row.name = text;
            row.edits = row.edits.checked_add(1).ok_or(Error::Overflow)?;
            Ok(())
        })? {
            return Err(Error::Application("row removed".into()));
        }
        Ok(())
    }
    #[action]
    fn changed(&self, cx: &Cx<'_>, edit: KeyedText) -> Result<()> {
        self.rename(cx, &edit.key, edit.text)
    }
    #[computed(name = "map.text.keys")]
    fn keys(&self, cx: &Cx<'_>) -> Result<Vec<String>> {
        Ok(TextMap::new(self.rows)
            .entries(cx)?
            .into_iter()
            .map(|(key, _)| key)
            .collect())
    }
    #[action]
    fn metadata(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<Row>::new(self.rows).edit_field(
            cx,
            "first\n🦀",
            Row::edits_field(),
            |edits| {
                *edits += 1;
                Ok(())
            },
        )?;
        Ok(())
    }
    #[action]
    fn remove_visible(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<Row>::new(self.rows).remove(cx, "first\n🦀")
    }
    #[action]
    fn prepend(&self, cx: &Cx<'_>) -> Result<()> {
        RecordMap::<Row>::new(self.rows).insert(cx, "aaa", &Row::default())
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let rows = RecordMap::<Row>::new(self.rows);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column {
                    gap: 8.0,
                    padding: 24.0,
                    ..default()
                });
                __blockui_tree.add(Text::new("Collection editor").key("heading"));
                if self.visible.get(cx)? {
                    for key in Self::keys_computed().get(cx)? {
                        {
                            let _scope_1 = __blockui_tree.scope(Column {
                                key: Key::named("map.text.row").scoped_named(&key),
                                gap: 8.0,
                                padding: 0.0,
                            });
                            if key == "second" {
                                __blockui_tree.add(Text::new("Notes").key("second.caption"));
                                __blockui_tree.add(
                                    TextArea::field(
                                        rows.text(key, Row::name_field()),
                                        Self::changed_action(),
                                    )
                                    .label("Second notes"),
                                );
                            } else {
                                __blockui_tree.add(
                                    Text::new("Name").key(Key::named("caption").scoped_named(&key)),
                                );
                                __blockui_tree.add(
                                    TextInput::field(
                                        rows.text(&key, Row::name_field()),
                                        Self::changed_action(),
                                    )
                                    .label(
                                        if key == "first\n🦀" {
                                            "First name".to_owned()
                                        } else {
                                            format!("Name {key}")
                                        },
                                    ),
                                );
                            }
                        };
                    }
                    __blockui_tree.add(
                        Text::new(rows.text("first\n🦀", Row::name_field())).key("first.preview"),
                    );
                }
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn direct_collection_editors_keep_field_dependencies_and_rollback_missing_rows()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    actor.dispatch(blockui_runtime::Message {
        action: Model::<DescriptorMode>::initialize_action().id,
        value: 0,
    })?;
    let input = actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.kind == abi::ENTRY)
        .unwrap()
        .0
        .to_owned();
    let textarea = actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.node.kind == abi::TEXTAREA)
        .unwrap()
        .0
        .to_owned();
    assert_ne!(input, textarea);
    assert_eq!(
        actor.view.bindings[&input].semantic_label.as_deref(),
        Some("First name")
    );
    assert_eq!(actor.view.bindings[&textarea].text, "Line\nβ");
    assert!(
        actor
            .view
            .root_dependencies
            .contains(&blockui_runtime::Dependency::Computed(abi::key(
                "map.text.keys"
            )))
    );
    assert!(
        !actor
            .view
            .root_dependencies
            .contains(&blockui_runtime::Dependency::State(abi::key(
                "map.text.rows"
            )))
    );
    let evaluations = actor.view.bindings[&input].evaluations;
    actor.dispatch(blockui_runtime::Message {
        action: Model::<DescriptorMode>::metadata_action().id,
        value: 0,
    })?;
    actor.dispatch(blockui_runtime::Message {
        action: Model::<DescriptorMode>::prepend_action().id,
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&input].evaluations, evaluations);
    let root_dependencies = actor.view.root_dependencies.clone();
    let area = actor.view.bindings[&textarea].evaluations;
    actor.edit_text(input, "A🦀bc")?;
    assert_eq!(actor.view.root_dependencies, root_dependencies);
    assert_eq!(
        actor.view.bindings[&abi::key("first.preview")].text,
        "A🦀bc"
    );
    assert_eq!(actor.view.bindings[&textarea].evaluations, area);
    let buffer = actor.text_list("map.text.rows").as_ptr();
    assert!(actor.edit_text(input, "bad\0text").is_err());
    assert!(
        actor
            .edit_text(input, &"x".repeat(abi::MAX_MESSAGE_TEXT_BYTES))
            .is_err()
    );
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: Model::<DescriptorMode>::remove_visible_action().id,
                value: 0
            })
            .is_err()
    );
    assert_eq!(actor.text_list("map.text.rows").as_ptr(), buffer);
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text_list("map.text.rows").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&input].text, "A🦀bc");
    assert_eq!(actor.view.bindings[&input].node.slot, 0);
    actor.edit_text(textarea, "Changed\n🦀")?;
    assert_eq!(actor.view.bindings[&textarea].text, "Changed\n🦀");
    Ok(())
}

#[test]
fn shared_action_routes_new_dynamic_rows_and_queued_typed_messages()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    actor.dispatch(blockui_runtime::Message {
        action: Model::<DescriptorMode>::initialize_action().id,
        value: 0,
    })?;
    actor.dispatch(blockui_runtime::Message {
        action: Model::<DescriptorMode>::prepend_action().id,
        value: 0,
    })?;
    let input = *actor
        .view
        .bindings
        .iter()
        .find(|(_, binding)| binding.semantic_label.as_deref() == Some("Name aaa"))
        .unwrap()
        .0;
    assert_eq!(
        actor.view.bindings[&input].node.action,
        Model::<DescriptorMode>::changed_action().spec().id
    );
    actor.edit_text(input, "Dynamic")?;
    assert_eq!(actor.view.bindings[&input].text, "Dynamic");
    actor.tell(
        Model::<DescriptorMode>::changed_action(),
        KeyedText {
            key: "first\n🦀".into(),
            text: "Queued\nβ".into(),
        },
    )?;
    actor.replace_for_test::<Model>()?;
    actor.tick()?;
    assert_eq!(
        actor.view.bindings[&abi::key("first.preview")].text,
        "Queued\nβ"
    );
    let buffer = actor.text_list("map.text.rows").as_ptr();
    actor.tell(
        Model::<DescriptorMode>::changed_action(),
        KeyedText {
            key: "missing".into(),
            text: "Stale".into(),
        },
    )?;
    assert!(actor.tick().is_err());
    assert_eq!(actor.text_list("map.text.rows").as_ptr(), buffer);
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: Model::<DescriptorMode>::changed_action().spec().id,
                value: 0
            })
            .is_err()
    );
    Ok(())
}
