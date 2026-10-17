//! Render the first page of a PDF through hayro and Wabou's retained image path.

use std::path::PathBuf;

use vello_common::{kurbo::Affine, peniko::Color};
use wabou_pdf::{PageRenderOptions, PdfDocument};
use wabou_shell::{PaintContext, PaintScene, Scene, TextContext, renderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: render_page <input.pdf> [output.png] [scale]")?;
    let output = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("pdf-page.png"));
    let scale = std::env::args()
        .nth(3)
        .map(|value| value.parse::<f32>())
        .transpose()?
        .unwrap_or(2.0);

    let document = PdfDocument::from_bytes(std::fs::read(&input)?)?;
    let page = document.render_page(
        0,
        PageRenderOptions {
            scale,
            ..PageRenderOptions::default()
        },
    )?;
    let page_pixels = [page.width(), page.height()];
    let page_image = page.into_widget_image()?;

    let logical_width = page_pixels[0] as f32 / scale;
    let logical_height = page_pixels[1] as f32 / scale;
    let margin = 24.0_f32;
    let output_width = ((logical_width + margin * 2.0) * scale).round() as u32;
    let output_height = ((logical_height + margin * 2.0) * scale).round() as u32;
    let mut text = TextContext::new();
    let mut paint = PaintContext::new(logical_width, logical_height, f64::from(scale), &mut text);
    paint.draw_raster_image(&page_image);
    let fragment = paint.finish();
    let mut scene = Scene::new();
    scene.append_scene(
        fragment,
        Affine::translate((f64::from(margin), f64::from(margin))) * Affine::scale(f64::from(scale)),
    );
    renderer::render_to_png(
        &scene,
        output_width,
        output_height,
        Color::from_rgb8(232, 232, 234),
        &output.to_string_lossy(),
    )?;

    println!(
        "rendered page 1/{} at {}x{} px to {}",
        document.page_count(),
        page_pixels[0],
        page_pixels[1],
        output.display()
    );
    Ok(())
}
