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
        let index = self
            .solid_to_node
            .get(&reference)
            .and_then(|reference| {
                self.children
                    .get(&parent)
                    .and_then(|children| children.iter().position(|node| node == reference))
            })
            .unwrap_or_else(|| self.children.get(&parent).map_or(0, Vec::len));
        let children = self.children.entry(parent).or_default();
        children.insert(index.min(children.len()), child);
        self.logical_parent.insert(child, parent);
        Some(child)
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
    use super::*;

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
}
