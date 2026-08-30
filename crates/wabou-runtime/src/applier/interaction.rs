use super::*;

impl Applier {
    fn update_hover_target(
        &mut self,
        pointer_id: gpui_shell::PointerId,
        target: Option<NodeKey>,
        modifiers: Modifiers,
    ) -> bool {
        let old = self
            .interaction
            .input
            .pointer_routes
            .get(&pointer_id)
            .and_then(|route| route.hovered_target);
        if target == old {
            return false;
        }
        // Hit testing deliberately returns the deepest painted node, while
        // hover state belongs to the nearest interactive ancestor. Crossing
        // children of one control must not make that control leave and enter
        // itself (for example, moving from a button label to its status dot).
        let hover_boundary = |this: &Self, target: Option<NodeKey>| {
            target.and_then(|target| {
                this.listener_target_in_chain(target, event::POINTERENTER)
                    .or_else(|| this.listener_target_in_chain(target, event::POINTERLEAVE))
            })
        };
        let old_boundary = hover_boundary(self, old);
        let new_boundary = hover_boundary(self, target);
        let mut changed = false;
        if let Some(old) = old {
            changed |= self.dispatch_pointer(old, event::POINTEROUT, None, modifiers);
            if old_boundary != new_boundary {
                changed |= self.dispatch_pointer(old, event::POINTERLEAVE, None, modifiers);
            }
        }
        if let Some(new) = target {
            changed |= self.dispatch_pointer(new, event::POINTEROVER, None, modifiers);
            if old_boundary != new_boundary {
                changed |= self.dispatch_pointer(new, event::POINTERENTER, None, modifiers);
            }
        }
        self.interaction
            .input
            .pointer_routes
            .entry(pointer_id)
            .or_default()
            .hovered_target = target;
        if self.interaction.input.pointer_properties.primary {
            self.interaction.input.hovered_target = target;
        }
        changed
    }

    pub(super) fn handle_pointer_enter(
        &mut self,
        pointer: gpui_shell::PointerEvent,
    ) -> EventResponse {
        let (x, y) = (pointer.position.x, pointer.position.y);
        self.interaction.input.update_pointer(&pointer);
        let target = self.interaction.input.hit_test(x, y);
        Self::response(self.update_hover_target(pointer.properties.id, target, pointer.modifiers))
    }

    pub(super) fn handle_pointer_leave(
        &mut self,
        pointer: gpui_shell::PointerEvent,
    ) -> EventResponse {
        self.interaction.input.update_pointer(&pointer);
        let mut changed = self.update_hover_target(pointer.properties.id, None, pointer.modifiers);
        self.interaction
            .input
            .pointer_routes
            .remove(&pointer.properties.id);
        if let Some((node, _)) = self.interaction.scroll.hovered.take() {
            self.interaction
                .scroll
                .activity
                .insert(node, Instant::now());
            changed = true;
        }
        Self::response(changed)
    }

    pub(super) fn dispatch_image_resource_error(
        &mut self,
        node: taffy::NodeId,
        resource: Option<crate::ImageResourceHandle>,
        error: &str,
    ) {
        let Some(&target) = self.document.node_store.node_to_solid.get(&node) else {
            return;
        };
        let payload = serde_json::json!({ "resource": resource, "error": error }).to_string();
        self.dispatch_json(target, event::RESOURCEERROR, &payload);
    }

    pub(super) fn dispatch_image_resource_ready(
        &mut self,
        target: crate::protocol::NodeKey,
        resource: crate::ImageResourceHandle,
        width: f32,
        height: f32,
    ) {
        let payload = serde_json::json!({
            "resource": resource,
            "width": width,
            "height": height,
        })
        .to_string();
        self.dispatch_json(target, event::RESOURCEREADY, &payload);
    }

