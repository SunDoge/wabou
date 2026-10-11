use std::{fs, path::Path};
fn write(path: &Path, text: &str) -> Result<(), String> {
    fs::write(path, text).map_err(|e| e.to_string())
}
pub fn create(
    path: &Path,
    sdk: Option<&Path>,
    wabou_native: bool,
    native_sdk: Option<&Path>,
    command_name: &str,
) -> Result<(), String> {
    if path.symlink_metadata().is_ok() {
        return Err(format!(
            "{} already exists; no files were changed",
            path.display()
        ));
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("application path needs a UTF-8 name")?;
    if !name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err("application name must start with a lowercase letter and contain lowercase letters, digits, - or _".into());
    }
    if matches!(
        name.replace('_', "-").as_str(),
        "blockui"
            | "blockui-abi"
            | "blockui-macros"
            | "blockui-web"
            | "blockui-test"
            | "blockui-runtime"
            | "blockui-motion"
            | "blockui-dev"
            | "ui-contract"
            | "wabou-native"
    ) {
        return Err(
            "application name conflicts with a framework dependency; choose another name".into(),
        );
    }
    let sdk = sdk
        .map(Path::to_owned)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
    let sdk = sdk
        .canonicalize()
        .map_err(|e| format!("cannot find SDK checkout: {e}"))?;
    for crate_name in ["blockui", "blockui-web", "blockui-test"] {
        if !sdk
            .join("crates")
            .join(crate_name)
            .join("Cargo.toml")
            .is_file()
        {
            return Err(format!("{} is not a BlockUI SDK checkout", sdk.display()));
        }
    }
    if wabou_native && !sdk.join("crates/wabou-native/Cargo.toml").is_file() {
        return Err("selected SDK checkout does not contain wabou-native".into());
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let stage = tempfile::tempdir_in(parent).map_err(|e| e.to_string())?;
    fs::create_dir(stage.path().join("src")).map_err(|e| e.to_string())?;
    fs::create_dir(stage.path().join("web")).map_err(|e| e.to_string())?;
    let quote = |p: std::path::PathBuf| serde_json::to_string(&p.to_string_lossy()).unwrap();
    let sdk_name = if wabou_native {
        "wabou-native"
    } else {
        "blockui"
    };
    let facade_path = native_sdk
        .map(Path::to_owned)
        .unwrap_or_else(|| sdk.join("crates").join(sdk_name));
    let facade_path = facade_path
        .canonicalize()
        .map_err(|e| format!("cannot find native SDK: {e}"))?;
    if !facade_path.join("Cargo.toml").is_file() {
        return Err("native SDK path must contain Cargo.toml".into());
    }
    let sdk_path = quote(facade_path);
    let test_dependency = if wabou_native {
        format!("wabou-native = {{ path = {sdk_path}, features = [\"testing\"] }}")
    } else {
        format!(
            "blockui-test = {{ path = {} }}",
            quote(sdk.join("crates/blockui-test"))
        )
    };
    write(
        &stage.path().join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
[workspace]
[lib]
crate-type = ["cdylib", "rlib"]
[dependencies]
{sdk_name} = {{ path = {}, features = ["macros"] }}
[target.'cfg(target_arch = "wasm32")'.dependencies]
blockui-web = {{ path = {} }}
wasm-bindgen = "=0.2.127"
[dev-dependencies]
{test_dependency}
[package.metadata.blockui]
web-assets = "web"
web-shared-assets = true
[profile.dev]
debug = 0
[profile.release]
strip = "debuginfo"
"#,
            sdk_path,
            quote(sdk.join("crates/blockui-web"))
        ),
    )?;
    write(
        &stage.path().join("src/lib.rs"),
        &include_str!("../assets/counter.rs")
            .replace(
                "use blockui::*;",
                if wabou_native {
                    "use wabou_native::*;"
                } else {
                    "use blockui::*;"
                },
            )
            .replace(
                "use blockui_test::",
                if wabou_native {
                    "use wabou_native::testing::"
                } else {
                    "use blockui_test::"
                },
            ),
    )?;
    let index = include_str!("../assets/index.html")
        .replace("web_demo.js", &format!("{}.js", name.replace('-', "_")))
        .replace("./style.css", "./pkg/base.css")
        .replace("./renderer.js", "./pkg/renderer.js");
    write(&stage.path().join("web/index.html"), &index)?;
    write(
        &stage.path().join(".gitignore"),
        "/target/\n/.runtime/\n/web/pkg/\n",
    )?;
    write(
        &stage.path().join("README.md"),
        &format!(
            "# {name}\n\nThis app uses the local SDK checkout at `{}`.\n\n```sh\nblockui dev\nblockui run\nblockui build --release\nblockui check {name}\nblockui check {name} --tests\nblockui test {name} --lib --filter increment\nblockui test\ncargo fmt\nblockui dev --web\n```\n\nNative development hot-reloads the UI plugin. Browser dev hot-reloads Wasm while preserving actor state; browser builds are static releases.\nInstall wasm32-unknown-unknown and the matching wasm-bindgen CLI before using --web.\n\n## Reactive updates\n\nThe generated counter uses `#[app_impl]` and `#[ui]` blocks with ordinary Rust widget expressions. No extra `Render` implementation, `#[view]` annotation or `Ui` parameter is needed. `live!(format!(\"Count: {{}}\", self.count.get()))` installs a text observer: only that text updates when count changes. Read state inside `live!` to subscribe; a value read beforehand is a snapshot. The same `live!` marker works in `.style(...)` for an independent style observer. Use `cargo fmt` to format the source.\n\nFor conditional children, mount `live!(if self.open.get() {{ self.details() }} else {{ View::empty() }}).mount(\"details\")` with `#[ui]`. A helper method containing `#[ui]` markers is inferred by `#[app_impl]`; call it as an ordinary Rust method. Reads inside the mounted expression are dynamically collected for that subtree. Give repeated list items business keys so their state survives reordering.\n\nThe generated headless test checks text evaluation counts, preserved state and stable locators across compiled-in implementation replacement. Real dylib loading is a separate native development check. Run `blockui test {name} --lib --filter increment` for the generated counter test: the explicit package and library selection skips metadata discovery. Run `blockui test` for the full application test selection. Both run without opening a window or initializing the renderer. Use `blockui check {name}` for library type feedback and `blockui check {name} --tests` to include test types without linking or running them; explicit package names also skip check metadata discovery.\n",
            sdk.display()
        ),
    )?;
    if wabou_native {
        let manifest_path = stage.path().join("Cargo.toml");
        let mut manifest = fs::read_to_string(&manifest_path).map_err(|e| e.to_string())?;
        manifest.push_str("\n[package.metadata.wabou]\nfrontend = \"rust\"\n");
        manifest.push_str(&format!("\n[features]\nnative = [\"wabou-native/native\"]\n\n[[bin]]\nname = \"{name}\"\npath = \"src/main.rs\"\nrequired-features = [\"native\"]\n"));
        write(&manifest_path, &manifest)?;
        let library_path = stage.path().join("src/lib.rs");
        let source = fs::read_to_string(&library_path).map_err(|e| e.to_string())?;
        write(
            &library_path,
            &source.replace("struct Counter", "pub struct Counter"),
        )?;
        write(
            &stage.path().join("src/main.rs"),
            &include_str!("../assets/counter-main.rs")
                .replace("counter_app", &name.replace('-', "_"))
                .replace("APP_TITLE", name),
        )?;
    }
    if wabou_native {
        let readme_path = stage.path().join("README.md");
        let mut readme = fs::read_to_string(&readme_path).map_err(|e| e.to_string())?;
        readme.push_str("\n## Wabou native SDK\n\nThis project uses the prototype `wabou-native` facade. UI imports come from `wabou_native`; headless tests use `wabou_native::testing`, enabled through dev-dependencies. Development commands use the CLI entry shown above. Use `blockui build --static --release` or `blockui run --static --release` for the linked native binary; `blockui dev` continues to use dylib hot reload.\n");
        write(&readme_path, &readme)?;
    }
    let readme_path = stage.path().join("README.md");
    let readme = fs::read_to_string(&readme_path).map_err(|e| e.to_string())?;
    write(
        &readme_path,
        &readme.replace("blockui ", &format!("{command_name} ")),
    )?;
    fs::rename(stage.path(), path).map_err(|e| e.to_string())?;
    println!(
        "Created {}\n  cd {}\n  {command_name} dev",
        path.display(),
        path.display()
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framework_names_are_rejected_before_creating_files() {
        let directory = tempfile::tempdir().unwrap();
        for name in [
            "blockui",
            "blockui-abi",
            "blockui_macros",
            "blockui-web",
            "blockui_test",
            "blockui-runtime",
            "blockui_motion",
            "blockui-dev",
            "ui_contract",
            "wabou_native",
        ] {
            let path = directory.path().join(name);
            let error = create(
                &path,
                Some(Path::new("missing-sdk")),
                false,
                None,
                "blockui",
            )
            .unwrap_err();
            assert!(error.contains("framework dependency"), "{name}: {error}");
            assert!(!path.exists());
        }
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
    }
    #[test]
    fn never_overwrites_existing_projects() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("keep"), "owned").unwrap();
        assert!(create(dir.path(), None, false, None, "blockui").is_err());
        assert_eq!(
            fs::read_to_string(dir.path().join("keep")).unwrap(),
            "owned"
        );
    }
}
