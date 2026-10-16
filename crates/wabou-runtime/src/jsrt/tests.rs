use super::*;
use std::{cell::Cell, rc::Rc};

#[derive(Clone, rquickjs::JsLifetime)]
struct GeneratedHostState(Rc<Cell<u32>>);

#[rquickjs::module(rename_vars = "camelCase")]
mod generated_host_module {
    use super::GeneratedHostState;

    #[rquickjs::function]
    #[qjs(rename = "addOne")]
    pub fn add_one(value: u32) -> u32 {
        value + 1
    }

    #[rquickjs::function]
    pub fn increment(ctx: rquickjs::Ctx<'_>) -> rquickjs::Result<u32> {
        let state = ctx.userdata::<GeneratedHostState>().ok_or_else(|| {
            rquickjs::Exception::throw_type(&ctx, "generated host state is unavailable")
        })?;
        let value = state.0.get() + 1;
        state.0.set(value);
        Ok(value)
    }
}

#[test]
fn mounted_stateful_capability_releases_capture_on_runtime_drop() {
    for mode in ["uninvoked", "invoked", "invalid_global"] {
        let runtime = JsRuntime::new_with_options(JsRuntimeOptions::default()).expect("runtime");
        let state = Arc::new(Mutex::new(0_u32));
        let weak = Arc::downgrade(&state);
        runtime
            .mount_host_module("host", move |ctx, capability| {
                capability.set(
                    "tick",
                    Function::new(ctx, move || -> JsResult<()> {
                        *state.lock().unwrap() += 1;
                        Ok(())
                    })?,
                )
            })
            .expect("mount stateful capability");
        assert!(
            weak.upgrade().is_some(),
            "mounted function retains its capture"
        );
        if mode == "invoked" {
            runtime
                .with(|ctx| ctx.eval::<(), _>("__wabou_host_modules.host.tick()"))
                .expect("invoke capability");
            assert_eq!(*weak.upgrade().unwrap().lock().unwrap(), 1);
        } else if mode == "invalid_global" {
            assert!(runtime.eval_string("host.tick()").is_err());
            assert_eq!(*weak.upgrade().unwrap().lock().unwrap(), 0);
        }
        drop(runtime);
        assert!(
            weak.upgrade().is_none(),
            "runtime teardown finalizes the capture"
        );
    }
}

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
fn core_prelude_exposes_the_neutral_global_without_claiming_dom_support() {
    let runtime = JsRuntime::new().expect("runtime");
    let contract = runtime
        .with(|ctx| {
            ctx.eval::<Vec<bool>, _>(
                r#"
                [
                  self === globalThis,
                  "window",
                  "document",
                  "scrollX",
                  "scrollY",
                  "scrollTo",
                  "addEventListener",
                  "removeEventListener",
                ].map((value, index) => index === 0 ? value : !(value in globalThis))
                "#,
            )
        })
        .expect("inspect compatibility globals");
    assert_eq!(contract, vec![true; 8]);
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
        atoms.resolve(wabou_protocol::Atom::from_raw(ids[0])),
        Some("width")
    );
    assert_eq!(
        atoms.resolve(wabou_protocol::Atom::from_raw(ids[2])),
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
        .mount_host_module("workspace", |ctx, capability| {
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
            ctx.eval::<String, _>("__wabou_host_modules.workspace.basename('/tmp/readme.md')")
        })
        .expect("call mounted function");
    assert_eq!(value, "readme.md");
    let leaked = runtime
        .with(|ctx| ctx.eval::<bool, _>("typeof globalThis.basename !== 'undefined'"))
        .expect("inspect globals");
    assert!(!leaked);
    assert!(
        runtime
            .mount_host_module("workspace", |_ctx, _capability| Ok(()))
            .is_err()
    );
}

