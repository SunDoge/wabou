//! Standalone project scaffolding for Git-based preview releases.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(crate) const DEFAULT_REVISION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

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
        "ui/index.tsx",
        include_str!("../../../templates/basic/ui/index.tsx"),
    ),
    (
        "vite.config.ts",
        include_str!("../../../templates/basic/vite.config.ts"),
    ),
    (
        "wabou.toml",
        include_str!("../../../templates/basic/wabou.toml"),
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
    if repository.trim().is_empty() || revision.trim().is_empty() {
        return Err("Wabou repository and revision must not be empty".into());
    }
    if repository.starts_with('-') || revision.starts_with('-') {
        return Err("Wabou repository and revision must not start with `-`".into());
    }
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
        assert!(root.join("ui/index.tsx").is_file());
        for (relative, _) in TEMPLATE_FILES {
            let rendered = fs::read_to_string(root.join(relative)).unwrap();
            assert!(!rendered.contains("__WABOU_PROJECT_NAME__"));
        }
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn project_names_are_safe_for_cargo_and_package_json() {
        assert!(validate_project_name("hello-wabou_2").is_ok());
        assert!(validate_project_name("2bad").is_err());
        assert!(validate_project_name("bad/name").is_err());
        assert!(validate_project_name("bad\"name").is_err());
    }
}
