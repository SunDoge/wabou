#![cfg(feature = "testing")]
use blockui::*;
#[derive(Record, Default)]
#[record(name = "progress.fields.Settings")]
struct Settings {
    fraction: f64,
    edits: i64,
}
#[actor(name = "progress.fields")]
struct Model {
    settings: State<Settings>,
}
#[actor_impl]
impl Model {
    #[action(name = "progress.fields.change")]
    fn change(&self, cx: &Cx<'_>, value: f64) -> Result<()> {
        Settings::fraction_field().set(self.settings, cx, value)?;
        Settings::edits_field().update(self.settings, cx, |edits| {
            *edits = edits.checked_add(1).ok_or(Error::Overflow)?;
            Ok(())
        })?;
        if value == 0.75 {
            return Err(Error::Application("rejected fraction".into()));
        }
        Ok(())
    }
    #[action(name = "progress.fields.metadata")]
    fn metadata(&self, cx: &Cx<'_>, _: i64) -> Result<()> {
        Settings::edits_field().update(self.settings, cx, |edits| {
            *edits = edits.checked_add(1).ok_or(Error::Overflow)?;
            Ok(())
        })
    }
    fn view(&self, _: &Cx<'_>) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Progress::field(self.settings.float(Settings::fraction_field()))
                        .label("Download")
                        .key("progress"),
                );
                __blockui_tree.add(
                    Text::new(self.settings.display(Settings::fraction_field())).key("fraction"),
                );
                __blockui_tree
                    .add(Text::new(self.settings.display(Settings::edits_field())).key("edits"));
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn progress_field_rejects_invalid_fractions_on_retained_refresh_and_rolls_back()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let msg = |value: f64| blockui_runtime::Message {
        action: abi::key("progress.fields.change"),
        value: value.to_bits() as i64,
    };
    actor.dispatch(msg(0.5))?;
    let progress = abi::key("progress");
    assert_eq!(actor.view.bindings[&progress].node.slot, 0);
    assert_eq!(actor.view.bindings[&progress].text, "0.5");
    assert_eq!(
        actor.view.bindings[&progress].semantic_label.as_deref(),
        Some("Download")
    );
    let evaluations = actor.view.bindings[&progress].evaluations;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("progress.fields.metadata"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&progress].evaluations, evaluations);
    let encoded = actor.text("progress.fields.settings").to_owned();
    let buffer = actor.text("progress.fields.settings").as_ptr();
    for value in [-0.1, 1.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.75] {
        assert!(actor.dispatch(msg(value)).is_err());
        assert_eq!(actor.text("progress.fields.settings"), encoded);
        assert_eq!(actor.text("progress.fields.settings").as_ptr(), buffer);
        assert_eq!(actor.view.bindings[&progress].text, "0.5");
    }
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text("progress.fields.settings").as_ptr(), buffer);
    for value in [0.0, 1.0, -0.0] {
        actor.dispatch(msg(value))?;
        assert_eq!(actor.view.bindings[&progress].text, value.to_string());
    }
    Ok(())
}
