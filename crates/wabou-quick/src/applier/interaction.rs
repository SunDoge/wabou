use super::*;

impl Applier {
    pub(super) fn rebuild_hit_geometry(&mut self, placed: &[PlacedNode]) {
        self.input.hit_items.clear();
        self.scrollbar_hits.clear();
        let placed_by_id: HashMap<_, _> = placed.iter().map(|node| (node.node_id, node)).collect();
        let mut transforms = HashMap::with_capacity(placed.len());
        let mut clip_chains: HashMap<NodeId, Vec<HitClip>> = HashMap::with_capacity(placed.len());
        let mut content_hits = HashMap::new();
        let mut scrollbar_hits = HashMap::new();
        for node in placed {
            let parent_transform = node
                .parent_node_id
                .and_then(|parent| transforms.get(&parent).copied())
                .unwrap_or(Affine::IDENTITY);
            let transform = wabou_shell::scene::resolve_node_transform(node, parent_transform);
            let mut clips = node
                .parent_node_id
                .and_then(|parent| clip_chains.get(&parent).cloned())
                .unwrap_or_default();
            if let Some(parent) = node.parent_node_id
                && let Some(parent_node) = placed_by_id.get(&parent)
                && let Some(rect) = parent_node.own_clip
            {
                clips.push(HitClip {
                    rect,
                    radius: parent_node.own_clip_radius,
                    transform: transforms[&parent],
                });
            }
            if let Some(&solid_id) = self.node_store.node_to_solid.get(&node.node_id) {
                content_hits.insert(
                    node.node_id,
                    HitNode {
                        solid_id,
                        rect: node.rect,
                        transform,
                        clips: clips.clone(),
                        pointer_events: node.paint.pointer_events,
                    },
                );
            }
            if node.scroll.opacity > 0.0 && node.scroll.range.iter().any(|range| *range > 0.5) {
                let hit = ScrollbarHit {
                    node: node.node_id,
                    placed: node.clone(),
                    transform,
                };
                self.scrollbar_hits.push(hit.clone());
                scrollbar_hits.insert(node.node_id, hit);
            }
            transforms.insert(node.node_id, transform);
            clip_chains.insert(node.node_id, clips);
        }
        for event in subtree_events(placed) {
            match event {
                SubtreeEvent::Enter(node) => {
                    if let Some(hit) = content_hits.remove(&node.node_id) {
                        self.input.hit_items.push(HitItem::Content(hit));
                    }
                }
                SubtreeEvent::Exit(node) => {
                    if let Some(hit) = scrollbar_hits.remove(&node.node_id) {
                        self.input.hit_items.push(HitItem::Scrollbar(Box::new(hit)));
                    }
                }
            }
        }
    }

