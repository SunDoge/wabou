//! Decoded, reusable raster images for retained scene painting.

use std::sync::Arc;

use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};

/// A decoded RGBA image retained by the host and shared by every node using it.
#[derive(Clone, Debug)]
pub struct RasterImage {
    brush: ImageBrush,
    size: [f32; 2],
}

impl RasterImage {
    /// Decode an encoded PNG with dimensions bounded for untrusted sources.
    pub fn decode_png(bytes: &[u8]) -> Result<Self, image::ImageError> {
        let decoder = image::codecs::png::PngDecoder::new(std::io::Cursor::new(bytes))?;
        use image::ImageDecoder as _;
        let (width, height) = decoder.dimensions();
        if width == 0 || height == 0 || width > 1024 || height > 1024 {
            return Err(image::ImageError::Limits(
                image::error::LimitError::from_kind(image::error::LimitErrorKind::DimensionError),
            ));
        }
        let rgba = image::DynamicImage::from_decoder(decoder)?.into_rgba8();
        let data = ImageData {
            data: Blob::new(Arc::new(rgba.into_raw())),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width,
            height,
        };
        Ok(Self {
            brush: ImageBrush::new(data),
            size: [width as f32, height as f32],
        })
    }

    pub fn brush(&self) -> &ImageBrush {
        &self.brush
    }

    pub fn size(&self) -> [f32; 2] {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder as _;

    #[test]
    fn decodes_png_into_a_reusable_vello_brush() {
        let mut encoded = Vec::new();
        image::codecs::png::PngEncoder::new(&mut encoded)
            .write_image(
                &[255, 0, 0, 255, 0, 255, 0, 255],
                2,
                1,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        let image = RasterImage::decode_png(&encoded).unwrap();
        assert_eq!(image.size(), [2.0, 1.0]);
        assert_eq!(image.brush().image.data.data().len(), 8);
    }

    #[test]
    fn rejects_oversized_png_dimensions_before_decoding_pixels() {
        let mut encoded = Vec::new();
        image::codecs::png::PngEncoder::new(&mut encoded)
            .write_image(&vec![0; 1025 * 4], 1025, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        assert!(RasterImage::decode_png(&encoded).is_err());
    }
}
