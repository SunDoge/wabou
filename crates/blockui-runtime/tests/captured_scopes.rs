#![cfg(feature = "testing")]
use blockui::*;
use std::cell::Cell;
const PARAM: State<i64> = State::<i64>::new("scope.param", 0);
const LEFT: State<i64> = State::<i64>::new("scope.left", 1);
const RIGHT: State<i64> = State::<i64>::new("scope.right", 2);
const ACTIVE: State<bool> = State::<bool>::new("scope.active", true);
const INNER: State<i64> = State::<i64>::new("scope.inner", 0);
const SIZE: State<i64> = State::<i64>::new("scope.size", 180);
const DRAFT: State<String> = State::<String>::new("scope.draft");
const SET_PARAM: Action = Action::new("scope.param.set", |cx, n| PARAM.set(cx, n));
const SET_LEFT: Action = Action::new("scope.left.set", |cx, n| LEFT.set(cx, n));
const SET_RIGHT: Action = Action::new("scope.right.set", |cx, n| RIGHT.set(cx, n));
const SET_INNER: Action = Action::new("scope.inner.set", |cx, n| INNER.set(cx, n));
const SET_SIZE: Action = Action::new("scope.size.set", |cx, n| SIZE.set(cx, n));
const SWITCH: Action = Action::new("scope.switch", |cx, n| ACTIVE.set(cx, n != 0));
thread_local! {
    static BUILDS: Cell<[usize; 4]> = const { Cell::new([0; 4]) };
    static LIVE: Cell<usize> = const { Cell::new(0) };
}
fn built(index: usize) {
    BUILDS.with(|counts| {
        let mut n = counts.get();
        n[index] += 1;
        counts.set(n);
    });
}
struct Capture(String);
impl Capture {
    fn new(value: String) -> Self {
        LIVE.with(|n| n.set(n.get() + 1));
        Self(value)
    }
    fn text(&self) -> &str {
        &self.0
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        LIVE.with(|n| n.set(n.get() - 1));
    }
}
struct App<const NEW: bool>;
impl<const NEW: bool> Actor for App<NEW> {
    const STATE: &'static [StateSpec] = &[
        PARAM.spec(),
        LEFT.spec(),
        RIGHT.spec(),
        ACTIVE.spec(),
        INNER.spec(),
        SIZE.spec(),
        DRAFT.spec(),
    ];
    const ACTIONS: &'static [Action] =
        &[SET_PARAM, SET_LEFT, SET_RIGHT, SET_INNER, SET_SIZE, SWITCH];
    fn view(cx: &Cx<'_>) -> Result<View> {
        built(0);
        let param = PARAM.get(cx)?;
        if param == 3 {
            return Ok(Column::default().children(vec![]));
        }
        let capture = Capture::new(format!("{}:{param}", if NEW { "new" } else { "old" }));
        let source = if NEW { RIGHT } else { LEFT };
        let outer = Scope::reactive("outer", move |cx| {
            built(1);
            let value = if ACTIVE.get(cx)? {
                source.get(cx)?
            } else {
                RIGHT.get(cx)?
            };
            let nested_capture = Capture::new(format!("nested {value}"));
            let inner = Scope::reactive("inner", move |cx| {
                built(2);
                Ok(Column::default().children(vec![
                    Text::new(format!("{} / {}", nested_capture.text(), INNER.get(cx)?))
                        .key("label")
                        .into_view(),
                ]))
            });
            if param == 2 || value == 99 {
                return Err(Error::Application("reject captured subtree".into()));
            }
            Ok(Column::default()
                .children(vec![
                    Text::new(format!("{}={value}", capture.text()))
                        .key("label")
                        .into_view(),
                    TextInput::new(DRAFT).key("editor").into_view(),
                    inner,
                ])
                .style_reactive(|cx| {
                    Ok(ViewStyle {
                        width: SIZE.get(cx)? as f32,
                        ..default()
                    })
                }))
        });
        let fixed_capture = Capture::new("fixed".into());
        let fixed = Scope::reactive("fixed", move |_| {
            built(3);
            Ok(Column::default().children(vec![
                Text::new(fixed_capture.text()).key("label").into_view(),
            ]))
        });
        Ok(Column::default().children(vec![outer, fixed]))
    }
}
fn scoped(parent: Key, child: &str) -> u64 {
    parent.scoped(Key::named(child).0).0
}
fn send(
    actor: &mut blockui_runtime::Actor,
    action: Action,
    value: i64,
) -> std::result::Result<(), String> {
    actor.tell(action, value)?;
    actor.tick()
}
#[test]
fn captured_structural_boundaries_refresh_only_dependencies_and_replace_parameters()
-> std::result::Result<(), String> {
    BUILDS.with(|n| n.set([0; 4]));
    let mut actor = blockui_runtime::Actor::for_test::<App<false>>()?;
    let outer = Key::named("outer");
    let label = scoped(outer, "label");
    let inner = Key(scoped(outer, "inner"));
    let inner_label = scoped(inner, "label");
    let fixed_label = scoped(Key::named("fixed"), "label");
    assert_eq!(BUILDS.with(Cell::get), [1, 1, 1, 1]);
    assert_eq!(actor.view.bindings[&label].text, "old:0=1");
    assert_eq!(actor.view.bindings[&inner_label].text, "nested 1 / 0");
    assert_eq!(actor.view.bindings[&fixed_label].text, "fixed");
    send(&mut actor, SET_SIZE, 280)?;
    assert_eq!(actor.view.bindings[&outer.0].node.presentation.width, 280.0);
    assert_eq!(BUILDS.with(Cell::get), [1, 1, 1, 1]);
    send(&mut actor, SET_INNER, 5)?;
    assert_eq!(BUILDS.with(Cell::get), [1, 1, 2, 1]);
    send(&mut actor, SET_LEFT, 7)?;
    assert_eq!(BUILDS.with(Cell::get), [1, 2, 3, 1]);
    assert_eq!(actor.view.bindings[&inner_label].text, "nested 7 / 5");
    send(&mut actor, SWITCH, 0)?;
    let builds = BUILDS.with(Cell::get);
    send(&mut actor, SET_LEFT, 8)?;
    assert_eq!(
        BUILDS.with(Cell::get),
        builds,
        "inactive structural reads unsubscribe"
    );
    send(&mut actor, SET_RIGHT, 9)?;
    assert_eq!(actor.view.bindings[&label].text, "old:0=9");
    send(&mut actor, SET_PARAM, 1)?;
    assert_eq!(
        actor.view.bindings[&label].text, "old:1=9",
        "fresh captures invalidate old cached subtree"
    );
    let builds = BUILDS.with(Cell::get);
    send(&mut actor, SET_PARAM, 1)?;
    assert_eq!(BUILDS.with(Cell::get), builds, "equal writes stay silent");
    Ok(())
}
#[test]
fn captured_scope_failures_unmounts_and_replacements_preserve_live_actor()
-> std::result::Result<(), String> {
    assert_eq!(LIVE.with(Cell::get), 0);
    let mut actor = blockui_runtime::Actor::for_test::<App<false>>()?;
    let mut other = blockui_runtime::Actor::for_test::<App<true>>()?;
    assert_eq!(LIVE.with(Cell::get), 6);
    let outer = Key::named("outer");
    let label = scoped(outer, "label");
    let old = actor.view.bindings[&label].clone();
    assert!(send(&mut actor, SET_PARAM, 2).is_err());
    assert_eq!(actor.integer("scope.param"), 0);
    assert_eq!(actor.view.bindings[&label].text, old.text);
    assert_eq!(
        LIVE.with(Cell::get),
        6,
        "candidate captures dropped on abort"
    );
    assert!(send(&mut actor, SET_LEFT, 99).is_err());
    assert_eq!(actor.integer("scope.left"), 1);
    assert_eq!(LIVE.with(Cell::get), 6);
    send(&mut actor, SET_LEFT, 8)?;
    assert_eq!(actor.view.bindings[&label].text, "old:0=8");
    let address = actor.state_address();
    let editor = scoped(outer, "editor");
    actor.view.renderer.get_mut(&editor).unwrap().selection = (1, 4);
    actor.replace_for_test::<App<true>>()?;
    assert_eq!(actor.state_address(), address);
    assert_eq!(actor.view.renderer[&editor].selection, (1, 4));
    assert_eq!(actor.view.bindings[&label].text, "new:0=2");
    assert_eq!(LIVE.with(Cell::get), 6);
    send(&mut actor, SET_RIGHT, 12)?;
    assert_eq!(other.view.bindings[&label].text, "new:0=2");
    send(&mut actor, SET_PARAM, 3)?;
    assert_eq!(
        LIVE.with(Cell::get),
        3,
        "unmount retires nested captures too"
    );
    drop(actor);
    send(&mut other, SET_RIGHT, 13)?;
    assert_eq!(other.view.bindings[&label].text, "new:0=13");
    drop(other);
    assert_eq!(LIVE.with(Cell::get), 0);
    Ok(())
}
struct Collision<const KIND: u8>;
impl<const KIND: u8> Actor for Collision<KIND> {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    const SCOPES: &'static [Scope] = &[Scope::new("declared", |_| {
        Ok(Column::default().children(vec![]))
    })];
    fn view(_: &Cx<'_>) -> Result<View> {
        let name = if KIND == 0 { "declared" } else { "duplicate" };
        let build: fn(&Cx<'_>) -> Result<View> = |_| Ok(Column::default().children(vec![]));
        let first = Scope::reactive(name, build);
        Ok(Column::default().children(if KIND == 0 {
            vec![first]
        } else {
            vec![first, Scope::reactive(name, build)]
        }))
    }
}
#[test]
fn captured_scope_rejects_declared_and_repeated_identities() {
    assert!(blockui_runtime::Actor::for_test::<Collision<0>>().is_err());
    assert!(blockui_runtime::Actor::for_test::<Collision<1>>().is_err());
}

