use super::*;

#[test]
fn solid_resources_settle_native_promises_and_return_runtime_to_idle() {
    const RESOURCE_FIXTURE: &str = include_str!("../../gen/resource-test-runtime.js");
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
    let response = applier.handle_event(UiEvent::WindowMetrics(wabou_shell::WindowMetrics {
        window_id: 1,
        logical_width: 800,
        logical_height: 600,
        physical_width: 1600,
        physical_height: 1200,
        scale_factor: 2.0,
        maximized: true,
        focused: true,
    }));
    assert!(response.request_redraw);
    let payload = applier
        .js
        .with(|ctx| {
            ctx.eval::<String, _>(
                "globalThis.__host_got.find((x) => x.topic === 'wabou:window-metrics').payload",
            )
        })
        .unwrap();
    assert!(payload.contains("logicalWidth"));
    assert!(payload.contains("\"windowId\":1"));
    assert!(payload.contains("\"scaleFactor\":2.0"));
}

#[test]
fn window_bridge_is_available_during_initial_boot_and_targets_ids() {
    const CORE_FIXTURE: &str = include_str!("../../gen/test-runtime.js");
    let js = JsRuntime::new().expect("runtime");
    let mut applier =
        Applier::from_runtime_with_factories_and_window(js, builtin_factories(), Color::BLACK, 17);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");
    applier
        .js
        .with(|ctx| {
            ctx.eval::<(), _>(
                r#"
            globalThis.created = __wabou_test_host_api.createWindow({
              title: "Child", width: 640, height: 480,
              resizable: false, decorations: false, transparent: true
            });
            created.setTitle("Renamed");
            created.minimize();
            created.setMaximized(true);
            created.startDragging();
            created.close();
            globalThis.currentWindowId = __wabou_test_host_api.currentWindow().id;
            "#,
            )
        })
        .expect("call public window bridge");

    assert_eq!(
        applier
            .js
            .with(|ctx| ctx.eval::<u64, _>("currentWindowId"))
            .expect("current window id"),
        17
    );
    let created = match applier.take_effect().map(|request| request.payload) {
        Some(wabou_shell::EffectPayload::WindowCreate(request)) => {
            let window_id = request.window_id;
            let options = request.options;
            assert_eq!(options.title, "Child");
            assert_eq!(options.initial_inner_size, (640, 480));
            assert!(!options.resizable);
            assert!(!options.decorations);
            assert!(options.transparent);
            window_id
        }
        effect => panic!("unexpected effect: {effect:?}"),
    };
    for command in [
        wabou_shell::WindowCommand::SetTitle("Renamed".into()),
        wabou_shell::WindowCommand::Minimize,
        wabou_shell::WindowCommand::SetMaximized(true),
        wabou_shell::WindowCommand::StartDragging,
        wabou_shell::WindowCommand::Close,
    ] {
        assert_eq!(
            applier.take_effect().map(|request| request.payload),
            Some(wabou_shell::EffectPayload::WindowControl {
                window_id: created,
                command,
            })
        );
    }
}

#[test]
fn clipboard_bridge_routes_native_completions_back_to_javascript() {
    const CORE_FIXTURE: &str = include_str!("../../gen/test-runtime.js");
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");
    applier
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
        Some(wabou_shell::EffectRequest {
            id,
            payload: wabou_shell::EffectPayload::ClipboardWrite { text },
            ..
        }) => {
            assert_eq!(text, "hello");
            id
        }
        effect => panic!("unexpected write effect: {effect:?}"),
    };
    assert!(write_request.0 >= (1_u64 << 31));
    let read_request = match applier.take_effect() {
        Some(wabou_shell::EffectRequest {
            id,
            payload: wabou_shell::EffectPayload::ClipboardRead,
            ..
        }) => id,
        effect => panic!("unexpected read effect: {effect:?}"),
    };

    applier.complete_effect(wabou_shell::EffectCompletion {
        id: write_request,
        op: wabou_shell::effect::builtin::CLIPBOARD_WRITE,
        result: wabou_shell::EffectResult::Unit,
    });
    applier.js.poll_async_runtime();
    applier.complete_effect(wabou_shell::EffectCompletion {
        id: read_request,
        op: wabou_shell::effect::builtin::CLIPBOARD_READ,
        result: wabou_shell::EffectResult::ClipboardText(Some("world".into())),
    });
    for _ in 0..4 {
        applier.js.poll_async_runtime();
    }
    let completions = applier
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(clipboardResults)"))
        .expect("clipboard completions");
    assert_eq!(completions, r#"[["write",true],["read","world"]]"#);
}

