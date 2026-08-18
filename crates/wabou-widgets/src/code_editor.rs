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
const FALLBACK_CELL_WIDTH: f32 = 8.4;
const GUTTER_WIDTH: f32 = 58.0;
const TEXT_INSET: f32 = 10.0;
const CONTENT_CHANGED: wabou_shell::WidgetChanges =
    wabou_shell::WidgetChanges::REDRAW.union(wabou_shell::WidgetChanges::SEMANTICS);

#[derive(Clone, Copy, Debug)]
struct EditorGeometry {
    cell_width: f32,
    line_height: f32,
    gutter_width: f32,
    text_inset: f32,
}

impl Default for EditorGeometry {
    fn default() -> Self {
        Self {
            cell_width: FALLBACK_CELL_WIDTH,
            line_height: LINE_HEIGHT,
            gutter_width: GUTTER_WIDTH,
            text_inset: TEXT_INSET,
        }
    }
}

impl EditorGeometry {
    fn text_origin_x(self) -> f32 {
        self.gutter_width + self.text_inset
    }

    fn x_for_cell(self, cell: usize) -> f32 {
        self.text_origin_x() + cell as f32 * self.cell_width
    }

    fn cell_for_x(self, x: f32) -> usize {
        ((x - self.text_origin_x()).max(0.0) / self.cell_width).round() as usize
    }

    fn row_for_y(self, y: f32) -> usize {
        (y.max(0.0) / self.line_height).floor() as usize
    }

    fn y_for_row(self, row: usize) -> f32 {
        row as f32 * self.line_height
    }

    fn visible_rows(self, height: f32) -> usize {
        (height / self.line_height).floor().max(1.0) as usize
    }

    fn visible_columns(self, width: f32) -> usize {
        ((width - self.text_origin_x()) / self.cell_width)
            .floor()
            .max(8.0) as usize
    }
}

/// Native multiline code editor backed by `editor-core`.
///
/// The current implementation provides JSON highlighting, Unicode-aware
/// selection/editing, IME, clipboard copy/paste, undo/redo, and soft wrapping.
pub struct CodeEditor {
    state: EditorStateManager,
    highlighter: RegexHighlightProcessor,
    cached_value: String,
    scroll_row: usize,
    viewport: [f32; 2],
    geometry: EditorGeometry,
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
    /// Construct an empty editable JSON-oriented editor.
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
            geometry: EditorGeometry::default(),
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
        self.geometry.visible_rows(self.viewport[1])
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
        let row = (self.scroll_row + self.geometry.row_for_y(y))
            .min(self.state.total_visual_lines().saturating_sub(1));
        let x_cells = self.geometry.cell_for_x(x);
        self.state
            .visual_position_to_logical(row, x_cells)
            .unwrap_or_else(|| self.state.editor().cursor_position())
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
        if primary && key.eq_ignore_ascii_case("c") {
            return self
                .selected_text()
                .map_or(WidgetEventResult::IGNORED, WidgetEventResult::copy);
        }
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
                "Enter" => Some(Command::Edit(EditCommand::InsertText { text: "\n".into() })),
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

