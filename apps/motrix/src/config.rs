use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub engine_mode: EngineMode,
    pub external_endpoint: String,
    pub external_secret: String,
    pub download_dir: String,
    pub split: i32,
    pub max_concurrent_downloads: u32,
    pub notify_on_complete: bool,
    pub notify_on_error: bool,
    pub resume_all_when_app_launched: bool,
    pub new_task_show_downloading: bool,
    pub bt_trackers: Vec<String>,
    pub max_overall_download_limit: String,
    pub max_overall_upload_limit: String,
    pub user_agent: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EngineMode {
    #[default]
    Managed,
    External,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Light,
            engine_mode: EngineMode::Managed,
            external_endpoint: "ws://127.0.0.1:6800/jsonrpc".to_owned(),
            external_secret: String::new(),
            download_dir: directories::UserDirs::new()
                .and_then(|directories| {
                    directories
                        .download_dir()
                        .map(|path| path.to_string_lossy().into_owned())
                })
                .unwrap_or_default(),
            split: 16,
            max_concurrent_downloads: 5,
            notify_on_complete: true,
            notify_on_error: true,
            resume_all_when_app_launched: false,
            new_task_show_downloading: true,
            bt_trackers: vec![
                "udp://tracker.opentrackr.org:1337/announce".to_owned(),
                "udp://open.stealth.si:80/announce".to_owned(),
                "udp://tracker.torrent.eu.org:451/announce".to_owned(),
            ],
            max_overall_download_limit: "0".to_owned(),
            max_overall_upload_limit: "0".to_owned(),
            user_agent: "Motrix-Wabou/0.1".to_owned(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(config_dir: &Path) -> Self {
        Self {
            path: config_dir.join("config.json"),
        }
    }

    pub fn load(&self) -> Result<AppConfig, String> {
        match fs::read_to_string(&self.path) {
            Ok(source) => serde_json::from_str(&source)
                .map_err(|error| format!("cannot parse {}: {error}", self.path.display())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(error) => Err(format!("cannot read {}: {error}", self.path.display())),
        }
    }

    pub fn directory(&self) -> Result<&Path, String> {
        self.path
            .parent()
            .ok_or_else(|| "configuration path has no parent".to_owned())
    }

    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| "configuration path has no parent".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
        let source = serde_json::to_string_pretty(config).map_err(|error| error.to_string())?;
        fs::write(&self.path, source)
            .map_err(|error| format!("cannot write {}: {error}", self.path.display()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))
                .map_err(|error| format!("cannot secure {}: {error}", self.path.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_round_trips() {
        let root = std::env::temp_dir().join(format!("motrix-config-{}", uuid::Uuid::new_v4()));
        let store = ConfigStore::new(&root);
        let config = AppConfig {
            engine_mode: EngineMode::External,
            external_endpoint: "ws://host:6800/jsonrpc".into(),
            split: 8,
            ..AppConfig::default()
        };
        store.save(&config).expect("save config");
        assert_eq!(store.load().expect("load config"), config);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(root.join("config.json"))
                    .expect("metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        let _ = fs::remove_dir_all(root);
    }
}
