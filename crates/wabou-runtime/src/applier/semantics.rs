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

fn attribute(declared: &Declared, atoms: &AtomPool, wanted: &str) -> Option<Arc<str>> {
    declared
        .attrs
        .iter()
        .find_map(|(name, value)| (atoms.resolve(*name) == Some(wanted)).then(|| value.clone()))
}

fn semantic_role(role: &str) -> SemanticRole {
    match role {
        "button" => SemanticRole::Button,
        "group" => SemanticRole::Group,
        "textbox" => SemanticRole::TextInput,
        "img" => SemanticRole::Image,
        "radiogroup" => SemanticRole::RadioGroup,
        "link" => SemanticRole::Link,
        "dialog" | "alertdialog" => SemanticRole::Dialog,
        "alert" => SemanticRole::Alert,
        "status" => SemanticRole::Status,
        "checkbox" => SemanticRole::CheckBox,
        "radio" => SemanticRole::RadioButton,
        "switch" => SemanticRole::Switch,
        "combobox" => SemanticRole::ComboBox,
        "listbox" => SemanticRole::ListBox,
        "option" => SemanticRole::Option,
        "menu" => SemanticRole::Menu,
        "menuitem" => SemanticRole::MenuItem,
        "tree" => SemanticRole::Tree,
        "treeitem" => SemanticRole::TreeItem,
        "table" => SemanticRole::Table,
        "row" => SemanticRole::Row,
        "cell" => SemanticRole::Cell,
        "gridcell" => SemanticRole::GridCell,
        "columnheader" => SemanticRole::ColumnHeader,
        "rowheader" => SemanticRole::RowHeader,
        "slider" => SemanticRole::Slider,
        "progressbar" => SemanticRole::ProgressBar,
        "tablist" => SemanticRole::TabList,
        "tab" => SemanticRole::Tab,
        "tabpanel" => SemanticRole::TabPanel,
        "grid" => SemanticRole::Grid,
        "heading" => SemanticRole::Heading,
        "label" => SemanticRole::Label,
        _ => SemanticRole::Generic,
    }
}