    pub(super) fn handle_pointer_up(&mut self, pointer: gpui_shell::PointerEvent) -> EventResponse {
        let (x, y) = (pointer.position.x, pointer.position.y);
        let button = pointer.button.unwrap_or(PointerButton::Primary);
        self.interaction.input.update_pointer(&pointer);
        if pointer.properties.primary
            && let Some(drag) = self.interaction.scroll.drag.take()
        {
            self.interaction
                .scroll
                .activity
                .insert(drag.node, Instant::now());
            return Self::response(true);
        }
        let pointer_id = pointer.properties.id;
        let (captured, dragged) = {
            let route = self
                .interaction
                .input
                .pointer_routes
                .entry(pointer_id)
                .or_default();
            if button == PointerButton::Primary
                && let Some((down_x, down_y)) = route.down_position
            {
                let dx = x - down_x;
                let dy = y - down_y;
                route.dragged |= dx * dx + dy * dy > CLICK_DRAG_THRESHOLD_SQUARED;
            }
            (route.down_target, route.dragged)
        };
        if pointer.properties.primary {
            self.interaction.input.pointer_down_target = captured;
            self.interaction.input.pointer_dragged = dragged;
        }
        let target = self.interaction.input.hit_test(x, y);
        let mut changed = captured.is_some_and(|captured| {
            self.handle_widget_event(captured, &UiEvent::Pointer(pointer))
                .is_some_and(|response| response.handled || response.request_redraw)
        });
        if button == PointerButton::Primary && pointer.properties.primary {
            changed |= self.extend_text_selection(target, x, y);
            self.interaction.text_selection.next_scroll = None;
            if dragged {
                self.interaction.text_selection.last_click = None;
            }
        }
        // A pressed JS node owns the gesture until release, matching native
        // controls and keeping drag cleanup deterministic outside its bounds.
        let release_target = captured.or(target);
        changed |= release_target.is_some_and(|target| {
            self.dispatch_pointer(target, event::POINTERUP, Some(button), pointer.modifiers)
        });
        let captured_click_target =
            captured.and_then(|target| self.listener_target_in_chain(target, event::CLICK));
        let click_target =
            target.and_then(|target| self.listener_target_in_chain(target, event::CLICK));
        if let Some(target) = click_target
            && button == PointerButton::Primary
            && pointer.properties.primary
            && !dragged
            && Some(target) == captured_click_target
        {
            let mut data = [0.0; event_data::LEN];
            data[event_data::CLIENT_X as usize] = self.interaction.input.pointer_position.0;
            data[event_data::CLIENT_Y as usize] = self.interaction.input.pointer_position.1;
            let local = self.interaction.input.local_position(target, x, y);
            data[event_data::OFFSET_X as usize] = local.0;
            data[event_data::OFFSET_Y as usize] = local.1;
            data[event_data::BUTTON as usize] = Self::web_button(button) as f64;
            data[event_data::BUTTONS as usize] =
                Self::web_buttons(self.interaction.input.pointer_buttons) as f64;
            data[event_data::MODS as usize] = pointer.modifiers.bits() as f64;
            Self::fill_pointer_properties(&mut data, pointer.properties);
            let (dispatched, _) = self.dispatch_cancellable_numeric(target, event::CLICK, data);
            changed |= dispatched;

            let now = Instant::now();
            let is_double_click = self.interaction.input.last_primary_click.is_some_and(
                |(time, last_target, last_x, last_y)| {
                    last_target == target
                        && now.duration_since(time) <= Duration::from_millis(400)
                        && (x - last_x).abs() <= 4.0
                        && (y - last_y).abs() <= 4.0
                },
            );
            if is_double_click {
                let (dispatched, _) =
                    self.dispatch_cancellable_numeric(target, event::DBLCLICK, data);
                changed |= dispatched;
                self.interaction.input.last_primary_click = None;
            } else {
                self.interaction.input.last_primary_click = Some((now, target, x, y));
            }
        }
        let captured_context_target =
            captured.and_then(|target| self.listener_target_in_chain(target, event::CONTEXTMENU));
        let context_target =
            target.and_then(|target| self.listener_target_in_chain(target, event::CONTEXTMENU));
        if let Some(target) = context_target
            && button == PointerButton::Secondary
            && Some(target) == captured_context_target
        {
            let mut data = [0.0; event_data::LEN];
            data[event_data::CLIENT_X as usize] = x;
            data[event_data::CLIENT_Y as usize] = y;
            let local = self.interaction.input.local_position(target, x, y);
            data[event_data::OFFSET_X as usize] = local.0;
            data[event_data::OFFSET_Y as usize] = local.1;
            data[event_data::BUTTON as usize] = Self::web_button(button) as f64;
            data[event_data::BUTTONS as usize] =
                Self::web_buttons(self.interaction.input.pointer_buttons) as f64;
            data[event_data::MODS as usize] = pointer.modifiers.bits() as f64;
            Self::fill_pointer_properties(&mut data, pointer.properties);
            let (dispatched, _) =
                self.dispatch_cancellable_numeric(target, event::CONTEXTMENU, data);
            changed |= dispatched;
        }
        if let Some(route) = self.interaction.input.pointer_routes.get_mut(&pointer_id) {
            route.down_target = None;
            route.down_position = None;
            route.dragged = false;
        }
        if pointer.properties.primary {
            self.interaction.input.pointer_down_target.take();
            self.interaction.input.pointer_down_position = None;
            self.interaction.input.pointer_dragged = false;
        }
        changed |= self.sync_text_selection_change();
        Self::response(changed)
    }

