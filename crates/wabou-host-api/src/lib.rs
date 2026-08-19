//! Rust-owned schema for the low-frequency synchronous QuickJS host API.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
#[cfg(feature = "bindings")]
use wabou_bindgen::{FunctionModule, NativeMethod};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(Type))]
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
#[cfg_attr(feature = "bindings", derive(Type))]
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
#[cfg_attr(feature = "bindings", derive(Type))]
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
#[cfg_attr(feature = "bindings", derive(Type))]
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
#[cfg_attr(feature = "bindings", derive(Type))]
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

    pub const OPEN_URL: NativeMethod<(String,), bool> = NativeMethod::sync("openUrl", &["url"]);
    pub const LOAD_FONT: NativeMethod<(String,), bool> = NativeMethod::sync("loadFont", &["path"]);
    pub const FRAME_STATS: NativeMethod<(), Option<FrameStats>> =
        NativeMethod::sync("frameStats", &[]);
    pub const LAYOUT_SNAPSHOT: NativeMethod<(Vec<u32>,), LayoutSnapshot> =
        NativeMethod::sync("layoutSnapshot", &["ids"]);
    pub const SYSTEM_LOCALE: NativeMethod<(), String> = NativeMethod::sync("systemLocale", &[]);
    pub const SYSTEM_TIME_ZONE: NativeMethod<(), String> =
        NativeMethod::sync("systemTimeZone", &[]);
    pub const SYSTEM_CALENDAR_DATE: NativeMethod<(), CalendarDateInfo> =
        NativeMethod::sync("systemCalendarDate", &[]);
}

#[cfg(feature = "bindings")]
/// Generate the TypeScript contract for the synchronous native host API.
pub fn bindings() -> FunctionModule {
    FunctionModule::new("NativeHostApi")
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
