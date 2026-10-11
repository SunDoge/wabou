//! Real SDK periodic task admission, current-code delivery and reload policies.
use blockui_abi::key;
use blockui_runtime::{Actor, Message};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn start(actor: &mut Actor, mode: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key("start_interval"),
        value: mode,
    })
}
fn next(actor: &mut Actor) -> Result<i64, String> {
    let old = actor.integer("count");
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        actor.tick()?;
        let new = actor.integer("count");
        if new != old {
            return Ok(new - old);
        }
        if Instant::now() >= deadline {
            return Err("periodic callback did not arrive".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn stop(actor: &mut Actor) -> Result<(), String> {
    actor.dispatch(Message {
        action: key("stop_interval"),
        value: 0,
    })
}
fn check_pending_interval(
    actor: &mut Actor,
    candidate: &Path,
    accepted: bool,
    step: i64,
) -> Result<(), String> {
    let id = key("counter.interval");
    let state = actor.state_address();
    let slot = actor.slot_address("count");
    let version = actor.implementation_version();
    let schema = actor.schema_version;
    let generation = actor.tasks[&id].generation();
    let worker = actor.tasks[&id].worker_id();
    // A completion is pending while preparation allocates/discards any restart
    // candidate. It must remain owned by the original live interval generation.
    std::thread::sleep(Duration::from_millis(40));
    let count = actor.integer("count");
    let mailbox = actor.mailbox.len();
    for _ in 0..8 {
        let result = actor.check_reload(candidate);
        assert_eq!(result.is_ok(), accepted, "{result:?}");
        assert_eq!(actor.integer("count"), count);
        assert_eq!(actor.state_address(), state);
        assert_eq!(actor.slot_address("count"), slot);
        assert_eq!(actor.implementation_version(), version);
        assert_eq!(actor.schema_version, schema);
        assert_eq!(actor.mailbox.len(), mailbox);
        assert_eq!(actor.tasks.len(), 1);
        assert_eq!(actor.tasks[&id].generation(), generation);
        assert_eq!(actor.tasks[&id].worker_id(), worker);
        assert_eq!(actor.artifact_count(), 1);
        assert_eq!(actor.mapped_implementation_count()?, 1);
    }
    assert_eq!(next(actor)?, step);
    // The original task must still rearm after delivery; discarded dormant
    // restart handles must neither cancel it nor publish extra completions.
    assert_eq!(next(actor)?, step);
    Ok(())
}
pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() != 3 && (args.len() != 4 || args[3] != "--window") {
        return Err("interval-proof base replacement rejected [--window]".into());
    }
    #[cfg(not(feature = "native-window"))]
    if args.len() == 4 {
        return Err("--window requires the native feature".into());
    }
    let mut actor = Actor::new(746, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[0]))?;
    #[cfg(feature = "native-window")]
    if args.get(3).map(String::as_str) == Some("--window") {
        return blockui_runtime::native::interval_window_proof(
            actor,
            Path::new(&args[1]),
            Path::new(&args[2]),
        );
    }
    let id = key("counter.interval");
    assert!(start(&mut actor, 1).is_err());
    assert!(actor.tasks.is_empty());
    start(&mut actor, 0)?;
    assert!(start(&mut actor, 0).is_err());
    let generation = actor.tasks[&id].generation();
    let worker = actor.tasks[&id].worker_id();
    let state = actor.state_address();
    let slot = actor.slot_address("count");
    assert_eq!(next(&mut actor)?, 1);
    check_pending_interval(&mut actor, Path::new(&args[1]), true, 1)?;
    check_pending_interval(&mut actor, Path::new(&args[2]), false, 1)?;
    actor.reload(Path::new(&args[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("count"), slot);
    assert_eq!(actor.tasks[&id].generation(), generation);
    assert_eq!(actor.tasks[&id].worker_id(), worker);
    assert_eq!(next(&mut actor)?, 10);
    assert!(actor.reload(Path::new(&args[2])).is_err());
    assert_eq!(actor.tasks[&id].generation(), generation);
    assert_eq!(next(&mut actor)?, 10);
    stop(&mut actor)?;
    assert!(actor.tasks.is_empty());
    let old = actor.integer("count");
    std::thread::sleep(Duration::from_millis(40));
    actor.tick()?;
    assert_eq!(actor.integer("count"), old);
    start(&mut actor, 2)?;
    check_pending_interval(&mut actor, Path::new(&args[0]), true, 10)?;
    check_pending_interval(&mut actor, Path::new(&args[2]), false, 10)?;
    let before_restart = actor.tasks[&id].generation();
    actor.reload(Path::new(&args[0]))?;
    assert_ne!(actor.tasks[&id].generation(), before_restart);
    assert_eq!(actor.tasks[&id].worker_id(), worker);
    assert_eq!(next(&mut actor)?, 1);
    stop(&mut actor)?;
    start(&mut actor, 3)?;
    check_pending_interval(&mut actor, Path::new(&args[1]), true, 1)?;
    check_pending_interval(&mut actor, Path::new(&args[2]), false, 1)?;
    actor.reload(Path::new(&args[1]))?;
    assert!(actor.tasks.is_empty());
    let old = actor.integer("count");
    std::thread::sleep(Duration::from_millis(40));
    actor.tick()?;
    assert_eq!(actor.integer("count"), old);
    start(&mut actor, 4)?;
    let failure_generation = actor.tasks[&id].generation();
    for _ in 0..2 {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if actor.tick().is_err() {
                break;
            }
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(actor.integer("count"), old);
        assert_eq!(actor.tasks[&id].generation(), failure_generation);
    }
    stop(&mut actor)?;
    start(&mut actor, 0)?;
    assert_eq!(next(&mut actor)?, 10);
    stop(&mut actor)?;
    assert_eq!(actor.artifact_count(), 1);
    println!(
        "PASS SDK interval admission rollback, duplicate rejection, state/slot/task/worker identity through real reload, new-code periodic delivery, rejected reload continuation, Preserve/Restart/Cancel, pending completion and task generation retained through repeated executable checks, and callback fault recovery"
    );
    Ok(())
}
