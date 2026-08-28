//! Retained logical/Taffy node ownership for the protocol applier.

use super::*;

/// Owns the identity and structure invariants shared by protocol, style and layout.
pub(super) struct NodeStore {
    pub(super) tree: TaffyTree<Paint>,
    pub(super) root: NodeId,
    pub(super) solid_to_node: HashMap<NodeKey, NodeId>,
    pub(super) node_to_solid: HashMap<NodeId, NodeKey>,
    /// Logical child order. Taffy children are a projection produced by the IFC.
    pub(super) children: HashMap<NodeId, Vec<NodeId>>,
    pub(super) logical_parent: HashMap<NodeId, NodeId>,
    pub(super) declared: HashMap<NodeId, Declared>,
    pub(super) collapsed_text: HashMap<NodeId, Arc<str>>,
    pub(super) inline_roots: HashSet<NodeId>,
}

impl NodeStore {
    pub(super) fn new() -> Self {
        let mut tree = TaffyTree::new();
        let root_style = taffy::Style {
            size: taffy::geometry::Size {
                width: taffy::Dimension::percent(1.0),
                height: taffy::Dimension::percent(1.0),
            },
            ..taffy::Style::default()
        };
        let root = tree.new_leaf(root_style).expect("root leaf");
        let _ = tree.set_node_context(root, Some(Paint::default()));

        Self {
            tree,
            root,
            solid_to_node: HashMap::from([(NodeKey::ROOT, root)]),
            node_to_solid: HashMap::from([(root, NodeKey::ROOT)]),
            children: HashMap::from([(root, Vec::new())]),
            logical_parent: HashMap::new(),
            declared: HashMap::from([(root, Declared::default())]),
            collapsed_text: HashMap::new(),
            inline_roots: HashSet::new(),
        }
    }

    pub(super) fn solid_id_for_node(&self, node: NodeId) -> Option<NodeKey> {
        self.node_to_solid.get(&node).copied()
    }

    pub(super) fn create_leaf(&mut self, solid_id: NodeKey, declared: Declared) -> NodeId {
        let node = self
            .tree
            .new_leaf(taffy::Style::default())
            .expect("new_leaf");
        self.solid_to_node.insert(solid_id, node);
        self.node_to_solid.insert(node, solid_id);
        self.declared.insert(node, declared);
        self.children.insert(node, Vec::new());
        node
    }

    pub(super) fn append(&mut self, parent: NodeKey, child: NodeKey) -> Option<NodeId> {
        let (&parent, &child) = (
            self.solid_to_node.get(&parent)?,
            self.solid_to_node.get(&child)?,
        );
        if child == self.root || parent == child || self.is_logical_descendant(parent, child) {
            return None;
        }
        self.detach_for_move(child);
        self.children.entry(parent).or_default().push(child);
        self.logical_parent.insert(child, parent);
        Some(child)
    }

    pub(super) fn insert_before(
        &mut self,
        parent: NodeKey,
        child: NodeKey,
        reference: NodeKey,
    ) -> Option<NodeId> {
        let (&parent, &child) = (
            self.solid_to_node.get(&parent)?,
            self.solid_to_node.get(&child)?,
        );
        let &reference = self.solid_to_node.get(&reference)?;
        if child == reference {
            return Some(child);
        }
        if child == self.root || parent == child || self.is_logical_descendant(parent, child) {
            return None;
        }

        // Solid's universal renderer uses append/insert-before for both initial
        // insertion and keyed moves. Mirror DOM move semantics: a node has one
        // logical parent and appears at most once in that parent's child list.
        self.detach_for_move(child);
        let index = self
            .children
            .get(&parent)
            .and_then(|children| children.iter().position(|node| *node == reference))
            .unwrap_or_else(|| self.children.get(&parent).map_or(0, Vec::len));
        let children = self.children.entry(parent).or_default();
        children.insert(index.min(children.len()), child);
        self.logical_parent.insert(child, parent);
        Some(child)
    }

    fn detach_for_move(&mut self, child: NodeId) {
        if let Some(parent) = self.logical_parent.remove(&child)
            && let Some(children) = self.children.get_mut(&parent)
        {
            // `retain` also repairs a tree produced by an older duplicate
            // insertion instead of leaving a stale occurrence behind.
            children.retain(|node| *node != child);
        }
    }

