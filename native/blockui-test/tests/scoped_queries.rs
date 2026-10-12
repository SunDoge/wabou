use blockui::*;
use blockui_test::{Role, Screen};
use std::cell::Cell;
const LEFT: State<i64> = State::<i64>::new("test.left", 0);
const RIGHT: State<i64> = State::<i64>::new("test.right", 0);
const OPEN: State<bool> = State::<bool>::new("test.open", false);
const DRAFT: State<String> = State::<String>::new("test.draft");
thread_local! { static BUILDS: Cell<[usize; 3]> = const { Cell::new([0; 3]) }; }
fn built(index: usize) {
    BUILDS.with(|counts| {
        let mut n = counts.get();
        n[index] += 1;
        counts.set(n);
    });
}
fn counter<const NEW: bool>(name: &'static str, value: State<i64>, index: usize) -> View {
    let title = String::from(name);
    Scope::reactive(name, move |cx| {
        built(index);
        let mut children = vec![
            Heading::new(title.clone()).key("heading").into_view(),
            Text::reactive(move |cx| Ok(format!("Count: {}", value.get(cx)?)))
                .key("value")
                .into_view(),
            Button::label("Increment")
                .key("increment")
                .on_click(move |cx| value.update(cx, |n| Ok(n + if NEW { 10 } else { 1 })))
                .into_view(),
            Field::new("Draft", TextInput::new(DRAFT).key("draft")).into_view(),
        ];
        if index == 1 {
            children.push(
                Button::label("Details")
                    .key("toggle")
                    .on_click(|cx| OPEN.set(cx, !OPEN.get(cx)?))
                    .into_view(),
            );
            if OPEN.get(cx)? {
                children.push(Scope::reactive("details", |_| {
                    Ok(Column::default().children(vec![
                        Text::new("Nested details").into_view(),
                        Button::label("Increment")
                            .key("increment")
                            .on_click(|cx| LEFT.update(cx, |n| Ok(n + 2)))
                            .into_view(),
                    ]))
                }));
            }
        }
        Ok(Column::default().children(children))
    })
}
struct App<const NEW: bool>;
impl<const NEW: bool> Actor for App<NEW> {
    const STATE: &'static [StateSpec] = &[LEFT.spec(), RIGHT.spec(), OPEN.spec(), DRAFT.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        built(0);
        Ok(Row::default().children(vec![
            counter::<NEW>("Left", LEFT, 1),
            counter::<NEW>("Right", RIGHT, 2),
        ]))
    }
}
#[test]
fn repeated_local_scopes_are_queryable_without_derived_identity_math()
-> std::result::Result<(), String> {
    BUILDS.with(|n| n.set([0; 3]));
    let mut screen = Screen::render::<App<false>>()?;
    let ambiguous = screen.get_by_role(Role::Button, "Increment").unwrap_err();
    assert!(
        ambiguous.contains("found 2; visible candidates:"),
        "{ambiguous}"
    );
    assert_eq!(ambiguous.matches("key 0x").count(), 2);
    assert!(ambiguous.starts_with("in screen: "), "{ambiguous}");

    assert!(screen.get_by_text("Count: 0").is_err());
    let left = screen.scope_of(screen.get_by_role(Role::Heading, "Left")?)?;
    let right = screen.get_by_key("Right")?;
    let tree = screen.within(right)?.debug_tree();
    assert!(tree.contains("Heading \"Right\""), "{tree}");
    assert!(tree.contains("Count: 0"), "{tree}");
    assert!(tree.contains("text=1"), "{tree}");
    assert!(!tree.contains("Left"), "{tree}");
    assert!(!tree.contains("Details"), "{tree}");
    let missing = screen
        .within(right)?
        .get_by_role(Role::Button, "Missing")
        .unwrap_err();
    assert!(missing.contains("Increment"), "{missing}");
    let context = format!("in subtree {:#018x}", Key::named("Right").0);
    assert!(missing.starts_with(&context), "{missing}");
    let text_missing = screen.within(right)?.get_by_text("Missing").unwrap_err();
    assert!(text_missing.starts_with(&context), "{text_missing}");
    assert!(!text_missing.contains("Left"), "{text_missing}");
    assert!(!missing.contains("Details"), "{missing}");
    assert_eq!(
        BUILDS.with(Cell::get),
        [1, 1, 1],
        "diagnostics do not evaluate scopes"
    );
    let increment = screen
        .within(left)?
        .get_by_role(Role::Button, "Increment")?;
    let draft = screen.within(left)?.get_by_role(Role::Textbox, "Draft")?;
    assert!(
        screen
            .within(left)?
            .query_all_by_role(Role::Heading, "Right")
            .is_empty()
    );
    assert_eq!(
        screen
            .within(increment)?
            .get_by_role(Role::Button, "Increment")?,
        increment,
        "subtree queries include the root"
    );
    screen.input(draft, "Retained draft")?;
    screen.click(increment)?;
    screen.within(left)?.get_by_text("Count: 1")?;
    screen.within(right)?.get_by_text("Count: 0")?;
    assert_eq!(
        BUILDS.with(Cell::get),
        [1, 1, 1],
        "reactive text does not rebuild either scope"
    );
    let toggle = screen.within(left)?.get_by_role(Role::Button, "Details")?;
    screen.click(toggle)?;
    let details = screen.scope_of(screen.within(left)?.get_by_text("Nested details")?)?;
    assert_ne!(details, left, "nearest nested scope owns its descendants");
    assert_eq!(
        screen
            .within(left)?
            .query_all_by_role(Role::Button, "Increment")
            .len(),
        2
    );
    let nested_increment = screen
        .within(details)?
        .get_by_role(Role::Button, "Increment")?;
    screen.click(nested_increment)?;
    screen.within(left)?.get_by_text("Count: 3")?;
    screen.click(toggle)?;
    assert!(
        screen.within(details).is_err(),
        "retired scope is not a query boundary"
    );
    assert_inactive_operations(
        &mut screen,
        nested_increment,
        "unmounted element",
        "no longer mounted",
    );
    assert!(screen.evaluations(nested_increment).is_err());
    screen.within(left)?.get_by_text("Count: 3")?;
    assert_eq!(screen.text(draft)?, "Retained draft");
    Ok(())
}
struct Rejected;
impl Actor for Rejected {
    const STATE: &'static [StateSpec] = &[
        LEFT.spec(),
        RIGHT.spec(),
        OPEN.spec(),
        DRAFT.spec(),
        State::<i64>::new("test.new", 0).spec(),
    ];
    const ACTIONS: &'static [Action] = &[];
    fn migrate(_: &Cx<'_>, _: u64) -> Result<()> {
        Err(Error::Application("reject migration".into()))
    }
    fn view(cx: &Cx<'_>) -> Result<View> {
        App::<true>::view(cx)
    }
}
#[test]
fn scoped_locators_resolve_current_code_and_survive_failed_replacement()
-> std::result::Result<(), String> {
    let mut screen = Screen::render::<App<false>>()?;
    let left = screen.get_by_key("Left")?;
    let increment = screen
        .within(left)?
        .get_by_role(Role::Button, "Increment")?;
    let draft = screen.within(left)?.get_by_role(Role::Textbox, "Draft")?;
    screen.click(increment)?;
    screen.input(draft, "unsaved")?;
    let state = screen.actor().state_address();
    screen.replace::<App<true>>()?;
    assert_eq!(screen.actor().state_address(), state);
    screen.click(increment)?;
    screen.within(left)?.get_by_text("Count: 11")?;
    assert_eq!(screen.text(draft)?, "unsaved");
    assert!(screen.replace::<Rejected>().is_err());
    assert_eq!(screen.actor().state_address(), state);
    screen.click(increment)?;
    screen.within(left)?.get_by_text("Count: 21")?;
    assert_eq!(screen.text(draft)?, "unsaved");
    Ok(())
}
const TAB: State<i64> = State::<i64>::new("test.tab", 0);
struct Hidden;
impl Actor for Hidden {
    const STATE: &'static [StateSpec] = &[TAB.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Ok(Column::default()
            .key("unscoped")
            .children(vec![Tabs::new("Settings", TAB).children(
                cx,
                vec![
                    Tab::new(
                        0,
                        "Account",
                        Text::new("Account content").key("account.content"),
                    ),
                    Tab::new(1, "Security", Text::new("Security content")),
                ],
            )?]))
    }
}
#[test]
fn scoped_queries_reject_hidden_missing_and_unscoped_boundaries() -> std::result::Result<(), String>
{
    let mut screen = Screen::render::<Hidden>()?;
    assert!(screen.get_by_key("missing").is_err());
    let root = screen.get_by_key("unscoped")?;
    assert!(screen.scope_of(root).is_err());
    let panel = screen.get_by_role(Role::Tabpanel, "Account")?;
    let content = screen.within(panel)?.get_by_text("Account content")?;
    screen.click(screen.get_by_role(Role::Tab, "Security")?)?;
    let hidden_panel = screen.style(panel).unwrap_err();
    assert!(
        hidden_panel.contains("element is hidden (key 0x"),
        "{hidden_panel}"
    );
    assert_inactive_operations(
        &mut screen,
        panel,
        "Tabpanel \"Account\"",
        "element is hidden",
    );
    assert!(screen.within(panel).is_err());
    assert!(screen.get_by_key("account.content").is_err());
    let evaluations = screen.root_evaluations();
    let hidden = screen.text(content).unwrap_err();
    assert!(
        hidden.contains("element is no longer mounted (key 0x"),
        "{hidden}"
    );
    assert_eq!(screen.evaluations(content).unwrap_err(), hidden);
    assert_eq!(screen.style(content).unwrap_err(), hidden);
    let tree = screen.debug_tree();
    assert!(!tree.contains("Account content"), "{tree}");
    assert!(tree.contains("Security content"), "{tree}");
    let security = tree
        .lines()
        .find(|line| line.contains("Tab \"Security\""))
        .unwrap();
    let account = tree
        .lines()
        .find(|line| line.contains("Tab \"Account\""))
        .unwrap();
    assert!(security.contains("selected=true"), "{tree}");
    assert!(account.contains("selected=false"), "{tree}");
    assert_eq!(screen.root_evaluations(), evaluations);
    assert!(screen.query_all_by_text("Account content").is_empty());
    screen.get_by_text("Security content")?;
    let missing = screen.get_by_text("Missing content").unwrap_err();
    assert!(missing.contains("Security content"), "{missing}");
    assert!(!missing.contains("Account content"), "{missing}");
    screen.click(screen.get_by_role(Role::Tab, "Account")?)?;
    assert_eq!(
        screen.within(panel)?.get_by_text("Account content")?,
        content
    );
    Ok(())
}

