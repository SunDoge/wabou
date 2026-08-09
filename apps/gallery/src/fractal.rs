//! Gallery-only Julia set widget used to demonstrate the external widget API.

use std::sync::Arc;

use wabou_quick::widget_api::{
    TextContext, UiEvent, Widget, WidgetEventResult,
    vello::{
        Scene,
        kurbo::Affine,
        peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat},
    },
};

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
        let mut rgba = vec![0u8; (RENDER_SIZE * RENDER_SIZE * 4) as usize];
        let scale = 2.0 * VIEW / RENDER_SIZE as f64;
        for y in 0..RENDER_SIZE {
            let zy = (y as f64 - RENDER_SIZE as f64 / 2.0) * scale;
            for x in 0..RENDER_SIZE {
                let zx = (x as f64 - RENDER_SIZE as f64 / 2.0) * scale;
                let (escaped_radius, iterations) = julia_iter(zx, zy, self.cx, self.cy);
                let (r, g, b) = paint(escaped_radius, iterations);
                let offset = ((y * RENDER_SIZE + x) * 4) as usize;
                rgba[offset..offset + 4].copy_from_slice(&[r, g, b, 255]);
            }
        }
        rgba
    }
}

impl Widget for JuliaWidget {
    fn paint(&mut self, width: f32, height: f32, _text: &mut TextContext) -> Scene {
        let key = (self.cx, self.cy);
        if self.cached != Some(key) {
            let data = ImageData {
                data: Blob::new(Arc::new(self.render().into_boxed_slice())),
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
            scene.draw_image(
                brush,
                Affine::scale_non_uniform(
                    width as f64 / RENDER_SIZE as f64,
                    height as f64 / RENDER_SIZE as f64,
                ),
            );
        }
        scene
    }

    fn attribute_changed(&mut self, name: &str, value: &str) {
        if let Ok(value) = value.trim().parse::<f64>() {
            match name {
                "cx" => self.cx = value,
                "cy" => self.cy = value,
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
    for iteration in 0..MAX_ITER {
        let xx = x * x;
        let yy = y * y;
        if xx + yy > 4.0 {
            return (xx + yy, iteration);
        }
        (x, y) = (xx - yy + cx, 2.0 * x * y + cy);
    }
    (4.0, MAX_ITER)
}

fn paint(radius: f64, iterations: u32) -> (u8, u8, u8) {
    if radius > 4.0 {
        hsl_to_rgb(iterations as f64 / 200.0, 0.85, 0.55)
    } else {
        (10, 10, 20)
    }
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    if s == 0.0 {
        let value = (l * 255.0) as u8;
        return (value, value, value);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    (
        (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0) as u8,
        (hue_to_rgb(p, q, h) * 255.0) as u8,
        (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0) as u8,
    )
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
