//! Hinted raster text fragments for ordinary axis-aligned UI text.

use std::sync::Arc;

use anyrender::{PaintScene, Scene};
use parley::{Layout, PositionedLayoutItem};
use swash::scale::{Render, ScaleContext, Source, StrikeWith, image::Content};
use vello::kurbo::Affine;
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};

const SUBPIXEL_VARIANTS: f32 = 4.0;
const MAX_RASTER_DIMENSION: u32 = 2048;
const MAX_RASTER_PIXELS: u64 = 512 * 1024;

fn quantized_physical_baseline(logical_y: f32, scale: f32, origin_y: f32) -> f32 {
    (logical_y.mul_add(scale, origin_y)).round()
}

struct RasterGlyph {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    content: Content,
    data: Vec<u8>,
    color: [u8; 4],
}

pub(super) fn rasterize_layout(
    layout: &Layout<[u8; 4]>,
    device_scale: f64,
    subpixel_variant: [u8; 2],
    scale_cx: &mut ScaleContext,
) -> Option<Scene> {
    let scale = device_scale as f32;
    let origin_x = f32::from(subpixel_variant[0].min(3)) / SUBPIXEL_VARIANTS;
    let origin_y = f32::from(subpixel_variant[1].min(3)) / SUBPIXEL_VARIANTS;
    let mut glyphs = Vec::new();
    let mut bounds = [i32::MAX, i32::MAX, i32::MIN, i32::MIN];

    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let run = glyph_run.run();
            // A synthetic skew needs transformed raster bounds, so retain its
            // vector fallback. Synthetic emboldening is deliberately ignored:
            // many CJK fallback families lack an exact 500 face and rendering
            // the nearest native face through hinted Swash is both sharper and
            // less artificially heavy than falling back to unhinted outlines.
            if run.synthesis().skew().is_some() {
                return None;
            }
            let font_data = run.font();
            let font =
                swash::FontRef::from_index(font_data.data.as_ref(), font_data.index as usize)?;
            let mut scaler = scale_cx
                .builder(font)
                .size(run.font_size() * scale)
                .hint(true)
                .normalized_coords(run.normalized_coords())
                .build();
            let sources = [
                Source::ColorOutline(0),
                Source::ColorBitmap(StrikeWith::BestFit),
                Source::Bitmap(StrikeWith::ExactSize),
                Source::Outline,
            ];
            for glyph in glyph_run.positioned_glyphs() {
                let glyph_id = u16::try_from(glyph.id).ok()?;
                let physical_x = glyph.x * scale + origin_x;
                // Parley layouts are kept in logical coordinates so layout
                // remains stable across monitors. Quantize the baseline only
                // after applying the final device scale and text origin, as
                // required by Parley's non-quantized builder contract. Blitz
                // gets the same property from its scale-aware quantized builder.
                let physical_y = quantized_physical_baseline(glyph.y, scale, origin_y);
                let base_x = physical_x.floor() as i32;
                let base_y = physical_y.floor() as i32;
                let mut renderer = Render::new(&sources);
                renderer
                    .format(swash::zeno::Format::Alpha)
                    .offset(swash::zeno::Vector::new(
                        physical_x.fract(),
                        physical_y.fract(),
                    ))
                    .default_color(glyph_run.style().brush);
                let image = renderer.render(&mut scaler, glyph_id)?;
                if image.placement.width == 0 || image.placement.height == 0 {
                    continue;
                }
                let x = base_x + image.placement.left;
                let y = base_y - image.placement.top;
                let right = x.checked_add(image.placement.width as i32)?;
                let bottom = y.checked_add(image.placement.height as i32)?;
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(right);
                bounds[3] = bounds[3].max(bottom);
                glyphs.push(RasterGlyph {
                    x,
                    y,
                    width: image.placement.width,
                    height: image.placement.height,
                    content: image.content,
                    data: image.data,
                    color: glyph_run.style().brush,
                });
            }
        }
    }

    if glyphs.is_empty() {
        return None;
    }
    let width = u32::try_from(bounds[2].checked_sub(bounds[0])?).ok()?;
    let height = u32::try_from(bounds[3].checked_sub(bounds[1])?).ok()?;
    if width == 0
        || height == 0
        || width > MAX_RASTER_DIMENSION
        || height > MAX_RASTER_DIMENSION
        || u64::from(width) * u64::from(height) > MAX_RASTER_PIXELS
    {
        return None;
    }

    let mut rgba = vec![0_u8; width as usize * height as usize * 4];
    for glyph in glyphs {
        composite_glyph(&mut rgba, width, bounds, &glyph);
    }
    let data = ImageData {
        data: Blob::new(Arc::new(rgba)),
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    };
    let mut scene = Scene::new();
    scene.draw_image(
        (&ImageBrush::new(data)).into(),
        Affine::translate((f64::from(bounds[0]), f64::from(bounds[1]))),
    );
    Some(scene)
}

fn composite_glyph(rgba: &mut [u8], target_width: u32, bounds: [i32; 4], glyph: &RasterGlyph) {
    for row in 0..glyph.height as usize {
        for col in 0..glyph.width as usize {
            let pixel = row * glyph.width as usize + col;
            let (rgb, coverage, style_alpha) = match glyph.content {
                Content::Mask => (
                    [glyph.color[0], glyph.color[1], glyph.color[2]],
                    glyph.data[pixel],
                    glyph.color[3],
                ),
                Content::Color | Content::SubpixelMask => {
                    let offset = pixel * 4;
                    (
                        [
                            glyph.data[offset],
                            glyph.data[offset + 1],
                            glyph.data[offset + 2],
                        ],
                        glyph.data[offset + 3],
                        glyph.color[3],
                    )
                }
            };
            let source_alpha = u16::from(coverage) * u16::from(style_alpha) / u16::from(u8::MAX);
            if source_alpha == 0 {
                continue;
            }
            let x = (glyph.x - bounds[0]) as usize + col;
            let y = (glyph.y - bounds[1]) as usize + row;
            let destination = (y * target_width as usize + x) * 4;
            blend_source_over(&mut rgba[destination..destination + 4], rgb, source_alpha);
        }
    }
}

fn blend_source_over(destination: &mut [u8], source_rgb: [u8; 3], source_alpha: u16) {
    let destination_alpha = u16::from(destination[3]);
    let inverse_source = u16::from(u8::MAX) - source_alpha;
    let output_alpha = source_alpha + destination_alpha * inverse_source / u16::from(u8::MAX);
    for channel in 0..3 {
        let premultiplied = u32::from(source_rgb[channel]) * u32::from(source_alpha)
            + u32::from(destination[channel])
                * u32::from(destination_alpha)
                * u32::from(inverse_source)
                / u32::from(u8::MAX);
        destination[channel] = (premultiplied / u32::from(output_alpha.max(1))) as u8;
    }
    destination[3] = output_alpha as u8;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_over_preserves_straight_alpha_colors() {
        let mut destination = [0, 0, 255, 128];
        blend_source_over(&mut destination, [255, 0, 0], 128);
        assert_eq!(destination[3], 191);
        assert!(destination[0] > destination[2]);
        assert_eq!(destination[1], 0);
    }

    #[test]
    fn baseline_is_quantized_after_device_scale_and_origin() {
        assert_eq!(quantized_physical_baseline(12.2, 1.0, 0.25), 12.0);
        assert_eq!(quantized_physical_baseline(12.2, 1.25, 0.25), 16.0);
        assert_eq!(quantized_physical_baseline(12.2, 2.0, 0.25), 25.0);
    }
}
