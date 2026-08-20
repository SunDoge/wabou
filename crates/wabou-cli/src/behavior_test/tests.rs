use std::fs;
use std::path::Path;

use serde_json::json;

use super::*;

#[test]
fn report_reconstructs_shared_state_through_selected_test() {
    let report = json!({
        "version": 1,
        "tests": [
            { "name": "prepares state", "traceStart": 0, "traceEnd": 2 },
            { "name": "fails here", "traceStart": 2, "traceEnd": 4 }
        ],
        "trace": [
            { "action": "showWindow", "windowId": 1 },
            { "action": "showWindow", "windowId": 2 },
            { "action": "showWindow", "windowId": 3 },
            { "action": "showWindow", "windowId": 4 },
            { "action": "showWindow", "windowId": 5 }
        ]
    });

    let replay = replay_actions_from_value(&report, Some("fails here")).unwrap();
    assert_eq!(replay.as_array().unwrap().len(), 4);
    assert_eq!(replay[0]["windowId"], 1);
    assert_eq!(replay[3]["windowId"], 4);
}

#[test]
fn input_validation_reports_actionable_errors() {
    let trace = json!([{ "action": "showWindow", "windowId": 1 }]);
    assert_eq!(
        replay_actions_from_value(&json!({ "version": 1, "actions": trace.clone() }), None)
            .unwrap(),
        trace
    );
    assert_error_contains(&trace, Some("test"), "requires report.json");

    let malformed = json!({
        "tests": [{ "name": "broken", "traceEnd": 2 }],
        "trace": [{ "action": "showWindow", "windowId": 1 }]
    });
    assert_error_contains(&malformed, Some("broken"), "exceeds trace length");
    assert_error_contains(&malformed, Some("missing"), "no test named");
    assert_error_contains(
        &json!({
            "tests": [{
                "name": "reversed",
                "traceStart": 2,
                "traceEnd": 1
            }],
            "trace": [{ "action": "showWindow", "windowId": 1 }]
        }),
        Some("reversed"),
        "traceStart 2 exceeds traceEnd 1",
    );
    assert_error_contains(
        &json!({
            "tests": [{
                "name": "fractional",
                "traceStart": 0.5,
                "traceEnd": 1
            }],
            "trace": [{ "action": "showWindow", "windowId": 1 }]
        }),
        Some("fractional"),
        "invalid traceStart",
    );
    assert_error_contains(
        &json!({ "version": 2, "actions": [{ "action": "click" }] }),
        None,
        "unsupported replay artifact version 2",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "clickByRole",
                "windowId": "not-a-window",
                "role": "button"
            }]
        }),
        None,
        "invalid replay action 0",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertWindowState",
                "windowId": 1,
                "expected": { "presence": "visible", "surfaceGeneration": 1 },
                "wait": { "timeout": -1, "interval": 16 }
            }]
        }),
        None,
        "wait.timeout must be a finite non-negative number",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertWindowState",
                "windowId": 1,
                "expected": { "presence": "visible", "surfaceGeneration": 1 },
                "wait": { "timeout": 100, "interval": 16, "stableFor": 101 }
            }]
        }),
        None,
        "wait.stableFor cannot exceed wait.timeout",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "showWindow",
                "windowId": 1,
                "windowID": 2
            }]
        }),
        None,
        "unknown field `windowID`",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertWindowState",
                "windowId": 1,
                "expected": {
                    "presence": "visible",
                    "surfaceGeneration": 1,
                    "surfaceGenration": 2
                },
                "wait": { "timeout": 1, "interval": 0 }
            }]
        }),
        None,
        "unknown field `surfaceGenration`",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertByRole",
                "windowId": 1,
                "role": "button",
                "label": "Default",
                "index": 0,
                "assertion": { "type": "count", "expected": 2 },
                "wait": { "timeout": 1000, "interval": 16 }
            }]
        }),
        None,
        "count assertion requires an unindexed locator",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertByRole",
                "windowId": 1,
                "role": "button",
                "label": "Default",
                "assertion": {
                    "type": "count",
                    "expected": 9_007_199_254_740_992_u64
                },
                "wait": { "timeout": 1000, "interval": 16 }
            }]
        }),
        None,
        "locator count must not exceed",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertByRole",
                "windowId": 1,
                "role": "slider",
                "label": "Volume",
                "assertion": {
                    "type": "numericRange",
                    "expected": { "value": 0.3 },
                    "tolerance": -1
                },
                "wait": { "timeout": 1000, "interval": 16 }
            }]
        }),
        None,
        "numeric range assertion tolerance must be a finite non-negative number",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertByRole",
                "windowId": 1,
                "role": "button",
                "label": "Save",
                "assertion": {
                    "type": "bounds",
                    "expected": {},
                    "tolerance": 0.5
                },
                "wait": { "timeout": 1000, "interval": 16 }
            }]
        }),
        None,
        "bounds assertion expected object cannot be empty",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "assertByRole",
                "windowId": 1,
                "role": "button",
                "label": "Save",
                "assertion": {
                    "type": "bounds",
                    "expected": { "width": 100 },
                    "tolerance": -1
                },
                "wait": { "timeout": 1000, "interval": 16 }
            }]
        }),
        None,
        "bounds assertion tolerance must be a finite non-negative number",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "clickByRole",
                "windowId": 1,
                "role": "button",
                "label": "Default",
                "index": 4_294_967_296_u64
            }]
        }),
        None,
        "index must not exceed",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "inputByRole",
                "windowId": 1,
                "role": "textbox",
                "label": "Name",
                "input": { "type": "key", "key": "", "modifiers": 0 }
            }]
        }),
        None,
        "input key must be a non-empty string",
    );
    assert_error_contains(
        &json!({
            "version": 1,
            "actions": [{
                "action": "inputByRole",
                "windowId": 1,
                "role": "textbox",
                "label": "Name",
                "input": { "type": "key", "key": "a", "modifiers": 16 }
            }]
        }),
        None,
        "input modifiers contain unknown bits",
    );
}

