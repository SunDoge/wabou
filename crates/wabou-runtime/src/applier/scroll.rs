use super::*;

impl Applier {
    pub(super) fn scroll_into_view(&mut self, target: u32) -> bool {
        let Some(mut node) = self.node_store.solid_to_node.get(&target).copied() else {
            return false;
        };
        let Some(mut rect) = self.placed_rects.get(&node).copied() else {
            return false;
        };
        let mut changed = false;
        while let Some(parent) = self.node_store.tree.parent(node) {
            if let Some(metrics) = self.scroll_metrics.get(&parent).copied() {
                let offset = self.scroll_offsets.entry(parent).or_insert(metrics.offset);
                let mut parent_changed = false;
                for axis in 0..2 {
                    if !metrics.scrollable[axis] {
                        continue;
                    }
                    let start = rect[axis];
                    let end = rect[axis + 2];
                    let port_start = metrics.port[axis];
                    let port_end = metrics.port[axis + 2];
                    let delta = if end - start >= port_end - port_start {
                        (start + end - port_start - port_end) * 0.5
                    } else if start < port_start {
                        start - port_start
                    } else if end > port_end {
                        end - port_end
                    } else {
                        0.0
                    };
                    let previous = offset[axis];
                    offset[axis] = (previous + delta).clamp(0.0, metrics.range[axis]);
                    let applied = offset[axis] - previous;
                    if applied != 0.0 {
                        rect[axis] -= applied;
                        rect[axis + 2] -= applied;
                        changed = true;
                        parent_changed = true;
                    }
                }
                if parent_changed {
                    self.queue_scroll_event(parent);
                }
            }
            node = parent;
        }
        self.projections.semantics_dirty |= changed;
        changed
    }

    pub(super) fn update_scrollbar_visuals(&mut self, placed: &mut [PlacedNode]) {
        let now = Instant::now();
        let mut expired = Vec::new();
        for node in placed {
            let held = self
                .scrollbar_drag
                .is_some_and(|drag| drag.node == node.node_id)
                || self
                    .hovered_scrollbar
                    .is_some_and(|(owner, _)| owner == node.node_id);
            node.scroll.opacity = match node.paint.scrollbar.visibility {
                ScrollbarVisibility::Always => {
                    expired.push(node.node_id);
                    1.0
                }
                ScrollbarVisibility::Hidden => {
                    expired.push(node.node_id);
                    0.0
                }
                ScrollbarVisibility::Auto => {
                    let opacity =
                        self.scrollbar_activity
                            .get(&node.node_id)
                            .map_or(0.0, |started| {
                                scrollbar_auto_opacity(
                                    now.duration_since(*started),
                                    node.paint.scrollbar.hide_delay,
                                    node.paint.scrollbar.fade_duration,
                                    held,
                                )
                            });
                    if opacity <= 0.0 {
                        expired.push(node.node_id);
                    }
                    opacity
                }
            };
            node.scroll.interaction = if self
                .scrollbar_drag
                .is_some_and(|drag| drag.node == node.node_id)
            {
                2
            } else if self
                .hovered_scrollbar
                .is_some_and(|(owner, _)| owner == node.node_id)
            {
                1
            } else {
                0
            };
        }
        for node in expired {
            self.scrollbar_activity.remove(&node);
        }
    }

    pub(super) fn scrollbar_at(&self, x: f64, y: f64) -> Option<(NodeId, ScrollbarTarget)> {
        let point = Point::new(x, y);
        for item in self.input.hit_items.iter().rev() {
            match item {
                HitItem::Scrollbar(hit) => {
                    if let Some(target) =
                        scrollbar_hit(&hit.placed, hit.transform.inverse() * point)
                    {
                        return Some((hit.node, target));
                    }
                }
                HitItem::Content(node)
                    if node.pointer_events
                        && node.clips.iter().all(|clip| {
                            hit_contains(clip.rect, clip.radius, clip.transform, point)
                        })
                        && hit_contains(node.rect, 0.0, node.transform, point) =>
                {
                    return None;
                }
                _ => {}
            }
        }
        None
    }

