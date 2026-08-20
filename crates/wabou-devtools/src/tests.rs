use super::*;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use wabou_host_api::NodeKey;

#[test]
fn query_and_inspect_are_bounded_and_semantic() {
    let mut state = DebugState::default();
    state.publish(DebugSnapshot {
        status: DebugStatus {
            protocol_version: PROTOCOL_VERSION,
            node_count: 1,
            ..Default::default()
        },
        nodes: vec![DebugNode {
            id: NodeKey::new(7, 1),
            tag: "span".into(),
            text: Some("1 comments".into()),
            classes: vec!["metadata".into()],
            ..Default::default()
        }],
    });
    let result = state
        .execute(&request(1, "queryNodes", json!({"query":"comments"})).command)
        .unwrap();
    assert_eq!(result.as_array().unwrap().len(), 1);
    let result = state
        .execute(&request(2, "inspectNode", json!({"id":{"lo":7,"hi":1}})).command)
        .unwrap();
    assert_eq!(result["tag"], "span");
}

#[test]
fn inspect_at_point_uses_paint_order_pointer_events_and_effective_clip() {
    let mut state = DebugState::default();
    let node = |id, pointer_events, clip: Option<Rect>| DebugNode {
        id,
        tag: "div".into(),
        rect: Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        },
        clip: DebugClipInfo {
            effective: clip.map(|rect| DebugClip {
                node_id: id,
                kind: "effective".into(),
                coordinate_space: "window-logical".into(),
                rect,
                radius: 0.0,
                transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            }),
            ..Default::default()
        },
        computed: DebugComputedStyle {
            pointer_events,
            ..Default::default()
        },
        ..Default::default()
    };
    state.publish(DebugSnapshot {
        nodes: vec![
            node(NodeKey::new(1, 1), true, None),
            node(NodeKey::new(2, 1), false, None),
            node(
                NodeKey::new(3, 1),
                true,
                Some(Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 10.0,
                    height: 10.0,
                }),
            ),
        ],
        ..Default::default()
    });

    let result = state
        .execute(&request(1, "inspectAtPoint", json!({"x": 50, "y": 50})).command)
        .unwrap();
    assert_eq!(result["node"]["id"], json!({"lo":1,"hi":1}));
}

#[test]
fn capture_case_freezes_snapshot_when_the_screenshot_completes() {
    let mut state = DebugState::default();
    state.request_capture_case(Some((12.0, 34.0)));
    state.publish(DebugSnapshot {
        status: DebugStatus {
            revision: 7,
            ..Default::default()
        },
        ..Default::default()
    });
    let path = std::env::temp_dir().join("wabou-capture-case-test.png");
    state.complete_screenshot(Ok(path.clone()));
    state.publish(DebugSnapshot {
        status: DebugStatus {
            revision: 8,
            ..Default::default()
        },
        ..Default::default()
    });

    let capture = state.capture_case_result().unwrap().as_ref().unwrap();
    assert_eq!(capture.snapshot.status.revision, 7);
    assert_eq!(capture.screenshot_path, path);
    assert_eq!(capture.point.as_ref().unwrap().x, 12.0);
}

