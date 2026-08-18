use super::*;

impl Applier {
    pub(super) fn dispatch_image_resource_result(
        &mut self,
        node: taffy::NodeId,
        url: &str,
        result: &crate::asset_cache::RasterAsset,
    ) {
        let Some(&target) = self.node_store.node_to_solid.get(&node) else {
            return;
        };
        match result {
            Ok(_) => {
                let payload = serde_json::json!({ "url": url }).to_string();
                self.dispatch_json(target, event::RESOURCEREADY, &payload);
            }
            Err(error) => {
                let payload = serde_json::json!({ "url": url, "error": error }).to_string();
                self.dispatch_json(target, event::RESOURCEERROR, &payload);
            }
        }
    }

    pub(super) fn handle_pointer_up(
        &mut self,
        pointer: wabou_shell::PointerEvent,
    ) -> EventResponse {
        let (x, y) = (pointer.position.x, pointer.position.y);
        let button = pointer.button.unwrap_or(PointerButton::Primary);
        self.input.pointer_position = (x, y);
        self.input.pointer_buttons = pointer.buttons;
        if let Some(drag) = self.scroll.drag.take() {
            self.scroll.activity.insert(drag.node, Instant::now());
            return Self::response(true);
        }
        if button == PointerButton::Primary
            && let Some((down_x, down_y)) = self.input.pointer_down_position
        {
            let dx = x - down_x;
            let dy = y - down_y;
            self.input.pointer_dragged |= dx * dx + dy * dy > CLICK_DRAG_THRESHOLD_SQUARED;
        }
        let target = self.input.hit_test(x, y);
        let captured = self.input.pointer_down_target;
        let mut changed = captured.is_some_and(|captured| {
            self.handle_widget_event(captured, &UiEvent::Pointer(pointer))
                .is_some_and(|response| response.handled || response.request_redraw)
        });
        if button == PointerButton::Primary {
            changed |= self.extend_text_selection(target, x, y);
            self.text_selection.next_scroll = None;
            if self.input.pointer_dragged {
                self.text_selection.last_click = None;
            }
        }
        changed |= target.is_some_and(|target| {
            self.dispatch_pointer(target, event::POINTERUP, Some(button), pointer.modifiers)
        });
        if let Some(target) = target
            && button == PointerButton::Primary
            && !self.input.pointer_dragged
            && Some(target) == self.input.pointer_down_target
        {
            let mut data = [0.0; event_data::LEN];
            data[event_data::CLIENT_X as usize] = self.input.pointer_position.0;
            data[event_data::CLIENT_Y as usize] = self.input.pointer_position.1;
            let local = self.input.local_position(target, x, y);
            data[event_data::OFFSET_X as usize] = local.0;
            data[event_data::OFFSET_Y as usize] = local.1;
            data[event_data::BUTTON as usize] = Self::web_button(button) as f64;
            data[event_data::BUTTONS as usize] =
                Self::web_buttons(self.input.pointer_buttons) as f64;
            data[event_data::MODS as usize] = pointer.modifiers.bits() as f64;
            let (dispatched, prevented) =
                self.dispatch_cancellable_numeric(target, event::CLICK, data);
            changed |= dispatched;
            if !prevented {
                changed |= self.open_link_default(target);
            }
        }
        self.input.pointer_down_target.take();
        self.input.pointer_down_position = None;
        self.input.pointer_dragged = false;
        changed |= self.sync_text_selection_change();
        Self::response(changed)
    }

    pub(super) fn handle_pointer_cancel(
        &mut self,
        pointer: wabou_shell::PointerEvent,
    ) -> EventResponse {
        if let Some(drag) = self.scroll.drag.take() {
            self.scroll.activity.insert(drag.node, Instant::now());
            Self::response(true)
        } else {
            Self::response(self.cancel_pointer_gesture(pointer))
        }
    }