    pub(super) fn handle_pointer_cancel(
        &mut self,
        pointer: gpui_shell::PointerEvent,
    ) -> EventResponse {
        if pointer.properties.primary
            && let Some(drag) = self.interaction.scroll.drag.take()
        {
            self.interaction
                .scroll
                .activity
                .insert(drag.node, Instant::now());
            Self::response(true)
        } else {
            Self::response(self.cancel_pointer_gesture(pointer))
        }
    }

    pub(super) fn handle_pointer_down(
        &mut self,
        pointer: gpui_shell::PointerEvent,
    ) -> EventResponse {
        let (x, y) = (pointer.position.x, pointer.position.y);
        let button = pointer.button.unwrap_or(PointerButton::Primary);
        self.interaction.input.update_pointer(&pointer);
        if button == PointerButton::Primary
            && pointer.properties.primary
            && let Some((node, target)) = self.scrollbar_at(x, y)
            && let Some(hit) = self
                .interaction
                .scroll
                .hits
                .iter()
                .find(|hit| hit.node == node)
        {
            self.interaction
                .scroll
                .activity
                .insert(node, Instant::now());
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
                let offset = self
                    .interaction
                    .scroll
                    .offsets
                    .entry(node)
                    .or_insert([0.0; 2]);
                offset[index] = (offset[index] + direction * viewport)
                    .clamp(0.0, hit.placed.scroll.range[index]);
                self.queue_scroll_event(node);
                self.frame.projections.semantics_dirty = true;
                return Self::response(true);
            }
            let local = hit.transform.inverse() * Point::new(x, y);
            self.interaction.scroll.drag = Some(ScrollbarDrag {
                node,
                axis: target.axis,
                last_position: match target.axis {
                    ScrollAxis::Horizontal => local.x,
                    ScrollAxis::Vertical => local.y,
                },
            });
            return Self::response(true);
        }

