use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub download_dir: String,
    pub split: i32,
    pub max_connection_per_server: u32,
    pub min_split_size: String,
    pub max_concurrent_downloads: u32,
    pub notify_on_complete: bool,
    pub notify_on_error: bool,
    pub resume_all_when_app_launched: bool,
    pub new_task_show_downloading: bool,
    pub warn_before_quit: bool,
    pub dht_enabled: bool,
    pub pex_enabled: bool,
    pub bt_max_peers: u32,
    pub listen_port: u16,
    pub nat_enabled: bool,
    pub nat_protocol: NatProtocol,
    pub seed_ratio: f64,
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
            max_concurrent_downloads: 5,
            notify_on_complete: true,
            notify_on_error: true,
            resume_all_when_app_launched: false,
            new_task_show_downloading: true,
            warn_before_quit: true,
            dht_enabled: true,
            pex_enabled: true,
            bt_max_peers: 128,
            listen_port: 6881,
            nat_enabled: true,
            nat_protocol: NatProtocol::Auto,
            seed_ratio: 1.0,
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

impl AppConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=64).contains(&self.split) {
            return Err("split count must be between 1 and 64".to_owned());
        }
        if !(1..=64).contains(&self.max_connection_per_server) {
            return Err("connections per server must be between 1 and 64".to_owned());
        }
        if !(1..=128).contains(&self.max_concurrent_downloads) {
            return Err("maximum concurrent downloads must be between 1 and 128".to_owned());
        }
        if parse_byte_size(&self.min_split_size).is_none_or(|value| value < 64 * 1024) {
            return Err("minimum split size must be at least 64K".to_owned());
        }
        for (label, value) in [
            ("download limit", &self.max_overall_download_limit),
            ("upload limit", &self.max_overall_upload_limit),
        ] {
            if parse_byte_size(value).is_none() {
                return Err(format!(
                    "{label} must be 0 or an integer size using K, M, or G"
                ));
            }
        }
        if !(1..=10_000).contains(&self.bt_max_peers) {
            return Err("maximum peers must be between 1 and 10000".to_owned());
        }
        if self.listen_port == 0 {
            return Err("BT listen port must be between 1 and 65535".to_owned());
        }
        if !self.seed_ratio.is_finite() || self.seed_ratio < 0.0 {
            return Err("seed ratio must be zero or a positive number".to_owned());
        }
        if self.user_agent.trim().is_empty() {
            return Err("HTTP User-Agent is required".to_owned());
        }
        if self.proxy.enabled {
            if self.proxy.host.trim().is_empty() {
                return Err("proxy host is required when the proxy is enabled".to_owned());
            }
            if self.proxy.port == 0 {
                return Err("proxy port must be between 1 and 65535".to_owned());
            }
        }
        if self.speed_profiles.is_empty() || self.speed_profiles.len() > 8 {
            return Err("between 1 and 8 speed profiles are required".to_owned());
        }
        let mut names = std::collections::HashSet::new();
        for profile in &self.speed_profiles {
            let name = profile.name.trim().to_lowercase();
            if name.is_empty() {
                return Err("every speed profile requires a name".to_owned());
            }
            if !names.insert(name) {
                return Err("speed profile names must be unique".to_owned());
            }
            if parse_byte_size(&profile.download_limit).is_none()
                || parse_byte_size(&profile.upload_limit).is_none()
            {
                return Err(
                    "speed profile limits must be 0 or integer sizes using K, M, or G".to_owned(),
                );
            }
        }
        Ok(())
    }
}

pub(crate) fn parse_byte_size(value: &str) -> Option<u64> {
    let value = value.trim();
    let split = value
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(value.len());
    let number = value[..split].parse::<u64>().ok()?;
    let multiplier = match value[split..].trim().to_ascii_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" | "KIB" => 1024,
        "M" | "MB" | "MIB" => 1024 * 1024,
        "G" | "GB" | "GIB" => 1024 * 1024 * 1024,
        _ => return None,
    };
    number.checked_mul(multiplier)
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
            Ok(source) => {
                let config: AppConfig = serde_json::from_str(&source)
                    .map_err(|error| format!("cannot parse {}: {error}", self.path.display()))?;
                config
                    .validate()
                    .map_err(|error| format!("invalid {}: {error}", self.path.display()))?;
                Ok(config)
            }
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
        config.validate()?;
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
        assert!(config.nat_enabled);
        assert_eq!(config.nat_protocol, NatProtocol::Auto);
        assert_eq!(config.speed_profiles, AppConfig::default().speed_profiles);
    }

    #[test]
    fn config_rejects_values_the_engine_would_ignore_or_clamp() {
        let cases = [
            AppConfig {
                min_split_size: "1.5M".to_owned(),
                ..AppConfig::default()
            },
            AppConfig {
                max_overall_download_limit: "2T".to_owned(),
                ..AppConfig::default()
            },
            AppConfig {
                proxy: ProxyConfig {
                    enabled: true,
                    host: String::new(),
                    port: 8080,
                },
                ..AppConfig::default()
            },
        ];
        for config in cases {
            assert!(config.validate().is_err(), "accepted {config:?}");
        }
    }
}
