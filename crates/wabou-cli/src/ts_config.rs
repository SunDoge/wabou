//! Evaluate trusted development configuration with Bun; runtime consumes only JSON.
use super::config::PackageConfig;
use super::{App, Result};
use serde::Deserialize;
use std::{fs, path::PathBuf, process::Command};

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Ui {
    entry: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Bindings {
    pub(super) output: PathBuf,
    pub(super) sources: Vec<PathBuf>,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            output: PathBuf::from("ui/generated/rust-bindings.d.ts"),
            sources: vec![PathBuf::from("src")],
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Configuration {
    #[serde(default, rename = "window")]
    _window: Option<serde_json::Value>,
    #[serde(default)]
    ui: Ui,
    #[serde(default)]
    entry: Option<String>,
    #[serde(rename = "sourceMap")]
    #[serde(default)]
    pub(super) source_map: Option<bool>,
    #[serde(default)]
    pub(super) package: Option<PackageConfig>,
    #[serde(default)]
    pub(super) bindings: Bindings,
}
pub(super) fn load(app: &App, command: &str, mode: &str) -> Result<Option<Configuration>> {
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
const exported = await module.default;
const authored = exported?.__wabouConfig ?? exported;
const environment = { command: process.env.WABOU_CONFIG_COMMAND ?? 'serve', mode: process.env.WABOU_MODE ?? 'development', isSsrBuild: false, isPreview: false };
const config = typeof authored === 'function' ? await authored(environment) : await authored;
if (!config || typeof config !== 'object' || Array.isArray(config)) throw new Error('wabou.config.ts must export a configuration object');
const allowed = new Set(['target', 'browser', 'root', 'entry', 'outDir', 'sourceMap', 'globalName', 'vite', 'workspaceSource', 'theme', 'themeContrast', 'ignoreClasses', 'intl', 'window', 'ui', 'package', 'bindings']);
const selected = { window: config.window, ui: config.ui, entry: config.entry, sourceMap: config.sourceMap, package: config.package, bindings: config.bindings };
for (const key of Object.keys(config)) {
  if (!allowed.has(key)) selected[key] = config[key];
}
writeFileSync(process.argv[2], JSON.stringify(selected, (key, value) => {
  if (typeof value === 'function' || typeof value === 'symbol' || typeof value === 'bigint' ||
      (typeof value === 'number' && !Number.isFinite(value))) {
    throw new Error(`Configuration field ${key || '<root>'} must contain JSON data`);
  }
  return value;
}));
"#;
    let status = Command::new("bun")
        .current_dir(&app.root)
        .env("WABOU_CONFIG_COMMAND", command)
        .env("WABOU_MODE", mode)
        .args(["--eval", script])
        .arg(&path)
        .arg(output.path())
        .status()?;
    super::ensure(status, "Wabou TS configuration")?;
    let config = serde_json::from_slice(&fs::read(output.path())?)?;
    Ok(Some(config))
}

pub(super) fn entry(app: &App, command: &str, mode: &str) -> Result<Option<String>> {
    let Some(config) = load(app, command, mode)? else {
        return Ok(None);
    };
    let entry = config
        .entry
        .or(config.ui.entry)
        .unwrap_or_else(|| app.entry.clone());
    if !app.root.join(&entry).is_file() {
        return Err(format!("UI entry does not exist: {entry}").into());
    }
    Ok(Some(entry))
}

pub(super) fn bindings(app: &App) -> Result<Bindings> {
    Ok(load(app, "bindings", "development")?
        .map(|config| config.bindings)
        .unwrap_or_default())
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
const authored = async () => ({ window: { title: 'Configured', width: 900, decorations: false }, ui: { entry: './custom.tsx' } });
const viteConfig = () => ({ plugins: [() => {}] });
Object.defineProperty(viteConfig, '__wabouConfig', { value: authored });
export default viteConfig;
"#).unwrap();
        assert_eq!(
            entry(&app, "serve", "development").unwrap().as_deref(),
            Some("./custom.tsx")
        );
    }
    #[test]
    fn top_level_entry_matches_the_vite_configuration_contract() {
        let directory = tempfile::tempdir().unwrap();
        let app = app(directory.path());
        fs::write(
            directory.path().join("wabou.config.ts"),
            "export default ({ mode }) => ({ entry: mode === 'fixture' ? './custom.tsx' : './missing.tsx' });",
        )
        .unwrap();
        assert_eq!(
            entry(&app, "build", "fixture").unwrap().as_deref(),
            Some("./custom.tsx")
        );
    }

    #[test]
    fn reads_source_binding_paths_from_the_shared_configuration() {
        let directory = tempfile::tempdir().unwrap();
        let app = app(directory.path());
        fs::write(
            directory.path().join("wabou.config.ts"),
            "export default { bindings: { output: 'types/native.d.ts', sources: ['src', '../api/src'] } };",
        )
        .unwrap();
        let bindings = bindings(&app).unwrap();
        assert_eq!(bindings.output, PathBuf::from("types/native.d.ts"));
        assert_eq!(
            bindings.sources,
            [PathBuf::from("src"), PathBuf::from("../api/src")]
        );
    }
    #[test]
    fn rejects_non_data_configuration_values() {
        let directory = tempfile::tempdir().unwrap();
        let app = app(directory.path());
        for expression in ["() => 900", "NaN", "Infinity", "Symbol('width')", "900n"] {
            fs::write(directory.path().join("wabou.config.ts"), format!(
                "export default {{ ui: {{ entry: './custom.tsx' }}, window: {{ width: {expression} }} }};"
            )).unwrap();
            let message = entry(&app, "serve", "development")
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
        let message = entry(&app, "serve", "development")
            .expect_err("accepted misspelled field")
            .to_string();
        assert!(message.contains("unknown field `windows`"), "{message}");
        fs::write(
            directory.path().join("wabou.config.ts"),
            "export default { ui: { entry: 'missing.tsx' } };",
        )
        .unwrap();
        let message = entry(&app, "serve", "development")
            .expect_err("accepted missing entry")
            .to_string();
        assert!(message.contains("UI entry does not exist"), "{message}");
    }
}
