use blockui::*;
use blockui_test::{Role, Screen};
const COUNT: State<i64> = State::<i64>::new("root.count", 0);
const INCREMENT: Action = Action::new("root.increment", |cx, _| COUNT.update(cx, |n| Ok(n + 1)));
fn tree(value: View) -> View {
    Column::default().children(vec![value, Button::new("Increment", INCREMENT).into_view()])
}
struct Root;
impl Actor for Root {
    const STATE: &'static [StateSpec] = &[COUNT.spec()];
    const ACTIONS: &'static [Action] = &[INCREMENT];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let count = COUNT.get(cx)?;
        if count == 2 {
            return Err(Error::Application("reject root".into()));
        }
        Ok(tree(Text::new(count.to_string()).key("value").into_view()))
    }
}
struct Reactive;
impl Actor for Reactive {
    const STATE: &'static [StateSpec] = Root::STATE;
    const ACTIONS: &'static [Action] = Root::ACTIONS;
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(tree(
            Text::reactive(|cx| Ok(COUNT.get(cx)?.to_string()))
                .key("value")
                .into_view(),
        ))
    }
}
const PANEL: Scope = Scope::new("root.panel", |cx| {
    Ok(Text::new(COUNT.get(cx)?.to_string())
        .key("value")
        .into_view())
});
struct Scoped;
impl Actor for Scoped {
    const STATE: &'static [StateSpec] = Root::STATE;
    const ACTIONS: &'static [Action] = Root::ACTIONS;
    const SCOPES: &'static [Scope] = &[PANEL];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(tree(PANEL.into_view()))
    }
}
#[test]
fn root_count_tracks_committed_builds_and_rejects_failed_projection()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<Root>()?;
    assert_eq!(screen.root_evaluations(), 1);
    let button = screen.get_by_role(Role::Button, "Increment")?;
    screen.click(button)?;
    screen.get_by_text("1")?;
    assert_eq!(screen.root_evaluations(), 2);
    assert!(screen.click(button).is_err());
    screen.get_by_text("1")?;
    assert_eq!(screen.root_evaluations(), 2);
    screen.replace::<Root>()?;
    assert_eq!(screen.root_evaluations(), 3);
    assert!(screen.replace::<Rejected>().is_err());
    assert_eq!(screen.root_evaluations(), 3);
    Ok(())
}
struct Rejected;
impl Actor for Rejected {
    const STATE: &'static [StateSpec] = Root::STATE;
    const ACTIONS: &'static [Action] = Root::ACTIONS;
    fn view(_: &Cx<'_>) -> Result<View> {
        Err(Error::Application("reject candidate".into()))
    }
}
fn independent<A: Actor>() -> std::result::Result<(), String> {
    let mut screen = Screen::render::<A>()?;
    let value = screen.get_by_text("0")?;
    assert_eq!(screen.root_evaluations(), 1);
    screen.click(screen.get_by_role(Role::Button, "Increment")?)?;
    assert_eq!(screen.get_by_text("1")?, value);
    assert_eq!(screen.root_evaluations(), 1);
    screen.replace::<A>()?;
    assert_eq!(screen.root_evaluations(), 2);
    Ok(())
}
#[test]
fn text_and_scope_updates_leave_root_count_unchanged() -> std::result::Result<(), String> {
    independent::<Reactive>()?;
    independent::<Scoped>()
}
