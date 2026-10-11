//! Portable distribution with the installed CLI as the resident runtime.
use super::*;
#[cfg(feature = "embedded-renderer")]
use wabou_launch::{Implementation, UiSource};

pub(super) fn bundle(
    workspace: &Path,
    app: &App,
    configuration: &Path,
    out: &Path,
    static_link: bool,
) -> Result<()> {
    #[cfg(not(feature = "embedded-renderer"))]
    let _ = (workspace, app, configuration, out, static_link);
    #[cfg(not(feature = "embedded-renderer"))]
    return Err("bundle requires a CLI with embedded-renderer".into());
    #[cfg(feature = "embedded-renderer")]
    {
        // Never overwrite an existing distribution or a source directory.
        if out.exists() {
            return Err(format!("output already exists: {}", out.display()).into());
        }
        let mut config = load_launch_config(configuration)?;
        ensure(
            build_frontend(workspace, app, &[], BuildProfile::Release, false)?,
            "Vite build",
        )?;
        let parent = out
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let staging = tempfile::tempdir_in(parent)?;
        if !static_link {
            fs::create_dir(staging.path().join("modules"))?;
        }
        let linked_executable = if static_link {
            Some(build_linked(workspace, &config)?)
        } else {
            None
        };
        for (index, capability) in config.capabilities.iter_mut().enumerate() {
            if static_link {
                capability.implementation = Implementation::Library {
                    path: "linked".into(),
                };
                continue;
            }
            let library = match &capability.implementation {
                Implementation::Library { path } => path.clone(),
                Implementation::Project {
                    manifest,
                    features,
                    target_directory,
                    ..
                } => {
                    let mut cargo = Command::new("cargo");
                    cargo
                        .args([
                            "build",
                            "--release",
                            "--message-format=json-render-diagnostics",
                            "--manifest-path",
                        ])
                        .arg(manifest);
                    apply_cargo_features(&mut cargo, features);
                    if let Some(directory) = target_directory {
                        cargo.arg("--target-dir").arg(directory);
                    }
                    let output = cargo.output()?;
                    std::io::Write::write_all(&mut std::io::stderr(), &output.stderr)?;
                    ensure(output.status, "capability release build")?;
                    find_library(&output.stdout)?
                }
            };
            let filename = library
                .file_name()
                .ok_or("capability library has no filename")?;
            let relative =
                PathBuf::from("modules").join(format!("{index}-{}", filename.to_string_lossy()));
            fs::copy(library, staging.path().join(&relative))?;
            capability.implementation = Implementation::Library { path: relative };
        }
        let ui_bundle = bundle_path(workspace, app, BuildProfile::Release)?;
        config.source = UiSource::Bundle {
            path: PathBuf::from("resources")
                .join(ui_bundle.file_name().ok_or("bundle has no filename")?),
        };
        copy_tree(
            ui_bundle.parent().ok_or("bundle has no directory")?,
            &staging.path().join("resources"),
        )?;
        config.resource_directory = Some("resources".into());
        fs::write(
            staging.path().join("app.json"),
            serde_json::to_vec_pretty(&config)?,
        )?;
        let executable = if cfg!(windows) { "wabou.exe" } else { "wabou" };
        fs::copy(
            linked_executable
                .as_ref()
                .map(|(_, path)| path.clone())
                .unwrap_or(env::current_exe()?),
            staging.path().join(executable),
        )?;
        fs::write(
            staging.path().join("README.txt"),
            format!(
                "Run: {executable}{}\nResource files belong in resources/.\n",
                if static_link { "" } else { " launch app.json" }
            ),
        )?;
        // Publish only after all builds and copies succeed.
        fs::rename(staging.path(), out)?;
        println!("[wabou] portable application: {}", out.display());
        Ok(())
    }
}

