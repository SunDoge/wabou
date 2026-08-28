use super::*;

impl Applier {
    pub(super) fn invalidate_widget_changes(&mut self, changes: wabou_shell::WidgetChanges) {
        if changes
            .intersects(wabou_shell::WidgetChanges::VALUE | wabou_shell::WidgetChanges::SEMANTICS)
        {
            self.frame.projections.semantics_dirty = true;
        }
        if changes
            .intersects(wabou_shell::WidgetChanges::MEASURE | wabou_shell::WidgetChanges::LAYOUT)
        {
            self.document.invalidation.insert(InvalidationFlags::LAYOUT);
        } else if changes.contains(wabou_shell::WidgetChanges::REDRAW) {
            self.document
                .invalidation
                .insert(InvalidationFlags::GEOMETRY);
        }
    }

    pub(super) fn handle_widget_event(
        &mut self,
        target: NodeKey,
        input: &UiEvent,
    ) -> Option<EventResponse> {
        let node = *self.document.node_store.solid_to_node.get(&target)?;
        let input = self
            .document
            .widget_manager
            .geometries
            .get(&node)
            .map_or_else(
                || input.clone(),
                |geometry| localize_widget_event(input, *geometry),
            );
        let (result, selection) = {
            let widget = self.document.widget_manager.widgets.get_mut(&node)?;
            let result = widget.handle_event(&input);
            let selection = result
                .selection_changed()
                .then(|| widget.text_selection())
                .flatten();
            (result, selection)
        };
        self.drain_widget_host_actions(node);
        self.drain_widget_node_events(node);
        self.invalidate_widget_changes(result.changes());
        if let Some(selection) = selection {
            let _ = self.dispatch_json(
                target,
                event::TEXTSELECTIONCHANGE,
                &serde_json::json!({
                    "anchor": selection.anchor,
                    "head": selection.head,
                    "text": selection.text,
                    "kind": selection.kind.as_str(),
                })
                .to_string(),
            );
        }
        if !result.is_handled() {
            return None;
        }
        // Value sync is deferred to build_frame: `current_value()` reads
        // `cached_value`, which is only fresh after `paint_widgets` applies the
        // pending edits queued above. Reading + dispatching here would send a
        // stale value to JS. `flush_value_sync` drains this set after paint.
        if result.value_changed() {
            self.document
                .widget_manager
                .pending_value_sync
                .insert(target);
        }
        Some(EventResponse {
            handled: true,
            request_redraw: result.requests_redraw(),
            consume_key_text: result.consumes_key_text(),
            text_input: None,
            clipboard: result.clipboard_request().cloned(),
        })
    }

    pub(super) fn enqueue_widget_host_action(
        &mut self,
        node: NodeId,
        action: wabou_shell::HostAction,
    ) {
        let action = match action {
            wabou_shell::HostAction::ReadClipboard { request_id } => {
                let host_request_id = self.document.widget_manager.next_host_action_id;
                self.document.widget_manager.next_host_action_id = (self
                    .document
                    .widget_manager
                    .next_host_action_id
                    .wrapping_add(1)
                    & HOST_ACTION_SEQUENCE_MASK)
                    .max(1);
                self.document
                    .widget_manager
                    .host_action_routes
                    .insert(host_request_id, (node, request_id));
                wabou_shell::HostAction::ReadClipboard {
                    request_id: host_request_id,
                }
            }
            action => action,
        };
        self.runtime
            .pending_host_actions
            .borrow_mut()
            .push_back(action);
    }

    pub(super) fn drain_widget_host_actions(&mut self, node: NodeId) {
        while let Some(action) = self
            .document
            .widget_manager
            .widgets
            .get_mut(&node)
            .and_then(|widget| widget.take_host_action())
        {
            self.enqueue_widget_host_action(node, action);
        }
    }

