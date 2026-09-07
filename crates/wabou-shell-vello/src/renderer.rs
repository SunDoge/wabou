//! Headless Vello Hybrid rendering to PNG and RGBA pixels.

use std::{
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use image::ImageEncoder as _;
use snafu::{OptionExt, ResultExt};
use vello_common::peniko::Color;

use crate::{Scene, renderer_backend::HybridImageRenderer};

static IMAGE_RENDERER: OnceLock<Mutex<HybridImageRenderer>> = OnceLock::new();

/// Render a retained Wabou scene to an RGBA image using Vello Hybrid.
pub fn render_to_image(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
) -> crate::Result<image::RgbaImage> {
    let mut renderer = IMAGE_RENDERER
        .get_or_init(|| Mutex::new(HybridImageRenderer::new(width, height)))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    image::RgbaImage::from_raw(
        width,
        height,
        renderer.render(scene, width, height, base_color),
    )
    .context(crate::error::InvalidImageBufferSnafu { width, height })
}

/// Render `scene` and encode it as a PNG at `out_path`.
pub fn render_to_png(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
    out_path: &str,
) -> crate::Result<()> {
    let image = render_to_image(scene, width, height, base_color)?;
    image.save(out_path).context(crate::error::SavePngSnafu {
        path: PathBuf::from(out_path),
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(out_path, std::fs::Permissions::from_mode(0o600)).context(
            crate::error::SecurePngSnafu {
                path: PathBuf::from(out_path),
            },
        )?;
    }
    Ok(())
}

/// Render into an already-open, atomically reserved PNG artifact.
pub fn render_to_png_file(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
    file: &mut std::fs::File,
    path: &Path,
) -> crate::Result<()> {
    let image = render_to_image(scene, width, height, base_color)?;
    image::codecs::png::PngEncoder::new(file)
        .write_image(
            image.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .context(crate::error::SavePngSnafu {
            path: path.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PaintScene;
    use vello_common::{
        kurbo::{Affine, Rect},
        peniko::Fill,
    };

    #[test]
    fn direct_hybrid_renderer_replays_wabou_scene() {
        let mut scene = Scene::new();
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(20, 180, 240),
            None,
            &Rect::new(8.0, 8.0, 24.0, 24.0),
        );
        let image = render_to_image(&scene, 32, 32, Color::BLACK).unwrap();
        assert_eq!(image.get_pixel(16, 16).0, [20, 180, 240, 255]);
        assert_eq!(image.get_pixel(2, 2).0, [0, 0, 0, 255]);
    }

    #[test]
    fn direct_hybrid_renderer_draws_public_raster_images() {
        let raster =
            crate::WidgetRasterImage::from_rgba8(2, 2, [240, 20, 60, 255].repeat(4)).unwrap();
        let mut text = crate::TextContext::new();
        let mut paint = crate::PaintContext::new(16.0, 16.0, 1.0, &mut text);
        paint.draw_raster_image(&raster);
        let image = render_to_image(&paint.finish(), 16, 16, Color::BLACK).unwrap();
        assert_eq!(image.get_pixel(8, 8).0, [240, 20, 60, 255]);
    }

    #[test]
    fn direct_hybrid_renderer_applies_retained_svg_transform() {
        let svg = crate::svg::SvgImage::parse(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><rect width="8" height="8" fill="#14b8a6"/></svg>"##,
        ).unwrap();
        let mut scene = Scene::new();
        scene.draw_svg(svg.document().clone(), Affine::translate((12.0, 6.0)));
        let image = render_to_image(&scene, 32, 24, Color::BLACK).unwrap();
        assert_eq!(image.get_pixel(14, 8).0, [20, 184, 166, 255]);
        assert_eq!(image.get_pixel(2, 2).0, [0, 0, 0, 255]);
    }

    #[test]
    fn direct_hybrid_renderer_composes_svg_groups_and_clip_paths() {
        let svg = crate::svg::SvgImage::parse(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="12">
                <defs><clipPath id="clip"><path d="M0 0h4v4H0z"/></clipPath></defs>
                <g transform="translate(3 2)" clip-path="url(#clip)">
                    <g transform="translate(1 0)"><rect width="8" height="4" fill="#f43f5e"/></g>
                </g>
            </svg>"##,
        )
        .unwrap();
        let mut scene = Scene::new();
        scene.draw_svg(svg.document().clone(), Affine::translate((10.0, 5.0)));
        let image = render_to_image(&scene, 36, 24, Color::BLACK).unwrap();

        assert_eq!(image.get_pixel(15, 8).0, [244, 63, 94, 255]);
        assert_eq!(image.get_pixel(18, 8).0, [0, 0, 0, 255]);
    }
}
