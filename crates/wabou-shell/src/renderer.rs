//! Headless (offscreen) rendering to a PNG file.

use std::num::NonZeroUsize;
use std::path::PathBuf;

use snafu::{OptionExt, ResultExt};
use vello::peniko::Color;
use vello::wgpu::TextureUsages;
use vello::{AaConfig, AaSupport, RenderParams, Renderer as VelloRenderer, RendererOptions, Scene};
use wgpu_context::{BufferRendererConfig, WGPUContext};

/// Render `scene` to an RGBA8 buffer and encode it as PNG at `out_path`.
pub fn render_to_png(
    scene: &Scene,
    width: u32,
    height: u32,
    base_color: Color,
    out_path: &str,
) -> crate::Result<()> {
    let mut context = WGPUContext::new();
    let buffer_renderer =
        pollster::block_on(context.create_buffer_renderer(BufferRendererConfig {
            width,
            height,
            usage: TextureUsages::STORAGE_BINDING,
        }))
        .context(crate::error::CreateBufferRendererSnafu)?;

    let mut renderer = VelloRenderer::new(
        buffer_renderer.device(),
        RendererOptions {
            use_cpu: false,
            antialiasing_support: AaSupport::area_only(),
            num_init_threads: NonZeroUsize::new(1),
            pipeline_cache: None,
        },
    )
    .context(crate::error::CreateVelloRendererSnafu)?;

    let view = buffer_renderer.target_texture_view();
    renderer
        .render_to_texture(
            buffer_renderer.device(),
            buffer_renderer.queue(),
            scene,
            &view,
            &RenderParams {
                base_color,
                width,
                height,
                antialiasing_method: AaConfig::Area,
            },
        )
        .context(crate::error::RenderSceneSnafu)?;

    let mut buf = vec![0u8; (width as usize) * (height as usize) * 4];
    buffer_renderer.copy_texture_to_buffer(&mut buf);

    let img = image::RgbaImage::from_raw(width, height, buf)
        .context(crate::error::InvalidImageBufferSnafu { width, height })?;
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
