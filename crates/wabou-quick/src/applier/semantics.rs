//! Semantic-tree projection kept out of the core protocol/layout applier.

use super::*;

fn attribute(declared: &Declared, atoms: &AtomPool, wanted: &str) -> Option<Arc<str>> {
    declared
        .attrs
        .iter()
        .find_map(|(name, value)| (atoms.resolve(*name) == Some(wanted)).then(|| value.clone()))
}

fn semantic_role(tag: &str, declared: &Declared, atoms: &AtomPool) -> SemanticRole {
    match attribute(declared, atoms, "role").as_deref().unwrap_or(tag) {
        "button" => SemanticRole::Button,
        "textbox" | "input" | "textarea" | "password-input" | "code-editor" => {
            SemanticRole::TextInput
        }
        "img" | "image" => SemanticRole::Image,
        "link" | "a" => SemanticRole::Link,
        "dialog" | "alertdialog" => SemanticRole::Dialog,
        "alert" => SemanticRole::Alert,
        "status" => SemanticRole::Status,
        "checkbox" => SemanticRole::CheckBox,
        "radio" => SemanticRole::RadioButton,
        "switch" => SemanticRole::Switch,
        "combobox" => SemanticRole::ComboBox,
        "listbox" => SemanticRole::ListBox,
        "option" => SemanticRole::Option,
        "text" | "#text" | "label" => SemanticRole::Label,
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
                        | SemanticRole::Generic
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
) -> Vec<u64> {
    store
        .children
        .get(&node)
        .into_iter()
        .flatten()
        .filter(|child| present.contains(child) && !hidden.contains(child))
        .filter_map(|child| store.node_to_solid.get(child).copied())
        .map(u64::from)
        .collect()
}

pub(super) fn rebuild(applier: &mut Applier, placed: &[PlacedNode]) {
    let present: HashSet<_> = placed.iter().map(|node| node.node_id).collect();
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
    let modal_node = placed
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
    let modal_root = modal_node
        .and_then(|node| applier.node_store.solid_id_for_node(node))
        .map(u64::from);
    let mut nodes = Vec::with_capacity(placed.len().saturating_sub(1));
    for placed_node in placed {
        if placed_node.node_id == applier.node_store.root || hidden.contains(&placed_node.node_id) {
            continue;
        }
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
        let label = attribute(declared, &atoms, "aria-label")
            .or_else(|| attribute(declared, &atoms, "alt"))
            .map(|value| value.to_string())
            .or_else(|| placed_node.paint.text.as_deref().map(str::to_owned));
        let children =
            semantic_children(&applier.node_store, placed_node.node_id, &present, &hidden);
        let bounds = transformed_bounds(placed_node.rect, semantic_transforms.get(&solid_id));
        nodes.push(SemanticNode {
            id: u64::from(solid_id),
            role: semantic_role(tag, declared, &atoms),
            label,
            bounds,
            children,
            disabled: attribute(declared, &atoms, "disabled").is_some()
                || attribute(declared, &atoms, "aria-disabled").as_deref() == Some("true"),
        });
    }
    infer_descendant_labels(&mut nodes);
    let root_children = semantic_children(
        &applier.node_store,
        applier.node_store.root,
        &present,
        &hidden,
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
