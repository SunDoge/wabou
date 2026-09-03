//! Platform-aware shortcut matching layered over raw keyboard input.

#![warn(missing_docs)]

pub use wabou_shell_api::StandardShortcut;

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
            synthetic: false,
        };

        assert!(event.matches_standard_shortcut(StandardShortcut::Copy));
        assert!(!event.matches_standard_shortcut(StandardShortcut::Paste));

        let mut shifted = event;
        shifted.modifiers |= Modifiers::SHIFT;
        assert!(!shifted.matches_standard_shortcut(StandardShortcut::Copy));
    }

    #[test]
    fn standard_shortcuts_use_the_key_without_modifiers() {
        let primary = if cfg!(target_os = "macos") {
            Modifiers::META
        } else {
            Modifiers::CONTROL
        };
        let event = KeyEvent {
            phase: KeyPhase::Down,
            // Some platform/layout combinations do not expose the printable
            // logical key while Control or Command is held.
            key: "Unidentified".into(),
            key_without_modifiers: "v".into(),
            code: "KeyV".into(),
            text: None,
            text_with_all_modifiers: None,
            location: KeyLocation::Standard,
            modifiers: primary,
            repeat: false,
            synthetic: false,
        };

        assert!(event.matches_standard_shortcut(StandardShortcut::Paste));
    }
}
