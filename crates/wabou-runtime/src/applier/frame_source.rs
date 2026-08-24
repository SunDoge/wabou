use super::*;

impl Applier {
    fn drain_pending_fonts(&mut self, tcx: &mut TextContext) {
        let Some(fonts) = self.runtime.pending_fonts.clone() else {
            return;
        };
        for bytes in std::mem::take(&mut *fonts.borrow_mut()) {
            tcx.load_font(bytes);
        }
    }

    fn drain_pending_stylesheet(&mut self) {
        let Some(pending) = self.runtime.pending_css.clone() else {
            return;
        };
        let Some(update) = pending.borrow_mut().take() else {
            return;
        };
        match update {
            StylesheetUpdate::Ir(sheet) if sheet.validate().is_ok() => {
                for diagnostic in &sheet.diagnostics {
                    tracing::warn!(target: "stylesheet", %diagnostic);
                }
                let (rule_index, universal_rules) = {
                    let mut atoms = self.document.atoms.borrow_mut();
                    let mut rule_index: HashMap<Atom, Vec<usize>> = HashMap::new();
                    let mut universal_rules = Vec::new();
                    for (index, rule) in sheet.rules.iter().enumerate() {
                        for declaration in &rule.declarations {
                            atoms.intern(&declaration.property);
                        }
                        if rule.class_name == "*" {
                            universal_rules.push(index);
                        } else {
                            rule_index
                                .entry(atoms.intern(&rule.class_name))
                                .or_default()
                                .push(index);
                        }
                    }
                    (rule_index, universal_rules)
                };
                self.document.style.theme = sheet.theme.clone();
                if let Some(themes) = &sheet.color_themes {
                    let selected = self
                        .document
                        .style
                        .active_color_theme
                        .as_ref()
                        .filter(|name| themes.themes.contains_key(*name))
                        .cloned()
                        .unwrap_or_else(|| themes.default.clone());
                    self.document.style.active_theme_colors =
                        Arc::new(themes.themes[&selected].colors.clone());
                    self.document.style.theme.colors.extend(
                        self.document
                            .style
                            .active_theme_colors
                            .iter()
                            .map(|(name, color)| (name.clone(), *color)),
                    );
                    self.document.style.active_color_theme = Some(selected);
                } else {
                    self.document.style.active_color_theme = None;
                    self.document.style.active_theme_colors = Arc::new(HashMap::new());
                }
                self.document.style.sheet = Some(sheet);
                self.document.style.rule_index = rule_index;
                self.document.style.universal_rules = universal_rules;
                self.document.style.utility_cache.clear();
                self.document.style.class_resolution_cache.clear();
                self.document.style.warned_utility_classes.clear();
                self.document.style.warned_ir_properties.clear();
            }
            StylesheetUpdate::Ir(sheet) => tracing::error!(
                version = sheet.version,
                supported = style_ir::VERSION,
                "invalid or unsupported Style IR"
            ),
        }
        self.recompute_all();
    }

