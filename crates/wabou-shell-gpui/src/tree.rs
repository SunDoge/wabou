use std::collections::BTreeMap;

use gpui::{SharedString, Style};

use crate::{DirtyKind, FrameBatch, NodeKey, PendingNode};

/// One lightweight cached node in the GPUI projection.
///
/// It is not application state. The Solid/runtime retained tree remains
/// authoritative and may reconstruct this cache after HMR or backend restart.
#[derive(Clone, Debug)]
pub struct ProjectedNode {
    pub key: NodeKey,
    pub parent: Option<NodeKey>,
    pub children: Vec<NodeKey>,
    pub style: Style,
    pub text: Option<SharedString>,
}

/// Structural projection failure, always reported before GPUI sees a frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionError {
    DuplicateNode(NodeKey),
    MissingNode(NodeKey),
    MissingParent(NodeKey),
    ParentCycle { node: NodeKey, parent: NodeKey },
    InvalidChildIndex { parent: NodeKey, index: usize },
}

/// Retained projection updated by completed Solid mutation batches.
#[derive(Debug, Default)]
pub struct ProjectionTree {
    nodes: BTreeMap<NodeKey, ProjectedNode>,
    roots: Vec<NodeKey>,
    dirty: FrameBatch,
}

impl ProjectionTree {
    #[must_use]
    pub fn node(&self, key: NodeKey) -> Option<&ProjectedNode> {
        self.nodes.get(&key)
    }

