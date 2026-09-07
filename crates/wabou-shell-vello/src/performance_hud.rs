//! Renderer-native performance diagnostics drawn outside the application tree.

use std::{sync::Arc, time::Instant};

use vello_common::{
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};

use crate::{FrameStats, PaintScene, Scene, TextContext};

const PANEL_WIDTH: f64 = 276.0;
const PANEL_HEIGHT: f64 = 112.0;
const PANEL_MARGIN: f64 = 12.0;

pub(crate) struct PerformanceHud {
    enabled: bool,
    last_frame: Option<Instant>,
    fps: f64,
}

impl PerformanceHud {
    pub(crate) fn from_environment() -> Self {
        Self::new(std::env::var("WABOU_PERFORMANCE_HUD").is_ok_and(|value| parse_enabled(&value)))
    }

    fn new(enabled: bool) -> Self {
        Self {
            enabled,
            last_frame: None,
            fps: 0.0,
        }
    }

    pub(crate) fn begin_frame(&mut self, now: Instant) {
        if !self.enabled {
            return;
        }
        if let Some(previous) = self.last_frame {
            let seconds = now.duration_since(previous).as_secs_f64();
            if seconds > 0.25 {
                // Wabou renders on demand. A long gap means the window was
                // idle, not that it was presenting at a very low frame rate.
                self.fps = 0.0;
            } else if seconds > 0.0 {
                let sample = (1.0 / seconds).min(999.0);
                self.fps = if self.fps == 0.0 {
                    sample
                } else {
                    self.fps * 0.85 + sample * 0.15
                };
            }
        }
        self.last_frame = Some(now);
    }

