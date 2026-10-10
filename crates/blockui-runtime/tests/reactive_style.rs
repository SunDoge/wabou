#![cfg(feature = "testing")]
use blockui::*;
use std::cell::Cell;

const ACTIVE: State<bool> = State::<bool>::new("style.active", true);
const LEFT: State<i64> = State::<i64>::new("style.left", 18);
const RIGHT: State<i64> = State::<i64>::new("style.right", 24);
const TEXT: State<i64> = State::<i64>::new("style.text", 0);
const LABEL: Key = Key::named("style.label");
const OTHER: Key = Key::named("style.other");
thread_local! { static PROJECTIONS: Cell<usize> = const { Cell::new(0) }; }
const SET_LEFT: Action = Action::new("style.set-left", |cx, n| LEFT.set(cx, n));
const SET_RIGHT: Action = Action::new("style.set-right", |cx, n| RIGHT.set(cx, n));
const SET_TEXT: Action = Action::new("style.set-text", |cx, n| TEXT.set(cx, n));
const SWITCH: Action = Action::new("style.switch", |cx, n| ACTIVE.set(cx, n != 0));
const BATCH: Action = Action::new("style.batch", |cx, n| {
    LEFT.set(cx, n)?;
    RIGHT.set(cx, n + 1)?;
    TEXT.set(cx, n)
});
fn appearance(cx: &Cx<'_>) -> Result<ViewStyle> {
    Ok(ViewStyle {
        font_size: if ACTIVE.get(cx)? {
            LEFT.get(cx)?
        } else {
            RIGHT.get(cx)?
        } as f32,
        text_tone: TextTone::Accent,
        ..default()
    })
}
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[ACTIVE.spec(), LEFT.spec(), RIGHT.spec(), TEXT.spec()];
    const ACTIONS: &'static [Action] = &[SET_LEFT, SET_RIGHT, SET_TEXT, SWITCH, BATCH];
    fn view(_: &Cx<'_>) -> Result<View> {
        PROJECTIONS.with(|n| n.set(n.get() + 1));
        Ok(Column::default().children(vec![
            Text::new(TEXT).key(LABEL).style_reactive(appearance),
            Text::new("unchanged").key(OTHER).into_view(),
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
#[test]
fn style_and_text_are_independent_and_dynamic_dependencies_are_replaced()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    let counts = |actor: &blockui_runtime::Actor| {
        let b = &actor.view.bindings[&LABEL.0];
        (b.evaluations, b.style_evaluations)
    };
    let projections = PROJECTIONS.with(Cell::get);
    assert_eq!(counts(&actor), (1, 1));
    dispatch(&mut actor, SET_LEFT, 20)?;
    assert_eq!(
        actor.view.bindings[&LABEL.0].node.presentation.font_size,
        20.0
    );
    assert_eq!(counts(&actor), (1, 2));
    dispatch(&mut actor, SET_RIGHT, 28)?;
    assert_eq!(counts(&actor), (1, 2), "inactive branch is not subscribed");
    dispatch(&mut actor, SWITCH, 0)?;
    assert_eq!(
        actor.view.bindings[&LABEL.0].node.presentation.font_size,
        28.0
    );
    assert_eq!(counts(&actor), (1, 3));
    dispatch(&mut actor, SET_LEFT, 22)?;
    assert_eq!(counts(&actor), (1, 3), "old branch dependency is removed");
    dispatch(&mut actor, SET_TEXT, 7)?;
    assert_eq!(actor.view.bindings[&LABEL.0].text, "7");
    assert_eq!(
        counts(&actor),
        (2, 3),
        "text update skips style calculation"
    );
    dispatch(&mut actor, BATCH, 30)?;
    assert_eq!(
        counts(&actor),
        (3, 4),
        "one evaluation per observer per turn"
    );
    dispatch(&mut actor, BATCH, 30)?;
    assert_eq!(counts(&actor), (3, 4), "equal writes do not notify");
    assert_eq!(actor.view.bindings[&OTHER.0].evaluations, 1);
    assert_eq!(
        PROJECTIONS.with(Cell::get),
        projections,
        "root was not projected again"
    );
    Ok(())
}
#[test]
fn invalid_styles_roll_back_state_text_and_dependencies_atomically()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    let original = actor.view.bindings[&LABEL.0].clone();
    let error = dispatch(&mut actor, BATCH, 97).unwrap_err();
    assert!(error.contains("invalid widget presentation"), "{error}");
    assert_eq!(actor.integer("style.left"), 18);
    assert_eq!(actor.integer("style.text"), 0);
    let after = &actor.view.bindings[&LABEL.0];
    assert_eq!(after.node, original.node);
    assert_eq!(after.text, original.text);
    assert_eq!(after.dependencies, original.dependencies);
    assert_eq!(after.style_evaluations, original.style_evaluations);
    dispatch(&mut actor, SET_LEFT, 19)?;
    assert_eq!(
        actor.view.bindings[&LABEL.0].node.presentation.font_size,
        19.0
    );
    Ok(())
}
struct Replacement;
impl Actor for Replacement {
    const STATE: &'static [StateSpec] = App::STATE;
    const ACTIONS: &'static [Action] = App::ACTIONS;
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(
            Column::default().children(vec![Text::new(TEXT).key(LABEL).style_reactive(|cx| {
                Ok(ViewStyle {
                    font_size: RIGHT.get(cx)? as f32,
                    ..default()
                })
            })]),
        )
    }
}
#[test]
fn replacement_rebuilds_observers_and_preserves_native_state() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    actor.view.renderer.get_mut(&LABEL.0).unwrap().focus = true;
    let address = actor.state_address();
    actor.replace_for_test::<Replacement>()?;
    assert_eq!(actor.state_address(), address);
    assert!(actor.view.renderer[&LABEL.0].focus);
    assert!(!actor.view.bindings.contains_key(&OTHER.0));
    let evaluations = actor.view.bindings[&LABEL.0].style_evaluations;
    dispatch(&mut actor, SET_LEFT, 20)?;
    assert_eq!(actor.view.bindings[&LABEL.0].style_evaluations, evaluations);
    dispatch(&mut actor, SET_RIGHT, 30)?;
    assert_eq!(
        actor.view.bindings[&LABEL.0].node.presentation.font_size,
        30.0
    );
    Ok(())
}

