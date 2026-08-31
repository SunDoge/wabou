use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use gpui::{SharedString, Style};
use std::hash::{Hash, Hasher};
use wabou_protocol::TEXT_BEHAVIOR_SINGLE_LINE;

use crate::element::ProjectedElementContext;
use crate::{
    DirtyKind, FrameBatch, NodeKey, PendingNode, ProjectedElement, ProjectedInputSink,
    ProjectedNativeElementFactory,
};

#[derive(Clone, Debug)]
pub struct ProjectedSvgSource {
    pub bytes: std::sync::Arc<[u8]>,
    pub cache_key: SharedString,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedScrollbarStyle {
    pub visibility: u8,
    pub hide_delay: f32,
    pub fade_duration: f32,
    pub thickness: f32,
    pub margin: f32,
    pub min_thumb_length: f32,
    pub radius: f32,
    pub colors: [u32; 4],
}

impl Default for ProjectedScrollbarStyle {
    fn default() -> Self {
        Self {
            visibility: 0,
            hide_delay: 500.0,
            fade_duration: 200.0,
            thickness: 10.0,
            margin: 2.0,
            min_thumb_length: 32.0,
            radius: -1.0,
            colors: [0x0000_0000, 0x6474_8fbe, 0x6474_8fe1, 0x4755_69ff],
        }
    }
}

/// Explicit semantic kind retained for every projected protocol node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectedNodeKind {
    /// Canonical application root, not authored by JSX.
    Root,
    /// Authored element tag. Custom/native widget names are preserved verbatim.
    Element(SharedString),
    /// Text node emitted by the Solid renderer.
    Text,
}

/// One lightweight cached node in the GPUI projection.
///
/// It is not application state. The Solid/runtime retained tree remains
/// authoritative and may reconstruct this cache after HMR or backend restart.
#[derive(Clone, Debug)]
pub struct ProjectedNode {
    pub key: NodeKey,
    pub kind: ProjectedNodeKind,
    pub parent: Option<NodeKey>,
    /// Whether this node participates in the projected element tree.
    pub attached: bool,
    pub children: Vec<NodeKey>,
    pub style: Style,
    pub text: Option<SharedString>,
    /// Raw structured configuration authored for an application-defined
    /// native widget. The protocol owns validation and transport; the widget
    /// factory decides how to deserialize its typed configuration.
    pub widget_config: Option<SharedString>,
    /// Explicit text assembly policy emitted by the JavaScript primitive.
    pub text_behavior: u8,
    /// Maximum visible lines. Zero means unlimited.
    pub text_max_lines: u32,
    /// Optional GPUI-owned display asset projected from a graphic source.
    pub image: Option<std::sync::Arc<gpui::Image>>,
    /// Trusted inline SVG bytes retained separately so GPUI can apply its
    /// native paint transformation instead of flattening the SVG to an image.
    pub svg_source: Option<ProjectedSvgSource>,
    pub scrollbar_style: Option<ProjectedScrollbarStyle>,
    pub(crate) vector_path: Option<std::sync::Arc<crate::vector_path::ProjectedVectorPath>>,
    /// Runtime affine transform emitted by the Solid renderer. GPUI applies
    /// the full matrix to inline SVG and translation to ordinary elements;
    /// unsupported ordinary-element affine parts remain observable.
    pub transform: [f32; 6],
    /// Authored attributes after the latest completed Solid flush.
    pub attributes: BTreeMap<SharedString, SharedString>,
    /// Guest event codes registered on this exact retained node.
    pub listeners: BTreeSet<u8>,
    /// Explicit keyboard focus order, or `None` when the node is not focusable.
    pub focus_order: Option<i32>,
    /// Whether native interaction is blocked for this node and its subtree.
    pub interaction_blocked: bool,
    /// Whether this exact node may become a pointer hit target. Unlike
    /// `interaction_blocked`, this does not suppress interactive descendants.
    pub pointer_events: bool,
    /// GPUI deferred-draw priority for this node. Wabou intentionally exposes
    /// only non-negative stacking priorities rather than CSS stacking contexts.
    pub z_index: usize,
    /// Explicit Wabou overlay plane: content (0), floating (1), modal (2).
    pub overlay_plane: u8,
    /// Whether focus traversal is contained inside this subtree.
    pub focus_contained: bool,
}

impl ProjectedNode {
    #[must_use]
    pub fn draw_priority(&self) -> usize {
        usize::from(self.overlay_plane) * 1_000_000 + self.z_index.min(999_999)
    }
}

