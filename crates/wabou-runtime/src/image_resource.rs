//! Application-visible raster resources shared by rendering and native work.

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

const MAX_SOURCE_PIXELS: u64 = 100 * 1024 * 1024;
const MAX_NETWORK_RESOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CONCURRENT_NETWORK_LOADS: usize = 8;

/// Full-width generational identity for one decoded image resource.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResourceHandle {
    /// Slot index in the host resource table.
    pub lo: u32,
    /// Slot generation. Stale handles never address a replacement resource.
    pub hi: u32,
}

impl ImageResourceHandle {
    fn from_key(key: DefaultKey) -> Self {
        let ffi = key.data().as_ffi();
        Self {
            lo: ffi as u32,
            hi: (ffi >> 32) as u32,
        }
    }

    fn key(self) -> DefaultKey {
        DefaultKey::from(KeyData::from_ffi(
            u64::from(self.lo) | (u64::from(self.hi) << 32),
        ))
    }
}

/// One immutable source image plus its GPUI image representation.
pub struct ImageResource {
    source: Arc<image::DynamicImage>,
    gpui: Arc<gpui_shell::gpui::Image>,
}

impl ImageResource {
    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|error| error.to_string())?;
        let format = reader
            .format()
            .ok_or_else(|| "image format could not be determined".to_owned())?;
        let (width, height) = reader
            .into_dimensions()
            .map_err(|error| error.to_string())?;
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_SOURCE_PIXELS {
            return Err("image dimensions exceed the source resource limit".into());
        }
        let source = Arc::new(image::load_from_memory(bytes).map_err(|error| error.to_string())?);
        Ok(Self {
            source,
            gpui: Arc::new(gpui_shell::gpui::Image::from_bytes(
                gpui_image_format(format)?,
                bytes.to_vec(),
            )),
        })
    }

    /// Original pixel dimensions, independent of any view fit or zoom.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.source.width(), self.source.height())
    }

    /// Copy the original image into RGB8 for OCR or other native processing.
    pub fn to_rgb8(&self) -> image::RgbImage {
        self.source.to_rgb8()
    }

    /// Lazily decoded and cached by GPUI when an image element first paints.
    pub(crate) fn gpui_image(&self) -> Arc<gpui_shell::gpui::Image> {
        self.gpui.clone()
    }

    pub(crate) fn to_rgba8(&self) -> image::RgbaImage {
        self.source.to_rgba8()
    }
}

fn gpui_image_format(format: image::ImageFormat) -> Result<gpui_shell::gpui::ImageFormat, String> {
    use gpui_shell::gpui::ImageFormat as Gpui;
    Ok(match format {
        image::ImageFormat::Png => Gpui::Png,
        image::ImageFormat::Jpeg => Gpui::Jpeg,
        image::ImageFormat::WebP => Gpui::Webp,
        image::ImageFormat::Gif => Gpui::Gif,
        image::ImageFormat::Bmp => Gpui::Bmp,
        image::ImageFormat::Tiff => Gpui::Tiff,
        image::ImageFormat::Ico => Gpui::Ico,
        image::ImageFormat::Pnm => Gpui::Pnm,
        other => return Err(format!("image format {other:?} is not supported by GPUI")),
    })
}

#[derive(Default)]
struct StoreInner {
    images: SlotMap<DefaultKey, Arc<ImageResource>>,
}

/// Process-wide decoded image registry. Clones address the same resources.
#[derive(Clone)]
pub struct ImageResourceStore {
    inner: Arc<Mutex<StoreInner>>,
    http: reqwest::Client,
    network_slots: Arc<tokio::sync::Semaphore>,
}

impl Default for ImageResourceStore {
    fn default() -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(5))
            .timeout(std::time::Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            inner: Arc::new(Mutex::new(StoreInner::default())),
            http,
            network_slots: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_NETWORK_LOADS)),
        }
    }
}