    pub(super) fn update_scrollbar_visuals(&mut self, placed: &mut [PlacedNode]) {
        let now = Instant::now();
        self.scrollbar_activity.retain(|_, started| {
            now.duration_since(*started) < SCROLLBAR_FADE_DELAY + SCROLLBAR_FADE_DURATION
        });
        for node in placed {
            node.scroll.opacity = match node.paint.scrollbar.visibility {
                ScrollbarVisibility::Always => 1.0,
                ScrollbarVisibility::Hidden => 0.0,
                ScrollbarVisibility::Auto => self
                    .scrollbar_activity
                    .get(&node.node_id)
                    .map_or(0.0, |started| {
                        let elapsed = now.duration_since(*started);
                        if elapsed <= SCROLLBAR_FADE_DELAY {
                            1.0
                        } else {
                            1.0 - (elapsed - SCROLLBAR_FADE_DELAY).as_secs_f32()
                                / SCROLLBAR_FADE_DURATION.as_secs_f32()
                        }
                    })
                    .clamp(0.0, 1.0),
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
    }

    pub(super) fn rebuild_semantic_snapshot(&mut self, placed: &[PlacedNode]) {
        semantics::rebuild(self, placed);
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
        }
        self.projections.semantics_dirty |= changed;
        changed
    }

    pub(super) fn text_selection_scroll_delta(&self) -> Option<(u32, f32, f32)> {
        if self.input.pointer_buttons & 1 == 0 {
            return None;
        }
        let active = self.active_text_selection.as_ref()?;
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
        self.next_text_selection_scroll = self
            .text_selection_scroll_delta()
            .is_some()
            .then(Instant::now);
    }

    pub(super) fn tick_text_selection_autoscroll(&mut self) -> bool {
        if !self
            .next_text_selection_scroll
            .is_some_and(|deadline| Instant::now() >= deadline)
        {
            return false;
        }
        let Some((target, dx, dy)) = self.text_selection_scroll_delta() else {
            self.next_text_selection_scroll = None;
            return false;
        };
        let changed = self.scroll_nearest(target, dx, dy);
        self.next_text_selection_scroll =
            changed.then(|| Instant::now() + Duration::from_millis(50));
        changed
    }

    pub(super) fn has_listener_in_chain(&self, mut solid_id: u32, code: u8) -> bool {
        loop {
            if self
                .input
                .listeners
                .get(&solid_id)
                .is_some_and(|events| events.contains(code))
            {
                return true;
            }
            let Some(&node) = self.node_store.solid_to_node.get(&solid_id) else {
                return false;
            };
            let Some(parent) = self.node_store.tree.parent(node) else {
                return false;
            };
            let Some(parent_id) = self.node_store.solid_id_for_node(parent) else {
                return false;
            };
            solid_id = parent_id;
        }
    }

    pub(super) fn dispatch_pointer(
        &mut self,
        target: u32,
        code: u8,
        button: Option<PointerButton>,
        modifiers: Modifiers,
    ) -> bool {
        if !self.has_listener_in_chain(target, code) {
            return false;
        }
        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = self.input.pointer_position.0;
        data[event_data::CLIENT_Y as usize] = self.input.pointer_position.1;
        data[event_data::BUTTON as usize] = button.map_or(0, Self::web_button) as f64;
        data[event_data::BUTTONS as usize] = Self::web_buttons(self.input.pointer_buttons) as f64;
        data[event_data::MODS as usize] = modifiers.bits() as f64;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id: 0,
            cancellable: false,
            payload: NodeEventPayload::Numeric(data),
        });
        if let Err(error) = self.js.dispatch_host_frame(&[event]) {
            tracing::warn!(?error, target, code, "event dispatch failed");
            return false;
        }
        true
    }

    pub(super) fn dispatch_cancellable_numeric(
        &mut self,
        target: u32,
        code: u8,
        data: [f64; event_data::LEN],
    ) -> (bool, bool) {
        if !self.has_listener_in_chain(target, code) {
            return (false, false);
        }
        self.input.next_host_event_id = self.input.next_host_event_id.wrapping_add(1).max(1);
        let event_id = self.input.next_host_event_id;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id,
            cancellable: true,
            payload: NodeEventPayload::Numeric(data),
        });
        match self.js.dispatch_host_frame(&[event]) {
            Ok(disposition) => (true, disposition.is_prevented(event_id)),
            Err(error) => {
                tracing::warn!(?error, target, code, "event dispatch failed");
                (false, false)
            }
        }
    }

    pub(super) fn dispatch_cancellable_json(
        &mut self,
        target: u32,
        code: u8,
        payload: String,
    ) -> (bool, bool) {
        if !self.has_listener_in_chain(target, code) {
            return (false, false);
        }
        self.input.next_host_event_id = self.input.next_host_event_id.wrapping_add(1).max(1);
        let event_id = self.input.next_host_event_id;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id,
            cancellable: true,
            payload: NodeEventPayload::Json(payload),
        });
        match self.js.dispatch_host_frame(&[event]) {
            Ok(disposition) => (true, disposition.is_prevented(event_id)),
            Err(error) => {
                tracing::warn!(?error, target, code, "event dispatch failed");
                (false, false)
            }
        }
    }

    pub(super) fn link_url(&self, mut target: u32) -> Option<String> {
        let atoms = self.atoms.borrow();
        loop {
            let node = *self.node_store.solid_to_node.get(&target)?;
            if let Some(declared) = self.node_store.declared.get(&node)
                && declared.tag.and_then(|tag| atoms.resolve(tag)) == Some("a")
                && let Some((_, href)) = declared
                    .attrs
                    .iter()
                    .find(|(name, _)| atoms.resolve(**name) == Some("href"))
            {
                return Some(href.to_string());
            }
            let parent = self.node_store.tree.parent(node)?;
            target = self.node_store.solid_id_for_node(parent)?;
        }
    }

    pub(super) fn open_link_default(&mut self, target: u32) -> bool {
        let Some(raw) = self.link_url(target) else {
            return false;
        };
        self.pending_host_actions
            .borrow_mut()
            .push_back(wabou_shell::HostAction::OpenUrl(raw));
        true
    }

    /// Translate Wabou's compact native button representation only at the
    /// Solid/Web compatibility boundary.
    pub(super) fn web_button(button: PointerButton) -> u8 {
        match button {
            PointerButton::Primary => 0,
            PointerButton::Auxiliary => 1,
            PointerButton::Secondary => 2,
            PointerButton::Other(index) => index.min(u8::MAX as u16) as u8,
        }
    }

    pub(super) fn web_buttons(native: u32) -> u32 {
        (native & 1) | ((native & 2) << 1) | ((native & 4) >> 1) | (native & !7)
    }

    pub(super) fn response(handled: bool) -> EventResponse {
        if handled {
            EventResponse::handled()
        } else {
            EventResponse::IGNORED
        }
    }

    pub(super) fn dispatch_json(&mut self, target: u32, code: u8, payload: &str) -> bool {
        if !self.has_listener_in_chain(target, code) {
            return false;
        }
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id: 0,
            cancellable: false,
            payload: NodeEventPayload::Json(payload.to_owned()),
        });
        if let Err(error) = self.js.dispatch_host_frame(&[event]) {
            tracing::warn!(?error, target, code, "event dispatch failed");
            return false;
        }
        true
    }

    pub(super) fn is_text_input_target(&self, target: u32) -> bool {
        let Some(&node) = self.node_store.solid_to_node.get(&target) else {
            return false;
        };
        if self
            .widget_manager
            .widgets
            .get(&node)
            .is_some_and(|widget| widget.accepts_focus())
        {
            return true;
        }
        let atoms = self.atoms.borrow();
        self.node_store
            .declared
            .get(&node)
            .and_then(|decl| decl.tag)
            .and_then(|tag| atoms.resolve(tag))
            == Some("input")
    }

    pub(super) fn set_focused_target(&mut self, target: Option<u32>) -> bool {
        let old = self.input.focused_target;
        if old == target {
            return false;
        }
        self.projections.semantics_dirty = true;
        self.input.focused_target = target;
        let mut changed = false;
        if let Some(old) = old {
            if self.input.window_focused
                && let Some(node) = self.node_store.solid_to_node.get(&old)
                && let Some(widget) = self.widget_manager.widgets.get_mut(node)
            {
                widget.focus_changed(false);
            }
            changed |= self.dispatch_json(old, event::BLUR, "");
            changed |= self.dispatch_json(old, event::FOCUSOUT, "");
        }
        if let Some(new) = target {
            if self.input.window_focused
                && let Some(node) = self.node_store.solid_to_node.get(&new)
                && let Some(widget) = self.widget_manager.widgets.get_mut(node)
            {
                widget.focus_changed(true);
            }
            changed |= self.dispatch_json(new, event::FOCUS, "");
            changed |= self.dispatch_json(new, event::FOCUSIN, "");
        }
        changed
    }

    pub(super) fn set_window_focused(&mut self, focused: bool) -> bool {
        if self.input.window_focused == focused {
            return false;
        }
        self.input.window_focused = focused;
        let Some(target) = self.input.focused_target else {
            return false;
        };
        let mut changed = false;
        if let Some(node) = self.node_store.solid_to_node.get(&target)
            && let Some(widget) = self.widget_manager.widgets.get_mut(node)
        {
            widget.focus_changed(focused);
            changed = true;
        }
        let (focus, focus_within) = if focused {
            (event::FOCUS, event::FOCUSIN)
        } else {
            (event::BLUR, event::FOCUSOUT)
        };
        changed |= self.dispatch_json(target, focus, "");
        changed |= self.dispatch_json(target, focus_within, "");
        changed
    }
}
