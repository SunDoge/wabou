use super::*;

#[derive(Default)]
struct TestClock(std::sync::atomic::AtomicU64);

impl TestClock {
    fn advance_ms(&self, milliseconds: u64) {
        self.0.fetch_add(milliseconds, Ordering::Relaxed);
    }
}

impl crate::clock::Clock for TestClock {
    fn now_ms(&self) -> f64 {
        self.0.load(Ordering::Relaxed) as f64
    }
}

#[test]
fn core_prelude_keeps_application_frames_in_deep_error_stacks() {
    let runtime = JsRuntime::new().expect("runtime");
    let limit = runtime
        .with(|ctx| ctx.eval::<u32, _>("Error.stackTraceLimit"))
        .expect("read stack trace limit");
    assert_eq!(limit, 100);
}

#[test]
fn runtime_options_expose_the_quickjs_stack_limit() {
    assert_eq!(
        JsRuntimeOptions::default().stack_size(),
        DEFAULT_QUICKJS_STACK_SIZE
    );
    assert_eq!(
        JsRuntimeOptions::default()
            .max_stack_size(8 * 1024 * 1024)
            .stack_size(),
        8 * 1024 * 1024
    );
}

#[test]
#[should_panic(expected = "QuickJS stack size must be greater than zero")]
fn runtime_options_reject_a_zero_stack_limit() {
    let _ = JsRuntimeOptions::default().max_stack_size(0);
}

#[test]
fn boot_reports_quickjs_stack_exhaustion_with_the_configured_limit() {
    let mut runtime =
        JsRuntime::new_with_options(JsRuntimeOptions::default().max_stack_size(256 * 1024))
            .expect("runtime");
    let error = runtime
        .boot("function recurse() { recurse(); } recurse();")
        .expect_err("recursive bundle must exhaust the configured stack");
    let diagnostic = error.to_string();
    assert!(diagnostic.contains("stack"), "{diagnostic}");
    assert!(diagnostic.contains("262144 bytes"), "{diagnostic}");
    assert!(diagnostic.contains("native thread stack"), "{diagnostic}");
}

