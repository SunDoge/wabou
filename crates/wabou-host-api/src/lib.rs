//! Rust-owned schema for the low-frequency synchronous QuickJS host API.

use serde::{Deserialize, Serialize};
#[cfg(feature = "bindings")]
use specta::Type;
#[cfg(feature = "bindings")]
use wabou_bindings::FunctionModule;

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct FrameStats {
    pub build_frame_ms: f64,
    pub js_tick_ms: f64,
    pub scene_ms: f64,
    pub present_ms: f64,
    pub node_count: usize,
    pub viewport_w: u32,
    pub viewport_h: u32,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct LayoutNodeMetrics {
    pub id: u32,
    pub rect: LayoutRect,
    pub clip: LayoutRect,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(Type))]
pub struct LayoutSnapshot {
    pub revision: u64,
    pub viewport: LayoutRect,
    pub nodes: Vec<LayoutNodeMetrics>,
}

#[allow(unused_variables)]
#[cfg(feature = "bindings")]
mod contract {
    use super::*;

    #[specta::specta]
    pub fn open_url(url: String) -> bool {
        unreachable!("binding contract functions are not invoked")
    }

    #[specta::specta]
    pub fn load_font(path: String) -> bool {
        unreachable!("binding contract functions are not invoked")
    }

    #[specta::specta]
    pub fn frame_stats() -> Option<FrameStats> {
        unreachable!("binding contract functions are not invoked")
    }

    #[specta::specta]
    pub fn layout_snapshot(ids: Vec<u32>) -> LayoutSnapshot {
        unreachable!("binding contract functions are not invoked")
    }
}

#[cfg(feature = "bindings")]
pub fn bindings() -> FunctionModule {
    FunctionModule::from_specta(
        "NativeHostApi",
        specta::functions::collect_types![
            contract::open_url,
            contract::load_font,
            contract::frame_stats,
            contract::layout_snapshot,
        ],
    )
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
        assert!(!output.contains("Promise<"));
    }
}
