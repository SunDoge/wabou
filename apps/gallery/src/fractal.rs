//! Gallery-only Julia set rendered as an application-defined GPUI widget.

#[cfg(feature = "gpui")]
use std::{
    io::Cursor,
    sync::{Arc, Mutex},
};

#[cfg(feature = "gpui")]
use wabou::{NativeWidgetContext, gpui};

const RENDER_SIZE: u32 = 480;
const MAX_ITER: u32 = 160;
const VIEW: f64 = 1.5;

/// Creates the GPUI-native Julia widget factory used by the default shell.
///
/// The cache is owned by the application registration rather than a transient
/// GPUI element. Attribute changes select a deterministic image while ordinary
/// frame rebuilds reuse the already encoded source.
#[cfg(feature = "gpui")]
pub fn gpui_factory()
-> impl for<'a> Fn(NativeWidgetContext<'a>, &mut gpui::Window, &mut gpui::App) -> gpui::AnyElement
+ Send
+ Sync
+ 'static {
    use gpui::{IntoElement as _, Styled as _};

    let image = Arc::new(Mutex::new(None::<((u64, u64), Arc<gpui::Image>)>));
    move |context, _window, _cx| {
        let cx = context
            .attribute("cx")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.7885);
        let cy = context
            .attribute("cy")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);
        let cache_key = (cx.to_bits(), cy.to_bits());
        let current = {
            let mut image = image.lock().expect("Julia image cache lock");
            if image.as_ref().is_none_or(|(key, _)| *key != cache_key) {
                *image = Some((cache_key, Arc::new(encode_gpui_image(cx, cy))));
            }
            image
                .as_ref()
                .expect("Julia cache initialized above")
                .1
                .clone()
        };

        gpui::img(current).size_full().into_any_element()
    }
}

/// Application-defined Julia widget for the Winit + Vello Hybrid backend.
#[cfg(not(feature = "gpui"))]
pub struct WinitFractal {
    cx: f64,
    cy: f64,
    image: Option<((u64, u64), wabou::WinitRasterImage)>,
}

#[cfg(not(feature = "gpui"))]
impl Default for WinitFractal {
    fn default() -> Self {
        Self {
            cx: 0.7885,
            cy: 0.0,
            image: None,
        }
    }
}

#[cfg(not(feature = "gpui"))]
impl wabou::WinitWidget for WinitFractal {
    fn paint(&mut self, paint: &mut wabou::WinitPaintContext<'_>) {
        let key = (self.cx.to_bits(), self.cy.to_bits());
        if self.image.as_ref().is_none_or(|(cached, _)| *cached != key) {
            let image = wabou::WinitRasterImage::from_rgba8(
                RENDER_SIZE,
                RENDER_SIZE,
                render_rgba(self.cx, self.cy),
            )
            .expect("Julia renderer emits a complete RGBA image");
            self.image = Some((key, image));
        }
        paint.draw_raster_image(
            &self
                .image
                .as_ref()
                .expect("Julia image initialized above")
                .1,
        );
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> wabou::WinitWidgetChanges {
        let target = match name {
            "cx" => &mut self.cx,
            "cy" => &mut self.cy,
            _ => return wabou::WinitWidgetChanges::empty(),
        };
        let Ok(next) = value.parse::<f64>() else {
            return wabou::WinitWidgetChanges::empty();
        };
        if next.is_finite() && target.to_bits() != next.to_bits() {
            *target = next;
            self.image = None;
            wabou::WinitWidgetChanges::REDRAW
        } else {
            wabou::WinitWidgetChanges::empty()
        }
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([RENDER_SIZE as f32, RENDER_SIZE as f32])
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

#[cfg(feature = "gpui")]
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

#[cfg(all(test, not(feature = "gpui")))]
mod hybrid_tests {
    use super::*;
    use wabou::WinitWidget as _;

    #[test]
    fn hybrid_fractal_reuses_pixels_until_parameters_change() {
        let mut widget = WinitFractal::default();
        assert_eq!(widget.intrinsic_size(), Some([480.0, 480.0]));
        assert!(widget.image.is_none());

        let mut text = wabou::WinitTextContext::new();
        let mut paint = wabou::WinitPaintContext::new(64.0, 64.0, 1.0, &mut text);
        widget.paint(&mut paint);
        assert!(widget.image.is_some());
        let original = widget.image.as_ref().unwrap().0;

        assert!(
            widget
                .attribute_changed("cx", "-0.4")
                .contains(wabou::WinitWidgetChanges::REDRAW)
        );
        assert!(widget.image.is_none());
        let mut paint = wabou::WinitPaintContext::new(64.0, 64.0, 1.0, &mut text);
        widget.paint(&mut paint);
        assert_ne!(widget.image.as_ref().unwrap().0, original);
    }
}
