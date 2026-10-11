//! Public SDK tasks use stable host sockets through real DLL replacements.
use blockui_abi::key;
use blockui_runtime::{Actor, ActorId, Message, Runtime};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::AsRawFd,
    path::Path,
    time::{Duration, Instant},
};
fn action(actor: &mut Actor, name: &str, value: i64) -> Result<(), String> {
    actor.dispatch(Message {
        action: key(name),
        value,
    })
}
fn until(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(3);
    while !ready() {
        assert!(Instant::now() < deadline, "bounded wait expired");
        std::thread::sleep(Duration::from_millis(1));
    }
}
fn complete(actor: &mut Actor, count: i64, text: &str) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while actor.integer("sdk.net.completed") < count {
        actor.tick()?;
        if Instant::now() >= deadline {
            return Err("completion deadline expired".into());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(actor.integer("sdk.net.completed"), count);
    assert_eq!(actor.text("sdk.net.result"), text);
    Ok(())
}
fn accept(listener: &TcpListener) -> TcpStream {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match listener.accept() {
            Ok((peer, _)) => {
                peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
                return peer;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(1))
            }
            Err(e) => panic!("bounded accept: {e}"),
        }
    }
}
fn main() -> Result<(), String> {
    let paths: Vec<_> = std::env::args().skip(1).collect();
    if paths.len() != 5 {
        return Err(
            "sdk_socket_reload <base> <replacement> <rejected> <evolved> <input-file>".into(),
        );
    }
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    listener.set_nonblocking(true).unwrap();
    let mut actor = Actor::new(650, Path::new(".runtime"))?;
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.network_worker_count(), 0);
    actor.set_text(
        key("sdk.net.endpoint"),
        &listener.local_addr().unwrap().to_string(),
    )?;
    let state = actor.state_address();
    let slot = actor.slot_address("sdk.net.endpoint");
    for mode in [1, 2, 4, 6] {
        assert!(action(&mut actor, "net.connect", mode).is_err());
        assert_eq!(actor.socket_count(), 0);
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    action(&mut actor, "net.connect", 0)?;
    let mut peer = accept(&listener);
    // Even a completion already produced by the host runs the current code.
    actor.reload(Path::new(&paths[1]))?;
    complete(&mut actor, 1, "v2:connected")?;
    let resource = key("sdk.net.socket");
    let fd = actor.socket(resource).unwrap().as_raw_fd();
    let task = key("sdk.net.read_task");
    action(&mut actor, "net.read", 0)?;
    until(|| actor.network_waiting_count() == 1);
    let generation = actor.tasks[&task].generation();
    let worker = actor.tasks[&task].worker_id();
    assert_eq!(actor.network_buffer_bytes(), 128);
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    actor.reload(Path::new(&paths[0]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("sdk.net.endpoint"), slot);
    assert_eq!(actor.socket(resource).unwrap().as_raw_fd(), fd);
    assert_eq!(actor.tasks[&task].generation(), generation);
    assert_eq!(actor.tasks[&task].worker_id(), worker);
    peer.write_all(b"first").unwrap();
    complete(&mut actor, 2, "v1:first")?;
    assert_eq!(actor.network_buffer_bytes(), 5);
    action(&mut actor, "net.write", 0)?;
    let mut bytes = [0; 8];
    peer.read_exact(&mut bytes).unwrap();
    assert_eq!(&bytes, b"v1:first");
    complete(&mut actor, 3, "v1:sent")?;
    // Schema evolution retains connection, slot and in-flight work too.
    action(&mut actor, "net.read", 0)?;
    until(|| actor.network_waiting_count() == 1);
    let generation = actor.tasks[&task].generation();
    actor.reload(Path::new(&paths[3]))?;
    assert_eq!(actor.slot_address("sdk.net.endpoint"), slot);
    assert_eq!(actor.socket(resource).unwrap().as_raw_fd(), fd);
    assert_eq!(actor.tasks[&task].generation(), generation);
    assert_eq!(actor.integer("sdk.net.bonus"), 0);
    peer.write_all(b"schema").unwrap();
    complete(&mut actor, 4, "v2:schema")?;
    action(&mut actor, "net.read", 3)?;
    until(|| actor.network_waiting_count() == 1);
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert!(actor.tasks.contains_key(&task));
    actor.reload(Path::new(&paths[1]))?;
    assert!(!actor.tasks.contains_key(&task));
    until(|| actor.network_waiting_count() == 0);
    assert_eq!(actor.network_buffer_bytes(), 0);
    assert_eq!(actor.socket(resource).unwrap().as_raw_fd(), fd);
    // Closing a resource does not revoke an admitted Preserve read's own handle.
    action(&mut actor, "net.read", 6)?;
    until(|| actor.network_waiting_count() == 1);
    assert_eq!(actor.socket_count(), 0);
    actor.reload(Path::new(&paths[0]))?;
    peer.write_all(b"retained").unwrap();
    complete(&mut actor, 5, "v1:retained")?;
    action(&mut actor, "net.clear", 0)?;
    until(|| actor.network_buffer_bytes() == 0);
    assert_eq!(actor.network_worker_count(), 1);
    assert_eq!(actor.io_worker_count(), 0);
    let mut eof = [0];
    assert_eq!(peer.read(&mut eof).unwrap(), 0);
    // Resource Cancel is independent from a Preserve task using that connection.
    action(&mut actor, "net.connect", 5)?;
    let mut peer = accept(&listener);
    complete(&mut actor, 6, "v1:connected")?;
    action(&mut actor, "net.read", 0)?;
    until(|| actor.network_waiting_count() == 1);
    assert!(actor.reload(Path::new(&paths[2])).is_err());
    assert_eq!(actor.socket_count(), 1);
    actor.reload(Path::new(&paths[1]))?;
    assert_eq!(actor.socket_count(), 0);
    assert!(actor.tasks.contains_key(&task));
    peer.write_all(b"resource-policy").unwrap();
    complete(&mut actor, 7, "v2:resource-policy")?;
    action(&mut actor, "net.clear", 0)?;
    until(|| actor.network_buffer_bytes() == 0);
    assert_eq!(peer.read(&mut eof).unwrap(), 0);

    // File completions follow the same mailbox/result identity rules as sockets.
    actor.set_text(key("sdk.net.endpoint"), &paths[4])?;
    let wakes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observed = wakes.clone();
    actor.set_wake(std::sync::Arc::new(move || {
        observed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }));
    let before = wakes.load(std::sync::atomic::Ordering::SeqCst);
    action(&mut actor, "file.read", 0)?;
    for _ in 0..256 {
        actor.enqueue(Message {
            action: key("net.noop"),
            value: 0,
        })?;
    }
    until(|| wakes.load(std::sync::atomic::Ordering::SeqCst) > before);
    actor.tick()?;
    assert_eq!(actor.integer("sdk.net.completed"), 7);
    actor.tick()?;
    assert_eq!(actor.integer("sdk.net.completed"), 7);
    assert!(!actor.tasks.contains_key(&task));
    actor.reload(Path::new(&paths[0]))?;
    std::fs::write(&paths[4], b"new").map_err(|error| error.to_string())?;
    action(&mut actor, "file.read", 0)?;
    complete(&mut actor, 8, "v1:new")?;
    action(&mut actor, "net.clear", 0)?;
    assert_eq!(actor.io_result_count(), 0);
    println!(
        "PASS queued file results: full mailbox backpressure, old result callback invalidation after task reuse, and new-code completion across actual DLL replacement"
    );

    // A coalescing host only delivers events while the pending flag is set.
    let pending = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let notification = pending.clone();
    let wakes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let observer = wakes.clone();
    actor.set_wake(std::sync::Arc::new(move || {
        observer.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        notification.store(true, std::sync::atomic::Ordering::SeqCst);
    }));
    let before = wakes.load(std::sync::atomic::Ordering::SeqCst);
    action(&mut actor, "file.batch", 0)?;
    until(|| wakes.load(std::sync::atomic::Ordering::SeqCst) >= before + 64);
    let state = actor.state_address();
    actor.reload(Path::new(&paths[1]))?;
    let mut turns = 0;
    while pending.swap(false, std::sync::atomic::Ordering::SeqCst) {
        turns += 1;
        assert!(turns <= 4, "coalesced event loop failed to become idle");
        actor.tick()?;
    }
    assert!(actor.tasks.is_empty());
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.io_result_bytes(), 64 * 3);
    for index in 0..64 {
        assert!(actor.cancel_task(key(&format!("file.batch.{index}"))));
    }
    assert_eq!(actor.io_result_count(), 0);
    println!(
        "PASS coalesced event loop: all 64 queued file results survive actual DLL replacement and drain through bounded turns without another producer wake or idle polling"
    );

    // Two actors in one Runtime share both executor and readiness pool/budget.
    let mut runtime = Runtime::new(".runtime");
    let ids = [ActorId(651), ActorId(652)];
    for id in ids {
        if id == ids[0] {
            runtime.spawn(id, Path::new(&paths[0]))?;
        } else {
            runtime.reconcile_children(ids[0], &[(id, Path::new(&paths[0]))])?;
            assert_eq!(runtime.parent(id), Some(ids[0]));
        }
        assert_eq!(runtime.actor(id).unwrap().network_worker_count(), 0);
        runtime.tell(
            id,
            Message {
                action: key("net.endpoint_port"),
                value: i64::from(listener.local_addr().unwrap().port()),
            },
        )?;
        runtime.tell(
            id,
            Message {
                action: key("net.connect"),
                value: 0,
            },
        )?;
    }
    until(|| {
        assert!(runtime.tick().is_empty());
        ids.iter()
            .all(|id| runtime.actor(*id).unwrap().integer("sdk.net.completed") == 1)
    });
    let mut peers = [accept(&listener), accept(&listener)];
    for id in ids {
        runtime.tell(
            id,
            Message {
                action: key("net.read"),
                value: 0,
            },
        )?;
    }
    until(|| {
        assert!(runtime.tick().is_empty());
        runtime.actor(ids[0]).unwrap().network_waiting_count() == 2
    });
    assert_eq!(
        runtime.actor(ids[0]).unwrap().tasks[&task].worker_id(),
        runtime.actor(ids[1]).unwrap().tasks[&task].worker_id()
    );
    for id in ids {
        assert_eq!(runtime.actor(id).unwrap().network_worker_count(), 1);
        assert_eq!(runtime.actor(id).unwrap().network_buffer_bytes(), 256);
    }
    let state = runtime.actor(ids[0]).unwrap().state_address();
    let child_state = runtime.actor(ids[1]).unwrap().state_address();
    assert!(
        runtime
            .reconcile_children(
                ids[0],
                &[
                    (ids[1], Path::new(&paths[0])),
                    (ActorId(653), Path::new(&paths[0])),
                    (
                        ActorId(654),
                        Path::new(".runtime/missing-child-implementation.so")
                    ),
                ]
            )
            .is_err()
    );
    assert_eq!(runtime.len(), 2);
    assert_eq!(runtime.actor(ids[0]).unwrap().state_address(), state);
    assert_eq!(runtime.actor(ids[1]).unwrap().state_address(), child_state);
    assert_eq!(runtime.actor(ids[0]).unwrap().network_waiting_count(), 2);
    assert!(runtime.actor(ActorId(653)).is_none());
    runtime.reload_group(&ids, Path::new(&paths[1]))?;
    for peer in &mut peers {
        peer.write_all(b"shared").unwrap();
    }
    until(|| {
        assert!(runtime.tick().is_empty());
        ids.iter()
            .all(|id| runtime.actor(*id).unwrap().integer("sdk.net.completed") == 2)
    });
    for id in ids {
        assert_eq!(
            runtime.actor(id).unwrap().text("sdk.net.result"),
            "v2:shared"
        );
        assert_eq!(runtime.actor(id).unwrap().network_buffer_bytes(), 12);
    }
    runtime.reconcile_children(ids[0], &[])?;
    assert!(runtime.actor(ids[1]).is_none());
    assert_eq!(runtime.actor(ids[0]).unwrap().network_buffer_bytes(), 6);
    assert_eq!(runtime.actor(ids[0]).unwrap().state_address(), state);
    // A new incarnation of the removed child uses the same shared executor too.
    runtime.reconcile_children(ids[0], &[(ids[1], Path::new(&paths[0]))])?;
    assert_eq!(
        runtime.actor(ids[1]).unwrap().integer("sdk.net.completed"),
        0
    );
    let gate = std::sync::Arc::new(std::sync::Mutex::new((false, None::<std::task::Waker>)));
    let pending = gate.clone();
    let message = Message {
        action: key("net.noop"),
        value: 0,
    };
    runtime.spawn_async(
        ids[0],
        99,
        std::future::poll_fn(move |cx| {
            let mut state = pending.lock().unwrap();
            if state.0 {
                std::task::Poll::Ready(message)
            } else {
                state.1 = Some(cx.waker().clone());
                std::task::Poll::Pending
            }
        }),
        message,
        blockui_runtime::AsyncReloadPolicy::Preserve,
    )?;
    runtime.spawn_async(
        ids[1],
        99,
        std::future::pending(),
        message,
        blockui_runtime::AsyncReloadPolicy::Preserve,
    )?;
    until(|| gate.lock().unwrap().1.is_some());
    assert_eq!(
        runtime.actor(ids[0]).unwrap().tasks[&99].worker_id(),
        runtime.actor(ids[1]).unwrap().tasks[&99].worker_id()
    );
    runtime.reconcile_children(ids[0], &[])?;
    assert!(runtime.actor(ids[0]).unwrap().tasks.contains_key(&99));
    let wake = {
        let mut state = gate.lock().unwrap();
        state.0 = true;
        state.1.take().unwrap()
    };
    wake.wake();
    until(|| {
        assert!(runtime.tick().is_empty());
        !runtime.actor(ids[0]).unwrap().tasks.contains_key(&99)
    });
    assert!(runtime.remove(ids[0]));
    for peer in &mut peers {
        assert_eq!(peer.read(&mut eof).unwrap(), 0);
    }
    println!(
        "PASS SDK socket shared Runtime pool: two actors retain one readiness worker, one future worker and shared buffer accounting across group DLL replacement, transactional child staging, child removal/recreation and independent destruction"
    );
    println!(
        "PASS SDK socket actions preserve state, connection, task generation and worker through real DLL replacement/schema migration; rejected upgrades preserve Cancel work, accepted upgrades cancel, dormant rollback performs no I/O, admitted reads survive resource close and result buffers release"
    );
    Ok(())
}
