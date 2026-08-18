use std::path::Path;
use std::process::Command;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
use std::env;

use super::{Result, ensure_workspace_package_exports, find_workspace, load_app};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Status {
    Pass,
    Warning,
    Fail,
}

struct Check {
    status: Status,
    name: String,
    detail: String,
}

impl Check {
    fn new(status: Status, name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            status,
            name: name.into(),
            detail: detail.into(),
        }
    }
}

fn command_version(program: &str, args: &[&str]) -> std::result::Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!("exited with {}", output.status));
    }
    let text = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    Ok(String::from_utf8_lossy(text)
        .lines()
        .next()
        .unwrap_or("available")
        .trim()
        .to_owned())
}

fn tool(checks: &mut Vec<Check>, program: &str, args: &[&str], required: bool, remediation: &str) {
    match command_version(program, args) {
        Ok(version) => checks.push(Check::new(Status::Pass, program, version)),
        Err(error) => checks.push(Check::new(
            if required {
                Status::Fail
            } else {
                Status::Warning
            },
            program,
            format!("{error}; {remediation}"),
        )),
    }
}

#[cfg(target_os = "linux")]
fn platform(checks: &mut Vec<Check>) {
    if command_version("pkg-config", &["--version"]).is_err() {
        checks.push(Check::new(
            Status::Fail,
            "Linux native dependencies",
            "pkg-config is unavailable; install the packages listed in README.md",
        ));
        return;
    }

    for (module, package) in [
        ("gtk+-3.0", "libgtk-3-dev"),
        ("egl", "libegl1-mesa-dev"),
        ("fontconfig", "libfontconfig1-dev"),
    ] {
        let available = Command::new("pkg-config")
            .args(["--exists", module])
            .status()
            .is_ok_and(|status| status.success());
        checks.push(Check::new(
            if available {
                Status::Pass
            } else {
                Status::Fail
            },
            format!("native library {module}"),
            if available {
                "available through pkg-config".to_owned()
            } else {
                format!("not found; install `{package}`")
            },
        ));
    }
}

#[cfg(target_os = "macos")]
fn platform(checks: &mut Vec<Check>) {
    match command_version("xcode-select", &["-p"]) {
        Ok(path) => checks.push(Check::new(Status::Pass, "Xcode command-line tools", path)),
        Err(error) => checks.push(Check::new(
            Status::Fail,
            "Xcode command-line tools",
            format!("{error}; run `xcode-select --install`"),
        )),
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn platform(checks: &mut Vec<Check>) {
    checks.push(Check::new(
        Status::Warning,
        "native platform",
        format!(
            "{} is not part of Wabou's currently documented support matrix",
            env::consts::OS
        ),
    ));
}

pub(super) fn run(cwd: &Path, app_path: Option<&Path>) -> Result<()> {
    let mut checks = Vec::new();
    tool(
        &mut checks,
        "rustc",
        &["--version"],
        true,
        "install stable Rust from https://rustup.rs",
    );
    tool(
        &mut checks,
        "cargo",
        &["--version"],
        true,
        "install stable Rust from https://rustup.rs",
    );
    tool(
        &mut checks,
        "bun",
        &["--version"],
        true,
        "run `mise install` or install Bun",
    );
    tool(
        &mut checks,
        "git",
        &["--version"],
        false,
        "install Git to create and update source-based Wabou applications",
    );
    platform(&mut checks);

    let project_start = app_path
        .map(|path| {
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                cwd.join(path)
            }
        })
        .unwrap_or_else(|| cwd.to_path_buf());
    match find_workspace(&project_start) {
        Ok(workspace) => match load_app(&workspace, cwd, app_path) {
            Ok(app) => {
                checks.push(Check::new(
                    Status::Pass,
                    "Wabou application",
                    app.root.display().to_string(),
                ));
                let modules = workspace.join("node_modules");
                checks.push(Check::new(
                    if modules.is_dir() {
                        Status::Pass
                    } else {
                        Status::Fail
                    },
                    "JavaScript dependencies",
                    if modules.is_dir() {
                        modules.display().to_string()
                    } else {
                        format!("{} is missing; run `bun install`", modules.display())
                    },
                ));

                let submodule = workspace.join("vendor/wabou");
                if workspace.join(".gitmodules").is_file() {
                    checks.push(Check::new(
                        if submodule.join("Cargo.toml").is_file() {
                            Status::Pass
                        } else {
                            Status::Fail
                        },
                        "Wabou source submodule",
                        if submodule.join("Cargo.toml").is_file() {
                            submodule.display().to_string()
                        } else {
                            "not initialized; run `git submodule update --init`".to_owned()
                        },
                    ));
                }

                match ensure_workspace_package_exports(&workspace) {
                    Ok(()) => checks.push(Check::new(
                        Status::Pass,
                        "Wabou JavaScript artifacts",
                        "present and current",
                    )),
                    Err(error) => checks.push(Check::new(
                        Status::Fail,
                        "Wabou JavaScript artifacts",
                        error.to_string().replace('\n', "; "),
                    )),
                }
            }
            Err(error) => checks.push(Check::new(
                Status::Fail,
                "Wabou application",
                error.to_string(),
            )),
        },
        Err(error) => checks.push(Check::new(Status::Fail, "Wabou project", error.to_string())),
    }

    println!("Wabou doctor\n");
    for check in &checks {
        let label = match check.status {
            Status::Pass => "ok",
            Status::Warning => "warn",
            Status::Fail => "fail",
        };
        println!("[{label}] {}: {}", check.name, check.detail);
    }
    let failures = checks
        .iter()
        .filter(|check| check.status == Status::Fail)
        .count();
    let warnings = checks
        .iter()
        .filter(|check| check.status == Status::Warning)
        .count();
    println!(
        "\n{} passed, {warnings} warnings, {failures} failures",
        checks.len() - warnings - failures
    );
    if failures == 0 {
        Ok(())
    } else {
        Err(format!("Wabou doctor found {failures} blocking problem(s)").into())
    }
}
