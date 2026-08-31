//! Typed input emitted by GPUI hit targets toward the Wabou runtime.

use std::{ops::Range, rc::Rc};

use gpui::{App, Bounds, InputHandler, Pixels, Point, UTF16Selection, Window, point, px, size};

use crate::NodeKey;

/// Pointer phase delivered by a projected GPUI element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedPointerPhase {
    /// The pointer moved over the target.
    Move,
    /// A pointer button was pressed.
    Down,
    /// A pointer button was released.
    Up,
}

/// Pointer button independent of GPUI and the guest event model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedPointerButton {
    /// Primary/left button.
    Primary,
    /// Auxiliary/middle button.
    Auxiliary,
    /// Secondary/right button.
    Secondary,
    /// Navigation or otherwise backend-specific button.
    Other,
}

/// One pointer transition with an explicit retained target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedPointerEvent {
    /// Deepest projected node selected by GPUI hit testing.
    pub target: NodeKey,
    /// Native transition phase.
    pub phase: ProjectedPointerPhase,
    /// Window-logical horizontal coordinate.
    pub x: f32,
    /// Window-logical vertical coordinate.
    pub y: f32,
    /// Horizontal coordinate relative to the target border box.
    pub local_x: f32,
    /// Vertical coordinate relative to the target border box.
    pub local_y: f32,
    /// Button changed by this transition.
    pub button: Option<ProjectedPointerButton>,
    /// Whether Shift is held.
    pub shift: bool,
    /// Whether Control is held.
    pub control: bool,
    /// Whether Alt/Option is held.
    pub alt: bool,
    /// Whether Command/Super/Windows is held.
    pub platform: bool,
}

/// Native wheel phase independent of GPUI's platform representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedWheelPhase {
    /// A precise gesture started.
    Started,
    /// The wheel or gesture changed.
    Changed,
    /// A precise gesture ended.
    Ended,
    /// The platform cancelled the gesture.
    Cancelled,
}

/// Wheel transition targeted by GPUI hit testing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedWheelEvent {
    /// Deepest projected node selected by GPUI hit testing.
    pub target: NodeKey,
    /// Window-logical horizontal coordinate.
    pub x: f32,
    /// Window-logical vertical coordinate.
    pub y: f32,
    /// Horizontal coordinate relative to the target border box.
    pub local_x: f32,
    /// Vertical coordinate relative to the target border box.
    pub local_y: f32,
    /// Horizontal delta in source units.
    pub delta_x: f32,
    /// Vertical delta in source units.
    pub delta_y: f32,
    /// Whether the source units are precise logical pixels rather than lines.
    pub precise: bool,
    /// Native gesture lifecycle.
    pub phase: ProjectedWheelPhase,
    /// Whether Shift is held.
    pub shift: bool,
    /// Whether Control is held.
    pub control: bool,
    /// Whether Alt/Option is held.
    pub alt: bool,
    /// Whether Command/Super/Windows is held.
    pub platform: bool,
}

/// Logical scroll position produced by GPUI's retained scroll state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProjectedScrollEvent {
    /// Scrollable retained node.
    pub target: NodeKey,
    /// Logical horizontal offset measured from the content origin.
    pub x: f32,
    /// Logical vertical offset measured from the content origin.
    pub y: f32,
}

/// Input transition emitted from one retained GPUI hit target.
#[derive(Clone, Debug, PartialEq)]
pub enum ProjectedInputEvent {
    /// Semantic activation emitted by a native widget that already owns its
    /// pointer and keyboard interaction lifecycle.
    Activate { target: NodeKey },
    /// Pointer movement or button transition.
    Pointer(ProjectedPointerEvent),
    /// Wheel or trackpad transition.
    Wheel(ProjectedWheelEvent),
    /// Scroll position changed after native clamping.
    Scroll(ProjectedScrollEvent),
    /// Keyboard transition delivered through the GPUI root focus handle.
    Key(ProjectedKeyEvent),
    /// Platform text-input or IME transition.
    Ime(ProjectedImeEvent),
}

/// Platform text-input transition normalized for Wabou's focused input model.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectedImeEvent {
    /// Replace the active composition/selection with committed text.
    Commit(String),
    /// Update marked composition text and its byte-index cursor range.
    Preedit {
        /// Current marked text.
        text: String,
        /// Cursor/selection inside `text`, expressed as UTF-8 byte offsets.
        cursor: Option<(usize, usize)>,
    },
}

/// Focused text state captured from the authoritative Wabou widget each frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProjectedTextInputState {
    /// Whether the focused Wabou node currently accepts text input.
    pub accepts_text: bool,
    /// Controlled text value, when exposed by the focused widget.
    pub text: Option<String>,
    /// UTF-16 selection range in document order.
    pub selection: Option<Range<usize>>,
    /// Whether the selection's moving head precedes its anchor.
    pub selection_reversed: bool,
    /// Candidate-window anchor in window-logical coordinates.
    pub cursor_bounds: Option<[f32; 4]>,
}

