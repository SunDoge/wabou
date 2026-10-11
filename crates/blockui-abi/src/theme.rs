//! Shared, dependency-free theme recipes. Colors are RGBA8, dimensions logical px.
//! Only scalar scope IDs cross the actor ABI; resolved recipes belong to the host.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum ThemeMode {
    #[default]
    Inherit = 0,
    Light = 1,
    Dark = 2,
    System = 3,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum Density {
    #[default]
    Inherit = 0,
    Comfortable = 1,
    Compact = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub canvas: u32,
    pub panel: u32,
    pub muted: u32,
    pub text: u32,
    pub text_muted: u32,
    pub border: u32,
    pub accent: u32,
    pub on_accent: u32,
    pub accent_hover: u32,
    pub accent_pressed: u32,
    pub hover: u32,
    pub pressed: u32,
    pub danger: u32,
    pub on_danger: u32,
    pub danger_hover: u32,
    pub danger_pressed: u32,
    pub focus: u32,
}
impl Palette {
    pub const LIGHT: Self = Self {
        canvas: 0xf7f8f5ff,
        panel: 0xffffffff,
        muted: 0xf0f2edff,
        text: 0x252c27ff,
        text_muted: 0x69746bff,
        border: 0xdce2daff,
        accent: 0x285e49ff,
        on_accent: 0xffffffff,
        accent_hover: 0x214f3dff,
        accent_pressed: 0x193e30ff,
        hover: 0xe9eee7ff,
        pressed: 0xdde5daff,
        danger: 0xb53b35ff,
        on_danger: 0xffffffff,
        danger_hover: 0x9d302bff,
        danger_pressed: 0x852722ff,
        focus: 0x39795bff,
    };
    pub const DARK: Self = Self {
        canvas: 0x171c19ff,
        panel: 0x202722ff,
        muted: 0x29322cff,
        text: 0xe8eee7ff,
        text_muted: 0xa2b0a4ff,
        border: 0x3c493fff,
        accent: 0x98d5aeff,
        on_accent: 0x17291cff,
        accent_hover: 0xafe4c2ff,
        accent_pressed: 0xc3efd1ff,
        hover: 0x303d33ff,
        pressed: 0x3c4c3fff,
        danger: 0xf08b82ff,
        on_danger: 0x351715ff,
        danger_hover: 0xf5a198ff,
        danger_pressed: 0xfab7afff,
        focus: 0xa5dfb9ff,
    };
    pub const fn resolve(mode: u32, system_dark: bool) -> Self {
        if mode == 2 || (mode == 3 && system_dark) {
            Self::DARK
        } else {
            Self::LIGHT
        }
    }
    /// Stable names used by the browser adapter to publish inherited CSS variables.
    pub fn colors(self) -> [(&'static str, u32); 17] {
        [
            ("canvas", self.canvas),
            ("panel", self.panel),
            ("muted", self.muted),
            ("text", self.text),
            ("text-muted", self.text_muted),
            ("border", self.border),
            ("accent", self.accent),
            ("on-accent", self.on_accent),
            ("accent-hover", self.accent_hover),
            ("accent-pressed", self.accent_pressed),
            ("hover", self.hover),
            ("pressed", self.pressed),
            ("danger", self.danger),
            ("on-danger", self.on_danger),
            ("danger-hover", self.danger_hover),
            ("danger-pressed", self.danger_pressed),
            ("focus", self.focus),
        ]
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    pub control_height: f32,
    pub control_padding_x: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub control_radius: f32,
    pub panel_radius: f32,
}
impl Metrics {
    pub const fn resolve(density: u32) -> Self {
        Self {
            control_height: if density == 2 { 30.0 } else { 36.0 },
            control_padding_x: if density == 2 { 9.0 } else { 12.0 },
            font_size: 14.0,
            line_height: 20.0,
            control_radius: 6.0,
            panel_radius: 10.0,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn presentation_roundtrip_preserves_scope_ids_and_rejects_old_wire_size() {
        let presentation = crate::Presentation {
            fill_height: 1,
            theme: 3,
            density: 2,
            font_size: 14.0,
            ..Default::default()
        };
        assert_eq!(
            crate::Presentation::decode(&presentation.encode()),
            Some(presentation)
        );
        assert!(crate::Presentation::decode(&[0; 24]).is_none());
        assert!(crate::Presentation::decode(&[0; 32]).is_none());
    }
}