    pub(super) fn drain_widget_node_events(&mut self, node: NodeId) -> bool {
        let Some(target) = self.document.node_store.solid_id_for_node(node) else {
            return false;
        };
        let mut events = Vec::new();
        while let Some(event) = self
            .document
            .widget_manager
            .widgets
            .get_mut(&node)
            .and_then(|widget| widget.take_node_event())
        {
            events.push(event);
        }
        #[allow(clippy::unnecessary_fold)] // Every queued event must dispatch.
        events.into_iter().fold(false, |dispatched, event| {
            self.dispatch_json(target, event.event_code, &event.json) || dispatched
        })
    }

    /// Drain [`pending_value_sync`]: after `paint_widgets` has applied pending
    /// edits, read each widget's now-fresh `current_value()`, sync the `value`
    /// attr, and dispatch `INPUT` to JS.
    pub(super) fn flush_value_sync(&mut self) {
        if self.document.widget_manager.pending_value_sync.is_empty() {
            return;
        }
        let value_atom = self.document.atoms.borrow_mut().intern("value");
        for target in self
            .document
            .widget_manager
            .pending_value_sync
            .drain()
            .collect::<Vec<_>>()
        {
            let Some(&node) = self.document.node_store.solid_to_node.get(&target) else {
                continue;
            };
            let Some(value) = self
                .document
                .widget_manager
                .widgets
                .get(&node)
                .and_then(|w| w.current_value().map(str::to_owned))
            else {
                continue;
            };
            if let Some(decl) = self.document.node_store.declared.get_mut(&node) {
                decl.attrs.insert(value_atom, Arc::from(value.as_str()));
            }
            let payload = serde_json::json!({ "value": value }).to_string();
            let _ = self.dispatch_json(target, event::INPUT, &payload);
        }
    }

    pub(super) fn dispatch_resize_changes(&mut self) -> bool {
        let mut targets = self.frame.resize_targets.borrow_mut();
        let mut changes = Vec::new();
        for (&solid_id, last) in targets.iter_mut() {
            let Some(&node) = self.document.node_store.solid_to_node.get(&solid_id) else {
                continue;
            };
            let Ok(layout) = self.document.node_store.tree.layout(node) else {
                continue;
            };
            let width = (layout.size.width
                - layout.border.left
                - layout.border.right
                - layout.padding.left
                - layout.padding.right)
                .max(0.0);
            let height = (layout.size.height
                - layout.border.top
                - layout.border.bottom
                - layout.padding.top
                - layout.padding.bottom)
                .max(0.0);
            if *last != Some((width, height)) {
                *last = Some((width, height));
                changes.push((solid_id, width, height));
            }
        }
        drop(targets);
        if changes.is_empty() {
            return false;
        }
        let events: Vec<_> = changes
            .into_iter()
            .map(|(target, width, height)| {
                HostEvent::Resize(ResizeObservation {
                    target,
                    width,
                    height,
                })
            })
            .collect();
        if let Err(error) = self.runtime.js.dispatch_host_frame(&events) {
            tracing::warn!(?error, "ResizeObserver dispatch failed");
            return false;
        }
        true
    }

    /// Deliver resolved content styles before widget measurement.
    pub(super) fn sync_widget_styles(&mut self) {
        let mut changes = wabou_shell::WidgetChanges::empty();
        for (&node, widget) in &mut self.document.widget_manager.widgets {
            let Some(paint) = self.document.node_store.tree.get_node_context(node) else {
                continue;
            };
            let style = wabou_shell::WidgetStyle::from(paint);
            if self.document.widget_manager.styles.get(&node) != Some(&style) {
                changes |= widget.style_changed(&style);
                self.document.widget_manager.styles.insert(node, style);
            }
        }
        self.invalidate_widget_changes(changes);
    }