#[test]
fn dialog_and_notification_bridges_route_typed_effects_and_completions() {
    const CORE_FIXTURE: &str = include_str!("../../gen/test-runtime.js");
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    applier
        .boot(CORE_FIXTURE)
        .expect("boot public core fixture");
    applier
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
        wabou_shell::EffectPayload::DialogOpen(wabou_shell::OpenDialogRequest {
            multiple: true,
            ref filters,
            ..
        }) if filters == &[wabou_shell::DialogFilter {
            name: "Text".into(),
            extensions: vec!["txt".into()],
        }]
    ));
    let notification = applier.take_effect().expect("notification effect");
    assert!(matches!(
        notification.payload,
        wabou_shell::EffectPayload::NotificationShow(wabou_shell::NotificationRequest {
            ref title,
            silent: true,
            ..
        }) if title == "Ready"
    ));

    applier.complete_effect(wabou_shell::EffectCompletion {
        id: notification.id,
        op: wabou_shell::effect::builtin::NOTIFICATION_SHOW,
        result: wabou_shell::EffectResult::Unit,
    });
    applier.complete_effect(wabou_shell::EffectCompletion {
        id: dialog.id,
        op: wabou_shell::effect::builtin::DIALOG_OPEN,
        result: wabou_shell::EffectResult::DialogPaths(Some(vec!["/tmp/note.txt".into()])),
    });
    for _ in 0..4 {
        applier.js.poll_async_runtime();
    }
    let completions = applier
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(systemResults)"))
        .expect("system effect completions");
    assert_eq!(
        completions,
        r#"[["dialog",["/tmp/note.txt"]],["notification",true]]"#
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
    let mut first = make(1);
    let mut second = make(2);
    first
        .boot(r#"globalThis.localState = 'first'; __wabou_effect_submit(2, 2, '{"windowId":1}')"#)
        .expect("boot first");
    second
        .boot(r#"globalThis.localState = 'second'; __wabou_effect_submit(2, 2, '{"windowId":2}')"#)
        .expect("boot second");

    assert_eq!(
        first
            .js
            .with(|ctx| ctx.eval::<String, _>("localState"))
            .unwrap(),
        "first"
    );
    assert_eq!(
        second
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
        wabou_shell::EffectPayload::WindowControl {
            window_id: 1,
            command: wabou_shell::WindowCommand::Close,
        }
    );
    assert_eq!(
        second_effect.payload,
        wabou_shell::EffectPayload::WindowControl {
            window_id: 2,
            command: wabou_shell::WindowCommand::Close,
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
            source: "body{}".into(),
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
fn full_reload_clears_non_root_scene_nodes() {
    let js = JsRuntime::new().expect("runtime");
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let div = applier.atoms.borrow_mut().intern("div");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: div,
        attrs: vec![],
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    assert!(applier.node_store.solid_to_node.contains_key(&2));
    applier.perform_full_reload("test");
    assert!(!applier.node_store.solid_to_node.contains_key(&2));
    assert!(applier.node_store.solid_to_node.contains_key(&1));
    assert_eq!(
        applier.node_store.tree.child_count(applier.node_store.root),
        0
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
    let handle = applier.reload_handle();
    handle.send(ReloadMsg::FullReload).unwrap();
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
        let mut atoms = applier.atoms.borrow_mut();
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
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: svg,
        attrs: vec![
            (view_box, "0 0 24 24"),
            (width, "24"),
            (height, "24"),
            (fill, "none"),
            (stroke, "currentColor"),
        ],
    });
    applier.apply_op(&Op::CreateElement {
        id: 3,
        tag: path,
        attrs: vec![(d, "M3 12h18")],
    });
    applier.apply_op(&Op::AppendChild {
        parent: 2,
        child: 3,
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    applier.rebuild_layout_boxes();
    applier.inherit();

    let svg_node = applier.node_store.solid_to_node[&2];
    assert_eq!(applier.node_store.tree.child_count(svg_node), 0);
    assert_eq!(
        applier
            .node_store
            .tree
            .get_node_context(svg_node)
            .unwrap()
            .intrinsic_size,
        Some([24.0, 24.0])
    );
    assert!(
        applier
            .node_store
            .tree
            .get_node_context(svg_node)
            .unwrap()
            .svg
            .is_some()
    );
    assert_eq!(applier.resources.svg.len(), 1);

    applier.apply_op(&Op::DropNode { id: 2 });
    assert!(applier.resources.svg.is_empty());
}

#[test]
fn image_resource_failure_routes_to_the_current_node_handle() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let img = applier.atoms.borrow_mut().intern("img");
    applier.apply_op(&Op::CreateElement {
        id: 2,
        tag: img,
        attrs: vec![],
    });
    applier.apply_op(&Op::AppendChild {
        parent: 1,
        child: 2,
    });
    applier.apply_op(&Op::AddEventListener {
        id: 2,
        event_type: event::RESOURCEERROR,
    });

    let node = applier.node_store.solid_to_node[&2];
    applier.dispatch_image_resource_result(
        node,
        "https://example.test/icon.png",
        &Err(Arc::from("bad image")),
    );

    let dispatched = applier
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
            "url": "https://example.test/icon.png",
            "error": "bad image",
        })
    );
}

#[test]
fn image_completion_only_notifies_current_source_subscribers() {
    let js = JsRuntime::new().expect("runtime");
    install_host_frame_test_hook(&js);
    let mut applier = Applier::from_runtime(js, Color::BLACK);
    let img = applier.atoms.borrow_mut().intern("img");
    for id in [2, 3] {
        applier.apply_op(&Op::CreateElement {
            id,
            tag: img,
            attrs: vec![],
        });
        applier.apply_op(&Op::AppendChild {
            parent: 1,
            child: id,
        });
        applier.apply_op(&Op::AddEventListener {
            id,
            event_type: event::RESOURCEERROR,
        });
    }

    let first = applier.node_store.solid_to_node[&2];
    let second = applier.node_store.solid_to_node[&3];
    let first_source: Arc<str> = Arc::from("https://example.test/first.png");
    let second_source: Arc<str> = Arc::from("https://example.test/second.png");
    applier
        .resources
        .node_image_sources
        .insert(first, first_source.clone());
    applier
        .resources
        .node_image_sources
        .insert(second, second_source.clone());
    applier
        .resources
        .image_subscribers
        .entry(first_source.clone())
        .or_default()
        .insert(first);
    applier
        .resources
        .image_subscribers
        .entry(second_source.clone())
        .or_default()
        .insert(second);

    applier.finish_image_source(&first_source, &Err(Arc::from("first failed")));

    let dispatched = applier
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
            "url": "https://example.test/first.png",
            "error": "first failed",
        })
    );
    assert!(
        !applier
            .resources
            .image_subscribers
            .contains_key(&first_source)
    );
    assert_eq!(applier.resources.image_subscribers[&second_source].len(), 1);

    applier.clear_image_source(second);
    applier.finish_image_source(&second_source, &Err(Arc::from("stale failure")));
    let dispatched_after_stale = applier
        .js
        .with(|ctx| ctx.eval::<String, _>("JSON.stringify(globalThis.dispatched)"))
        .unwrap();
    assert!(!dispatched_after_stale.contains("stale failure"));
}
