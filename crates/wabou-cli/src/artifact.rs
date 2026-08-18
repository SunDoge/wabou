use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

use super::{App, Result};

pub(super) fn built_executable(workspace: &Path, app: &App, release: bool) -> Result<PathBuf> {
    let metadata = cargo_metadata(workspace, app)?;
    artifact_from_metadata(
        &metadata,
        &app.root.join("Cargo.toml").canonicalize()?,
        release,
    )
    .map(|(path, _)| path)
}

pub(super) fn app_binary(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    binary_target(&metadata, &manifest_path)
        .and_then(|target| target["name"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| "application binary target has no name".into())
}

pub(super) fn app_vite_feature(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    vite_feature(&metadata, &manifest_path)
        .map(str::to_owned)
        .ok_or_else(|| "application must depend on `wabou` or `wabou-runtime`".into())
}

pub(super) fn app_profiling_feature(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    framework_feature(&metadata, &manifest_path, "profiling")
        .ok_or_else(|| "application must depend on `wabou` or `wabou-runtime`".into())
}

pub(super) fn app_bindings_target(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    bindings_target(&metadata, &manifest_path)
        .and_then(|target| target["name"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| {
            "application must define one example sourced from `examples/wabou-bindgen.rs`".into()
        })
}

pub(super) fn framework_feature(
    metadata: &Value,
    manifest_path: &Path,
    feature: &str,
) -> Option<String> {
    let dependencies = package_metadata(metadata, manifest_path)?["dependencies"].as_array()?;
    if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou")
    {
        Some(format!("wabou/{feature}"))
    } else if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou-runtime")
    {
        Some(format!("wabou-runtime/{feature}"))
    } else {
        None
    }
}

pub(super) fn vite_feature<'a>(metadata: &'a Value, manifest_path: &Path) -> Option<&'a str> {
    let dependencies = package_metadata(metadata, manifest_path)?["dependencies"].as_array()?;
    if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou")
    {
        Some("wabou/vite")
    } else if dependencies
        .iter()
        .any(|dependency| dependency["name"] == "wabou-runtime")
    {
        Some("wabou-runtime/vite")
    } else {
        None
    }
}

pub(super) fn cargo_metadata(workspace: &Path, app: &App) -> Result<Value> {
    let manifest = app.root.join("Cargo.toml");
    let output = Command::new("cargo")
        .current_dir(workspace)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--manifest-path",
        ])
        .arg(manifest)
        .output()?;
    if !output.status.success() {
        return Err(format!("Cargo metadata failed with {}", output.status).into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

pub(super) fn binary_target<'a>(metadata: &'a Value, manifest_path: &Path) -> Option<&'a Value> {
    let package = package_metadata(metadata, manifest_path)?;
    let binaries = package["targets"]
        .as_array()?
        .iter()
        .filter(|target| {
            target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
        })
        .collect::<Vec<_>>();
    let package_name = package["name"].as_str();
    let named = binaries
        .iter()
        .copied()
        .find(|target| target["name"].as_str() == package_name);
    named.or_else(|| (binaries.len() == 1).then(|| binaries[0]))
}

pub(super) fn bindings_target<'a>(metadata: &'a Value, manifest_path: &Path) -> Option<&'a Value> {
    let package = package_metadata(metadata, manifest_path)?;
    let targets = package["targets"].as_array()?;
    let mut matching = targets.iter().filter(|target| {
        let is_example = target["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind == "example"));
        let is_bindgen_source = target["src_path"]
            .as_str()
            .and_then(|path| Path::new(path).file_name())
            .is_some_and(|name| name == "wabou-bindgen.rs");
        is_example && is_bindgen_source
    });
    let target = matching.next()?;
    matching.next().is_none().then_some(target)
}

pub(super) fn package_metadata<'a>(metadata: &'a Value, manifest_path: &Path) -> Option<&'a Value> {
    metadata["packages"].as_array()?.iter().find(|package| {
        package["manifest_path"]
            .as_str()
            .is_some_and(|path| Path::new(path) == manifest_path)
    })
}

pub(super) fn artifact_from_metadata(
    metadata: &Value,
    manifest_path: &Path,
    release: bool,
) -> Result<(PathBuf, String)> {
    let binary = binary_target(metadata, manifest_path)
        .and_then(|target| target["name"].as_str())
        .ok_or("app package has no unambiguous primary binary target")?;
    let target_dir = metadata["target_directory"]
        .as_str()
        .ok_or("Cargo metadata has no target directory")?;
    let profile = if release { "release" } else { "debug" };
    let filename = format!("{binary}{}", env::consts::EXE_SUFFIX);
    Ok((
        Path::new(target_dir).join(profile).join(&filename),
        filename,
    ))
}
