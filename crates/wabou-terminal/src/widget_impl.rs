use super::*;

/// Factory suitable for `HostBuilder::widget("terminal", terminal_widget)`.
pub fn terminal_widget() -> Box<dyn Widget> {
    Box::new(TerminalWidget::lazy_default_shell())
}

struct RowPaintContext<'a> {
    scene: &'a mut Scene,
    text: &'a mut TextContext,
    colors: &'a TermColors,
    default_background: Color,
    device_scale: f64,
}

fn cell_has_no_glyph(square: Square, character: char) -> bool {
    matches!(square.wide(), Wide::Spacer | Wide::LeadingSpacer)
        || character == '\0'
        || (character == ' ' && !square.has_extras())
        || character.is_control()
}

fn cell_font_weight(style: Style) -> f32 {
    if style.flags.contains(StyleFlags::BOLD) {
        700.0
    } else {
        400.0
    }
}

fn terminal_scene(width: f32, height: f32, background: Color) -> Scene {
    let mut scene = Scene::new();
    scene.fill(
        Fill::NonZero,
        Affine::IDENTITY,
        background,
        None,
        &Rect::new(0.0, 0.0, width as f64, height as f64),
    );
    scene
}

impl TerminalWidget {
    fn cursor_blinking(&self, terminal_blinking: bool) -> bool {
        self.cursor_blink.unwrap_or(terminal_blinking)
    }

    pub(super) fn schedule_cursor_blink(&mut self, terminal_blinking: bool) {
        let blinking =
            self.focused && !self.exit_reported && self.cursor_blinking(terminal_blinking);
        self.cursor_on = true;
        self.next_cursor_blink = blinking.then(|| Instant::now() + Duration::from_millis(500));
    }

    fn draw_visible_rows(
        &self,
        context: &mut RowPaintContext<'_>,
        selection: Option<SelectionRange>,
        display_offset: usize,
    ) {
        for (row_index, row) in self.visible_rows.iter().enumerate() {
            let y = row_index as f32 * self.line_height;
            for column in 0..self.size.columns.min(row.inner.len()) {
                let square = row[Column(column)];
                let point = Pos::new(
                    Line(row_index as i32 - display_offset as i32),
                    Column(column),
                );
                let selected = selection
                    .is_some_and(|selection| selection_contains_square(selection, point, square));
                let (character, style) = match square.content_tag() {
                    ContentTag::Codepoint => (
                        square.c(),
                        self.visible_styles
                            .get(square.style_id() as usize)
                            .copied()
                            .unwrap_or_default(),
                    ),
                    ContentTag::BgPalette => {
                        let background =
                            terminal_indexed_color(square.bg_palette_index(), context.colors);
                        self.draw_background_cell(
                            context.scene,
                            column,
                            row_index,
                            background,
                            selected,
                            context.device_scale,
                        );
                        continue;
                    }
                    ContentTag::BgRgb => {
                        let (red, green, blue) = square.bg_rgb();
                        self.draw_background_cell(
                            context.scene,
                            column,
                            row_index,
                            Color::from_rgb8(red, green, blue),
                            selected,
                            context.device_scale,
                        );
                        continue;
                    }
                };
                self.draw_codepoint_cell(
                    context.scene,
                    context.text,
                    square,
                    character,
                    style,
                    column,
                    row_index,
                    y,
                    selected,
                    context.colors,
                    context.default_background,
                    context.device_scale,
                );
            }
        }
    }