struct ManyCandidates;
impl Actor for ManyCandidates {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(
            (0..160)
                .map(|i| {
                    Button::label(format!("Candidate {i}\n{}", "长".repeat(200)))
                        .key(Key::named("candidate").scoped(i))
                        .into_view()
                })
                .collect(),
        ))
    }
}
#[test]
fn missing_query_diagnostics_bound_candidates_and_escape_labels() -> std::result::Result<(), String>
{
    let mut screen = Screen::render::<ManyCandidates>()?;
    let error = screen.get_by_role(Role::Button, "Missing").unwrap_err();
    assert_eq!(error.matches("key 0x").count(), 8);
    assert!(error.ends_with("…"), "{error}");
    assert!(error.contains("\\n"), "{error}");
    assert!(!error.contains('\n'));
    assert!(!error.contains(&"长".repeat(80)));
    let full_label = format!("Candidate 0\n{}", "长".repeat(200));
    let button = screen.get_by_role(Role::Button, &full_label)?;
    assert_eq!(screen.get_by_text(&full_label)?, button);
    let before = screen.evaluations(button)?;
    let interaction = screen.input(button, "private submitted input").unwrap_err();
    assert!(
        interaction.starts_with("input failed for Button"),
        "{interaction}"
    );
    assert!(
        interaction.contains("\\n") && !interaction.contains('\n'),
        "{interaction}"
    );
    assert!(
        !interaction.contains("private submitted input"),
        "{interaction}"
    );
    assert!(interaction.len() < 512, "{} bytes", interaction.len());
    let key = format!("Invalid\n{}private suffix", "长".repeat(200));
    let error = screen.press(button, &key).unwrap_err();
    assert!(
        error.contains("press failed for Button") && error.contains("unsupported component key"),
        "{error}"
    );
    assert!(error.len() < 1024 && !error.contains('\n'), "{error}");
    assert!(!error.contains("private suffix"), "{error}");
    // Identical previews must not turn distinct full strings into matches.
    let missing = format!("{full_label}different suffix");
    for error in [
        screen.get_by_role(Role::Button, &missing).unwrap_err(),
        screen.get_by_text(&missing).unwrap_err(),
    ] {
        assert!(error.contains("found 0;"), "{error}");
        assert!(error.len() < 4096, "{} bytes", error.len());
        assert!(!error.contains("different suffix"), "{error}");
        assert!(!error.contains('\n'));
    }
    assert_eq!(screen.evaluations(button)?, before);
    let tree = screen.debug_tree();
    assert_eq!(tree.lines().count(), 129);
    assert!(tree.ends_with("… (remaining nodes omitted)\n"));
    assert!(tree.contains("\\n"));
    assert!(!tree.contains(&"长".repeat(80)));
    assert_eq!(screen.evaluations(button)?, before);
    Ok(())
}

