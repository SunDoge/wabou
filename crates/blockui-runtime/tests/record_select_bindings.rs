#![cfg(feature = "testing")]
use blockui::*;
#[derive(Record, Default)]
#[record(name = "select.fields.Settings")]
struct Settings {
    choice: i64,
    edits: i64,
}
#[actor(name = "select.fields")]
struct Model {
    settings: State<Settings>,
}
#[actor_impl]
impl Model {
    #[action(name = "select.fields.change")]
    fn change(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
        Settings::choice_field().set(self.settings, cx, value)?;
        Settings::edits_field().update(self.settings, cx, |edits| {
            *edits = edits.checked_add(1).ok_or(Error::Overflow)?;
            Ok(())
        })?;
        if value == 4 {
            return Err(Error::Application("rejected choice".into()));
        }
        Ok(())
    }
    #[action(name = "select.fields.metadata")]
    fn metadata(&self, cx: &Cx<'_>, _: i64) -> Result<()> {
        Settings::edits_field().update(self.settings, cx, |edits| {
            *edits = edits.checked_add(1).ok_or(Error::Overflow)?;
            Ok(())
        })
    }
    fn view(&self, _: &Cx<'_>) -> View {
        let options = [
            (0, "Automatic", true),
            (1, "Light", true),
            (2, "Dark", true),
            (3, "Disabled", false),
            (4, "Restricted", true),
        ]
        .into_iter()
        .map(|(value, label, enabled)| SelectOption {
            value,
            label: label.into(),
            enabled,
        })
        .collect();
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Select::field(
                        "Theme",
                        self.settings.integer(Settings::choice_field()),
                        options,
                        Self::change_action(),
                    )
                    .key("choice"),
                );
                __blockui_tree
                    .add(Text::new(self.settings.display(Settings::choice_field())).key("value"));
                __blockui_tree
                    .add(Text::new(self.settings.display(Settings::edits_field())).key("edits"));
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn record_selection_validates_options_and_restores_rejected_writes()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let select = abi::key("choice");
    assert_eq!(actor.view.bindings[&select].node.slot, 0);
    assert_eq!(
        actor.view.bindings[&select].semantic_label.as_deref(),
        Some("Theme")
    );
    actor.edit_select(select, 2)?;
    assert_eq!(actor.integer_control_value(select), Some(2));
    let evaluations = actor.view.bindings[&select].evaluations;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("select.fields.metadata"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&select].evaluations, evaluations);
    let encoded = actor.text("select.fields.settings").to_owned();
    let buffer = actor.text("select.fields.settings").as_ptr();
    for value in [3, 4, 99] {
        assert!(actor.edit_select(select, value).is_err());
        assert_eq!(actor.text("select.fields.settings"), encoded);
        assert_eq!(actor.text("select.fields.settings").as_ptr(), buffer);
        assert_eq!(actor.integer_control_value(select), Some(2));
    }
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: abi::key("select.fields.change"),
                value: 99
            })
            .is_err()
    );
    assert_eq!(actor.text("select.fields.settings").as_ptr(), buffer);
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text("select.fields.settings").as_ptr(), buffer);
    actor.edit_select(select, 1)?;
    assert_eq!(actor.integer_control_value(select), Some(1));
    Ok(())
}
