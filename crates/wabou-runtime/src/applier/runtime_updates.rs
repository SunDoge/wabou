use super::*;

impl Applier {
    pub(super) fn handle_modifiers_changed(
        &mut self,
        modifiers: gpui_shell::Modifiers,
    ) -> EventResponse {
        const PRIMARY: u8 = 1 << 4;
        let bits = modifiers.bits()
            | if modifiers.primary_shortcut() {
                PRIMARY
            } else {
                0
            };
        let event = HostEvent::Application(crate::host_message::HostMessage::i32(
            "wabou:keyboard-modifiers",
            i32::from(bits),
        ));
        let handled = self.dispatch_host_frame(&[event]).is_ok();
        EventResponse {
            handled,
            request_redraw: handled,
            ..EventResponse::IGNORED
        }
    }

    pub(super) fn handle_app_lifecycle(
        &mut self,
        lifecycle: gpui_shell::AppLifecycleEvent,
    ) -> EventResponse {
        let state = match lifecycle {
            gpui_shell::AppLifecycleEvent::Resumed => "resumed",
            gpui_shell::AppLifecycleEvent::Suspended => "suspended",
            gpui_shell::AppLifecycleEvent::MemoryWarning => "memory-warning",
        };
        let event = HostEvent::Application(crate::host_message::HostMessage::str(
            "wabou:app-lifecycle",
            serde_json::json!({ "state": state }).to_string(),
        ));
        let handled = self.dispatch_host_frame(&[event]).is_ok();
        EventResponse {
            handled,
            request_redraw: handled,
            ..EventResponse::IGNORED
        }
    }

    pub(super) fn handle_gesture(&mut self, gesture: gpui_shell::GestureEvent) -> EventResponse {
        let phase = |phase| match phase {
            gpui_shell::GesturePhase::Started => "started",
            gpui_shell::GesturePhase::Changed => "changed",
            gpui_shell::GesturePhase::Ended => "ended",
            gpui_shell::GesturePhase::Cancelled => "cancelled",
        };
        let payload = match gesture {
            gpui_shell::GestureEvent::Pinch {
                delta,
                phase: value,
            } => serde_json::json!({ "type": "pinch", "delta": delta, "phase": phase(value) }),
            gpui_shell::GestureEvent::Pan {
                delta_x,
                delta_y,
                phase: value,
            } => serde_json::json!({
                "type": "pan", "deltaX": delta_x, "deltaY": delta_y, "phase": phase(value),
            }),
            gpui_shell::GestureEvent::Rotation {
                delta,
                phase: value,
            } => serde_json::json!({
                "type": "rotation", "delta": delta, "phase": phase(value),
            }),
            gpui_shell::GestureEvent::DoubleTap => {
                serde_json::json!({ "type": "double-tap" })
            }
            gpui_shell::GestureEvent::Pressure { pressure, stage } => serde_json::json!({
                "type": "pressure", "pressure": pressure, "stage": stage,
            }),
        };
        let event = HostEvent::Application(crate::host_message::HostMessage::str(
            "wabou:gesture",
            payload.to_string(),
        ));
        let handled = self.dispatch_host_frame(&[event]).is_ok();
        EventResponse {
            handled,
            request_redraw: handled,
            ..EventResponse::IGNORED
        }
    }

    pub(super) fn handle_file_drop(&mut self, event: gpui_shell::FileDropEvent) -> EventResponse {
        let phase = match event.phase {
            gpui_shell::FileDropPhase::Entered => "entered",
            gpui_shell::FileDropPhase::Moved => "moved",
            gpui_shell::FileDropPhase::Left => "left",
            gpui_shell::FileDropPhase::Dropped => "dropped",
        };
        let payload = serde_json::json!({
            "phase": phase,
            "paths": event.paths.into_iter()
                .map(|path| path.to_string_lossy().into_owned())
                .collect::<Vec<_>>(),
            "position": event.position.map(|position| serde_json::json!({
                "x": position.x,
                "y": position.y,
            })),
        })
        .to_string();
        let event = HostEvent::Application(crate::host_message::HostMessage::str(
            "wabou:file-drop",
            payload,
        ));
        let handled = self.dispatch_host_frame(&[event]).is_ok();
        EventResponse {
            handled,
            request_redraw: handled,
            ..EventResponse::IGNORED
        }
    }

