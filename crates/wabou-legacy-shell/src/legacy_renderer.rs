//! Renderer selection used only by legacy AnyRender experiments.

/// Renderer used by the isolated legacy AnyRender implementation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RendererBackend {
    /// Vello Classic through AnyRender.
    #[default]
    Vello,
    /// Skia through AnyRender.
    Skia,
}
