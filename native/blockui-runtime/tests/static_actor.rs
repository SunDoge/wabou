#![cfg(feature = "testing")]
use blockui::{Action, Actor, Column, Cx, Error, Result, State, StateSpec, Text, View};

const COUNT: State<i64> = State::<i64>::new("count", 0);
const TITLE: State<String> = State::<String>::with_default("title", "draft");
const ITEMS: State<Vec<i64>> = State::<Vec<i64>>::new("items");
struct BoundUpdates;
impl Actor for BoundUpdates {
    const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec(), ITEMS.spec()];
    const ACTIONS: &'static [Action] = &[Action::new("update", |cx, value| {
        COUNT.bind(cx)?.update(|count| Ok(count + 1))?;
        let result = TITLE.bind(cx)?.update(|title| {
            title.push('!');
            if value < 0 {
                return Err(Error::Overflow);
            }
            Ok(())
        });
        if value == -2 {
            return Ok(());
        }
        result?;
        ITEMS.bind(cx)?.update(|items| {
            items.push(value);
            if value == 3 {
                return Err(Error::Overflow);
            }
            Ok(())
        })
    })];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Counter::view(cx)
    }
}
#[test]
fn bound_updates_publish_owned_values_only_on_success() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<BoundUpdates>()?;
    let update = |value| blockui_runtime::Message {
        action: blockui::abi::key("update"),
        value,
    };
    actor.dispatch(update(1))?;
    assert_eq!(actor.integer("count"), 1);
    assert_eq!(actor.text("title"), "draft!");
    assert_eq!(actor.list("items"), &[1]);
    assert!(actor.dispatch(update(-1)).is_err());
    assert_eq!(actor.integer("count"), 1);
    assert_eq!(actor.text("title"), "draft!");
    actor.dispatch(update(-2))?;
    assert_eq!(actor.integer("count"), 2);
    assert_eq!(actor.text("title"), "draft!");
    assert!(actor.dispatch(update(3)).is_err());
    assert_eq!(actor.integer("count"), 2);
    assert_eq!(actor.text("title"), "draft!");
    assert_eq!(actor.list("items"), &[1]);
    Ok(())
}
const STEP: Action = Action::new("step", |cx, value| {
    COUNT.set(cx, COUNT.get(cx)? + 1)?;
    if value < 0 {
        return Err(Error::Overflow);
    }
    Ok(())
});
struct Counter;
impl Actor for Counter {
    const STATE: &'static [StateSpec] = &[COUNT.spec()];
    const ACTIONS: &'static [Action] = &[STEP];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(COUNT.get(cx)?.to_string()).key("count.text"));
            }
            __blockui_tree.finish()
        })
    }
}
struct Replacement;
impl Actor for Replacement {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = &[Action::new("step", |cx, _| {
        COUNT.set(cx, COUNT.get(cx)? + 10)
    })];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Counter::view(cx)
    }
}
struct Rejected;
impl Actor for Rejected {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = Counter::ACTIONS;
    fn view(_: &Cx<'_>) -> Result<View> {
        Err(Error::InvalidView("test rejection"))
    }
}

const SEND: Action = Action::new("send", |cx, value| {
    blockui::ActorRef(302).tell(cx, STEP, 0)?;
    if value < 0 {
        return Err(Error::Overflow);
    }
    Ok(())
});
struct Sender;
impl Actor for Sender {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = &[SEND];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Counter::view(cx)
    }
}
#[test]
fn compiled_in_actors_route_transactional_messages_through_replacement()
-> std::result::Result<(), String> {
    let root = std::env::temp_dir().join(format!("identity-static-routing-{}", std::process::id()));
    let mut runtime = blockui_runtime::Runtime::new(&root);
    let source = blockui_runtime::ActorId(301);
    let target = blockui_runtime::ActorId(302);
    runtime.spawn_for_test::<Sender>(source)?;
    runtime.spawn_for_test::<Counter>(target)?;
    assert!(runtime.spawn_for_test::<Counter>(target).is_err());
    let slot = runtime.actor(target).unwrap().slot_address("count");
    runtime.tell(
        source,
        blockui_runtime::Message {
            action: SEND.id,
            value: -1,
        },
    )?;
    assert_eq!(runtime.tick().len(), 1);
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(target).unwrap().integer("count"), 0);
    runtime.tell(
        source,
        blockui_runtime::Message {
            action: SEND.id,
            value: 0,
        },
    )?;
    runtime.replace_for_test::<Replacement>(target)?;
    for _ in 0..3 {
        assert!(runtime.tick().is_empty());
    }
    assert_eq!(runtime.actor(target).unwrap().integer("count"), 10);
    assert_eq!(runtime.actor(target).unwrap().slot_address("count"), slot);
    assert!(runtime.replace_for_test::<Rejected>(target).is_err());
    runtime.tell(
        source,
        blockui_runtime::Message {
            action: SEND.id,
            value: 0,
        },
    )?;
    for _ in 0..3 {
        assert!(runtime.tick().is_empty());
    }
    assert_eq!(runtime.actor(target).unwrap().integer("count"), 20);
    drop(runtime);
    std::fs::remove_dir(root).map_err(|error| error.to_string())?;
    Ok(())
}
fn message(value: i64) -> blockui_runtime::Message {
    blockui_runtime::Message {
        action: blockui::abi::key("step"),
        value,
    }
}

const BOOLEAN: blockui::BoolAction =
    blockui::BoolAction::new("boolean", |cx, value| COUNT.set(cx, i64::from(value)));
const FLOAT: blockui::FloatAction =
    blockui::FloatAction::new("float", |cx, value| COUNT.set(cx, value.to_bits() as i64));
struct Scalars;
impl Actor for Scalars {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = &[BOOLEAN.spec(), FLOAT.spec()];
    fn view(cx: &Cx<'_>) -> Result<View> {
        Counter::view(cx)
    }
}
#[test]
fn typed_test_messages_keep_boolean_and_float_encoding() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Scalars>()?;
    actor.tell(BOOLEAN, true)?;
    actor.tick()?;
    assert_eq!(actor.integer("count"), 1);
    for bits in [
        (-0.0_f64).to_bits(),
        0xfff8_0000_0000_1234,
        f64::INFINITY.to_bits(),
    ] {
        actor.tell(FLOAT, f64::from_bits(bits))?;
        actor.tick()?;
        assert_eq!(actor.integer("count") as u64, bits);
    }
    Ok(())
}

#[test]
fn compiled_in_actor_uses_real_transactions_and_replacement() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<Counter>()?;
    let other = blockui_runtime::Actor::for_test::<Counter>()?;
    actor.dispatch(message(0))?;
    assert_eq!(actor.integer("count"), 1);
    assert_eq!(other.integer("count"), 0);
    let address = actor.slot_address("count");
    assert!(actor.dispatch(message(-1)).is_err());
    assert_eq!(actor.integer("count"), 1);
    assert_eq!(
        actor.view.bindings[&blockui::abi::key("count.text")].text,
        "1"
    );
    let version = actor.implementation_version();
    assert!(actor.replace_for_test::<Rejected>().is_err());
    assert_eq!(actor.implementation_version(), version);
    actor.tell(STEP, 0)?;
    actor.replace_for_test::<Replacement>()?;
    assert_eq!(actor.slot_address("count"), address);
    assert_eq!(actor.integer("count"), 1);
    actor.tick()?;
    assert_eq!(actor.integer("count"), 11);
    assert_eq!(
        actor.view.bindings[&blockui::abi::key("count.text")].text,
        "11"
    );
    Ok(())
}

static MAPPED_VIEWS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
struct MappedCounter;
impl Actor for MappedCounter {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = Counter::ACTIONS;
    fn view(_: &Cx<'_>) -> Result<View> {
        MAPPED_VIEWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Text {
                        text: COUNT.map_text(|count| format!("Count: {count}")),
                        ..Default::default()
                    }
                    .key("mapped.count"),
                );
                __blockui_tree.add(Text::new("Unchanged").key("mapped.static"));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn mapped_text_updates_without_rebuilding_view() -> std::result::Result<(), String> {
    use std::sync::atomic::Ordering;
    let mut actor = blockui_runtime::Actor::for_test::<MappedCounter>()?;
    let renders = MAPPED_VIEWS.load(Ordering::Relaxed);
    for count in 1..=3 {
        actor.dispatch(message(0))?;
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("mapped.count")].text,
            format!("Count: {count}")
        );
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("mapped.static")].text,
            "Unchanged"
        );
        assert_eq!(MAPPED_VIEWS.load(Ordering::Relaxed), renders);
    }
    assert!(actor.dispatch(message(-1)).is_err());
    assert_eq!(
        actor.view.bindings[&blockui::abi::key("mapped.count")].text,
        "Count: 3"
    );
    Ok(())
}

const RATIO: State<f64> = State::<f64>::new("ratio", 0.5);
const ENABLED: State<bool> = State::<bool>::new("enabled", false);
struct MappedValues<const UPDATED: bool>;
impl<const UPDATED: bool> Actor for MappedValues<UPDATED> {
    const STATE: &'static [StateSpec] = &[RATIO.spec(), ENABLED.spec(), TITLE.spec(), ITEMS.spec()];
    const ACTIONS: &'static [Action] = &[Action::new("change", |cx, _| {
        RATIO.set(cx, -0.0)?;
        ENABLED.set(cx, true)?;
        TITLE.set(cx, "你好 🌍")?;
        ITEMS.set(cx, &[3, 5])
    })];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Text {
                        text: RATIO.map_text(|v| format!("{}:{v}", UPDATED)),
                        ..Default::default()
                    }
                    .key("ratio.text"),
                );
                __blockui_tree.add(
                    Text {
                        text: ENABLED.map_text(|v| format!("{}:{v}", UPDATED)),
                        ..Default::default()
                    }
                    .key("enabled.text"),
                );
                __blockui_tree.add(
                    Text {
                        text: TITLE.map_text(|v| format!("{}:{v}", UPDATED)),
                        ..Default::default()
                    }
                    .key("title.text"),
                );
                __blockui_tree.add(
                    Text {
                        text: ITEMS.map_text(|v| format!("{}:{v:?}", UPDATED)),
                        ..Default::default()
                    }
                    .key("items.text"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn all_mapped_types_keep_values_and_replace_formatters() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<MappedValues<false>>()?;
    actor.dispatch(blockui_runtime::Message {
        action: blockui::abi::key("change"),
        value: 0,
    })?;
    let slot = actor.slot_address("title");
    for (key, value) in [
        ("ratio.text", "-0"),
        ("enabled.text", "true"),
        ("title.text", "你好 🌍"),
        ("items.text", "[3, 5]"),
    ] {
        assert_eq!(
            actor.view.bindings[&blockui::abi::key(key)].text,
            format!("false:{value}")
        );
    }
    actor.replace_for_test::<MappedValues<true>>()?;
    assert_eq!(actor.slot_address("title"), slot);
    for (key, value) in [
        ("ratio.text", "-0"),
        ("enabled.text", "true"),
        ("title.text", "你好 🌍"),
        ("items.text", "[3, 5]"),
    ] {
        assert_eq!(
            actor.view.bindings[&blockui::abi::key(key)].text,
            format!("true:{value}")
        );
    }
    Ok(())
}

struct WrongMappedKind;
impl Actor for WrongMappedKind {
    const STATE: &'static [StateSpec] = MappedValues::<false>::STATE;
    const ACTIONS: &'static [Action] = MappedValues::<false>::ACTIONS;
    fn view(_: &Cx<'_>) -> Result<View> {
        let wrong = State::<f64>::new("title", 0.0);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    Text {
                        text: wrong.map_text(|v| v.to_string()),
                        ..Default::default()
                    }
                    .key("title.text"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn mapped_kind_mismatch_rejects_candidate_and_keeps_current_actor()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<MappedValues<false>>()?;
    let version = actor.implementation_version();
    assert!(actor.replace_for_test::<WrongMappedKind>().is_err());
    assert_eq!(actor.implementation_version(), version);
    actor.dispatch(blockui_runtime::Message {
        action: blockui::abi::key("change"),
        value: 0,
    })?;
    assert_eq!(
        actor.view.bindings[&blockui::abi::key("title.text")].text,
        "false:你好 🌍"
    );
    Ok(())
}

struct TestBranch;
impl Actor for TestBranch {
    const STATE: &'static [StateSpec] = &[];
    const ACTIONS: &'static [Action] = &[];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(blockui::ChildActor {
                    key: "branch.leaf".into(),
                    implementation: blockui::Implementation::named("leaf"),
                });
            }
            __blockui_tree.finish()
        })
    }
}
struct TestTree<const BROKEN: bool>;
impl<const BROKEN: bool> Actor for TestTree<BROKEN> {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = Counter::ACTIONS;
    fn view(cx: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(COUNT.get(cx)?.to_string()).key("tree.count"));
                __blockui_tree.add(blockui::ChildActor {
                    key: "tree.branch".into(),
                    implementation: blockui::Implementation::named("branch"),
                });
                if BROKEN {
                    __blockui_tree.add(blockui::ChildActor {
                        key: "tree.staged".into(),
                        implementation: blockui::Implementation::named("branch"),
                    });
                    __blockui_tree.add(blockui::ChildActor {
                        key: "tree.rejected".into(),
                        implementation: blockui::Implementation::named("rejected"),
                    });
                }
            }
            __blockui_tree.finish()
        })
    }
}
struct EventTree;
impl Actor for EventTree {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = Counter::ACTIONS;
    fn view(cx: &Cx<'_>) -> Result<View> {
        if COUNT.get(cx)? == 0 {
            TestTree::<false>::view(cx)
        } else {
            TestTree::<true>::view(cx)
        }
    }
}
#[test]
fn failed_recursive_child_staging_reclaims_capacity_and_preserves_live_tree()
-> std::result::Result<(), String> {
    use blockui_runtime::{ActorId, Runtime};
    let directory =
        std::env::temp_dir().join(format!("identity-staged-tree-{}", std::process::id()));
    let mut runtime = Runtime::new(&directory);
    runtime.register_for_test::<Counter>("leaf")?;
    runtime.register_for_test::<TestBranch>("branch")?;
    runtime.register_for_test::<Rejected>("rejected")?;
    let root = ActorId(911);
    for _ in 0..20 {
        assert!(runtime.spawn_for_test::<TestTree<true>>(root).is_err());
        assert!(runtime.actor(root).is_none());
        assert!(runtime.declared_children().is_empty());
        assert_eq!(
            std::fs::read_dir(&directory)
                .map_err(|e| e.to_string())?
                .count(),
            0
        );
    }
    runtime.spawn_for_test::<TestTree<false>>(root)?;
    let branch = runtime
        .child(root, blockui::abi::key("tree.branch"))
        .unwrap();
    let leaf = runtime
        .child(branch, blockui::abi::key("branch.leaf"))
        .unwrap();
    assert_eq!(runtime.parent(leaf), Some(branch));
    let slot = runtime.actor(leaf).unwrap().slot_address("count");
    runtime.tell(leaf, message(0))?;
    assert!(runtime.tick().is_empty());
    let version = runtime.actor(root).unwrap().implementation_version();
    for _ in 0..20 {
        assert!(runtime.replace_for_test::<TestTree<true>>(root).is_err());
        assert_eq!(
            runtime.actor(root).unwrap().implementation_version(),
            version
        );
        assert_eq!(runtime.declared_children().len(), 2);
        assert_eq!(
            std::fs::read_dir(&directory)
                .map_err(|e| e.to_string())?
                .count(),
            3
        );
        assert_eq!(runtime.actor(leaf).unwrap().slot_address("count"), slot);
        assert_eq!(runtime.actor(leaf).unwrap().integer("count"), 1);
    }
    runtime.replace_for_test::<TestTree<false>>(root)?;
    runtime.tell(leaf, message(0))?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(leaf).unwrap().integer("count"), 2);
    runtime.replace_for_test::<EventTree>(root)?;
    for _ in 0..20 {
        runtime.tell(root, message(0))?;
        assert_eq!(runtime.tick().len(), 1);
        assert_eq!(runtime.actor(root).unwrap().integer("count"), 0);
        assert_eq!(runtime.declared_children().len(), 2);
        assert_eq!(
            std::fs::read_dir(&directory)
                .map_err(|e| e.to_string())?
                .count(),
            3
        );
        assert_eq!(runtime.actor(leaf).unwrap().integer("count"), 2);
    }
    runtime.replace_for_test::<TestTree<false>>(root)?;
    runtime.tell(root, message(0))?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(root).unwrap().integer("count"), 1);
    runtime.compose(root)?;
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

