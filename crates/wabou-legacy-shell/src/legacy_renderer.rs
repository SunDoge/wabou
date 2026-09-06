//! Renderer selection for the Winit/Taffy application backend.

/// Renderer used by the Winit/Taffy application backend.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RendererBackend {
    /// Vello Hybrid through AnyRender.
    #[default]
    VelloHybrid,
    /// Vello Classic through AnyRender.
    Vello,
    /// Skia through AnyRender.
    Skia,
}
