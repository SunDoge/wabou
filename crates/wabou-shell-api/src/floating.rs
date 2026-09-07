//! Backend-neutral anchored floating-surface geometry.

use serde::Deserialize;

/// Complete positioning request authored by the JavaScript overlay primitive.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FloatingPosition {
    /// Node or viewport point used as the positioning anchor.
    pub anchor: FloatingAnchor,
    /// Preferred side and alignment.
    #[serde(default)]
    pub placement: FloatingPlacement,
    /// Gap between the anchor and floating surface.
    #[serde(default = "default_offset")]
    pub offset: f32,
    /// Minimum distance from the viewport edge.
    #[serde(default = "default_margin")]
    pub margin: f32,
}

impl FloatingPosition {
    /// Parse and validate the private renderer attribute payload.
    pub fn parse(value: &str) -> Option<Self> {
        let position: Self = serde_json::from_str(value).ok()?;
        let valid_anchor = match position.anchor {
            FloatingAnchor::Node { id } => id.lo != 0 && id.hi != 0,
            FloatingAnchor::Point { x, y } => x.is_finite() && y.is_finite(),
        };
        (valid_anchor
            && position.offset.is_finite()
            && position.margin.is_finite()
            && position.margin >= 0.0)
            .then_some(position)
    }
}

/// Anchor used by a floating surface.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum FloatingAnchor {
    /// Retained Wabou node identity.
    Node {
        /// Full generational node key.
        id: FloatingNodeKey,
    },
    /// Logical viewport coordinate, used by context menus.
    Point {
        /// Horizontal logical coordinate.
        x: f32,
        /// Vertical logical coordinate.
        y: f32,
    },
}

/// Wire-safe generational node identity.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FloatingNodeKey {
    /// Slot index.
    pub lo: u32,
    /// Slot generation.
    pub hi: u32,
}

/// Requested side and alignment of a floating surface.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FloatingPlacement {
    /// Centered above the anchor.
    Top,
    /// Above, aligned to the leading edge.
    TopStart,
    /// Above, aligned to the trailing edge.
    TopEnd,
    /// Centered below the anchor.
    Bottom,
    /// Below, aligned to the leading edge.
    #[default]
    BottomStart,
    /// Below, aligned to the trailing edge.
    BottomEnd,
    /// Centered to the left of the anchor.
    Left,
    /// Left, aligned to the leading edge.
    LeftStart,
    /// Left, aligned to the trailing edge.
    LeftEnd,
    /// Centered to the right of the anchor.
    Right,
    /// Right, aligned to the leading edge.
    RightStart,
    /// Right, aligned to the trailing edge.
    RightEnd,
}

impl FloatingPlacement {
    /// Resolve the side and cross-axis alignment encoded by this placement.
    pub const fn side_align(self) -> (FloatingSide, FloatingAlign) {
        match self {
            Self::Top => (FloatingSide::Top, FloatingAlign::Center),
            Self::TopStart => (FloatingSide::Top, FloatingAlign::Start),
            Self::TopEnd => (FloatingSide::Top, FloatingAlign::End),
            Self::Bottom => (FloatingSide::Bottom, FloatingAlign::Center),
            Self::BottomStart => (FloatingSide::Bottom, FloatingAlign::Start),
            Self::BottomEnd => (FloatingSide::Bottom, FloatingAlign::End),
            Self::Left => (FloatingSide::Left, FloatingAlign::Center),
            Self::LeftStart => (FloatingSide::Left, FloatingAlign::Start),
            Self::LeftEnd => (FloatingSide::Left, FloatingAlign::End),
            Self::Right => (FloatingSide::Right, FloatingAlign::Center),
            Self::RightStart => (FloatingSide::Right, FloatingAlign::Start),
            Self::RightEnd => (FloatingSide::Right, FloatingAlign::End),
        }
    }
}

/// Physical side selected after collision handling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatingSide {
    /// Above the anchor.
    Top,
    /// Below the anchor.
    Bottom,
    /// Left of the anchor.
    Left,
    /// Right of the anchor.
    Right,
}

/// Cross-axis alignment relative to the anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatingAlign {
    /// Leading edges align.
    Start,
    /// Centers align.
    Center,
    /// Trailing edges align.
    End,
}

/// Final viewport-relative floating geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatingResolvedPosition {
    /// Top-left logical viewport coordinate.
    pub origin: [f32; 2],
    /// Side selected after flipping.
    pub side: FloatingSide,
}

const fn default_offset() -> f32 {
    6.0
}

const fn default_margin() -> f32 {
    8.0
}

