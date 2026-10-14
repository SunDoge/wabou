//! Typed registries for independently owned native resources.
//!
//! Tree-owned state should continue to use its owning [`wabou_host_api::NodeKey`].
//! This module is for objects whose lifetime can outlive or move independently
//! of a retained node.

use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use slotmap::{DefaultKey, Key, KeyData, SlotMap};

pub use wabou_host_api::ResourceKey;

/// A SlotMap whose public boundary only exposes family-branded wire keys.
pub struct ResourceRegistry<Family, Value> {
    entries: SlotMap<DefaultKey, Value>,
    family: PhantomData<fn() -> Family>,
}

impl<Family, Value> Default for ResourceRegistry<Family, Value> {
    fn default() -> Self {
        Self {
            entries: SlotMap::with_key(),
            family: PhantomData,
        }
    }
}

impl<Family, Value> ResourceRegistry<Family, Value> {
    /// Create an empty typed registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a native resource and return its complete generational handle.
    pub fn insert(&mut self, value: Value) -> ResourceKey<Family> {
        ResourceKey::from_ffi(self.entries.insert(value).data().as_ffi())
            .expect("SlotMap generated an invalid resource key")
    }

    /// Resolve a live handle. Removed generations return `None`.
    pub fn get(&self, key: ResourceKey<Family>) -> Option<&Value> {
        self.entries
            .get(DefaultKey::from(KeyData::from_ffi(key.as_ffi())))
    }

    /// Mutably resolve a live handle. Removed generations return `None`.
    pub fn get_mut(&mut self, key: ResourceKey<Family>) -> Option<&mut Value> {
        self.entries
            .get_mut(DefaultKey::from(KeyData::from_ffi(key.as_ffi())))
    }

    /// Remove a resource and invalidate this generation.
    pub fn remove(&mut self, key: ResourceKey<Family>) -> Option<Value> {
        self.entries
            .remove(DefaultKey::from(KeyData::from_ffi(key.as_ffi())))
    }

    /// Number of currently live resources.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no resources are live.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

struct ResidentEntries<Family, Name, Value> {
    resources: ResourceRegistry<Family, Value>,
    names: HashMap<Name, u64>,
}

/// Application-owned resources that remain alive while SolidJS code reloads.
///
/// The Rust host owns this store and clones it into capability handlers. JS only
/// receives a typed generational [`ResourceKey`], so a module replacement never
/// moves or recreates the Rust value. Named resources can be reopened after a
/// full UI remount: the same name returns the same live handle and value.
///
/// Keep long-running or asynchronously accessed values behind `Arc` and make
/// the closure passed to [`Self::with`] short; access holds the registry lock.
pub struct ResidentResources<Family, Value, Name = String> {
    entries: Arc<Mutex<ResidentEntries<Family, Name, Value>>>,
}

impl<Family, Value, Name> Clone for ResidentResources<Family, Value, Name> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<Family, Value, Name> Default for ResidentResources<Family, Value, Name> {
    fn default() -> Self {
        Self {
            entries: Arc::new(Mutex::new(ResidentEntries {
                resources: ResourceRegistry::new(),
                names: HashMap::new(),
            })),
        }
    }
}

impl<Family, Value, Name> ResidentResources<Family, Value, Name> {
    /// Create an empty resident store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an unnamed resource. It remains alive until explicitly removed or
    /// until the Rust application drops the last clone of this store.
    pub fn insert(&self, value: Value) -> ResourceKey<Family> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resources
            .insert(value)
    }

    /// Read a live resource under the registry lock.
    pub fn with<T>(&self, key: ResourceKey<Family>, access: impl FnOnce(&Value) -> T) -> Option<T> {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.resources.get(key).map(access)
    }

