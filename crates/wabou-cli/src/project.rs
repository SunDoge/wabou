use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::Result;

pub(super) struct App {
    pub(super) name: String,
    pub(super) root: PathBuf,
    pub(super) frontend: PathBuf,
    pub(super) entry: String,
}

pub(super) fn find_workspace(start: &Path) -> Result<PathBuf> {
    for dir in start.ancestors() {
        let manifest = dir.join("Cargo.toml");
        if fs::read_to_string(&manifest).is_ok_and(|text| text.contains("[workspace]")) {
            return Ok(dir.to_path_buf());
        }
    }
    find_app_root(start).ok_or_else(|| "not inside a Wabou Cargo project".into())
}

pub(super) fn load_app(workspace: &Path, cwd: &Path, app_path: Option<&Path>) -> Result<App> {
    let root = match app_path {
        Some(path) if path.is_absolute() => path.to_path_buf(),
        Some(path) => cwd.join(path),
        None => find_app_root(cwd).unwrap_or_else(|| workspace.join("apps/gallery")),
    };
    if !root.join("Cargo.toml").is_file() {
        return Err(format!(
            "{} is not a Wabou app: Cargo.toml is missing",
            root.display()
        )
        .into());
    }
    if !root.join("package.json").is_file() {
        return Err(format!(
            "{} is not a Wabou app: package.json is missing",
            root.display()
        )
        .into());
    }
    let entry = if root.join("ui/index.tsx").is_file() {
        "ui/index.tsx"
    } else {
        return Err(format!("{} has no conventional ui/index.tsx entry", root.display()).into());
    };
    Ok(App {
        name: root
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("app directory must have a UTF-8 name")?
            .to_string(),
        frontend: root.clone(),
        root,
        entry: entry.to_string(),
    })
}

pub(super) fn find_app_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| {
            fs::read_to_string(dir.join("Cargo.toml")).is_ok_and(|text| text.contains("[package]"))
        })
        .map(Path::to_path_buf)
}

fn dependency_names(manifest: &Value) -> Vec<&str> {
    ["dependencies", "devDependencies"]
        .into_iter()
        .filter_map(|section| manifest.get(section)?.as_object())
        .flat_map(|dependencies| dependencies.keys().map(String::as_str))
        .collect()
}

fn installed_package_manifest(workspace: &Path, app: &App, package: &str) -> bool {
    [
        app.frontend.join("node_modules"),
        workspace.join("node_modules"),
    ]
    .into_iter()
    .any(|modules| modules.join(package).join("package.json").is_file())
}

/// Fail before starting Vite when Bun's workspace links have not been installed.
/// Vite otherwise reports each import independently, hiding the single setup issue.
pub(super) fn ensure_javascript_dependencies(workspace: &Path, app: &App) -> Result<()> {
    let manifest_path = app.frontend.join("package.json");
    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path)?).map_err(|error| {
            format!(
                "invalid package manifest {}: {error}",
                manifest_path.display()
            )
        })?;
    let mut missing = dependency_names(&manifest)
        .into_iter()
        .filter(|package| !installed_package_manifest(workspace, app, package))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    missing.sort();
    missing.dedup();
    if missing.is_empty() {
        return Ok(());
    }

    Err(format!(
        "JavaScript dependencies are not installed for {}:\n  - {}\nrun `bun install --frozen-lockfile` from {} and retry",
        app.root.display(),
        missing.join("\n  - "),
        workspace.display(),
    )
    .into())
}

fn collect_dist_exports(value: &Value, output: &mut Vec<String>) {
    match value {
        Value::String(path) if path.starts_with("./dist/") => output.push(path[2..].to_owned()),
        Value::Array(values) => {
            for value in values {
                collect_dist_exports(value, output);
            }
        }
        Value::Object(values) => {
            for (condition, value) in values {
                if condition != "types" && condition != "wabou-source" {
                    collect_dist_exports(value, output);
                }
            }
        }
        _ => {}
    }
}

/// Catch interrupted workspace package builds before Vite turns a missing
/// tracked entrypoint into an opaque `externalize-deps` resolution failure.
pub(super) fn ensure_workspace_package_exports(workspace: &Path) -> Result<()> {
    let mut missing = Vec::new();
    for packages in [
        workspace.join("packages"),
        workspace.join("vendor/wabou/packages"),
    ] {
        if !packages.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&packages)? {
            let package = entry?.path();
            let manifest_path = package.join("package.json");
            if !manifest_path.is_file() {
                continue;
            }
            let manifest: Value = serde_json::from_str(&fs::read_to_string(&manifest_path)?)
                .map_err(|error| {
                    format!(
                        "invalid package manifest {}: {error}",
                        manifest_path.display()
                    )
                })?;
            let Some(_package_name) = manifest
                .get("name")
                .and_then(Value::as_str)
                .filter(|name| name.starts_with("@wabou/"))
            else {
                continue;
            };
            let mut exports = Vec::new();
            if let Some(value) = manifest.get("exports") {
                collect_dist_exports(value, &mut exports);
            }
            exports.sort();
            exports.dedup();
            missing.extend(
                exports
                    .into_iter()
                    .map(|path| package.join(path))
                    .filter(|path| !path.is_file()),
            );
        }
    }
    if !missing.is_empty() {
        missing.sort();
        let paths = missing
            .iter()
            .map(|path| {
                path.strip_prefix(workspace)
                    .unwrap_or(path)
                    .display()
                    .to_string()
            })
            .collect::<Vec<_>>()
            .join("\n  - ");
        return Err(format!(
            "Wabou workspace JavaScript package artifacts are missing:\n  - {paths}\nrun `bun run packages:build` from {} and retry",
            workspace.display()
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(root: &Path) -> App {
        App {
            name: "gallery".into(),
            root: root.join("apps/gallery"),
            frontend: root.join("apps/gallery"),
            entry: "ui/index.tsx".into(),
        }
    }

    #[test]
    fn javascript_dependency_preflight_reports_missing_workspace_links() {
        let root = tempfile::tempdir().unwrap();
        let app = test_app(root.path());
        fs::create_dir_all(&app.frontend).unwrap();
        fs::write(
            app.frontend.join("package.json"),
            r#"{
                "dependencies": {"@wabou/vite": "workspace:*"},
                "devDependencies": {"@inlang/paraglide-js": "2.24.1"}
            }"#,
        )
        .unwrap();

        let error = ensure_javascript_dependencies(root.path(), &app).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("@wabou/vite"));
        assert!(message.contains("@inlang/paraglide-js"));
        assert!(message.contains("bun install --frozen-lockfile"));
    }

    #[test]
    fn javascript_dependency_preflight_accepts_root_and_app_installations() {
        let root = tempfile::tempdir().unwrap();
        let app = test_app(root.path());
        fs::create_dir_all(&app.frontend).unwrap();
        fs::write(
            app.frontend.join("package.json"),
            r#"{
                "dependencies": {"@wabou/vite": "workspace:*"},
                "devDependencies": {"local-tool": "1.0.0"}
            }"#,
        )
        .unwrap();
        for manifest in [
            root.path().join("node_modules/@wabou/vite/package.json"),
            app.frontend.join("node_modules/local-tool/package.json"),
        ] {
            fs::create_dir_all(manifest.parent().unwrap()).unwrap();
            fs::write(manifest, "{}").unwrap();
        }

        ensure_javascript_dependencies(root.path(), &app).unwrap();
    }
}
