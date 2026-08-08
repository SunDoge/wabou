//! Semantic-tree projection kept out of the core protocol/layout applier.

use super::*;

pub(super) fn rebuild(applier: &mut Applier, placed: &[PlacedNode]) {
    let present: HashSet<_> = placed.iter().map(|node| node.node_id).collect();
    let atoms = applier.atoms.borrow();
    let attribute = |declared: &Declared, wanted: &str| {
        declared
            .attrs
            .iter()
            .find_map(|(name, value)| (atoms.resolve(*name) == Some(wanted)).then(|| value.clone()))
    };
    let semantic_transforms: HashMap<_, _> = applier
        .input
        .hit_items
        .iter()
        .filter_map(|item| match item {
            HitItem::Content(node) => Some((node.solid_id, node.transform)),
            HitItem::Scrollbar(_) => None,
        })
        .collect();
    let modal_node = placed
        .iter()
        .rev()
        .find(|node| {
            node.paint.overlay_plane == OverlayPlane::Modal
                && applier
                    .node_store
                    .declared
                    .get(&node.node_id)
                    .is_some_and(|declared| {
                        attribute(declared, "aria-modal").as_deref() == Some("true")
                    })
        })
        .map(|node| node.node_id);
    let modal_root = modal_node
        .and_then(|node| applier.node_store.solid_id_for_node(node))
        .map(u64::from);
    let role_for = |tag: &str, declared: &Declared| {
        let role = attribute(declared, "role");
        match role.as_deref().unwrap_or(tag) {
            "button" => SemanticRole::Button,
            "textbox" | "input" | "textarea" => SemanticRole::TextInput,
            "img" | "image" => SemanticRole::Image,
            "link" | "a" => SemanticRole::Link,
            "dialog" | "alertdialog" => SemanticRole::Dialog,
            "text" | "#text" | "label" => SemanticRole::Label,
            _ => SemanticRole::Generic,
        }
    };
    let mut nodes = Vec::with_capacity(placed.len().saturating_sub(1));
    for placed_node in placed {
        if placed_node.node_id == applier.node_store.root {
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
        let label = attribute(declared, "aria-label")
            .or_else(|| attribute(declared, "alt"))
            .map(|value| value.to_string())
            .or_else(|| placed_node.paint.text.as_deref().map(str::to_owned));
        let children = applier
            .node_store
            .children
            .get(&placed_node.node_id)
            .into_iter()
            .flatten()
            .filter(|child| present.contains(child))
            .filter_map(|child| applier.node_store.node_to_solid.get(child).copied())
            .map(u64::from)
            .collect();
        let bounds = semantic_transforms
            .get(&solid_id)
            .map_or(placed_node.rect, |transform| {
                let [x0, y0, x1, y1] = placed_node.rect.map(f64::from);
                let points = [
                    *transform * Point::new(x0, y0),
                    *transform * Point::new(x1, y0),
                    *transform * Point::new(x0, y1),
                    *transform * Point::new(x1, y1),
                ];
                [
                    points
                        .iter()
                        .map(|point| point.x)
                        .fold(f64::INFINITY, f64::min) as f32,
                    points
                        .iter()
                        .map(|point| point.y)
                        .fold(f64::INFINITY, f64::min) as f32,
                    points
                        .iter()
                        .map(|point| point.x)
                        .fold(f64::NEG_INFINITY, f64::max) as f32,
                    points
                        .iter()
                        .map(|point| point.y)
                        .fold(f64::NEG_INFINITY, f64::max) as f32,
                ]
            });
        nodes.push(SemanticNode {
            id: u64::from(solid_id),
            role: role_for(tag, declared),
            label,
            bounds,
            children,
            disabled: attribute(declared, "disabled").is_some()
                || attribute(declared, "aria-disabled").as_deref() == Some("true"),
        });
    }
    let root_children = applier
        .node_store
        .children
        .get(&applier.node_store.root)
        .into_iter()
        .flatten()
        .filter(|child| present.contains(child))
        .filter_map(|child| applier.node_store.node_to_solid.get(child).copied())
        .map(u64::from)
        .collect();
    let focused = applier.input.focused_target.map(u64::from);
    let focus = if let (Some(modal), Some(modal_node)) = (modal_root, modal_node) {
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
                    )
            })
            .map(|node| node.id)
            .unwrap_or(modal);
        focused
            .filter(|focused| inside_modal(*focused))
            .or(Some(fallback))
    } else {
        focused
    };
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