struct Invalid<const MODE: u8>;
impl<const MODE: u8> Actor for Invalid<MODE> {
    const STATE: &'static [StateSpec] = &[LEFT.spec()];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(
            Column::default().children(vec![Text::new("invalid").style_reactive(
                |cx| match MODE {
                    0 => Ok(ViewStyle {
                        font_size: f32::NAN,
                        ..default()
                    }),
                    1 => Ok(ViewStyle {
                        surface: Surface::Panel,
                        ..default()
                    }),
                    2 => {
                        LEFT.set(cx, 22)?;
                        Ok(default())
                    }
                    _ => Err(Error::Application("style computation failed".into())),
                },
            )]),
        )
    }
}
#[test]
fn style_admission_rejects_nan_incompatible_properties_writes_and_callback_errors() {
    assert!(blockui_runtime::Actor::for_test::<Invalid<0>>().is_err());
    assert!(blockui_runtime::Actor::for_test::<Invalid<1>>().is_err());
    assert!(blockui_runtime::Actor::for_test::<Invalid<2>>().is_err());
    assert!(blockui_runtime::Actor::for_test::<Invalid<3>>().is_err());
}

thread_local! {
    static CAPTURE_LIVE: Cell<usize> = const { Cell::new(0) };
    static CAPTURE_DROPS: Cell<usize> = const { Cell::new(0) };
}
struct StyleCapture(String);
impl StyleCapture {
    fn new() -> Self {
        CAPTURE_LIVE.with(|n| n.set(n.get() + 1));
        Self("component-owned parameter".into())
    }
    fn read(&self) {
        assert!(!self.0.is_empty());
    }
}
impl Drop for StyleCapture {
    fn drop(&mut self) {
        CAPTURE_LIVE.with(|n| n.set(n.get() - 1));
        CAPTURE_DROPS.with(|n| n.set(n.get() + 1));
    }
}
const SHAPE: State<i64> = State::<i64>::new("captured.shape", 0);
const SET_SHAPE: Action = Action::new("captured.shape.set", |cx, n| SHAPE.set(cx, n));
struct Captured<const REPLACEMENT: bool>;
impl<const REPLACEMENT: bool> Actor for Captured<REPLACEMENT> {
    const STATE: &'static [StateSpec] = &[LEFT.spec(), RIGHT.spec(), SHAPE.spec()];
    const ACTIONS: &'static [Action] = &[SET_LEFT, SET_RIGHT, SET_SHAPE];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let shape = SHAPE.get(cx)?;
        if shape == 3 {
            return Ok(
                Column::default().children(vec![Text::new("no style").key(LABEL).into_view()])
            );
        }
        let state = if REPLACEMENT || shape == 1 {
            RIGHT
        } else {
            LEFT
        };
        let capture = StyleCapture::new();
        Ok(
            Column::default().children(vec![Text::new("component").key(LABEL).style_reactive(
                move |cx| {
                    capture.read();
                    Ok(ViewStyle {
                        font_size: if shape == 2 {
                            f32::NAN
                        } else {
                            state.get(cx)? as f32
                        },
                        ..default()
                    })
                },
            )]),
        )
    }
}
#[test]
fn captured_styles_track_local_handles_and_release_on_rollback_unmount_and_replacement()
-> std::result::Result<(), String> {
    CAPTURE_LIVE.with(|n| n.set(0));
    CAPTURE_DROPS.with(|n| n.set(0));
    let mut actor = blockui_runtime::Actor::for_test::<Captured<false>>()?;
    assert_eq!(CAPTURE_LIVE.with(Cell::get), 1);
    dispatch(&mut actor, SET_LEFT, 20)?;
    assert_eq!(
        actor.view.bindings[&LABEL.0].node.presentation.font_size,
        20.0
    );
    assert_eq!(
        CAPTURE_DROPS.with(Cell::get),
        0,
        "property update keeps capture"
    );
    assert!(dispatch(&mut actor, SET_SHAPE, 2).is_err());
    assert_eq!(actor.integer("captured.shape"), 0);
    assert_eq!(CAPTURE_LIVE.with(Cell::get), 1);
    assert_eq!(
        CAPTURE_DROPS.with(Cell::get),
        1,
        "failed candidate is disposed"
    );
    dispatch(&mut actor, SET_LEFT, 21)?;
    assert_eq!(
        actor.view.bindings[&LABEL.0].node.presentation.font_size,
        21.0
    );
    actor.replace_for_test::<Captured<true>>()?;
    assert_eq!(CAPTURE_LIVE.with(Cell::get), 1);
    assert_eq!(
        CAPTURE_DROPS.with(Cell::get),
        2,
        "retired implementation is disposed"
    );
    let count = actor.view.bindings[&LABEL.0].style_evaluations;
    dispatch(&mut actor, SET_LEFT, 22)?;
    assert_eq!(actor.view.bindings[&LABEL.0].style_evaluations, count);
    dispatch(&mut actor, SET_RIGHT, 30)?;
    assert_eq!(
        actor.view.bindings[&LABEL.0].node.presentation.font_size,
        30.0
    );
    dispatch(&mut actor, SET_SHAPE, 3)?;
    assert_eq!(CAPTURE_LIVE.with(Cell::get), 0);
    assert_eq!(
        CAPTURE_DROPS.with(Cell::get),
        3,
        "unmounted capture is disposed"
    );
    dispatch(&mut actor, SET_SHAPE, 0)?;
    assert_eq!(CAPTURE_LIVE.with(Cell::get), 1);
    drop(actor);
    assert_eq!(
        CAPTURE_LIVE.with(Cell::get),
        0,
        "actor disposal releases captures"
    );
    Ok(())
}
