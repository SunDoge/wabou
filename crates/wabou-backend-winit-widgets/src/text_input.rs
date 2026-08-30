//! TextInput and TextArea widgets backed by `parley::PlainEditor`.
//!
//! PlainEditor handles text editing, caret geometry, selection, and
//! hit-testing. `handle_event` stores pending edits (the editor needs
//! FontContext/LayoutContext which aren't available in handle_event);
//! `paint` applies them via the driver, refreshes layout, then paints
//! selection rects + caret + glyph runs.

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyrender::{PaintScene, Scene};
use parley::{PlainEditor, PositionedLayoutItem};
use serde::Deserialize;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};
use wabou_shell::style::TextAlign;
use wabou_shell::text::{
    SingleLineTextMetrics, TextContext, brush_for_color, layout_text_styled,
    single_line_text_metrics,
};
use wabou_shell::{ImeEvent, KeyEvent, KeyPhase, PointerEvent, PointerPhase, UiEvent};

use wabou_shell::{
    PaintContext, Widget, WidgetChanges, WidgetEventResult, WidgetStyle, WidgetTextSelection,
    WidgetTextSelectionKind,
};

const SELECTION_COLOR: Color = Color::from_rgba8(99, 102, 241, 80);
const PLACEHOLDER_COLOR: Color = Color::from_rgb8(0x64, 0x74, 0x8b);
const CONTENT_CHANGED: WidgetChanges = WidgetChanges::REDRAW.union(WidgetChanges::SEMANTICS);