/// Structural projection failure, always reported before GPUI sees a frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionError {
    DuplicateNode(NodeKey),
    MissingNode(NodeKey),
    MissingParent(NodeKey),
    ParentCycle { node: NodeKey, parent: NodeKey },
    InvalidChildIndex { parent: NodeKey, index: usize },
    InvalidGraphicData(NodeKey),
}

/// Immutable view of one committed retained node table.
#[derive(Clone, Debug)]
pub struct ProjectionSnapshot {
    nodes: Rc<BTreeMap<NodeKey, ProjectedNode>>,
}

impl ProjectionSnapshot {
    #[must_use]
    pub fn node(&self, key: NodeKey) -> Option<&ProjectedNode> {
        self.nodes.get(&key)
    }

    pub fn keys(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.nodes.keys().copied()
    }
}

/// Retained projection updated by completed Solid mutation batches.
#[derive(Debug, Default)]
pub struct ProjectionTree {
    nodes: Rc<BTreeMap<NodeKey, ProjectedNode>>,
    roots: Vec<NodeKey>,
    dirty: FrameBatch,
}

impl ProjectionTree {
    fn nodes_mut(&mut self) -> &mut BTreeMap<NodeKey, ProjectedNode> {
        Rc::make_mut(&mut self.nodes)
    }

    /// Freeze the retained node table for a GPUI callback without cloning
    /// styles, attributes, text, or image handles.
    #[must_use]
    pub fn snapshot(&self) -> ProjectionSnapshot {
        ProjectionSnapshot {
            nodes: self.nodes.clone(),
        }
    }

    #[must_use]
    pub fn node(&self, key: NodeKey) -> Option<&ProjectedNode> {
        self.nodes.get(&key)
    }

    #[must_use]
    pub fn roots(&self) -> &[NodeKey] {
        &self.roots
    }

