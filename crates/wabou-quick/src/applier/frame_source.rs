use super::*;

impl FrameSource for Applier {
    fn set_device_scale(&mut self, scale: f64) {
        self.device_scale = scale.max(f64::EPSILON);
    }

    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode> {
        let build_span = tracing::trace_span!(target: "wabou::perf", "quick.build_frame");
        let _build_guard = build_span.enter();
        self.invalidation.remove(InvalidationFlags::TICK);
        self.js.take_async_wake();
        self.js.poll_async_runtime();

        // Drain fonts queued by the Host API (typically once at boot):
        // register each blob into the shared text FontContext + clear the layout
        // cache so subsequent text measurement picks up the new family.
        if let Some(pf) = self.pending_fonts.clone() {
            let queued = std::mem::take(&mut *pf.borrow_mut());
            for bytes in queued {
                tcx.load_font(bytes);
            }
        }

        // Drain a stylesheet pushed through the private host ABI (the UnoCSS Vite
        // plugin's virtual module): replace the Style IR + re-resolve every node.
        // Clone the Rc out so the mutable self borrows below don't alias it.
        if let Some(p) = self.pending_css.clone()
            && let Some(update) = p.borrow_mut().take()
        {
            match update {
                StylesheetUpdate::Ir(sheet) if sheet.validate().is_ok() => {
                    for diagnostic in &sheet.diagnostics {
                        tracing::warn!(target: "stylesheet", %diagnostic);
                    }
                    // Build the class→rules index (interning each rule's
                    // class_name so node-side class atoms match by identity)
                    // so recompute_node_now matches in O(C) not O(R).
                    let (rule_index, universal_rules) = {
                        let mut atoms = self.atoms.borrow_mut();
                        let mut rule_index: HashMap<Atom, Vec<usize>> = HashMap::new();
                        let mut universal_rules = Vec::new();
                        for (idx, rule) in sheet.rules.iter().enumerate() {
                            for declaration in &rule.declarations {
                                atoms.intern(&declaration.property);
                            }
                            if rule.class_name == "*" {
                                universal_rules.push(idx);
                            } else {
                                rule_index
                                    .entry(atoms.intern(&rule.class_name))
                                    .or_default()
                                    .push(idx);
                            }
                        }
                        (rule_index, universal_rules)
                    };
                    self.style_theme = sheet.theme.clone();
                    if let Some(themes) = &sheet.color_themes {
                        let selected = self
                            .active_color_theme
                            .as_ref()
                            .filter(|name| themes.themes.contains_key(*name))
                            .cloned()
                            .unwrap_or_else(|| themes.default.clone());
                        self.active_theme_colors =
                            Arc::new(themes.themes[&selected].colors.clone());
                        // Runtime-created utilities must accept the same
                        // semantic color names as the compiler. These values
                        // validate parsing; declarations remain theme tokens.
                        self.style_theme.colors.extend(
                            self.active_theme_colors
                                .iter()
                                .map(|(name, color)| (name.clone(), *color)),
                        );
                        self.active_color_theme = Some(selected);
                    } else {
                        self.active_color_theme = None;
                        self.active_theme_colors = Arc::new(HashMap::new());
                    }
                    self.style_ir = Some(sheet);
                    self.rule_index = rule_index;
                    self.universal_rules = universal_rules;
                    self.utility_cache.clear();
                    self.class_resolution_cache.clear();
                    self.warned_utility_classes.clear();
                    self.warned_ir_properties.clear();
                }
                StylesheetUpdate::Ir(sheet) => {
                    tracing::error!(
                        version = sheet.version,
                        supported = style_ir::VERSION,
                        "invalid or unsupported Style IR"
                    );
                }
            }
            self.recompute_all();
        }

        if let Some(pending) = self.pending_color_theme.clone()
            && let Some(name) = pending.borrow_mut().take()
        {
            let selected = self
                .style_ir
                .as_ref()
                .and_then(|sheet| sheet.color_themes.as_ref())
                .and_then(|themes| themes.themes.get(&name));
            if let Some(theme) = selected {
                if self.active_color_theme.as_deref() != Some(name.as_str()) {
                    self.active_theme_colors = Arc::new(theme.colors.clone());
                    self.active_color_theme = Some(name);
                    self.class_resolution_cache.clear();
                    self.recompute_color_palette();
                }
            } else {
                tracing::warn!(theme = %name, "unknown Wabou color theme");
            }
        }

        if let Some(pending) = self.pending_color_palette.clone()
            && let Some(colors) = pending.borrow_mut().take()
        {
            let tokens = self
                .style_ir
                .as_ref()
                .and_then(|sheet| sheet.color_themes.as_ref())
                .and_then(|themes| themes.themes.get(&themes.default))
                .map(|theme| {
                    let mut tokens = theme.colors.keys().cloned().collect::<Vec<_>>();
                    tokens.sort_unstable();
                    tokens
                });
            if let Some(tokens) = tokens {
                if tokens.len() == colors.len() {
                    self.active_theme_colors = Arc::new(tokens.into_iter().zip(colors).collect());
                    self.class_resolution_cache.clear();
                    self.recompute_color_palette();
                } else {
                    tracing::warn!(
                        expected = tokens.len(),
                        actual = colors.len(),
                        "ignored Wabou color palette with the wrong token count"
                    );
                }
            }
        }

        // Drain Vite HMR updates (from the background websocket client) before
        // the tick so re-imported module effects land in this frame's flush.
        // Style IR is already applied above via pending_css (same frame as the
        // virtual:wabou-stylesheet JS update when both fire together).
        let hmr = self.drain_hmr_batch();
        if !matches!(hmr, HmrDrainResult::Idle) {
            self.last_hmr_result = hmr;
        }
        self.has_hmr_pending.store(false, Ordering::Release);

        // Host application messages before tick so subscribe
        // handlers can update signals before this frame's rAF flush.
        self.drain_host_messages();
        self.dispatch_scroll_changes();

        // One rAF round-trip: runs queued rAF callbacks (Solid effects re-emit
        // ops), flushes the writer → __wabou_flush lands bytes here. Timed so
        // the host overlay can show the QuickJS portion of build_frame.
        let js_t0 = std::time::Instant::now();
        let (bytes, has_raf) = {
            let span = tracing::trace_span!(target: "wabou::perf", "quick.js_tick");
            let _guard = span.enter();
            match self.js.tick() {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(target: "bridge", "JS tick failed: {e:?}");
                    self.has_raf = false;
                    return Vec::new();
                }
            }
        };
        let js_tick_ms = js_t0.elapsed().as_secs_f64() * 1000.0;
        self.js_tick_ema = self.js_tick_ema * 0.9 + js_tick_ms * 0.1;
        self.last_viewport = (width, height);
        self.has_raf = has_raf;
        if !bytes.is_empty() {
            match decode_frame(&bytes) {
                Ok(frame) => {
                    if let Some(state) = &self.projections.debug_state
                        && let Ok(mut state) = state.write()
                    {
                        state.push_frame(wabou_devtools::DebugFrame {
                            direction: "jsToHost".into(),
                            sequence: u64::from(frame.seq),
                            byte_len: bytes.len(),
                            record_count: frame.ops.len(),
                            bytes_hex: Some(wabou_devtools::bytes_hex(&bytes, 4096)),
                        });
                    }
                    self.apply_frame(&frame)
                }
                Err(e) => tracing::error!(target: "bridge", "decode frame failed: {e}"),
            }
        }
        // Arm rquickjs's async scheduler after this tick may have started new
        // work. Pending IO keeps its waker and does not imply animation.
        self.js.poll_async_runtime();

