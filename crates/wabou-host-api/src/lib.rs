//! Rust-owned schema for the low-frequency synchronous QuickJS host API.

#![warn(missing_docs)]

use serde::{Deserialize, Deserializer, Serialize, de};
#[cfg(feature = "bindings")]
use wabou_bindgen::{FunctionModule, NativeMethod};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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
        self.lo != 0 && self.hi != 0 && self.hi % 2 == 1
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
/// Timing and scene-size metrics for the most recently presented frame.
pub struct FrameStats {
    /// Total Rust frame construction time in milliseconds.
    pub build_frame_ms: f64,
    /// QuickJS animation-frame callback time in milliseconds.
    pub js_tick_ms: f64,
    /// Vello scene construction time in milliseconds.
    pub scene_ms: f64,
    /// Surface rendering and presentation time in milliseconds.
    pub present_ms: f64,
    /// Number of retained nodes in the frame.
    pub node_count: usize,
    /// Logical viewport width.
    pub viewport_w: u32,
    /// Logical viewport height.
    pub viewport_h: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
/// Axis-aligned rectangle in logical window coordinates.
pub struct LayoutRect {
    /// Left edge.
    pub x: f32,
    /// Top edge.
    pub y: f32,
    /// Non-negative width.
    pub width: f32,
    /// Non-negative height.
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
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
/// Immutable layout projection returned by the synchronous host API.
pub struct LayoutSnapshot {
    /// Monotonic layout revision used to detect stale snapshots.
    pub revision: u64,
    /// Current logical viewport.
    pub viewport: LayoutRect,
    /// Metrics for the requested node identifiers that still exist.
    pub nodes: Vec<LayoutNodeMetrics>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
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

    pub const CALENDAR_DATE_INFO: &str = r#"interface CalendarDateInfo {
  year: number;
  month: number;
  day: number;
}"#;
    pub const FRAME_STATS_TYPE: &str = r#"interface FrameStats {
  build_frame_ms: number;
  js_tick_ms: number;
  scene_ms: number;
  present_ms: number;
  node_count: number;
  viewport_w: number;
  viewport_h: number;
}"#;
    pub const LAYOUT_RECT: &str = r#"interface LayoutRect {
  x: number;
  y: number;
  width: number;
  height: number;
}"#;
    pub const NODE_KEY: &str = r#"interface NodeKey {
  readonly lo: number;
  readonly hi: number;
}"#;
    pub const LAYOUT_NODE_METRICS: &str = r#"interface LayoutNodeMetrics {
  id: NodeKey;
  rect: LayoutRect;
  clip: LayoutRect;
}"#;
    pub const LAYOUT_SNAPSHOT_TYPE: &str = r#"interface LayoutSnapshot {
  revision: number;
  viewport: LayoutRect;
  nodes: LayoutNodeMetrics[];
}"#;
}

#[cfg(feature = "bindings")]
/// Generate the TypeScript contract for the synchronous native host API.
pub fn bindings() -> FunctionModule {
    FunctionModule::new("NativeHostApi")
        .declaration("CalendarDateInfo", contract::CALENDAR_DATE_INFO)
        .declaration("FrameStats", contract::FRAME_STATS_TYPE)
        .declaration("NodeKey", contract::NODE_KEY)
        .declaration("LayoutRect", contract::LAYOUT_RECT)
        .declaration("LayoutNodeMetrics", contract::LAYOUT_NODE_METRICS)
        .declaration("LayoutSnapshot", contract::LAYOUT_SNAPSHOT_TYPE)
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