    /// After layout, call `Widget::paint` for each widget node + store the
    /// resulting Scene fragment in the matching PlacedNode's `paint.widget`.
    /// `build_scene` composites it at the node's content-box origin.
    pub(super) fn paint_widgets(&mut self, placed: &mut [PlacedNode], tcx: &mut TextContext) {
        self.interaction.ime_cursor_area = None;
        let visible = placed
            .iter()
            .filter(|node| node.content_size[0] > 0.0 && node.content_size[1] > 0.0)
            .map(|node| node.node_id)
            .collect::<HashSet<_>>();
        let mut visibility_changes = wabou_shell::WidgetChanges::empty();
        let mut visibility_changed_nodes = Vec::new();
        for (&node, widget) in &mut self.document.widget_manager.widgets {
            let is_visible = visible.contains(&node);
            if self.document.widget_manager.visibility.get(&node) != Some(&is_visible) {
                visibility_changes |= widget.visibility_changed(is_visible);
                self.document
                    .widget_manager
                    .visibility
                    .insert(node, is_visible);
                visibility_changed_nodes.push(node);
            }
        }
        self.invalidate_widget_changes(visibility_changes);
        for node in visibility_changed_nodes {
            self.drain_widget_host_actions(node);
            self.drain_widget_node_events(node);
        }
        let mut transforms = HashMap::with_capacity(placed.len());
        for n in placed.iter_mut() {
            let parent_transform = n
                .parent_node_id
                .and_then(|parent| transforms.get(&parent).copied())
                .unwrap_or(Affine::IDENTITY);
            let transform = wabou_shell::scene::resolve_node_transform(n, parent_transform);
            transforms.insert(n.node_id, transform);
            if let Some(w) = self.document.widget_manager.widgets.get_mut(&n.node_id) {
                let window_to_local = Affine::translate((
                    -f64::from(n.content_origin[0]),
                    -f64::from(n.content_origin[1]),
                )) * transform.inverse();
                let [width, height] = n.content_size;
                let geometry = wabou_shell::WidgetGeometry {
                    content_size: [width, height],
                    device_scale: self.frame.device_scale,
                    local_to_window: window_to_local.inverse().as_coeffs(),
                    window_to_local: window_to_local.as_coeffs(),
                };
                if self.document.widget_manager.geometries.get(&n.node_id) != Some(&geometry) {
                    w.layout_changed(geometry);
                    self.document
                        .widget_manager
                        .geometries
                        .insert(n.node_id, geometry);
                }
                if width > 0.0 && height > 0.0 {
                    let border_inset = n.border_widths.into_iter().fold(0.0_f32, f32::max);
                    let inner_radius =
                        (f64::from(n.paint.border_radius) - f64::from(border_inset)).max(0.0);
                    let mut paint = wabou_shell::PaintContext::new_clipped_at(
                        width,
                        height,
                        inner_radius,
                        self.frame.device_scale,
                        geometry.local_to_window,
                        tcx,
                    );
                    w.paint(&mut paint);
                    n.paint.widget = Some(std::sync::Arc::new(paint.finish()));
                }
                if self.interaction.input.focused_target
                    == self.document.node_store.solid_id_for_node(n.node_id)
                    && let Some([x0, y0, x1, y1]) = w.ime_cursor_area()
                {
                    let local_to_window = window_to_local.inverse();
                    let points = [
                        local_to_window * Point::new(f64::from(x0), f64::from(y0)),
                        local_to_window * Point::new(f64::from(x1), f64::from(y0)),
                        local_to_window * Point::new(f64::from(x0), f64::from(y1)),
                        local_to_window * Point::new(f64::from(x1), f64::from(y1)),
                    ];
                    self.interaction.ime_cursor_area = Some([
                        points
                            .iter()
                            .map(|point| point.x)
                            .fold(f64::INFINITY, f64::min),
                        points
                            .iter()
                            .map(|point| point.y)
                            .fold(f64::INFINITY, f64::min),
                        points
                            .iter()
                            .map(|point| point.x)
                            .fold(f64::NEG_INFINITY, f64::max),
                        points
                            .iter()
                            .map(|point| point.y)
                            .fold(f64::NEG_INFINITY, f64::max),
                    ]);
                }
            }
        }
        let widget_nodes = self
            .document
            .widget_manager
            .widgets
            .keys()
            .copied()
            .collect::<Vec<_>>();
        for node in widget_nodes {
            self.drain_widget_host_actions(node);
            self.drain_widget_node_events(node);
        }
    }

