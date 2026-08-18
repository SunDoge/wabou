use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Subcommand;
use serde_json::{Value, json};
use wabou_devtools::{DebugCaptureCase, call, discover_socket, empty_params, request};

use super::artifact::built_executable;
use super::config::{BuildProfile, bundle_path};
use super::frontend;
use super::project::load_app;
use super::{Result, ensure};

#[derive(Subcommand)]
pub(super) enum InspectCommand {
    Status,
    Query {
        query: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    Node {
        id: u32,
    },
    At {
        x: f32,
        y: f32,
    },
    Frames {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    Screenshot,
    Capture {
        #[arg(long)]
        x: Option<f32>,
        #[arg(long)]
        y: Option<f32>,
        #[arg(long, value_name = "DIR")]
        output: PathBuf,
    },
}

pub(super) fn run(workspace: &Path) -> Result<()> {
    ensure(command(workspace)?.status()?, "Wabou DevTools")
}

/// Resolve the GUI as a helper executable. Inside the Wabou source workspace we
/// build its frontend and host directly; installed CLIs find a sibling binary
/// first and then use PATH.
pub(super) fn command(workspace: &Path) -> Result<Command> {
    let source = workspace.join("apps/devtools");
    if source.join("Cargo.toml").is_file() && source.join("package.json").is_file() {
        let app = load_app(workspace, workspace, Some(&source))?;
        ensure(
            frontend::run(workspace, &app, "build", &[], BuildProfile::Debug, true)?,
            "DevTools Vite build",
        )?;
        let manifest = app.root.join("Cargo.toml");
        let mut cargo = Command::new("cargo");
        cargo
            .current_dir(workspace)
            .args(["build", "--manifest-path"])
            .arg(manifest);
        ensure(cargo.status()?, "DevTools Rust build")?;
        let executable = built_executable(workspace, &app, false)?;
        let mut command = Command::new(executable);
        command.env(
            "WABOU_BUNDLE_PATH",
            bundle_path(workspace, &app, BuildProfile::Debug)?,
        );
        return Ok(command);
    }

    let executable = find_helper(
        &env::current_exe()?,
        env::var_os("PATH").as_deref(),
        "wabou-devtools",
    )
    .ok_or(
        "wabou-devtools was not found next to wabou or on PATH; install the Wabou DevTools package",
    )?;
    Ok(Command::new(executable))
}

fn find_helper(current_exe: &Path, path: Option<&OsStr>, name: &str) -> Option<PathBuf> {
    let filename = format!("{name}{}", env::consts::EXE_SUFFIX);
    if let Some(parent) = current_exe.parent() {
        let sibling = parent.join(&filename);
        if sibling.is_file() {
            return Some(sibling);
        }
    }
    path.and_then(|path| {
        env::split_paths(path)
            .map(|dir| dir.join(&filename))
            .find(|candidate| candidate.is_file())
    })
}

pub(super) fn inspect(socket: Option<PathBuf>, command: InspectCommand) -> Result<()> {
    let path = socket.map_or_else(discover_socket, Ok)?;
    let capture_output = match &command {
        InspectCommand::Capture { output, .. } => Some(output.clone()),
        _ => None,
    };
    let (method, params): (&str, Value) = match command {
        InspectCommand::Status => ("status", empty_params()),
        InspectCommand::Query { query, limit } => {
            ("queryNodes", json!({ "query": query, "limit": limit }))
        }
        InspectCommand::Node { id } => ("inspectNode", json!({ "id": id })),
        InspectCommand::At { x, y } => ("inspectAtPoint", json!({ "x": x, "y": y })),
        InspectCommand::Frames { limit } => ("recentFrames", json!({ "limit": limit })),
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
    if let Some(output) = capture_output {
        let capture: DebugCaptureCase = serde_json::from_value(result)?;
        write_capture_case(&output, capture)?;
        println!("{}", output.display());
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }
    Ok(())
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
    fn sibling_helper_precedes_path() {
        let root = tempfile::tempdir().unwrap();
        let bin = root.path().join("bin");
        let path_bin = root.path().join("path-bin");
        fs::create_dir_all(&bin).unwrap();
        fs::create_dir_all(&path_bin).unwrap();
        let helper_name = format!("wabou-devtools{}", env::consts::EXE_SUFFIX);
        let sibling = bin.join(&helper_name);
        fs::write(&sibling, []).unwrap();
        fs::write(path_bin.join(&helper_name), []).unwrap();

        assert_eq!(
            find_helper(
                &bin.join(format!("wabou{}", env::consts::EXE_SUFFIX)),
                Some(path_bin.as_os_str()),
                "wabou-devtools",
            ),
            Some(sibling)
        );
    }

    #[test]
    fn helper_falls_back_to_path() {
        let root = tempfile::tempdir().unwrap();
        let path_bin = root.path().join("path-bin");
        fs::create_dir_all(&path_bin).unwrap();
        let helper = path_bin.join(format!("wabou-devtools{}", env::consts::EXE_SUFFIX));
        fs::write(&helper, []).unwrap();

        assert_eq!(
            find_helper(
                &root.path().join("missing/wabou"),
                Some(path_bin.as_os_str()),
                "wabou-devtools",
            ),
            Some(helper)
        );
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
                    id: 42,
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
}
