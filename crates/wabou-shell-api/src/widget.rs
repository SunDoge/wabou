//! Renderer-independent native widget presentation snapshots.

use crate::accessibility::SemanticRole;
use serde::{Deserialize, Serialize};

/// Optional semantic information contributed by a native widget.
/// Explicit JS/ARIA attributes remain authoritative when both are present.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct WidgetAccessibility {
    /// Native role used when the host element does not declare one.
    pub role: Option<SemanticRole>,
    /// Accessible name used when the host element does not declare one.
    pub label: Option<String>,
    /// Current textual value exposed to assistive technology.
    pub value: Option<String>,
    /// Whether every textual value associated with this widget is sensitive.
    ///
    /// The runtime suppresses both the widget value and any value authored by
    /// JavaScript when this is set. This lets custom secret controls opt into
    /// the same protection without relying on a reserved element tag.
    pub value_is_sensitive: bool,
    /// Disabled state when it cannot be inferred from the host element.
    pub disabled: Option<bool>,
}

/// Selection owned by a native widget but synchronized to its JavaScript owner.
///
/// Offsets use UTF-16 code units so JavaScript can consume them without
/// converting from Rust byte offsets.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct WidgetTextSelection {
    /// Fixed end of the selection in UTF-16 code units.
    pub anchor: usize,
    /// Moving end (and caret position) in UTF-16 code units.
    pub head: usize,
    /// Selected text, or `None` for a collapsed selection.
    pub text: Option<String>,
    /// Gesture granularity reported to JavaScript.
    pub kind: WidgetTextSelectionKind,
}

/// Snapshot exposed by a focused native text widget to the platform IME.
///
/// This is deliberately backend-neutral. JavaScript-facing ranges use UTF-16
/// code units, while `surrounding_*` offsets use UTF-8 bytes as required by
/// Winit's platform IME adapters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WidgetImeState {
    /// Local area which the candidate window should avoid.
    pub cursor_area: [f32; 4],
    /// Committed text around the caret, excluding an active preedit range.
    pub surrounding_text: String,
    /// Caret byte offset within `surrounding_text`.
    pub surrounding_cursor: usize,
    /// Selection anchor byte offset within `surrounding_text`.
    pub surrounding_anchor: usize,
    /// Current selection in UTF-16 code units.
    pub selection_utf16: std::ops::Range<usize>,
    /// Whether the moving end precedes the fixed end.
    pub selection_reversed: bool,
    /// Active preedit range in UTF-16 code units.
    pub marked_range_utf16: Option<std::ops::Range<usize>>,
    /// Semantic input purpose used by the platform keyboard/IME.
    pub purpose: WidgetImePurpose,
    /// Whether the editor accepts line breaks.
    pub multiline: bool,
    /// Whether completion suggestions are desirable.
    pub completion: bool,
    /// Whether spelling corrections are desirable.
    pub spellcheck: bool,
}

/// Backend-neutral text-input purpose.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WidgetImePurpose {
    /// General text entry.
    #[default]
    Normal,
    /// Sensitive password entry.
    Password,
    /// Terminal command entry.
    Terminal,
    /// Signed or decimal number entry.
    Number,
    /// Telephone number entry.
    Phone,
    /// URL entry.
    Url,
    /// Email-address entry.
    Email,
    /// Sensitive numeric PIN entry.
    Pin,
    /// Calendar date entry.
    Date,
    /// Time-of-day entry.
    Time,
    /// Combined date and time entry.
    DateTime,
}

/// Granularity used to create a native widget selection.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetTextSelectionKind {
    /// Character/caret selection.
    Simple,
    /// Word selection, normally produced by a double click.
    Word,
    /// Logical-line selection, normally produced by a triple click.
    Line,
}

impl WidgetTextSelectionKind {
    /// Stable event payload spelling.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Word => "word",
            Self::Line => "line",
        }
    }
}
