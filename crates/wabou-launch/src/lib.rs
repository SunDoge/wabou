//! Lightweight configuration for the CLI's frontend-only fallback host.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Launch contract revision.
pub const LAUNCH_VERSION: u32 = 1;

/// UI source loaded by the resident runtime.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum UiSource {
    /// Production bundle.
    Bundle { path: PathBuf },
    /// Vite entry used during development.
    Vite { url: String, entry: String },
}

/// Window and source configuration for a frontend-only preview.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LaunchConfig {
    /// Launch contract revision.
    pub version: u32,
    /// Native window title.
    pub title: String,
    /// Initial logical width.
    pub width: u32,
    /// Initial logical height.
    pub height: u32,
    /// Whether the operating system draws native decorations.
    #[serde(
        default = "native_decorations",
        skip_serializing_if = "decorations_enabled"
    )]
    pub decorations: bool,
    /// JavaScript source loaded by the host.
    pub source: UiSource,
    /// Base directory for relative UI resources.
    #[serde(default)]
    pub resource_directory: Option<PathBuf>,
}

impl LaunchConfig {
    /// Create a default window configuration for a source.
    pub fn new(source: UiSource) -> Self {
        Self {
            version: LAUNCH_VERSION,
            title: "Wabou".into(),
            width: 800,
            height: 600,
            decorations: true,
            source,
            resource_directory: None,
        }
    }

    /// Resolve source and resource paths relative to a configuration directory.
    pub fn resolve_paths(&mut self, directory: &Path) {
        fn resolve(path: &mut PathBuf, directory: &Path) {
            if path.is_relative() {
                *path = directory.join(&*path);
            }
        }
        match &mut self.resource_directory {
            Some(path) => resolve(path, directory),
            None => self.resource_directory = Some(directory.to_owned()),
        }
        if let UiSource::Bundle { path } = &mut self.source {
            resolve(path, directory);
        }
    }
}

fn native_decorations() -> bool {
    true
}
fn decorations_enabled(value: &bool) -> bool {
    *value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_decorations_default_to_native_and_custom_chrome_round_trips() {
        let mut config = LaunchConfig::new(UiSource::Bundle {
            path: "bundle.js".into(),
        });
        let native = serde_json::to_value(&config).unwrap();
        assert!(native.get("decorations").is_none());
        assert!(
            serde_json::from_value::<LaunchConfig>(native)
                .unwrap()
                .decorations
        );
        config.decorations = false;
        let custom = serde_json::to_value(&config).unwrap();
        assert_eq!(custom["decorations"], false);
        assert!(
            !serde_json::from_value::<LaunchConfig>(custom)
                .unwrap()
                .decorations
        );
    }
}