    pub(super) fn handle_pointer_down(
        &mut self,
        pointer: wabou_shell::PointerEvent,
    ) -> EventResponse {
        let (x, y) = (pointer.position.x, pointer.position.y);
        let button = pointer.button.unwrap_or(PointerButton::Primary);
        self.input.pointer_position = (x, y);
        self.input.pointer_buttons = pointer.buttons;
        if button == PointerButton::Primary
            && let Some((node, target)) = self.scrollbar_at(x, y)
            && let Some(hit) = self.scroll.hits.iter().find(|hit| hit.node == node)
        {
            self.scroll.activity.insert(node, Instant::now());
            if target.part != ScrollbarPart::Thumb {
                let index = usize::from(target.axis == ScrollAxis::Vertical);
                let viewport = match target.axis {
                    ScrollAxis::Horizontal => hit.placed.scroll.port[2] - hit.placed.scroll.port[0],
                    ScrollAxis::Vertical => hit.placed.scroll.port[3] - hit.placed.scroll.port[1],
                };
                let direction = if target.part == ScrollbarPart::TrackBefore {
                    -1.0
                } else {
                    1.0
                };
                let offset = self.scroll.offsets.entry(node).or_insert([0.0; 2]);
                offset[index] = (offset[index] + direction * viewport)
                    .clamp(0.0, hit.placed.scroll.range[index]);
                self.queue_scroll_event(node);
                self.projections.semantics_dirty = true;
                return Self::response(true);
            }
            let local = hit.transform.inverse() * Point::new(x, y);
            self.scroll.drag = Some(ScrollbarDrag {
                node,
                axis: target.axis,
                last_position: match target.axis {
                    ScrollAxis::Horizontal => local.x,
                    ScrollAxis::Vertical => local.y,
                },
            });
            return Self::response(true);
        }

        let target = self.input.hit_test(x, y);
        self.input.pointer_down_target = target;
        self.input.pointer_down_position = Some((x, y));
        self.input.pointer_dragged = false;
        let mut changed = self.set_focused_target(self.pointer_focus_target(target));
        if button == PointerButton::Primary {
            self.text_selection.next_scroll = None;
            changed |= target
                .is_some_and(|target| self.begin_text_selection(target, x, y, pointer.modifiers));
        }
        if let Some(target) = target
            && let Some(mut response) = self.handle_widget_event(target, &UiEvent::Pointer(pointer))
        {
            response.text_input = Some(self.is_text_input_target(target));
            return response;
        }
        let handled = changed
            | target.is_some_and(|target| {
                self.dispatch_pointer(target, event::POINTERDOWN, Some(button), pointer.modifiers)
            });
        EventResponse {
            handled,
            request_redraw: handled,
            consume_key_text: false,
            text_input: Some(target.is_some_and(|target| self.is_text_input_target(target))),
            clipboard: None,
        }
    }

    pub(super) fn handle_pointer_move(
        &mut self,
        pointer: wabou_shell::PointerEvent,
    ) -> EventResponse {
        let (x, y) = (pointer.position.x, pointer.position.y);
        self.input.pointer_buttons = pointer.buttons;
        self.input.pointer_position = (x, y);
        let hovered_scrollbar = self
            .scrollbar_at(x, y)
            .map(|(node, target)| (node, target.axis))
            .or_else(|| self.scrollbar_edge_at(x, y));
        let scrollbar_hover_changed = hovered_scrollbar != self.scroll.hovered;
        let previous_hover = self.scroll.hovered;
        self.scroll.hovered = hovered_scrollbar;
        if let Some((node, _)) = hovered_scrollbar {
            self.scroll.activity.insert(node, Instant::now());
        }
        if scrollbar_hover_changed && let Some((node, _)) = previous_hover {
            self.scroll.activity.insert(node, Instant::now());
        }
        if self.scroll.drag.is_some() {
            let changed = self.drag_scrollbar(x, y);
            return EventResponse {
                handled: true,
                request_redraw: changed || scrollbar_hover_changed,
                ..EventResponse::IGNORED
            };
        }
        if pointer.buttons & 1 != 0
            && let Some((down_x, down_y)) = self.input.pointer_down_position
        {
            let dx = x - down_x;
            let dy = y - down_y;
            self.input.pointer_dragged |= dx * dx + dy * dy > CLICK_DRAG_THRESHOLD_SQUARED;
        }
        let target = self.input.hit_test(x, y);
        let mut changed = scrollbar_hover_changed;
        if let Some(captured) = self.input.pointer_down_target
            && let Some(response) = self.handle_widget_event(captured, &UiEvent::Pointer(pointer))
        {
            changed |= response.handled || response.request_redraw;
        }
        if pointer.buttons & 1 != 0 {
            changed |= self.extend_text_selection(target, x, y);
            self.arm_text_selection_autoscroll();
        }
        if target != self.input.hovered_target {
            if let Some(old) = self.input.hovered_target {
                changed |= self.dispatch_pointer(old, event::POINTERLEAVE, None, pointer.modifiers);
            }
            if let Some(new) = target {
                changed |= self.dispatch_pointer(new, event::POINTERENTER, None, pointer.modifiers);
            }
            self.input.hovered_target = target;
        }
        if let Some(target) = target {
            changed |= self.dispatch_pointer(target, event::POINTERMOVE, None, pointer.modifiers);
        }
        Self::response(changed)
    }