#[test]
fn overlay_command_updates_runtime_diagnostics() {
    let mut state = DebugState::default();
    let value = state
        .execute(
            &request(
                1,
                "setOverlay",
                json!({
                    "layout": true,
                    "clips": true,
                    "hitTarget": true,
                    "selectedNode": {"lo":42,"hi":1}
                }),
            )
            .command,
        )
        .expect("set overlay");
    assert_eq!(value["selectedNode"], json!({"lo":42,"hi":1}));
    assert_eq!(
        state.overlay(),
        DebugOverlay {
            layout: true,
            clips: true,
            hit_target: true,
            selected_node: Some(NodeKey::new(42, 1)),
        }
    );
    assert!(state.take_overlay_change());
    assert!(!state.take_overlay_change());

    state.publish(DebugSnapshot {
        status: DebugStatus {
            revision: 9,
            ..Default::default()
        },
        ..Default::default()
    });

    let status = state
        .execute(&request(2, "status", empty_params()).command)
        .expect("read status after overlay update");
    assert_eq!(status["overlay"], value);
    assert_eq!(status["revision"], 9);

    state.record_overlay_paint(DebugOverlayPaintStats {
        enabled: true,
        layout_bounds: 7,
        clip_bounds: 2,
        highlights: 1,
        ..Default::default()
    });
    let status = state
        .execute(&request(3, "status", empty_params()).command)
        .expect("read status after overlay paint");
    assert_eq!(status["overlayPaint"]["sequence"], 1);
    assert_eq!(status["overlayPaint"]["layout_bounds"], 7);

    state.set_overlay(DebugOverlay {
        clips: true,
        ..Default::default()
    });
    let pending = state
        .execute(&request(4, "status", empty_params()).command)
        .expect("read pending overlay status");
    assert_eq!(pending["overlayPaint"]["sequence"], 1);
    assert_eq!(pending["overlayPaint"]["enabled"], false);
    assert_eq!(pending["overlayPaint"]["layout_bounds"], 0);

    state.publish(DebugSnapshot::default());
    let status = state
        .execute(&request(5, "status", empty_params()).command)
        .expect("read status after snapshot replacement");
    assert_eq!(status["overlayPaint"]["sequence"], 1);
    assert_eq!(status["overlay"]["clips"], true);
}

#[cfg(unix)]
#[test]
fn server_refuses_to_replace_a_regular_file() {
    let path = std::env::temp_dir().join(format!("wabou-regular-file-{}.sock", std::process::id()));
    fs::write(&path, b"keep me").unwrap();
    let error = match serve(DebugState::shared(), path.clone()) {
        Ok(_) => panic!("regular file must not be replaced"),
        Err(error) => error,
    };
    assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
    assert_eq!(fs::read(&path).unwrap(), b"keep me");
    fs::remove_file(path).unwrap();
}

#[cfg(unix)]
#[test]
fn unix_socket_round_trip_uses_versioned_status() {
    let state = DebugState::shared();
    state.write().unwrap().publish(DebugSnapshot {
        status: DebugStatus {
            protocol_version: PROTOCOL_VERSION,
            pid: 42,
            revision: 9,
            ..Default::default()
        },
        nodes: Vec::new(),
    });
    let path = std::env::temp_dir().join(format!(
        "wabou-test-{}-{}.sock",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _server = serve(state.clone(), path.clone()).unwrap();
    let response = call(&path, &request(1, "status", empty_params())).unwrap();
    assert_eq!(response.result.unwrap()["revision"], 9);
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let screenshot_state = state.clone();
    let worker = std::thread::spawn(move || {
        loop {
            let request = { screenshot_state.write().unwrap().take_screenshot_request() };
            if let Some(path) = request {
                screenshot_state
                    .write()
                    .unwrap()
                    .complete_screenshot(Ok(path));
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
    let response = call(&path, &request(2, "captureScreenshot", empty_params())).unwrap();
    worker.join().unwrap();
    assert!(
        response
            .result
            .unwrap_or_else(|| panic!("capture failed: {:?}", response.error))["path"]
            .as_str()
            .unwrap()
            .ends_with(".png")
    );

    let capture_state = state.clone();
    let worker = std::thread::spawn(move || {
        loop {
            let request = { capture_state.write().unwrap().take_screenshot_request() };
            if let Some(path) = request {
                capture_state.write().unwrap().complete_screenshot(Ok(path));
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    });
    let response = call(&path, &request(3, "captureCase", json!({"x": 10, "y": 20}))).unwrap();
    worker.join().unwrap();
    let capture = response.result.unwrap();
    assert_eq!(capture["snapshot"]["status"]["revision"], 9);
    assert_eq!(capture["point"]["x"], 10.0);
}
