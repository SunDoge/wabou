//! JuliaWidget — a morphing Julia set. JS animates the Julia parameter `c`
//! around the classic "beauty circle" (radius 0.7885 in c-space); Rust does
//! the per-pixel iteration + paints a vello image. The morph is a closed
//! loop (c returns exactly to its start) so there's no snap, no precision
//! wall (coordinates stay in [-1.5,1.5]), and it never ends — an infinite,
//! seamless, high-clarity animation.

use std::sync::Arc;

use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use wabou_shell::UiEvent;
use wabou_shell::text::TextContext;

use super::{Widget, WidgetEventResult};

const RENDER_SIZE: u32 = 480;
const MAX_ITER: u32 = 160;
const VIEW: f64 = 1.5;

pub struct JuliaWidget {
    cx: f64,
    cy: f64,
    cached: Option<(f64, f64)>,
    image: Option<ImageBrush>,
}

impl JuliaWidget {
    pub fn new() -> Self {
        Self {
            cx: 0.7885,
            cy: 0.0,
            cached: None,
            image: None,
        }
    }

    fn render(&self) -> Vec<u8> {
        let w = RENDER_SIZE;
        let h = RENDER_SIZE;
        let mut rgba = vec![0u8; (w * h * 4) as usize];
        let scale = 2.0 * VIEW / w as f64;
        for y in 0..h {
            let zy = (y as f64 - h as f64 / 2.0) * scale;
            for x in 0..w {
                let zx = (x as f64 - w as f64 / 2.0) * scale;
                let (res, iter) = julia_iter(zx, zy, self.cx, self.cy);
                let (r, g, b) = paint(res, iter);
                let i = ((y * w + x) * 4) as usize;
                rgba[i] = r;
                rgba[i + 1] = g;
                rgba[i + 2] = b;
                rgba[i + 3] = 255;
            }
        }
        rgba
    }
}

impl Widget for JuliaWidget {
    fn paint(&mut self, width: f32, height: f32, _tcx: &mut TextContext) -> Scene {
        let key = (self.cx, self.cy);
        if self.cached != Some(key) {
            let rgba = self.render();
            let data = ImageData {
                data: Blob::new(Arc::new(rgba.into_boxed_slice())),
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width: RENDER_SIZE,
                height: RENDER_SIZE,
            };
            self.image = Some(ImageBrush::new(data));
            self.cached = Some(key);
        }
        let mut scene = Scene::new();
        if let Some(brush) = &self.image {
            let sx = width as f64 / RENDER_SIZE as f64;
            let sy = height as f64 / RENDER_SIZE as f64;
            scene.draw_image(brush, Affine::scale_non_uniform(sx, sy));
        }
        scene
    }

    fn attribute_changed(&mut self, name: &str, value: &str) {
        if let Ok(v) = value.trim().parse::<f64>() {
            match name {
                "cx" => self.cx = v,
                "cy" => self.cy = v,
                _ => {}
            }
        }
    }

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::IGNORED
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([384.0, 384.0])
    }
}

fn julia_iter(zx: f64, zy: f64, cx: f64, cy: f64) -> (f64, u32) {
    let (mut x, mut y) = (zx, zy);
    for i in 0..MAX_ITER {
        let xx = x * x;
        let yy = y * y;
        if xx + yy > 4.0 {
            return (xx + yy, i);
        }
        let nx = xx - yy + cx;
        let ny = 2.0 * x * y + cy;
        x = nx;
        y = ny;
    }
    (4.0, MAX_ITER)
}

fn paint(r: f64, n: u32) -> (u8, u8, u8) {
    if r > 4.0 {
        hsl_to_rgb(n as f64 / 200.0, 0.85, 0.55)
    } else {
        (10, 10, 20)
    }
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let (r, g, b);
    if s == 0.0 {
        r = l;
        g = l;
        b = l;
    } else {
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let p = 2.0 * l - q;
        r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        g = hue_to_rgb(p, q, h);
        b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    }
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}
