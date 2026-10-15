//! Standalone project scaffolding for Git-based preview releases.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(crate) const DEFAULT_REVISION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

pub(crate) fn default_revision() -> String {
    DEFAULT_REVISION.to_owned()
}

const TEMPLATE_FILES: &[(&str, &str)] = &[
    (
        ".gitignore",
        include_str!("../../../templates/basic/.gitignore"),
    ),
    (
        "Cargo.toml",
        include_str!("../../../templates/basic/Cargo.toml"),
    ),
    (
        "README.md",
        include_str!("../../../templates/basic/README.md"),
    ),
    (
        "package.json",
        include_str!("../../../templates/basic/package.json"),
    ),
    (
        "src/main.rs",
        include_str!("../../../templates/basic/src/main.rs"),
    ),
    (
        "tsconfig.json",
        include_str!("../../../templates/basic/tsconfig.json"),
    ),
    (
        "tests/app.behavior.ts",
        include_str!("../../../templates/basic/tests/app.behavior.ts"),
    ),
    (
        "ui/index.tsx",
        include_str!("../../../templates/basic/ui/index.tsx"),
    ),
    (
        "wabou.config.ts",
        include_str!("../../../templates/basic/wabou.config.ts"),
    ),
];

pub(crate) fn create(destination: &Path, repository: &str, revision: &str) -> Result<()> {
    if destination.exists() {
        return Err(format!(
            "cannot create {}: destination already exists",
            destination.display()
        )
        .into());
    }
    validate_source(repository, revision)?;
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("project path must end in a UTF-8 directory name")?;
    validate_project_name(name)?;

    fs::create_dir_all(destination)?;
    let result = initialize(destination, name, repository, revision);
    if let Err(error) = result {
        fs::remove_dir_all(destination)?;
        return Err(error);
    }

    println!("Created Wabou application at {}", destination.display());
    println!("  cd {}", destination.display());
    println!("  bun install");
    println!("  bun run dev");
    Ok(())
}

pub(crate) fn vendor(root: &Path, repository: &str, revision: &str) -> Result<()> {
    validate_source(repository, revision)?;
    if !root.is_dir() {
        return Err(format!("application directory does not exist: {}", root.display()).into());
    }
    let root = root.canonicalize()?;
    if git_output(&root, &["rev-parse", "--is-inside-work-tree"])?.trim() != "true" {
        return Err(format!("{} is not inside a Git worktree", root.display()).into());
    }

    let target = root.join("vendor/wabou");
    let tracked =
        git_output(&root, &["ls-files", "--stage", "--", "vendor/wabou"])?.starts_with("160000 ");
    if target.join(".git").exists() {
        ensure_clean_submodule(&target)?;
    } else if tracked {
        run_git(
            &root,
            &["submodule", "update", "--init", "--", "vendor/wabou"],
        )?;
    } else {
        if target.exists() {
            return Err(format!(
                "cannot add Wabou submodule: {} already exists",
                target.display()
            )
            .into());
        }
        fs::create_dir_all(target.parent().ok_or("submodule path has no parent")?)?;
        run_git(
            &root,
            &[
                "-c",
                "protocol.file.allow=always",
                "submodule",
                "add",
                "--quiet",
                "--",
                repository,
                "vendor/wabou",
            ],
        )?;
    }

    ensure_clean_submodule(&target)?;
    run_git(&target, &["fetch", "--quiet", "origin", "--tags"])?;
    run_git(&target, &["checkout", "--quiet", "--detach", revision])?;
    run_git(&root, &["add", "--", "vendor/wabou"])?;
    println!(
        "Vendored Wabou {revision} at {}",
        root.join("vendor/wabou").display()
    );
    Ok(())
}

fn initialize(root: &Path, name: &str, repository: &str, revision: &str) -> Result<()> {
    write_project(root, name)?;
    run_git(root, &["init", "--quiet"])?;
    run_git(
        root,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            "--quiet",
            "--",
            repository,
            "vendor/wabou",
        ],
    )?;
    run_git(
        root,
        &["-C", "vendor/wabou", "checkout", "--quiet", revision],
    )?;
    run_git(root, &["add", "."])?;
    Ok(())
}

fn validate_source(repository: &str, revision: &str) -> Result<()> {
    if repository.trim().is_empty() || revision.trim().is_empty() {
        return Err("Wabou repository and revision must not be empty".into());
    }
    if repository.starts_with('-') || revision.starts_with('-') {
        return Err("Wabou repository and revision must not start with `-`".into());
    }
    Ok(())
}

fn ensure_clean_submodule(target: &Path) -> Result<()> {
    let status = git_output(target, &["status", "--porcelain"])?;
    if status.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "refusing to update dirty Wabou submodule at {}; commit or discard its changes first",
            target.display()
        )
        .into())
    }
}

