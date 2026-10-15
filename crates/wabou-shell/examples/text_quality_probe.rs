//! Render shared text-quality cases through the production Parley/Glifo path.

use std::path::PathBuf;
use std::sync::Arc;
use std::{fmt::Write as _, fs};

use parley::PositionedLayoutItem;
use serde::Deserialize;
use vello_common::kurbo::Affine;
use vello_common::peniko::Color;
use wabou_shell::style::TextAlign;
use wabou_shell::text::{TextRun, layout_text_styled};
use wabou_shell::{PaintContext, PaintScene, Scene, TextContext, renderer};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Cases {
    width: u32,
    height: u32,
    background: String,
    foreground: String,
    font_family: String,
    scales: Vec<u32>,
    rows: Vec<Row>,
}

#[derive(Deserialize)]
struct Row {
    text: String,
    size: f32,
    weight: f32,
    x: f32,
    baseline: f32,
}

fn color(value: &str) -> Color {
    let value = value.strip_prefix('#').unwrap_or(value);
    let packed = u32::from_str_radix(value, 16).expect("six-digit hex color");
    Color::from_rgb8(
        ((packed >> 16) & 0xff) as u8,
        ((packed >> 8) & 0xff) as u8,
        (packed & 0xff) as u8,
    )
}

fn rgba(value: &str) -> [u8; 4] {
    let color = color(value).to_rgba8();
    [color.r, color.g, color.b, color.a]
}

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let manifest = std::env::args_os()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tools/text-quality/cases.json"));
    let cases: Cases = serde_json::from_slice(&std::fs::read(&manifest).unwrap_or_else(|error| {
        panic!("read text quality manifest {}: {error}", manifest.display())
    }))
    .expect("parse text quality manifest");
    std::fs::create_dir_all(&output).expect("create output directory");

    for scale in cases.scales.iter().copied() {
        let logical_size = [cases.width as f32, cases.height as f32];
        let mut text = TextContext::new();
        let foreground = rgba(&cases.foreground);
        let family: Arc<str> = Arc::from(cases.font_family.as_str());
        let layouts = cases
            .rows
            .iter()
            .map(|row| {
                layout_text_styled(
                    &mut text,
                    Arc::from(row.text.as_str()),
                    row.size,
                    row.weight,
                    false,
                    None,
                    TextAlign::Start,
                    foreground,
                    Arc::<[TextRun]>::from([]),
                    Some(&family),
                    None,
                )
            })
            .collect::<Vec<_>>();
        let mut report = format!(
            "backend=wabou-parley-vello\nscale={scale}\nrequested_family={}\n",
            cases.font_family
        );
        for (row_index, (row, layout)) in cases.rows.iter().zip(&layouts).enumerate() {
            let line = layout.lines().next().expect("text quality row has a line");
            let metrics = line.metrics();
            writeln!(
                report,
                "row={row_index} size={} weight={} baseline={} advance={} line_height={} text={}",
                row.size,
                row.weight,
                metrics.baseline,
                metrics.advance,
                metrics.line_height,
                row.text
            )
            .expect("write Wabou text report");
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                write!(
                    report,
                    "  font_blob={:?} font_index={} glyphs=",
                    run.font().data.id(),
                    run.font().index
                )
                .expect("write Wabou run report");
                for (index, glyph) in glyph_run.positioned_glyphs().enumerate() {
                    if index > 0 {
                        report.push(',');
                    }
                    write!(report, "{}:{:.3}:{:.3}", glyph.id, glyph.x, glyph.y)
                        .expect("write Wabou glyph report");
                }
                report.push('\n');
            }
        }
        fs::write(
            output.join(format!("wabou-text-quality-{scale}x.txt")),
            report,
        )
        .expect("write Wabou text report");
        let mut paint = PaintContext::new(
            logical_size[0],
            logical_size[1],
            f64::from(scale),
            &mut text,
        );
        for (row, layout) in cases.rows.iter().zip(&layouts) {
            let layout_baseline = layout
                .lines()
                .next()
                .expect("text quality row has a line")
                .metrics()
                .baseline;
            paint.draw_text_layout(
                layout,
                [f64::from(row.x), f64::from(row.baseline - layout_baseline)],
            );
        }
        // PaintContext returns a logical widget fragment. Production appends
        // that fragment under the window's device transform, so reproduce the
        // same final step before rendering a physical comparison image.
        let fragment = paint.finish();
        let mut scene = Scene::new();
        scene.append_scene(fragment, Affine::scale(f64::from(scale)));
        let path = output.join(format!("wabou-text-quality-{scale}x.png"));
        renderer::render_to_png(
            &scene,
            cases.width * scale,
            cases.height * scale,
            color(&cases.background),
            &path.to_string_lossy(),
        )
        .expect("render text quality sheet");
        println!("{}", path.display());
    }
}
