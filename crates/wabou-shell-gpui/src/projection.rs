use std::collections::BTreeMap;

use bitflags::bitflags;

/// Stable generational identity shared by the Solid protocol and GPUI projection.
///
/// Both halves remain explicit so deleting and recreating a node cannot make a
/// stale JavaScript handle address a new GPUI element.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeKey {
    pub index: u32,
    pub generation: u32,
}

impl NodeKey {
    #[must_use]
    pub const fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Losslessly encode this retained identity for GPUI's element-state path.
    ///
    /// GPUI creates transient [`gpui::LayoutId`] values while requesting
    /// layout. They must never escape into the Solid protocol. `ElementId`, in
    /// contrast, is the stable identity GPUI uses to preserve element state
    /// across frames.
    #[must_use]
    pub const fn gpui_element_id(self) -> gpui::ElementId {
        let packed = ((self.generation as u64) << u32::BITS) | self.index as u64;
        gpui::ElementId::Integer(packed)
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
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingNode {
    pub key: NodeKey,
    pub dirty: DirtyKind,
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
        let oldest = NodeKey::new(u32::MAX, 0).gpui_element_id();
        let recreated = NodeKey::new(u32::MAX, 1).gpui_element_id();
        let highest = NodeKey::new(u32::MAX, u32::MAX).gpui_element_id();

        assert_eq!(oldest, gpui::ElementId::Integer(u32::MAX as u64));
        assert_eq!(
            recreated,
            gpui::ElementId::Integer((1_u64 << u32::BITS) | u32::MAX as u64)
        );
        assert_eq!(highest, gpui::ElementId::Integer(u64::MAX));
        assert_ne!(oldest, recreated);
    }
}
