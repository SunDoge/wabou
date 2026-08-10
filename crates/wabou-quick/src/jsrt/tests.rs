use super::*;

#[derive(Default)]
struct TestClock(std::sync::atomic::AtomicU64);

impl TestClock {
    fn advance_ms(&self, milliseconds: u64) {
        self.0.fetch_add(milliseconds, Ordering::Relaxed);
    }
}

impl crate::Clock for TestClock {
    fn now_ms(&self) -> f64 {
        self.0.load(Ordering::Relaxed) as f64
    }
}

#[test]
fn animation_frame_uses_the_injected_clock_timestamp() {
    const CORE_FIXTURE: &str = include_str!("../gen/test-runtime.js");
    let clock = Arc::new(TestClock::default());
    let mut runtime = JsRuntime::new_with_clock(clock.clone()).expect("runtime");
    runtime.boot(CORE_FIXTURE).expect("boot fixture");
    runtime
        .with(|ctx| ctx.eval::<(), _>("requestAnimationFrame(t => globalThis.frameTime = t)"))
        .expect("schedule animation frame");
    clock.advance_ms(125);
    runtime.tick().expect("tick fixture");
    let times = runtime
        .with(|ctx| ctx.eval::<Vec<f64>, _>("[globalThis.frameTime, performance.now()]"))
        .expect("read timestamps");
    assert_eq!(times, [125.0, 125.0]);
}

#[test]
fn javascript_and_rust_share_runtime_atom_ids() {
    let runtime = JsRuntime::new().expect("runtime");
    let ids = runtime
        .with(|ctx| {
            ctx.eval::<Vec<u32>, _>(
                r#"[__wabou_intern("width"), __wabou_intern("width"), __wabou_intern("height")]"#,
            )
        })
        .expect("intern from JavaScript");

    assert_eq!(ids[0], ids[1]);
    assert_ne!(ids[0], ids[2]);
    let atoms = runtime.atom_pool_handle();
    let atoms = atoms.borrow();
    assert_eq!(atoms.resolve(crate::Atom::from_raw(ids[0])), Some("width"));
    assert_eq!(atoms.resolve(crate::Atom::from_raw(ids[2])), Some("height"));
}

#[test]
fn mounted_capabilities_are_namespaced_and_reject_duplicates() {
    let runtime = JsRuntime::new().expect("runtime");
    runtime
        .mount_capability("workspace", |ctx, capability| {
            capability.set(
                "basename",
                Function::new(ctx, |path: String| {
                    path.rsplit('/').next().unwrap_or_default().to_owned()
                })?,
            )
        })
        .expect("mount workspace capability");
    let value = runtime
        .with(|ctx| {
            ctx.eval::<String, _>("__wabou_capabilities.workspace.basename('/tmp/readme.md')")
        })
        .expect("call mounted function");
    assert_eq!(value, "readme.md");
    let leaked = runtime
        .with(|ctx| ctx.eval::<bool, _>("typeof globalThis.basename !== 'undefined'"))
        .expect("inspect globals");
    assert!(!leaked);
    assert!(
        runtime
            .mount_capability("workspace", |_ctx, _capability| Ok(()))
            .is_err()
    );
}

#[test]
fn sleep_uses_rquickjs_async_scheduler_and_wakes_host() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let runtime = JsRuntime::new().expect("runtime");
    let wake_count = Arc::new(AtomicUsize::new(0));
    let callback_count = wake_count.clone();
    runtime.set_wake_callback(Arc::new(move || {
        callback_count.fetch_add(1, Ordering::Release);
    }));
    runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    "globalThis.sleepDone = false; __wabou_sleep(10).then(() => globalThis.sleepDone = true);",
                )
            })
            .expect("start sleep");
    assert!(!runtime.poll_async_runtime(), "sleep should park");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if runtime.take_async_wake() {
            runtime.poll_async_runtime();
        }
        let done = runtime
            .with(|ctx| ctx.eval::<bool, _>("globalThis.sleepDone"))
            .expect("read sleep state");
        if done {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "sleep timed out");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    // `RuntimeWake::notify` publishes the pending bit before invoking the
    // callback. The test thread can therefore observe and drain the bit,
    // finish the Promise, and reach this assertion while the notifying
    // thread is still between those two operations under heavy load.
    let callback_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    while wake_count.load(Ordering::Acquire) == 0 && std::time::Instant::now() < callback_deadline {
        std::thread::yield_now();
    }
    assert!(wake_count.load(Ordering::Acquire) >= 1);
}

