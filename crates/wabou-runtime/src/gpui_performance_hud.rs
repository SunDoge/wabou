//! GPUI-native performance overlay kept outside the Solid projection tree.

use gpui_base::{StyledExt as _, Theme};
use std::collections::VecDeque;
use wabou_shell::FrameStats;
use wabou_shell::gpui::{
    Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
};

pub(crate) struct GpuiPerformanceHud {
    stats: Option<FrameStats>,
    fps: f64,
    frame_ms: f64,
    frame_times: VecDeque<std::time::Instant>,
    last_diagnostic_at: std::time::Instant,
    projection_revision: u64,
    projection_materializations: u64,
    materialization_samples: VecDeque<(std::time::Instant, u64)>,
    #[cfg(test)]
    render_count: u64,
}

impl GpuiPerformanceHud {
    pub(crate) fn new() -> Self {
        Self {
            stats: None,
            fps: 0.0,
            frame_ms: 0.0,
            frame_times: VecDeque::new(),
            last_diagnostic_at: std::time::Instant::now(),
            projection_revision: 0,
            projection_materializations: 0,
            materialization_samples: VecDeque::new(),
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

    #[cfg(test)]
    pub(crate) fn render_count(&self) -> u64 {
        self.render_count
    }
}

impl Render for GpuiPerformanceHud {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        #[cfg(test)]
        {
            self.render_count = self.render_count.wrapping_add(1);
        }
        // A platform frame alone does not dirty an Entity. Explicitly notify
        // this HUD on the next frame so its clock remains live without making
        // the Solid projection or QuickJS runtime participate in the loop.
        let hud = cx.weak_entity();
        window.on_next_frame(move |_, cx| {
            let _ = hud.update(cx, |_, hud_cx| hud_cx.notify());
        });
        let now = std::time::Instant::now();
        record_frame_sample(
            &mut self.frame_times,
            now,
            &mut self.fps,
            &mut self.frame_ms,
        );
        record_counter_sample(
            &mut self.materialization_samples,
            now,
            self.projection_materializations,
        );
        let materializations_per_second = counter_rate(&self.materialization_samples);
        if self.last_diagnostic_at.elapsed() >= std::time::Duration::from_secs(1) {
            tracing::debug!(
                target: "wabou::perf",
                fps = self.fps,
                frame_ms = self.frame_ms,
                materializations_per_second,
                "native performance HUD sample"
            );
            self.last_diagnostic_at = now;
        }
        let theme = Theme::global(cx);
        let colors = &theme.tokens.colors;
        let stats = self.stats.unwrap_or_default();
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
                    .child(format!("{:>4.0} FPS", self.fps)),
            )
            .child(format!(
                "JS {:>6.2} ms  projection {:>6.2} ms",
                stats.js_tick_ms, stats.scene_ms
            ))
            .child(format!(
                "frame {:>6.2} ms  nodes {:>6}",
                self.frame_ms, stats.node_count
            ))
            .child(format!(
                "build {:>6.2} ms  materialize {:>4.0}/s",
                stats.build_frame_ms, materializations_per_second
            ))
            .child(format!(
                "viewport {}×{}  revision {}",
                stats.viewport_w, stats.viewport_h, self.projection_revision
            ))
    }
}

const SAMPLE_WINDOW: std::time::Duration = std::time::Duration::from_secs(1);

fn record_frame_sample(
    samples: &mut VecDeque<std::time::Instant>,
    now: std::time::Instant,
    fps: &mut f64,
    frame_ms: &mut f64,
) {
    if let Some(previous) = samples.back() {
        *frame_ms = now.duration_since(*previous).as_secs_f64() * 1_000.0;
    }
    samples.push_back(now);
    while samples
        .front()
        .is_some_and(|sample| now.duration_since(*sample) > SAMPLE_WINDOW)
    {
        samples.pop_front();
    }
    if let (Some(first), Some(last)) = (samples.front(), samples.back()) {
        let elapsed = last.duration_since(*first).as_secs_f64();
        if elapsed > 0.0 {
            *fps = (samples.len().saturating_sub(1) as f64) / elapsed;
        }
    }
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
    fn sliding_window_fps_ignores_time_before_the_first_render() {
        let start = std::time::Instant::now();
        let mut samples = VecDeque::new();
        let mut fps = 0.0;
        let mut frame_ms = 0.0;

        // The HUD may be constructed long before its first render. Sampling
        // begins here, so startup and route-loading time cannot depress FPS.
        record_frame_sample(&mut samples, start, &mut fps, &mut frame_ms);
        record_frame_sample(
            &mut samples,
            start + std::time::Duration::from_millis(16),
            &mut fps,
            &mut frame_ms,
        );

        assert!((fps - 62.5).abs() < 0.01);
        assert!((frame_ms - 16.0).abs() < 0.01);
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