    /// Wake an auto-hidden overlay scrollbar before the pointer reaches its
    /// narrow painted track. The hot zone is display-only: pointer down still
    /// uses `scrollbar_at`, so content beside an invisible track remains
    /// clickable and dragging starts only on visible scrollbar geometry.
    pub(super) fn scrollbar_edge_at(&self, x: f64, y: f64) -> Option<(NodeId, ScrollAxis)> {
        const EDGE_HOT_ZONE: f64 = 16.0;
        let point = Point::new(x, y);
        for hit in self.scrollbar_hits.iter().rev() {
            if hit.placed.paint.scrollbar.visibility != ScrollbarVisibility::Auto {
                continue;
            }
            let local = hit.transform.inverse() * point;
            let [x0, y0, x1, y1] = hit.placed.scroll.port;
            let x0 = f64::from(x0);
            let y0 = f64::from(y0);
            let x1 = f64::from(x1);
            let y1 = f64::from(y1);
            if local.x >= x0
                && local.x <= x1
                && local.y >= y0
                && local.y <= y1
                && hit.placed.scroll.scrollable[1]
                && hit.placed.scroll.range[1] > 0.5
                && x1 - local.x <= EDGE_HOT_ZONE
            {
                return Some((hit.node, ScrollAxis::Vertical));
            }
            if local.x >= x0
                && local.x <= x1
                && local.y >= y0
                && local.y <= y1
                && hit.placed.scroll.scrollable[0]
                && hit.placed.scroll.range[0] > 0.5
                && y1 - local.y <= EDGE_HOT_ZONE
            {
                return Some((hit.node, ScrollAxis::Horizontal));
            }
        }
        None
    }

    pub(super) fn drag_scrollbar(&mut self, x: f64, y: f64) -> bool {
        let Some(mut drag) = self.scrollbar_drag else {
            return false;
        };
        let Some(hit) = self.scrollbar_hits.iter().find(|hit| hit.node == drag.node) else {
            self.scrollbar_drag = None;
            return false;
        };
        let local = hit.transform.inverse() * Point::new(x, y);
        let position = match drag.axis {
            ScrollAxis::Horizontal => local.x,
            ScrollAxis::Vertical => local.y,
        };
        let delta = (position - drag.last_position) * scrollbar_drag_ratio(&hit.placed, drag.axis);
        drag.last_position = position;
        self.scrollbar_drag = Some(drag);
        let offset = self.scroll_offsets.entry(drag.node).or_insert([0.0; 2]);
        let index = usize::from(drag.axis == ScrollAxis::Vertical);
        let old = offset[index];
        offset[index] = (offset[index] + delta as f32).clamp(0.0, hit.placed.scroll.range[index]);
        let changed = offset[index] != old;
        self.projections.semantics_dirty |= changed;
        self.scrollbar_activity.insert(drag.node, Instant::now());
        if changed {
            self.queue_scroll_event(drag.node);
        }
        changed
    }

    pub(super) fn scroll_nearest(&mut self, target: u32, delta_x: f32, delta_y: f32) -> bool {
        let Some(mut node) = self.node_store.solid_to_node.get(&target).copied() else {
            return false;
        };
        loop {
            let scrollable = self.node_store.tree.style(node).ok().is_some_and(|style| {
                style.overflow.x == taffy::Overflow::Scroll
                    || style.overflow.y == taffy::Overflow::Scroll
            });
            if scrollable {
                let Ok(layout) = self.node_store.tree.layout(node) else {
                    return false;
                };
                let viewport_width =
                    (layout.size.width - layout.border.left - layout.border.right).max(0.0);
                let viewport_height =
                    (layout.size.height - layout.border.top - layout.border.bottom).max(0.0);
                let max_x = (layout.content_size.width - viewport_width).max(0.0);
                let max_y = (layout.content_size.height - viewport_height).max(0.0);
                let style = self
                    .node_store
                    .tree
                    .style(node)
                    .expect("style checked above");
                let offset = self.scroll_offsets.entry(node).or_insert([0.0, 0.0]);
                let old = *offset;
                if style.overflow.x == taffy::Overflow::Scroll {
                    offset[0] = (offset[0] + delta_x).clamp(0.0, max_x);
                }
                if style.overflow.y == taffy::Overflow::Scroll {
                    offset[1] = (offset[1] + delta_y).clamp(0.0, max_y);
                }
                if *offset != old {
                    self.projections.semantics_dirty = true;
                    self.scrollbar_activity.insert(node, Instant::now());
                    self.queue_scroll_event(node);
                    return true;
                }
            }
            let Some(parent) = self.node_store.tree.parent(node) else {
                return false;
            };
            node = parent;
        }
    }

