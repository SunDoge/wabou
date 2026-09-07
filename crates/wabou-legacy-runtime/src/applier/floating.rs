//! Backend-neutral floating-surface placement for the legacy/Hybrid tree.

use super::*;
use wabou_shell_api::{FloatingAnchor, FloatingPosition, resolve_floating_position};

const FLOATING_POSITION_ATTRIBUTE: &str = "__wabou_floating_position";

fn translate_rect(rect: &mut [f32; 4], delta: [f32; 2]) {
    rect[0] += delta[0];
    rect[1] += delta[1];
    rect[2] += delta[0];
    rect[3] += delta[1];
}

fn translate_subtree(placed: &mut [PlacedNode], root_index: usize, origin: [f32; 2]) {
    let root_depth = placed[root_index].depth;
    let delta = [
        origin[0] - placed[root_index].rect[0],
        origin[1] - placed[root_index].rect[1],
    ];
    if delta == [0.0, 0.0] {
        return;
    }
    for (_, node) in placed[root_index..]
        .iter_mut()
        .enumerate()
        .take_while(|(offset, node)| *offset == 0 || node.depth > root_depth)
    {
        translate_rect(&mut node.rect, delta);
        node.content_origin[0] += delta[0];
        node.content_origin[1] += delta[1];
        translate_rect(&mut node.scroll.port, delta);
        if let Some(clip) = &mut node.own_clip {
            translate_rect(clip, delta);
        }
        // A clip established inside the floating subtree moves with it. An
        // inherited viewport/overlay clip remains in window coordinates.
        if node.clip_depth.is_some_and(|depth| depth >= root_depth)
            && let Some(clip) = &mut node.clip
        {
            translate_rect(clip, delta);
        }
    }
}

fn hide_subtree(placed: &mut [PlacedNode], root_index: usize) {
    let root_depth = placed[root_index].depth;
    for (_, node) in placed[root_index..]
        .iter_mut()
        .enumerate()
        .take_while(|(offset, node)| *offset == 0 || node.depth > root_depth)
    {
        node.paint.pointer_events = false;
    }
    placed[root_index].paint.opacity = 0.0;
}

