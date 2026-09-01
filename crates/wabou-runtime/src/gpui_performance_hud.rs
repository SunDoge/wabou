//! GPUI-native performance overlay kept outside the Solid projection tree.

use gpui_base::{StyledExt as _, Theme};
use wabou_shell::FrameStats;
use wabou_shell::gpui::{
    Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
};

pub(crate) struct GpuiPerformanceHud {
    stats: Option<FrameStats>,
    fps: f64,
    previous_frame_at: std::time::Instant,
    last_diagnostic_at: std::time::Instant,
    projection_revision: u64,
    #[cfg(test)]
    render_count: u64,
}

impl GpuiPerformanceHud {
    pub(crate) fn new() -> Self {
        Self {
            stats: None,
            fps: 0.0,
            previous_frame_at: std::time::Instant::now(),
            last_diagnostic_at: std::time::Instant::now(),
            projection_revision: 0,
            #[cfg(test)]
            render_count: 0,
        }
    }

    pub(crate) fn update(
        &mut self,
        stats: Option<FrameStats>,
        projection_revision: u64,
        cx: &mut Context<Self>,
    ) {
        if self.stats == stats && self.projection_revision == projection_revision {
            return;
        }
        self.stats = stats;
        self.projection_revision = projection_revision;
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
        let elapsed = now.duration_since(self.previous_frame_at).as_secs_f64();
        self.previous_frame_at = now;
        if elapsed > 0.0 {
            let sample = 1.0 / elapsed;
            self.fps = if self.fps == 0.0 {
                sample
            } else {
                self.fps * 0.9 + sample * 0.1
            };
        }
        if self.last_diagnostic_at.elapsed() >= std::time::Duration::from_secs(1) {
            tracing::debug!(target: "wabou::perf", fps = self.fps, "native performance HUD sample");
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
                stats.build_frame_ms, stats.node_count
            ))
            .child(format!(
                "viewport {}×{}  revision {}",
                stats.viewport_w, stats.viewport_h, self.projection_revision
            ))
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
}
