use std::fs::{self, OpenOptions};
use std::path::Path;
use std::process::{Command, ExitStatus};

use fs4::fs_std::FileExt as _;

use super::Result;
use super::config::{BuildProfile, profile_resource_dir};
use super::project::{App, ensure_javascript_dependencies, ensure_workspace_package_exports};

fn vite_build_command(app: &App, args: &[&str]) -> Command {
    let mut command = Command::new("bun");
    command
        .current_dir(&app.frontend)
        .args(["x", "vite", "build"])
        .args(args);
    // Authored build modes may select a fixture or another dedicated entry.
    if !args.contains(&"--mode") {
        command.env("WABOU_VITE_ENTRY", &app.entry);
    }
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
    ensure_javascript_dependencies(workspace, app)?;
    ensure_workspace_package_exports(workspace)?;
    let configuration = default_vite_config(app)?;
    let mut command = vite_build_command(app, args);
    if let Some(configuration) = &configuration {
        command.arg("--config").arg(configuration.path());
    }
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

#[cfg_attr(not(feature = "embedded-renderer"), allow(dead_code))]
pub(super) fn lock(workspace: &Path, app: &App) -> Result<fs::File> {
    build_lock(workspace, app)
}

pub(super) fn build_test_script(
    workspace: &Path,
    app: &App,
    entry: &Path,
    output: &Path,
) -> Result<ExitStatus> {
    ensure_javascript_dependencies(workspace, app)?;
    ensure_workspace_package_exports(workspace)?;
    let output_dir = output
        .parent()
        .ok_or_else(|| format!("test script output has no parent: {}", output.display()))?;
    fs::create_dir_all(output_dir)?;
    let file_name = output
        .file_name()
        .ok_or_else(|| format!("test script output has no file name: {}", output.display()))?;
    let config_path = output_dir.join("test-script.vite.config.mjs");
    let conditions = if workspace.join("packages/test/src/index.ts").is_file() {
        r#"["browser", "wabou-source"]"#
    } else {
        r#"["browser"]"#
    };
    fs::write(
        &config_path,
        format!(
            r#"import solid from "@solidjs/vite-plugin";

export default {{
  plugins: [solid({{ solid: {{ generate: "universal", moduleName: "@wabou/core/renderer" }} }})],
  resolve: {{ conditions: {conditions} }},
  build: {{
    emptyOutDir: false,
    minify: false,
    sourcemap: true,
    outDir: {},
    lib: {{ entry: {}, formats: ["iife"], name: "WabouTest", fileName: () => {} }},
    rollupOptions: {{ output: {{ inlineDynamicImports: true }} }},
  }},
}};
"#,
            serde_json::to_string(output_dir)?,
            serde_json::to_string(entry)?,
            serde_json::to_string(&file_name.to_string_lossy())?,
        ),
    )?;
    let mut command =
        vite_build_command(app, &["--config", config_path.to_string_lossy().as_ref()]);
    command.status().map_err(|error| {
        format!(
            "failed to build test script with Vite in {}: {error}",
            app.frontend.display()
        )
        .into()
    })
}

/// Adapt the single authored Wabou configuration for Vite, or provide defaults.
pub(super) fn default_vite_config(app: &App) -> Result<Option<tempfile::NamedTempFile>> {
    let project = app.root.join("wabou.config.ts");
    if project.is_file() {
        let file = tempfile::Builder::new().suffix(".mjs").tempfile()?;
        fs::write(
            file.path(),
            format!(
                "export {{ default }} from {};\n",
                serde_json::to_string(&project.canonicalize()?)?
            ),
        )?;
        return Ok(Some(file));
    }
    if ["ts", "js", "mts", "mjs", "cts", "cjs"]
        .iter()
        .any(|extension| app.root.join(format!("vite.config.{extension}")).is_file())
    {
        return Ok(None);
    }
    let output = Command::new("bun").current_dir(&app.root).args(["--eval", "import { fileURLToPath } from 'node:url'; console.log(JSON.stringify(fileURLToPath(import.meta.resolve('@wabou/vite'))));"]).output()?;
    super::ensure(output.status, "resolve @wabou/vite")?;
    let module: String = serde_json::from_slice(&output.stdout)?;
    let file = tempfile::Builder::new().suffix(".mjs").tempfile()?;
    let options = serde_json::json!({"root": app.root.canonicalize()?, "entry": app.entry, "outDir": "dist/resources"});
    fs::write(
        file.path(),
        format!(
            "import {{ defineConfig }} from {};\nexport default defineConfig({options});\n",
            serde_json::to_string(&module)?
        ),
    )?;
    Ok(Some(file))
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

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
        assert!(!command.get_envs().any(|(key, _)| key == "WABOU_VITE_ENTRY"));
        assert!(
            vite_build_command(&app, &[])
                .get_envs()
                .any(|(key, value)| key == "WABOU_VITE_ENTRY"
                    && value == Some(std::ffi::OsStr::new("ui/index.tsx")))
        );
        assert_eq!(command.get_program(), "bun");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["x", "vite", "build", "--mode", "fixture"]
        );
        assert_eq!(command.get_current_dir(), Some(root.path()));
    }
}
