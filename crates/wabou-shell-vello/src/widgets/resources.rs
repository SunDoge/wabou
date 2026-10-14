//! Bounded resident image pool. Code generations never own its buffers.
use crate::WidgetRasterImage;
use slotmap::{DefaultKey, Key, KeyData, SlotMap};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU32, AtomicUsize, Ordering},
};
use vello_common::peniko::Blob;
use wabou_host_api::ResourceKey;
use wabou_shell_api::widget_implementation::WidgetImageKey as ImageKey;
static NEXT_SCOPE: AtomicU32 = AtomicU32::new(1);

/// Host-owned resource pool shared by widgets and capability host operations.
/// This first resource family is RGBA images; SVG/shader caches remain resident.
pub struct HostResourcePool {
    scope: u32,
    budget: usize,
    used: Arc<AtomicUsize>,
    images: Mutex<SlotMap<DefaultKey, WidgetRasterImage>>,
}
struct ChargedPixels {
    bytes: Box<[u8]>,
    used: Arc<AtomicUsize>,
}
impl AsRef<[u8]> for ChargedPixels {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl Drop for ChargedPixels {
    fn drop(&mut self) {
        self.used.fetch_sub(self.bytes.len(), Ordering::AcqRel);
    }
}
impl Default for HostResourcePool {
    fn default() -> Self {
        Self::new(64 * 1024 * 1024)
    }
}
impl HostResourcePool {
    /// Bound resident pixel storage, including images pinned by already built frames.
    pub fn new(byte_budget: usize) -> Self {
        let mut scope = NEXT_SCOPE.load(Ordering::Relaxed);
        loop {
            let next = scope
                .checked_add(1)
                .expect("resource pool scope space exhausted");
            match NEXT_SCOPE.compare_exchange_weak(
                scope,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => scope = actual,
            }
        }
        Self {
            scope,
            budget: byte_budget,
            used: Arc::new(AtomicUsize::new(0)),
            images: Mutex::new(SlotMap::with_key()),
        }
    }
    /// Pixel bytes still alive, including retired images retained by paint scenes.
    pub fn used_bytes(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }
    /// Allocate an image. The creator must release its key when its owner ends.
    pub fn insert_rgba8(&self, width: u32, height: u32, rgba: Vec<u8>) -> Result<ImageKey, String> {
        let expected = (u64::from(width) * u64::from(height)).checked_mul(4);
        if width == 0 || height == 0 || expected != Some(rgba.len() as u64) {
            return Err("invalid RGBA8 image dimensions".into());
        }
        let mut used = self.used.load(Ordering::Acquire);
        loop {
            let next = used
                .checked_add(rgba.len())
                .filter(|next| *next <= self.budget)
                .ok_or("host resource pool pixel budget exceeded")?;
            match self
                .used
                .compare_exchange_weak(used, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(actual) => used = actual,
            }
        }
        let pixels = ChargedPixels {
            bytes: rgba.into_boxed_slice(),
            used: self.used.clone(),
        };
        let image = WidgetRasterImage::from_blob(width, height, Blob::new(Arc::new(pixels)))
            .map_err(|error| error.to_string())?;
        let key = self
            .images
            .lock()
            .map_err(|_| "host resource pool poisoned")?
            .insert(image);
        Ok(ImageKey::new(
            self.scope,
            ResourceKey::from_ffi(key.data().as_ffi()).expect("valid slotmap key"),
        )
        .expect("valid pool scope"))
    }
    fn slot(&self, image: ImageKey) -> Result<DefaultKey, String> {
        if image.scope() != self.scope {
            return Err("image belongs to a different resource pool".into());
        }
        Ok(DefaultKey::from(KeyData::from_ffi(image.key().as_ffi())))
    }
    /// Borrow retained pixel storage without copying it or exposing a GPU handle.
    pub fn image(&self, image: ImageKey) -> Result<WidgetRasterImage, String> {
        let key = self.slot(image)?;
        self.images
            .lock()
            .map_err(|_| "host resource pool poisoned")?
            .get(key)
            .cloned()
            .ok_or_else(|| "unknown or retired image handle".into())
    }
    /// Retire a key. Pixel storage is charged until its last scene reference drops.
    pub fn release_image(&self, image: ImageKey) -> Result<(), String> {
        let key = self.slot(image)?;
        self.images
            .lock()
            .map_err(|_| "host resource pool poisoned")?
            .remove(key)
            .ok_or("unknown or retired image handle")?;
        Ok(())
    }
    /// SDK host-operation dispatcher. Uploads are width/height LE u32 plus raw bytes.
    pub fn call(&self, operation: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        match operation {
            "widget.image.create" => {
                if request.len() < 8 {
                    return Err("truncated image upload".into());
                }
                let width = u32::from_le_bytes(request[..4].try_into().unwrap());
                let height = u32::from_le_bytes(request[4..8].try_into().unwrap());
                let key = self.insert_rgba8(width, height, request[8..].to_vec())?;
                serde_json::to_vec(&key).map_err(|error| error.to_string())
            }
            "widget.image.release" => {
                let image = serde_json::from_slice(request).map_err(|error| error.to_string())?;
                self.release_image(image)?;
                Ok(b"null".to_vec())
            }
            _ => Err(format!("unknown image resource operation: {operation}")),
        }
    }
}

/// Resident creator ownership. Dropping it retires the pool entry; cached frames
/// keep their buffers alive without retaining a valid lookup handle.
pub struct HostImage {
    pool: Arc<HostResourcePool>,
    key: ImageKey,
}
impl HostImage {
    /// Copy the data-only handle into widget state/config or a drawing command.
    pub fn key(&self) -> ImageKey {
        self.key
    }
}
impl Drop for HostImage {
    fn drop(&mut self) {
        let _ = self.pool.release_image(self.key);
    }
}
impl HostResourcePool {
    /// Allocate with automatic creator cleanup. Keep this owner in resident Rust
    /// application state, independent of the current implementation library.
    pub fn allocate_rgba8(
        self: &Arc<Self>,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<HostImage, String> {
        let key = self.insert_rgba8(width, height, rgba)?;
        Ok(HostImage {
            pool: self.clone(),
            key,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keys_are_scoped_and_retired_generations_cannot_alias_new_images() {
        let pool = HostResourcePool::new(8);
        let other = HostResourcePool::new(8);
        let first = pool.insert_rgba8(1, 1, vec![255; 4]).unwrap();
        let foreign = other.insert_rgba8(1, 1, vec![0; 4]).unwrap();
        assert_eq!(first.key().lo(), foreign.key().lo());
        assert!(pool.image(foreign).is_err());
        assert!(pool.release_image(foreign).is_err());
        pool.release_image(first).unwrap();
        let replacement = pool.insert_rgba8(1, 1, vec![0; 4]).unwrap();
        assert_eq!(first.key().lo(), replacement.key().lo());
        assert_ne!(first.key().hi(), replacement.key().hi());
        assert!(pool.image(first).is_err());
        assert!(pool.release_image(first).is_err());
        assert_eq!(pool.used_bytes(), 4);
    }
    #[test]
    fn scene_references_keep_retired_pixels_charged_until_the_last_reference_drops() {
        let pool = Arc::new(HostResourcePool::new(4));
        let owner = pool.allocate_rgba8(1, 1, vec![255; 4]).unwrap();
        let key = owner.key();
        let image = pool.image(key).unwrap();
        let mut text = crate::TextContext::new();
        let mut cx = crate::PaintContext::new(10.0, 10.0, 1.0, &mut text);
        cx.draw_raster_image(&image);
        let scene = cx.finish();
        drop(image);
        drop(owner);
        assert!(pool.image(key).is_err());
        assert_eq!(pool.used_bytes(), 4);
        assert!(pool.insert_rgba8(1, 1, vec![0; 4]).is_err());
        drop(scene);
        assert_eq!(pool.used_bytes(), 0);
        assert!(pool.insert_rgba8(1, 1, vec![0; 4]).is_ok());
    }
    #[test]
    fn invalid_uploads_do_not_consume_budget_and_binary_upload_avoids_json_expansion() {
        let pool = HostResourcePool::new(256 * 256 * 4);
        assert!(pool.insert_rgba8(u32::MAX, u32::MAX, Vec::new()).is_err());
        assert!(pool.call("widget.image.create", b"short").is_err());
        assert_eq!(pool.used_bytes(), 0);
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&256u32.to_le_bytes());
        bytes.extend_from_slice(&256u32.to_le_bytes());
        bytes.resize(8 + 256 * 256 * 4, 255);
        let reply = pool.call("widget.image.create", &bytes).unwrap();
        assert!(reply.len() < 128);
        let key: ImageKey = serde_json::from_slice(&reply).unwrap();
        assert_eq!(pool.image(key).unwrap().dimensions(), [256, 256]);
        assert_eq!(pool.used_bytes(), 256 * 256 * 4);
        pool.call("widget.image.release", &reply).unwrap();
        assert_eq!(pool.used_bytes(), 0);
    }
}