fn primitive_semantic_role(tag: &str) -> SemanticRole {
    match tag {
        "text" | "#text" => SemanticRole::Label,
        "img" | "svg" => SemanticRole::Image,
        _ => SemanticRole::Generic,
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
        if matches!(node.role, SemanticRole::Label)
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
    nodes: &[SemanticNode],
    modal_root: Option<u64>,
    modal_node: Option<NodeId>,
) -> Option<u64> {
    let focused = applier.input.focused_target.map(u64::from);
    let (Some(modal), Some(modal_node)) = (modal_root, modal_node) else {
        return focused;
    };
    let inside_modal = |solid: u64| {
        applier
            .node_store
            .solid_to_node
            .get(&(solid as u32))
            .is_some_and(|node| applier.node_store.is_logical_descendant(*node, modal_node))
    };
    let fallback = nodes
        .iter()
        .find(|node| {
            inside_modal(node.id)
                && matches!(
                    node.role,
                    SemanticRole::Dialog
                        | SemanticRole::Button
                        | SemanticRole::TextInput
                        | SemanticRole::Link
                        | SemanticRole::CheckBox
                        | SemanticRole::RadioButton
                        | SemanticRole::Switch
                        | SemanticRole::ComboBox
                        | SemanticRole::ListBox
                        | SemanticRole::Option
                        | SemanticRole::MenuItem
                        | SemanticRole::TreeItem
                        | SemanticRole::Slider
                        | SemanticRole::Tab
                )
        })
        .map(|node| node.id)
        .unwrap_or(modal);
    focused
        .filter(|focused| inside_modal(*focused))
        .or(Some(fallback))
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
    let atoms = applier.atoms.borrow();
    let semantic_transforms: HashMap<_, _> = applier
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
            subtree_has_attribute(&applier.node_store, &atoms, node.node_id, "inert", None)
                || subtree_has_attribute(
                    &applier.node_store,
                    &atoms,
                    node.node_id,
                    "aria-hidden",
                    Some("true"),
                )
        })
        .map(|node| node.node_id)
        .collect();
    let presentational = applier
        .node_store
        .declared
        .iter()
        .filter_map(|(node, declared)| {
            matches!(
                attribute(declared, &atoms, "role").as_deref(),
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
                    .node_store
                    .declared
                    .get(&node.node_id)
                    .is_some_and(|declared| {
                        attribute(declared, &atoms, "aria-modal").as_deref() == Some("true")
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
                    .node_store
                    .is_logical_descendant(node.node_id, container)
            {
                return None;
            }
            let declared = applier.node_store.declared.get(&node.node_id)?;
            matches!(
                attribute(declared, &atoms, "role").as_deref(),
                Some("dialog" | "alertdialog")
            )
            .then_some(node.node_id)
        })
    });
    let modal_root = modal_node
        .and_then(|node| applier.node_store.solid_id_for_node(node))
        .map(u64::from);
    let source_order =
        semantic_source_order(&applier.node_store, &present, &hidden, &presentational);
    // HTML IDs are authored strings while the accessibility tree uses stable
    // Solid node ids. Resolve only currently exposed semantic targets, and use
    // the first source-order occurrence when invalid duplicate IDs exist.
    let mut semantic_ids = HashMap::<Arc<str>, u64>::new();
    for node_id in &source_order {
        let Some(declared) = applier.node_store.declared.get(node_id) else {
            continue;
        };
        let Some(id) = attribute(declared, &atoms, "id") else {
            continue;
        };
        let Some(solid) = applier.node_store.node_to_solid.get(node_id) else {
            continue;
        };
        semantic_ids.entry(id).or_insert_with(|| u64::from(*solid));
    }
    let mut nodes = Vec::with_capacity(source_order.len());
    for node_id in source_order {
        let Some(placed_node) = placed_by_node.get(&node_id).copied() else {
            continue;
        };
        let Some(&solid_id) = applier.node_store.node_to_solid.get(&placed_node.node_id) else {
            continue;
        };
        let Some(declared) = applier.node_store.declared.get(&placed_node.node_id) else {
            continue;
        };
        let tag = declared
            .tag
            .and_then(|tag| atoms.resolve(tag))
            .unwrap_or("view");
        let widget_semantics = applier
            .widget_manager
            .widgets
            .get(&placed_node.node_id)
            .map(|widget| widget.accessibility())
            .unwrap_or_default();
        let label = attribute(declared, &atoms, "aria-label")
            .or_else(|| attribute(declared, &atoms, "alt"))
            .map(|value| value.to_string())
            .or_else(|| placed_node.paint.text.as_deref().map(str::to_owned));
        let is_secret = tag == "password-input"
            || attribute(declared, &atoms, "type").as_deref() == Some("password");
        let explicit_role = attribute(declared, &atoms, "role");
        let value = (!is_secret)
            .then(|| {
                attribute(declared, &atoms, "aria-valuetext")
                    .or_else(|| attribute(declared, &atoms, "value"))
                    .map(|value| value.to_string())
                    .or(widget_semantics.value)
                    .or_else(|| {
                        explicit_role
                            .as_deref()
                            .filter(|role| *role != "label")
                            .and_then(|_| placed_node.paint.text.as_deref().map(str::to_owned))
                    })
            })
            .flatten();
        let numeric_attribute = |name| {
            attribute(declared, &atoms, name)
                .and_then(|value| value.parse::<f64>().ok())
                .filter(|value| value.is_finite())
        };
        let children = semantic_children(
            &applier.node_store,
            placed_node.node_id,
            &present,
            &hidden,
            &presentational,
        );
        let controls = attribute(declared, &atoms, "aria-controls")
            .map(|value| {
                value
                    .split_whitespace()
                    .filter_map(|id| semantic_ids.get(id).copied())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let active_descendant = attribute(declared, &atoms, "aria-activedescendant")
            .and_then(|value| semantic_ids.get(value.trim()).copied());
        let bounds = transformed_bounds(placed_node.rect, semantic_transforms.get(&solid_id));
        nodes.push(SemanticNode {
            id: u64::from(solid_id),
            role: if explicit_role.is_some() {
                semantic_role(explicit_role.as_deref().unwrap_or_default())
            } else if declared.text.is_some() {
                SemanticRole::Label
            } else {
                widget_semantics
                    .role
                    .unwrap_or_else(|| primitive_semantic_role(tag))
            },
            label: label.or(widget_semantics.label),
            value,
            numeric_value: numeric_attribute("aria-valuenow"),
            min_numeric_value: numeric_attribute("aria-valuemin"),
            max_numeric_value: numeric_attribute("aria-valuemax"),
            bounds,
            children,
            controls,
            active_descendant,
            disabled: attribute(declared, &atoms, "disabled").is_some()
                || attribute(declared, &atoms, "aria-disabled").as_deref() == Some("true")
                || widget_semantics.disabled.unwrap_or(false),
            states: SemanticStates {
                checked: semantic_toggle(attribute(declared, &atoms, "aria-checked")),
                pressed: semantic_toggle(attribute(declared, &atoms, "aria-pressed")),
                selected: semantic_bool(attribute(declared, &atoms, "aria-selected")),
                expanded: semantic_bool(attribute(declared, &atoms, "aria-expanded")),
            },
        });
    }
    infer_descendant_labels(&mut nodes);
    let root_children = semantic_children(
        &applier.node_store,
        applier.node_store.root,
        &present,
        &hidden,
        &presentational,
    );
    let focus = semantic_focus(applier, &nodes, modal_root, modal_node);
    applier.projections.semantic_snapshot = Arc::new(SemanticSnapshot {
        revision: applier
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
    fn explicit_roles_map_to_native_semantics_without_tag_inference() {
        for (name, expected) in [
            ("menu", SemanticRole::Menu),
            ("menuitem", SemanticRole::MenuItem),
            ("tree", SemanticRole::Tree),
            ("treeitem", SemanticRole::TreeItem),
            ("grid", SemanticRole::Grid),
        ] {
            assert_eq!(semantic_role(name), expected);
        }
        assert_eq!(primitive_semantic_role("checkbox"), SemanticRole::Generic);
        assert_eq!(primitive_semantic_role("text"), SemanticRole::Label);
        assert_eq!(primitive_semantic_role("img"), SemanticRole::Image);
    }
}
