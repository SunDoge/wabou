//! Linux measurement of stable-host pending futures; no plugin future is retained.
use blockui_runtime::{Actor, ActorId, Message, Runtime};
use std::{
    path::Path,
    task::Poll,
    time::{Duration, Instant},
};

fn process_status() -> (u64, u64, u64) {
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    let value = |name: &str| {
        status
            .lines()
            .find_map(|line| {
                let rest = line.strip_prefix(name)?;
                rest.split_whitespace().next()?.parse::<u64>().ok()
            })
            .unwrap()
    };
    (value("Threads:"), value("VmSize:"), value("VmRSS:"))
}
fn measure_idle_actors(root: &Path) -> Result<(), String> {
    let baseline = process_status();
    let start = Instant::now();
    let mut actors = Vec::new();
    for id in 1..=64 {
        let actor = Actor::new(1000 + id, root)?;
        assert_eq!(actor.io_worker_count(), 0);
        assert!(actor.tasks.is_empty());
        actors.push(actor);
    }
    let admission_us = start.elapsed().as_micros();
    let active = process_status();
    let start = Instant::now();
    drop(actors);
    let cancel_us = start.elapsed().as_micros();
    let final_status = process_status();
    assert_eq!(active.0, baseline.0);
    assert_eq!(final_status.0, baseline.0);
    assert!(
        std::fs::read_dir(root)
            .map_err(|e| e.to_string())?
            .next()
            .is_none()
    );
    println!(
        "{{\"task_kind\":\"idle_actors\",\"actor_count\":64,\"baseline_threads\":{},\"active_threads\":{},\"final_threads\":{},\"baseline_virtual_kib\":{},\"active_virtual_kib\":{},\"baseline_rss_kib\":{},\"active_rss_kib\":{},\"admission_us\":{},\"cancel_us\":{}}}",
        baseline.0,
        active.0,
        final_status.0,
        baseline.1,
        active.1,
        baseline.2,
        active.2,
        admission_us,
        cancel_us
    );
    Ok(())
}
fn measure_io_actors(root: &Path, implementation: &Path) -> Result<(), String> {
    let mut runtime = Runtime::new(root);
    for id in 1..=64 {
        runtime.spawn(ActorId(id), implementation)?;
    }
    let baseline = process_status();
    let start = Instant::now();
    for id in 1..=64 {
        runtime.tell(
            ActorId(id),
            Message {
                action: blockui_abi::key("create_for_actor"),
                value: id as i64,
            },
        )?;
    }
    assert!(runtime.tick().is_empty());
    let admission_us = start.elapsed().as_micros();
    let active = process_status();
    let workers = runtime.actor(ActorId(1)).unwrap().io_worker_count();
    assert_eq!(workers, 2);
    for id in 1..=64 {
        assert_eq!(
            runtime.actor(ActorId(id)).unwrap().io_worker_count(),
            workers
        );
    }
    assert_eq!(active.0, baseline.0 + workers as u64);
    let deadline = Instant::now() + Duration::from_secs(5);
    while (1..=64).any(|id| runtime.actor(ActorId(id)).unwrap().integer("task_result") == 0) {
        let faults = runtime.tick();
        assert!(faults.is_empty(), "{faults:?}");
        if Instant::now() >= deadline {
            return Err("I/O actor completion timed out".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    for id in 1..=64 {
        let actor = runtime.actor(ActorId(id)).unwrap();
        assert_eq!(actor.integer("task_result"), 1);
        assert!(actor.tasks.is_empty());
        assert_eq!(actor.io_result_count(), 0);
        assert_eq!(
            std::fs::read(root.join(format!("actor-{id}.txt"))).map_err(|e| e.to_string())?,
            b"IO scaling"
        );
    }
    let start = Instant::now();
    drop(runtime);
    let cancel_us = start.elapsed().as_micros();
    let deadline = Instant::now() + Duration::from_secs(5);
    while process_status().0 != baseline.0 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    let final_status = process_status();
    assert_eq!(final_status.0, baseline.0);
    assert_eq!(
        std::fs::read_dir(root).map_err(|e| e.to_string())?.count(),
        64
    );
    println!(
        "{{\"task_kind\":\"io_actors\",\"actor_count\":64,\"io_workers\":{workers},\"baseline_threads\":{},\"active_threads\":{},\"final_threads\":{},\"baseline_virtual_kib\":{},\"active_virtual_kib\":{},\"baseline_rss_kib\":{},\"active_rss_kib\":{},\"admission_us\":{admission_us},\"cancel_us\":{cancel_us}}}",
        baseline.0, active.0, final_status.0, baseline.1, active.1, baseline.2, active.2
    );
    Ok(())
}

fn measure_timer_actors(root: &Path, implementation: &Path) -> Result<(), String> {
    let mut runtime = Runtime::new(root);
    for id in 1..=64 {
        runtime.spawn(ActorId(id), implementation)?;
    }
    let baseline = process_status();
    let start = Instant::now();
    for id in 1..=64 {
        runtime.tell(
            ActorId(id),
            Message {
                action: blockui_abi::key("start_interval"),
                value: 0,
            },
        )?;
    }
    assert!(runtime.tick().is_empty());
    let admission_us = start.elapsed().as_micros();
    let active = process_status();
    assert_eq!(active.0, baseline.0 + 1);
    let task = blockui_abi::key("counter.interval");
    let worker = runtime.actor(ActorId(1)).unwrap().tasks[&task].worker_id();
    assert!(worker.is_some());
    for id in 1..=64 {
        let actor = runtime.actor(ActorId(id)).unwrap();
        assert_eq!(actor.tasks.len(), 1);
        assert_eq!(actor.tasks[&task].worker_id(), worker);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while (1..=64).any(|id| runtime.actor(ActorId(id)).unwrap().integer("count") == 0) {
        assert!(runtime.tick().is_empty());
        if Instant::now() >= deadline {
            return Err("timer actor completion timeout".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    for id in 1..=64 {
        assert!(runtime.actor(ActorId(id)).unwrap().integer("count") >= 1);
    }
    let start = Instant::now();
    drop(runtime);
    let cancel_us = start.elapsed().as_micros();
    let final_status = process_status();
    assert_eq!(final_status.0, baseline.0);
    assert_eq!(
        std::fs::read_dir(root).map_err(|e| e.to_string())?.count(),
        0
    );
    println!(
        "{{\"task_kind\":\"timer_actors\",\"actor_count\":64,\"timer_workers\":1,\"baseline_threads\":{},\"active_threads\":{},\"final_threads\":{},\"baseline_virtual_kib\":{},\"active_virtual_kib\":{},\"baseline_rss_kib\":{},\"active_rss_kib\":{},\"admission_us\":{admission_us},\"cancel_us\":{cancel_us}}}",
        baseline.0, active.0, final_status.0, baseline.1, active.1, baseline.2, active.2
    );
    Ok(())
}

fn main() -> Result<(), String> {
    let root = std::env::args()
        .nth(1)
        .ok_or("provide owned artifact directory")?;
    if std::env::args().nth(2).as_deref() == Some("idle_actors") {
        return measure_idle_actors(Path::new(&root));
    }
    if std::env::args().nth(2).as_deref() == Some("io_actors") {
        let implementation = std::env::args()
            .nth(3)
            .ok_or("I/O scaling requires a counter implementation")?;
        return measure_io_actors(Path::new(&root), Path::new(&implementation));
    }
    if std::env::args().nth(2).as_deref() == Some("timer_actors") {
        let implementation = std::env::args()
            .nth(3)
            .ok_or("timer scaling requires counter DLL")?;
        return measure_timer_actors(Path::new(&root), Path::new(&implementation));
    }
    let timers = std::env::args().nth(2).as_deref() == Some("timers");
    let mut actor = Actor::new(991, Path::new(&root))?;
    let baseline = process_status();
    let (sender, receiver) = std::sync::mpsc::channel();
    let start = Instant::now();
    for id in 1..=64 {
        if timers {
            actor.delay(
                id,
                Duration::from_secs(86_400),
                Message {
                    action: 1,
                    value: 0,
                },
            )?;
            continue;
        }
        let mut started = Some(sender.clone());
        actor.spawn_async(
            id,
            std::future::poll_fn(move |_| {
                if let Some(sender) = started.take() {
                    let _ = sender.send(());
                }
                Poll::Pending
            }),
            Message {
                action: 1,
                value: 0,
            },
        )?;
    }
    for _ in 0..if timers { 0 } else { 64 } {
        receiver
            .recv_timeout(Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
    }
    let admission_us = start.elapsed().as_micros();
    let active = process_status();
    let start = Instant::now();
    for id in 1..=64 {
        assert!(actor.cancel_task(id));
    }
    let cancel_us = start.elapsed().as_micros();
    let final_status = process_status();
    assert!(actor.tasks.is_empty());
    assert_eq!(active.0, baseline.0 + 1);
    assert_eq!(
        final_status.0,
        baseline.0 + 1,
        "owned workers are reclaimed; shared Future worker remains available"
    );
    println!(
        "{{\"task_kind\":\"{}\",\"pending_tasks\":64,\"baseline_threads\":{},\"active_threads\":{},\"final_threads\":{},\"baseline_virtual_kib\":{},\"active_virtual_kib\":{},\"baseline_rss_kib\":{},\"active_rss_kib\":{},\"admission_us\":{},\"cancel_us\":{}}}",
        if timers { "timers" } else { "futures" },
        baseline.0,
        active.0,
        final_status.0,
        baseline.1,
        active.1,
        baseline.2,
        active.2,
        admission_us,
        cancel_us
    );
    Ok(())
}
