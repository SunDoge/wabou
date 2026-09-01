use super::*;

#[test]
fn host_messages_dispatch_without_waiting_for_a_render_frame() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.__wabou_dispatch_host_frame = () => {
                  __wabou_effect_submit(7, 1, "null");
                  return { needsTick: false };
                };
                "#,
            )
        })
        .expect("install host-frame fixture");

    applier
        .host_message_handle()
        .emit_null("application.quit")
        .expect("enqueue host message");

    assert!(FrameSource::poll_async(&mut applier));
    assert!(matches!(
        FrameSource::take_effect(&mut applier).map(|request| request.payload),
        Some(gpui_shell::EffectPayload::ApplicationExit)
    ));
}

#[test]
fn solid_resources_settle_native_promises_and_return_runtime_to_idle() {
    const RESOURCE_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/gen/resource-test-runtime.js"
    ));
    let js = JsRuntime::new().expect("runtime");
    js.mount_capability("promiseTest", |ctx, capability| {
        capability.set(
            "resolve",
            rquickjs::Function::new(
                ctx.clone(),
                rquickjs::prelude::Async(|| async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok::<_, rquickjs::Error>("ready".to_owned())
                }),
            )?,
        )?;
        capability.set(
            "reject",
            rquickjs::Function::new(
                ctx,
                rquickjs::prelude::Async(|| async {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Err::<String, _>(rquickjs::Error::new_from_js_message(
                        "native promise",
                        "string",
                        "native rejected",
                    ))
                }),
            )?,
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let wake_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let callback_count = wake_count.clone();
    FrameSource::set_wake_callback(
        &mut applier,
        Arc::new(move || {
            callback_count.fetch_add(1, Ordering::Release);
        }),
    );
    applier
        .boot(RESOURCE_FIXTURE)
        .expect("boot resource fixture");

    let mut text_context = TextContext::new();
    applier.build_frame(&mut text_context, 400, 200);
    let texts = |applier: &Applier| {
        applier
            .document
            .node_store
            .declared
            .values()
            .filter_map(|declared| declared.text.as_deref().map(str::to_owned))
            .collect::<Vec<_>>()
    };
    let initial = texts(&applier);
    assert!(
        initial.iter().any(|text| text == "success pending"),
        "{initial:?}"
    );
    assert!(
        initial.iter().any(|text| text == "failure pending"),
        "{initial:?}"
    );
    // Solid schedules a renderer flush with requestAnimationFrame during the
    // initial mount. Drain that bounded work before checking whether the
    // unresolved native Promises themselves keep requesting frames.
    for _ in 0..8 {
        if !FrameSource::has_anim(&applier) {
            break;
        }
        applier.build_frame(&mut text_context, 400, 200);
    }
    assert!(
        !FrameSource::has_anim(&applier),
        "pending Promises must sleep after the renderer flushes"
    );

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if FrameSource::poll_async(&mut applier) {
            applier.build_frame(&mut text_context, 400, 200);
        }
        let current = texts(&applier);
        if current.iter().any(|text| text == "success ready")
            && current.iter().any(|text| text.starts_with("caught "))
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "resource settlement timed out: {current:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }

    assert!(wake_count.load(Ordering::Acquire) > 0);
    while FrameSource::poll_async(&mut applier) {
        applier.build_frame(&mut text_context, 400, 200);
    }
    assert!(!FrameSource::has_anim(&applier));
    assert!(!FrameSource::poll_async(&mut applier));
}

#[test]
fn window_metrics_reach_js_without_waiting_for_a_resize_frame() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let response = applier.handle_event(UiEvent::WindowMetrics(gpui_shell::WindowMetrics {
        window_key: gpui_shell::WindowResourceKey::from_parts(1, 1).unwrap(),
        logical_width: 800,
        logical_height: 600,
        physical_width: 1600,
        physical_height: 1200,
        scale_factor: 2.0,
        maximized: true,
        focused: true,
        outer_x: Some(120),
        outer_y: Some(80),
        occluded: false,
        color_scheme: Some(gpui_shell::ColorScheme::Dark),
        reduced_motion: false,
    }));
    assert!(response.request_redraw);
    assert_eq!(applier.frame.device_scale, 2.0);
    let payload = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "globalThis.__host_got.find((x) => x.topic === 'wabou:window-metrics').payload",
            )
        })
        .unwrap();
    let payload: serde_json::Value = serde_json::from_str(&payload).expect("window metrics JSON");
    assert_eq!(payload["windowId"]["lo"], 1);
    assert_eq!(payload["windowId"]["hi"], 1);
    assert_eq!(payload["logicalWidth"], 800);
    assert_eq!(payload["scaleFactor"], 2.0);
    assert_eq!(payload["outerX"], 120);
    assert_eq!(payload["outerY"], 80);
    assert_eq!(payload["occluded"], false);
    assert_eq!(payload["colorScheme"], "dark");
}

