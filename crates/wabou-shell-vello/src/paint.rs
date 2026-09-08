//! Wabou's retained 2D paint command stream.
//!
//! Vello Hybrid builds viewport-specific strips eagerly, so widgets cannot retain
//! and append a `vello_hybrid::Scene` directly. This small command stream is the
//! stable fragment boundary owned by Wabou; the window and headless renderers
//! replay it straight into Vello Hybrid at the end of a frame.

use std::sync::Arc;

use vello_common::kurbo::{Affine, BezPath, Rect, Shape, Stroke, Vec2};
use vello_common::peniko::{
    BlendMode, Brush, BrushRef, Color, Fill, FontData, ImageBrushRef, Style, StyleRef,
};

use crate::ShaderEffect;

/// Font variation coordinate in normalized OpenType representation.
pub type NormalizedCoord = i16;

/// A positioned glyph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    /// Glyph identifier in the selected font.
    pub id: u32,
    /// Horizontal glyph origin.
    pub x: f32,
    /// Vertical glyph origin.
    pub y: f32,
}

/// Parsed SVG content retained independently of the viewport-specific Hybrid scene.
#[derive(Clone)]
pub struct SvgDocument(pub Arc<crate::svg::usvg::Tree>);

impl std::fmt::Debug for SvgDocument {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SvgDocument")
            .finish_non_exhaustive()
    }
}

impl PartialEq for SvgDocument {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

/// One retained drawing operation.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum PaintCommand {
    /// Begin an isolated, clipped layer.
    PushLayer {
        /// Compositing mode.
        blend: BlendMode,
        /// Layer opacity.
        alpha: f32,
        /// Clip transform.
        transform: Affine,
        /// Clip outline.
        clip: BezPath,
        /// Optional Vello Hybrid filter.
        filter: Option<Arc<vello_common::filter_effects::Filter>>,
    },
    /// Begin a clip-only scope.
    PushClip { transform: Affine, clip: BezPath },
    /// End the most recent layer or clip.
    PopLayer,
    /// Stroke a path.
    Stroke {
        style: Stroke,
        transform: Affine,
        brush: Brush,
        brush_transform: Option<Affine>,
        shape: BezPath,
    },
    /// Fill a path.
    Fill {
        fill: Fill,
        transform: Affine,
        brush: Brush,
        brush_transform: Option<Affine>,
        shape: BezPath,
    },
    /// Draw positioned glyphs.
    GlyphRun {
        font: FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: Vec<NormalizedCoord>,
        embolden: Vec2,
        style: Style,
        brush: Brush,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: Vec<Glyph>,
    },
    /// Draw a blurred rounded rectangle.
    BoxShadow {
        transform: Affine,
        rect: Rect,
        brush: Color,
        radius: f64,
        std_dev: f64,
    },
    /// Draw an already parsed SVG document.
    Svg {
        document: SvgDocument,
        transform: Affine,
    },
    /// Draw a renderer-owned custom WGSL effect surface.
    Shader {
        effect: ShaderEffect,
        transform: Affine,
    },
}

impl PaintCommand {
    fn transformed(mut self, parent: Affine) -> Self {
        let transform = match &mut self {
            Self::PushLayer { transform, .. }
            | Self::PushClip { transform, .. }
            | Self::Stroke { transform, .. }
            | Self::Fill { transform, .. }
            | Self::GlyphRun { transform, .. }
            | Self::BoxShadow { transform, .. }
            | Self::Svg { transform, .. }
            | Self::Shader { transform, .. } => transform,
            Self::PopLayer => return self,
        };
        *transform = parent * *transform;
        self
    }
}

/// Cloneable, backend-specific retained scene fragment used by Wabou widgets.
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    tolerance: f64,
    /// Recorded paint commands.
    pub commands: Vec<PaintCommand>,
}

impl Default for Scene {
    fn default() -> Self {
        Self {
            tolerance: 0.1,
            commands: Vec::new(),
        }
    }
}

impl Scene {
    /// Create an empty scene fragment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an empty scene with a path flattening tolerance.
    pub fn with_tolerance(tolerance: f64) -> Self {
        Self {
            tolerance,
            commands: Vec::new(),
        }
    }
}

