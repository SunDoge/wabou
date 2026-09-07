//! Direct Vello Hybrid window renderer and retained-command replay.

use std::sync::Arc;

use glifo::FontEmbolden;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WindowHandle,
};
use rustc_hash::FxHashMap;
use vello_common::kurbo::{Affine, Diagonal2, Shape};
use vello_common::paint::{ImageId, ImageSource, PaintType};
use vello_common::peniko::{Brush, BrushRef, Color, Fill, StyleRef};
use vello_hybrid::{
    RenderSettings, RenderSize, RenderTargetConfig, Renderer, Resources, Scene as HybridScene,
    TextureBindings,
};
use wgpu::{CommandEncoderDescriptor, CompositeAlphaMode, Features, PresentMode, TextureFormat};
use wgpu_context::{
    AlphaConversion, DeviceHandle, SurfaceRenderer, SurfaceRendererConfiguration,
    TextureConfiguration, WGPUContext,
};
use winit::window::Window;

use crate::{Glyph, NormalizedCoord, PaintScene, Scene};

#[cfg(target_os = "android")]
const DEFAULT_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;
#[cfg(not(target_os = "android"))]
const DEFAULT_TEXTURE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;
const DEFAULT_TOLERANCE: f64 = 0.1;

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

struct ActiveRenderer {
    renderer: Renderer,
    resources: Resources,
    surface: SurfaceRenderer<'static>,
}

/// Renderer owned by a Wabou window. There is deliberately no backend switch:
/// Wabou's Vello shell talks to Vello Hybrid directly.
pub(crate) struct HybridWindowRenderer {
    context: WGPUContext,
    active: Option<ActiveRenderer>,
    scene: HybridScene,
    image_cache: FxHashMap<u64, ImageId>,
    transparent: bool,
}

impl HybridWindowRenderer {
    pub(crate) fn new(transparent: bool) -> crate::Result<Self> {
        tracing::info!(target: "wabou::renderer", transparent, "initializing Vello Hybrid window renderer");
        Ok(Self {
            context: WGPUContext::with_features_and_limits(
                Some(Features::CLEAR_TEXTURE | Features::PIPELINE_CACHE),
                None,
            ),
            active: None,
            scene: HybridScene::new(1, 1),
            image_cache: FxHashMap::default(),
            transparent,
        })
    }