        let target = self.interaction.input.hit_test(x, y);
        {
            let route = self
                .interaction
                .input
                .pointer_routes
                .entry(pointer.properties.id)
                .or_default();
            route.down_target = target;
            route.down_position = Some((x, y));
            route.dragged = false;
        }
        if pointer.properties.primary {
            self.interaction.input.pointer_down_target = target;
            self.interaction.input.pointer_down_position = Some((x, y));
            self.interaction.input.pointer_dragged = false;
        }
        self.interaction.use_pointer_modality();
        let mut changed = if pointer.properties.primary {
            self.set_focused_target(self.pointer_focus_target(target))
        } else {
            false
        };
        if button == PointerButton::Primary && pointer.properties.primary {
            self.interaction.text_selection.next_scroll = None;
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
        pointer: gpui_shell::PointerEvent,
    ) -> EventResponse {
        let (x, y) = (pointer.position.x, pointer.position.y);
        self.interaction.input.update_pointer(&pointer);
        let hovered_scrollbar = if pointer.properties.primary {
            self.scrollbar_at(x, y)
                .map(|(node, target)| (node, target.axis))
                .or_else(|| self.scrollbar_edge_at(x, y))
        } else {
            self.interaction.scroll.hovered
        };
        let scrollbar_hover_changed = hovered_scrollbar != self.interaction.scroll.hovered;
        let previous_hover = self.interaction.scroll.hovered;
        self.interaction.scroll.hovered = hovered_scrollbar;
        if let Some((node, _)) = hovered_scrollbar {
            self.interaction
                .scroll
                .activity
                .insert(node, Instant::now());
        }
        if scrollbar_hover_changed && let Some((node, _)) = previous_hover {
            self.interaction
                .scroll
                .activity
                .insert(node, Instant::now());
        }
        if pointer.properties.primary && self.interaction.scroll.drag.is_some() {
            let changed = self.drag_scrollbar(x, y);
            return EventResponse {
                handled: true,
                request_redraw: changed || scrollbar_hover_changed,
                ..EventResponse::IGNORED
            };
        }
        let pointer_id = pointer.properties.id;
        let (captured, dragged) = {
            let route = self
                .interaction
                .input
                .pointer_routes
                .entry(pointer_id)
                .or_default();
            if pointer.buttons & 1 != 0
                && let Some((down_x, down_y)) = route.down_position
            {
                let dx = x - down_x;
                let dy = y - down_y;
                route.dragged |= dx * dx + dy * dy > CLICK_DRAG_THRESHOLD_SQUARED;
            }
            (route.down_target, route.dragged)
        };
        if pointer.properties.primary {
            self.interaction.input.pointer_down_target = captured;
            self.interaction.input.pointer_dragged = dragged;
        }
        let target = self.interaction.input.hit_test(x, y);
        let mut changed = scrollbar_hover_changed;
        if let Some(captured) = captured
            && let Some(response) = self.handle_widget_event(captured, &UiEvent::Pointer(pointer))
        {
            changed |= response.handled || response.request_redraw;
        }
        if pointer.properties.primary && pointer.buttons & 1 != 0 {
            changed |= self.extend_text_selection(target, x, y);
            self.arm_text_selection_autoscroll();
        }
        changed |= self.update_hover_target(pointer_id, target, pointer.modifiers);
        let dispatch_target = if pointer.buttons != 0 {
            captured.or(target)
        } else {
            target
        };
        if let Some(target) = dispatch_target {
            changed |= self.dispatch_pointer(target, event::POINTERMOVE, None, pointer.modifiers);
        }
        Self::response(changed)
    }

