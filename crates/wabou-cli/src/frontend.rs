use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Command, ExitStatus};

use fs4::fs_std::FileExt as _;

use super::Result;
use super::config::{BuildProfile, profile_resource_dir};
use super::project::{App, ensure_workspace_package_exports};

fn vite_build_command(app: &App, args: &[&str]) -> Command {
    let mut command = Command::new("bun");
    command
        .current_dir(&app.frontend)
        .args(["x", "vite", "build"])
        .args(args);
    command
}

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

pub(super) fn build_unlocked(
    workspace: &Path,
    app: &App,
    args: &[&str],
    profile: BuildProfile,
    source_map: bool,
) -> Result<ExitStatus> {
    ensure_workspace_package_exports(workspace)?;
    let mut command = vite_build_command(app, args);
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
    command.status().map_err(|error| {
        format!(
            "failed to start Vite in {}: {error}; install Bun and project dependencies or run Wabou through `mise exec --`",
            app.frontend.display()
        )
        .into()
    })
}

pub(super) fn build(
    workspace: &Path,
    app: &App,
    args: &[&str],
    profile: BuildProfile,
    source_map: bool,
) -> Result<ExitStatus> {
    let _lock = build_lock(workspace, app)?;
    build_unlocked(workspace, app, args, profile, source_map)
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

    #[test]
    fn vite_build_does_not_depend_on_an_application_script_name() {
        let root = tempfile::tempdir().unwrap();
        let app = App {
            name: "app".into(),
            root: root.path().into(),
            frontend: root.path().into(),
            entry: "ui/index.tsx".into(),
        };
        let command = vite_build_command(&app, &["--mode", "fixture"]);
        assert_eq!(command.get_program(), "bun");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["x", "vite", "build", "--mode", "fixture"]
        );
        assert_eq!(command.get_current_dir(), Some(root.path()));
    }
}