#[test]
fn native_file_drop_reaches_js_with_paths_and_logical_position() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let response = applier.handle_event(UiEvent::FileDrop(gpui_shell::FileDropEvent {
        phase: gpui_shell::FileDropPhase::Dropped,
        paths: vec!["/tmp/one.yaml".into(), "/tmp/two.torrent".into()],
        position: Some(gpui_shell::Point { x: 24.5, y: 31.0 }),
    }));
    assert!(response.request_redraw);
    let payload = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "globalThis.__host_got.find((x) => x.topic === 'wabou:file-drop').payload",
            )
        })
        .unwrap();
    let payload: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(payload["phase"], "dropped");
    assert_eq!(payload["paths"][0], "/tmp/one.yaml");
    assert_eq!(payload["paths"][1], "/tmp/two.torrent");
    assert_eq!(payload["position"]["x"], 24.5);
    assert_eq!(payload["position"]["y"], 31.0);
}

#[test]
fn native_gesture_reaches_js_with_explicit_non_dom_semantics() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let response = applier.handle_event(UiEvent::Gesture(gpui_shell::GestureEvent::Pan {
        delta_x: 12.5,
        delta_y: -4.0,
        phase: gpui_shell::GesturePhase::Changed,
    }));
    assert!(response.request_redraw);
    let payload = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "globalThis.__host_got.find((x) => x.topic === 'wabou:gesture').payload",
            )
        })
        .unwrap();
    let payload: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(payload["type"], "pan");
    assert_eq!(payload["deltaX"], 12.5);
    assert_eq!(payload["deltaY"], -4.0);
    assert_eq!(payload["phase"], "changed");
}

#[test]
fn application_lifecycle_reaches_js_without_a_render_frame() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let response = applier.handle_event(UiEvent::AppLifecycle(
        gpui_shell::AppLifecycleEvent::MemoryWarning,
    ));
    assert!(response.request_redraw);
    let payload = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "globalThis.__host_got.find((x) => x.topic === 'wabou:app-lifecycle').payload",
            )
        })
        .unwrap();
    let payload: serde_json::Value = serde_json::from_str(&payload).unwrap();
    assert_eq!(payload["state"], "memory-warning");
}

#[test]
fn modifier_changes_reach_js_as_typed_host_state() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let response = applier.handle_event(UiEvent::ModifiersChanged(
        gpui_shell::Modifiers::CONTROL | gpui_shell::Modifiers::SHIFT,
    ));
    assert!(response.request_redraw);
    let payload = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<i32, _>(
                "globalThis.__host_got.find((x) => x.topic === 'wabou:keyboard-modifiers').payload",
            )
        })
        .unwrap();
    assert_eq!(payload & 0b1111, 0b0011);
    assert_eq!((payload & 0b1_0000) != 0, cfg!(not(target_os = "macos")));
}