    pub(super) fn handle_wheel_event(&mut self, wheel: gpui_shell::WheelEvent) -> EventResponse {
        self.interaction.input.pointer_position = (wheel.position.x, wheel.position.y);
        // Wheel events carry their own pointer position. Re-hit-test it even
        // when the cached hover target is still alive: semantic automation,
        // trackpads and virtualized content can move the wheel position
        // without first delivering a pointer-move event.
        self.interaction.input.hovered_target =
            self.interaction
                .input
                .hit_test(
                    self.interaction.input.pointer_position.0,
                    self.interaction.input.pointer_position.1,
                )
                .or_else(|| {
                    self.interaction.input.hovered_target.filter(|target| {
                        self.document.node_store.solid_to_node.contains_key(target)
                    })
                });
        let Some(target) = self.interaction.input.hovered_target else {
            return EventResponse::IGNORED;
        };

        // Rust widgets, such as terminal scrollback, get first refusal before
        // JavaScript listeners and native overflow scrolling.
        if let Some(response) = self.handle_widget_event(target, &UiEvent::Wheel(wheel)) {
            return response;
        }

        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = self.interaction.input.pointer_position.0;
        data[event_data::CLIENT_Y as usize] = self.interaction.input.pointer_position.1;
        let local = self.interaction.input.local_position(
            target,
            self.interaction.input.pointer_position.0,
            self.interaction.input.pointer_position.1,
        );
        data[event_data::OFFSET_X as usize] = local.0;
        data[event_data::OFFSET_Y as usize] = local.1;
        data[event_data::MODS as usize] = wheel.modifiers.bits() as f64;
        data[event_data::DELTA_X as usize] = wheel.delta_x;
        data[event_data::DELTA_Y as usize] = wheel.delta_y;
        data[event_data::PHASE as usize] = match wheel.phase {
            gpui_shell::GesturePhase::Started => 0.0,
            gpui_shell::GesturePhase::Changed => 1.0,
            gpui_shell::GesturePhase::Ended => 2.0,
            gpui_shell::GesturePhase::Cancelled => 3.0,
        };
        let (dispatched, prevented) = self.dispatch_cancellable_numeric(target, event::WHEEL, data);
        let scrolled = !prevented
            && self.scroll_nearest(
                target,
                wheel.delta_x as f32,
                wheel.delta_y as f32,
                wheel.delta_mode == gpui_shell::WheelDeltaMode::Line,
            );
        Self::response(dispatched || scrolled)
    }

