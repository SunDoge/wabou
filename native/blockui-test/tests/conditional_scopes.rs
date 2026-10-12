use blockui::*;
use blockui_test::{Role, Screen};
use std::cell::Cell;
const OPEN: State<bool> = State::<bool>::new("when.open", false);
const DRAFT: State<String> = State::<String>::new("when.draft");
const CONDITION_FAIL: State<bool> = State::<bool>::new("when.condition.fail", false);
const FAIL: State<bool> = State::<bool>::new("when.fail", false);
thread_local! { static BUILDS: Cell<usize> = const { Cell::new(0) }; }
struct App<const STEP: i64>;
const COUNT: State<i64> = State::<i64>::new("when.count", 0);
impl<const STEP: i64> Actor for App<STEP> {
    const STATE: &'static [StateSpec] = &[
        OPEN.spec(),
        DRAFT.spec(),
        FAIL.spec(),
        COUNT.spec(),
        CONDITION_FAIL.spec(),
    ];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        let title = String::from("Details");
        Ok(Column::default().children(vec![
            Heading::new("Outside").key("when.heading").into_view(),
            Button::label("Toggle")
                .key("when.toggle")
                .on_click(|cx| OPEN.set(cx, !OPEN.get(cx)?)),
            Button::label("Fail")
                .key("when.fail.button")
                .on_click(|cx| FAIL.set(cx, !FAIL.get(cx)?)),
            Button::label("Fail condition")
                .key("when.condition.fail.button")
                .on_click(|cx| CONDITION_FAIL.set(cx, true)),
            Scope::when(
                "when.details",
                |cx| {
                    if CONDITION_FAIL.get(cx)? {
                        return Err(Error::Application("rejected condition".into()));
                    }
                    OPEN.get(cx)
                },
                move |cx| {
                    BUILDS.with(|count| count.set(count.get() + 1));
                    if FAIL.get(cx)? {
                        return Err(Error::Application("rejected details".into()));
                    }
                    Ok(Card::default().children(vec![
                        Heading::new(title.clone()).into_view(),
                        Field::new("Draft", TextInput::new(DRAFT)).into_view(),
                        Text::new(COUNT).key("count").into_view(),
                        Button::label("Increment")
                            .key("increment")
                            .on_click(|cx| COUNT.update(cx, |value| Ok(value + STEP))),
                    ]))
                },
            ),
        ]))
    }
}
#[test]
fn conditional_scope_is_lazy_local_and_reopens_with_actor_state() -> std::result::Result<(), String>
{
    BUILDS.with(|count| count.set(0));
    let mut screen = Screen::render::<App<1>>()?;
    let heading = screen.get_by_role(Role::Heading, "Outside")?;
    let before = screen.evaluations(heading)?;
    let toggle = screen.get_by_role(Role::Button, "Toggle")?;
    assert_eq!(BUILDS.with(Cell::get), 0);
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Fail condition")?)
            .is_err()
    );
    assert!(!screen.actor().boolean("when.condition.fail"));
    assert_eq!(BUILDS.with(Cell::get), 0);
    assert!(screen.get_by_role(Role::Heading, "Details").is_err());
    screen.click(toggle)?;
    assert_eq!(BUILDS.with(Cell::get), 1);
    let details = screen.scope_of(screen.get_by_role(Role::Heading, "Details")?)?;
    let draft = screen
        .within(details)?
        .get_by_role(Role::Textbox, "Draft")?;
    let increment = screen
        .within(details)?
        .get_by_role(Role::Button, "Increment")?;
    screen.input(draft, "Retained")?;
    screen.click(increment)?;
    assert_eq!(screen.actor().integer("when.count"), 1);
    assert_eq!(
        BUILDS.with(Cell::get),
        1,
        "text/input updates do not rebuild content"
    );
    screen.click(toggle)?;
    assert!(screen.within(details).is_err());
    assert!(screen.click(increment).is_err());
    screen.click(screen.get_by_role(Role::Button, "Fail")?)?;
    assert_eq!(
        BUILDS.with(Cell::get),
        1,
        "closed content unsubscribes its structural reads"
    );
    assert!(screen.click(toggle).is_err());
    assert!(!screen.actor().boolean("when.open"));
    screen.click(screen.get_by_role(Role::Button, "Fail")?)?;
    screen.click(toggle)?;
    assert_eq!(screen.text(draft)?, "Retained");
    assert_eq!(screen.evaluations(heading)?, before);
    screen.replace::<App<10>>()?;
    screen.click(increment)?;
    assert_eq!(screen.actor().integer("when.count"), 11);
    assert_eq!(screen.text(draft)?, "Retained");
    assert!(
        screen
            .click(screen.get_by_role(Role::Button, "Fail")?)
            .is_err()
    );
    assert!(!screen.actor().boolean("when.fail"));
    screen.click(increment)?;
    assert_eq!(screen.actor().integer("when.count"), 21);
    Ok(())
}
