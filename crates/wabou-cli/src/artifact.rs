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
    if package_metadata(&metadata, &manifest_path).is_none() {
        return Err(format!(
            "application Cargo package {} is absent from cargo metadata",
            manifest_path.display()
        )
        .into());
    }
    binary_target(&metadata, &manifest_path)
        .and_then(|target| target["name"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| "application binary target has no name".into())
}

pub(super) fn app_package(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    package_metadata(&metadata, &manifest_path)
        .ok_or("application Cargo package is absent from metadata")?["name"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "application Cargo package has no name".into())
}

pub(super) fn app_dev_features(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    dev_features(&metadata, &manifest_path)
        .ok_or_else(|| "application must depend on `wabou` or `wabou-runtime`".into())
}

pub(super) fn dev_features(metadata: &Value, manifest_path: &Path) -> Option<String> {
    Some(format!(
        "{},{}",
        framework_feature(metadata, manifest_path, "vite")?,
        framework_feature(metadata, manifest_path, "devtools")?
    ))
}

pub(super) fn app_profiling_feature(workspace: &Path, app: &App) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    framework_feature(&metadata, &manifest_path, "profiling")
        .ok_or_else(|| "application must depend on `wabou` or `wabou-runtime`".into())
}

pub(super) fn app_framework_feature(workspace: &Path, app: &App, feature: &str) -> Result<String> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    framework_feature(&metadata, &manifest_path, feature)
        .ok_or_else(|| "application must depend on `wabou` or `wabou-runtime`".into())
}

pub(super) fn app_bindings_target(workspace: &Path, app: &App) -> Result<String> {
    optional_app_bindings_target(workspace, app)?.ok_or_else(|| {
        "application must define one example sourced from `examples/wabou-bindgen.rs`".into()
    })
}

pub(super) fn optional_app_bindings_target(workspace: &Path, app: &App) -> Result<Option<String>> {
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    Ok(bindings_target(&metadata, &manifest_path)
        .and_then(|target| target["name"].as_str())
        .map(str::to_owned))
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
            .is_some_and(|path| paths_refer_to_same_manifest(Path::new(path), manifest_path))
    })
}

fn paths_refer_to_same_manifest(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    let Ok(left) = left.canonicalize() else {
        return false;
    };
    let Ok(right) = right.canonicalize() else {
        return false;
    };
    left == right
}

pub(super) fn artifact_from_metadata(
    metadata: &Value,
    manifest_path: &Path,
    release: bool,
) -> Result<(PathBuf, String)> {
    artifact_from_metadata_for_target(metadata, manifest_path, release, None)
}

pub(super) fn artifact_from_metadata_for_target(
    metadata: &Value,
    manifest_path: &Path,
    release: bool,
    target: Option<&str>,
) -> Result<(PathBuf, String)> {
    let binary = binary_target(metadata, manifest_path)
        .and_then(|target| target["name"].as_str())
        .ok_or("app package has no unambiguous primary binary target")?;
    let target_dir = metadata["target_directory"]
        .as_str()
        .ok_or("Cargo metadata has no target directory")?;
    let profile = if release { "release" } else { "debug" };
    let suffix = if target.is_some_and(|target| target.contains("windows")) {
        ".exe"
    } else {
        env::consts::EXE_SUFFIX
    };
    let filename = format!("{binary}{suffix}");
    let target_dir = target.map_or_else(
        || Path::new(target_dir).to_path_buf(),
        |target| Path::new(target_dir).join(cargo_target_directory_name(target)),
    );
    Ok((target_dir.join(profile).join(&filename), filename))
}

fn cargo_target_directory_name(target: &str) -> &str {
    ["-gnu.", "-musl."]
        .into_iter()
        .find_map(|marker| {
            target
                .find(marker)
                .map(|offset| &target[..offset + marker.len() - 1])
        })
        .unwrap_or(target)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn package_lookup_accepts_canonical_equivalent_manifest_paths() {
        let root = tempfile::tempdir().unwrap();
        let app = root.path().join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("Cargo.toml"), "[package]\nname = \"app\"\n").unwrap();
        let canonical = app.join("Cargo.toml").canonicalize().unwrap();
        let equivalent = app.join("..").join("app").join("Cargo.toml");
        let metadata = serde_json::json!({
            "packages": [{
                "name": "app",
                "manifest_path": equivalent,
                "targets": [{"name": "app", "kind": ["bin"]}]
            }]
        });

        assert_eq!(
            package_metadata(&metadata, &canonical).unwrap()["name"],
            "app"
        );
        assert_eq!(binary_target(&metadata, &canonical).unwrap()["name"], "app");
    }
}
