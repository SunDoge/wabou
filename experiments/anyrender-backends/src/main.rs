use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use anyrender::filters::FilterEffect;
use anyrender::recording::Scene;
use anyrender::{Filter, ImageRenderer, PaintScene};
use anyrender_skia::SkiaImageRenderer;
use anyrender_vello::VelloImageRenderer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use kurbo::{Affine, Circle, Rect, RoundedRect, Stroke};
use peniko::{BlendMode, Color, ColorStop, Fill, Gradient};

const WIDTH: u32 = 960;
const HEIGHT: u32 = 560;

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("out"));
    std::fs::create_dir_all(&output).expect("create output directory");

    let mut recording = Scene::new();
    draw_comparison_scene(&mut recording);

    render::<SkiaImageRenderer>("skia", &recording, &output);
    render::<VelloImageRenderer>("vello", &recording, &output);
    render::<VelloCpuImageRenderer>("vello-cpu", &recording, &output);

    println!("vello-hybrid: skipped (anyrender_vello_hybrid 0.10 has no ImageRenderer)");
}

fn render<R: ImageRenderer>(name: &str, recording: &Scene, output: &Path) {
    let started = Instant::now();
    let mut renderer = R::new(WIDTH, HEIGHT);
    let initialized = started.elapsed();

    let started = Instant::now();
    let mut pixels = Vec::new();
    renderer.render_to_vec(
        |scene| scene.append_scene(recording.clone(), Affine::IDENTITY),
        &mut pixels,
    );
    let rendered = started.elapsed();

    let path = output.join(format!("{name}.png"));
    image::save_buffer(
        &path,
        &pixels,
        WIDTH,
        HEIGHT,
        image::ExtendedColorType::Rgba8,
    )
    .expect("save PNG");
    println!(
        "{name}: init={:.2?} render={:.2?} output={}",
        initialized,
        rendered,
        path.display()
    );
}

fn draw_comparison_scene(scene: &mut impl PaintScene) {
    let viewport = Rect::new(0.0, 0.0, f64::from(WIDTH), f64::from(HEIGHT));
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgb8(12, 16, 28),
        None,
        &viewport,
    );

    draw_panel(scene, Rect::new(40.0, 40.0, 450.0, 520.0));
    draw_panel(scene, Rect::new(510.0, 40.0, 920.0, 520.0));
    draw_common_orb(scene, (245.0, 270.0));
    draw_filtered_orb(scene, (715.0, 270.0));
}

fn draw_panel(scene: &mut impl PaintScene, rect: Rect) {
    scene.draw_box_shadow(
        Affine::IDENTITY,
        rect,
        Color::from_rgba8(0, 0, 0, 150),
        28.0,
        18.0,
    );
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        Color::from_rgb8(25, 31, 48),
        None,
        &RoundedRect::from_rect(rect, 28.0),
    );
    scene.stroke(
        &Stroke::new(1.0),
        Affine::IDENTITY,
        Color::from_rgba8(255, 255, 255, 32),
        None,
        &RoundedRect::from_rect(rect, 28.0),
    );
}

fn draw_common_orb(scene: &mut impl PaintScene, center: (f64, f64)) {
    let circle = Circle::new(center, 142.0);
    scene.draw_box_shadow(
        Affine::IDENTITY,
        Rect::new(
            center.0 - 125.0,
            center.1 - 125.0,
            center.0 + 125.0,
            center.1 + 125.0,
        ),
        Color::from_rgba8(82, 78, 255, 165),
        125.0,
        34.0,
    );
    let base = Gradient::new_sweep(center, -0.7, 5.58).with_stops([
        ColorStop {
            offset: 0.0,
            color: Color::from_rgb8(64, 51, 219).into(),
        },
        ColorStop {
            offset: 0.28,
            color: Color::from_rgb8(30, 210, 245).into(),
        },
        ColorStop {
            offset: 0.56,
            color: Color::from_rgb8(143, 67, 255).into(),
        },
        ColorStop {
            offset: 0.8,
            color: Color::from_rgb8(250, 70, 183).into(),
        },
        ColorStop {
            offset: 1.0,
            color: Color::from_rgb8(64, 51, 219).into(),
        },
    ]);
    scene.fill(Fill::NonZero, Affine::IDENTITY, &base, None, &circle);

    let light = Gradient::new_radial((195.0, 216.0), 190.0).with_stops([
        ColorStop {
            offset: 0.0,
            color: Color::from_rgba8(255, 255, 255, 210).into(),
        },
        ColorStop {
            offset: 0.22,
            color: Color::from_rgba8(163, 239, 255, 95).into(),
        },
        ColorStop {
            offset: 1.0,
            color: Color::TRANSPARENT.into(),
        },
    ]);
    scene.fill(Fill::NonZero, Affine::IDENTITY, &light, None, &circle);
    draw_rim(scene, circle);
}

fn draw_filtered_orb(scene: &mut impl PaintScene, center: (f64, f64)) {
    let circle = Circle::new(center, 142.0);
    scene.draw_box_shadow(
        Affine::IDENTITY,
        Rect::new(
            center.0 - 125.0,
            center.1 - 125.0,
            center.0 + 125.0,
            center.1 + 125.0,
        ),
        Color::from_rgba8(240, 56, 190, 170),
        125.0,
        38.0,
    );
    let filter = Arc::new(Filter::single(FilterEffect::blur(18.0)));
    scene.push_layer(
        BlendMode::default(),
        1.0,
        Affine::IDENTITY,
        &circle,
        Some(filter),
        None,
    );
    let base = Gradient::new_linear((580.0, 120.0), (850.0, 420.0)).with_stops([
        ColorStop {
            offset: 0.0,
            color: Color::from_rgb8(255, 225, 250).into(),
        },
        ColorStop {
            offset: 0.22,
            color: Color::from_rgb8(255, 52, 178).into(),
        },
        ColorStop {
            offset: 0.5,
            color: Color::from_rgb8(97, 38, 218).into(),
        },
        ColorStop {
            offset: 0.72,
            color: Color::from_rgb8(38, 211, 241).into(),
        },
        ColorStop {
            offset: 1.0,
            color: Color::from_rgb8(10, 15, 48).into(),
        },
    ]);
    scene.fill(Fill::NonZero, Affine::IDENTITY, &base, None, &circle);
    for (x, y, radius, color) in [
        (662.0, 215.0, 76.0, Color::from_rgba8(255, 255, 255, 190)),
        (780.0, 225.0, 92.0, Color::from_rgba8(248, 66, 215, 180)),
        (690.0, 340.0, 86.0, Color::from_rgba8(37, 225, 255, 180)),
    ] {
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            color,
            None,
            &Circle::new((x, y), radius),
        );
    }
    scene.pop_layer();
    draw_rim(scene, circle);
}

fn draw_rim(scene: &mut impl PaintScene, circle: Circle) {
    scene.stroke(
        &Stroke::new(2.0),
        Affine::IDENTITY,
        Color::from_rgba8(255, 255, 255, 125),
        None,
        &circle,
    );
}
