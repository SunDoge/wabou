//! Standalone project scaffolding for Git-based preview releases.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(crate) const DEFAULT_REVISION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

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
    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("ui"))?;
    fs::write(
        root.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[dependencies]
wabou = {{ path = "vendor/wabou/crates/wabou", features = ["vite"] }}
"#,
        ),
    )?;
    fs::write(
        root.join("src/main.rs"),
        format!(
            r#"use wabou::{{HostBuilder, WindowOptions}};

fn main() -> wabou::Result<()> {{
    HostBuilder::new()
        .window(WindowOptions::new().title("{name}"))
        .run()
}}
"#,
        ),
    )?;
    fs::write(
        root.join("package.json"),
        format!(
            r#"{{
  "name": "{name}",
  "private": true,
  "type": "module",
  "workspaces": ["vendor/wabou/packages/*"],
  "scripts": {{
    "wabou": "wabou",
    "dev": "wabou dev",
    "build": "vite build",
    "check": "tsc --noEmit"
  }},
  "dependencies": {{
    "@solidjs/web": "2.0.0-rc.0",
    "@wabou/core": "workspace:*",
    "@wabou/primitives": "workspace:*",
    "@wabou/solid-renderer": "workspace:*",
    "solid-js": "2.0.0-rc.0"
  }},
  "devDependencies": {{
    "@types/bun": "^1.3.14",
    "@wabou/vite": "workspace:*",
    "typescript": "^5.9.0",
    "vite": "^5.4.0"
  }}
}}
"#,
        ),
    )?;
    fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "jsxImportSource": "@wabou/solid-renderer",
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "skipLibCheck": true,
    "types": ["bun"]
  },
  "include": ["ui/**/*.ts", "ui/**/*.tsx"]
}
"#,
    )?;
    fs::write(
        root.join("vite.config.ts"),
        r#"import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig({
  outDir: "dist/resources",
});
"#,
    )?;
    fs::write(
        root.join("ui/index.tsx"),
        format!(
            r#"import {{ mount }} from "@wabou/core";
import {{ Column, Text }} from "@wabou/primitives";
import "virtual:wabou-stylesheet";

mount(() => (
  <Column class="h-full items-center justify-center gap-3 bg-background">
    <Text class="text-2xl font-semibold text-foreground">{name}</Text>
    <Text class="text-muted">Your Wabou application is ready.</Text>
  </Column>
));
"#,
        ),
    )?;
    fs::write(root.join(".gitignore"), "/node_modules\n/dist\n/target\n")?;
    fs::write(
        root.join("README.md"),
        format!(
            r#"# {name}

A native desktop application built with [Wabou](https://github.com/SunDoge/wabou).

```bash
git submodule update --init
bun install
bun run dev
```

The `vendor/wabou` submodule pins the Rust host and JavaScript packages to one
compatible revision. Install the CLI from that same tag. Use
`bun run wabou --help` to list development, testing, rendering, and packaging
commands.
"#,
        ),
    )?;
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