struct GroupParent;
impl Actor for GroupParent {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = Counter::ACTIONS;
    fn view(cx: &Cx<'_>) -> Result<View> {
        let count = COUNT.get(cx)?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                for index in 0..count + 2 {
                    __blockui_tree.add(blockui::ChildActor {
                        key: blockui::Key::named("group.leaf").scoped(index as u64),
                        implementation: blockui::Implementation::named("leaf"),
                    });
                }
            }
            __blockui_tree.finish()
        })
    }
}
struct RejectSecond;
impl Actor for RejectSecond {
    const STATE: &'static [StateSpec] = Counter::STATE;
    const ACTIONS: &'static [Action] = Replacement::ACTIONS;
    fn view(cx: &Cx<'_>) -> Result<View> {
        if COUNT.get(cx)? == 1 {
            Err(Error::InvalidView("second candidate rejects"))
        } else {
            Counter::view(cx)
        }
    }
}
#[test]
fn compiled_group_replacement_is_atomic_and_updates_future_children()
-> std::result::Result<(), String> {
    use blockui_runtime::{ActorId, Runtime};
    let directory =
        std::env::temp_dir().join(format!("identity-group-test-{}", std::process::id()));
    let mut runtime = Runtime::new(&directory);
    runtime.register_for_test::<Counter>("leaf")?;
    let root = ActorId(912);
    runtime.spawn_for_test::<GroupParent>(root)?;
    let key = |index| blockui::Key::named("group.leaf").scoped(index).0;
    let first = runtime.child(root, key(0)).unwrap();
    let second = runtime.child(root, key(1)).unwrap();
    runtime.tell(second, message(0))?;
    assert!(runtime.tick().is_empty());
    let version = runtime.actor(first).unwrap().implementation_version();
    let slot = runtime.actor(first).unwrap().slot_address("count");
    assert!(runtime.replace_group_for_test::<Replacement>(&[]).is_err());
    assert!(
        runtime
            .replace_group_for_test::<Replacement>(&[first, first])
            .is_err()
    );
    assert!(
        runtime
            .replace_group_for_test::<RejectSecond>(&[first, second])
            .is_err()
    );
    assert_eq!(
        runtime.actor(first).unwrap().implementation_version(),
        version
    );
    runtime.tell(first, message(0))?;
    runtime.tell(root, message(0))?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(first).unwrap().integer("count"), 1);
    let third = runtime.child(root, key(2)).unwrap();
    runtime.tell(third, message(0))?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(third).unwrap().integer("count"), 1);
    runtime.tell(first, message(0))?;
    runtime.replace_group_for_test::<Replacement>(&[first, second, third])?;
    assert_eq!(runtime.actor(first).unwrap().slot_address("count"), slot);
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(first).unwrap().integer("count"), 11);
    runtime.tell(root, message(0))?;
    assert!(runtime.tick().is_empty());
    let fourth = runtime.child(root, key(3)).unwrap();
    runtime.tell(fourth, message(0))?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(fourth).unwrap().integer("count"), 10);
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

#[test]
fn invalid_reload_groups_reject_before_reading_candidate_artifacts()
-> std::result::Result<(), String> {
    use blockui_runtime::{ActorId, Runtime};
    let directory =
        std::env::temp_dir().join(format!("identity-invalid-group-{}", std::process::id()));
    let mut runtime = Runtime::new(&directory);
    let root = ActorId(913);
    runtime.spawn_for_test::<Counter>(root)?;
    let missing = directory.join("missing-candidate.so");
    let version = runtime.actor(root).unwrap().implementation_version();
    for (targets, error) in [
        (vec![], "reload group must have targets"),
        (vec![root, root], "duplicate or unknown reload target"),
        (vec![ActorId(914)], "duplicate or unknown reload target"),
    ] {
        assert_eq!(runtime.reload_group(&targets, &missing).unwrap_err(), error);
        assert_eq!(
            runtime
                .replace_group_for_test::<Replacement>(&targets)
                .unwrap_err(),
            error
        );
    }
    assert_eq!(
        runtime.actor(root).unwrap().implementation_version(),
        version
    );
    assert_eq!(
        std::fs::read_dir(&directory)
            .map_err(|e| e.to_string())?
            .count(),
        1
    );
    runtime.tell(root, message(0))?;
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(root).unwrap().integer("count"), 1);
    drop(runtime);
    std::fs::remove_dir(directory).map_err(|e| e.to_string())?;
    Ok(())
}

mod conditional_migration_execution {
    use super::*;
    #[blockui::actor]
    struct Alternate {
        #[state(key = "count")]
        count: State<i64>,
        extra: State<i64>,
    }
    #[blockui::actor_impl]
    impl Alternate {
        #[cfg(feature = "testing")]
        fn migrate(&self, cx: &Cx<'_>, _: u64) -> Result<()> {
            self.count.set(cx, 77)
        }
        // Last declaration must not hide the active adapter above.
        #[cfg_attr(feature = "testing", cfg(any()))]
        fn migrate(&self, _: &Cx<'_>, _: u64) -> Result<()> {
            Err(Error::UnknownAction)
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
    }
    #[blockui::actor]
    struct Absent {
        #[state(key = "count")]
        count: State<i64>,
        another: State<bool>,
    }
    #[blockui::actor_impl]
    impl Absent {
        #[cfg_attr(feature = "testing", cfg(any()))]
        fn migrate(&self, _: &Cx<'_>, _: u64) -> Result<()> {
            Err(Error::UnknownAction)
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
    }
    #[test]
    fn active_adapter_executes_and_absent_adapter_uses_default() -> std::result::Result<(), String>
    {
        let mut actor = blockui_runtime::Actor::for_test::<Counter>()?;
        let storage = actor.state_address();
        let slot = actor.slot_address("count");
        actor.replace_for_test::<Alternate>()?;
        assert_eq!(actor.integer("count"), 77);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.slot_address("count"), slot);
        let schema = actor.schema_version;
        actor.replace_for_test::<Absent>()?;
        assert_ne!(actor.schema_version, schema);
        assert_eq!(actor.integer("count"), 77);
        assert_eq!(actor.state_address(), storage);
        assert_eq!(actor.slot_address("count"), slot);
        Ok(())
    }
}

mod computed_lists {
    use super::*;
    use blockui::{Binding, Computed, IntoView};
    const FILTER: Computed<Vec<i64>> = Computed::<Vec<i64>>::new("even.items", |cx| {
        let items = ITEMS.get(cx)?;
        if items.contains(&-1) {
            return Ok(vec![0; blockui::abi::MAX_LIST_ITEMS + 1]);
        }
        Ok(items.into_iter().filter(|value| value % 2 == 0).collect())
    });
    const LENGTH: Computed<i64> =
        Computed::<i64>::new("even.length", |cx| Ok(FILTER.len(cx)? as i64));
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[ITEMS.spec()];
        const COMPUTEDS: &'static [blockui::ComputedSpec] = &[FILTER.spec(), LENGTH.spec()];
        const ACTIONS: &'static [Action] = &[Action::new("set", |cx, value| {
            ITEMS.set(
                cx,
                if value < 0 {
                    &[-1]
                } else if value == 0 {
                    &[2, 3, 4]
                } else {
                    &[2, 3, 4, 5]
                },
            )
        })];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(vec![
                Text::new(Binding::named("list.text", |cx| {
                    Ok(format!("{:?}:{}", FILTER.get(cx)?, LENGTH.get(cx)?))
                }))
                .key("list.text")
                .into_view(),
            ]))
        }
    }
    struct Replacement;
    impl Actor for Replacement {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [blockui::ComputedSpec] = &[
            Computed::<Vec<i64>>::new("even.items", |cx| {
                let mut values: Vec<_> = ITEMS
                    .get(cx)?
                    .into_iter()
                    .filter(|value| value % 2 == 0)
                    .collect();
                values.reverse();
                Ok(values)
            })
            .spec(),
            LENGTH.spec(),
        ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct WrongReadType;
    impl Actor for WrongReadType {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [blockui::ComputedSpec] = Model::COMPUTEDS;
        fn view(cx: &Cx<'_>) -> Result<View> {
            let text = Computed::<String>::new("even.items", |_| Ok(String::new())).get(cx)?;
            Ok(Column::default().children(vec![Text::new(text).into_view()]))
        }
    }
    #[test]
    fn list_memos_prune_equal_results_and_keep_durable_state_on_replacement()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let slot = actor.slot_address("items");
        actor.tell(Model::ACTIONS[0], 0)?;
        actor.tick()?;
        assert_eq!(
            actor.computed("even.items"),
            Some(&blockui_runtime::Value::IntegerList(vec![2, 4]))
        );
        let evaluations = actor.computed_evaluations("even.length");
        actor.tell(Model::ACTIONS[0], 1)?;
        actor.tick()?;
        assert_eq!(actor.computed_evaluations("even.length"), evaluations);
        let state = actor.list("items").to_vec();
        let implementation = actor.implementation_version();
        assert!(actor.replace_for_test::<WrongReadType>().is_err());
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(actor.list("items"), state);
        assert_eq!(
            actor.computed("even.items"),
            Some(&blockui_runtime::Value::IntegerList(vec![2, 4]))
        );
        actor.tell(Model::ACTIONS[0], -1)?;
        assert!(actor.tick().is_err());
        assert_eq!(actor.list("items"), state);
        actor.replace_for_test::<Replacement>()?;
        assert_eq!(actor.list("items"), state);
        assert_eq!(actor.slot_address("items"), slot);
        assert_eq!(
            actor.computed("even.items"),
            Some(&blockui_runtime::Value::IntegerList(vec![4, 2]))
        );
        Ok(())
    }
}