    fn draw_background_cell(
        &self,
        scene: &mut Scene,
        column: usize,
        row: usize,
        background: Color,
        selected: bool,
        device_scale: f64,
    ) {
        fill_cell(
            scene,
            column,
            row,
            self.cell_width,
            self.line_height,
            device_scale,
            background,
        );
        if selected {
            fill_cell(
                scene,
                column,
                row,
                self.cell_width,
                self.line_height,
                device_scale,
                self.selection_background,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_codepoint_cell(
        &self,
        scene: &mut Scene,
        tcx: &mut TextContext,
        square: Square,
        character: char,
        style: Style,
        column: usize,
        row: usize,
        y: f32,
        selected: bool,
        colors: &TermColors,
        default_background: Color,
        device_scale: f64,
    ) {
        let mut foreground = terminal_ansi_color(
            style.fg,
            true,
            colors,
            self.theme_foreground,
            self.theme_background,
        );
        let mut background = terminal_ansi_color(
            style.bg,
            false,
            colors,
            self.theme_foreground,
            self.theme_background,
        );
        if style.flags.contains(StyleFlags::INVERSE) {
            std::mem::swap(&mut foreground, &mut background);
        }
        if background != default_background {
            fill_cell(
                scene,
                column,
                row,
                self.cell_width,
                self.line_height,
                device_scale,
                background,
            );
        }
        if selected {
            fill_cell(
                scene,
                column,
                row,
                self.cell_width,
                self.line_height,
                device_scale,
                self.selection_background,
            );
        }
        if style.flags.contains(StyleFlags::HIDDEN) {
            return;
        }
        if style.flags.contains(StyleFlags::DIM) {
            foreground = dim(foreground);
        }
        if selected && let Some(selection_foreground) = self.selection_foreground {
            foreground = selection_foreground;
        }
        draw_cell_decorations(
            scene,
            column,
            y,
            self.cell_width,
            self.line_height,
            style,
            foreground,
            colors,
            self.theme_foreground,
            self.theme_background,
            selected.then_some(self.selection_foreground).flatten(),
        );
        if cell_has_no_glyph(square, character) {
            return;
        }

        if box_drawing::draw_box_drawing(
            scene,
            character,
            box_drawing::BoxCell {
                column,
                row,
                width: self.cell_width,
                height: self.line_height,
                device_scale,
            },
            foreground,
        ) {
            return;
        }

        let cell_text = cell_text(
            square,
            square
                .extras_id()
                .and_then(|extras_id| self.visible_extras.get(&extras_id)),
        );
        let font_weight = cell_font_weight(style);
        let layout = layout_text_styled(
            tcx,
            Arc::from(cell_text),
            self.font_size,
            font_weight,
            false,
            None,
            Default::default(),
            foreground.to_rgba8().to_u8_array(),
            Arc::from([]),
            Some(&self.font_family),
            None,
        );
        let glyph_scene = tcx.glyph_scene_scaled(&layout, device_scale);
        let x = column as f64 * self.cell_width as f64;
        let text_y = y as f64 + ((self.line_height - layout.height()) * 0.5).max(0.0) as f64;
        let italic = if style.flags.contains(StyleFlags::ITALIC) {
            Affine::skew(-0.18, 0.0)
        } else {
            Affine::IDENTITY
        };
        scene.append_scene(
            (*glyph_scene).clone(),
            Affine::translate((x, text_y)) * italic * Affine::scale(device_scale.recip()),
        );
    }

    fn update_cursor_blink(&mut self) {
        if self.focused
            && self
                .next_cursor_blink
                .is_some_and(|time| Instant::now() >= time)
        {
            self.cursor_on = !self.cursor_on;
            self.next_cursor_blink = Some(Instant::now() + Duration::from_millis(500));
        }
    }

    fn draw_cursor(
        &self,
        scene: &mut Scene,
        cursor: &CursorState,
        display_offset: usize,
        colors: &TermColors,
    ) {
        if display_offset != 0 || !cursor.is_visible() || cursor.pos.row < 0 {
            return;
        }
        let x = cursor.pos.col.0 as f64 * self.cell_width as f64;
        let y = cursor.pos.row.0 as f64 * self.line_height as f64;
        let Some(visual) = cursor_visual(
            self.focused,
            self.cursor_on,
            cursor.content,
            x,
            y,
            self.cell_width as f64,
            self.line_height as f64,
        ) else {
            return;
        };
        let color = terminal_ansi_color(
            AnsiColor::Named(NamedColor::Cursor),
            true,
            colors,
            self.theme_foreground,
            self.theme_background,
        );
        match visual {
            CursorVisual::Filled(rect) => scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                color.with_alpha(0.43),
                None,
                &rect,
            ),
            CursorVisual::Hollow(rect) => {
                scene.stroke(&Stroke::new(1.0), Affine::IDENTITY, color, None, &rect);
            }
        }
    }

    fn draw_spawn_error(
        &self,
        scene: &mut Scene,
        text: &mut TextContext,
        width: f32,
        device_scale: f64,
    ) {
        let Some(error) = &self.spawn_error else {
            return;
        };
        let layout = layout_text_styled(
            text,
            Arc::from(format!("terminal: {error}")),
            13.0,
            400.0,
            false,
            None,
            Default::default(),
            [248, 113, 113, 255],
            Arc::from([]),
            Some(&self.font_family),
            Some(width),
        );
        scene.append_scene(
            (*text.glyph_scene_scaled(&layout, device_scale)).clone(),
            Affine::translate((4.0, 4.0)) * Affine::scale(device_scale.recip()),
        );
    }

    fn handle_pointer_event(&mut self, pointer: &wabou_shell::PointerEvent) -> WidgetEventResult {
        if pointer.phase == PointerPhase::Down && pointer.button != Some(PointerButton::Primary) {
            self.last_click = None;
        }
        if pointer.phase == PointerPhase::Down
            && pointer.button == Some(PointerButton::Primary)
            && terminal_primary_shortcut(pointer.modifiers)
            && let Some(url) = self.hyperlink_at(pointer.position.x, pointer.position.y)
        {
            self.last_click = None;
            self.pending_hyperlink = Some(PendingHyperlink {
                url,
                origin: (pointer.position.x, pointer.position.y),
                cancelled: false,
            });
            return WidgetEventResult::HANDLED;
        }
        if let Some(pending) = self.pending_hyperlink.as_mut()
            && pointer.phase == PointerPhase::Move
        {
            let distance = (pointer.position.x - pending.origin.0)
                .hypot(pointer.position.y - pending.origin.1);
            pending.cancelled |= distance > SELECTION_DRAG_THRESHOLD;
            return WidgetEventResult::HANDLED;
        }
        if pointer.phase == PointerPhase::Up
            && let Some(pending) = self.pending_hyperlink.take()
        {
            if !pending.cancelled
                && self
                    .hyperlink_at(pointer.position.x, pointer.position.y)
                    .as_deref()
                    == Some(pending.url.as_str())
            {
                self.pending_host_actions
                    .push_back(HostAction::OpenUrl(pending.url));
            }
            return WidgetEventResult::HANDLED;
        }
        if pointer.phase == PointerPhase::Cancel && self.pending_hyperlink.take().is_some() {
            return WidgetEventResult::HANDLED;
        }
        if !self.selecting && self.report_pointer(pointer) {
            return WidgetEventResult::HANDLED;
        }
        match (pointer.phase, pointer.button, self.selecting) {
            (PointerPhase::Down, Some(PointerButton::Primary), _) => {
                self.begin_or_extend_selection(
                    pointer.position.x,
                    pointer.position.y,
                    pointer.modifiers,
                );
                WidgetEventResult::HANDLED
            }
            (PointerPhase::Move, _, true) => {
                self.update_selection(pointer.position.x, pointer.position.y);
                WidgetEventResult::HANDLED
            }
            (PointerPhase::Up, _, true) => {
                self.update_selection(pointer.position.x, pointer.position.y);
                self.finish_selection_gesture();
                WidgetEventResult::HANDLED
            }
            (PointerPhase::Cancel, _, true) => {
                self.last_click = None;
                self.finish_selection_gesture();
                WidgetEventResult::HANDLED
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn handle_key_event(&mut self, key: &wabou_shell::KeyEvent) -> WidgetEventResult {
        if key.phase == KeyPhase::Down {
            self.last_click = None;
        }
        if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("a") {
            if key.phase == KeyPhase::Down {
                self.select_all();
            }
            return WidgetEventResult::HANDLED;
        }
        if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("c") {
            return if key.phase == KeyPhase::Down {
                self.selected_text()
                    .map_or(WidgetEventResult::HANDLED, WidgetEventResult::copy)
            } else {
                WidgetEventResult::HANDLED
            };
        }
        if terminal_clipboard_shortcut(key.modifiers) && key.key.eq_ignore_ascii_case("v") {
            return if self.exit_reported {
                WidgetEventResult::HANDLED
            } else if key.phase == KeyPhase::Down {
                WidgetEventResult::paste()
            } else {
                WidgetEventResult::HANDLED
            };
        }
        let mode = self.terminal.lock().mode();
        if key.phase == KeyPhase::Down
            && key.modifiers == Modifiers::SHIFT
            && !mode.contains(Mode::ALT_SCREEN)
            && let Some(scroll) = scrollback_key(&key.key)
        {
            self.terminal.lock().scroll_display(scroll);
            return WidgetEventResult::HANDLED;
        }
        if self.exit_reported {
            return WidgetEventResult::HANDLED;
        }
        let bytes = self.key_bytes(key);
        if bytes.is_empty() {
            return WidgetEventResult::IGNORED;
        }
        self.begin_terminal_input();
        self.send_bytes(bytes);
        if key.phase == KeyPhase::Down {
            WidgetEventResult::handled_consuming_key_text()
        } else {
            WidgetEventResult::HANDLED
        }
    }

    fn handle_wheel_event(&mut self, wheel: &wabou_shell::WheelEvent) -> WidgetEventResult {
        self.last_click = None;
        let context = self.wheel_context(wheel);
        let lines = self.wheel_lines.push(context, wheel.delta_y);
        if self.selecting {
            self.scroll_active_selection(wheel, lines);
        } else if !self.report_wheel(wheel, lines)
            && !self.report_alternate_scroll(lines)
            && lines != 0
        {
            self.terminal.lock().scroll_display(Scroll::Delta(lines));
        }
        // Fractional trackpad input remains terminal-owned until it reaches a
        // complete grid line.
        WidgetEventResult::HANDLED
    }
}

fn scrollback_key(key: &str) -> Option<Scroll> {
    match key {
        "Home" => Some(Scroll::Top),
        "End" => Some(Scroll::Bottom),
        "PageUp" => Some(Scroll::PageUp),
        "PageDown" => Some(Scroll::PageDown),
        _ => None,
    }
}

impl Widget for TerminalWidget {
    fn measure(&mut self, cx: &mut wabou_shell::MeasureContext<'_>) -> Option<[f32; 2]> {
        self.update_font_metrics(cx.text());
        self.intrinsic_size()
    }

    fn paint(&mut self, cx: &mut wabou_shell::PaintContext<'_>) {
        let [width, height] = cx.size();
        let device_scale = cx.device_scale();
        let tcx = cx.text();
        self.update_font_metrics(tcx);
        self.resize(width, height, device_scale);
        self.ensure_launched();
        self.tick_selection_autoscroll();
        self.update_cursor_blink();

        let (
            cursor,
            selection,
            display_offset,
            colors,
            atlas_placements,
            kitty_placements,
            history_size,
        ) = {
            let mut terminal = self.terminal.lock();
            let damage = terminal.peek_damage_event().unwrap_or(TerminalDamage::Noop);
            terminal.snapshot_visible(
                &damage,
                self.size.columns,
                &mut self.visible_rows,
                &mut self.visible_styles,
                &mut self.visible_extras,
            );
            // `damage()` advances Rio's remembered cursor position; resetting
            // only the dirty lines would otherwise leave CursorOnly pending.
            {
                let _consumed = terminal.damage();
            }
            terminal.reset_damage();
            (
                terminal.cursor(),
                terminal
                    .selection
                    .as_ref()
                    .and_then(|selection| selection.to_range(&*terminal)),
                terminal.display_offset(),
                terminal.colors,
                terminal.graphics.atlas_placements.clone(),
                terminal
                    .graphics
                    .kitty_placements
                    .values()
                    .cloned()
                    .collect::<Vec<_>>(),
                terminal.lines_evicted() as i64 + terminal.history_size() as i64,
            )
        };
        let default_background = terminal_ansi_color(
            AnsiColor::Named(NamedColor::Background),
            false,
            &colors,
            self.theme_foreground,
            self.theme_background,
        );
        let mut scene = terminal_scene(width, height, default_background);

        let scale = device_scale;
        let viewport = rio_vt::ansi::graphics::OverlayViewport {
            cell_width: (f64::from(self.cell_width) * scale) as f32,
            cell_height: (f64::from(self.line_height) * scale) as f32,
            origin_x: 0.0,
            origin_y: 0.0,
            history_size,
            display_offset: display_offset as i64,
            screen_lines: self.size.rows as i64,
        };
        let clip = [
            0.0,
            0.0,
            self.size.columns as f32 * viewport.cell_width,
            self.size.rows as f32 * viewport.cell_height,
        ];
        self.graphics
            .draw_atlas(&mut scene, &atlas_placements, &viewport, clip, scale);
        self.graphics.draw_kitty(
            &mut scene,
            &kitty_placements,
            KittyLayer::BehindText,
            &viewport,
            clip,
            scale,
        );

        self.draw_visible_rows(
            &mut RowPaintContext {
                scene: &mut scene,
                text: tcx,
                colors: &colors,
                default_background,
                device_scale,
            },
            selection,
            display_offset,
        );

        self.draw_cursor(&mut scene, &cursor, display_offset, &colors);

        self.graphics.draw_kitty(
            &mut scene,
            &kitty_placements,
            KittyLayer::AboveText,
            &viewport,
            clip,
            scale,
        );

        self.draw_spawn_error(&mut scene, tcx, width, device_scale);
        cx.scene_mut().append_scene(scene, Affine::IDENTITY);
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        match event {
            UiEvent::Pointer(pointer) => self.handle_pointer_event(pointer),
            UiEvent::TextInput(text) | UiEvent::Ime(ImeEvent::Commit(text)) => {
                if self.exit_reported {
                    return WidgetEventResult::HANDLED;
                }
                self.begin_terminal_input();
                self.send_bytes(text.as_bytes().to_vec());
                WidgetEventResult::HANDLED
            }
            UiEvent::Paste(text) => {
                if self.exit_reported {
                    return WidgetEventResult::HANDLED;
                }
                let bracketed = self.terminal.lock().mode().contains(Mode::BRACKETED_PASTE);
                self.begin_terminal_input();
                self.send_bytes(encode_paste(text, bracketed));
                WidgetEventResult::HANDLED
            }
            UiEvent::Key(key) => self.handle_key_event(key),
            UiEvent::Wheel(wheel) => self.handle_wheel_event(wheel),
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> WidgetChanges {
        match name {
            "command" if !self.launch_started || self.spawn_error.is_some() => {
                self.launch_started = false;
                self.spawn_error = None;
                let launch = self.launch.get_or_insert_with(LaunchConfig::default_shell);
                launch.login_shell = value.is_empty();
                launch.command = if value.is_empty() {
                    default_shell_command()
                } else {
                    value.to_owned()
                };
            }
            "args" if !self.launch_started || self.spawn_error.is_some() => {
                match serde_json::from_str::<Vec<String>>(value) {
                    Ok(args) => {
                        self.launch_started = false;
                        self.launch
                            .get_or_insert_with(LaunchConfig::default_shell)
                            .args = args;
                        self.launch_error = None;
                        self.spawn_error = None;
                    }
                    Err(error) => {
                        let message = format!("invalid terminal args JSON: {error}");
                        self.launch_error = Some(message.clone());
                        self.spawn_error = Some(message);
                    }
                }
            }
            "cwd" if !self.launch_started || self.spawn_error.is_some() => {
                self.launch_started = false;
                self.spawn_error = None;
                self.launch
                    .get_or_insert_with(LaunchConfig::default_shell)
                    .cwd = (!value.is_empty()).then(|| value.to_owned());
            }
            "command" | "args" | "cwd" => {
                tracing::warn!(
                    attribute = name,
                    "ignored terminal launch option after PTY start"
                );
            }
            "font-size" => {
                if let Ok(size) = value.trim_end_matches("px").parse::<f32>() {
                    self.font_size = size.max(6.0);
                    self.metrics_dirty = true;
                }
            }
            "line-height" => {
                if let Ok(height) = value.trim_end_matches("px").parse::<f32>() {
                    self.explicit_line_height = Some(height.max(0.0));
                    self.metrics_dirty = true;
                }
            }
            "font-family" => {
                self.font_family = Arc::from(value);
                self.metrics_dirty = true;
            }
            "allow-clipboard-read" => {
                self.allow_clipboard_read = matches!(value, "" | "true" | "1");
            }
            "cursor-blink" => {
                self.cursor_blink = Some(matches!(value, "" | "true" | "1"));
                let terminal_blinking = self.terminal.lock().blinking_cursor;
                self.schedule_cursor_blink(terminal_blinking);
            }
            "sync-window-title" => {
                let enabled = matches!(value, "" | "true" | "1");
                if self.sync_window_title && !enabled {
                    self.pending_host_actions
                        .push_back(HostAction::SetWindowTitle(None));
                }
                self.sync_window_title = enabled;
            }
            "selection-background" => {
                if let Some(color) = wabou_shell::style::parse_color(value) {
                    self.selection_background = color;
                }
            }
            "selection-foreground" => {
                if let Some(color) = wabou_shell::style::parse_color(value) {
                    self.selection_foreground = Some(color);
                }
            }
            "inherit-theme" => {
                self.inherit_theme = matches!(value, "" | "true" | "1");
                if !self.inherit_theme {
                    self.theme_foreground = named_color(NamedColor::Foreground, true);
                    self.theme_background = named_color(NamedColor::Background, false);
                }
            }
            _ => {}
        }
        match name {
            "font-size" | "line-height" | "font-family" => {
                WidgetChanges::MEASURE | WidgetChanges::REDRAW
            }
            "command"
            | "args"
            | "cwd"
            | "selection-background"
            | "selection-foreground"
            | "inherit-theme"
            | "cursor-blink" => WidgetChanges::REDRAW,
            _ => WidgetChanges::empty(),
        }
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn accepts_text_input(&self) -> bool {
        true
    }

    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        wabou_shell::WidgetAccessibility {
            role: Some(wabou_shell::SemanticRole::TextInput),
            ..Default::default()
        }
    }

    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        if !self.focused {
            return None;
        }
        let terminal = self.terminal.lock();
        if terminal.display_offset() != 0 {
            return None;
        }
        let cursor = terminal.cursor();
        if cursor.pos.row < 0 {
            return None;
        }
        let x = cursor.pos.col.0 as f32 * self.cell_width;
        let y = cursor.pos.row.0 as f32 * self.line_height;
        Some([
            x,
            y,
            x + self.cell_width.max(1.0),
            y + self.line_height.max(1.0),
        ])
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> WidgetChanges {
        if !self.inherit_theme {
            return WidgetChanges::empty();
        }
        self.theme_foreground = style.color;
        self.theme_background = style
            .background
            .unwrap_or_else(|| named_color(NamedColor::Background, false));
        WidgetChanges::REDRAW
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([
            DEFAULT_COLUMNS as f32 * self.cell_width,
            DEFAULT_ROWS as f32 * self.line_height,
        ])
    }

    fn focus_changed(&mut self, focused: bool) -> WidgetChanges {
        if !focused {
            self.pending_hyperlink = None;
            self.last_click = None;
        }
        if !focused && self.selecting {
            self.finish_selection_gesture();
        }
        self.focused = focused;
        self.cursor_on = true;
        let terminal = self.terminal.lock();
        let terminal_blinking = terminal.blinking_cursor;
        let report_focus = terminal.mode().contains(Mode::FOCUS_IN_OUT);
        drop(terminal);
        self.schedule_cursor_blink(terminal_blinking);
        if report_focus && !self.exit_reported {
            self.send_bytes(if focused {
                b"\x1b[I".to_vec()
            } else {
                b"\x1b[O".to_vec()
            });
        }
        WidgetChanges::REDRAW
    }

    fn animation_deadline(&self) -> Option<Instant> {
        [self.next_cursor_blink, self.next_selection_scroll]
            .into_iter()
            .flatten()
            .min()
    }

    fn set_wake_callback(&mut self, wake: WakeCallback) {
        self.listener.set_wake(wake);
    }

    fn poll_async(&mut self) -> bool {
        self.handle_rio_events()
    }

    fn take_host_action(&mut self) -> Option<HostAction> {
        self.pending_host_actions.pop_front()
    }

    fn take_node_event(&mut self) -> Option<WidgetNodeEvent> {
        self.pending_node_events.pop_front()
    }

    fn complete_host_action(&mut self, result: HostActionResult) {
        match result {
            HostActionResult::Clipboard { request_id, text } => {
                if let Some(formatter) = self.pending_clipboard_loads.remove(&request_id) {
                    self.send_bytes(formatter(text.as_deref().unwrap_or("")).into_bytes());
                }
            }
            HostActionResult::ClipboardWrite { .. } => {}
        }
    }

    fn attribute_removed(&mut self, name: &str) -> WidgetChanges {
        match name {
            "command" if !self.launch_started || self.spawn_error.is_some() => {
                let _ = self.attribute_changed("command", "");
            }
            "args" if !self.launch_started || self.spawn_error.is_some() => {
                let _ = self.attribute_changed("args", "[]");
            }
            "cwd" if !self.launch_started || self.spawn_error.is_some() => {
                let _ = self.attribute_changed("cwd", "");
            }
            "command" | "args" | "cwd" => {
                tracing::warn!(
                    attribute = name,
                    "ignored terminal launch option removal after PTY start"
                );
            }
            "allow-clipboard-read" => self.allow_clipboard_read = false,
            "cursor-blink" => {
                self.cursor_blink = None;
                let terminal_blinking = self.terminal.lock().blinking_cursor;
                self.schedule_cursor_blink(terminal_blinking);
            }
            "sync-window-title" => {
                if self.sync_window_title {
                    self.pending_host_actions
                        .push_back(HostAction::SetWindowTitle(None));
                }
                self.sync_window_title = false;
            }
            "selection-background" => {
                self.selection_background = DEFAULT_SELECTION_BACKGROUND;
            }
            "selection-foreground" => self.selection_foreground = None,
            "inherit-theme" => {
                self.inherit_theme = false;
                self.theme_foreground = named_color(NamedColor::Foreground, true);
                self.theme_background = named_color(NamedColor::Background, false);
            }
            "font-size" => {
                self.font_size = DEFAULT_FONT_SIZE;
                self.metrics_dirty = true;
            }
            "line-height" => {
                self.explicit_line_height = None;
                self.metrics_dirty = true;
            }
            "font-family" => {
                self.font_family = Arc::from("monospace");
                self.metrics_dirty = true;
            }
            _ => {}
        }
        match name {
            "font-size" | "line-height" | "font-family" => {
                WidgetChanges::MEASURE | WidgetChanges::REDRAW
            }
            "command"
            | "args"
            | "cwd"
            | "selection-background"
            | "selection-foreground"
            | "inherit-theme"
            | "cursor-blink" => WidgetChanges::REDRAW,
            _ => WidgetChanges::empty(),
        }
    }

    fn unmount(&mut self) {
        self.shutdown_pty();
        if self.sync_window_title {
            self.sync_window_title = false;
            self.pending_host_actions
                .push_back(HostAction::SetWindowTitle(None));
        }
    }
}

#[cfg(test)]
impl TerminalWidget {
    pub(crate) fn paint(&mut self, width: f32, height: f32, text: &mut TextContext) -> Scene {
        self.paint_scaled(width, height, 1.0, text)
    }

    pub(crate) fn paint_scaled(
        &mut self,
        width: f32,
        height: f32,
        device_scale: f64,
        text: &mut TextContext,
    ) -> Scene {
        let mut cx = wabou_shell::PaintContext::new(width, height, device_scale, text);
        <Self as Widget>::paint(self, &mut cx);
        cx.finish()
    }
}
