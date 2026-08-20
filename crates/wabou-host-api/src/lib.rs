//! Rust-owned schema for the low-frequency synchronous QuickJS host API.

#![warn(missing_docs)]

use std::marker::PhantomData;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
#[cfg(feature = "bindings")]
use wabou_bindgen::{FunctionModule, NativeMethod};

const fn valid_resource_key_parts(lo: u32, hi: u32) -> bool {
    lo != 0 && hi != 0 && hi % 2 == 1
}

/// Full-width generational identity for one family of native resources.
///
/// The family is a Rust-only brand. Every wire boundary uses the stable
/// `{ lo, hi }` representation so JavaScript never has to represent a `u64`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ResourceKey<Family> {
    lo: u32,
    hi: u32,
    family: PhantomData<fn() -> Family>,
}

impl<Family> ResourceKey<Family> {
    /// Construct a validated key from its complete wire representation.
    pub const fn from_parts(lo: u32, hi: u32) -> Option<Self> {
        if !valid_resource_key_parts(lo, hi) {
            return None;
        }
        Some(Self {
            lo,
            hi,
            family: PhantomData,
        })
    }

    /// Reconstruct and validate a key packed in SlotMap's FFI representation.
    pub const fn from_ffi(value: u64) -> Option<Self> {
        Self::from_parts(value as u32, (value >> 32) as u32)
    }

    /// Low 32 bits: the non-zero SlotMap slot index.
    pub const fn lo(self) -> u32 {
        self.lo
    }

    /// High 32 bits: the non-zero odd SlotMap generation.
    pub const fn hi(self) -> u32 {
        self.hi
    }

    /// Convert to the compact representation used only inside Rust.
    pub const fn as_ffi(self) -> u64 {
        self.lo as u64 | ((self.hi as u64) << 32)
    }

    /// Return the explicit wire pair.
    pub const fn into_parts(self) -> (u32, u32) {
        (self.lo, self.hi)
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

impl<Family> std::fmt::Display for ResourceKey<Family> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}v{}", self.lo, self.hi)
    }
}

#[cfg(test)]
mod resource_key_tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum ImageResource {}
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum FontResource {}

    #[test]
    fn resource_keys_share_one_validated_wire_shape_without_losing_the_family() {
        let image = ResourceKey::<ImageResource>::from_parts(7, 3).unwrap();
        let encoded = serde_json::to_string(&image).unwrap();
        let decoded: ResourceKey<ImageResource> = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.into_parts(), (7, 3));
        assert_eq!(
            ResourceKey::<ImageResource>::from_ffi(image.as_ffi()),
            Some(image)
        );
        assert_eq!(image.to_string(), "7v3");
        assert!(ResourceKey::<FontResource>::from_parts(7, 2).is_none());
        // An image key cannot be passed where ResourceKey<FontResource> is
        // required; resource families remain distinct before serialization.
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(any(feature = "bindings", feature = "specta"), derive(specta::Type))]
/// Full-width generational identity for one retained node.
pub struct NodeKey {
    /// Slot index. Zero is reserved by the wire protocol.
    pub lo: u32,
    /// Non-zero odd generation, matching SlotMap's FFI representation.
    pub hi: u32,
}

impl NodeKey {
    /// Synthetic native window root.
    pub const ROOT: Self = Self { lo: 1, hi: 1 };

    /// Construct a key from its complete wire representation.
    pub const fn new(lo: u32, hi: u32) -> Self {
        Self { lo, hi }
    }

    /// Reconstruct a key from the two-u32 FFI representation packed in a u64.
    pub const fn from_ffi(value: u64) -> Self {
        Self {
            lo: value as u32,
            hi: (value >> 32) as u32,
        }
    }

    /// Whether this is a valid retained-node wire identity.
    pub const fn is_valid(self) -> bool {
        valid_resource_key_parts(self.lo, self.hi)
    }
}

impl<'de> Deserialize<'de> for NodeKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WireNodeKey {
            lo: u32,
            hi: u32,
        }

        let wire = WireNodeKey::deserialize(deserializer)?;
        let key = Self::new(wire.lo, wire.hi);
        if !key.is_valid() {
            return Err(de::Error::custom(
                "invalid NodeKey: lo must be non-zero and hi must be non-zero and odd",
            ));
        }
        Ok(key)
    }
}

impl From<NodeKey> for u64 {
    fn from(value: NodeKey) -> Self {
        u64::from(value.lo) | (u64::from(value.hi) << 32)
    }
}