    pub(super) fn handle_window_metrics(
        &mut self,
        metrics: gpui_shell::WindowMetrics,
    ) -> EventResponse {
        // WindowMetrics is the authoritative cross-backend scale transition.
        // Native shells also set it before each build, but deterministic and
        // embedded hosts may only deliver this event.
        self.frame.device_scale = metrics.scale_factor.max(f64::EPSILON);
        let payload = serde_json::json!({
            "windowId": metrics.window_key,
            "logicalWidth": metrics.logical_width,
            "logicalHeight": metrics.logical_height,
            "physicalWidth": metrics.physical_width,
            "physicalHeight": metrics.physical_height,
            "scaleFactor": metrics.scale_factor,
            "maximized": metrics.maximized,
            "focused": metrics.focused,
            "outerX": metrics.outer_x,
            "outerY": metrics.outer_y,
            "occluded": metrics.occluded,
            "colorScheme": metrics.color_scheme.map(|scheme| match scheme {
                gpui_shell::ColorScheme::Light => "light",
                gpui_shell::ColorScheme::Dark => "dark",
            }),
        })
        .to_string();
        let event = HostEvent::Application(crate::host_message::HostMessage::str(
            "wabou:window-metrics",
            payload,
        ));
        let handled = self.dispatch_host_frame(&[event]).is_ok();
        EventResponse {
            handled,
            request_redraw: handled,
            ..EventResponse::IGNORED
        }
    }

    pub(super) fn drain_host_messages(&mut self) {
        let batch = self.runtime.host_message_inbox.drain_batch();
        if batch.is_empty() {
            return;
        }
        let events: Vec<_> = batch.into_iter().map(HostEvent::Application).collect();
        if let Err(e) = self.dispatch_host_frame(&events) {
            tracing::error!(target: "host_message", error = ?e, count = events.len(), "dispatch Host application frame failed");
        }
    }

    /// Take a [`ReloadHandle`] the HMR client uses to push updates; the applier
    /// drains them in `FrameSource::build_frame` before the next tick.
    pub fn reload_handle(&mut self) -> ReloadHandle {
        self.runtime.reload.handle()
    }

    /// Record the Vite entry path so declined HMR can re-import it in-process.
    pub fn set_vite_entry(&mut self, entry: impl Into<String>) {
        self.runtime.reload.set_vite_entry(entry);
    }

    /// Last HMR batch outcome (updated each `build_frame` that drains the queue).
    pub fn last_hmr_result(&self) -> &HmrDrainResult {
        self.runtime.reload.last_result()
    }

    /// Drain every pending [`ReloadMsg`] into one batch and apply it.
    ///
    /// **Order:** native CSS updates are logged only (Style IR arrives via
    /// `pending_css` / virtual stylesheet in the same frame). JS updates run
    /// next. Transform errors retain the last-good scene; declined updates or
    /// an explicit full-reload payload re-import the Vite entry when configured.
    pub(super) fn drain_hmr_batch(&mut self) -> HmrDrainResult {
        let Some(batch) = self.runtime.reload.drain() else {
            return HmrDrainResult::Idle;
        };
        self.apply_hmr_batch(batch)
    }

    pub(super) fn apply_hmr_batch(&mut self, batch: HmrBatch) -> HmrDrainResult {
        if let Some(diagnostic) = batch.error {
            tracing::error!(target: "hmr", diagnostic = %diagnostic, "Vite update failed; keeping last-good UI");
            let event = HostEvent::Application(crate::host_message::HostMessage::str(
                "wabou:dev-server-error",
                diagnostic.clone(),
            ));
            if let Err(error) = self.dispatch_host_frame(&[event]) {
                tracing::error!(target: "hmr", ?error, "failed to dispatch Vite diagnostic to JavaScript");
            }
            return HmrDrainResult::Error { diagnostic };
        }

        // Native Vite CSS channel: styles that affect layout must go through
        // Style IR (`virtual:wabou-stylesheet` → `__wabou_set_stylesheet`), which
        // is already drained earlier in build_frame via `pending_css`.
        for path in &batch.css_paths {
            tracing::warn!(
                target: "hmr",
                %path,
                "ignoring native Vite css-update; layout styles use virtual:wabou-stylesheet → Style IR"
            );
        }

        if batch.full_reload {
            let reason = batch
                .full_reload_reason
                .unwrap_or_else(|| "vite full-reload".into());
            self.perform_full_reload(&reason);
            return HmrDrainResult::FullReload { reason };
        }

        #[cfg(feature = "vite")]
        let mut applied = 0usize;
        // Without the vite feature, count queued updates so diagnostics stay
        // useful even though the updates cannot be evaluated.
        #[cfg(not(feature = "vite"))]
        let applied = batch.js_updates.len();
        for update in batch.js_updates {
            #[cfg(feature = "vite")]
            {
                match self.runtime.js.apply_hmr_update(
                    &update.path,
                    &update.accepted_path,
                    update.timestamp,
                    update.source,
                ) {
                    Ok(true) => {
                        applied += 1;
                        tracing::debug!(
                            target: "hmr",
                            path = %update.path,
                            "HMR update accepted"
                        );
                    }
                    Ok(false) => {
                        let reason =
                            format!("module declined or missing hot context: {}", update.path);
                        tracing::warn!(target: "hmr", %reason);
                        self.perform_full_reload(&reason);
                        return HmrDrainResult::FullReload { reason };
                    }
                    Err(e) => {
                        let reason = format!("apply_hmr failed for {}: {e:?}", update.path);
                        tracing::error!(target: "hmr", %reason);
                        self.perform_full_reload(&reason);
                        return HmrDrainResult::FullReload { reason };
                    }
                }
            }
            #[cfg(not(feature = "vite"))]
            {
                let _ = update;
                tracing::warn!(
                    target: "hmr",
                    "received HMR update but binary built without `vite` feature"
                );
            }
        }
        if applied > 0 || !batch.css_paths.is_empty() {
            self.dispatch_dev_server_ready();
            // CSS-only batches still report Applied (Style IR may have updated
            // via pending_css in the same frame).
            HmrDrainResult::Applied {
                js_updates: applied,
            }
        } else {
            HmrDrainResult::Idle
        }
    }

