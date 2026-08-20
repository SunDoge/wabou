//! Semantic-tree projection kept out of the core protocol/layout applier.

use super::*;

fn semantic_bool(value: Option<Arc<str>>) -> Option<bool> {
    match value.as_deref() {
        Some("true") | Some("") => Some(true),
        Some("false") => Some(false),
        _ => None,
    }
}

fn semantic_toggle(value: Option<Arc<str>>) -> Option<SemanticToggleState> {
    match value.as_deref() {
        Some("true") | Some("") => Some(SemanticToggleState::On),
        Some("false") => Some(SemanticToggleState::Off),
        Some("mixed") => Some(SemanticToggleState::Mixed),
        _ => None,
    }
}

fn semantic_current(value: Option<Arc<str>>) -> Option<SemanticCurrent> {
    match value.as_deref() {
        Some("true") | Some("") => Some(SemanticCurrent::True),
        Some("page") => Some(SemanticCurrent::Page),
        Some("step") => Some(SemanticCurrent::Step),
        Some("location") => Some(SemanticCurrent::Location),
        Some("date") => Some(SemanticCurrent::Date),
        Some("time") => Some(SemanticCurrent::Time),
        _ => None,
    }
}

fn semantic_popup(value: Option<Arc<str>>) -> Option<SemanticPopup> {
    match value.as_deref() {
        Some("menu") => Some(SemanticPopup::Menu),
        Some("listbox") => Some(SemanticPopup::ListBox),
        Some("tree") => Some(SemanticPopup::Tree),
        Some("grid") => Some(SemanticPopup::Grid),
        Some("dialog") => Some(SemanticPopup::Dialog),
        _ => None,
    }
}

fn transformed_bounds(rect: [f32; 4], transform: Option<&Affine>) -> [f32; 4] {
    let Some(transform) = transform else {
        return rect;
    };
    let [x0, y0, x1, y1] = rect.map(f64::from);
    let points = [
        *transform * Point::new(x0, y0),
        *transform * Point::new(x1, y0),
        *transform * Point::new(x0, y1),
        *transform * Point::new(x1, y1),
    ];
    let axis_bounds = |axis: fn(&Point) -> f64, initial: f64, fold: fn(f64, f64) -> f64| {
        points.iter().map(axis).fold(initial, fold) as f32
    };
    [
        axis_bounds(|point| point.x, f64::INFINITY, f64::min),
        axis_bounds(|point| point.y, f64::INFINITY, f64::min),
        axis_bounds(|point| point.x, f64::NEG_INFINITY, f64::max),
        axis_bounds(|point| point.y, f64::NEG_INFINITY, f64::max),
    ]
}

fn infer_descendant_labels(nodes: &mut [SemanticNode]) {
    let indices = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id, index))
        .collect::<HashMap<_, _>>();
    fn collect(
        id: u64,
        nodes: &[SemanticNode],
        indices: &HashMap<u64, usize>,
        output: &mut Vec<String>,
    ) {
        let Some(node) = indices.get(&id).and_then(|index| nodes.get(*index)) else {
            return;
        };
        if (matches!(node.role, SemanticRole::Label)
            || (matches!(node.role, SemanticRole::Generic) && node.children.is_empty()))
            && let Some(label) = node.label.as_deref()
            && !label.trim().is_empty()
        {
            output.push(label.trim().to_owned());
            return;
        }
        for child in &node.children {
            collect(*child, nodes, indices, output);
        }
    }

    let inferred = nodes
        .iter()
        .map(|node| {
            if node.label.is_some()
                || !matches!(
                    node.role,
                    SemanticRole::Button
                        | SemanticRole::Link
                        | SemanticRole::Dialog
                        | SemanticRole::AlertDialog
                        | SemanticRole::ComboBox
                        | SemanticRole::Option
                        | SemanticRole::MenuItem
                        | SemanticRole::TreeItem
                        | SemanticRole::Row
                        | SemanticRole::Cell
                        | SemanticRole::ColumnHeader
                        | SemanticRole::RowHeader
                        | SemanticRole::Generic
                        | SemanticRole::Tab
                )
            {
                return None;
            }
            let mut parts = Vec::new();
            for child in &node.children {
                collect(*child, nodes, &indices, &mut parts);
            }
            (!parts.is_empty()).then(|| parts.join(" "))
        })
        .collect::<Vec<_>>();
    for (node, inferred) in nodes.iter_mut().zip(inferred) {
        if node.label.is_none() {
            node.label = inferred;
        }
    }

    let status_values = nodes
        .iter()
        .map(|node| {
            if node.value.is_some() || node.role != SemanticRole::Status {
                return None;
            }
            let mut parts = Vec::new();
            for child in &node.children {
                collect(*child, nodes, &indices, &mut parts);
            }
            (!parts.is_empty()).then(|| parts.join(" "))
        })
        .collect::<Vec<_>>();
    for (node, value) in nodes.iter_mut().zip(status_values) {
        if node.value.is_none() {
            node.value = value;
        }
    }
}

