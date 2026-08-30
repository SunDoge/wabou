//! Shared in-memory cache for raw resources and their decoded forms.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use moka::sync::Cache;

use wabou_shell::svg::SvgImage;

const DECODED_SVG_ENTRIES: usize = 256;
const ENCODED_MEMORY_BYTES: usize = 16 * 1024 * 1024;
const MAX_NETWORK_RESOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_CONCURRENT_NETWORK_LOADS: usize = 8;
const NETWORK_IMAGE_MAX_AGE: Duration = Duration::from_secs(7 * 24 * 60 * 60);

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

/// Raw and decoded resources stay in memory for the application lifetime.
///
/// Explicit image handles own decoded application resources. This cache only
/// deduplicates network bytes and SVG parsing within a process; persistent
/// application data belongs to the separate application cache APIs.
pub struct ResourceCache {
    raw: Cache<String, MemoryRawResource>,
    decoded_svgs: Cache<String, SvgAsset>,
    http: reqwest::Client,
    network_slots: Arc<tokio::sync::Semaphore>,
}

impl std::fmt::Debug for ResourceCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceCache")
            .field("raw_entries", &self.raw.entry_count())
            .field("decoded_svgs", &self.decoded_svgs.entry_count())
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
        let raw = Cache::builder()
            .max_capacity(ENCODED_MEMORY_BYTES as u64)
            .weigher(|key: &String, value: &MemoryRawResource| {
                key.len()
                    .saturating_add(value.bytes.len())
                    .min(u32::MAX as usize) as u32
            })
            .build();
        Self {
            raw,
            decoded_svgs: Cache::new(DECODED_SVG_ENTRIES as u64),
            http: Self::http_client(),
            network_slots: Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_NETWORK_LOADS)),
        }
    }

    pub fn svg(&self, key: &str) -> Option<SvgAsset> {
        self.decoded_svgs.get(key)
    }

    pub fn insert_svg(&self, key: impl Into<String>, value: SvgAsset) {
        self.decoded_svgs.insert(key.into(), value);
    }

    pub async fn raw(&self, namespace: ResourceNamespace, identifier: &str) -> Option<Arc<[u8]>> {
        let key = namespace.key(identifier);
        let entry = self.raw.get(&key)?;
        if namespace
            .max_age
            .is_some_and(|max_age| entry.stored_at.elapsed().is_ok_and(|age| age > max_age))
        {
            self.raw.invalidate(&key);
            None
        } else {
            Some(entry.bytes)
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
        self.raw.insert(
            key,
            MemoryRawResource {
                bytes: value,
                stored_at: SystemTime::now(),
            },
        );
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

    #[cfg(test)]
    pub fn decoded_svg_entries(&self) -> usize {
        self.decoded_svgs.run_pending_tasks();
        self.decoded_svgs.entry_count() as usize
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