fn validate_project_name(name: &str) -> Result<()> {
    let mut chars = name.chars();
    if !chars.next().is_some_and(|ch| ch.is_ascii_alphabetic())
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return Err(format!(
            "invalid project name `{name}`; use ASCII letters, numbers, hyphens, or underscores, starting with a letter"
        )
        .into());
    }
    Ok(())
}

fn run_git(current_dir: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(current_dir)
        .status()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if !status.success() {
        return Err(format!("git {} failed with {status}", args.join(" ")).into());
    }
    Ok(())
}

fn git_output(current_dir: &Path, args: &[&str]) -> Result<String> {
    let Output {
        status,
        stdout,
        stderr,
    } = Command::new("git")
        .args(args)
        .current_dir(current_dir)
        .output()
        .map_err(|error| format!("failed to start git: {error}"))?;
    if !status.success() {
        return Err(format!(
            "git {} failed with {status}: {}",
            args.join(" "),
            String::from_utf8_lossy(&stderr).trim()
        )
        .into());
    }
    Ok(String::from_utf8(stdout)?.trim().to_owned())
}

fn write_project(root: &Path, name: &str) -> Result<()> {
    for (relative, template) in TEMPLATE_FILES {
        let destination = root.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            destination,
            template.replace("__WABOU_PROJECT_NAME__", name),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn writes_a_standalone_git_preview_project() {
        let root = env::temp_dir().join(format!("wabou-new-project-{}", std::process::id()));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        write_project(&root, "hello-wabou").unwrap();
        let cargo = fs::read_to_string(root.join("Cargo.toml")).unwrap();
        let package = fs::read_to_string(root.join("package.json")).unwrap();
        assert!(cargo.contains("vendor/wabou/crates/wabou"));
        assert!(package.contains("vendor/wabou/packages/*"));
        assert!(package.contains("\"@wabou/ui\": \"workspace:*\""));
        assert!(!package.contains("@wabou/core/renderer"));
        assert!(root.join("ui/index.tsx").is_file());
        assert!(root.join("tests/app.behavior.ts").is_file());
        for (relative, _) in TEMPLATE_FILES {
            let rendered = fs::read_to_string(root.join(relative)).unwrap();
            assert!(!rendered.contains("__WABOU_PROJECT_NAME__"));
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn adds_updates_and_protects_a_vendored_submodule() {
        let source = tempfile::tempdir().unwrap();
        run_git(source.path(), &["init", "--quiet"]).unwrap();
        run_git(source.path(), &["config", "user.name", "Wabou Test"]).unwrap();
        run_git(
            source.path(),
            &["config", "user.email", "wabou@example.invalid"],
        )
        .unwrap();
        fs::write(source.path().join("version.txt"), "one").unwrap();
        run_git(source.path(), &["add", "version.txt"]).unwrap();
        run_git(source.path(), &["commit", "--quiet", "-m", "one"]).unwrap();
        let first = git_output(source.path(), &["rev-parse", "HEAD"]).unwrap();

        let app = tempfile::tempdir().unwrap();
        run_git(app.path(), &["init", "--quiet"]).unwrap();
        vendor(app.path(), source.path().to_str().unwrap(), &first).unwrap();
        assert_eq!(
            fs::read_to_string(app.path().join("vendor/wabou/version.txt")).unwrap(),
            "one"
        );
        let staged = git_output(app.path(), &["diff", "--cached", "--name-only"]).unwrap();
        assert!(staged.lines().any(|path| path == ".gitmodules"));
        assert!(staged.lines().any(|path| path == "vendor/wabou"));

        run_git(
            app.path(),
            &["submodule", "deinit", "--force", "--", "vendor/wabou"],
        )
        .unwrap();
        assert!(!app.path().join("vendor/wabou/.git").exists());
        vendor(app.path(), source.path().to_str().unwrap(), &first).unwrap();

        fs::write(source.path().join("version.txt"), "two").unwrap();
        run_git(source.path(), &["add", "version.txt"]).unwrap();
        run_git(source.path(), &["commit", "--quiet", "-m", "two"]).unwrap();
        let second = git_output(source.path(), &["rev-parse", "HEAD"]).unwrap();
        vendor(app.path(), source.path().to_str().unwrap(), &second).unwrap();
        assert_eq!(
            fs::read_to_string(app.path().join("vendor/wabou/version.txt")).unwrap(),
            "two"
        );

        fs::write(app.path().join("vendor/wabou/version.txt"), "dirty").unwrap();
        let message = vendor(app.path(), source.path().to_str().unwrap(), &second)
            .expect_err("dirty submodule was updated")
            .to_string();
        assert!(message.contains("refusing to update dirty Wabou submodule"));
    }

    #[test]
    fn project_names_are_safe_for_cargo_and_package_json() {
        assert!(validate_project_name("hello-wabou_2").is_ok());
        assert!(validate_project_name("2bad").is_err());
        assert!(validate_project_name("bad/name").is_err());
        assert!(validate_project_name("bad\"name").is_err());
    }
}