    /// Re-import the Vite entry while retaining the last-good native scene.
    pub(super) fn perform_full_reload(&mut self, reason: &str) {
        tracing::warn!(target: "hmr", %reason, "performing in-process full reload");

        #[cfg(feature = "vite")]
        {
            if let Some(entry) = self.runtime.reload.vite_entry().map(str::to_owned) {
                // Never block the render thread waiting for a half-written save.
                // A failed import leaves the current scene visible and the next
                // Vite notification retries with the newly transformed graph.
                match self.runtime.js.reboot_vite_entry(&entry) {
                    Ok(()) => {
                        tracing::info!(target: "hmr", %entry, "vite entry re-imported after full reload");
                        self.document
                            .invalidation
                            .insert(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
                        self.runtime.has_raf = true;
                        self.dispatch_dev_server_ready();
                    }
                    Err(e) => {
                        tracing::error!(
                            target: "hmr",
                            %entry,
                            error = ?e,
                            "full reload re-import failed; keeping last-good UI until the next update"
                        );
                    }
                }
                return;
            }
        }

        tracing::error!(
            target: "hmr",
            %reason,
            "full reload requested but no vite entry is configured — restart wabou-runtime"
        );
    }

    fn dispatch_dev_server_ready(&mut self) {
        let event = HostEvent::Application(crate::host_message::HostMessage::str(
            "wabou:dev-server-ready",
            "{}",
        ));
        if let Err(error) = self.dispatch_host_frame(&[event]) {
            tracing::error!(target: "hmr", ?error, "failed to clear the Vite diagnostic in JavaScript");
        }
    }

    /// Clear all application-owned retained state while preserving the host root.
    ///
    /// This is used by Vite full reloads and by layout-fixture batches. The
    /// JavaScript owner must dispose its Solid root as well; this method makes
    /// the native half independently deterministic when a prior fixture left
    /// focus, scrolling, widgets, or resources behind.
    pub fn reset_scene_tree(&mut self) {
        let doomed: Vec<NodeId> = self
            .document
            .node_store
            .solid_to_node
            .iter()
            .filter(|(solid, _)| **solid != NodeKey::ROOT)
            .map(|(_, node)| *node)
            .collect();
        for node in doomed {
            let _ = self.document.node_store.tree.remove(node);
        }
        self.document
            .node_store
            .solid_to_node
            .retain(|id, _| *id == NodeKey::ROOT);
        self.document
            .node_store
            .node_to_solid
            .retain(|_, id| *id == NodeKey::ROOT);
        self.document
            .node_store
            .declared
            .retain(|node, _| *node == self.document.node_store.root);
        self.document.node_store.children.clear();
        self.document
            .node_store
            .children
            .insert(self.document.node_store.root, Vec::new());
        let _ = self
            .document
            .node_store
            .tree
            .set_children(self.document.node_store.root, &[]);
        self.document.node_store.collapsed_text.clear();
        self.document.node_store.inline_roots.clear();
        self.document.resources.clear_scene_bindings();
        self.document.runtime_transforms.clear();
        self.document.overlay_planes.clear();
        self.interaction.scroll.styles.clear();
        self.document.style.diagnostics.clear();
        #[cfg(any(feature = "devtools", test))]
        self.document.style.cascade.clear();
        for widget in self.document.widget_manager.widgets.values_mut() {
            widget.unmount();
        }
        self.document.widget_manager.widgets.clear();
        self.document.widget_manager.styles.clear();
        self.document.widget_manager.geometries.clear();
        self.document.widget_manager.visibility.clear();
        self.document.widget_manager.host_action_routes.clear();
        self.interaction = InteractionState::new();
        self.document.node_store.logical_parent.clear();
        self.frame.projections.semantic_snapshot = Arc::new(SemanticSnapshot::default());
        self.frame.projections.semantics_dirty = true;
        self.document.widget_manager.pending_value_sync.clear();
        self.document.widget_manager.pending_selection_sync.clear();
        self.document.dirty_styles.clear();
        self.document
            .invalidation
            .insert(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
        if let Ok(mut targets) = self.frame.resize_targets.try_borrow_mut() {
            targets.clear();
        }
    }
}