/// A pending edit to apply on the next `paint` (when FontContext is available).
enum PendingEdit {
    Insert(String),
    SetCompose(String, Option<(usize, usize)>),
    CommitCompose(String),
    ClearCompose,
    DeleteSurrounding(usize, usize),
    Delete,
    DeleteForward,
    MoveLeft,
    MoveRight,
    MoveWordLeft,
    MoveWordRight,
    SelectLeft,
    SelectRight,
    SelectWordLeft,
    SelectWordRight,
    MoveUp,
    MoveDown,
    SelectUp,
    SelectDown,
    MoveToStart,
    MoveToEnd,
    SelectToStart,
    SelectToEnd,
    MoveToPoint(f32, f32),
    ExtendToPoint(f32, f32),
    SelectWordAtPoint(f32, f32),
    SelectLineAtPoint(f32, f32),
    SelectAll,
    SelectByteRange(usize, usize),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TextInputConfig {
    selection: TextInputSelection,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextInputSelection {
    anchor: usize,
    head: usize,
}

fn utf16_offset_to_byte(text: &str, offset: usize) -> Option<usize> {
    if offset == 0 {
        return Some(0);
    }
    let mut utf16 = 0;
    for (byte, character) in text.char_indices() {
        utf16 += character.len_utf16();
        if utf16 == offset {
            return Some(byte + character.len_utf8());
        }
        if utf16 > offset {
            return None;
        }
    }
    (utf16 == offset).then_some(text.len())
}

/// Native single- or multiline plain-text editor backed by Parley.
pub struct TextInput {
    editor: PlainEditor<[u8; 4]>,
    placeholder: String,
    font_size: f32,
    font_weight: f32,
    font_italic: bool,
    line_height: Option<(f32, bool)>,
    font_family: Option<Arc<str>>,
    text_color: Color,
    focused: bool,
    blink_on: bool,
    next_blink: Option<Instant>,
    pending: Vec<PendingEdit>,
    needs_refresh: bool,
    /// Cached value string (updated in paint after edits) for current_value().
    cached_value: String,
    selecting: bool,
    last_click: Option<(Instant, f32, f32, u8)>,
    multiline: bool,
    viewport_width: f32,
    viewport_height: f32,
    scroll_x: f32,
    scroll_y: f32,
    disabled: bool,
    read_only: bool,
    text_metrics: Option<SingleLineTextMetrics>,
    single_line_y_offset: f32,
    selection_kind: WidgetTextSelectionKind,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

fn snap_metrics_baseline(
    mut metrics: SingleLineTextMetrics,
    device_scale: f64,
) -> SingleLineTextMetrics {
    let scale = device_scale.max(f64::EPSILON) as f32;
    let snapped_baseline = (metrics.baseline * scale).round() / scale;
    metrics.line_box[1] += snapped_baseline - metrics.baseline;
    metrics.baseline = snapped_baseline;
    metrics
}

impl TextInput {
    /// Construct an empty single-line text input.
    pub fn new() -> Self {
        Self::with_multiline(false)
    }

    /// Construct an empty multiline textarea.
    pub fn multiline() -> Self {
        Self::with_multiline(true)
    }

    fn with_multiline(multiline: bool) -> Self {
        let mut editor = PlainEditor::new(16.0);
        editor
            .edit_styles()
            .insert(parley::StyleProperty::LineHeight(
                parley::LineHeight::Absolute(16.0 * 1.2),
            ));
        Self {
            editor,
            placeholder: String::new(),
            font_size: 16.0,
            font_weight: 400.0,
            font_italic: false,
            line_height: None,
            font_family: None,
            text_color: Color::from_rgb8(0xe2, 0xe8, 0xf0),
            focused: false,
            blink_on: true,
            next_blink: None,
            pending: Vec::new(),
            // PlainEditor does not have a shaped layout until its driver is
            // refreshed. The empty editor still defines the canonical input
            // baseline used by the placeholder, so initialize it on the first
            // paint even when no authored style differs from the defaults.
            needs_refresh: true,
            cached_value: String::new(),
            selecting: false,
            last_click: None,
            multiline,
            viewport_width: 0.0,
            viewport_height: 0.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
            disabled: false,
            read_only: false,
            text_metrics: None,
            single_line_y_offset: 0.0,
            selection_kind: WidgetTextSelectionKind::Simple,
        }
    }

    fn queue(&mut self, edit: PendingEdit) {
        self.pending.push(edit);
        self.blink_on = true;
        self.next_blink = Some(Instant::now() + Duration::from_millis(500));
    }

    fn editable(&self) -> bool {
        !self.disabled && !self.read_only
    }

    fn caret_color(&self) -> Color {
        // The computed foreground already follows the active JS theme. Using
        // it keeps the caret visible on both light and dark input surfaces.
        self.text_color
    }

    fn used_line_height(&self) -> (f32, bool) {
        self.line_height.unwrap_or((self.font_size * 1.2, false))
    }

    fn sync_editor_line_height(&mut self) {
        let (value, relative) = self.used_line_height();
        self.editor
            .edit_styles()
            .insert(parley::StyleProperty::LineHeight(if relative {
                parley::LineHeight::FontSizeRelative(value)
            } else {
                parley::LineHeight::Absolute(value)
            }));
        self.needs_refresh = true;
    }

    fn local_point(&self, x: f64, y: f64) -> (f32, f32) {
        let mut local_x = x as f32;
        let mut local_y = y as f32;
        if self.multiline {
            local_y += self.scroll_y;
        } else {
            local_x += self.scroll_x;
            local_y -= self.single_line_y_offset;
        }
        (local_x, local_y)
    }

    fn clamp_scroll(&mut self) {
        let Some(layout) = self.editor.try_layout() else {
            self.scroll_x = 0.0;
            self.scroll_y = 0.0;
            return;
        };
        if self.multiline {
            self.scroll_x = 0.0;
            // Keep one line of virtual space below an active composition.
            // IMEs commonly replace a long phonetic preedit with wider CJK
            // glyphs. Without this reserve, crossing a wrap boundary moves
            // the viewport by a whole line while the user selects a candidate.
            let composition_reserve = if self.editor.is_composing() {
                self.used_line_height().0
            } else {
                0.0
            };
            self.scroll_y = self.scroll_y.clamp(
                0.0,
                (layout.height() + composition_reserve - self.viewport_height).max(0.0),
            );
        } else {
            self.scroll_y = 0.0;
            self.scroll_x = self
                .scroll_x
                .clamp(0.0, (layout.full_width() - self.viewport_width).max(0.0));
        }
    }

    fn reveal_caret_with_composition_reserve(&mut self, reserve_composition_line: bool) {
        if let Some(caret) = self.editor.cursor_geometry(1.5) {
            if self.multiline {
                if caret.y0 < f64::from(self.scroll_y) {
                    self.scroll_y = caret.y0 as f32;
                } else {
                    let composition_reserve =
                        if reserve_composition_line && self.editor.is_composing() {
                            self.used_line_height().0
                        } else {
                            0.0
                        };
                    if caret.y1 + f64::from(composition_reserve)
                        > f64::from(self.scroll_y + self.viewport_height)
                    {
                        self.scroll_y =
                            (caret.y1 as f32 + composition_reserve - self.viewport_height).max(0.0);
                    }
                }
            } else if caret.x0 < f64::from(self.scroll_x) {
                self.scroll_x = caret.x0 as f32;
            } else if caret.x1 > f64::from(self.scroll_x + self.viewport_width) {
                self.scroll_x = (caret.x1 as f32 - self.viewport_width).max(0.0);
            }
        }
        self.clamp_scroll();
    }

    fn reveal_caret(&mut self) {
        self.reveal_caret_with_composition_reserve(true);
    }

    fn handle_pointer(&mut self, event: &PointerEvent) -> WidgetEventResult {
        match event.phase {
            PointerPhase::Down if event.button == Some(wabou_shell::PointerButton::Primary) => {
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                if event.modifiers.shift() {
                    self.queue(PendingEdit::ExtendToPoint(local_x, local_y));
                } else {
                    let now = Instant::now();
                    let clicks = self.last_click.map_or(1, |(time, x, y, count)| {
                        if now.duration_since(time) <= Duration::from_millis(400)
                            && (local_x - x).abs() <= 4.0
                            && (local_y - y).abs() <= 4.0
                        {
                            count % 3 + 1
                        } else {
                            1
                        }
                    });
                    self.last_click = Some((now, local_x, local_y, clicks));
                    self.selection_kind = match clicks {
                        2 => WidgetTextSelectionKind::Word,
                        3 => WidgetTextSelectionKind::Line,
                        _ => WidgetTextSelectionKind::Simple,
                    };
                    self.queue(match clicks {
                        2 => PendingEdit::SelectWordAtPoint(local_x, local_y),
                        3 => PendingEdit::SelectLineAtPoint(local_x, local_y),
                        _ => PendingEdit::MoveToPoint(local_x, local_y),
                    });
                }
                self.selecting = true;
                WidgetEventResult::selection_changed_result()
            }
            PointerPhase::Move if self.selecting => {
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                self.queue(PendingEdit::ExtendToPoint(local_x, local_y));
                WidgetEventResult::selection_changed_result()
            }
            PointerPhase::Up if self.selecting => {
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                self.queue(PendingEdit::ExtendToPoint(local_x, local_y));
                self.selecting = false;
                WidgetEventResult::selection_changed_result()
            }
            PointerPhase::Cancel if self.selecting => {
                self.selecting = false;
                WidgetEventResult::HANDLED
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn horizontal_edit(right: bool, extend: bool, by_word: bool) -> PendingEdit {
        match (right, extend, by_word) {
            (false, false, false) => PendingEdit::MoveLeft,
            (false, false, true) => PendingEdit::MoveWordLeft,
            (false, true, false) => PendingEdit::SelectLeft,
            (false, true, true) => PendingEdit::SelectWordLeft,
            (true, false, false) => PendingEdit::MoveRight,
            (true, false, true) => PendingEdit::MoveWordRight,
            (true, true, false) => PendingEdit::SelectRight,
            (true, true, true) => PendingEdit::SelectWordRight,
        }
    }

    fn handle_key(&mut self, event: &KeyEvent) -> WidgetEventResult {
        self.selection_kind = WidgetTextSelectionKind::Simple;
        match event.key.as_str() {
            _ if event.matches_standard_shortcut(wabou_shell::StandardShortcut::Copy) => self
                .editor
                .selected_text()
                .map(str::to_owned)
                .map_or(WidgetEventResult::IGNORED, WidgetEventResult::copy),
            _ if event.matches_standard_shortcut(wabou_shell::StandardShortcut::Cut) => {
                if self.editable()
                    && let Some(text) = self.editor.selected_text().map(str::to_owned)
                {
                    self.queue(PendingEdit::Delete);
                    WidgetEventResult::copy_with_value_change(text).with_selection_changed()
                } else {
                    WidgetEventResult::IGNORED
                }
            }
            _ if event.matches_standard_shortcut(wabou_shell::StandardShortcut::Paste) => {
                if self.editable() {
                    WidgetEventResult::paste()
                } else {
                    WidgetEventResult::IGNORED
                }
            }
            "Backspace" if self.editable() => {
                self.queue(PendingEdit::Delete);
                WidgetEventResult::VALUE_CHANGED.with_selection_changed()
            }
            "Delete" if self.editable() => {
                self.queue(PendingEdit::DeleteForward);
                WidgetEventResult::VALUE_CHANGED.with_selection_changed()
            }
            "ArrowLeft" => {
                self.queue(Self::horizontal_edit(
                    false,
                    event.modifiers.shift(),
                    event.modifiers.control() || event.modifiers.alt(),
                ));
                WidgetEventResult::selection_changed_result()
            }
            "ArrowRight" => {
                self.queue(Self::horizontal_edit(
                    true,
                    event.modifiers.shift(),
                    event.modifiers.control() || event.modifiers.alt(),
                ));
                WidgetEventResult::selection_changed_result()
            }
            "ArrowUp" if self.multiline => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectUp
                } else {
                    PendingEdit::MoveUp
                });
                WidgetEventResult::selection_changed_result()
            }
            "ArrowDown" if self.multiline => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectDown
                } else {
                    PendingEdit::MoveDown
                });
                WidgetEventResult::selection_changed_result()
            }
            "Enter" if self.multiline && self.editable() => {
                self.queue(PendingEdit::Insert("\n".into()));
                WidgetEventResult::value_changed_consuming_key_text().with_selection_changed()
            }
            "Home" => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectToStart
                } else {
                    PendingEdit::MoveToStart
                });
                WidgetEventResult::selection_changed_result()
            }
            "End" => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectToEnd
                } else {
                    PendingEdit::MoveToEnd
                });
                WidgetEventResult::selection_changed_result()
            }
            _ if event.matches_standard_shortcut(wabou_shell::StandardShortcut::SelectAll) => {
                self.queue(PendingEdit::SelectAll);
                WidgetEventResult::selection_changed_result()
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn refresh_editor(&mut self, tcx: &mut TextContext) {
        if self.pending.is_empty() && !self.needs_refresh {
            self.clamp_scroll();
            return;
        }
        let multiline = self.multiline;
        let was_composing = self.editor.is_composing();
        let scroll_before_edit = self.scroll_y;
        {
            let mut driver = self.editor.driver(&mut tcx.font_cx, &mut tcx.layout_cx);
            for edit in self.pending.drain(..) {
                match edit {
                    PendingEdit::Insert(text) => driver.insert_or_replace_selection(&text),
                    PendingEdit::SetCompose(text, cursor) => driver.set_compose(&text, cursor),
                    PendingEdit::CommitCompose(text) => {
                        driver.clear_compose();
                        driver.insert_or_replace_selection(&text);
                    }
                    PendingEdit::ClearCompose => driver.clear_compose(),
                    PendingEdit::DeleteSurrounding(before, after) => {
                        driver.clear_compose();
                        if let Some(before) = NonZeroUsize::new(before) {
                            driver.delete_bytes_before_selection(before);
                        }
                        if let Some(after) = NonZeroUsize::new(after) {
                            driver.delete_bytes_after_selection(after);
                        }
                    }
                    PendingEdit::Delete => driver.backdelete(),
                    PendingEdit::DeleteForward => driver.delete(),
                    PendingEdit::MoveLeft => driver.move_left(),
                    PendingEdit::MoveRight => driver.move_right(),
                    PendingEdit::MoveWordLeft => driver.move_word_left(),
                    PendingEdit::MoveWordRight => driver.move_word_right(),
                    PendingEdit::SelectLeft => driver.select_left(),
                    PendingEdit::SelectRight => driver.select_right(),
                    PendingEdit::SelectWordLeft => driver.select_word_left(),
                    PendingEdit::SelectWordRight => driver.select_word_right(),
                    PendingEdit::MoveUp => driver.move_up(),
                    PendingEdit::MoveDown => driver.move_down(),
                    PendingEdit::SelectUp => driver.select_up(),
                    PendingEdit::SelectDown => driver.select_down(),
                    PendingEdit::MoveToStart if multiline => driver.move_to_line_start(),
                    PendingEdit::MoveToStart => driver.move_to_text_start(),
                    PendingEdit::MoveToEnd if multiline => driver.move_to_line_end(),
                    PendingEdit::MoveToEnd => driver.move_to_text_end(),
                    PendingEdit::SelectToStart if multiline => driver.select_to_line_start(),
                    PendingEdit::SelectToStart => driver.select_to_text_start(),
                    PendingEdit::SelectToEnd if multiline => driver.select_to_line_end(),
                    PendingEdit::SelectToEnd => driver.select_to_text_end(),
                    PendingEdit::MoveToPoint(x, y) => driver.move_to_point(x, y),
                    PendingEdit::ExtendToPoint(x, y) => driver.extend_selection_to_point(x, y),
                    PendingEdit::SelectWordAtPoint(x, y) => driver.select_word_at_point(x, y),
                    PendingEdit::SelectLineAtPoint(x, y) => driver.select_line_at_point(x, y),
                    PendingEdit::SelectAll => driver.select_all(),
                    PendingEdit::SelectByteRange(anchor, head) if anchor == head => {
                        driver.move_to_byte(anchor);
                    }
                    PendingEdit::SelectByteRange(anchor, head) => {
                        driver.select_byte_range(anchor, head);
                    }
                }
            }
            driver.refresh_layout();
        }
        self.needs_refresh = false;
        if multiline && was_composing && self.editor.is_composing() {
            // Candidate changes can reshape a phonetic preedit into wider CJK
            // glyphs. Keep the viewport anchored across those transient
            // updates, moving it only if the resulting caret is actually out
            // of view.
            self.scroll_y = scroll_before_edit;
            self.reveal_caret_with_composition_reserve(false);
        } else {
            self.reveal_caret();
        }
    }

    fn sync_cached_value(&mut self) {
        if self.editor.text() == self.cached_value.as_str() {
            return;
        }
        self.cached_value.clear();
        for chunk in self.editor.text() {
            self.cached_value.push_str(chunk);
        }
    }

    fn paint_placeholder(
        &self,
        cx: &mut PaintContext<'_>,
        width: f32,
        canonical_metrics: Option<SingleLineTextMetrics>,
    ) -> Option<SingleLineTextMetrics> {
        if !self.cached_value.is_empty()
            || self.editor.is_composing()
            || self.placeholder.is_empty()
        {
            return None;
        }
        let layout = layout_text_styled(
            cx.text(),
            Arc::from(self.placeholder.as_str()),
            self.font_size,
            self.font_weight,
            self.font_italic,
            Some(self.used_line_height()),
            TextAlign::Start,
            brush_for_color(PLACEHOLDER_COLOR),
            Arc::from([]),
            self.font_family.as_ref(),
            self.multiline.then_some(width.max(0.0)),
        );
        // The placeholder uses a separate layout from PlainEditor. Its line
        // metrics can differ fractionally even with identical authored styles,
        // which made the first typed glyph jump when the placeholder vanished.
        // Use the primary-font strut as the canonical input baseline and
        // retain only the placeholder's measured width.
        let metrics = if self.multiline {
            None
        } else {
            canonical_metrics.map(|mut metrics| {
                metrics.line_box[2] = layout.width();
                metrics
            })
        };
        let y_offset = metrics
            .zip(layout.lines().next())
            .map_or(0.0, |(metrics, line)| {
                f64::from(metrics.baseline - line.metrics().baseline)
            });
        cx.draw_text_layout(&layout, [0.0, y_offset]);
        metrics
    }

    fn paint_editor(
        &mut self,
        scene: &mut Scene,
        device_scale: f64,
        canonical_metrics: Option<SingleLineTextMetrics>,
    ) -> Option<SingleLineTextMetrics> {
        let layout = self.editor.try_layout()?;
        let metrics = canonical_metrics.map(|mut metrics| {
            metrics.line_box[2] = layout.width();
            metrics
        });
        self.single_line_y_offset = metrics
            .zip(layout.lines().next())
            .map_or(0.0, |(metrics, line)| {
                metrics.baseline - line.metrics().baseline
            });
        let transform = Affine::translate((
            if self.multiline {
                0.0
            } else {
                -f64::from(self.scroll_x)
            },
            if self.multiline {
                -f64::from(self.scroll_y)
            } else {
                f64::from(self.single_line_y_offset)
            },
        ));
        if self.focused {
            for (bounds, _) in self.editor.selection_geometry() {
                scene.fill(
                    Fill::NonZero,
                    transform,
                    SELECTION_COLOR,
                    None,
                    &Rect::new(bounds.x0, bounds.y0, bounds.x1, bounds.y1),
                );
            }
        }
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                if glyph_run.positioned_glyphs().next().is_some() {
                    scene.draw_glyphs(
                        glyph_run.run().font(),
                        glyph_run.run().font_size() * device_scale as f32,
                        true,
                        glyph_run.run().normalized_coords(),
                        vello::kurbo::Vec2::ZERO,
                        Fill::NonZero,
                        self.text_color,
                        1.0,
                        transform * Affine::scale(device_scale.recip()),
                        None,
                        glyph_run.positioned_glyphs().map(|glyph| anyrender::Glyph {
                            id: glyph.id,
                            x: glyph.x * device_scale as f32,
                            y: glyph.y * device_scale as f32,
                        }),
                    );
                }
            }
        }
        if self.focused
            && self.blink_on
            && let Some(cursor) = self.editor.cursor_geometry(1.5)
        {
            scene.fill(
                Fill::NonZero,
                transform,
                self.caret_color(),
                None,
                &Rect::new(cursor.x0, cursor.y0, cursor.x1, cursor.y1),
            );
        }
        metrics
    }
}

