#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::{Actor as Host, Message};
const TEXT: State<String> = State::<String>::with_default("badge.text", "Draft");
const UPDATE: Action = Action::new("badge.update", |cx, _| TEXT.set(cx, "Published 🦀"));
struct Fixture;
impl Actor for Fixture {
    const STATE: &'static [StateSpec] = &[TEXT.spec()];
    const ACTIONS: &'static [Action] = &[UPDATE];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().key("root").children(vec![
            Badge::new(TEXT)
                .key("badge")
                .variant(BadgeVariant::Outline)
                .into_view(),
            Separator::horizontal().key("horizontal").into_view(),
            Separator::vertical(24.0).key("vertical").into_view(),
            Text::new("Unchanged").key("other").into_view(),
        ]))
    }
}
#[test]
fn badge_text_updates_selectively_and_is_not_actionable() -> std::result::Result<(), String> {
    let mut a = Host::for_test::<Fixture>()?;
    let badge = abi::key("badge");
    let other = abi::key("other");
    let unchanged = a.projected_view().bindings[&other].evaluations;
    let evaluated = a.projected_view().bindings[&badge].evaluations;
    assert!(a.dispatch_widget(badge).is_err());
    a.dispatch(Message {
        action: UPDATE.encode(0).0,
        value: 0,
    })?;
    let view = a.projected_view();
    assert_eq!(view.bindings[&badge].text, "Published 🦀");
    assert_eq!(view.bindings[&badge].evaluations, evaluated + 1);
    assert_eq!(view.bindings[&other].evaluations, unchanged);
    assert_eq!(view.bindings[&badge].node.action, 0);
    assert_eq!(
        view.bindings[&abi::key("horizontal")].node.semantics.role,
        abi::semantics::HORIZONTAL_SEPARATOR
    );
    assert_eq!(
        view.bindings[&abi::key("vertical")].node.semantics.role,
        abi::semantics::VERTICAL_SEPARATOR
    );
    Ok(())
}
