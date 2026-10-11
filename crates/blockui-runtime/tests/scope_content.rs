#![cfg(feature = "testing")]
use blockui::*;
const CAPTION: State<String> = State::<String>::with_default("scope.caption", "first");
const SHAPE: State<i64> = State::<i64>::new("scope.shape", 0);
const SIZE: State<i64> = State::<i64>::new("scope.size", 14);
const ROOT: State<i64> = State::<i64>::new("scope.root", 0);
const SET: Action = Action::new("scope.set", |cx, value| SHAPE.set(cx, value));
const SET_SIZE: Action = Action::new("scope.size.set", |cx, value| SIZE.set(cx, value));
const SET_ROOT: Action = Action::new("scope.root.set", |cx, value| ROOT.set(cx, value));
const SCROLL: Scope = Scope::new("scope.scroll", |cx| {
    let shape = SHAPE.get(cx)?;
    let prefix = format!("caption {shape}");
    Ok(ScrollView {
        text: Binding::captured(move |cx| Ok(format!("{prefix}: {}", CAPTION.get(cx)?))),
        ..default()
    }
    .children(vec![
        TextInput::new(CAPTION).key("scope.editor").into_view(),
    ])
    .style_reactive(|cx| {
        Ok(ViewStyle {
            width: SIZE.get(cx)? as f32,
            ..default()
        })
    }))
});
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[CAPTION.spec(), SHAPE.spec(), SIZE.spec(), ROOT.spec()];
    const ACTIONS: &'static [Action] = &[SET, SET_SIZE, SET_ROOT];
    const SCOPES: &'static [Scope] = &[SCROLL];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let _ = ROOT.get(cx)?;
        Ok(Column::default().children(vec![SCROLL.into_view()]))
    }
}
#[test]
fn scope_root_content_and_style_survive_cache_and_structural_refresh()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    let root = Key::named("scope.scroll").0;
    let editor = Key::named("scope.editor").0;
    let dispatch = |actor: &mut blockui_runtime::Actor, action: Action, value| {
        actor.dispatch(blockui_runtime::Message {
            action: action.id,
            value,
        })
    };
    assert_eq!(actor.view.bindings[&root].text, "caption 0: first");
    let initial = actor.view.bindings[&root].clone();
    actor.edit_text(editor, "second")?;
    assert_eq!(actor.view.bindings[&root].text, "caption 0: second");
    assert_eq!(
        actor.view.bindings[&root].style_evaluations,
        initial.style_evaluations
    );
    dispatch(&mut actor, SET_SIZE, 20)?;
    assert_eq!(actor.view.bindings[&root].node.presentation.width, 20.0);
    assert_eq!(actor.view.bindings[&root].text, "caption 0: second");
    dispatch(&mut actor, SET_ROOT, 1)?;
    assert_eq!(actor.view.bindings[&root].text, "caption 0: second");
    dispatch(&mut actor, SET, 1)?;
    assert_eq!(actor.view.bindings[&root].text, "caption 1: second");
    assert_eq!(actor.view.bindings[&root].node.presentation.width, 20.0);
    actor.replace_for_test::<App>()?;
    actor.edit_text(editor, "third")?;
    assert_eq!(actor.view.bindings[&root].text, "caption 1: third");
    Ok(())
}

