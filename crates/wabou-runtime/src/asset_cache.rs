//! Shared memory/disk cache for raw resources and their decoded forms.

use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use foyer::{
    BlockEngineConfig, Cache, CacheBuilder, DeviceBuilder, FsDeviceBuilder, HybridCache,
    HybridCacheBuilder, HybridCachePolicy, PsyncIoEngineConfig,
};

use wabou_shell::svg::SvgImage;

const DECODED_SVG_ENTRIES: usize = 256;
const RAW_MEMORY_ENTRIES: usize = 256;
const ENCODED_MEMORY_BYTES: usize = 16 * 1024 * 1024;
const ENCODED_DISK_BYTES: usize = 256 * 1024 * 1024;
const MAX_NETWORK_RESOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CONCURRENT_NETWORK_LOADS: usize = 8;
const NETWORK_IMAGE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);
const RAW_ENVELOPE_MAGIC: &[u8; 4] = b"WRC1";

pub type SvgAsset = Result<Arc<SvgImage>, Arc<str>>;

/// A stable namespace for one class of cached resources.
///
/// Namespaces are part of the persisted key, so unrelated loaders can safely
/// cache the same identifier without sharing bytes or invalidation policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceNamespace {
    name: &'static str,
    max_age: Option<Duration>,
}

impl ResourceNamespace {
    pub const NETWORK_IMAGE: Self = Self {
        name: "network-image-v2",
        max_age: Some(NETWORK_IMAGE_MAX_AGE),
    };

    #[cfg(test)]
    const fn new(name: &'static str) -> Self {
        Self {
            name,
            max_age: None,
        }
    }

    #[cfg(test)]
    const fn expiring(name: &'static str, max_age: Duration) -> Self {
        Self {
            name,
            max_age: Some(max_age),
        }
    }

    fn key(self, identifier: &str) -> String {
        format!("{}\0{identifier}", self.name)
    }
}

#[derive(Clone)]
struct MemoryRawResource {
    bytes: Arc<[u8]>,
    stored_at: SystemTime,
}

enum RawResourceCache {
    Memory(Cache<String, MemoryRawResource>),
    Hybrid(HybridCache<String, Vec<u8>>),
}

/// Raw resources use a memory or hybrid memory/file cache. Decoded resources
/// stay memory-only because renderer objects are not stable disk data.
pub struct ResourceCache {
    raw: RawResourceCache,
    decoded_svgs: Cache<String, SvgAsset>,
    runtime: Option<Arc<tokio::runtime::Runtime>>,
    http: reqwest::Client,
    network_slots: Arc<tokio::sync::Semaphore>,
}

impl std::fmt::Debug for ResourceCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceCache")
            .field(
                "disk_enabled",
                &matches!(self.raw, RawResourceCache::Hybrid(_)),
            )
            .field("decoded_svgs", &self.decoded_svgs.usage())
            .finish()
    }
}

impl ResourceCache {
    fn http_client() -> reqwest::Client {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    }

