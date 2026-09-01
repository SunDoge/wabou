//! Framework-owned GPUI widgets mounted behind explicit Wabou tags.

use std::{collections::HashMap, sync::Arc};

use serde::Deserialize;

use gpui_base::slider::{SliderEvent, SliderState};
use gpui_base::{AxisExt as _, SliderIndicator, SliderThumb, SliderTrack, Theme};
use wabou_shell::gpui::prelude::FluentBuilder as _;
use wabou_shell::gpui::{
    AppContext as _, Axis, Bounds, Context, Entity, InteractiveElement as _, IntoElement as _,
    MouseButton, ParentElement as _, Pixels, Render, StatefulInteractiveElement as _, Styled as _,
    Subscription, Window, bounds, canvas, div, ease_in_out, fill, point, px, relative, size,
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

struct GpuiIndeterminateProgress {
    timeline: NativeLoopTimeline,
}

fn slider_default_max() -> f32 {
    100.0
}

fn slider_default_step() -> f32 {
    1.0
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct SliderConfig {
    #[serde(default)]
    min: f32,
    #[serde(default = "slider_default_max")]
    max: f32,
    #[serde(default = "slider_default_step")]
    step: f32,
    #[serde(default)]
    value: f32,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    orientation: SliderOrientation,
    #[serde(default)]
    reversed: bool,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl SliderOrientation {
    fn axis(self) -> Axis {
        match self {
            Self::Horizontal => Axis::Horizontal,
            Self::Vertical => Axis::Vertical,
        }
    }
}

impl Default for SliderConfig {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: slider_default_max(),
            step: slider_default_step(),
            value: 0.0,
            disabled: false,
            orientation: SliderOrientation::Horizontal,
            reversed: false,
        }
    }
}

impl SliderConfig {
    fn normalized(mut self) -> Self {
        if self.max <= self.min {
            self.max = self.min + 1.0;
        }
        if self.step <= 0.0 {
            self.step = 1.0;
        }
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    fn creates_same_state(self, other: Self) -> bool {
        self.min == other.min && self.max == other.max && self.step == other.step
    }

    fn state(self) -> SliderState {
        SliderState::new()
            .min(self.min)
            .max(self.max)
            .step(self.step)
            .default_value(self.value)
    }
}

struct GpuiSlider {
    config: SliderConfig,
    state: Entity<SliderState>,
    _subscription: Subscription,
}

impl GpuiSlider {
    fn new(
        config: SliderConfig,
        events: wabou_shell::NativeWidgetEventSink,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|_| config.state());
        let subscription = cx.subscribe(&state, move |_, _, event, cx| {
            if let SliderEvent::Change(value) = event {
                events.change_f64(value.end() as f64, cx);
            }
        });
        Self {
            config,
            state,
            _subscription: subscription,
        }
    }

    fn synchronize(&mut self, config: SliderConfig, window: &mut Window, cx: &mut Context<Self>) {
        if self.config.value != config.value {
            self.state.update(cx, |state, state_cx| {
                state.set_value(config.value, window, state_cx)
            });
        }
        if self.config.disabled != config.disabled
            || self.config.orientation != config.orientation
            || self.config.reversed != config.reversed
        {
            cx.notify();
        }
        self.config = config;
    }
}

impl Render for GpuiSlider {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl wabou_shell::gpui::IntoElement {
        let colors = Theme::global(cx).tokens.colors;
        let percentage = self.state.read(cx).percentage().end;
        let state = self.state.clone();
        let disabled = self.config.disabled;
        let axis = self.config.orientation.axis();
        let reversed = self.config.reversed;
        div()
            .size_full()
            .flex()
            .items_center()
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                gpui_base::GlobalState::suppress_text_selection(cx);
            })
            .when(disabled, |this| this.opacity(0.45))
            .when(!disabled, |this| {
                this.on_mouse_up(
                    MouseButton::Left,
                    window.listener_for(&state, |state, _, _, cx| state.handle_release(cx)),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    window.listener_for(&state, |state, _, _, cx| state.handle_release(cx)),
                )
            })
            .child(
                SliderTrack::new(&self.state)
                    .axis(axis)
                    .disabled(disabled)
                    .relative()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(
                        SliderIndicator::new(&self.state)
                            .relative()
                            .when(axis.is_horizontal(), |this| this.w_full().h(px(6.0)))
                            .when(axis.is_vertical(), |this| this.h_full().w(px(6.0)))
                            .rounded_full()
                            .bg(colors.muted)
                            .child(
                                div()
                                    .absolute()
                                    .when(axis.is_horizontal(), |this| {
                                        this.top_0().bottom_0().when_else(
                                            reversed,
                                            |this| this.left(relative(percentage)).right_0(),
                                            |this| this.left_0().right(relative(1.0 - percentage)),
                                        )
                                    })
                                    .when(axis.is_vertical(), |this| {
                                        this.left_0().right_0().when_else(
                                            reversed,
                                            |this| this.bottom(relative(percentage)).top_0(),
                                            |this| this.bottom_0().top(relative(1.0 - percentage)),
                                        )
                                    })
                                    .rounded_full()
                                    .bg(colors.accent),
                            )
                            .child(
                                SliderThumb::new(&self.state)
                                    .axis(axis)
                                    .disabled(disabled)
                                    .absolute()
                                    .when(axis.is_horizontal(), |this| {
                                        this.top(px(-5.0)).left(relative(percentage)).ml(px(-8.0))
                                    })
                                    .when(axis.is_vertical(), |this| {
                                        this.left(px(-5.0))
                                            .bottom(relative(percentage))
                                            .mb(px(-8.0))
                                    })
                                    .size(px(16.0))
                                    .rounded_full()
                                    .border_1()
                                    .border_color(colors.border)
                                    .bg(colors.surface)
                                    .hover(|this| this.border_color(colors.ring))
                                    .active(|this| this.border_color(colors.ring)),
                            ),
                    ),
            )
    }
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

