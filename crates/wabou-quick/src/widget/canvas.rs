//! A simple Canvas widget — the "hello world" of the Widget trait.
//!
//! Paints custom shapes (rect + circle + line) to prove the pipeline:
//! Widget::paint → Scene → build_scene append. No interaction.
//! Registered when the applier encounters `tag == "canvas"`.

use vello::Scene;
use vello::kurbo::{Affine, Circle, Line, Rect, Stroke};
use vello::peniko::{Color, Fill};
use wabou_shell::UiEvent;
use wabou_shell::text::TextContext;

use super::{Widget, WidgetEventResult};

pub struct Canvas;

impl Widget for Canvas {
    fn paint(&mut self, width: f32, height: f32, _tcx: &mut TextContext) -> Scene {
        let mut scene = Scene::new();
        let w = width as f64;
        let h = height as f64;

        // A checkerboard background to show the widget fills the content box.
        let tile = 16.0;
        let cols = (w / tile).ceil() as u32;
        let rows = (h / tile).ceil() as u32;
        for r in 0..rows {
            for c in 0..cols {
                if (r + c) % 2 == 0 {
                    scene.fill(
                        Fill::NonZero,
                        Affine::IDENTITY,
                        Color::from_rgb8(0x1e, 0x29, 0x3b),
                        None,
                        &Rect::new(
                            c as f64 * tile,
                            r as f64 * tile,
                            (c + 1) as f64 * tile,
                            (r + 1) as f64 * tile,
                        ),
                    );
                }
            }
        }

        // A circle in the center.
        let cx = w / 2.0;
        let cy = h / 2.0;
        let radius = w.min(h) * 0.3;
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            Color::from_rgba8(0x63, 0x66, 0xf1, 0x80),
            None,
            &Circle::new((cx, cy), radius),
        );

        // A diagonal line.
        scene.stroke(
            &Stroke::new(2.0),
            Affine::IDENTITY,
            Color::from_rgb8(0x22, 0xd3, 0xee),
            None,
            &Line::new((0.0, 0.0), (w, h)),
        );

        scene
    }

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::IGNORED
    }
}
