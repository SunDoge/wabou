use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use serde::Deserialize;
use unicode_width::UnicodeWidthChar;
use vello_common::{
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};
use wabou_shell::{
    KeyPhase, PaintContext, PointerPhase, UiEvent, Widget, WidgetEventResult, WidgetGeometry,
    WidgetImePurpose, WidgetImeState, WidgetStyle, WidgetTextSelection, WidgetTextSelectionKind,
    decode_widget_config,
    style::TextAlign,
    text::{TextRun, brush_for_color, layout_text_styled},
};
use wabou_shell_vello::{PaintScene, Scene};

use crate::text_input::{surrounding_excerpt, utf16_offset_to_byte};

const FONT_SIZE: f32 = 14.0;
const LINE_HEIGHT: f32 = 22.0;
const FALLBACK_CELL_WIDTH: f32 = 8.4;
const GUTTER_WIDTH: f32 = 58.0;
const TEXT_INSET: f32 = 10.0;
const TAB_WIDTH: usize = 2;
const CONTENT_CHANGED: wabou_shell::WidgetChanges =
    wabou_shell::WidgetChanges::REDRAW.union(wabou_shell::WidgetChanges::SEMANTICS);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum HighlightKind {
    Property,
    String,
    Number,
    Boolean,
    Null,
    Added,
    Removed,
    Meta,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HighlightRange {
    from: usize,
    to: usize,
    kind: HighlightKind,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct SyntaxConfig {
    language: String,
    offset_encoding: String,
    document_length: usize,
    ranges: Vec<HighlightRange>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SelectionConfig {
    anchor: usize,
    head: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct CompositionConfig {
    text: String,
    cursor_start: Option<usize>,
    cursor_end: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EditorViewportConfig {
    selection: SelectionConfig,
    composition: Option<CompositionConfig>,
    syntax: Option<SyntaxConfig>,
}

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
    fn cell_for_x(self, x: f32) -> f32 {
        (x - self.text_origin_x()).max(0.0) / self.cell_width
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

#[derive(Clone, Debug)]
struct VisualCell {
    ch: char,
    from: usize,
    to: usize,
    start_cell: usize,
    end_cell: usize,
}

#[derive(Clone, Debug)]
struct VisualLine {
    logical_line: usize,
    wrapped: bool,
    start: usize,
    end: usize,
    cells: Vec<VisualCell>,
}

impl VisualLine {
    fn display_text(&self) -> String {
        self.cells.iter().map(|cell| cell.ch).collect()
    }

    fn cell_for_offset(&self, offset: usize) -> usize {
        self.cells
            .iter()
            .find(|cell| offset <= cell.from)
            .map_or_else(
                || self.cells.last().map_or(0, |cell| cell.end_cell),
                |cell| cell.start_cell,
            )
    }

    fn offset_for_cell(&self, cell: f32) -> usize {
        for item in &self.cells {
            let midpoint = (item.start_cell + item.end_cell) as f32 / 2.0;
            if cell < midpoint {
                return item.from;
            }
            if cell <= item.end_cell as f32 {
                return item.to;
            }
        }
        self.end
    }
}

/// Controlled native viewport for Wabou's general-purpose `Editor` primitive.
///
/// It deliberately does not edit text. JavaScript owns CodeMirror transactions;
/// this widget owns paint, scrolling, pointer hit testing, clipboard requests,
/// and the platform IME projection.
pub struct EditorViewport {
    value: String,
    selection: SelectionConfig,
    selection_kind: WidgetTextSelectionKind,
    composition: Option<CompositionConfig>,
    highlight_ranges: Vec<HighlightRange>,
    scroll_row: usize,
    viewport: [f32; 2],
    geometry: EditorGeometry,
    focused: bool,
    selecting: bool,
    disabled: bool,
    read_only: bool,
    last_click: Option<(Instant, f32, f32, u8)>,
    text_color: Color,
    font_family: Option<Arc<str>>,
    font_italic: bool,
}

impl EditorViewport {
    /// Construct an empty editor viewport.
    pub fn new() -> Self {
        Self {
            value: String::new(),
            selection: SelectionConfig::default(),
            selection_kind: WidgetTextSelectionKind::Simple,
            composition: None,
            highlight_ranges: Vec::new(),
            scroll_row: 0,
            viewport: [0.0, 0.0],
            geometry: EditorGeometry::default(),
            focused: false,
            selecting: false,
            disabled: false,
            read_only: false,
            last_click: None,
            text_color: Color::from_rgb8(0xe6, 0xe9, 0xef),
            font_family: Some(Arc::from("monospace")),
            font_italic: false,
        }
    }

    fn document_len(&self) -> usize {
        self.value.encode_utf16().count()
    }

    fn visual_lines(&self) -> Vec<VisualLine> {
        let columns = self.geometry.visible_columns(self.viewport[0]);
        let mut result = Vec::new();
        let mut logical_line = 0;
        let mut line_start = 0;
        for segment in self.value.split_inclusive('\n') {
            let text = segment.strip_suffix('\n').unwrap_or(segment);
            self.wrap_line(text, line_start, logical_line, columns, &mut result);
            line_start += segment.encode_utf16().count();
            logical_line += 1;
        }
        if self.value.is_empty() || self.value.ends_with('\n') {
            self.wrap_line("", line_start, logical_line, columns, &mut result);
        }
        result
    }

    fn wrap_line(
        &self,
        text: &str,
        start: usize,
        logical_line: usize,
        columns: usize,
        output: &mut Vec<VisualLine>,
    ) {
        let mut cells = Vec::new();
        let mut offset = start;
        let mut width = 0;
        let mut wrapped = false;
        let push = |output: &mut Vec<VisualLine>,
                    cells: &mut Vec<VisualCell>,
                    wrapped: bool,
                    fallback: usize| {
            let line_start = cells.first().map_or(fallback, |cell| cell.from);
            let line_end = cells.last().map_or(fallback, |cell| cell.to);
            output.push(VisualLine {
                logical_line,
                wrapped,
                start: line_start,
                end: line_end,
                cells: std::mem::take(cells),
            });
        };
        for ch in text.chars() {
            let utf16 = ch.len_utf16();
            let char_width = if ch == '\t' {
                TAB_WIDTH - (width % TAB_WIDTH)
            } else {
                UnicodeWidthChar::width(ch).unwrap_or(1).max(1)
            };
            if width > 0 && width + char_width > columns {
                push(output, &mut cells, wrapped, offset);
                wrapped = true;
                width = 0;
            }
            if ch == '\t' {
                for index in 0..char_width {
                    cells.push(VisualCell {
                        ch: ' ',
                        from: offset,
                        to: offset + utf16,
                        start_cell: width + index,
                        end_cell: width + index + 1,
                    });
                }
            } else {
                cells.push(VisualCell {
                    ch,
                    from: offset,
                    to: offset + utf16,
                    start_cell: width,
                    end_cell: width + char_width,
                });
            }
            width += char_width;
            offset += utf16;
        }
        push(output, &mut cells, wrapped, offset);
    }

    fn visible_rows(&self) -> usize {
        self.geometry.visible_rows(self.viewport[1])
    }
    fn max_scroll_row(&self) -> usize {
        self.visual_lines()
            .len()
            .saturating_sub(self.visible_rows())
    }
    fn clamp_scroll_row(&mut self) {
        self.scroll_row = self.scroll_row.min(self.max_scroll_row());
    }

    fn offset_from_pointer(&self, x: f32, y: f32) -> usize {
        let lines = self.visual_lines();
        let row = (self.scroll_row + self.geometry.row_for_y(y)).min(lines.len().saturating_sub(1));
        lines
            .get(row)
            .map_or(0, |line| line.offset_for_cell(self.geometry.cell_for_x(x)))
    }

    fn utf16_to_byte(&self, target: usize) -> usize {
        let mut utf16 = 0;
        for (byte, ch) in self.value.char_indices() {
            if utf16 >= target {
                return byte;
            }
            utf16 += ch.len_utf16();
        }
        self.value.len()
    }

    fn selected_text(&self) -> Option<String> {
        let from = self.selection.anchor.min(self.selection.head);
        let to = self.selection.anchor.max(self.selection.head);
        (from != to)
            .then(|| self.value[self.utf16_to_byte(from)..self.utf16_to_byte(to)].to_owned())
    }

    fn select_word_at(&mut self, offset: usize) {
        let chars: Vec<_> = self
            .value
            .char_indices()
            .map(|(byte, ch)| (byte, ch, ch.len_utf16()))
            .collect();
        let mut utf16 = 0;
        let index = chars.iter().position(|(_, _, width)| {
            let hit = utf16 <= offset && offset < utf16 + width;
            utf16 += width;
            hit
        });
        let Some(index) = index else {
            self.selection = SelectionConfig {
                anchor: offset,
                head: offset,
            };
            return;
        };
        let class = |ch: char| ch.is_alphanumeric() || ch == '_';
        let wanted = class(chars[index].1);
        let mut from = index;
        let mut to = index + 1;
        while from > 0 && class(chars[from - 1].1) == wanted && !chars[from - 1].1.is_whitespace() {
            from -= 1;
        }
        while to < chars.len() && class(chars[to].1) == wanted && !chars[to].1.is_whitespace() {
            to += 1;
        }
        let anchor = chars[..from].iter().map(|item| item.2).sum();
        let head = chars[..to].iter().map(|item| item.2).sum();
        self.selection = SelectionConfig { anchor, head };
    }

    fn select_line_at(&mut self, offset: usize) {
        let byte = self.utf16_to_byte(offset);
        let start_byte = self.value[..byte].rfind('\n').map_or(0, |index| index + 1);
        let end_byte = self.value[byte..]
            .find('\n')
            .map_or(self.value.len(), |index| byte + index);
        self.selection = SelectionConfig {
            anchor: self.value[..start_byte].encode_utf16().count(),
            head: self.value[..end_byte].encode_utf16().count(),
        };
    }

    fn color_at_offset(&self, offset: usize) -> Color {
        let index = self
            .highlight_ranges
            .partition_point(|range| range.to <= offset);
        let kind = self
            .highlight_ranges
            .get(index)
            .and_then(|range| (range.from <= offset && offset < range.to).then_some(range.kind));
        let [red, green, blue, _] = self.text_color.to_rgba8().to_u8_array();
        let light_surface = u16::from(red) + u16::from(green) + u16::from(blue) < 384;
        match (light_surface, kind) {
            (true, Some(HighlightKind::Property | HighlightKind::Meta)) => {
                Color::from_rgb8(0x05, 0x50, 0xae)
            }
            (true, Some(HighlightKind::String | HighlightKind::Added)) => {
                Color::from_rgb8(0x11, 0x63, 0x29)
            }
            (true, Some(HighlightKind::Number)) => Color::from_rgb8(0x95, 0x38, 0x00),
            (true, Some(HighlightKind::Boolean)) => Color::from_rgb8(0x82, 0x50, 0xdf),
            (true, Some(HighlightKind::Null)) => Color::from_rgb8(0x57, 0x60, 0x6a),
            (true, Some(HighlightKind::Removed)) => Color::from_rgb8(0xcf, 0x22, 0x2e),
            (false, Some(HighlightKind::Property | HighlightKind::Meta)) => {
                Color::from_rgb8(0x89, 0xb4, 0xfa)
            }
            (false, Some(HighlightKind::String | HighlightKind::Added)) => {
                Color::from_rgb8(0xa6, 0xe3, 0xa1)
            }
            (false, Some(HighlightKind::Number)) => Color::from_rgb8(0xfa, 0xb3, 0x87),
            (false, Some(HighlightKind::Boolean)) => Color::from_rgb8(0xc6, 0x9d, 0xf7),
            (false, Some(HighlightKind::Null)) => Color::from_rgb8(0x7f, 0x84, 0x9c),
            (false, Some(HighlightKind::Removed)) => Color::from_rgb8(0xf3, 0x8b, 0xa8),
            (_, None) => self.text_color,
        }
    }

    fn paint_text_line(
        &self,
        scene: &mut Scene,
        paint: &mut PaintContext<'_>,
        line: &VisualLine,
        y: f64,
    ) {
        let text = line.display_text();
        let mut runs = Vec::new();
        let mut byte = 0;
        let mut run_start = 0;
        let mut color = line
            .cells
            .first()
            .map_or(self.text_color, |cell| self.color_at_offset(cell.from));
        for cell in &line.cells {
            let next = self.color_at_offset(cell.from);
            if next != color {
                runs.push((run_start..byte, color));
                run_start = byte;
                color = next;
            }
            byte += cell.ch.len_utf8();
        }
        if byte > run_start {
            runs.push((run_start..byte, color));
        }
        let runs: Vec<_> = runs
            .into_iter()
            .map(|(range, color)| TextRun {
                range,
                font_size: FONT_SIZE,
                font_weight: 400.0,
                font_italic: self.font_italic,
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
            self.font_italic,
            Some((self.geometry.line_height, false)),
            TextAlign::Start,
            brush_for_color(self.text_color),
            runs.into(),
            self.font_family.as_ref(),
            None,
        );
        let glyphs = paint.text().glyph_scene_scaled(&layout, scale);
        scene.append_scene(
            (*glyphs).clone(),
            Affine::translate((f64::from(self.geometry.text_origin_x()), y))
                * Affine::scale(scale.recip()),
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
            self.font_italic,
            Some((self.geometry.line_height, false)),
            TextAlign::Start,
            brush_for_color(Color::from_rgb8(0x68, 0x6f, 0x86)),
            Arc::from([]),
            self.font_family.as_ref(),
            None,
        );
        let glyphs = paint.text().glyph_scene_scaled(&layout, scale);
        scene.append_scene(
            (*glyphs).clone(),
            Affine::translate((10.0, y)) * Affine::scale(scale.recip()),
        );
    }

    fn caret_geometry(&self) -> Option<(usize, usize)> {
        let lines = self.visual_lines();
        lines.iter().enumerate().find_map(|(row, line)| {
            (line.start <= self.selection.head && self.selection.head <= line.end)
                .then(|| (row, line.cell_for_offset(self.selection.head)))
        })
    }

    fn composition_cursor_utf16(&self) -> Option<usize> {
        let composition = self.composition.as_ref()?;
        let cursor = composition.cursor_end?;
        composition
            .text
            .get(..cursor)
            .map(|prefix| prefix.encode_utf16().count())
    }

    fn ime_caret_area(&self) -> Option<[f32; 4]> {
        let (row, mut cell) = self.caret_geometry()?;
        if let Some(composition) = self.composition.as_ref()
            && let Some(cursor) = composition.cursor_end
            && let Some(prefix) = composition.text.get(..cursor)
        {
            cell += prefix
                .chars()
                .map(|character| UnicodeWidthChar::width(character).unwrap_or(1).max(1))
                .sum::<usize>();
        }
        let visible_row = row.saturating_sub(self.scroll_row);
        let x = self
            .geometry
            .x_for_cell(cell)
            .clamp(0.0, self.viewport[0].max(0.0));
        let y = self
            .geometry
            .y_for_row(visible_row)
            .clamp(0.0, (self.viewport[1] - self.geometry.line_height).max(0.0));
        Some([x, y, x + 1.5, y + self.geometry.line_height])
    }
}

impl Default for EditorViewport {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for EditorViewport {
    fn paint(&mut self, paint: &mut PaintContext<'_>) {
        self.viewport = paint.size();
        let metrics = layout_text_styled(
            paint.text(),
            Arc::from("0"),
            FONT_SIZE,
            400.0,
            self.font_italic,
            Some((self.geometry.line_height, false)),
            TextAlign::Start,
            brush_for_color(self.text_color),
            Arc::from([]),
            self.font_family.as_ref(),
            None,
        );
        self.geometry.cell_width = metrics.width().max(f32::EPSILON);
        self.clamp_scroll_row();
        let lines = self.visual_lines();
        let rows = self.visible_rows();
        let mut scene = Scene::new();
        scene.push_clip_layer(
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

        let selected_from = self.selection.anchor.min(self.selection.head);
        let selected_to = self.selection.anchor.max(self.selection.head);
        for (visible_index, line) in lines
            .iter()
            .skip(self.scroll_row)
            .take(rows + 1)
            .enumerate()
        {
            let y = f64::from(self.geometry.y_for_row(visible_index));
            if !line.wrapped {
                self.paint_line_number(&mut scene, paint, line.logical_line + 1, y);
            }
            let from = selected_from.max(line.start);
            let to = selected_to.min(line.end);
            if self.focused && from < to {
                let x0 = self.geometry.x_for_cell(line.cell_for_offset(from));
                let x1 = self.geometry.x_for_cell(line.cell_for_offset(to));
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
            self.paint_text_line(&mut scene, paint, line, y);
        }

        if self.focused
            && let Some((row, cell)) = self.caret_geometry()
            && row >= self.scroll_row
            && row < self.scroll_row + rows
        {
            let x = self.geometry.x_for_cell(cell);
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
            if let Some(composition) = &self.composition
                && !composition.text.is_empty()
            {
                // Cursor offsets are retained in the controlled preedit model;
                // platform candidate placement currently uses the base caret.
                let _composition_cursor = (composition.cursor_start, composition.cursor_end);
                let layout = layout_text_styled(
                    paint.text(),
                    Arc::from(composition.text.as_str()),
                    FONT_SIZE,
                    400.0,
                    self.font_italic,
                    Some((self.geometry.line_height, false)),
                    TextAlign::Start,
                    brush_for_color(self.text_color),
                    Arc::from([]),
                    self.font_family.as_ref(),
                    None,
                );
                let scale = paint.device_scale();
                let glyphs = paint.text().glyph_scene_scaled(&layout, scale);
                scene.append_scene(
                    (*glyphs).clone(),
                    Affine::translate((f64::from(x), f64::from(y))) * Affine::scale(scale.recip()),
                );
            }
        }
        scene.pop_layer();
        paint.scene_mut().append_scene(scene, Affine::IDENTITY);
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if self.disabled {
            return WidgetEventResult::IGNORED;
        }
        match event {
            UiEvent::Pointer(event)
                if event.phase == PointerPhase::Down
                    && event.button == Some(wabou_shell::PointerButton::Primary) =>
            {
                let (x, y) = (event.position.x as f32, event.position.y as f32);
                let offset = self.offset_from_pointer(x, y);
                let now = Instant::now();
                let clicks = self.last_click.map_or(1, |(at, old_x, old_y, count)| {
                    if now.duration_since(at) <= Duration::from_millis(400)
                        && (x - old_x).abs() <= 4.0
                        && (y - old_y).abs() <= 4.0
                    {
                        count.saturating_add(1).min(3)
                    } else {
                        1
                    }
                });
                self.last_click = Some((now, x, y, clicks));
                if event.modifiers.shift() {
                    self.selection.head = offset;
                } else {
                    self.selection = SelectionConfig {
                        anchor: offset,
                        head: offset,
                    };
                }
                self.selection_kind = WidgetTextSelectionKind::Simple;
                if clicks == 2 {
                    self.select_word_at(offset);
                    self.selection_kind = WidgetTextSelectionKind::Word;
                } else if clicks == 3 {
                    self.select_line_at(offset);
                    self.selection_kind = WidgetTextSelectionKind::Line;
                }
                self.selecting = clicks == 1;
                WidgetEventResult::selection_changed_result()
            }
            UiEvent::Pointer(event) if event.phase == PointerPhase::Move && self.selecting => {
                let y = event.position.y as f32;
                if y < 0.0 {
                    self.scroll_row = self.scroll_row.saturating_sub(1);
                } else if y > self.viewport[1] {
                    self.scroll_row = (self.scroll_row + 1).min(self.max_scroll_row());
                }
                self.selection.head = self.offset_from_pointer(
                    event.position.x as f32,
                    y.clamp(0.0, self.viewport[1].max(0.0)),
                );
                self.selection_kind = WidgetTextSelectionKind::Simple;
                WidgetEventResult::selection_changed_result()
            }
            UiEvent::Pointer(event) if event.phase == PointerPhase::Up && self.selecting => {
                self.selection.head = self.offset_from_pointer(
                    event.position.x as f32,
                    (event.position.y as f32).clamp(0.0, self.viewport[1].max(0.0)),
                );
                self.selecting = false;
                WidgetEventResult::selection_changed_result()
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
                    .min(self.max_scroll_row());
                if previous == self.scroll_row {
                    WidgetEventResult::IGNORED
                } else {
                    WidgetEventResult::HANDLED
                }
            }
            UiEvent::Key(event)
                if event.phase == KeyPhase::Down
                    && event.modifiers.primary_shortcut()
                    && event.key.eq_ignore_ascii_case("c") =>
            {
                self.selected_text()
                    .map_or(WidgetEventResult::IGNORED, WidgetEventResult::copy)
            }
            UiEvent::Key(event)
                if event.phase == KeyPhase::Down
                    && event.modifiers.primary_shortcut()
                    && event.key.eq_ignore_ascii_case("v")
                    && !self.read_only =>
            {
                WidgetEventResult::paste()
            }
            // CodeMirror receives editing keys, committed text and paste via
            // the JS event bridge. This widget is a controlled viewport and
            // must never mutate its mirrored document independently.
            UiEvent::Key(event) if event.phase == KeyPhase::Down => WidgetEventResult::IGNORED,
            UiEvent::TextInput(_) | UiEvent::Paste(_) => WidgetEventResult::IGNORED,
            UiEvent::Ime(wabou_shell::ImeEvent::Commit(_)) => WidgetEventResult::IGNORED,
            UiEvent::Ime(wabou_shell::ImeEvent::Preedit { text, cursor }) if !self.read_only => {
                self.composition = (!text.is_empty()).then(|| CompositionConfig {
                    text: text.clone(),
                    cursor_start: cursor.map(|range| range.0),
                    cursor_end: cursor.map(|range| range.1),
                });
                WidgetEventResult::HANDLED
            }
            UiEvent::Ime(wabou_shell::ImeEvent::DeleteSurrounding { .. }) => {
                WidgetEventResult::IGNORED
            }
            UiEvent::Ime(wabou_shell::ImeEvent::Disabled) => {
                self.composition = None;
                WidgetEventResult::HANDLED
            }
            UiEvent::Ime(wabou_shell::ImeEvent::Enabled) => WidgetEventResult::HANDLED,
            UiEvent::Focus(focused) => {
                self.focused = *focused;
                WidgetEventResult::HANDLED
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn layout_changed(&mut self, geometry: WidgetGeometry) {
        self.viewport = geometry.content_size;
        self.clamp_scroll_row();
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> wabou_shell::WidgetChanges {
        match name {
            "value" if value != self.value => {
                self.value.clear();
                self.value.push_str(value);
                let length = self.document_len();
                self.selection.anchor = self.selection.anchor.min(length);
                self.selection.head = self.selection.head.min(length);
                self.clamp_scroll_row();
                CONTENT_CHANGED
            }
            "value" => wabou_shell::WidgetChanges::empty(),
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

    fn config_changed(&mut self, json: &str) -> Result<wabou_shell::WidgetChanges, String> {
        let config: EditorViewportConfig = decode_widget_config(json)?;
        let length = self.document_len();
        if config.selection.anchor > length || config.selection.head > length {
            return Err("Editor viewport selection lies outside the document".into());
        }
        if let Some(syntax) = &config.syntax {
            if syntax.language != "json" && syntax.language != "diff" {
                return Err(format!(
                    "unsupported Editor viewport language `{}`",
                    syntax.language
                ));
            }
            if syntax.offset_encoding != "utf16" {
                return Err(format!(
                    "unsupported Editor viewport offset encoding `{}`",
                    syntax.offset_encoding
                ));
            }
            if syntax.document_length != length {
                return Err("Editor viewport syntax length does not match its value".into());
            }
            if syntax
                .ranges
                .iter()
                .any(|range| range.from > range.to || range.to > length)
            {
                return Err("Editor viewport highlight range lies outside the document".into());
            }
            if syntax
                .ranges
                .windows(2)
                .any(|pair| pair[0].to > pair[1].from)
            {
                return Err("Editor viewport highlight ranges overlap or are not sorted".into());
            }
        }
        self.selection = config.selection;
        self.composition = config.composition;
        self.highlight_ranges = config.syntax.map_or_else(Vec::new, |syntax| syntax.ranges);
        Ok(wabou_shell::WidgetChanges::REDRAW)
    }

    fn config_removed(&mut self) -> wabou_shell::WidgetChanges {
        self.highlight_ranges.clear();
        self.composition = None;
        wabou_shell::WidgetChanges::REDRAW
    }
    fn style_changed(&mut self, style: &WidgetStyle) -> wabou_shell::WidgetChanges {
        self.text_color = style.color;
        self.font_italic = style.font_italic;
        wabou_shell::WidgetChanges::REDRAW
    }
    fn text_selection(&self) -> Option<WidgetTextSelection> {
        Some(WidgetTextSelection {
            anchor: self.selection.anchor,
            head: self.selection.head,
            text: self.selected_text(),
            kind: self.selection_kind,
        })
    }
    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        wabou_shell::WidgetAccessibility {
            role: Some(wabou_shell::SemanticRole::TextInput),
            value: Some(self.value.clone()),
            disabled: Some(self.disabled),
            ..Default::default()
        }
    }
    fn current_value(&self) -> Option<&str> {
        Some(&self.value)
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
        if !focused {
            self.composition = None;
        }
        wabou_shell::WidgetChanges::REDRAW
    }
    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        self.focused.then(|| self.ime_caret_area()).flatten()
    }
    fn ime_state(&self) -> Option<WidgetImeState> {
        if !self.focused || self.disabled {
            return None;
        }
        let from = self.selection.anchor.min(self.selection.head);
        let to = self.selection.anchor.max(self.selection.head);
        let cursor = utf16_offset_to_byte(&self.value, self.selection.head)?;
        let anchor = utf16_offset_to_byte(&self.value, self.selection.anchor)?;
        let (surrounding_text, surrounding_cursor, surrounding_anchor) =
            surrounding_excerpt(&self.value, cursor, anchor);

        let composition_length = self
            .composition
            .as_ref()
            .map_or(0, |composition| composition.text.encode_utf16().count());
        let composition_cursor = self
            .composition_cursor_utf16()
            .unwrap_or(composition_length);
        let selection = if self.composition.is_some() {
            from + composition_cursor..from + composition_cursor
        } else {
            from..to
        };
        Some(WidgetImeState {
            cursor_area: self.ime_caret_area()?,
            surrounding_text,
            surrounding_cursor,
            surrounding_anchor,
            selection_utf16: selection,
            selection_reversed: self.composition.is_none()
                && self.selection.head < self.selection.anchor,
            marked_range_utf16: self
                .composition
                .as_ref()
                .map(|_| from..from + composition_length),
            purpose: WidgetImePurpose::Normal,
            multiline: true,
            completion: false,
            spellcheck: false,
        })
    }
    fn ime_text_for_range(&self, range_utf16: std::ops::Range<usize>) -> Option<String> {
        let start = utf16_offset_to_byte(&self.value, range_utf16.start)?;
        let end = utf16_offset_to_byte(&self.value, range_utf16.end)?;
        (start <= end).then(|| self.value[start..end].to_owned())
    }
    fn ime_bounds_for_range(&self, range_utf16: std::ops::Range<usize>) -> Option<[f32; 4]> {
        if range_utf16.start > range_utf16.end || range_utf16.end > self.document_len() {
            return None;
        }
        let lines = self.visual_lines();
        let mut bounds: Option<[f32; 4]> = None;
        for (row, line) in lines.iter().enumerate() {
            let from = range_utf16.start.max(line.start);
            let to = range_utf16.end.min(line.end);
            if from > to || (from == to && range_utf16.start != range_utf16.end) {
                continue;
            }
            let x0 = self.geometry.x_for_cell(line.cell_for_offset(from));
            let x1 = self
                .geometry
                .x_for_cell(line.cell_for_offset(to))
                .max(x0 + 1.5);
            let y0 = (row as f32 - self.scroll_row as f32) * self.geometry.line_height;
            let current = [x0, y0, x1, y0 + self.geometry.line_height];
            bounds = Some(bounds.map_or(current, |old| {
                [
                    old[0].min(current[0]),
                    old[1].min(current[1]),
                    old[2].max(current[2]),
                    old[3].max(current[3]),
                ]
            }));
            if range_utf16.start == range_utf16.end {
                break;
            }
        }
        bounds
    }
    fn ime_character_index_for_point(&self, point: wabou_shell::Point) -> Option<usize> {
        Some(self.offset_from_pointer(point.x as f32, point.y as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{GesturePhase, Modifiers, Point, PointerButton, PointerEvent, WheelEvent};

    fn pointer(phase: PointerPhase, x: f64, y: f64, buttons: u32) -> UiEvent {
        UiEvent::Pointer(PointerEvent {
            phase,
            position: Point { x, y },
            button: Some(PointerButton::Primary),
            buttons,
            modifiers: Modifiers::default(),
            properties: Default::default(),
        })
    }

    fn configure(editor: &mut EditorViewport, anchor: usize, head: usize) {
        editor
            .config_changed(&format!(
                r#"{{"selection":{{"anchor":{anchor},"head":{head}}},"composition":null,"syntax":null}}"#
            ))
            .unwrap();
    }

    #[test]
    fn native_text_events_do_not_mutate_the_controlled_document() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "{\n  \"enabled\": true\n}");
        configure(&mut editor, 0, 0);

        assert!(
            !editor
                .handle_event(&UiEvent::TextInput(" ".into()))
                .is_handled()
        );
        assert!(
            !editor
                .handle_event(&UiEvent::Paste("// edited".into()))
                .is_handled()
        );
        assert!(
            !editor
                .handle_event(&UiEvent::Ime(wabou_shell::ImeEvent::Commit("中文".into())))
                .is_handled()
        );
        assert_eq!(editor.current_value(), Some("{\n  \"enabled\": true\n}"));
    }

    #[test]
    fn native_pointer_reports_selection_without_editing_document() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "one\ntwo");
        editor.layout_changed(WidgetGeometry {
            content_size: [640.0, 420.0],
            ..Default::default()
        });
        let result = editor.handle_event(&pointer(PointerPhase::Down, 80.0, 4.0, 1));
        assert!(result.selection_changed());
        assert_eq!(editor.value, "one\ntwo");
        assert!(editor.text_selection().is_some());
    }

    #[test]
    fn clipboard_uses_controlled_utf16_selection() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "A😀B");
        configure(&mut editor, 1, 3);
        assert_eq!(editor.selected_text().as_deref(), Some("😀"));
    }

    #[test]
    fn short_documents_do_not_scroll() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "one\ntwo");
        editor.layout_changed(WidgetGeometry {
            content_size: [640.0, 420.0],
            ..Default::default()
        });
        let result = editor.handle_event(&UiEvent::Wheel(WheelEvent {
            position: Point { x: 100.0, y: 100.0 },
            delta_x: 0.0,
            delta_y: 220.0,
            delta_mode: wabou_shell::WheelDeltaMode::Pixel,
            phase: GesturePhase::Changed,
            modifiers: Modifiers::default(),
        }));
        assert!(!result.is_handled());
        assert_eq!(editor.scroll_row, 0);
    }

    #[test]
    fn rejects_syntax_for_a_different_document() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "true");
        let error = editor.config_changed(r#"{"selection":{"anchor":0,"head":0},"composition":null,"syntax":{"language":"json","offsetEncoding":"utf16","documentLength":5,"ranges":[]}}"#).unwrap_err();
        assert!(error.contains("length"));
    }

    #[test]
    fn accepts_semantic_diff_highlights() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "@@ -1 +1 @@\n-old\n+new");
        let changes = editor
            .config_changed(
                r#"{"selection":{"anchor":0,"head":0},"composition":null,"syntax":{"language":"diff","offsetEncoding":"utf16","documentLength":21,"ranges":[{"from":0,"to":11,"kind":"meta"},{"from":12,"to":16,"kind":"removed"},{"from":17,"to":21,"kind":"added"}]}}"#,
            )
            .expect("valid diff syntax config");

        assert!(changes.contains(wabou_shell::WidgetChanges::REDRAW));
        assert_eq!(editor.highlight_ranges.len(), 3);
        assert_eq!(
            editor.color_at_offset(17),
            Color::from_rgb8(0xa6, 0xe3, 0xa1)
        );
        editor.text_color = Color::from_rgb8(0x1f, 0x23, 0x2b);
        assert_eq!(
            editor.color_at_offset(17),
            Color::from_rgb8(0x11, 0x63, 0x29)
        );
    }

    #[test]
    fn ime_snapshot_preserves_utf8_surrounding_selection_and_tracks_preedit() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "A😀日本B");
        editor.layout_changed(WidgetGeometry {
            content_size: [640.0, 420.0],
            ..Default::default()
        });
        configure(&mut editor, 3, 5);
        editor.focus_changed(true);

        let selected = editor.ime_state().expect("selected IME state");
        assert_eq!(selected.surrounding_text, "A😀日本B");
        assert_eq!(selected.surrounding_anchor, "A😀".len());
        assert_eq!(selected.surrounding_cursor, "A😀日本".len());
        assert_eq!(selected.selection_utf16, 3..5);
        assert!(!selected.selection_reversed);
        assert_eq!(selected.marked_range_utf16, None);
        assert!(selected.cursor_area[2] - selected.cursor_area[0] <= 2.0);

        editor.handle_event(&UiEvent::Ime(wabou_shell::ImeEvent::Preedit {
            text: "かな".into(),
            cursor: Some((6, 6)),
        }));
        let composing = editor.ime_state().expect("composing IME state");
        assert_eq!(composing.surrounding_text, "A😀日本B");
        assert_eq!(composing.marked_range_utf16, Some(3..5));
        assert_eq!(composing.selection_utf16, 5..5);
        assert!(composing.cursor_area[0] > selected.cursor_area[0]);
    }

    #[test]
    fn empty_preedit_and_focus_loss_clear_editor_composition() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "text");
        configure(&mut editor, 4, 4);
        editor.focus_changed(true);
        editor.handle_event(&UiEvent::Ime(wabou_shell::ImeEvent::Preedit {
            text: "draft".into(),
            cursor: Some((5, 5)),
        }));
        assert!(editor.composition.is_some());

        editor.handle_event(&UiEvent::Ime(wabou_shell::ImeEvent::Preedit {
            text: String::new(),
            cursor: None,
        }));
        assert!(editor.composition.is_none());

        editor.handle_event(&UiEvent::Ime(wabou_shell::ImeEvent::Preedit {
            text: "again".into(),
            cursor: Some((5, 5)),
        }));
        editor.focus_changed(false);
        assert!(editor.composition.is_none());
        assert!(editor.ime_state().is_none());
    }

    #[test]
    fn ime_range_queries_use_utf16_and_scroll_relative_geometry() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "first\nA😀B\nlast");
        editor.layout_changed(WidgetGeometry {
            content_size: [640.0, LINE_HEIGHT],
            ..Default::default()
        });
        editor.scroll_row = 1;

        assert_eq!(editor.ime_text_for_range(7..9).as_deref(), Some("😀"));
        assert_eq!(editor.ime_text_for_range(8..9), None);
        let bounds = editor
            .ime_bounds_for_range(7..9)
            .expect("visible emoji bounds");
        assert_eq!(bounds[1], 0.0);
        assert_eq!(bounds[3], LINE_HEIGHT);
        let offset = editor
            .ime_character_index_for_point(wabou_shell::Point {
                x: f64::from((bounds[0] + bounds[2]) * 0.5),
                y: f64::from(LINE_HEIGHT * 0.5),
            })
            .expect("point offset");
        assert!((7..=9).contains(&offset));
    }

    #[test]
    fn ime_delete_surrounding_is_forwarded_without_mutating_the_viewport() {
        let mut editor = EditorViewport::new();
        editor.attribute_changed("value", "前A😀選択B後");
        configure(&mut editor, 4, 6);

        let result = editor.handle_event(&UiEvent::Ime(wabou_shell::ImeEvent::DeleteSurrounding {
            before_bytes: "A😀".len(),
            after_bytes: "B".len(),
        }));
        assert!(!result.is_handled());
        assert_eq!(editor.current_value(), Some("前A😀選択B後"));
        assert_eq!(editor.selection.anchor, 4);
        assert_eq!(editor.selection.head, 6);
    }
}
