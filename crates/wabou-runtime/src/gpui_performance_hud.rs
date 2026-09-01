//! GPUI-native performance overlay kept outside the Solid projection tree.

use gpui_base::{StyledExt as _, Theme, ThemeAppearance};
use std::collections::VecDeque;
use wabou_shell::FrameStats;
use wabou_shell::gpui::{
    Context, FrameTiming, FrameTimingCollector, Hsla, IntoElement, ParentElement as _, Render,
    Styled as _, Window, div, rgb_to_hsla, rgba, set_frame_trace_enabled,
};

pub(crate) struct GpuiPerformanceHud {
    stats: Option<FrameStats>,
    fps: f64,
    draw_ms: f64,
    dirty_to_draw_ms: f64,
    invalidations_per_frame: f64,
    frame_timings: VecDeque<FrameTiming>,
    frame_timing_collector: FrameTimingCollector,
    last_diagnostic_at: std::time::Instant,
    projection_revision: u64,
    projection_materializations: u64,
    materialization_samples: VecDeque<(std::time::Instant, u64)>,
    sample_after_draw: bool,
    #[cfg(test)]
    render_count: u64,
}

impl GpuiPerformanceHud {
    pub(crate) fn new() -> Self {
        set_frame_trace_enabled(true);
        Self {
            stats: None,
            fps: 0.0,
            draw_ms: 0.0,
            dirty_to_draw_ms: 0.0,
            invalidations_per_frame: 0.0,
            frame_timings: VecDeque::new(),
            frame_timing_collector: FrameTimingCollector::new(),
            last_diagnostic_at: std::time::Instant::now(),
            projection_revision: 0,
            projection_materializations: 0,
            materialization_samples: VecDeque::new(),
            sample_after_draw: true,
            #[cfg(test)]
            render_count: 0,
        }
    }

    pub(crate) fn update(
        &mut self,
        stats: Option<FrameStats>,
        projection_revision: u64,
        projection_materializations: u64,
        cx: &mut Context<Self>,
    ) {
        if self.stats == stats
            && self.projection_revision == projection_revision
            && self.projection_materializations == projection_materializations
        {
            return;
        }
        self.stats = stats;
        self.projection_revision = projection_revision;
        self.projection_materializations = projection_materializations;
        cx.notify();
    }

    fn sample_after_completed_draw(
        &mut self,
        window_id: wabou_shell::gpui::WindowId,
        now: std::time::Instant,
    ) {
        self.frame_timings.extend(
            self.frame_timing_collector
                .collect_unseen()
                .into_iter()
                .filter(|timing| timing.window_id == window_id),
        );
        summarize_frame_timings(
            &mut self.frame_timings,
            now,
            &mut self.fps,
            &mut self.draw_ms,
            &mut self.dirty_to_draw_ms,
            &mut self.invalidations_per_frame,
        );
        record_counter_sample(
            &mut self.materialization_samples,
            now,
            self.projection_materializations,
        );
        if self.last_diagnostic_at.elapsed() >= std::time::Duration::from_secs(1) {
            tracing::debug!(
                target: "wabou::perf",
                draw_rate_hz = self.fps,
                draw_ms = self.draw_ms,
                dirty_to_draw_ms = self.dirty_to_draw_ms,
                invalidations_per_frame = self.invalidations_per_frame,
                materializations_per_second = counter_rate(&self.materialization_samples),
                "native performance HUD sample"
            );
            self.last_diagnostic_at = now;
        }
    }

    #[cfg(test)]
    pub(crate) fn render_count(&self) -> u64 {
        self.render_count
    }

    #[cfg(test)]
    pub(crate) fn sample_after_draw_pending(&self) -> bool {
        self.sample_after_draw
    }
}