impl LegacyRuntimeController {
    /// Resolve private floating attributes after Taffy has measured both the
    /// anchor and surface. Returns whether floating geometry participates in
    /// the current frame's retained projections.
    pub(super) fn position_floating_nodes(
        &self,
        placed: &mut [PlacedNode],
        viewport_size: [f32; 2],
    ) -> bool {
        let indices: HashMap<NodeId, usize> = placed
            .iter()
            .enumerate()
            .map(|(index, node)| (node.node_id, index))
            .collect();
        let atoms = self.document.atoms.borrow();
        let mut found = false;

        for root_index in 0..placed.len() {
            let node_id = placed[root_index].node_id;
            let Some(position) = self
                .document
                .node_store
                .declared
                .get(&node_id)
                .and_then(|declared| declared.attribute(&atoms, FLOATING_POSITION_ATTRIBUTE))
                .and_then(|value| FloatingPosition::parse(&value))
            else {
                continue;
            };
            found = true;

            let anchor = match position.anchor {
                FloatingAnchor::Node { id } => self
                    .document
                    .node_store
                    .solid_to_node
                    .get(&NodeKey::new(id.lo, id.hi))
                    .and_then(|node| indices.get(node))
                    .map(|index| placed[*index].rect),
                FloatingAnchor::Point { x, y } => Some([x, y, x, y]),
            };
            let Some(anchor) = anchor else {
                // A replacement/HMR frame can briefly outlive its trigger.
                // Match GPUI by keeping the surface non-interactive and
                // invisible instead of flashing it at the viewport origin.
                hide_subtree(placed, root_index);
                continue;
            };
            let popup_size = [
                placed[root_index].rect[2] - placed[root_index].rect[0],
                placed[root_index].rect[3] - placed[root_index].rect[1],
            ];
            let resolved = resolve_floating_position(position, anchor, popup_size, viewport_size);
            translate_subtree(placed, root_index, resolved.origin);
        }

        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn placed(node_id: NodeId, depth: usize, rect: [f32; 4]) -> PlacedNode {
        PlacedNode {
            node_id,
            parent_node_id: None,
            depth,
            rect,
            content_origin: [rect[0], rect[1]],
            content_size: [rect[2] - rect[0], rect[3] - rect[1]],
            clip: None,
            clip_radius: 0.0,
            clip_radii: legacy_shell::style::CornerRadii::default(),
            clip_depth: None,
            own_clip: None,
            own_clip_radius: 0.0,
            own_clip_radii: legacy_shell::style::CornerRadii::default(),
            border_widths: [0.0; 4],
            scroll: layout::ScrollMetrics {
                port: rect,
                ..Default::default()
            },
            paint: Paint::default(),
        }
    }

    fn create_node(applier: &mut Applier, id: NodeKey, tag: Atom) -> NodeId {
        applier.apply_op(&Op::CreateElement { id, tag });
        applier.document.node_store.solid_to_node[&id]
    }

    #[test]
    fn positions_a_floating_subtree_without_moving_its_sibling() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (view, floating) = {
            let mut atoms = applier.document.atoms.borrow_mut();
            (
                atoms.intern("view"),
                atoms.intern(FLOATING_POSITION_ATTRIBUTE),
            )
        };
        let anchor = create_node(&mut applier, NodeKey::new(2, 1), view);
        let popup = create_node(&mut applier, NodeKey::new(3, 1), view);
        let child = create_node(&mut applier, NodeKey::new(4, 1), view);
        let sibling = create_node(&mut applier, NodeKey::new(5, 1), view);
        applier.apply_op(&Op::SetAttribute {
            id: NodeKey::new(3, 1),
            name: floating,
            value: r#"{"anchor":{"kind":"node","id":{"lo":2,"hi":1}},"placement":"bottom-start","offset":6,"margin":8}"#,
        });

        let mut nodes = vec![
            placed(anchor, 1, [20.0, 20.0, 120.0, 52.0]),
            placed(popup, 1, [0.0, 0.0, 100.0, 50.0]),
            placed(child, 2, [10.0, 10.0, 90.0, 40.0]),
            placed(sibling, 1, [200.0, 10.0, 240.0, 40.0]),
        ];
        nodes[2].own_clip = Some([10.0, 10.0, 90.0, 40.0]);
        nodes[2].clip = Some([0.0, 0.0, 100.0, 50.0]);
        nodes[2].clip_depth = Some(1);

        assert!(applier.position_floating_nodes(&mut nodes, [800.0, 600.0]));
        assert_eq!(nodes[1].rect, [20.0, 58.0, 120.0, 108.0]);
        assert_eq!(nodes[2].rect, [30.0, 68.0, 110.0, 98.0]);
        assert_eq!(nodes[2].own_clip, Some([30.0, 68.0, 110.0, 98.0]));
        assert_eq!(nodes[2].clip, Some([20.0, 58.0, 120.0, 108.0]));
        assert_eq!(nodes[3].rect, [200.0, 10.0, 240.0, 40.0]);
    }

    #[test]
    fn hides_a_floating_subtree_when_its_anchor_generation_is_stale() {
        let js = JsRuntime::new().expect("runtime");
        let mut applier = Applier::from_runtime(js, Color::BLACK);
        let (view, floating) = {
            let mut atoms = applier.document.atoms.borrow_mut();
            (
                atoms.intern("view"),
                atoms.intern(FLOATING_POSITION_ATTRIBUTE),
            )
        };
        let popup = create_node(&mut applier, NodeKey::new(3, 1), view);
        let child = create_node(&mut applier, NodeKey::new(4, 1), view);
        applier.apply_op(&Op::SetAttribute {
            id: NodeKey::new(3, 1),
            name: floating,
            value: r#"{"anchor":{"kind":"node","id":{"lo":2,"hi":99}}}"#,
        });
        let mut nodes = vec![
            placed(popup, 1, [0.0, 0.0, 100.0, 50.0]),
            placed(child, 2, [10.0, 10.0, 90.0, 40.0]),
        ];

        assert!(applier.position_floating_nodes(&mut nodes, [800.0, 600.0]));
        assert_eq!(nodes[0].paint.opacity, 0.0);
        assert!(!nodes[0].paint.pointer_events);
        assert!(!nodes[1].paint.pointer_events);
    }
}