struct Cached;
const STATIC: Scope = Scope::new("static", |_| {
    built(1);
    let owned = String::from("cached capture");
    Ok(
        Column::default().children(vec![Scope::reactive("local", move |cx| {
            built(2);
            Ok(Column::default().children(vec![
                Text::new(format!("{owned} {}", INNER.get(cx)?))
                    .key("label")
                    .into_view(),
            ]))
        })]),
    )
});
impl Actor for Cached {
    const STATE: &'static [StateSpec] = &[PARAM.spec(), INNER.spec()];
    const ACTIONS: &'static [Action] = &[SET_PARAM, SET_INNER];
    const SCOPES: &'static [Scope] = &[STATIC];
    fn view(cx: &Cx<'_>) -> Result<View> {
        built(0);
        let _ = PARAM.get(cx)?;
        Ok(Column::default().children(vec![STATIC.into_view()]))
    }
}
#[test]
fn static_scope_cache_retains_nested_captured_builders() -> std::result::Result<(), String> {
    BUILDS.with(|n| n.set([0; 4]));
    let mut actor = blockui_runtime::Actor::for_test::<Cached>()?;
    assert_eq!(BUILDS.with(Cell::get), [1, 1, 1, 0]);
    send(&mut actor, SET_PARAM, 1)?;
    assert_eq!(BUILDS.with(Cell::get), [2, 1, 1, 0]);
    send(&mut actor, SET_INNER, 7)?;
    assert_eq!(BUILDS.with(Cell::get), [2, 1, 2, 0]);
    assert_eq!(
        actor.view.bindings[&scoped(Key::named("local"), "label")].text,
        "cached capture 7"
    );
    Ok(())
}
struct Instances;
impl Actor for Instances {
    const STATE: &'static [StateSpec] = &[LEFT.spec(), RIGHT.spec()];
    const ACTIONS: &'static [Action] = &[SET_LEFT, SET_RIGHT];
    fn view(_: &Cx<'_>) -> Result<View> {
        fn component(name: &str, value: State<i64>) -> View {
            let identity = Key::named(name);
            let path = InstancePath::EMPTY.scoped(identity);
            __component_view(
                Scope::reactive("local", move |cx| {
                    Ok(Column::default().children(vec![
                        Text::new(value.get(cx)?.to_string())
                            .key("label")
                            .into_view(),
                    ]))
                }),
                path,
                identity,
            )
        }
        Ok(Row::default().children(vec![component("first", LEFT), component("second", RIGHT)]))
    }
}
#[test]
fn captured_scope_mounts_follow_component_instance_identity() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Instances>()?;
    let first = scoped(Key(scoped(Key::named("first"), "local")), "label");
    let second = scoped(Key(scoped(Key::named("second"), "local")), "label");
    assert_ne!(first, second);
    assert_eq!(actor.view.bindings[&first].text, "1");
    assert_eq!(actor.view.bindings[&second].text, "2");
    let evaluations = actor.view.bindings[&second].evaluations;
    send(&mut actor, SET_LEFT, 7)?;
    assert_eq!(actor.view.bindings[&first].text, "7");
    assert_eq!(actor.view.bindings[&second].text, "2");
    assert_eq!(actor.view.bindings[&second].evaluations, evaluations);
    Ok(())
}

