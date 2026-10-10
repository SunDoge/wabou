//! Inspect the host's shaped font/glyph records without opening a window.
use wabou_shell_vello::{PaintCommand, TextContext, text::layout_text};
fn main() {
    let text = std::env::args().nth(1).unwrap_or_else(|| "A🦀😀Z".into());
    let reference = std::env::args()
        .nth(2)
        .map(|path| std::fs::read(path).unwrap());
    let mut context = TextContext::new();
    let layout = layout_text(&mut context, &text, 16.0);
    let scene = context.glyph_scene(&layout);
    let image = wabou_shell_vello::renderer::render_to_image(
        &scene,
        160,
        48,
        vello_common::peniko::Color::WHITE,
    )
    .unwrap();
    let colored = image
        .pixels()
        .filter(|p| p[0].max(p[1]).max(p[2]) - p[0].min(p[1]).min(p[2]) > 30)
        .count();
    println!("colored_pixels={colored}");
    if let Some(output) = std::env::args().nth(3) {
        image.save(output).unwrap();
    }
    let mut matched_font = false;
    for command in &scene.commands {
        if let PaintCommand::GlyphRun {
            font,
            font_size,
            glyphs,
            ..
        } = command
        {
            let bytes = font.data.data();
            matched_font |= reference.as_deref() == Some(bytes);
            println!(
                "font_bytes={} reference_font={} size={} glyphs={:?}",
                bytes.len(),
                reference.as_deref() == Some(bytes),
                font_size,
                glyphs
            );
        }
    }
    if std::env::args().nth(4).as_deref() == Some("--require-color") {
        assert!(matched_font, "reference bitmap font was not used");
        assert!(colored > 0, "bitmap glyph did not produce colored pixels");
    }
}