#[test]
fn pure_javascript_compatibility_probes_resolve_json() {
    let mut runtime =
        JsRuntime::new_with_options(JsRuntimeOptions::default().max_stack_size(4 * 1024 * 1024))
            .expect("runtime");
    runtime
        .boot("globalThis.library = { double: async value => value * 2 };")
        .expect("boot pure JavaScript bundle");

    let result = runtime
        .eval_promise_json(
            "library.double(21).then(value => ({ value }))",
            std::time::Duration::from_secs(1),
        )
        .expect("evaluate compatibility probe");
    assert_eq!(result, r#"{"value":42}"#);
}

#[test]
fn pure_javascript_compatibility_probes_preserve_rejections() {
    let mut runtime = JsRuntime::new().expect("runtime");
    runtime.boot("").expect("boot empty bundle");
    let error = runtime
        .eval_promise_json(
            "Promise.reject(new Error('unsupported API'))",
            std::time::Duration::from_secs(1),
        )
        .expect_err("probe must reject");
    assert!(error.contains("unsupported API"), "{error}");
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
fn tick_preserves_a_protocol_frame_delivered_before_the_native_turn() {
    const CORE_FIXTURE: &str = include_str!("../gen/test-runtime.js");
    let mut runtime = JsRuntime::new().expect("runtime");
    runtime.boot(CORE_FIXTURE).expect("boot fixture");
    let _ = runtime.tick().expect("flush initial mount");
    runtime
        .with(|ctx| ctx.eval::<(), _>("__wabou_flush(new Uint8Array([87, 65, 66, 79, 85]));"))
        .expect("deliver protocol bytes between native turns");

    let (bytes, _) = runtime.tick().expect("consume pending protocol frame");
    assert_eq!(bytes, b"WABOU");
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
    assert_eq!(
        atoms.resolve(crate::atom::Atom::from_raw(ids[0])),
        Some("width")
    );
    assert_eq!(
        atoms.resolve(crate::atom::Atom::from_raw(ids[2])),
        Some("height")
    );
}

#[test]
fn color_theme_palette_fills_a_typed_array_without_json() {
    let runtime = JsRuntime::new().expect("runtime");
    *runtime.color_themes.borrow_mut() = Some(crate::style_ir::ColorThemes {
        default: "light".to_owned(),
        themes: std::collections::HashMap::from([(
            "light".to_owned(),
            crate::style_ir::ColorTheme {
                _appearance: crate::style_ir::Appearance::Light,
                colors: std::collections::HashMap::from([
                    ("foreground".to_owned(), 0x1122_33ff),
                    ("background".to_owned(), 0xaabb_ccff),
                ]),
            },
        )]),
    });

    let result = runtime
        .with(|ctx| {
            ctx.eval::<Vec<u32>, _>(
                r#"
                (() => {
                  const length = __wabou_get_color_theme_palette("light", undefined);
                  const palette = new Uint32Array(length);
                  const written = __wabou_get_color_theme_palette("light", palette);
                  return [length, written, ...palette];
                })()
                "#,
            )
        })
        .expect("read typed color palette");

    // Tokens are stable in lexical order: background, then foreground.
    assert_eq!(result, [2, 2, 0xaabb_ccff, 0x1122_33ff]);
}

#[test]
fn layout_snapshot_fills_a_versioned_typed_array_without_json() {
    let runtime = JsRuntime::new().expect("runtime");
    let id = NodeKey::new(7, 3);
    {
        let metrics = runtime.layout_metrics_handle();
        let mut snapshot = metrics.borrow_mut();
        snapshot.revision = 0x1_2345_6789;
        snapshot.viewport = LayoutRect {
            x: 1.0,
            y: 2.0,
            width: 800.0,
            height: 600.0,
        };
        snapshot.nodes.insert(
            id,
            LayoutMetric {
                rect: LayoutRect {
                    x: 10.0,
                    y: 20.0,
                    width: 30.0,
                    height: 40.0,
                },
                clip: LayoutRect {
                    x: 5.0,
                    y: 6.0,
                    width: 70.0,
                    height: 80.0,
                },
                scroll: LayoutScrollMetrics {
                    offset_x: 9.0,
                    offset_y: 10.0,
                    range_x: 11.0,
                    range_y: 12.0,
                },
            },
        );
    }

    let packed = runtime
        .with(|ctx| {
            ctx.eval::<Vec<f64>, _>(
                r#"
                (() => {
                  const ids = new Uint32Array([7, 3, 99, 1]);
                  const required = __wabou_layout_snapshot(ids, undefined);
                  const output = new Float64Array(required);
                  const written = __wabou_layout_snapshot(ids, output);
                  return [required, written, ...output];
                })()
                "#,
            )
        })
        .expect("read packed layout snapshot");

    assert_eq!(
        packed[0..10],
        [
            22.0,
            22.0,
            1.0,
            0x2345_6789 as f64,
            1.0,
            1.0,
            2.0,
            800.0,
            600.0,
            1.0
        ]
    );
    assert_eq!(
        packed[10..],
        [
            7.0, 3.0, 10.0, 20.0, 30.0, 40.0, 5.0, 6.0, 70.0, 80.0, 9.0, 10.0, 11.0, 12.0,
        ]
    );
}

#[test]
fn web_crypto_uses_native_randomness_and_off_thread_digests() {
    const CORE_FIXTURE: &str = include_str!("../gen/test-runtime.js");
    let mut runtime = JsRuntime::new().expect("runtime");
    runtime.boot(CORE_FIXTURE).expect("boot core fixture");
    runtime
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.cryptoResult = null;
                const random = crypto.getRandomValues(new Uint8Array(64));
                const uuid = crypto.randomUUID();
                crypto.subtle.digest("SHA-256", new Uint8Array([97, 98, 99]))
                  .then(value => globalThis.cryptoResult = {
                    randomHasEntropy: random.some(byte => byte !== 0),
                    uuid,
                    digest: Array.from(new Uint8Array(value)),
                  });
                "#,
            )
        })
        .expect("start native crypto operations");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while runtime
        .with(|ctx| ctx.eval::<bool, _>("globalThis.cryptoResult === null"))
        .expect("inspect crypto result")
    {
        runtime.poll_async_runtime();
        assert!(
            std::time::Instant::now() < deadline,
            "native digest timed out"
        );
        std::thread::yield_now();
    }
    let result = runtime
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.cryptoResult)"))
        .expect("serialize crypto result");
    let result: serde_json::Value = serde_json::from_str(&result).expect("crypto result JSON");
    assert_eq!(result["randomHasEntropy"], true);
    assert!(
        result["uuid"]
            .as_str()
            .is_some_and(|uuid| uuid.len() == 36 && uuid.as_bytes()[14] == b'4')
    );
    assert_eq!(
        result["digest"],
        serde_json::json!([
            186, 120, 22, 191, 143, 1, 207, 234, 65, 65, 64, 222, 93, 174, 34, 35, 176, 3, 97, 163,
            150, 23, 122, 156, 180, 16, 255, 97, 242, 0, 21, 173
        ])
    );
}