#[component]
fn LocalCounter(#[state(default = 3)] count: State<i64>) -> View {
    let title = String::from("Local count");
    Scope::reactive("panel", move |cx| {
        Ok(Column::default().children(vec![
            Text::new(format!("{title}: {}", count.get(cx)?))
                .key("label")
                .into_view(),
            Button::label("Increment")
                .key("increment")
                .on_click(move |cx| count.update(cx, |n| Ok(n + 1)))
                .into_view(),
        ]))
    })
}
#[app]
fn LocalPair(left: LocalCounter, right: LocalCounter) -> View {
    Row::default().children(vec![left.view(), right.view()])
}
#[test]
fn function_components_use_local_scopes_and_current_events_without_registration()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<LocalPair>()?;
    assert!(actor.scope_ids().is_empty(), "no declared registry entries");
    assert_eq!(actor.view.scope_dependencies.len(), 2);
    let pair = LocalPair::new();
    let buttons: Vec<_> = actor
        .view
        .order
        .iter()
        .copied()
        .filter(|key| actor.view.bindings[key].node.kind == abi::BUTTON)
        .collect();
    assert_eq!(buttons.len(), 2);
    actor.dispatch_widget(buttons[0])?;
    actor.dispatch_widget(buttons[0])?;
    assert_eq!(actor.integer_by_key(pair.left.count.key()), 5);
    assert_eq!(actor.integer_by_key(pair.right.count.key()), 3);
    let state = actor.state_address();
    actor.replace_for_test::<LocalPair>()?;
    actor.dispatch_widget(buttons[1])?;
    assert_eq!(actor.integer_by_key(pair.left.count.key()), 5);
    assert_eq!(actor.integer_by_key(pair.right.count.key()), 4);
    assert_eq!(actor.state_address(), state);
    Ok(())
}