    #[cfg(test)]
    pub(super) fn measure_widgets(&mut self, tcx: &mut TextContext) {
        let changed: Vec<_> = self
            .document
            .widget_manager
            .widgets
            .iter_mut()
            .filter_map(|(&node, widget)| {
                let mut cx = wabou_shell::MeasureContext::new(
                    [None, None],
                    [
                        wabou_shell::WidgetAvailableSpace::MaxContent,
                        wabou_shell::WidgetAvailableSpace::MaxContent,
                    ],
                    self.frame.device_scale,
                    tcx,
                );
                let measured = widget.measure(&mut cx);
                let current = self
                    .document
                    .node_store
                    .tree
                    .get_node_context(node)
                    .and_then(|paint| paint.intrinsic_size);
                (measured != current).then_some((node, measured))
            })
            .collect();
        for (node, measured) in changed {
            if let Some(mut paint) = self
                .document
                .node_store
                .tree
                .get_node_context(node)
                .cloned()
            {
                paint.intrinsic_size = measured;
                let _ = self
                    .document
                    .node_store
                    .tree
                    .set_node_context(node, Some(paint));
                self.document.invalidation.insert(InvalidationFlags::LAYOUT);
            }
        }
    }
}

fn localize_widget_event(input: &UiEvent, geometry: wabou_shell::WidgetGeometry) -> UiEvent {
    let transform = Affine::new(geometry.window_to_local);
    let local = |point: wabou_shell::Point| {
        let point = transform * Point::new(point.x, point.y);
        wabou_shell::Point {
            x: point.x,
            y: point.y,
        }
    };
    match input {
        UiEvent::Pointer(pointer) => UiEvent::Pointer(wabou_shell::PointerEvent {
            position: local(pointer.position),
            ..*pointer
        }),
        UiEvent::Wheel(wheel) => UiEvent::Wheel(wabou_shell::WheelEvent {
            position: local(wheel.position),
            ..*wheel
        }),
        input => input.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{Modifiers, PointerButton, PointerEvent, PointerPhase, WheelEvent};

    fn geometry() -> wabou_shell::WidgetGeometry {
        wabou_shell::WidgetGeometry {
            content_size: [200.0, 100.0],
            device_scale: 2.0,
            local_to_window: [2.0, 0.0, 0.0, 2.0, 100.0, 20.0],
            window_to_local: [0.5, 0.0, 0.0, 0.5, -50.0, -10.0],
        }
    }

    #[test]
    fn pointer_events_are_localized_at_the_widget_boundary() {
        let event = UiEvent::Pointer(PointerEvent {
            phase: PointerPhase::Down,
            position: wabou_shell::Point { x: 120.0, y: 30.0 },
            button: Some(PointerButton::Primary),
            buttons: 1,
            modifiers: Modifiers::default(),
            properties: wabou_shell::PointerProperties::default(),
        });

        let UiEvent::Pointer(local) = localize_widget_event(&event, geometry()) else {
            panic!("expected pointer event");
        };
        assert_eq!(local.position, wabou_shell::Point { x: 10.0, y: 5.0 });
    }

    #[test]
    fn wheel_position_is_localized_without_changing_the_delta() {
        let event = UiEvent::Wheel(WheelEvent {
            position: wabou_shell::Point { x: 120.0, y: 30.0 },
            delta_x: 3.0,
            delta_y: -8.0,
            phase: wabou_shell::GesturePhase::Changed,
            modifiers: Modifiers::default(),
        });

        let UiEvent::Wheel(local) = localize_widget_event(&event, geometry()) else {
            panic!("expected wheel event");
        };
        assert_eq!(local.position, wabou_shell::Point { x: 10.0, y: 5.0 });
        assert_eq!((local.delta_x, local.delta_y), (3.0, -8.0));
    }
}