    pub(crate) fn resume(&mut self, window: Arc<dyn Window>, width: u32, height: u32) {
        let owned: Arc<SharedWindow> = Arc::new(SharedWindow(window));
        let surface = self
            .context
            .create_surface(owned)
            .expect("failed to create Vello Hybrid surface");
        let existing_device = self.context.find_compatible_device_handle(Some(&surface));
        let created_device = existing_device.is_none();
        let device = existing_device.unwrap_or_else(|| {
            pollster::block_on(DeviceHandle::new_from_compatible_surface(
                self.context.instance.clone(),
                Some(&surface),
                self.context.extra_features(),
                self.context.override_limits(),
            ))
            .expect("failed to create Vello Hybrid device")
        });
        let requested_alpha_mode = if self.transparent {
            #[cfg(target_vendor = "apple")]
            {
                CompositeAlphaMode::PostMultiplied
            }
            #[cfg(not(target_vendor = "apple"))]
            {
                CompositeAlphaMode::PreMultiplied
            }
        } else {
            CompositeAlphaMode::Opaque
        };
        let capabilities = surface.get_capabilities(&device.adapter);
        let alpha_mode = select_alpha_mode(requested_alpha_mode, &capabilities.alpha_modes);
        #[cfg(not(target_vendor = "apple"))]
        let intermediate_texture =
            (alpha_mode == CompositeAlphaMode::PostMultiplied).then_some(TextureConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                format: DEFAULT_TEXTURE_FORMAT,
                alpha_conversion: Some(AlphaConversion::Unpremultiply),
            });
        // Metal reports straight alpha for a surface which is premultiplied in practice.
        #[cfg(target_vendor = "apple")]
        let intermediate_texture = None;
        let surface_renderer = SurfaceRenderer::new(
            surface,
            SurfaceRendererConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                formats: vec![DEFAULT_TEXTURE_FORMAT],
                width,
                height,
                present_mode: PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode,
                view_formats: vec![],
            },
            intermediate_texture,
            device.clone(),
        )
        .expect("failed to configure Vello Hybrid surface");
        let (renderer, resources) = Renderer::new_with(
            surface_renderer.device(),
            &RenderTargetConfig {
                format: DEFAULT_TEXTURE_FORMAT,
                width,
                height,
            },
            RenderSettings::default(),
        );
        if created_device {
            self.context.device_pool.push(device);
        }
        self.scene = HybridScene::new(width as u16, height as u16);
        self.active = Some(ActiveRenderer {
            renderer,
            resources,
            surface: surface_renderer,
        });
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.scene.reset_and_resize(width as u16, height as u16);
        if let Some(active) = &mut self.active {
            active.surface.resize(width, height);
        }
    }

    pub(crate) fn render(
        &mut self,
        recorded: &mut Scene,
        width: u32,
        height: u32,
        base_color: Color,
    ) {
        let Some(active) = &mut self.active else {
            return;
        };
        let mut encoder =
            active
                .surface
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Wabou Vello Hybrid frame"),
                });
        self.scene.reset_and_resize(width as u16, height as u16);
        let mut painter = HybridPainter {
            scene: &mut self.scene,
            renderer: &mut active.renderer,
            resources: &mut active.resources,
            device: active.surface.device(),
            queue: active.surface.queue(),
            encoder: &mut encoder,
            image_cache: &mut self.image_cache,
            layers: Vec::new(),
        };
        if base_color != Color::TRANSPARENT {
            painter.scene.set_paint(base_color);
            painter.scene.fill_rect(&vello_common::kurbo::Rect::new(
                0.0,
                0.0,
                f64::from(width),
                f64::from(height),
            ));
        }
        painter.append_scene(std::mem::take(recorded), Affine::IDENTITY);

        let Ok(view) = active.surface.target_texture_view() else {
            active.surface.clear_surface_texture();
            return;
        };
        active
            .renderer
            .render(
                &self.scene,
                &mut active.resources,
                active.surface.device(),
                active.surface.queue(),
                &mut encoder,
                &RenderSize { width, height },
                &view,
                &TextureBindings::new(),
            )
            .expect("failed to render Vello Hybrid frame");
        active.surface.queue().submit([encoder.finish()]);
        drop(view);
        if active.surface.maybe_blit_and_present().is_ok() {
            let _ = active.surface.device().poll(wgpu::PollType::Poll);
        }
    }
}

fn select_alpha_mode(
    requested: CompositeAlphaMode,
    supported: &[CompositeAlphaMode],
) -> CompositeAlphaMode {
    if supported.contains(&requested) {
        return requested;
    }
    supported
        .iter()
        .copied()
        .min_by_key(|mode| match mode {
            CompositeAlphaMode::PreMultiplied => 0,
            CompositeAlphaMode::PostMultiplied => 1,
            CompositeAlphaMode::Opaque | CompositeAlphaMode::Inherit | CompositeAlphaMode::Auto => {
                2
            }
        })
        .unwrap_or(CompositeAlphaMode::Auto)
}

enum LayerKind {
    Layer,
    Clip,
}

pub(crate) struct HybridPainter<'a> {
    scene: &'a mut HybridScene,
    renderer: &'a mut Renderer,
    resources: &'a mut Resources,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    encoder: &'a mut wgpu::CommandEncoder,
    image_cache: &'a mut FxHashMap<u64, ImageId>,
    layers: Vec<LayerKind>,
}

impl HybridPainter<'_> {
    fn paint(&mut self, brush: BrushRef<'_>) -> PaintType {
        match brush {
            Brush::Solid(color) => PaintType::Solid(color),
            Brush::Gradient(gradient) => PaintType::Gradient(gradient.clone()),
            Brush::Image(image) => {
                let key = image.image.data.id();
                let id = if let Some(id) = self.image_cache.get(&key) {
                    *id
                } else {
                    let ImageSource::Pixmap(pixmap) =
                        ImageSource::from_peniko_image_data(image.image)
                    else {
                        unreachable!()
                    };
                    let id = self.renderer.upload_image(
                        self.resources,
                        self.device,
                        self.queue,
                        self.encoder,
                        &pixmap,
                    );
                    self.image_cache.insert(key, id);
                    id
                };
                PaintType::Image(vello_common::peniko::ImageBrush {
                    image: ImageSource::OpaqueId {
                        id,
                        may_have_transparency: true,
                    },
                    sampler: image.sampler,
                })
            }
        }
    }
}

