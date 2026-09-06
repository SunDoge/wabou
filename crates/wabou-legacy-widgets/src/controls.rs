//! Backend-neutral animated indicators and slider controls.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use anyrender::PaintScene;
use serde::Deserialize;
use vello::kurbo::{Affine, Circle, Rect};
use vello::peniko::{Color, Fill};
use wabou_protocol::event;
use wabou_shell::{
    PaintContext, PointerButton, PointerPhase, UiEvent, Widget, WidgetChanges, WidgetEventResult,
    WidgetGeometry, WidgetNodeEvent, WidgetStyle, decode_widget_config,
};

const FRAME_INTERVAL: Duration = Duration::from_millis(16);
const SPINNER_DOTS: usize = 8;
const DEFAULT_ACCENT: Color = Color::from_rgb8(0x25, 0x63, 0xeb);

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct NativeLoopAnimation {
    kind: NativeLoopKind,
    duration: f32,
    speed: f32,
    paused: bool,
    #[serde(rename = "reducedMotion")]
    reduced_motion: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum NativeLoopKind {
    Loop,
}

impl Default for NativeLoopAnimation {
    fn default() -> Self {
        Self {
            kind: NativeLoopKind::Loop,
            duration: 1.0,
            speed: 1.0,
            paused: false,
            reduced_motion: false,
        }
    }
}

impl NativeLoopAnimation {
    fn normalized(self) -> Self {
        Self {
            duration: positive_finite(self.duration, 1.0),
            speed: positive_finite(self.speed, 1.0),
            ..self
        }
    }

    fn running(self) -> bool {
        !self.paused && !self.reduced_motion
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct LoopConfig {
    animation: NativeLoopAnimation,
}

struct LoopTimeline {
    animation: NativeLoopAnimation,
    started: Instant,
}

impl LoopTimeline {
    fn new() -> Self {
        Self {
            animation: NativeLoopAnimation::default(),
            started: Instant::now(),
        }
    }

    fn synchronize(&mut self, animation: NativeLoopAnimation) -> bool {
        let animation = animation.normalized();
        if animation == self.animation {
            return false;
        }
        if animation.duration != self.animation.duration || animation.speed != self.animation.speed
        {
            self.started = Instant::now();
        }
        self.animation = animation;
        true
    }

    fn phase(&self) -> f32 {
        if !self.animation.running() {
            return 0.0;
        }
        (self.started.elapsed().as_secs_f32() * self.animation.speed / self.animation.duration)
            .fract()
    }

    fn deadline(&self) -> Option<Instant> {
        self.animation
            .running()
            .then(|| Instant::now() + FRAME_INTERVAL)
    }
}

fn positive_finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

/// Native eight-dot activity indicator used by the Winit backend.
pub struct Spinner {
    timeline: LoopTimeline,
    color: Color,
}

impl Spinner {
    /// Construct a spinner with the default animation policy.
    pub fn new() -> Self {
        Self {
            timeline: LoopTimeline::new(),
            color: DEFAULT_ACCENT,
        }
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Spinner {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let extent = f64::from(cx.width().min(cx.height()));
        let dot = (extent * 0.18).max(1.0);
        let orbit = (extent - dot) * 0.5;
        let center = (f64::from(cx.width()) * 0.5, f64::from(cx.height()) * 0.5);
        let sampled_phase = self.timeline.phase();
        let phase = f64::from(sampled_phase) * std::f64::consts::TAU;
        for index in 0..SPINNER_DOTS {
            let angle = phase + index as f64 * std::f64::consts::TAU / SPINNER_DOTS as f64;
            let x = center.0 + orbit * angle.cos();
            let y = center.1 + orbit * angle.sin();
            let head = (sampled_phase * SPINNER_DOTS as f32).floor() as usize % SPINNER_DOTS;
            let age = (head + SPINNER_DOTS - index) % SPINNER_DOTS;
            let alpha = 1.0 - age as f32 / (SPINNER_DOTS as f32 + 1.0);
            cx.scene_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                self.color.with_alpha(alpha),
                None,
                &Circle::new((x, y), dot * 0.5),
            );
        }
    }

    fn config_changed(&mut self, json: &str) -> Result<WidgetChanges, String> {
        let config = decode_widget_config::<LoopConfig>(json)?;
        Ok(if self.timeline.synchronize(config.animation) {
            WidgetChanges::REDRAW
        } else {
            WidgetChanges::empty()
        })
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> WidgetChanges {
        self.color = style.color;
        WidgetChanges::REDRAW
    }

    fn animation_deadline(&self) -> Option<Instant> {
        self.timeline.deadline()
    }
}

/// Native moving fill used for an indeterminate progress track.
pub struct IndeterminateProgress {
    timeline: LoopTimeline,
    color: Color,
}

impl IndeterminateProgress {
    /// Construct an indeterminate progress fill.
    pub fn new() -> Self {
        Self {
            timeline: LoopTimeline::new(),
            color: DEFAULT_ACCENT,
        }
    }
}

impl Default for IndeterminateProgress {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for IndeterminateProgress {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let (left, right) = indeterminate_edges(self.timeline.phase());
        let width = f64::from(cx.width());
        let height = f64::from(cx.height());
        let rect = Rect::new(
            width * f64::from(left),
            0.0,
            width * f64::from(right),
            height,
        );
        cx.scene_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            self.color,
            None,
            &rect.to_rounded_rect(height * 0.5),
        );
    }

    fn config_changed(&mut self, json: &str) -> Result<WidgetChanges, String> {
        let config = decode_widget_config::<LoopConfig>(json)?;
        Ok(if self.timeline.synchronize(config.animation) {
            WidgetChanges::REDRAW
        } else {
            WidgetChanges::empty()
        })
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> WidgetChanges {
        self.color = style.color;
        WidgetChanges::REDRAW
    }

    fn animation_deadline(&self) -> Option<Instant> {
        self.timeline.deadline()
    }
}

fn indeterminate_edges(phase: f32) -> (f32, f32) {
    let phase = phase.clamp(0.0, 1.0);
    let left = ease_in_out(((phase - 0.5) * 2.0).clamp(0.0, 1.0));
    let right_inset = ease_in_out((1.0 - phase).clamp(0.0, 1.0));
    (left, 1.0 - right_inset)
}

fn ease_in_out(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
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
        if !self.min.is_finite() {
            self.min = 0.0;
        }
        if !self.max.is_finite() || self.max <= self.min {
            self.max = self.min + 1.0;
        }
        self.step = positive_finite(self.step, 1.0);
        self.value = self.value.clamp(self.min, self.max);
        self
    }

    fn fraction(self) -> f32 {
        ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }
}

/// Pointer-driven range control for the Winit backend.
pub struct Slider {
    config: SliderConfig,
    geometry: WidgetGeometry,
    dragging: bool,
    color: Color,
    events: VecDeque<WidgetNodeEvent>,
}

impl Slider {
    /// Construct a slider with the same defaults as the TypeScript component.
    pub fn new() -> Self {
        Self {
            config: SliderConfig::default(),
            geometry: WidgetGeometry::default(),
            dragging: false,
            color: DEFAULT_ACCENT,
            events: VecDeque::new(),
        }
    }

    fn update_from_position(&mut self, x: f64, y: f64) -> bool {
        let [width, height] = self.geometry.content_size;
        let extent = match self.config.orientation {
            SliderOrientation::Horizontal => width,
            SliderOrientation::Vertical => height,
        };
        if extent <= 0.0 {
            return false;
        }
        let position = match self.config.orientation {
            SliderOrientation::Horizontal => x as f32 / width,
            SliderOrientation::Vertical => 1.0 - y as f32 / height,
        };
        let fraction = if self.config.reversed {
            1.0 - position
        } else {
            position
        }
        .clamp(0.0, 1.0);
        let raw = self.config.min + fraction * (self.config.max - self.config.min);
        let stepped = self.config.min
            + ((raw - self.config.min) / self.config.step).round() * self.config.step;
        let value = stepped.clamp(self.config.min, self.config.max);
        if value == self.config.value {
            return false;
        }
        self.config.value = value;
        self.events.push_back(WidgetNodeEvent::json(
            event::CHANGE,
            format!(r#"{{"value":{value}}}"#),
        ));
        true
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Slider {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        let width = f64::from(cx.width());
        let height = f64::from(cx.height());
        let fraction = f64::from(self.config.fraction());
        let fraction = if self.config.reversed {
            1.0 - fraction
        } else {
            fraction
        };
        let disabled_alpha = if self.config.disabled { 0.45 } else { 1.0 };
        let track = self.color.with_alpha(0.22 * disabled_alpha);
        let accent = self.color.with_alpha(disabled_alpha);
        let (track_rect, fill_rect, thumb) = match self.config.orientation {
            SliderOrientation::Horizontal => {
                let center = height * 0.5;
                let x = width * fraction;
                (
                    Rect::new(0.0, center - 3.0, width, center + 3.0),
                    if self.config.reversed {
                        Rect::new(x, center - 3.0, width, center + 3.0)
                    } else {
                        Rect::new(0.0, center - 3.0, x, center + 3.0)
                    },
                    (x, center),
                )
            }
            SliderOrientation::Vertical => {
                let center = width * 0.5;
                let y = height * (1.0 - fraction);
                (
                    Rect::new(center - 3.0, 0.0, center + 3.0, height),
                    if self.config.reversed {
                        Rect::new(center - 3.0, 0.0, center + 3.0, y)
                    } else {
                        Rect::new(center - 3.0, y, center + 3.0, height)
                    },
                    (center, y),
                )
            }
        };
        for (rect, color) in [(track_rect, track), (fill_rect, accent)] {
            cx.scene_mut().fill(
                Fill::NonZero,
                Affine::IDENTITY,
                color,
                None,
                &rect.to_rounded_rect(3.0),
            );
        }
        cx.scene_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            accent,
            None,
            &Circle::new(thumb, 8.0),
        );
    }

    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if self.config.disabled {
            return WidgetEventResult::IGNORED;
        }
        let UiEvent::Pointer(pointer) = event else {
            return WidgetEventResult::IGNORED;
        };
        match pointer.phase {
            PointerPhase::Down if pointer.button == Some(PointerButton::Primary) => {
                self.dragging = true;
                let _ = self.update_from_position(pointer.position.x, pointer.position.y);
                WidgetEventResult::HANDLED
            }
            PointerPhase::Move if self.dragging => {
                let _ = self.update_from_position(pointer.position.x, pointer.position.y);
                WidgetEventResult::HANDLED
            }
            PointerPhase::Up | PointerPhase::Cancel if self.dragging => {
                if pointer.phase == PointerPhase::Up {
                    let _ = self.update_from_position(pointer.position.x, pointer.position.y);
                }
                self.dragging = false;
                WidgetEventResult::HANDLED
            }
            _ => WidgetEventResult::IGNORED,
        }
    }

    fn layout_changed(&mut self, geometry: WidgetGeometry) {
        self.geometry = geometry;
    }

    fn config_changed(&mut self, json: &str) -> Result<WidgetChanges, String> {
        let config = decode_widget_config::<SliderConfig>(json)?.normalized();
        if config == self.config {
            return Ok(WidgetChanges::empty());
        }
        self.config = config;
        Ok(WidgetChanges::REDRAW | WidgetChanges::SEMANTICS)
    }

    fn style_changed(&mut self, style: &WidgetStyle) -> WidgetChanges {
        self.color = style.color;
        WidgetChanges::REDRAW
    }

    fn accepts_focus(&self) -> bool {
        !self.config.disabled
    }

    fn take_node_event(&mut self) -> Option<WidgetNodeEvent> {
        self.events.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wabou_shell::{Modifiers, Point, PointerEvent, PointerProperties, WidgetHarness};

    fn pointer(phase: PointerPhase, x: f64, y: f64) -> UiEvent {
        UiEvent::Pointer(PointerEvent {
            phase,
            position: Point { x, y },
            button: (phase != PointerPhase::Move).then_some(PointerButton::Primary),
            buttons: u32::from(phase != PointerPhase::Up),
            modifiers: Modifiers::empty(),
            properties: PointerProperties::default(),
        })
    }

    #[test]
    fn loop_policy_controls_native_redraw_deadlines() {
        let mut spinner = Spinner::new();
        assert!(spinner.animation_deadline().is_some());
        spinner
            .config_changed(
                r#"{"animation":{"kind":"loop","duration":1,"speed":1,"paused":true,"reducedMotion":false}}"#,
            )
            .unwrap();
        assert!(spinner.animation_deadline().is_none());
    }

    #[test]
    fn slider_emits_change_values_during_a_pointer_drag() {
        let mut harness = WidgetHarness::new(Slider::new());
        harness.layout(WidgetGeometry {
            content_size: [100.0, 28.0],
            ..WidgetGeometry::default()
        });
        harness
            .widget_mut()
            .config_changed(
                r#"{"min":0,"max":10,"step":1,"value":0,"disabled":false,"orientation":"horizontal","reversed":false}"#,
            )
            .unwrap();
        assert!(
            harness
                .event(&pointer(PointerPhase::Down, 46.0, 14.0))
                .is_handled()
        );
        let emitted = harness.widget_mut().take_node_event().unwrap();
        assert_eq!(emitted.event_code, event::CHANGE);
        assert_eq!(emitted.json, r#"{"value":5}"#);
    }

    #[test]
    fn disabled_slider_ignores_pointer_input() {
        let mut harness = WidgetHarness::new(Slider::new());
        harness
            .widget_mut()
            .config_changed(
                r#"{"min":0,"max":10,"step":1,"value":0,"disabled":true,"orientation":"horizontal","reversed":false}"#,
            )
            .unwrap();
        assert!(
            !harness
                .event(&pointer(PointerPhase::Down, 50.0, 10.0))
                .is_handled()
        );
        assert!(harness.widget_mut().take_node_event().is_none());
    }

    #[test]
    fn indeterminate_edges_remain_ordered_and_bounded() {
        for phase in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let (left, right) = indeterminate_edges(phase);
            assert!((0.0..=1.0).contains(&left));
            assert!((0.0..=1.0).contains(&right));
            assert!(left <= right);
        }
        assert_eq!(indeterminate_edges(0.0), (0.0, 0.0));
        assert_eq!(indeterminate_edges(1.0), (1.0, 1.0));
    }
}
