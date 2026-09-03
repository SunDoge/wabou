//! Backend-neutral color values used at native application boundaries.

use serde::{Deserialize, Serialize};

/// An sRGB color encoded as eight-bit red, green, blue, and alpha channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RgbaColor {
    /// Red channel.
    pub red: u8,
    /// Green channel.
    pub green: u8,
    /// Blue channel.
    pub blue: u8,
    /// Alpha channel.
    pub alpha: u8,
}

impl RgbaColor {
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::from_rgba8(0, 0, 0, 0);

    /// Construct an opaque sRGB color.
    #[must_use]
    pub const fn from_rgb8(red: u8, green: u8, blue: u8) -> Self {
        Self::from_rgba8(red, green, blue, u8::MAX)
    }

    /// Construct an sRGB color with an explicit alpha channel.
    #[must_use]
    pub const fn from_rgba8(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_preserve_exact_channel_values() {
        assert_eq!(
            RgbaColor::from_rgb8(1, 2, 3),
            RgbaColor::from_rgba8(1, 2, 3, 255)
        );
        assert_eq!(RgbaColor::TRANSPARENT.alpha, 0);
    }
}
