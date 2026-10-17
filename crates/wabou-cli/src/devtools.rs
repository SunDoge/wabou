use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde_json::{Value, json};
use wabou_devtools::{DebugCaptureCase, NodeKey, call, discover_socket, empty_params, request};

use super::Result;

#[derive(usage_rs::Subcommands)]
pub(super) enum InspectCommand {
    Status,
    /// Start/stop bounded performance recording (clears previous records).
    ProfileMode {
        #[usage(choices("off", "counters", "trace"))]
        mode: String,
        #[usage(long, default = "16384")]
        capacity: usize,
    },
    /// Export a Perfetto-compatible timeline and duration summaries.
    Profile {
        #[usage(long)]
        output: Option<PathBuf>,
    },
    Query {
        query: String,
        #[usage(long, default = "100")]
        limit: usize,
    },
    Node {
        /// Node key as `slot:generation`; bare slots use generation 1.
        id: NodeKeyArg,
    },
    At {
        x: f32,
        y: f32,
    },
    Frames {
        #[usage(long, default = "20")]
        limit: usize,
    },
    /// Validate the latest retained snapshot for broken framework invariants.
    Validate,
    Screenshot,
    Capture {
        #[usage(long)]
        x: Option<f32>,
        #[usage(long)]
        y: Option<f32>,
        #[usage(long, value_name = "DIR")]
        output: PathBuf,
    },
}

#[derive(Clone, Copy, Debug)]
pub(super) struct NodeKeyArg(NodeKey);

impl FromStr for NodeKeyArg {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let (lo, hi) = value.split_once(':').unwrap_or((value, "1"));
        let key = NodeKey::new(
            lo.parse().map_err(|_| {
                format!("invalid node key `{value}`; expected SLOT or SLOT:GENERATION")
            })?,
            hi.parse().map_err(|_| {
                format!("invalid node key `{value}`; expected SLOT or SLOT:GENERATION")
            })?,
        );
        if !key.is_valid() {
            return Err(format!(
                "invalid node key `{value}`; slot must be non-zero and generation must be non-zero and odd"
            ));
        }
        Ok(Self(key))
    }
}

pub(super) fn inspect(socket: Option<PathBuf>, command: InspectCommand) -> Result<()> {
    let path = socket.map_or_else(discover_socket, Ok)?;
    let profile_output = match &command {
        InspectCommand::Profile { output } => output.clone(),
        _ => None,
    };
    let validate_snapshot = matches!(&command, InspectCommand::Validate);
    let capture_output = match &command {
        InspectCommand::Capture { output, .. } => Some(output.clone()),
        _ => None,
    };
    let (method, params): (&str, Value) = match command {
        InspectCommand::ProfileMode { mode, capacity } => {
            ("profileConfigure", json!({"mode":mode,"capacity":capacity}))
        }
        InspectCommand::Profile { .. } => ("profileSnapshot", empty_params()),
        InspectCommand::Status => ("status", empty_params()),
        InspectCommand::Query { query, limit } => {
            ("queryNodes", json!({ "query": query, "limit": limit }))
        }
        InspectCommand::Node { id } => ("inspectNode", json!({ "id": id.0 })),
        InspectCommand::At { x, y } => ("inspectAtPoint", json!({ "x": x, "y": y })),
        InspectCommand::Frames { limit } => ("recentFrames", json!({ "limit": limit })),
        InspectCommand::Validate => ("validateSnapshot", empty_params()),
        InspectCommand::Screenshot => ("captureScreenshot", empty_params()),
        InspectCommand::Capture { x, y, .. } => {
            let params = match (x, y) {
                (Some(x), Some(y)) => json!({ "x": x, "y": y }),
                (None, None) => empty_params(),
                _ => return Err("--x and --y must be provided together".into()),
            };
            ("captureCase", params)
        }
    };
    let response = call(&path, &request(1, method, params))
        .map_err(|error| format!("cannot connect to {}: {error}", path.display()))?;
    if let Some(error) = response.error {
        return Err(error.into());
    }
    let result = response.result.unwrap_or(Value::Null);
    let validation_error = validate_snapshot
        .then(|| snapshot_validation_error(&result))
        .flatten();
    if let Some(output) = profile_output {
        fs::write(&output, serde_json::to_vec_pretty(&result)?)?;
        println!("{}", output.display());
    } else if let Some(output) = capture_output {
        let capture: DebugCaptureCase = serde_json::from_value(result)?;
        write_capture_case(&output, capture)?;
        println!("{}", output.display());
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    if let Some(error) = validation_error {
        return Err(error.into());
    }
    Ok(())
}

fn snapshot_validation_error(result: &Value) -> Option<String> {
    (result["valid"] == false).then(|| {
        format!(
            "snapshot validation failed with {} errors",
            result["errorCount"].as_u64().unwrap_or(0)
        )
    })
}

fn write_capture_case(output: &Path, mut capture: DebugCaptureCase) -> Result<()> {
    fs::create_dir_all(output)?;
    let screenshot = output.join("screenshot.png");
    fs::copy(&capture.screenshot_path, screenshot)?;
    capture.screenshot_path = PathBuf::from("screenshot.png");
    fs::write(
        output.join("manifest.json"),
        serde_json::to_vec_pretty(&capture)?,
    )?;
    fs::write(
        output.join("tree.json"),
        serde_json::to_vec_pretty(&capture.snapshot)?,
    )?;
    if let Some(node) = capture.point.as_ref().and_then(|point| point.node.as_ref()) {
        fs::write(
            output.join("selected-node.json"),
            serde_json::to_vec_pretty(node)?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use wabou_devtools::{DebugNode, DebugPointInspection, DebugSnapshot};

    use super::*;

    #[test]
    fn node_key_argument_preserves_the_full_generation() {
        assert_eq!(
            serde_json::to_value(NodeKeyArg::from_str("42:3").unwrap().0).unwrap(),
            json!({"lo": 42, "hi": 3})
        );
        assert_eq!(NodeKeyArg::from_str("42").unwrap().0, NodeKey::new(42, 1));
        for invalid in ["0", "42:0", "42:2", "x:1", "1:x", "1:3:5"] {
            assert!(NodeKeyArg::from_str(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn capture_writer_emits_frame_matched_bundle() {
        let parent = tempfile::tempdir().unwrap();
        let output = parent.path().join("capture");
        let source = parent.path().join("source.png");
        fs::write(&source, b"png").unwrap();
        let capture = DebugCaptureCase {
            screenshot_path: source,
            snapshot: DebugSnapshot::default(),
            point: Some(DebugPointInspection {
                x: 1.0,
                y: 2.0,
                node: Some(DebugNode {
                    id: wabou_devtools::NodeKey::new(42, 1),
                    ..Default::default()
                }),
                ancestors: Vec::new(),
            }),
            frames: Vec::new(),
        };

        write_capture_case(&output, capture).unwrap();

        assert_eq!(fs::read(output.join("screenshot.png")).unwrap(), b"png");
        assert!(output.join("manifest.json").is_file());
        assert!(output.join("tree.json").is_file());
        assert!(output.join("selected-node.json").is_file());
    }

    #[test]
    fn validation_exit_status_ignores_warnings_but_rejects_errors() {
        assert_eq!(
            snapshot_validation_error(&json!({"valid": true, "warningCount": 2})),
            None
        );
        assert_eq!(
            snapshot_validation_error(&json!({"valid": false, "errorCount": 3})),
            Some("snapshot validation failed with 3 errors".to_owned())
        );
    }
}
