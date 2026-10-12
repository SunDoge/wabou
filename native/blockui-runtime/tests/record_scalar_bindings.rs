#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record, Default)]
#[record(name = "scalar.fields.Values")]
struct Values {
    signed: i64,
    unsigned: u64,
    ratio: f64,
    enabled: bool,
}
#[actor(name = "scalar.fields")]
struct Model {
    values: State<Values>,
}
#[actor_impl]
impl Model {
    #[action(name = "scalar.fields.change")]
    fn change(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        match mode {
            0 => self.values.update(cx, |v| {
                v.signed = i64::MIN;
                v.unsigned = u64::MAX;
                v.ratio = -0.0;
                v.enabled = true;
                Ok(())
            }),
            1 => Values::signed_field().set(self.values, cx, 7),
            2 => {
                Values::signed_field().set(self.values, cx, 99)?;
                Err(Error::Application("reject".into()))
            }
            3 => Values::ratio_field().set(self.values, cx, f64::INFINITY),
            4 => Values::ratio_field().set(self.values, cx, f64::NAN),
            5 => Values::signed_field().set(self.values, cx, i64::MAX),
            _ => Err(Error::Application("unknown".into())),
        }
    }
    #[action(name = "scalar.fields.toggle")]
    fn toggle(&self, cx: &Cx<'_>, value: bool) -> Result<()> {
        Values::enabled_field().set(self.values, cx, value)?;
        Values::signed_field().update(self.values, cx, |count| {
            *count = count.checked_add(1).ok_or(Error::Overflow)?;
            Ok(())
        })
    }
    fn view(&self, _: &Cx<'_>) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree
                    .add(Text::new(self.values.display(Values::signed_field())).key("signed"));
                __blockui_tree
                    .add(Text::new(self.values.display(Values::unsigned_field())).key("unsigned"));
                __blockui_tree
                    .add(Text::new(self.values.display(Values::ratio_field())).key("ratio"));
                __blockui_tree
                    .add(Text::new(self.values.display(Values::enabled_field())).key("enabled"));
                __blockui_tree.add(
                    Checkbox::field(
                        "Enabled",
                        self.values.boolean(Values::enabled_field()),
                        Self::toggle_action(),
                    )
                    .key("toggle"),
                );
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn scalar_fields_format_track_and_rollback() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let msg = |value| blockui_runtime::Message {
        action: abi::key("scalar.fields.change"),
        value,
    };
    assert_eq!(actor.view.bindings[&abi::key("enabled")].text, "false");
    actor.dispatch(msg(0))?;
    for (key, expected) in [
        ("signed", i64::MIN.to_string()),
        ("unsigned", u64::MAX.to_string()),
        ("ratio", "-0".into()),
        ("enabled", "true".into()),
    ] {
        assert_eq!(actor.view.bindings[&abi::key(key)].text, expected);
    }
    let evaluations = actor.view.bindings[&abi::key("ratio")].evaluations;
    actor.dispatch(msg(1))?;
    assert_eq!(actor.view.bindings[&abi::key("signed")].text, "7");
    assert_eq!(
        actor.view.bindings[&abi::key("ratio")].evaluations,
        evaluations
    );
    let buffer = actor.text("scalar.fields.values").as_ptr();
    assert!(actor.dispatch(msg(2)).unwrap_err().contains("reject"));
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    assert_eq!(actor.view.bindings[&abi::key("signed")].text, "7");
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    actor.dispatch(msg(3))?;
    assert_eq!(actor.view.bindings[&abi::key("ratio")].text, "inf");
    actor.dispatch(msg(4))?;
    assert_eq!(actor.view.bindings[&abi::key("ratio")].text, "NaN");
    Ok(())
}

#[test]
fn checkbox_record_source_reads_current_field_and_dispatches_current_action()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let toggle = abi::key("toggle");
    assert_eq!(actor.checkbox_value(toggle), Some(false));
    assert_eq!(actor.view.bindings[&toggle].node.slot, 0);
    assert_eq!(actor.view.bindings[&toggle].text, "☐ Enabled");
    let evaluations = actor.view.bindings[&abi::key("ratio")].evaluations;
    actor.dispatch_widget(toggle)?;
    assert_eq!(actor.checkbox_value(toggle), Some(true));
    assert_eq!(actor.view.bindings[&toggle].text, "☑ Enabled");
    assert_eq!(actor.view.bindings[&abi::key("signed")].text, "1");
    assert_eq!(
        actor.view.bindings[&abi::key("ratio")].evaluations,
        evaluations
    );
    let buffer = actor.text("scalar.fields.values").as_ptr();
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    actor.dispatch_widget(toggle)?;
    assert_eq!(actor.checkbox_value(toggle), Some(false));
    assert_eq!(actor.view.bindings[&abi::key("signed")].text, "2");
    assert_eq!(actor.checkbox_value(abi::key("signed")), None);
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("scalar.fields.change"),
        value: 5,
    })?;
    let encoded = actor.text("scalar.fields.values").to_owned();
    let buffer = actor.text("scalar.fields.values").as_ptr();
    assert!(actor.dispatch_widget(toggle).is_err());
    assert_eq!(actor.text("scalar.fields.values"), encoded);
    assert_eq!(actor.text("scalar.fields.values").as_ptr(), buffer);
    assert_eq!(actor.checkbox_value(toggle), Some(false));
    assert_eq!(actor.view.bindings[&toggle].text, "☐ Enabled");
    Ok(())
}

