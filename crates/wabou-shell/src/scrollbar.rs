//! Overlay scrollbar geometry shared by Vello painting and native hit testing.

use vello::kurbo::{Point, Rect};

use crate::layout::PlacedNode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollbarPart {
    Thumb,
    TrackBefore,
    TrackAfter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollbarTarget {
    pub axis: ScrollAxis,
    pub part: ScrollbarPart,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedScrollbarGeometry {
    pub track: Rect,
    pub thumb: Rect,
}

/// Resolve viewport-dependent scrollbar dimensions once so paint, hit testing
/// and dragging cannot disagree for extreme custom values.
pub fn resolve(node: &PlacedNode, axis: ScrollAxis) -> Option<ResolvedScrollbarGeometry> {
    let index = usize::from(axis == ScrollAxis::Vertical);
    if node.scroll.range[index] <= 0.5 || !node.scroll.scrollable[index] {
        return None;
    }
    let [x0, y0, x1, y1] = node.scroll.port;
    let port = Rect::new(x0.into(), y0.into(), x1.into(), y1.into());
    let main = match axis {
        ScrollAxis::Horizontal => port.width(),
        ScrollAxis::Vertical => port.height(),
    }
    .max(0.0);
    let cross = match axis {
        ScrollAxis::Horizontal => port.height(),
        ScrollAxis::Vertical => port.width(),
    }
    .max(0.0);
    if main <= 0.0 || cross <= 0.0 {
        return None;
    }
    let style = node.paint.scrollbar;
    let thickness = f64::from(style.thickness).clamp(0.0, cross);
    let margin = f64::from(style.margin).clamp(0.0, (cross - thickness).max(0.0));
    let track = match axis {
        ScrollAxis::Horizontal => Rect::new(
            port.x0,
            port.y1 - margin - thickness,
            port.x1,
            port.y1 - margin,
        ),
        ScrollAxis::Vertical => Rect::new(
            port.x1 - margin - thickness,
            port.y0,
            port.x1 - margin,
            port.y1,
        ),
    };
    let range = f64::from(node.scroll.range[index]);
    let length = (main * main / (main + range))
        .max(f64::from(style.min_thumb_length))
        .clamp(0.0, main);
    let progress = (f64::from(node.scroll.offset[index]) / range).clamp(0.0, 1.0);
    let start = progress * (main - length);
    let thumb = match axis {
        ScrollAxis::Horizontal => Rect::new(
            port.x0 + start,
            track.y0,
            port.x0 + start + length,
            track.y1,
        ),
        ScrollAxis::Vertical => Rect::new(
            track.x0,
            port.y0 + start,
            track.x1,
            port.y0 + start + length,
        ),
    };
    Some(ResolvedScrollbarGeometry { track, thumb })
}

pub fn track(node: &PlacedNode, axis: ScrollAxis) -> Option<Rect> {
    resolve(node, axis).map(|geometry| geometry.track)
}

pub fn thumb(node: &PlacedNode, axis: ScrollAxis) -> Option<Rect> {
    resolve(node, axis).map(|geometry| geometry.thumb)
}

pub fn hit(node: &PlacedNode, point: Point) -> Option<ScrollbarTarget> {
    for axis in [ScrollAxis::Vertical, ScrollAxis::Horizontal] {
        let Some(thumb) = thumb(node, axis) else {
            continue;
        };
        if thumb.contains(point) {
            return Some(ScrollbarTarget {
                axis,
                part: ScrollbarPart::Thumb,
            });
        }
        if track(node, axis).is_some_and(|track| track.contains(point)) {
            let before = match axis {
                ScrollAxis::Horizontal => point.x < thumb.x0,
                ScrollAxis::Vertical => point.y < thumb.y0,
            };
            return Some(ScrollbarTarget {
                axis,
                part: if before {
                    ScrollbarPart::TrackBefore
                } else {
                    ScrollbarPart::TrackAfter
                },
            });
        }
    }
    None
}

pub fn drag_ratio(node: &PlacedNode, axis: ScrollAxis) -> f64 {
    let Some(thumb) = thumb(node, axis) else {
        return 0.0;
    };
    let index = usize::from(axis == ScrollAxis::Vertical);
    let viewport = match axis {
        ScrollAxis::Horizontal => node.scroll.port[2] - node.scroll.port[0],
        ScrollAxis::Vertical => node.scroll.port[3] - node.scroll.port[1],
    } as f64;
    let length = match axis {
        ScrollAxis::Horizontal => thumb.width(),
        ScrollAxis::Vertical => thumb.height(),
    };
    let play = viewport - length;
    if play > 0.0 {
        f64::from(node.scroll.range[index]) / play
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Paint;

    fn node(offset: f32) -> PlacedNode {
        PlacedNode {
            node_id: taffy::tree::NodeId::from(0_u64),
            parent_node_id: None,
            depth: 0,
            rect: [0.0, 0.0, 100.0, 100.0],
            content_origin: [0.0, 0.0],
            content_size: [100.0, 100.0],
            clip: None,
            clip_radius: 0.0,
            clip_depth: None,
            own_clip: Some([0.0, 0.0, 100.0, 100.0]),
            own_clip_radius: 0.0,
            border_widths: [0.0; 4],
            scroll: crate::layout::ScrollMetrics {
                port: [0.0, 0.0, 100.0, 100.0],
                scrollable: [false, true],
                range: [0.0, 900.0],
                offset: [0.0, offset],
                opacity: 1.0,
                interaction: 0,
            },
            paint: Paint::default(),
        }
    }

    #[test]
    fn blitz_geometry_maps_half_scroll_to_half_track() {
        let node = node(450.0);
        assert_eq!(
            thumb(&node, ScrollAxis::Vertical),
            Some(Rect::new(88.0, 34.0, 98.0, 66.0))
        );
        assert!((drag_ratio(&node, ScrollAxis::Vertical) - 900.0 / 68.0).abs() < 0.001);
        assert_eq!(
            hit(&node, Point::new(95.0, 50.0)),
            Some(ScrollbarTarget {
                axis: ScrollAxis::Vertical,
                part: ScrollbarPart::Thumb
            })
        );
    }

    #[test]
    fn custom_appearance_changes_shared_paint_and_hit_geometry() {
        let mut node = node(450.0);
        node.paint.scrollbar.thickness = 14.0;
        node.paint.scrollbar.margin = 3.0;
        node.paint.scrollbar.min_thumb_length = 40.0;
        assert_eq!(
            thumb(&node, ScrollAxis::Vertical),
            Some(Rect::new(83.0, 30.0, 97.0, 70.0))
        );
        assert_eq!(
            hit(&node, Point::new(90.0, 90.0)),
            Some(ScrollbarTarget {
                axis: ScrollAxis::Vertical,
                part: ScrollbarPart::TrackAfter,
            })
        );
    }

    #[test]
    fn extreme_custom_dimensions_are_clamped_to_the_scroll_port() {
        let mut node = node(450.0);
        node.paint.scrollbar.thickness = 500.0;
        node.paint.scrollbar.margin = 500.0;
        node.paint.scrollbar.min_thumb_length = 500.0;
        let geometry = resolve(&node, ScrollAxis::Vertical).unwrap();
        assert_eq!(geometry.track, Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(geometry.thumb, Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(drag_ratio(&node, ScrollAxis::Vertical), 0.0);
    }
}