    pub(super) fn scroll_node(&mut self, target: u32, x: f32, y: f32, relative: bool) -> bool {
        let Some(&node) = self.node_store.solid_to_node.get(&target) else {
            return false;
        };
        let Ok(style) = self.node_store.tree.style(node) else {
            return false;
        };
        let scroll_x = style.overflow.x == taffy::Overflow::Scroll;
        let scroll_y = style.overflow.y == taffy::Overflow::Scroll;
        if !scroll_x && !scroll_y {
            return false;
        }
        let Ok(layout) = self.node_store.tree.layout(node) else {
            return false;
        };
        let viewport_width =
            (layout.size.width - layout.border.left - layout.border.right).max(0.0);
        let viewport_height =
            (layout.size.height - layout.border.top - layout.border.bottom).max(0.0);
        let max_x = (layout.content_size.width - viewport_width).max(0.0);
        let max_y = (layout.content_size.height - viewport_height).max(0.0);
        let offset = self.scroll_offsets.entry(node).or_insert([0.0, 0.0]);
        let old = *offset;
        if scroll_x && x.is_finite() {
            offset[0] = (if relative { offset[0] + x } else { x }).clamp(0.0, max_x);
        }
        if scroll_y && y.is_finite() {
            offset[1] = (if relative { offset[1] + y } else { y }).clamp(0.0, max_y);
        }
        let changed = *offset != old;
        if changed {
            self.scrollbar_activity.insert(node, Instant::now());
            self.queue_scroll_event(node);
        }
        self.projections.semantics_dirty |= changed;
        changed
    }

    pub(super) fn queue_scroll_event(&mut self, node: NodeId) {
        let Some(target) = self.node_store.solid_id_for_node(node) else {
            return;
        };
        if !self.has_listener_in_chain(target, event::SCROLL) {
            return;
        }
        let offset = self.scroll_offsets.get(&node).copied().unwrap_or([0.0; 2]);
        self.pending_scroll_events.insert(target, offset);
    }

    pub(super) fn dispatch_scroll_changes(&mut self) -> bool {
        if self.pending_scroll_events.is_empty() {
            return false;
        }
        let events = std::mem::take(&mut self.pending_scroll_events)
            .into_iter()
            .map(|(target, offset)| {
                let mut data = [0.0; event_data::LEN];
                data[event_data::SCROLL_X as usize] = f64::from(offset[0]);
                data[event_data::SCROLL_Y as usize] = f64::from(offset[1]);
                HostEvent::Node(HostNodeEvent {
                    target,
                    event_code: event::SCROLL,
                    event_id: 0,
                    cancellable: false,
                    payload: NodeEventPayload::Numeric(data),
                })
            })
            .collect::<Vec<_>>();
        match self.js.dispatch_host_frame(&events) {
            Ok(_) => true,
            Err(error) => {
                tracing::warn!(?error, "scroll observation dispatch failed");
                false
            }
        }
    }

    pub(super) fn clamp_scroll_offsets(&mut self, placed: &[PlacedNode]) -> bool {
        let mut changed = Vec::new();
        for item in placed {
            let Some(offset) = self.scroll_offsets.get_mut(&item.node_id) else {
                continue;
            };
            let old = *offset;
            offset[0] = offset[0].clamp(0.0, item.scroll.range[0]);
            offset[1] = offset[1].clamp(0.0, item.scroll.range[1]);
            if *offset != old {
                changed.push(item.node_id);
            }
        }
        for node in &changed {
            self.queue_scroll_event(*node);
        }
        !changed.is_empty()
    }

