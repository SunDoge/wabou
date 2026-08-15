use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use editor_core::{
    Command, CursorCommand, EditCommand, EditorStateManager, Position, Selection, ViewCommand,
};
use editor_core_highlight_simple::{
    RegexHighlightProcessor, SIMPLE_STYLE_BOOLEAN, SIMPLE_STYLE_NULL, SIMPLE_STYLE_NUMBER,
    SIMPLE_STYLE_STRING, SimpleJsonStyles,
};
use vello::{
    Scene,
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};
use wabou_shell::{
    ImeEvent, KeyPhase, PaintContext, PointerPhase, UiEvent, Widget, WidgetEventResult, WidgetStyle,
};
use wabou_shell::{
    style::TextAlign,
    text::{TextRun, brush_for_color, layout_text_styled},
};

const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 22.0;
const CELL_WIDTH: f32 = 8.4;
const GUTTER_WIDTH: f32 = 58.0;
const TEXT_INSET: f32 = 10.0;

pub struct CodeEditor {
    state: EditorStateManager,
    highlighter: RegexHighlightProcessor,
    cached_value: String,
    scroll_row: usize,
    viewport: [f32; 2],
    window_to_local: [f64; 6],
    focused: bool,
    selecting: bool,
    disabled: bool,
    read_only: bool,
    last_click: Option<(Instant, f32, f32, u8)>,
    composition: Option<(usize, usize)>,
    text_color: Color,
    font_family: Option<Arc<str>>,
}

impl CodeEditor {
    pub fn new() -> Self {
        Self::from_text("")
    }

    fn from_text(text: &str) -> Self {
        let mut editor = Self {
            state: EditorStateManager::new(text, 80),
            highlighter: RegexHighlightProcessor::json_default(SimpleJsonStyles::default())
                .expect("built-in JSON highlighting regexes are valid"),
            cached_value: text.to_owned(),
            scroll_row: 0,
            viewport: [0.0, 0.0],
            window_to_local: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            focused: false,
            selecting: false,
            disabled: false,
            read_only: false,
            last_click: None,
            composition: None,
            text_color: Color::from_rgb8(0xe6, 0xe9, 0xef),
            font_family: Some(Arc::from("monospace")),
        };
        editor.refresh_derived_state();
        editor
    }

    fn execute(&mut self, command: Command) -> bool {
        let document_may_change = matches!(
            &command,
            Command::Edit(edit) if !matches!(edit, EditCommand::EndUndoGroup)
        );
        if self.state.execute(command).is_err() {
            return false;
        }
        if document_may_change {
            self.cached_value = self.state.editor().get_text();
            self.refresh_derived_state();
        }
        self.reveal_cursor();
        true
    }

    fn refresh_derived_state(&mut self) {
        let _ = self.state.apply_processor(&mut self.highlighter);
    }

    fn visible_rows(&self) -> usize {
        (self.viewport[1] / LINE_HEIGHT).floor().max(1.0) as usize
    }

    fn reveal_cursor(&mut self) {
        let cursor = self.state.editor().cursor_position();
        let Some((row, _)) = self
            .state
            .logical_position_to_visual(cursor.line, cursor.column)
        else {
            return;
        };
        let visible = self.visible_rows();
        if row < self.scroll_row {
            self.scroll_row = row;
        } else if row >= self.scroll_row + visible {
            self.scroll_row = row + 1 - visible;
        }
    }

    fn position_from_pointer(&self, x: f32, y: f32) -> Position {
        let row = (self.scroll_row + (y.max(0.0) / LINE_HEIGHT).floor() as usize)
            .min(self.state.total_visual_lines().saturating_sub(1));
        let x_cells = ((x - GUTTER_WIDTH - TEXT_INSET).max(0.0) / CELL_WIDTH).round() as usize;
        self.state
            .visual_position_to_logical(row, x_cells)
            .unwrap_or_else(|| self.state.editor().cursor_position())
    }

