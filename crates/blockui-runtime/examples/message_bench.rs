//! Warm end-to-end message cost, including the actor's reactive view work.
use blockui_runtime::{Actor, ActorId, Message, Runtime};
use std::{path::Path, time::Instant};
fn measure(mut run: impl FnMut() -> Result<(), String>) -> Result<(f64, f64, f64), String> {
    for _ in 0..100 {
        run()?;
    }
    let mut samples = Vec::new();
    for _ in 0..7 {
        let start = Instant::now();
        for _ in 0..1000 {
            run()?;
        }
        samples.push(start.elapsed().as_secs_f64() * 1000.0); // batch ms == mean message us
    }
    samples.sort_by(f64::total_cmp);
    Ok((samples[0], samples[3], samples[6]))
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    let path = Path::new(args.get(1).ok_or("message_bench <base counter.so>")?);
    let message = Message {
        action: blockui_abi::key("increment"),
        value: 0,
    };
    let mut actor = Actor::new(43, Path::new(".runtime"))?;
    actor.reload(path)?;
    let initial = actor.integer("count");
    let direct = measure(|| actor.dispatch(message))?;
    assert_eq!(actor.integer("count"), initial + 7100);
    let queued = measure(|| {
        actor.enqueue(message)?;
        actor.tick()
    })?;
    assert_eq!(actor.integer("count"), initial + 14200);
    let mut runtime = Runtime::new(".runtime");
    runtime.spawn(ActorId(44), path)?;
    let initial = runtime.actor(ActorId(44)).unwrap().integer("count");
    let routed = measure(|| {
        runtime.tell(ActorId(44), message)?;
        let errors = runtime.tick();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("{errors:?}"))
        }
    })?;
    assert_eq!(
        runtime.actor(ActorId(44)).unwrap().integer("count"),
        initial + 7100
    );
    let mut requests = Runtime::new(".runtime");
    let source = ActorId(45);
    let target = ActorId(46);
    requests.spawn(source, path)?;
    requests.spawn(target, path)?;
    let tick = |runtime: &mut Runtime| -> Result<(), String> {
        let errors = runtime.tick();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("{errors:?}"))
        }
    };
    let ask = measure(|| {
        requests.tell(
            source,
            Message {
                action: blockui_abi::key("ask"),
                value: target.0 as i64,
            },
        )?;
        for _ in 0..3 {
            tick(&mut requests)?;
        }
        Ok(())
    })?;
    assert_eq!(requests.actor(target).unwrap().integer("count"), 7100);
    assert_eq!(requests.actor(source).unwrap().integer("task_result"), 7100);
    let cancel = measure(|| {
        requests.tell(
            source,
            Message {
                action: blockui_abi::key("ask_cancel"),
                value: target.0 as i64,
            },
        )?;
        for _ in 0..2 {
            tick(&mut requests)?;
        }
        Ok(())
    })?;
    assert_eq!(requests.actor(target).unwrap().integer("count"), 7100);
    assert_eq!(requests.actor(source).unwrap().integer("task_result"), -4);
    println!(
        "{{\"batch_messages\":1000,\"batches\":7,\"dispatch_us\":{:?},\"mailbox_us\":{:?},\"runtime_us\":{:?},\"sdk_ask_us\":{:?},\"sdk_cancel_us\":{:?}}}",
        [direct.0, direct.1, direct.2],
        [queued.0, queued.1, queued.2],
        [routed.0, routed.1, routed.2],
        [ask.0, ask.1, ask.2],
        [cancel.0, cancel.1, cancel.2]
    );
    Ok(())
}