mod direct_computed_text {
    use super::*;
    use blockui::{Computed, ComputedSpec, IntoView};
    const CAPTION: Computed<String> =
        Computed::<String>::new("caption", |cx| Ok(format!("Count {}", COUNT.get(cx)?)));
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec()];
        const ACTIONS: &'static [Action] = &[Action::new("step", |cx, n| COUNT.set(cx, n))];
        const COMPUTEDS: &'static [ComputedSpec] = &[CAPTION.spec()];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(vec![Text::new(CAPTION).into_view()]))
        }
    }
    struct Missing;
    impl Actor for Missing {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct WrongType;
    impl Actor for WrongType {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [ComputedSpec] =
            &[Computed::<i64>::new("caption", |cx| COUNT.get(cx)).spec()];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct Replacement;
    impl Actor for Replacement {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [ComputedSpec] =
            &[
                Computed::<String>::new("caption", |cx| Ok(format!("New {}", COUNT.get(cx)?)))
                    .spec(),
            ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    #[test]
    fn direct_binding_rejects_bad_candidates_and_uses_current_computation()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        actor.tell(Model::ACTIONS[0], 7)?;
        actor.tick()?;
        let slot = actor.slot_address("count");
        let implementation = actor.implementation_version();
        assert!(actor.replace_for_test::<Missing>().is_err());
        assert!(actor.replace_for_test::<WrongType>().is_err());
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(actor.integer("count"), 7);
        assert_eq!(actor.slot_address("count"), slot);
        assert_eq!(
            actor.computed("caption"),
            Some(&blockui_runtime::Value::Text("Count 7".into()))
        );
        actor.tell(Model::ACTIONS[0], 8)?;
        actor.tick()?;
        assert_eq!(
            actor.computed("caption"),
            Some(&blockui_runtime::Value::Text("Count 8".into()))
        );
        actor.replace_for_test::<Replacement>()?;
        assert_eq!(
            actor.computed("caption"),
            Some(&blockui_runtime::Value::Text("New 8".into()))
        );
        assert_eq!(actor.slot_address("count"), slot);
        Ok(())
    }
}

mod direct_computed_scalars {
    use super::*;
    use blockui::{Computed, ComputedSpec, IntoView};
    const NUMBER: Computed<i64> = Computed::<i64>::new("number", |cx| COUNT.get(cx));
    const FRACTION: Computed<f64> =
        Computed::<f64>::new("fraction", |cx| Ok(COUNT.get(cx)? as f64 / 2.0));
    const POSITIVE: Computed<bool> = Computed::<bool>::new("positive", |cx| Ok(COUNT.get(cx)? > 0));
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec()];
        const ACTIONS: &'static [Action] = &[Action::new("set", |cx, value| COUNT.set(cx, value))];
        const COMPUTEDS: &'static [ComputedSpec] =
            &[NUMBER.spec(), FRACTION.spec(), POSITIVE.spec()];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(vec![
                Text::new(NUMBER).key("number.text").into_view(),
                Text::new(FRACTION).key("fraction.text").into_view(),
                Text::new(POSITIVE).key("positive.text").into_view(),
            ]))
        }
    }
    struct Replacement;
    impl Actor for Replacement {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [ComputedSpec] = &[
            Computed::<i64>::new("number", |cx| Ok(COUNT.get(cx)? + 10)).spec(),
            FRACTION.spec(),
            POSITIVE.spec(),
        ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct WrongType;
    impl Actor for WrongType {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [ComputedSpec] = &[
            NUMBER.spec(),
            FRACTION.spec(),
            Computed::<i64>::new("positive", |_| Ok(1)).spec(),
        ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    fn texts(actor: &blockui_runtime::Actor) -> Vec<&str> {
        ["number.text", "fraction.text", "positive.text"]
            .into_iter()
            .map(|name| actor.view.bindings[&blockui::abi::key(name)].text.as_str())
            .collect()
    }
    #[test]
    fn scalar_bindings_track_values_replace_code_and_reject_wrong_types()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        actor.tell(Model::ACTIONS[0], -3)?;
        actor.tick()?;
        assert_eq!(texts(&actor), ["-3", "-1.5", "false"]);
        let slot = actor.slot_address("count");
        let implementation = actor.implementation_version();
        assert!(actor.replace_for_test::<WrongType>().is_err());
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(texts(&actor), ["-3", "-1.5", "false"]);
        actor.replace_for_test::<Replacement>()?;
        assert_eq!(texts(&actor), ["7", "-1.5", "false"]);
        actor.tell(Model::ACTIONS[0], 5)?;
        actor.tick()?;
        assert_eq!(texts(&actor), ["15", "2.5", "true"]);
        assert_eq!(actor.slot_address("count"), slot);
        Ok(())
    }
}

mod direct_state_text {
    use super::*;
    static VIEWS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] =
            &[COUNT.spec(), RATIO.spec(), ENABLED.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] = &[
            Action::new("change", |cx, _| {
                COUNT.set(cx, -7)?;
                RATIO.set(cx, -0.0)?;
                ENABLED.set(cx, true)?;
                TITLE.set(cx, "你好 🌍")
            }),
            Action::new("count_only", |cx, value| COUNT.set(cx, value)),
        ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            VIEWS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let title = TITLE.bind(cx)?;
            let enabled = ENABLED.bind(cx)?;
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(COUNT).key("direct.count"));
                    __blockui_tree.add(Text::new(RATIO).key("direct.ratio"));
                    __blockui_tree.add(Text::new(&enabled).key("direct.enabled"));
                    __blockui_tree.add(Text::new(&title).key("direct.title"));
                }
                __blockui_tree.finish()
            })
        }
    }
    #[blockui::component(name = "binding_diagnostics")]
    struct InvalidFields {
        #[state(key = "title")]
        wrong_title: State<i64>,
    }
    #[blockui::component_impl]
    impl InvalidFields {
        fn view(&self) -> View {
            View::empty()
        }
    }
    #[blockui::actor(name = "outer_binding_diagnostics")]
    struct NestedInvalidFields {
        child: InvalidFields,
    }
    struct InvalidBinding<const MISSING: bool>;
    impl<const MISSING: bool> Actor for InvalidBinding<MISSING> {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        fn view(cx: &Cx<'_>) -> Result<View> {
            let name = if MISSING { "missing" } else { "title" };
            if MISSING {
                State::<i64>::new(name, 0).bind(cx)?;
            } else {
                NestedInvalidFields::new().bind(cx)?;
            }
            Model::view(cx)
        }
    }
    #[test]
    fn descriptor_and_bound_field_bindings_update_and_survive_replacement()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let storage = actor.state_address();
        actor.tell(Model::ACTIONS[0], 0)?;
        actor.tick()?;
        let views = VIEWS.load(std::sync::atomic::Ordering::Relaxed);
        let unchanged: Vec<_> = ["ratio", "enabled", "title"]
            .into_iter()
            .map(|name| {
                let key = blockui::abi::key(&format!("direct.{name}"));
                (key, actor.view.bindings[&key].evaluations)
            })
            .collect();
        let count_key = blockui::abi::key("direct.count");
        let evaluations = actor.view.bindings[&count_key].evaluations;
        actor.tell(Model::ACTIONS[1], -9)?;
        actor.tick()?;
        assert_eq!(VIEWS.load(std::sync::atomic::Ordering::Relaxed), views);
        assert_eq!(actor.view.bindings[&count_key].text, "-9");
        assert_eq!(actor.view.bindings[&count_key].evaluations, evaluations + 1);
        for (key, evaluations) in unchanged {
            assert_eq!(actor.view.bindings[&key].evaluations, evaluations);
        }
        let evaluations = actor.view.bindings[&count_key].evaluations;
        actor.tell(Model::ACTIONS[1], -9)?;
        actor.tick()?;
        assert_eq!(actor.view.bindings[&count_key].evaluations, evaluations);
        assert_eq!(VIEWS.load(std::sync::atomic::Ordering::Relaxed), views);
        let implementation = actor.implementation_version();
        for (name, error) in [
            (
                "title",
                actor
                    .replace_for_test::<InvalidBinding<false>>()
                    .unwrap_err(),
            ),
            (
                "missing",
                actor
                    .replace_for_test::<InvalidBinding<true>>()
                    .unwrap_err(),
            ),
        ] {
            assert!(
                error.contains(&format!(
                    "state {:#018x} validation failed: expected i64",
                    blockui::abi::key(name)
                )),
                "{error}"
            );
            assert!(error.contains("slot missing or kind mismatch"), "{error}");
            if name == "title" {
                assert!(
                    error.contains("binding binding_diagnostics.wrong_title:"),
                    "{error}"
                );
                assert!(
                    !error.contains("outer_binding_diagnostics.child"),
                    "{error}"
                );
            }
            assert_eq!(actor.implementation_version(), implementation);
            assert_eq!(actor.state_address(), storage);
            assert_eq!(actor.view.bindings[&count_key].text, "-9");
        }
        actor.replace_for_test::<Model>()?;
        assert_eq!(actor.state_address(), storage);
        for (name, text) in [
            ("count", "-9"),
            ("ratio", "-0"),
            ("enabled", "true"),
            ("title", "你好 🌍"),
        ] {
            assert_eq!(
                actor.view.bindings[&blockui::abi::key(&format!("direct.{name}"))].text,
                text
            );
        }
        Ok(())
    }
}

mod allocation_rollback {
    use super::*;
    const ITEMS: State<Vec<i64>> = State::<Vec<i64>>::with_default("items", &[7, 9]);
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[TITLE.spec(), ITEMS.spec()];
        const ACTIONS: &'static [Action] = &[Action::new("fail", |cx, _| {
            TITLE.set(cx, "first edit")?;
            ITEMS.set(cx, &[100, 200])?;
            TITLE.set(cx, "second edit")?;
            ITEMS.set(cx, &[300])?;
            Err(blockui::Error::UnknownAction)
        })];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(TITLE));
                }
                __blockui_tree.finish()
            })
        }
    }
    struct LateFailure;
    impl Actor for LateFailure {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = &[Action::new("edit", |cx, value| {
            TITLE.set(cx, if value == 0 { "reject" } else { "recovered" })?;
            ITEMS.set(cx, &[42])
        })];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        Text::new(blockui::Binding::named("late.text", |cx| {
                            let text = TITLE.get(cx)?;
                            if text == "reject" {
                                Err(blockui::Error::InvalidView("late failure"))
                            } else {
                                Ok(text)
                            }
                        }))
                        .key("late.text"),
                    );
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn downstream_failure_restores_buffers_and_old_view_then_recovers()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<LateFailure>()?;
        let text_buffer = actor.text("title").as_ptr();
        let list_buffer = actor.list("items").as_ptr();
        let binding = blockui::abi::key("late.text");
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: LateFailure::ACTIONS[0].id,
                    value: 0
                })
                .is_err()
        );
        assert_eq!(actor.text("title"), "draft");
        assert_eq!(actor.list("items"), &[7, 9]);
        assert_eq!(actor.text("title").as_ptr(), text_buffer);
        assert_eq!(actor.list("items").as_ptr(), list_buffer);
        assert_eq!(actor.view.bindings[&binding].text, "draft");
        actor.dispatch(blockui_runtime::Message {
            action: LateFailure::ACTIONS[0].id,
            value: 1,
        })?;
        assert_eq!(actor.text("title"), "recovered");
        assert_eq!(actor.list("items"), &[42]);
        assert_eq!(actor.view.bindings[&binding].text, "recovered");
        Ok(())
    }
    #[test]
    fn failed_multiwrite_restores_original_text_and_list_allocations()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let text = actor.text("title").to_owned();
        let list = actor.list("items").to_vec();
        let text_buffer = actor.text("title").as_ptr();
        let list_buffer = actor.list("items").as_ptr();
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: Model::ACTIONS[0].id,
                    value: 0
                })
                .is_err()
        );
        assert_eq!(actor.text("title"), text);
        assert_eq!(actor.list("items"), list);
        assert_eq!(actor.text("title").as_ptr(), text_buffer);
        assert_eq!(actor.list("items").as_ptr(), list_buffer);
        Ok(())
    }
}

mod structural_work_baseline {
    use super::*;
    use blockui::Binding;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ROOTS: AtomicUsize = AtomicUsize::new(0);
    static FIXED_PANELS: AtomicUsize = AtomicUsize::new(0);
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] =
            &[Action::new("set_count", |cx, value| COUNT.set(cx, value))];
        fn view(cx: &Cx<'_>) -> Result<View> {
            ROOTS.fetch_add(1, Ordering::Relaxed);
            let panel = {
                FIXED_PANELS.fetch_add(1, Ordering::Relaxed);
                {
                    let __blockui_tree = ::blockui::Ui::new();
                    {
                        let _scope_0 = __blockui_tree.scope(Column::default().key("fixed.panel"));
                        __blockui_tree.add(
                            Text::new(Binding::named("fixed.title", |cx| TITLE.get(cx)))
                                .key("fixed.title"),
                        );
                    }
                    __blockui_tree.finish()
                }
            };
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(panel).__local_identity(stringify!(panel)),
                    );
                    if COUNT.get(cx)? > 0 {
                        __blockui_tree.add(Text::new("Visible").key("conditional.text"));
                    }
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn structural_change_rebuilds_unrelated_panel_but_reuses_its_binding()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let roots = ROOTS.load(Ordering::Relaxed);
        let panels = FIXED_PANELS.load(Ordering::Relaxed);
        let key = blockui::abi::key("fixed.title");
        let evaluations = actor.view.bindings[&key].evaluations;
        actor.tell(Model::ACTIONS[0], 1)?;
        actor.tick()?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots + 1);
        assert_eq!(FIXED_PANELS.load(Ordering::Relaxed), panels + 1);
        assert_eq!(actor.view.bindings[&key].evaluations, evaluations);
        assert_eq!(actor.view.bindings[&key].text, "draft");
        assert!(
            actor
                .view
                .bindings
                .contains_key(&blockui::abi::key("conditional.text"))
        );
        Ok(())
    }
}

mod scope_registry {
    use super::*;
    use blockui::Scope;
    const PANEL: Scope = Scope::new("panel", |_| {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Panel"));
            }
            __blockui_tree.finish()
        })
    });
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec()];
        const ACTIONS: &'static [Action] = Counter::ACTIONS;
        const SCOPES: &'static [Scope] = &[PANEL];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
    }
    struct Duplicate;
    impl Actor for Duplicate {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const SCOPES: &'static [Scope] = &[PANEL, PANEL];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
    }
    #[test]
    fn candidate_registry_failure_retains_current_identity_and_state()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        actor.dispatch(message(0))?;
        let implementation = actor.implementation_version();
        assert_eq!(actor.scope_ids(), vec![PANEL.id()]);
        assert_eq!(
            actor.replace_for_test::<Duplicate>().unwrap_err(),
            format!(
                "invalid structural scope registry: duplicate scope identity {:#018x}",
                PANEL.id()
            )
        );
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(actor.scope_ids(), vec![PANEL.id()]);
        assert_eq!(actor.integer("count"), 1);
        actor.replace_for_test::<Counter>()?;
        assert!(actor.scope_ids().is_empty());
        assert_eq!(actor.integer("count"), 1);
        Ok(())
    }
}

mod scope_mounting {
    use super::*;
    use blockui::{IntoView, Scope};
    const PANEL: Scope = Scope::new("panel", |cx| {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(TITLE).key("panel.title"));
                if COUNT.get(cx)? > 0 {
                    __blockui_tree.add(Text::new("Visible").key("panel.conditional"));
                }
            }
            __blockui_tree.finish()
        })
    });
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] = Counter::ACTIONS;
        const SCOPES: &'static [Scope] = &[PANEL];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(PANEL).__local_identity(stringify!(PANEL)),
                    );
                }
                __blockui_tree.finish()
            })
        }
    }
    struct Replacement;
    impl Actor for Replacement {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const SCOPES: &'static [Scope] = &[Scope::new("panel", |_| {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(blockui::Row::default());
                    __blockui_tree.add(Text::new("New code").key("panel.title"));
                }
                __blockui_tree.finish()
            })
        })];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct Missing;
    impl Actor for Missing {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct Recursive;
    impl Actor for Recursive {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const SCOPES: &'static [Scope] = &[Scope::new("panel", |_| {
            Ok(Column::default().children(vec![PANEL.into_view()]))
        })];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct LateFailure;
    impl Actor for LateFailure {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const SCOPES: &'static [Scope] = &[Scope::new("panel", |cx| {
            if COUNT.get(cx)? > 0 {
                return Err(blockui::Error::InvalidView("scope rejected state"));
            }
            Ok(Column::default()
                .children(vec![Text::new("Initial").key("panel.title").into_view()]))
        })];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    #[test]
    fn mounts_track_structure_replace_builder_and_reject_bad_candidates()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let slot = actor.slot_address("count");
        assert!(
            !actor
                .view
                .bindings
                .contains_key(&blockui::abi::key("panel.conditional"))
        );
        actor.dispatch(message(0))?;
        assert!(
            actor
                .view
                .bindings
                .contains_key(&blockui::abi::key("panel.conditional"))
        );
        let implementation = actor.implementation_version();
        assert!(actor.replace_for_test::<Missing>().is_err());
        assert!(actor.replace_for_test::<Recursive>().is_err());
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(actor.integer("count"), 1);
        actor.replace_for_test::<Replacement>()?;
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("panel.title")].text,
            "New code"
        );
        assert_eq!(actor.slot_address("count"), slot);
        Ok(())
    }
    #[test]
    fn failed_scope_expansion_rolls_back_action_and_keeps_old_tree()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<LateFailure>()?;
        assert!(actor.dispatch(message(0)).is_err());
        assert_eq!(actor.integer("count"), 0);
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("panel.title")].text,
            "Initial"
        );
        Ok(())
    }
}

mod scope_dependency_isolation {
    use super::*;
    use blockui::{Computed, ComputedSpec, Scope};
    use blockui_runtime::Dependency;
    const VISIBLE: Computed<bool> =
        Computed::<bool>::new("visible", |cx| Ok(!TITLE.get(cx)?.is_empty()));
    const INNER: Scope = Scope::new("inner", |cx| {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                if VISIBLE.get(cx)? {
                    __blockui_tree.add(Text::new(TITLE).key("inner.text"));
                }
            }
            __blockui_tree.finish()
        })
    });
    const OUTER: Scope = Scope::new("outer", |cx| {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                if COUNT.get(cx)? >= 0 {
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(INNER).__local_identity(stringify!(INNER)),
                    );
                }
            }
            __blockui_tree.finish()
        })
    });
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] = &[
            Action::new("set_count", |cx, value| COUNT.set(cx, value)),
            Action::new("clear", |cx, _| TITLE.set(cx, "")),
        ];
        const COMPUTEDS: &'static [ComputedSpec] = &[VISIBLE.spec()];
        const SCOPES: &'static [Scope] = &[OUTER, INNER];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(OUTER).__local_identity(stringify!(OUTER)),
                    );
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn nested_scope_reads_are_separate_and_disposed_scope_dependencies_are_removed()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let outer = OUTER.id();
        let inner = INNER.id();
        assert!(actor.view.root_dependencies.is_empty());
        assert_eq!(
            actor.view.scope_dependencies[&outer].dependencies,
            std::collections::BTreeSet::from([Dependency::State(COUNT.key())])
        );
        assert_eq!(
            actor.view.scope_dependencies[&inner].dependencies,
            std::collections::BTreeSet::from([Dependency::Computed(blockui::abi::key("visible"))])
        );
        let outer_versions = actor.view.scope_dependencies[&outer].versions.clone();
        let inner_versions = actor.view.scope_dependencies[&inner].versions.clone();
        actor.tell(Model::ACTIONS[1], 0)?;
        actor.tick()?;
        assert_eq!(
            actor.view.scope_dependencies[&outer].versions,
            outer_versions
        );
        assert_ne!(
            actor.view.scope_dependencies[&inner].versions,
            inner_versions
        );
        assert!(
            !actor
                .view
                .bindings
                .contains_key(&blockui::abi::key("inner.text"))
        );
        actor.tell(Model::ACTIONS[0], -1)?;
        actor.tick()?;
        assert!(!actor.view.scope_dependencies.contains_key(&inner));
        assert!(actor.view.scope_dependencies.contains_key(&outer));
        Ok(())
    }
}