/// GPUI input handler forwarding platform composition to the Wabou runtime.
#[derive(Clone)]
pub struct ProjectedInputHandler {
    input: ProjectedInputSink,
    state: ProjectedTextInputState,
}

impl ProjectedInputHandler {
    /// Create a handler from one completed Wabou frame snapshot.
    #[must_use]
    pub fn new(input: ProjectedInputSink, state: ProjectedTextInputState) -> Self {
        Self { input, state }
    }
}

fn utf16_to_byte(text: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    for (byte, character) in text.char_indices() {
        if utf16 >= offset {
            return byte;
        }
        utf16 += character.len_utf16();
        if utf16 > offset {
            return byte;
        }
    }
    text.len()
}

impl InputHandler for ProjectedInputHandler {
    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<UTF16Selection> {
        self.state.selection.clone().map(|range| UTF16Selection {
            range,
            reversed: self.state.selection_reversed,
        })
    }

    fn marked_text_range(&mut self, _window: &mut Window, _cx: &mut App) -> Option<Range<usize>> {
        None
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<String> {
        let text = self.state.text.as_deref()?;
        let start = utf16_to_byte(text, range_utf16.start);
        let end = utf16_to_byte(text, range_utf16.end);
        *adjusted_range = Some(range_utf16);
        text.get(start..end).map(str::to_owned)
    }

    fn replace_text_in_range(
        &mut self,
        _replacement_range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut App,
    ) {
        (self.input)(
            ProjectedInputEvent::Ime(ProjectedImeEvent::Commit(text.to_owned())),
            cx,
        );
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut App,
    ) {
        let cursor = new_selected_range.map(|range| {
            (
                utf16_to_byte(new_text, range.start),
                utf16_to_byte(new_text, range.end),
            )
        });
        (self.input)(
            ProjectedInputEvent::Ime(ProjectedImeEvent::Preedit {
                text: new_text.to_owned(),
                cursor,
            }),
            cx,
        );
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut App) {
        (self.input)(
            ProjectedInputEvent::Ime(ProjectedImeEvent::Preedit {
                text: String::new(),
                cursor: None,
            }),
            cx,
        );
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        window: &mut Window,
        _cx: &mut App,
    ) -> Option<Bounds<Pixels>> {
        let [x0, y0, x1, y1] = self.state.cursor_bounds?;
        let window_origin = window.window_bounds().get_bounds().origin;
        Some(Bounds::new(
            window_origin + point(px(x0), px(y0)),
            size(px((x1 - x0).max(1.0)), px((y1 - y0).max(1.0))),
        ))
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<Pixels>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Option<usize> {
        None
    }

    fn text_length_utf16(&mut self, _window: &mut Window, _cx: &mut App) -> Option<usize> {
        self.state
            .text
            .as_deref()
            .map(|text| text.encode_utf16().count())
    }

    fn accepts_text_input(&mut self, _window: &mut Window, _cx: &mut App) -> bool {
        self.state.accepts_text
    }

    fn prefers_ime_for_printable_keys(&mut self, _window: &mut Window, _cx: &mut App) -> bool {
        self.state.accepts_text
    }
}

/// Keyboard phase independent of GPUI's event types.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectedKeyPhase {
    /// A key was pressed or repeated.
    Down,
    /// A key was released.
    Up,
}

/// Keyboard transition normalized at the GPUI shell boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedKeyEvent {
    /// Press or release phase.
    pub phase: ProjectedKeyPhase,
    /// GPUI's layout-independent key identity.
    pub key: String,
    /// Character produced by the active layout, when available.
    pub key_char: Option<String>,
    /// Whether this is an automatic repeat.
    pub repeat: bool,
    /// Whether Shift is held.
    pub shift: bool,
    /// Whether Control is held.
    pub control: bool,
    /// Whether Alt/Option is held.
    pub alt: bool,
    /// Whether Command/Super/Windows is held.
    pub platform: bool,
}

/// UI-thread callback installed while a projection is materialized.
pub type ProjectedInputSink = Rc<dyn Fn(ProjectedInputEvent, &mut App)>;

#[cfg(test)]
mod tests {
    use super::utf16_to_byte;

    #[test]
    fn utf16_offsets_map_to_stable_utf8_composition_cursor_boundaries() {
        let text = "a😀日";
        assert_eq!(utf16_to_byte(text, 0), 0);
        assert_eq!(utf16_to_byte(text, 1), 1);
        assert_eq!(utf16_to_byte(text, 2), 1, "split surrogate clamps down");
        assert_eq!(utf16_to_byte(text, 3), 5);
        assert_eq!(utf16_to_byte(text, 4), text.len());
        assert_eq!(utf16_to_byte(text, 99), text.len());
    }
}
