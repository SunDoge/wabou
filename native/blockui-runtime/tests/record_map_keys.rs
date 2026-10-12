#![cfg(feature = "testing")]
use blockui::*;
use std::cell::Cell;
#[derive(Record)]
struct Row {
    name: String,
}
const STORAGE: State<Vec<String>> = State::<Vec<String>>::new("keys.rows");
const ROWS: RecordMap<Row> = RecordMap::new(STORAGE);
const KEYS: Computed<Vec<String>> =
    Computed::<Vec<String>>::new("keys.membership", |cx| ROWS.keys(cx));
const CHANGE: Action = Action::new("keys.change", |cx, mode| match mode {
    0 => {
        ROWS.insert(
            cx,
            "beta",
            &Row {
                name: "Beta".into(),
            },
        )?;
        ROWS.insert(
            cx,
            "alpha",
            &Row {
                name: "Alpha".into(),
            },
        )
    }
    1 => ROWS.insert(
        cx,
        "alpha",
        &Row {
            name: "Changed".into(),
        },
    ),
    2 => ROWS.insert(
        cx,
        "gamma",
        &Row {
            name: "Gamma".into(),
        },
    ),
    3 => ROWS.remove(cx, "beta"),
    _ => STORAGE.set(
        cx,
        &[
            "duplicate".into(),
            "bad".into(),
            "duplicate".into(),
            "bad".into(),
        ],
    ),
});
thread_local! { static BUILDS: Cell<usize> = const { Cell::new(0) }; }
const LIST: Scope = Scope::new("keys.list", |cx| {
    BUILDS.with(|count| count.set(count.get() + 1));
    let children = KEYS
        .get(cx)?
        .into_iter()
        .map(|key| {
            let identity = Key::named("row").scoped(Key::named(&key).0);
            Text::reactive(move |cx| {
                ROWS.field(cx, &key, Row::name_field())?
                    .ok_or(Error::Application("missing row".into()))
            })
            .key(identity)
            .into_view()
        })
        .collect();
    Ok(Column::default().children(children))
});
struct App;
impl Actor for App {
    const STATE: &'static [StateSpec] = &[STORAGE.spec()];
    const ACTIONS: &'static [Action] = &[CHANGE];
    const COMPUTEDS: &'static [ComputedSpec] = &[KEYS.spec()];
    const SCOPES: &'static [Scope] = &[LIST];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![LIST.into_view()]))
    }
}
#[test]
fn memoized_keys_separate_membership_from_row_field_updates() -> std::result::Result<(), String> {
    BUILDS.with(|count| count.set(0));
    let mut actor = blockui_runtime::Actor::for_test::<App>()?;
    let change = |actor: &mut blockui_runtime::Actor, value| {
        actor.dispatch(blockui_runtime::Message {
            action: CHANGE.id,
            value,
        })
    };
    assert_eq!(BUILDS.with(Cell::get), 1);
    change(&mut actor, 0)?;
    assert_eq!(BUILDS.with(Cell::get), 2);
    let row = |name: &str| Key::named("row").scoped(Key::named(name).0).0;
    let alpha = row("alpha");
    let beta = row("beta");
    let view = actor.projected_view();
    assert!(
        view.order.iter().position(|key| *key == alpha)
            < view.order.iter().position(|key| *key == beta)
    );
    let before_alpha = view.bindings[&alpha].evaluations;
    let before_beta = view.bindings[&beta].evaluations;
    actor.view.renderer.get_mut(&alpha).unwrap().selection = (1, 3);
    change(&mut actor, 1)?;
    assert_eq!(
        BUILDS.with(Cell::get),
        2,
        "field edits must not rebuild list structure"
    );
    assert_eq!(actor.view.bindings[&alpha].text, "Changed");
    assert_eq!(actor.view.bindings[&alpha].evaluations, before_alpha + 1);
    assert_eq!(actor.view.bindings[&beta].evaluations, before_beta);
    let memo_evaluations = actor.computed_evaluations("keys.membership");
    change(&mut actor, 1)?;
    assert_eq!(
        actor.computed_evaluations("keys.membership"),
        memo_evaluations
    );
    change(&mut actor, 2)?;
    assert_eq!(BUILDS.with(Cell::get), 3);
    assert_eq!(actor.view.bindings[&row("gamma")].text, "Gamma");
    assert_eq!(actor.view.renderer[&alpha].selection, (1, 3));
    change(&mut actor, 3)?;
    assert_eq!(BUILDS.with(Cell::get), 4);
    assert!(!actor.view.bindings.contains_key(&beta));
    let storage = actor.text_list("keys.rows").to_vec();
    assert!(change(&mut actor, 4).is_err());
    assert_eq!(actor.text_list("keys.rows"), storage);
    assert_eq!(actor.view.bindings[&alpha].text, "Changed");
    actor.replace_for_test::<App>()?;
    change(&mut actor, 0)?;
    assert_eq!(actor.view.bindings[&alpha].text, "Alpha");
    assert_eq!(actor.view.bindings[&beta].text, "Beta");
    Ok(())
}
