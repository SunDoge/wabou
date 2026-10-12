#![cfg(feature = "testing")]
use blockui::*;

#[derive(Record, Default)]
#[record(name = "slider.fields.Settings")]
struct Settings {
    volume: i64,
    other: bool,
}
#[actor(name = "slider.fields")]
struct Model {
    settings: State<Settings>,
}
#[actor_impl]
impl Model {
    #[action(name = "slider.fields.change")]
    fn change(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
        Settings::volume_field().set(self.settings, cx, value)?;
        if value == 80 {
            return Err(Error::Application("rejected volume".into()));
        }
        Ok(())
    }
    #[action(name = "slider.fields.metadata")]
    fn metadata(&self, cx: &Cx<'_>, _: i64) -> Result<()> {
        Settings::other_field().update(self.settings, cx, |value| {
            *value = !*value;
            Ok(())
        })
    }
    fn view(&self, _: &Cx<'_>) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Slider::field(
                        self.settings.integer(Settings::volume_field()),
                        IntegerRange {
                            min: 0,
                            max: 100,
                            step: 10,
                        },
                        Self::change_action(),
                    )
                    .label("Volume")
                    .key("volume"),
                );
                __blockui_tree
                    .add(Text::new(self.settings.display(Settings::volume_field())).key("value"));
                __blockui_tree
                    .add(Text::new(self.settings.display(Settings::other_field())).key("other"));
            }
            __blockui_tree.finish()
        }
    }
}
#[actor(name = "slider.legacy")]
struct Legacy {
    volume: State<i64>,
}
#[actor_impl]
impl Legacy {
    #[action(name = "slider.legacy.change")]
    fn change(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
        self.volume.set(cx, value)
    }
    fn view(&self, _: &Cx<'_>) -> View {
        {
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Slider::new(
                        self.volume,
                        IntegerRange {
                            min: 0,
                            max: 100,
                            step: 10,
                        },
                        Self::change_action(),
                    )
                    .key("legacy.volume"),
                );
            }
            __blockui_tree.finish()
        }
    }
}
#[test]
fn record_slider_tracks_field_and_enforces_range_with_atomic_rollback()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
    let slider = abi::key("volume");
    assert_eq!(actor.integer_control_value(slider), Some(0));
    assert_eq!(actor.view.bindings[&slider].node.slot, 0);
    actor.edit_slider(slider, 40)?;
    assert_eq!(actor.integer_control_value(slider), Some(40));
    assert_eq!(actor.view.bindings[&abi::key("value")].text, "40");
    let evaluations = actor.view.bindings[&slider].evaluations;
    actor.dispatch(blockui_runtime::Message {
        action: abi::key("slider.fields.metadata"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&slider].evaluations, evaluations);
    let encoded = actor.text("slider.fields.settings").to_owned();
    let buffer = actor.text("slider.fields.settings").as_ptr();
    for value in [-10, 5, 110, 80] {
        assert!(actor.edit_slider(slider, value).is_err());
        assert_eq!(actor.text("slider.fields.settings"), encoded);
        assert_eq!(actor.text("slider.fields.settings").as_ptr(), buffer);
        assert_eq!(actor.integer_control_value(slider), Some(40));
    }
    for value in [5, 120] {
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: abi::key("slider.fields.change"),
                    value
                })
                .is_err()
        );
        assert_eq!(actor.text("slider.fields.settings").as_ptr(), buffer);
        assert_eq!(actor.view.bindings[&abi::key("value")].text, "40");
    }
    actor.replace_for_test::<Model>()?;
    assert_eq!(actor.text("slider.fields.settings").as_ptr(), buffer);
    actor.edit_slider(slider, 60)?;
    assert_eq!(actor.integer_control_value(slider), Some(60));
    assert_eq!(actor.integer_control_value(abi::key("value")), None);
    Ok(())
}

#[test]
fn retained_state_slider_rejects_programmatic_out_of_range_writes()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Legacy>()?;
    actor.edit_slider(abi::key("legacy.volume"), 40)?;
    let slot = actor.slot_address("slider.legacy.volume");
    for value in [5, -10, 120] {
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: abi::key("slider.legacy.change"),
                    value
                })
                .is_err()
        );
        assert_eq!(
            actor.integer_control_value(abi::key("legacy.volume")),
            Some(40)
        );
        assert_eq!(actor.slot_address("slider.legacy.volume"), slot);
    }
    Ok(())
}
