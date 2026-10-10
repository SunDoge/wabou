//! Host-only staged view admission. No callback here comes from an implementation dylib.
use blockui_abi::Node;

/// Dropping an uncommitted admission must release its staged host resources.
/// Commit is infallible and runs after actor state, view and implementation commit.
pub(crate) trait Admission {
    fn commit(self: Box<Self>);
}
pub(crate) type Validator = Box<dyn FnMut(&[Node]) -> Result<Box<dyn Admission>, String>>;

pub(crate) fn validate(
    validator: &mut Option<Validator>,
    nodes: impl IntoIterator<Item = Node>,
) -> Result<Option<Box<dyn Admission>>, String> {
    let Some(validator) = validator else {
        if nodes
            .into_iter()
            .any(|node| node.kind == blockui_abi::CHILD)
        {
            return Err(
                "child declarations require a managed Runtime implementation registry".into(),
            );
        }
        return Ok(None);
    };
    let nodes: Vec<_> = nodes.into_iter().collect();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| validator(&nodes)))
        .map_err(|_| "host view admission panicked".to_owned())?
        .map(Some)
}

/// Real dylib proof for the host admission transaction used by child lifecycle staging.
pub fn proof(
    base: &std::path::Path,
    replacement: &std::path::Path,
    migration: &std::path::Path,
    dynamic: Option<&std::path::Path>,
) -> Result<(), String> {
    use crate::{Actor, ActorId, Message, Runtime};
    use std::{
        cell::{Cell, RefCell},
        path::PathBuf,
        rc::Rc,
    };
    struct ChildAdmission {
        child: Option<Actor>,
        published: Rc<RefCell<Option<Actor>>>,
        commits: Rc<Cell<u32>>,
    }
    impl Admission for ChildAdmission {
        fn commit(mut self: Box<Self>) {
            if let Some(child) = self.child.take() {
                *self.published.borrow_mut() = Some(child);
            }
            self.commits.set(self.commits.get() + 1);
        }
    }
    let root = PathBuf::from(".runtime");
    let parent = ActorId(500);
    let recipient = ActorId(502);
    let mut runtime = Runtime::new(&root);
    runtime.spawn(parent, base)?;
    runtime.spawn(recipient, base)?;
    let policy = Rc::new(Cell::new(1));
    let published = Rc::new(RefCell::new(None));
    let commits = Rc::new(Cell::new(0));
    let gate_policy = policy.clone();
    let gate_published = published.clone();
    let gate_commits = commits.clone();
    let child_library = base.to_owned();
    let child_root = root.clone();
    runtime.actor_mut(parent).unwrap().validator = Some(Box::new(move |nodes| {
        assert!(!nodes.is_empty());
        let child = if gate_published.borrow().is_none() {
            let mut child = Actor::new(501, &child_root)?;
            child.reload(&child_library)?;
            Some(child)
        } else {
            None
        };
        let admission = Box::new(ChildAdmission {
            child,
            published: gate_published.clone(),
            commits: gate_commits.clone(),
        });
        match gate_policy.get() {
            1 => Err("child admission rejected".into()),
            2 => panic!("intentional admission panic"),
            _ => Ok(admission as Box<dyn Admission>),
        }
    }));
    let actor = runtime.actor(parent).unwrap();
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    let state = actor.state_address();
    let slot = actor.slot_address("count");
    let notes_slot = actor.slot_address("notes");
    let text: Vec<_> = actor
        .view
        .order
        .iter()
        .map(|key| actor.view.bindings[key].text.clone())
        .collect();
    assert!(
        runtime
            .reload(parent, replacement)
            .unwrap_err()
            .contains("child admission")
    );
    assert_eq!(
        runtime.actor(parent).unwrap().implementation_version(),
        version
    );
    assert_eq!(runtime.actor(parent).unwrap().schema_version, schema);
    assert_eq!(
        runtime.actor(parent).unwrap().computed("counter.double"),
        Some(&crate::Value::Integer(0))
    );
    assert!(
        runtime
            .reload(parent, migration)
            .unwrap_err()
            .contains("child admission")
    );
    assert_eq!(runtime.actor(parent).unwrap().slot_address("bonus"), 0);
    assert_eq!(runtime.actor(parent).unwrap().schema_version, schema);
    let increment = Message {
        action: blockui_abi::key("increment"),
        value: 0,
    };
    assert!(
        runtime
            .actor_mut(parent)
            .unwrap()
            .dispatch(increment)
            .is_err()
    );
    assert!(
        runtime
            .actor_mut(parent)
            .unwrap()
            .set_text(blockui_abi::key("notes"), "rejected edit")
            .is_err()
    );
    assert_eq!(runtime.actor(parent).unwrap().integer("count"), 0);
    assert_eq!(runtime.actor(parent).unwrap().text("notes"), "");
    assert_eq!(runtime.actor(parent).unwrap().state_address(), state);
    assert_eq!(runtime.actor(parent).unwrap().slot_address("count"), slot);
    assert_eq!(
        runtime.actor(parent).unwrap().slot_address("notes"),
        notes_slot
    );
    assert_eq!(
        runtime
            .actor(parent)
            .unwrap()
            .view
            .order
            .iter()
            .map(|key| runtime.actor(parent).unwrap().view.bindings[key]
                .text
                .clone())
            .collect::<Vec<_>>(),
        text
    );
    let effects = Message {
        action: blockui_abi::key("admission_effects"),
        value: recipient.0 as i64,
    };
    assert!(
        runtime
            .actor_mut(parent)
            .unwrap()
            .dispatch(effects)
            .is_err()
    );
    assert!(runtime.actor(parent).unwrap().tasks.is_empty());
    assert!(runtime.tick().is_empty());
    assert_eq!(runtime.actor(recipient).unwrap().integer("task_result"), 0);
    assert_eq!(commits.get(), 0);
    assert!(published.borrow().is_none());
    assert!(
        !root
            .join(format!("actor-501-{}", std::process::id()))
            .exists()
    );
    policy.set(2);
    assert!(
        runtime
            .actor_mut(parent)
            .unwrap()
            .dispatch(increment)
            .unwrap_err()
            .contains("panicked")
    );
    assert_eq!(runtime.actor(parent).unwrap().integer("count"), 0);
    assert!(
        runtime
            .reload(parent, replacement)
            .unwrap_err()
            .contains("panicked")
    );
    assert_eq!(
        runtime.actor(parent).unwrap().implementation_version(),
        version
    );
    assert!(
        !root
            .join(format!("actor-501-{}", std::process::id()))
            .exists()
    );
    policy.set(0);
    runtime.reload(parent, replacement)?;
    assert_eq!(commits.get(), 1);
    assert!(published.borrow().is_some());
    let child_state = published.borrow().as_ref().unwrap().state_address();
    runtime.actor_mut(parent).unwrap().dispatch(effects)?;
    assert_eq!(runtime.actor(parent).unwrap().integer("count"), 10);
    assert_eq!(runtime.actor(parent).unwrap().slot_address("count"), slot);
    assert_eq!(runtime.actor(parent).unwrap().tasks.len(), 1);
    assert_eq!(commits.get(), 2);
    runtime.tick();
    assert_eq!(runtime.actor(recipient).unwrap().integer("task_result"), 99);
    assert_eq!(
        published.borrow().as_ref().unwrap().state_address(),
        child_state
    );
    runtime.reload(parent, migration)?;
    assert_eq!(runtime.actor(parent).unwrap().integer("bonus"), 20);
    assert_eq!(runtime.actor(parent).unwrap().slot_address("count"), slot);
    assert_eq!(
        published.borrow().as_ref().unwrap().state_address(),
        child_state
    );
    if let Some(dynamic) = dynamic {
        runtime.reload(parent, dynamic)?;
        let order = runtime.actor(parent).unwrap().view.order.clone();
        policy.set(1);
        assert!(
            runtime
                .actor_mut(parent)
                .unwrap()
                .dispatch(increment)
                .is_err()
        );
        assert_eq!(runtime.actor(parent).unwrap().integer("count"), 10);
        assert_eq!(runtime.actor(parent).unwrap().view.order, order);
        assert_eq!(
            runtime.actor(parent).unwrap().computed("counter.double"),
            Some(&crate::Value::Integer(20))
        );
        policy.set(2);
        assert!(
            runtime
                .actor_mut(parent)
                .unwrap()
                .dispatch(increment)
                .is_err()
        );
        assert_eq!(runtime.actor(parent).unwrap().view.order, order);
        assert_eq!(runtime.actor(parent).unwrap().integer("count"), 10);
        policy.set(0);
        runtime.actor_mut(parent).unwrap().dispatch(increment)?;
        assert_eq!(runtime.actor(parent).unwrap().integer("count"), 11);
        assert_ne!(runtime.actor(parent).unwrap().view.order, order);
        assert_eq!(runtime.actor(parent).unwrap().slot_address("count"), slot);
    }
    assert_eq!(runtime.actor(parent).unwrap().artifact_count(), 1);
    drop(runtime);
    drop(published.borrow_mut().take());
    // The second candidate rejects after the first has fully prepared.
    // Neither compatible replacement nor migration may partially publish.
    let mut group = Runtime::new(&root);
    let a = ActorId(700);
    let b = ActorId(701);
    group.spawn(a, base)?;
    group.spawn(b, base)?;
    group.actor_mut(a).unwrap().dispatch(increment)?;
    group.actor_mut(b).unwrap().dispatch(increment)?;
    group.actor_mut(b).unwrap().dispatch(increment)?;
    group
        .actor_mut(a)
        .unwrap()
        .set_text(blockui_abi::key("notes"), "group A")?;
    let old_versions = [
        group.actor(a).unwrap().implementation_version(),
        group.actor(b).unwrap().implementation_version(),
    ];
    let addresses = [
        group.actor(a).unwrap().slot_address("count"),
        group.actor(b).unwrap().slot_address("count"),
    ];
    let gate = Rc::new(Cell::new(true));
    let commits = Rc::new(Cell::new(0));
    struct CountCommit(Rc<Cell<u32>>);
    impl Admission for CountCommit {
        fn commit(self: Box<Self>) {
            self.0.set(self.0.get() + 1);
        }
    }
    let counter = commits.clone();
    group.actor_mut(a).unwrap().validator = Some(Box::new(move |_| {
        Ok(Box::new(CountCommit(counter.clone())))
    }));
    let reject = gate.clone();
    group.actor_mut(b).unwrap().validator = Some(Box::new(move |_| {
        if reject.get() {
            Err("second group candidate rejected".into())
        } else {
            Ok(Box::new(CountCommit(Rc::new(Cell::new(0)))))
        }
    }));
    for candidate in [replacement, migration] {
        assert!(
            group
                .reload_group(&[a, b], candidate)
                .unwrap_err()
                .contains("second group")
        );
        assert_eq!(commits.get(), 0);
        for (index, id) in [a, b].iter().enumerate() {
            let actor = group.actor(*id).unwrap();
            assert_eq!(actor.implementation_version(), old_versions[index]);
            assert_eq!(actor.slot_address("count"), addresses[index]);
            assert_eq!(actor.integer("count"), (index + 1) as i64);
            assert_eq!(actor.slot_address("bonus"), 0);
            assert_eq!(
                actor.computed("counter.double"),
                Some(&crate::Value::Integer(((index + 1) * 2) as i64))
            );
            assert_eq!(actor.artifact_count(), 1);
        }
        assert_eq!(group.actor(a).unwrap().text("notes"), "group A");
    }
    assert!(group.reload_group(&[a, a], replacement).is_err());
    assert!(group.reload_group(&[], replacement).is_err());
    gate.set(false);
    group.reload_group(&[a, b], replacement)?;
    assert_eq!(commits.get(), 1);
    for (index, id) in [a, b].iter().enumerate() {
        group.actor_mut(*id).unwrap().dispatch(increment)?;
        assert_eq!(
            group.actor(*id).unwrap().integer("count"),
            (index + 1) as i64 + 10
        );
        assert_eq!(
            group.actor(*id).unwrap().slot_address("count"),
            addresses[index]
        );
    }
    group.reload_group(&[a, b], migration)?;
    for (index, id) in [a, b].iter().enumerate() {
        assert_eq!(
            group.actor(*id).unwrap().integer("bonus"),
            ((index + 1) as i64 + 10) * 2
        );
        assert_eq!(
            group.actor(*id).unwrap().slot_address("count"),
            addresses[index]
        );
    }
    drop(group);
    println!(
        "PASS grouped compatible/migrating replacements reject the second candidate without publishing the first, preserve state slots/computed/code and commit only fully admitted groups"
    );
    println!(
        "PASS host view admission stages real child dylibs, rejects/panics before code/schema/state/UI/effects commit, releases staged children and commits successful turns"
    );
    Ok(())
}
