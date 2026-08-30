use super::*;
use crate::session::{TerminalInputResult, TerminalInvalidation};
use wabou_runtime::{Widget, WidgetChanges, WidgetEventResult, WidgetNodeEvent, WidgetStyle};
use wabou_shell::text::{TextContext, layout_text_styled};

pub(crate) fn legacy_color(color: TerminalColor) -> Color {
    let [r, g, b, a] = color.components();
    Color::from_rgba8(r, g, b, a)
}

fn terminal_color_from_legacy(color: Color) -> TerminalColor {
    let [r, g, b, a] = color.to_rgba8().to_u8_array();
    TerminalColor::rgba(r, g, b, a)
}

fn legacy_terminal_ansi_color(
    color: AnsiColor,
    foreground: bool,
    colors: &TermColors,
    theme_foreground: TerminalColor,
    theme_background: TerminalColor,
) -> Color {
    legacy_color(color::resolve_ansi_color(
        color,
        foreground,
        colors,
        theme_foreground,
        theme_background,
    ))
}

impl TerminalInputResult {
    fn into_legacy(self) -> WidgetEventResult {
        match self {
            Self::Ignored => WidgetEventResult::IGNORED,
            Self::Handled => WidgetEventResult::HANDLED,
            Self::HandledConsumingText => WidgetEventResult::handled_consuming_key_text(),
            Self::Clipboard(wabou_shell_api::ClipboardRequest::Read) => WidgetEventResult::paste(),
            Self::Clipboard(wabou_shell_api::ClipboardRequest::Write(text)) => {
                WidgetEventResult::copy(text)
            }
        }
    }
}

impl TerminalInvalidation {
    fn into_legacy(self) -> WidgetChanges {
        match (self.measure, self.redraw) {
            (true, true) => WidgetChanges::MEASURE | WidgetChanges::REDRAW,
            (true, false) => WidgetChanges::MEASURE,
            (false, true) => WidgetChanges::REDRAW,
            (false, false) => WidgetChanges::empty(),
        }
    }
}

impl From<TerminalNodeEvent> for WidgetNodeEvent {
    fn from(event: TerminalNodeEvent) -> Self {
        Self {
            event_code: match event.kind {
                TerminalEventKind::Exit => wabou_runtime::event::TERMINALEXIT,
                TerminalEventKind::Progress => wabou_runtime::event::TERMINALPROGRESS,
                TerminalEventKind::Notification => wabou_runtime::event::TERMINALNOTIFICATION,
                TerminalEventKind::TitleChange => wabou_runtime::event::TERMINALTITLECHANGE,
                TerminalEventKind::CurrentDirectoryChange => {
                    wabou_runtime::event::TERMINALCWDCHANGE
                }
                TerminalEventKind::SelectionChange => wabou_runtime::event::TERMINALSELECTIONCHANGE,
                TerminalEventKind::Bell => wabou_runtime::event::TERMINALBELL,
            },
            json: event.json,
        }
    }
}

/// Legacy AnyRender factory suitable for
/// `HostBuilder::widget("terminal", terminal_widget)`.
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
    fn update_legacy_font_metrics(&mut self, tcx: &mut TextContext) {
        if !self.metrics_dirty {
            return;
        }
        let layout = layout_text_styled(
            tcx,
            Arc::from("0"),
            self.font_size,
            400.0,
            false,
            None,
            Default::default(),
            [255, 255, 255, 255],
            Arc::from([]),
            Some(&self.font_family),
            None,
        );
        let line_height = self.explicit_line_height.map_or_else(
            || (layout.height() * 1.1).max(self.font_size),
            |line_height| line_height.max(self.font_size),
        );
        self.set_font_metrics(layout.width(), line_height);
    }

    /// Apply one terminal configuration attribute independently of a shell
    /// widget trait. Native adapters translate the returned invalidation into
    /// their own layout and paint scheduling model.
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
                legacy_color(self.selection_background),
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
        let mut foreground = legacy_terminal_ansi_color(
            style.fg,
            true,
            colors,
            self.theme_foreground,
            self.theme_background,
        );
        let mut background = legacy_terminal_ansi_color(
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
                legacy_color(self.selection_background),
            );
        }
        if style.flags.contains(StyleFlags::HIDDEN) {
            return;
        }
        if style.flags.contains(StyleFlags::DIM) {
            foreground = dim(foreground);
        }
        if selected && let Some(selection_foreground) = self.selection_foreground {
            foreground = legacy_color(selection_foreground);
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
            legacy_color(self.theme_foreground),
            legacy_color(self.theme_background),
            selected
                .then_some(self.selection_foreground)
                .flatten()
                .map(legacy_color),
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
        let color = legacy_terminal_ansi_color(
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
}

impl Widget for TerminalWidget {
    fn measure(&mut self, cx: &mut wabou_shell::MeasureContext<'_>) -> Option<[f32; 2]> {
        self.update_legacy_font_metrics(cx.text());
        self.intrinsic_size()
    }

    fn paint(&mut self, cx: &mut wabou_shell::PaintContext<'_>) {
        let [width, height] = cx.size();
        let device_scale = cx.device_scale();
        let tcx = cx.text();
        self.update_legacy_font_metrics(tcx);
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
        let default_background = legacy_terminal_ansi_color(
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
        self.handle_native_event(event).into_legacy()
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> WidgetChanges {
        self.apply_native_attribute(name, value).into_legacy()
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
        self.theme_foreground = terminal_color_from_legacy(style.color);
        self.theme_background = terminal_color_from_legacy(
            style
                .background
                .unwrap_or_else(|| named_color(NamedColor::Background, false)),
        );
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
        self.install_native_wake(wake);
    }

    fn poll_async(&mut self) -> bool {
        self.poll_native_events()
    }

    fn take_host_action(&mut self) -> Option<HostAction> {
        self.pending_host_actions.pop_front()
    }

    fn take_node_event(&mut self) -> Option<WidgetNodeEvent> {
        self.pending_node_events.pop_front().map(Into::into)
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
                self.theme_foreground = color::terminal_named_color(NamedColor::Foreground, true);
                self.theme_background = color::terminal_named_color(NamedColor::Background, false);
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