    pub(super) fn text_selection_scroll_delta(&self) -> Option<(u32, f32, f32)> {
        if self.input.pointer_buttons & 1 == 0 {
            return None;
        }
        let active = self.text_selection.active.as_ref()?;
        // Autoscroll belongs to the endpoint currently following the pointer.
        // The stable anchor can live in a different scroll container during
        // a cross-panel selection.
        let target = active.focus_target;
        let mut node = *self.node_store.solid_to_node.get(&target)?;
        let pointer = [
            self.input.pointer_position.0 as f32,
            self.input.pointer_position.1 as f32,
        ];
        let axis_delta = |position: f32, start: f32, end: f32| {
            let outside = if position < start {
                position - start
            } else if position > end {
                position - end
            } else {
                0.0
            };
            if outside == 0.0 {
                0.0
            } else {
                outside.signum() * (outside.abs() * 0.35).clamp(4.0, 40.0)
            }
        };

        loop {
            let style = self.node_store.tree.style(node).ok()?;
            let scroll_x = style.overflow.x == taffy::Overflow::Scroll;
            let scroll_y = style.overflow.y == taffy::Overflow::Scroll;
            if (scroll_x || scroll_y)
                && let (Some(rect), Ok(layout)) = (
                    self.placed_rects.get(&node),
                    self.node_store.tree.layout(node),
                )
            {
                let x0 = rect[0] + layout.border.left;
                let y0 = rect[1] + layout.border.top;
                let x1 = rect[2] - layout.border.right;
                let y1 = rect[3] - layout.border.bottom;
                let dx = if scroll_x {
                    axis_delta(pointer[0], x0, x1)
                } else {
                    0.0
                };
                let dy = if scroll_y {
                    axis_delta(pointer[1], y0, y1)
                } else {
                    0.0
                };
                if dx != 0.0 || dy != 0.0 {
                    return Some((target, dx, dy));
                }
            }
            node = self.node_store.tree.parent(node)?;
        }
    }

    pub(super) fn arm_text_selection_autoscroll(&mut self) {
        self.text_selection.next_scroll = self
            .text_selection_scroll_delta()
            .is_some()
            .then(Instant::now);
    }

    pub(super) fn tick_text_selection_autoscroll(&mut self) -> bool {
        let Some(deadline) = self.text_selection.next_scroll else {
            return false;
        };
        if Instant::now() < deadline {
            return false;
        }
        let Some((target, dx, dy)) = self.text_selection_scroll_delta() else {
            self.text_selection.next_scroll = None;
            return false;
        };
        let changed = self.scroll_nearest(target, dx, dy);
        self.text_selection.next_scroll =
            changed.then(|| Instant::now() + Duration::from_millis(50));
        changed
    }
}

fn scrollbar_auto_opacity(elapsed: Duration, delay: Duration, fade: Duration, held: bool) -> f32 {
    if held {
        return 1.0;
    }
    if elapsed <= delay {
        return 1.0;
    }
    if fade.is_zero() {
        return 0.0;
    }
    (1.0 - elapsed.saturating_sub(delay).as_secs_f32() / fade.as_secs_f32()).clamp(0.0, 1.0)
}

#[cfg(test)]
mod scrollbar_visibility_tests {
    use super::scrollbar_auto_opacity;
    use std::time::Duration;

    #[test]
    fn auto_opacity_waits_then_fades_and_supports_immediate_hide() {
        let delay = Duration::from_millis(700);
        let fade = Duration::from_millis(200);
        assert_eq!(
            scrollbar_auto_opacity(Duration::ZERO, delay, fade, false),
            1.0
        );
        assert_eq!(
            scrollbar_auto_opacity(Duration::from_millis(700), delay, fade, false),
            1.0
        );
        assert!(
            (scrollbar_auto_opacity(Duration::from_millis(800), delay, fade, false) - 0.5).abs()
                < 0.001
        );
        assert_eq!(
            scrollbar_auto_opacity(Duration::from_millis(901), delay, fade, false),
            0.0
        );
        assert_eq!(
            scrollbar_auto_opacity(Duration::from_millis(701), delay, Duration::ZERO, false),
            0.0
        );
        assert_eq!(
            scrollbar_auto_opacity(Duration::from_secs(10), delay, fade, true),
            1.0,
            "hover and drag hold the scrollbar fully visible"
        );
    }
}