thread_local! {
    static LIVE_CAPTIONS: std::rc::Rc<std::cell::Cell<usize>> = Default::default();
}
struct CaptionLease(std::rc::Rc<std::cell::Cell<usize>>);
impl CaptionLease {
    fn new() -> Self {
        LIVE_CAPTIONS.with(|count| {
            count.set(count.get() + 1);
            Self(count.clone())
        })
    }
}
impl Drop for CaptionLease {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}
fn conditional_root(cx: &Cx<'_>) -> Result<View> {
    let shape = SHAPE.get(cx)?;
    if shape == 1 {
        return Ok(View::empty());
    }
    let lease = CaptionLease::new();
    Ok(ScrollView {
        text: Binding::captured(move |cx| {
            let _ = &lease;
            if shape == 4 {
                return Err(Error::Application("reject root caption".into()));
            }
            Ok(format!("generation {shape}: {}", CAPTION.get(cx)?))
        }),
        ..default()
    }
    .children(vec![
        TextInput::new(CAPTION).key("scope.editor").into_view(),
    ])
    .style_reactive(move |_| {
        Ok(ViewStyle {
            width: if shape == 2 { f32::NAN } else { 200.0 },
            ..default()
        })
    }))
}
const CONDITIONAL: Scope = Scope::new("scope.conditional", conditional_root);
struct Conditional<const CAPTURED: bool>;
impl<const CAPTURED: bool> Actor for Conditional<CAPTURED> {
    const STATE: &'static [StateSpec] = App::STATE;
    const ACTIONS: &'static [Action] = App::ACTIONS;
    const SCOPES: &'static [Scope] = if CAPTURED { &[] } else { &[CONDITIONAL] };
    fn view(cx: &Cx<'_>) -> Result<View> {
        let _ = ROOT.get(cx)?;
        let scope = if CAPTURED {
            Scope::when(
                "scope.conditional",
                |cx| Ok(SHAPE.get(cx)? != 1),
                conditional_root,
            )
        } else {
            CONDITIONAL.into_view()
        };
        Ok(Column::default().children(vec![scope]))
    }
}
fn assert_root_capture_lifecycle<const CAPTURED: bool>() -> std::result::Result<(), String> {
    let live = || LIVE_CAPTIONS.with(|count| count.get());
    assert_eq!(live(), 0);
    let mut actor = blockui_runtime::Actor::for_test::<Conditional<CAPTURED>>()?;
    let root = Key::named("scope.conditional").0;
    let send = |actor: &mut blockui_runtime::Actor, action: Action, value| {
        actor.dispatch(blockui_runtime::Message {
            action: action.id,
            value,
        })
    };
    assert_eq!(live(), 1);
    assert_eq!(actor.view.bindings[&root].text, "generation 0: first");
    assert!(send(&mut actor, SET, 2).is_err());
    assert_eq!(actor.integer("scope.shape"), 0);
    assert_eq!(actor.view.bindings[&root].text, "generation 0: first");
    assert_eq!(
        live(),
        1,
        "failed root must retire staged captures, retaining live ones"
    );
    assert!(send(&mut actor, SET, 4).is_err());
    assert_eq!(actor.integer("scope.shape"), 0);
    assert_eq!(actor.view.bindings[&root].text, "generation 0: first");
    assert_eq!(
        live(),
        1,
        "failed text evaluation must release the staged root callback"
    );
    send(&mut actor, SET_ROOT, 1)?;
    assert_eq!(
        live(),
        1,
        "cached/replaced builder parameters must not leak root content"
    );
    send(&mut actor, SET, 3)?;
    assert_eq!(actor.view.bindings[&root].text, "generation 3: first");
    assert_eq!(live(), 1);
    actor.replace_for_test::<Conditional<CAPTURED>>()?;
    assert_eq!(
        live(),
        1,
        "replacement retires the previous implementation's captures"
    );
    send(&mut actor, SET, 1)?;
    assert_eq!(live(), 0, "closed root must release its former caption");
    assert_eq!(actor.view.bindings[&root].node.semantics.hidden, 1);
    assert!(actor.view.bindings[&root].text_dependencies.is_empty());
    send(&mut actor, SET, 0)?;
    assert_eq!(live(), 1);
    assert_eq!(actor.view.bindings[&root].text, "generation 0: first");
    drop(actor);
    assert_eq!(live(), 0, "disposal must release all root captures");
    Ok(())
}
#[test]
fn declared_scope_root_captures_retire_transactionally() -> std::result::Result<(), String> {
    assert_root_capture_lifecycle::<false>()
}
#[test]
fn captured_scope_root_captures_retire_transactionally() -> std::result::Result<(), String> {
    assert_root_capture_lifecycle::<true>()
}
