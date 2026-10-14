//! Linked fallback host for frontend-only previews.
//!
//! Normal applications link their Rust capabilities directly into their own
//! executable. This host intentionally supports only a window and a UI source;
//! it does not load Rust code or expose an application capability ABI.

use wabou_backend_vello_hybrid::{VelloHybridHostBuilder, WindowOptions};
use wabou_launch::{LAUNCH_VERSION, LaunchConfig, UiSource};

/// Run a frontend-only application on the main thread.
pub fn run_config(config: LaunchConfig) -> Result<(), String> {
    let _trace_export = wabou_telemetry::ExportOnDrop;
    if config.version != LAUNCH_VERSION {
        return Err("incompatible runtime launch contract".into());
    }
    if config.width == 0 || config.height == 0 {
        return Err("window dimensions must be positive".into());
    }
    let host = VelloHybridHostBuilder::new().window(
        WindowOptions::new()
            .title(config.title)
            .decorations(config.decorations)
            .initial_inner_size(config.width, config.height),
    );
    let host = match config.source {
        UiSource::Bundle { path } => host
            .javascript_source(std::fs::read_to_string(path).map_err(|error| error.to_string())?),
        #[cfg(feature = "development")]
        UiSource::Vite { url, entry } => host.vite_source(url, entry),
        #[cfg(not(feature = "development"))]
        UiSource::Vite { .. } => return Err("this release host does not support Vite".into()),
    };
    host.run().map_err(|error| error.to_string())
}
