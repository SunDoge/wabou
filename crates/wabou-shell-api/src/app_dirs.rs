//! Resolution of application-private paths using platform conventions.

#![warn(missing_docs)]

use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Stable identity used to derive operating-system-standard private directories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppDirectoryConfig {
    qualifier: String,
    organization: String,
    application: String,
}

impl AppDirectoryConfig {
    /// Define the stable reverse-domain-style identity used by the OS.
    ///
    /// Changing these values after release changes the resolved directories
    /// and therefore appears to users as lost application data.
    pub fn new(
        qualifier: impl Into<String>,
        organization: impl Into<String>,
        application: impl Into<String>,
    ) -> Self {
        Self {
            qualifier: qualifier.into(),
            organization: organization.into(),
            application: application.into(),
        }
    }
}

/// Absolute roots for application-owned files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDirectories {
    /// User-editable configuration directory.
    pub config_dir: PathBuf,
    /// Roaming or user-level durable data directory.
    pub data_dir: PathBuf,
    /// Machine-local durable data directory.
    pub local_data_dir: PathBuf,
    /// Reconstructible cache directory.
    pub cache_dir: PathBuf,
    /// Default directory for application logs.
    pub log_dir: PathBuf,
    /// Read-only resources shipped beside the application.
    pub resource_dir: PathBuf,
    /// Application-namespaced temporary directory.
    pub temp_dir: PathBuf,
}

impl AppDirectories {
    /// Resolve platform paths, returning `None` when no home directory exists.
    ///
    /// This function only computes paths; it does not create directories.
    pub fn resolve(config: &AppDirectoryConfig, resource: impl AsRef<Path>) -> Option<Self> {
        if let Some(root) = std::env::var_os("WABOU_TEST_APP_DATA_ROOT").map(PathBuf::from) {
            return root.is_absolute().then(|| Self::isolated(root, resource));
        }
        let project =
            ProjectDirs::from(&config.qualifier, &config.organization, &config.application)?;
        Some(Self {
            config_dir: project.config_dir().to_owned(),
            data_dir: project.data_dir().to_owned(),
            local_data_dir: project.data_local_dir().to_owned(),
            cache_dir: project.cache_dir().to_owned(),
            log_dir: project.data_local_dir().join("logs"),
            resource_dir: resource.as_ref().to_owned(),
            temp_dir: std::env::temp_dir().join(&config.application),
        })
    }

    /// Resolve every writable directory below one explicit private root.
    ///
    /// This is primarily used by Wabou's behavior-test runner so application
    /// code that resolves its own directories cannot read or mutate real user
    /// state. The read-only resource directory remains unchanged.
    pub fn isolated(root: impl AsRef<Path>, resource: impl AsRef<Path>) -> Self {
        let root = root.as_ref();
        Self {
            config_dir: root.join("config"),
            data_dir: root.join("data"),
            local_data_dir: root.join("local-data"),
            cache_dir: root.join("cache"),
            log_dir: root.join("logs"),
            resource_dir: resource.as_ref().to_owned(),
            temp_dir: root.join("temp"),
        }
    }

    /// Default root for durable native stores such as KV and databases.
    pub fn storage(&self) -> PathBuf {
        self.local_data_dir.join("storage")
    }

    /// Namespaced durable root for one native storage capability.
    pub fn storage_namespace(&self, namespace: &str) -> Option<PathBuf> {
        is_safe_namespace(namespace).then(|| self.storage().join(namespace))
    }
}

fn is_safe_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && namespace != "."
        && namespace != ".."
        && namespace
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_namespaces_cannot_escape_local_data() {
        let dirs = AppDirectories::resolve(
            &AppDirectoryConfig::new("dev", "Wabou", "Directory Test"),
            "/opt/wabou/resources",
        )
        .expect("platform has a home directory");

        assert_eq!(dirs.resource_dir, PathBuf::from("/opt/wabou/resources"));
        assert!(
            dirs.storage_namespace("kv")
                .unwrap()
                .ends_with("storage/kv")
        );
        assert_eq!(dirs.storage_namespace("../other-app"), None);
        assert_eq!(dirs.storage_namespace("nested/path"), None);
    }

    #[test]
    fn isolated_directories_keep_every_writable_path_below_the_test_root() {
        let root = PathBuf::from("test-root");
        let resource = PathBuf::from("app-resources");
        let directories = AppDirectories::isolated(&root, &resource);

        for path in [
            &directories.config_dir,
            &directories.data_dir,
            &directories.local_data_dir,
            &directories.cache_dir,
            &directories.log_dir,
            &directories.temp_dir,
        ] {
            assert!(path.starts_with(&root));
        }
        assert_eq!(directories.resource_dir, resource);
    }
}
