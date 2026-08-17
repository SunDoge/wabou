//! Behavior-test artifact preparation and replay validation.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::Result;

#[cfg(test)]
mod tests;

pub(super) fn default_artifact_dir(test_dir: &Path, replaying: bool) -> PathBuf {
    test_dir.join(if replaying {
        "replay-artifacts"
    } else {
        "artifacts"
    })
}

pub(super) fn prepare_artifact_dir(directory: &Path) -> Result<()> {
    fs::create_dir_all(directory)?;
    for name in [
        "report.json",
        "report.json.tmp",
        "trace.json",
        "trace.json.tmp",
        "semantics.json",
        "semantics.json.tmp",
        "failure.png",
    ] {
        let artifact = directory.join(name);
        match fs::remove_file(&artifact) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(format!(
                    "cannot remove stale artifact {}: {error}",
                    artifact.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all_fields = "camelCase", deny_unknown_fields)]
enum ReplayAction {
    #[serde(rename = "nativeClose")]
    NativeClose {
        window_id: u64,
        platform: ReplayPlatform,
    },
    #[serde(rename = "showWindow")]
    ShowWindow { window_id: u64 },
    #[serde(rename = "clickByRole")]
    ClickByRole {
        window_id: u64,
        role: ReplayRole,
        label: String,
        #[serde(default)]
        index: Option<u64>,
        #[serde(default)]
        wait: Option<ReplayWait>,
    },
    #[serde(rename = "inputByRole")]
    InputByRole {
        window_id: u64,
        role: ReplayRole,
        label: String,
        #[serde(default)]
        index: Option<u64>,
        input: ReplayInput,
        #[serde(default)]
        wait: Option<ReplayWait>,
    },
    #[serde(rename = "waitForByRole")]
    WaitForByRole {
        window_id: u64,
        role: ReplayRole,
        label: String,
        #[serde(default)]
        index: Option<u64>,
        wait: ReplayWait,
    },
    #[serde(rename = "assertByRole")]
    AssertByRole {
        window_id: u64,
        role: ReplayRole,
        label: String,
        #[serde(default)]
        index: Option<u64>,
        assertion: ReplayLocatorAssertion,
        wait: ReplayWait,
    },
    #[serde(rename = "assertWindowState")]
    AssertWindowState {
        window_id: u64,
        expected: ReplayWindowState,
        wait: ReplayWait,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ReplayPlatform {
    Wayland,
    MutableVisibility,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ReplayRole {
    Button,
    Group,
    Textbox,
    Link,
    Dialog,
    Alert,
    Status,
    Checkbox,
    Radio,
    Switch,
    Combobox,
    Listbox,
    Option,
    Menu,
    Menuitem,
    Tree,
    Treeitem,
    Table,
    Row,
    Cell,
    Columnheader,
    Rowheader,
    Slider,
    Progressbar,
    Heading,
    Label,
    Img,
    Radiogroup,
    Tablist,
    Tab,
    Tabpanel,
    Grid,
    Gridcell,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayWait {
    timeout: f64,
    interval: f64,
}

#[derive(Debug, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum ReplayInput {
    Probe,
    Drag { delta_x: f64, delta_y: f64 },
    Key { key: String, modifiers: u8 },
    Text { text: String },
    Paste { text: String },
    Ime { text: String },
    Wheel { delta_x: f64, delta_y: f64 },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum ReplayLocatorAssertion {
    Absent,
    Count {
        expected: u64,
    },
    Text {
        expected: String,
    },
    Value {
        expected: String,
    },
    NumericRange {
        expected: ReplayNumericRange,
        tolerance: f64,
    },
    Disabled {
        expected: bool,
    },
    Checked {
        expected: ReplayToggleState,
    },
    Selected {
        expected: bool,
    },
    Expanded {
        expected: bool,
    },
    Pressed {
        expected: bool,
    },
    Focused {
        expected: bool,
    },
    Bounds {
        expected: ReplayBounds,
        tolerance: f64,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReplayBounds {
    #[serde(default)]
    x: Option<f64>,
    #[serde(default)]
    y: Option<f64>,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    height: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReplayNumericRange {
    #[serde(default)]
    value: Option<f64>,
    #[serde(default)]
    min: Option<f64>,
    #[serde(default)]
    max: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ReplayToggleState {
    Boolean(bool),
    Mixed(ReplayMixedState),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ReplayMixedState {
    Mixed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReplayWindowState {
    presence: ReplayWindowPresence,
    surface_generation: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum ReplayWindowPresence {
    Visible,
    Hidden,
    SurfaceReleased,
    Closed,
}

const MAX_SAFE_JAVASCRIPT_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_LOCATOR_INDEX: u64 = u32::MAX as u64;

fn validate_index(index: Option<u64>) -> std::result::Result<(), String> {
    if index.is_some_and(|index| index > MAX_LOCATOR_INDEX) {
        return Err(format!("index must not exceed {MAX_LOCATOR_INDEX}"));
    }
    Ok(())
}

impl ReplayAction {
    fn validate(&self) -> std::result::Result<(), String> {
        let (window_id, wait) = match self {
            Self::NativeClose {
                window_id,
                platform,
            } => {
                let _ = platform;
                (*window_id, None)
            }
            Self::ShowWindow { window_id } => (*window_id, None),
            Self::ClickByRole {
                window_id,
                role,
                label,
                index,
                wait,
            } => {
                let _ = (role, label);
                validate_index(*index)?;
                (*window_id, wait.as_ref())
            }
            Self::InputByRole {
                window_id,
                role,
                label,
                index,
                input,
                wait,
            } => {
                let _ = (role, label);
                validate_index(*index)?;
                input.validate()?;
                (*window_id, wait.as_ref())
            }
            Self::WaitForByRole {
                window_id,
                role,
                label,
                index,
                wait,
            } => {
                let _ = (role, label);
                validate_index(*index)?;
                (*window_id, Some(wait))
            }
            Self::AssertByRole {
                window_id,
                role,
                label,
                index,
                assertion,
                wait,
            } => {
                let _ = (role, label);
                validate_index(*index)?;
                if index.is_some() && matches!(assertion, ReplayLocatorAssertion::Count { .. }) {
                    return Err("count assertion requires an unindexed locator".into());
                }
                assertion.validate()?;
                (*window_id, Some(wait))
            }
            Self::AssertWindowState {
                window_id,
                expected,
                wait,
            } => {
                expected.validate()?;
                (*window_id, Some(wait))
            }
        };
        if window_id == 0 || window_id > MAX_SAFE_JAVASCRIPT_INTEGER {
            return Err(format!(
                "windowId must be an integer between 1 and {MAX_SAFE_JAVASCRIPT_INTEGER}"
            ));
        }
        if let Some(wait) = wait {
            wait.validate()?;
        }
        Ok(())
    }
}

impl ReplayWait {
    fn validate(&self) -> std::result::Result<(), String> {
        for (name, value) in [
            ("wait.timeout", self.timeout),
            ("wait.interval", self.interval),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("{name} must be a finite non-negative number"));
            }
        }
        Ok(())
    }
}

impl ReplayInput {
    fn validate(&self) -> std::result::Result<(), String> {
        match self {
            Self::Probe => {}
            Self::Drag { delta_x, delta_y } | Self::Wheel { delta_x, delta_y } => {
                if !delta_x.is_finite() || !delta_y.is_finite() {
                    return Err("input deltas must be finite numbers".into());
                }
            }
            Self::Key { key, modifiers } => {
                if key.is_empty() {
                    return Err("input key must be a non-empty string".into());
                }
                if modifiers & !0b1111 != 0 {
                    return Err("input modifiers contain unknown bits".into());
                }
            }
            Self::Text { text } | Self::Paste { text } | Self::Ime { text } => {
                let _ = text;
            }
        }
        Ok(())
    }
}

impl ReplayLocatorAssertion {
    fn validate(&self) -> std::result::Result<(), String> {
        match self {
            Self::Absent => {}
            Self::Count { expected } => {
                if *expected > MAX_SAFE_JAVASCRIPT_INTEGER {
                    return Err(format!(
                        "locator count must not exceed {MAX_SAFE_JAVASCRIPT_INTEGER}"
                    ));
                }
            }
            Self::Text { expected } | Self::Value { expected } => {
                let _ = expected;
            }
            Self::NumericRange {
                expected,
                tolerance,
            } => {
                let values = [expected.value, expected.min, expected.max];
                if values.iter().all(Option::is_none) {
                    return Err("numeric range assertion expected object cannot be empty".into());
                }
                if values.into_iter().flatten().any(|value| !value.is_finite()) {
                    return Err("numeric range assertion values must be finite numbers".into());
                }
                if !tolerance.is_finite() || *tolerance < 0.0 {
                    return Err(
                        "numeric range assertion tolerance must be a finite non-negative number"
                            .into(),
                    );
                }
            }
            Self::Disabled { expected }
            | Self::Selected { expected }
            | Self::Expanded { expected }
            | Self::Pressed { expected }
            | Self::Focused { expected } => {
                let _ = expected;
            }
            Self::Checked { expected } => match expected {
                ReplayToggleState::Boolean(value) => {
                    let _ = value;
                }
                ReplayToggleState::Mixed(value) => {
                    let _ = value;
                }
            },
            Self::Bounds {
                expected,
                tolerance,
            } => {
                let values = [expected.x, expected.y, expected.width, expected.height];
                if values.iter().all(Option::is_none) {
                    return Err("bounds assertion expected object cannot be empty".into());
                }
                if values.into_iter().flatten().any(|value| !value.is_finite()) {
                    return Err("bounds assertion values must be finite numbers".into());
                }
                if !tolerance.is_finite() || *tolerance < 0.0 {
                    return Err(
                        "bounds assertion tolerance must be a finite non-negative number".into(),
                    );
                }
            }
        }
        Ok(())
    }
}

impl ReplayWindowState {
    fn validate(&self) -> std::result::Result<(), String> {
        let _ = self.presence;
        if self.surface_generation > MAX_SAFE_JAVASCRIPT_INTEGER {
            return Err(format!(
                "expected.surfaceGeneration must not exceed {MAX_SAFE_JAVASCRIPT_INTEGER}"
            ));
        }
        Ok(())
    }
}

fn validate_actions(actions: Vec<Value>) -> Result<Value> {
    for (index, action) in actions.iter().enumerate() {
        let parsed = serde_json::from_value::<ReplayAction>(action.clone())
            .map_err(|error| format!("invalid replay action {index}: {error}"))?;
        parsed
            .validate()
            .map_err(|error| format!("invalid replay action {index}: {error}"))?;
    }
    Ok(Value::Array(actions))
}

pub(super) fn replay_actions(path: &Path, test_name: Option<&str>) -> Result<Value> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("cannot read replay input {}: {error}", path.display()))?;
    let parsed: Value = serde_json::from_str(&contents)
        .map_err(|error| format!("invalid replay input {}: {error}", path.display()))?;

    replay_actions_from_value(&parsed, test_name)
}

fn replay_actions_from_value(parsed: &Value, test_name: Option<&str>) -> Result<Value> {
    if let Some(actions) = parsed.as_array() {
        if test_name.is_some() {
            return Err("--replay-test requires report.json, not a bare trace array".into());
        }
        return validate_actions(actions.clone());
    }

    let report = parsed
        .as_object()
        .ok_or("replay input must be a trace array or test report object")?;
    if let Some(version) = report.get("version") {
        let version = version
            .as_u64()
            .ok_or("replay artifact version must be a positive integer")?;
        if version != 1 {
            return Err(
                format!("unsupported replay artifact version {version}; expected 1").into(),
            );
        }
    }
    if let Some(actions) = report.get("actions").and_then(Value::as_array) {
        if test_name.is_some() {
            return Err("--replay-test requires report.json, not trace.json".into());
        }
        return validate_actions(actions.clone());
    }
    let trace = report
        .get("trace")
        .and_then(Value::as_array)
        .ok_or("replay artifact does not contain an actions or trace array")?;
    let Some(test_name) = test_name else {
        return validate_actions(trace.clone());
    };
    let tests = report
        .get("tests")
        .and_then(Value::as_array)
        .ok_or("test report does not contain a tests array")?;
    let matching = tests
        .iter()
        .filter(|test| test.get("name").and_then(Value::as_str) == Some(test_name))
        .collect::<Vec<_>>();
    let test = match matching.as_slice() {
        [] => return Err(format!("test report has no test named {test_name:?}").into()),
        [test] => *test,
        _ => return Err(format!("test report contains duplicate test name {test_name:?}").into()),
    };
    let trace_start = match test.get("traceStart") {
        Some(value) => value
            .as_u64()
            .and_then(|start| usize::try_from(start).ok())
            .ok_or_else(|| format!("test {test_name:?} has an invalid traceStart"))?,
        // Reports written before trace ranges were introduced only carried
        // traceEnd. Their selected-test replay already had prefix semantics.
        None => 0,
    };
    let trace_end = test
        .get("traceEnd")
        .and_then(Value::as_u64)
        .and_then(|end| usize::try_from(end).ok())
        .ok_or_else(|| format!("test {test_name:?} has an invalid traceEnd"))?;
    if trace_start > trace_end {
        return Err(format!(
            "test {test_name:?} traceStart {trace_start} exceeds traceEnd {trace_end}"
        )
        .into());
    }
    if trace_end > trace.len() {
        return Err(format!(
            "test {test_name:?} traceEnd {trace_end} exceeds trace length {}",
            trace.len()
        )
        .into());
    }

    // Tests in one scenario share a running application. Replaying the prefix
    // reconstructs state established by earlier tests before the selected
    // test's final recorded action.
    validate_actions(trace[..trace_end].to_vec())
}
