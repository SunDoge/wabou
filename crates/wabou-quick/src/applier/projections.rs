//! Retained, revisioned projections derived from the render tree.

use super::*;

pub(super) struct FrameProjections {
    pub(super) layout_metrics: Rc<RefCell<LayoutMetricsSnapshot>>,
    pub(super) semantics_enabled: bool,
    pub(super) semantics_dirty: bool,
    pub(super) semantic_snapshot: Arc<SemanticSnapshot>,
    pub(super) debug_state: Option<wabou_devtools::SharedDebugState>,
    pub(super) debug_revision: u64,
    pub(super) debug_dirty: bool,
}

impl FrameProjections {
    pub(super) fn new(layout_metrics: Rc<RefCell<LayoutMetricsSnapshot>>) -> Self {
        Self {
            layout_metrics,
            semantics_enabled: false,
            semantics_dirty: true,
            semantic_snapshot: Arc::new(SemanticSnapshot::default()),
            debug_state: None,
            debug_revision: 0,
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
