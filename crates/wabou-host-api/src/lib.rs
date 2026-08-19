//! Rust-owned schema for the low-frequency synchronous QuickJS host API.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use wabou_bindgen::{FunctionModule, NativeMethod};

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
    pub id: u32,
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
        NativeMethod::sync("layoutSnapshot", &[("ids", "number[]")], "LayoutSnapshot");
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
    pub const LAYOUT_NODE_METRICS: &str = r#"interface LayoutNodeMetrics {
  id: number;
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
        assert!(output.contains("layoutSnapshot(ids: number[]): LayoutSnapshot"));
        assert!(output.contains("systemLocale(): string"));
        assert!(output.contains("systemTimeZone(): string"));
        assert!(output.contains("systemCalendarDate(): CalendarDateInfo"));
        assert!(!output.contains("Promise<"));
    }
}
