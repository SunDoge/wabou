//! TextInput widget — a single-line text input backed by `parley::PlainEditor`.
//!
//! PlainEditor handles text editing, caret geometry, selection, and
//! hit-testing. `handle_event` stores pending edits (the editor needs
//! FontContext/LayoutContext which aren't available in handle_event);
//! `paint` applies them via the driver, refreshes layout, then paints
//! selection rects + caret + glyph runs.

use std::time::{Duration, Instant};

use parley::{PlainEditor, PositionedLayoutItem};
use vello::Scene;
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};
use wabou_shell::text::TextContext;
use wabou_shell::{KeyPhase, PointerPhase, UiEvent};

use super::{Widget, WidgetEventResult};

const SELECTION_COLOR: Color = Color::from_rgba8(99, 102, 241, 80);
const CARET_COLOR: Color = Color::from_rgb8(0xe2, 0xe8, 0xf0);
const PLACEHOLDER_COLOR: Color = Color::from_rgb8(0x64, 0x74, 0x8b);

/// A pending edit to apply on the next `paint` (when FontContext is available).
enum PendingEdit {
    Insert(String),
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

pub struct TextInput {
    editor: PlainEditor<[u8; 4]>,
    placeholder: String,
    font_size: f32,
    text_color: Color,
    focused: bool,
    blink_on: bool,
    next_blink: Option<Instant>,
    pending: Vec<PendingEdit>,
    needs_refresh: bool,
    window_to_local: [f64; 6],
    /// Cached value string (updated in paint after edits) for current_value().
    cached_value: String,
    selecting: bool,
    device_scale: f64,
    last_click: Option<(Instant, f32, f32, u8)>,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            editor: PlainEditor::new(16.0),
            placeholder: String::new(),
            font_size: 16.0,
            text_color: Color::from_rgb8(0xe2, 0xe8, 0xf0),
            focused: false,
            blink_on: true,
            next_blink: None,
            pending: Vec::new(),
            needs_refresh: false,
            window_to_local: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            cached_value: String::new(),
            selecting: false,
            device_scale: 1.0,
            last_click: None,
        }
    }

    fn queue(&mut self, edit: PendingEdit) {
        self.pending.push(edit);
        self.blink_on = true;
        self.next_blink = Some(Instant::now() + Duration::from_millis(500));
    }

    fn local_point(&self, x: f64, y: f64) -> (f32, f32) {
        let [a, b, c, d, e, f] = self.window_to_local;
        ((a * x + c * y + e) as f32, (b * x + d * y + f) as f32)
    }
}