    pub fn keys(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.nodes.keys().copied()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Materialize a fresh GPUI element tree for one retained root.
    ///
    /// The objects are intentionally ephemeral; their stable GPUI element IDs
    /// preserve state and paint caches across frames.
    pub fn element(&self, root: NodeKey) -> Result<ProjectedElement, ProjectionError> {
        let snapshot = self.snapshot();
        ProjectedElement::from_tree(snapshot, root, ProjectedElementContext::default(), false)
    }

    /// Materialize a root whose GPUI hit targets emit typed pointer events.
    pub fn interactive_element(
        &self,
        root: NodeKey,
        input: ProjectedInputSink,
        focus: gpui::FocusHandle,
        text_input: crate::ProjectedTextInputState,
        native: Option<ProjectedNativeElementFactory>,
    ) -> Result<ProjectedElement, ProjectionError> {
        let snapshot = self.snapshot();
        ProjectedElement::from_tree(
            snapshot,
            root,
            ProjectedElementContext {
                input: Some(input),
                root_focus: Some(focus),
                text_input: Some(text_input),
                native,
                layout_bounds: None,
                graphic_paint_states: None,
                scroll_handles: None,
                uniform_list_handles: None,
            },
            false,
        )
    }

    pub(crate) fn interactive_element_with_layout_bounds(
        &self,
        root: NodeKey,
        input: ProjectedInputSink,
        focus: gpui::FocusHandle,
        text_input: crate::ProjectedTextInputState,
        native: Option<ProjectedNativeElementFactory>,
        layout_bounds: crate::element::ProjectedLayoutBounds,
        graphic_paint_states: crate::element::ProjectedGraphicPaintStates,
        scroll_handles: std::rc::Rc<
            std::collections::BTreeMap<NodeKey, crate::ProjectedScrollHandle>,
        >,
        uniform_list_handles: std::rc::Rc<
            std::collections::BTreeMap<NodeKey, gpui::UniformListScrollHandle>,
        >,
    ) -> Result<ProjectedElement, ProjectionError> {
        let snapshot = self.snapshot();
        ProjectedElement::from_tree(
            snapshot,
            root,
            ProjectedElementContext {
                input: Some(input),
                root_focus: Some(focus),
                text_input: Some(text_input),
                native,
                layout_bounds: Some(layout_bounds),
                graphic_paint_states: Some(graphic_paint_states),
                scroll_handles: Some(scroll_handles),
                uniform_list_handles: Some(uniform_list_handles),
            },
            false,
        )
    }

    pub fn insert(
        &mut self,
        key: NodeKey,
        parent: Option<NodeKey>,
        index: usize,
        style: Style,
        text: Option<SharedString>,
        kind: ProjectedNodeKind,
    ) -> Result<(), ProjectionError> {
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
                return Err(ProjectionError::InvalidChildIndex { parent: key, index });
            }
            None => {}
        }
        self.insert_detached(key, style, text, kind)?;
        match parent {
            Some(parent) => self.attach_child(key, parent, index),
            None => self.attach_root(key, index),
        }
    }

    /// Register a node created by Solid before it is inserted into the UI tree.
    pub fn insert_detached(
        &mut self,
        key: NodeKey,
        style: Style,
        text: Option<SharedString>,
        kind: ProjectedNodeKind,
    ) -> Result<(), ProjectionError> {
        if self.nodes.contains_key(&key) {
            return Err(ProjectionError::DuplicateNode(key));
        }

        self.nodes_mut().insert(
            key,
            ProjectedNode {
                key,
                kind,
                parent: None,
                attached: false,
                children: Vec::new(),
                style,
                text,
                widget_config: None,
                text_behavior: 0,
                text_max_lines: 0,
                image: None,
                svg_source: None,
                scrollbar_style: None,
                vector_path: None,
                transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                attributes: BTreeMap::new(),
                listeners: BTreeSet::new(),
                focus_order: None,
                interaction_blocked: false,
                pointer_events: true,
                z_index: 0,
                overlay_plane: 0,
                focus_contained: false,
            },
        );
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

    /// Insert or move a retained node under a parent at an explicit child index.
    pub fn attach_child(
        &mut self,
        key: NodeKey,
        parent: NodeKey,
        index: usize,
    ) -> Result<(), ProjectionError> {
        if !self.nodes.contains_key(&key) {
            return Err(ProjectionError::MissingNode(key));
        }
        if !self.nodes.contains_key(&parent) {
            return Err(ProjectionError::MissingParent(parent));
        }
        let mut ancestor = Some(parent);
        while let Some(current) = ancestor {
            if current == key {
                return Err(ProjectionError::ParentCycle { node: key, parent });
            }
            ancestor = self.nodes.get(&current).and_then(|node| node.parent);
        }

        let moving_within_parent =
            self.nodes[&key].parent == Some(parent) && self.nodes[&key].attached;
        let child_count = self.nodes[&parent].children.len() - usize::from(moving_within_parent);
        if index > child_count {
            return Err(ProjectionError::InvalidChildIndex { parent, index });
        }

        self.detach_from_location(key);
        self.nodes_mut()
            .get_mut(&parent)
            .expect("parent was validated")
            .children
            .insert(index, key);
        let node = self.nodes_mut().get_mut(&key).expect("child was validated");
        node.parent = Some(parent);
        node.attached = true;
        self.invalidate_layout_chain(parent);
        self.dirty
            .invalidate(key, DirtyKind::LAYOUT | DirtyKind::PAINT);
        Ok(())
    }

    /// Detach a node without destroying its retained identity or subtree.
    pub fn detach(&mut self, key: NodeKey) -> Result<(), ProjectionError> {
        if !self.nodes.contains_key(&key) {
            return Err(ProjectionError::MissingNode(key));
        }
        self.detach_from_location(key);
        self.dirty
            .invalidate(key, DirtyKind::LAYOUT | DirtyKind::PAINT);
        Ok(())
    }

    fn attach_root(&mut self, key: NodeKey, index: usize) -> Result<(), ProjectionError> {
        if index > self.roots.len() {
            return Err(ProjectionError::InvalidChildIndex { parent: key, index });
        }
        self.roots.insert(index, key);
        self.nodes_mut()
            .get_mut(&key)
            .expect("node was just inserted")
            .attached = true;
        Ok(())
    }

    fn detach_from_location(&mut self, key: NodeKey) {
        let (parent, attached) = {
            let node = &self.nodes[&key];
            (node.parent, node.attached)
        };
        if !attached {
            return;
        }
        if let Some(parent) = parent {
            self.nodes_mut()
                .get_mut(&parent)
                .expect("attached parent remains retained")
                .children
                .retain(|child| *child != key);
            self.invalidate_layout_chain(parent);
        } else {
            self.roots.retain(|root| *root != key);
        }
        let node = self
            .nodes_mut()
            .get_mut(&key)
            .expect("node remains retained");
        node.parent = None;
        node.attached = false;
    }

    pub fn update_style(
        &mut self,
        key: NodeKey,
        style: Style,
        dirty: DirtyKind,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
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
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .text = text;
        self.dirty
            .invalidate(key, DirtyKind::TEXT | DirtyKind::LAYOUT | DirtyKind::PAINT);
        self.invalidate_layout_ancestors(key);
        Ok(())
    }

    pub fn update_widget_config(
        &mut self,
        key: NodeKey,
        config: Option<SharedString>,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .widget_config = config;
        self.dirty.invalidate(
            key,
            DirtyKind::LAYOUT | DirtyKind::PAINT | DirtyKind::SEMANTICS,
        );
        Ok(())
    }

    pub fn update_text_behavior(&mut self, key: NodeKey, flags: u8) -> Result<(), ProjectionError> {
        let node = self
            .nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?;
        node.text_behavior = flags;
        node.style.text.white_space =
            (flags & TEXT_BEHAVIOR_SINGLE_LINE != 0).then_some(gpui::WhiteSpace::Nowrap);
        self.dirty
            .invalidate(key, DirtyKind::TEXT | DirtyKind::LAYOUT | DirtyKind::PAINT);
        self.invalidate_layout_ancestors(key);
        Ok(())
    }

    pub fn update_text_max_lines(
        &mut self,
        key: NodeKey,
        max_lines: u32,
    ) -> Result<(), ProjectionError> {
        let node = self
            .nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?;
        node.text_max_lines = max_lines;
        node.style.text.line_clamp = usize::try_from(max_lines).ok().filter(|lines| *lines > 0);
        self.dirty
            .invalidate(key, DirtyKind::TEXT | DirtyKind::LAYOUT | DirtyKind::PAINT);
        self.invalidate_layout_ancestors(key);
        Ok(())
    }

    /// Replace the display image attached to one retained node.
    pub fn update_image(
        &mut self,
        key: NodeKey,
        image: Option<std::sync::Arc<gpui::Image>>,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .image = image;
        self.dirty
            .invalidate(key, DirtyKind::LAYOUT | DirtyKind::PAINT);
        self.invalidate_layout_ancestors(key);
        Ok(())
    }

    pub fn update_svg_source(
        &mut self,
        key: NodeKey,
        source: Option<std::sync::Arc<[u8]>>,
    ) -> Result<(), ProjectionError> {
        let source = source.map(|bytes| {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            bytes.hash(&mut hasher);
            ProjectedSvgSource {
                cache_key: format!("wabou-inline-svg:{:016x}", hasher.finish()).into(),
                bytes,
            }
        });
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .svg_source = source;
        self.dirty.invalidate(key, DirtyKind::PAINT);
        Ok(())
    }

    pub fn update_scrollbar_style(
        &mut self,
        key: NodeKey,
        style: ProjectedScrollbarStyle,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .scrollbar_style = Some(style);
        self.dirty
            .invalidate(key, DirtyKind::PAINT | DirtyKind::INTERACTION);
        Ok(())
    }

    pub(crate) fn update_vector_path(
        &mut self,
        key: NodeKey,
        vector_path: Option<std::sync::Arc<crate::vector_path::ProjectedVectorPath>>,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .vector_path = vector_path;
        self.dirty.invalidate(key, DirtyKind::PAINT);
        Ok(())
    }

    pub fn update_transform(
        &mut self,
        key: NodeKey,
        transform: [f32; 6],
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .transform = transform;
        self.dirty
            .invalidate(key, DirtyKind::PAINT | DirtyKind::INTERACTION);
        Ok(())
    }

    /// Set one authored attribute without publishing a partial frame.
    pub fn update_attribute(
        &mut self,
        key: NodeKey,
        name: SharedString,
        value: SharedString,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .attributes
            .insert(name, value);
        self.dirty.invalidate(
            key,
            DirtyKind::LAYOUT | DirtyKind::PAINT | DirtyKind::SEMANTICS,
        );
        Ok(())
    }

    /// Remove one authored attribute without publishing a partial frame.
    pub fn remove_attribute(&mut self, key: NodeKey, name: &str) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .attributes
            .remove(name);
        self.dirty.invalidate(
            key,
            DirtyKind::LAYOUT | DirtyKind::PAINT | DirtyKind::SEMANTICS,
        );
        Ok(())
    }

    pub fn add_event_listener(
        &mut self,
        key: NodeKey,
        event_type: u8,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .listeners
            .insert(event_type);
        self.dirty.invalidate(key, DirtyKind::INTERACTION);
        Ok(())
    }

    pub fn remove_event_listener(
        &mut self,
        key: NodeKey,
        event_type: u8,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .listeners
            .remove(&event_type);
        self.dirty.invalidate(key, DirtyKind::INTERACTION);
        Ok(())
    }

    pub fn update_interaction_policy(
        &mut self,
        key: NodeKey,
        focus_order: Option<i32>,
        blocked: bool,
        contained: bool,
    ) -> Result<(), ProjectionError> {
        let node = self
            .nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?;
        node.focus_order = focus_order;
        node.interaction_blocked = blocked;
        node.focus_contained = contained;
        self.dirty
            .invalidate(key, DirtyKind::INTERACTION | DirtyKind::SEMANTICS);
        Ok(())
    }

    pub fn update_pointer_events(
        &mut self,
        key: NodeKey,
        enabled: bool,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .pointer_events = enabled;
        self.dirty.invalidate(key, DirtyKind::INTERACTION);
        Ok(())
    }

    pub fn update_z_index(&mut self, key: NodeKey, z_index: usize) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .z_index = z_index;
        self.dirty
            .invalidate(key, DirtyKind::PAINT | DirtyKind::INTERACTION);
        Ok(())
    }

    pub fn update_overlay_plane(
        &mut self,
        key: NodeKey,
        overlay_plane: u8,
    ) -> Result<(), ProjectionError> {
        self.nodes_mut()
            .get_mut(&key)
            .ok_or(ProjectionError::MissingNode(key))?
            .overlay_plane = overlay_plane.min(2);
        self.dirty.invalidate(
            key,
            DirtyKind::PAINT | DirtyKind::INTERACTION | DirtyKind::SEMANTICS,
        );
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
            self.nodes_mut()
                .get_mut(&parent)
                .expect("a projected child must retain its parent")
                .children
                .retain(|child| *child != key);
            self.invalidate_layout_chain(parent);
        } else if self.nodes[&key].attached {
            self.roots.retain(|root| *root != key);
        }

        for removed_key in &removed {
            self.nodes_mut().remove(removed_key);
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
        tree.insert(
            key(node),
            parent,
            index,
            Style::default(),
            None,
            ProjectedNodeKind::Element("view".into()),
        )
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
    fn solid_created_nodes_can_attach_move_and_detach_without_changing_identity() {
        let mut tree = ProjectionTree::default();
        insert(&mut tree, 1, None);
        insert(&mut tree, 2, None);
        tree.insert_detached(
            key(3),
            Style::default(),
            Some("retained".into()),
            ProjectedNodeKind::Text,
        )
        .unwrap();

        assert!(!tree.node(key(3)).unwrap().attached);
        tree.attach_child(key(3), key(1), 0).unwrap();
        assert_eq!(tree.node(key(1)).unwrap().children, [key(3)]);
        assert_eq!(tree.node(key(3)).unwrap().parent, Some(key(1)));

        tree.attach_child(key(3), key(2), 0).unwrap();
        assert!(tree.node(key(1)).unwrap().children.is_empty());
        assert_eq!(tree.node(key(2)).unwrap().children, [key(3)]);
        assert_eq!(tree.node(key(3)).unwrap().text.as_deref(), Some("retained"));

        tree.detach(key(3)).unwrap();
        assert!(!tree.node(key(3)).unwrap().attached);
        assert_eq!(tree.node(key(3)).unwrap().parent, None);
        assert!(tree.node(key(2)).unwrap().children.is_empty());
    }

    #[test]
    fn rejects_duplicate_and_dangling_identity_before_gpui_render() {
        let mut tree = ProjectionTree::default();
        insert(&mut tree, 1, None);

        assert_eq!(
            tree.insert(
                key(1),
                None,
                1,
                Style::default(),
                None,
                ProjectedNodeKind::Element("view".into()),
            ),
            Err(ProjectionError::DuplicateNode(key(1)))
        );
        assert_eq!(
            tree.insert(
                key(2),
                Some(key(99)),
                0,
                Style::default(),
                None,
                ProjectedNodeKind::Element("view".into()),
            ),
            Err(ProjectionError::MissingParent(key(99)))
        );
        assert!(tree.node(key(2)).is_none());
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

    #[test]
    fn snapshots_keep_the_last_committed_nodes_while_the_next_batch_mutates() {
        let mut tree = ProjectionTree::default();
        tree.insert(
            key(1),
            None,
            0,
            Style::default(),
            Some("before".into()),
            ProjectedNodeKind::Text,
        )
        .unwrap();
        let snapshot = tree.snapshot();

        tree.update_text(key(1), Some("after".into())).unwrap();

        assert_eq!(
            snapshot.node(key(1)).unwrap().text.as_deref(),
            Some("before")
        );
        assert_eq!(tree.node(key(1)).unwrap().text.as_deref(), Some("after"));
    }
}