#[test]
fn window_bridge_is_available_during_initial_boot_and_targets_ids() {
    const CORE_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/gen/test-runtime.js"
    ));
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime_with_factories_and_window(
        js,
        builtin_factories(),
        Color::BLACK,
        gpui_shell::WindowResourceKey::from_parts(17, 1).unwrap(),
    );
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");
    applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
            globalThis.created = __wabou_test_host_api.createWindow({
              title: "Child", width: 640, height: 480,
              resizable: false, decorations: false, background: "transparent"
            }).then(created => {
              created.setTitle("Renamed");
              created.minimize();
              created.setMaximized(true);
              created.startDragging();
              created.show();
              created.close();
            });
            globalThis.currentWindowId = JSON.stringify(
              __wabou_test_host_api.currentWindow().id
            );
            "#,
            )
        })
        .expect("call public window bridge");

    assert_eq!(
        applier
            .runtime
            .js
            .with(|ctx| ctx.eval::<String, _>("currentWindowId"))
            .expect("current window id"),
        r#"{"lo":17,"hi":1}"#
    );
    let create_request = match applier.take_effect() {
        Some(request) => {
            let gpui_shell::EffectPayload::WindowCreate(window) = &request.payload else {
                panic!("unexpected effect: {:?}", request.payload)
            };
            let options = &window.options;
            assert_eq!(options.title, "Child");
            assert_eq!(options.initial_inner_size, (640, 480));
            assert!(!options.resizable);
            assert!(!options.decorations);
            assert!(options.transparent);
            request
        }
        None => panic!("missing create-window effect"),
    };
    let created_key = gpui_shell::WindowResourceKey::from_parts(42, 3).unwrap();
    applier.complete_effect(gpui_shell::EffectCompletion {
        id: create_request.id,
        op: gpui_shell::effect::builtin::WINDOW_CREATE,
        result: gpui_shell::EffectResult::Window(created_key),
    });
    for _ in 0..4 {
        applier.runtime.js.poll_async_runtime();
    }
    for command in [
        gpui_shell::WindowCommand::SetTitle("Renamed".into()),
        gpui_shell::WindowCommand::Minimize,
        gpui_shell::WindowCommand::SetMaximized(true),
        gpui_shell::WindowCommand::StartDragging,
        gpui_shell::WindowCommand::Show,
        gpui_shell::WindowCommand::Close,
    ] {
        assert_eq!(
            applier.take_effect().map(|request| request.payload),
            Some(gpui_shell::EffectPayload::WindowControl {
                window_id: created_key,
                command,
            })
        );
    }
}

#[test]
fn clipboard_bridge_routes_native_completions_back_to_javascript() {
    const CORE_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/gen/test-runtime.js"
    ));
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");
    applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
            globalThis.clipboardResults = [];
            __wabou_test_host_api.clipboard.writeText("hello").then(
              () => clipboardResults.push(["write", true]),
              () => clipboardResults.push(["write", false]),
            );
            __wabou_test_host_api.clipboard.readText().then(
              text => clipboardResults.push(["read", text]),
            );
            "#,
            )
        })
        .expect("call public clipboard bridge");

    let write_request = match applier.take_effect() {
        Some(gpui_shell::EffectRequest {
            id,
            payload: gpui_shell::EffectPayload::ClipboardWrite { text },
            ..
        }) => {
            assert_eq!(text, "hello");
            id
        }
        effect => panic!("unexpected write effect: {effect:?}"),
    };
    assert_ne!(write_request.0, 0);
    let read_request = match applier.take_effect() {
        Some(gpui_shell::EffectRequest {
            id,
            payload: gpui_shell::EffectPayload::ClipboardRead,
            ..
        }) => id,
        effect => panic!("unexpected read effect: {effect:?}"),
    };
    assert_ne!(read_request, write_request);

    applier.complete_effect(gpui_shell::EffectCompletion {
        id: write_request,
        op: gpui_shell::effect::builtin::CLIPBOARD_WRITE,
        result: gpui_shell::EffectResult::Unit,
    });
    applier.runtime.js.poll_async_runtime();
    applier.complete_effect(gpui_shell::EffectCompletion {
        id: read_request,
        op: gpui_shell::effect::builtin::CLIPBOARD_READ,
        result: gpui_shell::EffectResult::ClipboardText(Some("world".into())),
    });
    for _ in 0..4 {
        applier.runtime.js.poll_async_runtime();
    }
    let completions = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(clipboardResults)"))
        .expect("clipboard completions");
    assert_eq!(completions, r#"[["write",true],["read","world"]]"#);
}