mod local_scope_refresh {
    use super::*;
    use blockui::{IntoView, Scope};
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ROOTS: AtomicUsize = AtomicUsize::new(0);
    static COUNTS: AtomicUsize = AtomicUsize::new(0);
    static ANONYMOUS: AtomicUsize = AtomicUsize::new(0);
    static TITLES: AtomicUsize = AtomicUsize::new(0);
    const COUNTER_PANEL: Scope = Scope::new("local.count", |cx| {
        COUNTS.fetch_add(1, Ordering::Relaxed);
        let count = COUNT.get(cx)?;
        if count < 0 {
            return Err(blockui::Error::InvalidView("negative scope"));
        }
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(count.to_string()).key("local.count.text"));
                if count > 0 {
                    __blockui_tree.add(Text::new("Visible").key("local.extra"));
                }
            }
            __blockui_tree.finish()
        })
    });
    const TITLE_PANEL: Scope = Scope::new("local.title", |cx| {
        TITLES.fetch_add(1, Ordering::Relaxed);
        Ok(Column::default().children(vec![
            Text::new(TITLE.get(cx)?)
                .key("local.title.text")
                .into_view(),
        ]))
    });
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] = &[
            Action::new("count", |cx, n| COUNT.set(cx, n)),
            Action::new("title", |cx, _| TITLE.set(cx, "changed")),
            Action::new("reject", |cx, _| {
                TITLE.set(cx, "rejected")?;
                COUNT.set(cx, -1)
            }),
        ];
        const SCOPES: &'static [Scope] = &[COUNTER_PANEL, TITLE_PANEL];
        fn view(_: &Cx<'_>) -> Result<View> {
            ROOTS.fetch_add(1, Ordering::Relaxed);
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(COUNTER_PANEL)
                            .__local_identity(stringify!(COUNTER_PANEL)),
                    );
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(TITLE_PANEL)
                            .__local_identity(stringify!(TITLE_PANEL)),
                    );
                    __blockui_tree.add(
                        Text::new(blockui::bind!(|cx| {
                            ANONYMOUS.fetch_add(1, Ordering::Relaxed);
                            TITLE.get(cx)
                        }))
                        .key("local.anonymous"),
                    );
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn refreshes_only_affected_builders_and_publishes_all_scopes_atomically()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let roots = ROOTS.load(Ordering::Relaxed);
        let counts = COUNTS.load(Ordering::Relaxed);
        let titles = TITLES.load(Ordering::Relaxed);
        let anonymous = ANONYMOUS.load(Ordering::Relaxed);
        let text_buffer = actor.view.bindings[&blockui::abi::key("local.anonymous")]
            .text
            .as_ptr();
        actor.tell(Model::ACTIONS[0], 1)?;
        actor.tick()?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
        assert_eq!(COUNTS.load(Ordering::Relaxed), counts + 1);
        assert_eq!(TITLES.load(Ordering::Relaxed), titles);
        assert_eq!(ANONYMOUS.load(Ordering::Relaxed), anonymous);
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("local.anonymous")]
                .text
                .as_ptr(),
            text_buffer
        );
        assert!(
            actor
                .view
                .bindings
                .contains_key(&blockui::abi::key("local.extra"))
        );
        actor.tell(Model::ACTIONS[1], 0)?;
        actor.tick()?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
        assert_eq!(COUNTS.load(Ordering::Relaxed), counts + 1);
        assert_eq!(TITLES.load(Ordering::Relaxed), titles + 1);
        assert_eq!(ANONYMOUS.load(Ordering::Relaxed), anonymous + 1);
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("local.anonymous")].text,
            "changed"
        );
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: Model::ACTIONS[2].id,
                    value: 0
                })
                .is_err()
        );
        assert_eq!(actor.integer("count"), 1);
        assert_eq!(actor.text("title"), "changed");
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("local.title.text")].text,
            "changed"
        );
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("local.count.text")].text,
            "1"
        );
        actor.tell(Model::ACTIONS[0], 0)?;
        actor.tick()?;
        assert!(
            !actor
                .view
                .bindings
                .contains_key(&blockui::abi::key("local.extra"))
        );
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
        Ok(())
    }
}

mod declared_scopes {
    use blockui::*;
    #[actor(name = "declared")]
    struct Model {
        count: State<i64>,
    }
    #[actor_impl]
    impl Model {
        #[action]
        fn increment(&self, cx: &Cx<'_>) -> Result<()> {
            self.count.update(cx, |n| Ok(n + 1))
        }
        #[scope(name = "stable.panel")]
        fn panel(&self, cx: &Cx<'_>) -> Result<View> {
            let count = self.count.get(cx)?;
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(count.to_string()).key("declared.count"));
                    if count > 0 {
                        __blockui_tree.add(Text::new("Inserted").key("declared.extra"));
                    }
                }
                __blockui_tree.finish()
            })
        }
        #[scope]
        fn status(&self, _: &Cx<'_>) -> View {
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new("Ready").key("declared.status"));
                }
                __blockui_tree.finish()
            }
        }
        #[cfg(any())]
        #[scope(name = "stable.panel")]
        fn absent(&self, _: &Cx<'_>) -> View {
            unreachable!()
        }
        fn view(&self, _: &Cx<'_>) -> View {
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Self::panel_scope());
                    __blockui_tree.add(Self::status_scope());
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn declared_scope_registers_and_updates_conditionally_without_manual_table()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        assert_eq!(actor.scope_ids().len(), 2);
        assert!(actor.scope_ids().contains(&abi::key("stable.panel")));
        assert!(actor.scope_ids().contains(&abi::key("Model.status")));
        assert!(actor.view.root_dependencies.is_empty());
        assert!(
            !actor
                .view
                .bindings
                .contains_key(&abi::key("declared.extra"))
        );
        actor.tell(Model::increment_action(), 0)?;
        actor.tick()?;
        assert_eq!(actor.view.bindings[&abi::key("declared.count")].text, "1");
        assert!(
            actor
                .view
                .bindings
                .contains_key(&abi::key("declared.extra"))
        );
        assert_eq!(
            actor.view.scope_dependencies[&abi::key("stable.panel")].dependencies,
            std::collections::BTreeSet::from([blockui_runtime::Dependency::State(abi::key(
                "declared.count"
            ))])
        );
        Ok(())
    }
}

mod nested_scope_publication {
    use blockui::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ROOTS: AtomicUsize = AtomicUsize::new(0);
    static OUTERS: AtomicUsize = AtomicUsize::new(0);
    static INNERS: AtomicUsize = AtomicUsize::new(0);
    #[actor(name = "nested")]
    struct Model {
        parent: State<i64>,
        child: State<i64>,
        root: State<i64>,
    }
    #[actor_impl]
    impl Model {
        #[action]
        fn both(&self, cx: &Cx<'_>) -> Result<()> {
            self.parent.update(cx, |n| Ok(n + 1))?;
            self.child.update(cx, |n| Ok(n + 1))
        }
        #[action]
        fn child_only(&self, cx: &Cx<'_>) -> Result<()> {
            self.child.update(cx, |n| Ok(n + 1))
        }
        #[action]
        fn root_only(&self, cx: &Cx<'_>) -> Result<()> {
            self.root.update(cx, |n| Ok(n + 1))
        }
        #[action]
        fn root_and_child(&self, cx: &Cx<'_>) -> Result<()> {
            self.root.update(cx, |n| Ok(n + 1))?;
            self.child.update(cx, |n| Ok(n + 1))
        }
        #[scope]
        fn inner(&self, cx: &Cx<'_>) -> Result<View> {
            INNERS.fetch_add(1, Ordering::Relaxed);
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Row::default());
                    __blockui_tree
                        .add(Text::new(self.child.get(cx)?.to_string()).key("nested.inner.text"));
                }
                __blockui_tree.finish()
            })
        }
        #[scope]
        fn outer(&self, cx: &Cx<'_>) -> Result<View> {
            OUTERS.fetch_add(1, Ordering::Relaxed);
            let parent = self.parent.get(cx)?;
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(parent.to_string()).key("nested.outer.text"));
                    __blockui_tree.add(Self::inner_scope());
                    if parent == 2 {
                        __blockui_tree.add(Text::new("Collision").key("nested.retained"));
                    }
                }
                __blockui_tree.finish()
            })
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            ROOTS.fetch_add(1, Ordering::Relaxed);
            let root = self.root.get(cx)?;
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(root.to_string()).key("nested.retained"));
                    __blockui_tree.add(Self::outer_scope());
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn nested_dirty_scopes_build_once_and_collisions_retain_published_tree()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let roots = ROOTS.load(Ordering::Relaxed);
        let outers = OUTERS.load(Ordering::Relaxed);
        let inners = INNERS.load(Ordering::Relaxed);
        actor.tell(Model::both_action(), 0)?;
        actor.tick()?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
        assert_eq!(OUTERS.load(Ordering::Relaxed), outers + 1);
        assert_eq!(INNERS.load(Ordering::Relaxed), inners + 1);
        assert_eq!(
            actor.view.bindings[&abi::key("nested.inner.text")].text,
            "1"
        );
        let order = actor.view.order.clone();
        let versions: Vec<_> = actor
            .view
            .scope_dependencies
            .iter()
            .map(|(id, scope)| (*id, scope.versions.clone()))
            .collect();
        let address = actor.state_address();
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: Model::both_action().id,
                    value: 0
                })
                .is_err()
        );
        assert_eq!(actor.integer("nested.parent"), 1);
        assert_eq!(actor.integer("nested.child"), 1);
        assert_eq!(actor.view.order, order);
        assert_eq!(
            actor
                .view
                .scope_dependencies
                .iter()
                .map(|(id, scope)| (*id, scope.versions.clone()))
                .collect::<Vec<_>>(),
            versions
        );
        assert_eq!(actor.state_address(), address);
        assert_eq!(actor.view.bindings[&abi::key("nested.retained")].text, "0");
        assert_eq!(
            actor.view.bindings[&abi::key("nested.outer.text")].text,
            "1"
        );
        let outers = OUTERS.load(Ordering::Relaxed);
        let inners = INNERS.load(Ordering::Relaxed);
        actor.tell(Model::child_only_action(), 0)?;
        actor.tick()?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
        assert_eq!(OUTERS.load(Ordering::Relaxed), outers);
        assert_eq!(INNERS.load(Ordering::Relaxed), inners + 1);
        assert_eq!(
            actor.view.bindings[&abi::key("nested.inner.text")].text,
            "2"
        );
        assert_eq!(
            actor.view.bindings[&Model::inner_scope().id()].node.kind,
            abi::ROW
        );
        actor.tell(Model::root_only_action(), 0)?;
        actor.tick()?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots + 1);
        assert_eq!(OUTERS.load(Ordering::Relaxed), outers);
        assert_eq!(INNERS.load(Ordering::Relaxed), inners + 1);
        assert_eq!(actor.view.bindings[&abi::key("nested.retained")].text, "1");
        actor.tell(Model::root_and_child_action(), 0)?;
        actor.tick()?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots + 2);
        assert_eq!(OUTERS.load(Ordering::Relaxed), outers + 1);
        assert_eq!(INNERS.load(Ordering::Relaxed), inners + 2);
        assert_eq!(
            actor.view.bindings[&abi::key("nested.inner.text")].text,
            "3"
        );
        assert_eq!(actor.view.bindings[&abi::key("nested.retained")].text, "2");
        Ok(())
    }
}

mod dynamic_scope_dependencies {
    use blockui::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static BUILDS: AtomicUsize = AtomicUsize::new(0);
    static MEMO_BUILDS: AtomicUsize = AtomicUsize::new(0);
    #[actor(name = "branches")]
    struct Model {
        enabled: State<bool>,
        left: State<i64>,
        right: State<i64>,
    }
    #[actor_impl]
    impl Model {
        #[action]
        fn select(&self, cx: &Cx<'_>, enabled: bool) -> Result<()> {
            self.enabled.set(cx, enabled)
        }
        #[action]
        fn left(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
            self.left.set(cx, value)
        }
        #[action]
        fn right(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
            self.right.set(cx, value)
        }
        #[computed]
        fn positive(&self, cx: &Cx<'_>) -> Result<bool> {
            Ok(self.left.get(cx)? > 0)
        }
        #[scope]
        fn selected(&self, cx: &Cx<'_>) -> Result<View> {
            BUILDS.fetch_add(1, Ordering::Relaxed);
            let value = if self.enabled.get(cx)? {
                self.left.get(cx)?
            } else {
                self.right.get(cx)?
            };
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(value.to_string()).key("branches.value"));
                }
                __blockui_tree.finish()
            })
        }
        #[scope]
        fn memo(&self, cx: &Cx<'_>) -> Result<View> {
            MEMO_BUILDS.fetch_add(1, Ordering::Relaxed);
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    if self.positive(cx)? {
                        __blockui_tree.add(Text::new("Positive").key("branches.positive"));
                    }
                }
                __blockui_tree.finish()
            })
        }
        fn view(&self, _: &Cx<'_>) -> View {
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Self::selected_scope());
                    __blockui_tree.add(Self::memo_scope());
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn scopes_retire_old_branch_reads_and_prune_equal_computed_results()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let selected = Model::selected_scope().id();
        let builds = BUILDS.load(Ordering::Relaxed);
        actor.tell(Model::left_action(), 1)?;
        actor.tick()?;
        assert_eq!(BUILDS.load(Ordering::Relaxed), builds); // inactive branch
        let memos = MEMO_BUILDS.load(Ordering::Relaxed);
        actor.tell(Model::left_action(), 2)?;
        actor.tick()?;
        assert_eq!(MEMO_BUILDS.load(Ordering::Relaxed), memos); // true -> true
        actor.tell(Model::select_action(), true)?;
        actor.tick()?;
        assert_eq!(BUILDS.load(Ordering::Relaxed), builds + 1);
        assert_eq!(actor.view.bindings[&abi::key("branches.value")].text, "2");
        assert_eq!(
            actor.view.scope_dependencies[&selected].dependencies,
            std::collections::BTreeSet::from([
                blockui_runtime::Dependency::State(abi::key("branches.enabled")),
                blockui_runtime::Dependency::State(abi::key("branches.left")),
            ])
        );
        actor.tell(Model::right_action(), 10)?;
        actor.tick()?;
        assert_eq!(BUILDS.load(Ordering::Relaxed), builds + 1);
        actor.tell(Model::left_action(), -1)?;
        actor.tick()?;
        assert_eq!(BUILDS.load(Ordering::Relaxed), builds + 2);
        assert_eq!(MEMO_BUILDS.load(Ordering::Relaxed), memos + 1);
        assert!(
            !actor
                .view
                .bindings
                .contains_key(&abi::key("branches.positive"))
        );
        actor.tell(Model::select_action(), false)?;
        actor.tick()?;
        assert_eq!(actor.view.bindings[&abi::key("branches.value")].text, "10");
        actor.tell(Model::left_action(), -2)?;
        actor.tick()?;
        assert_eq!(BUILDS.load(Ordering::Relaxed), builds + 3);
        assert_eq!(MEMO_BUILDS.load(Ordering::Relaxed), memos + 1);
        Ok(())
    }
}

