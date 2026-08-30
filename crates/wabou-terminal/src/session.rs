//! Backend-neutral results produced by the terminal session state machine.

/// An event emitted by the terminal session toward its JavaScript owner.
///
/// The session owns this DTO so it does not depend on a particular native
/// widget backend. Adapters translate it into their backend event envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalNodeEvent {
    pub kind: TerminalEventKind,
    pub json: String,
}

impl TerminalNodeEvent {
    pub(super) fn json(kind: TerminalEventKind, json: impl Into<String>) -> Self {
        Self {
            kind,
            json: json.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalEventKind {
    Exit,
    Progress,
    Notification,
    TitleChange,
    CurrentDirectoryChange,
    SelectionChange,
    Bell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum TerminalInputResult {
    Ignored,
    Handled,
    HandledConsumingText,
    Clipboard(wabou_shell_api::ClipboardRequest),
}

impl TerminalInputResult {
    pub(super) const fn is_handled(&self) -> bool {
        !matches!(self, Self::Ignored)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct TerminalInvalidation {
    pub(super) measure: bool,
    pub(super) redraw: bool,
}

impl TerminalInvalidation {
    pub(super) const REDRAW: Self = Self {
        measure: false,
        redraw: true,
    };
    pub(super) const MEASURE_AND_REDRAW: Self = Self {
        measure: true,
        redraw: true,
    };
}
