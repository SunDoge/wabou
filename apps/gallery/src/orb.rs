//! Animated Vello orbs used to demonstrate native procedural painting.

use std::f64::consts::TAU;
use std::time::{Duration, Instant};

use wabou::widget_api::{
    PaintContext, UiEvent, Widget, WidgetChanges, WidgetEventResult,
    vello::{
        kurbo::{Affine, BezPath, Circle, Point, Stroke},
        peniko::{Color, Fill, Gradient},
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum OrbPreset {
    #[default]
    Aurora,
    Plasma,
    Chrome,
}

impl OrbPreset {
    fn parse(value: &str) -> Self {
        match value {
            "plasma" => Self::Plasma,
            "chrome" => Self::Chrome,
            _ => Self::Aurora,
        }
    }
}

/// A lightweight procedural orb painted entirely into Wabou's Vello scene.
pub struct OrbWidget {
    preset: OrbPreset,
    speed: f64,
    epoch: Instant,
    next_frame: Instant,
}

impl Default for OrbWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl OrbWidget {
    /// Creates an animated aurora orb.
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            preset: OrbPreset::default(),
            speed: 1.0,
            epoch: now,
            next_frame: now,
        }
    }

    fn elapsed(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64() * self.speed
    }
}

impl Widget for OrbWidget {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let [width, height] = cx.size();
        let size = f64::from(width.min(height));
        let center = Point::new(f64::from(width) * 0.5, f64::from(height) * 0.5);
        let radius = size * 0.34;
        let time = self.elapsed();

        paint_glow(cx, center, radius, self.preset);
        let silhouette = blob_path(center, radius, time * 0.38, 7, 0.075);
        cx.scene_mut()
            .push_clip_layer(Fill::NonZero, Affine::IDENTITY, &silhouette);
        match self.preset {
            OrbPreset::Aurora => paint_aurora(cx, center, radius, time),
            OrbPreset::Plasma => paint_plasma(cx, center, radius, time),
            OrbPreset::Chrome => paint_chrome(cx, center, radius, time),
        }
        paint_glass(cx, center, radius, time);
        cx.scene_mut().pop_layer();

        cx.scene_mut().stroke(
            &Stroke::new((size * 0.006).max(1.0)),
            Affine::IDENTITY,
            Color::from_rgba8(255, 255, 255, 96),
            None,
            &silhouette,
        );
        self.next_frame = Instant::now() + Duration::from_millis(16);
    }

    fn attribute_changed(&mut self, name: &str, value: &str) -> WidgetChanges {
        match name {
            "preset" => self.preset = OrbPreset::parse(value),
            "speed" => {
                if let Ok(speed) = value.parse::<f64>() {
                    self.speed = speed.clamp(0.0, 4.0);
                }
            }
            _ => return WidgetChanges::empty(),
        }
        WidgetChanges::REDRAW
    }

    fn handle_event(&mut self, _event: &UiEvent) -> WidgetEventResult {
        WidgetEventResult::IGNORED
    }

    fn animation_deadline(&self) -> Option<Instant> {
        (self.speed > 0.0).then_some(self.next_frame)
    }

    fn intrinsic_size(&self) -> Option<[f32; 2]> {
        Some([320.0, 320.0])
    }
}

fn paint_glow(cx: &mut PaintContext<'_>, center: Point, radius: f64, preset: OrbPreset) {
    let glow = match preset {
        OrbPreset::Aurora => Color::from_rgba8(72, 112, 255, 90),
        OrbPreset::Plasma => Color::from_rgba8(233, 61, 255, 88),
        OrbPreset::Chrome => Color::from_rgba8(91, 219, 255, 72),
    };
    let gradient = Gradient::new_radial(center, (radius * 1.42) as f32).with_stops([
        (0.0, glow),
        (0.68, glow),
        (1.0, Color::TRANSPARENT),
    ]);
    cx.scene_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &gradient,
        None,
        &Circle::new(center, radius * 1.42),
    );
}

fn paint_aurora(cx: &mut PaintContext<'_>, center: Point, radius: f64, time: f64) {
    let base = Gradient::new_sweep(center, (time * 0.08) as f32, (TAU + time * 0.08) as f32)
        .with_stops([
            (0.0, Color::from_rgb8(60, 45, 190)),
            (0.25, Color::from_rgb8(41, 205, 255)),
            (0.52, Color::from_rgb8(117, 67, 255)),
            (0.76, Color::from_rgb8(255, 77, 190)),
            (1.0, Color::from_rgb8(60, 45, 190)),
        ]);
    cx.scene_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &base,
        None,
        &Circle::new(center, radius * 1.12),
    );
    for (index, color) in [
        Color::from_rgba8(32, 243, 255, 155),
        Color::from_rgba8(255, 77, 210, 135),
        Color::from_rgba8(139, 92, 246, 170),
    ]
    .into_iter()
    .enumerate()
    {
        let phase = time * (0.7 + index as f64 * 0.13) + index as f64 * 2.1;
        let offset = Point::new(
            center.x + phase.cos() * radius * 0.42,
            center.y + phase.sin() * radius * 0.34,
        );
        let gradient = Gradient::new_radial(offset, (radius * 0.8) as f32)
            .with_stops([(0.0, color), (1.0, Color::TRANSPARENT)]);
        cx.scene_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &gradient,
            None,
            &Circle::new(offset, radius * 0.8),
        );
    }
}