    #[must_use]
    pub fn roots(&self) -> &[NodeKey] {
        &self.roots
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn insert(
        &mut self,
        key: NodeKey,
        parent: Option<NodeKey>,
        index: usize,
        style: Style,
        text: Option<SharedString>,
    ) -> Result<(), ProjectionError> {
        if self.nodes.contains_key(&key) {
            return Err(ProjectionError::DuplicateNode(key));
        }
        if parent == Some(key) {
            return Err(ProjectionError::ParentCycle {
                node: key,
                parent: key,
            });
        }

        match parent {
            Some(parent) => {
                let child_count = self
                    .nodes
                    .get(&parent)
                    .ok_or(ProjectionError::MissingParent(parent))?
                    .children
                    .len();
                if index > child_count {
                    return Err(ProjectionError::InvalidChildIndex { parent, index });
                }
            }
            None if index > self.roots.len() => {
                // A root has no parent key, so report the inserted key as the
                // structural owner of the invalid root position.
                return Err(ProjectionError::InvalidChildIndex { parent: key, index });
            }
            None => {}
        }

        self.nodes.insert(
            key,
            ProjectedNode {
                key,
                parent,
                children: Vec::new(),
                style,
                text,
            },
        );
        if let Some(parent) = parent {
            self.nodes
                .get_mut(&parent)
                .expect("parent was validated")
                .children
                .insert(index, key);
            self.invalidate_layout_chain(parent);
        } else {
            self.roots.insert(index, key);
        }
        self.dirty.invalidate(
            key,
            DirtyKind::LAYOUT
                | DirtyKind::TEXT
                | DirtyKind::PAINT
                | DirtyKind::INTERACTION
                | DirtyKind::SEMANTICS,
        );
        Ok(())
    }

    pub fn update_style(
        &mut self,
        key: NodeKey,
        style: Style,
        dirty: DirtyKind,
    ) -> Result<(), ProjectionError> {
        self.nodes
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .style = style;
        self.dirty.invalidate(key, dirty);
        if dirty.contains(DirtyKind::LAYOUT) {
            self.invalidate_layout_ancestors(key);
        }
        Ok(())
    }

    pub fn update_text(
        &mut self,
        key: NodeKey,
        text: Option<SharedString>,
    ) -> Result<(), ProjectionError> {
        self.nodes
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .text = text;
        self.dirty
            .invalidate(key, DirtyKind::TEXT | DirtyKind::LAYOUT | DirtyKind::PAINT);
        self.invalidate_layout_ancestors(key);
        Ok(())
    }

    pub fn remove(&mut self, key: NodeKey) -> Result<Vec<NodeKey>, ProjectionError> {
        let parent = self
            .nodes
            .get(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .parent;
        let mut removed = Vec::new();
        self.collect_subtree(key, &mut removed);

        if let Some(parent) = parent {
            self.nodes
                .get_mut(&parent)
                .expect("a projected child must retain its parent")
                .children
                .retain(|child| *child != key);
            self.invalidate_layout_chain(parent);
        } else {
            self.roots.retain(|root| *root != key);
        }

        for removed_key in &removed {
            self.nodes.remove(removed_key);
            self.dirty.invalidate(
                *removed_key,
                DirtyKind::LAYOUT
                    | DirtyKind::PAINT
                    | DirtyKind::INTERACTION
                    | DirtyKind::SEMANTICS,
            );
        }
        Ok(removed)
    }

    /// Finish one Solid flush. Call GPUI `notify()` only when this is non-empty.
    #[must_use]
    pub fn commit(&mut self) -> Vec<PendingNode> {
        self.dirty.commit()
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.dirty.revision()
    }

    fn collect_subtree(&self, key: NodeKey, output: &mut Vec<NodeKey>) {
        output.push(key);
        for child in &self.nodes[&key].children {
            self.collect_subtree(*child, output);
        }
    }

    fn invalidate_layout_ancestors(&mut self, key: NodeKey) {
        let mut parent = self.nodes.get(&key).and_then(|node| node.parent);
        while let Some(key) = parent {
            self.dirty.invalidate(key, DirtyKind::LAYOUT);
            parent = self.nodes.get(&key).and_then(|node| node.parent);
        }
    }

    fn invalidate_layout_chain(&mut self, key: NodeKey) {
        self.dirty.invalidate(key, DirtyKind::LAYOUT);
        self.invalidate_layout_ancestors(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(index: u32) -> NodeKey {
        NodeKey::new(index, 1)
    }

    fn insert(tree: &mut ProjectionTree, node: u32, parent: Option<u32>) {
        let parent = parent.map(key);
        let index = parent
            .and_then(|parent| tree.node(parent).map(|node| node.children.len()))
            .unwrap_or_else(|| tree.roots().len());
        tree.insert(key(node), parent, index, Style::default(), None)
            .unwrap();
    }

    #[test]
    fn child_structure_changes_invalidate_every_layout_ancestor() {
        let mut tree = ProjectionTree::default();
        insert(&mut tree, 1, None);
        insert(&mut tree, 2, Some(1));
        let _ = tree.commit();

        insert(&mut tree, 3, Some(2));
        let dirty = tree.commit();

        assert_eq!(dirty.len(), 3);
        assert!(
            dirty
                .iter()
                .any(|node| { node.key == key(1) && node.dirty == DirtyKind::LAYOUT })
        );
        assert!(
            dirty
                .iter()
                .any(|node| { node.key == key(2) && node.dirty == DirtyKind::LAYOUT })
        );
        assert!(dirty.iter().any(|node| {
            node.key == key(3) && node.dirty.contains(DirtyKind::LAYOUT | DirtyKind::PAINT)
        }));
    }

    #[test]
    fn removing_a_node_removes_its_subtree_and_invalidates_parent() {
        let mut tree = ProjectionTree::default();
        insert(&mut tree, 1, None);
        insert(&mut tree, 2, Some(1));
        insert(&mut tree, 3, Some(2));
        let _ = tree.commit();

        assert_eq!(tree.remove(key(2)).unwrap(), [key(2), key(3)]);
        assert_eq!(tree.len(), 1);
        assert!(tree.node(key(2)).is_none());
        assert!(tree.node(key(3)).is_none());
        assert!(tree.node(key(1)).unwrap().children.is_empty());
        assert!(
            tree.commit()
                .iter()
                .any(|node| node.key == key(1) && node.dirty.contains(DirtyKind::LAYOUT))
        );
    }

    #[test]
    fn rejects_duplicate_and_dangling_identity_before_gpui_render() {
        let mut tree = ProjectionTree::default();
        insert(&mut tree, 1, None);

        assert_eq!(
            tree.insert(key(1), None, 1, Style::default(), None),
            Err(ProjectionError::DuplicateNode(key(1)))
        );
        assert_eq!(
            tree.insert(key(2), Some(key(99)), 0, Style::default(), None),
            Err(ProjectionError::MissingParent(key(99)))
        );
    }

    #[test]
    fn empty_commit_does_not_advance_projection_revision() {
        let mut tree = ProjectionTree::default();
        assert!(tree.commit().is_empty());
        assert_eq!(tree.revision(), 0);

        insert(&mut tree, 1, None);
        assert!(!tree.commit().is_empty());
        assert_eq!(tree.revision(), 1);
        assert!(tree.commit().is_empty());
        assert_eq!(tree.revision(), 1);
    }
}
