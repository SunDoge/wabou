//! Run taffy layout on a retained tree and flatten the result into an ordered
//! list of absolutely-positioned boxes for the renderer.

use std::collections::HashMap;

use taffy::geometry::Size;
use taffy::{AvailableSpace, NodeId, TaffyTree, TraversePartialTree};

use crate::style::Paint;
use crate::text::TextContext;

/// A node after layout: its absolute border-box rect + resolved paint.
#[derive(Clone)]
pub struct PlacedNode {
    /// The taffy NodeId — lets the host associate Rust-side widgets with
    /// placed nodes (e.g. to call `Widget::paint` after layout).
    pub node_id: taffy::NodeId,
    pub parent_node_id: Option<taffy::NodeId>,
    pub depth: usize,
    /// Absolute border-box: (x0, y0, x1, y1).
    pub rect: [f32; 4],
    /// Absolute content-box top-left: (x, y). Text runs are anchored here.
    pub content_origin: [f32; 2],
    /// Content-box size after removing border and padding.
    pub content_size: [f32; 2],
    /// Accumulated ancestor overflow clip in absolute coordinates.
    pub clip: Option<[f32; 4]>,
    /// Radius of the accumulated clip when its nearest edge is rounded.
    pub clip_radius: f32,
    /// Depth of the nearest ancestor that established `clip`.
    pub clip_depth: Option<usize>,
    /// This node's own overflow clip, used for replaced/native widget content.
    /// Descendants receive the same geometry through `clip` on their entries.
    pub own_clip: Option<[f32; 4]>,
    pub own_clip_radius: f32,
    /// Resolved physical border widths: top, right, bottom, left.
    pub border_widths: [f32; 4],
    pub scroll: ScrollMetrics,
    pub paint: Paint,
}

/// Finite scroll geometry in logical window coordinates. Axis-specific clips
/// are intentionally separate because their unconstrained edges may be infinite.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollMetrics {
    pub port: [f32; 4],
    pub scrollable: [bool; 2],
    pub range: [f32; 2],
    pub offset: [f32; 2],
    /// Host-owned overlay visibility. Zero also disables native hit testing.
    pub opacity: f32,
    /// 0 resting, 1 hovered, 2 actively dragged.
    pub interaction: u8,
}

#[derive(Clone, Copy, Default)]
struct ClipState {
    rect: Option<[f32; 4]>,
    radius: f32,
    depth: Option<usize>,
}

/// A depth-derived traversal boundary for the flattened retained tree.
/// `Exit` is the reusable paint/hit boundary for owner-local overlays.
#[derive(Clone, Copy)]
pub enum SubtreeEvent<'a> {
    Enter(&'a PlacedNode),
    Exit(&'a PlacedNode),
}

pub struct SubtreeEvents<'a> {
    nodes: &'a [PlacedNode],
    index: usize,
    open: Vec<&'a PlacedNode>,
}

pub fn subtree_events(nodes: &[PlacedNode]) -> SubtreeEvents<'_> {
    SubtreeEvents {
        nodes,
        index: 0,
        open: Vec::new(),
    }
}

impl<'a> Iterator for SubtreeEvents<'a> {
    type Item = SubtreeEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.nodes.get(self.index)
            && self
                .open
                .last()
                .is_some_and(|open| open.depth >= next.depth)
        {
            return self.open.pop().map(SubtreeEvent::Exit);
        }
        if let Some(node) = self.nodes.get(self.index) {
            self.index += 1;
            self.open.push(node);
            return Some(SubtreeEvent::Enter(node));
        }
        self.open.pop().map(SubtreeEvent::Exit)
    }
}

pub fn compute_and_walk_with_scroll(
    tree: &mut TaffyTree<Paint>,
    root: NodeId,
    width: f32,
    height: f32,
    tcx: &mut TextContext,
    scroll_offsets: &HashMap<NodeId, [f32; 2]>,
) -> Vec<PlacedNode> {
    let available = Size {
        width: AvailableSpace::Definite(width),
        height: AvailableSpace::Definite(height),
    };
    // Measure text leaves through parley (system fonts + shaping). The closure
    // takes ownership of the `&mut TextContext` reborrow for the duration of
    // layout; the caller may reuse `tcx` afterwards.
    let _ = tree.compute_layout_with_measure(
        root,
        available,
        move |known, avail, _id, ctx: Option<&mut Paint>, _style| {
            if let Some(paint) = ctx {
                if let Some(text) = &paint.text {
                    let max_width = paint
                        .wrap_text
                        .then(|| {
                            known.width.or(match avail.width {
                                AvailableSpace::Definite(width) => Some(width),
                                AvailableSpace::MinContent | AvailableSpace::MaxContent => None,
                            })
                        })
                        .flatten();
                    let l = crate::text::layout_text_styled(
                        tcx,
                        text.clone(),
                        paint.font_size,
                        paint.font_weight,
                        paint.line_height,
                        paint.text_align,
                        crate::text::brush_for_color(paint.text_color),
                        paint.text_runs.clone(),
                        paint.font_family.as_ref(),
                        max_width,
                    );
                    return Size {
                        width: known.width.unwrap_or(l.width()),
                        height: l.height(),
                    };
                }
                if let Some([width, height]) = paint.intrinsic_size {
                    return Size {
                        width: known.width.unwrap_or(width),
                        height: known.height.unwrap_or(height),
                    };
                }
            }
            Size::ZERO
        },
    );

    flatten_with_scroll(tree, root, scroll_offsets)
}