mod editor_names {
    use super::*;
    use blockui::{TextArea, TextInput};
    struct Model<const VERSION: u8>;
    impl<const VERSION: u8> Actor for Model<VERSION> {
        const STATE: &'static [StateSpec] = &[TITLE.spec()];
        const ACTIONS: &'static [Action] = &[];
        fn view(_: &Cx<'_>) -> Result<View> {
            let label = match VERSION {
                0 => "Notes",
                1 => "Edited notes",
                _ => "",
            };
            Ok(Column::default().children(vec![
                TextInput::new(TITLE).label(label).key("names.input"),
                TextArea::new(TITLE).label(label).key("names.area"),
            ]))
        }
    }
    #[test]
    fn editor_names_replace_and_clear_without_changing_durable_text()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model<0>>()?;
        for name in ["names.input", "names.area"] {
            assert_eq!(
                actor.view.bindings[&blockui::abi::key(name)]
                    .semantic_label
                    .as_deref(),
                Some("Notes")
            );
            assert_eq!(actor.view.bindings[&blockui::abi::key(name)].text, "draft");
        }
        actor.set_text(TITLE.key(), "a😀b")?;
        let slot = actor.slot_address("title");
        actor.replace_for_test::<Model<1>>()?;
        for name in ["names.input", "names.area"] {
            assert_eq!(
                actor.view.bindings[&blockui::abi::key(name)]
                    .semantic_label
                    .as_deref(),
                Some("Edited notes")
            );
            assert_eq!(actor.view.bindings[&blockui::abi::key(name)].text, "a😀b");
        }
        actor.replace_for_test::<Model<2>>()?;
        assert_eq!(actor.slot_address("title"), slot);
        for name in ["names.input", "names.area"] {
            assert!(
                actor.view.bindings[&blockui::abi::key(name)]
                    .semantic_label
                    .is_none()
            );
            assert_eq!(actor.view.bindings[&blockui::abi::key(name)].text, "a😀b");
        }
        Ok(())
    }
}

mod text_collections {
    use super::*;
    use blockui::{actor, actor_impl};
    #[actor(name = "text_collections")]
    struct Model {
        items: State<Vec<String>>,
    }
    #[actor_impl]
    impl Model {
        #[action]
        fn change(&self, cx: &Cx<'_>, mode: i64) -> Result<()> {
            let bound = self.bind(cx)?;
            match mode {
                0 => bound
                    .items
                    .set(&["标题😀".into(), "".into(), "a\0b".into()]),
                1 => bound.items.set(&bound.items.get()),
                2 => bound.items.update(|items| {
                    items.push("bad".into());
                    Err(Error::Overflow)
                }),
                3 => {
                    bound.items.set(&["bad".into()])?;
                    Err(Error::Overflow)
                }
                4 => bound
                    .items
                    .set(&vec![String::new(); blockui::abi::MAX_LIST_ITEMS + 1]),
                5 => bound
                    .items
                    .set(&["x".repeat(blockui::abi::MAX_TEXT_LIST_BYTES), "y".into()]),
                _ => State::<Vec<i64>>::new("text_collections.items")
                    .get(cx)
                    .map(|_| ()),
            }
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            let bound = self.bind(cx)?;
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        Text::new(bound.items.map_text(|items| items.join(" | ")))
                            .key("text_collections.summary"),
                    );
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn typed_text_lists_reject_bad_updates_and_keep_equal_bindings_cached()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        assert!(actor.text_list("text_collections.items").is_empty());
        let message = |value| blockui_runtime::Message {
            action: Model::change_action().id,
            value,
        };
        actor.dispatch(message(0))?;
        let buffer = actor.text_list("text_collections.items").as_ptr();
        let text = actor.text_list("text_collections.items")[0].as_ptr();
        let key = blockui::abi::key("text_collections.summary");
        let evaluations = actor.view.bindings[&key].evaluations;
        actor.dispatch(message(1))?;
        assert_eq!(actor.view.bindings[&key].evaluations, evaluations);
        for mode in 2..=6 {
            assert!(actor.dispatch(message(mode)).is_err());
            assert_eq!(
                actor.text_list("text_collections.items"),
                ["标题😀", "", "a\0b"]
            );
            assert_eq!(actor.text_list("text_collections.items").as_ptr(), buffer);
            assert_eq!(actor.text_list("text_collections.items")[0].as_ptr(), text);
            assert_eq!(actor.view.bindings[&key].evaluations, evaluations);
        }
        Ok(())
    }
}

mod computed_text_lists {
    use super::*;
    use blockui::{Binding, Computed, IntoView};
    const ITEMS: State<Vec<String>> = State::<Vec<String>>::new("strings");
    const FILTER: Computed<Vec<String>> = Computed::<Vec<String>>::new("strings.filtered", |cx| {
        let items = ITEMS.get(cx)?;
        if items.iter().any(|item| item == "overflow") {
            return Ok(vec![String::new(); blockui::abi::MAX_LIST_ITEMS + 1]);
        }
        if items.iter().any(|item| item == "byte-overflow") {
            return Ok(vec![
                "x".repeat(blockui::abi::MAX_TEXT_LIST_BYTES / 2),
                "y".repeat(blockui::abi::MAX_TEXT_LIST_BYTES / 2),
                "😀".into(),
            ]);
        }
        Ok(items.into_iter().filter(|item| !item.is_empty()).collect())
    });
    const LENGTH: Computed<i64> =
        Computed::<i64>::new("strings.length", |cx| Ok(FILTER.len(cx)? as i64));
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[ITEMS.spec()];
        const COMPUTEDS: &'static [blockui::ComputedSpec] = &[FILTER.spec(), LENGTH.spec()];
        const ACTIONS: &'static [Action] = &[Action::new("set-strings", |cx, value| {
            let values = if value == -2 {
                vec!["byte-overflow".into()]
            } else if value < 0 {
                vec!["overflow".into()]
            } else if value == 0 {
                vec!["标题😀".into(), "a\0b".into()]
            } else {
                vec!["标题😀".into(), "a\0b".into(), String::new()]
            };
            ITEMS.set(cx, &values)
        })];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(vec![
                Text::new(Binding::named("filtered", |cx| {
                    Ok(format!("{:?}:{}", FILTER.get(cx)?, LENGTH.get(cx)?))
                }))
                .key("filtered")
                .into_view(),
            ]))
        }
    }
    struct Replacement;
    impl Actor for Replacement {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [blockui::ComputedSpec] = &[
            Computed::<Vec<String>>::new("strings.filtered", |cx| {
                let mut items = ITEMS
                    .get(cx)?
                    .into_iter()
                    .filter(|item| !item.is_empty())
                    .collect::<Vec<_>>();
                items.reverse();
                Ok(items)
            })
            .spec(),
            LENGTH.spec(),
        ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    struct WrongReader;
    impl Actor for WrongReader {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [blockui::ComputedSpec] = Model::COMPUTEDS;
        fn view(cx: &Cx<'_>) -> Result<View> {
            // Even ignoring the returned error must not admit a faulted projection.
            let _ = Computed::<Vec<String>>::new("strings.length", |_| Ok(vec![])).get(cx);
            Model::view(cx)
        }
    }
    struct OversizedCandidate;
    impl Actor for OversizedCandidate {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        const COMPUTEDS: &'static [blockui::ComputedSpec] = &[
            Computed::<Vec<String>>::new("strings.filtered", |_| {
                Ok(vec!["x".repeat(blockui::abi::MAX_TEXT_LIST_BYTES + 1)])
            })
            .spec(),
            LENGTH.spec(),
        ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Model::view(cx)
        }
    }
    #[test]
    fn text_list_computeds_prune_equal_outputs_and_rollback_overflow()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        actor.tell(Model::ACTIONS[0], 0)?;
        actor.tick()?;
        assert_eq!(
            actor.computed("strings.filtered"),
            Some(&blockui_runtime::Value::TextList(vec![
                "标题😀".into(),
                "a\0b".into()
            ]))
        );
        let memo_buffer = match actor.computed("strings.filtered").unwrap() {
            blockui_runtime::Value::TextList(items) => items.as_ptr(),
            _ => unreachable!(),
        };
        let evaluations = actor.computed_evaluations("strings.length");
        actor.tell(Model::ACTIONS[0], 1)?;
        actor.tick()?;
        assert_eq!(actor.computed_evaluations("strings.length"), evaluations);
        assert!(
            matches!(actor.computed("strings.filtered"), Some(blockui_runtime::Value::TextList(items)) if items.as_ptr() == memo_buffer)
        );
        let retained = actor.text_list("strings").to_vec();
        let buffer = actor.text_list("strings").as_ptr();
        let published = actor.view.bindings[&blockui::abi::key("filtered")]
            .text
            .clone();
        for value in [-1, -2] {
            actor.tell(Model::ACTIONS[0], value)?;
            assert!(actor.tick().is_err());
            assert_eq!(actor.text_list("strings"), retained);
            assert_eq!(actor.text_list("strings").as_ptr(), buffer);
            assert_eq!(actor.computed_evaluations("strings.length"), evaluations);
            assert_eq!(
                actor.view.bindings[&blockui::abi::key("filtered")].text,
                published
            );
        }
        let implementation = actor.implementation_version();
        assert!(actor.replace_for_test::<WrongReader>().is_err());
        assert!(actor.replace_for_test::<OversizedCandidate>().is_err());
        assert_eq!(actor.implementation_version(), implementation);
        assert_eq!(actor.text_list("strings").as_ptr(), buffer);
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("filtered")].text,
            published
        );
        assert!(
            matches!(actor.computed("strings.filtered"), Some(blockui_runtime::Value::TextList(items)) if items.as_ptr() == memo_buffer)
        );
        let slot = actor.slot_address("strings");
        actor.replace_for_test::<Replacement>()?;
        assert_eq!(actor.slot_address("strings"), slot);
        assert_eq!(actor.text_list("strings"), retained);
        assert_eq!(
            actor.computed("strings.filtered"),
            Some(&blockui_runtime::Value::TextList(vec![
                "a\0b".into(),
                "标题😀".into()
            ]))
        );
        Ok(())
    }
}

mod sliced_defaults {
    use super::*;
    use blockui::{actor, actor_impl};
    const DEFAULTS: &[&str] = &["draft", "标题😀", "a\0b"];
    const EMPTY: &[&str] = &[];
    #[actor(name = "slices")]
    struct Model {
        #[state(default = DEFAULTS)]
        labels: State<Vec<String>>,
        #[state(default = EMPTY)]
        empty: State<Vec<String>>,
        #[state(default = &[])]
        inline_empty: State<Vec<String>>,
    }
    #[actor_impl]
    impl Model {
        fn view(&self, _: &Cx<'_>) -> View {
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(self.labels.map_text(|labels| labels.join(","))));
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn named_string_slices_initialize_host_owned_defaults() -> std::result::Result<(), String> {
        let actor = blockui_runtime::Actor::for_test::<Model>()?;
        assert_eq!(actor.text_list("slices.labels"), DEFAULTS);
        assert!(actor.text_list("slices.empty").is_empty());
        assert!(actor.text_list("slices.inline_empty").is_empty());
        assert_ne!(
            actor.text_list("slices.labels")[0].as_ptr(),
            DEFAULTS[0].as_ptr()
        );
        Ok(())
    }
}

mod declared_tasks {
    use super::*;
    use blockui::{File, Key, Resource, Task, actor, actor_impl};
    #[actor(name = "clock")]
    struct Clock {
        count: State<i64>,
        timer: Task,
        #[resource(key = "clock.document")]
        file: Resource<File>,
        #[task(key = "clock.cancel", reload = blockui::ReloadPolicy::Cancel)]
        cancelled: Task,
    }
    #[actor_impl]
    impl Clock {
        #[action]
        fn start(&self, cx: &Cx<'_>) -> Result<()> {
            let bound = self.bind(cx)?;
            assert_eq!(bound.file.key(), self.file.key());
            bound.cancelled.delay(
                std::time::Duration::from_millis(25),
                Action::remote("clock.cancelled"),
                99,
            )?;
            bound.timer.delay(
                std::time::Duration::from_millis(20),
                Self::finish_action(),
                7,
            )
        }
        #[action(name = "clock.finish")]
        fn finish(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
            self.count.set(cx, value)
        }
        fn view(&self, _: &Cx<'_>) -> View {
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(self.count.map_text(|value| value.to_string())));
                }
                __blockui_tree.finish()
            }
        }
    }
    struct Replacement;
    impl Actor for Replacement {
        const STATE: &'static [StateSpec] = Clock::STATE;
        const ACTIONS: &'static [Action] = &[
            Action::new("clock.finish", |cx, value| {
                State::<i64>::new("clock.count", 0).set(cx, value + 10)
            }),
            Action::new("clock.cancelled", |cx, value| {
                State::<i64>::new("clock.count", 0).set(cx, value)
            }),
        ];
        fn view(cx: &Cx<'_>) -> Result<View> {
            <Clock as Actor>::view(cx)
        }
    }
    #[test]
    fn declared_task_identity_is_scoped_and_completion_uses_new_code()
    -> std::result::Result<(), String> {
        assert_eq!(Clock::SCHEMA.len(), 1);
        assert_eq!(Clock::new().timer.key(), blockui::abi::key("clock.timer"));
        assert_eq!(Clock::new().file.key(), blockui::abi::key("clock.document"));
        let scope = Key::named("child");
        assert_eq!(
            Clock::new().scoped(scope).file.key(),
            scope.scoped(Clock::new().file.key()).0
        );
        assert_eq!(
            Clock::new().scoped(scope).timer.key(),
            scope.scoped(Clock::new().timer.key()).0
        );
        let mut actor = blockui_runtime::Actor::for_test::<Clock>()?;
        let slot = actor.slot_address("clock.count");
        actor.tell(Clock::start_action(), 0)?;
        actor.tick()?;
        actor.replace_for_test::<Replacement>()?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while actor.integer("clock.count") != 17 && std::time::Instant::now() < deadline {
            actor.tick()?;
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::thread::sleep(std::time::Duration::from_millis(40));
        actor.tick()?;
        assert_eq!(actor.integer("clock.count"), 17);
        assert_eq!(actor.slot_address("clock.count"), slot);
        Ok(())
    }
}

