//! Typed registries for independently owned native resources.
//!
//! Tree-owned state should continue to use its owning [`wabou_host_api::NodeKey`].
//! This module is for objects whose lifetime can outlive or move independently
//! of a retained node.

use std::marker::PhantomData;

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
}