#[test]
fn dialog_and_notification_bridges_route_typed_effects_and_completions() {
    const CORE_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/gen/test-runtime.js"
    ));
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");
    applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.systemResults = [];
                __wabou_test_host_api.dialog.open({
                  multiple: true,
                  filters: [{ name: "Text", extensions: [".txt"] }],
                }).then(paths => systemResults.push(["dialog", paths]));
                __wabou_test_host_api.notification.show({
                  title: "Ready",
                  silent: true,
                }).then(() => systemResults.push(["notification", true]));
                "#,
            )
        })
        .expect("call public system bridges");

    let dialog = applier.take_effect().expect("dialog effect");
    assert!(matches!(
        dialog.payload,
        gpui_shell::EffectPayload::DialogOpen(gpui_shell::OpenDialogRequest {
            multiple: true,
            ref filters,
            ..
        }) if filters == &[gpui_shell::DialogFilter {
            name: "Text".into(),
            extensions: vec!["txt".into()],
        }]
    ));
    let notification = applier.take_effect().expect("notification effect");
    assert!(matches!(
        notification.payload,
        gpui_shell::EffectPayload::NotificationShow(gpui_shell::NotificationRequest {
            ref title,
            silent: true,
            ..
        }) if title == "Ready"
    ));

    applier.complete_effect(gpui_shell::EffectCompletion {
        id: notification.id,
        op: gpui_shell::effect::builtin::NOTIFICATION_SHOW,
        result: gpui_shell::EffectResult::Unit,
    });
    applier.complete_effect(gpui_shell::EffectCompletion {
        id: dialog.id,
        op: gpui_shell::effect::builtin::DIALOG_OPEN,
        result: gpui_shell::EffectResult::DialogPaths(Some(vec!["/tmp/note.txt".into()])),
    });
    for _ in 0..4 {
        applier.runtime.js.poll_async_runtime();
    }
    let completions = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(systemResults)"))
        .expect("system effect completions");
    assert_eq!(
        completions,
        r#"[["dialog",["/tmp/note.txt"]],["notification",true]]"#
    );
}

#[test]
fn replayed_effect_completion_wakes_javascript_jobs() {
    const CORE_FIXTURE: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/gen/test-runtime.js"
    ));
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");

    let trace = crate::effect_trace::EffectTrace::fixtures();
    trace
        .enqueue_fixture(
            gpui_shell::effect::builtin::DIALOG_PICK_DIRECTORY,
            gpui_shell::EffectResult::DialogPaths(Some(vec!["/tmp/wabou".into()])),
        )
        .expect("queue dialog fixture");
    applier.runtime.effect_bridge.set_trace(trace);

    let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let callback_wakes = wakes.clone();
    FrameSource::set_wake_callback(
        &mut applier,
        Arc::new(move || {
            callback_wakes.fetch_add(1, Ordering::Release);
        }),
    );
    applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
                globalThis.replayedDirectory = null;
                __wabou_test_host_api.dialog.pickDirectory().then(
                  path => { replayedDirectory = path; },
                );
                "#,
            )
        })
        .expect("submit replayed dialog effect");

    wakes.store(0, Ordering::Release);
    assert!(
        applier.take_effect().is_none(),
        "fixture must replace live IO"
    );
    assert_eq!(wakes.load(Ordering::Acquire), 1);
    assert_eq!(
        applier
            .runtime
            .js
            .with(|ctx| ctx.eval::<String, _>("replayedDirectory"))
            .expect("replayed directory"),
        "/tmp/wabou"
    );
}

