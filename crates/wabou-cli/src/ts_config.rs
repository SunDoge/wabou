//! Evaluate trusted development configuration with Bun; runtime consumes only JSON.
use super::{App, Result};
use serde::Deserialize;
use std::{fs, process::Command};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Window {
    title: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    decorations: Option<bool>,
}
#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Ui {
    entry: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Configuration {
    #[serde(default)]
    window: Window,
    #[serde(default)]
    ui: Ui,
}
pub(super) struct Prepared {
    pub file: tempfile::NamedTempFile,
    pub entry: String,
}
pub(super) fn prepare(app: &App) -> Result<Option<Prepared>> {
    let path = app.root.join("wabou.config.ts");
    if !path.is_file() {
        return Ok(None);
    }
    let path = path.canonicalize()?;
    let output = tempfile::NamedTempFile::new()?;
    let script = r#"
import { pathToFileURL } from 'node:url';
import { writeFileSync } from 'node:fs';
const module = await import(pathToFileURL(process.argv[1]).href);
const config = typeof module.default === 'function' ? await module.default() : await module.default;
if (!config || typeof config !== 'object' || Array.isArray(config)) throw new Error('wabou.config.ts must export a configuration object');
writeFileSync(process.argv[2], JSON.stringify(config, (key, value) => {
  if (typeof value === 'function' || typeof value === 'symbol' || typeof value === 'bigint' ||
      (typeof value === 'number' && !Number.isFinite(value))) {
    throw new Error(`Configuration field ${key || '<root>'} must contain JSON data`);
  }
  return value;
}));
"#;
    let status = Command::new("bun")
        .current_dir(&app.root)
        .args(["--eval", script])
        .arg(&path)
        .arg(output.path())
        .status()?;
    super::ensure(status, "Wabou TS configuration")?;
    let config: Configuration = serde_json::from_slice(&fs::read(output.path())?)?;
    let entry = config.ui.entry.unwrap_or_else(|| app.entry.clone());
    if !app.root.join(&entry).is_file() {
        return Err(format!("UI entry does not exist: {entry}").into());
    }
    let mut launch = wabou_launch::LaunchConfig::new(wabou_launch::UiSource::Bundle {
        path: entry.clone().into(),
    });
    launch.title = config.window.title.unwrap_or(launch.title);
    launch.width = config.window.width.unwrap_or(launch.width);
    launch.height = config.window.height.unwrap_or(launch.height);
    launch.decorations = config.window.decorations.unwrap_or(launch.decorations);
    if launch.width == 0 || launch.height == 0 {
        return Err("window dimensions must be positive".into());
    }
    launch.resolve_paths(path.parent().ok_or("configuration has no directory")?);
    fs::write(output.path(), serde_json::to_vec(&launch)?)?;
    Ok(Some(Prepared {
        file: output,
        entry,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app(directory: &std::path::Path) -> App {
        fs::write(directory.join("custom.tsx"), "export {};").unwrap();
        App {
            name: "config-test".into(),
            root: directory.to_owned(),
            frontend: directory.to_owned(),
            entry: "ui/index.tsx".into(),
        }
    }
    #[test]
    fn evaluates_async_factory_and_resolves_paths_against_authored_config() {
        let directory = tempfile::tempdir().unwrap();
        let app = app(directory.path());
        fs::write(directory.path().join("wabou.config.ts"), r#"
console.log('config logs must not corrupt the protocol');
export default async () => ({ window: { title: 'Configured', width: 900, decorations: false }, ui: { entry: './custom.tsx' } });
"#).unwrap();
        let prepared = prepare(&app).unwrap().unwrap();
        assert_eq!(prepared.entry, "./custom.tsx");
        let config: wabou_launch::LaunchConfig =
            serde_json::from_slice(&fs::read(prepared.file.path()).unwrap()).unwrap();
        assert_eq!(config.title, "Configured");
        assert_eq!(config.width, 900);
        assert_eq!(config.height, 600);
        assert!(!config.decorations);
    }
    #[test]
    fn rejects_non_data_configuration_values() {
        let directory = tempfile::tempdir().unwrap();
        let app = app(directory.path());
        for expression in ["() => 900", "NaN", "Infinity", "Symbol('width')", "900n"] {
            fs::write(directory.path().join("wabou.config.ts"), format!(
                "export default {{ ui: {{ entry: './custom.tsx' }}, window: {{ width: {expression} }} }};"
            )).unwrap();
            let message = prepare(&app)
                .err()
                .unwrap_or_else(|| panic!("accepted {expression}"))
                .to_string();
            assert!(
                message.contains("Wabou TS configuration failed"),
                "unexpected error for {expression}: {message}"
            );
        }
    }
    #[test]
    fn rejects_misspelled_properties_and_missing_entries() {
        let directory = tempfile::tempdir().unwrap();
        let app = app(directory.path());
        fs::write(
            directory.path().join("wabou.config.ts"),
            "export default { windows: {} };",
        )
        .unwrap();
        let message = prepare(&app)
            .err()
            .expect("accepted misspelled field")
            .to_string();
        assert!(message.contains("unknown field `windows`"), "{message}");
        fs::write(
            directory.path().join("wabou.config.ts"),
            "export default { ui: { entry: 'missing.tsx' } };",
        )
        .unwrap();
        let message = prepare(&app)
            .err()
            .expect("accepted missing entry")
            .to_string();
        assert!(message.contains("UI entry does not exist"), "{message}");
    }
}
