//! GPUI-native performance overlay kept outside the Solid projection tree.

use gpui_base::{StyledExt as _, Theme};
use wabou_shell::FrameStats;
use wabou_shell::gpui::{
    Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
};

pub(crate) struct GpuiPerformanceHud {
    stats: Option<FrameStats>,
    fps: f64,
    projection_revision: u64,
}

impl GpuiPerformanceHud {
    pub(crate) fn new() -> Self {
        Self {
            stats: None,
            fps: 0.0,
            projection_revision: 0,
        }
    }

    pub(crate) fn update(
        &mut self,
        stats: Option<FrameStats>,
        fps: f64,
        projection_revision: u64,
        cx: &mut Context<Self>,
    ) {
        self.stats = stats;
        self.fps = fps;
        self.projection_revision = projection_revision;
        cx.notify();
    }
}

impl Render for GpuiPerformanceHud {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