    fn drain_pending_color_theme(&mut self) {
        let Some(pending) = self.runtime.pending_color_theme.clone() else {
            return;
        };
        let Some(name) = pending.borrow_mut().take() else {
            return;
        };
        let selected = self
            .document
            .style
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.color_themes.as_ref())
            .and_then(|themes| themes.themes.get(&name));
        if let Some(theme) = selected {
            if self.document.style.active_color_theme.as_deref() != Some(name.as_str()) {
                self.document.style.active_theme_colors = Arc::new(theme.colors.clone());
                self.document.style.active_color_theme = Some(name);
                self.document.style.class_resolution_cache.clear();
                self.recompute_color_palette();
            }
        } else {
            tracing::warn!(theme = %name, "unknown Wabou color theme");
        }
    }

    fn drain_pending_color_palette(&mut self) {
        let Some(pending) = self.runtime.pending_color_palette.clone() else {
            return;
        };
        let Some(colors) = pending.borrow_mut().take() else {
            return;
        };
        let tokens = self
            .document
            .style
            .sheet
            .as_ref()
            .and_then(|sheet| sheet.color_themes.as_ref())
            .and_then(|themes| themes.themes.get(&themes.default))
            .map(|theme| {
                let mut tokens = theme.colors.keys().cloned().collect::<Vec<_>>();
                tokens.sort_unstable();
                tokens
            });
        let Some(tokens) = tokens else {
            return;
        };
        if tokens.len() == colors.len() {
            self.document.style.active_theme_colors =
                Arc::new(tokens.into_iter().zip(colors).collect());
            self.document.style.class_resolution_cache.clear();
            self.recompute_color_palette();
        } else {
            tracing::warn!(
                expected = tokens.len(),
                actual = colors.len(),
                "ignored Wabou color palette with the wrong token count"
            );
        }
    }

    fn run_javascript_tick(&mut self, width: u32, height: u32) -> bool {
        let hmr = self.drain_hmr_batch();
        if !matches!(hmr, HmrDrainResult::Idle) {
            self.runtime.reload.record_result(hmr);
        }
        self.runtime.reload.clear_pending();
        self.drain_host_messages();
        self.dispatch_scroll_changes();

        let started = std::time::Instant::now();
        let result = {
            #[cfg(feature = "profiling")]
            let span = tracing::trace_span!(target: "wabou::perf", "quick.js_tick");
            #[cfg(feature = "profiling")]
            let _guard = span.enter();
            self.runtime.js.tick()
        };
        let (bytes, has_raf) = match result {
            Ok(result) => result,
            Err(error) => {
                tracing::error!(target: "bridge", "JS tick failed: {error:?}");
                self.runtime.has_raf = false;
                return false;
            }
        };
        let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
        self.frame.js_tick_ema = self.frame.js_tick_ema * 0.9 + elapsed_ms * 0.1;
        self.frame.last_viewport = (width, height);
        self.runtime.has_raf = has_raf;
        if !bytes.is_empty() {
            let decoded = {
                #[cfg(feature = "profiling")]
                let span = tracing::trace_span!(
                    target: "wabou::perf",
                    "quick.protocol.decode",
                    bytes = bytes.len() as u64,
                );
                #[cfg(feature = "profiling")]
                let _guard = span.enter();
                decode_frame(&bytes)
            };
            match decoded {
                Ok(frame) => {
                    self.runtime.protocol_revision = self.runtime.protocol_revision.wrapping_add(1);
                    #[cfg(any(feature = "devtools", test))]
                    if let Some(state) = &self.frame.projections.debug_state
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
                    {
                        #[cfg(feature = "profiling")]
                        let span = tracing::trace_span!(
                            target: "wabou::perf",
                            "quick.protocol.apply",
                            ops = frame.ops.len() as u64,
                            class_cache_hits = tracing::field::Empty,
                            class_cache_misses = tracing::field::Empty,
                            runtime_utility_fallbacks = tracing::field::Empty,
                        );
                        #[cfg(feature = "profiling")]
                        let _guard = span.enter();
                        self.apply_frame(&frame);
                        #[cfg(feature = "profiling")]
                        {
                            span.record("class_cache_hits", self.frame.profile_class_cache_hits);
                            span.record(
                                "class_cache_misses",
                                self.frame.profile_class_cache_misses,
                            );
                            span.record(
                                "runtime_utility_fallbacks",
                                self.frame.profile_runtime_utility_fallbacks,
                            );
                        }
                    }
                }
                Err(error) => tracing::error!(target: "bridge", "decode frame failed: {error}"),
            }
        }
        self.runtime.js.poll_async_runtime();
        true
    }
}

impl FrameSource for Applier {
    fn set_device_scale(&mut self, scale: f64) {
        self.frame.device_scale = scale.max(f64::EPSILON);
    }