struct Mixed;
impl Actor for Mixed {
    const STATE: &'static [StateSpec] = &[LEFT.spec(), RIGHT.spec()];
    const ACTIONS: &'static [Action] = &[SET_LEFT, SET_RIGHT];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            Text::reactive(|cx| Ok(RIGHT.get(cx)?.to_string()))
                .key("independent")
                .style_reactive(|cx| {
                    Ok(ViewStyle {
                        width: LEFT.get(cx)? as f32,
                        ..default()
                    })
                }),
            Scope::reactive("changing", |cx| {
                let n = LEFT.get(cx)?;
                Ok(Column::default().children(vec![Text::new(n.to_string()).into_view()]))
            }),
        ]))
    }
}
#[test]
fn structural_refresh_keeps_retained_text_and_style_observers_independent()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Mixed>()?;
    let id = Key::named("independent").0;
    assert_eq!(
        (
            actor.view.bindings[&id].evaluations,
            actor.view.bindings[&id].style_evaluations
        ),
        (1, 1)
    );
    send(&mut actor, SET_LEFT, 7)?;
    assert_eq!(
        (
            actor.view.bindings[&id].evaluations,
            actor.view.bindings[&id].style_evaluations
        ),
        (1, 2)
    );
    assert_eq!(actor.view.bindings[&id].text, "2");
    assert_eq!(actor.view.bindings[&id].node.presentation.width, 7.0);
    send(&mut actor, SET_RIGHT, 8)?;
    assert_eq!(
        (
            actor.view.bindings[&id].evaluations,
            actor.view.bindings[&id].style_evaluations
        ),
        (2, 2)
    );
    Ok(())
}
