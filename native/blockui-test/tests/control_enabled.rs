use blockui::*;
use blockui_test::{Role, Screen};

const ACTIVE: State<bool> = State::<bool>::new("enabled.active", false);
const CHECKED: State<bool> = State::<bool>::new("enabled.checked", false);
const DRAFT: State<String> = State::<String>::new("enabled.draft");
const CALLS: State<i64> = State::<i64>::new("enabled.calls", 0);
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] =
        &[ACTIVE.spec(), CHECKED.spec(), DRAFT.spec(), CALLS.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let active = ACTIVE.get(cx)?;
        Ok(Column::default().children(vec![
            Field::new(
                "Draft",
                TextInput::new(DRAFT)
                    .on_input(|cx, _| CALLS.update(cx, |n| Ok(n + 1)))
                    .key("enabled.input")
                    .enabled(!active)
                    .enabled(active),
            )
            .into_view(),
            Field::new(
                "Always disabled",
                TextInput {
                    enabled: false,
                    ..TextInput::new(DRAFT)
                }
                .on_input(|cx, _| CALLS.update(cx, |n| Ok(n + 1)))
                .key("enabled.original"),
            )
            .into_view(),
            Checkbox::with_handler("Checked", CHECKED, |cx, value| {
                CHECKED.set(cx, value)?;
                CALLS.update(cx, |n| Ok(n + 1))
            })
            .enabled(active)
            .key("enabled.checkbox")
            .into_view(),
            Collapsible::new("Checked details", CHECKED).children(cx, Text::new("Details"))?,
            Button::label("Toggle enabled")
                .key("enabled.toggle")
                .on_click(|cx| ACTIVE.update(cx, |v| Ok(!v))),
        ]))
    }
}

#[test]
fn callback_controls_preserve_enabled_overrides_in_fields_and_after_replacement()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<App>()?;
    let input = screen.get_by_role(Role::Textbox, "Draft")?;
    let checkbox = screen.get_by_key("enabled.checkbox")?;
    assert!(screen.disabled(input)?);
    assert!(screen.disabled(checkbox)?);
    let evaluations = screen.root_evaluations();
    let tree = screen.debug_tree();
    let line = tree
        .lines()
        .find(|line| line.contains("Checkbox "))
        .unwrap();
    assert!(
        line.contains("disabled=true") && line.contains("checked=false"),
        "{tree}"
    );
    assert!(
        tree.lines()
            .any(|line| line.contains("Button \"Checked details\"")
                && line.contains("expanded=false")),
        "{tree}"
    );
    assert_eq!(screen.root_evaluations(), evaluations);
    let error = screen.input(input, "blocked").unwrap_err();
    assert!(
        error.contains("input failed") && error.contains("Textbox") && error.contains("Draft"),
        "{error}"
    );
    assert!(
        error.contains(&format!("key {:#018x}", Key::named("enabled.input").0)),
        "{error}"
    );
    for error in [
        screen.input(checkbox, "blocked").unwrap_err(),
        screen.select(checkbox, 0).unwrap_err(),
    ] {
        assert!(
            error.contains("Checkbox") && error.contains("Checked"),
            "{error}"
        );
    }
    for (operation, error) in [
        ("click", screen.click(input).unwrap_err()),
        ("context_menu", screen.context_menu(input).unwrap_err()),
        ("press", screen.press(input, "Enter").unwrap_err()),
    ] {
        assert!(
            error.contains(&format!("{operation} failed for Textbox \"Draft\"")),
            "{error}"
        );
    }
    screen.click(checkbox)?;
    assert_eq!(screen.actor().integer("enabled.calls"), 0);
    assert_eq!(screen.actor().text("enabled.draft"), "");
    screen.click(screen.get_by_role(Role::Button, "Toggle enabled")?)?;
    let original = screen.get_by_role(Role::Textbox, "Always disabled")?;
    assert!(screen.disabled(original)?);
    assert!(screen.input(original, "still blocked").is_err());
    assert!(!screen.disabled(input)?);
    assert!(!screen.disabled(checkbox)?);
    screen.input(input, "accepted")?;
    screen.click(checkbox)?;
    assert_eq!(screen.actor().integer("enabled.calls"), 2);
    let evaluations = screen.evaluations(checkbox)?;
    let tree = screen.debug_tree();
    let line = tree
        .lines()
        .find(|line| line.contains("Checkbox "))
        .unwrap();
    assert!(
        line.contains("checked=true") && !line.contains("disabled=true"),
        "{tree}"
    );
    assert!(
        tree.lines()
            .any(|line| line.contains("Button \"Checked details\"")
                && line.contains("expanded=true")),
        "{tree}"
    );
    assert_eq!(screen.evaluations(checkbox)?, evaluations);
    screen.replace::<App>()?;
    screen.input(input, "new implementation")?;
    assert_eq!(screen.actor().integer("enabled.calls"), 3);
    screen.click(screen.get_by_role(Role::Button, "Toggle enabled")?)?;
    assert!(screen.disabled(input)?);
    screen.click(checkbox)?;
    assert_eq!(screen.actor().integer("enabled.calls"), 3);
    Ok(())
}
