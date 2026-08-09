use super::*;

impl Applier {
    pub(super) fn drain_host_messages(&mut self) {
        let batch = self.host_msg_inbox.drain_batch();
        if batch.is_empty() {
            return;
        }
        let events: Vec<_> = batch.into_iter().map(HostEvent::Application).collect();
        if let Err(e) = self.js.dispatch_host_frame(&events) {
            tracing::error!(target: "host_msg", error = ?e, count = events.len(), "dispatch Host application frame failed");
        }
    }

    /// Take a [`ReloadHandle`] the HMR client uses to push updates; the applier
    /// drains them in [`build_frame`] before the next tick.
    pub fn reload_handle(&mut self) -> ReloadHandle {
        let (tx, rx) = mpsc::channel();
        self.reload_rx = Some(rx);
        ReloadHandle {
            tx,
            pending: self.has_hmr_pending.clone(),
        }
    }

    /// Record the Vite entry path so declined HMR can re-import it in-process.
    pub fn set_vite_entry(&mut self, entry: impl Into<String>) {
        self.vite_entry = Some(entry.into());
    }

    /// Last HMR batch outcome (updated each `build_frame` that drains the queue).
    pub fn last_hmr_result(&self) -> &HmrDrainResult {
        &self.last_hmr_result
    }

    /// Drain every pending [`ReloadMsg`] into one batch and apply it.
    ///
    /// **Order:** native CSS updates are logged only (Style IR arrives via
    /// `pending_css` / virtual stylesheet in the same frame). JS updates run
    /// next; any reject/error or explicit full-reload payload resets the scene
    /// and re-imports the Vite entry when configured.
    pub(super) fn drain_hmr_batch(&mut self) -> HmrDrainResult {
        let msgs = {
            let Some(rx) = &self.reload_rx else {
                return HmrDrainResult::Idle;
            };
            let mut msgs = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                msgs.push(msg);
            }
            msgs
        };
        if msgs.is_empty() {
            return HmrDrainResult::Idle;
        }
        let batch = plan_hmr_batch(msgs);
        self.apply_hmr_batch(batch)
    }

    pub(super) fn apply_hmr_batch(&mut self, batch: HmrBatch) -> HmrDrainResult {
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
                match self.js.apply_hmr_update(
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
            // CSS-only batches still report Applied (Style IR may have updated
            // via pending_css in the same frame).
            HmrDrainResult::Applied {
                js_updates: applied,
            }
        } else {
            HmrDrainResult::Idle
        }
    }

    /// Drop all non-root host nodes and re-import the Vite entry when possible.
    pub(super) fn perform_full_reload(&mut self, reason: &str) {
        tracing::warn!(target: "hmr", %reason, "performing in-process full reload");
        self.reset_scene_tree();

        #[cfg(feature = "vite")]
        {
            if let Some(entry) = self.vite_entry.clone() {
                match self.js.reboot_vite_entry(&entry) {
                    Ok(()) => {
                        tracing::info!(target: "hmr", %entry, "vite entry re-imported after full reload");
                        self.invalidation
                            .insert(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
                        self.has_raf = true;
                    }
                    Err(e) => {
                        tracing::error!(
                            target: "hmr",
                            %entry,
                            error = ?e,
                            "full reload re-import failed — restart wabou-quick"
                        );
                    }
                }
                return;
            }
        }

        tracing::error!(
            target: "hmr",
            %reason,
            "full reload requested but no vite entry is configured — restart wabou-quick"
        );
    }

    /// Clear retained UI state down to the host root (solid id 1).
    pub(super) fn reset_scene_tree(&mut self) {
        let doomed: Vec<NodeId> = self
            .node_store
            .solid_to_node
            .iter()
            .filter(|(solid, _)| **solid != 1)
            .map(|(_, node)| *node)
            .collect();
        for node in doomed {
            let _ = self.node_store.tree.remove(node);
        }
        self.node_store.solid_to_node.retain(|id, _| *id == 1);
        self.node_store.node_to_solid.retain(|_, id| *id == 1);
        self.node_store
            .declared
            .retain(|node, _| *node == self.node_store.root);
        self.node_store.children.clear();
        self.node_store
            .children
            .insert(self.node_store.root, Vec::new());
        let _ = self.node_store.tree.set_children(self.node_store.root, &[]);
        self.node_store.collapsed_text.clear();
        self.node_store.inline_roots.clear();
        self.svg_cache.clear();
        self.runtime_transforms.clear();
        self.overlay_planes.clear();
        self.scrollbar_styles.clear();
        self.style_diagnostics.clear();
        self.widget_manager.widgets.clear();
        self.widget_manager.styles.clear();
        self.input.listeners.clear();
        self.scroll_offsets.clear();
        self.pending_scroll_events.clear();
        self.scrollbar_hits.clear();
        self.scrollbar_drag = None;
        self.hovered_scrollbar = None;
        self.scrollbar_activity.clear();
        self.node_store.logical_parent.clear();
        self.projections.semantic_snapshot = Arc::new(SemanticSnapshot::default());
        self.projections.semantics_dirty = true;
        self.widget_manager.pending_value_sync.clear();
        self.dirty_styles.clear();
        self.input.pointer_down_target = None;
        self.input.pointer_down_position = None;
        self.input.pointer_dragged = false;
        self.input.hovered_target = None;
        self.input.focused_target = None;
        self.invalidation
            .insert(InvalidationFlags::LAYOUT | InvalidationFlags::INHERIT);
        if let Ok(mut targets) = self.resize_targets.try_borrow_mut() {
            targets.clear();
        }
    }
}