mod render_receiver_actions {
    use blockui::*;
    #[actor(name = "render.actions")]
    struct Model {
        count: State<i64>,
        enabled: State<bool>,
        amount: State<f64>,
        label: State<String>,
    }
    #[actor_impl(render)]
    impl Model {
        #[action]
        fn rename(&self, cx: &Cx<'_>, value: std::string::String) -> Result<()> {
            self.label.set(cx, &value)
        }
        #[action]
        fn forward_name(&self, cx: &Cx<'_>, target: i64) -> Result<()> {
            ActorRef(target as u64).tell(cx, Self::rename_action(), "資料 😀".to_owned())
        }
        #[scope(name = "render.panel")]
        fn panel(&self, cx: &Cx<'_>) -> Result<View> {
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree
                        .add(Text::new(self.count.get(cx)?.to_string()).key("render.count"));
                }
                __blockui_tree.finish()
            })
        }
        #[action(name = "render.bump")]
        fn bump(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
            self.count.set(cx, value)
        }
        #[action(name = "render.enable")]
        fn enable(&self, cx: &Cx<'_>, value: bool) -> Result<()> {
            self.enabled.set(cx, value)
        }
        #[action(name = "render.amount")]
        fn amount(&self, cx: &Cx<'_>, value: f64) -> Result<()> {
            self.amount.set(cx, value)
        }
    }
    impl Render for ModelBound<'_, '_> {
        fn render(&self) -> View {
            let _: TextAction = self.rename_action();
            let _: BoolAction = self.enable_action();
            let _: FloatAction = self.amount_action();
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Button {
                        on_click: self.bump_action(),
                        ..default()
                    });
                    __blockui_tree.add(self.panel_scope());
                    __blockui_tree.add(Text::new(&self.label).key("render.label"));
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn direct_typed_text_tell_queues_across_replacement_and_enforces_bounds()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let token = Model::<DescriptorMode>::rename_action();
        let address = actor.slot_address("render.actions.label");
        actor.tell(token, "queued 資料".to_owned())?;
        assert_eq!(actor.text("render.actions.label"), "");
        actor.replace_for_test::<Model>()?;
        actor.tick()?;
        assert_eq!(actor.text("render.actions.label"), "queued 資料");
        assert_eq!(actor.slot_address("render.actions.label"), address);
        assert_eq!(
            actor.view.bindings[&abi::key("render.label")].text,
            "queued 資料"
        );
        for text in [String::new(), "x".repeat(abi::MAX_MESSAGE_TEXT_BYTES)] {
            actor.tell(token, text.clone())?;
            actor.tick()?;
            assert_eq!(actor.text("render.actions.label"), text);
        }
        assert!(
            actor
                .tell(token, "x".repeat(abi::MAX_MESSAGE_TEXT_BYTES + 1))
                .is_err()
        );
        assert!(actor.mailbox.is_empty());
        for _ in 0..256 {
            actor.tell(token, "waiting".to_owned())?;
        }
        assert_eq!(
            actor.tell(token, "overflow".to_owned()).unwrap_err(),
            "mailbox full"
        );
        assert_eq!(actor.mailbox.len(), 256);
        for _ in 0..8 {
            actor.tick()?;
        }
        assert!(actor.mailbox.is_empty());
        assert_eq!(actor.text("render.actions.label"), "waiting");
        Ok(())
    }
    #[test]
    fn bound_string_action_updates_only_its_recipient_reactive_text()
    -> std::result::Result<(), String> {
        use blockui_runtime::{ActorId, Runtime};
        let root = std::env::temp_dir().join(format!("identity-bound-text-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn_for_test::<Model>(ActorId(1))?;
            runtime.spawn_for_test::<Model>(ActorId(2))?;
            let address = runtime
                .actor(ActorId(2))
                .unwrap()
                .slot_address("render.actions.label");
            runtime.tell(
                ActorId(1),
                blockui_runtime::Message {
                    action: Model::<DescriptorMode>::forward_name_action().id,
                    value: 2,
                },
            )?;
            for _ in 0..3 {
                assert!(runtime.tick().is_empty());
            }
            let recipient = runtime.actor(ActorId(2)).unwrap();
            assert_eq!(recipient.text("render.actions.label"), "資料 😀");
            assert_eq!(recipient.slot_address("render.actions.label"), address);
            assert_eq!(
                recipient.view.bindings[&abi::key("render.label")].text,
                "資料 😀"
            );
            assert_eq!(
                runtime
                    .actor(ActorId(1))
                    .unwrap()
                    .text("render.actions.label"),
                ""
            );
        }
        std::fs::remove_dir(root).unwrap();
        Ok(())
    }
    #[test]
    fn bound_action_descriptors_preserve_scalar_types_and_route_current_code()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        for (name, value) in [
            ("render.bump", 7),
            ("render.enable", 1),
            ("render.amount", 2.5_f64.to_bits() as i64),
        ] {
            actor.dispatch(blockui_runtime::Message {
                action: abi::key(name),
                value,
            })?;
        }
        assert_eq!(actor.scope_ids(), vec![abi::key("render.panel")]);
        assert!(actor.view.root_dependencies.is_empty());
        assert_eq!(actor.view.bindings[&abi::key("render.count")].text, "7");
        assert_eq!(actor.integer("render.actions.count"), 7);
        assert!(actor.boolean("render.actions.enabled"));
        assert_eq!(actor.float("render.actions.amount"), 2.5);
        Ok(())
    }
}

mod isolated_structural_work {
    use super::*;
    use blockui::Scope;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static ROOTS: AtomicUsize = AtomicUsize::new(0);
    static FIXED: AtomicUsize = AtomicUsize::new(0);
    static DYNAMIC: AtomicUsize = AtomicUsize::new(0);
    const FIXED_SCOPE: Scope = Scope::new("isolated.fixed", |_| {
        FIXED.fetch_add(1, Ordering::Relaxed);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new(TITLE).key("isolated.title"));
            }
            __blockui_tree.finish()
        })
    });
    const DYNAMIC_SCOPE: Scope = Scope::new("isolated.dynamic", |cx| {
        DYNAMIC.fetch_add(1, Ordering::Relaxed);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                if COUNT.get(cx)? > 0 {
                    __blockui_tree.add(Text::new("Visible").key("isolated.visible"));
                }
            }
            __blockui_tree.finish()
        })
    });
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] = &[Action::new("isolated.count", |cx, value| {
            COUNT.set(cx, value)
        })];
        const SCOPES: &'static [Scope] = &[FIXED_SCOPE, DYNAMIC_SCOPE];
        fn view(_: &Cx<'_>) -> Result<View> {
            ROOTS.fetch_add(1, Ordering::Relaxed);
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(FIXED_SCOPE)
                            .__local_identity(stringify!(FIXED_SCOPE)),
                    );
                    __blockui_tree.add(
                        ::blockui::IntoView::into_view(DYNAMIC_SCOPE)
                            .__local_identity(stringify!(DYNAMIC_SCOPE)),
                    );
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn scoped_structure_changes_do_not_rebuild_root_or_fixed_panel()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let roots = ROOTS.load(Ordering::Relaxed);
        let fixed = FIXED.load(Ordering::Relaxed);
        let dynamic = DYNAMIC.load(Ordering::Relaxed);
        let title = abi_key("isolated.title");
        let evaluations = actor.view.bindings[&title].evaluations;
        actor.dispatch(blockui_runtime::Message {
            action: abi_key("isolated.count"),
            value: 1,
        })?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
        assert_eq!(FIXED.load(Ordering::Relaxed), fixed);
        assert_eq!(DYNAMIC.load(Ordering::Relaxed), dynamic + 1);
        assert_eq!(actor.view.bindings[&title].evaluations, evaluations);
        assert!(
            actor
                .view
                .bindings
                .contains_key(&abi_key("isolated.visible"))
        );
        actor.dispatch(blockui_runtime::Message {
            action: abi_key("isolated.count"),
            value: 0,
        })?;
        assert_eq!(ROOTS.load(Ordering::Relaxed), roots);
        assert_eq!(FIXED.load(Ordering::Relaxed), fixed);
        assert_eq!(DYNAMIC.load(Ordering::Relaxed), dynamic + 2);
        assert!(
            !actor
                .view
                .bindings
                .contains_key(&abi_key("isolated.visible"))
        );
        Ok(())
    }
    fn abi_key(name: &str) -> u64 {
        blockui::abi::key(name)
    }
}

mod text_list_ranges {
    use blockui::*;
    #[actor(name = "ranges")]
    struct Model {
        #[state(default = &["first", "标题😀", "a\0b", ""])]
        items: State<Vec<String>>,
    }
    #[actor_impl]
    impl Model {
        #[computed]
        fn reversed(&self, cx: &Cx<'_>) -> Result<Vec<String>> {
            let mut values = self.items.get(cx)?;
            values.reverse();
            Ok(values)
        }
        #[action(name = "ranges.clear")]
        fn clear(&self, cx: &Cx<'_>) -> Result<()> {
            self.items.set(cx, &[])
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            let full = self.items.get(cx)?;
            assert_eq!(self.items.len(cx)?, full.len());
            assert_eq!(self.items.is_empty(cx)?, full.is_empty());
            assert_eq!(
                self.items.get_range(cx, 1..usize::MAX)?,
                full.iter().skip(1).cloned().collect::<Vec<_>>()
            );
            assert!(self.items.get_range(cx, usize::MAX..usize::MAX)?.is_empty());
            let start = 2;
            assert!(self.items.get_range(cx, start..1).is_err());
            let computed = Self::reversed_computed();
            let reversed = full.iter().rev().cloned().collect::<Vec<_>>();
            assert_eq!(computed.len(cx)?, reversed.len());
            assert_eq!(computed.is_empty(cx)?, reversed.is_empty());
            assert_eq!(
                computed.get_range(cx, 0..2)?,
                reversed.into_iter().take(2).collect::<Vec<_>>()
            );
            assert!(computed.get_range(cx, 0..0)?.is_empty());
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(full.len().to_string()));
                }
                __blockui_tree.finish()
            })
        }
    }
    impl Render for ModelBound<'_, '_> {
        fn render(&self) -> View {
            assert_eq!(self.items.is_empty(), self.items.get_range(0..1).is_empty());
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(format!(
                        "{}:{:?}",
                        self.items.len(),
                        self.items.get_range(0..2)
                    )));
                }
                __blockui_tree.finish()
            }
        }
    }
    struct BoundModel;
    impl Actor for BoundModel {
        const STATE: &'static [StateSpec] = Model::STATE;
        const ACTIONS: &'static [Action] = Model::ACTIONS;
        fn view(cx: &Cx<'_>) -> Result<View> {
            render_bound(&Model::new(), cx)
        }
    }
    #[test]
    fn bound_render_reads_ranges_without_explicit_context() -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<BoundModel>()?;
        assert!(
            actor
                .view
                .bindings
                .values()
                .any(|binding| binding.text.contains("标题😀"))
        );
        actor.dispatch(blockui_runtime::Message {
            action: abi::key("ranges.clear"),
            value: 0,
        })?;
        assert!(
            actor
                .view
                .bindings
                .values()
                .any(|binding| binding.text == "0:[]")
        );
        Ok(())
    }
    #[test]
    fn clipped_ranges_preserve_unicode_nul_and_track_collection_changes()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        actor.dispatch(blockui_runtime::Message {
            action: abi::key("ranges.clear"),
            value: 0,
        })?;
        assert!(actor.text_list("ranges.items").is_empty());
        assert!(
            actor
                .view
                .bindings
                .values()
                .any(|binding| binding.text == "0")
        );
        Ok(())
    }
}

mod integer_list_lengths {
    use blockui::*;
    #[actor(name = "lengths")]
    struct Model {
        #[state(default = &[1, 2, 3])]
        items: State<Vec<i64>>,
    }
    #[actor_impl]
    impl Model {
        #[computed]
        fn doubled(&self, cx: &Cx<'_>) -> Result<Vec<i64>> {
            Ok(self.items.get(cx)?.into_iter().map(|v| v * 2).collect())
        }
        fn view(&self, cx: &Cx<'_>) -> Result<View> {
            let computed = Self::doubled_computed();
            assert_eq!(computed.len(cx)?, self.items.len(cx)?);
            assert_eq!(computed.is_empty(cx)?, self.items.is_empty(cx)?);
            let full = self.items.get(cx)?;
            assert_eq!(
                self.items.get_range(cx, 1..usize::MAX)?,
                full.iter().skip(1).copied().collect::<Vec<_>>()
            );
            assert_eq!(
                computed.get_range(cx, 1..2)?,
                full.iter()
                    .skip(1)
                    .take(1)
                    .map(|v| v * 2)
                    .collect::<Vec<_>>()
            );
            assert!(self.items.get_range(cx, usize::MAX..usize::MAX)?.is_empty());
            assert!(computed.get_range(cx, usize::MAX..usize::MAX)?.is_empty());
            let start = 2;
            assert!(self.items.get_range(cx, start..1).is_err());
            assert!(computed.get_range(cx, start..1).is_err());
            render_bound(self, cx)
        }
        #[action(name = "lengths.clear")]
        fn clear(&self, cx: &Cx<'_>) -> Result<()> {
            self.items.set(cx, &[])
        }
    }
    impl Render for ModelBound<'_, '_> {
        fn render(&self) -> View {
            assert_eq!(self.items.get_range(0..1).is_empty(), self.items.is_empty());
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(format!(
                        "{}:{}",
                        self.items.len(),
                        self.items.is_empty()
                    )));
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn bound_and_computed_lengths_track_clearing() -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        assert!(actor.view.bindings.values().any(|b| b.text == "3:false"));
        actor.dispatch(blockui_runtime::Message {
            action: abi::key("lengths.clear"),
            value: 0,
        })?;
        assert!(actor.view.bindings.values().any(|b| b.text == "0:true"));
        Ok(())
    }
}

mod button_arguments {
    use super::*;
    use blockui::{Button, IntoView};
    struct Presets<const NEW: bool>;
    impl<const NEW: bool> Actor for Presets<NEW> {
        const STATE: &'static [StateSpec] = Counter::STATE;
        const ACTIONS: &'static [Action] = &[
            Action::new("set.integer", |cx, value| {
                COUNT.set(cx, if NEW { value + 100 } else { value })
            }),
            BOOLEAN.spec(),
            FLOAT.spec(),
        ];
        fn view(_: &Cx<'_>) -> Result<View> {
            let integer = Self::ACTIONS[0];
            Ok(Column::default().children(vec![
                Button::default()
                    .on_click_with(integer, if NEW { 9 } else { -7 })
                    .key("integer"),
                Button::default()
                    .on_click_with(BOOLEAN, true)
                    .key("boolean"),
                Button::default()
                    .on_click_with(FLOAT, f64::from_bits(0xfff8_0000_0000_1234))
                    .key("float"),
                Button {
                    enabled: false,
                    ..Default::default()
                }
                .on_click_with(integer, 99)
                .key("disabled"),
                Button {
                    on_click: integer,
                    ..Default::default()
                }
                .into_view()
                .key("ordinary"),
            ]))
        }
    }
    #[test]
    fn current_projection_dispatches_typed_arguments_after_replacement()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Presets<false>>()?;
        let address = actor.slot_address("count");
        actor.dispatch_widget(blockui::abi::key("integer"))?;
        assert_eq!(actor.integer("count"), -7);
        actor.dispatch_widget(blockui::abi::key("boolean"))?;
        assert_eq!(actor.integer("count"), 1);
        actor.dispatch_widget(blockui::abi::key("float"))?;
        assert_eq!(actor.integer("count") as u64, 0xfff8_0000_0000_1234);
        actor.replace_for_test::<Presets<true>>()?;
        assert_eq!(actor.slot_address("count"), address);
        assert_eq!(actor.integer("count") as u64, 0xfff8_0000_0000_1234);
        actor.dispatch_widget(blockui::abi::key("integer"))?;
        assert_eq!(actor.integer("count"), 109);
        actor.dispatch_widget(blockui::abi::key("disabled"))?;
        assert_eq!(actor.integer("count"), 109);
        actor.dispatch_widget(blockui::abi::key("ordinary"))?;
        assert_eq!(actor.integer("count"), 100);
        Ok(())
    }
}