fn assert_inactive_operations(
    screen: &mut Screen,
    locator: blockui_test::Locator,
    identity: &str,
    reason: &str,
) {
    let tree = screen.debug_tree();
    let evaluations = screen.root_evaluations();
    let errors = [
        ("click", screen.click(locator).unwrap_err()),
        ("input", screen.input(locator, "ignored").unwrap_err()),
        ("select", screen.select(locator, 1).unwrap_err()),
        ("context_menu", screen.context_menu(locator).unwrap_err()),
        (
            "expect_text",
            screen.expect_text(locator, "ignored").unwrap_err(),
        ),
    ];
    for (operation, error) in errors {
        assert!(
            error.starts_with(&format!("{operation} failed for {identity}")),
            "{error}"
        );
        assert!(
            error.contains(reason) && error.contains("key 0x"),
            "{error}"
        );
    }
    assert_eq!(screen.debug_tree(), tree);
    assert_eq!(screen.root_evaluations(), evaluations);
}

#[test]
fn text_expectations_use_committed_values_and_bound_failure_previews()
-> std::result::Result<(), String> {
    BUILDS.with(|n| n.set([0; 3]));
    let mut screen = Screen::render::<App<false>>()?;
    let right = screen.get_by_key("Right")?;
    let value = screen.within(right)?.get_by_text("Count: 0")?;
    let evaluations = screen.evaluations(value)?;
    let root = screen.root_evaluations();
    let tree = screen.debug_tree();
    screen.expect_text(value, "Count: 0")?;
    let expected = "missing\n".repeat(100);
    let error = screen.expect_text(value, &expected).unwrap_err();
    assert!(error.starts_with("expect_text failed for"), "{error}");
    assert!(
        error.contains("expected text") && error.contains("received \"Count: 0\""),
        "{error}"
    );
    assert!(error.contains("missing\\n"), "{error}");
    assert!(!error.contains(&expected));
    assert!(
        error.contains("key 0x") && error.contains("current UI:\n"),
        "{error}"
    );
    assert!(error.ends_with(&tree), "{error}");
    assert_eq!(screen.debug_tree(), tree);
    assert_eq!(screen.evaluations(value)?, evaluations);
    assert_eq!(screen.root_evaluations(), root);
    assert_eq!(BUILDS.with(Cell::get), [1, 1, 1]);
    let increment = screen
        .within(right)?
        .get_by_role(Role::Button, "Increment")?;
    screen.click(increment)?;
    screen.expect_text(value, "Count: 1")?;
    let after = screen.evaluations(value)?;
    screen.expect_text(value, "Count: 1")?;
    assert_eq!(screen.evaluations(value)?, after);
    assert_eq!(screen.root_evaluations(), root);
    Ok(())
}

#[test]
fn text_expectations_compare_beyond_the_unicode_preview() -> std::result::Result<(), String> {
    struct LongText;
    impl Actor for LongText {
        const STATE: &'static [StateSpec] = &[];
        const ACTIONS: &'static [Action] = &[];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(vec![Text::new("界".repeat(100) + "A").into_view()]))
        }
    }
    let screen = Screen::render::<LongText>()?;
    let actual = "界".repeat(100) + "A";
    let label = screen.get_by_text(&actual)?;
    screen.expect_text(label, &actual)?;
    let error = screen
        .expect_text(label, &("界".repeat(100) + "B"))
        .unwrap_err();
    assert!(
        error.contains("first difference at character 101"),
        "{error}"
    );
    let error = screen.expect_text(label, &"界".repeat(100)).unwrap_err();
    assert!(
        error.contains("first difference at character 101"),
        "{error}"
    );
    let error = screen.expect_text(label, &(actual + "B")).unwrap_err();
    assert!(
        error.contains("first difference at character 102"),
        "{error}"
    );
    Ok(())
}