impl Render for GpuiPerformanceHud {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(test)]
        {
            self.render_count = self.render_count.wrapping_add(1);
        }
        // Frame timing is recorded after rendering finishes. Request exactly
        // one post-mount follow-up so the first timing becomes visible. Later
        // samples are collected only when real application work renders the
        // HUD again; the profiler must never manufacture its own frame loop.
        if std::mem::take(&mut self.sample_after_draw) {
            let hud = cx.weak_entity();
            window.on_next_frame(move |_, cx| {
                let _ = hud.update(cx, |_, hud_cx| hud_cx.notify());
            });
        }
        self.sample_after_completed_draw(
            window.window_handle().window_id(),
            std::time::Instant::now(),
        );
        let materializations_per_second = counter_rate(&self.materialization_samples);
        let theme = Theme::global(cx);
        let colors = &theme.tokens.colors;
        let stats = self.stats.unwrap_or_default();
        let draw_color = frame_health_color(
            frame_health(self.draw_ms),
            theme.appearance,
            colors.muted_foreground,
            colors.destructive,
        );
        let latency_color = frame_health_color(
            frame_health(self.dirty_to_draw_ms),
            theme.appearance,
            colors.muted_foreground,
            colors.destructive,
        );
        div()
            .absolute()
            .top_3()
            .right_3()
            .w_64()
            .flex()
            .flex_col()
            .gap_1()
            .p_3()
            .rounded_lg()
            .border_1()
            .border_color(colors.border)
            .bg(colors.surface)
            .text_color(colors.surface_foreground)
            .shadow_lg()
            .text_xs()
            .child(
                div()
                    .flex()
                    .justify_between()
                    .font_semibold()
                    .child("Performance HUD")
                    .child(format!("{:>4.0} draw/s", self.fps)),
            )
            .child(format!(
                "JS {:>6.2} ms  projection {:>6.2} ms",
                stats.js_tick_ms, stats.scene_ms
            ))
            .child(
                div().flex().justify_between().child("draw").child(
                    div()
                        .font_semibold()
                        .text_color(draw_color)
                        .child(format!("{:>6.2} ms", self.draw_ms)),
                ),
            )
            .child(
                div().flex().justify_between().child("dirty→draw").child(
                    div()
                        .font_semibold()
                        .text_color(latency_color)
                        .child(format!("{:>6.2} ms", self.dirty_to_draw_ms)),
                ),
            )
            .child(format!(
                "invalidations {:>4.1}/frame  nodes {:>6}",
                self.invalidations_per_frame, stats.node_count
            ))
            .child(format!(
                "build {:>6.2} ms  materialize {:>4.0}/s",
                stats.build_frame_ms, materializations_per_second,
            ))
            .child(format!(
                "viewport {}×{}  revision {}",
                stats.viewport_w, stats.viewport_h, self.projection_revision
            ))
    }
}

