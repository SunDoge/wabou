//! Decoded, reusable raster images for retained scene painting.

#![warn(missing_docs)]

use std::sync::Arc;

use vello::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};

const MAX_DECODED_PIXELS: u64 = 4 * 1024 * 1024;
const MAX_RETAINED_DIMENSION: u32 = 1024;

/// A decoded RGBA image retained by the host and shared by every node using it.
#[derive(Clone, Debug)]
pub struct RasterImage {
    brush: ImageBrush,
    size: [f32; 2],
    byte_len: usize,
}

impl RasterImage {
    /// Decode a supported raster format by content, with dimensions bounded
    /// before allocating decoded pixels for untrusted sources.
    pub fn decode(bytes: &[u8]) -> Result<Self, image::ImageError> {
        let image = match image::guess_format(bytes)? {
            image::ImageFormat::Png => {
                let decoder = image::codecs::png::PngDecoder::new(std::io::Cursor::new(bytes))?;
                Self::decode_with(decoder)?
            }
            image::ImageFormat::Ico => {
                let decoder = image::codecs::ico::IcoDecoder::new(std::io::Cursor::new(bytes))?;
                Self::decode_with(decoder)?
            }
            image::ImageFormat::Jpeg => {
                let decoder = image::codecs::jpeg::JpegDecoder::new(std::io::Cursor::new(bytes))?;
                Self::decode_with(decoder)?
            }
            image::ImageFormat::WebP => {
                let decoder = image::codecs::webp::WebPDecoder::new(std::io::Cursor::new(bytes))?;
                Self::decode_with(decoder)?
            }
            format => {
                return Err(image::ImageError::Unsupported(
                    image::error::UnsupportedError::from_format_and_kind(
                        image::error::ImageFormatHint::Exact(format),
                        image::error::UnsupportedErrorKind::Format(
                            image::error::ImageFormatHint::Exact(format),
                        ),
                    ),
                ));
            }
        };
        Ok(Self::from_rgba(image))
    }

    fn decode_with(
        decoder: impl image::ImageDecoder,
    ) -> Result<image::RgbaImage, image::ImageError> {
        let (width, height) = decoder.dimensions();
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_DECODED_PIXELS {
            return Err(image::ImageError::Limits(
                image::error::LimitError::from_kind(image::error::LimitErrorKind::DimensionError),
            ));
        }
        let image = image::DynamicImage::from_decoder(decoder)?;
        if width > MAX_RETAINED_DIMENSION || height > MAX_RETAINED_DIMENSION {
            Ok(image
                .resize(
                    MAX_RETAINED_DIMENSION,
                    MAX_RETAINED_DIMENSION,
                    image::imageops::FilterType::Triangle,
                )
                .into_rgba8())
        } else {
            Ok(image.into_rgba8())
        }
    }

    /// Build a renderer image from already decoded RGBA pixels.
    pub fn from_rgba(rgba: image::RgbaImage) -> Self {
        let (width, height) = rgba.dimensions();
        let byte_len = rgba.len();
        let data = ImageData {
            data: Blob::new(Arc::new(rgba.into_raw())),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width,
            height,
        };
        Self {
            brush: ImageBrush::new(data),
            size: [width as f32, height as f32],
            byte_len,
        }
    }

    /// Borrow the retained Vello image brush.
    pub fn brush(&self) -> &ImageBrush {
        &self.brush
    }

    /// Decoded `[width, height]` after safety downscaling.
    pub fn size(&self) -> [f32; 2] {
        self.size
    }

    /// Number of decoded RGBA bytes retained by this image.
    pub fn byte_len(&self) -> usize {
        self.byte_len
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
        let image = RasterImage::decode(&encoded).unwrap();
        assert_eq!(image.size(), [2.0, 1.0]);
        assert_eq!(image.brush().image.data.data().len(), 8);
    }

    #[test]
    fn downsizes_large_images_before_retaining_them() {
        let mut encoded = Vec::new();
        image::codecs::png::PngEncoder::new(&mut encoded)
            .write_image(
                &vec![0; 1140 * 1140 * 4],
                1140,
                1140,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        assert_eq!(
            RasterImage::decode(&encoded).unwrap().size(),
            [1024.0, 1024.0]
        );
    }

    #[test]
    fn decodes_ico_by_content_instead_of_url_extension() {
        let mut encoded = Vec::new();
        image::codecs::ico::IcoEncoder::new(&mut encoded)
            .write_image(&[255, 0, 0, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        let image = RasterImage::decode(&encoded).unwrap();
        assert_eq!(image.size(), [1.0, 1.0]);
    }

    #[test]
    fn decodes_jpeg_by_content() {
        let mut encoded = Vec::new();
        image::codecs::jpeg::JpegEncoder::new(&mut encoded)
            .write_image(&[255, 0, 0], 1, 1, image::ExtendedColorType::Rgb8)
            .unwrap();
        assert_eq!(RasterImage::decode(&encoded).unwrap().size(), [1.0, 1.0]);
    }

    #[test]
    fn decodes_webp_by_content() {
        let mut encoded = Vec::new();
        image::codecs::webp::WebPEncoder::new_lossless(&mut encoded)
            .write_image(&[255, 0, 0, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        assert_eq!(RasterImage::decode(&encoded).unwrap().size(), [1.0, 1.0]);
    }
}
