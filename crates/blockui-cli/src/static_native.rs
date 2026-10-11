//! Build/run an application-owned executable instead of the dynamic host/plugin.
use blockui_dev::project::{Project, checked};
use std::process::Command;

pub fn launch(
    project: &Project,
    features: &str,
    release: bool,
    capture: bool,
    run: bool,
) -> Result<(), String> {
    let (mut build, name) = command(project, features, "build")?;
    if release {
        build.arg("--release");
    }
    checked(&mut build)?;
    let executable = project
        .target
        .join(if release { "release" } else { "debug" })
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    if !run {
        println!("{}", executable.display());
        return Ok(());
    }
    let mut command = Command::new(executable);
    command.current_dir(&project.root);
    if capture {
        command.arg("--capture");
    }
    checked(&mut command)
}

pub fn command(
    project: &Project,
    features: &str,
    operation: &str,
) -> Result<(Command, String), String> {
    let targets = project.package["targets"]
        .as_array()
        .ok_or("missing Cargo targets")?;
    let binaries: Vec<_> = targets
        .iter()
        .filter(|target| {
            target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind == "bin"))
        })
        .collect();
    let explicit = project.package["metadata"]["blockui"]["static-bin"].as_str();
    let selected = explicit.unwrap_or(project.name());
    let binary = binaries
        .iter()
        .find(|target| target["name"] == selected)
        .copied()
        .or_else(|| {
            if explicit.is_none() && binaries.len() == 1 {
                Some(binaries[0])
            } else {
                None
            }
        })
        .ok_or(
            "static native build needs one binary or a package.metadata.blockui.static-bin target",
        )?;
    let name = binary["name"].as_str().ok_or("missing binary name")?;
    let mut enabled = vec![features.to_owned()];
    if let Some(required) = binary["required-features"].as_array() {
        enabled.extend(
            required
                .iter()
                .filter_map(|feature| feature.as_str().map(str::to_owned)),
        );
    }
    let features = enabled.join(",");
    let mut command = project.cargo(operation, features.trim_matches(','));
    command.args(["--bin", name]);
    Ok((command, name.to_owned()))
}