    pub(crate) fn paint(
        &self,
        scene: &mut Scene,
        text: &mut TextContext,
        stats: FrameStats,
        viewport: [u32; 2],
        scale: f64,
    ) {
        if !self.enabled
            || f64::from(viewport[0]) < PANEL_WIDTH + PANEL_MARGIN * 2.0
            || f64::from(viewport[1]) < PANEL_HEIGHT + PANEL_MARGIN * 2.0
        {
            return;
        }
        let left = f64::from(viewport[0]) - PANEL_WIDTH - PANEL_MARGIN;
        let top = PANEL_MARGIN;
        let panel = Rect::new(left, top, left + PANEL_WIDTH, top + PANEL_HEIGHT);
        let device = Affine::scale(scale);
        scene.draw_box_shadow(device, panel, Color::from_rgba8(15, 23, 42, 70), 10.0, 10.0);
        scene.fill(
            Fill::NonZero,
            device,
            Color::from_rgba8(15, 23, 42, 238),
            None,
            &panel.to_rounded_rect(9.0),
        );
        scene.stroke(
            &vello_common::kurbo::Stroke::new(1.0),
            device,
            Color::from_rgba8(71, 85, 105, 210),
            None,
            &panel.to_rounded_rect(9.0),
        );

        let total = stats.build_frame_ms + stats.scene_ms + stats.present_ms;
        let health = health_color(total);
        scene.fill(
            Fill::NonZero,
            device,
            health,
            None,
            &Rect::new(left + 14.0, top + 15.0, left + 18.0, top + 31.0).to_rounded_rect(2.0),
        );

        self.text(
            scene,
            text,
            "Wabou HUD",
            [left + 26.0, top + 13.0],
            12.0,
            600.0,
            [248, 250, 252, 255],
            scale,
        );
        let cadence = if self.fps == 0.0 {
            "idle".to_owned()
        } else {
            format!("{:>3.0} fps", self.fps)
        };
        self.text(
            scene,
            text,
            &cadence,
            [left + 224.0, top + 13.0],
            12.0,
            600.0,
            [248, 250, 252, 255],
            scale,
        );
        self.text(
            scene,
            text,
            &format!(
                "frame {:>6.2} ms        nodes {:>6}",
                total, stats.node_count
            ),
            [left + 14.0, top + 40.0],
            11.0,
            500.0,
            health.to_rgba8().to_u8_array(),
            scale,
        );
        self.text(
            scene,
            text,
            &format!(
                "JS {:>6.2}   build {:>6.2}   scene {:>6.2}",
                stats.js_tick_ms, stats.build_frame_ms, stats.scene_ms
            ),
            [left + 14.0, top + 62.0],
            11.0,
            400.0,
            [203, 213, 225, 255],
            scale,
        );
        self.text(
            scene,
            text,
            &format!(
                "present {:>6.2} ms       viewport {}×{}",
                stats.present_ms, stats.viewport_w, stats.viewport_h
            ),
            [left + 14.0, top + 82.0],
            11.0,
            400.0,
            [148, 163, 184, 255],
            scale,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn text(
        &self,
        scene: &mut Scene,
        text: &mut TextContext,
        value: &str,
        origin: [f64; 2],
        size: f32,
        weight: f32,
        color: [u8; 4],
        scale: f64,
    ) {
        let layout = crate::text::layout_text_styled(
            text,
            Arc::from(value),
            size,
            weight,
            false,
            None,
            crate::style::TextAlign::Start,
            color,
            Arc::from([]),
            None,
            None,
        );
        let device = Affine::scale(scale);
        crate::scene::draw_text_layout_into(
            scene,
            text,
            &layout,
            device,
            device * Affine::translate((origin[0], origin[1])),
            scale,
        );
    }
}

fn health_color(milliseconds: f64) -> Color {
    if milliseconds <= 0.0 || !milliseconds.is_finite() {
        Color::from_rgb8(148, 163, 184)
    } else if milliseconds < 8.0 {
        Color::from_rgb8(74, 222, 128)
    } else if milliseconds < 16.7 {
        Color::from_rgb8(251, 191, 36)
    } else {
        Color::from_rgb8(248, 113, 113)
    }
}

fn parse_enabled(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_flag_and_health_thresholds_are_explicit() {
        for value in ["1", "true", "TRUE", " yes ", "on"] {
            assert!(parse_enabled(value));
        }
        for value in ["", "0", "false", "enabled"] {
            assert!(!parse_enabled(value));
        }
        assert_eq!(health_color(4.0), Color::from_rgb8(74, 222, 128));
        assert_eq!(health_color(12.0), Color::from_rgb8(251, 191, 36));
        assert_eq!(health_color(20.0), Color::from_rgb8(248, 113, 113));

        let start = Instant::now();
        let mut hud = PerformanceHud::new(true);
        hud.begin_frame(start);
        hud.begin_frame(start + std::time::Duration::from_millis(16));
        assert!(hud.fps > 60.0);
        hud.begin_frame(start + std::time::Duration::from_secs(1));
        assert_eq!(hud.fps, 0.0);
    }

    #[test]
    fn hud_paints_outside_the_application_scene_when_enabled() {
        let mut scene = Scene::new();
        let mut text = TextContext::new();
        PerformanceHud::new(false).paint(
            &mut scene,
            &mut text,
            FrameStats::default(),
            [640, 160],
            1.0,
        );
        assert!(scene.commands.is_empty());

        PerformanceHud::new(true).paint(
            &mut scene,
            &mut text,
            FrameStats {
                build_frame_ms: 2.0,
                scene_ms: 1.0,
                present_ms: 2.0,
                node_count: 42,
                viewport_w: 640,
                viewport_h: 480,
                ..FrameStats::default()
            },
            [640, 160],
            1.0,
        );
        assert!(scene.commands.len() >= 8);
        let image = crate::renderer::render_to_image(&scene, 640, 160, Color::BLACK).unwrap();
        let panel = image.get_pixel(500, 52).0;
        assert_ne!(panel, [0, 0, 0, 255]);
        assert!(panel[2] > panel[0], "expected slate HUD surface: {panel:?}");

        let mut tiny_scene = Scene::new();
        PerformanceHud::new(true).paint(
            &mut tiny_scene,
            &mut text,
            FrameStats::default(),
            [240, 120],
            1.0,
        );
        assert!(tiny_scene.commands.is_empty());
    }
}
