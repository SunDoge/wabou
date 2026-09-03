use super::*;
use std::collections::HashSet;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::sync::atomic::{AtomicUsize, Ordering};
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
fn snapshot_validation_reports_structural_geometry_and_reference_evidence() {
    let first = NodeKey::new(1, 1);
    let second = NodeKey::new(2, 1);
    let missing = NodeKey::new(99, 1);
    let mut state = DebugState::default();
    state.publish(DebugSnapshot {
        status: DebugStatus {
            revision: 12,
            node_count: 2,
            device_scale: 0.0,
            focused_node: Some(missing),
            ..Default::default()
        },
        nodes: vec![
            DebugNode {
                id: first,
                parent_id: Some(second),
                rect: Rect {
                    width: 10.0,
                    height: 10.0,
                    ..Default::default()
                },
                content_rect: Rect {
                    x: -2.0,
                    width: 12.0,
                    height: 10.0,
                    ..Default::default()
                },
                style_diagnostics: vec!["unsupported utility".to_owned()],
                semantic: Some(DebugSemanticProjection {
                    controls: vec![missing],
                    ..Default::default()
                }),
                ..Default::default()
            },
            DebugNode {
                id: second,
                parent_id: Some(first),
                rect: Rect {
                    width: -1.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ],
    });

    let report: DebugValidationReport = serde_json::from_value(
        state
            .execute(&request(1, "validateSnapshot", empty_params()).command)
            .expect("validate snapshot"),
    )
    .expect("validation report");
    assert_eq!(report.revision, 12);
    assert!(!report.valid);
    assert!(report.error_count >= 6, "{:#?}", report.issues);
    assert_eq!(report.warning_count, 1);
    assert!(!report.truncated);
    let codes = report
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<HashSet<_>>();
    for code in [
        "invalid-device-scale",
        "invalid-geometry",
        "content-outside-border",
        "parent-cycle",
        "dangling-interaction-target",
        "dangling-semantic-reference",
        "style-diagnostic",
    ] {
        assert!(codes.contains(code), "missing {code}: {:#?}", report.issues);
    }
}

#[test]
fn snapshot_validation_accepts_a_self_consistent_tree() {
    let root = NodeKey::new(1, 1);
    let child = NodeKey::new(2, 1);
    let mut state = DebugState::default();
    state.publish(DebugSnapshot {
        status: DebugStatus {
            revision: 3,
            node_count: 2,
            focused_node: Some(child),
            ..Default::default()
        },
        nodes: vec![
            DebugNode {
                id: root,
                rect: Rect {
                    width: 100.0,
                    height: 100.0,
                    ..Default::default()
                },
                content_rect: Rect {
                    width: 100.0,
                    height: 100.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            DebugNode {
                id: child,
                parent_id: Some(root),
                rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 20.0,
                    height: 20.0,
                },
                content_rect: Rect {
                    x: 10.0,
                    y: 10.0,
                    width: 20.0,
                    height: 20.0,
                },
                ..Default::default()
            },
        ],
    });

    let report = state.validation_report();
    assert!(report.valid, "{:#?}", report.issues);
    assert!(report.issues.is_empty());
}

#[test]
fn snapshot_validation_bounds_serialized_findings_without_losing_counts() {
    let mut state = DebugState::default();
    state.publish(DebugSnapshot {
        status: DebugStatus {
            node_count: MAX_VALIDATION_ISSUES + 10,
            ..Default::default()
        },
        nodes: (1..=(MAX_VALIDATION_ISSUES + 10))
            .map(|slot| DebugNode {
                id: NodeKey::new(slot as u32, 1),
                rect: Rect {
                    width: -1.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .collect(),
    });

    let report = state.validation_report();
    assert!(report.truncated);
    assert_eq!(report.issues.len(), MAX_VALIDATION_ISSUES);
    assert!(report.error_count > report.issues.len());
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
    let requested = state
        .request_capture_case(Some((12.0, 34.0)))
        .expect("request capture case");
    state.publish(DebugSnapshot {
        status: DebugStatus {
            revision: 7,
            ..Default::default()
        },
        ..Default::default()
    });
    state
        .complete_screenshot(&requested, Ok(requested.clone()))
        .expect("complete capture case");
    state.publish(DebugSnapshot {
        status: DebugStatus {
            revision: 8,
            ..Default::default()
        },
        ..Default::default()
    });

    let capture = state.capture_case_result().unwrap().as_ref().unwrap();
    assert_eq!(capture.snapshot.status.revision, 7);
    assert_eq!(capture.screenshot_path, requested);
    assert_eq!(capture.point.as_ref().unwrap().x, 12.0);
}

#[test]
fn capture_remains_single_flight_after_renderer_drains_request() {
    let mut state = DebugState::default();
    let first = state.request_screenshot().expect("request first capture");
    let (queued_path, _file) = state
        .take_screenshot_request()
        .expect("renderer drains first capture");
    assert_eq!(queued_path, first);

    let error = state
        .request_capture_case(None)
        .expect_err("in-flight capture must reject another client");
    assert!(error.contains("already in progress"));

    assert!(state.cancel_screenshot(&first));
    assert!(!first.exists());
    let second = state
        .request_capture_case(None)
        .expect("capture can recover after cancellation");
    assert_ne!(first, second);
}

#[cfg(unix)]
#[test]
fn capture_artifacts_are_reserved_inside_a_private_directory() {
    let mut state = DebugState::default();
    let path = state.request_screenshot().expect("reserve capture");
    let parent = path.parent().expect("capture parent");
    assert_eq!(
        fs::metadata(parent).unwrap().permissions().mode() & 0o777,
        0o700
    );
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let (queued_path, file) = state
        .take_screenshot_request()
        .expect("take reserved artifact");
    assert_eq!(queued_path, path);
    drop(file);
    state
        .complete_screenshot(&path, Err("renderer failed".to_owned()))
        .expect("publish renderer failure");
    assert!(!path.exists(), "failed captures must not leave artifacts");
}

#[test]
fn completed_capture_artifacts_have_bounded_retention() {
    let mut state = DebugState {
        capture_retention: 2,
        ..Default::default()
    };
    let mut captures = Vec::new();
    for _ in 0..3 {
        let path = state.request_screenshot().expect("reserve capture");
        let (_, file) = state
            .take_screenshot_request()
            .expect("take capture request");
        drop(file);
        state
            .complete_screenshot(&path, Ok(path.clone()))
            .expect("complete capture");
        captures.push(path);
    }

    assert!(!captures[0].exists(), "oldest capture must be evicted");
    assert!(captures[1].exists());
    assert!(captures[2].exists());
    assert_eq!(state.screenshot_result(), Some(&Ok(captures[2].clone())));
}

#[test]
fn renderer_cannot_redirect_a_reserved_capture_path() {
    let mut state = DebugState::default();
    let requested = state.request_screenshot().expect("reserve capture");
    let (_, file) = state
        .take_screenshot_request()
        .expect("take capture request");
    drop(file);
    let redirected = requested.with_file_name("redirected.png");

    state
        .complete_screenshot(&requested, Ok(redirected.clone()))
        .expect("publish rejected renderer result");
    let error = state
        .screenshot_result()
        .expect("capture result")
        .as_ref()
        .expect_err("redirected result must fail");
    assert!(error.contains("unexpected path"));
    assert!(!requested.exists());
    assert!(!redirected.exists());
}

#[test]
fn stale_capture_completion_cannot_finish_a_new_request() {
    let mut state = DebugState::default();
    let stale = state.request_screenshot().expect("request stale capture");
    assert!(state.cancel_screenshot(&stale));
    let current = state.request_screenshot().expect("request current capture");

    let error = state
        .complete_screenshot(&stale, Ok(stale.clone()))
        .expect_err("stale completion must be rejected");
    assert!(error.contains("stale DevTools capture"));
    assert!(state.screenshot_result().is_none());

    state
        .complete_screenshot(&current, Ok(current.clone()))
        .expect("complete current capture");
    assert_eq!(state.screenshot_result(), Some(&Ok(current)));
}

#[cfg(unix)]
#[test]
fn timed_out_capture_releases_the_single_flight_slot() {
    let state = DebugState::shared();
    let error = execute_capture_with_timeout(
        &state,
        &DebugCommand::CaptureScreenshot(EmptyParams {}),
        std::time::Duration::ZERO,
    )
    .expect_err("capture without a renderer must time out");
    assert!(error.contains("screenshot timed out"));

    state
        .write()
        .unwrap()
        .request_screenshot()
        .expect("timeout must release the capture slot");
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

#[test]
fn overlay_only_wakes_for_an_effective_configuration_change() {
    let mut state = DebugState::default();
    let wakes = Arc::new(AtomicUsize::new(0));
    let callback_wakes = wakes.clone();
    state.set_wake(Arc::new(move || {
        callback_wakes.fetch_add(1, Ordering::Release);
    }));

    assert!(!state.set_overlay(DebugOverlay::default()));
    assert_eq!(wakes.load(Ordering::Acquire), 0);
    assert!(!state.take_overlay_change());

    let overlay = DebugOverlay {
        layout: true,
        ..Default::default()
    };
    assert!(state.set_overlay(overlay));
    assert_eq!(wakes.load(Ordering::Acquire), 1);
    assert!(state.take_overlay_change());

    assert!(!state.set_overlay(overlay));
    assert_eq!(wakes.load(Ordering::Acquire), 1);
    assert!(!state.take_overlay_change());
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

#[test]
#[cfg(any(unix, windows))]
fn local_socket_round_trip_uses_versioned_status() {
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
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );

    let screenshot_state = state.clone();
    let worker = std::thread::spawn(move || {
        loop {
            let request = { screenshot_state.write().unwrap().take_screenshot_request() };
            if let Some((path, _file)) = request {
                screenshot_state
                    .write()
                    .unwrap()
                    .complete_screenshot(&path, Ok(path.clone()))
                    .unwrap();
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
            if let Some((path, _file)) = request {
                capture_state
                    .write()
                    .unwrap()
                    .complete_screenshot(&path, Ok(path.clone()))
                    .unwrap();
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