    fn build_frame(&mut self, tcx: &mut TextContext, width: u32, height: u32) -> Vec<PlacedNode> {
        #[cfg(feature = "profiling")]
        let build_span = tracing::trace_span!(target: "wabou::perf", "quick.build_frame");
        #[cfg(feature = "profiling")]
        let _build_guard = build_span.enter();
        #[cfg(feature = "profiling")]
        {
            self.frame.profile_class_cache_hits = 0;
            self.frame.profile_class_cache_misses = 0;
            self.frame.profile_runtime_utility_fallbacks = 0;
        }
        self.document.invalidation.remove(InvalidationFlags::TICK);
        self.runtime.js.take_async_wake();
        self.runtime.js.poll_async_runtime();

        self.drain_pending_fonts(tcx);

        self.drain_pending_stylesheet();
        self.drain_pending_color_theme();
        self.drain_pending_color_palette();

        if !self.run_javascript_tick(width, height) {
            return Vec::new();
        }

        let selection_scrolled = self.tick_text_selection_autoscroll();
        // Only re-inherit when a change can affect inherited content styles.
        // Per-frame non-inherited animation sets LAYOUT but not INHERIT, so
        // this O(N) pass remains skipped for those frames.
        if self
            .document
            .invalidation
            .contains(InvalidationFlags::INHERIT)
        {
            {
                #[cfg(feature = "profiling")]
                let span = tracing::trace_span!(
                    target: "wabou::perf",
                    "quick.style.inherit",
                    nodes = self.document.node_store.solid_to_node.len() as u64,
                );
                #[cfg(feature = "profiling")]
                let _guard = span.enter();
                self.inherit();
            }
            self.document
                .invalidation
                .remove(InvalidationFlags::INHERIT);
        }
        {
            #[cfg(feature = "profiling")]
            let span = tracing::trace_span!(target: "wabou::perf", "quick.widgets.measure");
            #[cfg(feature = "profiling")]
            let _guard = span.enter();
            self.sync_widget_styles();
        }
        let viewport = (width, height);
        let viewport_changed = self.document.layout_viewport != Some(viewport);
        let semantic_layout_dirty = self
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
            || viewport_changed;
        let mut placed = if self
            .document
            .invalidation
            .contains(InvalidationFlags::LAYOUT)
            || viewport_changed
        {
            // A root percentage has no containing block in taffy and resolves
            // to zero. Only update it when the viewport changes: set_style
            // invalidates Taffy's retained layout cache.
            if viewport_changed
                && let Ok(style) = self
                    .document
                    .node_store
                    .tree
                    .style(self.document.node_store.root)
            {
                let mut style = style.clone();
                style.size.width = taffy::Dimension::length(width as f32);
                style.size.height = taffy::Dimension::length(height as f32);
                let _ = self
                    .document
                    .node_store
                    .tree
                    .set_style(self.document.node_store.root, style);
            }
            let mut placed = {
                #[cfg(feature = "profiling")]
                let span = tracing::trace_span!(
                    target: "wabou::perf",
                    "quick.layout.compute",
                    nodes = self.document.node_store.solid_to_node.len() as u64,
                    viewport_width = width,
                    viewport_height = height,
                );
                #[cfg(feature = "profiling")]
                let _guard = span.enter();
                layout::compute_and_walk_with_scroll_and_widgets(
                    &mut self.document.node_store.tree,
                    self.document.node_store.root,
                    [width as f32, height as f32],
                    tcx,
                    self.frame.device_scale,
                    |node, cx| {
                        self.document
                            .widget_manager
                            .widgets
                            .get_mut(&node)
                            .and_then(|widget| widget.measure(cx))
                    },
                    &self.interaction.scroll.offsets,
                )
            };
            if self.clamp_scroll_offsets(&placed) {
                placed = layout::flatten_with_scroll(
                    &self.document.node_store.tree,
                    self.document.node_store.root,
                    &self.interaction.scroll.offsets,
                );
            }
            self.document.invalidation.remove(InvalidationFlags::LAYOUT);
            self.document.layout_viewport = Some(viewport);
            let resize_changed = self.dispatch_resize_changes();
            self.document
                .invalidation
                .set(InvalidationFlags::TICK, resize_changed);
            {
                #[cfg(feature = "profiling")]
                let span = tracing::trace_span!(target: "wabou::perf", "quick.widgets.paint");
                #[cfg(feature = "profiling")]
                let _guard = span.enter();
                self.paint_widgets(&mut placed, tcx);
            }
            placed
        } else {
            let mut placed = layout::flatten_with_scroll(
                &self.document.node_store.tree,
                self.document.node_store.root,
                &self.interaction.scroll.offsets,
            );
            {
                #[cfg(feature = "profiling")]
                let span = tracing::trace_span!(target: "wabou::perf", "quick.widgets.paint");
                #[cfg(feature = "profiling")]
                let _guard = span.enter();
                self.paint_widgets(&mut placed, tcx);
            }
            placed
        };
        {
            let projection_dirty = self.frame.projections.semantics_dirty
                || semantic_layout_dirty
                || selection_scrolled;
            // Hit geometry, focus order, and selectable-text indices are retained
            // projections of the placed tree. A requestAnimationFrame callback can
            // produce no host operations, so rebuilding all of them on every such
            // frame is unnecessary O(N) work. Scrollbar fades are included because
            // their changing opacity controls whether a scrollbar participates in
            // hit testing.
            let scrollbars_changing = !self.interaction.scroll.activity.is_empty()
                || self.interaction.scroll.drag.is_some()
                || self.interaction.scroll.hovered.is_some();
            let geometry_dirty = projection_dirty
                || self
                    .document
                    .invalidation
                    .contains(InvalidationFlags::GEOMETRY)
                || scrollbars_changing;
            #[cfg(feature = "profiling")]
            let span = tracing::trace_span!(
                target: "wabou::perf",
                "quick.projections",
                nodes = placed.len() as u64,
                semantic_layout_dirty,
                selection_scrolled,
                geometry_dirty,
            );
            #[cfg(feature = "profiling")]
            let _guard = span.enter();
            self.update_scrollbar_visuals(&mut placed);
            if geometry_dirty {
                self.rebuild_hit_geometry(&placed);
            }
            if projection_dirty {
                self.interaction.scroll.placed_rects.clear();
                self.interaction
                    .scroll
                    .placed_rects
                    .extend(placed.iter().map(|placed| (placed.node_id, placed.rect)));
                self.rebuild_focus_order(&placed);
            }
            if projection_dirty {
                self.publish_layout_metrics(&placed, width, height);
            }
            if projection_dirty || self.interaction.text_selection.active.is_some() {
                self.prepare_text_selection(&mut placed, tcx);
            }
            if selection_scrolled {
                let target = self.interaction.input.hit_test(
                    self.interaction.input.pointer_position.0,
                    self.interaction.input.pointer_position.1,
                );
                self.extend_text_selection(
                    target,
                    self.interaction.input.pointer_position.0,
                    self.interaction.input.pointer_position.1,
                );
                self.prepare_text_selection(&mut placed, tcx);
            }
            if self.interaction.input.pointer_buttons & 1 == 0 {
                self.sync_text_selection_change();
            }
            if self.frame.projections.semantics_enabled && projection_dirty {
                self.rebuild_semantic_snapshot(&placed);
            }
            self.frame.projections.semantics_dirty = false;
            // After paint applied pending edits, sync widget values → JS.
            self.flush_value_sync();
            #[cfg(any(feature = "devtools", test))]
            if projection_dirty || self.frame.projections.debug_dirty {
                self.publish_debug_snapshot(&placed, tcx);
                self.frame.projections.debug_dirty = false;
            }
            self.document
                .invalidation
                .remove(InvalidationFlags::GEOMETRY);
        }
        placed
    }

