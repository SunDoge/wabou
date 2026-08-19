//! Typed registries for independently owned native resources.
//!
//! Tree-owned state should continue to use its owning [`wabou_host_api::NodeKey`].
//! This module is for objects whose lifetime can outlive or move independently
//! of a retained node.

use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use slotmap::{Key, KeyData, SlotMap};

/// Full-width wire representation of one typed SlotMap key.
///
/// `Family` is normally a distinct key type declared with
/// [`slotmap::new_key_type!`]. It is deliberately absent from the serialized
/// `{ lo, hi }` shape; the capability or frame record supplies the family.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResourceKey<Family> {
    lo: u32,
    hi: u32,
    family: PhantomData<fn() -> Family>,
}

impl<Family> ResourceKey<Family> {
    /// Construct and validate a key received as two wire fields.
    pub const fn from_parts(lo: u32, hi: u32) -> Option<Self> {
        if lo == 0 || hi == 0 || hi % 2 == 0 {
            return None;
        }
        Some(Self {
            lo,
            hi,
            family: PhantomData,
        })
    }

    /// Low 32 bits of SlotMap's FFI representation.
    pub const fn lo(self) -> u32 {
        self.lo
    }

    /// High 32 bits of SlotMap's FFI representation.
    pub const fn hi(self) -> u32 {
        self.hi
    }

    /// Return the explicit wire pair.
    pub const fn into_parts(self) -> (u32, u32) {
        (self.lo, self.hi)
    }
}

impl<Family: Key> ResourceKey<Family> {
    /// Convert a typed SlotMap key without passing the packed u64 to JS.
    pub fn from_slotmap(key: Family) -> Self {
        let raw = key.data().as_ffi();
        Self {
            lo: raw as u32,
            hi: (raw >> 32) as u32,
            family: PhantomData,
        }
    }

    fn into_slotmap(self) -> Family {
        Family::from(KeyData::from_ffi(
            u64::from(self.lo) | (u64::from(self.hi) << 32),
        ))
    }
}

impl<Family> Serialize for ResourceKey<Family> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct WireKey {
            lo: u32,
            hi: u32,
        }
        WireKey {
            lo: self.lo,
            hi: self.hi,
        }
        .serialize(serializer)
    }
}

impl<'de, Family> Deserialize<'de> for ResourceKey<Family> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WireKey {
            lo: u32,
            hi: u32,
        }
        let wire = WireKey::deserialize(deserializer)?;
        Self::from_parts(wire.lo, wire.hi).ok_or_else(|| {
            de::Error::custom("resource key lo must be non-zero and hi must be non-zero and odd")
        })
    }
}

/// A SlotMap whose public boundary only exposes family-branded wire keys.
pub struct ResourceRegistry<Family: Key, Value> {
    entries: SlotMap<Family, Value>,
}

impl<Family: Key, Value> Default for ResourceRegistry<Family, Value> {
    fn default() -> Self {
        Self {
            entries: SlotMap::with_key(),
        }
    }
}

impl<Family: Key, Value> ResourceRegistry<Family, Value> {
    /// Create an empty typed registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a native resource and return its complete generational handle.
    pub fn insert(&mut self, value: Value) -> ResourceKey<Family> {
        ResourceKey::from_slotmap(self.entries.insert(value))
    }

    /// Resolve a live handle. Removed generations return `None`.
    pub fn get(&self, key: ResourceKey<Family>) -> Option<&Value> {
        self.entries.get(key.into_slotmap())
    }

    /// Mutably resolve a live handle. Removed generations return `None`.
    pub fn get_mut(&mut self, key: ResourceKey<Family>) -> Option<&mut Value> {
        self.entries.get_mut(key.into_slotmap())
    }

    /// Remove a resource and invalidate this generation.
    pub fn remove(&mut self, key: ResourceKey<Family>) -> Option<Value> {
        self.entries.remove(key.into_slotmap())
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

    slotmap::new_key_type! {
        struct ImageKey;
        struct FontKey;
    }

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
