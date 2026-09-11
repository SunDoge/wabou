//! Reusable window + Vello Hybrid backend + scene + text context.
//!
//! Extracted from the app so multiple hosts (the static-JSON `wabou` bin, the
//! SolidJS-driven `wabou-runtime` crate) share one windowing setup. A host
//! provides a [`crate::source::FrameSource`] that fills the scene each frame;
//! [`Shell`] handles presentation + vsync + resize.

#![warn(missing_docs)]

use snafu::ResultExt;
use std::sync::Arc;

use crate::Scene;
use vello_common::peniko::Color;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::accessibility::AccessibilityState;
use crate::renderer_backend::HybridWindowRenderer;
use crate::source::{WindowInputMode, WindowLevel, WindowOptions};
use crate::text::TextContext;
use wabou_shell_api::WindowBackground;

/// Native rendering resources owned by one live window.
pub struct Shell {
    /// Platform window handle.
    pub window: Arc<dyn Window>,
    /// Direct Vello Hybrid renderer for this window.
    renderer: HybridWindowRenderer,
    /// Current physical render size.
    surface_size: [u32; 2],
    /// Reused backend-neutral application scene for the current frame.
    pub scene: Scene,
    /// Shared text shaping and retained glyph resources.
    pub tcx: TextContext,
    /// Platform accessibility adapter for the window.
    pub accessibility: AccessibilityState,
}

impl Shell {
    /// Create the window + Vello Hybrid backend + a fresh
    /// [`TextContext`]. Returns a typed error when any window/GPU step fails.
    pub fn create(
        event_loop: &dyn ActiveEventLoop,
        options: &WindowOptions,
    ) -> crate::Result<Shell> {
        let transparent = options.background != WindowBackground::Opaque;
        if matches!(
            options.background,
            WindowBackground::Blurred | WindowBackground::Mica | WindowBackground::MicaAlt
        ) {
            tracing::warn!(
                background = ?options.background,
                "native compositor material is not implemented; using a transparent surface"
            );
        }
        let mut attrs = WindowAttributes::default()
            .with_title(options.title.clone())
            .with_visible(false)
            .with_resizable(options.resizable)
            .with_decorations(options.decorations)
            .with_transparent(transparent)
            .with_window_level(match options.window_level {
                WindowLevel::AlwaysOnBottom => winit::window::WindowLevel::AlwaysOnBottom,
                WindowLevel::Normal => winit::window::WindowLevel::Normal,
                WindowLevel::AlwaysOnTop => winit::window::WindowLevel::AlwaysOnTop,
            })
            .with_surface_size(winit::dpi::LogicalSize::new(
                options.initial_inner_size.0,
                options.initial_inner_size.1,
            ));
        if let Some((width, height)) = options.min_inner_size {
            attrs = attrs.with_min_surface_size(winit::dpi::LogicalSize::new(width, height));
        }

        let window: Arc<dyn Window> = Arc::from(
            event_loop
                .create_window(attrs)
                .context(crate::error::CreateWindowSnafu)?,
        );
        let physical_size = window.surface_size();
        let accessibility = AccessibilityState::new(window.clone(), options.title.clone());
        if options.input_mode == WindowInputMode::Passthrough
            && let Err(error) = window.set_cursor_hittest(false)
        {
            tracing::warn!(?error, "pointer passthrough is unavailable for this window");
        }
        let surface_width = physical_size.width.max(1);
        let surface_height = physical_size.height.max(1);
        let mut renderer = HybridWindowRenderer::new(transparent)?;
        renderer.resume(window.clone(), surface_width, surface_height);
        // Expose the window after its rendering resources are ready.
        window.set_visible(true);

        Ok(Shell {
            window,
            renderer,
            surface_size: [surface_width, surface_height],
            scene: Scene::new(),
            tcx: TextContext::new(),
            accessibility,
        })
    }

    /// Current surface size (w,h).
    pub fn size(&self) -> (u32, u32) {
        (self.surface_size[0], self.surface_size[1])
    }

    /// Resize the physical render surface; zero-sized requests are ignored.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_size = [width, height];
            self.renderer.resize(width, height);
            self.window.request_redraw();
        }
    }

    /// Current physical-pixels-per-logical-pixel scale reported by the window.
    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor().max(f64::EPSILON)
    }

    /// Query maximize state without changing native window styles.
    pub fn is_maximized(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            use objc2_app_kit::{NSView, NSWindowStyleMask};
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            // winit 0.31 beta's is_maximized temporarily adds a title bar to
            // borderless/non-resizable windows. That emits resize events; when
            // queried while publishing metrics it creates an endless event loop.
            let Ok(handle) = self.window.window_handle() else {
                return false;
            };
            let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
                return false;
            };
            // SAFETY: Shell owns the live window, and metrics are queried on
            // the AppKit event-loop thread. The borrowed NSView cannot outlive it.
            let view = unsafe { handle.ns_view.cast::<NSView>().as_ref() };
            let Some(window) = view.window() else {
                return false;
            };
            let required = NSWindowStyleMask::Titled | NSWindowStyleMask::Resizable;
            if window.styleMask().contains(required) {
                return window.isZoomed();
            }
            // AppKit's isZoomed is unreliable for borderless windows. Compare
            // their frame with the screen's work area, in logical coordinates.
            let Some(screen) = window.screen() else {
                return false;
            };
            let frame = window.frame();
            let work = screen.visibleFrame();
            return (frame.origin.x - work.origin.x).abs() <= 1.0
                && (frame.origin.y - work.origin.y).abs() <= 1.0
                && (frame.size.width - work.size.width).abs() <= 1.0
                && (frame.size.height - work.size.height).abs() <= 1.0;
        }
        #[cfg(not(target_os = "macos"))]
        self.window.is_maximized()
    }

    /// CSS/layout viewport in logical pixels; the surface remains physical.
    /// Current logical client-area size, clamped to at least 1×1.
    pub fn logical_size(&self) -> (u32, u32) {
        let (width, height) = self.size();
        let scale = self.scale_factor();
        (
            (width as f64 / scale).ceil().max(1.0) as u32,
            (height as f64 / scale).ceil().max(1.0) as u32,
        )
    }

    /// Render and present [`Self::scene`], returning whether presentation succeeded.
    pub fn present(&mut self, base_color: Color) -> bool {
        self.accessibility.publish_root(self.window.as_ref());
        self.renderer.render(
            &mut self.scene,
            self.surface_size[0],
            self.surface_size[1],
            base_color,
        );
        true
    }

    /// Borrow the platform window.
    pub fn window(&self) -> &Arc<dyn Window> {
        &self.window
    }
    /// Mutably borrow shared text shaping resources.
    pub fn tcx_mut(&mut self) -> &mut TextContext {
        &mut self.tcx
    }
    /// Mutably borrow the frame's retained Wabou paint scene.
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }
}