    fn base_color(&self) -> Color {
        self.document.base_color
    }

    fn set_semantics_enabled(&mut self, enabled: bool) {
        self.frame.projections.set_semantics_enabled(enabled);
    }

    fn semantic_snapshot(&self) -> Option<Arc<SemanticSnapshot>> {
        self.frame.projections.semantic_snapshot()
    }

    fn handle_semantic_action(&mut self, action: SemanticAction) -> bool {
        let target = match action {
            SemanticAction::Click { target }
            | SemanticAction::Focus { target }
            | SemanticAction::Blur { target }
            | SemanticAction::ScrollIntoView { target } => {
                let key = NodeKey::from_ffi(target);
                key.is_valid().then_some(key)
            }
        };
        let Some(target) =
            target.filter(|target| self.document.node_store.solid_to_node.contains_key(target))
        else {
            return false;
        };
        if !self
            .frame
            .projections
            .semantic_snapshot
            .nodes
            .iter()
            .any(|node| node.id == u64::from(target))
        {
            return false;
        }
        if let Some(modal) = self.frame.projections.semantic_snapshot.modal_root {
            let Some(modal_node) = self
                .document
                .node_store
                .solid_to_node
                .get(&NodeKey::from_ffi(modal))
                .copied()
            else {
                return false;
            };
            if !self
                .document
                .node_store
                .solid_to_node
                .get(&target)
                .is_some_and(|node| {
                    self.document
                        .node_store
                        .is_logical_descendant(*node, modal_node)
                })
            {
                return false;
            }
        }
        match action {
            SemanticAction::Click { .. } => {
                self.dispatch_pointer(target, event::CLICK, None, Modifiers::empty())
            }
            SemanticAction::Focus { .. } => {
                let changed = self.interaction.input.focused_target != Some(target);
                self.interaction.use_keyboard_modality();
                self.set_focused_target(Some(target));
                changed
            }
            SemanticAction::Blur { .. } => {
                let changed = self.interaction.input.focused_target == Some(target);
                if changed {
                    self.set_focused_target(None);
                }
                changed
            }
            SemanticAction::ScrollIntoView { .. } => self.scroll_into_view(target),
        }
    }

