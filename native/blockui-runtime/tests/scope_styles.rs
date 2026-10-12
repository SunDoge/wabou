#![cfg(feature = "testing")]
use blockui::*;
use std::cell::Cell;
const ROOT: State<i64> = State::<i64>::new("scope-style.root", 0);
const OUTER: State<i64> = State::<i64>::new("scope-style.outer", 0);
const INNER: State<i64> = State::<i64>::new("scope-style.inner", 0);
const SIZE: State<i64> = State::<i64>::new("scope-style.size", 14);
const TEXT: State<i64> = State::<i64>::new("scope-style.text", 0);
const SET_ROOT: Action = Action::new("scope-style.root.set", |cx, n| ROOT.set(cx, n));
const SET_OUTER: Action = Action::new("scope-style.outer.set", |cx, n| OUTER.set(cx, n));
const SET_INNER: Action = Action::new("scope-style.inner.set", |cx, n| INNER.set(cx, n));
const SET_SIZE: Action = Action::new("scope-style.size.set", |cx, n| SIZE.set(cx, n));
const SET_TEXT: Action = Action::new("scope-style.text.set", |cx, n| TEXT.set(cx, n));
const BATCH_OUTER_SIZE: Action = Action::new("scope-style.outer-batch", |cx, n| {
    OUTER.set(cx, 1)?;
    SIZE.set(cx, n)
});
const BATCH_ROOT_SIZE: Action = Action::new("scope-style.batch", |cx, n| {
    ROOT.set(cx, n)?;
    SIZE.set(cx, n)
});
thread_local! { static BUILDS: Cell<[usize; 4]> = const { Cell::new([0; 4]) }; }
fn built(index: usize) {
    BUILDS.with(|counts| {
        let mut n = counts.get();
        n[index] += 1;
        counts.set(n);
    });
}
const INNER_SCOPE: Scope = Scope::new("scope-style.inner-panel", |cx| {
    built(2);
    let shape = INNER.get(cx)?;
    Ok(Column::default()
        .children(vec![
            Text::new(format!("inner {shape}"))
                .key("scope-style.inner-text")
                .into_view(),
        ])
        .style_reactive(|cx| {
            Ok(ViewStyle {
                width: SIZE.get(cx)? as f32 * 5.0,
                ..default()
            })
        }))
});
const OUTER_SCOPE: Scope = Scope::new("scope-style.outer-panel", |cx| {
    built(1);
    let shape = OUTER.get(cx)?;
    let mut children = vec![
        INNER_SCOPE.into_view(),
        Text::new(TEXT).key("scope-style.text-label").into_view(),
    ];
    if shape == 1 {
        children.push(Text::new("extra").key("scope-style.extra").into_view());
    }
    let view = Column::default().children(children);
    if shape == 3 {
        return Ok(view);
    }
    Ok(view.style_reactive(move |cx| {
        Ok(ViewStyle {
            width: if shape == 2 {
                f32::NAN
            } else {
                SIZE.get(cx)? as f32 * 10.0
            },
            surface: Surface::Panel,
            radius: 8.0,
            ..default()
        })
    }))
});
const FIXED_SCOPE: Scope = Scope::new("scope-style.fixed-panel", |_| {
    built(3);
    Ok(Column::default()
        .children(vec![Text::new("fixed").into_view()])
        .style_reactive(|_| {
            Ok(ViewStyle {
                width: 120.0,
                ..default()
            })
        }))
});
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[
        ROOT.spec(),
        OUTER.spec(),
        INNER.spec(),
        SIZE.spec(),
        TEXT.spec(),
    ];
    const ACTIONS: &'static [Action] = &[
        SET_ROOT,
        SET_OUTER,
        SET_INNER,
        SET_SIZE,
        SET_TEXT,
        BATCH_ROOT_SIZE,
        BATCH_OUTER_SIZE,
    ];
    const SCOPES: &'static [Scope] = &[OUTER_SCOPE, INNER_SCOPE, FIXED_SCOPE];
    fn view(cx: &Cx<'_>) -> Result<View> {
        built(0);
        let _ = ROOT.get(cx)?;
        Ok(Column::default().children(vec![OUTER_SCOPE.into_view(), FIXED_SCOPE.into_view()]))
    }
}
fn dispatch(
    actor: &mut blockui_runtime::Actor,
    action: Action,
    value: i64,
) -> std::result::Result<(), String> {
    actor.tell(action, value)?;
    actor.tick()
}
fn width(actor: &blockui_runtime::Actor, scope: Scope) -> f32 {
    actor.view.bindings[&scope.id()].node.presentation.width
}
#[test]
fn scope_root_styles_update_independently_and_survive_cache_and_nested_refresh()
-> std::result::Result<(), String> {
    BUILDS.with(|n| n.set([0; 4]));
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    assert_eq!(width(&actor, OUTER_SCOPE), 140.0);
    assert_eq!(width(&actor, INNER_SCOPE), 70.0);
    assert_eq!(width(&actor, FIXED_SCOPE), 120.0);
    assert_eq!(BUILDS.with(Cell::get), [1, 1, 1, 1]);
    dispatch(&mut actor, SET_SIZE, 16)?;
    assert_eq!(width(&actor, OUTER_SCOPE), 160.0);
    assert_eq!(width(&actor, INNER_SCOPE), 80.0);
    assert_eq!(
        BUILDS.with(Cell::get),
        [1, 1, 1, 1],
        "style update does not run builders"
    );
    let outer_evals = actor.view.bindings[&OUTER_SCOPE.id()].style_evaluations;
    dispatch(&mut actor, SET_TEXT, 5)?;
    assert_eq!(
        actor.view.bindings[&OUTER_SCOPE.id()].style_evaluations,
        outer_evals
    );
    dispatch(&mut actor, SET_ROOT, 1)?;
    assert_eq!(
        BUILDS.with(Cell::get),
        [2, 1, 1, 1],
        "root reuses cached scope builders"
    );
    assert_eq!(width(&actor, OUTER_SCOPE), 160.0);
    assert_eq!(
        actor.view.bindings[&OUTER_SCOPE.id()].style_evaluations,
        outer_evals
    );
    dispatch(&mut actor, SET_INNER, 1)?;
    assert_eq!(BUILDS.with(Cell::get), [2, 1, 2, 1]);
    assert_eq!(width(&actor, INNER_SCOPE), 80.0);
    assert_eq!(width(&actor, OUTER_SCOPE), 160.0);
    let inner_evals = actor.view.bindings[&INNER_SCOPE.id()].style_evaluations;
    dispatch(&mut actor, SET_OUTER, 1)?;
    assert_eq!(
        BUILDS.with(Cell::get),
        [2, 2, 2, 1],
        "unchanged nested builder stays cached"
    );
    assert_eq!(
        actor.view.bindings[&INNER_SCOPE.id()].style_evaluations,
        inner_evals
    );
    assert_eq!(width(&actor, OUTER_SCOPE), 160.0);
    assert_eq!(width(&actor, INNER_SCOPE), 80.0);
    assert!(
        actor
            .view
            .bindings
            .contains_key(&Key::named("scope-style.extra").0)
    );
    let before = actor.view.bindings[&OUTER_SCOPE.id()].clone();
    assert!(dispatch(&mut actor, SET_OUTER, 2).is_err());
    assert_eq!(actor.integer("scope-style.outer"), 1);
    assert_eq!(actor.view.bindings[&OUTER_SCOPE.id()].node, before.node);
    assert_eq!(
        actor.view.bindings[&OUTER_SCOPE.id()].dependencies,
        before.dependencies
    );
    dispatch(&mut actor, SET_SIZE, 18)?;
    assert_eq!(
        width(&actor, OUTER_SCOPE),
        180.0,
        "old callback survives failed projection"
    );
    assert_eq!(width(&actor, INNER_SCOPE), 90.0);
    assert_eq!(width(&actor, FIXED_SCOPE), 120.0);
    dispatch(&mut actor, SET_OUTER, 3)?;
    assert_eq!(width(&actor, OUTER_SCOPE), 0.0);
    assert!(
        actor.view.bindings[&OUTER_SCOPE.id()]
            .style_dependencies
            .is_empty()
    );
    dispatch(&mut actor, SET_SIZE, 20)?;
    assert_eq!(
        width(&actor, OUTER_SCOPE),
        0.0,
        "removed root style is unsubscribed"
    );
    assert_eq!(width(&actor, INNER_SCOPE), 100.0);
    let builds = BUILDS.with(Cell::get);
    dispatch(&mut actor, BATCH_ROOT_SIZE, 22)?;
    let mut expected = builds;
    expected[0] += 1;
    assert_eq!(
        BUILDS.with(Cell::get),
        expected,
        "dirty style refreshes while nested builders stay cached"
    );
    assert_eq!(width(&actor, INNER_SCOPE), 110.0);
    assert_eq!(width(&actor, OUTER_SCOPE), 0.0);
    let builds = BUILDS.with(Cell::get);
    dispatch(&mut actor, BATCH_OUTER_SIZE, 26)?;
    let mut expected = builds;
    expected[1] += 1;
    assert_eq!(
        BUILDS.with(Cell::get),
        expected,
        "local refresh reuses nested builder while refreshing its dirty style"
    );
    assert_eq!(width(&actor, INNER_SCOPE), 130.0);
    assert_eq!(width(&actor, OUTER_SCOPE), 260.0);
    Ok(())
}

struct StyledMarker;
impl Actor for StyledMarker {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    const SCOPES: &'static [Scope] = &[FIXED_SCOPE];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![FIXED_SCOPE.style_reactive(|_| Ok(default()))]))
    }
}
#[test]
fn style_on_mount_marker_is_rejected_instead_of_silently_losing_a_callback() {
    let error = match blockui_runtime::Actor::for_test::<StyledMarker>() {
        Ok(_) => panic!("styled mount marker was accepted"),
        Err(error) => error,
    };
    assert!(error.contains("scope builder root"), "{error}");
}
