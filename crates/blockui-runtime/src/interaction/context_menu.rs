//! Pointer placement for context menus. Hosts own the point; plugins only own
//! menu state and commands. The result keeps an ordinary-sized panel in view.
pub fn origin(point: [f64; 2], size: [f64; 2], viewport: [f64; 2]) -> Option<[f64; 2]> {
    if point
        .into_iter()
        .chain(size)
        .chain(viewport)
        .any(|v| !v.is_finite())
        || size.into_iter().chain(viewport).any(|v| v < 0.0)
    {
        return None;
    }
    Some(std::array::from_fn(|axis| {
        let margin = 16.0f64.min(viewport[axis] / 2.0);
        let position = if point[axis] + size[axis] <= viewport[axis] - margin {
            point[axis]
        } else {
            point[axis] - size[axis]
        };
        position.clamp(margin, (viewport[axis] - margin - size[axis]).max(margin))
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pointer_origin_and_edge_flip() {
        assert_eq!(
            origin([100.0, 120.0], [240.0, 80.0], [900.0, 600.0]),
            Some([100.0, 120.0])
        );
        assert_eq!(
            origin([890.0, 590.0], [240.0, 80.0], [900.0, 600.0]),
            Some([644.0, 504.0])
        );
        assert_eq!(
            origin([-10.0, -20.0], [240.0, 80.0], [900.0, 600.0]),
            Some([16.0, 16.0])
        );
    }
    #[test]
    fn tiny_viewports_and_invalid_geometry() {
        assert_eq!(
            origin([2.0, 2.0], [240.0, 80.0], [10.0, 10.0]),
            Some([5.0, 5.0])
        );
        assert_eq!(origin([f64::NAN, 2.0], [240.0, 80.0], [900.0, 600.0]), None);
        assert_eq!(origin([2.0, 2.0], [-1.0, 80.0], [900.0, 600.0]), None);
    }
}
