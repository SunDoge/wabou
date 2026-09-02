use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_packager::Config;
use serde_json::{Value, json};

use super::artifact::{app_binary, cargo_metadata, package_metadata};
use super::config::{BuildProfile, PackageConfig, PackageFormat, distribution_root};
use super::project::App;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[cfg(target_os = "macos")]
fn macos_packager_config() -> Value {
    let identity = std::env::var("APPLE_SIGNING_IDENTITY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "-".to_owned());
    json!({ "signingIdentity": identity })
}

#[cfg(not(target_os = "macos"))]
fn macos_packager_config() -> Value {
    Value::Null
}

/// Translate Wabou's private package model at one narrow adapter boundary.
/// The JSON is retained for diagnostics, but packaging uses the typed library
/// API and never shells out to a separately installed cargo subcommand.
fn run_backend(value: &Value, diagnostic_path: &Path) -> Result<Vec<PathBuf>> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(diagnostic_path, &bytes)?;
    let config = decode_config(&bytes)?;
    let outputs = cargo_packager::package(&config)
        .map_err(|error| format!("native package backend failed: {error}"))?;
    Ok(outputs
        .into_iter()
        .flat_map(|output| output.paths)
        .collect())
}

pub(super) fn package_built_application(
    workspace: &Path,
    app: &App,
    config: &PackageConfig,
    format_override: &[PackageFormat],
) -> Result<()> {
    let (stage, binary) = stage_application(workspace, app, config)?;
    let metadata = cargo_metadata(workspace, app)?;
    let manifest_path = app.root.join("Cargo.toml").canonicalize()?;
    let version = package_metadata(&metadata, &manifest_path)
        .and_then(|package| package["version"].as_str())
        .ok_or("Cargo metadata has no application version")?;
    let formats = if format_override.is_empty() {
        &config.formats
    } else {
        format_override
    };
    if formats.is_empty() {
        return Err("wabou.toml must declare at least one package format".into());
    }

    let package_root = distribution_root(workspace, app)?;
    let bundles = package_root.join("bundles");
    fs::create_dir_all(&bundles)?;
    let packager_config = package_root.join("packager.json");
    let resources = stage.join("resources");
    let icons = config
        .icons
        .iter()
        .map(|path| app.root.join(path).to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    let license_file = config
        .license_file
        .as_ref()
        .map(|path| app.root.join(path).to_string_lossy().into_owned());
    let generated = json!({
        "name": app.name,
        "productName": config.product_name,
        "version": version,
        "identifier": config.identifier,
        "description": config.description,
        "authors": config.authors,
        "copyright": config.copyright,
        "licenseFile": license_file,
        "icons": icons,
        "binaries": [{ "path": binary, "main": true }],
        "binariesDir": stage,
        "resources": [{ "src": resources, "target": "resources" }],
        "formats": formats.iter().map(|format| format.as_str()).collect::<Vec<_>>(),
        "outDir": bundles,
        "macos": macos_packager_config(),
    });
    let outputs = run_backend(&generated, &packager_config)?;
    for output in outputs {
        println!("[wabou] packaged {}", output.display());
    }
    Ok(())
}

fn stage_application(
    workspace: &Path,
    app: &App,
    config: &PackageConfig,
) -> Result<(PathBuf, String)> {
    let package_root = distribution_root(workspace, app)?;
    let stage = package_root.join("stage");
    if stage.is_dir() {
        fs::remove_dir_all(&stage)?;
    }
    let resources = stage.join("resources");
    fs::create_dir_all(&resources)?;
    let binary = app_binary(workspace, app)?;
    let release_root = package_root.join(BuildProfile::Release.as_str());
    fs::copy(release_root.join(&binary), stage.join(&binary))?;
    fs::copy(
        release_root.join("resources/bundle.js"),
        resources.join("bundle.js"),
    )?;

    let app_root = app.root.canonicalize()?;
    for relative in &config.resources {
        let source = app.root.join(relative).canonicalize().map_err(|error| {
            format!(
                "cannot stage package resource {}: {error}",
                relative.display()
            )
        })?;
        if !source.starts_with(&app_root) {
            return Err(format!(
                "package resource {} escapes the application directory",
                relative.display()
            )
            .into());
        }
        let name = source
            .file_name()
            .ok_or("package resource must have a file name")?;
        copy_resource(&source, &resources.join(name))?;
    }
    Ok((stage, binary))
}

fn copy_resource(source: &Path, destination: &Path) -> Result<()> {
    if fs::symlink_metadata(source)?.file_type().is_symlink() {
        return Err(format!(
            "package resources cannot contain symbolic links: {}",
            source.display()
        )
        .into());
    }
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_resource(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else if source.is_file() {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, destination)?;
    } else {
        return Err(format!("unsupported package resource {}", source.display()).into());
    }
    Ok(())
}

fn decode_config(bytes: &[u8]) -> Result<Config> {
    serde_json::from_slice(bytes).map_err(|error| {
        format!("generated cargo-packager configuration is invalid: {error}").into()
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[cfg(target_os = "macos")]
    use super::macos_packager_config;
    use super::{copy_resource, decode_config};

    #[test]
    fn generated_adapter_input_is_a_typed_packager_config() {
        let config = decode_config(
            br#"{
                "name":"example",
                "productName":"Example",
                "version":"1.2.3",
                "identifier":"dev.wabou.example",
                "binaries":[{"path":"example","main":true}],
                "binariesDir":"/tmp/stage",
                "formats":["deb"],
                "outDir":"/tmp/bundles"
            }"#,
        )
        .unwrap();
        assert_eq!(config.product_name, "Example");
        assert_eq!(config.version, "1.2.3");
        assert_eq!(config.binaries.len(), 1);
    }

    #[test]
    fn recursively_stages_application_resources() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("assets/nested");
        let destination = root.path().join("stage/assets");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("fixture.txt"), "staged").unwrap();

        copy_resource(&root.path().join("assets"), &destination).unwrap();

        assert_eq!(
            fs::read_to_string(destination.join("nested/fixture.txt")).unwrap(),
            "staged"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_packages_always_receive_a_bundle_signing_identity() {
        let config = macos_packager_config();
        let identity = config["signingIdentity"].as_str().unwrap();
        assert!(!identity.is_empty());
    }
}
