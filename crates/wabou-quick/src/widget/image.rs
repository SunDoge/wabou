//! ImageWidget — displays a raster image (PNG/JPEG/GIF/WebP/BMP) from a file
//! path. JS passes the path via `<img src="photo.png" />`; Rust decodes the
//! image (via the `image` crate, like blitz's `ImageHandler::parse`) into an
//! RGBA buffer, wraps it in a vello `ImageBrush`, and paints it with
//! `draw_image`. The image is decoded once (cached by `src`) and scaled to
//! fill the content box.

use std::sync::Arc;

use vello::Scene;
use vello::kurbo::Affine;
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use wabou_shell::UiEvent;
use wabou_shell::text::TextContext;

use super::{Widget, WidgetEventResult};

pub struct ImageWidget {
    src: Option<String>,
    image: Option<ImageBrush>,
    img_w: u32,
    img_h: u32,
    cached_src: Option<String>,
}

impl Default for ImageWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageWidget {
    pub fn new() -> Self {
        Self {
            src: None,
            image: None,
            img_w: 0,
            img_h: 0,
            cached_src: None,
        }
    }
}

impl Widget for ImageWidget {
    fn paint(&mut self, width: f32, height: f32, _tcx: &mut TextContext) -> Scene {
        if let Some(src) = &self.src
            && self.cached_src.as_deref() != Some(src.as_str())
        {
            match image::open(src) {
                Ok(img) => {
                    let rgba = img.into_rgba8();
                    self.img_w = rgba.width();
                    self.img_h = rgba.height();
                    let data = ImageData {
                        data: Blob::new(Arc::new(rgba.into_raw().into_boxed_slice())),
                        format: ImageFormat::Rgba8,
                        alpha_type: ImageAlphaType::Alpha,
                        width: self.img_w,
                        height: self.img_h,
                    };
                    self.image = Some(ImageBrush::new(data));
                    self.cached_src = Some(src.clone());
                }
                Err(e) => {
                    tracing::warn!(src = %src, error = %e, "image decode failed");
                    self.image = None;
                    self.cached_src = None;
                }
            }
        }
        let mut scene = Scene::new();
        if let Some(brush) = &self.image
            && self.img_w > 0
            && self.img_h > 0
        {
            let sx = width as f64 / self.img_w as f64;
            let sy = height as f64 / self.img_h as f64;
            scene.draw_image(brush, Affine::scale_non_uniform(sx, sy));
        }
        scene
    }

    fn attribute_changed(&mut self, name: &str, value: &str) {
        if name == "src" {
            self.src = Some(value.to_string());
        }
    }

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::IGNORED
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        if self.img_w > 0 && self.img_h > 0 {
            Some([self.img_w as f32, self.img_h as f32])
        } else {
            None
        }
    }
}
