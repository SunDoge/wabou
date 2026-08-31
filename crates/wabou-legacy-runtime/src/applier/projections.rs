//! Retained, revisioned projections derived from the render tree.

use super::*;

pub(super) struct FrameProjections {
    pub(super) layout_metrics: Rc<RefCell<LayoutMetricsSnapshot>>,
    pub(super) semantics_enabled: bool,
    pub(super) semantics_dirty: bool,
    pub(super) semantic_snapshot: Arc<SemanticSnapshot>,
    #[cfg(any(feature = "devtools", test))]
    pub(super) debug_state: Option<wabou_devtools::SharedDebugState>,
    #[cfg(any(feature = "devtools", test))]
    pub(super) debug_revision: u64,
    #[cfg(any(feature = "devtools", test))]
    pub(super) debug_dirty: bool,
}

impl FrameProjections {
    pub(super) fn new(layout_metrics: Rc<RefCell<LayoutMetricsSnapshot>>) -> Self {
        Self {
            layout_metrics,
            semantics_enabled: false,
            semantics_dirty: true,
            semantic_snapshot: Arc::new(SemanticSnapshot::default()),
            #[cfg(any(feature = "devtools", test))]
            debug_state: None,
            #[cfg(any(feature = "devtools", test))]
            debug_revision: 0,
            #[cfg(any(feature = "devtools", test))]
            debug_dirty: true,
        }
    }

    pub(super) fn set_semantics_enabled(&mut self, enabled: bool) {
        let changed = enabled != self.semantics_enabled;
        if enabled && changed {
            self.semantics_dirty = true;
        }
        self.semantics_enabled = enabled;
        if !enabled && changed {
            self.semantic_snapshot = Arc::new(SemanticSnapshot::default());
        }
    }

    pub(super) fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        self.semantics_enabled
            .then(|| self.semantic_snapshot.clone())
    }
}

impl Applier {
    pub(super) fn publish_layout_metrics(&self, placed: &[PlacedNode], width: u32, height: u32) {
        let viewport = LayoutRect {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };
        let mut snapshot = self.frame.projections.layout_metrics.borrow_mut();
        snapshot.revision = snapshot.revision.wrapping_add(1);
        snapshot.viewport = viewport;
        snapshot.nodes.clear();
        snapshot.nodes.reserve(placed.len());
        for placed_node in placed {
            let Some(&id) = self
                .document
                .node_store
                .node_to_solid
                .get(&placed_node.node_id)
            else {
                continue;
            };
            let rect = |value: [f32; 4]| LayoutRect {
                x: value[0],
                y: value[1],
                width: (value[2] - value[0]).max(0.0),
                height: (value[3] - value[1]).max(0.0),
            };
            snapshot.nodes.insert(
                id,
                LayoutMetric {
                    rect: rect(placed_node.rect),
                    clip: placed_node.clip.map_or(viewport, rect),
                    scroll: wabou_host_api::LayoutScrollMetrics {
                        offset_x: self
                            .interaction
                            .scroll
                            .offsets
                            .get(&placed_node.node_id)
                            .map_or(0.0, |offset| offset[0]),
                        offset_y: self
                            .interaction
                            .scroll
                            .offsets
                            .get(&placed_node.node_id)
                            .map_or(0.0, |offset| offset[1]),
                        range_x: placed_node.scroll.range[0],
                        range_y: placed_node.scroll.range[1],
                    },
                },
            );
        }
    }
}
