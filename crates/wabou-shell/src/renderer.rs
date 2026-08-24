//! Headless (offscreen) rendering to a PNG file.

use std::path::{Path, PathBuf};

use anyrender::{ImageRenderer, PaintScene, Scene};
use image::ImageEncoder as _;
use snafu::{OptionExt, ResultExt};
use vello::kurbo::Affine;
use vello::peniko::Color;

use crate::RendererBackend;

/// Render `scene` to an RGBA8 buffer and encode it as PNG at `out_path`.
pub fn render_to_png(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
    out_path: &str,
) -> crate::Result<()> {
    render_to_png_with_backend(
        scene,
        width,
        height,
        base_color,
        RendererBackend::Vello,
        out_path,
    )
}

/// Render a scene through a selected AnyRender backend and encode it as PNG.
pub fn render_to_png_with_backend(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
    backend: RendererBackend,
    out_path: &str,
) -> crate::Result<()> {
    let img = render_to_image(scene, width, height, base_color, backend)?;
    img.save(out_path).context(crate::error::SavePngSnafu {
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

/// Render `scene` into an already-open, atomically reserved PNG artifact.
pub fn render_to_png_file(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
    file: &mut std::fs::File,
    path: &Path,
) -> crate::Result<()> {
    let img = render_to_image(scene, width, height, base_color, RendererBackend::Vello)?;
    image::codecs::png::PngEncoder::new(file)
        .write_image(img.as_raw(), width, height, image::ExtendedColorType::Rgba8)
        .context(crate::error::SavePngSnafu {
            path: path.to_owned(),
        })?;
    Ok(())
}

fn render_to_image(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
    backend: RendererBackend,
) -> crate::Result<image::RgbaImage> {
    fn render<R: ImageRenderer>(
        scene: &Scene,
        width: u32,
        height: u32,
        base_color: Color,
    ) -> Vec<u8> {
        let mut renderer = R::new(width, height);
        let mut buf = Vec::new();
        renderer.render_to_vec(
            |painter| {
                painter.fill(
                    vello::peniko::Fill::NonZero,
                    Affine::IDENTITY,
                    base_color,
                    None,
                    &vello::kurbo::Rect::new(0.0, 0.0, f64::from(width), f64::from(height)),
                );
                painter.append_scene(scene.clone(), Affine::IDENTITY);
            },
            &mut buf,
        );
        buf
    }

    let buf = match backend {
        RendererBackend::Vello => {
            render::<anyrender_vello::VelloImageRenderer>(scene, width, height, base_color)
        }
        RendererBackend::Skia => {
            #[cfg(feature = "renderer-skia")]
            {
                render::<anyrender_skia::SkiaImageRenderer>(scene, width, height, base_color)
            }
            #[cfg(not(feature = "renderer-skia"))]
            {
                return Err(crate::Error::RendererBackendUnavailable {
                    backend: "skia",
                    feature: "`wabou-shell/renderer-skia`",
                });
            }
        }
    };

    image::RgbaImage::from_raw(width, height, buf)
        .context(crate::error::InvalidImageBufferSnafu { width, height })
}

#[cfg(test)]
mod tests {
    use super::*;
    use vello::kurbo::Rect;
    use vello::peniko::Fill;

    fn comparison_scene() -> Scene {
        let mut scene = Scene::new();
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(20, 180, 240),
            None,
            &Rect::new(8.0, 8.0, 24.0, 24.0),
        );
        scene
    }

    #[test]
    fn default_offscreen_renderer_replays_anyrender_scene() {
        let image = render_to_image(
            &comparison_scene(),
            32,
            32,
            Color::BLACK,
            RendererBackend::Vello,
        )
        .unwrap();
        assert_eq!(image.get_pixel(16, 16).0, [20, 180, 240, 255]);
        assert_eq!(image.get_pixel(2, 2).0, [0, 0, 0, 255]);
    }

    #[cfg(feature = "renderer-skia")]
    #[test]
    fn skia_offscreen_renderer_replays_the_same_anyrender_scene() {
        let image = render_to_image(
            &comparison_scene(),
            32,
            32,
            Color::BLACK,
            RendererBackend::Skia,
        )
        .unwrap();
        assert_eq!(image.get_pixel(16, 16).0, [20, 180, 240, 255]);
        assert_eq!(image.get_pixel(2, 2).0, [0, 0, 0, 255]);
    }
}
