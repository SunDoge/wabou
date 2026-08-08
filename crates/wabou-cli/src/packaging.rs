use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use cargo_packager::Config;
use serde_json::Value;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Translate Wabou's private package model at one narrow adapter boundary.
/// The JSON is retained for diagnostics, but packaging uses the typed library
/// API and never shells out to a separately installed cargo subcommand.
pub(crate) fn package(value: &Value, diagnostic_path: &Path) -> Result<Vec<PathBuf>> {
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(diagnostic_path, &bytes)?;
    let config = decode_config(&bytes)?;
    let outputs = cargo_packager::package(&config)
        .map_err(|error| format!("native package backend failed: {error}"))?;
    Ok(outputs
        .into_iter()
        .flat_map(|output| output.paths)
        .collect())
}

fn decode_config(bytes: &[u8]) -> Result<Config> {
    serde_json::from_slice(bytes)
        .map_err(|error| format!("generated cargo-packager configuration is invalid: {error}").into())
}

#[cfg(test)]
mod tests {
    use super::decode_config;

    #[test]
    fn generated_adapter_input_is_a_typed_packager_config() {
        let config = decode_config(
            br#"{
                "name":"example",
                "productName":"Example",
                "version":"1.2.3",
                "identifier":"dev.wabou.example",
                "binaries":[{"path":"example","main":true}],
                "binariesDir":"/tmp/stage",
                "formats":["deb"],
                "outDir":"/tmp/bundles"
            }"#,
        )
        .unwrap();
        assert_eq!(config.product_name, "Example");
        assert_eq!(config.version, "1.2.3");
        assert_eq!(config.binaries.len(), 1);
    }
}
