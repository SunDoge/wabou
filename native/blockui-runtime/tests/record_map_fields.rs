#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record, Default)]
#[record(name = "map.fields.row")]
struct Row {
    name: String,
    metadata: bool,
}
#[actor(name = "map.fields")]
struct Model {
    #[state(codec = Row)]
    rows: State<Vec<String>>,
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
                name: "First".into(),
                ..Default::default()
            },
        )?;
        rows.insert(
            cx,
            "second",
            &Row {
                name: "Second".into(),
                ..Default::default()
            },
        )
    }
    #[action]
    fn change(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        let rows = RecordMap::<Row>::new(self.rows);
        match mode {
            0 => {
                rows.edit_field(cx, "first\n🦀", Row::metadata_field(), |value| {
                    *value = !*value;
                    Ok(())
                })?;
            }
            1 | 2 => {
                rows.edit_field(cx, "first\n🦀", Row::name_field(), |value| {
                    *value = "Changed".into();
                    Ok(())
                })?;
            }
            3 => {
                rows.edit_field(cx, "second", Row::name_field(), |value| {
                    *value = "Other".into();
                    Ok(())
                })?;
            }
            4 => rows.remove(cx, "first\n🦀")?,
            5 => rows.insert(
                cx,
                "missing",
                &Row {
                    name: "Inserted".into(),
                    ..Default::default()
                },
            )?,
            6 => {
                assert!(!rows.edit_field(cx, "absent", Row::name_field(), |_| Err(
                    Error::Application("must not run".into())
                ))?);
            }
            7 => {
                rows.edit_field(cx, "first\n🦀", Row::name_field(), |value| {
                    rows.remove(cx, "first\n🦀")?;
                    *value = "Stale".into();
                    Ok(())
                })?;
            }
            _ => return Err(Error::Application("invalid mode".into())),
        }
        if mode == 2 {
            return Err(Error::Application("rollback field".into()));
        }
        Ok(())
    }
    #[computed(name = "map.fields.name")]
    fn name(&self, cx: &Cx<'_>) -> Result<String> {
        Ok(RecordMap::<Row>::new(self.rows)
            .field(cx, "first\n🦀", Row::name_field())?
            .unwrap_or_else(|| "Absent".into()))
    }
    #[computed(name = "map.fields.metadata")]
    fn metadata(&self, cx: &Cx<'_>) -> Result<bool> {
        Ok(RecordMap::<Row>::new(self.rows)
            .field(cx, "first\n🦀", Row::metadata_field())?
            .unwrap_or(false))
    }
    #[computed(name = "map.fields.missing")]
    fn missing(&self, cx: &Cx<'_>) -> Result<String> {
        Ok(RecordMap::<Row>::new(self.rows)
            .field(cx, "missing", Row::name_field())?
            .unwrap_or_else(|| "Absent".into()))
    }
    #[computed(name = "map.fields.whole")]
    fn whole(&self, cx: &Cx<'_>) -> Result<String> {
        Ok(RecordMap::<Row>::new(self.rows)
            .get(cx, "first\n🦀")?
            .map(|row| format!("{}:{}", row.name, row.metadata))
            .unwrap_or_else(|| "Absent".into()))
    }
    fn view(&self, _: &Cx<'_>) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(Self::name_computed()).key("name"));
                __blockui_tree.add(Text::new(Self::metadata_computed()).key("metadata"));
                __blockui_tree.add(Text::new(Self::missing_computed()).key("missing"));
                __blockui_tree.add(Text::new(Self::whole_computed()).key("whole"));
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn collection_fields_skip_siblings_track_absence_and_rollback_conflicts()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let message = |mode| blockui_runtime::Message {
        action: Model::<DescriptorMode>::change_action().id,
        value: mode,
    };
    actor.dispatch(blockui_runtime::Message {
        action: Model::<DescriptorMode>::initialize_action().id,
        value: 0,
    })?;
    let name = actor.computed_evaluations("map.fields.name");
    let metadata = actor.computed_evaluations("map.fields.metadata");
    let whole = actor.computed_evaluations("map.fields.whole");
    actor.dispatch(message(0))?;
    assert_eq!(actor.computed_evaluations("map.fields.name"), name);
    assert!(actor.computed_evaluations("map.fields.metadata") > metadata);
    assert!(actor.computed_evaluations("map.fields.whole") > whole);
    let metadata = actor.computed_evaluations("map.fields.metadata");
    actor.dispatch(message(1))?;
    assert_eq!(actor.computed_evaluations("map.fields.metadata"), metadata);
    assert_eq!(actor.view.bindings[&abi::key("name")].text, "Changed");
    let name = actor.computed_evaluations("map.fields.name");
    actor.dispatch(message(3))?;
    assert_eq!(actor.computed_evaluations("map.fields.name"), name);
    let buffer = actor.text_list("map.fields.rows").as_ptr();
    assert!(actor.dispatch(message(2)).is_err());
    assert!(actor.dispatch(message(7)).is_err());
    assert_eq!(actor.text_list("map.fields.rows").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&abi::key("name")].text, "Changed");
    actor.dispatch(message(6))?;
    actor.dispatch(message(5))?;
    assert_eq!(actor.view.bindings[&abi::key("missing")].text, "Inserted");
    actor.dispatch(message(4))?;
    assert_eq!(actor.view.bindings[&abi::key("name")].text, "Absent");
    actor.dispatch(blockui_runtime::Message {
        action: Model::<DescriptorMode>::initialize_action().id,
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&abi::key("name")].text, "First");
    Ok(())
}