        let selection_scrolled = self.tick_text_selection_autoscroll();
        // Only re-inherit when a change can affect inherited content styles.
        // Per-frame non-inherited animation sets LAYOUT but not INHERIT, so
        // this O(N) pass remains skipped for those frames.
        if self.invalidation.contains(InvalidationFlags::INHERIT) {
            self.inherit();
            self.invalidation.remove(InvalidationFlags::INHERIT);
        }
        self.sync_widget_styles();
        self.measure_widgets(tcx);
        let viewport = (width, height);
        let viewport_changed = self.layout_viewport != Some(viewport);
        let semantic_layout_dirty =
            self.invalidation.contains(InvalidationFlags::LAYOUT) || viewport_changed;
        let mut placed = if self.invalidation.contains(InvalidationFlags::LAYOUT)
            || viewport_changed
        {
            // A root percentage has no containing block in taffy and resolves
            // to zero. Only update it when the viewport changes: set_style
            // invalidates Taffy's retained layout cache.
            if viewport_changed && let Ok(style) = self.node_store.tree.style(self.node_store.root)
            {
                let mut style = style.clone();
                style.size.width = taffy::Dimension::length(width as f32);
                style.size.height = taffy::Dimension::length(height as f32);
                let _ = self.node_store.tree.set_style(self.node_store.root, style);
            }
            let mut placed = layout::compute_and_walk_with_scroll(
                &mut self.node_store.tree,
                self.node_store.root,
                width as f32,
                height as f32,
                tcx,
                &self.scroll_offsets,
            );
            if self.clamp_scroll_offsets(&placed) {
                placed = layout::flatten_with_scroll(
                    &self.node_store.tree,
                    self.node_store.root,
                    &self.scroll_offsets,
                );
            }
            self.invalidation.remove(InvalidationFlags::LAYOUT);
            self.layout_viewport = Some(viewport);
            let resize_changed = self.dispatch_resize_changes();
            self.invalidation
                .set(InvalidationFlags::TICK, resize_changed);
            self.paint_widgets(&mut placed, tcx);
            placed
        } else {
            let mut placed = layout::flatten_with_scroll(
                &self.node_store.tree,
                self.node_store.root,
                &self.scroll_offsets,
            );
            self.paint_widgets(&mut placed, tcx);
            placed
        };
        self.update_scrollbar_visuals(&mut placed);
        self.placed_rects.clear();
        self.placed_rects
            .extend(placed.iter().map(|placed| (placed.node_id, placed.rect)));
        self.rebuild_hit_geometry(&placed);
        self.rebuild_focus_order(&placed);
        let projection_dirty =
            self.projections.semantics_dirty || semantic_layout_dirty || selection_scrolled;
        if projection_dirty {
            self.publish_layout_metrics(&placed, width, height);
        }
        self.prepare_text_selection(&mut placed, tcx);
        if selection_scrolled {
            let target = self
                .input
                .hit_test(self.input.pointer_position.0, self.input.pointer_position.1);
            self.extend_text_selection(
                target,
                self.input.pointer_position.0,
                self.input.pointer_position.1,
            );
            self.prepare_text_selection(&mut placed, tcx);
        }
        if self.input.pointer_buttons & 1 == 0 {
            self.sync_text_selection_change();
        }
        if self.projections.semantics_enabled && projection_dirty {
            self.rebuild_semantic_snapshot(&placed);
        }
        self.projections.semantics_dirty = false;
        // After paint applied pending edits, sync widget values → JS.
        self.flush_value_sync();
        if projection_dirty || self.projections.debug_dirty {
            self.publish_debug_snapshot(&placed);
            self.projections.debug_dirty = false;
        }
        placed
    }

    fn base_color(&self) -> Color {
        self.base_color
    }

    fn set_semantics_enabled(&mut self, enabled: bool) {
        self.projections.set_semantics_enabled(enabled);
    }

    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        self.projections.semantic_snapshot()
    }

    fn handle_semantic_action(&mut self, action: SemanticAction) -> bool {
        let target = match action {
            SemanticAction::Click { target }
            | SemanticAction::Focus { target }
            | SemanticAction::Blur { target } => u32::try_from(target).ok(),
        };
        let Some(target) =
            target.filter(|target| self.node_store.solid_to_node.contains_key(target))
        else {
            return false;
        };
        if !self
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .any(|node| node.id == u64::from(target))
        {
            return false;
        }
        if let Some(modal) = self.projections.semantic_snapshot.modal_root {
            let Some(modal_node) = u32::try_from(modal)
                .ok()
                .and_then(|modal| self.node_store.solid_to_node.get(&modal).copied())
            else {
                return false;
            };
            if !self
                .node_store
                .solid_to_node
                .get(&target)
                .is_some_and(|node| self.node_store.is_logical_descendant(*node, modal_node))
            {
                return false;
            }
        }
        match action {
            SemanticAction::Click { .. } => {
                self.dispatch_pointer(target, event::CLICK, None, Modifiers::empty())
            }
            SemanticAction::Focus { .. } => {
                let changed = self.input.focused_target != Some(target);
                self.set_focused_target(Some(target));
                changed
            }
            SemanticAction::Blur { .. } => {
                let changed = self.input.focused_target == Some(target);
                if changed {
                    self.set_focused_target(None);
                }
                changed
            }
        }
    }

    fn ime_cursor_area(&self) -> Option<[f64; 4]> {
        self.ime_cursor_area
    }

    fn paint_debug_overlay(
        &mut self,
        scene: &mut Scene,
        placed: &[PlacedNode],
        tcx: &mut TextContext,
        device_scale: f64,
    ) {
        let Some(state) = &self.projections.debug_state else {
            return;
        };
        let Ok(state) = state.read() else { return };
        let overlay = state.overlay();
        if !overlay.is_enabled() {
            return;
        }
        let device = Affine::scale(device_scale);
        let hovered = self.input.hovered_target;
        let mut clips = HashSet::new();

        for node in placed {
            let Some(&solid_id) = self.node_store.node_to_solid.get(&node.node_id) else {
                continue;
            };
            let [x0, y0, x1, y1] = node.rect;
            let rect = Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64);
            if overlay.layout {
                scene.stroke(
                    &Stroke::new(1.0),
                    device,
                    Color::from_rgba8(56, 189, 248, 190),
                    None,
                    &rect,
                );
            }
            if overlay.clips
                && let Some(clip) = node.clip
                && clips.insert(clip.map(f32::to_bits))
            {
                scene.stroke(
                    &Stroke::new(1.5),
                    device,
                    Color::from_rgba8(251, 146, 60, 220),
                    None,
                    &Rect::new(
                        clip[0] as f64,
                        clip[1] as f64,
                        clip[2] as f64,
                        clip[3] as f64,
                    ),
                );
            }

            let is_hit = overlay.hit_target && hovered == Some(solid_id);
            let is_selected = overlay.selected_node == Some(solid_id);
            if !is_hit && !is_selected {
                continue;
            }
            let accent = if is_selected {
                Color::from_rgba8(168, 85, 247, 255)
            } else {
                Color::from_rgba8(244, 63, 94, 255)
            };
            scene.fill(
                Fill::NonZero,
                device,
                Color::from_rgba8(244, 63, 94, 25),
                None,
                &rect,
            );
            scene.stroke(&Stroke::new(2.0), device, accent, None, &rect);

            let atoms = self.atoms.borrow();
            let tag = self
                .node_store
                .declared
                .get(&node.node_id)
                .and_then(|declared| declared.tag)
                .and_then(|tag| atoms.resolve(tag))
                .unwrap_or("#text");
            let label: Arc<str> = format!("{tag}#{solid_id}").into();
            drop(atoms);
            let layout = layout_text_styled(
                tcx,
                label,
                11.0,
                600.0,
                None,
                TextAlign::Start,
                [255, 255, 255, 255],
                Arc::from([]),
                None,
                None,
            );
            let label_width = layout.width() as f64 + 8.0;
            let label_height = layout.height() as f64 + 4.0;
            let label_y = (y0 as f64 - label_height).max(0.0);
            let label_rect = Rect::new(
                x0 as f64,
                label_y,
                x0 as f64 + label_width,
                label_y + label_height,
            );
            scene.fill(Fill::NonZero, device, accent, None, &label_rect);
            let glyphs = tcx.glyph_scene_scaled(&layout, device_scale);
            scene.append(
                &glyphs,
                Some(
                    device
                        * Affine::translate((x0 as f64 + 4.0, label_y + 2.0))
                        * Affine::scale(device_scale.recip()),
                ),
            );
        }
    }

    fn push_frame_stats(&mut self, stats: &FrameStats) {
        if let Some(cell) = &self.frame_stats {
            // The app fills build_frame/scene/present/node_count; fold in the
            // QuickJS tick EMA + last viewport the applier measured.
            let mut s = *stats;
            s.js_tick_ms = self.js_tick_ema;
            s.viewport_w = self.last_viewport.0;
            s.viewport_h = self.last_viewport.1;
            *cell.borrow_mut() = Some(s);
        }
    }

    fn has_anim(&self) -> bool {
        self.has_raf
            || self.has_hmr_pending.load(Ordering::Acquire)
            || self.host_msg_inbox.has_pending()
            || self.js.has_async_wake()
            || self.invalidation.contains(InvalidationFlags::TICK)
    }

    fn animation_deadline(&self) -> Option<Instant> {
        let now = Instant::now();
        let scrollbar_deadline = self
            .scrollbar_activity
            .iter()
            .filter_map(|(node, started)| {
                if self.scrollbar_drag.is_some_and(|drag| drag.node == *node)
                    || self
                        .hovered_scrollbar
                        .is_some_and(|(owner, _)| owner == *node)
                {
                    return None;
                }
                let style = self.node_store.tree.get_node_context(*node)?.scrollbar;
                if style.visibility != ScrollbarVisibility::Auto {
                    return None;
                }
                let fade_start = *started + style.hide_delay;
                if now < fade_start {
                    Some(fade_start)
                } else if style.fade_duration.is_zero() || now >= fade_start + style.fade_duration {
                    None
                } else {
                    Some(now + Duration::from_millis(16))
                }
            });
        self.widget_manager
            .widgets
            .values()
            .filter_map(|widget| widget.animation_deadline())
            .chain(self.next_text_selection_scroll)
            .chain(scrollbar_deadline)
            .min()
    }

    fn set_wake_callback(&mut self, wake: WakeCallback) {
        if let Some(state) = &self.projections.debug_state
            && let Ok(mut state) = state.write()
        {
            state.set_wake(wake.clone());
        }
        self.js.set_wake_callback(wake.clone());
        for widget in self.widget_manager.widgets.values_mut() {
            widget.set_wake_callback(wake.clone());
        }
        self.host_msg_inbox.set_wake(wake.clone());
        *self.host_action_wake.borrow_mut() = Some(wake.clone());
        self.wake_callback = Some(wake);
    }

    fn poll_async(&mut self) -> bool {
        let was_woken = self.js.take_async_wake();
        self.js.poll_async_runtime();
        let mut widget_woken = false;
        let mut host_actions = Vec::new();
        let mut node_events = Vec::new();
        for (node, widget) in &mut self.widget_manager.widgets {
            widget_woken |= widget.poll_async();
            while let Some(action) = widget.take_host_action() {
                host_actions.push((*node, action));
            }
            while let Some(event) = widget.take_node_event() {
                node_events.push((*node, event));
            }
        }
        for (node, action) in host_actions {
            self.enqueue_widget_host_action(node, action);
        }
        for (node, event) in node_events {
            let Some(target) = self.node_store.solid_id_for_node(node) else {
                continue;
            };
            widget_woken |= self.dispatch_json(target, event.event_code, &event.json);
        }
        let screenshot_pending = self
            .projections
            .debug_state
            .as_ref()
            .and_then(|state| state.read().ok())
            .is_some_and(|state| state.has_screenshot_request());
        let overlay_changed = self
            .projections
            .debug_state
            .as_ref()
            .and_then(|state| state.write().ok())
            .is_some_and(|mut state| state.take_overlay_change());
        widget_woken || was_woken || screenshot_pending || overlay_changed
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.pending_host_actions.borrow_mut().pop_front()
    }

    fn complete_host_action(&mut self, result: wabou_shell::HostActionResult) {
        match result {
            wabou_shell::HostActionResult::Clipboard { request_id, text } => {
                let Some((node, widget_request_id)) =
                    self.widget_manager.host_action_routes.remove(&request_id)
                else {
                    return;
                };
                if let Some(widget) = self.widget_manager.widgets.get_mut(&node) {
                    widget.complete_host_action(wabou_shell::HostActionResult::Clipboard {
                        request_id: widget_request_id,
                        text,
                    });
                }
            }
            wabou_shell::HostActionResult::ClipboardWrite {
                request_id,
                success,
            } => {
                let _ = (request_id, success);
            }
        }
    }

    fn take_effect(&mut self) -> Option<wabou_shell::EffectRequest> {
        while let Some(completion) = self.replay_completions.borrow_mut().pop_front() {
            if self
                .pending_js_effects
                .borrow_mut()
                .remove(&completion.id.0)
            {
                complete_js_effect(&self.js, &completion);
            }
        }
        self.pending_effects.borrow_mut().pop_front()
    }

    fn complete_effect(&mut self, completion: wabou_shell::EffectCompletion) {
        if let Some(trace) = self.effect_trace.borrow().as_ref() {
            trace.complete(&completion);
        }
        if self
            .pending_js_effects
            .borrow_mut()
            .remove(&completion.id.0)
        {
            complete_js_effect(&self.js, &completion);
        }
    }

    fn take_screenshot_request(&mut self) -> Option<std::path::PathBuf> {
        self.projections
            .debug_state
            .as_ref()?
            .write()
            .ok()?
            .take_screenshot_request()
    }

    fn complete_screenshot(&mut self, result: Result<std::path::PathBuf, String>) {
        if let Some(state) = &self.projections.debug_state
            && let Ok(mut state) = state.write()
        {
            state.complete_screenshot(result);
        }
    }

    fn handle_event(&mut self, input: UiEvent) -> EventResponse {
        self.projections.debug_dirty |= self.projections.debug_state.is_some();
        if let UiEvent::WindowMetrics(metrics) = &input {
            return self.handle_window_metrics(*metrics);
        }
        if matches!(
            &input,
            UiEvent::Key(key) if key.phase == KeyPhase::Down
        ) || matches!(
            &input,
            UiEvent::TextInput(_) | UiEvent::Ime(_) | UiEvent::Paste(_) | UiEvent::Wheel(_)
        ) || matches!(
            &input,
            UiEvent::Pointer(pointer)
                if pointer.phase == PointerPhase::Down
                    && pointer.button != Some(PointerButton::Primary)
        ) {
            self.last_text_click = None;
        }
        // Application shortcuts get first refusal before a focused Rust
        // widget consumes the key. This is the native equivalent of a
        // cancellable keydown boundary: preventDefault keeps shortcuts like
        // Cmd+T out of a terminal PTY while ordinary keys continue unchanged.
        let keydown_dispatched = if let UiEvent::Key(key) = &input
            && key.phase == KeyPhase::Down
            && let Some(target) = self.input.focused_target
        {
            let payload = key_event_payload(key);
            let (dispatched, prevented) =
                self.dispatch_cancellable_json(target, event::KEYDOWN, payload);
            if prevented {
                return EventResponse {
                    handled: true,
                    request_redraw: true,
                    consume_key_text: true,
                    text_input: None,
                    clipboard: None,
                };
            }
            dispatched
        } else {
            false
        };
        // Keyboard and committed text belong to the focused widget. Pointer
        // focus is established in the pointer-down branch before delivery.
        let widget_response = if matches!(
            input,
            UiEvent::Key(_) | UiEvent::TextInput(_) | UiEvent::Ime(_) | UiEvent::Paste(_)
        ) && let Some(target) = self.input.focused_target
        {
            self.handle_widget_event(target, &input)
        } else {
            None
        };
        if widget_response.is_none()
            && let UiEvent::Key(key) = &input
            && key.matches_standard_shortcut(wabou_shell::StandardShortcut::Copy)
            && let Some(text) = self.selected_text()
        {
            return EventResponse {
                handled: true,
                request_redraw: false,
                consume_key_text: false,
                text_input: None,
                clipboard: Some(wabou_shell::ClipboardRequest::Write(text)),
            };
        }
        if widget_response.is_none()
            && let UiEvent::Key(key) = &input
            && key.matches_standard_shortcut(wabou_shell::StandardShortcut::SelectAll)
            && self.select_all_text()
        {
            self.sync_text_selection_change();
            return EventResponse {
                handled: true,
                request_redraw: true,
                consume_key_text: false,
                text_input: None,
                clipboard: None,
            };
        }
        if widget_response.is_none()
            && let UiEvent::Key(key) = &input
            && key.phase == KeyPhase::Down
            && key.key == "Tab"
            && !key.modifiers.control()
            && !key.modifiers.alt()
            && !key.modifiers.meta()
            && let Some(target) = self.advance_focus(key.modifiers.shift())
        {
            return EventResponse {
                handled: true,
                request_redraw: true,
                consume_key_text: true,
                text_input: Some(self.is_text_input_target(target)),
                clipboard: None,
            };
        }

        let handled = match input {
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Move => {
                return self.handle_pointer_move(pointer);
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Down => {
                return self.handle_pointer_down(pointer);
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Up => {
                return self.handle_pointer_up(pointer);
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Cancel => {
                return self.handle_pointer_cancel(pointer);
            }
            UiEvent::Wheel(wheel) => return self.handle_wheel_event(wheel),
            UiEvent::Key(key) if key.phase == KeyPhase::Down => keydown_dispatched,
            UiEvent::Key(key) if key.phase == KeyPhase::Up => {
                self.input.focused_target.is_some_and(|target| {
                    let payload = key_event_payload(&key);
                    self.dispatch_json(target, event::KEYUP, &payload)
                })
            }
            UiEvent::TextInput(text)
            | UiEvent::Ime(wabou_shell::ImeEvent::Commit(text))
            | UiEvent::Paste(text) => self.input.focused_target.is_some_and(|target| {
                let payload = serde_json::json!({ "data": text }).to_string();
                self.dispatch_json(target, event::IMECOMMIT, &payload)
            }),
            UiEvent::Ime(_) => widget_response.is_some(),
            UiEvent::Focus(focused) => return self.handle_window_focus(focused),
            UiEvent::Pointer(_) | UiEvent::Key(_) | UiEvent::WindowMetrics(_) => false,
        };
        if let Some(widget) = widget_response {
            EventResponse {
                handled: widget.handled || handled,
                request_redraw: widget.request_redraw || handled,
                consume_key_text: widget.consume_key_text,
                text_input: widget.text_input,
                clipboard: widget.clipboard,
            }
        } else {
            Self::response(handled)
        }
    }
}