    fn local_point(&self, x: f64, y: f64) -> (f32, f32) {
        let [a, b, c, d, e, f] = self.window_to_local;
        ((a * x + c * y + e) as f32, (b * x + d * y + f) as f32)
    }

    fn select_all(&mut self) -> bool {
        let last_line = self.state.editor().line_count().saturating_sub(1);
        let last_column = self
            .state
            .editor()
            .line_index()
            .get_line_text(last_line)
            .map(|line| line.trim_end_matches(['\r', '\n']).chars().count())
            .unwrap_or(0);
        self.execute(Command::Cursor(CursorCommand::SetSelection {
            start: Position::new(0, 0),
            end: Position::new(last_line, last_column),
        }))
    }

    fn cursor_offset(&self) -> usize {
        let cursor = self.state.editor().cursor_position();
        self.state
            .editor()
            .line_index()
            .position_to_char_offset(cursor.line, cursor.column)
    }

    fn update_composition(&mut self, text: &str, cursor: Option<(usize, usize)>) -> bool {
        let (start, previous_len) = self
            .composition
            .unwrap_or_else(|| (self.cursor_offset(), 0));
        let text_len = text.chars().count();
        let (selection_start, selection_end) = cursor
            .map(|(from, to)| {
                let from = floor_char_boundary(text, from.min(text.len()));
                let to = floor_char_boundary(text, to.min(text.len()));
                (
                    start + text[..from].chars().count(),
                    start + text[..to].chars().count(),
                )
            })
            .unwrap_or((start + text_len, start + text_len));
        let changed = self.execute(Command::Edit(
            EditCommand::ReplaceCoalescingUndoWithSelection {
                start,
                length: previous_len,
                text: text.to_owned(),
                selection_start,
                selection_end,
            },
        ));
        self.composition = (!text.is_empty()).then_some((start, text_len));
        changed
    }

    fn commit_composition(&mut self, text: &str) -> bool {
        let changed = if let Some((start, previous_len)) = self.composition.take() {
            self.execute(Command::Edit(EditCommand::ReplaceCoalescingUndo {
                start,
                length: previous_len,
                text: text.to_owned(),
            }))
        } else {
            self.execute(Command::Edit(EditCommand::InsertText {
                text: text.to_owned(),
            }))
        };
        let _ = self.execute(Command::Edit(EditCommand::EndUndoGroup));
        changed
    }

    fn key_down(&mut self, key: &str, shift: bool, primary: bool) -> WidgetEventResult {
        let command = if primary && key.eq_ignore_ascii_case("z") {
            Some(Command::Edit(if shift {
                EditCommand::Redo
            } else {
                EditCommand::Undo
            }))
        } else if primary && key.eq_ignore_ascii_case("a") {
            self.select_all();
            return WidgetEventResult::handled_consuming_key_text();
        } else {
            match key {
                "ArrowLeft" if shift => Some(Command::Cursor(CursorCommand::ExpandSelectionBy {
                    unit: editor_core::ExpandSelectionUnit::Character,
                    direction: editor_core::ExpandSelectionDirection::Backward,
                    count: 1,
                })),
                "ArrowRight" if shift => Some(Command::Cursor(CursorCommand::ExpandSelectionBy {
                    unit: editor_core::ExpandSelectionUnit::Character,
                    direction: editor_core::ExpandSelectionDirection::Forward,
                    count: 1,
                })),
                "ArrowLeft" => Some(Command::Cursor(CursorCommand::MoveGraphemeLeft)),
                "ArrowRight" => Some(Command::Cursor(CursorCommand::MoveGraphemeRight)),
                "ArrowUp" => Some(Command::Cursor(CursorCommand::MoveVisualBy {
                    delta_rows: -1,
                })),
                "ArrowDown" => Some(Command::Cursor(CursorCommand::MoveVisualBy {
                    delta_rows: 1,
                })),
                "Home" => Some(Command::Cursor(CursorCommand::MoveToVisualLineStart)),
                "End" => Some(Command::Cursor(CursorCommand::MoveToVisualLineEnd)),
                "Backspace" => Some(Command::Edit(EditCommand::DeleteGraphemeBack)),
                "Delete" => Some(Command::Edit(EditCommand::DeleteGraphemeForward)),
                _ => None,
            }
        };
        let Some(command) = command else {
            return WidgetEventResult::IGNORED;
        };
        let changes_value = matches!(command, Command::Edit(_));
        if changes_value && self.read_only {
            return WidgetEventResult::IGNORED;
        }
        if !self.execute(command) {
            return WidgetEventResult::IGNORED;
        }
        if changes_value {
            WidgetEventResult::value_changed_consuming_key_text()
        } else {
            WidgetEventResult::handled_consuming_key_text()
        }
    }

