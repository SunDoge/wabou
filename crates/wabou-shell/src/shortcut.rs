//! Platform-aware shortcut matching layered over raw keyboard input.

use crate::{KeyEvent, KeyPhase};

/// Editing shortcuts whose physical bindings follow platform conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardShortcut {
    Copy,
    Cut,
    Paste,
    SelectAll,
}

impl KeyEvent {
    /// Match a platform-standard editing shortcut. Physical combinations such
    /// as terminal Control+C should continue to inspect `modifiers` directly.
    pub fn matches_standard_shortcut(&self, shortcut: StandardShortcut) -> bool {
        let key = match shortcut {
            StandardShortcut::Copy => "c",
            StandardShortcut::Cut => "x",
            StandardShortcut::Paste => "v",
            StandardShortcut::SelectAll => "a",
        };
        self.phase == KeyPhase::Down
            && self.modifiers.primary_shortcut()
            && !self.modifiers.shift()
            && self.key.eq_ignore_ascii_case(key)
    }
}

#[cfg(test)]
mod tests {
    use super::StandardShortcut;
    use crate::{KeyEvent, KeyLocation, KeyPhase, Modifiers};

    #[test]
    fn standard_shortcuts_use_the_platform_primary_modifier() {
        let primary = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        };
        let event = KeyEvent {
            phase: KeyPhase::Down,
            key: "C".into(),
            key_without_modifiers: "c".into(),
            code: "KeyC".into(),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers: primary,
            repeat: false,
        };

        assert!(event.matches_standard_shortcut(StandardShortcut::Copy));
        assert!(!event.matches_standard_shortcut(StandardShortcut::Paste));

        let mut shifted = event;
        shifted.modifiers |= Modifiers::SHIFT;
        assert!(!shifted.matches_standard_shortcut(StandardShortcut::Copy));
    }
}