    fn ime_cursor_area(&self) -> Option<[f64; 4]> {
        self.interaction.ime_cursor_area
    }

    #[cfg(any(feature = "devtools", test))]
    fn paint_debug_overlay(
        &mut self,
        scene: &mut Scene,
        placed: &[PlacedNode],
        tcx: &mut TextContext,
        device_scale: f64,
    ) {
        let Some(state) = &self.frame.projections.debug_state else {
            return;
        };
        let Ok(guard) = state.read() else { return };
        let overlay = guard.overlay();
        drop(guard);
        let mut paint_stats = wabou_host_api::DebugOverlayPaintStats {
            enabled: overlay.is_enabled(),
            ..Default::default()
        };
        if !overlay.is_enabled() {
            if let Ok(mut state) = state.write() {
                state.record_overlay_paint(paint_stats);
            }
            tracing::trace!(target: "wabou::devtools", "painted disabled debug overlay pass");
            return;
        }
        let device = Affine::scale(device_scale);
        let hovered = self.interaction.input.hovered_target;
        let mut clips = HashSet::new();

        for node in placed {
            let Some(&solid_id) = self.document.node_store.node_to_solid.get(&node.node_id) else {
                continue;
            };
            let [x0, y0, x1, y1] = node.rect;
            let rect = Rect::new(x0 as f64, y0 as f64, x1 as f64, y1 as f64);
            if overlay.layout {
                // A dark halo keeps the diagnostic visible over both light and
                // dark application surfaces. The cyan core remains the stable
                // meaning of a layout border box.
                scene.stroke(
                    &Stroke::new(3.0),
                    device,
                    Color::from_rgba8(15, 23, 42, 210),
                    None,
                    &rect,
                );
                scene.stroke(
                    &Stroke::new(1.25),
                    device,
                    Color::from_rgba8(56, 189, 248, 255),
                    None,
                    &rect,
                );
                paint_stats.layout_bounds = paint_stats.layout_bounds.saturating_add(1);
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
                paint_stats.clip_bounds = paint_stats.clip_bounds.saturating_add(1);
            }

            let is_hit = overlay.hit_target && hovered == Some(solid_id);
            let is_selected = overlay.selected_node == Some(solid_id);
            if !is_hit && !is_selected {
                continue;
            }
            paint_stats.highlights = paint_stats.highlights.saturating_add(1);
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

            // A selected border box alone is ambiguous for the failures this
            // overlay is intended to diagnose: padding, native-widget clips,
            // and text origins all use the content box. Paint that box as a
            // separate layer so the overlay describes the geometry consumed
            // by descendants and widgets, rather than merely proving that a
            // node exists.
            let [content_x, content_y] = node.content_origin;
            let [content_width, content_height] = node.content_size;
            let content_rect = Rect::new(
                content_x as f64,
                content_y as f64,
                (content_x + content_width) as f64,
                (content_y + content_height) as f64,
            );
            scene.fill(
                Fill::NonZero,
                device,
                Color::from_rgba8(14, 165, 233, 38),
                None,
                &content_rect,
            );
            scene.stroke(
                &Stroke::new(1.25),
                device,
                Color::from_rgba8(34, 211, 238, 255),
                None,
                &content_rect,
            );

            let [border_top, border_right, border_bottom, border_left] = node.border_widths;
            if node.border_widths.iter().any(|width| *width > 0.0) {
                let border_inner = Rect::new(
                    (x0 + border_left) as f64,
                    (y0 + border_top) as f64,
                    (x1 - border_right).max(x0 + border_left) as f64,
                    (y1 - border_bottom).max(y0 + border_top) as f64,
                );
                scene.stroke(
                    &Stroke::new(1.0),
                    device,
                    Color::from_rgba8(251, 191, 36, 255),
                    None,
                    &border_inner,
                );
            }

            let atoms = self.document.atoms.borrow();
            let tag = self
                .document
                .node_store
                .declared
                .get(&node.node_id)
                .and_then(|declared| declared.tag)
                .and_then(|tag| atoms.resolve(tag))
                .unwrap_or("#text");
            let border_width = (x1 - x0).max(0.0);
            let border_height = (y1 - y0).max(0.0);
            let label: Arc<str> = format!(
                "{tag}#{solid_id}  {border_width:.0} x {border_height:.0} | content {content_width:.0} x {content_height:.0}"
            )
            .into();
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
        if let Ok(mut state) = state.write() {
            state.record_overlay_paint(paint_stats);
        }
        tracing::trace!(
            target: "wabou::devtools",
            layout_bounds = paint_stats.layout_bounds,
            clip_bounds = paint_stats.clip_bounds,
            highlights = paint_stats.highlights,
            "painted debug overlay"
        );
    }

    #[cfg(not(any(feature = "devtools", test)))]
    fn paint_debug_overlay(
        &mut self,
        _scene: &mut Scene,
        _placed: &[PlacedNode],
        _tcx: &mut TextContext,
        _device_scale: f64,
    ) {
    }

    fn push_frame_stats(&mut self, stats: &FrameStats) {
        if let Some(cell) = &self.runtime.frame_stats {
            // The app fills build_frame/scene/present/node_count; fold in the
            // QuickJS tick EMA + last viewport the applier measured.
            let mut s = *stats;
            s.js_tick_ms = self.frame.js_tick_ema;
            s.viewport_w = self.frame.last_viewport.0;
            s.viewport_h = self.frame.last_viewport.1;
            *cell.borrow_mut() = Some(s);
        }
    }

    fn pointer_cursor(&self) -> wabou_shell::style::CursorStyle {
        self.interaction
            .input
            .hovered_target
            .and_then(|solid| self.document.node_store.solid_to_node.get(&solid))
            .and_then(|node| self.document.node_store.tree.get_node_context(*node))
            .map_or(wabou_shell::style::CursorStyle::Default, |paint| {
                paint.cursor
            })
    }

    fn has_anim(&self) -> bool {
        self.runtime.has_raf
            || self.runtime.reload.is_pending()
            || self.runtime.host_message_inbox.has_pending()
            || self.runtime.js.has_async_wake()
            || self.document.invalidation.contains(InvalidationFlags::TICK)
    }

    fn animation_deadline(&self) -> Option<Instant> {
        let now = Instant::now();
        let scrollbar_deadline =
            self.interaction
                .scroll
                .activity
                .iter()
                .filter_map(|(node, started)| {
                    if self
                        .interaction
                        .scroll
                        .drag
                        .is_some_and(|drag| drag.node == *node)
                        || self
                            .interaction
                            .scroll
                            .hovered
                            .is_some_and(|(owner, _)| owner == *node)
                    {
                        return None;
                    }
                    let style = self
                        .document
                        .node_store
                        .tree
                        .get_node_context(*node)?
                        .scrollbar;
                    if style.visibility != ScrollbarVisibility::Auto {
                        return None;
                    }
                    let fade_start = *started + style.hide_delay;
                    if now < fade_start {
                        Some(fade_start)
                    } else if style.fade_duration.is_zero()
                        || now >= fade_start + style.fade_duration
                    {
                        None
                    } else {
                        Some(now + Duration::from_millis(16))
                    }
                });
        self.document
            .widget_manager
            .widgets
            .values()
            .filter_map(|widget| widget.animation_deadline())
            .chain(self.interaction.text_selection.next_scroll)
            .chain(scrollbar_deadline)
            .min()
    }

    fn set_wake_callback(&mut self, wake: WakeCallback) {
        #[cfg(any(feature = "devtools", test))]
        if let Some(state) = &self.frame.projections.debug_state
            && let Ok(mut state) = state.write()
        {
            state.set_wake(wake.clone());
        }
        self.runtime.js.set_wake_callback(wake.clone());
        for widget in self.document.widget_manager.widgets.values_mut() {
            widget.set_wake_callback(wake.clone());
        }
        self.runtime.host_message_inbox.set_wake(wake.clone());
        self.runtime.effect_bridge.set_wake_callback(wake.clone());
        self.runtime.wake_callback = Some(wake);
    }

    fn poll_async(&mut self) -> bool {
        let was_woken = self.runtime.js.take_async_wake();
        let js_progressed = self.runtime.js.poll_async_runtime();
        // Host messages are application events, not render events. Drain them
        // on the event-loop wake path as well as at the next frame boundary so
        // tray/background applications keep responding while their native
        // surface is hidden or has been released.
        let host_messages_pending = self.runtime.host_message_inbox.has_pending();
        if host_messages_pending {
            self.drain_host_messages();
        }
        let mut widget_woken = false;
        let mut host_actions = Vec::new();
        let mut node_events = Vec::new();
        for (node, widget) in &mut self.document.widget_manager.widgets {
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
            let Some(target) = self.document.node_store.solid_id_for_node(node) else {
                continue;
            };
            widget_woken |= self.dispatch_json(target, event.event_code, &event.json);
        }
        #[cfg(any(feature = "devtools", test))]
        let screenshot_pending = self
            .frame
            .projections
            .debug_state
            .as_ref()
            .and_then(|state| state.read().ok())
            .is_some_and(|state| state.has_screenshot_request());
        #[cfg(not(any(feature = "devtools", test)))]
        let screenshot_pending = false;
        #[cfg(any(feature = "devtools", test))]
        let overlay_changed = self
            .frame
            .projections
            .debug_state
            .as_ref()
            .and_then(|state| state.write().ok())
            .is_some_and(|mut state| state.take_overlay_change());
        #[cfg(not(any(feature = "devtools", test)))]
        let overlay_changed = false;
        widget_woken
            || was_woken
            || js_progressed
            || host_messages_pending
            || screenshot_pending
            || overlay_changed
    }

    fn take_host_action(&mut self) -> Option<wabou_shell::HostAction> {
        self.runtime.pending_host_actions.borrow_mut().pop_front()
    }

    fn complete_host_action(&mut self, result: wabou_shell::HostActionResult) {
        match result {
            wabou_shell::HostActionResult::Clipboard { request_id, text } => {
                let Some((node, widget_request_id)) = self
                    .document
                    .widget_manager
                    .host_action_routes
                    .remove(&request_id)
                else {
                    return;
                };
                if let Some(widget) = self.document.widget_manager.widgets.get_mut(&node) {
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
        self.runtime.effect_bridge.take(&self.runtime.js)
    }

    fn complete_effect(&mut self, completion: wabou_shell::EffectCompletion) {
        self.runtime
            .effect_bridge
            .complete(&self.runtime.js, completion);
    }

    #[cfg(any(feature = "devtools", test))]
    fn take_screenshot_request(&mut self) -> Option<wabou_shell::ScreenshotRequest> {
        let (path, file) = self
            .frame
            .projections
            .debug_state
            .as_ref()?
            .write()
            .ok()?
            .take_screenshot_request()?;
        Some(wabou_shell::ScreenshotRequest { path, file })
    }

    #[cfg(not(any(feature = "devtools", test)))]
    fn take_screenshot_request(&mut self) -> Option<wabou_shell::ScreenshotRequest> {
        None
    }

    #[cfg(any(feature = "devtools", test))]
    fn complete_screenshot(
        &mut self,
        requested_path: &std::path::Path,
        result: Result<std::path::PathBuf, String>,
    ) {
        if let Some(state) = &self.frame.projections.debug_state
            && let Ok(mut state) = state.write()
            && let Err(error) = state.complete_screenshot(requested_path, result)
        {
            tracing::warn!(target: "wabou::devtools", %error);
        }
    }

    #[cfg(not(any(feature = "devtools", test)))]
    fn complete_screenshot(
        &mut self,
        _requested_path: &std::path::Path,
        _result: Result<std::path::PathBuf, String>,
    ) {
    }

    fn handle_event(&mut self, input: UiEvent) -> EventResponse {
        #[cfg(any(feature = "devtools", test))]
        {
            self.frame.projections.debug_dirty |= self.frame.projections.debug_state.is_some();
        }
        if let UiEvent::WindowMetrics(metrics) = &input {
            return self.handle_window_metrics(*metrics);
        }
        if let UiEvent::FileDrop(event) = input {
            return self.handle_file_drop(event);
        }
        if matches!(
            &input,
            UiEvent::Key(_) | UiEvent::TextInput(_) | UiEvent::Ime(_) | UiEvent::Paste(_)
        ) {
            return self.handle_focused_input(input);
        }
        if matches!(&input, UiEvent::Wheel(_))
            || matches!(
                &input,
                UiEvent::Pointer(pointer)
                    if pointer.phase == PointerPhase::Down
                        && pointer.button != Some(PointerButton::Primary)
            )
        {
            self.interaction.text_selection.last_click = None;
        }

        match input {
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Move => {
                self.handle_pointer_move(pointer)
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Down => {
                self.handle_pointer_down(pointer)
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Up => {
                self.handle_pointer_up(pointer)
            }
            UiEvent::Pointer(pointer) if pointer.phase == PointerPhase::Cancel => {
                self.handle_pointer_cancel(pointer)
            }
            UiEvent::Wheel(wheel) => self.handle_wheel_event(wheel),
            UiEvent::Focus(focused) => self.handle_window_focus(focused),
            UiEvent::Pointer(_)
            | UiEvent::Key(_)
            | UiEvent::TextInput(_)
            | UiEvent::Ime(_)
            | UiEvent::Paste(_)
            | UiEvent::FileDrop(_)
            | UiEvent::WindowMetrics(_) => EventResponse::IGNORED,
        }
    }
}
