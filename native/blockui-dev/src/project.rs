//! Cargo-based discovery shared by every developer entry point.
use serde_json::Value;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
fn discovery_output(
    command: &mut Command,
    operation: &str,
) -> Result<std::process::Output, String> {
    let program = command.get_program().to_owned();
    let output = command
        .output()
        .map_err(|error| format!("Cargo {operation} via {program:?} could not start: {error}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim()
        } else {
            stderr.trim()
        };
        return Err(format!(
            "Cargo {operation} via {program:?} failed ({}): {}",
            output.status, detail
        ));
    }
    Ok(output)
}

/// Use one explicitly selected Cargo for discovery, builds and native watching.
/// The override is an executable path/name, never a shell command or arguments.
pub fn cargo_command() -> Command {
    let selected = std::env::var_os("BLOCKUI_CARGO").filter(|path| !path.is_empty());
    let mut command = if let Some(program) = selected {
        let program = absolute_program(program);
        let mut command = Command::new(&program);
        command.env("BLOCKUI_CARGO", program);
        command
    } else {
        Command::new("cargo")
    };
    compiler_environment(&mut command);
    command
}
fn absolute_program(mut program: std::ffi::OsString) -> std::ffi::OsString {
    let path = Path::new(&program);
    if path.is_relative()
        && path.components().count() > 1
        && let Ok(directory) = std::env::current_dir()
    {
        program = directory.join(path).into_os_string();
    }
    program
}
/// Preserve explicit compiler paths across child working-directory changes.
pub fn compiler_environment(command: &mut Command) {
    for name in ["RUSTC", "CARGO_BUILD_RUSTC"] {
        if let Some(program) = std::env::var_os(name).filter(|value| !value.is_empty()) {
            command.env(name, absolute_program(program));
        }
    }
}
/// Match Cargo's explicit compiler environment selection for host probing.
pub fn rustc_command() -> Command {
    let program = std::env::var_os("RUSTC")
        .filter(|value| !value.is_empty())
        .or_else(|| std::env::var_os("CARGO_BUILD_RUSTC").filter(|value| !value.is_empty()))
        .unwrap_or_else(|| "rustc".into());
    Command::new(absolute_program(program))
}
pub fn checked(command: &mut Command) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|e| format!("cannot run {command:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed ({status}): {command:?}"))
    }
}
/// Cargo must prune target-specific edges before computing native watch roots.
pub fn native_platform() -> Result<String, String> {
    static HOST: std::sync::OnceLock<Result<String, String>> = std::sync::OnceLock::new();
    HOST.get_or_init(|| {
        let mut command = rustc_command();
        let compiler = command.get_program().to_string_lossy().into_owned();
        let output = command
            .arg("-vV")
            .output()
            .map_err(|error| format!("cannot probe Rust host target with {compiler:?}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "cannot probe Rust host target with {compiler:?} ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find_map(|line| {
                line.strip_prefix("host: ")
                    .filter(|host| !host.trim().is_empty())
                    .map(str::to_owned)
            })
            .ok_or_else(|| {
                format!("Rust compiler {compiler:?} did not report a host target in -vV output")
            })
    })
    .clone()
}

// Cargo validates the selected manifest and workspace; avoid starting another
// Cargo process just to find the nearest ordinary manifest on disk.
fn ancestor_manifest(directory: &Path) -> Option<PathBuf> {
    directory
        .ancestors()
        .map(|parent| parent.join("Cargo.toml"))
        .find(|manifest| manifest.is_file())
}

pub struct Project {
    pub metadata: Value,
    pub package: Value,
    pub manifest: PathBuf,
    pub root: PathBuf,
    pub target: PathBuf,
}
impl Project {
    pub fn discover(
        app: Option<&str>,
        manifest: Option<&Path>,
        target: Option<&Path>,
        features: &str,
        web: bool,
    ) -> Result<Self, String> {
        Self::discover_impl(app, manifest, target, features, web, true)
    }
    /// Tests and native checks only need application/workspace metadata, not a host graph.
    pub fn discover_for_tests(
        app: Option<&str>,
        manifest: Option<&Path>,
        target: Option<&Path>,
        features: &str,
    ) -> Result<Self, String> {
        Self::discover_impl(app, manifest, target, features, false, false)
    }
    fn discover_impl(
        app: Option<&str>,
        manifest: Option<&Path>,
        target: Option<&Path>,
        features: &str,
        web: bool,
        full_graph: bool,
    ) -> Result<Self, String> {
        let path = app.map(Path::new).filter(|p| {
            // A supplied manifest selects the workspace; a bare package name
            // must not turn into a path merely because a sibling directory has
            // that name (including the launcher's own package selection).
            let bare_name = p.components().count() == 1
                && matches!(p.components().next(), Some(std::path::Component::Normal(_)));
            (manifest.is_none() || !bare_name || p.is_file()) && (p.is_dir() || p.is_file())
        });
        if path.is_some() && manifest.is_some() {
            return Err("use either an app path or --manifest-path".into());
        }
        let manifest = if let Some(path) = path {
            if path.is_dir() {
                path.join("Cargo.toml")
            } else {
                path.to_owned()
            }
        } else if let Some(path) = manifest {
            path.to_owned()
        } else if let Some(manifest) = std::env::current_dir()
            .ok()
            .and_then(|directory| ancestor_manifest(&directory))
        {
            manifest
        } else {
            let output = discovery_output(
                cargo_command().args(["locate-project", "--message-format", "plain"]),
                "locate-project",
            )?;
            PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
        };
        let manifest = manifest.canonicalize().map_err(|error| {
            format!("cannot resolve application manifest {manifest:?}: {error}")
        })?;
        let mut command = cargo_command();
        command
            .args(["metadata", "--format-version", "1", "--manifest-path"])
            .arg(&manifest);
        if !features.is_empty() {
            command.args(["--features", features]);
        }
        if full_graph {
            command.arg("--filter-platform").arg(if web {
                "wasm32-unknown-unknown".to_owned()
            } else {
                native_platform()?
            });
        } else {
            command.arg("--no-deps");
        }
        let output = discovery_output(&mut command, "metadata")?;
        let metadata: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
            format!(
                "Cargo metadata via {:?} returned invalid JSON for {manifest:?}: {error}",
                command.get_program()
            )
        })?;
        let packages = metadata["packages"].as_array().ok_or("missing packages")?;
        let named = app.filter(|_| path.is_none());
        let package = if let Some(name) = named {
            packages.iter().find(|p| p["name"].as_str() == Some(name))
        } else {
            packages
                .iter()
                .find(|p| p["manifest_path"].as_str() == manifest.to_str())
                .or_else(|| {
                    metadata["metadata"]["blockui"][if web {
                        "default-web-app"
                    } else {
                        "default-app"
                    }]
                    .as_str()
                    .and_then(|name| packages.iter().find(|p| p["name"] == name))
                })
        }
        .ok_or_else(|| match named {
            Some(name) => format!("application package {name:?} was not found in Cargo metadata for {manifest:?}; check the package name in Cargo.toml"),
            None => "select an application package or directory (this manifest has no default BlockUI app)".into(),
        })?
        .clone();
        let root = PathBuf::from(
            metadata["workspace_root"]
                .as_str()
                .ok_or("missing workspace root")?,
        );
        let target = match target {
            Some(path) if path.is_absolute() => path.to_owned(),
            Some(path) => std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(path),
            None => PathBuf::from(
                metadata["target_directory"]
                    .as_str()
                    .ok_or("missing target directory")?,
            ),
        };
        Ok(Self {
            metadata,
            package,
            manifest,
            root,
            target,
        })
    }
    pub fn name(&self) -> &str {
        self.package["name"].as_str().unwrap()
    }
    pub fn package_root(&self) -> &Path {
        Path::new(self.package["manifest_path"].as_str().unwrap())
            .parent()
            .unwrap()
    }
    pub fn cargo(&self, operation: &str, features: &str) -> Command {
        let mut command = cargo_command();
        command
            .current_dir(&self.root)
            .arg(operation)
            .arg("--manifest-path")
            .arg(&self.manifest)
            .args(["-p", self.name(), "--target-dir"])
            .arg(&self.target);
        if !features.is_empty() {
            command.args(["--features", features]);
        }
        command
    }
    pub fn sdk_root(&self) -> Result<PathBuf, String> {
        self.metadata["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == "blockui")
            .and_then(|p| p["manifest_path"].as_str())
            .map(Path::new)
            .and_then(Path::parent)
            .and_then(Path::parent)
            .and_then(Path::parent)
            .map(Path::to_owned)
            .ok_or_else(|| "application does not depend on the BlockUI SDK".into())
    }
}
