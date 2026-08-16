//! Deterministic native-window lifecycle decisions.
//!
//! The model deliberately knows nothing about winit. Production and test
//! backends consume the same effects, so close-to-tray behavior can be tested
//! without a compositor while platform smoke tests only need to verify the
//! thin effect executor.

#![warn(missing_docs)]

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Platform abilities that affect window lifecycle strategy.
pub struct WindowCapabilities {
    /// An existing native window can be hidden and shown again.
    pub mutable_visibility: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Resource state of one logical Wabou window.
pub enum WindowPresence {
    #[default]
    /// Native window and render surface are visible.
    Visible,
    /// Native window exists but is hidden.
    Hidden,
    /// Logical window exists but its platform surface was released.
    SurfaceReleased,
    /// Logical and native window are permanently closed.
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Application request applied to a logical window.
pub enum WindowIntent {
    /// Hide while preserving the logical window for tray restoration.
    Hide,
    /// Make a hidden/released logical window visible.
    Show,
    /// Permanently close the logical window.
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Minimal platform operation selected by a lifecycle transition.
pub enum WindowEffect {
    /// Toggle visibility without recreating the surface.
    SetVisible(bool),
    /// Drop a surface on platforms where hiding is unreliable.
    ReleaseSurface,
    /// Create a replacement surface for a retained logical window.
    RecreateSurface,
    /// Permanently destroy the native window.
    Close,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Deterministic state machine for close-to-tray and native surface lifetime.
pub struct WindowLifecycle {
    presence: WindowPresence,
    surface_generation: u64,
}

impl WindowLifecycle {
    /// Construct a visible lifecycle with its first surface generation.
    pub fn visible() -> Self {
        Self {
            presence: WindowPresence::Visible,
            surface_generation: 1,
        }
    }

    /// Return the current native-resource presence.
    pub fn presence(self) -> WindowPresence {
        self.presence
    }

    /// Monotonic generation incremented whenever a surface is recreated.
    pub fn surface_generation(self) -> u64 {
        self.surface_generation
    }

    /// Apply an intent and return the platform operation required, if any.
    ///
    /// Repeated or impossible transitions are idempotent and return `None`.
    pub fn transition(
        &mut self,
        intent: WindowIntent,
        capabilities: WindowCapabilities,
    ) -> Option<WindowEffect> {
        match (self.presence, intent) {
            (WindowPresence::Visible, WindowIntent::Hide) if capabilities.mutable_visibility => {
                self.presence = WindowPresence::Hidden;
                Some(WindowEffect::SetVisible(false))
            }
            (WindowPresence::Visible, WindowIntent::Hide) => {
                self.presence = WindowPresence::SurfaceReleased;
                Some(WindowEffect::ReleaseSurface)
            }
            (WindowPresence::Hidden, WindowIntent::Show) => {
                self.presence = WindowPresence::Visible;
                Some(WindowEffect::SetVisible(true))
            }
            (WindowPresence::SurfaceReleased, WindowIntent::Show) => {
                self.presence = WindowPresence::Visible;
                self.surface_generation = self.surface_generation.saturating_add(1);
                Some(WindowEffect::RecreateSurface)
            }
            (WindowPresence::Closed, _) => None,
            (_, WindowIntent::Close) => {
                self.presence = WindowPresence::Closed;
                Some(WindowEffect::Close)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_close_to_tray_releases_and_recreates_the_surface() {
        let mut window = WindowLifecycle::visible();
        let wayland = WindowCapabilities {
            mutable_visibility: false,
        };

        assert_eq!(
            window.transition(WindowIntent::Hide, wayland),
            Some(WindowEffect::ReleaseSurface)
        );
        assert_eq!(window.presence(), WindowPresence::SurfaceReleased);
        assert_eq!(
            window.transition(WindowIntent::Show, wayland),
            Some(WindowEffect::RecreateSurface)
        );
        assert_eq!(window.presence(), WindowPresence::Visible);
        assert_eq!(window.surface_generation(), 2);
    }

    #[test]
    fn mutable_visibility_platforms_reuse_the_surface() {
        let mut window = WindowLifecycle::visible();
        let native_hide = WindowCapabilities {
            mutable_visibility: true,
        };

        assert_eq!(
            window.transition(WindowIntent::Hide, native_hide),
            Some(WindowEffect::SetVisible(false))
        );
        assert_eq!(
            window.transition(WindowIntent::Show, native_hide),
            Some(WindowEffect::SetVisible(true))
        );
        assert_eq!(window.surface_generation(), 1);
    }

    #[test]
    fn repeated_visibility_intents_are_idempotent() {
        let mut window = WindowLifecycle::visible();
        let wayland = WindowCapabilities::default();
        window.transition(WindowIntent::Hide, wayland);
        assert_eq!(window.transition(WindowIntent::Hide, wayland), None);
        window.transition(WindowIntent::Show, wayland);
        assert_eq!(window.transition(WindowIntent::Show, wayland), None);
    }
}