fn semantic_focus(
    applier: &Applier,
    modal_root: Option<u64>,
    modal_node: Option<NodeId>,
) -> Option<u64> {
    let focused = applier.interaction.input.focused_target.map(u64::from);
    let (Some(modal), Some(modal_node)) = (modal_root, modal_node) else {
        return focused;
    };
    let inside_modal = |solid: u64| {
        applier
            .document
            .node_store
            .solid_to_node
            .get(&NodeKey::from_ffi(solid))
            .is_some_and(|node| {
                applier
                    .document
                    .node_store
                    .is_logical_descendant(*node, modal_node)
            })
    };
    focused
        .filter(|focused| inside_modal(*focused))
        .or(Some(modal))
}

fn semantic_children(
    store: &NodeStore,
    node: NodeId,
    present: &HashSet<NodeId>,
    hidden: &HashSet<NodeId>,
    presentational: &HashSet<NodeId>,
) -> Vec<u64> {
    fn append(
        store: &NodeStore,
        parent: NodeId,
        present: &HashSet<NodeId>,
        hidden: &HashSet<NodeId>,
        presentational: &HashSet<NodeId>,
        output: &mut Vec<u64>,
    ) {
        for child in store.children.get(&parent).into_iter().flatten() {
            if hidden.contains(child) {
                continue;
            }
            if presentational.contains(child) {
                append(store, *child, present, hidden, presentational, output);
            } else if present.contains(child)
                && let Some(solid) = store.node_to_solid.get(child)
            {
                output.push(u64::from(*solid));
            }
        }
    }

    let mut children = Vec::new();
    append(store, node, present, hidden, presentational, &mut children);
    children
}

fn semantic_source_order(
    store: &NodeStore,
    present: &HashSet<NodeId>,
    hidden: &HashSet<NodeId>,
    presentational: &HashSet<NodeId>,
) -> Vec<NodeId> {
    let mut ordered = Vec::with_capacity(present.len().saturating_sub(1));
    let mut stack = store
        .children
        .get(&store.root)
        .into_iter()
        .flatten()
        .rev()
        .copied()
        .collect::<Vec<_>>();
    while let Some(node) = stack.pop() {
        if hidden.contains(&node) {
            continue;
        }
        if present.contains(&node) && !presentational.contains(&node) {
            ordered.push(node);
        }
        if let Some(children) = store.children.get(&node) {
            stack.extend(children.iter().rev().copied());
        }
    }
    ordered
}

