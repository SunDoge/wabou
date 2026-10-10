//! Native development launcher. No Python process is involved.
use clap::Parser;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug, Default)]
#[command(about = "Build and run a native actor application")]
pub struct Options {
    #[arg(long, help = "Stable executable package for a separated UI crate")]
    pub host_package: Option<String>,
    #[arg(long, alias = "manifest-path")]
    pub manifest: Option<PathBuf>,
    #[arg(long, default_value = "")]
    pub features: String,
    #[arg(long)]
    pub capture: bool,
    #[arg(long)]
    pub prepare_only: bool,
    #[arg(long)]
    pub configuration_only: bool,
    #[arg(long)]
    pub target_dir: Option<PathBuf>,
    #[arg(long)]
    pub release: bool,
    #[arg(long)]
    pub build_only: bool,
    #[arg(long, help = "Open a specific 7GUIs task")]
    pub task: Option<String>,
}

/// Arguments understood by the framework's generic stable native host.
#[derive(Parser)]
pub struct HostOptions {
    pub library: PathBuf,
    #[arg(long)]
    pub watch: Option<PathBuf>,
    #[arg(long)]
    pub capture: bool,
}
impl HostOptions {
    pub fn from_env() -> Self {
        Self::parse()
    }
}

/// Interpret Cargo's graph; only reachable local packages are watched.
pub fn configuration(
    metadata: &Value,
    manifest: &Path,
    package: &str,
    target: &Path,
    features: &str,
) -> Result<Value, String> {
    configuration_profile(metadata, manifest, package, target, features, "debug")
}

pub fn configuration_profile(
    metadata: &Value,
    manifest: &Path,
    package: &str,
    target: &Path,
    features: &str,
    profile: &str,
) -> Result<Value, String> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or("missing Cargo packages")?;
    let selected: Vec<_> = packages
        .iter()
        .filter(|p| p["name"].as_str() == Some(package))
        .collect();
    if selected.len() != 1 {
        return Err(format!("Expected one package named {package}"));
    }
    let selected = selected[0];
    let targets: Vec<_> = selected["targets"]
        .as_array()
        .ok_or("missing targets")?
        .iter()
        .filter(|t| {
            t["crate_types"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|k| k == "cdylib"))
        })
        .collect();
    if targets.len() != 1 {
        return Err(format!("{package} must have one cdylib target"));
    }
    let name = targets[0]["name"].as_str().ok_or("missing cdylib name")?;
    let nodes = metadata["resolve"]["nodes"]
        .as_array()
        .ok_or("missing resolved Cargo graph")?;
    let mut pending = vec![selected["id"].as_str().ok_or("missing package id")?];
    let mut visited = BTreeSet::new();
    while let Some(id) = pending.pop() {
        if !visited.insert(id) {
            continue;
        }
        let node = nodes
            .iter()
            .find(|n| n["id"].as_str() == Some(id))
            .ok_or("missing dependency node")?;
        if let Some(dependencies) = node["deps"].as_array() {
            for dependency in dependencies {
                let relevant = dependency["dep_kinds"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|k| k["kind"] != "dev"));
                if relevant {
                    pending.push(dependency["pkg"].as_str().ok_or("invalid dependency id")?);
                }
            }
        } else {
            // Also accept the compact graph used by callers and older fixtures.
            for dependency in node["dependencies"]
                .as_array()
                .ok_or("missing dependencies")?
            {
                pending.push(dependency.as_str().ok_or("invalid dependency id")?);
            }
        }
    }
    let mut sources = Vec::new();
    for id in visited {
        let p = packages
            .iter()
            .find(|p| p["id"].as_str() == Some(id))
            .ok_or("missing dependency package")?;
        if p["source"].is_null() {
            sources.push(
                Path::new(p["manifest_path"].as_str().ok_or("missing manifest path")?)
                    .parent()
                    .ok_or("invalid manifest path")?
                    .to_owned(),
            );
        }
    }
    let workspace = Path::new(
        metadata["workspace_root"]
            .as_str()
            .ok_or("missing workspace root")?,
    );
    sources.extend([workspace.join("Cargo.toml"), workspace.join("Cargo.lock")]);
    let sources = std::env::join_paths(sources).map_err(|e| e.to_string())?;
    let sources = sources.to_str().ok_or("watch paths must be UTF-8")?;
    let artifact = target.join(profile).join(format!(
        "{}{}{}",
        std::env::consts::DLL_PREFIX,
        name,
        std::env::consts::DLL_SUFFIX
    ));
    let mut config = json!({"BLOCKUI_DEV_MANIFEST":manifest,"BLOCKUI_DEV_PACKAGE":package,
        "BLOCKUI_DEV_ARTIFACT":artifact,"BLOCKUI_DEV_TARGET":target,
        "BLOCKUI_DEV_SOURCES":sources,"BLOCKUI_DEV_FEATURES":features});
    if let Some(example) = selected["metadata"]["blockui"]["host-example"].as_str() {
        config["BLOCKUI_DEV_HOST_EXAMPLE"] = json!(example);
    }
    if let Some(host) = selected["metadata"]["blockui"]["host-package"].as_str() {
        config["BLOCKUI_DEV_HOST_PACKAGE"] = json!(host);
    }
    Ok(config)
}

