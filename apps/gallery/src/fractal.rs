//! Gallery-only Julia set rendered as an application-defined GPUI widget.

use std::{io::Cursor, sync::Arc, time::Duration};

use wabou::gpui::AppContext as _;
use wabou::{NativeWidgetContext, NativeWidgetMount, gpui};

const RENDER_SIZE: u32 = 480;
const MAX_ITER: u32 = 160;
const VIEW: f64 = 1.5;
const MIN_FRAME_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Copy, Debug, PartialEq)]
struct FractalParameters {
    cx: f64,
    cy: f64,
}

/// Retained GPUI state keeps expensive image generation out of the parent
/// projection's render pass. Rapid Solid updates replace `requested`; after
/// the current frame completes, only the newest request is rendered.
struct FractalWidget {
    image: Arc<gpui::Image>,
    schedule: FractalRenderSchedule,
}

#[derive(Debug)]
struct FractalRenderSchedule {
    requested: FractalParameters,
    rendered: Option<FractalParameters>,
    rendering: bool,
}

impl FractalRenderSchedule {
    fn new(parameters: FractalParameters) -> Self {
        Self {
            requested: parameters,
            rendered: None,
            rendering: false,
        }
    }

    fn request(&mut self, parameters: FractalParameters) -> Option<FractalParameters> {
        self.requested = parameters;
        self.start_latest()
    }

    fn complete(&mut self, parameters: FractalParameters) {
        self.rendered = Some(parameters);
    }

    fn resume(&mut self) -> Option<FractalParameters> {
        self.rendering = false;
        self.start_latest()
    }

    fn start_latest(&mut self) -> Option<FractalParameters> {
        if self.rendering || self.rendered == Some(self.requested) {
            return None;
        }
        self.rendering = true;
        Some(self.requested)
    }
}

impl FractalWidget {
    fn new(parameters: FractalParameters) -> Self {
        Self {
            image: Arc::new(encode_rgba_image(1, 1, vec![10, 10, 20, 255])),
            schedule: FractalRenderSchedule::new(parameters),
        }
    }

    fn synchronize(&mut self, parameters: FractalParameters, cx: &mut gpui::Context<Self>) {
        if let Some(parameters) = self.schedule.request(parameters) {
            self.render_parameters(parameters, cx);
        }
    }

    fn render_parameters(&mut self, parameters: FractalParameters, cx: &mut gpui::Context<Self>) {
        cx.spawn(async move |widget, cx| {
            let image = cx
                .background_spawn(async move {
                    Arc::new(encode_gpui_image(parameters.cx, parameters.cy))
                })
                .await;
            let _ = widget.update(cx, |widget, cx| {
                widget.image = image;
                widget.schedule.complete(parameters);
                cx.notify();
            });
            cx.background_executor().timer(MIN_FRAME_INTERVAL).await;
            let _ = widget.update(cx, |widget, cx| {
                if let Some(parameters) = widget.schedule.resume() {
                    widget.render_parameters(parameters, cx);
                }
            });
        })
        .detach();
    }
}

impl gpui::Render for FractalWidget {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::Styled as _;

        gpui::img(self.image.clone()).size_full()
    }
}

/// Creates the GPUI-native Julia widget factory used by the default shell.
///
/// Image generation is retained by the widget entity. Attribute changes never
/// perform the Julia calculation inside GPUI's render pass, and animation
/// updates are coalesced while the latest image is generated in the background.
pub fn gpui_factory()
-> impl for<'a> Fn(NativeWidgetContext<'a>, &mut gpui::Window, &mut gpui::App) -> NativeWidgetMount
+ Send
+ Sync
+ 'static {
    use gpui::IntoElement as _;

    move |context, _window, app| {
        let cx = context
            .attribute("cx")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.7885);
        let cy = context
            .attribute("cy")
            .and_then(|value| value.parse::<f64>().ok())
            .unwrap_or(0.0);
        let parameters = FractalParameters { cx, cy };
        let retained = context.entity::<FractalWidget>();
        let entity = retained
            .clone()
            .unwrap_or_else(|| app.new(|_| FractalWidget::new(parameters)));
        entity.update(app, |widget, widget_cx| {
            widget.synchronize(parameters, widget_cx);
        });

        NativeWidgetMount::entity(entity.clone(), entity.into_any_element())
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
    encode_rgba_image(RENDER_SIZE, RENDER_SIZE, render_rgba(cx, cy))
}

fn encode_rgba_image(width: u32, height: u32, rgba: Vec<u8>) -> gpui::Image {
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_raw(width, height, rgba)
            .expect("image renderer emits complete RGBA data"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractal_image_is_a_non_empty_png() {
        let image = encode_gpui_image(0.7885, 0.0);
        assert_eq!(image.format, gpui::ImageFormat::Png);
        assert!(image.bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }

    #[test]
    fn rapid_requests_are_coalesced_to_the_latest_parameters() {
        let initial = FractalParameters {
            cx: 0.7885,
            cy: 0.0,
        };
        let intermediate = FractalParameters { cx: 0.5, cy: 0.4 };
        let latest = FractalParameters { cx: -0.2, cy: 0.7 };
        let mut schedule = FractalRenderSchedule::new(initial);

        assert_eq!(schedule.request(initial), Some(initial));
        assert_eq!(schedule.request(intermediate), None);
        assert_eq!(schedule.request(latest), None);
        assert_eq!(schedule.requested, latest);
        schedule.complete(initial);
        assert_eq!(schedule.resume(), Some(latest));
        schedule.complete(latest);
        assert_eq!(schedule.resume(), None);
        assert_eq!(schedule.rendered, Some(latest));
    }
}
