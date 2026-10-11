//! Stable-host text futures retain owned payloads across implementation swaps.
use blockui_abi::key;
use blockui_runtime::{Actor, AsyncMessage, AsyncReloadPolicy, Message};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    task::{Poll, Waker},
    time::{Duration, Instant},
};
fn pending(
    ready: Arc<AtomicBool>,
    payload: String,
    polled: mpsc::SyncSender<Waker>,
    owner: Arc<()>,
) -> impl std::future::Future<Output = AsyncMessage> + Send {
    let mut owner = Some(owner);
    std::future::poll_fn(move |cx| {
        let _ = &owner;
        if ready.load(Ordering::Acquire) {
            owner.take();
            Poll::Ready(AsyncMessage::text(key("text_message"), payload.clone()).unwrap())
        } else {
            let _ = polled.try_send(cx.waker().clone());
            Poll::Pending
        }
    })
}
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("async_text <base.so> <replacement.so> <rejected.so>".into());
    }
    let mut actor = Actor::new(630, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[0]))?;
    let state = actor.state_address();
    let slot = actor.slot_address("notes");
    let ready = Arc::new(AtomicBool::new(false));
    let (tx, rx) = mpsc::sync_channel(1);
    let owner = Arc::new(());
    let weak = Arc::downgrade(&owner);
    actor.spawn_async_message(
        1,
        pending(ready.clone(), "网络响应\n😀".into(), tx, owner),
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Preserve,
    )?;
    let waker = rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|e| e.to_string())?;
    let worker = actor.tasks[&1].worker_id();
    actor.reload(Path::new(&args[1]))?;
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("notes"), slot);
    assert_eq!(actor.tasks[&1].worker_id(), worker);
    // A full mailbox delays delivery without losing the owned completion.
    for _ in 0..256 {
        actor.enqueue(Message {
            action: key("increment"),
            value: 0,
        })?;
    }
    ready.store(true, Ordering::Release);
    waker.wake();
    let deadline = Instant::now() + Duration::from_secs(3);
    while actor.text("notes") != "v2:网络响应\n😀" && Instant::now() < deadline {
        actor.tick()?;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(actor.text("notes"), "v2:网络响应\n😀");
    assert!(weak.upgrade().is_none());
    assert_eq!(actor.slot_address("notes"), slot);
    assert!(!actor.tasks.contains_key(&1));
    let before = actor.text("notes").to_owned();
    actor.spawn_async_message(
        2,
        async { AsyncMessage::text(key("text_message"), "reject").unwrap() },
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Preserve,
    )?;
    let deadline = Instant::now() + Duration::from_secs(3);
    let mut rejected = false;
    while Instant::now() < deadline {
        if actor.tick().is_err() {
            rejected = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(rejected);
    assert_eq!(actor.text("notes"), before);
    assert!(!actor.tasks.contains_key(&2));
    let (tx, rx) = mpsc::sync_channel(1);
    let owner = Arc::new(());
    let weak = Arc::downgrade(&owner);
    let ready = Arc::new(AtomicBool::new(false));
    actor.spawn_async_message(
        3,
        pending(ready, "cancelled".into(), tx, owner),
        Message {
            action: key("task_done"),
            value: -1,
        },
        AsyncReloadPolicy::Cancel,
    )?;
    let _waker = rx
        .recv_timeout(Duration::from_secs(2))
        .map_err(|e| e.to_string())?;
    assert!(actor.reload(Path::new(&args[2])).is_err());
    assert!(weak.upgrade().is_some());
    assert!(actor.tasks.contains_key(&3));
    actor.reload(Path::new(&args[0]))?;
    assert!(weak.upgrade().is_none());
    assert!(!actor.tasks.contains_key(&3));
    actor.tick()?;
    assert_eq!(actor.text("notes"), before);
    assert!(AsyncMessage::text(0, "bad").is_err());
    assert!(
        AsyncMessage::text(
            key("text_message"),
            "🦀".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES / 4 + 1)
        )
        .is_err()
    );
    assert!(
        AsyncMessage::text(
            key("text_message"),
            "x".repeat(blockui_abi::MAX_MESSAGE_TEXT_BYTES)
        )
        .is_ok()
    );
    println!(
        "PASS stable-host text futures: Unicode payload executes current code after real DLL replacement, retains state/slot/worker, full mailbox delays delivery, handler failure rolls back, rejected upgrade retains Cancel task and successful reload drops it, payload limits reject invalid construction"
    );
    Ok(())
}
