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

use parley::{PlainEditor, PositionedLayoutItem};
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};
use wabou_shell::style::TextAlign;
use wabou_shell::text::{TextContext, brush_for_color, layout_text_styled};
use wabou_shell::{ImeEvent, KeyEvent, KeyPhase, PointerEvent, PointerPhase, UiEvent};

use wabou_shell::{PaintContext, Widget, WidgetChanges, WidgetEventResult, WidgetStyle};

use crate::single_line_y_offset;

const SELECTION_COLOR: Color = Color::from_rgba8(99, 102, 241, 80);
const CARET_COLOR: Color = Color::from_rgb8(0xe2, 0xe8, 0xf0);
const PLACEHOLDER_COLOR: Color = Color::from_rgb8(0x64, 0x74, 0x8b);

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
}

/// Native single- or multiline plain-text editor backed by Parley.
pub struct TextInput {
    editor: PlainEditor<[u8; 4]>,
    placeholder: String,
    font_size: f32,
    font_weight: f32,
    line_height: Option<(f32, bool)>,
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
    scroll_y: f32,
    disabled: bool,
    read_only: bool,
    password: bool,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
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
        Self {
            editor: PlainEditor::new(16.0),
            placeholder: String::new(),
            font_size: 16.0,
            font_weight: 400.0,
            line_height: None,
            text_color: Color::from_rgb8(0xe2, 0xe8, 0xf0),
            focused: false,
            blink_on: true,
            next_blink: None,
            pending: Vec::new(),
            needs_refresh: false,
            cached_value: String::new(),
            selecting: false,
            last_click: None,
            multiline,
            viewport_width: 0.0,
            viewport_height: 0.0,
            scroll_y: 0.0,
            disabled: false,
            read_only: false,
            password: false,
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

    fn local_point(&self, x: f64, y: f64) -> (f32, f32) {
        let local_x = x as f32;
        let mut local_y = y as f32;
        if self.multiline {
            local_y += self.scroll_y;
        }
        (local_x, local_y)
    }

    fn clamp_scroll(&mut self) {
        let content_height = self
            .editor
            .try_layout()
            .map_or(0.0, |layout| layout.height());
        self.scroll_y = self
            .scroll_y
            .clamp(0.0, (content_height - self.viewport_height).max(0.0));
    }

    fn reveal_caret(&mut self) {
        if !self.multiline {
            return;
        }
        if let Some(caret) = self.editor.cursor_geometry(1.5) {
            if caret.y0 < f64::from(self.scroll_y) {
                self.scroll_y = caret.y0 as f32;
            } else if caret.y1 > f64::from(self.scroll_y + self.viewport_height) {
                self.scroll_y = (caret.y1 as f32 - self.viewport_height).max(0.0);
            }
        }
        self.clamp_scroll();
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
                    self.queue(match clicks {
                        2 => PendingEdit::SelectWordAtPoint(local_x, local_y),
                        3 => PendingEdit::SelectLineAtPoint(local_x, local_y),
                        _ => PendingEdit::MoveToPoint(local_x, local_y),
                    });
                }
                self.selecting = true;
                WidgetEventResult::HANDLED
            }
            PointerPhase::Move if self.selecting => {
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                self.queue(PendingEdit::ExtendToPoint(local_x, local_y));
                WidgetEventResult::HANDLED
            }
            PointerPhase::Up if self.selecting => {
                let (local_x, local_y) = self.local_point(event.position.x, event.position.y);
                self.queue(PendingEdit::ExtendToPoint(local_x, local_y));
                self.selecting = false;
                WidgetEventResult::HANDLED
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
                    WidgetEventResult::copy_with_value_change(text)
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
                WidgetEventResult::VALUE_CHANGED
            }
            "Delete" if self.editable() => {
                self.queue(PendingEdit::DeleteForward);
                WidgetEventResult::VALUE_CHANGED
            }
            "ArrowLeft" => {
                self.queue(Self::horizontal_edit(
                    false,
                    event.modifiers.shift(),
                    event.modifiers.control() || event.modifiers.alt(),
                ));
                WidgetEventResult::HANDLED
            }
            "ArrowRight" => {
                self.queue(Self::horizontal_edit(
                    true,
                    event.modifiers.shift(),
                    event.modifiers.control() || event.modifiers.alt(),
                ));
                WidgetEventResult::HANDLED
            }
            "ArrowUp" if self.multiline => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectUp
                } else {
                    PendingEdit::MoveUp
                });
                WidgetEventResult::HANDLED
            }
            "ArrowDown" if self.multiline => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectDown
                } else {
                    PendingEdit::MoveDown
                });
                WidgetEventResult::HANDLED
            }
            "Enter" if self.multiline && self.editable() => {
                self.queue(PendingEdit::Insert("\n".into()));
                WidgetEventResult::value_changed_consuming_key_text()
            }
            "Home" => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectToStart
                } else {
                    PendingEdit::MoveToStart
                });
                WidgetEventResult::HANDLED
            }
            "End" => {
                self.queue(if event.modifiers.shift() {
                    PendingEdit::SelectToEnd
                } else {
                    PendingEdit::MoveToEnd
                });
                WidgetEventResult::HANDLED
            }
            _ if event.matches_standard_shortcut(wabou_shell::StandardShortcut::SelectAll) => {
                self.queue(PendingEdit::SelectAll);
                WidgetEventResult::HANDLED
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
                }
            }
            driver.refresh_layout();
        }
        self.needs_refresh = false;
        self.reveal_caret();
    }

    fn paint_placeholder(
        &self,
        scene: &mut Scene,
        tcx: &mut TextContext,
        width: f32,
        height: f32,
        device_scale: f64,
    ) {
        if !self.cached_value.is_empty() || self.placeholder.is_empty() {
            return;
        }
        let layout = layout_text_styled(
            tcx,
            Arc::from(self.placeholder.as_str()),
            self.font_size,
            self.font_weight,
            self.line_height,
            TextAlign::Start,
            brush_for_color(PLACEHOLDER_COLOR),
            Arc::from([]),
            None,
            self.multiline.then_some(width.max(0.0)),
        );
        let y_offset = if self.multiline {
            0.0
        } else {
            single_line_y_offset(height, layout.height(), self.font_size)
        };
        let glyph_scene = tcx.glyph_scene_scaled(&layout, device_scale);
        scene.append(
            &glyph_scene,
            Some(Affine::translate((0.0, y_offset)) * Affine::scale(device_scale.recip())),
        );
    }

    fn paint_editor(
        &self,
        scene: &mut Scene,
        tcx: &mut TextContext,
        height: f32,
        device_scale: f64,
    ) {
        let Some(layout) = self.editor.try_layout() else {
            return;
        };
        let transform = Affine::translate((
            0.0,
            if self.multiline {
                -f64::from(self.scroll_y)
            } else {
                single_line_y_offset(height, layout.height(), self.font_size)
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
        if !self.cached_value.is_empty() && !self.password {
            for line in layout.lines() {
                for item in line.items() {
                    let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                        continue;
                    };
                    let glyphs: Vec<_> = glyph_run
                        .positioned_glyphs()
                        .map(|glyph| vello::Glyph {
                            id: glyph.id,
                            x: glyph.x * device_scale as f32,
                            y: glyph.y * device_scale as f32,
                        })
                        .collect();
                    if !glyphs.is_empty() {
                        scene
                            .draw_glyphs(glyph_run.run().font())
                            .font_size(glyph_run.run().font_size() * device_scale as f32)
                            .hint(true)
                            .brush(self.text_color)
                            .transform(transform * Affine::scale(device_scale.recip()))
                            .draw(Fill::NonZero, glyphs.into_iter());
                    }
                }
            }
        } else if !self.cached_value.is_empty() {
            let masked = "•".repeat(self.cached_value.chars().count());
            let masked_layout = layout_text_styled(
                tcx,
                Arc::from(masked),
                self.font_size,
                self.font_weight,
                self.line_height,
                TextAlign::Start,
                brush_for_color(self.text_color),
                Arc::from([]),
                None,
                None,
            );
            let glyph_scene = tcx.glyph_scene_scaled(&masked_layout, device_scale);
            scene.append(
                &glyph_scene,
                Some(transform * Affine::scale(device_scale.recip())),
            );
        }
        if self.focused
            && self.blink_on
            && let Some(cursor) = self.editor.cursor_geometry(1.5)
        {
            scene.fill(
                Fill::NonZero,
                transform,
                CARET_COLOR,
                None,
                &Rect::new(cursor.x0, cursor.y0, cursor.x1, cursor.y1),
            );
        }
    }
}

impl Widget for TextInput {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let [width, height] = cx.size();
        let device_scale = cx.device_scale();
        let tcx = cx.text();
        if self.multiline && self.viewport_width != width {
            self.viewport_width = width;
            self.editor.set_width(Some(width.max(0.0)));
            self.needs_refresh = true;
        }
        self.viewport_height = height.max(0.0);

        // Blink.
        if self.focused && self.next_blink.is_some_and(|d| Instant::now() >= d) {
            self.blink_on = !self.blink_on;
            self.next_blink = Some(Instant::now() + Duration::from_millis(500));
        }

        self.refresh_editor(tcx);

        // Cache value for current_value().
        self.cached_value = self.editor.text().to_string();

        let mut scene = Scene::new();
        let h = height as f64;
        if self.multiline {
            scene.push_clip_layer(
                Fill::NonZero,
                Affine::IDENTITY,
                &Rect::new(0.0, 0.0, f64::from(width.max(0.0)), h.max(0.0)),
            );
        }

        self.paint_placeholder(&mut scene, tcx, width, height, device_scale);
        self.paint_editor(&mut scene, tcx, height, device_scale);

        if self.multiline {
            scene.pop_layer();
        }
        cx.scene_mut().append(&scene, None);
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if self.disabled {
            return WidgetEventResult::IGNORED;
        }
        match event {
            UiEvent::Pointer(event) => self.handle_pointer(event),
            UiEvent::Wheel(event) if self.multiline => {
                let previous = self.scroll_y;
                self.scroll_y += event.delta_y as f32;
                self.clamp_scroll();
                if self.scroll_y != previous {
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
                WidgetEventResult::VALUE_CHANGED
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
                    WidgetEventResult::VALUE_CHANGED
                }
            }
            UiEvent::Ime(ImeEvent::DeleteSurrounding {
                before_bytes,
                after_bytes,
            }) if self.editable() => {
                self.queue(PendingEdit::DeleteSurrounding(*before_bytes, *after_bytes));
                WidgetEventResult::VALUE_CHANGED
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
            "type" => self.password = !self.multiline && value == "password",
            _ => {}
        }
        match name {
            "value" | "placeholder" | "font-size" | "color" | "disabled" | "readOnly" | "type" => {
                WidgetChanges::REDRAW | WidgetChanges::SEMANTICS
            }
            _ => WidgetChanges::empty(),
        }
    }

    fn attribute_removed(&mut self, name: &str) -> WidgetChanges {
        match name {
            "disabled" => self.disabled = false,
            "readOnly" => self.read_only = false,
            "placeholder" => self.placeholder.clear(),
            "type" => self.password = false,
            _ => {}
        }
        match name {
            "disabled" | "readOnly" | "placeholder" | "type" => {
                WidgetChanges::REDRAW | WidgetChanges::SEMANTICS
            }
            _ => WidgetChanges::empty(),
        }
    }

    fn current_value(&self) -> Option<&str> {
        Some(&self.cached_value)
    }

    fn accessibility(&self) -> wabou_shell::WidgetAccessibility {
        wabou_shell::WidgetAccessibility {
            role: Some(wabou_shell::SemanticRole::TextInput),
            value: (!self.password).then(|| self.cached_value.clone()),
            disabled: Some(self.disabled),
            ..Default::default()
        }
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> WidgetChanges {
        self.text_color = style.color;
        if self.font_size != style.font_size {
            self.font_size = style.font_size;
            self.editor
                .edit_styles()
                .insert(parley::StyleProperty::FontSize(style.font_size));
            self.needs_refresh = true;
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
        if self.line_height != style.line_height {
            self.line_height = style.line_height;
            let line_height = match style.line_height {
                Some((value, true)) => parley::LineHeight::FontSizeRelative(value),
                Some((value, false)) => parley::LineHeight::Absolute(value),
                None => parley::LineHeight::default(),
            };
            self.editor
                .edit_styles()
                .insert(parley::StyleProperty::LineHeight(line_height));
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
        let y_offset = if self.multiline { -self.scroll_y } else { 0.0 };
        Some([
            area.x0 as f32,
            area.y0 as f32 + y_offset,
            area.x1 as f32,
            area.y1 as f32 + y_offset,
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

        input.attribute_changed("type", "PASSWORD");
        assert!(!input.password);
        input.attribute_changed("type", "password");
        assert!(input.password);
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
    fn password_type_masks_painting_without_changing_the_value() {
        let mut input = TextInput::new();
        input.attribute_changed("value", "sëcret🔑");
        input.attribute_changed("type", "password");
        assert_eq!(input.accessibility().value, None);
        let mut tcx = TextContext::new();
        input.paint(200.0, 32.0, &mut tcx);

        assert!(input.password);
        assert_eq!(input.current_value(), Some("sëcret🔑"));

        input.attribute_removed("type");
        assert!(!input.password);
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
        assert!(input.ime_cursor_area().is_some());

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