/// Minimal paint sink shared by Wabou scene construction and the Hybrid replay.
pub trait PaintScene {
    /// Remove every recorded operation.
    fn reset(&mut self);
    /// Push an isolated clipped layer.
    fn push_layer(
        &mut self,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        clip: &impl Shape,
        filter: Option<Arc<vello_common::filter_effects::Filter>>,
        backdrop_filter: Option<Arc<vello_common::filter_effects::Filter>>,
    );
    /// Push a clip-only scope.
    fn push_clip_layer(&mut self, transform: Affine, clip: &impl Shape);
    /// Pop the most recent layer or clip.
    fn pop_layer(&mut self);
    /// Stroke a shape.
    fn stroke<'a>(
        &mut self,
        style: &Stroke,
        transform: Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    );
    /// Fill a shape.
    fn fill<'a>(
        &mut self,
        fill: Fill,
        transform: Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    );
    /// Draw positioned glyphs.
    #[allow(clippy::too_many_arguments)]
    fn draw_glyphs<'a, 's: 'a>(
        &'s mut self,
        font: &'a FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: &'a [NormalizedCoord],
        embolden: Vec2,
        style: impl Into<StyleRef<'a>>,
        brush: impl Into<BrushRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    );
    /// Draw a blurred box shadow.
    fn draw_box_shadow(
        &mut self,
        transform: Affine,
        rect: Rect,
        brush: Color,
        radius: f64,
        std_dev: f64,
    );
    /// Draw a retained SVG document.
    fn draw_svg(&mut self, document: SvgDocument, transform: Affine);
    /// Draw a retained custom WGSL effect.
    fn draw_shader_effect(&mut self, effect: ShaderEffect, transform: Affine);

    /// Append a retained fragment under `transform`.
    fn append_scene(&mut self, scene: Scene, transform: Affine) {
        for command in scene.commands {
            match command {
                PaintCommand::PushLayer {
                    blend,
                    alpha,
                    transform: local,
                    clip,
                    filter,
                } => self.push_layer(blend, alpha, transform * local, &clip, filter, None),
                PaintCommand::PushClip {
                    transform: local,
                    clip,
                } => self.push_clip_layer(transform * local, &clip),
                PaintCommand::PopLayer => self.pop_layer(),
                PaintCommand::Stroke {
                    style,
                    transform: local,
                    brush,
                    brush_transform,
                    shape,
                } => self.stroke(&style, transform * local, &brush, brush_transform, &shape),
                PaintCommand::Fill {
                    fill,
                    transform: local,
                    brush,
                    brush_transform,
                    shape,
                } => self.fill(fill, transform * local, &brush, brush_transform, &shape),
                PaintCommand::GlyphRun {
                    font,
                    font_size,
                    hint,
                    normalized_coords,
                    embolden,
                    style,
                    brush,
                    brush_alpha,
                    transform: local,
                    glyph_transform,
                    glyphs,
                } => self.draw_glyphs(
                    &font,
                    font_size,
                    hint,
                    &normalized_coords,
                    embolden,
                    &style,
                    &brush,
                    brush_alpha,
                    transform * local,
                    glyph_transform,
                    glyphs.into_iter(),
                ),
                PaintCommand::BoxShadow {
                    transform: local,
                    rect,
                    brush,
                    radius,
                    std_dev,
                } => self.draw_box_shadow(transform * local, rect, brush, radius, std_dev),
                PaintCommand::Svg {
                    document,
                    transform: local,
                } => self.draw_svg(document, transform * local),
                PaintCommand::Shader {
                    effect,
                    transform: local,
                } => self.draw_shader_effect(effect, transform * local),
            }
        }
    }

    /// Draw an image at its intrinsic size.
    fn draw_image(&mut self, image: ImageBrushRef<'_>, transform: Affine) {
        let bounds = Rect::new(
            0.0,
            0.0,
            f64::from(image.image.width),
            f64::from(image.image.height),
        );
        self.fill(Fill::NonZero, transform, image, None, &bounds);
    }
}

fn own_brush(brush: BrushRef<'_>) -> Brush {
    match brush {
        BrushRef::Solid(color) => Brush::Solid(color),
        BrushRef::Gradient(gradient) => Brush::Gradient(gradient.clone()),
        BrushRef::Image(image) => Brush::Image(image.to_owned()),
    }
}

impl PaintScene for Scene {
    fn reset(&mut self) {
        self.commands.clear();
    }

    fn push_layer(
        &mut self,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        clip: &impl Shape,
        filter: Option<Arc<vello_common::filter_effects::Filter>>,
        _backdrop_filter: Option<Arc<vello_common::filter_effects::Filter>>,
    ) {
        self.commands.push(PaintCommand::PushLayer {
            blend: blend.into(),
            alpha,
            transform,
            clip: clip.into_path(self.tolerance),
            filter,
        });
    }

    fn push_clip_layer(&mut self, transform: Affine, clip: &impl Shape) {
        self.commands.push(PaintCommand::PushClip {
            transform,
            clip: clip.into_path(self.tolerance),
        });
    }

    fn pop_layer(&mut self) {
        self.commands.push(PaintCommand::PopLayer);
    }

    fn stroke<'a>(
        &mut self,
        style: &Stroke,
        transform: Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.commands.push(PaintCommand::Stroke {
            style: style.clone(),
            transform,
            brush: own_brush(brush.into()),
            brush_transform,
            shape: shape.into_path(self.tolerance),
        });
    }

    fn fill<'a>(
        &mut self,
        fill: Fill,
        transform: Affine,
        brush: impl Into<BrushRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.commands.push(PaintCommand::Fill {
            fill,
            transform,
            brush: own_brush(brush.into()),
            brush_transform,
            shape: shape.into_path(self.tolerance),
        });
    }

    fn draw_glyphs<'a, 's: 'a>(
        &'s mut self,
        font: &'a FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: &'a [NormalizedCoord],
        embolden: Vec2,
        style: impl Into<StyleRef<'a>>,
        brush: impl Into<BrushRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        self.commands.push(PaintCommand::GlyphRun {
            font: font.clone(),
            font_size,
            hint,
            normalized_coords: normalized_coords.to_vec(),
            embolden,
            style: style.into().to_owned(),
            brush: own_brush(brush.into()),
            brush_alpha,
            transform,
            glyph_transform,
            glyphs: glyphs.collect(),
        });
    }

    fn draw_box_shadow(
        &mut self,
        transform: Affine,
        rect: Rect,
        brush: Color,
        radius: f64,
        std_dev: f64,
    ) {
        self.commands.push(PaintCommand::BoxShadow {
            transform,
            rect,
            brush,
            radius,
            std_dev,
        });
    }

    fn draw_svg(&mut self, document: SvgDocument, transform: Affine) {
        self.commands.push(PaintCommand::Svg {
            document,
            transform,
        });
    }

    fn append_scene(&mut self, scene: Scene, transform: Affine) {
        self.commands.extend(
            scene
                .commands
                .into_iter()
                .map(|command| command.transformed(transform)),
        );
    }

    fn draw_shader_effect(&mut self, effect: ShaderEffect, transform: Affine) {
        self.commands
            .push(PaintCommand::Shader { effect, transform });
    }
}
