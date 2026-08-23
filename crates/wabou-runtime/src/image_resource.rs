//! Application-visible raster resources shared by rendering and native work.

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};
use slotmap::{DefaultKey, Key, KeyData, SlotMap};

const MAX_SOURCE_PIXELS: u64 = 100 * 1024 * 1024;
const MAX_DRAWABLE_DIMENSION: u32 = 4096;

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

/// One immutable source image plus a bounded renderer-ready derivative.
pub struct ImageResource {
    source: Arc<image::DynamicImage>,
    drawable: Arc<wabou_shell::image::RasterImage>,
}

impl ImageResource {
    fn decode(bytes: &[u8]) -> Result<Self, String> {
        let reader = image::ImageReader::new(std::io::Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|error| error.to_string())?;
        let (width, height) = reader
            .into_dimensions()
            .map_err(|error| error.to_string())?;
        if width == 0 || height == 0 || u64::from(width) * u64::from(height) > MAX_SOURCE_PIXELS {
            return Err("image dimensions exceed the source resource limit".into());
        }
        let source = Arc::new(image::load_from_memory(bytes).map_err(|error| error.to_string())?);
        let drawable_rgba = if width > MAX_DRAWABLE_DIMENSION || height > MAX_DRAWABLE_DIMENSION {
            source
                .resize(
                    MAX_DRAWABLE_DIMENSION,
                    MAX_DRAWABLE_DIMENSION,
                    image::imageops::FilterType::Triangle,
                )
                .into_rgba8()
        } else {
            source.to_rgba8()
        };
        Ok(Self {
            source,
            drawable: Arc::new(wabou_shell::image::RasterImage::from_rgba(drawable_rgba)),
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

    pub(crate) fn drawable(&self) -> Arc<wabou_shell::image::RasterImage> {
        self.drawable.clone()
    }
}

#[derive(Default)]
struct StoreInner {
    images: SlotMap<DefaultKey, Arc<ImageResource>>,
    cache: Option<Arc<crate::asset_cache::ResourceCache>>,
}

/// Process-wide decoded image registry. Clones address the same resources.
#[derive(Clone, Default)]
pub struct ImageResourceStore(Arc<Mutex<StoreInner>>);

impl ImageResourceStore {
    pub(crate) fn set_cache(&self, cache: Arc<crate::asset_cache::ResourceCache>) {
        if let Ok(mut inner) = self.0.lock() {
            inner.cache = Some(cache);
        }
    }

    /// Decode bytes into a new resource. Creation never deduplicates identity.
    pub fn create(&self, bytes: &[u8]) -> Result<ImageResourceHandle, String> {
        let resource = Arc::new(ImageResource::decode(bytes)?);
        let mut inner = self.0.lock().map_err(|_| "image store lock poisoned")?;
        let stored = inner.images.insert(resource);
        Ok(ImageResourceHandle::from_key(stored))
    }

    /// Read a file and decode it into a new resource.
    pub fn create_file(&self, path: impl AsRef<Path>) -> Result<ImageResourceHandle, String> {
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        self.create(&bytes)
    }

    /// Fetch through Wabou's bounded raw-resource cache and create a new identity.
    pub async fn create_network(&self, url: &str) -> Result<ImageResourceHandle, String> {
        let url = url::Url::parse(url).map_err(|error| error.to_string())?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err("network image URL must use HTTP(S)".into());
        }
        let cache = self
            .0
            .lock()
            .map_err(|_| "image store lock poisoned")?
            .cache
            .clone()
            .ok_or_else(|| "image resource cache is not initialized".to_owned())?;
        let bytes = cache.network_image_bytes(url).await?;
        self.create(&bytes)
    }

    /// Resolve a live handle. A stale generation returns `None`.
    pub fn get(&self, handle: ImageResourceHandle) -> Option<Arc<ImageResource>> {
        self.0.lock().ok()?.images.get(handle.key()).cloned()
    }

    /// Explicitly release a resource and invalidate every copy of its handle.
    pub fn remove(&self, handle: ImageResourceHandle) -> bool {
        let Ok(mut inner) = self.0.lock() else {
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
            let mut inner = store.0.lock().unwrap();
            ImageResourceHandle::from_key(inner.images.insert(resource.clone()))
        };
        assert!(store.remove(first));
        let second = {
            let mut inner = store.0.lock().unwrap();
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
        assert_eq!(resource.drawable().size(), [4_096.0, 1.0]);
    }
}