#[test]
fn mounts_rquickjs_generated_module_exports() {
    let runtime = JsRuntime::new().expect("runtime");
    runtime
        .mount_host_module_def::<js_generated_host_module>("generated")
        .expect("mount generated module");

    let keys = runtime
        .with(|ctx| {
            ctx.eval::<String, _>("JSON.stringify(Object.keys(__wabou_host_modules.generated))")
        })
        .expect("inspect generated exports");
    assert_eq!(keys, r#"["addOne","increment"]"#);

    let value = runtime
        .with(|ctx| ctx.eval::<u32, _>("__wabou_host_modules.generated.addOne(41)"))
        .expect("call generated export");
    assert_eq!(value, 42);
}

#[test]
fn mounts_rquickjs_generated_module_with_runtime_state() {
    let runtime = JsRuntime::new().expect("runtime");
    let value = Rc::new(Cell::new(9));
    runtime
        .mount_host_module_def_with_state::<js_generated_host_module, _>(
            "statefulGenerated",
            GeneratedHostState(value.clone()),
        )
        .expect("mount generated module with state");
    assert!(
        runtime
            .mount_host_module_def_with_state::<js_generated_host_module, _>(
                "secondStatefulGenerated",
                GeneratedHostState(Rc::new(Cell::new(100))),
            )
            .is_err(),
        "one runtime must not silently replace module state of the same type"
    );

    let result = runtime
        .with(|ctx| ctx.eval::<u32, _>("__wabou_host_modules.statefulGenerated.increment()"))
        .expect("call stateful generated export");
    assert_eq!(result, 10);
    assert_eq!(value.get(), 10);
}

#[test]
fn capability_names_use_the_generated_contract_identifier_rules() {
    let runtime = JsRuntime::new().expect("runtime");
    for invalid in ["", "1workspace", "work-space", "工作区"] {
        assert!(
            runtime
                .mount_host_module(invalid, |_ctx, _capability| Ok(()))
                .is_err(),
            "accepted invalid capability name {invalid:?}"
        );
    }
    runtime
        .mount_host_module("_workspace2", |_ctx, _capability| Ok(()))
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

    // Scheduler notifications can be stale/coalesced: a wake requests a poll,
    // but does not promise that the timer is ready or that a JS job will run.
    // Inject such a notification so this contract is exercised deterministically.
    runtime.runtime_wake.notify();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let progressed = runtime.take_async_wake() && runtime.poll_async_runtime();
        let done = runtime
            .with(|ctx| ctx.eval::<bool, _>("globalThis.sleepDone"))
            .expect("read sleep state");
        if done {
            assert!(
                progressed,
                "settling a Promise must report JS progress so the shell schedules a frame"
            );
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
fn installing_a_wake_callback_replays_work_queued_during_boot() {
    use std::sync::atomic::{AtomicUsize, Ordering};

    let runtime = JsRuntime::new().expect("runtime");
    runtime
        .with(|ctx| {
            ctx.eval::<(), _>(
                "globalThis.bootWorkDone = false; __wabou_sleep(1).then(() => globalThis.bootWorkDone = true);",
            )
        })
        .expect("start boot work");
    assert!(
        runtime.poll_async_runtime(),
        "boot work must enter the scheduler"
    );
    std::thread::sleep(std::time::Duration::from_millis(20));

    let wake_count = Arc::new(AtomicUsize::new(0));
    let callback_count = wake_count.clone();
    runtime.set_wake_callback(Arc::new(move || {
        callback_count.fetch_add(1, Ordering::Release);
    }));

    assert_eq!(wake_count.load(Ordering::Acquire), 1);
    assert!(runtime.take_async_wake());
    assert!(runtime.poll_async_runtime());
    assert!(
        runtime
            .with(|ctx| ctx.eval::<bool, _>("globalThis.bootWorkDone"))
            .expect("read boot work state")
    );
}

#[test]
fn promise_jobs_are_time_sliced() {
    let runtime = JsRuntime::new().expect("runtime");
    let wake_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let wake_count_for_callback = wake_count.clone();
    runtime.set_wake_callback(Arc::new(move || {
        wake_count_for_callback.fetch_add(1, Ordering::Relaxed);
    }));
    assert_eq!(
        wake_count.swap(0, Ordering::Relaxed),
        1,
        "installing a wake callback schedules the initial runtime pump",
    );
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
fn fetch_sends_binary_request_body_without_json_encoding() {
    use std::io::{Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let address = listener.local_addr().expect("server address");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .expect("set read timeout");
        let mut request = Vec::new();
        let header_end = loop {
            let mut chunk = [0u8; 1024];
            let count = stream.read(&mut chunk).expect("read request");
            assert!(count > 0, "connection closed before request body");
            request.extend_from_slice(&chunk[..count]);
            if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
        };
        while request.len() < header_end + 4 {
            let mut chunk = [0u8; 4];
            let count = stream.read(&mut chunk).expect("read request body");
            assert!(count > 0, "connection closed before request body");
            request.extend_from_slice(&chunk[..count]);
        }
        let headers = String::from_utf8_lossy(&request[..header_end]);
        assert!(headers.starts_with("PUT /upload HTTP/1.1"));
        assert!(headers.to_ascii_lowercase().contains("content-length: 4"));
        assert_eq!(&request[header_end..header_end + 4], &[0, 255, 128, 1]);
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\nconnection: close\r\n\r\n")
            .expect("write response");
    });

    let runtime = JsRuntime::new().expect("runtime");
    runtime
        .with(|ctx| {
            ctx.eval::<(), _>(format!(
                "globalThis.uploaded = false; __wabou_fetch('http://{address}/upload', '{{\"method\":\"PUT\"}}', 0, new Uint8Array([0,255,128,1])).then(() => uploaded = true);"
            ))
        })
        .expect("start upload");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        runtime.poll_async_runtime();
        if runtime
            .with(|ctx| ctx.eval::<bool, _>("uploaded"))
            .expect("inspect upload result")
        {
            break;
        }
        assert!(std::time::Instant::now() < deadline, "upload timed out");
        std::thread::yield_now();
    }
    server.join().expect("join server");
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
    let mut expected = crate::host_abi::host_function_names(Some("runtime"))
        .into_iter()
        .map(str::to_owned)
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
fn native_intl_host_functions_execute_inside_quickjs() {
    let runtime = JsRuntime::new().expect("runtime");
    let output = runtime
        .with(|ctx| {
            ctx.eval::<String, _>(
                r#"JSON.stringify({
                    number: __wabou_format_number("de-DE", 1234.5, "{}"),
                    parsed: __wabou_parse_number("de-DE", "1.234,5", "{}"),
                    plural: __wabou_plural_category("en", 1, "{}"),
                    date: __wabou_format_date("en", 0, '{"dateStyle":"medium","timeZone":"UTC"}'),
                    firstWeekday: __wabou_first_weekday("de-DE"),
                    relative: __wabou_format_relative_time("en", -1, "day", '{"numeric":"auto"}')
                })"#,
            )
        })
        .expect("native formatters");
    let output: serde_json::Value = serde_json::from_str(&output).expect("formatter result JSON");
    assert_eq!(output["number"], "1.234,5");
    assert_eq!(output["parsed"], 1234.5);
    assert_eq!(output["plural"], "one");
    assert_eq!(output["firstWeekday"], 1);
    assert_eq!(output["relative"], "yesterday");
    assert!(output["date"].as_str().unwrap().contains("1970"));
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

#[test]
fn web_api_url_and_streaming_encoding_contracts() {
    let runtime = JsRuntime::new_with_options(JsRuntimeOptions::default()).unwrap();
    runtime.with(|ctx| ctx.eval::<(), _>(r#"
        function equal(actual, expected) { if (JSON.stringify(actual) !== JSON.stringify(expected)) throw Error(JSON.stringify({actual, expected})); }
        const u = new URL('../a?q=one+two&q=3', 'https://例子.test:443/path/file');
        equal(u.hostname, 'xn--fsqu00a.test');
        equal(u.pathname, '/a');
        const params = u.searchParams;
        equal(params.getAll('q'), ['one two', '3']);
        params.set('q', 'é !');
        equal(u.search, '?q=%C3%A9+%21');
        u.search = '?z=2&a=1&a=0';
        equal(u.searchParams === params, true);
        params.sort();
        equal(u.search, '?a=1&a=0&z=2');
        u.port = 'invalid'; equal(u.port, '');
        equal(URL.canParse('bad'), false);
        equal(new URL('mailto:a@b').origin, 'null');
        equal(new URLSearchParams('x=%FF&y=a+b').get('x'), '�');
        const iter = new URLSearchParams('a=1&b=2');
        const entries = iter.entries(); entries.next(); iter.append('c', '3');
        equal(Array.from(entries), [['b','2'], ['c','3']]);
        const dest = new Uint8Array(5).fill(99);
        equal(new TextEncoder().encodeInto('😀ab', dest), {read:3,written:5});
        equal(Array.from(dest), [240,159,152,128,97]);
        equal(Array.from(new TextEncoder().encode('\ud800')), [239,191,189]);
        const controller = new AbortController();
        const reason = {message:'stop'}; controller.abort(reason);
        equal(controller.signal.reason === reason, true);
        let thrown = false; try { controller.signal.throwIfAborted(); } catch(e) { thrown = e === reason; } equal(thrown, true);
        equal(AbortSignal.any([controller.signal]).reason === reason, true);
        const d = new TextDecoder();
        equal(d.decode(new Uint8Array([0xe2,0x82]), {stream:true}), '');
        equal(d.decode(new Uint8Array([0xac])), '€');
        equal(d.decode(new Uint8Array([0xef,0xbb,0xbf,65])), 'A');
        equal(d.decode(new Uint8Array([0xef,0xbb,0xbf,66])), 'B');
        equal(new TextDecoder('latin1').decode(new Uint8Array([0x80])), '€');
        equal(new TextDecoder('utf-16le').decode(new Uint8Array([65,0])), 'A');
        equal(new TextDecoder('utf-8',{ignoreBOM:true}).decode(new Uint8Array([0xef,0xbb,0xbf])), '\ufeff');
        let rejected = false; try { new TextDecoder('nope'); } catch(e) { rejected = e instanceof RangeError; } equal(rejected,true);
        const fatal = new TextDecoder('utf-8', {fatal:true});
        rejected = false; try { fatal.decode(new Uint8Array([0xff])); } catch(e) { rejected = e instanceof TypeError; } equal(rejected,true);
        equal(fatal.decode(new Uint8Array([65])), 'A');
    "#)).unwrap();
    // Teardown also proves the native codec does not retain a Ctx cycle.
    drop(runtime);
}

#[test]
fn web_api_cancelled_fetch_never_opens_connection() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let address = listener.local_addr().unwrap();
    let runtime = JsRuntime::new().unwrap();
    runtime.with(|ctx| ctx.eval::<(), _>(format!(
        "globalThis.cancelled = false; __wabou_fetch_start(1); __wabou_fetch_cancel(1); __wabou_fetch('http://{address}/', '{{}}', 1).catch(() => cancelled = true);"
    ))).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        runtime.poll_async_runtime();
        if runtime
            .with(|ctx| ctx.eval::<bool, _>("cancelled"))
            .unwrap()
        {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "cancellation did not settle"
        );
        std::thread::yield_now();
    }
    assert_eq!(
        listener.accept().unwrap_err().kind(),
        std::io::ErrorKind::WouldBlock
    );
}

#[test]
fn web_api_cancellation_drops_an_inflight_http_request() {
    use std::io::Read;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (accepted_tx, accepted_rx) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
            .unwrap();
        let mut buffer = [0; 2048];
        assert!(socket.read(&mut buffer).unwrap() > 0);
        accepted_tx.send(()).unwrap();
        // With no response sent, only cancellation can close this connection.
        assert_eq!(socket.read(&mut buffer).unwrap(), 0);
    });
    let runtime = JsRuntime::new().unwrap();
    runtime.with(|ctx| ctx.eval::<(), _>(format!("globalThis.stopped = false; __wabou_fetch_start(2); __wabou_fetch('http://{address}/', '{{}}', 2).catch(() => stopped = true);"))).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        runtime.poll_async_runtime();
        if accepted_rx.try_recv().is_ok() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "request was not received"
        );
        std::thread::yield_now();
    }
    runtime
        .with(|ctx| ctx.eval::<(), _>("__wabou_fetch_cancel(2)"))
        .unwrap();
    loop {
        runtime.poll_async_runtime();
        if runtime.with(|ctx| ctx.eval::<bool, _>("stopped")).unwrap() {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "request was not cancelled"
        );
        std::thread::yield_now();
    }
    server.join().unwrap();
}

