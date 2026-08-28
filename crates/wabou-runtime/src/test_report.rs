use std::path::{Path, PathBuf};

use crate::test_driver::TestController;

pub(super) fn finish_test_report(controller: TestController) -> crate::Result<()> {
    let report = controller
        .take_report()
        .ok_or_else(|| crate::Error::TestScenario {
            message: "host exited or timed out before the scenario reported a result".into(),
        })?;
    let mut value = serde_json::from_str::<serde_json::Value>(&report).map_err(|error| {
        crate::Error::TestScenario {
            message: format!("scenario returned invalid JSON: {error}"),
        }
    })?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| crate::Error::TestScenario {
            message: "scenario returned an unversioned test report".into(),
        })?;
    if version != 1 {
        return Err(crate::Error::TestScenario {
            message: format!("unsupported test report version {version}; expected 1"),
        });
    }
    let headless = std::env::var("WABOU_TEST_HEADLESS").is_ok_and(|value| value != "0");
    value
        .as_object_mut()
        .ok_or_else(|| crate::Error::TestScenario {
            message: "scenario returned a non-object test report".into(),
        })?
        .insert("environment".into(), test_environment(headless));
    let passed = value
        .get("passed")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let artifact_directory = std::env::var_os("WABOU_TEST_ARTIFACT_DIR").map(PathBuf::from);
    if let Some(directory) = artifact_directory.as_deref() {
        std::fs::create_dir_all(directory).map_err(|error| crate::Error::TestScenario {
            message: format!(
                "cannot create artifact directory {}: {error}",
                directory.display()
            ),
        })?;
        write_test_artifact(directory, "report.json", &format!("{value:#}\n"))?;
        if let Some(trace) = value.get("trace") {
            let artifact = test_trace_artifact(trace);
            write_test_artifact(directory, "trace.json", &format!("{artifact:#}\n"))?;
        }
        if !passed {
            let semantics = controller.semantic_artifact();
            write_test_artifact(directory, "semantics.json", &format!("{semantics:#}\n"))?;
        }
    }
    let summary = test_report_summary(&value, artifact_directory.as_deref());
    if !passed {
        return Err(crate::Error::TestScenario { message: summary });
    }
    println!("{summary}");
    Ok(())
}

pub(super) fn write_test_artifact(
    directory: &Path,
    name: &str,
    contents: &str,
) -> crate::Result<()> {
    let destination = directory.join(name);
    let temporary = directory.join(format!("{name}.tmp"));
    std::fs::write(&temporary, contents).map_err(|error| crate::Error::TestScenario {
        message: format!(
            "cannot write temporary test artifact {}: {error}",
            temporary.display()
        ),
    })?;
    if let Err(error) = std::fs::rename(&temporary, &destination) {
        let _ = std::fs::remove_file(&temporary);
        return Err(crate::Error::TestScenario {
            message: format!(
                "cannot publish test artifact {}: {error}",
                destination.display()
            ),
        });
    }
    Ok(())
}

pub(super) fn test_trace_artifact(trace: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "actions": trace,
    })
}

pub(super) fn test_environment(headless: bool) -> serde_json::Value {
    serde_json::json!({
        "backend": if headless { "deterministic" } else { "native" },
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "wabouVersion": env!("CARGO_PKG_VERSION"),
    })
}

pub(super) fn test_report_summary(
    value: &serde_json::Value,
    artifact_directory: Option<&Path>,
) -> String {
    let tests = value
        .get("tests")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let passed = tests
        .iter()
        .filter(|test| test.get("passed").and_then(serde_json::Value::as_bool) == Some(true))
        .count();
    let failed = tests.len().saturating_sub(passed);
    let actions = value
        .get("trace")
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let duration_ms = tests
        .iter()
        .filter_map(|test| test.get("durationMs").and_then(serde_json::Value::as_f64))
        .sum::<f64>();
    let mut summary = format!(
        "test result: {}. {passed} passed; {failed} failed; {actions} actions; {duration_ms:.1}ms",
        if failed == 0 { "ok" } else { "FAILED" }
    );
    for test in tests
        .iter()
        .filter(|test| test.get("passed").and_then(serde_json::Value::as_bool) != Some(true))
    {
        let name = test
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("test");
        summary.push_str(&format!("\n\n---- {name} ----"));
        if let Some(error) = test.get("error").and_then(serde_json::Value::as_str) {
            summary.push('\n');
            summary.push_str(error);
        }
    }
    if failed > 0
        && let Some(environment) = value.get("environment")
    {
        let backend = environment
            .get("backend")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let os = environment
            .get("os")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let arch = environment
            .get("arch")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let version = environment
            .get("wabouVersion")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        summary.push_str(&format!(
            "\nenvironment: {backend}; {os}/{arch}; wabou {version}"
        ));
    }
    if let Some(directory) = artifact_directory {
        summary.push_str(&format!("\nartifacts: {}", directory.display()));
    }
    summary
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{test_environment, test_report_summary, test_trace_artifact, write_test_artifact};

    #[test]
    fn summary_keeps_diagnostics_but_omits_action_payloads() {
        let report = serde_json::json!({
            "environment": {
                "backend": "deterministic",
                "os": "linux",
                "arch": "x86_64",
                "wabouVersion": "0.1.0-test",
            },
            "passed": false,
            "tests": [
                {
                    "name": "opens dialog",
                    "passed": true,
                    "durationMs": 12.25,
                },
                {
                    "name": "submits form",
                    "passed": false,
                    "durationMs": 7.75,
                    "error": "expected Save to be enabled\n    at form.tsx:42:3",
                },
            ],
            "trace": [
                { "action": "clickByRole", "label": "private action payload" },
                { "action": "inputByRole", "input": { "text": "secret" } },
            ],
        });

        let summary = test_report_summary(&report, Some(Path::new("/tmp/artifacts")));
        assert!(summary.contains("test result: FAILED. 1 passed; 1 failed; 2 actions; 20.0ms"));
        assert!(summary.contains("---- submits form ----"));
        assert!(summary.contains("at form.tsx:42:3"));
        assert!(summary.contains("environment: deterministic; linux/x86_64; wabou 0.1.0-test"));
        assert!(summary.contains("artifacts: /tmp/artifacts"));
        assert!(!summary.contains("private action payload"));
        assert!(!summary.contains("secret"));
    }

    #[test]
    fn standalone_trace_has_an_explicit_schema_version() {
        let trace = serde_json::json!([{ "action": "showWindow", "windowId": 1 }]);
        assert_eq!(
            test_trace_artifact(&trace),
            serde_json::json!({
                "version": 1,
                "actions": trace,
            })
        );
    }

    #[test]
    fn artifacts_are_published_without_leaving_the_temporary_file() {
        let directory = tempfile::tempdir().unwrap();
        write_test_artifact(directory.path(), "report.json", "{\"passed\":true}\n").unwrap();
        assert_eq!(
            std::fs::read_to_string(directory.path().join("report.json")).unwrap(),
            "{\"passed\":true}\n"
        );
        assert!(!directory.path().join("report.json.tmp").exists());
    }

    #[test]
    fn environment_distinguishes_deterministic_and_native_runs() {
        let deterministic = test_environment(true);
        let native = test_environment(false);
        assert_eq!(deterministic["backend"], "deterministic");
        assert_eq!(native["backend"], "native");
        assert_eq!(deterministic["os"], std::env::consts::OS);
        assert_eq!(deterministic["arch"], std::env::consts::ARCH);
        assert_eq!(deterministic["wabouVersion"], env!("CARGO_PKG_VERSION"));
    }
}