#[test]
fn applier_host_ffi_surface_matches_the_generated_schema() {
    let mut expected = crate::host_abi::HOST_ABI
        .iter()
        .filter(|entry| {
            entry.direction == crate::host_abi::Direction::Host && entry.feature.is_none()
        })
        .map(|entry| entry.name.to_owned())
        .collect::<Vec<_>>();
    expected.sort();

    let applier = Applier::from_runtime(JsRuntime::new().expect("runtime"), Color::BLACK);
    let mut actual = applier
        .runtime
        .js
        .with(|ctx| {
            ctx.eval::<Vec<String>, _>(
                r#"Object.keys(globalThis).filter(key => key.startsWith("__wabou"))"#,
            )
        })
        .expect("enumerate installed host ABI");
    actual.sort();
    assert_eq!(actual, expected, "schema and embedded host ABI drifted");
}

#[test]
fn window_runtimes_keep_globals_and_action_queues_isolated() {
    let make = |window_id| {
        Applier::from_runtime_with_factories_and_window(
            JsRuntime::new().expect("runtime"),
            builtin_factories(),
            Color::BLACK,
            window_id,
        )
    };
    let mut first = make(gpui_shell::WindowResourceKey::from_parts(1, 1).unwrap());
    let mut second = make(gpui_shell::WindowResourceKey::from_parts(2, 1).unwrap());
    first
        .boot(r#"globalThis.localState = 'first'; __wabou_effect_submit(2, 2, '{"windowId":{"lo":1,"hi":1}}')"#)
        .expect("boot first");
    second
        .boot(r#"globalThis.localState = 'second'; __wabou_effect_submit(2, 2, '{"windowId":{"lo":2,"hi":1}}')"#)
        .expect("boot second");

    assert_eq!(
        first
            .runtime
            .js
            .with(|ctx| ctx.eval::<String, _>("localState"))
            .unwrap(),
        "first"
    );
    assert_eq!(
        second
            .runtime
            .js
            .with(|ctx| ctx.eval::<String, _>("localState"))
            .unwrap(),
        "second"
    );
    let first_effect = first.take_effect().expect("first window effect");
    let second_effect = second.take_effect().expect("second window effect");
    assert_ne!(first_effect.id, second_effect.id);
    assert_eq!(
        first_effect.payload,
        gpui_shell::EffectPayload::WindowControl {
            window_id: gpui_shell::WindowResourceKey::from_parts(1, 1).unwrap(),
            command: gpui_shell::WindowCommand::Close,
        }
    );
    assert_eq!(
        second_effect.payload,
        gpui_shell::EffectPayload::WindowControl {
            window_id: gpui_shell::WindowResourceKey::from_parts(2, 1).unwrap(),
            command: gpui_shell::WindowCommand::Close,
        }
    );
    assert_eq!(first.take_effect(), None);
    assert_eq!(second.take_effect(), None);
}

#[test]
fn hmr_batch_coalesces_full_reload_over_partial_updates() {
    let batch = plan_hmr_batch([
        ReloadMsg::HmrUpdate {
            path: "/a.tsx".into(),
            accepted_path: "/a.tsx".into(),
            timestamp: 1,
            source: "export {}".into(),
        },
        ReloadMsg::CssUpdate {
            path: "/x.css".into(),
        },
        ReloadMsg::FullReload,
        ReloadMsg::HmrUpdate {
            path: "/b.tsx".into(),
            accepted_path: "/b.tsx".into(),
            timestamp: 2,
            source: "export {}".into(),
        },
    ]);
    assert!(batch.full_reload);
    assert_eq!(batch.js_updates.len(), 2);
    assert_eq!(batch.css_paths, vec!["/x.css".to_string()]);
}

#[test]
fn hmr_batch_preserves_js_update_order() {
    let batch = plan_hmr_batch([
        ReloadMsg::HmrUpdate {
            path: "/a.tsx".into(),
            accepted_path: "/a.tsx".into(),
            timestamp: 1,
            source: "a".into(),
        },
        ReloadMsg::HmrUpdate {
            path: "/b.tsx".into(),
            accepted_path: "/b.tsx".into(),
            timestamp: 2,
            source: "b".into(),
        },
    ]);
    assert!(!batch.full_reload);
    assert_eq!(
        batch
            .js_updates
            .iter()
            .map(|u| u.path.as_str())
            .collect::<Vec<_>>(),
        vec!["/a.tsx", "/b.tsx"]
    );
}

#[test]
fn vite_transform_error_does_not_reload_or_duplicate_the_last_good_scene() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let view = applier.document.atoms.borrow_mut().intern("view");
    let child = NodeKey::new(2, 1);
    applier.apply_op(&Op::CreateElement {
        id: child,
        tag: view,
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::ROOT,
        child,
    });

    let result = applier.apply_hmr_batch(plan_hmr_batch([
        ReloadMsg::FullReload,
        ReloadMsg::Error {
            diagnostic: r#"{"message":"transform failed"}"#.into(),
        },
    ]));

    assert!(matches!(result, HmrDrainResult::Error { .. }));
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .child_count(applier.document.node_store.root),
        1,
        "a transform error must leave the one last-good application root intact"
    );
    assert!(
        applier
            .document
            .node_store
            .solid_to_node
            .contains_key(&child)
    );
}