const SAMPLE_WINDOW: std::time::Duration = std::time::Duration::from_secs(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrameHealth {
    Unknown,
    Healthy,
    Warning,
    Critical,
}

fn frame_health(milliseconds: f64) -> FrameHealth {
    if milliseconds <= 0.0 || !milliseconds.is_finite() {
        FrameHealth::Unknown
    } else if milliseconds < 8.0 {
        FrameHealth::Healthy
    } else if milliseconds < 16.7 {
        FrameHealth::Warning
    } else {
        FrameHealth::Critical
    }
}

fn frame_health_color(
    health: FrameHealth,
    appearance: ThemeAppearance,
    unknown: Hsla,
    critical: Hsla,
) -> Hsla {
    match (health, appearance) {
        (FrameHealth::Unknown, _) => unknown,
        (FrameHealth::Healthy, ThemeAppearance::Light) => rgb_to_hsla(rgba(0x1680_3cff)),
        (FrameHealth::Healthy, ThemeAppearance::Dark) => rgb_to_hsla(rgba(0x4ade_80ff)),
        (FrameHealth::Warning, ThemeAppearance::Light) => rgb_to_hsla(rgba(0xb453_09ff)),
        (FrameHealth::Warning, ThemeAppearance::Dark) => rgb_to_hsla(rgba(0xfbbf_24ff)),
        (FrameHealth::Critical, _) => critical,
    }
}

fn summarize_frame_timings(
    samples: &mut VecDeque<FrameTiming>,
    now: std::time::Instant,
    fps: &mut f64,
    draw_ms: &mut f64,
    dirty_to_draw_ms: &mut f64,
    invalidations_per_frame: &mut f64,
) {
    while samples
        .front()
        .is_some_and(|sample| now.saturating_duration_since(sample.draw_end) > SAMPLE_WINDOW)
    {
        samples.pop_front();
    }
    if let (Some(first), Some(last)) = (samples.front(), samples.back()) {
        let elapsed = last.draw_end.duration_since(first.draw_end).as_secs_f64();
        if elapsed > 0.0 {
            *fps = (samples.len().saturating_sub(1) as f64) / elapsed;
        }
    }
    if samples.is_empty() {
        return;
    }
    let sample_count = samples.len() as f64;
    *draw_ms = samples
        .iter()
        .map(|timing| timing.draw_duration().as_secs_f64() * 1_000.0)
        .sum::<f64>()
        / sample_count;
    let observed_dirty_samples = samples
        .iter()
        .filter_map(|timing| timing.dirty_to_draw_duration())
        .collect::<Vec<_>>();
    if !observed_dirty_samples.is_empty() {
        *dirty_to_draw_ms = observed_dirty_samples
            .iter()
            .map(|duration| duration.as_secs_f64() * 1_000.0)
            .sum::<f64>()
            / observed_dirty_samples.len() as f64;
    }
    *invalidations_per_frame = samples
        .iter()
        .map(|timing| timing.invalidations as f64)
        .sum::<f64>()
        / sample_count;
}

fn record_counter_sample(
    samples: &mut VecDeque<(std::time::Instant, u64)>,
    now: std::time::Instant,
    value: u64,
) {
    samples.push_back((now, value));
    while samples
        .front()
        .is_some_and(|(sample, _)| now.duration_since(*sample) > SAMPLE_WINDOW)
    {
        samples.pop_front();
    }
}

fn counter_rate(samples: &VecDeque<(std::time::Instant, u64)>) -> f64 {
    let (Some((first_at, first)), Some((last_at, last))) = (samples.front(), samples.back()) else {
        return 0.0;
    };
    let elapsed = last_at.duration_since(*first_at).as_secs_f64();
    if elapsed == 0.0 {
        0.0
    } else {
        last.saturating_sub(*first) as f64 / elapsed
    }
}

pub(crate) fn performance_hud_enabled() -> bool {
    std::env::var("WABOU_PERFORMANCE_HUD").is_ok_and(|value| parse_enabled(&value))
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
    fn hud_environment_values_are_explicit() {
        for value in ["1", "true", "TRUE", " yes ", "on"] {
            assert!(parse_enabled(value), "{value}");
        }
        for value in ["", "0", "false", "off", "enabled"] {
            assert!(!parse_enabled(value), "{value}");
        }
    }

    #[test]
    fn frame_health_matches_interactive_frame_budgets() {
        assert_eq!(frame_health(0.0), FrameHealth::Unknown);
        assert_eq!(frame_health(f64::NAN), FrameHealth::Unknown);
        assert_eq!(frame_health(7.99), FrameHealth::Healthy);
        assert_eq!(frame_health(8.0), FrameHealth::Warning);
        assert_eq!(frame_health(16.69), FrameHealth::Warning);
        assert_eq!(frame_health(16.7), FrameHealth::Critical);
    }

    #[test]
    fn sliding_window_uses_gpui_draw_timings_instead_of_hud_render_time() {
        let start = std::time::Instant::now();
        let frame = |draw_start, draw_end, dirty_at, invalidations| FrameTiming {
            window_id: wabou_shell::gpui::WindowId::from(1),
            dirty_at,
            invalidations,
            draw_start,
            draw_end,
        };
        let mut samples = VecDeque::from([
            frame(
                start,
                start + std::time::Duration::from_millis(4),
                Some(start - std::time::Duration::from_millis(2)),
                2,
            ),
            frame(
                start + std::time::Duration::from_millis(14),
                start + std::time::Duration::from_millis(20),
                Some(start + std::time::Duration::from_millis(10)),
                4,
            ),
        ]);
        let mut fps = 0.0_f64;
        let mut draw_ms = 0.0_f64;
        let mut dirty_to_draw_ms = 0.0_f64;
        let mut invalidations_per_frame = 0.0_f64;

        summarize_frame_timings(
            &mut samples,
            start + std::time::Duration::from_millis(20),
            &mut fps,
            &mut draw_ms,
            &mut dirty_to_draw_ms,
            &mut invalidations_per_frame,
        );

        assert!((fps - 62.5).abs() < 0.01);
        assert!((draw_ms - 5.0).abs() < 0.01);
        assert!((dirty_to_draw_ms - 8.0).abs() < 0.01);
        assert!((invalidations_per_frame - 3.0).abs() < 0.01);
    }

    #[test]
    fn counter_rate_reports_projection_materializations_in_the_same_window() {
        let start = std::time::Instant::now();
        let mut samples = VecDeque::new();
        record_counter_sample(&mut samples, start, 4);
        record_counter_sample(
            &mut samples,
            start + std::time::Duration::from_millis(500),
            7,
        );

        assert!((counter_rate(&samples) - 6.0).abs() < 0.01);
    }
}
