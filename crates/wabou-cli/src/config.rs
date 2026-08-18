use std::fs;
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::Deserialize;

use super::Result;
use super::project::App;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub(super) enum PackageFormat {
    App,
    Dmg,
    Nsis,
    Wix,
    Deb,
    Appimage,
    Pacman,
}

impl PackageFormat {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Dmg => "dmg",
            Self::Nsis => "nsis",
            Self::Wix => "wix",
            Self::Deb => "deb",
            Self::Appimage => "appimage",
            Self::Pacman => "pacman",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WabouPackageFile {
    package: PackageConfig,
    #[serde(default)]
    build: Option<BuildConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
struct BuildConfig {
    out_dir: PathBuf,
    #[serde(default)]
    source_map: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BuildProfile {
    Debug,
    Release,
}

impl BuildProfile {
    pub(super) fn from_release(release: bool) -> Self {
        if release { Self::Release } else { Self::Debug }
    }

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub(super) struct PackageConfig {
    pub(super) product_name: String,
    pub(super) identifier: String,
    #[serde(default)]
    pub(super) description: Option<String>,
    #[serde(default)]
    pub(super) authors: Vec<String>,
    #[serde(default)]
    pub(super) copyright: Option<String>,
    #[serde(default)]
    pub(super) license_file: Option<PathBuf>,
    #[serde(default)]
    pub(super) icons: Vec<String>,
    #[serde(default)]
    pub(super) resources: Vec<PathBuf>,
    #[serde(default)]
    pub(super) formats: Vec<PackageFormat>,
}

fn load_config_file(app: &App) -> Result<Option<WabouPackageFile>> {
    let path = app.root.join("wabou.toml");
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!("cannot read configuration {}: {error}", path.display()).into());
        }
    };
    toml::from_str(&source)
        .map(Some)
        .map_err(|error| format!("invalid {}: {error}", path.display()).into())
}

pub(super) fn configured_resource_dir(workspace: &Path, app: &App) -> Result<PathBuf> {
    if let Some(build) = load_config_file(app)?.and_then(|file| file.build) {
        return Ok(app.root.join(build.out_dir));
    }
    let dist = workspace.join("dist");
    Ok(if workspace == app.root {
        dist.join("resources")
    } else {
        dist.join(&app.name).join("resources")
    })
}

pub(super) fn profile_resource_dir(
    workspace: &Path,
    app: &App,
    profile: BuildProfile,
) -> Result<PathBuf> {
    let configured = configured_resource_dir(workspace, app)?;
    let name = configured.file_name().unwrap_or_default();
    Ok(configured
        .parent()
        .unwrap_or(&configured)
        .join(profile.as_str())
        .join(name))
}

pub(super) fn distribution_root(workspace: &Path, app: &App) -> Result<PathBuf> {
    let resources = configured_resource_dir(workspace, app)?;
    Ok(resources.parent().unwrap_or(&resources).to_path_buf())
}

pub(super) fn profile_application_dir(
    workspace: &Path,
    app: &App,
    profile: BuildProfile,
) -> Result<PathBuf> {
    Ok(distribution_root(workspace, app)?.join(profile.as_str()))
}

pub(super) fn bundle_path(workspace: &Path, app: &App, profile: BuildProfile) -> Result<PathBuf> {
    Ok(profile_resource_dir(workspace, app, profile)?.join("bundle.js"))
}

pub(super) fn configured_source_map(app: &App, profile: BuildProfile) -> Result<bool> {
    let setting = load_config_file(app)?
        .and_then(|file| file.build)
        .and_then(|build| build.source_map);
    Ok(setting.unwrap_or(profile == BuildProfile::Debug))
}

pub(super) fn load_package_config(app: &App) -> Result<PackageConfig> {
    let path = app.root.join("wabou.toml");
    let file = load_config_file(app)?.ok_or_else(|| {
        format!(
            "cannot read package configuration {}: file not found",
            path.display()
        )
    })?;
    if file.package.product_name.trim().is_empty() {
        return Err("package.product-name cannot be empty".into());
    }
    let identifier = file.package.identifier.as_str();
    if !identifier.contains('.')
        || !identifier
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-'))
    {
        return Err("package.identifier must be a reverse-domain identifier".into());
    }
    Ok(file.package)
}