/// Re-flatten a previously computed Taffy layout with new scroll offsets.
/// Scrolling changes paint coordinates and clips, not intrinsic layout, so the
/// interactive host can use this path without remeasuring the entire tree.
pub fn flatten_with_scroll(
    tree: &TaffyTree<Paint>,
    root: NodeId,
    scroll_offsets: &HashMap<NodeId, [f32; 2]>,
) -> Vec<PlacedNode> {
    let mut out = Vec::new();
    walk(
        tree,
        root,
        None,
        0,
        0.0,
        0.0,
        ClipState::default(),
        scroll_offsets,
        &mut out,
    );
    out
}

/// Depth-first walk accumulating absolute positions. `x0`,`y0` is the absolute
/// border-box origin of the *parent*. A node's own border-box origin is
/// `(parent_x0 + node.location.x, parent_y0 + node.location.y)`.
fn intersect(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        a[0].max(b[0]),
        a[1].max(b[1]),
        a[2].min(b[2]),
        a[3].min(b[3]),
    ]
}

#[allow(clippy::too_many_arguments)]
fn walk(
    tree: &TaffyTree<Paint>,
    node: NodeId,
    parent_node_id: Option<NodeId>,
    depth: usize,
    parent_x0: f32,
    parent_y0: f32,
    inherited_clip: ClipState,
    scroll_offsets: &HashMap<NodeId, [f32; 2]>,
    out: &mut Vec<PlacedNode>,
) {
    let layout = match tree.layout(node) {
        Ok(l) => l,
        Err(_) => return,
    };
    if let Ok(style) = tree.style(node)
        && style.display == taffy::Display::None
    {
        return;
    }

    let x0 = parent_x0 + layout.location.x;
    let y0 = parent_y0 + layout.location.y;
    let w = layout.size.width;
    let h = layout.size.height;
    let rect = [x0, y0, x0 + w, y0 + h];

    let cx = x0 + layout.border.left + layout.padding.left;
    let cy = y0 + layout.border.top + layout.padding.top;
    let content_width =
        (w - layout.border.left - layout.border.right - layout.padding.left - layout.padding.right)
            .max(0.0);
    let content_height =
        (h - layout.border.top - layout.border.bottom - layout.padding.top - layout.padding.bottom)
            .max(0.0);
    let scroll = scroll_offsets.get(&node).copied().unwrap_or([0.0, 0.0]);
    let style = tree.style(node).ok();
    let scrollable = style.map_or([false; 2], |style| {
        [
            style.overflow.x == taffy::Overflow::Scroll,
            style.overflow.y == taffy::Overflow::Scroll,
        ]
    });
    let scroll_range = [
        (layout.content_size.width - (w - layout.border.left - layout.border.right)).max(0.0),
        (layout.content_size.height - (h - layout.border.top - layout.border.bottom)).max(0.0),
    ];

    if let Some(paint) = tree.get_node_context(node)
        && w > 0.0
        && h > 0.0
    {
        out.push(PlacedNode {
            node_id: node,
            parent_node_id,
            depth,
            rect,
            content_origin: [cx, cy],
            content_size: [content_width, content_height],
            clip: inherited_clip.rect,
            clip_radius: inherited_clip.radius,
            clip_depth: inherited_clip.depth,
            own_clip: None,
            own_clip_radius: 0.0,
            border_widths: [
                layout.border.top,
                layout.border.right,
                layout.border.bottom,
                layout.border.left,
            ],
            scroll: ScrollMetrics {
                port: [
                    x0 + layout.border.left,
                    y0 + layout.border.top,
                    x0 + w - layout.border.right,
                    y0 + h - layout.border.bottom,
                ],
                scrollable,
                range: scroll_range,
                offset: scroll,
                opacity: 0.0,
                interaction: 0,
            },
            paint: paint.clone(),
        });
    }

    let mut child_clip = inherited_clip;
    if let Ok(style) = tree.style(node) {
        let clips_x = style.overflow.x != taffy::Overflow::Visible;
        let clips_y = style.overflow.y != taffy::Overflow::Visible;
        if clips_x || clips_y {
            let mut own = [
                f32::NEG_INFINITY,
                f32::NEG_INFINITY,
                f32::INFINITY,
                f32::INFINITY,
            ];
            if clips_x {
                own[0] = x0 + layout.border.left;
                own[2] = x0 + w - layout.border.right;
            }
            if clips_y {
                own[1] = y0 + layout.border.top;
                own[3] = y0 + h - layout.border.bottom;
            }
            let next_clip = child_clip.rect.map_or(own, |clip| intersect(clip, own));
            child_clip.radius = if clips_x && clips_y && next_clip == own {
                tree.get_node_context(node).map_or(0.0, |paint| {
                    (paint.border_radius
                        - layout
                            .border
                            .top
                            .max(layout.border.right)
                            .max(layout.border.bottom)
                            .max(layout.border.left))
                    .max(0.0)
                })
            } else if child_clip.rect != Some(next_clip) {
                0.0
            } else {
                child_clip.radius
            };
            child_clip.rect = Some(next_clip);
            child_clip.depth = Some(depth);
            if let Some(placed) = out.last_mut()
                && placed.node_id == node
            {
                placed.own_clip = Some(next_clip);
                placed.own_clip_radius = child_clip.radius;
            }
        }
    }
    // A clipped subtree with an empty effective clip is fully invisible. The
    // zero-sized clipping node itself may have been omitted from `out`; walking
    // its descendants would therefore lose the layer boundary and flash them
    // un-clipped for one frame during collapse animations.
    if child_clip
        .rect
        .is_some_and(|clip| clip[2] <= clip[0] || clip[3] <= clip[1])
    {
        return;
    }
    // Sibling-relative z order (Slint/Qt-Quick model): higher z paints later.
    let mut children: Vec<NodeId> = tree.child_ids(node).collect();
    children.sort_by_key(|c| {
        tree.get_node_context(*c)
            .map_or(Default::default(), |p| (p.overlay_plane, p.z_index))
    });
    for child in children {
        walk(
            tree,
            child,
            Some(node),
            depth + 1,
            x0 - scroll[0],
            y0 - scroll[1],
            child_clip,
            scroll_offsets,
            out,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_offsets_children_and_propagates_overflow_clip() {
        let mut tree = TaffyTree::<Paint>::new();
        let child = tree
            .new_leaf(taffy::Style {
                size: Size {
                    width: taffy::Dimension::length(100.0),
                    height: taffy::Dimension::length(300.0),
                },
                flex_shrink: 0.0,
                ..Default::default()
            })
            .unwrap();
        let container = tree
            .new_with_children(
                taffy::Style {
                    size: Size {
                        width: taffy::Dimension::length(100.0),
                        height: taffy::Dimension::length(100.0),
                    },
                    overflow: taffy::Point {
                        x: taffy::Overflow::Visible,
                        y: taffy::Overflow::Scroll,
                    },
                    ..Default::default()
                },
                &[child],
            )
            .unwrap();
        let root = tree
            .new_with_children(
                taffy::Style {
                    size: Size {
                        width: taffy::Dimension::length(200.0),
                        height: taffy::Dimension::length(200.0),
                    },
                    ..Default::default()
                },
                &[container],
            )
            .unwrap();
        for node in [root, container, child] {
            tree.set_node_context(node, Some(Paint::default())).unwrap();
        }
        let mut scroll = HashMap::new();
        scroll.insert(container, [0.0, 50.0]);
        let mut text = TextContext::new();
        let placed =
            compute_and_walk_with_scroll(&mut tree, root, 200.0, 200.0, &mut text, &scroll);

        assert_eq!(placed[2].rect, [0.0, -50.0, 100.0, 250.0]);
        assert_eq!(
            placed[2].clip,
            Some([f32::NEG_INFINITY, 0.0, f32::INFINITY, 100.0])
        );
        assert_eq!(placed[2].clip_depth, Some(1));
    }

    #[test]
    fn rounded_overflow_propagates_inner_clip_radius() {
        let mut tree = TaffyTree::<Paint>::new();
        let child = tree
            .new_leaf(taffy::Style {
                size: Size {
                    width: taffy::Dimension::length(10.0),
                    height: taffy::Dimension::length(10.0),
                },
                ..Default::default()
            })
            .unwrap();
        let container = tree
            .new_with_children(
                taffy::Style {
                    size: Size {
                        width: taffy::Dimension::length(100.0),
                        height: taffy::Dimension::length(100.0),
                    },
                    border: taffy::Rect::length(1.0_f32),
                    overflow: taffy::Point {
                        x: taffy::Overflow::Hidden,
                        y: taffy::Overflow::Hidden,
                    },
                    ..Default::default()
                },
                &[child],
            )
            .unwrap();
        let root = tree
            .new_with_children(taffy::Style::default(), &[container])
            .unwrap();
        tree.set_node_context(root, Some(Paint::default())).unwrap();
        tree.set_node_context(
            container,
            Some(Paint {
                border_radius: 12.0,
                ..Paint::default()
            }),
        )
        .unwrap();
        tree.set_node_context(child, Some(Paint::default()))
            .unwrap();

        let mut text = TextContext::new();
        let placed =
            compute_and_walk_with_scroll(&mut tree, root, 100.0, 100.0, &mut text, &HashMap::new());

        assert_eq!(placed[2].clip, Some([1.0, 1.0, 99.0, 99.0]));
        assert_eq!(placed[2].clip_radius, 11.0);
        assert_eq!(placed[1].own_clip, Some([1.0, 1.0, 99.0, 99.0]));
        assert_eq!(placed[1].own_clip_radius, 11.0);
    }

    #[test]
    fn empty_overflow_clip_prunes_visible_descendants() {
        let mut tree = TaffyTree::<Paint>::new();
        let child = tree
            .new_leaf(taffy::Style {
                size: Size {
                    width: taffy::Dimension::length(80.0),
                    height: taffy::Dimension::length(20.0),
                },
                ..Default::default()
            })
            .unwrap();
        let collapsed = tree
            .new_with_children(
                taffy::Style {
                    size: Size {
                        width: taffy::Dimension::length(100.0),
                        height: taffy::Dimension::length(0.0),
                    },
                    overflow: taffy::Point {
                        x: taffy::Overflow::Hidden,
                        y: taffy::Overflow::Hidden,
                    },
                    ..Default::default()
                },
                &[child],
            )
            .unwrap();
        let root = tree
            .new_with_children(
                taffy::Style {
                    size: Size {
                        width: taffy::Dimension::length(100.0),
                        height: taffy::Dimension::length(100.0),
                    },
                    ..Default::default()
                },
                &[collapsed],
            )
            .unwrap();
        for node in [root, collapsed, child] {
            tree.set_node_context(node, Some(Paint::default())).unwrap();
        }

        let mut text = TextContext::new();
        let placed =
            compute_and_walk_with_scroll(&mut tree, root, 100.0, 100.0, &mut text, &HashMap::new());

        assert_eq!(placed.len(), 1);
        assert_eq!(placed[0].node_id, root);
    }

    #[test]
    fn overlay_plane_orders_before_z_index() {
        use crate::style::OverlayPlane;

        let mut tree = TaffyTree::<Paint>::new();
        let leaf_style = taffy::Style {
            size: Size {
                width: taffy::Dimension::length(10.0),
                height: taffy::Dimension::length(10.0),
            },
            ..Default::default()
        };
        let modal = tree.new_leaf(leaf_style.clone()).unwrap();
        let content = tree.new_leaf(leaf_style).unwrap();
        let root = tree
            .new_with_children(
                taffy::Style {
                    size: Size {
                        width: taffy::Dimension::length(100.0),
                        height: taffy::Dimension::length(100.0),
                    },
                    ..Default::default()
                },
                &[modal, content],
            )
            .unwrap();
        tree.set_node_context(root, Some(Paint::default())).unwrap();
        tree.set_node_context(
            content,
            Some(Paint {
                overlay_plane: OverlayPlane::Content,
                z_index: 10_000,
                ..Paint::default()
            }),
        )
        .unwrap();
        tree.set_node_context(
            modal,
            Some(Paint {
                overlay_plane: OverlayPlane::Modal,
                z_index: -10_000,
                ..Paint::default()
            }),
        )
        .unwrap();

        let mut text = TextContext::new();
        let placed =
            compute_and_walk_with_scroll(&mut tree, root, 100.0, 100.0, &mut text, &HashMap::new());
        assert_eq!(
            placed.iter().map(|node| node.node_id).collect::<Vec<_>>(),
            vec![root, content, modal]
        );
    }
}
