#![cfg(feature = "testing")]
use blockui::*;
use blockui_runtime::{Actor as HostActor, Message};
const SELECTED: State<i64> = State::<i64>::new("disclosure.selection", 0);
const MANY: State<Vec<i64>> = State::<Vec<i64>>::with_default("disclosure.many", &[0]);
const OPEN: State<bool> = State::<bool>::new("disclosure.open", false);
const MODE: State<i64> = State::<i64>::new("disclosure.mode", 0);
const BAD: Action = Action::new("disclosure.bad", |cx, value| match value {
    1 => SELECTED.set(cx, 99),
    2 => MANY.set(cx, &[0, 0]),
    3 => MANY.set(cx, &[99]),
    _ => MODE.set(cx, value),
});
struct Controlled;
fn path() -> InstancePath {
    InstancePath::EMPTY.scoped(Key::named("instance"))
}
fn trigger(value: u64) -> u64 {
    path().resolve(
        Key::named("single")
            .scoped(value)
            .scoped(abi::key("trigger"))
            .0,
    )
}
impl Actor for Controlled {
    const STATE: &'static [StateSpec] = &[SELECTED.spec(), MANY.spec(), MODE.spec(), OPEN.spec()];
    const ACTIONS: &'static [Action] = &[BAD];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let mode = MODE.get(cx)?;
        let view = Column::default().key("component").children(vec![
            Accordion::single(SELECTED)
                .enabled(mode != 7)
                .key("single")
                .heading_level(if mode == 4 { 0 } else { 3 })
                .children(
                    cx,
                    vec![
                        AccordionItem::new(0, "First", Text::new("First content").key("first")),
                        AccordionItem::new(
                            if mode == 5 {
                                0
                            } else if mode == 6 {
                                Accordion::NONE
                            } else {
                                1
                            },
                            "Second",
                            Text::new("Second content").key("second"),
                        ),
                    ],
                )?,
            Accordion::multiple(MANY).key("multiple").children(
                cx,
                vec![
                    AccordionItem::new(0, "Many first", Text::new("Many first details")),
                    AccordionItem::new(1, "Many second", Text::new("Many second details")),
                ],
            )?,
            Collapsible::new("Managed disclosure", OPEN)
                .enabled(false)
                .key("disabled")
                .children(cx, Text::new("Managed content"))?,
        ]);
        Ok(Column::default().children(vec![__component_view(
            view,
            path(),
            Key::named("instance.root"),
        )]))
    }
}
#[test]
fn invalid_selection_structure_and_heading_level_roll_back_atomically()
-> std::result::Result<(), String> {
    let mut actor = HostActor::for_test::<Controlled>()?;
    let before = actor.projected_view().order.clone();
    for value in 1..=6 {
        assert!(
            actor
                .dispatch(Message {
                    action: BAD.id,
                    value
                })
                .is_err(),
            "invalid mode {value} was accepted"
        );
        assert_eq!(actor.integer("disclosure.selection"), 0);
        assert_eq!(actor.integer("disclosure.mode"), 0);
        assert_eq!(actor.list("disclosure.many"), &[0]);
        assert_eq!(actor.projected_view().order, before);
    }
    let first = trigger(0);
    assert_eq!(
        actor.projected_view().bindings[&first].node.disabled,
        0,
        "locked headers stay focusable"
    );
    assert_eq!(
        actor.projected_view().bindings[&first]
            .node
            .semantics
            .locked,
        1
    );
    actor.dispatch_widget(first)?;
    assert_eq!(
        actor.integer("disclosure.selection"),
        0,
        "non-collapsible single retains its open item"
    );
    actor.dispatch_widget(trigger(1))?;
    assert_eq!(actor.integer("disclosure.selection"), 1);
    let controls = actor.projected_view().bindings[&trigger(1)]
        .node
        .semantics
        .controls;
    assert_eq!(
        actor.projected_view().bindings[&controls]
            .node
            .semantics
            .labelled_by,
        trigger(1)
    );
    assert_eq!(
        actor.projected_view().bindings[&controls]
            .node
            .semantics
            .hidden,
        0
    );
    Ok(())
}

#[test]
fn disabled_group_and_collapsible_do_not_mutate_controlled_state() -> std::result::Result<(), String>
{
    let mut actor = HostActor::for_test::<Controlled>()?;
    actor.dispatch(Message {
        action: BAD.id,
        value: 7,
    })?;
    actor.dispatch_widget(trigger(1))?;
    assert_eq!(actor.integer("disclosure.selection"), 0);
    let disabled = path().resolve(Key::named("disabled").scoped(abi::key("trigger")).0);
    actor.dispatch_widget(disabled)?;
    assert!(!actor.boolean("disclosure.open"));
    Ok(())
}

#[test]
fn composed_children_have_independent_disclosure_relationships_and_selection()
-> std::result::Result<(), String> {
    use blockui_runtime::{ActorId, Runtime};
    struct Parent;
    impl Actor for Parent {
        const STATE: &'static [StateSpec] = &[];
        const ACTIONS: &'static [Action] = &[];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(
                ["first", "second"]
                    .into_iter()
                    .map(|name| {
                        ChildActor {
                            key: name.into(),
                            implementation: Implementation::named("controlled"),
                        }
                        .into_view()
                    })
                    .collect(),
            ))
        }
    }
    let directory =
        std::env::temp_dir().join(format!("blockui-disclosure-family-{}", std::process::id()));
    let mut runtime = Runtime::new(&directory);
    runtime.register_for_test::<Controlled>("controlled")?;
    let root = ActorId(803);
    runtime.spawn_for_test::<Parent>(root)?;
    let first = runtime.child(root, abi::key("first")).unwrap();
    let second = runtime.child(root, abi::key("second")).unwrap();
    let composed = runtime.compose(root)?;
    for child in [first, second] {
        let trigger = composed.widget(child, trigger(0)).unwrap();
        let panel = composed.view.bindings[&trigger].node.semantics.controls;
        assert_eq!(
            composed.view.bindings[&panel].node.semantics.labelled_by,
            trigger
        );
    }
    composed.dispatch(&mut runtime, composed.widget(second, trigger(1)).unwrap())?;
    drop(composed);
    let after = runtime.compose(root)?;
    assert_eq!(
        after.view.bindings[&after.widget(first, trigger(0)).unwrap()]
            .node
            .semantics
            .expanded,
        2
    );
    assert_eq!(
        after.view.bindings[&after.widget(second, trigger(0)).unwrap()]
            .node
            .semantics
            .expanded,
        1
    );
    assert_eq!(
        after.view.bindings[&after.widget(second, trigger(1)).unwrap()]
            .node
            .semantics
            .expanded,
        2
    );
    drop(after);
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
