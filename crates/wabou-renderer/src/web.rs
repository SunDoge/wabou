//! Browser Canvas backend for the same retained paint scenes used by native windows.

use crate::{PaintCommand, Scene};
use glifo::FontEmbolden;
use std::collections::{HashMap, HashSet};
use vello_common::kurbo::{Affine, Diagonal2};
use vello_common::paint::{ImageId, ImageSource, PaintType};
use vello_common::peniko::{Brush, Color, Style};
use vello_gpu::{RenderSize, Resources, WebGlRenderer, WebGlTextureBindings};
use web_sys::HtmlCanvasElement;

/// A persistent WebGL2 renderer. The browser owns the canvas and JavaScript engine.
pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    renderer: WebGlRenderer,
    resources: Resources,
    scene: vello_gpu::Scene,
    images: HashMap<u64, ImageId>,
}

impl WebRenderer {
    /// Initialize Vello against a browser canvas. WebGL2 is required.
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let (renderer, resources) = WebGlRenderer::new(&canvas).map_err(|e| e.to_string())?;
        Ok(Self {
            canvas,
            renderer,
            resources,
            scene: vello_gpu::Scene::new(1, 1),
            images: HashMap::new(),
        })
    }

    fn paint(&mut self, brush: &Brush, used: &mut HashSet<u64>) -> Result<PaintType, String> {
        Ok(match brush {
            Brush::Solid(color) => PaintType::Solid(*color),
            Brush::Gradient(gradient) => PaintType::Gradient(gradient.clone()),
            Brush::Image(image) => {
                let key = image.image.data.id();
                used.insert(key);
                let id = if let Some(id) = self.images.get(&key) {
                    *id
                } else {
                    let ImageSource::Pixmap(pixmap) =
                        ImageSource::from_peniko_image_data(&image.image)
                    else {
                        return Err("unsupported image storage".into());
                    };
                    let id = self
                        .renderer
                        .upload_image(&mut self.resources, &pixmap)
                        .map_err(|e| e.to_string())?;
                    self.images.insert(key, id);
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
        })
    }

    /// Replay a shared Wabou scene into the canvas at its physical dimensions.
    /// Unsupported WGSL effects are rejected before altering the current canvas.
    pub fn render(&mut self, scene: &Scene, width: u32, height: u32) -> Result<(), String> {
        let width = u16::try_from(width)
            .ok()
            .filter(|v| *v != 0)
            .ok_or("invalid canvas width")?;
        let height = u16::try_from(height)
            .ok()
            .filter(|v| *v != 0)
            .ok_or("invalid canvas height")?;
        if scene
            .commands
            .iter()
            .any(|c| matches!(c, PaintCommand::Shader { .. }))
        {
            return Err("custom WGSL effects require a WebGPU backend".into());
        }
        self.scene.reset_and_resize(width, height);
        let mut used = HashSet::new();
        let mut layers = Vec::new();
        for command in &scene.commands {
            match command {
                PaintCommand::PushLayer {
                    blend,
                    alpha,
                    transform,
                    clip,
                    filter,
                } => {
                    self.scene.set_transform(*transform);
                    self.scene.push_clip_path(clip);
                    self.scene.push_layer(
                        None,
                        Some(*blend),
                        Some(*alpha),
                        None,
                        filter.as_ref().map(|f| (**f).clone()),
                    );
                    layers.push(true);
                }
                PaintCommand::PushClip { transform, clip } => {
                    self.scene.set_transform(*transform);
                    self.scene.push_clip_path(clip);
                    layers.push(false);
                }
                PaintCommand::PopLayer => {
                    let layer = layers.pop().ok_or("unbalanced paint layer")?;
                    if layer {
                        self.scene.pop_layer();
                    }
                    self.scene.pop_clip();
                }
                PaintCommand::Fill {
                    fill,
                    transform,
                    brush,
                    brush_transform,
                    shape,
                } => {
                    let paint = self.paint(brush, &mut used)?;
                    self.scene.set_transform(*transform);
                    self.scene.set_paint(paint);
                    self.scene
                        .set_paint_transform(brush_transform.unwrap_or(Affine::IDENTITY));
                    self.scene.set_fill_rule(*fill);
                    self.scene.fill_path(shape);
                }
                PaintCommand::Stroke {
                    style,
                    transform,
                    brush,
                    brush_transform,
                    shape,
                } => {
                    let paint = self.paint(brush, &mut used)?;
                    self.scene.set_transform(*transform);
                    self.scene.set_paint(paint);
                    self.scene
                        .set_paint_transform(brush_transform.unwrap_or(Affine::IDENTITY));
                    self.scene.set_stroke(style.clone());
                    self.scene.stroke_path(shape);
                }
                PaintCommand::GlyphRun {
                    font,
                    font_size,
                    hint,
                    normalized_coords,
                    embolden,
                    style,
                    brush,
                    brush_alpha,
                    transform,
                    glyph_transform,
                    glyphs,
                } => {
                    let brush = brush.clone().multiply_alpha(*brush_alpha);
                    let paint = self.paint(&brush, &mut used)?;
                    self.scene.set_transform(*transform);
                    self.scene.set_paint(paint);
                    let glyphs = glyphs.iter().map(|g| glifo::Glyph {
                        id: g.id,
                        x: g.x,
                        y: g.y,
                    });
                    match style {
                        Style::Fill(fill) => {
                            self.scene.set_fill_rule(*fill);
                            self.scene
                                .glyph_run(&mut self.resources, font)
                                .font_size(*font_size)
                                .hint(*hint)
                                .normalized_coords(normalized_coords)
                                .font_embolden(FontEmbolden::new(Diagonal2::new(
                                    embolden.x, embolden.y,
                                )))
                                .glyph_transform(glyph_transform.unwrap_or_default())
                                .fill_glyphs(glyphs)
                                .map_err(|e| e.to_string())?;
                        }
                        Style::Stroke(stroke) => {
                            self.scene.set_stroke(stroke.clone());
                            self.scene
                                .glyph_run(&mut self.resources, font)
                                .font_size(*font_size)
                                .hint(*hint)
                                .normalized_coords(normalized_coords)
                                .glyph_transform(glyph_transform.unwrap_or_default())
                                .stroke_glyphs(glyphs)
                                .map_err(|e| e.to_string())?;
                        }
                    }
                }
                PaintCommand::BoxShadow {
                    transform,
                    rect,
                    brush,
                    radius,
                    std_dev,
                } => {
                    self.scene.set_transform(*transform);
                    self.scene.set_paint(*brush);
                    self.scene.fill_blurred_rounded_rect(
                        rect,
                        *radius as f32,
                        *std_dev as f32,
                        false,
                    );
                }
                PaintCommand::Svg {
                    document,
                    transform,
                } => {
                    crate::svg::append_tree_with_transform(
                        &mut self.scene,
                        &document.0,
                        *transform,
                    )
                    .map_err(|e| format!("SVG projection: {e:?}"))?;
                }
                PaintCommand::Shader { .. } => unreachable!(),
            }
        }
        if !layers.is_empty() {
            return Err("unbalanced paint layers".into());
        }
        // Assigning either canvas dimension clears its drawing buffer, even when
        // the assigned value is unchanged. Keep the previous frame visible
        // while ordinary scene updates are rendered; resize only when the
        // physical viewport actually changed.
        let physical_width = u32::from(width);
        let physical_height = u32::from(height);
        if self.canvas.width() != physical_width {
            self.canvas.set_width(physical_width);
        }
        if self.canvas.height() != physical_height {
            self.canvas.set_height(physical_height);
        }
        self.renderer
            .render(
                &self.scene,
                &mut self.resources,
                &RenderSize { width, height },
                &WebGlTextureBindings::new(),
                Color::TRANSPARENT,
            )
            .map_err(|e| e.to_string())?;
        let retired: Vec<_> = self
            .images
            .iter()
            .filter(|(key, _)| !used.contains(key))
            .map(|(key, id)| (*key, *id))
            .collect();
        for (key, id) in retired {
            self.renderer
                .destroy_image(&mut self.resources, id)
                .map_err(|e| e.to_string())?;
            self.images.remove(&key);
        }
        Ok(())
    }
}
