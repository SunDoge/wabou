//! Pointer, keyboard and focus routing state.

use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct EventMask(u64);

impl EventMask {
    fn bit(code: u8) -> Option<u64> {
        code.checked_sub(1)
            .filter(|bit| u32::from(*bit) < u64::BITS)
            .map(|bit| 1 << bit)
    }

    pub(super) fn insert(&mut self, code: u8) {
        if let Some(bit) = Self::bit(code) {
            self.0 |= bit;
        }
    }

    pub(super) fn remove(&mut self, code: u8) {
        if let Some(bit) = Self::bit(code) {
            self.0 &= !bit;
        }
    }

    pub(super) fn contains(self, code: u8) -> bool {
        Self::bit(code).is_some_and(|bit| self.0 & bit != 0)
    }

    #[cfg(any(feature = "devtools", test))]
    pub(super) fn codes(self) -> impl Iterator<Item = u8> {
        (1..=u64::BITS as u8).filter(move |&code| self.contains(code))
    }
}

#[derive(Clone)]
pub(super) struct HitClip {
    pub(super) rect: [f32; 4],
    pub(super) radius: f32,
    pub(super) transform: Affine,
}

#[derive(Clone)]
pub(super) struct HitNode {
    pub(super) solid_id: NodeKey,
    pub(super) rect: [f32; 4],
    pub(super) transform: Affine,
    pub(super) clips: Vec<HitClip>,
    pub(super) pointer_events: bool,
}

#[derive(Clone)]
pub(super) enum HitItem {
    Content(HitNode),
    Scrollbar(Box<ScrollbarHit>),
}

pub(super) fn hit_contains(rect: [f32; 4], radius: f32, transform: Affine, point: Point) -> bool {
    let [a, b, c, d, _, _] = transform.as_coeffs();
    let determinant = a * d - b * c;
    if !determinant.is_finite() || determinant.abs() <= f64::EPSILON {
        return false;
    }
    let local = transform.inverse() * point;
    let [x0, y0, x1, y1] = rect.map(f64::from);
    if local.x < x0 || local.y < y0 || local.x >= x1 || local.y >= y1 {
        return false;
    }
    let radius = f64::from(radius).min((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
    if radius <= 0.0
        || (local.x >= x0 + radius && local.x < x1 - radius)
        || (local.y >= y0 + radius && local.y < y1 - radius)
    {
        return true;
    }
    let center_x = if local.x < x0 + radius {
        x0 + radius
    } else {
        x1 - radius
    };
    let center_y = if local.y < y0 + radius {
        y0 + radius
    } else {
        y1 - radius
    };
    (local.x - center_x).powi(2) + (local.y - center_y).powi(2) <= radius.powi(2)
}

#[derive(Clone, Copy, Default)]
pub(super) struct PointerRouteState {
    pub(super) position: (f64, f64),
    pub(super) buttons: u32,
    pub(super) properties: wabou_shell_gpui::PointerProperties,
    pub(super) hovered_target: Option<NodeKey>,
    pub(super) down_target: Option<NodeKey>,
    pub(super) down_position: Option<(f64, f64)>,
    pub(super) dragged: bool,
}

#[derive(Default)]
pub(super) struct InputRouter {
    pub(super) listeners: HashMap<NodeKey, EventMask>,
    pub(super) pointer_position: (f64, f64),
    pub(super) pointer_buttons: u32,
    pub(super) pointer_properties: wabou_shell_gpui::PointerProperties,
    pub(super) pointer_routes: HashMap<wabou_shell_gpui::PointerId, PointerRouteState>,
    pub(super) pointer_down_target: Option<NodeKey>,
    pub(super) pointer_down_position: Option<(f64, f64)>,
    pub(super) pointer_dragged: bool,
    /// Last completed primary activation used to synthesize `dblclick`.
    pub(super) last_primary_click: Option<(Instant, NodeKey, f64, f64)>,
    pub(super) next_host_event_id: u32,
    pub(super) hovered_target: Option<NodeKey>,
    pub(super) focused_target: Option<NodeKey>,
    /// Whether the current input modality should expose keyboard focus UI.
    pub(super) focus_visible: bool,
    pub(super) focusable_targets: HashSet<NodeKey>,
    pub(super) focus_order: Vec<NodeKey>,
    pub(super) window_focused: bool,
    pub(super) hit_items: Vec<HitItem>,
    /// Explicit target supplied by a native backend with authoritative hit testing.
    pub(super) target_override: Option<NodeKey>,
}

impl InputRouter {
    pub(super) fn new() -> Self {
        Self {
            window_focused: true,
            ..Self::default()
        }
    }

    pub(super) fn hit_test(&self, x: f64, y: f64) -> Option<NodeKey> {
        if let Some(target) = self.target_override {
            return Some(target);
        }
        let point = Point::new(x, y);
        for item in self.hit_items.iter().rev() {
            match item {
                HitItem::Content(node)
                    if node.pointer_events
                        && node.clips.iter().all(|clip| {
                            hit_contains(clip.rect, clip.radius, clip.transform, point)
                        })
                        && hit_contains(node.rect, 0.0, node.transform, point) =>
                {
                    return Some(node.solid_id);
                }
                HitItem::Scrollbar(hit)
                    if scrollbar_hit(&hit.placed, hit.transform.inverse() * point).is_some() =>
                {
                    return None;
                }
                _ => {}
            }
        }
        None
    }

    pub(super) fn update_pointer(&mut self, pointer: &wabou_shell_gpui::PointerEvent) {
        let route = self
            .pointer_routes
            .entry(pointer.properties.id)
            .or_default();
        route.position = (pointer.position.x, pointer.position.y);
        route.buttons = pointer.buttons;
        route.properties = pointer.properties;
        if pointer.properties.primary {
            self.pointer_position = route.position;
            self.pointer_buttons = pointer.buttons;
            self.pointer_properties = pointer.properties;
        }
    }

    pub(super) fn local_position(&self, target: NodeKey, x: f64, y: f64) -> (f64, f64) {
        let point = Point::new(x, y);
        self.hit_items
            .iter()
            .rev()
            .find_map(|item| match item {
                HitItem::Content(node) if node.solid_id == target => {
                    let local = node.transform.inverse() * point;
                    Some((
                        local.x - f64::from(node.rect[0]),
                        local.y - f64::from(node.rect[1]),
                    ))
                }
                _ => None,
            })
            .unwrap_or((x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_position_uses_the_target_transform_and_rect_origin() {
        let mut router = InputRouter::new();
        router.hit_items.push(HitItem::Content(HitNode {
            solid_id: NodeKey::new(7, 1),
            rect: [20.0, 30.0, 120.0, 130.0],
            transform: Affine::translate((100.0, 50.0)),
            clips: Vec::new(),
            pointer_events: true,
        }));

        assert_eq!(
            router.local_position(NodeKey::new(7, 1), 145.0, 95.0),
            (25.0, 15.0)
        );
    }

    #[test]
    fn authoritative_backend_target_overrides_legacy_geometry_for_one_event() {
        let mut router = InputRouter::new();
        router.hit_items.push(HitItem::Content(HitNode {
            solid_id: NodeKey::new(2, 1),
            rect: [0.0, 0.0, 100.0, 100.0],
            transform: Affine::IDENTITY,
            clips: Vec::new(),
            pointer_events: true,
        }));
        router.target_override = Some(NodeKey::new(9, 3));

        assert_eq!(router.hit_test(50.0, 50.0), Some(NodeKey::new(9, 3)));
        router.target_override = None;
        assert_eq!(router.hit_test(50.0, 50.0), Some(NodeKey::new(2, 1)));
    }
}