impl GpuiIndeterminateProgress {
    fn new(config: SpinnerConfig) -> Self {
        Self {
            timeline: NativeLoopTimeline::new(config.animation),
        }
    }

    fn synchronize(&mut self, config: SpinnerConfig) -> bool {
        self.timeline.synchronize(config.animation)
    }
}

fn indeterminate_progress_edges(phase: f32) -> (f32, f32) {
    let phase = phase.clamp(0.0, 1.0);
    let left = ease_in_out(((phase - 0.5) / 0.5).clamp(0.0, 1.0));
    let right = ease_in_out(1.0 - phase);
    (left, right)
}

impl Render for GpuiIndeterminateProgress {
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
        let (left, right) = indeterminate_progress_edges(phase);
        let color = Theme::global(cx).tokens.colors.accent;
        div().size_full().relative().overflow_hidden().child(
            div()
                .absolute()
                .top_0()
                .bottom_0()
                .left(relative(left))
                .right(relative(right))
                .rounded_full()
                .bg(color),
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
    HashMap::from([
        (
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
        ),
        (
            "slider".to_owned(),
            Arc::new(
                |context: wabou_shell::NativeWidgetContext<'_>,
                 window: &mut wabou_shell::gpui::Window,
                 cx: &mut wabou_shell::gpui::App| {
                    let config = context
                        .config_json()
                        .and_then(|json| serde_json::from_str::<SliderConfig>(json).ok())
                        .unwrap_or_default()
                        .normalized();
                    let retained = context.entity::<GpuiSlider>();
                    let entity = retained
                        .filter(|entity| entity.read(cx).config.creates_same_state(config))
                        .unwrap_or_else(|| {
                            let events = context.events();
                            cx.new(|cx| GpuiSlider::new(config, events, cx))
                        });
                    entity.update(cx, |slider, slider_cx| {
                        slider.synchronize(config, window, slider_cx);
                    });
                    NativeWidgetMount::entity(entity.clone(), entity.into_any_element())
                },
            ) as NativeWidgetFactory,
        ),
        (
            "progress-indeterminate".to_owned(),
            Arc::new(
                |context: wabou_shell::NativeWidgetContext<'_>,
                 _: &mut wabou_shell::gpui::Window,
                 cx: &mut wabou_shell::gpui::App| {
                    let config = context
                        .config_json()
                        .and_then(|json| serde_json::from_str(json).ok())
                        .unwrap_or_default();
                    let entity = context
                        .entity::<GpuiIndeterminateProgress>()
                        .unwrap_or_else(|| cx.new(|_| GpuiIndeterminateProgress::new(config)));
                    if context.entity::<GpuiIndeterminateProgress>().is_some() {
                        entity.update(cx, |progress, cx| {
                            if progress.synchronize(config) {
                                cx.notify();
                            }
                        });
                    }
                    NativeWidgetMount::entity(entity.clone(), entity.into_any_element())
                },
            ) as NativeWidgetFactory,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc};

    use wabou_shell::gpui::{Modifiers, TestAppContext};

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

    #[test]
    fn indeterminate_progress_edges_stay_ordered_inside_the_track() {
        for phase in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let (left, right) = indeterminate_progress_edges(phase);
            assert!((0.0..=1.0).contains(&left));
            assert!((0.0..=1.0).contains(&right));
            assert!(left + right <= 1.0 + f32::EPSILON);
        }
    }

    #[test]
    fn slider_config_normalizes_invalid_ranges_before_gpui_state_creation() {
        let config = SliderConfig {
            min: 10.0,
            max: 5.0,
            step: 0.0,
            value: 99.0,
            disabled: false,
            ..Default::default()
        }
        .normalized();
        assert_eq!(config.max, 11.0);
        assert_eq!(config.step, 1.0);
        assert_eq!(config.value, 11.0);
        assert!(builtin_native_widgets().contains_key("slider"));
    }

    #[gpui::test]
    fn slider_owns_pointer_value_changes_without_starting_text_selection(cx: &mut TestAppContext) {
        cx.update(gpui_base::GlobalState::init);
        let changed = Rc::new(Cell::new(None));
        let observed = changed.clone();
        let key = wabou_shell::NodeKey::new(42, 1);
        let (_view, cx) = cx.add_window_view(move |_, cx| {
            let input = Rc::new(move |event, _: &mut wabou_shell::gpui::App| {
                if let wabou_shell::ProjectedInputEvent::ValueChange { target, value } = event {
                    assert_eq!(target, key);
                    observed.set(Some(value));
                }
            });
            GpuiSlider::new(
                SliderConfig {
                    value: 100.0,
                    ..Default::default()
                },
                wabou_shell::NativeWidgetContext::new(key, &Default::default(), None, None, input)
                    .events(),
                cx,
            )
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let center = cx.update(|window, _| window.bounds().center());
        cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::default());
        cx.update(|_, cx| {
            assert!(gpui_base::GlobalState::is_text_selection_suppressed(cx));
        });
        cx.simulate_mouse_up(center, MouseButton::Left, Modifiers::default());

        assert!(
            changed
                .get()
                .is_some_and(|value| (value - 50.0).abs() < 0.5)
        );
    }
}
