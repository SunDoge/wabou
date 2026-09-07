//! ImageWidget — displays a raster image (PNG/JPEG/GIF/WebP/BMP) from a file
//! path. The public `Image` component passes the path to the internal `img`
//! primitive; Rust decodes the
//! image (via the `image` crate, like blitz's `ImageHandler::parse`) into an
//! RGBA buffer, wraps it in a vello `ImageBrush`, and paints it with
//! `draw_image`. The image is decoded once (cached by `src`) and scaled to
//! fill the content box.

use std::sync::Arc;

use anyrender::PaintScene;
use vello::kurbo::Affine;
use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use wabou_shell::UiEvent;
use wabou_shell::{PaintContext, Widget, WidgetEventResult};

/// Legacy local-file raster image widget used for the intrinsic `img` tag.
///
/// Network/resource-cache loading is owned by the QuickJS frame source; this
/// widget only decodes a path explicitly delivered through `src`.
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
    /// Construct an image widget without a source.
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
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
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
        if let Some(brush) = &self.image
            && self.img_w > 0
            && self.img_h > 0
        {
            let sx = cx.width() as f64 / self.img_w as f64;
            let sy = cx.height() as f64 / self.img_h as f64;
            cx.scene_mut()
                .draw_image(brush.into(), Affine::scale_non_uniform(sx, sy));
        }
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> wabou_shell::WidgetChanges {
        if name == "src" {
            self.src = Some(value.to_string());
            wabou_shell::WidgetChanges::MEASURE | wabou_shell::WidgetChanges::REDRAW
        } else {
            wabou_shell::WidgetChanges::empty()
        }
    }

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::IGNORED
    }

    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        wabou_shell::WidgetAccessibility {
            role: Some(wabou_shell::SemanticRole::Image),
            ..Default::default()
        }
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        if self.img_w > 0 && self.img_h > 0 {
            Some([self.img_w as f32, self.img_h as f32])
        } else {
            None
        }
    }
}