#[test]
fn failed_full_reload_keeps_the_last_good_scene_interactive() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.document.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: div,
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    applier
        .document
        .widget_manager
        .visibility
        .insert(node, true);
    applier
        .document
        .widget_manager
        .host_action_routes
        .insert(7, (node, 9));
    applier.interaction.input.focused_target = Some(NodeKey::new(2, 1));
    applier
        .interaction
        .scroll
        .offsets
        .insert(node, [12.0, 24.0]);
    assert!(
        applier
            .document
            .node_store
            .solid_to_node
            .contains_key(&NodeKey::new(2, 1))
    );
    applier.perform_full_reload("test");
    assert!(
        applier
            .document
            .node_store
            .solid_to_node
            .contains_key(&NodeKey::new(2, 1))
    );
    assert!(
        applier
            .document
            .node_store
            .solid_to_node
            .contains_key(&NodeKey::new(1, 1))
    );
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .child_count(applier.document.node_store.root),
        1
    );
    assert!(
        applier
            .document
            .widget_manager
            .visibility
            .contains_key(&node)
    );
    assert_eq!(
        applier.document.widget_manager.host_action_routes.get(&7),
        Some(&(node, 9))
    );
    assert_eq!(
        applier.interaction.input.focused_target,
        Some(NodeKey::new(2, 1))
    );
    assert_eq!(
        applier.interaction.scroll.offsets.get(&node),
        Some(&[12.0, 24.0])
    );
}

#[test]
fn host_messages_are_delivered_to_js_before_tick() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    js.with(|ctx| {
        ctx.eval::<(), _>(
            r#"
            globalThis.__wabou_tick = () => false;
            globalThis.__wabou_has_raf = () => false;
            "#,
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let handle = applier.host_message_handle();
    handle.emit_str("logs", "hello").unwrap();
    handle.emit_i32("count", 7).unwrap();

    let mut text = TextContext::new();
    applier.build_frame(&mut text, 100, 100);

    let got: String = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.__host_got)"))
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&got).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 2);
    assert_eq!(v[0]["topic"], "logs");
    assert_eq!(v[0]["payload"], "hello");
    assert_eq!(v[1]["topic"], "count");
    assert_eq!(v[1]["payload"], 7);
}

#[test]
fn hmr_queue_full_reload_is_drained_as_full_reload_result() {
    let js = JsRuntime::new().expect("runtime");
    js.with(|ctx| {
        ctx.eval::<(), _>(
            "globalThis.__wabou_tick = () => false; globalThis.__wabou_has_raf = () => false;",
        )
    })
    .unwrap();
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let wakes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let callback_wakes = wakes.clone();
    FrameSource::set_wake_callback(
        &mut applier,
        Arc::new(move || {
            callback_wakes.fetch_add(1, Ordering::Relaxed);
        }),
    );
    let handle = applier.reload_handle();
    handle.send(ReloadMsg::FullReload).unwrap();
    assert_eq!(wakes.load(Ordering::Relaxed), 1);
    assert!(FrameSource::poll_async(&mut applier));
    let mut text = TextContext::new();
    applier.build_frame(&mut text, 100, 100);
    assert!(matches!(
        applier.last_hmr_result(),
        HmrDrainResult::FullReload { .. }
    ));
}