    fn selected_text(&self) -> Option<String> {
        let (from, to) = self.selected_offsets()?;
        Some(
            self.cached_value
                .chars()
                .skip(from)
                .take(to - from)
                .collect(),
        )
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
                line_height: Some((self.geometry.line_height, false)),
                color: brush_for_color(color),
            })
            .collect();
        let scale = paint.device_scale();
        let layout = layout_text_styled(
            paint.text(),
            Arc::from(text),
            FONT_SIZE,
            400.0,
            Some((self.geometry.line_height, false)),
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
                    f64::from(self.geometry.x_for_cell(line.segment_x_start_cells)),
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
            Some((self.geometry.line_height, false)),
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
        let metrics = layout_text_styled(
            paint.text(),
            Arc::from("0"),
            FONT_SIZE,
            400.0,
            Some((self.geometry.line_height, false)),
            TextAlign::Start,
            brush_for_color(self.text_color),
            Arc::from([]),
            self.font_family.as_ref(),
            None,
        );
        self.geometry.cell_width = metrics.width().max(f32::EPSILON);
        let columns = self.geometry.visible_columns(self.viewport[0]);
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
                f64::from(self.geometry.gutter_width),
                f64::from(self.viewport[1]),
            ),
        );

        for (visible_index, line) in grid.lines.iter().enumerate() {
            let y = f64::from(self.geometry.y_for_row(visible_index));
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
                    let x0 = self
                        .geometry
                        .x_for_cell(line.segment_x_start_cells + x_before);
                    let x1 = x0 + selected_width as f32 * self.geometry.cell_width;
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        Color::from_rgba8(0x45, 0x5a, 0x7a, 0xc0),
                        None,
                        &Rect::new(
                            f64::from(x0),
                            y,
                            f64::from(x1),
                            y + f64::from(self.geometry.line_height),
                        ),
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
                let x = self.geometry.x_for_cell(x_cells);
                let y = self.geometry.y_for_row(row - self.scroll_row);
                scene.fill(
                    Fill::NonZero,
                    Affine::IDENTITY,
                    Color::from_rgb8(0x89, 0xb4, 0xfa),
                    None,
                    &Rect::new(
                        f64::from(x),
                        f64::from(y + 3.0),
                        f64::from(x + 1.5),
                        f64::from(y + self.geometry.line_height - 3.0),
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
                let (local_x, local_y) = (event.position.x as f32, event.position.y as f32);
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
                let (local_x, local_y) = (event.position.x as f32, event.position.y as f32);
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
                let (local_x, local_y) = (event.position.x as f32, event.position.y as f32);
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
                let lines = (event.delta_y / f64::from(self.geometry.line_height)).round() as isize;
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

    fn attribute_changed(&mut self, name: &str, value: &str) -> wabou_shell::WidgetChanges {
        match name {
            "value" if value != self.cached_value => {
                let mut replacement = Self::from_text(value);
                replacement.viewport = self.viewport;
                replacement.geometry = self.geometry;
                replacement.focused = self.focused;
                replacement.disabled = self.disabled;
                replacement.read_only = self.read_only;
                replacement.text_color = self.text_color;
                replacement.font_family = self.font_family.clone();
                *self = replacement;
                CONTENT_CHANGED
            }
            "value" => CONTENT_CHANGED,
            "disabled" => {
                self.disabled = value != "false";
                CONTENT_CHANGED
            }
            "readOnly" => {
                self.read_only = value != "false";
                CONTENT_CHANGED
            }
            "font-family" => {
                self.font_family = Some(Arc::from(value));
                wabou_shell::WidgetChanges::REDRAW
            }
            _ => wabou_shell::WidgetChanges::empty(),
        }
    }

    fn attribute_removed(&mut self, name: &str) -> wabou_shell::WidgetChanges {
        match name {
            "disabled" => {
                self.disabled = false;
                CONTENT_CHANGED
            }
            "readOnly" => {
                self.read_only = false;
                CONTENT_CHANGED
            }
            "font-family" => {
                self.font_family = Some(Arc::from("monospace"));
                wabou_shell::WidgetChanges::REDRAW
            }
            _ => wabou_shell::WidgetChanges::empty(),
        }
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> wabou_shell::WidgetChanges {
        self.text_color = style.color;
        wabou_shell::WidgetChanges::REDRAW
    }

    fn current_value(&self) -> Option<&str> {
        Some(&self.cached_value)
    }

    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        wabou_shell::WidgetAccessibility {
            role: Some(wabou_shell::SemanticRole::TextInput),
            value: Some(self.cached_value.clone()),
            disabled: Some(self.disabled),
            ..Default::default()
        }
    }

    fn accepts_focus(&self) -> bool {
        !self.disabled
    }

    fn accepts_text_input(&self) -> bool {
        !self.disabled && !self.read_only
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([640.0, 420.0])
    }

    fn focus_changed(&mut self, focused: bool) -> wabou_shell::WidgetChanges {
        self.focused = focused;
        wabou_shell::WidgetChanges::REDRAW
    }

    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        let cursor = self.state.editor().cursor_position();
        let (row, x_cells) = self
            .state
            .logical_position_to_visual(cursor.line, cursor.column)?;
        Some([
            self.geometry.x_for_cell(x_cells),
            self.geometry.y_for_row(row.saturating_sub(self.scroll_row)),
            2.0,
            self.geometry.line_height,
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
    fn enter_inserts_a_newline_and_replaces_the_selection() {
        let mut editor = CodeEditor::from_text("true");
        editor.execute(Command::Cursor(CursorCommand::SetSelection {
            start: Position::new(0, 1),
            end: Position::new(0, 3),
        }));

        let result = editor.key_down("Enter", false, false);

        assert_eq!(editor.cached_value, "t\ne");
        assert!(result.is_handled());
        assert!(result.consumes_key_text());
        assert!(result.value_changed());
    }

    #[test]
    fn enter_does_not_modify_a_read_only_editor() {
        let mut editor = CodeEditor::from_text("true");
        editor.read_only = true;

        let result = editor.key_down("Enter", false, false);

        assert_eq!(editor.cached_value, "true");
        assert!(!result.is_handled());
    }

    #[test]
    fn editor_accepts_only_the_canonical_read_only_attribute() {
        let mut editor = CodeEditor::from_text("true");

        assert!(editor.attribute_changed("readonly", "true").is_empty());
        assert!(!editor.read_only);
        assert!(!editor.attribute_changed("readOnly", "true").is_empty());
        assert!(editor.read_only);
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
        assert_eq!(editor.selected_text().as_deref(), Some("ne\n世界\nth"));
        let result = editor.key_down("c", false, true);
        assert_eq!(
            result.clipboard_request(),
            Some(&wabou_shell::ClipboardRequest::Write("ne\n世界\nth".into()))
        );
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
        assert!(editor.geometry.cell_width.is_finite() && editor.geometry.cell_width > 0.0);
        let third_cell = editor.geometry.x_for_cell(3);
        editor.handle_event(&pointer(PointerPhase::Down, f64::from(third_cell), 10.0, 1));
        assert_eq!(editor.state.editor().cursor_position(), Position::new(0, 1));
        let _scene = paint.finish();
    }

    #[test]
    fn pointer_drag_extends_selection_until_release() {
        let mut editor = CodeEditor::from_text("abcdef\nsecond");
        editor.viewport = [640.0, 88.0];

        editor.handle_event(&pointer(
            PointerPhase::Down,
            f64::from(editor.geometry.text_origin_x()),
            10.0,
            1,
        ));
        editor.handle_event(&pointer(
            PointerPhase::Move,
            f64::from(editor.geometry.x_for_cell(4)),
            10.0,
            1,
        ));
        editor.handle_event(&pointer(
            PointerPhase::Up,
            f64::from(editor.geometry.x_for_cell(4)),
            10.0,
            0,
        ));

        assert_eq!(editor.selected_offsets(), Some((0, 4)));
        assert!(!editor.selecting);
    }

    #[test]
    fn pointer_coordinates_are_content_box_local() {
        let mut editor = CodeEditor::from_text("abcdef");
        editor.viewport = [640.0, 88.0];
        let local_x = editor.geometry.x_for_cell(3);
        editor.handle_event(&pointer(PointerPhase::Down, f64::from(local_x), 10.0, 1));
        assert_eq!(editor.state.editor().cursor_position(), Position::new(0, 3));
    }

    #[test]
    fn pointer_insertion_uses_the_same_measured_cell_geometry_as_paint() {
        let mut editor = CodeEditor::from_text("{\n  \"enabled\": true\n}");
        editor.viewport = [640.0, 88.0];
        let column = 15;
        editor.handle_event(&pointer(
            PointerPhase::Down,
            f64::from(editor.geometry.x_for_cell(column)),
            f64::from(editor.geometry.y_for_row(1) + 10.0),
            1,
        ));
        assert_eq!(
            editor.state.editor().cursor_position(),
            Position::new(1, column)
        );
        editor.handle_event(&UiEvent::TextInput(",".into()));
        assert_eq!(editor.cached_value, "{\n  \"enabled\": tr,ue\n}");
    }

    #[test]
    fn editor_geometry_round_trips_cells_and_rows() {
        let geometry = EditorGeometry {
            cell_width: 9.25,
            ..EditorGeometry::default()
        };
        for cell in 0..40 {
            assert_eq!(geometry.cell_for_x(geometry.x_for_cell(cell)), cell);
        }
        for row in 0..20 {
            assert_eq!(geometry.row_for_y(geometry.y_for_row(row) + 0.5), row);
        }
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