/// Resolve side flipping, cross-axis alignment, and viewport clamping.
pub fn resolve_floating_position(
    position: FloatingPosition,
    anchor: [f32; 4],
    popup_size: [f32; 2],
    viewport_size: [f32; 2],
) -> FloatingResolvedPosition {
    let (preferred, align) = position.placement.side_align();
    let margin = position.margin;
    let right_limit = (viewport_size[0] - margin).max(margin);
    let bottom_limit = (viewport_size[1] - margin).max(margin);
    let available_left = (anchor[0] - margin).max(0.0);
    let available_right = (right_limit - anchor[2]).max(0.0);
    let available_above = (anchor[1] - margin).max(0.0);
    let available_below = (bottom_limit - anchor[3]).max(0.0);
    let side = match preferred {
        FloatingSide::Right if popup_size[0] <= available_right => FloatingSide::Right,
        FloatingSide::Right if popup_size[0] <= available_left => FloatingSide::Left,
        FloatingSide::Right if available_right >= available_left => FloatingSide::Right,
        FloatingSide::Right => FloatingSide::Left,
        FloatingSide::Left if popup_size[0] <= available_left => FloatingSide::Left,
        FloatingSide::Left if popup_size[0] <= available_right => FloatingSide::Right,
        FloatingSide::Left if available_left >= available_right => FloatingSide::Left,
        FloatingSide::Left => FloatingSide::Right,
        FloatingSide::Bottom if popup_size[1] <= available_below => FloatingSide::Bottom,
        FloatingSide::Bottom if popup_size[1] <= available_above => FloatingSide::Top,
        FloatingSide::Bottom if available_below >= available_above => FloatingSide::Bottom,
        FloatingSide::Bottom => FloatingSide::Top,
        FloatingSide::Top if popup_size[1] <= available_above => FloatingSide::Top,
        FloatingSide::Top if popup_size[1] <= available_below => FloatingSide::Bottom,
        FloatingSide::Top if available_below >= available_above => FloatingSide::Bottom,
        FloatingSide::Top => FloatingSide::Top,
    };
    let aligned_x = match align {
        FloatingAlign::Start => anchor[0],
        FloatingAlign::Center => (anchor[0] + anchor[2] - popup_size[0]) * 0.5,
        FloatingAlign::End => anchor[2] - popup_size[0],
    };
    let aligned_y = match align {
        FloatingAlign::Start => anchor[1],
        FloatingAlign::Center => (anchor[1] + anchor[3] - popup_size[1]) * 0.5,
        FloatingAlign::End => anchor[3] - popup_size[1],
    };
    let mut origin = match side {
        FloatingSide::Top => [aligned_x, anchor[1] - popup_size[1] - position.offset],
        FloatingSide::Bottom => [aligned_x, anchor[3] + position.offset],
        FloatingSide::Left => [anchor[0] - popup_size[0] - position.offset, aligned_y],
        FloatingSide::Right => [anchor[2] + position.offset, aligned_y],
    };
    origin[0] = origin[0].min(right_limit - popup_size[0]).max(margin);
    origin[1] = origin[1].min(bottom_limit - popup_size[1]).max(margin);
    FloatingResolvedPosition { origin, side }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_places_a_bottom_start_popup() {
        let position = FloatingPosition::parse(
            r#"{"anchor":{"kind":"node","id":{"lo":19,"hi":7}},"placement":"bottom-start","offset":6,"margin":8}"#,
        )
        .unwrap();
        assert_eq!(
            resolve_floating_position(
                position,
                [522.0, 270.0, 810.0, 302.0],
                [288.0, 186.0],
                [1259.0, 883.0],
            ),
            FloatingResolvedPosition {
                origin: [522.0, 308.0],
                side: FloatingSide::Bottom,
            }
        );
    }

    #[test]
    fn flips_and_clamps_near_the_viewport_edge() {
        let position = FloatingPosition::parse(
            r#"{"anchor":{"kind":"point","x":390,"y":290},"placement":"bottom-end","offset":6,"margin":8}"#,
        )
        .unwrap();
        let resolved = resolve_floating_position(
            position,
            [390.0, 290.0, 390.0, 290.0],
            [180.0, 120.0],
            [400.0, 300.0],
        );
        assert_eq!(resolved.side, FloatingSide::Top);
        assert_eq!(resolved.origin, [210.0, 164.0]);
    }

    #[test]
    fn rejects_invalid_wire_values() {
        assert!(
            FloatingPosition::parse(r#"{"anchor":{"kind":"node","id":{"lo":0,"hi":1}}}"#).is_none()
        );
        assert!(FloatingPosition::parse(r#"{"anchor":{"kind":"point","x":null,"y":1}}"#).is_none());
    }
}
