use anyrender::{ImageRenderer, RenderContext, ResourceId};
use rustc_hash::FxHashMap;
use vello_common::{TextureId, paint::ImageId};
use vello_hybrid::{
    RenderSize, RenderTargetConfig, Renderer, Resources, Scene, TextureBindings,
};
use wgpu::{CommandEncoderDescriptor, TextureFormat, TextureUsages, TextureView};
use wgpu_context::{BufferRenderer, BufferRendererConfig, WGPUContext};

use crate::{ImageManager, VelloHybridScenePainter};

/// Offscreen AnyRender adapter backed by Vello Hybrid.
///
/// This intentionally shares the same scene painter as the window renderer so
/// screenshots and pixel tests exercise the production command translation.
pub struct VelloHybridImageRenderer {
    buffer_renderer: BufferRenderer,
    renderer: Renderer,
    resources: Resources,
    scene: Scene,
    texture_bindings: FxHashMap<ResourceId, TextureView>,
    cached_images: FxHashMap<u64, ImageId>,
}

impl RenderContext for VelloHybridImageRenderer {}

impl ImageRenderer for VelloHybridImageRenderer {
    type ScenePainter<'a>
        = VelloHybridScenePainter<'a>
    where
        Self: 'a;

    fn new(width: u32, height: u32) -> Self {
        let mut context = WGPUContext::with_features_and_limits(
            Some(wgpu::Features::CLEAR_TEXTURE),
            None,
        );
        let buffer_renderer = pollster::block_on(context.create_buffer_renderer(
            BufferRendererConfig {
                width,
                height,
                usage: TextureUsages::RENDER_ATTACHMENT,
            },
        ))
        .expect("no compatible device found for Vello Hybrid offscreen rendering");
        let (renderer, resources) = Renderer::new(
            buffer_renderer.device(),
            &RenderTargetConfig {
                format: TextureFormat::Rgba8Unorm,
                width,
                height,
            },
        );

        Self {
            buffer_renderer,
            renderer,
            resources,
            scene: Scene::new(width as u16, height as u16),
            texture_bindings: FxHashMap::default(),
            cached_images: FxHashMap::default(),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        let size = self.buffer_renderer.size();
        if size.width == width && size.height == height {
            return;
        }
        self.buffer_renderer.resize(width, height);
        let (renderer, resources) = Renderer::new(
            self.buffer_renderer.device(),
            &RenderTargetConfig {
                format: TextureFormat::Rgba8Unorm,
                width,
                height,
            },
        );
        self.renderer = renderer;
        self.resources = resources;
        self.scene = Scene::new(width as u16, height as u16);
        self.cached_images.clear();
    }

    fn reset(&mut self) {
        self.scene.reset();
    }

    fn render_to_vec<F: FnOnce(&mut Self::ScenePainter<'_>)>(
        &mut self,
        draw_fn: F,
        cpu_buffer: &mut Vec<u8>,
    ) {
        let size = self.buffer_renderer.size();
        cpu_buffer.resize((size.width * size.height * 4) as usize, 0);
        self.render(draw_fn, cpu_buffer);
    }

    fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(
        &mut self,
        draw_fn: F,
        cpu_buffer: &mut [u8],
    ) {
        let mut encoder = self
            .buffer_renderer
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Vello Hybrid offscreen render"),
            });
        let image_manager = ImageManager::new(
            &mut self.renderer,
            &mut self.resources,
            self.buffer_renderer.device(),
            self.buffer_renderer.queue(),
            &mut encoder,
            &mut self.cached_images,
        );
        draw_fn(&mut VelloHybridScenePainter::new(
            &mut self.scene,
            image_manager,
            &mut self.texture_bindings,
            &self.buffer_renderer.device_handle,
        ));

        let mut bindings = TextureBindings::new();
        for (resource_id, texture_view) in &self.texture_bindings {
            bindings.insert(TextureId(resource_id.into_ffi()), texture_view.clone());
        }
        let size = self.buffer_renderer.size();
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                self.buffer_renderer.device(),
                self.buffer_renderer.queue(),
                &mut encoder,
                &RenderSize {
                    width: size.width,
                    height: size.height,
                },
                &self.buffer_renderer.target_texture_view(),
                &bindings,
            )
            .expect("failed to render Vello Hybrid scene offscreen");
        self.buffer_renderer.queue().submit([encoder.finish()]);
        self.buffer_renderer.copy_texture_to_buffer(cpu_buffer);
        self.scene.reset();
    }
}
