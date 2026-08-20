//! Shared frame construction for renderers without a native swapchain.

use std::time::Instant;

use vello::Scene;
use vello::peniko::Color;

use crate::layout::PlacedNode;
use crate::scene;
use crate::source::{FrameSource, FrameStats};
use crate::text::TextContext;

/// Persistent timing state for a headless window.
///
/// A profiler owns the same exponential moving average used by the native
/// window path. Reusing it across frames keeps screenshots and deterministic
/// behavior tests on the public `FrameStats` contract without inventing fake
/// presentation timings.
#[derive(Debug, Default)]
pub struct HeadlessFrameProfiler {
    stats: FrameStats,
}

impl HeadlessFrameProfiler {
    /// Construct and profile one complete headless scene.
    ///
    /// Presentation remains zero because no swapchain is involved. Debug
    /// overlays are encoded after the application scene, matching native
    /// ordering, and their paint evidence is therefore observable in tests.
    pub fn build(
        &mut self,
        source: &mut dyn FrameSource,
        text: &mut TextContext,
        width: u32,
        height: u32,
        scale_factor: f64,
        base_color: Color,
    ) -> Vec<PlacedNode> {
        let build_started = Instant::now();
        let nodes = source.build_frame(text, width, height);
        let build_frame_ms = build_started.elapsed().as_secs_f64() * 1_000.0;

        let scene_started = Instant::now();
        let mut output = Scene::new();
        scene::build_scene_scaled(
            &mut output,
            &nodes,
            text,
            width,
            height,
            base_color,
            scale_factor,
        );
        source.paint_debug_overlay(&mut output, &nodes, text, scale_factor);
        let scene_ms = scene_started.elapsed().as_secs_f64() * 1_000.0;

        self.stats
            .update(build_frame_ms, scene_ms, 0.0, nodes.len());
        source.push_frame_stats(&self.stats);
        nodes
    }

    /// Most recently accumulated headless timings.
    pub fn stats(&self) -> FrameStats {
        self.stats
    }
}