impl Widget for TextInput {
    fn paint(&mut self, _width: f32, height: f32, tcx: &mut TextContext) -> Scene {
        // Blink.
        if self.focused && self.next_blink.is_some_and(|d| Instant::now() >= d) {
            self.blink_on = !self.blink_on;
            self.next_blink = Some(Instant::now() + Duration::from_millis(500));
        }

        // Apply pending edits + refresh layout (needs FontContext/LayoutContext).
        if !self.pending.is_empty() || self.needs_refresh {
            {
                let mut driver = self.editor.driver(&mut tcx.font_cx, &mut tcx.layout_cx);
                for edit in self.pending.drain(..) {
                    match edit {
                        PendingEdit::Insert(s) => driver.insert_or_replace_selection(&s),
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
                        PendingEdit::MoveToStart => driver.move_to_text_start(),
                        PendingEdit::MoveToEnd => driver.move_to_text_end(),
                        PendingEdit::SelectToStart => driver.select_to_text_start(),
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
        }

        // Cache value for current_value().
        self.cached_value = self.editor.text().to_string();

        let mut scene = Scene::new();
        let h = height as f64;

        // Determine display text + color.
        let value_str = self.editor.text().to_string();
        let is_empty = value_str.is_empty();
        let display_text = if is_empty {
            self.placeholder.as_str()
        } else {
            &value_str
        };
        let text_color = if is_empty {
            PLACEHOLDER_COLOR
        } else {
            self.text_color
        };

        // If editor layout is available, paint selection + caret + text from it.
        if let Some(layout) = self.editor.try_layout() {
            let text_height = layout.height() as f64;
            let y_offset = ((h - text_height) * 0.5).max(0.0);
            let transform = Affine::translate((0.0, y_offset));

            // Selection rects.
            if self.focused {
                for (bbox, _line) in self.editor.selection_geometry() {
                    scene.fill(
                        Fill::NonZero,
                        transform,
                        SELECTION_COLOR,
                        None,
                        &Rect::new(bbox.x0, bbox.y0, bbox.x1, bbox.y1),
                    );
                }
            }

            // Text glyph runs.
            if !display_text.is_empty() {
                for line in layout.lines() {
                    for item in line.items() {
                        if let PositionedLayoutItem::GlyphRun(gr) = item {
                            let font_data = gr.run().font().clone();
                            let font_size = gr.run().font_size();
                            let glyphs: Vec<vello::Glyph> = gr
                                .positioned_glyphs()
                                .map(|g| vello::Glyph {
                                    id: g.id,
                                    x: g.x * self.device_scale as f32,
                                    y: g.y * self.device_scale as f32,
                                })
                                .collect();
                            if !glyphs.is_empty() {
                                scene
                                    .draw_glyphs(&font_data)
                                    .font_size(font_size * self.device_scale as f32)
                                    .hint(true)
                                    .brush(text_color)
                                    .transform(transform * Affine::scale(self.device_scale.recip()))
                                    .draw(Fill::NonZero, glyphs.into_iter());
                            }
                        }
                    }
                }
            }

            // Caret.
            if self.focused && self.blink_on {
                if let Some(cursor) = self.editor.cursor_geometry(1.5) {
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

        scene
    }

    fn paint_scaled(
        &mut self,
        width: f32,
        height: f32,
        device_scale: f64,
        tcx: &mut TextContext,
    ) -> Scene {
        self.device_scale = device_scale.max(f64::EPSILON);
        self.paint(width, height, tcx)
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        match event {
            UiEvent::Pointer(e)
                if e.phase == PointerPhase::Down
                    && e.button == Some(wabou_shell::PointerButton::Primary) =>
            {
                // Convert absolute pointer to local content-box coords.
                let (local_x, local_y) = self.local_point(e.position.x, e.position.y);
                if e.modifiers.shift() {
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
            UiEvent::Pointer(e) if e.phase == PointerPhase::Move && self.selecting => {
                let (local_x, local_y) = self.local_point(e.position.x, e.position.y);
                self.queue(PendingEdit::ExtendToPoint(local_x, local_y));
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(e) if e.phase == PointerPhase::Up && self.selecting => {
                let (local_x, local_y) = self.local_point(e.position.x, e.position.y);
                self.queue(PendingEdit::ExtendToPoint(local_x, local_y));
                self.selecting = false;
                WidgetEventResult::HANDLED
            }
            UiEvent::Pointer(e) if e.phase == PointerPhase::Cancel && self.selecting => {
                self.selecting = false;
                WidgetEventResult::HANDLED
            }
            UiEvent::TextInput(text) | UiEvent::Paste(text) => {
                let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                if filtered.is_empty() {
                    return WidgetEventResult::IGNORED;
                }
                self.queue(PendingEdit::Insert(filtered));
                WidgetEventResult::VALUE_CHANGED
            }
            UiEvent::Key(e) if e.phase == KeyPhase::Down => match e.key.as_str() {
                key if key.eq_ignore_ascii_case("c") && e.modifiers.primary_shortcut() => self
                    .editor
                    .selected_text()
                    .map(str::to_owned)
                    .map_or(WidgetEventResult::IGNORED, WidgetEventResult::copy),
                key if key.eq_ignore_ascii_case("x") && e.modifiers.primary_shortcut() => {
                    if let Some(text) = self.editor.selected_text().map(str::to_owned) {
                        self.queue(PendingEdit::Delete);
                        WidgetEventResult::copy_with_value_change(text)
                    } else {
                        WidgetEventResult::IGNORED
                    }
                }
                key if key.eq_ignore_ascii_case("v") && e.modifiers.primary_shortcut() => {
                    WidgetEventResult::paste()
                }
                "Backspace" => {
                    self.queue(PendingEdit::Delete);
                    WidgetEventResult::VALUE_CHANGED
                }
                "Delete" => {
                    self.queue(PendingEdit::DeleteForward);
                    WidgetEventResult::VALUE_CHANGED
                }
                "ArrowLeft" => {
                    self.queue(
                        match (
                            e.modifiers.shift(),
                            e.modifiers.control() || e.modifiers.alt(),
                        ) {
                            (true, true) => PendingEdit::SelectWordLeft,
                            (true, false) => PendingEdit::SelectLeft,
                            (false, true) => PendingEdit::MoveWordLeft,
                            (false, false) => PendingEdit::MoveLeft,
                        },
                    );
                    WidgetEventResult::HANDLED
                }
                "ArrowRight" => {
                    self.queue(
                        match (
                            e.modifiers.shift(),
                            e.modifiers.control() || e.modifiers.alt(),
                        ) {
                            (true, true) => PendingEdit::SelectWordRight,
                            (true, false) => PendingEdit::SelectRight,
                            (false, true) => PendingEdit::MoveWordRight,
                            (false, false) => PendingEdit::MoveRight,
                        },
                    );
                    WidgetEventResult::HANDLED
                }
                "Home" => {
                    self.queue(if e.modifiers.shift() {
                        PendingEdit::SelectToStart
                    } else {
                        PendingEdit::MoveToStart
                    });
                    WidgetEventResult::HANDLED
                }
                "End" => {
                    self.queue(if e.modifiers.shift() {
                        PendingEdit::SelectToEnd
                    } else {
                        PendingEdit::MoveToEnd
                    });
                    WidgetEventResult::HANDLED
                }
                key if key.eq_ignore_ascii_case("a") && e.modifiers.primary_shortcut() => {
                    self.queue(PendingEdit::SelectAll);
                    WidgetEventResult::HANDLED
                }
                _ => WidgetEventResult::IGNORED,
            },
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn attribute_changed(&mut self, name: &str, value: &str) {
        match name {
            "value" => {
                self.editor.set_text(value);
                self.needs_refresh = true;
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
            _ => {}
        }
    }

    fn current_value(&self) -> Option<&str> {
        Some(&self.cached_value)
    }

    fn style_changed(&mut self, style: &super::WidgetStyle) {
        self.text_color = style.color;
        if self.font_size != style.font_size {
            self.font_size = style.font_size;
            self.editor
                .edit_styles()
                .insert(parley::StyleProperty::FontSize(style.font_size));
            self.needs_refresh = true;
        }
    }

    fn accepts_focus(&self) -> bool {
        true
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([120.0, 32.0])
    }

    fn focus_changed(&mut self, focused: bool) {
        self.focused = focused;
        self.blink_on = true;
        self.next_blink = focused.then(|| Instant::now() + Duration::from_millis(500));
    }

    fn animation_deadline(&self) -> Option<Instant> {
        self.next_blink
    }

    fn set_window_to_local(&mut self, transform: [f64; 6]) {
        self.window_to_local = transform;
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
mod tests {
    use super::*;
    use wabou_shell::{ClipboardRequest, KeyEvent, Modifiers, Point, PointerButton, PointerEvent};

    fn pointer(phase: PointerPhase, x: f64, buttons: u32) -> UiEvent {
        UiEvent::Pointer(PointerEvent {
            phase,
            position: Point { x, y: 10.0 },
            button: Some(PointerButton::Primary),
            buttons,
            modifiers: Modifiers::default(),
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
    fn pointer_coordinates_follow_widget_affine_geometry() {
        let mut input = TextInput::new();
        input.set_window_to_local([0.5, 0.0, 0.0, 0.5, -50.0, 0.0]);
        input.handle_event(&pointer(PointerPhase::Down, 120.0, 1));
        assert!(matches!(
            input.pending.last(),
            Some(PendingEdit::MoveToPoint(x, y)) if (*x - 10.0).abs() < f32::EPSILON && (*y - 5.0).abs() < f32::EPSILON
        ));
    }

    #[test]
    fn shifted_clipboard_shortcut_is_case_insensitive() {
        let mut input = TextInput::new();
        let result = input.handle_event(&UiEvent::Key(KeyEvent {
            phase: KeyPhase::Down,
            key: "V".into(),
            key_without_modifiers: "v".into(),
            code: "KeyV".into(),
            text: None,
            text_with_all_modifiers: None,
            location: Default::default(),
            modifiers: Modifiers::CONTROL | Modifiers::SHIFT,
            repeat: false,
        }));

        assert_eq!(result.clipboard_request(), Some(&ClipboardRequest::Read));
    }
}