    fn selected_offsets(&self) -> Option<(usize, usize)> {
        let Selection { start, end, .. } = self.state.editor().selection()?;
        let (start, end) = if start <= end {
            (*start, *end)
        } else {
            (*end, *start)
        };
        let index = self.state.editor().line_index();
        let from = index.position_to_char_offset(start.line, start.column);
        let to = index.position_to_char_offset(end.line, end.column);
        (to > from).then_some((from, to))
    }

    fn color_for_styles(&self, styles: &[u32]) -> Color {
        if styles.contains(&SIMPLE_STYLE_STRING) {
            Color::from_rgb8(0xa6, 0xe3, 0xa1)
        } else if styles.contains(&SIMPLE_STYLE_NUMBER) {
            Color::from_rgb8(0xfa, 0xb3, 0x87)
        } else if styles.contains(&SIMPLE_STYLE_BOOLEAN) {
            Color::from_rgb8(0xc6, 0x9d, 0xf7)
        } else if styles.contains(&SIMPLE_STYLE_NULL) {
            Color::from_rgb8(0x7f, 0x84, 0x9c)
        } else {
            self.text_color
        }
    }

    fn paint_text_line(
        &self,
        scene: &mut Scene,
        paint: &mut PaintContext<'_>,
        line: &editor_core::HeadlessLine,
        y: f64,
    ) {
        let mut text = String::new();
        let mut style_ranges = Vec::new();
        let mut current_run: Option<(usize, Color)> = None;
        for cell in &line.cells {
            let color = self.color_for_styles(&cell.styles);
            if current_run.is_none_or(|(_, previous)| previous != color) {
                if let Some((start, previous)) = current_run.take() {
                    style_ranges.push((start..text.len(), previous));
                }
                current_run = Some((text.len(), color));
            }
            text.push(cell.ch);
        }
        if let Some((start, color)) = current_run {
            style_ranges.push((start..text.len(), color));
        }
        let runs: Vec<_> = style_ranges
            .into_iter()
            .map(|(range, color)| TextRun {
                range,
                font_size: FONT_SIZE,
                font_weight: 400.0,
                line_height: Some((LINE_HEIGHT, false)),
                color: brush_for_color(color),
            })
            .collect();
        let scale = paint.device_scale();
        let layout = layout_text_styled(
            paint.text(),
            Arc::from(text),
            FONT_SIZE,
            400.0,
            Some((LINE_HEIGHT, false)),
            TextAlign::Start,
            brush_for_color(self.text_color),
            runs.into(),
            self.font_family.as_ref(),
            None,
        );
        let glyphs = paint.text().glyph_scene_scaled(&layout, scale);
        scene.append(
            &glyphs,
            Some(
                Affine::translate((
                    f64::from(
                        GUTTER_WIDTH + TEXT_INSET + line.segment_x_start_cells as f32 * CELL_WIDTH,
                    ),
                    y,
                )) * Affine::scale(scale.recip()),
            ),
        );
    }

