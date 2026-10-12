#![cfg(feature = "testing")]
use blockui::*;
use std::cell::Cell;
const LEFT: State<i64> = State::<i64>::new("capture.left", 18);
const RIGHT: State<i64> = State::<i64>::new("capture.right", 24);
const ACTIVE: State<bool> = State::<bool>::new("capture.active", true);
const SIZE: State<i64> = State::<i64>::new("capture.size", 14);
const SHAPE: State<i64> = State::<i64>::new("capture.shape", 0);
const SET_LEFT: Action = Action::new("capture.left.set", |cx, n| LEFT.set(cx, n));
const SET_RIGHT: Action = Action::new("capture.right.set", |cx, n| RIGHT.set(cx, n));
const SWITCH: Action = Action::new("capture.switch", |cx, n| ACTIVE.set(cx, n != 0));
const SET_SIZE: Action = Action::new("capture.size.set", |cx, n| SIZE.set(cx, n));
const SET_SHAPE: Action = Action::new("capture.shape.set", |cx, n| SHAPE.set(cx, n));
const FIRST: Key = Key::named("capture.first");
const SECOND: Key = Key::named("capture.second");
thread_local! {
    static LIVE: Cell<usize> = const { Cell::new(0) };
    static DROPS: Cell<usize> = const { Cell::new(0) };
    static VIEWS: Cell<usize> = const { Cell::new(0) };
}
struct Capture(String);
impl Capture {
    fn new(value: String) -> Self {
        LIVE.with(|n| n.set(n.get() + 1));
        Self(value)
    }
    fn prefix(&self) -> &str {
        &self.0
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        LIVE.with(|n| n.set(n.get() - 1));
        DROPS.with(|n| n.set(n.get() + 1));
    }
}
struct App<const REPLACEMENT: bool>;
impl<const REPLACEMENT: bool> Actor for App<REPLACEMENT> {
    const STATE: &'static [StateSpec] = &[
        LEFT.spec(),
        RIGHT.spec(),
        ACTIVE.spec(),
        SIZE.spec(),
        SHAPE.spec(),
    ];
    const ACTIONS: &'static [Action] = &[SET_LEFT, SET_RIGHT, SWITCH, SET_SIZE, SET_SHAPE];
    fn view(cx: &Cx<'_>) -> Result<View> {
        VIEWS.with(|n| n.set(n.get() + 1));
        let shape = SHAPE.get(cx)?;
        if shape == 3 {
            return Ok(Column::default().children(vec![]));
        }
        let prefix = Capture::new(format!(
            "{}:{shape}",
            if REPLACEMENT { "new" } else { "old" }
        ));
        let left = if REPLACEMENT { RIGHT } else { LEFT };
        let right = RIGHT;
        let size = SIZE;
        let style = Capture::new("style".into());
        let second = Capture::new("second".into());
        Ok(Column::default().children(vec![
            Text::new(bind!(move |cx| {
                let value = if ACTIVE.get(cx)? {
                    left.get(cx)?
                } else {
                    right.get(cx)?
                };
                if shape == 2 || value == 99 {
                    return Err(Error::Application("captured text rejected".into()));
                }
                Ok(format!("{}={value}", prefix.prefix()))
            }))
            .key(FIRST)
            .style_reactive(size.map_style_with(move |value| {
                assert_eq!(style.prefix(), "style");
                ViewStyle {
                    font_size: value as f32,
                    ..default()
                }
            })),
            Text::new(right.map_text_with(move |value| format!("{}={value}", second.prefix())))
                .key(SECOND)
                .into_view(),
        ]))
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
fn counts(actor: &blockui_runtime::Actor) -> (u64, u64, u64) {
    let first = &actor.view.bindings[&FIRST.0];
    (
        first.evaluations,
        first.style_evaluations,
        actor.view.bindings[&SECOND.0].evaluations,
    )
}
#[test]
fn captured_text_and_style_observe_independently_without_rebuilding_view()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<App<false>>()?;
    let views = VIEWS.with(Cell::get);
    assert_eq!(actor.view.bindings[&FIRST.0].text, "old:0=18");
    assert_eq!(counts(&actor), (1, 1, 1));
    dispatch(&mut actor, SET_LEFT, 20)?;
    assert_eq!(counts(&actor), (2, 1, 1));
    dispatch(&mut actor, SET_SIZE, 16)?;
    assert_eq!(counts(&actor), (2, 2, 1));
    dispatch(&mut actor, SWITCH, 0)?;
    assert_eq!(actor.view.bindings[&FIRST.0].text, "old:0=24");
    dispatch(&mut actor, SET_LEFT, 21)?;
    assert_eq!(counts(&actor), (3, 2, 1), "inactive dependency removed");
    dispatch(&mut actor, SET_RIGHT, 30)?;
    assert_eq!(counts(&actor), (4, 2, 2));
    dispatch(&mut actor, SET_RIGHT, 30)?;
    assert_eq!(counts(&actor), (4, 2, 2), "equal writes are silent");
    assert_eq!(VIEWS.with(Cell::get), views);
    let old = actor.view.bindings[&FIRST.0].clone();
    assert!(dispatch(&mut actor, SET_RIGHT, 99).is_err());
    assert_eq!(actor.integer("capture.right"), 30);
    assert_eq!(actor.view.bindings[&FIRST.0].text, old.text);
    assert_eq!(counts(&actor), (4, 2, 2));
    assert_eq!(actor.view.bindings[&FIRST.0].dependencies, old.dependencies);
    dispatch(&mut actor, SET_RIGHT, 31)?;
    assert_eq!(actor.view.bindings[&FIRST.0].text, "old:0=31");
    Ok(())
}
#[test]
fn captures_are_transactional_isolated_and_disposed_before_replacement()
-> std::result::Result<(), String> {
    assert_eq!(LIVE.with(Cell::get), 0);
    let mut actor = blockui_runtime::Actor::for_test::<App<false>>()?;
    let mut other = blockui_runtime::Actor::for_test::<App<true>>()?;
    assert_eq!(LIVE.with(Cell::get), 6);
    dispatch(&mut other, SET_RIGHT, 29)?;
    assert_eq!(actor.view.bindings[&FIRST.0].text, "old:0=18");
    let dropped = DROPS.with(Cell::get);
    assert!(dispatch(&mut actor, SET_SHAPE, 2).is_err());
    assert_eq!(actor.integer("capture.shape"), 0);
    assert_eq!(LIVE.with(Cell::get), 6);
    assert_eq!(DROPS.with(Cell::get), dropped + 3);
    dispatch(&mut actor, SET_SHAPE, 1)?;
    assert_eq!(
        actor.view.bindings[&FIRST.0].text, "old:1=18",
        "owned snapshot refreshed on projection"
    );
    let address = actor.state_address();
    actor.view.renderer.get_mut(&FIRST.0).unwrap().selection = (1, 3);
    actor.replace_for_test::<App<true>>()?;
    assert_eq!(actor.state_address(), address);
    assert_eq!(actor.view.renderer[&FIRST.0].selection, (1, 3));
    assert_eq!(actor.view.bindings[&FIRST.0].text, "new:1=24");
    assert_eq!(LIVE.with(Cell::get), 6);
    let evaluated = counts(&actor);
    dispatch(&mut actor, SET_LEFT, 32)?;
    assert_eq!(counts(&actor), evaluated, "retired dependency is gone");
    dispatch(&mut actor, SET_RIGHT, 33)?;
    assert_eq!(actor.view.bindings[&FIRST.0].text, "new:1=33");
    dispatch(&mut actor, SET_SHAPE, 3)?;
    assert_eq!(
        LIVE.with(Cell::get),
        3,
        "unmount disposes both text and style captures"
    );
    drop(actor);
    assert_eq!(LIVE.with(Cell::get), 3, "other actor is independent");
    dispatch(&mut other, SET_RIGHT, 34)?;
    assert_eq!(other.view.bindings[&FIRST.0].text, "new:0=34");
    drop(other);
    assert_eq!(LIVE.with(Cell::get), 0);
    Ok(())
}

