//! Backend-neutral native-window requests.

use serde::{Deserialize, Serialize};

/// Initial stacking level requested for a native window.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowLevel {
    /// Keep the window below ordinary application windows where supported.
    AlwaysOnBottom,
    /// Use the platform's ordinary application-window level.
    #[default]
    Normal,
    /// Keep the window above ordinary application windows where supported.
    AlwaysOnTop,
}

/// Whether a native window participates in pointer hit testing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowInputMode {
    /// Deliver pointer input to the window normally.
    #[default]
    Interactive,
    /// Let pointer input pass through to windows underneath where supported.
    Passthrough,
}

/// Renderer selected by the legacy Winit backend.
///
/// GPUI ignores this transitional field because GPUI owns its renderer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RendererBackend {
    /// Vello's compute renderer through AnyRender.
    #[default]
    Vello,
    /// Skia through AnyRender.
    Skia,
}

/// Requested properties used when creating a native window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowOptions {
    /// Initial native window title.
    pub title: String,
    /// Requested logical client-area size.
    pub initial_inner_size: (u32, u32),
    /// Optional minimum logical client-area size.
    pub min_inner_size: Option<(u32, u32)>,
    /// Whether the user may resize the window.
    pub resizable: bool,
    /// Whether the OS draws its standard title bar and border.
    pub decorations: bool,
    /// Whether the native surface preserves alpha.
    pub transparent: bool,
    /// Requested initial stacking level.
    pub window_level: WindowLevel,
    /// Requested initial pointer hit-test behavior.
    pub input_mode: WindowInputMode,
    /// Renderer used only by the legacy backend.
    #[serde(default)]
    pub renderer: RendererBackend,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            title: "wabou".into(),
            initial_inner_size: (800, 600),
            min_inner_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            window_level: WindowLevel::Normal,
            input_mode: WindowInputMode::Interactive,
            renderer: RendererBackend::Vello,
        }
    }
}

impl WindowOptions {
    /// Construct options with platform-friendly defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the initial native window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the requested logical client-area size, clamped to at least 1×1.
    pub fn initial_inner_size(mut self, width: u32, height: u32) -> Self {
        self.initial_inner_size = (width.max(1), height.max(1));
        self
    }

    /// Set the minimum logical client-area size, clamped to at least 1×1.
    pub fn min_inner_size(mut self, width: u32, height: u32) -> Self {
        self.min_inner_size = Some((width.max(1), height.max(1)));
        self
    }

    /// Control whether the user may resize the window.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Enable or disable native window-manager decorations.
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    /// Request a native window whose background preserves rendered alpha.
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Set the initial native stacking level.
    pub fn window_level(mut self, window_level: WindowLevel) -> Self {
        self.window_level = window_level;
        self
    }

    /// Set the initial pointer hit-test behavior.
    pub fn input_mode(mut self, input_mode: WindowInputMode) -> Self {
        self.input_mode = input_mode;
        self
    }

    /// Select the renderer used by the legacy backend.
    pub fn renderer(mut self, renderer: RendererBackend) -> Self {
        self.renderer = renderer;
        self
    }
}

/// Command addressed to a Wabou-managed native window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WindowCommand {
    /// Request that the window close.
    Close,
    /// Minimize the window.
    Minimize,
    /// Set or clear the maximized state.
    SetMaximized(bool),
    /// Replace the native window title.
    SetTitle(String),
    /// Begin an OS-managed window drag from the current pointer gesture.
    StartDragging,
    /// Restore a hidden or surface-released logical window.
    Show,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_clamp_invalid_initial_dimensions_at_the_contract_boundary() {
        let options = WindowOptions::new()
            .initial_inner_size(0, 0)
            .min_inner_size(0, 0);

        assert_eq!(options.initial_inner_size, (1, 1));
        assert_eq!(options.min_inner_size, Some((1, 1)));
    }

    #[test]
    fn gpui_and_legacy_backends_share_one_serialized_window_contract() {
        let options = WindowOptions::new()
            .title("Contract")
            .transparent(true)
            .input_mode(WindowInputMode::Passthrough)
            .window_level(WindowLevel::AlwaysOnTop);
        let encoded = serde_json::to_string(&options).expect("serialize window options");
        let decoded: WindowOptions =
            serde_json::from_str(&encoded).expect("deserialize window options");

        assert_eq!(decoded, options);
    }
}