#[test]
fn encoding_stream_boundaries_run_inside_quickjs() {
    let mut runtime = JsRuntime::new().unwrap();
    runtime
        .boot(include_str!("../gen/test-runtime.js"))
        .unwrap();
    let result = runtime.eval_promise_json(r#"(async () => {
        const source = new ReadableStream({start(c) {
            for (const chunk of ['\ud83d', '', '\ude80', '\ud800']) c.enqueue(chunk);
            c.close();
        }});
        const chunks = [];
        for await (const chunk of source.pipeThrough(new TextEncoderStream())) chunks.push(Array.from(chunk));
        const bytes = new ReadableStream({start(c) { c.enqueue(new Uint8Array([0xe2,0x82])); c.close(); }});
        let fatal = false;
        try { for await (const chunk of bytes.pipeThrough(new TextDecoderStream('utf-8', {fatal:true}))) {} }
        catch(e) { fatal = e instanceof TypeError; }
        const transform = new TextEncoderStream();
        const writer = transform.writable.getWriter();
        const closed = writer.closed.catch(reason => reason);
        await transform.readable.cancel('cancel encoding');
        return {chunks, fatal, cancelled:await closed};
    })()"#, std::time::Duration::from_secs(1)).unwrap();
    assert_eq!(
        result,
        r#"{"chunks":[[240,159,154,128],[239,191,189]],"fatal":true,"cancelled":"cancel encoding"}"#
    );
}