struct Decorated;
impl Actor for Decorated {
    const STATE: &'static [StateSpec] = &[LEFT.spec(), SIZE.spec()];
    const ACTIONS: &'static [Action] = &[SET_LEFT, SET_SIZE];
    fn view(_: &Cx<'_>) -> Result<View> {
        let count = LEFT;
        let size = SIZE;
        Ok(Column::default().children(vec![
            Button::new(bind!(move |cx| Ok(format!("Count {}", count.get(cx)?))), SET_LEFT)
                .key(FIRST)
                .leading_icon(IconSource::svg(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M4 12h16" fill="none" stroke="currentColor"/></svg>"#))
                .style_reactive(move |cx| Ok(ViewStyle { font_size: size.get(cx)? as f32, ..default() })),
        ]))
    }
}
#[test]
fn captured_button_labels_keep_icon_resources_and_independent_styles()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Decorated>()?;
    let icon = actor.view.bindings[&FIRST.0].icon.clone().unwrap();
    dispatch(&mut actor, SET_LEFT, 22)?;
    assert_eq!(actor.view.bindings[&FIRST.0].text, "Count 22");
    assert_eq!(actor.view.bindings[&FIRST.0].style_evaluations, 1);
    dispatch(&mut actor, SET_SIZE, 18)?;
    assert_eq!(actor.view.bindings[&FIRST.0].evaluations, 2);
    assert_eq!(actor.view.bindings[&FIRST.0].style_evaluations, 2);
    assert!(std::sync::Arc::ptr_eq(
        &icon,
        actor.view.bindings[&FIRST.0].icon.as_ref().unwrap()
    ));
    Ok(())
}