impl ImageResourceStore {
    /// Decode bytes into a new resource. Creation never deduplicates identity.
    pub fn create(&self, bytes: &[u8]) -> Result<ImageResourceHandle, String> {
        let resource = Arc::new(ImageResource::decode(bytes)?);
        let mut inner = self.inner.lock().map_err(|_| "image store lock poisoned")?;
        let stored = inner.images.insert(resource);
        Ok(ImageResourceHandle::from_key(stored))
    }

    /// Read a file and decode it into a new resource.
    pub fn create_file(&self, path: impl AsRef<Path>) -> Result<ImageResourceHandle, String> {
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        self.create(&bytes)
    }

    /// Fetch once and create a new explicit resource identity.
    ///
    /// Display-only URL images use GPUI's asset and image caches. This path is
    /// for native consumers such as OCR which need stable access to source pixels.
    pub async fn create_network(&self, url: &str) -> Result<ImageResourceHandle, String> {
        let url = url::Url::parse(url).map_err(|error| error.to_string())?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err("network image URL must use HTTP(S)".into());
        }
        let _permit = self
            .network_slots
            .acquire()
            .await
            .map_err(|_| "network image loader is shutting down".to_owned())?;
        let mut response = self
            .http
            .get(url)
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        if response
            .content_length()
            .is_some_and(|size| size > MAX_NETWORK_RESOURCE_BYTES as u64)
        {
            return Err("image response exceeds 32 MiB".into());
        }
        let mut bytes = Vec::with_capacity(
            response
                .content_length()
                .unwrap_or_default()
                .min(MAX_NETWORK_RESOURCE_BYTES as u64) as usize,
        );
        while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
            if bytes.len() + chunk.len() > MAX_NETWORK_RESOURCE_BYTES {
                return Err("image response exceeds 32 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        self.create(&bytes)
    }

    /// Resolve a live handle. A stale generation returns `None`.
    pub fn get(&self, handle: ImageResourceHandle) -> Option<Arc<ImageResource>> {
        self.inner.lock().ok()?.images.get(handle.key()).cloned()
    }

    /// Explicitly release a resource and invalidate every copy of its handle.
    pub fn remove(&self, handle: ImageResourceHandle) -> bool {
        let Ok(mut inner) = self.inner.lock() else {
            return false;
        };
        let key = handle.key();
        if inner.images.remove(key).is_none() {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::ImageEncoder as _;

    #[test]
    fn stale_handles_do_not_resolve_reused_slots() {
        let store = ImageResourceStore::default();
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&[1, 2, 3, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        let resource = Arc::new(ImageResource::decode(&png).unwrap());
        let first = {
            let mut inner = store.inner.lock().unwrap();
            ImageResourceHandle::from_key(inner.images.insert(resource.clone()))
        };
        assert!(store.remove(first));
        let second = {
            let mut inner = store.inner.lock().unwrap();
            ImageResourceHandle::from_key(inner.images.insert(resource))
        };
        assert_eq!(first.lo, second.lo);
        assert_ne!(first.hi, second.hi);
        assert!(store.get(first).is_none());
        assert_eq!(store.get(second).unwrap().dimensions(), (1, 1));
    }

    #[test]
    fn creation_is_explicit_and_does_not_deduplicate_identity() {
        let store = ImageResourceStore::default();
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(&[1, 2, 3, 255], 1, 1, image::ExtendedColorType::Rgba8)
            .unwrap();
        let first = store.create(&png).unwrap();
        let second = store.create(&png).unwrap();
        assert_ne!(first, second);
        assert!(store.get(first).is_some());
        assert!(store.get(second).is_some());
    }

    #[test]
    fn source_dimensions_are_not_replaced_by_drawable_dimensions() {
        let store = ImageResourceStore::default();
        let mut png = Vec::new();
        image::codecs::png::PngEncoder::new(&mut png)
            .write_image(
                &vec![255; 4_100 * 4],
                4_100,
                1,
                image::ExtendedColorType::Rgba8,
            )
            .unwrap();
        let handle = store.create(&png).unwrap();
        let resource = store.get(handle).unwrap();
        assert_eq!(resource.dimensions(), (4_100, 1));
        assert_eq!(resource.to_rgba8().dimensions(), (4_100, 1));
    }
}
