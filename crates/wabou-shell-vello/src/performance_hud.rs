//! Renderer-native performance diagnostics drawn outside the application tree.

use std::{sync::Arc, time::Instant};

use vello_common::{
    kurbo::{Affine, Rect},
    peniko::{Color, Fill},
};

use crate::{FrameStats, PaintScene, Scene, TextContext};

const PANEL_WIDTH: f64 = 320.0;
const PANEL_HEIGHT: f64 = 116.0;
const PANEL_MARGIN: f64 = 12.0;
const PRIMARY_TEXT: [u8; 4] = [15, 23, 42, 255];
const SECONDARY_TEXT: [u8; 4] = [71, 85, 105, 255];

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
        scene.draw_box_shadow(device, panel, Color::from_rgba8(15, 23, 42, 42), 8.0, 8.0);
        scene.fill(
            Fill::NonZero,
            device,
            Color::from_rgba8(248, 250, 252, 247),
            None,
            &panel.to_rounded_rect(9.0),
        );
        scene.stroke(
            &vello_common::kurbo::Stroke::new(1.0),
            device,
            Color::from_rgba8(203, 213, 225, 230),
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
            &Rect::new(left + 14.0, top + 16.0, left + 18.0, top + 35.0).to_rounded_rect(2.0),
        );

        self.text(
            scene,
            text,
            "Wabou HUD",
            [left + 26.0, top + 12.0],
            13.5,
            600.0,
            PRIMARY_TEXT,
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
            [left + 268.0, top + 12.0],
            12.5,
            600.0,
            PRIMARY_TEXT,
            scale,
        );

        self.text(
            scene,
            text,
            "Frame",
            [left + 14.0, top + 40.0],
            12.5,
            400.0,
            SECONDARY_TEXT,
            scale,
        );
        self.text(
            scene,
            text,
            &format!("{total:.2} ms"),
            [left + 60.0, top + 40.0],
            12.5,
            600.0,
            health.to_rgba8().to_u8_array(),
            scale,
        );
        self.text(
            scene,
            text,
            "Nodes",
            [left + 188.0, top + 40.0],
            12.5,
            400.0,
            SECONDARY_TEXT,
            scale,
        );
        self.text(
            scene,
            text,
            &stats.node_count.to_string(),
            [left + 234.0, top + 40.0],
            12.5,
            600.0,
            PRIMARY_TEXT,
            scale,
        );

        for (label, value, x) in [
            ("JS", stats.js_tick_ms, 14.0),
            ("Build", stats.build_frame_ms, 95.0),
            ("Scene", stats.scene_ms, 200.0),
        ] {
            self.text(
                scene,
                text,
                label,
                [left + x, top + 64.0],
                12.5,
                400.0,
                SECONDARY_TEXT,
                scale,
            );
            let value_x = x + match label {
                "JS" => 23.0,
                "Build" => 39.0,
                _ => 40.0,
            };
            self.text(
                scene,
                text,
                &format!("{value:.2}"),
                [left + value_x, top + 64.0],
                12.5,
                600.0,
                PRIMARY_TEXT,
                scale,
            );
        }

        self.text(
            scene,
            text,
            "Present",
            [left + 14.0, top + 88.0],
            12.5,
            400.0,
            SECONDARY_TEXT,
            scale,
        );
        self.text(
            scene,
            text,
            &format!("{:.2} ms", stats.present_ms),
            [left + 66.0, top + 88.0],
            12.5,
            600.0,
            PRIMARY_TEXT,
            scale,
        );
        self.text(
            scene,
            text,
            "Viewport",
            [left + 188.0, top + 88.0],
            12.5,
            400.0,
            SECONDARY_TEXT,
            scale,
        );
        self.text(
            scene,
            text,
            &format!("{}×{}", stats.viewport_w, stats.viewport_h),
            [left + 244.0, top + 88.0],
            12.5,
            600.0,
            PRIMARY_TEXT,
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
            Affine::IDENTITY,
            device * Affine::translate((origin[0], origin[1])),
            scale,
        );
    }
}

fn health_color(milliseconds: f64) -> Color {
    if milliseconds <= 0.0 || !milliseconds.is_finite() {
        Color::from_rgb8(148, 163, 184)
    } else if milliseconds < 8.0 {
        Color::from_rgb8(21, 128, 61)
    } else if milliseconds < 16.7 {
        Color::from_rgb8(180, 83, 9)
    } else {
        Color::from_rgb8(185, 28, 28)
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
    use image::GenericImageView;

    #[test]
    fn hud_flag_and_health_thresholds_are_explicit() {
        for value in ["1", "true", "TRUE", " yes ", "on"] {
            assert!(parse_enabled(value));
        }
        for value in ["", "0", "false", "enabled"] {
            assert!(!parse_enabled(value));
        }
        assert_eq!(health_color(4.0), Color::from_rgb8(21, 128, 61));
        assert_eq!(health_color(12.0), Color::from_rgb8(180, 83, 9));
        assert_eq!(health_color(20.0), Color::from_rgb8(185, 28, 28));

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
            [700, 180],
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
            [700, 180],
            1.0,
        );
        assert!(scene.commands.len() >= 20);
        let image = crate::renderer::render_to_image(&scene, 700, 180, Color::BLACK).unwrap();
        let panel = image.get_pixel(500, 52).0;
        assert_ne!(panel, [0, 0, 0, 255]);
        assert!(panel[0] > 220, "expected light HUD surface: {panel:?}");
        let darkest_body_pixel = image
            .view(380, 48, 300, 60)
            .pixels()
            .map(|(_, _, pixel)| {
                u16::from(pixel.0[0]) + u16::from(pixel.0[1]) + u16::from(pixel.0[2])
            })
            .min()
            .unwrap();
        assert!(
            darkest_body_pixel < 350,
            "expected high-contrast rasterized HUD text, got {darkest_body_pixel}"
        );

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

    #[test]
    fn hud_text_uses_the_same_logical_position_at_one_and_two_x() {
        for scale in [1_u32, 2] {
            let mut scene = Scene::new();
            let mut text = TextContext::new();
            PerformanceHud::new(true).paint(
                &mut scene,
                &mut text,
                FrameStats::default(),
                [700, 180],
                f64::from(scale),
            );
            let image =
                crate::renderer::render_to_image(&scene, 700 * scale, 180 * scale, Color::BLACK)
                    .unwrap();
            let header = image.view(390 * scale, 18 * scale, 110 * scale, 24 * scale);
            let dark_pixels = header
                .pixels()
                .filter(|(_, _, pixel)| {
                    u16::from(pixel.0[0]) + u16::from(pixel.0[1]) + u16::from(pixel.0[2]) < 350
                })
                .count();
            assert!(
                dark_pixels > 10 * scale as usize,
                "HUD heading must remain inside its card at {scale}x; found {dark_pixels} dark pixels",
            );
        }
    }
}