impl PaintScene for HybridPainter<'_> {
    fn reset(&mut self) {
        self.scene.reset();
    }

    fn push_layer(
        &mut self,
        blend: impl Into<vello_common::peniko::BlendMode>,
        alpha: f32,
        transform: Affine,
        clip: &impl Shape,
        filter: Option<Arc<vello_common::filter_effects::Filter>>,
        _backdrop_filter: Option<Arc<vello_common::filter_effects::Filter>>,
    ) {
        self.scene.set_transform(transform);
        self.scene
            .push_clip_path(&clip.into_path(DEFAULT_TOLERANCE));
        self.scene.push_layer(
            None,
            Some(blend.into()),
            Some(alpha),
            None,
            filter.map(|value| (*value).clone()),
        );
        self.layers.push(LayerKind::Layer);
    }

    fn push_clip_layer(&mut self, transform: Affine, clip: &impl Shape) {
        self.scene.set_transform(transform);
        self.scene
            .push_clip_path(&clip.into_path(DEFAULT_TOLERANCE));
        self.layers.push(LayerKind::Clip);
    }

    fn pop_layer(&mut self) {
        match self.layers.pop() {
            Some(LayerKind::Layer) => {
                self.scene.pop_layer();
                self.scene.pop_clip_path();
            }
            Some(LayerKind::Clip) => self.scene.pop_clip_path(),
            None => {}
        }
    }

    fn stroke<'a>(
        &mut self,
        style: &vello_common::kurbo::Stroke,
        transform: Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let paint = self.paint(brush.into());
        self.scene.set_transform(transform);
        self.scene.set_stroke(style.clone());
        self.scene.set_paint(paint);
        self.scene
            .set_paint_transform(brush_transform.unwrap_or(Affine::IDENTITY));
        self.scene.stroke_path(&shape.into_path(DEFAULT_TOLERANCE));
    }

    fn fill<'a>(
        &mut self,
        fill: Fill,
        transform: Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        let paint = self.paint(brush.into());
        self.scene.set_transform(transform);
        self.scene.set_fill_rule(fill);
        self.scene.set_paint(paint);
        self.scene
            .set_paint_transform(brush_transform.unwrap_or(Affine::IDENTITY));
        self.scene.fill_path(&shape.into_path(DEFAULT_TOLERANCE));
    }

    fn draw_glyphs<'a, 's: 'a>(
        &'s mut self,
        font: &'a vello_common::peniko::FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: &'a [NormalizedCoord],
        embolden: vello_common::kurbo::Vec2,
        style: impl Into<StyleRef<'a>>,
        brush: impl Into<BrushRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        let brush = brush.into().to_owned().multiply_alpha(brush_alpha);
        let paint = self.paint((&brush).into());
        self.scene.set_paint(paint);
        self.scene.set_transform(transform);
        match style.into() {
            StyleRef::Fill(fill) => {
                self.scene.set_fill_rule(fill);
                self.scene
                    .glyph_run(self.resources, font)
                    .font_size(font_size)
                    .hint(hint)
                    .normalized_coords(normalized_coords)
                    .font_embolden(FontEmbolden::new(Diagonal2::new(embolden.x, embolden.y)))
                    .glyph_transform(glyph_transform.unwrap_or_default())
                    .fill_glyphs(glyphs.map(|glyph| glifo::Glyph {
                        id: glyph.id,
                        x: glyph.x,
                        y: glyph.y,
                    }));
            }
            StyleRef::Stroke(stroke) => {
                self.scene.set_stroke(stroke.clone());
                self.scene
                    .glyph_run(self.resources, font)
                    .font_size(font_size)
                    .hint(hint)
                    .normalized_coords(normalized_coords)
                    .glyph_transform(glyph_transform.unwrap_or_default())
                    .stroke_glyphs(glyphs.map(|glyph| glifo::Glyph {
                        id: glyph.id,
                        x: glyph.x,
                        y: glyph.y,
                    }));
            }
        }
    }

    fn draw_box_shadow(
        &mut self,
        transform: Affine,
        rect: vello_common::kurbo::Rect,
        brush: Color,
        radius: f64,
        std_dev: f64,
    ) {
        self.scene.set_transform(transform);
        self.scene.set_paint(brush);
        self.scene
            .fill_blurred_rounded_rect(&rect, radius as f32, std_dev as f32, false);
    }

    fn draw_svg(&mut self, document: crate::SvgDocument, transform: Affine) {
        if let Err(error) =
            wabou_svg_vello_hybrid::append_tree_with_transform(self.scene, &document.0, transform)
        {
            tracing::warn!(?error, "failed to project retained SVG into Vello Hybrid");
        }
    }
}