    pub(super) fn handle_wheel_event(&mut self, wheel: wabou_shell::WheelEvent) -> EventResponse {
        self.input.pointer_position = (wheel.position.x, wheel.position.y);
        // Wheel events carry their own pointer position. Re-hit-test it even
        // when the cached hover target is still alive: semantic automation,
        // trackpads and virtualized content can move the wheel position
        // without first delivering a pointer-move event.
        self.input.hovered_target = self
            .input
            .hit_test(self.input.pointer_position.0, self.input.pointer_position.1)
            .or_else(|| {
                self.input
                    .hovered_target
                    .filter(|target| self.node_store.solid_to_node.contains_key(target))
            });
        let Some(target) = self.input.hovered_target else {
            return EventResponse::IGNORED;
        };

        // Rust widgets, such as terminal scrollback, get first refusal before
        // JavaScript listeners and native overflow scrolling.
        if let Some(response) = self.handle_widget_event(target, &UiEvent::Wheel(wheel)) {
            return response;
        }

        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = self.input.pointer_position.0;
        data[event_data::CLIENT_Y as usize] = self.input.pointer_position.1;
        let local = self.input.local_position(
            target,
            self.input.pointer_position.0,
            self.input.pointer_position.1,
        );
        data[event_data::OFFSET_X as usize] = local.0;
        data[event_data::OFFSET_Y as usize] = local.1;
        data[event_data::MODS as usize] = wheel.modifiers.bits() as f64;
        data[event_data::DELTA_X as usize] = wheel.delta_x;
        data[event_data::DELTA_Y as usize] = wheel.delta_y;
        let (dispatched, prevented) = self.dispatch_cancellable_numeric(target, event::WHEEL, data);
        let scrolled =
            !prevented && self.scroll_nearest(target, wheel.delta_x as f32, wheel.delta_y as f32);
        Self::response(dispatched || scrolled)
    }

    pub(super) fn rebuild_hit_geometry(&mut self, placed: &[PlacedNode]) {
        self.input.hit_items.clear();
        self.scroll.hits.clear();
        self.scroll.metrics.clear();
        let atoms = self.atoms.borrow();
        let placed_by_id: HashMap<_, _> = placed.iter().map(|node| (node.node_id, node)).collect();
        let mut transforms = HashMap::with_capacity(placed.len());
        let mut clip_chains: HashMap<NodeId, Vec<HitClip>> = HashMap::with_capacity(placed.len());
        let mut content_hits = HashMap::new();
        let mut scrollbar_hits = HashMap::new();
        for node in placed {
            if node.scroll.range.iter().any(|range| *range > 0.5) {
                self.scroll.metrics.insert(node.node_id, node.scroll);
            }
            let is_non_interactive_leaf = !node.paint.pointer_events
                && self
                    .node_store
                    .children
                    .get(&node.node_id)
                    .is_none_or(Vec::is_empty)
                && !node.scroll.range.iter().any(|range| *range > 0.5);
            if is_non_interactive_leaf {
                continue;
            }
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
            let inert =
                subtree_has_attribute(&self.node_store, &atoms, node.node_id, "inert", None);
            if node.paint.pointer_events
                && !inert
                && let Some(&solid_id) = self.node_store.node_to_solid.get(&node.node_id)
            {
                content_hits.insert(
                    node.node_id,
                    HitNode {
                        solid_id,
                        rect: node.rect,
                        transform,
                        clips: clips.clone(),
                        pointer_events: true,
                    },
                );
            }
            if !inert
                && node.paint.scrollbar.visibility != ScrollbarVisibility::Hidden
                && node.scroll.range.iter().any(|range| *range > 0.5)
            {
                let hit = ScrollbarHit {
                    node: node.node_id,
                    placed: node.clone(),
                    transform,
                };
                self.scroll.hits.push(hit.clone());
                if node.scroll.opacity > 0.0 {
                    scrollbar_hits.insert(node.node_id, hit);
                }
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

    pub(super) fn has_listener_in_chain(&self, solid_id: u32, code: u8) -> bool {
        self.listener_target_in_chain(solid_id, code).is_some()
    }

    pub(super) fn listener_target_in_chain(&self, mut solid_id: u32, code: u8) -> Option<u32> {
        loop {
            if self
                .input
                .listeners
                .get(&solid_id)
                .is_some_and(|events| events.contains(code))
            {
                return Some(solid_id);
            }
            let node = self.node_store.solid_to_node.get(&solid_id)?;
            let parent = self.node_store.tree.parent(*node)?;
            solid_id = self.node_store.solid_id_for_node(parent)?;
        }
    }

    pub(super) fn dispatch_pointer(
        &mut self,
        target: u32,
        code: u8,
        button: Option<PointerButton>,
        modifiers: Modifiers,
    ) -> bool {
        let Some(target) = self.listener_target_in_chain(target, code) else {
            return false;
        };
        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = self.input.pointer_position.0;
        data[event_data::CLIENT_Y as usize] = self.input.pointer_position.1;
        let local = self.input.local_position(
            target,
            self.input.pointer_position.0,
            self.input.pointer_position.1,
        );
        data[event_data::OFFSET_X as usize] = local.0;
        data[event_data::OFFSET_Y as usize] = local.1;
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
                let changes = widget.focus_changed(false);
                self.invalidate_widget_changes(changes);
            }
            changed |= self.dispatch_json(old, event::BLUR, "");
            changed |= self.dispatch_json(old, event::FOCUSOUT, "");
        }
        if let Some(new) = target {
            if self.input.window_focused
                && let Some(node) = self.node_store.solid_to_node.get(&new)
                && let Some(widget) = self.widget_manager.widgets.get_mut(node)
            {
                let changes = widget.focus_changed(true);
                self.invalidate_widget_changes(changes);
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
            let changes = widget.focus_changed(focused);
            self.invalidate_widget_changes(changes);
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