#[test]
fn inline_svg_cache_follows_node_lifetime() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let (svg, path, view_box, width, height, fill, stroke, d) = {
        let mut atoms = applier.document.atoms.borrow_mut();
        (
            atoms.intern("svg"),
            atoms.intern("path"),
            atoms.intern("viewBox"),
            atoms.intern("width"),
            atoms.intern("height"),
            atoms.intern("fill"),
            atoms.intern("stroke"),
            atoms.intern("d"),
        )
    };
    create_element_with_attrs(
        &mut applier,
        2,
        svg,
        &[
            (view_box, "0 0 24 24"),
            (width, "24"),
            (height, "24"),
            (fill, "none"),
            (stroke, "currentColor"),
        ],
    );
    create_element_with_attrs(&mut applier, 3, path, &[(d, "M3 12h18")]);
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(2, 1),
        child: NodeKey::new(3, 1),
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.rebuild_layout_boxes();
    applier.inherit();

    let svg_node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    assert_eq!(applier.document.node_store.tree.child_count(svg_node), 0);
    assert_eq!(
        applier
            .document
            .node_store
            .tree
            .get_node_context(svg_node)
            .unwrap()
            .intrinsic_size,
        Some([24.0, 24.0])
    );
    assert!(
        applier
            .document
            .node_store
            .tree
            .get_node_context(svg_node)
            .unwrap()
            .svg
            .is_some()
    );
    assert_eq!(applier.document.resources.svg.len(), 1);

    applier.apply_op(&Op::DropNode {
        id: NodeKey::new(2, 1),
    });
    assert!(applier.document.resources.svg.is_empty());
}

#[test]
fn image_resource_failure_routes_to_the_current_node_handle() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let img = applier.document.atoms.borrow_mut().intern("img");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: img,
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::RESOURCEERROR,
    });

    let node = applier.document.node_store.solid_to_node[&NodeKey::new(2, 1)];
    applier.dispatch_image_resource_error(
        node,
        Some(crate::ImageResourceHandle { lo: 7, hi: 3 }),
        "bad image",
    );

    let dispatched = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.dispatched)"))
        .unwrap();
    let dispatched: serde_json::Value = serde_json::from_str(&dispatched).unwrap();
    assert_eq!(dispatched[0][0], 2);
    assert_eq!(dispatched[0][1], event::RESOURCEERROR);
    let payload: serde_json::Value =
        serde_json::from_str(dispatched[0][2].as_str().unwrap()).unwrap();
    assert_eq!(
        payload,
        serde_json::json!({
            "resource": { "lo": 7, "hi": 3 },
            "error": "bad image",
        })
    );
}

#[test]
fn image_resource_ready_reports_intrinsic_dimensions() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let img = applier.document.atoms.borrow_mut().intern("img");
    applier.apply_op(&Op::CreateElement {
        id: NodeKey::new(2, 1),
        tag: img,
    });
    applier.apply_op(&Op::AppendChild {
        parent: NodeKey::new(1, 1),
        child: NodeKey::new(2, 1),
    });
    applier.apply_op(&Op::AddEventListener {
        id: NodeKey::new(2, 1),
        event_type: event::RESOURCEREADY,
    });

    applier.dispatch_image_resource_ready(
        NodeKey::new(2, 1),
        crate::ImageResourceHandle { lo: 9, hi: 5 },
        2.0,
        1.0,
    );

    let dispatched = applier
        .runtime
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.dispatched)"))
        .unwrap();
    let dispatched: serde_json::Value = serde_json::from_str(&dispatched).unwrap();
    let payload: serde_json::Value =
        serde_json::from_str(dispatched[0][2].as_str().unwrap()).unwrap();
    assert_eq!(
        payload,
        serde_json::json!({ "resource": { "lo": 9, "hi": 5 }, "width": 2.0, "height": 1.0 })
    );
}
