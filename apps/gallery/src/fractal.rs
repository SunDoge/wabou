//! Gallery-only Julia set rendered as an application-defined GPUI widget.

use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Arc, Mutex},
};

use wabou::{NativeWidgetContext, gpui};

const RENDER_SIZE: u32 = 480;
const MAX_ITER: u32 = 160;
const VIEW: f64 = 1.5;

/// Creates the GPUI-native Julia widget factory used by the default shell.
///
/// The cache is owned by the application registration rather than a transient
/// GPUI element. Attribute changes select a deterministic image while ordinary
/// frame rebuilds reuse the already encoded source.
pub fn gpui_factory()
-> impl for<'a> Fn(NativeWidgetContext<'a>) -> gpui::AnyElement + Send + Sync + 'static {
    use gpui::{IntoElement as _, Styled as _};

    let images = Arc::new(Mutex::new(HashMap::<(u64, u64), Arc<gpui::Image>>::new()));
    move |context| {
        let cx = context
            .attribute("cx")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.7885);
        let cy = context
            .attribute("cy")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);
        let cache_key = (cx.to_bits(), cy.to_bits());
        let image = {
            let mut images = images.lock().expect("Julia image cache lock");
            images
                .entry(cache_key)
                .or_insert_with(|| Arc::new(encode_gpui_image(cx, cy)))
                .clone()
        };

        gpui::img(image).size_full().into_any_element()
    }
}

fn render_rgba(cx: f64, cy: f64) -> Vec<u8> {
    let mut rgba = vec![0u8; (RENDER_SIZE * RENDER_SIZE * 4) as usize];
    let scale = 2.0 * VIEW / RENDER_SIZE as f64;
    for y in 0..RENDER_SIZE {
        let zy = (y as f64 - RENDER_SIZE as f64 / 2.0) * scale;
        for x in 0..RENDER_SIZE {
            let zx = (x as f64 - RENDER_SIZE as f64 / 2.0) * scale;
            let (escaped_radius, iterations) = julia_iter(zx, zy, cx, cy);
            let (r, g, b) = paint(escaped_radius, iterations);
            let offset = ((y * RENDER_SIZE + x) * 4) as usize;
            rgba[offset..offset + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    rgba
}

fn encode_gpui_image(cx: f64, cy: f64) -> gpui::Image {
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(RENDER_SIZE, RENDER_SIZE, render_rgba(cx, cy))
            .expect("Julia renderer emits a complete RGBA image"),
    )
    .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
    .expect("encoding an in-memory Julia image cannot fail");
    gpui::Image::from_bytes(gpui::ImageFormat::Png, png)
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
