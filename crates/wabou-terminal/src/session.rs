//! Backend-neutral results produced by the terminal session state machine.

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
