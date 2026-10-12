use blockui_abi::key;
use blockui_runtime::{Actor, Dependency, Message, Value};
use std::{collections::BTreeMap, path::Path};
fn action(actor: &mut Actor, name: &str, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(name),
        value,
    })
}
fn bindings(actor: &Actor) -> BTreeMap<u64, (String, u64)> {
    actor
        .view
        .bindings
        .iter()
        .map(|(key, b)| (*key, (b.text.clone(), b.evaluations)))
        .collect()
}
pub fn run(paths: &[String]) -> Result<(), String> {
    if paths.len() != 13 {
        return Err("graph-proof requires thirteen graph library variants".into());
    }
    let mut actor = Actor::new(60, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.computed_count(), 6);
    assert_eq!(actor.computed_evaluations("double"), 1);
    assert_eq!(actor.computed_evaluations("caption"), 1);
    assert_eq!(
        actor.view.bindings[&key("graph.caption.one")].text,
        "Derived: 0"
    );
    assert_eq!(
        actor.view.bindings[&key("graph.caption.two")].text,
        "Derived: 0"
    );
    let initial = bindings(&actor);
    action(&mut actor, "other", 1)?;
    assert_eq!(bindings(&actor), initial);
    assert_eq!(actor.computed_evaluations("double"), 1);
    action(&mut actor, "step", 1)?;
    assert_eq!(actor.computed("double"), Some(&Value::Integer(2)));
    assert_eq!(actor.computed_evaluations("double"), 2);
    assert_eq!(actor.computed_evaluations("caption"), 2);
    let parity_text = actor.computed_evaluations("parity_text");
    let parity_binding = actor.view.bindings[&key("graph.parity")].evaluations;
    action(&mut actor, "step", 2)?;
    assert_eq!(actor.integer("count"), 3);
    assert_eq!(actor.computed("double"), Some(&Value::Integer(6)));
    assert_eq!(actor.computed_evaluations("parity_text"), parity_text);
    assert_eq!(
        actor.view.bindings[&key("graph.parity")].evaluations,
        parity_binding
    );
    action(&mut actor, "select", 1)?;
    let deps = actor.computed_dependencies("switch").unwrap();
    assert!(deps.contains(&Dependency::State(key("other"))));
    assert!(!deps.contains(&Dependency::State(key("count"))));
    let switch = actor.computed_evaluations("switch");
    action(&mut actor, "step", 1)?;
    assert_eq!(actor.integer("count"), 4);
    assert_eq!(actor.computed_evaluations("switch"), switch);
    action(&mut actor, "other", 5)?;
    assert_eq!(actor.computed("switch"), Some(&Value::Integer(106)));
    assert_eq!(actor.computed_evaluations("switch"), switch + 1);
    actor.set_text(key("notes"), "unsaved graph state")?;
    let storage = actor.state_address();
    let slot = actor.slot_address("count");
    let note_slot = actor.slot_address("notes");
    let schema = actor.schema_version;
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.computed("double"), Some(&Value::Integer(12)));
    assert_eq!(actor.computed_evaluations("double"), 1);
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("count"), slot);
    assert_eq!(actor.slot_address("notes"), note_slot);
    assert_eq!(actor.schema_version, schema);
    assert_eq!(
        actor.view.bindings[&key("graph.caption.one")].text,
        "Derived: 12"
    );
    for index in [3, 4, 10, 11] {
        let version = actor.implementation_version();
        let before = bindings(&actor);
        let evals = actor.computed_evaluations("double");
        let error = actor.reload(Path::new(&paths[index])).unwrap_err();
        let expected = match index {
            3 => "cyclic",
            10 => "invalid computed registry",
            4 => "intentional computed failure",
            _ => "runtime operation rejected",
        };
        assert!(error.contains(expected), "{error}");
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(bindings(&actor), before);
        assert_eq!(actor.computed("double"), Some(&Value::Integer(12)));
        assert_eq!(actor.computed_evaluations("double"), evals);
        assert_eq!(actor.integer("count"), 4);
        assert_eq!(actor.artifact_count(), 1);
        action(&mut actor, "other", 1)?;
    }
    actor.reload(Path::new(&paths[2]))?;
    let other = actor.integer("other");
    assert_eq!(actor.computed("double"), Some(&Value::Integer(12 + other)));
    assert!(
        actor
            .computed_dependencies("double")
            .unwrap()
            .contains(&Dependency::State(key("other")))
    );
    action(&mut actor, "other", 1)?;
    assert_eq!(actor.computed("double"), Some(&Value::Integer(13 + other)));
    assert_eq!(actor.computed_evaluations("double"), 2);
    actor.reload(Path::new(&paths[8]))?;
    assert_eq!(actor.computed_count(), 5);
    assert!(actor.computed("unused").is_none());
    assert_eq!(actor.slot_address("count"), slot);
    actor.reload(Path::new(&paths[9]))?;
    assert_eq!(actor.computed("double"), Some(&Value::Text("v12".into())));
    assert_eq!(actor.schema_version, schema);
    assert_eq!(actor.slot_address("count"), slot);
    for index in [5, 6] {
        actor.reload(Path::new(&paths[index]))?;
        let before = bindings(&actor);
        let evals = actor.computed_evaluations("double");
        let failure = action(&mut actor, "step", 1).unwrap_err();
        if index == 6 {
            assert!(failure.contains("actor callback panicked:"), "{failure}");
        }
        assert!(actor.tasks.is_empty());
        assert_eq!(actor.integer("count"), 4);
        assert_eq!(actor.text("notes"), "unsaved graph state");
        assert_eq!(bindings(&actor), before);
        assert_eq!(actor.computed("double"), Some(&Value::Integer(8)));
        assert_eq!(actor.computed_evaluations("double"), evals);
        assert_eq!(actor.slot_address("notes"), note_slot);
        action(&mut actor, "other", 1)?;
    }
    actor.reload(Path::new(&paths[7]))?;
    let before = bindings(&actor);
    let evals = actor.computed_evaluations("double");
    assert!(
        action(&mut actor, "step", 1)
            .unwrap_err()
            .contains("intentional late render failure")
    );
    assert!(actor.tasks.is_empty());
    assert_eq!(actor.integer("count"), 4);
    assert_eq!(bindings(&actor), before);
    assert_eq!(actor.computed("double"), Some(&Value::Integer(8)));
    assert_eq!(actor.computed_evaluations("double"), evals);
    assert!(actor.set_text(key("notes"), "reject").is_err());
    assert_eq!(actor.text("notes"), "unsaved graph state");
    assert_eq!(bindings(&actor), before);
    assert_eq!(actor.slot_address("notes"), note_slot);
    action(&mut actor, "step", 2)?;
    assert_eq!(actor.integer("count"), 6);
    assert_eq!(actor.computed("double"), Some(&Value::Integer(12)));
    assert_eq!(actor.view.bindings[&1].text, "Count: 6");
    assert_eq!(actor.state_address(), storage);
    assert_eq!(actor.slot_address("count"), slot);
    assert_eq!(actor.artifact_count(), 1);
    actor.reload(Path::new(&paths[12]))?;
    assert!(!actor.view.bindings.contains_key(&key("graph.odd")));
    action(&mut actor, "step", 1)?;
    assert!(actor.view.bindings.contains_key(&key("graph.odd")));
    let order = actor.view.order.clone();
    let root_evaluations = actor.view.bindings[&key("graph.root")].evaluations;
    action(&mut actor, "step", 2)?;
    assert_eq!(actor.view.order, order);
    assert_eq!(
        actor.view.bindings[&key("graph.root")].evaluations,
        root_evaluations
    );
    assert_eq!(actor.computed("double"), Some(&Value::Integer(18)));
    for _ in 0..40 {
        actor.reload(Path::new(&paths[1]))?;
        assert_eq!(actor.computed_count(), 6);
        assert_eq!(actor.artifact_count(), 1);
        assert_eq!(actor.slot_address("count"), slot);
    }
    let mut precision = Actor::new(68, Path::new(".runtime"))?;
    precision.reload(Path::new(&paths[0]))?;
    for mode in [1, 2] {
        assert!(action(&mut precision, "timer_precision", mode).is_err());
        assert!(precision.tasks.is_empty());
    }
    action(&mut precision, "timer_precision", 0)?;
    precision.reload(Path::new(&paths[1]))?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while precision.integer("count") == 0 && std::time::Instant::now() < deadline {
        precision.tick()?;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert_eq!(precision.integer("count"), 10);
    assert!(precision.tasks.is_empty());
    drop(precision);
    let mut timer_actor = Actor::new(61, Path::new(".runtime"))?;
    timer_actor.reload(Path::new(&paths[0]))?;
    let timer_slot = timer_actor.slot_address("count");
    assert!(action(&mut timer_actor, "reject_start", 0).is_err());
    assert!(timer_actor.tasks.is_empty());
    assert!(action(&mut timer_actor, "start", -1).is_err());
    action(&mut timer_actor, "start", 100)?;
    assert_eq!(timer_actor.tasks.len(), 1);
    assert!(action(&mut timer_actor, "start", 100).is_err());
    assert!(action(&mut timer_actor, "reject_cancel", 0).is_err());
    assert_eq!(timer_actor.tasks.len(), 1);
    let task_address = (&timer_actor.tasks[&key("graph.timer")] as *const _) as usize;
    timer_actor.reload(Path::new(&paths[1]))?;
    assert_eq!(
        (&timer_actor.tasks[&key("graph.timer")] as *const _) as usize,
        task_address
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while timer_actor.integer("count") == 0 && std::time::Instant::now() < deadline {
        timer_actor.tick()?;
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(timer_actor.integer("count"), 10); // completion uses v2, not v1 callback
    assert_eq!(timer_actor.slot_address("count"), timer_slot);
    timer_actor.tick()?;
    assert!(timer_actor.tasks.is_empty());
    // An already-sent timer result is not accepted if its task was canceled/reused.
    action(&mut timer_actor, "start", 0)?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    // Synchronize with actual worker completion, without consuming its channel.
    while !timer_actor.tasks[&key("graph.timer")].is_finished()
        && std::time::Instant::now() < deadline
    {
        std::thread::yield_now();
    }
    assert!(timer_actor.tasks[&key("graph.timer")].is_finished());
    action(&mut timer_actor, "cancel", 0)?;
    action(&mut timer_actor, "start", 60_000)?;
    timer_actor.tick()?;
    assert_eq!(timer_actor.integer("count"), 10);
    assert_eq!(timer_actor.tasks.len(), 1);
    action(&mut timer_actor, "cancel", 0)?;
    action(&mut timer_actor, "start", 60_000)?;
    action(&mut timer_actor, "cancel", 0)?;
    assert!(timer_actor.tasks.is_empty());
    timer_actor.tick()?;
    assert_eq!(timer_actor.integer("count"), 10);
    action(&mut timer_actor, "start", 60_000)?;
    action(&mut timer_actor, "policies", 0)?;
    let preserve_worker = timer_actor.tasks[&key("graph.timer")].worker_id();
    let restart_worker = timer_actor.tasks[&key("graph.restart_timer")].generation();
    let cancel_worker = timer_actor.tasks[&key("graph.cancel_timer")].worker_id();
    assert!(timer_actor.reload(Path::new(&paths[3])).is_err());
    assert_eq!(
        timer_actor.tasks[&key("graph.restart_timer")].generation(),
        restart_worker
    );
    assert_eq!(
        timer_actor.tasks[&key("graph.cancel_timer")].worker_id(),
        cancel_worker
    );
    timer_actor.reload(Path::new(&paths[0]))?;
    assert_eq!(
        timer_actor.tasks[&key("graph.timer")].worker_id(),
        preserve_worker
    );
    assert!(!timer_actor.tasks.contains_key(&key("graph.cancel_timer")));
    assert_ne!(
        timer_actor.tasks[&key("graph.restart_timer")].generation(),
        restart_worker
    );
    drop(timer_actor); // cancellation unparks and joins, rather than waiting for the deadline
    // All three workers have sent results before reload; policies apply before
    // acceptance, rather than treating worker exit as a committed actor message.
    let mut ready = Actor::new(65, Path::new(".runtime"))?;
    ready.reload(Path::new(&paths[0]))?;
    action(&mut ready, "start", 0)?;
    action(&mut ready, "ready_policies", 0)?;
    let wait_ready = |actor: &Actor| {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while !actor.tasks.values().all(|task| task.is_finished())
            && std::time::Instant::now() < deadline
        {
            std::thread::yield_now();
        }
        assert!(actor.tasks.values().all(|task| task.is_finished()));
    };
    wait_ready(&ready);
    let retained = ready.tasks[&key("graph.timer")].worker_id();
    let restart = ready.tasks[&key("graph.restart_timer")].generation();
    assert!(ready.reload(Path::new(&paths[3])).is_err());
    assert_eq!(ready.tasks.len(), 3);
    assert_eq!(
        ready.tasks[&key("graph.restart_timer")].generation(),
        restart
    );
    ready.reload(Path::new(&paths[1]))?;
    assert_eq!(ready.tasks[&key("graph.timer")].worker_id(), retained);
    assert!(!ready.tasks.contains_key(&key("graph.cancel_timer")));
    assert_ne!(
        ready.tasks[&key("graph.restart_timer")].generation(),
        restart
    );
    wait_ready(&ready);
    ready.tick()?;
    assert_eq!(ready.integer("count"), 1010); // preserved 1 + restarted 100, new code x10
    assert!(ready.tasks.is_empty());
    // A full mailbox postpones acceptance; worker exit must not erase the task.
    action(&mut ready, "start", 0)?;
    wait_ready(&ready);
    for _ in 0..256 {
        ready.enqueue(Message {
            action: key("finish"),
            value: 0,
        })?;
    }
    ready.tick()?;
    assert!(ready.tasks.contains_key(&key("graph.timer")));
    for _ in 0..9 {
        ready.tick()?;
    }
    assert_eq!(ready.integer("count"), 1020);
    assert!(ready.tasks.is_empty());
    drop(ready);
    let mut bounded = Actor::new(62, Path::new(".runtime"))?;
    bounded.reload(Path::new(&paths[0]))?;
    assert!(action(&mut bounded, "fill", 65).is_err());
    assert!(bounded.tasks.is_empty()); // all requests discarded before workers spawn
    action(&mut bounded, "fill", 64)?;
    assert_eq!(bounded.tasks.len(), 64);
    assert!(action(&mut bounded, "start", 0).is_err());
    assert_eq!(bounded.tasks.len(), 64);
    drop(bounded);
    println!(
        "PASS typed SDK task preserves identity, completes through new implementation, rejects queued stale timer generation after cancel/reuse, applies policies to ready results and retains full-mailbox completions, rejects duplicate/invalid requests, commits only successful turns, rolls back cancellation, applies preserve/cancel/restart policies only on successful reload and cancels on shutdown"
    );
    println!(
        "PASS shared computed DAG, equality pruning, conditional dependencies, code/schema separation, changed computation and dependencies after reload, computed removal/type changes, cycle/registry/error rejection, read-only computed enforcement, structural equality pruning, handler/panic/late-render/text rollback, recovery and bounded repeated replacement"
    );
    Ok(())
}