#[derive(Record, Default)]
#[record(name = "selection.Row")]
struct SelectionRow {
    id: i64,
    selected: bool,
}
#[actor(name = "selection")]
struct SelectionModel {
    rows: State<Vec<SelectionRow>>,
}
#[actor_impl]
impl SelectionModel {
    #[action(name = "selection.seed")]
    fn seed(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
        if mode == 0 {
            self.rows.set(
                cx,
                &vec![
                    SelectionRow {
                        id: 42,
                        selected: false,
                    },
                    SelectionRow {
                        id: 7,
                        selected: true,
                    },
                ],
            )
        } else {
            self.rows.update(cx, |rows| {
                match mode {
                    1 => rows.reverse(),
                    2 => rows[0].id = rows[1].id,
                    _ => return Err(Error::Application("unknown mode".into())),
                }
                Ok(())
            })
        }
    }
    fn view(&self, cx: &Cx<'_>) -> Result<View> {
        let rows = self.rows.get(cx)?;
        let controls = rows
            .iter()
            .map(|row| {
                let id = row.id;
                let state = self.rows;
                Checkbox::field(
                    format!("Row {id}"),
                    SelectionRow::selected_field().boolean_from_list(
                        state,
                        SelectionRow::id_field(),
                        id,
                    ),
                    move |cx: &Cx<'_>, value| {
                        state.update(cx, |rows| {
                            let row = rows
                                .iter_mut()
                                .find(|row| row.id == id)
                                .ok_or_else(|| Error::Application("missing row".into()))?;
                            row.selected = value;
                            Ok(())
                        })
                    },
                )
                .into_view()
                .key(format!("selection.{id}"))
            })
            .collect();
        Ok(Column::default().children(controls))
    }
}
#[test]
fn list_checkbox_tracks_identity_across_reorder_and_replacement() -> std::result::Result<(), String>
{
    let mut actor = blockui_runtime::Actor::for_test::<SelectionModel>()?;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("selection.seed"),
        value: 0,
    })?;
    let key = abi::key("selection.42");
    let other = abi::key("selection.7");
    assert_eq!(actor.checkbox_value(key), Some(false));
    assert_eq!(actor.checkbox_value(other), Some(true));
    actor.dispatch_widget(key)?;
    assert_eq!(actor.checkbox_value(key), Some(true));
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("selection.seed"),
        value: 1,
    })?;
    assert_eq!(actor.checkbox_value(key), Some(true));
    actor.replace_for_test::<SelectionModel>()?;
    actor.dispatch_widget(key)?;
    assert_eq!(actor.checkbox_value(key), Some(false));
    assert_eq!(actor.checkbox_value(other), Some(true));
    let encoded = actor.text("selection.rows").to_owned();
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: abi::key("selection.seed"),
                value: 2,
            })
            .is_err()
    );
    assert_eq!(actor.text("selection.rows"), encoded);
    assert_eq!(actor.checkbox_value(key), Some(false));
    assert_eq!(actor.checkbox_value(other), Some(true));
    Ok(())
}