impl Widget for TextInput {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let [width, height] = cx.size();
        let device_scale = cx.device_scale();
        if self.multiline && self.viewport_width != width {
            self.viewport_width = width.max(0.0);
            self.editor.set_width(Some(width.max(0.0)));
            self.needs_refresh = true;
        } else {
            self.viewport_width = width.max(0.0);
        }
        self.viewport_height = height.max(0.0);

        let canonical_metrics = if self.multiline {
            None
        } else {
            let strut = layout_text_styled(
                cx.text(),
                Arc::from(""),
                self.font_size,
                self.font_weight,
                self.font_italic,
                Some(self.used_line_height()),
                TextAlign::Start,
                brush_for_color(self.text_color),
                Arc::from([]),
                self.font_family.as_ref(),
                None,
            );
            single_line_text_metrics(&strut, height)
                .map(|metrics| snap_metrics_baseline(metrics, device_scale))
        };

        // Blink.
        if self.focused && self.next_blink.is_some_and(|d| Instant::now() >= d) {
            self.blink_on = !self.blink_on;
            self.next_blink = Some(Instant::now() + Duration::from_millis(500));
        }

        self.refresh_editor(cx.text());

        // Cache value for current_value().
        self.sync_cached_value();

        let mut scene = Scene::new();
        let placeholder_metrics = self.paint_placeholder(cx, width, canonical_metrics);
        let editor_metrics = self.paint_editor(&mut scene, device_scale, canonical_metrics);
        self.text_metrics = placeholder_metrics.or(editor_metrics);
        cx.scene_mut().append_scene(scene, Affine::IDENTITY);
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if self.disabled {
            return WidgetEventResult::IGNORED;
        }
        match event {
            UiEvent::Pointer(event) => self.handle_pointer(event),
            UiEvent::Wheel(event) => {
                let previous = if self.multiline {
                    self.scroll_y
                } else {
                    self.scroll_x
                };
                if self.multiline {
                    self.scroll_y += event.delta_y as f32;
                } else {
                    self.scroll_x += event.delta_x as f32;
                }
                self.clamp_scroll();
                let current = if self.multiline {
                    self.scroll_y
                } else {
                    self.scroll_x
                };
                if current != previous {
                    WidgetEventResult::HANDLED
                } else {
                    WidgetEventResult::IGNORED
                }
            }
            UiEvent::TextInput(text) | UiEvent::Paste(text) if self.editable() => {
                let normalized;
                let text = if self.multiline {
                    normalized = text.replace("\r\n", "\n").replace('\r', "\n");
                    normalized.as_str()
                } else {
                    text.as_str()
                };
                let filtered: String = text
                    .chars()
                    .filter(|c| !c.is_control() || (self.multiline && matches!(c, '\n' | '\t')))
                    .collect();
                if filtered.is_empty() {
                    return WidgetEventResult::IGNORED;
                }
                self.queue(PendingEdit::Insert(filtered));
                self.selection_kind = WidgetTextSelectionKind::Simple;
                WidgetEventResult::VALUE_CHANGED.with_selection_changed()
            }
            UiEvent::Ime(ImeEvent::Preedit { text, cursor }) if self.editable() => {
                if text.is_empty() {
                    self.queue(PendingEdit::ClearCompose);
                } else {
                    self.queue(PendingEdit::SetCompose(text.clone(), *cursor));
                }
                WidgetEventResult::HANDLED
            }
            UiEvent::Ime(ImeEvent::Commit(text)) if self.editable() => {
                if text.is_empty() {
                    self.queue(PendingEdit::ClearCompose);
                    WidgetEventResult::HANDLED
                } else {
                    self.queue(PendingEdit::CommitCompose(text.clone()));
                    self.selection_kind = WidgetTextSelectionKind::Simple;
                    WidgetEventResult::VALUE_CHANGED.with_selection_changed()
                }
            }
            UiEvent::Ime(ImeEvent::DeleteSurrounding {
                before_bytes,
                after_bytes,
            }) if self.editable() => {
                self.queue(PendingEdit::DeleteSurrounding(*before_bytes, *after_bytes));
                self.selection_kind = WidgetTextSelectionKind::Simple;
                WidgetEventResult::VALUE_CHANGED.with_selection_changed()
            }
            UiEvent::Ime(ImeEvent::Disabled) => {
                self.queue(PendingEdit::ClearCompose);
                WidgetEventResult::HANDLED
            }
            UiEvent::Ime(ImeEvent::Enabled) => WidgetEventResult::HANDLED,
            UiEvent::Key(event) if event.phase == KeyPhase::Down => self.handle_key(event),
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn prepare_for_event(&mut self, text: &mut TextContext) -> WidgetChanges {
        if self.pending.is_empty() {
            return WidgetChanges::empty();
        }
        self.refresh_editor(text);
        self.sync_cached_value();
        CONTENT_CHANGED
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> WidgetChanges {
        match name {
            "value" => {
                // Controlled Solid values commonly echo the widget's own
                // input event. Preserve the live selection/caret for that
                // no-op; only external value changes replace the buffer.
                if self.editor.text() != value {
                    self.editor.set_text(value);
                    self.cached_value = value.to_owned();
                    self.needs_refresh = true;
                }
            }
            "placeholder" => {
                self.placeholder = value.to_string();
            }
            "font-size" => {
                if let Some(px) = parse_px(value) {
                    self.font_size = px;
                    self.editor
                        .edit_styles()
                        .insert(parley::StyleProperty::FontSize(px));
                    if self.line_height.is_none() {
                        self.sync_editor_line_height();
                    }
                    self.needs_refresh = true;
                }
            }
            "color" => {
                if let Some(c) = wabou_shell::style::parse_color(value) {
                    self.text_color = c;
                }
            }
            "disabled" => self.disabled = value != "false",
            "readOnly" => self.read_only = value != "false",
            _ => return WidgetChanges::empty(),
        }
        CONTENT_CHANGED
    }

    fn attribute_removed(&mut self, name: &str) -> WidgetChanges {
        match name {
            "disabled" => self.disabled = false,
            "readOnly" => self.read_only = false,
            "placeholder" => self.placeholder.clear(),
            _ => return WidgetChanges::empty(),
        }
        CONTENT_CHANGED
    }

    fn config_changed(&mut self, json: &str) -> Result<WidgetChanges, String> {
        let config: TextInputConfig = wabou_shell::decode_widget_config(json)?;
        let raw = self.editor.raw_text();
        let anchor = utf16_offset_to_byte(raw, config.selection.anchor)
            .ok_or_else(|| "TextInput selection anchor is not a UTF-16 boundary".to_owned())?;
        let head = utf16_offset_to_byte(raw, config.selection.head)
            .ok_or_else(|| "TextInput selection head is not a UTF-16 boundary".to_owned())?;
        let current = self.editor.raw_selection();
        if current.anchor().index() == anchor && current.focus().index() == head {
            return Ok(WidgetChanges::empty());
        }
        self.queue(PendingEdit::SelectByteRange(anchor, head));
        self.selection_kind = WidgetTextSelectionKind::Simple;
        Ok(CONTENT_CHANGED)
    }

    fn current_value(&self) -> Option<&str> {
        Some(&self.cached_value)
    }

    fn text_selection(&self) -> Option<WidgetTextSelection> {
        let raw = self.editor.raw_text();
        let selection = self.editor.raw_selection();
        let anchor_byte = selection.anchor().index().min(raw.len());
        let head_byte = selection.focus().index().min(raw.len());
        let range = selection.text_range();
        Some(WidgetTextSelection {
            anchor: raw[..anchor_byte].encode_utf16().count(),
            head: raw[..head_byte].encode_utf16().count(),
            text: (!selection.is_collapsed())
                .then(|| raw.get(range).map(str::to_owned))
                .flatten(),
            kind: self.selection_kind,
        })
    }

    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        wabou_shell::WidgetAccessibility {
            role: Some(wabou_shell::SemanticRole::TextInput),
            value: Some(self.cached_value.clone()),
            disabled: Some(self.disabled),
            ..Default::default()
        }
    }

    fn text_metrics(&self) -> Option<SingleLineTextMetrics> {
        self.text_metrics
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> WidgetChanges {
        self.text_color = style.color;
        let mut line_height_changed = false;
        if self.font_size != style.font_size {
            self.font_size = style.font_size;
            self.editor
                .edit_styles()
                .insert(parley::StyleProperty::FontSize(style.font_size));
            self.needs_refresh = true;
            line_height_changed = self.line_height.is_none();
        }
        if self.font_weight != style.font_weight {
            self.font_weight = style.font_weight;
            self.editor
                .edit_styles()
                .insert(parley::StyleProperty::FontWeight(parley::FontWeight::new(
                    style.font_weight,
                )));
            self.needs_refresh = true;
        }
        if self.font_italic != style.font_italic {
            self.font_italic = style.font_italic;
            self.editor
                .edit_styles()
                .insert(parley::StyleProperty::FontStyle(if style.font_italic {
                    parley::FontStyle::Italic
                } else {
                    parley::FontStyle::Normal
                }));
            self.needs_refresh = true;
        }
        if self.line_height != style.line_height {
            self.line_height = style.line_height;
            line_height_changed = true;
        }
        if line_height_changed {
            self.sync_editor_line_height();
        }
        if self.font_family != style.font_family {
            self.font_family = style.font_family.clone();
            if let Some(family) = self.font_family.as_ref() {
                self.editor
                    .edit_styles()
                    .insert(parley::StyleProperty::FontFamily(
                        parley::FontFamily::from(family.as_ref()).into_owned(),
                    ));
            } else {
                self.editor.edit_styles().remove(core::mem::discriminant(
                    &parley::StyleProperty::<[u8; 4]>::FontFamily(
                        parley::FontFamily::from("").into_owned(),
                    ),
                ));
            }
            self.needs_refresh = true;
        }
        WidgetChanges::REDRAW
    }

    fn accepts_focus(&self) -> bool {
        !self.disabled
    }

    fn accepts_text_input(&self) -> bool {
        !self.disabled && !self.read_only
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        if self.multiline {
            Some([240.0, 96.0])
        } else {
            Some([120.0, 32.0])
        }
    }

    fn focus_changed(&mut self, focused: bool) -> WidgetChanges {
        self.focused = focused;
        self.blink_on = true;
        self.next_blink = focused.then(|| Instant::now() + Duration::from_millis(500));
        WidgetChanges::REDRAW
    }

    fn animation_deadline(&self) -> Option<Instant> {
        self.next_blink
    }

    fn ime_cursor_area(&self) -> Option<[f32; 4]> {
        if !self.focused || self.disabled {
            return None;
        }
        let area = self.editor.ime_cursor_area();
        let caret = self.editor.cursor_geometry(1.5);
        let (x0, x1, y0, y1) = caret.map_or(
            (
                area.x1 as f32,
                area.x1 as f32 + 1.5,
                area.y0 as f32,
                area.y1 as f32,
            ),
            |caret| {
                (
                    caret.x0 as f32,
                    caret.x1 as f32,
                    caret.y0 as f32,
                    caret.y1 as f32,
                )
            },
        );
        let y_offset = if self.multiline {
            -self.scroll_y
        } else {
            self.single_line_y_offset
        };
        Some([
            x0 - self.scroll_x,
            y0 + y_offset,
            x1 - self.scroll_x,
            y1 + y_offset,
        ])
    }
}

fn parse_px(value: &str) -> Option<f32> {
    let v = value.trim();
    if let Some(r) = v.strip_suffix("rem") {
        return r.trim().parse::<f32>().ok().map(|n| n * 16.0);
    }
    if let Some(p) = v.strip_suffix("px") {
        return p.trim().parse::<f32>().ok();
    }
    v.parse::<f32>().ok()
}

#[cfg(test)]
impl TextInput {
    fn paint(&mut self, width: f32, height: f32, text: &mut TextContext) -> Scene {
        let mut cx = PaintContext::new(width, height, 1.0, text);
        <Self as Widget>::paint(self, &mut cx);
        cx.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{
        ClipboardRequest, KeyEvent, Modifiers, Point, PointerButton, PointerEvent, WheelEvent,
    };

    fn pointer(phase: PointerPhase, x: f64, buttons: u32) -> UiEvent {
        UiEvent::Pointer(PointerEvent {
            phase,
            position: Point { x, y: 10.0 },
            button: Some(PointerButton::Primary),
            buttons,
            modifiers: Modifiers::default(),
            properties: Default::default(),
        })
    }

    fn key(key: &str) -> UiEvent {
        UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: key.into(),
            key_without_modifiers: key.into(),
            code: key.into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Modifiers::default(),
            repeat: false,
            synthetic: false,
        })
    }

    #[test]
    fn pointer_drag_extends_editor_selection() {
        let mut input = TextInput::new();
        input.attribute_changed("value", "hello world");
        input.focus_changed(true);
        let mut tcx = TextContext::new();
        input.paint(200.0, 32.0, &mut tcx);

        input.handle_event(&pointer(PointerPhase::Down, 1.0, 1));
        input.handle_event(&pointer(PointerPhase::Move, 45.0, 1));
        input.handle_event(&pointer(PointerPhase::Up, 45.0, 0));
        input.paint(200.0, 32.0, &mut tcx);

        assert!(
            input
                .editor
                .selected_text()
                .is_some_and(|text| !text.is_empty())
        );
        assert!(!input.selecting);
        let selection = input.text_selection().expect("plain editor selection");
        assert!(selection.anchor != selection.head);
        assert!(selection.text.is_some_and(|text| !text.is_empty()));
    }

    #[test]
    fn text_selection_reports_javascript_utf16_offsets() {
        let mut input = TextInput::new();
        input.attribute_changed("value", "a😀b");
        input.focus_changed(true);
        let mut tcx = TextContext::new();
        input.paint(200.0, 32.0, &mut tcx);

        let result = input.handle_event(&key("End"));
        assert!(result.selection_changed());
        input.paint(200.0, 32.0, &mut tcx);
        let selection = input.text_selection().expect("caret selection");
        assert_eq!((selection.anchor, selection.head), (4, 4));
    }

    #[test]
    fn text_input_config_controls_utf16_selection_without_echo_work() {
        let mut input = TextInput::multiline();
        input.attribute_changed("value", "a😀bc");
        let mut tcx = TextContext::new();
        input.paint(200.0, 64.0, &mut tcx);

        let changes = input
            .config_changed(r#"{"selection":{"anchor":3,"head":4}}"#)
            .expect("valid UTF-16 selection");
        assert!(changes.contains(WidgetChanges::REDRAW));
        input.paint(200.0, 64.0, &mut tcx);

        let selection = input.text_selection().expect("selection");
        assert_eq!((selection.anchor, selection.head), (3, 4));
        assert_eq!(selection.text.as_deref(), Some("b"));
        assert!(
            input
                .config_changed(r#"{"selection":{"anchor":3,"head":4}}"#)
                .expect("selection echo")
                .is_empty()
        );
    }

    #[test]
    fn text_input_config_rejects_split_surrogates_and_out_of_range_offsets() {
        let mut input = TextInput::new();
        input.attribute_changed("value", "a😀b");

        assert_eq!(
            input
                .config_changed(r#"{"selection":{"anchor":2,"head":2}}"#)
                .unwrap_err(),
            "TextInput selection anchor is not a UTF-16 boundary"
        );
        assert_eq!(
            input
                .config_changed(r#"{"selection":{"anchor":5,"head":5}}"#)
                .unwrap_err(),
            "TextInput selection anchor is not a UTF-16 boundary"
        );
    }

    #[test]
    fn single_line_input_scrolls_caret_and_keeps_pointer_and_ime_coordinates_in_sync() {
        let mut input = TextInput::new();
        input.attribute_changed("value", "the quick brown fox jumps over the lazy dog");
        input.focus_changed(true);
        let mut tcx = TextContext::new();
        input.paint(48.0, 30.0, &mut tcx);
        input.queue(PendingEdit::MoveToEnd);
        input.paint(48.0, 30.0, &mut tcx);

        assert!(input.scroll_x > 0.0, "the end caret must remain visible");
        let (text_x, text_y) = input.local_point(4.0, 12.0);
        assert_eq!(text_x, 4.0 + input.scroll_x);
        assert_eq!(text_y, 12.0 - input.single_line_y_offset);

        let raw = input.editor.cursor_geometry(1.5).expect("caret");
        let ime = input.ime_cursor_area().expect("IME cursor area");
        assert_eq!(ime[0], raw.x0 as f32 - input.scroll_x);
        assert_eq!(ime[2], raw.x1 as f32 - input.scroll_x);

        input.queue(PendingEdit::MoveToStart);
        input.paint(48.0, 30.0, &mut tcx);
        assert_eq!(input.scroll_x, 0.0);
    }

    #[test]
    fn text_input_uses_the_computed_font_family() {
        let mut input = TextInput::new();
        let family: Arc<str> = Arc::from("Menlo, monospace");
        input.style_changed(&WidgetStyle {
            background: None,
            color: input.text_color,
            font_size: input.font_size,
            font_weight: input.font_weight,
            font_italic: input.font_italic,
            line_height: input.line_height,
            text_align: TextAlign::Start,
            font_family: Some(family.clone()),
        });

        assert_eq!(input.font_family, Some(family));
        assert!(input.needs_refresh);
    }

    #[test]
    fn mixed_script_preedit_keeps_the_multiline_caret_box_stable() {
        let mut input = TextInput::multiline();
        input.attribute_changed("font-size", "14px");
        input.attribute_changed("value", "abc");
        input.focus_changed(true);
        let mut tcx = TextContext::new();
        input.paint(240.0, 80.0, &mut tcx);
        let latin = input.editor.cursor_geometry(1.5).expect("Latin caret");
        let latin_line = *input
            .editor
            .try_layout()
            .and_then(|layout| layout.lines().next())
            .expect("Latin line")
            .metrics();
        let latin_box = (latin.y0 as f32, latin.y1 as f32);
        let latin_glyph_y = input
            .editor
            .try_layout()
            .and_then(|layout| layout.lines().next())
            .and_then(|line| {
                line.items().find_map(|item| match item {
                    PositionedLayoutItem::GlyphRun(run) => {
                        run.positioned_glyphs().next().map(|glyph| glyph.y)
                    }
                    _ => None,
                })
            })
            .expect("Latin glyph");

        input.handle_event(&UiEvent::Ime(ImeEvent::Preedit {
            text: "你".into(),
            cursor: Some((3, 3)),
        }));
        input.paint(240.0, 80.0, &mut tcx);
        let mixed = input.editor.cursor_geometry(1.5).expect("mixed caret");
        let mixed_line = *input
            .editor
            .try_layout()
            .and_then(|layout| layout.lines().next())
            .expect("mixed line")
            .metrics();
        let mixed_box = (mixed.y0 as f32, mixed.y1 as f32);
        let mixed_latin_glyph_y = input
            .editor
            .try_layout()
            .and_then(|layout| layout.lines().next())
            .and_then(|line| {
                line.items().find_map(|item| match item {
                    PositionedLayoutItem::GlyphRun(run) => {
                        run.positioned_glyphs().next().map(|glyph| glyph.y)
                    }
                    _ => None,
                })
            })
            .expect("mixed Latin glyph");

        assert_eq!(
            latin_box, mixed_box,
            "latin={latin:?} {latin_line:?} mixed={mixed:?} {mixed_line:?}"
        );
        assert_eq!(latin_glyph_y, mixed_latin_glyph_y);
    }

    #[test]
    fn multiline_preedit_does_not_jump_when_cjk_candidate_wraps() {
        let mut input = TextInput::multiline();
        input.attribute_changed("font-size", "14px");
        input.attribute_changed("value", "first line\nsecond line\naaaaa");
        input.focus_changed(true);
        let mut tcx = TextContext::new();
        input.paint(60.0, 38.0, &mut tcx);
        {
            let mut driver = input.editor.driver(&mut tcx.font_cx, &mut tcx.layout_cx);
            driver.move_to_text_end();
            driver.refresh_layout();
        }

        input.handle_event(&UiEvent::Ime(ImeEvent::Preedit {
            text: "zhongwen".into(),
            cursor: Some((8, 8)),
        }));
        input.paint(60.0, 38.0, &mut tcx);
        let phonetic_scroll = input.scroll_y;

        input.handle_event(&UiEvent::Ime(ImeEvent::Preedit {
            text: "中文".into(),
            cursor: Some((6, 6)),
        }));
        input.paint(60.0, 38.0, &mut tcx);

        assert!(
            (input.scroll_y - phonetic_scroll).abs() < 0.5,
            "composition viewport moved from {phonetic_scroll} to {}",
            input.scroll_y
        );
        let caret = input.editor.cursor_geometry(1.5).expect("caret");
        assert!(caret.y0 >= f64::from(input.scroll_y));
        assert!(caret.y1 <= f64::from(input.scroll_y + input.viewport_height));
    }

    #[test]
    fn empty_textarea_paints_preedit_instead_of_the_placeholder() {
        let mut input = TextInput::multiline();
        input.attribute_changed("placeholder", "Write a message");
        input.handle_event(&UiEvent::Ime(ImeEvent::Preedit {
            text: "中文".into(),
            cursor: Some((6, 6)),
        }));
        let mut tcx = TextContext::new();
        let scene = input.paint(240.0, 80.0, &mut tcx);

        assert!(input.cached_value.is_empty());
        assert!(input.editor.is_composing());
        assert!(!scene.commands.is_empty(), "preedit glyphs must be painted");
        let mut placeholder_context = PaintContext::new(240.0, 80.0, 1.0, &mut tcx);
        assert!(
            input
                .paint_placeholder(&mut placeholder_context, 240.0, None)
                .is_none(),
            "placeholder must stay hidden while composing"
        );
    }

    #[test]
    fn pointer_coordinates_are_widget_local() {
        let mut input = TextInput::new();
        input.handle_event(&pointer(PointerPhase::Down, 10.0, 5));
        assert!(matches!(
            input.pending.last(),
            Some(PendingEdit::MoveToPoint(x, y)) if (*x - 10.0).abs() < f32::EPSILON && (*y - 10.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn unknown_attributes_do_not_request_framework_work() {
        let mut input = TextInput::new();
        assert_eq!(
            input.attribute_changed("data-unknown", "value"),
            WidgetChanges::empty()
        );
        assert_eq!(
            input.attribute_removed("data-unknown"),
            WidgetChanges::empty()
        );
    }

    #[test]
    fn caret_follows_the_computed_foreground_color() {
        let mut input = TextInput::new();
        input.attribute_changed("color", "#111827");

        assert_eq!(
            input.caret_color(),
            wabou_shell::style::parse_color("#111827").unwrap()
        );
    }

    #[test]
    fn placeholder_and_first_typed_glyph_keep_the_same_baseline() {
        let mut input = TextInput::new();
        input.attribute_changed("font-size", "14px");
        input.attribute_changed("placeholder", "Choose a repository");
        let mut tcx = TextContext::new();
        input.paint(240.0, 30.0, &mut tcx);
        let placeholder = input.text_metrics.expect("placeholder metrics");

        input.handle_event(&UiEvent::TextInput("a".into()));
        input.paint(240.0, 30.0, &mut tcx);
        let typed = input.text_metrics.expect("typed text metrics");

        assert_eq!(placeholder.baseline, typed.baseline);
        assert_eq!(placeholder.line_box[1], typed.line_box[1]);

        let mut composing = TextInput::new();
        composing.attribute_changed("font-size", "14px");
        composing.attribute_changed("placeholder", "Choose a repository");
        composing.paint(240.0, 30.0, &mut tcx);
        let placeholder = composing.text_metrics.expect("placeholder metrics");
        composing.focus_changed(true);
        composing.handle_event(&UiEvent::Ime(ImeEvent::Preedit {
            text: "你".into(),
            cursor: Some((3, 3)),
        }));
        composing.paint(240.0, 30.0, &mut tcx);
        let preedit = composing.text_metrics.expect("preedit metrics");
        assert_eq!(placeholder.baseline, preedit.baseline);
        assert_eq!(placeholder.line_box[1], preedit.line_box[1]);
    }

    #[test]
    fn clipboard_shortcut_key_is_case_insensitive() {
        let mut input = TextInput::new();
        let result = input.handle_event(&UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "V".into(),
            key_without_modifiers: "v".into(),
            code: "KeyV".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: if cfg!(target_os = "macos") {
                Modifiers::META
            } else {
                Modifiers::CONTROL
            },
            repeat: false,
            synthetic: false,
        }));

        assert_eq!(result.clipboard_request(), Some(&ClipboardRequest::Read));
    }

    #[test]
    fn textarea_inserts_newlines_and_wraps_to_its_width() {
        let mut area = TextInput::multiline();
        area.attribute_changed("value", "first line");
        let mut tcx = TextContext::new();
        area.paint(80.0, 48.0, &mut tcx);
        area.handle_event(&key("End"));
        area.paint(80.0, 48.0, &mut tcx);

        let result = area.handle_event(&key("Enter"));
        area.handle_event(&UiEvent::TextInput("second line that wraps".into()));
        area.paint(80.0, 48.0, &mut tcx);

        assert!(result.value_changed());
        assert!(result.consumes_key_text());
        assert_eq!(
            area.current_value(),
            Some("first line\nsecond line that wraps")
        );
        assert!(
            area.editor
                .try_layout()
                .is_some_and(|layout| layout.height() > 48.0)
        );
    }

    #[test]
    fn textarea_wheel_scroll_is_clamped_to_multiline_content() {
        let mut area = TextInput::multiline();
        area.attribute_changed("value", "one\ntwo\nthree\nfour\nfive\nsix");
        let mut tcx = TextContext::new();
        area.paint(160.0, 32.0, &mut tcx);

        let result = area.handle_event(&UiEvent::Wheel(WheelEvent {
            position: Point { x: 10.0, y: 10.0 },
            delta_x: 0.0,
            delta_y: 10_000.0,
            delta_mode: wabou_shell::WheelDeltaMode::Pixel,
            phase: wabou_shell::GesturePhase::Changed,
            modifiers: Modifiers::default(),
        }));

        assert!(result.is_handled());
        assert!(area.scroll_y > 0.0);
        assert!(area.scroll_y <= area.editor.try_layout().unwrap().height() - 32.0);
    }

    #[test]
    fn readonly_textarea_allows_navigation_but_rejects_edits() {
        let mut area = TextInput::multiline();
        area.attribute_changed("value", "locked");
        area.attribute_changed("readOnly", "true");
        let mut tcx = TextContext::new();
        area.paint(160.0, 64.0, &mut tcx);

        assert!(!area.handle_event(&key("Enter")).is_handled());
        assert!(
            !area
                .handle_event(&UiEvent::TextInput("!".into()))
                .is_handled()
        );
        area.paint(160.0, 64.0, &mut tcx);
        assert_eq!(area.current_value(), Some("locked"));
    }

    #[test]
    fn text_input_accepts_only_canonical_control_attributes() {
        let mut input = TextInput::new();

        assert!(input.attribute_changed("readonly", "true").is_empty());
        assert!(!input.read_only);
        input.attribute_changed("readOnly", "true");
        assert!(input.read_only);

        assert!(input.attribute_changed("type", "password").is_empty());
    }

    #[test]
    fn controlled_value_echo_preserves_the_live_selection() {
        let mut area = TextInput::multiline();
        area.attribute_changed("value", "controlled");
        let mut tcx = TextContext::new();
        area.paint(160.0, 64.0, &mut tcx);
        area.queue(PendingEdit::SelectAll);
        area.paint(160.0, 64.0, &mut tcx);
        assert_eq!(area.editor.selected_text(), Some("controlled"));

        area.attribute_changed("value", "controlled");
        area.paint(160.0, 64.0, &mut tcx);
        assert_eq!(area.editor.selected_text(), Some("controlled"));
    }

    #[test]
    fn textarea_normalizes_windows_line_endings() {
        let mut area = TextInput::multiline();
        area.handle_event(&UiEvent::Paste("one\r\ntwo\rthree".into()));
        let mut tcx = TextContext::new();
        area.paint(160.0, 64.0, &mut tcx);
        assert_eq!(area.current_value(), Some("one\ntwo\nthree"));
    }

    #[test]
    fn ime_preedit_is_replaced_by_commit_and_reports_a_caret_area() {
        let mut input = TextInput::new();
        input.focus_changed(true);
        let mut tcx = TextContext::new();
        input.paint(200.0, 32.0, &mut tcx);

        assert!(
            input
                .handle_event(&UiEvent::Ime(ImeEvent::Preedit {
                    text: "に".into(),
                    cursor: Some((3, 3)),
                }))
                .is_handled()
        );
        input.paint(200.0, 32.0, &mut tcx);
        let raw_area = input.editor.cursor_geometry(1.5).expect("preedit caret");
        let area = input.ime_cursor_area().expect("IME cursor area");
        assert_eq!(area[1], raw_area.y0 as f32 + input.single_line_y_offset);
        assert_eq!(area[3], raw_area.y1 as f32 + input.single_line_y_offset);

        let mut paint = PaintContext::new_clipped(200.0, 32.0, 6.0, 2.0, &mut tcx);
        <TextInput as Widget>::paint(&mut input, &mut paint);
        let scene = paint.finish();
        let path =
            std::env::temp_dir().join(format!("wabou-ime-preedit-{}.png", std::process::id()));
        wabou_shell::renderer::render_to_png(
            &scene,
            400,
            64,
            Color::TRANSPARENT,
            &path.to_string_lossy(),
        )
        .expect("render IME preedit scene");
        std::fs::remove_file(path).expect("remove owned IME render");

        assert!(
            input
                .handle_event(&UiEvent::Ime(ImeEvent::Commit("日本".into())))
                .value_changed()
        );
        input.paint(200.0, 32.0, &mut tcx);
        assert_eq!(input.current_value(), Some("日本"));
    }

    #[test]
    fn disabling_ime_clears_uncommitted_preedit() {
        let mut input = TextInput::new();
        let mut tcx = TextContext::new();
        input.handle_event(&UiEvent::Ime(ImeEvent::Preedit {
            text: "draft".into(),
            cursor: None,
        }));
        input.paint(200.0, 32.0, &mut tcx);
        input.handle_event(&UiEvent::Ime(ImeEvent::Disabled));
        input.paint(200.0, 32.0, &mut tcx);
        assert_eq!(input.current_value(), Some(""));
    }
}