fn paint_plasma(cx: &mut PaintContext<'_>, center: Point, radius: f64, time: f64) {
    let base = Gradient::new_radial(
        Point::new(center.x - radius * 0.28, center.y - radius * 0.34),
        (radius * 1.6) as f32,
    )
    .with_stops([
        (0.0, Color::from_rgb8(255, 197, 245)),
        (0.22, Color::from_rgb8(255, 66, 180)),
        (0.58, Color::from_rgb8(111, 32, 211)),
        (1.0, Color::from_rgb8(18, 7, 60)),
    ]);
    cx.scene_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &base,
        None,
        &Circle::new(center, radius * 1.15),
    );
    for index in 0..5 {
        let phase = time * 0.55 + index as f64 * 1.25;
        let blob_center = Point::new(
            center.x + phase.cos() * radius * 0.32,
            center.y + (phase * 1.37).sin() * radius * 0.38,
        );
        let color = if index % 2 == 0 {
            Color::from_rgba8(255, 218, 252, 118)
        } else {
            Color::from_rgba8(92, 225, 255, 110)
        };
        let gradient = Gradient::new_radial(blob_center, (radius * 0.48) as f32)
            .with_stops([(0.0, color), (1.0, Color::TRANSPARENT)]);
        cx.scene_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &gradient,
            None,
            &Circle::new(blob_center, radius * 0.5),
        );
    }
}

fn paint_chrome(cx: &mut PaintContext<'_>, center: Point, radius: f64, time: f64) {
    let shift = time.sin() * radius * 0.14;
    let gradient = Gradient::new_linear(
        (center.x - radius, center.y - radius + shift),
        (center.x + radius, center.y + radius + shift),
    )
    .with_stops([
        (0.0, Color::from_rgb8(10, 16, 31)),
        (0.15, Color::from_rgb8(209, 237, 255)),
        (0.3, Color::from_rgb8(35, 74, 114)),
        (0.48, Color::from_rgb8(241, 252, 255)),
        (0.63, Color::from_rgb8(37, 179, 211)),
        (0.8, Color::from_rgb8(8, 23, 42)),
        (1.0, Color::from_rgb8(171, 219, 237)),
    ]);
    cx.scene_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &gradient,
        None,
        &Circle::new(center, radius * 1.15),
    );
}

fn paint_glass(cx: &mut PaintContext<'_>, center: Point, radius: f64, time: f64) {
    let highlight_center = Point::new(
        center.x - radius * (0.34 + time.sin() * 0.025),
        center.y - radius * 0.4,
    );
    let highlight = Gradient::new_radial(highlight_center, (radius * 0.82) as f32).with_stops([
        (0.0, Color::from_rgba8(255, 255, 255, 180)),
        (0.22, Color::from_rgba8(255, 255, 255, 54)),
        (1.0, Color::TRANSPARENT),
    ]);
    cx.scene_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &highlight,
        None,
        &Circle::new(highlight_center, radius * 0.82),
    );

    let shade_center = Point::new(center.x + radius * 0.3, center.y + radius * 0.38);
    let shade = Gradient::new_radial(shade_center, (radius * 0.92) as f32).with_stops([
        (0.0, Color::from_rgba8(3, 7, 18, 118)),
        (1.0, Color::TRANSPARENT),
    ]);
    cx.scene_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &shade,
        None,
        &Circle::new(shade_center, radius * 0.92),
    );
}

fn blob_path(center: Point, radius: f64, phase: f64, points: usize, wobble: f64) -> BezPath {
    let samples = (0..points)
        .map(|index| {
            let angle = index as f64 / points as f64 * TAU;
            let modulation = 1.0
                + wobble * (angle * 3.0 + phase).sin()
                + wobble * 0.55 * (angle * 5.0 - phase * 1.3).cos();
            Point::new(
                center.x + angle.cos() * radius * modulation,
                center.y + angle.sin() * radius * modulation,
            )
        })
        .collect::<Vec<_>>();
    let mut path = BezPath::new();
    let first_mid = midpoint(samples[points - 1], samples[0]);
    path.move_to(first_mid);
    for index in 0..points {
        let current = samples[index];
        let next = samples[(index + 1) % points];
        path.quad_to(current, midpoint(current, next));
    }
    path.close_path();
    path
}

fn midpoint(a: Point, b: Point) -> Point {
    Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou::widget_api::vello::kurbo::Shape;

    #[test]
    fn presets_are_explicit_and_unknown_values_fall_back() {
        assert_eq!(OrbPreset::parse("plasma"), OrbPreset::Plasma);
        assert_eq!(OrbPreset::parse("chrome"), OrbPreset::Chrome);
        assert_eq!(OrbPreset::parse("unknown"), OrbPreset::Aurora);
    }

    #[test]
    fn generated_blob_is_closed_and_contains_the_center() {
        let path = blob_path(Point::new(100.0, 100.0), 60.0, 0.5, 7, 0.075);
        assert!(path.contains(Point::new(100.0, 100.0)));
        assert!(path.bounding_box().width() > 100.0);
    }
}