mod application_errors {
    use super::*;
    struct Validated;
    impl Actor for Validated {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] = &[Action::new("validate", |cx, value| {
            COUNT.set(cx, value)?;
            TITLE.set(cx, "pending")?;
            if value == i64::MAX - 1 {
                std::panic::panic_any(42_u32);
            }
            if value == i64::MAX - 2 {
                std::panic::panic_any(String::from("owned panic text"));
            }
            if value == i64::MAX {
                panic!("intentional domain panic");
            }
            if value == i64::MIN {
                return Err(Error::Application(format!("{}😀tail", "x".repeat(4095))));
            }
            if value < 0 {
                return Err(Error::Application(format!(
                    "Count {value} must be nonnegative"
                )));
            }
            TITLE.set(cx, "accepted")
        })];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
    }
    #[test]
    fn recoverable_domain_error_rolls_back_turn_and_actor_remains_usable()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Validated>()?;
        let address = actor.slot_address("count");
        actor.dispatch(blockui_runtime::Message {
            action: blockui::abi::key("validate"),
            value: 3,
        })?;
        let snapshot = actor.view.bindings[&blockui::abi::key("count.text")]
            .text
            .clone();
        assert_eq!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: blockui::abi::key("validate"),
                    value: -1,
                })
                .unwrap_err(),
            "Count -1 must be nonnegative"
        );
        assert_eq!(actor.integer("count"), 3);
        assert_eq!(actor.text("title"), "accepted");
        assert_eq!(actor.slot_address("count"), address);
        assert_eq!(
            actor.view.bindings[&blockui::abi::key("count.text")].text,
            snapshot
        );
        let truncated = actor
            .dispatch(blockui_runtime::Message {
                action: blockui::abi::key("validate"),
                value: i64::MIN,
            })
            .unwrap_err();
        assert_eq!(truncated, "x".repeat(4095));
        assert_eq!(actor.integer("count"), 3);
        assert_eq!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: blockui::abi::key("validate"),
                    value: i64::MAX
                })
                .unwrap_err(),
            "actor callback panicked: intentional domain panic"
        );
        assert_eq!(actor.integer("count"), 3);
        assert_eq!(actor.text("title"), "accepted");
        for (value, expected) in [
            (
                i64::MAX - 1,
                "actor callback panicked: non-string panic payload",
            ),
            (i64::MAX - 2, "actor callback panicked: owned panic text"),
        ] {
            assert_eq!(
                actor
                    .dispatch(blockui_runtime::Message {
                        action: blockui::abi::key("validate"),
                        value
                    })
                    .unwrap_err(),
                expected
            );
            assert_eq!(actor.integer("count"), 3);
            assert_eq!(actor.text("title"), "accepted");
        }
        actor.replace_for_test::<Validated>()?;
        actor.dispatch(blockui_runtime::Message {
            action: blockui::abi::key("validate"),
            value: 4,
        })?;
        assert_eq!(actor.integer("count"), 4);
        assert_eq!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: 0,
                    value: 0
                })
                .unwrap_err(),
            "unknown actor action"
        );
        assert_eq!(
            Error::Application("Invalid name".into()).to_string(),
            "Invalid name"
        );
        Ok(())
    }
}

mod candidate_diagnostics {
    use super::*;
    use blockui::{Computed, ComputedSpec, IntoView, Scope};
    struct BadView;
    impl Actor for BadView {
        const STATE: &'static [StateSpec] = Counter::STATE;
        const ACTIONS: &'static [Action] = Counter::ACTIONS;
        fn view(_: &Cx<'_>) -> Result<View> {
            Err(Error::Application("candidate view unavailable".into()))
        }
    }
    const BAD: Computed<i64> = Computed::<i64>::new("bad", |_| {
        Err(Error::Application("computed input invalid".into()))
    });
    struct BadComputed;
    impl Actor for BadComputed {
        const STATE: &'static [StateSpec] = Counter::STATE;
        const ACTIONS: &'static [Action] = Counter::ACTIONS;
        const COMPUTEDS: &'static [ComputedSpec] = &[BAD.spec()];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(vec![Text::new(BAD).into_view()]))
        }
    }
    const PANEL: Scope = Scope::new("bad.panel", |_| {
        Err(Error::Application("scope data missing".into()))
    });
    struct BadScope;
    impl Actor for BadScope {
        const STATE: &'static [StateSpec] = Counter::STATE;
        const ACTIONS: &'static [Action] = Counter::ACTIONS;
        const SCOPES: &'static [Scope] = &[PANEL];
        fn view(_: &Cx<'_>) -> Result<View> {
            Ok(Column::default().children(vec![PANEL.into_view()]))
        }
    }
    struct BadMigration;
    impl Actor for BadMigration {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const ACTIONS: &'static [Action] = Counter::ACTIONS;
        fn view(cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
        fn migrate(cx: &Cx<'_>, _: u64) -> Result<()> {
            COUNT.set(cx, 777)?;
            Err(Error::Application("unsupported document schema".into()))
        }
    }
    #[test]
    fn rejected_candidates_return_details_and_preserve_the_current_actor()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Counter>()?;
        actor.dispatch(message(0))?;
        let version = actor.implementation_version();
        let address = actor.slot_address("count");
        assert_eq!(
            actor.replace_for_test::<BadView>().unwrap_err(),
            "candidate view unavailable"
        );
        assert_eq!(
            actor.replace_for_test::<BadComputed>().unwrap_err(),
            format!(
                "computed {:#018x} evaluation failed: computed input invalid",
                BAD.key()
            )
        );
        assert_eq!(
            actor.replace_for_test::<BadScope>().unwrap_err(),
            format!(
                "scope {:#018x} projection failed: scope data missing",
                PANEL.id()
            )
        );
        assert_eq!(
            actor.replace_for_test::<BadMigration>().unwrap_err(),
            "unsupported document schema"
        );
        assert_eq!(actor.integer("count"), 1);
        assert_eq!(actor.slot_address("count"), address);
        assert_eq!(actor.implementation_version(), version);
        actor.dispatch(message(0))?;
        assert_eq!(actor.integer("count"), 2);
        Ok(())
    }
}

mod request_feedback {
    use super::*;
    use blockui::{ActorRef, Render, RequestError, actor, actor_impl};
    const QUERY: Action = Action::request("feedback.query", |cx, _| {
        COUNT.set(cx, 999)?;
        Err(Error::Application("文件已关闭，请重试".into()))
    });
    struct Service;
    impl Actor for Service {
        const STATE: &'static [StateSpec] = Counter::STATE;
        const ACTIONS: &'static [Action] = &[QUERY];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
    }
    struct RepairedService;
    impl Actor for RepairedService {
        const STATE: &'static [StateSpec] = Counter::STATE;
        const ACTIONS: &'static [Action] = &[Action::request("feedback.query", |_, _| Ok(42))];
        fn view(cx: &Cx<'_>) -> Result<View> {
            Counter::view(cx)
        }
    }
    #[actor(name = "feedback")]
    struct Client {
        #[state(key = "feedback.message")]
        status: State<String>,
    }
    #[actor_impl(render)]
    impl Client {
        #[action(name = "feedback.load")]
        fn load(&self, cx: &Cx<'_>) -> Result<()> {
            self.status.set(cx, "Loading…")?;
            ActorRef(2).ask(
                cx,
                QUERY,
                0,
                Self::success_action(),
                Self::failed_action(),
                std::time::Duration::from_secs(30),
            )?;
            Ok(())
        }
        #[action(name = "feedback.success")]
        fn success(&self, cx: &Cx<'_>, value: i64) -> Result<()> {
            self.status.set(cx, &format!("Loaded {value}"))
        }
        #[action(name = "feedback.failure")]
        fn failed(&self, cx: &Cx<'_>, error: blockui::RequestError) -> Result<()> {
            assert_eq!(error, RequestError::HandlerFailed);
            self.status.set(cx, &cx.request_error()?.unwrap())
        }
    }
    impl Render for ClientBound<'_, '_> {
        fn render(&self) -> View {
            let _: Action = self.failed_action();
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(&self.status).key("feedback.text"));
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn request_error_is_committed_to_reactive_ui_state_and_retry_uses_repaired_service()
    -> std::result::Result<(), String> {
        use blockui_runtime::{ActorId, Runtime};
        let root =
            std::env::temp_dir().join(format!("identity-request-feedback-{}", std::process::id()));
        {
            let mut runtime = Runtime::new(&root);
            runtime.spawn_for_test::<Client>(ActorId(1))?;
            runtime.spawn_for_test::<Service>(ActorId(2))?;
            let address = runtime
                .actor(ActorId(1))
                .unwrap()
                .slot_address("feedback.message");
            runtime.tell(
                ActorId(1),
                blockui_runtime::Message {
                    action: Client::<blockui::DescriptorMode>::load_action().id,
                    value: 0,
                },
            )?;
            let mut faults = Vec::new();
            for _ in 0..3 {
                faults.extend(runtime.tick());
            }
            assert_eq!(faults.len(), 1);
            assert_eq!(faults[0].error, "文件已关闭，请重试");
            let client = runtime.actor(ActorId(1)).unwrap();
            assert_eq!(client.text("feedback.message"), "文件已关闭，请重试");
            assert_eq!(
                client.view.bindings[&blockui::abi::key("feedback.text")].text,
                "文件已关闭，请重试"
            );
            assert_eq!(runtime.actor(ActorId(2)).unwrap().integer("count"), 0);
            runtime.replace_for_test::<Client>(ActorId(1))?;
            runtime.replace_for_test::<RepairedService>(ActorId(2))?;
            assert_eq!(
                runtime
                    .actor(ActorId(1))
                    .unwrap()
                    .slot_address("feedback.message"),
                address
            );
            runtime.tell(
                ActorId(1),
                blockui_runtime::Message {
                    action: Client::<blockui::DescriptorMode>::load_action().id,
                    value: 0,
                },
            )?;
            for _ in 0..3 {
                assert!(runtime.tick().is_empty());
            }
            let client = runtime.actor(ActorId(1)).unwrap();
            assert_eq!(client.text("feedback.message"), "Loaded 42");
            assert_eq!(
                client.view.bindings[&blockui::abi::key("feedback.text")].text,
                "Loaded 42"
            );
        }
        std::fs::remove_dir_all(root).map_err(|e| e.to_string())?;
        Ok(())
    }
}

mod receiver_request_replies {
    use blockui::*;
    #[actor(name = "receiver.requests")]
    struct Queries {
        #[state(key = "calls")]
        calls: State<i64>,
    }
    #[actor_impl(render)]
    impl Queries {
        #[action]
        fn boolean_float(&self, cx: &Cx<'_>, input: bool) -> Result<f64> {
            self.calls.update(cx, |calls| Ok(calls + 1))?;
            Ok(if input { -0.0 } else { 1.5 })
        }
        #[action]
        fn float_boolean(
            &self,
            cx: &Cx<'_>,
            input: f64,
        ) -> std::result::Result<bool, blockui::Error> {
            self.calls.update(cx, |calls| Ok(calls + 1))?;
            Ok(input.is_sign_negative())
        }
        #[action]
        fn count(&self, cx: &Cx<'_>) -> Result<core::primitive::i64> {
            self.calls.get(cx)
        }
    }
    impl Render for QueriesBound<'_, '_> {
        fn render(&self) -> View {
            let _: RequestAction<bool, f64> = self.boolean_float_action();
            let _: RequestAction<f64, bool> = self.float_boolean_action();
            let _: RequestAction<i64, i64> = self.count_action();
            {
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    __blockui_tree.add(Text::new(
                        self.calls.map_text(|calls| format!("Calls: {calls}")),
                    ));
                }
                __blockui_tree.finish()
            }
        }
    }
    #[test]
    fn receiver_requests_preserve_scalar_types_validate_input_and_roll_back_tells()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Queries>()?;
        let bool_float: RequestAction<bool, f64> =
            Queries::<DescriptorMode>::boolean_float_action();
        let float_bool: RequestAction<f64, bool> =
            Queries::<DescriptorMode>::float_boolean_action();
        let dispatch = |actor: &mut blockui_runtime::Actor, action, value| {
            actor.dispatch_request(blockui_runtime::Message { action, value })
        };
        assert_eq!(
            dispatch(&mut actor, bool_float.reply_id(), 1)?.unwrap() as u64,
            (-0.0_f64).to_bits()
        );
        assert_eq!(
            dispatch(
                &mut actor,
                float_bool.reply_id(),
                (-0.0_f64).to_bits() as i64
            )?,
            Some(1)
        );
        assert_eq!(actor.integer("calls"), 2);
        assert!(dispatch(&mut actor, bool_float.reply_id(), 2).is_err());
        assert_eq!(actor.integer("calls"), 2);
        assert!(
            actor
                .dispatch(blockui_runtime::Message {
                    action: bool_float.reply_id(),
                    value: 0
                })
                .is_err()
        );
        assert_eq!(
            actor.integer("calls"),
            2,
            "ordinary tell must roll back writes when no reply scope exists"
        );
        assert_eq!(
            dispatch(
                &mut actor,
                Queries::<DescriptorMode>::count_action().reply_id(),
                0
            )?,
            Some(2)
        );
        Ok(())
    }
}

const CLOCK_VALUE: State<i64> = State::<i64>::new("clock.value", 0);
struct HostClock;
impl Actor for HostClock {
    const STATE: &'static [StateSpec] = &[CLOCK_VALUE.spec()];
    const ACTIONS: &'static [Action] = &[Action::new("clock.sample", |cx, fail| {
        let now = i64::try_from(cx.monotonic_ms()?).map_err(|_| Error::Overflow)?;
        CLOCK_VALUE.set(cx, now)?;
        if fail != 0 {
            return Err(Error::Application("rejected sample".into()));
        }
        Ok(())
    })];
    fn view(cx: &Cx<'_>) -> Result<View> {
        if cx.monotonic_ms().is_ok() {
            return Err(Error::Application("clock readable in view".into()));
        }
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(Text::new("Host clock"));
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn sdk_clock_epoch_survives_replacement_and_failed_sample_rolls_back()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<HostClock>()?;
    let sample = |value| blockui_runtime::Message {
        action: blockui::abi::key("clock.sample"),
        value,
    };
    actor.dispatch(sample(0))?;
    let first = actor.integer("clock.value");
    let slot = actor.slot_address("clock.value");
    std::thread::sleep(std::time::Duration::from_millis(3));
    actor.replace_for_test::<HostClock>()?;
    assert_eq!(actor.slot_address("clock.value"), slot);
    assert_eq!(actor.integer("clock.value"), first);
    actor.dispatch(sample(0))?;
    assert!(actor.integer("clock.value") >= first + 3);
    let retained = actor.integer("clock.value");
    assert!(actor.dispatch(sample(1)).is_err());
    assert_eq!(actor.integer("clock.value"), retained);
    Ok(())
}