/// Reusable offscreen renderer backed by the same replay path as windows.
pub(crate) struct HybridImageRenderer {
    buffer: wgpu_context::BufferRenderer,
    renderer: Renderer,
    resources: Resources,
    scene: HybridScene,
    image_cache: FxHashMap<u64, ImageId>,
}

impl HybridImageRenderer {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        let mut context =
            WGPUContext::with_features_and_limits(Some(Features::CLEAR_TEXTURE), None);
        let buffer = pollster::block_on(context.create_buffer_renderer(
            wgpu_context::BufferRendererConfig {
                width,
                height,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            },
        ))
        .expect("no compatible Vello Hybrid device for offscreen rendering");
        let (renderer, resources) = Renderer::new(
            buffer.device(),
            &RenderTargetConfig {
                format: TextureFormat::Rgba8Unorm,
                width,
                height,
            },
        );
        Self {
            buffer,
            renderer,
            resources,
            scene: HybridScene::new(width as u16, height as u16),
            image_cache: FxHashMap::default(),
        }
    }

    pub(crate) fn render(
        &mut self,
        recorded: &Scene,
        width: u32,
        height: u32,
        base_color: Color,
    ) -> Vec<u8> {
        if self.buffer.size().width != width || self.buffer.size().height != height {
            self.buffer.resize(width, height);
            (self.renderer, self.resources) = Renderer::new(
                self.buffer.device(),
                &RenderTargetConfig {
                    format: TextureFormat::Rgba8Unorm,
                    width,
                    height,
                },
            );
            self.image_cache.clear();
        }
        self.scene.reset_and_resize(width as u16, height as u16);
        let mut encoder = self
            .buffer
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Wabou Vello Hybrid offscreen frame"),
            });
        let mut painter = HybridPainter {
            scene: &mut self.scene,
            renderer: &mut self.renderer,
            resources: &mut self.resources,
            device: self.buffer.device(),
            queue: self.buffer.queue(),
            encoder: &mut encoder,
            image_cache: &mut self.image_cache,
            layers: Vec::new(),
        };
        if base_color != Color::TRANSPARENT {
            painter.scene.set_paint(base_color);
            painter.scene.fill_rect(&vello_common::kurbo::Rect::new(
                0.0,
                0.0,
                f64::from(width),
                f64::from(height),
            ));
        }
        painter.append_scene(recorded.clone(), Affine::IDENTITY);
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                self.buffer.device(),
                self.buffer.queue(),
                &mut encoder,
                &RenderSize { width, height },
                &self.buffer.target_texture_view(),
                &TextureBindings::new(),
            )
            .expect("failed to render Vello Hybrid scene offscreen");
        self.buffer.queue().submit([encoder.finish()]);
        let mut rgba = vec![0; (width * height * 4) as usize];
        self.buffer.copy_texture_to_buffer(&mut rgba);
        rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_alpha_selection_prefers_a_premultiplied_fallback() {
        assert_eq!(
            select_alpha_mode(
                CompositeAlphaMode::Opaque,
                &[
                    CompositeAlphaMode::Auto,
                    CompositeAlphaMode::PostMultiplied,
                    CompositeAlphaMode::PreMultiplied,
                ],
            ),
            CompositeAlphaMode::PreMultiplied,
        );
    }

    #[test]
    fn surface_alpha_selection_keeps_a_supported_request() {
        assert_eq!(
            select_alpha_mode(
                CompositeAlphaMode::Opaque,
                &[
                    CompositeAlphaMode::PreMultiplied,
                    CompositeAlphaMode::Opaque
                ],
            ),
            CompositeAlphaMode::Opaque,
        );
    }
}
