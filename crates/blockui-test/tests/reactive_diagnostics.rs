use blockui::*;
use blockui_test::Screen;
const FIRST: Computed<i64> = Computed::<i64>::new("cycle.first", |cx| SECOND.get(cx));
const SECOND: Computed<i64> = Computed::<i64>::new("cycle.second", |cx| FIRST.get(cx));
const MISSING: Computed<i64> = Computed::<i64>::new("missing.memo", |_| Ok(1));
const UNREGISTERED: Scope = Scope::new("missing.scope", |_| Ok(Column::default().children(vec![])));
struct Good;
impl Actor for Good {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![Text::new("Still running").key("live").into_view()]))
    }
}
struct Cyclic;
impl Actor for Cyclic {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    const COMPUTEDS: &'static [ComputedSpec] = &[FIRST.spec(), SECOND.spec()];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Good::view(cx)
    }
}
const FAILING: Computed<i64> = Computed::<i64>::new("failing.memo", |_| {
    Err(Error::Application("cannot derive total".into()))
});
thread_local! { static OUTER_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
const OUTER: Computed<i64> = Computed::<i64>::new("outer.memo", |cx| {
    OUTER_CALLS.with(|calls| calls.set(calls.get() + 1));
    FAILING.get(cx)
});
struct FailedMemo;
impl Actor for FailedMemo {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    const COMPUTEDS: &'static [ComputedSpec] = &[OUTER.spec(), FAILING.spec()];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Good::view(cx)
    }
}
struct MissingMemo;
impl Actor for MissingMemo {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Ok(Text::new(MISSING.get(cx)?.to_string()).into_view())
    }
}
struct MissingScope;
impl Actor for MissingScope {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(UNREGISTERED.into_view())
    }
}
struct FailedScope<const CAPTURED: bool>;
impl<const CAPTURED: bool> Actor for FailedScope<CAPTURED> {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    const SCOPES: &'static [Scope] = &[Scope::new("failed.static", |_| {
        Err(Error::Application("cannot build panel".into()))
    })];
    fn view(_: &Cx<'_>) -> Result<View> {
        if CAPTURED {
            Ok(Scope::reactive("failed.captured", |_| {
                Err(Error::Application("cannot build panel".into()))
            }))
        } else {
            Ok(Self::SCOPES[0].into_view())
        }
    }
}
struct LeafScope;
impl Actor for LeafScope {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Scope::reactive("leaf.scope", |_| {
            Ok(Text::new("Leaf").into_view())
        }))
    }
}
#[test]
fn rejected_candidates_report_reactive_identities_and_preserve_live_projection()
-> std::result::Result<(), String> {
    for result in [
        Screen::render::<MissingScope>(),
        Screen::with_manual_time::<MissingScope>(),
        Screen::with_services::<MissingScope>(Default::default()),
    ] {
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("unregistered scope should fail initial mount"),
        };
        assert!(
            error.starts_with(&format!(
                "render failed for actor {}:",
                std::any::type_name::<MissingScope>()
            )),
            "{error}"
        );
        assert!(error.contains("Actor::SCOPES"), "{error}");
        assert!(
            error.contains(&format!("{:#018x}", Key::named("missing.scope").0)),
            "{error}"
        );
    }
    let mut screen = Screen::render::<Good>()?;
    let live = screen.get_by_key("live")?;
    let before = screen.evaluations(live)?;
    let error = screen.scope_of(live).unwrap_err();
    assert!(error.contains("scope_of failed"), "{error}");
    assert!(error.contains("\"Still running\""), "{error}");
    assert!(error.contains("mount a Scope"), "{error}");
    assert_eq!(screen.evaluations(live)?, before);
    let error = screen.replace::<Cyclic>().unwrap_err();
    assert!(
        error.starts_with(&format!(
            "replace failed for actor {}:",
            std::any::type_name::<Cyclic>()
        )),
        "{error}"
    );
    assert!(error.contains("cyclic computation"), "{error}");
    let a = format!("{:#018x}", FIRST.key());
    let b = format!("{:#018x}", SECOND.key());
    assert!(
        error.contains(&format!("{a} -> {b} -> {a}"))
            || error.contains(&format!("{b} -> {a} -> {b}")),
        "{error}"
    );
    // Eager validation visits keys in order; enter the outer callback first so
    // this checks nested fault propagation, rather than an isolated inner failure.
    assert!(OUTER.key() < FAILING.key());
    OUTER_CALLS.with(|calls| calls.set(0));
    let error = screen.replace::<FailedMemo>().unwrap_err();
    assert_eq!(OUTER_CALLS.with(std::cell::Cell::get), 1);
    assert!(
        !error.contains(&format!("computed {:#018x} evaluation failed", OUTER.key())),
        "{error}"
    );
    assert!(
        error.contains(&format!(
            "computed {:#018x} evaluation failed",
            FAILING.key()
        )),
        "{error}"
    );
    assert!(error.contains("cannot derive total"), "{error}");
    assert_eq!(screen.text(live)?, "Still running");
    assert_eq!(screen.evaluations(live)?, before);
    let error = screen.replace::<MissingMemo>().unwrap_err();
    assert!(
        error.contains(&format!("{:#018x}", MISSING.key())),
        "{error}"
    );
    assert!(error.contains("Actor::COMPUTEDS"), "{error}");
    let error = screen.replace::<MissingScope>().unwrap_err();
    assert!(
        error.contains(&format!("{:#018x}", Key::named("missing.scope").0)),
        "{error}"
    );
    assert!(error.contains("Actor::SCOPES"), "{error}");
    for captured in [false, true] {
        let (error, name) = if captured {
            (
                screen.replace::<FailedScope<true>>().unwrap_err(),
                "failed.captured",
            )
        } else {
            (
                screen.replace::<FailedScope<false>>().unwrap_err(),
                "failed.static",
            )
        };
        assert!(
            error.contains(&format!(
                "scope {:#018x} projection failed",
                Key::named(name).0
            )),
            "{error}"
        );
        assert!(error.contains("cannot build panel"), "{error}");
        assert_eq!(screen.text(live)?, "Still running");
        assert_eq!(screen.evaluations(live)?, before);
    }
    assert_eq!(screen.text(live)?, "Still running");
    assert_eq!(screen.evaluations(live)?, before);
    screen.replace::<Good>()?;
    assert_eq!(screen.text(screen.get_by_key("live")?)?, "Still running");
    Ok(())
}

#[test]
fn leaf_scope_is_wrapped_without_rejecting_the_sdk_view() -> std::result::Result<(), String> {
    let screen = Screen::render::<LeafScope>()?;
    let leaf = screen.get_by_text("Leaf")?;
    let scope = screen.scope_of(leaf)?;
    assert_ne!(scope, leaf);
    assert_eq!(scope, screen.get_by_key("leaf.scope")?);
    let before = screen.evaluations(leaf)?;
    for error in [
        screen.scope_dependencies(leaf).unwrap_err(),
        screen.scope_evaluations(leaf).unwrap_err(),
    ] {
        assert!(error.contains("(key 0x"), "{error}");
        assert!(error.contains("scope_of(element)"), "{error}");
        assert!(error.contains("\"Leaf\""), "{error}");
    }
    assert_eq!(screen.evaluations(leaf)?, before);
    let node = screen.actor().projected_view().bindings[&Key::named("leaf.scope").0].node;
    assert_eq!(node.kind, abi::COLUMN);
    assert_eq!(node.padding, 0.0);
    assert_eq!(node.gap, 0.0);
    Ok(())
}