#[test]
fn buffered_response_memory_contract_runs_inside_quickjs() {
    let mut runtime = JsRuntime::new().unwrap();
    runtime
        .boot(include_str!("../gen/test-runtime.js"))
        .unwrap();
    let result = runtime.eval_promise_json(r#"(async () => {
        const source = new Uint8Array([99,65,66,88]);
        const response = new Response(source.subarray(1,3));
        source[1] = 90;
        const clone = response.clone();
        const bytes = await response.bytes();
        bytes[0] = 67;
        const cloned = Array.from(await clone.bytes());
        const buffer = await new Response(source.subarray(1,3)).arrayBuffer();
        const input = new Uint8Array([99,0xe2,0x82,0xac,88]);
        const decoded = new TextDecoder().decode(input.subarray(1,4));
        return {cloned,buffer:Array.from(new Uint8Array(buffer)),decoded,input:Array.from(input),nullBody:new Response(null).clone().body === null};
    })()"#, std::time::Duration::from_secs(1)).unwrap();
    assert_eq!(
        result,
        r#"{"cloned":[65,66],"buffer":[90,66],"decoded":"€","input":[99,226,130,172,88],"nullBody":true}"#
    );
}

#[test]
fn quickjs_memory_snapshot_reports_live_runtime_allocations() {
    let runtime = JsRuntime::new().unwrap();
    runtime
        .with(|ctx| ctx.eval::<(), _>("globalThis.memoryProbe = new Uint8Array(4096)"))
        .unwrap();
    let memory = runtime.memory_usage_snapshot();
    assert!(memory.malloc_bytes >= memory.used_bytes);
    assert!(memory.used_bytes > 0);
    assert!(memory.object_count > 0);
}

#[test]
fn solid_proxy_props_preserve_nested_native_children() {
    let mut runtime = JsRuntime::new().unwrap();
    runtime
        .boot(include_str!("../gen/test-runtime.js"))
        .unwrap();
    let result: String = runtime
        .with(|ctx| ctx.eval("JSON.stringify(__wabou_test_proxy_children())"))
        .unwrap();
    let result: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(result["mergedChild"], true);
    assert_eq!(result["omittedChild"], true);
}