    pub fn memory_only() -> Self {
        Self {
            raw: RawResourceCache::Memory(CacheBuilder::new(RAW_MEMORY_ENTRIES).build()),
            decoded_svgs: CacheBuilder::new(DECODED_SVG_ENTRIES).build(),
            runtime: None,
            http: Self::http_client(),
            network_slots: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_NETWORK_LOADS)),
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
        let resources_dir = cache_dir.join("wabou-resources-v2");
        std::fs::create_dir_all(&resources_dir).map_err(|error| error.to_string())?;
        let device = FsDeviceBuilder::new(&resources_dir)
            .with_capacity(ENCODED_DISK_BYTES)
            .build()
            .map_err(|error| error.to_string())?;
        let spawner = runtime.handle().clone().into();
        let raw = runtime
            .block_on(
                HybridCacheBuilder::new()
                    .with_name("wabou-resources")
                    .with_policy(HybridCachePolicy::WriteOnInsertion)
                    .with_flush_on_close(false)
                    .memory(ENCODED_MEMORY_BYTES)
                    .with_weighter(|key: &String, value: &Vec<u8>| key.len() + value.len())
                    .storage()
                    .with_io_engine_config(PsyncIoEngineConfig::new())
                    .with_engine_config(BlockEngineConfig::new(device))
                    .with_spawner(spawner)
                    .build(),
            )
            .map_err(|error| error.to_string())?;
        Ok(Self {
            raw: RawResourceCache::Hybrid(raw),
            decoded_svgs: CacheBuilder::new(DECODED_SVG_ENTRIES).build(),
            runtime: Some(runtime),
            http: Self::http_client(),
            network_slots: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_NETWORK_LOADS)),
        })
    }

    pub fn svg(&self, key: &str) -> Option<SvgAsset> {
        self.decoded_svgs
            .get(key)
            .map(|entry| entry.value().clone())
    }

    pub fn insert_svg(&self, key: impl Into<String>, value: SvgAsset) {
        self.decoded_svgs.insert(key.into(), value);
    }

    pub async fn raw(&self, namespace: ResourceNamespace, identifier: &str) -> Option<Arc<[u8]>> {
        let key = namespace.key(identifier);
        match &self.raw {
            RawResourceCache::Memory(cache) => {
                let entry = cache.get(&key)?;
                if namespace.max_age.is_some_and(|max_age| {
                    entry
                        .value()
                        .stored_at
                        .elapsed()
                        .is_ok_and(|age| age > max_age)
                }) {
                    drop(entry);
                    cache.remove(&key);
                    None
                } else {
                    Some(entry.value().bytes.clone())
                }
            }
            RawResourceCache::Hybrid(cache) => {
                let entry = cache.get(&key).await.ok().flatten()?;
                let value = entry.value();
                let (stored_at, bytes) = Self::decode_raw_envelope(value)?;
                if namespace
                    .max_age
                    .is_some_and(|max_age| stored_at.elapsed().is_ok_and(|age| age > max_age))
                {
                    drop(entry);
                    cache.remove(&key);
                    None
                } else {
                    Some(Arc::from(bytes))
                }
            }
        }
    }

    pub fn insert_raw(
        &self,
        namespace: ResourceNamespace,
        identifier: &str,
        value: impl Into<Arc<[u8]>>,
    ) {
        let key = namespace.key(identifier);
        let value = value.into();
        match &self.raw {
            RawResourceCache::Memory(cache) => {
                cache.insert(
                    key,
                    MemoryRawResource {
                        bytes: value,
                        stored_at: SystemTime::now(),
                    },
                );
            }
            RawResourceCache::Hybrid(cache) => {
                cache.insert(key, Self::encode_raw_envelope(&value));
            }
        }
    }

    fn encode_raw_envelope(value: &[u8]) -> Vec<u8> {
        let stored_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut encoded = Vec::with_capacity(RAW_ENVELOPE_MAGIC.len() + 8 + value.len());
        encoded.extend_from_slice(RAW_ENVELOPE_MAGIC);
        encoded.extend_from_slice(&stored_at.to_le_bytes());
        encoded.extend_from_slice(value);
        encoded
    }

    fn decode_raw_envelope(value: &[u8]) -> Option<(SystemTime, &[u8])> {
        let timestamp = value
            .strip_prefix(RAW_ENVELOPE_MAGIC)?
            .get(..8)?
            .try_into()
            .ok()
            .map(u64::from_le_bytes)?;
        Some((UNIX_EPOCH + Duration::from_secs(timestamp), &value[12..]))
    }

    /// Fetch a bounded remote image using the cache-wide connection pool and
    /// concurrency limit. Unknown-length responses are read incrementally so
    /// a server cannot bypass the encoded-byte budget.
    pub async fn network_image_bytes(&self, url: url::Url) -> Result<Arc<[u8]>, String> {
        if let Some(bytes) = self
            .raw(ResourceNamespace::NETWORK_IMAGE, url.as_str())
            .await
        {
            return Ok(bytes);
        }
        let _permit = self
            .network_slots
            .acquire()
            .await
            .map_err(|_| "network resource loader is shutting down".to_string())?;
        // Another window may have populated the shared cache while this load
        // waited for a slot.
        if let Some(bytes) = self
            .raw(ResourceNamespace::NETWORK_IMAGE, url.as_str())
            .await
        {
            return Ok(bytes);
        }
        let mut response = self
            .http
            .get(url.clone())
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        if response
            .content_length()
            .is_some_and(|size| size > MAX_NETWORK_RESOURCE_BYTES as u64)
        {
            return Err("image response exceeds 32 MiB".to_string());
        }
        let mut encoded = Vec::with_capacity(
            response
                .content_length()
                .unwrap_or_default()
                .min(MAX_NETWORK_RESOURCE_BYTES as u64) as usize,
        );
        while let Some(chunk) = response.chunk().await.map_err(|error| error.to_string())? {
            if encoded.len() + chunk.len() > MAX_NETWORK_RESOURCE_BYTES {
                return Err("image response exceeds 32 MiB".to_string());
            }
            encoded.extend_from_slice(&chunk);
        }
        let bytes: Arc<[u8]> = Arc::from(encoded);
        self.insert_raw(
            ResourceNamespace::NETWORK_IMAGE,
            url.as_str(),
            bytes.clone(),
        );
        Ok(bytes)
    }

    /// Close persistent storage while its dedicated runtime is still alive.
    ///
    /// Foyer also initiates a close from `Drop`, but that close is asynchronous.
    /// Waiting here prevents the runtime from cancelling storage flusher and
    /// reclaim tasks during application teardown.
    pub(crate) fn shutdown(&self) -> Result<(), String> {
        let (RawResourceCache::Hybrid(cache), Some(runtime)) = (&self.raw, &self.runtime) else {
            return Ok(());
        };
        runtime
            .block_on(cache.close())
            .map_err(|error| error.to_string())
    }

    #[cfg(test)]
    pub fn decoded_svg_entries(&self) -> usize {
        self.decoded_svgs.usage()
    }
}