#[cfg(any(feature = "embedded-renderer", test))]
fn find_library(messages: &[u8]) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    for line in messages.split(|byte| *byte == b'\n') {
        let Ok(value) = serde_json::from_slice::<Value>(line) else {
            continue;
        };
        if value["reason"] != "compiler-artifact"
            || !value["target"]["crate_types"]
                .as_array()
                .is_some_and(|types| types.iter().any(|kind| kind == "cdylib"))
        {
            continue;
        }
        if let Some(files) = value["filenames"].as_array() {
            candidates.extend(
                files
                    .iter()
                    .filter_map(Value::as_str)
                    .map(PathBuf::from)
                    .filter(|path| {
                        matches!(
                            path.extension().and_then(|ext| ext.to_str()),
                            Some("so" | "dylib" | "dll")
                        )
                    }),
            );
        }
    }
    if candidates.len() != 1 {
        return Err(
            "capability build must produce exactly one cdylib; use a single-package manifest"
                .into(),
        );
    }
    Ok(candidates.remove(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn selects_loadable_artifact_and_rejects_ambiguous_output() {
        let message = br#"{"reason":"compiler-artifact","target":{"crate_types":["rlib","cdylib"]},"filenames":["/tmp/libhost.rlib","/tmp/libhost.so"]}"#;
        assert_eq!(
            find_library(message).unwrap(),
            PathBuf::from("/tmp/libhost.so")
        );
        assert!(find_library(b"").is_err());
        let twice = [message.as_slice(), b"\n", message.as_slice()].concat();
        assert!(find_library(&twice).is_err());
    }
}

/// Generate a conventional Cargo application: no plugin files are needed at runtime.
#[cfg(feature = "embedded-renderer")]
fn build_linked(
    workspace: &Path,
    config: &wabou_launch::LaunchConfig,
) -> Result<(tempfile::TempDir, PathBuf)> {
    let project = tempfile::tempdir()?;
    let mut dependencies = toml::Table::new();
    for name in [
        "wabou-runtime-library",
        "wabou-launch",
        "wabou-capability-sdk",
    ] {
        let mut dependency = toml::Table::new();
        let path = workspace.join("crates").join(name);
        if path.join("Cargo.toml").is_file() {
            dependency.insert("path".into(), path.to_string_lossy().to_string().into());
        } else {
            dependency.insert(
                "version".into(),
                format!("={}", env!("CARGO_PKG_VERSION")).into(),
            );
        }
        if name == "wabou-capability-sdk" {
            dependency.insert(
                "features".into(),
                toml::Value::Array(vec!["static-link".into()]),
            );
        }
        dependencies.insert(name.into(), toml::Value::Table(dependency));
    }
    dependencies.insert("serde_json".into(), "1".into());
    let mut registrations = String::new();
    for (index, capability) in config.capabilities.iter().enumerate() {
        let Implementation::Project {
            manifest, features, ..
        } = &capability.implementation
        else {
            return Err("static bundle requires project implementations; native libraries cannot be converted to Rust libraries".into());
        };
        let output = Command::new("cargo")
            .args([
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--manifest-path",
            ])
            .arg(manifest)
            .output()?;
        ensure(output.status, "capability metadata")?;
        let metadata: Value = serde_json::from_slice(&output.stdout)?;
        let packages = metadata["packages"]
            .as_array()
            .ok_or("Cargo metadata has no packages")?;
        let manifest = manifest.canonicalize()?;
        let package = packages
            .iter()
            .find(|package| {
                package["manifest_path"]
                    .as_str()
                    .is_some_and(|path| Path::new(path) == manifest)
            })
            .ok_or("capability manifest must select one package")?;
        let package_name = package["name"]
            .as_str()
            .ok_or("capability package has no name")?;
        let alias = format!("capability_{index}");
        let mut dependency = toml::Table::new();
        dependency.insert("package".into(), package_name.into());
        dependency.insert(
            "path".into(),
            manifest
                .parent()
                .ok_or("manifest has no directory")?
                .to_string_lossy()
                .to_string()
                .into(),
        );
        dependency.insert(
            "features".into(),
            toml::Value::Array(features.iter().cloned().map(Into::into).collect()),
        );
        dependencies.insert(alias.clone(), toml::Value::Table(dependency));
        registrations.push_str(&format!(
            "({}, {alias}::wabou_capability_v2()),",
            serde_json::to_string(&capability.name)? + ".to_string()"
        ));
    }
    let mut manifest = toml::Table::new();
    let mut package = toml::Table::new();
    package.insert("name".into(), "wabou-application".into());
    package.insert("version".into(), "0.0.0".into());
    package.insert("edition".into(), "2024".into());
    manifest.insert("package".into(), toml::Value::Table(package));
    manifest.insert("dependencies".into(), toml::Value::Table(dependencies));
    manifest.insert("workspace".into(), toml::Value::Table(toml::Table::new()));
    fs::write(
        project.path().join("Cargo.toml"),
        toml::to_string(&manifest)?,
    )?;
    fs::create_dir(project.path().join("src"))?;
    fs::write(
        project.path().join("src/main.rs"),
        format!(
            r#"
fn main() -> Result<(), Box<dyn std::error::Error>> {{
    let directory = std::env::current_exe()?.parent().ok_or("executable has no directory")?.to_owned();
    let mut config: wabou_launch::LaunchConfig = serde_json::from_slice(&std::fs::read(directory.join("app.json"))?)?;
    config.resolve_paths(&directory);
    unsafe {{ wabou_runtime_library::run_linked(config, &[{registrations}]) }}?;
    Ok(())
}}
"#
        ),
    )?;
    let target = workspace.join("target");
    let mut cargo = Command::new("cargo");
    cargo
        .args(["build", "--release", "--manifest-path"])
        .arg(project.path().join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&target);
    ensure(cargo.status()?, "static application build")?;
    let binary = target.join("release").join(if cfg!(windows) {
        "wabou-application.exe"
    } else {
        "wabou-application"
    });
    Ok((project, binary))
}

#[cfg(feature = "embedded-renderer")]
fn copy_tree(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
