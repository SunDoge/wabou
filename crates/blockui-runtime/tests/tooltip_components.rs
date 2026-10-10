#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::{Actor as Host, Message};
const HINT: State<String> = State::<String>::with_default("hint.text", "Invite a teammate");
const COUNT: State<i64> = State::<i64>::new("hint.count", 0);
const UPDATE: Action = Action::new("hint.update", |cx, _| HINT.set(cx, "邀请 🦀"));
struct Fixture;
impl Actor for Fixture {
    const STATE: &'static [StateSpec] = &[HINT.spec(), COUNT.spec()];
    const ACTIONS: &'static [Action] = &[UPDATE];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().key("root").children(vec![
            Tooltip::new(HINT)
                .key("invite.hint")
                .delay_ms(400)
                .trigger(
                    Button::label("Invite")
                        .key("invite")
                        .on_click(|cx| COUNT.update(cx, |n| Ok(n + 1))),
                )?,
            Text::new("Unchanged").key("other").into_view(),
        ]))
    }
}
#[test]
fn text_is_selective_and_control_keeps_action_and_description() -> std::result::Result<(), String> {
    let mut a = Host::for_test::<Fixture>()?;
    let trigger = abi::key("invite");
    let hint = Key::named("invite.hint").scoped(abi::key("hint")).0;
    let other = abi::key("other");
    let view = a.projected_view();
    assert_eq!(view.bindings[&trigger].node.semantics.described_by, hint);
    assert_eq!(view.bindings[&hint].node.tooltip_anchor, trigger);
    assert_eq!(view.bindings[&hint].node.tooltip_delay_ms, 400);
    let before = view.bindings[&hint].evaluations;
    let unchanged = view.bindings[&other].evaluations;
    assert!(a.dispatch_widget(hint).is_err());
    a.dispatch_widget(trigger)?;
    assert_eq!(a.integer("hint.count"), 1);
    a.dispatch(Message {
        action: UPDATE.encode(0).0,
        value: 0,
    })?;
    assert_eq!(a.projected_view().bindings[&hint].text, "邀请 🦀");
    assert_eq!(a.projected_view().bindings[&hint].evaluations, before + 1);
    assert_eq!(a.projected_view().bindings[&other].evaluations, unchanged);
    Ok(())
}
#[test]
fn rejects_interactive_content_misuse_and_unbounded_configuration() {
    assert!(
        Tooltip::new("Hint")
            .trigger(Text::new("Plain text"))
            .is_err()
    );
    assert!(
        Tooltip::new("Hint")
            .delay_ms(10001)
            .trigger(Button::label("Action").on_click(|_| Ok(())))
            .is_err()
    );
    for width in [f32::NAN, f32::INFINITY, 0.0, 641.0] {
        assert!(
            Tooltip::new("Hint")
                .width(width)
                .trigger(Button::label("Action").on_click(|_| Ok(())))
                .is_err()
        );
    }
}
