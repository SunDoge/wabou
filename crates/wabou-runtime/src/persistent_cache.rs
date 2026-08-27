//! Small content-addressed cache for application-level JSON results.

use std::{io::Write, path::PathBuf};

use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};

/// Bounded on-disk cache for reproducible application results.
///
/// This cache is intended for derived values such as OCR, metadata, and model
/// responses. It is deliberately separate from Wabou's renderer asset cache.
/// Values are written atomically and malformed entries become ordinary misses.
#[derive(Clone, Debug)]
pub struct PersistentJsonCache {
    directory: PathBuf,
    capacity: usize,
}

impl PersistentJsonCache {
    /// Open a cache directory and prune its oldest entries to `capacity`.
    pub fn new(directory: impl Into<PathBuf>, capacity: usize) -> Result<Self, String> {
        let directory = directory.into();
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        let cache = Self {
            directory,
            capacity,
        };
        cache.prune();
        Ok(cache)
    }

    /// Produce a stable SHA-256 key while preserving byte-slice boundaries.
    pub fn content_key(parts: &[&[u8]]) -> String {
        let mut digest = Sha256::new();
        for part in parts {
            digest.update((part.len() as u64).to_le_bytes());
            digest.update(part);
        }
        format!("{:x}", digest.finalize())
    }

    /// Decode an entry, returning `None` for a miss or malformed JSON.
    pub fn get<T: DeserializeOwned>(&self, namespace: &str, key: &str) -> Option<T> {
        let path = self.path(namespace, key)?;
        let bytes = std::fs::read(&path).ok()?;
        match serde_json::from_slice(&bytes) {
            Ok(value) => Some(value),
            Err(_) => {
                let _ = std::fs::remove_file(path);
                None
            }
        }
    }

    /// Atomically encode and insert an entry. Existing immutable keys win.
    pub fn insert<T: Serialize>(
        &self,
        namespace: &str,
        key: &str,
        value: &T,
    ) -> Result<(), String> {
        let target = self.path(namespace, key).ok_or_else(|| {
            "cache namespace and key must be non-empty ASCII identifiers".to_owned()
        })?;
        if target.exists() {
            return Ok(());
        }
        let bytes = serde_json::to_vec(value).map_err(|error| error.to_string())?;
        let mut partial =
            tempfile::NamedTempFile::new_in(&self.directory).map_err(|error| error.to_string())?;
        partial
            .write_all(&bytes)
            .map_err(|error| error.to_string())?;
        match partial.persist_noclobber(&target) {
            Ok(_) => {
                self.prune();
                Ok(())
            }
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(error.error.to_string()),
        }
    }

    fn path(&self, namespace: &str, key: &str) -> Option<PathBuf> {
        if !identifier(namespace) || !identifier(key) {
            return None;
        }
        Some(self.directory.join(format!("{namespace}-{key}.json")))
    }

    fn prune(&self) {
        let Ok(entries) = std::fs::read_dir(&self.directory) else {
            return;
        };
        let mut files = entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .is_some_and(|value| value == "json")
            })
            .filter_map(|entry| {
                let modified = entry.metadata().ok()?.modified().ok()?;
                Some((modified, entry.path()))
            })
            .collect::<Vec<_>>();
        files.sort_by_key(|(modified, _)| *modified);
        let remove = files.len().saturating_sub(self.capacity);
        for (_, path) in files.into_iter().take(remove) {
            let _ = std::fs::remove_file(path);
        }
    }
}

fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn round_trips_and_rejects_path_like_identifiers() {
        let directory = directory();
        let cache = PersistentJsonCache::new(directory.path(), 4).unwrap();
        cache.insert("ocr", "one", &vec!["漫画"]).unwrap();
        cache.insert("ocr", "one", &vec!["replacement"]).unwrap();
        assert_eq!(cache.get::<Vec<String>>("ocr", "one").unwrap(), ["漫画"]);
        assert!(cache.insert("../escape", "one", &true).is_err());
        assert!(cache.get::<bool>("ocr", "../escape").is_none());
    }

    #[test]
    fn content_keys_preserve_boundaries_and_zero_capacity_does_not_retain() {
        assert_ne!(
            PersistentJsonCache::content_key(&[b"ab", b"c"]),
            PersistentJsonCache::content_key(&[b"a", b"bc"])
        );
        let directory = directory();
        let cache = PersistentJsonCache::new(directory.path(), 0).unwrap();
        cache.insert("value", "first", &1).unwrap();
        assert!(cache.get::<u32>("value", "first").is_none());
    }

    #[test]
    fn malformed_entries_are_removed_and_become_misses() {
        let directory = directory();
        let cache = PersistentJsonCache::new(directory.path(), 2).unwrap();
        let path = directory.path().join("test-bad.json");
        std::fs::write(&path, b"not json").unwrap();
        assert!(cache.get::<bool>("test", "bad").is_none());
        assert!(!path.exists());
    }
}
