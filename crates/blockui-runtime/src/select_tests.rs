use crate::*;
use blockui::{Action, Column, Cx, IntoView, Select, SelectOption, State, View, actor, actor_impl};
fn options() -> Vec<SelectOption> {
    [10, 20, 30, 40]
        .into_iter()
        .map(|value| SelectOption {
            value,
            label: format!("航程 {value} 🦀"),
            enabled: value != 20,
        })
        .collect()
}
#[actor(name = "choice")]
pub(crate) struct Choices {
    #[state(default = 10)]
    value: State<i64>,
}
#[actor_impl]
impl Choices {
    #[action(name = "choose")]
    fn choose(&self, cx: &Cx<'_>, value: i64) -> blockui::Result<()> {
        self.value.set(cx, value)?;
        if value == 30 {
            return Err(blockui::Error::Application("choice rejected".into()));
        }
        Ok(())
    }
    fn view(&self, _cx: &Cx<'_>) -> View {
        Column::default()
            .children(vec![
                Select::new("Flight type", self.value, options(), Self::choose_action())
                    .key("choice.input")
                    .into_view(),
            ])
            .into_view()
    }
}
#[cfg(feature = "native-window")]
#[actor(name = "choice")]
pub(crate) struct RevisedChoices {
    #[state(default = 10)]
    value: State<i64>,
}
#[cfg(feature = "native-window")]
#[actor_impl]
impl RevisedChoices {
    #[action(name = "choose")]
    fn choose(&self, cx: &Cx<'_>, value: i64) -> blockui::Result<()> {
        self.value.set(cx, value)
    }
    fn view(&self, _cx: &Cx<'_>) -> View {
        Column::default()
            .children(vec![
                Select::new(
                    "Revised flight type",
                    self.value,
                    [40, 10, 30, 20]
                        .into_iter()
                        .map(|value| SelectOption {
                            value,
                            label: format!("Updated {value}"),
                            enabled: value != 30,
                        })
                        .collect(),
                    Self::choose_action(),
                )
                .key("choice.input")
                .into_view(),
            ])
            .into_view()
    }
}
#[actor(name = "choice")]
struct Malformed {
    #[state(default = 10)]
    value: State<i64>,
}
#[actor_impl]
impl Malformed {
    fn view(&self, _cx: &Cx<'_>) -> View {
        Column::default()
            .children(vec![
                Select::new(
                    "Flight type",
                    self.value,
                    vec![],
                    Action::new("unused", |_, _| Ok(())),
                )
                .key("choice.input")
                .into_view(),
            ])
            .into_view()
    }
}
#[test]
fn controlled_selection_validates_current_options_and_rolls_back_rejection()
-> std::result::Result<(), String> {
    let mut actor = Actor::for_test::<Choices>()?;
    let widget = key("choice.input");
    assert_eq!(
        actor.view.bindings[&widget].semantic_label.as_deref(),
        Some("Flight type")
    );
    for value in [20, 99, 30] {
        assert!(actor.edit_select(widget, value).is_err());
        assert_eq!(actor.integer("choice.value"), 10);
    }
    actor.edit_select(widget, 40)?;
    assert_eq!(actor.integer("choice.value"), 40);
    let slot = actor.slot_address("choice.value");
    actor.replace_for_test::<Choices>()?;
    assert_eq!(actor.slot_address("choice.value"), slot);
    assert_eq!(actor.integer("choice.value"), 40);
    actor.edit_select(widget, 10)?;
    assert_eq!(actor.integer("choice.value"), 10);
    Ok(())
}
#[test]
fn malformed_select_replacement_keeps_old_actor_usable() -> std::result::Result<(), String> {
    let mut actor = Actor::for_test::<Choices>()?;
    let version = actor.implementation_version();
    assert!(actor.replace_for_test::<Malformed>().is_err());
    assert_eq!(actor.implementation_version(), version);
    actor.edit_select(key("choice.input"), 40)?;
    assert_eq!(actor.integer("choice.value"), 40);
    Ok(())
}
