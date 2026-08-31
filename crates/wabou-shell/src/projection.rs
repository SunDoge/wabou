use std::collections::BTreeMap;

use bitflags::bitflags;
pub use wabou_host_api::NodeKey;

/// GPUI-specific operations on Wabou's canonical protocol node identity.
pub trait GpuiNodeKeyExt {
    /// Losslessly encode this retained identity for GPUI's element-state path.
    ///
    /// GPUI creates transient [`gpui::LayoutId`] values while requesting
    /// layout. They must never escape into the Solid protocol. `ElementId`, in
    /// contrast, is the stable identity GPUI uses to preserve element state
    /// across frames.
    fn gpui_element_id(self) -> gpui::ElementId;
}

impl GpuiNodeKeyExt for NodeKey {
    fn gpui_element_id(self) -> gpui::ElementId {
        gpui::ElementId::Integer(self.into())
    }
}

bitflags! {
    /// Work invalidated by one or more Solid mutations during a flush.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct DirtyKind: u8 {
        const LAYOUT = 1 << 0;
        const TEXT = 1 << 1;
        const PAINT = 1 << 2;
        const INTERACTION = 1 << 3;
        const SEMANTICS = 1 << 4;
        /// Child membership/order or projected element identity changed.
        const STRUCTURE = 1 << 5;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingNode {
    pub key: NodeKey,
    pub dirty: DirtyKind,
}

/// Classified invalidation retained from one completed Solid flush.
///
/// Keeping this information past protocol application is required for the
/// GPUI backend to notify projection boundaries selectively instead of
/// reducing every non-empty flush to one root-view boolean.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProjectionInvalidationStats {
    pub revision: u64,
    pub dirty_nodes: usize,
    pub structure_nodes: usize,
    pub layout_nodes: usize,
    pub text_nodes: usize,
    pub paint_nodes: usize,
    pub interaction_nodes: usize,
    pub semantic_nodes: usize,
}

impl ProjectionInvalidationStats {
    #[must_use]
    pub fn from_pending(revision: u64, pending: &[PendingNode]) -> Self {
        let count = |kind| {
            pending
                .iter()
                .filter(|node| node.dirty.contains(kind))
                .count()
        };
        Self {
            revision,
            dirty_nodes: pending.len(),
            structure_nodes: count(DirtyKind::STRUCTURE),
            layout_nodes: count(DirtyKind::LAYOUT),
            text_nodes: count(DirtyKind::TEXT),
            paint_nodes: count(DirtyKind::PAINT),
            interaction_nodes: count(DirtyKind::INTERACTION),
            semantic_nodes: count(DirtyKind::SEMANTICS),
        }
    }

    #[must_use]
    pub const fn changed(self) -> bool {
        self.dirty_nodes != 0
    }
}

/// Coalesces all mutations produced by a single Solid flush.
///
/// The runtime commits this batch once, updates the retained GPUI projection,
/// and calls `cx.notify()` once for the affected coarse surface. Repeated writes
/// to one node therefore do not cause repeated GPUI rebuilds.
#[derive(Debug, Default)]
pub struct FrameBatch {
    revision: u64,
    pending: BTreeMap<NodeKey, DirtyKind>,
}

impl FrameBatch {
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn invalidate(&mut self, key: NodeKey, dirty: DirtyKind) {
        self.pending
            .entry(key)
            .and_modify(|current| *current |= dirty)
            .or_insert(dirty);
    }

    /// Complete one Solid flush and return its unique dirty nodes.
    ///
    /// Empty flushes do not advance the revision and should not notify GPUI.
    #[must_use]
    pub fn commit(&mut self) -> Vec<PendingNode> {
        if self.pending.is_empty() {
            return Vec::new();
        }

        self.revision = self.revision.wrapping_add(1);
        std::mem::take(&mut self.pending)
            .into_iter()
            .map(|(key, dirty)| PendingNode { key, dirty })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_solid_writes_become_one_gpui_invalidation() {
        let key = NodeKey::new(42, 7);
        let mut batch = FrameBatch::default();

        batch.invalidate(key, DirtyKind::PAINT);
        batch.invalidate(key, DirtyKind::TEXT | DirtyKind::LAYOUT);
        batch.invalidate(key, DirtyKind::PAINT);

        assert_eq!(
            batch.commit(),
            [PendingNode {
                key,
                dirty: DirtyKind::PAINT | DirtyKind::TEXT | DirtyKind::LAYOUT,
            }]
        );
        assert_eq!(batch.revision(), 1);
        assert!(batch.is_empty());
    }

    #[test]
    fn empty_flush_does_not_schedule_a_gpui_frame() {
        let mut batch = FrameBatch::default();

        assert!(batch.commit().is_empty());
        assert_eq!(batch.revision(), 0);
    }

    #[test]
    fn generation_keeps_recreated_nodes_distinct() {
        let mut batch = FrameBatch::default();
        batch.invalidate(NodeKey::new(9, 1), DirtyKind::PAINT);
        batch.invalidate(NodeKey::new(9, 2), DirtyKind::PAINT);

        let committed = batch.commit();
        assert_eq!(committed.len(), 2);
        assert_ne!(committed[0].key, committed[1].key);
    }

    #[test]
    fn node_key_maps_losslessly_to_stable_gpui_element_identity() {
        let oldest = NodeKey::new(u32::MAX, 1).gpui_element_id();
        let recreated = NodeKey::new(u32::MAX, 3).gpui_element_id();
        let highest = NodeKey::new(u32::MAX, u32::MAX).gpui_element_id();

        assert_eq!(
            oldest,
            gpui::ElementId::Integer((1_u64 << u32::BITS) | u32::MAX as u64)
        );
        assert_eq!(
            recreated,
            gpui::ElementId::Integer((3_u64 << u32::BITS) | u32::MAX as u64)
        );
        assert_eq!(highest, gpui::ElementId::Integer(u64::MAX));
        assert_ne!(oldest, recreated);
    }

    #[test]
    fn invalidation_stats_preserve_the_strongest_native_phase() {
        let pending = [
            PendingNode {
                key: NodeKey::new(1, 1),
                dirty: DirtyKind::STRUCTURE | DirtyKind::LAYOUT | DirtyKind::PAINT,
            },
            PendingNode {
                key: NodeKey::new(2, 1),
                dirty: DirtyKind::PAINT,
            },
        ];

        assert_eq!(
            ProjectionInvalidationStats::from_pending(7, &pending),
            ProjectionInvalidationStats {
                revision: 7,
                dirty_nodes: 2,
                structure_nodes: 1,
                layout_nodes: 1,
                paint_nodes: 2,
                ..ProjectionInvalidationStats::default()
            }
        );
    }
}