    fn paint_line_number(
        &self,
        scene: &mut Scene,
        paint: &mut PaintContext<'_>,
        number: usize,
        y: f64,
    ) {
        let scale = paint.device_scale();
        let layout = layout_text_styled(
            paint.text(),
            Arc::from(format!("{number:>4}")),
            FONT_SIZE,
            400.0,
            Some((LINE_HEIGHT, false)),
            TextAlign::Start,
            brush_for_color(Color::from_rgb8(0x68, 0x6f, 0x86)),
            Arc::from([]),
            self.font_family.as_ref(),
            None,
        );
        let glyphs = paint.text().glyph_scene_scaled(&layout, scale);
        scene.append(
            &glyphs,
            Some(Affine::translate((10.0, y)) * Affine::scale(scale.recip())),
        );
    }
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self::new()
    }
}

fn floor_char_boundary(text: &str, mut offset: usize) -> usize {
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

impl Widget for CodeEditor {
    fn paint(&mut self, paint: &mut PaintContext<'_>) {
        self.viewport = paint.size();
        let columns = ((self.viewport[0] - GUTTER_WIDTH - TEXT_INSET) / CELL_WIDTH)
            .floor()
            .max(8.0) as usize;
        let rows = self.visible_rows();
        let _ = self
            .state
            .execute(Command::View(ViewCommand::SetViewportWidth {
                width: columns,
            }));
        self.state.set_viewport_height(rows);
        self.state.set_scroll_top(self.scroll_row);

        let grid = self
            .state
            .get_viewport_content_styled(self.scroll_row, rows + 1);
        let mut scene = Scene::new();
        scene.push_clip_layer(
            Fill::NonZero,
            Affine::IDENTITY,
            &Rect::new(
                0.0,
                0.0,
                f64::from(self.viewport[0]),
                f64::from(self.viewport[1]),
            ),
        );
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgb8(0x14, 0x17, 0x1c),
            None,
            &Rect::new(
                0.0,
                0.0,
                f64::from(GUTTER_WIDTH),
                f64::from(self.viewport[1]),
            ),
        );

        for (visible_index, line) in grid.lines.iter().enumerate() {
            let y = visible_index as f64 * f64::from(LINE_HEIGHT);
            if !line.is_wrapped_part {
                self.paint_line_number(&mut scene, paint, line.logical_line_index + 1, y);
            }
            if self.focused
                && let Some((from, to)) = self.selected_offsets()
            {
                let segment_from = from.max(line.char_offset_start);
                let segment_to = to.min(line.char_offset_end);
                if segment_to > segment_from {
                    let cells_before = segment_from - line.char_offset_start;
                    let selected_cells = segment_to - segment_from;
                    let x_before: usize = line
                        .cells
                        .iter()
                        .take(cells_before)
                        .map(|cell| cell.width)
                        .sum();
                    let selected_width: usize = line
                        .cells
                        .iter()
                        .skip(cells_before)
                        .take(selected_cells)
                        .map(|cell| cell.width)
                        .sum();
                    let x0 = GUTTER_WIDTH
                        + TEXT_INSET
                        + (line.segment_x_start_cells + x_before) as f32 * CELL_WIDTH;
                    let x1 = x0 + selected_width as f32 * CELL_WIDTH;
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        Color::from_rgba8(0x45, 0x5a, 0x7a, 0xc0),
                        None,
                        &Rect::new(f64::from(x0), y, f64::from(x1), y + f64::from(LINE_HEIGHT)),
                    );
                }
            }
            self.paint_text_line(&mut scene, paint, line, y);
        }

        if self.focused {
            let cursor = self.state.editor().cursor_position();
            if let Some((row, x_cells)) = self
                .state
                .logical_position_to_visual(cursor.line, cursor.column)
                && row >= self.scroll_row
                && row < self.scroll_row + rows
            {
                let x = GUTTER_WIDTH + TEXT_INSET + x_cells as f32 * CELL_WIDTH;
                let y = (row - self.scroll_row) as f32 * LINE_HEIGHT;
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::from_rgb8(0x89, 0xb4, 0xfa),
                    None,
                    &Rect::new(
                        f64::from(x),
                        f64::from(y + 3.0),
                        f64::from(x + 1.5),
                        f64::from(y + LINE_HEIGHT - 3.0),
                    ),
                );
            }
        }
        scene.pop_layer();
        paint.scene_mut().append(&scene, None);
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if self.disabled {
            return WidgetEventResult::IGNORED;
        }
        match event {
            UiEvent::Pointer(event) if event.phase == PointerPhase::Down => {
                if event.button != Some(wabou_shell::PointerButton::Primary) {
                    return WidgetEventResult::IGNORED;
                }
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                let position = self.position_from_pointer(local_x, local_y);
                let now = Instant::now();
                let click_count = self.last_click.map_or(1, |(at, x, y, count)| {
                    if now.duration_since(at) <= Duration::from_millis(400)
                        && (local_x - x).abs() <= 4.0
                        && (local_y - y).abs() <= 4.0
                    {
                        count.saturating_add(1).min(3)
                    } else {
                        1
                    }
                });
                self.last_click = Some((now, local_x, local_y, click_count));
                let command = if event.modifiers.shift() {
                    CursorCommand::ExtendSelection { to: position }
                } else {
                    CursorCommand::MoveTo {
                        line: position.line,
                        column: position.column,
                    }
                };
                self.execute(Command::Cursor(command));
                if click_count == 2 {
                    self.execute(Command::Cursor(CursorCommand::SelectWord));
                } else if click_count == 3 {
                    self.execute(Command::Cursor(CursorCommand::SelectLine));
                }
                self.selecting = click_count == 1;
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(event) if event.phase == PointerPhase::Move && self.selecting => {
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                if local_y < 0.0 {
                    self.scroll_row = self.scroll_row.saturating_sub(1);
                } else if local_y > self.viewport[1] {
                    self.scroll_row = (self.scroll_row + 1)
                        .min(self.state.total_visual_lines().saturating_sub(1));
                }
                let position = self
                    .position_from_pointer(local_x, local_y.clamp(0.0, self.viewport[1].max(0.0)));
                self.execute(Command::Cursor(CursorCommand::ExtendSelection {
                    to: position,
                }));
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(event) if event.phase == PointerPhase::Up && self.selecting => {
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                let position = self
                    .position_from_pointer(local_x, local_y.clamp(0.0, self.viewport[1].max(0.0)));
                self.execute(Command::Cursor(CursorCommand::ExtendSelection {
                    to: position,
                }));
                self.selecting = false;
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(event) if event.phase == PointerPhase::Cancel && self.selecting => {
                self.selecting = false;
                WidgetEventResult::HANDLED
            }
            UiEvent::Wheel(event) => {
                let previous = self.scroll_row;
                let lines = (event.delta_y / f64::from(LINE_HEIGHT)).round() as isize;
                self.scroll_row = self
                    .scroll_row
                    .saturating_add_signed(lines)
                    .min(self.state.total_visual_lines().saturating_sub(1));
                if previous == self.scroll_row {
                    WidgetEventResult::IGNORED
                } else {
                    WidgetEventResult::HANDLED
                }
            }
            UiEvent::Key(event) if event.phase == KeyPhase::Down => self.key_down(
                &event.key,
                event.modifiers.shift(),
                event.modifiers.primary_shortcut(),
            ),
            UiEvent::TextInput(text) | UiEvent::Paste(text)
                if !self.read_only && !text.is_empty() =>
            {
                if self.execute(Command::Edit(EditCommand::InsertText {
                    text: text.clone(),
                })) {
                    WidgetEventResult::VALUE_CHANGED
                } else {
                    WidgetEventResult::IGNORED
                }
            }
            UiEvent::Ime(ImeEvent::Preedit { text, cursor }) if !self.read_only => {
                if self.update_composition(text, *cursor) {
                    WidgetEventResult::VALUE_CHANGED
                } else {
                    WidgetEventResult::IGNORED
                }
            }
            UiEvent::Ime(ImeEvent::Commit(text)) if !self.read_only => {
                if self.commit_composition(text) {
                    WidgetEventResult::VALUE_CHANGED
                } else {
                    WidgetEventResult::HANDLED
                }
            }
            UiEvent::Ime(ImeEvent::Disabled) if self.composition.is_some() => {
                self.update_composition("", None);
                let _ = self.execute(Command::Edit(EditCommand::EndUndoGroup));
                WidgetEventResult::VALUE_CHANGED
            }
            UiEvent::Focus(focused) => {
                self.focused = *focused;
                WidgetEventResult::HANDLED
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn attribute_changed(&mut self, name: &str, value: &str) {
        match name {
            "value" if value != self.cached_value => {
                let mut replacement = Self::from_text(value);
                replacement.viewport = self.viewport;
                replacement.window_to_local = self.window_to_local;
                replacement.focused = self.focused;
                replacement.disabled = self.disabled;
                replacement.read_only = self.read_only;
                replacement.text_color = self.text_color;
                replacement.font_family = self.font_family.clone();
                *self = replacement;
            }
            "disabled" => self.disabled = value != "false",
            "readonly" | "readOnly" | "read-only" => self.read_only = value != "false",
            _ => {}
        }
    }

    fn attribute_removed(&mut self, name: &str) {
        match name {
            "disabled" => self.disabled = false,
            "readonly" | "readOnly" | "read-only" => self.read_only = false,
            _ => {}
        }
    }

    fn style_changed(&mut self, style: &WidgetStyle) {
        self.text_color = style.color;
        self.font_family = style
            .font_family
            .clone()
            .or_else(|| Some(Arc::from("monospace")));
    }

    fn current_value(&self) -> Option<&str> {
        Some(&self.cached_value)
    }

    fn accepts_focus(&self) -> bool {
        !self.disabled
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([640.0, 420.0])
    }

    fn focus_changed(&mut self, focused: bool) {
        self.focused = focused;
    }

    fn set_window_to_local(&mut self, transform: [f64; 6]) {
        self.window_to_local = transform;
    }

    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        let cursor = self.state.editor().cursor_position();
        let (row, x_cells) = self
            .state
            .logical_position_to_visual(cursor.line, cursor.column)?;
        Some([
            GUTTER_WIDTH + TEXT_INSET + x_cells as f32 * CELL_WIDTH,
            row.saturating_sub(self.scroll_row) as f32 * LINE_HEIGHT,
            2.0,
            LINE_HEIGHT,
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{Modifiers, Point, PointerButton, PointerEvent};

    fn pointer(phase: PointerPhase, x: f64, y: f64, buttons: u32) -> UiEvent {
        UiEvent::Pointer(PointerEvent {
            phase,
            position: Point { x, y },
            button: Some(PointerButton::Primary),
            buttons,
            modifiers: Modifiers::default(),
        })
    }

    #[test]
    fn ordinary_editing_and_undo_use_editor_core_commands() {
        let mut editor = CodeEditor::from_text("{\"ok\":true}");
        editor.execute(Command::Cursor(CursorCommand::MoveTo {
            line: 0,
            column: 11,
        }));
        editor.execute(Command::Edit(EditCommand::InsertText { text: "!".into() }));
        assert_eq!(editor.cached_value, "{\"ok\":true}!");
        editor.execute(Command::Edit(EditCommand::Undo));
        assert_eq!(editor.cached_value, "{\"ok\":true}");
    }

    #[test]
    fn grapheme_deletion_does_not_split_emoji_sequences() {
        let mut editor = CodeEditor::from_text("👨‍👩‍👧‍👦x");
        editor.execute(Command::Cursor(CursorCommand::MoveTo {
            line: 0,
            column: 7,
        }));
        editor.execute(Command::Edit(EditCommand::DeleteGraphemeBack));
        assert_eq!(editor.cached_value, "x");
    }

    #[test]
    fn json_styles_reach_the_headless_snapshot() {
        let editor = CodeEditor::from_text("{\"port\": 9090, \"ok\": true}");
        let grid = editor.state.get_viewport_content_styled(0, 10);
        assert!(
            grid.lines[0]
                .cells
                .iter()
                .any(|cell| !cell.styles.is_empty())
        );
    }

    #[test]
    fn multiline_selection_is_converted_to_document_offsets() {
        let mut editor = CodeEditor::from_text("one\n世界\nthree");
        editor.execute(Command::Cursor(CursorCommand::SetSelection {
            start: Position::new(0, 1),
            end: Position::new(2, 2),
        }));
        assert_eq!(editor.selected_offsets(), Some((1, 9)));
    }

    #[test]
    fn ime_preedit_updates_commit_as_one_undo_group() {
        let mut editor = CodeEditor::from_text("");
        assert!(editor.update_composition("n", Some((1, 1))));
        assert!(editor.update_composition("你", Some((3, 3))));
        assert!(editor.commit_composition("你"));
        assert_eq!(editor.cached_value, "你");
        editor.execute(Command::Edit(EditCommand::Undo));
        assert_eq!(editor.cached_value, "");
    }

    #[test]
    fn ime_cursor_offsets_never_split_utf8() {
        let mut editor = CodeEditor::from_text("");
        assert!(editor.update_composition("你", Some((1, 2))));
        assert_eq!(editor.cached_value, "你");
    }

    #[test]
    fn visible_line_painting_accepts_hidpi_contexts() {
        let mut editor = CodeEditor::from_text("{\n  \"port\": 9090\n}");
        let mut text = wabou_shell::text::TextContext::new();
        let mut paint = PaintContext::new(640.0, 88.0, 2.0, &mut text);
        editor.paint(&mut paint);
        assert_eq!(editor.viewport, [640.0, 88.0]);
        let _scene = paint.finish();
    }

    #[test]
    fn pointer_drag_extends_selection_until_release() {
        let mut editor = CodeEditor::from_text("abcdef\nsecond");
        editor.viewport = [640.0, 88.0];

        editor.handle_event(&pointer(
            PointerPhase::Down,
            f64::from(GUTTER_WIDTH + TEXT_INSET),
            10.0,
            1,
        ));
        editor.handle_event(&pointer(
            PointerPhase::Move,
            f64::from(GUTTER_WIDTH + TEXT_INSET + CELL_WIDTH * 4.0),
            10.0,
            1,
        ));
        editor.handle_event(&pointer(
            PointerPhase::Up,
            f64::from(GUTTER_WIDTH + TEXT_INSET + CELL_WIDTH * 4.0),
            10.0,
            0,
        ));

        assert_eq!(editor.selected_offsets(), Some((0, 4)));
        assert!(!editor.selecting);
    }

    #[test]
    fn pointer_coordinates_follow_content_box_affine_geometry() {
        let mut editor = CodeEditor::from_text("abcdef");
        editor.viewport = [640.0, 88.0];
        editor.set_window_to_local([0.5, 0.0, 0.0, 0.5, -50.0, -10.0]);
        let local_x = GUTTER_WIDTH + TEXT_INSET + CELL_WIDTH * 3.0;
        editor.handle_event(&pointer(
            PointerPhase::Down,
            f64::from((local_x + 50.0) * 2.0),
            30.0,
            1,
        ));
        assert_eq!(editor.state.editor().cursor_position(), Position::new(0, 3));
    }

    #[test]
    fn cancelled_pointer_drag_releases_capture_state() {
        let mut editor = CodeEditor::from_text("abcdef");
        editor.viewport = [640.0, 88.0];
        editor.handle_event(&pointer(PointerPhase::Down, 68.0, 10.0, 1));
        editor.handle_event(&pointer(PointerPhase::Cancel, 68.0, 10.0, 0));
        assert!(!editor.selecting);
    }
}
