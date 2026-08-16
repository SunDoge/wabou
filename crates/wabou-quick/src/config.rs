//! App configuration for the wabou-quick host.

use vello::peniko::Color;

/// JavaScript source and initial viewport configuration for an [`crate::Applier`].
pub struct AppConfig {
    /// Bundled JavaScript evaluated when the runtime boots.
    pub js: String,
    /// Viewport clear color.
    pub base_color: Color,
    /// Initial logical viewport width for non-windowed hosts.
    pub width: u32,
    /// Initial logical viewport height for non-windowed hosts.
    pub height: u32,
    /// Vite development-server base URL.
    #[cfg(feature = "vite")]
    pub server_url: String,
    /// ESM entry imported from [`Self::server_url`].
    #[cfg(feature = "vite")]
    pub entry: String,
}

impl AppConfig {
    /// Construct configuration for a bundled JavaScript program.
    pub fn new(js: impl Into<String>) -> Self {
        Self {
            js: js.into(),
            base_color: Color::from_rgb8(0x0f, 0x17, 0x2a),
            width: 800,
            height: 600,
            #[cfg(feature = "vite")]
            server_url: String::new(),
            #[cfg(feature = "vite")]
            entry: String::new(),
        }
    }

    /// Set the initial logical viewport used by non-windowed hosts.
    pub fn with_viewport(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set the viewport clear color.
    pub fn with_base_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self
    }

    /// Vite dev-mode config: fetch ESM from the dev server at `server_url`,
    /// entry `entry` (e.g. `"src/index.tsx"`). Requires the `vite` feature.
    #[cfg(feature = "vite")]
    pub fn vite(server_url: impl Into<String>, entry: impl Into<String>) -> Self {
        Self {
            js: String::new(),
            base_color: Color::from_rgb8(0x0f, 0x17, 0x2a),
            width: 800,
            height: 600,
            server_url: server_url.into(),
            entry: entry.into(),
        }
    }
}
