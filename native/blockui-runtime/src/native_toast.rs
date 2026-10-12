//! A fixed nonmodal plane laid out independently of authored scroll ancestors.
use super::*;
impl<M: NativeModel> NativeSource<M> {
    pub(super) fn reconcile_toast_activity(&mut self) -> bool {
        let changes: Vec<_> = self
            .actor
            .view()
            .order
            .iter()
            .filter_map(|key| {
                let node = self.actor.view().bindings[key].node;
                if node.semantics.role != semantics::TOAST_VIEWPORT || node.action == 0 {
                    return None;
                }
                let hovered = self.pointer_position.is_some_and(|point| {
                    self.ids.get(key).is_some_and(|id| {
                        self.placed.iter().any(|placed| {
                            placed.node_id == *id
                                && point.x >= f64::from(placed.rect[0])
                                && point.x <= f64::from(placed.rect[2])
                                && point.y >= f64::from(placed.rect[1])
                                && point.y <= f64::from(placed.rect[3])
                        })
                    })
                });
                let paused = !self.window_focused
                    || self.modal.is_some()
                    || hovered
                    || self.focus.and_then(|focus| self.toast_owner(focus)) == Some(*key);
                (node.action_payload != i64::from(paused)).then_some((*key, paused))
            })
            .collect();
        let mut changed = false;
        for (key, paused) in changes {
            match self.actor.observe_toast_activity(key, paused) {
                Ok(()) => changed = true,
                Err(error) => self.diagnostic = error,
            }
        }
        changed
    }
    pub(super) fn toast_owner(&self, mut key: u64) -> Option<u64> {
        while let Some(binding) = self.actor.view().bindings.get(&key) {
            if binding.node.semantics.role == semantics::TOAST_VIEWPORT {
                return Some(key);
            }
            key = binding.node.parent;
        }
        None
    }
    fn toast_focus_available(&self, mut key: u64) -> bool {
        while let Some(binding) = self.actor.view().bindings.get(&key) {
            if binding.node.semantics.hidden != 0 {
                return false;
            }
            key = binding.node.parent;
        }
        key == 0
    }
    pub(super) fn navigate_toast(&mut self, name: &str) -> bool {
        if self.modal.is_some() {
            return false;
        }
        let target = match name {
            "F8" => self.actor.view().order.iter().copied().find(|key| {
                let node = self.actor.view().bindings[key].node;
                node.kind == BUTTON
                    && node.disabled == 0
                    && node.semantics.hidden == 0
                    && self.toast_focus_available(*key)
                    && self.toast_owner(*key).is_some()
            }),
            "Escape" if self.focus.and_then(|key| self.toast_owner(key)).is_some() => {
                self.toast_return_focus.filter(|key| {
                    self.actor.view().bindings.get(key).is_some_and(|b| {
                        b.node.disabled == 0
                            && b.node.semantics.hidden == 0
                            && self.toast_focus_available(*key)
                            && self.toast_owner(*key).is_none()
                    })
                })
            }
            _ => None,
        };
        let Some(target) = target else {
            return false;
        };
        self.set_focus(Some(target));
        self.reveal_widget(target);
        true
    }
    pub(super) fn append_toasts(
        &mut self,
        nodes: &mut Vec<PlacedNode>,
        tcx: &mut TextContext,
        width: u32,
        height: u32,
        reveal: Option<u64>,
    ) {
        let keys: Vec<_> = self
            .actor
            .view()
            .order
            .iter()
            .copied()
            .filter(|key| {
                let node = self.actor.view().bindings[key].node;
                node.semantics.hidden == 0
                    && ((node.semantics.role == semantics::TOAST_VIEWPORT && self.modal.is_none())
                        || (semantics::is_anchored_panel(node.semantics.role)
                            && self.in_modal(*key)))
            })
            .collect();
        let mut remaining = [0usize; 6];
        for key in &keys {
            if self.actor.view().bindings[key].node.semantics.role != semantics::TOAST_VIEWPORT {
                continue;
            }
            if self
                .actor
                .view()
                .order
                .iter()
                .any(|child| self.actor.view().bindings[child].node.parent == *key)
            {
                remaining[self.actor.view().bindings[key].node.overlay_position as usize] += 1;
            }
        }
        let mut occupied = [0.0f32; 6];
        for key in keys {
            if self.actor.view().bindings[&key].node.semantics.role == semantics::TOAST_VIEWPORT
                && !self
                    .actor
                    .view()
                    .order
                    .iter()
                    .any(|child| self.actor.view().bindings[child].node.parent == key)
            {
                continue;
            }
            let binding = self.actor.view().bindings[&key].node;
            let popover = semantics::is_anchored_panel(binding.semantics.role);
            let anchor = if popover {
                let Some(anchor) = nodes
                    .iter()
                    .find(|n| self.ids.get(&binding.semantics.labelled_by) == Some(&n.node_id))
                else {
                    continue;
                };
                Some(anchor.rect)
            } else {
                None
            };
            let position = binding.overlay_position as usize;
            let available =
                (height as f32 - 32.0 - if popover { 0.0 } else { occupied[position] }).max(0.0);
            if available == 0.0 {
                continue;
            }
            let id = self.ids[&key];
            let mut style = self.tree.style(id).unwrap().clone();
            style.size.width = Dimension::length((width as f32 - 32.0).clamp(
                0.0,
                if popover {
                    binding.presentation.width
                } else {
                    380.0
                },
            ));
            style.size.height = Dimension::auto();
            let count = if popover { 1 } else { remaining[position] };
            let share = (available - 12.0 * count.saturating_sub(1) as f32).max(0.0) / count as f32;
            if !popover {
                remaining[position] -= 1;
            }
            style.max_size.height = LengthPercentageAuto::length(if popover {
                share
            } else {
                share.min(height as f32 * 0.5)
            });
            style.flex_shrink = 0.0;
            if self.tree.style(id).unwrap() != &style {
                self.tree.set_style(id, style).unwrap();
            }
            let mut offsets: HashMap<_, _> = self
                .actor
                .view()
                .renderer
                .iter()
                .filter_map(|(k, s)| {
                    self.ids
                        .get(k)
                        .map(|id| (*id, [s.scroll_x as f32, s.scroll as f32]))
                })
                .collect();
            let mut overlay = shell::layout::compute_and_walk_with_scroll(
                &mut self.tree,
                id,
                width as f32,
                height as f32,
                tcx,
                &offsets,
            );
            let Some(root) = overlay.first() else {
                continue;
            };
            let mut requested = root.scroll.offset;
            if let Some(target) = reveal.filter(|target| self.popover_owner(*target) == Some(key))
                && let Some(target) = overlay
                    .iter()
                    .find(|node| self.ids.get(&target) == Some(&node.node_id))
            {
                for (axis, offset) in requested.iter_mut().enumerate() {
                    let start = root.scroll.port[axis];
                    let end = root.scroll.port[axis + 2];
                    let rect = target.rect;
                    *offset += if rect[axis] < start || rect[axis + 2] - rect[axis] > end - start {
                        rect[axis] - start
                    } else if rect[axis + 2] > end {
                        rect[axis + 2] - end
                    } else {
                        0.0
                    };
                }
            }
            let offset = [
                requested[0].clamp(0.0, root.scroll.range[0]),
                requested[1].clamp(0.0, root.scroll.range[1]),
            ];
            if offset != root.scroll.offset {
                let state = self.actor.view_mut().renderer.get_mut(&key).unwrap();
                state.scroll_x = f64::from(offset[0]);
                state.scroll = f64::from(offset[1]);
                offsets.insert(id, offset);
                overlay = shell::layout::compute_and_walk_with_scroll(
                    &mut self.tree,
                    id,
                    width as f32,
                    height as f32,
                    tcx,
                    &offsets,
                );
            }
            let root = &overlay[0];
            let root_width = root.rect[2] - root.rect[0];
            let x = match position % 3 {
                1 => 16.0f32.min(width as f32 * 0.5),
                2 => ((width as f32 - root_width) * 0.5).max(0.0),
                _ => (width as f32 - 16.0 - root_width).max(0.0),
            };
            let [x, y] = anchor.map_or_else(
                || {
                    let y = if position >= 3 {
                        16.0 + occupied[position]
                    } else {
                        height as f32 - 16.0 - occupied[position] - (root.rect[3] - root.rect[1])
                    };
                    [x, y]
                },
                |a| {
                    popover_origin(
                        a,
                        [root_width, root.rect[3] - root.rect[1]],
                        [width as f32, height as f32],
                        position as u32,
                    )
                },
            );
            let [x, y] = self
                .context_anchor
                .filter(|(trigger, _)| *trigger == binding.semantics.labelled_by)
                .and_then(|(_, point)| {
                    crate::interaction::context_menu::origin(
                        point,
                        [
                            f64::from(root_width),
                            f64::from(root.rect[3] - root.rect[1]),
                        ],
                        [f64::from(width), f64::from(height)],
                    )
                })
                .map_or([x, y], |point| point.map(|coordinate| coordinate as f32));
            let dx = x - root.rect[0];
            let dy = y - root.rect[1];
            if !popover {
                occupied[position] += root.rect[3] - root.rect[1] + 12.0;
            }
            let translate = |r: &mut [f32; 4]| {
                r[0] += dx;
                r[2] += dx;
                r[1] += dy;
                r[3] += dy;
            };
            for node in &mut overlay {
                translate(&mut node.rect);
                node.content_origin[0] += dx;
                node.content_origin[1] += dy;
                if let Some(clip) = &mut node.clip {
                    translate(clip);
                }
                if let Some(clip) = &mut node.own_clip {
                    translate(clip);
                }
                translate(&mut node.scroll.port);
            }
            nodes.extend(overlay);
        }
    }
}

