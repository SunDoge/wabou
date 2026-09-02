//! AnyRender window backend selection.

use std::sync::Arc;

use anyrender::{PaintScene, WindowRenderer as _};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};
use winit::window::Window;

use crate::RendererBackend;

struct SharedWindow(Arc<dyn Window>);

impl HasWindowHandle for SharedWindow {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        self.0.window_handle()
    }
}

impl HasDisplayHandle for SharedWindow {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        self.0.display_handle()
    }
}

pub(crate) enum AnyWindowRenderer {
    Vello(Box<anyrender_vello::VelloWindowRenderer>),
    #[cfg(feature = "renderer-skia")]
    Skia(Box<anyrender_skia::SkiaWindowRenderer>),
}

impl AnyWindowRenderer {
    pub(crate) fn new(backend: RendererBackend, transparent: bool) -> crate::Result<Self> {
        tracing::info!(
            target: "wabou::renderer",
            ?backend,
            transparent,
            "initializing AnyRender window renderer"
        );
        let alpha = if transparent {
            anyrender::CompositeAlphaMode::Transparent
        } else {
            anyrender::CompositeAlphaMode::Opaque
        };
        let config = anyrender::RendererConfig::new()
            .base_color(Color::TRANSPARENT)
            .composite_alpha_mode(alpha);
        match backend {
            RendererBackend::Vello => Ok(Self::Vello(Box::new(
                anyrender_vello::VelloWindowRenderer::with_options(config.clone()),
            ))),
            RendererBackend::Skia => {
                #[cfg(feature = "renderer-skia")]
                {
                    Ok(Self::Skia(Box::new(
                        anyrender_skia::SkiaWindowRenderer::with_options(config),
                    )))
                }
                #[cfg(not(feature = "renderer-skia"))]
                {
                    Err(crate::Error::RendererBackendUnavailable {
                        backend: "skia",
                        feature: "`wabou-shell/renderer-skia`",
                    })
                }
            }
        }
    }

    pub(crate) fn resume(&mut self, window: Arc<dyn Window>, width: u32, height: u32) {
        let window: Arc<dyn anyrender::WindowHandle> = Arc::new(SharedWindow(window));
        match self {
            Self::Vello(renderer) => renderer.resume(window, width, height, || {}),
            #[cfg(feature = "renderer-skia")]
            Self::Skia(renderer) => renderer.resume(window, width, height, || {}),
        }
        let ready = match self {
            Self::Vello(renderer) => renderer.complete_resume(),
            #[cfg(feature = "renderer-skia")]
            Self::Skia(renderer) => renderer.complete_resume(),
        };
        debug_assert!(ready, "native AnyRender backends resume synchronously");
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        match self {
            Self::Vello(renderer) => renderer.set_size(width, height),
            #[cfg(feature = "renderer-skia")]
            Self::Skia(renderer) => renderer.set_size(width, height),
        }
    }

    pub(crate) fn render(
        &mut self,
        scene: &mut anyrender::Scene,
        width: u32,
        height: u32,
        base_color: Color,
    ) {
        // A frame scene is rebuilt before every presentation. Hand its command
        // buffer to the backend instead of cloning every recorded path, image,
        // and text fragment at the window boundary.
        let scene = std::mem::take(scene);
        match self {
            Self::Vello(renderer) => renderer.render(|painter| {
                paint_frame(painter, scene, width, height, base_color);
            }),
            #[cfg(feature = "renderer-skia")]
            Self::Skia(renderer) => renderer.render(|painter| {
                paint_frame(painter, scene, width, height, base_color);
            }),
        }
    }
}

fn paint_frame(
    painter: &mut impl PaintScene,
    scene: anyrender::Scene,
    width: u32,
    height: u32,
    base_color: Color,
) {
    // Window renderers retain their backend scene between calls. The frame
    // passed here is complete, not incremental, so reset the final painter as
    // well as the backend-neutral recording scene. Without this, Vello's draw
    // data grows every frame until its fixed bin-data allocation overflows.
    painter.reset();
    painter.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        base_color,
        None,
        &Rect::new(0.0, 0.0, f64::from(width), f64::from(height)),
    );
    painter.append_scene(scene, Affine::IDENTITY);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_paint_starts_with_the_requested_surface_color() {
        let mut target = anyrender::Scene::new();
        paint_frame(
            &mut target,
            anyrender::Scene::new(),
            23,
            17,
            Color::from_rgb8(12, 34, 56),
        );

        assert_eq!(target.commands.len(), 1);
        assert!(matches!(
            &target.commands[0],
            anyrender::recording::RenderCommand::Fill(command)
                if command.brush == Color::from_rgb8(12, 34, 56).into()
        ));
    }

    #[test]
    fn frame_paint_replaces_the_previous_backend_scene() {
        let mut target = anyrender::Scene::new();
        let mut first = anyrender::Scene::new();
        first.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::WHITE,
            None,
            &Rect::new(0.0, 0.0, 10.0, 10.0),
        );
        paint_frame(&mut target, first, 20, 20, Color::BLACK);
        assert_eq!(target.commands.len(), 2);

        paint_frame(&mut target, anyrender::Scene::new(), 20, 20, Color::BLACK);
        assert_eq!(
            target.commands.len(),
            1,
            "the second frame must not retain commands from the first"
        );
    }
}
