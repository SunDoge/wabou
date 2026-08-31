//! Framework-owned GPUI widgets mounted behind explicit Wabou tags.

use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;

use gpui_base::Theme;
use wabou_shell::gpui::{
    AppContext as _, Bounds, Context, IntoElement as _, ParentElement as _, Pixels, Render,
    Styled as _, Window, bounds, canvas, div, fill, point, px, size,
};
use wabou_shell::{NativeWidgetFactory, NativeWidgetMount};

use crate::gpui_motion::{NativeLoopAnimation, NativeLoopTimeline};

const SPINNER_DOTS: usize = 8;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SpinnerConfig {
    animation: NativeLoopAnimation,
}

struct GpuiSpinner {
    timeline: NativeLoopTimeline,
}

impl GpuiSpinner {
    fn new(config: SpinnerConfig) -> Self {
        Self {
            timeline: NativeLoopTimeline::new(config.animation),
        }
    }

    fn synchronize(&mut self, config: SpinnerConfig) -> bool {
        self.timeline.synchronize(config.animation)
    }
}

impl Render for GpuiSpinner {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl wabou_shell::gpui::IntoElement {
        let reduced_motion = cx.reduce_motion();
        let phase = self.timeline.phase(reduced_motion);
        if self.timeline.is_running(reduced_motion) {
            window.request_animation_frame();
        }
        let color = Theme::global(cx).tokens.colors.primary;

        div().size_full().child(
            canvas(
                move |bounds, _, _| spinner_dots(bounds, phase),
                move |_, dots, window, _| {
                    for (index, bounds) in dots.into_iter().enumerate() {
                        let head = (phase * SPINNER_DOTS as f32).floor() as usize % SPINNER_DOTS;
                        let age = (head + SPINNER_DOTS - index) % SPINNER_DOTS;
                        let mut dot_color = color;
                        dot_color.alpha *= 1.0 - age as f32 / (SPINNER_DOTS as f32 + 1.0);
                        let radius = bounds.size.width / 2.0;
                        window.paint_quad(fill(bounds, dot_color).corner_radii(radius));
                    }
                },
            )
            .size_full(),
        )
    }
}

fn spinner_dots(outer: Bounds<Pixels>, phase: f32) -> Vec<Bounds<Pixels>> {
    let extent = outer.size.width.min(outer.size.height);
    let dot = (extent * 0.18).max(px(1.0));
    let radius = (extent - dot) * 0.5;
    let center = outer.center();
    let phase_angle = phase * std::f32::consts::TAU;
    (0..SPINNER_DOTS)
        .map(|index| {
            let angle = phase_angle + index as f32 * std::f32::consts::TAU / SPINNER_DOTS as f32;
            let center_x = center.x + radius * angle.cos();
            let center_y = center.y + radius * angle.sin();
            bounds(
                point(center_x - dot / 2.0, center_y - dot / 2.0),
                size(dot, dot),
            )
        })
        .collect()
}

pub(crate) fn builtin_native_widgets() -> HashMap<String, NativeWidgetFactory> {
    HashMap::from([(
        "spinner".to_owned(),
        Arc::new(
            |context: wabou_shell::NativeWidgetContext<'_>,
             _: &mut wabou_shell::gpui::Window,
             cx: &mut wabou_shell::gpui::App| {
                let config = context
                    .config_json()
                    .and_then(|json| serde_json::from_str(json).ok())
                    .unwrap_or_default();
                let entity = context
                    .entity::<GpuiSpinner>()
                    .unwrap_or_else(|| cx.new(|_| GpuiSpinner::new(config)));
                if context.entity::<GpuiSpinner>().is_some() {
                    entity.update(cx, |spinner, cx| {
                        if spinner.synchronize(config) {
                            cx.notify();
                        }
                    });
                }
                NativeWidgetMount::entity(entity.clone(), entity.into_any_element())
            },
        ) as NativeWidgetFactory,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_dots_stay_inside_the_authored_bounds() {
        let outer = bounds(point(px(10.0), px(20.0)), size(px(16.0), px(16.0)));
        let dots = spinner_dots(outer, 0.375);
        assert_eq!(dots.len(), SPINNER_DOTS);
        for dot in dots {
            let epsilon = px(0.001);
            assert!(dot.left() >= outer.left() - epsilon);
            assert!(dot.top() >= outer.top() - epsilon);
            assert!(dot.right() <= outer.right() + epsilon);
            assert!(dot.bottom() <= outer.bottom() + epsilon);
        }
    }
}