/// Physical placement shared by every anchored native panel. Prefer the opposite
/// side only when the requested side cannot fit and the opposite has more room.
pub(super) fn popover_origin(
    a: [f32; 4],
    size: [f32; 2],
    viewport: [f32; 2],
    placement: u32,
) -> [f32; 2] {
    let [w, h] = size;
    let [vw, vh] = viewport;
    let mut side = placement / 3;
    let align = placement % 3;
    let room = [
        vh - 16.0 - a[3] - 8.0,
        a[1] - 24.0,
        vw - 16.0 - a[2] - 8.0,
        a[0] - 24.0,
    ];
    let opposite = side ^ 1;
    let needed = if side < 2 { h } else { w };
    if room[side as usize] < needed && room[opposite as usize] > room[side as usize] {
        side = opposite;
    }
    let cross = |start: f32, end: f32, length: f32| match align {
        1 => (start + end - length) * 0.5,
        2 => end - length,
        _ => start,
    };
    let (x, y) = match side {
        0 => (cross(a[0], a[2], w), a[3] + 8.0),
        1 => (cross(a[0], a[2], w), a[1] - 8.0 - h),
        2 => (a[2] + 8.0, cross(a[1], a[3], h)),
        _ => (a[0] - 8.0 - w, cross(a[1], a[3], h)),
    };
    let mx = 16.0f32.min(vw * 0.5);
    let my = 16.0f32.min(vh * 0.5);
    [
        x.clamp(mx, (vw - 16.0 - w).max(mx)),
        y.clamp(my, (vh - 16.0 - h).max(my)),
    ]
}