impl std::fmt::Display for NodeKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}v{}", self.lo, self.hi)
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(any(feature = "bindings", feature = "specta"), derive(specta::Type))]
/// Timing and scene-size metrics for the most recently presented frame.
pub struct FrameStats {
    /// Total Rust frame construction time in milliseconds.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub build_frame_ms: f64,
    /// QuickJS animation-frame callback time in milliseconds.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub js_tick_ms: f64,
    /// Vello scene construction time in milliseconds.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub scene_ms: f64,
    /// Surface rendering and presentation time in milliseconds.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub present_ms: f64,
    /// Number of retained nodes in the frame.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub node_count: usize,
    /// Logical viewport width.
    pub viewport_w: u32,
    /// Logical viewport height.
    pub viewport_h: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(any(feature = "bindings", feature = "specta"), derive(specta::Type))]
/// Axis-aligned rectangle in logical window coordinates.
pub struct LayoutRect {
    /// Left edge.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub x: f32,
    /// Top edge.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub y: f32,
    /// Non-negative width.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub width: f32,
    /// Non-negative height.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(any(feature = "bindings", feature = "specta"), derive(specta::Type))]
/// Layout and effective clipping geometry for one Solid node.
pub struct LayoutNodeMetrics {
    /// Solid-side node identifier.
    pub id: NodeKey,
    /// Border box in logical window coordinates.
    pub rect: LayoutRect,
    /// Effective ancestor clip in logical window coordinates.
    pub clip: LayoutRect,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(any(feature = "bindings", feature = "specta"), derive(specta::Type))]
/// Immutable layout projection returned by the synchronous host API.
pub struct LayoutSnapshot {
    /// Monotonic layout revision used to detect stale snapshots.
    #[cfg_attr(feature = "bindings", specta(type = specta_typescript::Number))]
    pub revision: u64,
    /// Current logical viewport.
    pub viewport: LayoutRect,
    /// Metrics for the requested node identifiers that still exist.
    pub nodes: Vec<LayoutNodeMetrics>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(any(feature = "bindings", feature = "specta"), derive(specta::Type))]
/// A Gregorian calendar date without a time or time zone.
pub struct CalendarDateInfo {
    /// Proleptic Gregorian year.
    pub year: i32,
    /// One-based month.
    pub month: u8,
    /// One-based day of month.
    pub day: u8,
}

#[cfg(feature = "bindings")]
mod contract {
    use super::*;

    pub const OPEN_URL: NativeMethod =
        NativeMethod::sync("openUrl", &[("url", "string")], "boolean");
    pub const LOAD_FONT: NativeMethod =
        NativeMethod::sync("loadFont", &[("path", "string")], "boolean");
    pub const FRAME_STATS: NativeMethod =
        NativeMethod::sync("frameStats", &[], "FrameStats | null");
    pub const LAYOUT_SNAPSHOT: NativeMethod =
        NativeMethod::sync("layoutSnapshot", &[("ids", "NodeKey[]")], "LayoutSnapshot");
    pub const SYSTEM_LOCALE: NativeMethod = NativeMethod::sync("systemLocale", &[], "string");
    pub const SYSTEM_TIME_ZONE: NativeMethod = NativeMethod::sync("systemTimeZone", &[], "string");
    pub const SYSTEM_CALENDAR_DATE: NativeMethod =
        NativeMethod::sync("systemCalendarDate", &[], "CalendarDateInfo");
}

#[cfg(feature = "bindings")]
/// Generate the TypeScript contract for the synchronous native host API.
pub fn bindings() -> FunctionModule {
    FunctionModule::new("NativeHostApi")
        .response_dto::<CalendarDateInfo>()
        .response_dto::<FrameStats>()
        .response_dto::<LayoutSnapshot>()
        .method(contract::OPEN_URL)
        .method(contract::LOAD_FONT)
        .method(contract::FRAME_STATS)
        .method(contract::LAYOUT_SNAPSHOT)
        .method(contract::SYSTEM_LOCALE)
        .method(contract::SYSTEM_TIME_ZONE)
        .method(contract::SYSTEM_CALENDAR_DATE)
}

#[cfg(all(test, feature = "bindings"))]
mod tests {
    use super::*;

    #[test]
    fn exports_synchronous_native_host_contract() {
        let output = bindings().render();
        assert!(output.contains("openUrl(url: string): boolean"));
        assert!(output.contains("frameStats(): FrameStats | null"));
        assert!(output.contains("layoutSnapshot(ids: NodeKey[]): LayoutSnapshot"));
        assert!(output.contains("systemLocale(): string"));
        assert!(output.contains("systemTimeZone(): string"));
        assert!(output.contains("systemCalendarDate(): CalendarDateInfo"));
        assert!(!output.contains("Promise<"));
    }

    #[test]
    fn json_rejects_malformed_node_key_halves() {
        assert_eq!(
            serde_json::from_str::<NodeKey>(r#"{"lo":7,"hi":3}"#).unwrap(),
            NodeKey::new(7, 3)
        );
        for malformed in [
            r#"{"lo":0,"hi":1}"#,
            r#"{"lo":7,"hi":0}"#,
            r#"{"lo":7,"hi":2}"#,
        ] {
            assert!(serde_json::from_str::<NodeKey>(malformed).is_err());
        }
    }
}
