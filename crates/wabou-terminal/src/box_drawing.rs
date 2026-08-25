//! Cell-aligned rendering for box-drawing glyphs.
//!
//! Font rasterizers cannot guarantee that a box glyph reaches the exact edge of
//! its terminal cell at every font size and device scale. Keep the geometry in
//! logical cell coordinates and snap its stroke bounds to physical pixels.

use anyrender::{PaintScene, Scene};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{Color, Fill};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Arms {
    left: u8,
    right: u8,
    top: u8,
    bottom: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct BoxGeometry {
    rects: [Option<Rect>; 4],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BoxCell {
    pub column: usize,
    pub row: usize,
    pub width: f32,
    pub height: f32,
    pub device_scale: f64,
}

/// Draw the common light/heavy box-drawing set, returning `false` when the
/// character should fall back to the configured font.
pub(super) fn draw_box_drawing(
    scene: &mut Scene,
    character: char,
    cell: BoxCell,
    color: Color,
) -> bool {
    let Some(geometry) = box_geometry(character, cell) else {
        return false;
    };

    for rect in geometry.rects.into_iter().flatten() {
        scene.fill(Fill::NonZero, Affine::IDENTITY, color, None, &rect);
    }
    true
}

fn box_geometry(character: char, cell: BoxCell) -> Option<BoxGeometry> {
    let arms = arms(character)?;
    let scale = cell.device_scale.max(f64::EPSILON);
    let x0 = snap(cell.column as f64 * f64::from(cell.width), scale);
    let y0 = snap(cell.row as f64 * f64::from(cell.height), scale);
    let x1 = snap((cell.column + 1) as f64 * f64::from(cell.width), scale);
    let y1 = snap((cell.row + 1) as f64 * f64::from(cell.height), scale);
    let cx = snap((x0 + x1) * 0.5, scale);
    let cy = snap((y0 + y1) * 0.5, scale);

    let mut rects = [None; 4];
    if arms.left != 0 {
        let (top, bottom) = stroke_bounds(cy, arms.left, scale);
        rects[0] = Some(Rect::new(x0, top, cx, bottom));
    }
    if arms.right != 0 {
        let (top, bottom) = stroke_bounds(cy, arms.right, scale);
        rects[1] = Some(Rect::new(cx, top, x1, bottom));
    }
    if arms.top != 0 {
        let (left, right) = stroke_bounds(cx, arms.top, scale);
        rects[2] = Some(Rect::new(left, y0, right, cy));
    }
    if arms.bottom != 0 {
        let (left, right) = stroke_bounds(cx, arms.bottom, scale);
        rects[3] = Some(Rect::new(left, cy, right, y1));
    }
    Some(BoxGeometry { rects })
}

fn snap(value: f64, scale: f64) -> f64 {
    (value * scale).round() / scale
}

fn stroke_bounds(center: f64, weight: u8, scale: f64) -> (f64, f64) {
    let pixels = usize::from(weight).max(1);
    let start = ((center * scale) - pixels as f64 * 0.5).floor() / scale;
    (start, start + pixels as f64 / scale)
}

fn arms(character: char) -> Option<Arms> {
    let light = 1;
    let heavy = 2;
    // These direction tables are adapted from Alacritty's Apache-2.0
    // built-in font. Values encode light/heavy stroke width for each arm.
    let left = match character {
        '─' | '┐' | '┒' | '┘' | '┚' | '┤' | '┦' | '┧' | '┨' | '┬' | '┮' | '┰' | '┲' | '┴' | '┶'
        | '┸' | '┺' | '┼' | '┾' | '╀' | '╁' | '╂' | '╄' | '╆' | '╊' | '╴' | '╼' => {
            light
        }
        '━' | '┑' | '┓' | '┙' | '┛' | '┥' | '┩' | '┪' | '┫' | '┭' | '┯' | '┱' | '┳' | '┵' | '┷'
        | '┹' | '┻' | '┽' | '┿' | '╃' | '╅' | '╇' | '╈' | '╉' | '╋' | '╸' | '╾' => {
            heavy
        }
        _ => 0,
    };
    let right = match character {
        '─' | '┌' | '┎' | '└' | '┖' | '├' | '┞' | '┟' | '┠' | '┬' | '┭' | '┰' | '┱' | '┴' | '┵'
        | '┸' | '┹' | '┼' | '┽' | '╀' | '╁' | '╂' | '╃' | '╅' | '╉' | '╶' | '╾' => {
            light
        }
        '━' | '┍' | '┏' | '┕' | '┗' | '┝' | '┡' | '┢' | '┣' | '┮' | '┯' | '┲' | '┳' | '┶' | '┷'
        | '┺' | '┻' | '┾' | '┿' | '╄' | '╆' | '╇' | '╈' | '╊' | '╋' | '╺' | '╼' => {
            heavy
        }
        _ => 0,
    };
    let top = match character {
        '│' | '└' | '┕' | '┘' | '┙' | '├' | '┝' | '┟' | '┢' | '┤' | '┥' | '┧' | '┪' | '┴' | '┵'
        | '┶' | '┷' | '┼' | '┽' | '┾' | '┿' | '╁' | '╅' | '╆' | '╈' | '╵' | '╽' => {
            light
        }
        '┃' | '┖' | '┗' | '┚' | '┛' | '┞' | '┠' | '┡' | '┣' | '┦' | '┨' | '┩' | '┫' | '┸' | '┹'
        | '┺' | '┻' | '╀' | '╂' | '╃' | '╄' | '╇' | '╉' | '╊' | '╋' | '╹' | '╿' => {
            heavy
        }
        _ => 0,
    };
    let bottom = match character {
        '│' | '┌' | '┍' | '┐' | '┑' | '├' | '┝' | '┞' | '┡' | '┤' | '┥' | '┦' | '┩' | '┬' | '┭'
        | '┮' | '┯' | '┼' | '┽' | '┾' | '┿' | '╀' | '╃' | '╄' | '╇' | '╷' | '╿' => {
            light
        }
        '┃' | '┎' | '┏' | '┒' | '┓' | '┟' | '┠' | '┢' | '┣' | '┧' | '┨' | '┪' | '┫' | '┰' | '┱'
        | '┲' | '┳' | '╁' | '╂' | '╅' | '╆' | '╈' | '╉' | '╊' | '╋' | '╻' | '╽' => {
            heavy
        }
        _ => 0,
    };

    let result = Arms {
        left,
        right,
        top,
        bottom,
    };
    (result != Arms::default()).then_some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_box_characters_have_expected_arms() {
        assert_eq!(
            arms('─'),
            Some(Arms {
                left: 1,
                right: 1,
                top: 0,
                bottom: 0
            })
        );
        assert_eq!(
            arms('│'),
            Some(Arms {
                left: 0,
                right: 0,
                top: 1,
                bottom: 1
            })
        );
        assert_eq!(
            arms('┌'),
            Some(Arms {
                left: 0,
                right: 1,
                top: 0,
                bottom: 1
            })
        );
        assert_eq!(
            arms('╋'),
            Some(Arms {
                left: 2,
                right: 2,
                top: 2,
                bottom: 2
            })
        );
        assert_eq!(arms('A'), None);
    }

    #[test]
    fn stroke_bounds_are_pixel_aligned_at_fractional_scale() {
        let (start, end) = stroke_bounds(4.25, 1, 2.0);
        assert_eq!((start * 2.0, end * 2.0), (8.0, 9.0));
    }

    #[test]
    fn adjacent_cells_share_the_exact_box_drawing_edge() {
        let cell = |column| BoxCell {
            column,
            row: 0,
            width: 8.4,
            height: 20.0,
            device_scale: 1.5,
        };
        let first = box_geometry('─', cell(0)).unwrap();
        let second = box_geometry('─', cell(1)).unwrap();
        let first_right = first.rects[1].unwrap();
        let second_left = second.rects[0].unwrap();

        assert_eq!(first_right.x1, second_left.x0);
        assert_eq!(
            (first_right.y0, first_right.y1),
            (second_left.y0, second_left.y1)
        );
        assert_eq!(first_right.x1 * 1.5, (first_right.x1 * 1.5).round());
    }
}