impl Default for ResourceCache {
    fn default() -> Self {
        Self::memory_only()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoded_svg_is_shared_by_content_key() {
        let cache = ResourceCache::memory_only();
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
        let cache = ResourceCache::memory_only();
        cache.insert_svg("invalid", Err(Arc::from("bad SVG")));
        assert_eq!(
            cache.svg("invalid").unwrap().unwrap_err().as_ref(),
            "bad SVG"
        );
    }

    #[test]
    fn hybrid_cache_round_trips_encoded_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let cache = ResourceCache::with_disk(directory.path()).unwrap();
        cache.insert_raw(
            ResourceNamespace::NETWORK_IMAGE,
            "https://example.test/icon.png",
            vec![1, 2, 3, 4],
        );

        let bytes = cache.runtime.as_ref().unwrap().block_on(cache.raw(
            ResourceNamespace::NETWORK_IMAGE,
            "https://example.test/icon.png",
        ));
        assert_eq!(bytes.as_deref(), Some([1, 2, 3, 4].as_slice()));
        assert!(directory.path().join("wabou-resources-v2").is_dir());
        cache.shutdown().unwrap();
        // Closing is idempotent, which lets teardown guards cover every exit
        // path without coordinating with callers that already shut down.
        cache.shutdown().unwrap();
    }

    #[test]
    fn memory_only_cache_retains_raw_resources() {
        let cache = ResourceCache::memory_only();
        cache.insert_raw(
            ResourceNamespace::new("font-v1"),
            "inter",
            b"font".as_slice(),
        );
        let bytes =
            futures_lite::future::block_on(cache.raw(ResourceNamespace::new("font-v1"), "inter"));
        assert_eq!(bytes.as_deref(), Some(b"font".as_slice()));
    }

    #[test]
    fn raw_resource_namespaces_do_not_collide() {
        let cache = ResourceCache::memory_only();
        cache.insert_raw(
            ResourceNamespace::new("font-v1"),
            "shared",
            b"font".as_slice(),
        );
        cache.insert_raw(
            ResourceNamespace::new("data-v1"),
            "shared",
            b"data".as_slice(),
        );

        let font =
            futures_lite::future::block_on(cache.raw(ResourceNamespace::new("font-v1"), "shared"));
        let data =
            futures_lite::future::block_on(cache.raw(ResourceNamespace::new("data-v1"), "shared"));
        assert_eq!(font.as_deref(), Some(b"font".as_slice()));
        assert_eq!(data.as_deref(), Some(b"data".as_slice()));
    }

    #[test]
    fn expired_raw_resources_are_not_returned() {
        let cache = ResourceCache::memory_only();
        let namespace = ResourceNamespace::expiring("short-lived", Duration::ZERO);
        cache.insert_raw(namespace, "resource", b"stale".as_slice());
        let value = futures_lite::future::block_on(cache.raw(namespace, "resource"));
        assert!(value.is_none());
    }
}
