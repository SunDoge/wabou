use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Command, ExitStatus};

use fs4::fs_std::FileExt as _;

use super::Result;
use super::config::{BuildProfile, profile_resource_dir};
use super::project::{App, ensure_workspace_package_exports};

fn build_lock(workspace: &Path, app: &App) -> Result<fs::File> {
    let directory = workspace.join("target/wabou/frontend").join(&app.name);
    fs::create_dir_all(&directory)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(directory.join("frontend.lock"))?;
    file.lock_exclusive()?;
    Ok(file)
}

pub(super) fn run_unlocked(
    workspace: &Path,
    app: &App,
    script: &str,
    args: &[&str],
    profile: BuildProfile,
    source_map: bool,
) -> Result<ExitStatus> {
    ensure_workspace_package_exports(workspace)?;
    let mut command = Command::new("bun");
    command.current_dir(&app.frontend).args(["run", script]);
    command
        .env("WABOU_BUILD_PROFILE", profile.as_str())
        .env(
            "WABOU_ENV_DEBUG",
            if profile == BuildProfile::Debug {
                "true"
            } else {
                "false"
            },
        )
        .env(
            "WABOU_SOURCE_MAP",
            if source_map { "true" } else { "false" },
        )
        .env(
            "WABOU_OUT_DIR",
            profile_resource_dir(workspace, app, profile)?,
        );
    if !args.is_empty() {
        command.arg("--").args(args);
    }
    command.status().map_err(|error| {
        format!(
            "failed to run `bun run {script}` in {}: {error}; install Bun or run Wabou through `mise exec --`",
            app.frontend.display()
        )
        .into()
    })
}

pub(super) fn run(
    workspace: &Path,
    app: &App,
    script: &str,
    args: &[&str],
    profile: BuildProfile,
    source_map: bool,
) -> Result<ExitStatus> {
    let _lock = build_lock(workspace, app)?;
    run_unlocked(workspace, app, script, args, profile, source_map)
}

pub(super) fn lock(workspace: &Path, app: &App) -> Result<fs::File> {
    build_lock(workspace, app)
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

    use fs4::fs_std::FileExt as _;

    use super::*;

    #[test]
    fn build_lock_excludes_a_second_process_handle() {
        let root = tempfile::tempdir().unwrap();
        let app = App {
            name: "app".into(),
            root: root.path().into(),
            frontend: root.path().into(),
            entry: "ui/index.tsx".into(),
        };
        let first = build_lock(root.path(), &app).unwrap();
        let second = OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.path().join("target/wabou/frontend/app/frontend.lock"))
            .unwrap();

        assert!(!second.try_lock_exclusive().unwrap());
        drop(first);
        assert!(second.try_lock_exclusive().unwrap());
    }
}