    /// Mutate a live resource under the registry lock.
    pub fn with_mut<T>(
        &self,
        key: ResourceKey<Family>,
        access: impl FnOnce(&mut Value) -> T,
    ) -> Option<T> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        entries.resources.get_mut(key).map(access)
    }

    /// Remove a resource and invalidate every copy of its handle.
    pub fn remove(&self, key: ResourceKey<Family>) -> Option<Value>
    where
        Name: Eq + Hash,
    {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let value = entries.resources.remove(key)?;
        let raw = key.as_ffi();
        entries.names.retain(|_, candidate| *candidate != raw);
        Some(value)
    }

    /// Number of currently live resources.
    pub fn len(&self) -> usize {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resources
            .len()
    }

    /// Whether no resources are live.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Family, Value: Clone, Name> ResidentResources<Family, Value, Name> {
    /// Clone a live value and immediately release the registry lock.
    ///
    /// Store `Arc<T>` values and use this before asynchronous work so no lock is
    /// held across an await point.
    pub fn get_cloned(&self, key: ResourceKey<Family>) -> Option<Value> {
        self.with(key, Clone::clone)
    }
}

impl<Family, Value, Name> ResidentResources<Family, Value, Name>
where
    Name: Eq + Hash,
{
    /// Resolve a named resource if it is still live.
    pub fn named(&self, name: &Name) -> Option<ResourceKey<Family>> {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let key = ResourceKey::from_ffi(*entries.names.get(name)?)?;
        entries.resources.get(key).map(|_| key)
    }

    /// Return the existing live resource for `name`, or create it once.
    ///
    /// A reloaded Solid module can call the same open method again and recover
    /// the original resource even if its JavaScript handle variable was lost.
    pub fn get_or_insert_with(
        &self,
        name: Name,
        create: impl FnOnce() -> Value,
    ) -> ResourceKey<Family> {
        let mut entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(key) = entries
            .names
            .get(&name)
            .copied()
            .and_then(ResourceKey::from_ffi)
            && entries.resources.get(key).is_some()
        {
            return key;
        }
        let key = entries.resources.insert(create());
        entries.names.insert(name, key.as_ffi());
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum ImageKey {}
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum FontKey {}

    #[test]
    fn registry_round_trips_both_wire_halves_and_rejects_stale_keys() {
        let mut images = ResourceRegistry::<ImageKey, _>::new();
        let first = images.insert("first");
        let json = serde_json::to_string(&first).unwrap();
        let decoded: ResourceKey<ImageKey> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, first);
        assert_eq!(images.remove(first), Some("first"));

        let replacement = images.insert("replacement");
        assert_eq!(replacement.lo(), first.lo());
        assert_ne!(replacement.hi(), first.hi());
        assert_eq!(images.get(first), None);
        assert_eq!(images.get(replacement), Some(&"replacement"));
    }

    #[test]
    fn family_is_part_of_the_rust_type() {
        let mut images = ResourceRegistry::<ImageKey, _>::new();
        let mut fonts = ResourceRegistry::<FontKey, _>::new();
        let image: ResourceKey<ImageKey> = images.insert("pixels");
        let font: ResourceKey<FontKey> = fonts.insert("glyphs");

        assert_eq!(images.get(image), Some(&"pixels"));
        assert_eq!(fonts.get(font), Some(&"glyphs"));
        // `images.get(font)` intentionally does not compile: family confusion
        // is rejected before the untrusted wire pair reaches a SlotMap.
    }

    #[test]
    fn malformed_json_never_constructs_a_slotmap_key() {
        assert!(serde_json::from_str::<ResourceKey<ImageKey>>(r#"{"lo":0,"hi":1}"#).is_err());
        assert!(serde_json::from_str::<ResourceKey<ImageKey>>(r#"{"lo":1,"hi":2}"#).is_err());
    }

    #[test]
    fn named_resident_resources_rebind_after_ui_reload() {
        let resources = ResidentResources::<ImageKey, Vec<u8>>::new();
        let first = resources.get_or_insert_with("preview".to_owned(), || vec![1]);
        resources.with_mut(first, |bytes| bytes.push(2)).unwrap();

        // Model a new capability closure installed after a Solid module reload.
        let reloaded = resources.clone();
        let rebound = reloaded.get_or_insert_with("preview".to_owned(), || vec![9]);
        assert_eq!(rebound, first);
        assert_eq!(reloaded.with(rebound, Clone::clone), Some(vec![1, 2]));
        assert_eq!(reloaded.len(), 1);

        assert_eq!(resources.remove(first), Some(vec![1, 2]));
        let replacement = reloaded.get_or_insert_with("preview".to_owned(), || vec![3]);
        assert_ne!(replacement, first);
        assert_eq!(reloaded.with(first, Clone::clone), None);
    }
}