    pub(super) fn rebuild_hit_geometry(&mut self, placed: &[PlacedNode]) {
        self.interaction.input.hit_items.clear();
        self.interaction.scroll.hits.clear();
        self.interaction.scroll.metrics.clear();
        let placed_by_id: HashMap<_, _> = placed.iter().map(|node| (node.node_id, node)).collect();
        let mut transforms = HashMap::with_capacity(placed.len());
        let mut clip_chains: HashMap<NodeId, Vec<HitClip>> = HashMap::with_capacity(placed.len());
        let mut content_hits = HashMap::new();
        let mut scrollbar_hits = HashMap::new();
        for node in placed {
            if node.scroll.range.iter().any(|range| *range > 0.5) {
                self.interaction
                    .scroll
                    .metrics
                    .insert(node.node_id, node.scroll);
            }
            let is_non_interactive_leaf = !node.paint.pointer_events
                && self
                    .document
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
            let interaction_blocked =
                subtree_blocks_interaction(&self.document.node_store, node.node_id);
            if node.paint.pointer_events
                && !interaction_blocked
                && let Some(&solid_id) = self.document.node_store.node_to_solid.get(&node.node_id)
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
            if !interaction_blocked
                && node.paint.scrollbar.visibility != ScrollbarVisibility::Hidden
                && node.scroll.range.iter().any(|range| *range > 0.5)
            {
                let hit = ScrollbarHit {
                    node: node.node_id,
                    placed: node.clone(),
                    transform,
                };
                self.interaction.scroll.hits.push(hit.clone());
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
                        self.interaction.input.hit_items.push(HitItem::Content(hit));
                    }
                }
                SubtreeEvent::Exit(node) => {
                    if let Some(hit) = scrollbar_hits.remove(&node.node_id) {
                        self.interaction
                            .input
                            .hit_items
                            .push(HitItem::Scrollbar(Box::new(hit)));
                    }
                }
            }
        }
    }

    pub(super) fn has_listener_in_chain(&self, solid_id: NodeKey, code: u8) -> bool {
        self.listener_target_in_chain(solid_id, code).is_some()
    }

    pub(super) fn listener_target_in_chain(
        &self,
        mut solid_id: NodeKey,
        code: u8,
    ) -> Option<NodeKey> {
        loop {
            if self
                .interaction
                .input
                .listeners
                .get(&solid_id)
                .is_some_and(|events| events.contains(code))
            {
                return Some(solid_id);
            }
            let node = self.document.node_store.solid_to_node.get(&solid_id)?;
            let parent = self.document.node_store.tree.parent(*node)?;
            solid_id = self.document.node_store.solid_id_for_node(parent)?;
        }
    }

    pub(super) fn dispatch_pointer(
        &mut self,
        target: NodeKey,
        code: u8,
        button: Option<PointerButton>,
        modifiers: Modifiers,
    ) -> bool {
        let Some(target) = self.listener_target_in_chain(target, code) else {
            return false;
        };
        let mut data = [0.0; event_data::LEN];
        data[event_data::CLIENT_X as usize] = self.interaction.input.pointer_position.0;
        data[event_data::CLIENT_Y as usize] = self.interaction.input.pointer_position.1;
        let local = self.interaction.input.local_position(
            target,
            self.interaction.input.pointer_position.0,
            self.interaction.input.pointer_position.1,
        );
        data[event_data::OFFSET_X as usize] = local.0;
        data[event_data::OFFSET_Y as usize] = local.1;
        data[event_data::BUTTON as usize] = button.map_or(0, Self::web_button) as f64;
        data[event_data::BUTTONS as usize] =
            Self::web_buttons(self.interaction.input.pointer_buttons) as f64;
        data[event_data::MODS as usize] = modifiers.bits() as f64;
        Self::fill_pointer_properties(&mut data, self.interaction.input.pointer_properties);
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id: 0,
            cancellable: false,
            payload: NodeEventPayload::Numeric(crate::host_frame::NumericEventData::prefix(
                data,
                event_data::TWIST as usize + 1,
            )),
        });
        if let Err(error) = self.dispatch_host_frame(&[event]) {
            tracing::warn!(?error, ?target, code, "event dispatch failed");
            return false;
        }
        true
    }

    fn fill_pointer_properties(
        data: &mut [f64; event_data::LEN],
        properties: gpui_shell::PointerProperties,
    ) {
        data[event_data::POINTER_ID_LO as usize] = f64::from(properties.id.lo);
        data[event_data::POINTER_ID_HI as usize] = f64::from(properties.id.hi);
        data[event_data::POINTER_TYPE as usize] = match properties.pointer_type {
            gpui_shell::PointerType::Mouse => 0.0,
            gpui_shell::PointerType::Touch => 1.0,
            gpui_shell::PointerType::Pen => 2.0,
            gpui_shell::PointerType::Unknown => 3.0,
        };
        data[event_data::PRIMARY as usize] = f64::from(properties.primary);
        data[event_data::PRESSURE as usize] = properties.pressure.unwrap_or(f64::NAN);
        data[event_data::TANGENTIAL_PRESSURE as usize] =
            properties.tangential_pressure.unwrap_or(f64::NAN);
        data[event_data::TILT_X as usize] = properties.tilt_x.unwrap_or(f64::NAN);
        data[event_data::TILT_Y as usize] = properties.tilt_y.unwrap_or(f64::NAN);
        data[event_data::TWIST as usize] = properties.twist.unwrap_or(f64::NAN);
    }

    pub(super) fn dispatch_cancellable_numeric(
        &mut self,
        target: NodeKey,
        code: u8,
        data: [f64; event_data::LEN],
    ) -> (bool, bool) {
        if !self.has_listener_in_chain(target, code) {
            return (false, false);
        }
        self.interaction.input.next_host_event_id = self
            .interaction
            .input
            .next_host_event_id
            .wrapping_add(1)
            .max(1);
        let event_id = self.interaction.input.next_host_event_id;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id,
            cancellable: true,
            payload: NodeEventPayload::Numeric(crate::host_frame::NumericEventData::prefix(
                data,
                Self::numeric_event_len(code),
            )),
        });
        match self.dispatch_host_frame(&[event]) {
            Ok(disposition) => (true, disposition.is_prevented(event_id)),
            Err(error) => {
                tracing::warn!(?error, ?target, code, "event dispatch failed");
                (false, false)
            }
        }
    }

    fn numeric_event_len(code: u8) -> usize {
        if code == event::WHEEL {
            event_data::PHASE as usize + 1
        } else {
            event_data::TWIST as usize + 1
        }
    }

    pub(super) fn dispatch_cancellable_json(
        &mut self,
        target: NodeKey,
        code: u8,
        payload: String,
    ) -> (bool, bool) {
        if !self.has_listener_in_chain(target, code) {
            return (false, false);
        }
        self.interaction.input.next_host_event_id = self
            .interaction
            .input
            .next_host_event_id
            .wrapping_add(1)
            .max(1);
        let event_id = self.interaction.input.next_host_event_id;
        let event = HostEvent::Node(HostNodeEvent {
            target,
            event_code: code,
            event_id,
            cancellable: true,
            payload: NodeEventPayload::Json(payload),
        });
        match self.dispatch_host_frame(&[event]) {
            Ok(disposition) => (true, disposition.is_prevented(event_id)),
            Err(error) => {
                tracing::warn!(?error, ?target, code, "event dispatch failed");
                (false, false)
            }
        }
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

    pub(super) fn dispatch_json(&mut self, target: NodeKey, code: u8, payload: &str) -> bool {
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
        if let Err(error) = self.dispatch_host_frame(&[event]) {
            tracing::warn!(?error, ?target, code, "event dispatch failed");
            return false;
        }
        true
    }

    pub(super) fn is_text_input_target(&self, target: NodeKey) -> bool {
        let Some(&node) = self.document.node_store.solid_to_node.get(&target) else {
            return false;
        };
        self.document
            .widget_manager
            .widgets
            .get(&node)
            .is_some_and(|widget| widget.accepts_text_input())
    }

    pub(super) fn set_focused_target(&mut self, target: Option<NodeKey>) -> bool {
        let old = self.interaction.input.focused_target;
        if old == target {
            return false;
        }
        self.frame.projections.semantics_dirty = true;
        self.interaction.input.focused_target = target;
        let mut changed = false;
        if let Some(old) = old {
            if self.interaction.input.window_focused
                && let Some(node) = self.document.node_store.solid_to_node.get(&old)
                && let Some(widget) = self.document.widget_manager.widgets.get_mut(node)
            {
                let changes = widget.focus_changed(false);
                self.invalidate_widget_changes(changes);
            }
            changed |= self.dispatch_json(old, event::BLUR, "");
            changed |= self.dispatch_json(old, event::FOCUSOUT, "");
        }
        if let Some(new) = target {
            if self.interaction.input.window_focused
                && let Some(node) = self.document.node_store.solid_to_node.get(&new)
                && let Some(widget) = self.document.widget_manager.widgets.get_mut(node)
            {
                let changes = widget.focus_changed(true);
                self.invalidate_widget_changes(changes);
            }
            let payload = self.interaction.focus_event_payload();
            changed |= self.dispatch_json(new, event::FOCUS, &payload);
            changed |= self.dispatch_json(new, event::FOCUSIN, &payload);
        }
        changed
    }

    pub(super) fn set_window_focused(&mut self, focused: bool) -> bool {
        if self.interaction.input.window_focused == focused {
            return false;
        }
        self.interaction.input.window_focused = focused;
        let Some(target) = self.interaction.input.focused_target else {
            return false;
        };
        let mut changed = false;
        if let Some(node) = self.document.node_store.solid_to_node.get(&target)
            && let Some(widget) = self.document.widget_manager.widgets.get_mut(node)
        {
            let changes = widget.focus_changed(focused);
            self.invalidate_widget_changes(changes);
            changed = true;
        }
        let (focus, focus_within, payload) = if focused {
            (
                event::FOCUS,
                event::FOCUSIN,
                self.interaction.focus_event_payload(),
            )
        } else {
            (event::BLUR, event::FOCUSOUT, String::new())
        };
        changed |= self.dispatch_json(target, focus, &payload);
        changed |= self.dispatch_json(target, focus_within, &payload);
        changed
    }
}
