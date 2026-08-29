use std::fs;
use std::path::{Component, Path, PathBuf};

use super::Result;
use super::config::distribution_root;
use super::project::App;

fn safe_descendant(base: &Path, target: &Path) -> Result<()> {
    let relative = target.strip_prefix(base).map_err(|_| {
        format!(
            "refusing to clean {} because it is outside {}",
            target.display(),
            base.display()
        )
    })?;
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("refusing to clean unsafe path {}", target.display()).into());
    }
    Ok(())
}

fn remove_generated(base: &Path, target: &Path, removed: &mut Vec<PathBuf>) -> Result<()> {
    safe_descendant(base, target)?;
    let metadata = match fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(target)?;
    } else {
        fs::remove_dir_all(target)?;
    }
    removed.push(target.to_path_buf());
    Ok(())
}

fn package_dist_dirs(workspace: &Path) -> Result<Vec<PathBuf>> {
    let packages = workspace.join("packages");
    if !packages.is_dir() {
        return Ok(Vec::new());
    }
    let mut outputs = Vec::new();
    for entry in fs::read_dir(&packages)? {
        let package = entry?.path();
        if package.join("package.json").is_file() {
            outputs.push(package.join("dist"));
        }
    }
    outputs.sort();
    Ok(outputs)
}

pub(super) fn run(workspace: &Path, app: &App, packages: bool) -> Result<Vec<PathBuf>> {
    let mut removed = Vec::new();
    remove_generated(workspace, &distribution_root(workspace, app)?, &mut removed)?;

    for vite_cache in [
        app.frontend.join("node_modules/.vite"),
        workspace.join("node_modules/.vite"),
    ] {
        let base = if vite_cache.starts_with(&app.frontend) {
            app.frontend.as_path()
        } else {
            workspace
        };
        remove_generated(base, &vite_cache, &mut removed)?;
    }
    remove_generated(
        workspace,
        &workspace
            .join("target/wabou/frontend")
            .join(app.name.as_str()),
        &mut removed,
    )?;

    if packages {
        for output in package_dist_dirs(workspace)? {
            remove_generated(workspace, &output, &mut removed)?;
        }
    }

    if removed.is_empty() {
        println!("[wabou] JavaScript artifacts are already clean");
    } else {
        for path in &removed {
            println!("[wabou] removed {}", path.display());
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(root: &Path) -> App {
        App {
            name: "gallery".into(),
            root: root.join("apps/gallery"),
            frontend: root.join("apps/gallery"),
            entry: "ui/index.tsx".into(),
        }
    }

    #[test]
    fn removes_app_outputs_caches_and_optional_package_dist() {
        let root = tempfile::tempdir().unwrap();
        let app = app(root.path());
        for path in [
            root.path().join("dist/gallery/debug/resources"),
            root.path().join("node_modules/.vite/deps"),
            root.path().join("target/wabou/frontend/gallery"),
            root.path().join("packages/core/dist"),
        ] {
            fs::create_dir_all(path).unwrap();
        }
        fs::create_dir_all(root.path().join("packages/core")).unwrap();
        fs::write(root.path().join("packages/core/package.json"), "{}").unwrap();

        let removed = run(root.path(), &app, true).unwrap();

        assert!(removed.contains(&root.path().join("dist/gallery")));
        assert!(removed.contains(&root.path().join("node_modules/.vite")));
        assert!(removed.contains(&root.path().join("packages/core/dist")));
        assert!(!root.path().join("dist/gallery").exists());
        assert!(!root.path().join("packages/core/dist").exists());
    }

    #[test]
    fn preserves_package_artifacts_by_default() {
        let root = tempfile::tempdir().unwrap();
        let app = app(root.path());
        let dist = root.path().join("packages/core/dist");
        fs::create_dir_all(&dist).unwrap();
        fs::write(root.path().join("packages/core/package.json"), "{}").unwrap();

        run(root.path(), &app, false).unwrap();

        assert!(dist.exists());
    }

    #[test]
    fn refuses_configured_outputs_outside_the_workspace() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let app = app(root.path());
        fs::create_dir_all(&app.root).unwrap();
        fs::write(
            app.root.join("wabou.toml"),
            format!(
                "[package]\nproduct-name = \"Gallery\"\nidentifier = \"dev.wabou.gallery\"\n\n[build]\nout-dir = {:?}\n",
                outside.path().join("resources")
            ),
        )
        .unwrap();

        let error = run(root.path(), &app, false).unwrap_err().to_string();
        assert!(error.contains("outside"));
        assert!(outside.path().exists());
    }
}
