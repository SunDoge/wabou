use super::*;

impl Applier {
    pub(super) fn handle_widget_event(
        &mut self,
        target: u32,
        input: &UiEvent,
    ) -> Option<EventResponse> {
        let node = *self.node_store.solid_to_node.get(&target)?;
        let result = {
            let widget = self.widget_manager.widgets.get_mut(&node)?;
            widget.handle_event(input)
        };
        self.drain_widget_host_actions(node);
        self.drain_widget_node_events(node);
        if !result.is_handled() {
            return None;
        }
        // Value sync is deferred to build_frame: `current_value()` reads
        // `cached_value`, which is only fresh after `paint_widgets` applies the
        // pending edits queued above. Reading + dispatching here would send a
        // stale value to JS. `flush_value_sync` drains this set after paint.
        if result.value_changed() {
            self.widget_manager.pending_value_sync.insert(target);
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
                let host_request_id = self.widget_manager.next_host_action_id;
                self.widget_manager.next_host_action_id =
                    (self.widget_manager.next_host_action_id.wrapping_add(1)
                        & HOST_ACTION_SEQUENCE_MASK)
                        .max(1);
                self.widget_manager
                    .host_action_routes
                    .insert(host_request_id, (node, request_id));
                wabou_shell::HostAction::ReadClipboard {
                    request_id: host_request_id,
                }
            }
            action => action,
        };
        self.pending_host_actions.borrow_mut().push_back(action);
    }

    pub(super) fn drain_widget_host_actions(&mut self, node: NodeId) {
        while let Some(action) = self
            .widget_manager
            .widgets
            .get_mut(&node)
            .and_then(|widget| widget.take_host_action())
        {
            self.enqueue_widget_host_action(node, action);
        }
    }

    pub(super) fn drain_widget_node_events(&mut self, node: NodeId) -> bool {
        let Some(target) = self.node_store.solid_id_for_node(node) else {
            return false;
        };
        let mut events = Vec::new();
        while let Some(event) = self
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
        if self.widget_manager.pending_value_sync.is_empty() {
            return;
        }
        let value_atom = self.atoms.borrow_mut().intern("value");
        for target in self
            .widget_manager
            .pending_value_sync
            .drain()
            .collect::<Vec<_>>()
        {
            let Some(&node) = self.node_store.solid_to_node.get(&target) else {
                continue;
            };
            let Some(value) = self
                .widget_manager
                .widgets
                .get(&node)
                .and_then(|w| w.current_value().map(str::to_owned))
            else {
                continue;
            };
            if let Some(decl) = self.node_store.declared.get_mut(&node) {
                decl.attrs.insert(value_atom, Arc::from(value.as_str()));
            }
            let payload = serde_json::json!({ "value": value }).to_string();
            let _ = self.dispatch_json(target, event::INPUT, &payload);
        }
    }

    pub(super) fn dispatch_resize_changes(&mut self) -> bool {
        let mut targets = self.resize_targets.borrow_mut();
        let mut changes = Vec::new();
        for (&solid_id, last) in targets.iter_mut() {
            let Some(&node) = self.node_store.solid_to_node.get(&solid_id) else {
                continue;
            };
            let Ok(layout) = self.node_store.tree.layout(node) else {
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
        if let Err(error) = self.js.dispatch_host_frame(&events) {
            tracing::warn!(?error, "ResizeObserver dispatch failed");
            return false;
        }
        true
    }

    /// Deliver resolved content styles before widget measurement.
    pub(super) fn sync_widget_styles(&mut self) {
        for (&node, widget) in &mut self.widget_manager.widgets {
            let Some(paint) = self.node_store.tree.get_node_context(node) else {
                continue;
            };
            let style = wabou_shell::WidgetStyle::from(paint);
            if self.widget_manager.styles.get(&node) != Some(&style) {
                widget.style_changed(&style);
                self.widget_manager.styles.insert(node, style);
            }
        }
    }

    /// After layout, call `Widget::paint` for each widget node + store the
    /// resulting Scene fragment in the matching PlacedNode's `paint.widget`.
    /// `build_scene` composites it at the node's content-box origin.
    pub(super) fn paint_widgets(&mut self, placed: &mut [PlacedNode], tcx: &mut TextContext) {
        self.ime_cursor_area = None;
        let mut transforms = HashMap::with_capacity(placed.len());
        for n in placed.iter_mut() {
            let parent_transform = n
                .parent_node_id
                .and_then(|parent| transforms.get(&parent).copied())
                .unwrap_or(Affine::IDENTITY);
            let transform = wabou_shell::scene::resolve_node_transform(n, parent_transform);
            transforms.insert(n.node_id, transform);
            if let Some(w) = self.widget_manager.widgets.get_mut(&n.node_id) {
                w.set_position(n.rect[0], n.rect[1]);
                let window_to_local = Affine::translate((
                    -f64::from(n.content_origin[0]),
                    -f64::from(n.content_origin[1]),
                )) * transform.inverse();
                w.set_window_to_local(window_to_local.as_coeffs());
                let [width, height] = n.content_size;
                if width > 0.0 && height > 0.0 {
                    let scene = w.paint_scaled(width, height, self.device_scale, tcx);
                    n.paint.widget = Some(std::sync::Arc::new(scene));
                }
                if self.input.focused_target == self.node_store.solid_id_for_node(n.node_id)
                    && let Some([x0, y0, x1, y1]) = w.ime_cursor_area()
                {
                    let local_to_window = window_to_local.inverse();
                    let points = [
                        local_to_window * Point::new(f64::from(x0), f64::from(y0)),
                        local_to_window * Point::new(f64::from(x1), f64::from(y0)),
                        local_to_window * Point::new(f64::from(x0), f64::from(y1)),
                        local_to_window * Point::new(f64::from(x1), f64::from(y1)),
                    ];
                    self.ime_cursor_area = Some([
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
    }

    pub(super) fn measure_widgets(&mut self, tcx: &mut TextContext) {
        let changed: Vec<_> = self
            .widget_manager
            .widgets
            .iter_mut()
            .filter_map(|(&node, widget)| {
                let measured = widget.measure(tcx);
                let current = self
                    .node_store
                    .tree
                    .get_node_context(node)
                    .and_then(|paint| paint.intrinsic_size);
                (measured != current).then_some((node, measured))
            })
            .collect();
        for (node, measured) in changed {
            if let Some(mut paint) = self.node_store.tree.get_node_context(node).cloned() {
                paint.intrinsic_size = measured;
                let _ = self.node_store.tree.set_node_context(node, Some(paint));
                self.invalidation.insert(InvalidationFlags::LAYOUT);
            }
        }
    }
}
