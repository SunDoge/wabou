use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub engine_mode: EngineMode,
    pub external_endpoint: String,
    pub external_secret: String,
    pub download_dir: String,
    pub split: i32,
    pub max_connection_per_server: u32,
    pub min_split_size: String,
    pub file_allocation: FileAllocation,
    pub max_concurrent_downloads: u32,
    pub notify_on_complete: bool,
    pub notify_on_error: bool,
    pub resume_all_when_app_launched: bool,
    pub new_task_show_downloading: bool,
    pub warn_before_quit: bool,
    pub bt_trackers: Vec<String>,
    pub dht_enabled: bool,
    pub pex_enabled: bool,
    pub bt_max_peers: u32,
    pub listen_port: u16,
    pub dht_listen_port: u16,
    pub nat_enabled: bool,
    pub nat_protocol: NatProtocol,
    pub seed_ratio: f64,
    pub seed_time: u32,
    pub max_overall_download_limit: String,
    pub max_overall_upload_limit: String,
    pub speed_profiles: Vec<SpeedProfile>,
    pub user_agent: String,
    pub proxy: ProxyConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpeedProfile {
    pub name: String,
    pub download_limit: String,
    pub upload_limit: String,
}

impl SpeedProfile {
    fn new(name: &str, download_limit: &str, upload_limit: &str) -> Self {
        Self {
            name: name.to_owned(),
            download_limit: download_limit.to_owned(),
            upload_limit: upload_limit.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProxyConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub bypass: Vec<String>,
}

impl ProxyConfig {
    pub fn url(&self) -> Option<String> {
        (self.enabled && !self.host.trim().is_empty())
            .then(|| format!("{}://{}:{}", "http", self.host.trim(), self.port))
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: 8080,
            bypass: vec!["localhost".to_owned(), "127.0.0.1".to_owned()],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileAllocation {
    #[default]
    None,
    Prealloc,
    Trunc,
    Falloc,
}

impl FileAllocation {
    pub fn as_aria2_value(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Prealloc => "prealloc",
            Self::Trunc => "trunc",
            Self::Falloc => "falloc",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemeMode {
    #[default]
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EngineMode {
    #[default]
    Managed,
    External,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NatProtocol {
    #[default]
    Auto,
    Pcp,
    NatPmp,
    Upnp,
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
            max_connection_per_server: 16,
            min_split_size: "20M".to_owned(),
            file_allocation: FileAllocation::None,
            max_concurrent_downloads: 5,
            notify_on_complete: true,
            notify_on_error: true,
            resume_all_when_app_launched: false,
            new_task_show_downloading: true,
            warn_before_quit: true,
            bt_trackers: vec![
                "udp://tracker.opentrackr.org:1337/announce".to_owned(),
                "udp://open.stealth.si:80/announce".to_owned(),
                "udp://tracker.torrent.eu.org:451/announce".to_owned(),
            ],
            dht_enabled: true,
            pex_enabled: true,
            bt_max_peers: 128,
            listen_port: 6881,
            dht_listen_port: 6881,
            nat_enabled: true,
            nat_protocol: NatProtocol::Auto,
            seed_ratio: 1.0,
            seed_time: 60,
            max_overall_download_limit: "0".to_owned(),
            max_overall_upload_limit: "0".to_owned(),
            speed_profiles: vec![
                SpeedProfile::new("Unlimited", "0", "0"),
                SpeedProfile::new("Balanced", "10M", "1M"),
                SpeedProfile::new("Saver", "1M", "256K"),
            ],
            user_agent: "Motrix-Wabou/0.1".to_owned(),
            proxy: ProxyConfig::default(),
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
            theme: ThemeMode::System,
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

    #[test]
    fn older_configs_enable_quit_warning_by_default() {
        let config: AppConfig = serde_json::from_str(r#"{"theme":"light"}"#).unwrap();
        assert!(config.warn_before_quit);
        assert_eq!(config.listen_port, 6881);
        assert_eq!(config.dht_listen_port, 6881);
        assert!(config.nat_enabled);
        assert_eq!(config.nat_protocol, NatProtocol::Auto);
        assert_eq!(config.speed_profiles, AppConfig::default().speed_profiles);
    }
}