fn assert_error_contains(value: &Value, test_name: Option<&str>, expected: &str) {
    let error = replay_actions_from_value(value, test_name).unwrap_err();
    assert!(
        error.to_string().contains(expected),
        "expected {error:?} to contain {expected:?}"
    );
}

#[test]
fn validation_accepts_every_version_one_action_family() {
    let actions = json!([
        {
            "action": "respondToEffect",
            "operation": "dialogPickDirectory",
            "result": ["/tmp/downloads"]
        },
        {
            "action": "nativeClose",
            "windowId": { "lo": 1, "hi": 1 },
            "platform": "wayland"
        },
        { "action": "showWindow", "windowId": 1 },
        { "action": "clickByRole", "windowId": 1, "role": "button", "label": "Save", "index": 1 },
        { "action": "clickByRole", "windowId": 1, "role": "menuitem", "label": "Copy" },
        {
            "action": "inputByRole",
            "windowId": 1,
            "role": "textbox",
            "label": "Name",
            "input": { "type": "probe" }
        },
        {
            "action": "waitForByRole",
            "windowId": 1,
            "role": "status",
            "label": "Ready",
            "wait": { "timeout": 1000, "interval": 16, "stableFor": 400 }
        },
        {
            "action": "resizeWindow",
            "windowId": 1,
            "width": 900,
            "height": 600
        },
        {
            "action": "waitForByRole",
            "windowId": 1,
            "role": "treeitem",
            "label": "Documents",
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "button",
            "label": "Default",
            "assertion": { "type": "count", "expected": 2 },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "button",
            "label": "Downloads",
            "assertion": { "type": "current", "expected": "page" },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "dialog",
            "label": "Settings",
            "assertion": { "type": "absent" },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "checkbox",
            "label": "Select all",
            "assertion": { "type": "checked", "expected": "mixed" },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "slider",
            "label": "Volume",
            "assertion": {
                "type": "numericRange",
                "expected": { "value": 0.3, "min": 0, "max": 1 },
                "tolerance": 0.000000001
            },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "textbox",
            "label": "Search",
            "assertion": {
                "type": "viewport",
                "tolerance": 0.5
            },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "heading",
            "label": "Settings",
            "assertion": {
                "type": "bounds",
                "expected": { "width": 120, "height": 32 },
                "tolerance": 0.5
            },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertByRole",
            "windowId": 1,
            "role": "textbox",
            "label": "Search",
            "assertion": {
                "type": "withinBounds",
                "expected": { "x": 0, "y": 0, "width": 900, "height": 600 },
                "tolerance": 0.5
            },
            "wait": { "timeout": 1000, "interval": 16 }
        },
        {
            "action": "assertWindowState",
            "windowId": 1,
            "expected": { "presence": "surface-released", "surfaceGeneration": 2 },
            "wait": { "timeout": 1000, "interval": 16 }
        }
    ]);

    assert_eq!(
        replay_actions_from_value(&json!({ "version": 1, "actions": actions.clone() }), None)
            .unwrap(),
        actions
    );
}

#[test]
fn replay_artifacts_use_a_separate_default_directory() {
    let test_dir = Path::new("target/wabou-test/gallery");
    assert_eq!(
        default_artifact_dir(test_dir, false),
        test_dir.join("artifacts")
    );
    assert_eq!(
        default_artifact_dir(test_dir, true),
        test_dir.join("replay-artifacts")
    );
}

#[test]
fn preparation_removes_only_stale_framework_outputs() {
    let directory = tempfile::tempdir().unwrap();
    for name in [
        "report.json",
        "report.json.tmp",
        "trace.json",
        "trace.json.tmp",
        "semantics.json",
        "semantics.json.tmp",
        "failure.png",
        "notes.txt",
    ] {
        fs::write(directory.path().join(name), name).unwrap();
    }

    prepare_artifact_dir(directory.path()).unwrap();

    for name in [
        "report.json",
        "report.json.tmp",
        "trace.json",
        "trace.json.tmp",
        "semantics.json",
        "semantics.json.tmp",
        "failure.png",
    ] {
        assert!(
            !directory.path().join(name).exists(),
            "stale {name} remains"
        );
    }
    assert_eq!(
        fs::read_to_string(directory.path().join("notes.txt")).unwrap(),
        "notes.txt"
    );
    prepare_artifact_dir(directory.path()).unwrap();
    prepare_artifact_dir(&directory.path().join("nested")).unwrap();
}
