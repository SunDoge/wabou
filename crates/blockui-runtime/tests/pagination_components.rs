#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::Actor as Host;
const PAGE: State<i64> = State::<i64>::new("pagination.page", 0);
const PAGES: State<i64> = State::<i64>::new("pagination.pages", 10);
const SET_PAGE: Action = Action::new("pagination.set_page", |cx, value| PAGE.set(cx, value));
const SET_PAGES: Action = Action::new("pagination.set_pages", |cx, value| PAGES.set(cx, value));
struct Fixture;
impl Actor for Fixture {
    const STATE: &'static [StateSpec] = &[PAGE.spec(), PAGES.spec()];
    const ACTIONS: &'static [Action] = &[SET_PAGE, SET_PAGES];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Pagination::new(PAGE, PAGES.get(cx)?)
            .key("pager")
            .edge_controls(true)
            .view(cx)
    }
}
struct Replacement;
impl Actor for Replacement {
    const STATE: &'static [StateSpec] = Fixture::STATE;
    const ACTIONS: &'static [Action] = Fixture::ACTIONS;
    fn view(cx: &Cx<'_>) -> Result<View> {
        Pagination::new(PAGE, 3)
            .key("pager")
            .edge_controls(true)
            .view(cx)
    }
}
fn part(name: &str) -> u64 {
    Key::named("pager").scoped(abi::key(name)).0
}
fn tell(actor: &mut Host, action: Action, value: i64) -> std::result::Result<(), String> {
    actor.tell(action, value)?;
    actor.tick()
}
#[test]
fn pagination_boundaries_identity_and_current_replacement_actions()
-> std::result::Result<(), String> {
    let mut actor = Host::for_test::<Fixture>()?;
    assert_eq!(
        actor.projected_view().bindings[&Key::named("pager").0]
            .node
            .semantics
            .role,
        abi::semantics::NAVIGATION
    );
    assert_eq!(
        actor.projected_view().bindings[&part("status")]
            .node
            .semantics
            .role,
        abi::semantics::STATUS
    );
    assert_eq!(
        actor.projected_view().bindings[&part("count")].text,
        "Page 1 of 10"
    );
    actor.dispatch_widget(part("previous"))?;
    assert_eq!(actor.integer("pagination.page"), 0);
    actor.dispatch_widget(part("last"))?;
    assert_eq!(actor.integer("pagination.page"), 9);
    assert_eq!(
        actor.projected_view().bindings[&part("next")].node.disabled,
        1
    );
    actor.dispatch_widget(part("previous"))?;
    assert_eq!(actor.integer("pagination.page"), 8);
    actor.replace_for_test::<Replacement>()?;
    assert_eq!(
        actor.integer("pagination.page"),
        8,
        "view does not mutate state"
    );
    assert_eq!(
        actor.projected_view().bindings[&part("count")].text,
        "Page 3 of 3"
    );
    actor.dispatch_widget(part("previous"))?;
    assert_eq!(
        actor.integer("pagination.page"),
        1,
        "callback uses current page count"
    );
    actor.dispatch_widget(part("first"))?;
    assert_eq!(actor.integer("pagination.page"), 0);
    Ok(())
}
#[test]
fn pagination_empty_shrinking_negative_and_extreme_ranges() -> std::result::Result<(), String> {
    let mut actor = Host::for_test::<Fixture>()?;
    tell(&mut actor, SET_PAGE, i64::MAX)?;
    tell(&mut actor, SET_PAGES, 0)?;
    assert_eq!(
        actor.projected_view().bindings[&part("count")].text,
        "Page 0 of 0"
    );
    for name in ["first", "previous", "next", "last"] {
        assert_eq!(
            actor.projected_view().bindings[&part(name)].node.disabled,
            1
        );
        actor.dispatch_widget(part(name))?;
        assert_eq!(actor.integer("pagination.page"), i64::MAX);
    }
    tell(&mut actor, SET_PAGES, i64::MAX)?;
    actor.dispatch_widget(part("previous"))?;
    assert_eq!(actor.integer("pagination.page"), i64::MAX - 2);
    tell(&mut actor, SET_PAGE, i64::MIN)?;
    assert_eq!(
        actor.projected_view().bindings[&part("count")].text,
        format!("Page 1 of {}", i64::MAX)
    );
    actor.dispatch_widget(part("next"))?;
    assert_eq!(actor.integer("pagination.page"), 1);
    assert!(tell(&mut actor, SET_PAGES, -1).is_err());
    assert_eq!(
        actor.integer("pagination.pages"),
        i64::MAX,
        "invalid view rolls back"
    );
    Ok(())
}

