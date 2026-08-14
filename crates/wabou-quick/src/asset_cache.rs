//! Shared memory/disk cache for encoded and decoded visual assets.

use std::future::Future;
use std::path::Path;
use std::sync::Arc;

use foyer::{
    BlockEngineConfig, Cache, CacheBuilder, DeviceBuilder, FsDeviceBuilder, HybridCache,
    HybridCacheBuilder, HybridCachePolicy,
};

use wabou_shell::{image::RasterImage, svg::SvgImage};

const DECODED_RASTER_ENTRIES: usize = 128;
const DECODED_SVG_ENTRIES: usize = 256;
const ENCODED_MEMORY_BYTES: usize = 16 * 1024 * 1024;
const ENCODED_DISK_BYTES: usize = 256 * 1024 * 1024;

pub type RasterAsset = Result<Arc<RasterImage>, Arc<str>>;
pub type SvgAsset = Result<Arc<SvgImage>, Arc<str>>;

/// Encoded resources use a hybrid memory/file cache; decoded paint resources
/// stay memory-only because Vello scenes and brushes are not stable disk data.
pub struct AssetCache {
    encoded: Option<HybridCache<String, Vec<u8>>>,
    decoded_rasters: Cache<String, RasterAsset>,
    decoded_svgs: Cache<String, SvgAsset>,
    runtime: Option<Arc<tokio::runtime::Runtime>>,
}

impl std::fmt::Debug for AssetCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetCache")
            .field("disk_enabled", &self.encoded.is_some())
            .field("decoded_rasters", &self.decoded_rasters.usage())
            .field("decoded_svgs", &self.decoded_svgs.usage())
            .finish()
    }
}

impl AssetCache {
    pub fn memory_only() -> Self {
        Self {
            encoded: None,
            decoded_rasters: CacheBuilder::new(DECODED_RASTER_ENTRIES).build(),
            decoded_svgs: CacheBuilder::new(DECODED_SVG_ENTRIES).build(),
            runtime: None,
        }
    }

    pub fn with_disk(cache_dir: &Path) -> Result<Self, String> {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|error| error.to_string())?,
        );
        let assets_dir = cache_dir.join("wabou-assets-v1");
        std::fs::create_dir_all(&assets_dir).map_err(|error| error.to_string())?;
        let device = FsDeviceBuilder::new(&assets_dir)
            .with_capacity(ENCODED_DISK_BYTES)
            .build()
            .map_err(|error| error.to_string())?;
        let spawner = runtime.handle().clone().into();
        let encoded = runtime
            .block_on(
                HybridCacheBuilder::new()
                    .with_name("wabou-assets")
                    .with_policy(HybridCachePolicy::WriteOnInsertion)
                    .with_flush_on_close(false)
                    .memory(ENCODED_MEMORY_BYTES)
                    .with_weighter(|key: &String, value: &Vec<u8>| key.len() + value.len())
                    .storage()
                    .with_engine_config(BlockEngineConfig::new(device))
                    .with_spawner(spawner)
                    .build(),
            )
            .map_err(|error| error.to_string())?;
        Ok(Self {
            encoded: Some(encoded),
            decoded_rasters: CacheBuilder::new(DECODED_RASTER_ENTRIES).build(),
            decoded_svgs: CacheBuilder::new(DECODED_SVG_ENTRIES).build(),
            runtime: Some(runtime),
        })
    }

    pub fn raster(&self, key: &str) -> Option<RasterAsset> {
        self.decoded_rasters
            .get(key)
            .map(|entry| entry.value().clone())
    }

    pub fn insert_raster(&self, key: impl Into<String>, value: RasterAsset) {
        self.decoded_rasters.insert(key.into(), value);
    }

    pub fn svg(&self, key: &str) -> Option<SvgAsset> {
        self.decoded_svgs
            .get(key)
            .map(|entry| entry.value().clone())
    }

    pub fn insert_svg(&self, key: impl Into<String>, value: SvgAsset) {
        self.decoded_svgs.insert(key.into(), value);
    }

    pub async fn encoded(&self, key: &str) -> Option<Vec<u8>> {
        let cache = self.encoded.as_ref()?;
        cache
            .get(key)
            .await
            .ok()
            .flatten()
            .map(|entry| entry.value().clone())
    }

    pub fn insert_encoded(&self, key: impl Into<String>, value: Vec<u8>) {
        if let Some(cache) = &self.encoded {
            cache.insert(key.into(), value);
        }
    }

    pub fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        if let Some(runtime) = &self.runtime {
            runtime.spawn(future);
        } else if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            runtime.spawn(future);
        } else {
            std::thread::spawn(move || {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("build asset loading runtime")
                    .block_on(future);
            });
        }
    }

    #[cfg(test)]
    pub fn decoded_svg_entries(&self) -> usize {
        self.decoded_svgs.usage()
    }
}

impl Default for AssetCache {
    fn default() -> Self {
        Self::memory_only()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_svg_is_shared_by_content_key() {
        let cache = AssetCache::memory_only();
        let source = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1 1"/>"#;
        let image = Arc::new(SvgImage::parse(source).unwrap());
        cache.insert_svg(source, Ok(image.clone()));

        let first = cache.svg(source).unwrap().unwrap();
        let second = cache.svg(source).unwrap().unwrap();
        assert!(Arc::ptr_eq(&first, &image));
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(cache.decoded_svg_entries(), 1);
    }

    #[test]
    fn decoded_failures_are_cached_to_prevent_parse_loops() {
        let cache = AssetCache::memory_only();
        cache.insert_svg("invalid", Err(Arc::from("bad SVG")));
        assert_eq!(
            cache.svg("invalid").unwrap().unwrap_err().as_ref(),
            "bad SVG"
        );
    }

    #[test]
    fn hybrid_cache_round_trips_encoded_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let cache = AssetCache::with_disk(directory.path()).unwrap();
        cache.insert_encoded("https://example.test/icon.png", vec![1, 2, 3, 4]);

        let bytes = cache
            .runtime
            .as_ref()
            .unwrap()
            .block_on(cache.encoded("https://example.test/icon.png"));
        assert_eq!(bytes, Some(vec![1, 2, 3, 4]));
        assert!(directory.path().join("wabou-assets-v1").is_dir());
    }
}