#[test]
fn promise_jobs_are_time_sliced() {
    let runtime = JsRuntime::new().expect("runtime");
    runtime
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                    globalThis.jobCount = 0;
                    function nextJob() {
                      globalThis.jobCount++;
                      if (globalThis.jobCount < 1000) Promise.resolve().then(nextJob);
                    }
                    Promise.resolve().then(nextJob);
                    "#,
            )
        })
        .expect("queue promise jobs");

    assert!(
        runtime.poll_async_runtime(),
        "the first slice should exhaust its budget"
    );
    let first = runtime
        .with(|ctx| ctx.eval::<u32, _>("globalThis.jobCount"))
        .expect("read first slice count");
    assert!(first > 0 && first < 1000, "one poll drained {first} jobs");

    while runtime.poll_async_runtime() {}
    let final_count = runtime
        .with(|ctx| ctx.eval::<u32, _>("globalThis.jobCount"))
        .expect("read final job count");
    assert_eq!(final_count, 1000);
}

#[test]
fn fetch_wakes_host_and_resolves_on_js_thread() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("server address");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut request = [0u8; 4096];
        let count = stream.read(&mut request).expect("read request");
        let request = String::from_utf8_lossy(&request[..count]);
        assert!(
            request.starts_with("GET /story HTTP/1.1"),
            "unexpected request: {request}"
        );
        stream
                .write_all(
                    b"HTTP/1.1 201 Created\r\ncontent-type: application/json\r\ncontent-length: 10\r\nconnection: close\r\n\r\n{\"id\": 42}",
                )
                .expect("write response");
    });

    let runtime = JsRuntime::new().expect("runtime");
    let wake_count = Arc::new(AtomicUsize::new(0));
    let wake_count_for_callback = wake_count.clone();
    runtime.set_wake_callback(Arc::new(move || {
        wake_count_for_callback.fetch_add(1, Ordering::Release);
    }));
    runtime
            .with(|ctx| {
                ctx.eval::<(), _>(
                    format!(
                        "globalThis.fetchResult = null; __wabou_fetch('http://{address}/story', '{{}}').then(value => globalThis.fetchResult = value);"
                    ),
                )
            })
            .expect("start fetch");
    // The first scheduler slice may either reach the socket wait or
    // exhaust its ready-job budget while constructing the request. Both
    // are valid; only eventual wake + resolution is contractual.
    runtime.poll_async_runtime();

    let initial_result = runtime
        .with(|ctx| ctx.eval::<Option<String>, _>("globalThis.fetchResult"))
        .expect("inspect initial fetch result");
    let wake_required = initial_result.is_none();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let result = if let Some(result) = initial_result {
        result
    } else {
        loop {
            if runtime.take_async_wake() {
                runtime.poll_async_runtime();
            }
            if let Some(result) = runtime
                .with(|ctx| ctx.eval::<Option<String>, _>("globalThis.fetchResult"))
                .expect("inspect fetch result")
            {
                break result;
            }
            assert!(std::time::Instant::now() < deadline, "fetch timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    };
    if wake_required {
        // The pending bit is published before the callback is invoked, so
        // Promise completion can win the race by a few instructions.
        let callback_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
        while wake_count.load(Ordering::Acquire) == 0
            && std::time::Instant::now() < callback_deadline
        {
            std::thread::yield_now();
        }
        assert!(wake_count.load(Ordering::Acquire) >= 1);
    }
    runtime.take_async_wake();
    assert!(!runtime.poll_async_runtime(), "runtime should be idle");

    let result: serde_json::Value = serde_json::from_str(&result).expect("response JSON");
    assert_eq!(result["status"], 201);
    assert_eq!(result["statusText"], "Created");
    assert_eq!(result["body"], "{\"id\": 42}");
    assert_eq!(result["headers"]["content-type"], "application/json");
    server.join().expect("test server");
}

#[test]
fn host_ffi_surface_matches_contract() {
    // Lock the globals installed by a bare JsRuntime. Applier adds the
    // window, clipboard, and renderer-specific half of the host bridge;
    // its contract test compares that complete surface with host.ts.
    let runtime = JsRuntime::new().expect("runtime");
    let mut bridge: Vec<String> = runtime
        .with(|ctx| {
            ctx.eval::<Vec<String>, _>(
                r#"Object.keys(globalThis).filter(k => k.startsWith("__wabou"))"#,
            )
        })
        .expect("enumerate __wabou_* globals");
    bridge.sort();
    let mut expected = crate::host_abi::HOST_ABI
        .iter()
        .filter(|entry| {
            entry.direction == crate::host_abi::Direction::Host
                && entry.owner == "runtime"
                && entry.feature.is_none()
        })
        .map(|entry| entry.name.to_owned())
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(bridge, expected, "Rust-registered __wabou_* set drifted");

    let has_legacy_global = runtime
        .with(|ctx| ctx.eval::<bool, _>(r#"typeof globalThis.Wabou !== "undefined""#))
        .expect("check legacy Wabou global");
    assert!(!has_legacy_global, "legacy globalThis.Wabou must not exist");

    // Old names must be gone — a partial revert of the rename resurfaces here.
    let stale: Vec<String> = runtime
        .with(|ctx| {
            ctx.eval::<Vec<String>, _>(
                r#"
                    [
                      "__host_log", "__host_utf8_encode", "__host_utf8_decode",
                      "__bridge_flush", "__fetch", "__sleep",
                      "__resize_observe", "__resize_unobserve",
                      "__vite_update_style", "__vite_remove_style",
                    ].filter(k => typeof globalThis[k] !== "undefined")
                    "#,
            )
        })
        .expect("check for stale FFI names");
    assert!(stale.is_empty(), "stale FFI names still defined: {stale:?}");
}

#[test]
fn motion_value_animations_run_inside_quickjs() {
    const MOTION_FIXTURE: &str = include_str!("../gen/motion-test-runtime.js");
    let clock = Arc::new(TestClock::default());
    let mut runtime = JsRuntime::new_with_clock(clock.clone()).expect("runtime");
    runtime.boot(MOTION_FIXTURE).expect("boot Motion fixture");
    for frame in 0..300 {
        clock.advance_ms(10);
        runtime.tick().expect("drive animation frame");
        runtime.poll_async_runtime();
        if frame == 0 {
            let value = runtime
                .with(|ctx| ctx.eval::<f64, _>("globalThis.__wabou_motion_result.number"))
                .expect("read in-flight Motion value");
            assert!(
                value > 0.0 && value < 100.0,
                "40ms animation must still be in flight after 10ms: {value}"
            );
        }
        let done = runtime
            .with(|ctx| ctx.eval::<bool, _>("globalThis.__wabou_motion_result.done"))
            .expect("read Motion completion");
        if done {
            break;
        }
        assert!(frame < 299, "Motion fixture timed out");
    }
    let json = runtime
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.__wabou_motion_result)"))
        .expect("serialize Motion result");
    let result: serde_json::Value = serde_json::from_str(&json).expect("Motion result JSON");
    assert_eq!(result["number"].as_f64(), Some(100.0));
    assert_eq!(result["keyframe"].as_f64(), Some(10.0));
    assert!(result["spring"].as_f64().is_some_and(|value| value > 0.99));
    assert_eq!(result["cancelled"].as_f64(), Some(0.0));
    assert_eq!(result["transition"].as_f64(), Some(20.0));
    assert_eq!(result["transitionDone"].as_bool(), Some(true));
    assert!(
        result["transitionPeak"]
            .as_f64()
            .is_some_and(|value| value > 0.0 && value < 100.0),
        "retargeted transition should continue from an in-flight value"
    );
    assert!(result["color"].as_str().is_some_and(|color| {
        matches!(
            color,
            "#ffffff" | "rgb(255, 255, 255)" | "rgba(255, 255, 255, 1)"
        )
    }));
}
