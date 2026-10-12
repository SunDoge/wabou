//! Real-library proof for nested declaration tokens; every value belongs to the host.
use blockui_runtime::{Actor, Message, StateChange, Value};
use std::path::Path;
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 8 {
        return Err(
            "record_proof base replacement extra reordered duplicate scoped scoped-v2".into(),
        );
    }
    // A host future remains pending across a real implementation replacement.
    // Its owned completion is interpreted by the replacement, never old code.
    {
        let mut actor = Actor::new(50, Path::new(".runtime"))?;
        actor.reload(Path::new(&args[1]))?;
        let slot = actor.slot_address("editor.document.count");
        let (tx, rx) = std::sync::mpsc::sync_channel::<std::task::Waker>(1);
        let ready = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let gate = ready.clone();
        actor.spawn_async(
            123,
            async move {
                std::future::poll_fn(move |cx| {
                    if gate.load(std::sync::atomic::Ordering::Acquire) {
                        std::task::Poll::Ready(())
                    } else {
                        let _ = tx.try_send(cx.waker().clone());
                        std::task::Poll::Pending
                    }
                })
                .await;
                Message {
                    action: blockui_abi::key("update"),
                    value: 0,
                }
            },
            Message {
                action: blockui_abi::key("update"),
                value: 0,
            },
        )?;
        let waker = rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        actor.reload(Path::new(&args[2]))?;
        assert_eq!(actor.slot_address("editor.document.count"), slot);
        assert_eq!(actor.integer("editor.document.count"), 2);
        ready.store(true, std::sync::atomic::Ordering::Release);
        waker.wake();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while actor.integer("editor.document.count") == 2 {
            assert!(
                std::time::Instant::now() < deadline,
                "async completion timed out"
            );
            actor.tick()?;
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert_eq!(actor.integer("editor.document.count"), 12);
        actor.tick()?;
        assert_eq!(actor.integer("editor.document.count"), 12);
    }
    // A completion queued before cancellation must not target a reused ID.
    {
        let mut actor = Actor::new(51, Path::new(".runtime"))?;
        actor.reload(Path::new(&args[1]))?;
        let message = Message {
            action: blockui_abi::key("update"),
            value: 0,
        };
        actor.spawn_async(321, async move { message }, message)?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !actor.tasks[&321].is_finished() {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        // Worker exit proves its completion was buffered, not merely scheduled.
        assert!(actor.cancel_task(321));
        assert!(!actor.cancel_task(321));
        let (dropped_tx, dropped_rx) = std::sync::mpsc::channel();
        struct DropGuard(std::sync::mpsc::Sender<()>);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                self.0.send(()).unwrap();
            }
        }
        let guard = DropGuard(dropped_tx);
        actor.spawn_async(
            321,
            async move {
                let _guard = guard;
                std::future::pending::<()>().await;
                message
            },
            message,
        )?;
        actor.reload(Path::new(&args[2]))?;
        actor.tick()?;
        assert_eq!(actor.integer("editor.document.count"), 2);
        assert!(actor.tasks.contains_key(&321));
        assert!(actor.cancel_task(321));
        dropped_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        actor.tick()?;
        assert_eq!(actor.integer("editor.document.count"), 2);
    }
    // Completed host futures remain queued while a full actor mailbox drains.
    {
        let mut actor = Actor::new(52, Path::new(".runtime"))?;
        actor.reload(Path::new(&args[1]))?;
        let message = Message {
            action: blockui_abi::key("update"),
            value: 0,
        };
        for _ in 0..256 {
            actor.enqueue(message)?;
        }
        assert!(actor.enqueue(message).is_err());
        for id in 1..=64 {
            actor.spawn_async(id, async move { message }, message)?;
        }
        assert!(
            actor
                .spawn_async(65, async move { message }, message)
                .is_err()
        );
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        while actor.tasks.values().any(|task| !task.is_finished()) {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        actor.reload(Path::new(&args[2]))?;
        for _ in 0..12 {
            actor.tick()?;
        }
        assert!(actor.tasks.is_empty());
        // All 256 queued commands and 64 completions used v2, exactly once.
        assert_eq!(actor.integer("editor.document.count"), 2 + 320 * 10);
        actor.tick()?;
        assert_eq!(actor.integer("editor.document.count"), 3202);
    }
    // Failed candidate validation must not apply cancel-on-reload policy.
    {
        let mut actor = Actor::new(53, Path::new(".runtime"))?;
        actor.reload(Path::new(&args[1]))?;
        let message = Message {
            action: blockui_abi::key("update"),
            value: 0,
        };
        let (tx, rx) = std::sync::mpsc::channel();
        struct Guard(std::sync::mpsc::Sender<()>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.send(()).unwrap();
            }
        }
        let guard = Guard(tx);
        actor.spawn_async_with_policy(
            1,
            async move {
                let _guard = guard;
                std::future::pending::<()>().await;
                message
            },
            message,
            blockui_runtime::AsyncReloadPolicy::Cancel,
        )?;
        let slot = actor.slot_address("editor.document.count");
        assert!(actor.reload(Path::new(&args[5])).is_err());
        assert!(actor.tasks.contains_key(&1));
        assert!(matches!(
            rx.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Empty)
        ));
        assert_eq!(actor.slot_address("editor.document.count"), slot);
        actor.reload(Path::new(&args[2]))?;
        assert!(!actor.tasks.contains_key(&1));
        rx.recv_timeout(std::time::Duration::from_secs(2)).unwrap();
        actor.tick()?;
        assert_eq!(actor.integer("editor.document.count"), 2);
        assert_eq!(actor.slot_address("editor.document.count"), slot);
    }
    {
        let mut recovered = Actor::new(51, Path::new(".runtime"))?;
        recovered.reload(Path::new(&args[1]))?;
        recovered.set_text(blockui_abi::key("editor.document.title"), "Unicode 中文 🦀")?;
        let title = recovered.slot_address("editor.document.title");
        let state = recovered.state_address();
        for (path, expected_count) in [(&args[1], 3), (&args[2], 13)] {
            recovered.reload(Path::new(path))?;
            recovered.dispatch(Message {
                action: blockui_abi::key("update"),
                value: -4,
            })?;
            assert_eq!(recovered.integer("editor.document.count"), expected_count);
            assert_eq!(recovered.text("editor.document.title"), "Unicode 中文 🦀");
            assert_eq!(recovered.slot_address("editor.document.title"), title);
            assert_eq!(recovered.state_address(), state);
            assert_eq!(
                recovered.view.bindings[&blockui_abi::key("editor.title")].text,
                "Unicode 中文 🦀"
            );
        }
        println!(
            "PASS handled string-update error retains Unicode text while committing other writes through replacement"
        );
    }
    let mut actor = Actor::new(49, Path::new(".runtime"))?;
    actor.reload(Path::new(&args[1]))?;
    assert_eq!(actor.integer("editor.selection"), 1);
    assert_eq!(actor.integer("editor.document.count"), 2);
    assert_eq!(actor.text("editor.document.title"), "draft");
    let items_key = blockui_abi::key("editor.items");
    assert_eq!(actor.view.bindings[&items_key].text, "[7, 9]");
    let items_slot = actor.slot_address("editor.document.items");
    actor.dispatch(Message {
        action: blockui_abi::key("items"),
        value: 4,
    })?;
    assert_eq!(actor.view.bindings[&items_key].text, "[7, 9, 4]");
    assert!(
        actor
            .dispatch(Message {
                action: blockui_abi::key("items"),
                value: 7
            })
            .is_err()
    );
    assert!(
        actor
            .dispatch(Message {
                action: blockui_abi::key("items"),
                value: -2
            })
            .is_err()
    );
    assert_eq!(actor.integer("editor.document.count"), 2);
    assert_eq!(actor.view.bindings[&items_key].text, "[7, 9, 4]");
    let state = actor.state_address();
    let count = actor.slot_address("editor.document.count");
    let title = actor.slot_address("editor.document.title");
    let selection = actor.slot_address("editor.selection");
    assert!(
        actor
            .dispatch(Message {
                action: blockui_abi::key("update"),
                value: -3
            })
            .is_err()
    );
    assert_eq!(actor.integer("editor.document.count"), 2);
    assert_eq!(actor.text("editor.document.title"), "draft");
    assert_eq!(actor.slot_address("editor.document.count"), count);
    assert_eq!(actor.slot_address("editor.document.title"), title);
    println!(
        "PASS failing string update publishes no temporary mutation and rolls back earlier record writes"
    );
    let original_schema = actor.schema_version;
    let message = Message {
        action: blockui_abi::key("update"),
        value: 0,
    };
    actor.dispatch(message)?;
    assert_eq!(actor.integer("editor.document.count"), 3);
    assert_eq!(actor.text("editor.document.title"), "draft!");
    actor.set_text(blockui_abi::key("editor.document.title"), "unsaved 中文")?;
    actor.enqueue(message)?;
    actor.enqueue(Message {
        action: blockui_abi::key("items"),
        value: 5,
    })?;
    let list_buffer = actor.list("editor.document.items").as_ptr();
    actor.reload(Path::new(&args[2]))?;
    assert_eq!(actor.list("editor.document.items").as_ptr(), list_buffer);
    assert_eq!(actor.slot_address("editor.document.items"), items_slot);
    assert_eq!(actor.view.bindings[&items_key].text, "[7, 9, 4]");
    assert_eq!(actor.schema_version, original_schema);
    actor.tick()?;
    assert_eq!(actor.integer("editor.document.count"), 13);
    assert_eq!(actor.text("editor.document.title"), "unsaved 中文?");
    let preview = actor.preview_reload(Path::new(&args[3]))?;
    assert!(preview.changes.iter().any(|change| matches!(change, StateChange::Added { key, default:Value::Boolean(true) } if *key==blockui_abi::key("editor.document.published"))));
    assert_eq!(actor.view.bindings[&items_key].text, "[7, 9, 4, 105]");
    actor.reload(Path::new(&args[3]))?;
    assert!(actor.boolean("editor.document.published"));
    actor.reload(Path::new(&args[4]))?;
    assert_eq!(actor.schema_version, original_schema);
    assert_eq!(actor.state_address(), state);
    assert_eq!(actor.slot_address("editor.document.count"), count);
    assert_eq!(actor.slot_address("editor.document.title"), title);
    assert_eq!(actor.slot_address("editor.selection"), selection);
    assert_eq!(actor.integer("editor.document.count"), 13);
    assert_eq!(actor.text("editor.document.title"), "unsaved 中文?");
    assert!(
        actor
            .reload(Path::new(&args[5]))
            .unwrap_err()
            .contains("duplicate/zero state key")
    );
    assert_eq!(actor.slot_address("editor.document.title"), title);
    assert_eq!(actor.text("editor.document.title"), "unsaved 中文?");
    actor.reload(Path::new(&args[6]))?;
    let second_count = blockui_abi::key("editor.second-count");
    let second_title = blockui_abi::key("editor.second-title");
    assert_eq!(actor.view.bindings[&second_count].text, "2");
    assert_eq!(actor.view.bindings[&second_title].text, "draft");
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("editor.second-published")].text,
        "true"
    );
    let scoped_title = actor.view.bindings[&second_title].node.slot;
    assert_ne!(scoped_title, blockui_abi::key("editor.document.title"));
    actor.set_text(scoped_title, "second unsaved")?;
    actor.enqueue(Message {
        action: blockui_abi::key("update_second"),
        value: 0,
    })?;
    actor.reload(Path::new(&args[7]))?;
    actor.tick()?;
    assert_eq!(actor.view.bindings[&second_title].node.slot, scoped_title);
    assert_eq!(actor.view.bindings[&second_count].text, "12");
    assert_eq!(
        actor.view.bindings[&blockui_abi::key("editor.second-published")].text,
        "false"
    );
    assert!(actor.boolean("editor.document.published"));
    assert_eq!(actor.view.bindings[&second_title].text, "second unsaved?");
    assert_eq!(actor.integer("editor.document.count"), 13);
    assert_eq!(actor.text("editor.document.title"), "unsaved 中文?");
    assert_eq!(actor.slot_address("editor.document.title"), title);
    actor.dispatch(Message {
        action: blockui_abi::key("items"),
        value: 0,
    })?;
    assert_eq!(actor.view.bindings[&items_key].text, "[]");
    assert_eq!(actor.slot_address("editor.document.items"), items_slot);
    println!(
        "PASS nested record schemas retain host slot identity, edited document, selection and queued message across code changes, added defaults and reordered record fields; scoped copies isolate integer/bool/text; list buffers survive compatible reload, dynamic keyed rows update and rejected edits roll back"
    );
    Ok(())
}