mod launcher;
pub mod project;
pub use launcher::{launch, launch_project};

#[cfg(test)]
mod tests {
    use super::*;
    fn metadata() -> Value {
        json!({"workspace_root":"/project", "packages":[
            {"id":"app", "name":"app", "source":null, "manifest_path":"/project/app/Cargo.toml", "targets":[{"name":"app_impl","crate_types":["cdylib","rlib"]}]},
            {"id":"sdk", "name":"sdk", "source":null,"manifest_path":"/project/sdk/Cargo.toml"},
            {"id":"remote", "name":"remote", "source":"registry", "manifest_path":"/registry/remote/Cargo.toml"},
            {"id":"other", "name":"other", "source":null,"manifest_path":"/project/other/Cargo.toml"}
        ],"resolve":{"nodes":[
            {"id":"app","dependencies":["sdk","remote"]},
            {"id":"sdk","dependencies":["remote"]},
            {"id":"remote","dependencies":[]},
            {"id":"other","dependencies":[]}
        ]}})
    }
    #[test]
    fn only_reachable_local_sources_are_watched() {
        let config = configuration(
            &metadata(),
            Path::new("/project/Cargo.toml"),
            "app",
            Path::new("/target"),
            "variant",
        )
        .unwrap();
        let roots: Vec<_> =
            std::env::split_paths(config["BLOCKUI_DEV_SOURCES"].as_str().unwrap()).collect();
        assert_eq!(
            roots,
            vec![
                PathBuf::from("/project/app"),
                PathBuf::from("/project/sdk"),
                PathBuf::from("/project/Cargo.toml"),
                PathBuf::from("/project/Cargo.lock")
            ]
        );
        assert_eq!(config["BLOCKUI_DEV_FEATURES"], "variant");
        assert_eq!(
            Path::new(config["BLOCKUI_DEV_ARTIFACT"].as_str().unwrap())
                .file_name()
                .unwrap(),
            format!(
                "{}app_impl{}",
                std::env::consts::DLL_PREFIX,
                std::env::consts::DLL_SUFFIX
            )
            .as_str()
        );
    }
    #[test]
    fn test_only_dependencies_do_not_trigger_ui_rebuilds() {
        let mut data = metadata();
        data["resolve"]["nodes"][0]["deps"] = json!([
            {"pkg":"sdk", "dep_kinds":[{"kind":null}]},
            {"pkg":"other", "dep_kinds":[{"kind":"dev"}]}
        ]);
        let config = configuration(
            &data,
            Path::new("/project/Cargo.toml"),
            "app",
            Path::new("/target"),
            "",
        )
        .unwrap();
        assert!(
            !config["BLOCKUI_DEV_SOURCES"]
                .as_str()
                .unwrap()
                .contains("/project/other")
        );
        assert!(
            config["BLOCKUI_DEV_SOURCES"]
                .as_str()
                .unwrap()
                .contains("/project/sdk")
        );
    }
    #[test]
    fn separated_ui_selects_host_without_watching_its_business_sources() {
        let mut data = metadata();
        data["packages"][0]["metadata"] = json!({"blockui":{"host-package":"stable-host"}});
        let config = configuration(
            &data,
            Path::new("/project/Cargo.toml"),
            "app",
            Path::new("/target"),
            "",
        )
        .unwrap();
        assert_eq!(config["BLOCKUI_DEV_HOST_PACKAGE"], "stable-host");
        assert!(
            !config["BLOCKUI_DEV_SOURCES"]
                .as_str()
                .unwrap()
                .contains("/project/other")
        );
    }
    #[test]
    fn rejects_unknown_package_and_non_plugin_targets() {
        assert!(
            configuration(
                &metadata(),
                Path::new("/project/Cargo.toml"),
                "missing",
                Path::new("/target"),
                ""
            )
            .is_err()
        );
        let mut data = metadata();
        data["packages"][0]["targets"][0]["crate_types"] = json!(["bin"]);
        assert!(
            configuration(
                &data,
                Path::new("/project/Cargo.toml"),
                "app",
                Path::new("/target"),
                ""
            )
            .is_err()
        );
    }
}
