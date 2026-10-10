use super::{Options, configuration_profile};
use crate::project::{Project, checked};
use serde_json::Value;
use std::{path::PathBuf, process::Command};

pub fn launch(package: &str, watch: bool, options: Options) -> Result<(), String> {
    if watch && options.release {
        return Err("native hot reload uses debug builds; use blockui run/build --release".into());
    }
    let project = Project::discover(
        Some(package),
        options.manifest.as_deref(),
        options.target_dir.as_deref(),
        &options.features,
        false,
    )?;
    launch_project(&project, watch, options)
}

/// Launch using an already resolved native project, without another metadata query.
/// The project's manifest and target take precedence over their Options counterparts;
/// resolve the project with the same features supplied in Options.
pub fn launch_project(project: &Project, watch: bool, options: Options) -> Result<(), String> {
    if watch && options.release {
        return Err("native hot reload uses debug builds; use blockui run/build --release".into());
    }
    let profile = if options.release { "release" } else { "debug" };
    // The stable renderer benefits from optimization; UI reloads keep fast debug builds.
    let host_profile = if watch || options.release {
        "release"
    } else {
        "debug"
    };
    let config = configuration_profile(
        &project.metadata,
        &project.manifest,
        project.name(),
        &project.target,
        &options.features,
        profile,
    )?;
    if options.configuration_only {
        println!("{config}");
        return Ok(());
    }
    let host_name = options
        .host_package
        .as_deref()
        .or_else(|| config["BLOCKUI_DEV_HOST_PACKAGE"].as_str());
    let example = if host_name.is_none() {
        config["BLOCKUI_DEV_HOST_EXAMPLE"].as_str()
    } else {
        None
    };
    if options.task.is_some() && example != Some("seven_guis") {
        return Err("--task is only supported by the seven_guis host example".into());
    }
    let (host_manifest, host_bin, host_features) = if let Some(name) = host_name {
        let host = project.metadata["packages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == name)
            .ok_or_else(|| format!("host package {name} is not in the Cargo graph"))?;
        let bins: Vec<_> = host["targets"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|t| {
                t["kind"]
                    .as_array()
                    .is_some_and(|k| k.contains(&Value::from("bin")))
            })
            .collect();
        let binary = bins
            .iter()
            .find(|t| t["name"] == name)
            .copied()
            .or_else(|| (bins.len() == 1).then(|| bins[0]))
            .ok_or("host package must have one binary or a binary matching its package name")?;
        (
            PathBuf::from(host["manifest_path"].as_str().unwrap()),
            binary["name"].as_str().unwrap().to_owned(),
            host["features"]["native"].is_array(),
        )
    } else {
        (
            project
                .sdk_root()?
                .join("crates/blockui-runtime/Cargo.toml"),
            "blockui-host".into(),
            true,
        )
    };
    if !options.prepare_only {
        let mut build = crate::project::cargo_command();
        build
            .current_dir(&project.root)
            .arg("build")
            .arg("--manifest-path")
            .arg(&host_manifest)
            .arg("--target-dir")
            .arg(&project.target);
        if host_features {
            build.args(["--features", "native"]);
        }
        if let Some(example) = example {
            build.args(["--example", example]);
        } else {
            build.args(["--bin", &host_bin]);
        }
        if host_profile == "release" {
            build.arg("--release");
        }
        checked(&mut build)?;
    }
    // Native reload consumes only the dynamic implementation, not an rlib.
    let mut build = project.cargo("rustc", &options.features);
    build.args(["--lib", "--crate-type", "cdylib"]);
    if options.release {
        build.arg("--release");
    }
    checked(&mut build)?;
    if options.prepare_only {
        println!("{config}");
        return Ok(());
    }
    if options.build_only {
        println!("{}", config["BLOCKUI_DEV_ARTIFACT"].as_str().unwrap());
        return Ok(());
    }
    let temporary = tempfile::tempdir().map_err(|e| e.to_string())?;
    let config_path = temporary.path().join("reload.json");
    std::fs::write(&config_path, config.to_string()).map_err(|e| e.to_string())?;
    let executable = match example {
        Some(example) => project
            .target
            .join(host_profile)
            .join("examples")
            .join(format!("{example}{}", std::env::consts::EXE_SUFFIX)),
        None => project
            .target
            .join(host_profile)
            .join(format!("{host_bin}{}", std::env::consts::EXE_SUFFIX)),
    };
    let mut command = Command::new(executable);
    crate::project::compiler_environment(&mut command);
    if std::env::var_os("BLOCKUI_CARGO").is_some_and(|path| !path.is_empty()) {
        command.env(
            "BLOCKUI_CARGO",
            crate::project::cargo_command().get_program(),
        );
    }
    command.current_dir(&project.root).arg(
        config["BLOCKUI_DEV_ARTIFACT"]
            .as_str()
            .ok_or("missing artifact")?,
    );
    if watch {
        command.arg("--watch").arg(&config_path);
        for (key, value) in config.as_object().unwrap() {
            command.env(key, value.as_str().ok_or("invalid config field")?);
        }
    }
    if options.capture {
        command.arg("--capture");
    }
    if let Some(task) = options.task {
        command.args(["--task", &task]);
    }
    checked(&mut command)
}