    pub(super) fn remove_child(&mut self, parent: NodeKey, child: NodeKey) -> bool {
        let (Some(&parent), Some(&child)) = (
            self.solid_to_node.get(&parent),
            self.solid_to_node.get(&child),
        ) else {
            return false;
        };
        let Some(children) = self.children.get_mut(&parent) else {
            return false;
        };
        let previous_len = children.len();
        children.retain(|node| *node != child);
        if children.len() == previous_len {
            return false;
        }
        if self.logical_parent.get(&child) == Some(&parent) {
            self.logical_parent.remove(&child);
        }
        true
    }

    pub(super) fn remove(&mut self, solid_id: NodeKey) -> Option<NodeId> {
        let node = self.solid_to_node.remove(&solid_id)?;
        if let Some(parent) = self.logical_parent.get(&node).copied()
            && let Some(children) = self.children.get_mut(&parent)
        {
            children.retain(|child| *child != node);
        }
        self.node_to_solid.remove(&node);
        self.declared.remove(&node);
        self.collapsed_text.remove(&node);
        self.inline_roots.remove(&node);
        self.children.remove(&node);
        self.logical_parent.remove(&node);
        let _ = self.tree.remove(node);
        Some(node)
    }

    pub(super) fn is_logical_descendant(&self, node: NodeId, ancestor: NodeId) -> bool {
        let mut current = Some(node);
        while let Some(node) = current {
            if node == ancestor {
                return true;
            }
            current = self.logical_parent.get(&node).copied();
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap as StdHashMap, HashSet as StdHashSet};

    use proptest::prelude::*;

    use super::*;

    fn assert_structural_invariants(store: &NodeStore) {
        assert_eq!(
            store.solid_to_node.len(),
            store.node_to_solid.len(),
            "identity maps have different cardinality"
        );
        for (&key, &node) in &store.solid_to_node {
            assert_eq!(store.node_to_solid.get(&node), Some(&key));
            assert!(store.children.contains_key(&node));
            assert!(store.declared.contains_key(&node));
        }
        for (&node, &key) in &store.node_to_solid {
            assert_eq!(store.solid_to_node.get(&key), Some(&node));
        }

        assert!(!store.logical_parent.contains_key(&store.root));
        let mut occurrences = StdHashMap::<NodeId, usize>::new();
        for (&parent, children) in &store.children {
            assert!(store.node_to_solid.contains_key(&parent));
            let mut siblings = StdHashSet::new();
            for &child in children {
                assert_ne!(child, store.root, "root appeared as a child");
                assert!(store.node_to_solid.contains_key(&child));
                assert!(siblings.insert(child), "duplicate child in one parent");
                *occurrences.entry(child).or_default() += 1;
                assert_eq!(store.logical_parent.get(&child), Some(&parent));
            }
        }
        for (&child, &parent) in &store.logical_parent {
            assert!(store.node_to_solid.contains_key(&child));
            assert!(store.node_to_solid.contains_key(&parent));
            assert_eq!(occurrences.get(&child), Some(&1));

            let mut ancestors = StdHashSet::new();
            let mut current = Some(child);
            while let Some(node) = current {
                assert!(ancestors.insert(node), "logical parent cycle detected");
                current = store.logical_parent.get(&node).copied();
            }
        }
        for (&child, &count) in &occurrences {
            assert_eq!(count, 1, "child appeared under multiple parents");
            assert!(store.logical_parent.contains_key(&child));
        }
    }

    #[test]
    fn owns_bidirectional_identity_and_logical_structure() {
        let mut store = NodeStore::new();
        let first = store.create_leaf(NodeKey::new(2, 1), Declared::default());
        let second = store.create_leaf(NodeKey::new(3, 1), Declared::default());

        assert_eq!(store.append(NodeKey::ROOT, NodeKey::new(2, 1)), Some(first));
        assert_eq!(
            store.insert_before(NodeKey::ROOT, NodeKey::new(3, 1), NodeKey::new(2, 1)),
            Some(second)
        );
        assert_eq!(store.children[&store.root], [second, first]);
        assert!(store.is_logical_descendant(first, store.root));
        assert_eq!(store.solid_id_for_node(second), Some(NodeKey::new(3, 1)));

        assert_eq!(store.remove(NodeKey::new(3, 1)), Some(second));
        assert_eq!(store.children[&store.root], [first]);
        assert!(!store.node_to_solid.contains_key(&second));
        assert_structural_invariants(&store);
    }

    #[test]
    fn batches_large_sibling_lists_until_layout_projection() {
        let mut store = NodeStore::new();
        for solid_id in 2..=4097 {
            let solid_id = NodeKey::new(solid_id, 1);
            store.create_leaf(solid_id, Declared::default());
            assert!(store.append(NodeKey::ROOT, solid_id).is_some());
        }

        assert_eq!(store.children[&store.root].len(), 4096);
        assert!(store.tree.children(store.root).unwrap().is_empty());
    }

    #[test]
    fn append_moves_nodes_without_duplicating_them() {
        let mut store = NodeStore::new();
        let parent = store.create_leaf(NodeKey::new(2, 1), Declared::default());
        let child = store.create_leaf(NodeKey::new(3, 1), Declared::default());

        store.append(NodeKey::ROOT, NodeKey::new(3, 1));
        store.append(NodeKey::ROOT, NodeKey::new(3, 1));
        assert_eq!(store.children[&store.root], [child]);

        store.append(NodeKey::new(2, 1), NodeKey::new(3, 1));
        assert!(store.children[&store.root].is_empty());
        assert_eq!(store.children[&parent], [child]);
        assert_eq!(store.logical_parent[&child], parent);
    }

    #[test]
    fn insert_before_moves_existing_siblings_in_place() {
        let mut store = NodeStore::new();
        let first = store.create_leaf(NodeKey::new(2, 1), Declared::default());
        let second = store.create_leaf(NodeKey::new(3, 1), Declared::default());
        let third = store.create_leaf(NodeKey::new(4, 1), Declared::default());
        for key in [NodeKey::new(2, 1), NodeKey::new(3, 1), NodeKey::new(4, 1)] {
            store.append(NodeKey::ROOT, key);
        }

        store.insert_before(NodeKey::ROOT, NodeKey::new(4, 1), NodeKey::new(2, 1));
        assert_eq!(store.children[&store.root], [third, first, second]);

        store.insert_before(NodeKey::ROOT, NodeKey::new(2, 1), NodeKey::new(2, 1));
        assert_eq!(store.children[&store.root], [third, first, second]);
        assert_structural_invariants(&store);
    }

    #[test]
    fn rejects_self_parenting_and_ancestor_cycles_without_mutating_the_tree() {
        let mut store = NodeStore::new();
        let parent_key = NodeKey::new(2, 1);
        let child_key = NodeKey::new(3, 1);
        let parent = store.create_leaf(parent_key, Declared::default());
        let child = store.create_leaf(child_key, Declared::default());
        store.append(NodeKey::ROOT, parent_key);
        store.append(parent_key, child_key);
        let before = store
            .children
            .iter()
            .map(|(&node, children)| (node, children.clone()))
            .collect::<StdHashMap<_, _>>();

        assert_eq!(store.append(parent_key, parent_key), None);
        assert_eq!(store.append(child_key, parent_key), None);
        assert_eq!(store.append(parent_key, NodeKey::ROOT), None);
        assert_eq!(store.children, before);
        assert_eq!(store.logical_parent[&parent], store.root);
        assert_eq!(store.logical_parent[&child], parent);
        assert_structural_invariants(&store);
    }

    proptest! {
        #[test]
        fn arbitrary_move_sequences_preserve_tree_invariants(
            operations in prop::collection::vec((0_u8..3, 0_usize..9, 0_usize..9, 0_usize..9), 0..512)
        ) {
            let mut store = NodeStore::new();
            let keys = std::iter::once(NodeKey::ROOT)
                .chain((2..=9).map(|id| NodeKey::new(id, 1)))
                .collect::<Vec<_>>();
            for &key in &keys[1..] {
                store.create_leaf(key, Declared::default());
            }

            for (kind, parent, child, reference) in operations {
                match kind {
                    0 => { store.append(keys[parent], keys[child]); }
                    1 => { store.insert_before(keys[parent], keys[child], keys[reference]); }
                    _ => { store.remove_child(keys[parent], keys[child]); }
                }
                assert_structural_invariants(&store);
            }
        }
    }
}