struct Numbered;
impl Actor for Numbered {
    const STATE: &'static [StateSpec] = Fixture::STATE;
    const ACTIONS: &'static [Action] = Fixture::ACTIONS;
    fn view(cx: &Cx<'_>) -> Result<View> {
        Pagination::new(PAGE, PAGES.get(cx)?)
            .key("pager")
            .numbered(true)
            .view(cx)
    }
}
fn number(page: u64) -> u64 {
    Key::named("pager")
        .scoped(abi::key("numbers"))
        .scoped(page)
        .0
}
#[test]
fn numbered_pages_are_bounded_and_current_targets_survive_replacement()
-> std::result::Result<(), String> {
    let mut actor = Host::for_test::<Numbered>()?;
    tell(&mut actor, SET_PAGES, i64::MAX)?;
    let numbers = part("numbers");
    assert!(
        actor
            .projected_view()
            .bindings
            .values()
            .filter(|binding| binding.node.parent == numbers)
            .count()
            <= 7
    );
    assert_eq!(
        actor.projected_view().bindings[&number(0)]
            .node
            .semantics
            .current,
        1
    );
    actor.dispatch_widget(number((i64::MAX - 1) as u64))?;
    assert_eq!(actor.integer("pagination.page"), i64::MAX - 1);
    assert_eq!(
        actor.projected_view().bindings[&number(0)]
            .node
            .semantics
            .current,
        0
    );
    assert_eq!(
        actor.projected_view().bindings[&number((i64::MAX - 1) as u64)]
            .node
            .semantics
            .current,
        1
    );
    assert_eq!(
        actor
            .projected_view()
            .bindings
            .values()
            .filter(|binding| binding.node.semantics.current == 1)
            .count(),
        1
    );

    assert!(
        actor.dispatch_widget(number(1)).is_err(),
        "stale window targets are unmounted"
    );
    actor.dispatch_widget(number((i64::MAX - 2) as u64))?;
    assert_eq!(actor.integer("pagination.page"), i64::MAX - 2);
    actor.replace_for_test::<Numbered>()?;
    actor.dispatch_widget(number(0))?;
    assert_eq!(actor.integer("pagination.page"), 0);
    assert_eq!(
        actor.projected_view().bindings[&number(0)]
            .node
            .button_variant,
        0
    );
    tell(&mut actor, SET_PAGES, 0)?;
    assert!(!actor.projected_view().bindings.contains_key(&numbers));
    assert!(actor.dispatch_widget(number(0)).is_err());
    Ok(())
}

struct TwoRegions;
impl Actor for TwoRegions {
    const STATE: &'static [StateSpec] = Fixture::STATE;
    const ACTIONS: &'static [Action] = Fixture::ACTIONS;
    fn view(cx: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Pagination::new(PAGE, PAGES.get(cx)?)
                .key("pager")
                .view(cx)?,
            Pagination::new(PAGE, 3)
                .key("other")
                .numbered(true)
                .view(cx)?,
        ]))
    }
}
#[test]
fn disabled_navigation_focus_stays_local_and_prefers_current_page()
-> std::result::Result<(), String> {
    use blockui_runtime::interaction::navigation_focus::recover;
    let mut actor = Host::for_test::<TwoRegions>()?;
    tell(&mut actor, SET_PAGE, 9)?;
    assert_eq!(
        recover(actor.projected_view(), part("next")),
        Some(part("previous"))
    );
    assert_eq!(
        recover(actor.projected_view(), part("previous")),
        None,
        "enabled targets retain their own focus"
    );
    tell(&mut actor, SET_PAGES, 0)?;
    assert_eq!(
        recover(actor.projected_view(), part("next")),
        None,
        "another navigation region must not capture focus"
    );
    assert_eq!(recover(actor.projected_view(), u64::MAX), None);
    let mut actor = Host::for_test::<Numbered>()?;
    tell(&mut actor, SET_PAGE, 9)?;
    assert_eq!(
        recover(actor.projected_view(), part("next")),
        Some(number(9))
    );
    Ok(())
}
