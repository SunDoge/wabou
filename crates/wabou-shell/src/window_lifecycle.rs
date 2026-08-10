//! Deterministic native-window lifecycle decisions.
//!
//! The model deliberately knows nothing about winit. Production and test
//! backends consume the same effects, so close-to-tray behavior can be tested
//! without a compositor while platform smoke tests only need to verify the
//! thin effect executor.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowCapabilities {
    /// An existing native window can be hidden and shown again.
    pub mutable_visibility: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowPresence {
    #[default]
    Visible,
    Hidden,
    SurfaceReleased,
    Closed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowIntent {
    Hide,
    Show,
    Close,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowEffect {
    SetVisible(bool),
    ReleaseSurface,
    RecreateSurface,
    Close,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowLifecycle {
    presence: WindowPresence,
    surface_generation: u64,
}

impl WindowLifecycle {
    pub fn visible() -> Self {
        Self {
            presence: WindowPresence::Visible,
            surface_generation: 1,
        }
    }

    pub fn presence(self) -> WindowPresence {
        self.presence
    }

    pub fn surface_generation(self) -> u64 {
        self.surface_generation
    }

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
