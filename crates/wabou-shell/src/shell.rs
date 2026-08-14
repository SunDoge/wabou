//! Reusable window + GPU surface + vello renderer + scene + text context.
//!
//! Extracted from the app so multiple hosts (the static-JSON `wabou` bin, the
//! SolidJS-driven `wabou-quick` crate) share one windowing setup. A host
//! provides a [`crate::source::FrameSource`] that fills the scene each frame;
//! [`Shell`] handles presentation + vsync + resize.

use snafu::ResultExt;
use std::num::NonZeroUsize;
use std::sync::Arc;

use vello::peniko::Color;
use vello::wgpu;
use vello::{AaConfig, AaSupport, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};
use wgpu_context::{
    SurfaceRenderer, SurfaceRendererConfiguration, TextureConfiguration, WGPUContext,
};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::accessibility::AccessibilityState;
use crate::source::WindowOptions;
use crate::text::TextContext;

pub struct Shell {
    pub window: Arc<dyn Window>,
    pub surface: SurfaceRenderer<'static>,
    pub renderer: VelloRenderer,
    pub scene: Scene,
    pub tcx: TextContext,
    pub accessibility: AccessibilityState,
}

impl Shell {
    /// Create the window + wgpu surface + vello renderer + a fresh
    /// `TextContext`. `width`/`height` are the initial surface size. Prints to
    /// stderr and returns `None` if any GPU init step fails (the host can then
    /// decide whether to retry or exit).
    pub fn create(
        event_loop: &dyn ActiveEventLoop,
        options: &WindowOptions,
    ) -> crate::Result<Shell> {
        let mut attrs = WindowAttributes::default()
            .with_title(options.title.clone())
            .with_visible(false)
            .with_resizable(options.resizable)
            .with_decorations(options.decorations)
            .with_transparent(options.transparent)
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
        window.set_visible(true);
        let surface_width = physical_size.width.max(1);
        let surface_height = physical_size.height.max(1);

        let mut context = WGPUContext::new();
        let (surface, device_handle) = pollster::block_on(async {
            let surface = context.create_surface(window.clone())?;
            let device_id = context.find_or_create_device(Some(&surface)).await?;
            let device_handle = context.device_pool[device_id].clone();
            Ok::<_, wgpu_context::WgpuContextError>((surface, device_handle))
        })
        .context(crate::error::CreateSurfaceRendererSnafu)?;
        let alpha_mode = select_alpha_mode(
            options.transparent,
            &surface.get_capabilities(&device_handle.adapter).alpha_modes,
        );
        let surface = SurfaceRenderer::new(
            surface,
            SurfaceRendererConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                formats: vec![
                    wgpu::TextureFormat::Rgba8Unorm,
                    wgpu::TextureFormat::Bgra8Unorm,
                ],
                width: surface_width,
                height: surface_height,
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode,
                view_formats: vec![],
            },
            Some(TextureConfiguration {
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            }),
            device_handle,
        )
        .context(crate::error::CreateSurfaceRendererSnafu)?;

        let renderer = VelloRenderer::new(
            surface.device(),
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::all(),
                num_init_threads: NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )
        .context(crate::error::CreateVelloRendererSnafu)?;

        Ok(Shell {
            window,
            surface,
            renderer,
            scene: Scene::new(),
            tcx: TextContext::new(),
            accessibility,
        })
    }

    /// Current surface size (w,h).
    pub fn size(&self) -> (u32, u32) {
        (self.surface.config.width, self.surface.config.height)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            // A texture acquired for the old swapchain must not survive a
            // configure. SurfaceRenderer::resize does not clear it itself.
            self.surface.clear_surface_texture();
            self.surface.resize(width, height);
            self.window.request_redraw();
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.window.scale_factor().max(f64::EPSILON)
    }

    /// CSS/layout viewport in logical pixels; the surface remains physical.
    pub fn logical_size(&self) -> (u32, u32) {
        let (width, height) = self.size();
        let scale = self.scale_factor();
        (
            (width as f64 / scale).ceil().max(1.0) as u32,
            (height as f64 / scale).ceil().max(1.0) as u32,
        )
    }

    /// Encode `self.scene` and blit+present to the surface (vsync-blocks on
    /// `PresentMode::AutoVsync`).
    pub fn present(&mut self, base_color: Color) -> bool {
        self.accessibility.publish_root(self.window.as_ref());
        let (w, h) = self.size();
        let Ok(view) = self.surface.target_texture_view() else {
            self.surface.clear_surface_texture();
            return false;
        };
        if let Err(e) = self.renderer.render_to_texture(
            self.surface.device(),
            self.surface.queue(),
            &self.scene,
            &view,
            &RenderParams {
                base_color,
                width: w,
                height: h,
                antialiasing_method: AaConfig::Area,
            },
        ) {
            eprintln!("vello render_to_texture failed: {e}");
            self.surface.clear_surface_texture();
            return false;
        }
        // Match anyrender_vello/Blitz: release the target view before the
        // intermediate texture is blitted and the swapchain image presented.
        drop(view);
        // Match anyrender_vello: on a present error just skip this frame —
        // `acquire_reconfiguring_if_stale` (inside maybe_blit_and_present)
        // already reconfigures Outdated/Lost surfaces.
        if self.surface.maybe_blit_and_present().is_err() {
            return false;
        }
        // Presentation remains asynchronous; the surface's frame-latency
        // setting provides backpressure without serializing CPU and GPU.
        let _ = self.surface.device().poll(wgpu::PollType::Poll);
        true
    }

    pub fn window(&self) -> &Arc<dyn Window> {
        &self.window
    }
    pub fn tcx_mut(&mut self) -> &mut TextContext {
        &mut self.tcx
    }
    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }
}

fn select_alpha_mode(
    transparent: bool,
    supported: &[wgpu::CompositeAlphaMode],
) -> wgpu::CompositeAlphaMode {
    if !transparent {
        return wgpu::CompositeAlphaMode::Auto;
    }

    [
        wgpu::CompositeAlphaMode::PreMultiplied,
        wgpu::CompositeAlphaMode::PostMultiplied,
        wgpu::CompositeAlphaMode::Inherit,
    ]
    .into_iter()
    .find(|mode| supported.contains(mode))
    .unwrap_or(wgpu::CompositeAlphaMode::Auto)
}

#[cfg(test)]
mod tests {
    use super::select_alpha_mode;
    use vello::wgpu::CompositeAlphaMode;

    #[test]
    fn opaque_windows_leave_alpha_selection_to_wgpu() {
        assert_eq!(
            select_alpha_mode(false, &[CompositeAlphaMode::PreMultiplied]),
            CompositeAlphaMode::Auto
        );
    }

    #[test]
    fn transparent_windows_prefer_premultiplied_alpha() {
        assert_eq!(
            select_alpha_mode(
                true,
                &[
                    CompositeAlphaMode::PostMultiplied,
                    CompositeAlphaMode::PreMultiplied,
                ]
            ),
            CompositeAlphaMode::PreMultiplied
        );
    }

    #[test]
    fn transparent_windows_fall_back_to_supported_compositing() {
        assert_eq!(
            select_alpha_mode(true, &[CompositeAlphaMode::Inherit]),
            CompositeAlphaMode::Inherit
        );
        assert_eq!(
            select_alpha_mode(true, &[CompositeAlphaMode::Opaque]),
            CompositeAlphaMode::Auto
        );
    }
}
