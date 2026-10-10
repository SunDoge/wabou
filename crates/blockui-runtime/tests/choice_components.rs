#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::{Actor as HostActor, ActorId, Runtime};
const SELECTED: State<i64> = State::<i64>::new("choice.selected", 0);
const MODE: State<i64> = State::<i64>::new("choice.mode", 0);
const BAD: Action = Action::new("choice.bad", |cx, value| {
    MODE.set(cx, value)?;
    if value == 2 {
        SELECTED.set(cx, 99)?;
    }
    Ok(())
});
struct ScopedChoices;
fn path() -> InstancePath {
    InstancePath::EMPTY.scoped(Key::named("instance"))
}
fn tab(value: u64) -> u64 {
    path().resolve(Key::named("tabs").scoped(value).0)
}
impl Actor for ScopedChoices {
    const STATE: &'static [StateSpec] = &[SELECTED.spec(), MODE.spec()];
    const ACTIONS: &'static [Action] = &[BAD];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let mode = MODE.get(cx)?;
        let view = Tabs::new("Settings", SELECTED).key("tabs").children(
            cx,
            vec![
                Tab::new(0, "Account", Text::new("Account details").key("account")),
                Tab::new(
                    if mode == 1 { 0 } else { 2 },
                    "Security",
                    Text::new("Security details").key("security"),
                ),
            ],
        )?;
        let view = __component_view(view, path(), Key::named("instance.root"));
        Ok(Column::default().key("root").children(vec![view]))
    }
}
struct Parent;
impl Actor for Parent {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok(Column::default().children(vec![
            ChildActor {
                key: "first".into(),
                implementation: Implementation::named("choices"),
            }
            .into_view(),
            ChildActor {
                key: "second".into(),
                implementation: Implementation::named("choices"),
            }
            .into_view(),
        ]))
    }
}
#[test]
fn scoped_choice_relationships_resolve_and_duplicate_values_roll_back()
-> std::result::Result<(), String> {
    let mut actor = HostActor::for_test::<ScopedChoices>()?;
    let controls = actor.projected_view().bindings[&tab(0)]
        .node
        .semantics
        .controls;
    let panel = &actor.projected_view().bindings[&controls];
    assert_eq!(panel.node.semantics.labelled_by, tab(0));
    actor.dispatch_widget(tab(2))?;
    assert_eq!(actor.integer("choice.selected"), 2);
    let before = actor.projected_view().order.clone();
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: abi::key("choice.bad"),
                value: 1
            })
            .is_err()
    );
    assert_eq!(actor.integer("choice.mode"), 0);
    assert_eq!(actor.projected_view().order, before);
    assert!(
        actor
            .dispatch(blockui_runtime::Message {
                action: abi::key("choice.bad"),
                value: 2
            })
            .is_err()
    );
    assert_eq!(actor.integer("choice.selected"), 2);
    assert_eq!(actor.integer("choice.mode"), 0);
    actor.dispatch_widget(tab(0))?;
    assert_eq!(actor.integer("choice.selected"), 0);
    Ok(())
}
#[test]
fn family_composition_remaps_semantic_links_per_child() -> std::result::Result<(), String> {
    let directory =
        std::env::temp_dir().join(format!("blockui-choice-family-{}", std::process::id()));
    let mut runtime = Runtime::new(&directory);
    runtime.register_for_test::<ScopedChoices>("choices")?;
    let root = ActorId(802);
    runtime.spawn_for_test::<Parent>(root)?;
    let first = runtime.child(root, abi::key("first")).unwrap();
    let second = runtime.child(root, abi::key("second")).unwrap();
    let composed = runtime.compose(root)?;
    let a = composed.widget(first, tab(0)).unwrap();
    let b = composed.widget(second, tab(0)).unwrap();
    assert_ne!(a, b);
    for child in [first, second] {
        let tab = composed.widget(child, tab(0)).unwrap();
        let panel = composed.view.bindings[&tab].node.semantics.controls;
        assert_eq!(
            composed.view.bindings[&panel].node.semantics.labelled_by,
            tab
        );
    }
    composed.dispatch(&mut runtime, b)?;
    drop(composed);
    drop(runtime);
    assert_eq!(
        std::fs::read_dir(&directory)
            .map_err(|e| e.to_string())?
            .count(),
        0
    );
    std::fs::remove_dir(directory).map_err(|e| e.to_string())?;
    Ok(())
}