#[test]
fn whatwg_streams_run_inside_quickjs() {
    const CORE_FIXTURE: &str = include_str!("../gen/test-runtime.js");
    let mut runtime = JsRuntime::new().expect("runtime");
    runtime.boot(CORE_FIXTURE).expect("boot core fixture");
    runtime
        .with(|ctx| {
            ctx.eval::<(), _>(
                "globalThis.streamResult = null; __wabou_test_streams().then(value => globalThis.streamResult = value);",
            )
        })
        .expect("start stream pipeline");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while runtime
        .with(|ctx| ctx.eval::<bool, _>("globalThis.streamResult === null"))
        .expect("inspect stream result")
    {
        runtime.poll_async_runtime();
        assert!(
            std::time::Instant::now() < deadline,
            "WHATWG stream pipeline timed out"
        );
    }
    let result = runtime
        .with(|ctx| ctx.eval::<String, _>("globalThis.streamResult"))
        .expect("read stream result");
    assert_eq!(result, "QUICKJS");
}

#[test]
fn encoding_streams_run_inside_quickjs() {
    const CORE_FIXTURE: &str = include_str!("../gen/test-runtime.js");
    let mut runtime = JsRuntime::new().expect("runtime");
    runtime.boot(CORE_FIXTURE).expect("boot core fixture");
    let result = runtime
        .eval_promise_json(
            "__wabou_test_encoding_streams()",
            std::time::Duration::from_secs(1),
        )
        .expect("decode split UTF-8 inside QuickJS");
    assert_eq!(
        result,
        r#"{"text":"漫画","responseText":"buffered body","bodyUsed":true}"#
    );
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
fn capability_names_use_the_generated_contract_identifier_rules() {
    let runtime = JsRuntime::new().expect("runtime");
    for invalid in ["", "1workspace", "work-space", "工作区"] {
        assert!(
            runtime
                .mount_capability(invalid, |_ctx, _capability| Ok(()))
                .is_err(),
            "accepted invalid capability name {invalid:?}"
        );
    }
    runtime
        .mount_capability("_workspace2", |_ctx, _capability| Ok(()))
        .expect("shared identifier grammar accepts valid names");
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
    assert!(
        runtime.poll_async_runtime(),
        "starting the Promise must report JS progress"
    );
    assert!(
        !runtime
            .with(|ctx| ctx.eval::<bool, _>("globalThis.sleepDone"))
            .expect("read parked sleep state"),
        "the async sleep should still park after its initial scheduler work"
    );

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        if runtime.take_async_wake() {
            assert!(
                runtime.poll_async_runtime(),
                "settling a Promise must report JS progress so the shell schedules a frame"
            );
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
    let wake_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let wake_count_for_callback = wake_count.clone();
    runtime.set_wake_callback(Arc::new(move || {
        wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
    }));
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
    assert_eq!(
        wake_count.load(Ordering::Relaxed),
        0,
        "a UI-thread time slice must not recursively wake the event-loop proxy",
    );

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
                b"HTTP/1.1 201 Created\r\ncontent-type: application/octet-stream\r\ncontent-length: 4\r\nconnection: close\r\n\r\n\x00\xff\x80\x01",
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
        .with(|ctx| ctx.eval::<bool, _>("globalThis.fetchResult !== null"))
        .expect("inspect initial fetch result");
    let wake_required = !initial_result;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    if !initial_result {
        loop {
            if runtime.take_async_wake() {
                runtime.poll_async_runtime();
            }
            if runtime
                .with(|ctx| ctx.eval::<bool, _>("globalThis.fetchResult !== null"))
                .expect("inspect fetch result")
            {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "fetch timed out");
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
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

    let result = runtime
        .with(|ctx| {
            ctx.eval::<String, _>(
                r#"JSON.stringify({
                    ...globalThis.fetchResult,
                    body: Array.from(globalThis.fetchResult.body),
                    bodyIsUint8Array: globalThis.fetchResult.body instanceof Uint8Array,
                })"#,
            )
        })
        .expect("serialize fetch result for assertion");
    let result: serde_json::Value = serde_json::from_str(&result).expect("response JSON");
    assert_eq!(result["status"], 201);
    assert_eq!(result["statusText"], "Created");
    assert_eq!(result["bodyIsUint8Array"], true);
    assert_eq!(result["body"], serde_json::json!([0, 255, 128, 1]));
    assert_eq!(
        result["headers"]["content-type"],
        "application/octet-stream"
    );
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
fn application_can_configure_the_shared_debug_overlay() {
    let mut runtime = JsRuntime::new().expect("runtime");
    assert!(
        !runtime
            .with(|ctx| ctx.eval::<bool, _>("__wabou_set_debug_overlay(true, false, true)"))
            .expect("overlay without debug state")
    );

    let state = wabou_devtools::DebugState::shared();
    runtime.set_debug_state(state.clone());
    assert!(
        runtime
            .with(|ctx| ctx.eval::<bool, _>("__wabou_set_debug_overlay(true, false, true)"))
            .expect("overlay with debug state")
    );
    let overlay = state.read().expect("debug state").overlay();
    assert!(overlay.layout);
    assert!(!overlay.clips);
    assert!(overlay.hit_target);
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

#[test]
fn node_intrinsics_reject_malformed_key_halves() {
    let runtime = JsRuntime::new().expect("runtime");
    let rejected = runtime
        .with(|ctx| {
            ctx.eval::<Vec<bool>, _>(
                r#"
                [
                  () => __wabou_resize_observe(0, 1),
                  () => __wabou_resize_observe(2, 2),
                  () => __wabou_resize_unobserve(2, 0),
                  () => __wabou_layout_snapshot(new Uint32Array([2, 2]), undefined),
                  () => __wabou_layout_snapshot(new Uint32Array([2]), undefined),
                ].map(call => {
                  try { call(); return false; } catch { return true; }
                })
                "#,
            )
        })
        .expect("evaluate malformed node keys");
    assert_eq!(rejected, vec![true; 5]);
}

#[test]
fn harness_evaluation_preserves_values_and_exception_details() {
    let runtime = JsRuntime::new().expect("runtime");
    assert_eq!(
        runtime
            .eval_string("JSON.stringify(['first', 'second'])")
            .expect("string result"),
        r#"["first","second"]"#
    );
    let error = runtime
        .eval_script_diagnostic(
            "function fixtureFailure() { throw new Error('broken fixture') } fixtureFailure()",
        )
        .expect_err("fixture should fail");
    assert!(error.contains("broken fixture"));
    assert!(error.contains("fixtureFailure"));
}