pub(super) fn rebuild(applier: &mut Applier, placed: &[PlacedNode]) {
    let present: HashSet<_> = placed.iter().map(|node| node.node_id).collect();
    let placed_by_node = placed
        .iter()
        .map(|placed| (placed.node_id, placed))
        .collect::<HashMap<_, _>>();
    let atoms = applier.document.atoms.borrow();
    let semantic_transforms: HashMap<_, _> = applier
        .interaction
        .input
        .hit_items
        .iter()
        .filter_map(|item| match item {
            HitItem::Content(node) => Some((node.solid_id, node.transform)),
            HitItem::Scrollbar(_) => None,
        })
        .collect();
    let hidden: HashSet<_> = placed
        .iter()
        .filter(|node| {
            subtree_blocks_interaction(&applier.document.node_store, node.node_id)
                || subtree_has_attribute(
                    &applier.document.node_store,
                    &atoms,
                    node.node_id,
                    "aria-hidden",
                    Some("true"),
                )
        })
        .map(|node| node.node_id)
        .collect();
    let presentational = applier
        .document
        .node_store
        .declared
        .iter()
        .filter_map(|(node, declared)| {
            matches!(
                declared.attribute(&atoms, "role").as_deref(),
                Some("presentation" | "none")
            )
            .then_some(*node)
        })
        .collect::<HashSet<_>>();
    let modal_container = placed
        .iter()
        .rev()
        .find(|node| {
            !hidden.contains(&node.node_id)
                && node.paint.overlay_plane == OverlayPlane::Modal
                && applier
                    .document
                    .node_store
                    .declared
                    .get(&node.node_id)
                    .is_some_and(|declared| {
                        declared.attribute(&atoms, "aria-modal").as_deref() == Some("true")
                    })
        })
        .map(|node| node.node_id);
    let modal_node = modal_container.and_then(|container| {
        if !presentational.contains(&container) {
            return Some(container);
        }
        placed.iter().rev().find_map(|node| {
            if hidden.contains(&node.node_id)
                || presentational.contains(&node.node_id)
                || !applier
                    .document
                    .node_store
                    .is_logical_descendant(node.node_id, container)
            {
                return None;
            }
            let declared = applier.document.node_store.declared.get(&node.node_id)?;
            matches!(
                declared.attribute(&atoms, "role").as_deref(),
                Some("dialog")
            )
            .then_some(node.node_id)
        })
    });
    let modal_root = modal_node
        .and_then(|node| applier.document.node_store.solid_id_for_node(node))
        .map(u64::from);
    let source_order = semantic_source_order(
        &applier.document.node_store,
        &present,
        &hidden,
        &presentational,
    );
    // Authored semantic IDs are strings while the accessibility tree uses
    // stable Solid node ids. Resolve only currently exposed targets, and use
    // the first source-order occurrence when invalid duplicate IDs exist.
    let mut semantic_ids = HashMap::<Arc<str>, u64>::new();
    for node_id in &source_order {
        let Some(declared) = applier.document.node_store.declared.get(node_id) else {
            continue;
        };
        let Some(id) = declared.attribute(&atoms, "id") else {
            continue;
        };
        let Some(solid) = applier.document.node_store.node_to_solid.get(node_id) else {
            continue;
        };
        semantic_ids.entry(id).or_insert_with(|| u64::from(*solid));
    }
    let mut nodes = Vec::with_capacity(source_order.len());
    for node_id in source_order {
        let Some(placed_node) = placed_by_node.get(&node_id).copied() else {
            continue;
        };
        let Some(&solid_id) = applier
            .document
            .node_store
            .node_to_solid
            .get(&placed_node.node_id)
        else {
            continue;
        };
        let Some(declared) = applier
            .document
            .node_store
            .declared
            .get(&placed_node.node_id)
        else {
            continue;
        };
        let widget_semantics = applier
            .document
            .widget_manager
            .widgets
            .get(&placed_node.node_id)
            .map(|widget| widget.accessibility())
            .unwrap_or_default();
        let label = declared
            .attribute(&atoms, "aria-label")
            .map(|value| value.to_string())
            .or_else(|| placed_node.paint.text.as_deref().map(str::to_owned));
        let explicit_role = declared.attribute(&atoms, "role");
        let role = if explicit_role.is_some() {
            SemanticRole::from_name(explicit_role.as_deref().unwrap_or_default())
                .unwrap_or(SemanticRole::Generic)
        } else {
            widget_semantics.role.unwrap_or(SemanticRole::Generic)
        };
        let value = (!widget_semantics.value_is_sensitive)
            .then(|| {
                declared
                    .attribute(&atoms, "aria-valuetext")
                    .map(|value| value.to_string())
                    .or(widget_semantics.value)
                    .or_else(|| {
                        (role == SemanticRole::Status)
                            .then(|| placed_node.paint.text.as_deref().map(str::to_owned))
                            .flatten()
                    })
            })
            .flatten();
        let numeric_attribute = |name| {
            declared
                .attribute(&atoms, name)
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite())
        };
        let children = semantic_children(
            &applier.document.node_store,
            placed_node.node_id,
            &present,
            &hidden,
            &presentational,
        );
        let controls = declared
            .attribute(&atoms, "aria-controls")
            .map(|value| {
                value
                    .split_whitespace()
                    .filter_map(|id| semantic_ids.get(id).copied())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let active_descendant = declared
            .attribute(&atoms, "aria-activedescendant")
            .and_then(|value| semantic_ids.get(value.trim()).copied());
        let bounds = transformed_bounds(placed_node.rect, semantic_transforms.get(&solid_id));
        nodes.push(SemanticNode {
            id: u64::from(solid_id),
            role,
            label: label.or(widget_semantics.label),
            value,
            numeric_value: numeric_attribute("aria-valuenow"),
            min_numeric_value: numeric_attribute("aria-valuemin"),
            max_numeric_value: numeric_attribute("aria-valuemax"),
            bounds,
            children,
            controls,
            active_descendant,
            disabled: declared.attribute(&atoms, "aria-disabled").as_deref() == Some("true")
                || widget_semantics.disabled.unwrap_or(false),
            states: SemanticStates {
                checked: semantic_toggle(declared.attribute(&atoms, "aria-checked")),
                pressed: semantic_toggle(declared.attribute(&atoms, "aria-pressed")),
                selected: semantic_bool(declared.attribute(&atoms, "aria-selected")),
                expanded: semantic_bool(declared.attribute(&atoms, "aria-expanded")),
                current: semantic_current(declared.attribute(&atoms, "aria-current")),
                popup: semantic_popup(declared.attribute(&atoms, "aria-haspopup")),
                modal: semantic_bool(declared.attribute(&atoms, "aria-modal")),
            },
        });
    }
    infer_descendant_labels(&mut nodes);
    let root_children = semantic_children(
        &applier.document.node_store,
        applier.document.node_store.root,
        &present,
        &hidden,
        &presentational,
    );
    let focus = semantic_focus(applier, modal_root, modal_node);
    applier.frame.projections.semantic_snapshot = Arc::new(SemanticSnapshot {
        revision: applier
            .frame
            .projections
            .semantic_snapshot
            .revision
            .wrapping_add(1)
            .max(1),
        nodes,
        root_children,
        focus,
        modal_root,
    });
}

impl Applier {
    pub(super) fn rebuild_semantic_snapshot(&mut self, placed: &[PlacedNode]) {
        rebuild(self, placed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_roles_map_to_native_semantics() {
        for (name, expected) in [
            ("menu", SemanticRole::Menu),
            ("menuitem", SemanticRole::MenuItem),
            ("tree", SemanticRole::Tree),
            ("treeitem", SemanticRole::TreeItem),
            ("grid", SemanticRole::Grid),
        ] {
            assert_eq!(SemanticRole::from_name(name), Some(expected));
        }
        assert_eq!(SemanticRole::from_name("unknown"), None);
        assert_eq!(
            SemanticRole::from_name("alertdialog"),
            Some(SemanticRole::AlertDialog)
        );
    }

    #[test]
    fn popup_kind_requires_an_explicit_supported_value() {
        assert_eq!(
            semantic_popup(Some(Arc::from("menu"))),
            Some(SemanticPopup::Menu)
        );
        assert_eq!(semantic_popup(Some(Arc::from("true"))), None);
        assert_eq!(semantic_popup(Some(Arc::from(""))), None);
    }
}