const SLIDER_VALUE: State<i64> = State::<i64>::new("slider.value", 50);
struct SliderActor<const DISABLED: bool>;
impl<const DISABLED: bool> Actor for SliderActor<DISABLED> {
    const STATE: &'static [StateSpec] = &[SLIDER_VALUE.spec()];
    const ACTIONS: &'static [Action] = &[Action::new("slider.change", |cx, value| {
        SLIDER_VALUE.set(cx, value)?;
        if value == 80 {
            return Err(Error::Application("rejected edit".into()));
        }
        Ok(())
    })];
    fn view(_: &Cx<'_>) -> Result<View> {
        let slider = blockui::Slider {
            enabled: !DISABLED,
            ..blockui::Slider::new(
                SLIDER_VALUE,
                blockui::IntegerRange {
                    min: 0,
                    max: 100,
                    step: 10,
                },
                Self::ACTIONS[0],
            )
            .label("Duration")
            .key("slider.input")
        };
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(slider).__local_identity(stringify!(slider)),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn slider_uses_current_projection_range_action_and_transaction() -> std::result::Result<(), String>
{
    let mut actor = blockui_runtime::Actor::for_test::<SliderActor<false>>()?;
    let widget = blockui::abi::key("slider.input");
    let slot = actor.slot_address("slider.value");
    actor.edit_slider(widget, 70)?;
    assert_eq!(actor.integer("slider.value"), 70);
    for rejected in [-1, 101, 71, 80] {
        assert!(actor.edit_slider(widget, rejected).is_err());
        assert_eq!(actor.integer("slider.value"), 70);
    }
    assert_eq!(actor.view.bindings[&widget].text, "Duration");
    actor.replace_for_test::<SliderActor<true>>()?;
    assert_eq!(actor.slot_address("slider.value"), slot);
    assert!(actor.edit_slider(widget, 60).is_err());
    assert_eq!(actor.integer("slider.value"), 70);
    actor.replace_for_test::<SliderActor<false>>()?;
    actor.edit_slider(widget, 60)?;
    assert_eq!(actor.integer("slider.value"), 60);
    assert!(actor.edit_slider(0, 50).is_err());
    Ok(())
}

struct InvalidSlider<const CASE: u8>;
impl<const CASE: u8> Actor for InvalidSlider<CASE> {
    const STATE: &'static [StateSpec] = SliderActor::<false>::STATE;
    const ACTIONS: &'static [Action] = SliderActor::<false>::ACTIONS;
    fn view(_: &Cx<'_>) -> Result<View> {
        let mut range = blockui::IntegerRange {
            min: 0,
            max: 100,
            step: 10,
        };
        match CASE {
            0 => range.step = 0,
            1 => range.max = range.min,
            2 => range.step = 3,
            3 => range.max = i32::MAX,
            _ => range.min = 80,
        }
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    blockui::Slider::new(SLIDER_VALUE, range, Self::ACTIONS[0]).key("slider.input"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn invalid_slider_replacement_keeps_previous_state_view_and_action()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<SliderActor<false>>()?;
    let widget = blockui::abi::key("slider.input");
    let slot = actor.slot_address("slider.value");
    assert!(actor.replace_for_test::<InvalidSlider<0>>().is_err());
    assert!(actor.replace_for_test::<InvalidSlider<1>>().is_err());
    assert!(actor.replace_for_test::<InvalidSlider<2>>().is_err());
    assert!(actor.replace_for_test::<InvalidSlider<3>>().is_err());
    assert!(actor.replace_for_test::<InvalidSlider<4>>().is_err());
    assert_eq!(actor.slot_address("slider.value"), slot);
    assert_eq!(actor.view.bindings[&widget].text, "Duration");
    actor.edit_slider(widget, 60)?;
    assert_eq!(actor.integer("slider.value"), 60);
    Ok(())
}

const PROGRESS_VALUE: State<f64> = State::<f64>::new("progress.value", 0.0);
struct ProgressActor;
impl Actor for ProgressActor {
    const STATE: &'static [StateSpec] = &[PROGRESS_VALUE.spec()];
    const ACTIONS: &'static [Action] = &[Action::new("progress.set", |cx, bits| {
        PROGRESS_VALUE.set(cx, f64::from_bits(bits as u64))
    })];
    fn view(_: &Cx<'_>) -> Result<View> {
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    blockui::Progress::new(PROGRESS_VALUE)
                        .label("Download")
                        .key("progress.control"),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn progress_tracks_fraction_and_invalid_updates_roll_back() -> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<ProgressActor>()?;
    let widget = blockui::abi::key("progress.control");
    let slot = actor.slot_address("progress.value");
    let set = |actor: &mut blockui_runtime::Actor, value: f64| {
        actor.dispatch(blockui_runtime::Message {
            action: blockui::abi::key("progress.set"),
            value: value.to_bits() as i64,
        })
    };
    for value in [0.0, 0.25, 1.0] {
        set(&mut actor, value)?;
        assert_eq!(actor.view.bindings[&widget].text, value.to_string());
    }
    for value in [-0.1, 1.1, f64::NAN, f64::INFINITY] {
        assert!(set(&mut actor, value).is_err());
        assert_eq!(actor.view.bindings[&widget].text, "1");
    }
    assert_eq!(
        actor.view.bindings[&widget].semantic_label.as_deref(),
        Some("Download")
    );
    actor.replace_for_test::<ProgressActor>()?;
    assert_eq!(actor.slot_address("progress.value"), slot);
    assert_eq!(actor.view.bindings[&widget].text, "1");
    assert!(actor.edit_slider(widget, 0).is_err());
    Ok(())
}

const CANVAS_X: State<f64> = State::<f64>::new("canvas.x", 0.0);
const CANVAS_PRIMARY: blockui::PointAction =
    blockui::PointAction::new("canvas.primary", |cx, point| {
        CANVAS_X.set(cx, f64::from(point.x))
    });
const CANVAS_SECONDARY: blockui::PointAction =
    blockui::PointAction::new("canvas.secondary", |cx, point| {
        CANVAS_X.set(cx, f64::from(point.x))?;
        Err(Error::Application("rejected secondary edit".into()))
    });
struct CanvasActor<const INVALID: bool>;
impl<const INVALID: bool> Actor for CanvasActor<INVALID> {
    const STATE: &'static [StateSpec] = &[CANVAS_X.spec()];
    const ACTIONS: &'static [Action] = &[CANVAS_PRIMARY.spec(), CANVAS_SECONDARY.spec()];
    fn view(_: &Cx<'_>) -> Result<View> {
        let scene = blockui::CanvasScene {
            label: "Circle canvas".into(),
            circles: vec![blockui::DrawCircle {
                center: blockui::LocalPoint { x: 20.0, y: 30.0 },
                radius: if INVALID { f32::NAN } else { 26.0 },
                fill: None,
                stroke: Some(0x2563ebff),
                stroke_width: 2.0,
            }],
        };
        let canvas = blockui::Canvas {
            on_primary: Some(CANVAS_PRIMARY),
            on_secondary: Some(CANVAS_SECONDARY),
            ..blockui::Canvas::new(scene).key("canvas.control")
        };
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                __blockui_tree.add(
                    ::blockui::IntoView::into_view(canvas).__local_identity(stringify!(canvas)),
                );
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn canvas_admits_owned_scene_routes_current_point_actions_and_rejects_bad_candidate()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<CanvasActor<false>>()?;
    let widget = blockui::abi::key("canvas.control");
    let point = blockui::LocalPoint { x: 12.5, y: 20.25 };
    let slot = actor.slot_address("canvas.x");
    actor.edit_canvas(widget, point, false)?;
    assert_eq!(actor.float("canvas.x"), 12.5);
    assert_eq!(
        actor.view.bindings[&widget].semantic_label.as_deref(),
        Some("Circle canvas")
    );
    assert!(
        actor
            .edit_canvas(widget, blockui::LocalPoint { x: 30.0, ..point }, true)
            .is_err()
    );
    assert_eq!(actor.float("canvas.x"), 12.5);
    assert!(
        actor
            .edit_canvas(
                widget,
                blockui::LocalPoint {
                    x: f32::NAN,
                    ..point
                },
                false
            )
            .is_err()
    );
    assert!(actor.edit_canvas(0, point, false).is_err());
    assert!(actor.replace_for_test::<CanvasActor<true>>().is_err());
    assert_eq!(actor.slot_address("canvas.x"), slot);
    let scene = blockui::CanvasScene::from_wire(&actor.view.bindings[&widget].text).unwrap();
    assert_eq!(scene.circles[0].radius, 26.0);
    actor.replace_for_test::<CanvasActor<false>>()?;
    actor.edit_canvas(widget, blockui::LocalPoint { x: 40.0, ..point }, false)?;
    assert_eq!(actor.float("canvas.x"), 40.0);
    Ok(())
}

const DIALOG_OPEN: State<bool> = State::<bool>::new("dialog.open", true);
struct DialogActor<const REJECT: bool>;
impl<const REJECT: bool> Actor for DialogActor<REJECT> {
    const STATE: &'static [StateSpec] = &[DIALOG_OPEN.spec()];
    const ACTIONS: &'static [Action] = &[Action::new("dialog.close", |cx, _| {
        DIALOG_OPEN.set(cx, false)?;
        if REJECT {
            return Err(Error::Application("cannot close".into()));
        }
        Ok(())
    })];
    fn view(cx: &Cx<'_>) -> Result<View> {
        let open = DIALOG_OPEN.get(cx)?;
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(Column::default());
                if open {
                    {
                        let _scope_1 = __blockui_tree.scope(
                            blockui::Dialog::new("Edit", Self::ACTIONS[0]).key("dialog.control"),
                        );
                        __blockui_tree.add(Text::new("Editor"));
                    }
                }
            }
            __blockui_tree.finish()
        })
    }
}
#[test]
fn dialog_close_is_transactional_and_resolves_current_implementation()
-> std::result::Result<(), String> {
    let mut actor = blockui_runtime::Actor::for_test::<DialogActor<true>>()?;
    let key = blockui::abi::key("dialog.control");
    assert!(actor.dismiss_dialog(key).is_err());
    assert!(actor.boolean("dialog.open"));
    assert!(actor.view.bindings.contains_key(&key));
    actor.replace_for_test::<DialogActor<false>>()?;
    actor.dismiss_dialog(key)?;
    assert!(!actor.boolean("dialog.open"));
    assert!(!actor.view.bindings.contains_key(&key));
    assert!(actor.dismiss_dialog(key).is_err());
    Ok(())
}

mod root_scope_reuse {
    use super::*;
    use blockui::{Binding, Scope};
    use std::sync::atomic::{AtomicUsize, Ordering};
    static BUILDS: AtomicUsize = AtomicUsize::new(0);
    const PANEL: Scope = Scope::new("reuse.panel", |_| {
        BUILDS.fetch_add(1, Ordering::Relaxed);
        Ok({
            let __blockui_tree = ::blockui::Ui::new();
            {
                let _scope_0 = __blockui_tree.scope(blockui::Row {
                    gap: 7.0,
                    ..Default::default()
                });
                __blockui_tree.add(
                    Text::new(Binding::named("reuse.title.binding", |cx| TITLE.get(cx)))
                        .key("reuse.title"),
                );
            }
            __blockui_tree.finish()
        })
    });
    struct Model;
    impl Actor for Model {
        const STATE: &'static [StateSpec] = &[COUNT.spec(), TITLE.spec()];
        const SCOPES: &'static [Scope] = &[PANEL];
        const ACTIONS: &'static [Action] = &[Action::new("change", |cx, value| {
            COUNT.set(cx, value)?;
            TITLE.set(cx, &format!("Title {value}"))
        })];
        fn view(cx: &Cx<'_>) -> Result<View> {
            let count = COUNT.get(cx)?;
            Ok({
                let __blockui_tree = ::blockui::Ui::new();
                {
                    let _scope_0 = __blockui_tree.scope(Column::default());
                    if count == 3 {
                        {
                            let _scope_1 =
                                __blockui_tree.scope(Column::default().key("reuse.wrapper"));
                            __blockui_tree.add(
                                ::blockui::IntoView::into_view(PANEL)
                                    .__local_identity(stringify!(PANEL)),
                            );
                        }
                    } else if count >= 0 {
                        __blockui_tree.add(
                            ::blockui::IntoView::into_view(PANEL)
                                .__local_identity(stringify!(PANEL)),
                        );
                    }
                    __blockui_tree.add(Text::new(count.to_string()).key("reuse.count"));
                }
                __blockui_tree.finish()
            })
        }
    }
    #[test]
    fn root_changes_reuse_scopes_but_refresh_bindings_remount_and_reload_builders()
    -> std::result::Result<(), String> {
        let mut actor = blockui_runtime::Actor::for_test::<Model>()?;
        let builds = BUILDS.load(Ordering::Relaxed);
        let title = blockui::abi::key("reuse.title");
        let evaluations = actor.view.bindings[&title].evaluations;
        for count in [1, 3] {
            actor.dispatch(blockui_runtime::Message {
                action: Model::ACTIONS[0].id,
                value: count,
            })?;
            assert_eq!(BUILDS.load(Ordering::Relaxed), builds);
            assert_eq!(actor.view.bindings[&title].text, format!("Title {count}"));
            assert_eq!(
                actor.view.bindings[&PANEL.id()].node.kind,
                blockui::abi::ROW
            );
        }
        assert_eq!(actor.view.bindings[&title].evaluations, evaluations + 2);
        assert_eq!(
            actor.view.bindings[&PANEL.id()].node.parent,
            blockui::abi::key("reuse.wrapper")
        );
        actor.dispatch(blockui_runtime::Message {
            action: Model::ACTIONS[0].id,
            value: -1,
        })?;
        assert!(!actor.view.scope_dependencies.contains_key(&PANEL.id()));
        assert!(!actor.view.bindings.contains_key(&title));
        actor.dispatch(blockui_runtime::Message {
            action: Model::ACTIONS[0].id,
            value: 2,
        })?;
        assert_eq!(BUILDS.load(Ordering::Relaxed), builds + 1);
        assert_eq!(actor.view.bindings[&title].text, "Title 2");
        actor.replace_for_test::<Model>()?;
        assert_eq!(BUILDS.load(Ordering::Relaxed), builds + 2);
        assert_eq!(actor.view.bindings[&title].text, "Title 2");
        Ok(())
    }
}
