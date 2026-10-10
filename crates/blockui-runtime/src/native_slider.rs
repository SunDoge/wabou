//! Host-owned integer slider adapter over Wabou's Vello control.
//! Contains no actor callback or plugin pointer; the host drains owned values.
use std::collections::VecDeque;
use wabou_shell_vello::{
    self as shell, PaintContext, UiEvent, Widget, WidgetEventResult, WidgetGeometry,
};
use wabou_widgets_vello::Slider;

/// Integer range constrained to exactly representable f32 values in the reused control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntegerRange {
    pub min: i32,
    pub max: i32,
    pub step: i32,
}
impl IntegerRange {
    pub fn validate(self) -> Result<Self, String> {
        if self.min >= self.max || self.step <= 0 || self.min < -8_388_608 || self.max > 8_388_608 {
            return Err("slider needs an increasing exact integer range and positive step".into());
        }
        if (i64::from(self.max) - i64::from(self.min)) % i64::from(self.step) != 0 {
            return Err("slider endpoints must align to its step".into());
        }
        Ok(self)
    }
    fn normalize(self, value: i32) -> i32 {
        let offset = i64::from(value.clamp(self.min, self.max)) - i64::from(self.min);
        (i64::from(self.min)
            + ((offset + i64::from(self.step) / 2) / i64::from(self.step)) * i64::from(self.step))
            as i32
    }
}
/// Native retained control. Projection updates never emit user changes.
pub struct NativeSlider {
    inner: Slider,
    range: IntegerRange,
    value: i32,
    disabled: bool,
    changes: VecDeque<i64>,
}
impl NativeSlider {
    pub fn new(range: IntegerRange, value: i32) -> Result<Self, String> {
        let range = range.validate()?;
        let mut control = Self {
            inner: Slider::new(),
            range,
            value: range.normalize(value),
            disabled: false,
            changes: VecDeque::new(),
        };
        control.sync()?;
        Ok(control)
    }
    fn sync(&mut self) -> Result<(), String> {
        self.inner.config_changed(&serde_json::json!({"min": self.range.min, "max": self.range.max, "step": self.range.step, "value": self.value, "disabled": self.disabled}).to_string())?;
        Ok(())
    }
    pub fn project(
        &mut self,
        range: IntegerRange,
        value: i32,
        disabled: bool,
    ) -> Result<(), String> {
        let range = range.validate()?;
        if disabled && !self.disabled {
            self.cancel_drag();
        }
        self.range = range;
        self.value = range.normalize(value);
        self.disabled = disabled;
        self.sync()
    }
    pub fn value(&self) -> i32 {
        self.value
    }
    pub fn take_value(&mut self) -> Option<i64> {
        self.changes.pop_front()
    }
    fn publish(&mut self, value: i32) {
        let value = self.range.normalize(value);
        if value != self.value {
            self.value = value;
            // Native event bursts coalesce to one latest controlled value.
            self.changes.clear();
            self.changes.push_back(i64::from(value));
        }
    }
    fn cancel_drag(&mut self) {
        self.inner
            .handle_event(&UiEvent::Pointer(shell::PointerEvent {
                phase: shell::PointerPhase::Cancel,
                position: shell::Point { x: 0.0, y: 0.0 },
                button: None,
                buttons: 0,
                modifiers: shell::Modifiers::empty(),
                properties: Default::default(),
            }));
    }
}
impl Widget for NativeSlider {
    fn style_changed(&mut self, style: &shell::WidgetStyle) -> shell::WidgetChanges {
        self.inner.style_changed(style)
    }
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        self.inner.paint(cx);
    }
    fn layout_changed(&mut self, geometry: WidgetGeometry) {
        self.inner.layout_changed(geometry);
    }
    fn accepts_focus(&self) -> bool {
        !self.disabled
    }
    fn focus_changed(&mut self, focused: bool) -> shell::WidgetChanges {
        if !focused {
            self.cancel_drag();
        }
        shell::WidgetChanges::empty()
    }
    fn handle_event(&mut self, event: &UiEvent) -> WidgetEventResult {
        if self.disabled {
            return WidgetEventResult::IGNORED;
        }
        if let UiEvent::Key(key) = event {
            if key.phase != shell::KeyPhase::Down || !key.modifiers.is_empty() {
                return WidgetEventResult::IGNORED;
            }
            let value = match key.key.as_str() {
                "ArrowLeft" | "ArrowDown" => self.value.saturating_sub(self.range.step),
                "ArrowRight" | "ArrowUp" => self.value.saturating_add(self.range.step),
                "Home" => self.range.min,
                "End" => self.range.max,
                _ => return WidgetEventResult::IGNORED,
            };
            self.publish(value);
            self.sync().expect("validated slider configuration");
            return WidgetEventResult::HANDLED;
        }
        let result = self.inner.handle_event(event);
        while let Some(event) = self.inner.take_node_event() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&event.json)
                && let Some(number) = value["value"].as_f64()
                && number.is_finite()
            {
                self.publish(number.round() as i32);
            }
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn pointer(phase: shell::PointerPhase, x: f64) -> UiEvent {
        UiEvent::Pointer(shell::PointerEvent {
            phase,
            position: shell::Point { x, y: 14.0 },
            button: Some(shell::PointerButton::Primary),
            buttons: 1,
            modifiers: shell::Modifiers::empty(),
            properties: Default::default(),
        })
    }
    #[test]
    fn native_drag_emits_owned_integer_values_and_projection_is_silent() {
        let range = IntegerRange {
            min: 0,
            max: 60000,
            step: 100,
        };
        let mut slider = NativeSlider::new(range, 0).unwrap();
        slider.layout_changed(WidgetGeometry {
            content_size: [100.0, 28.0],
            ..Default::default()
        });
        assert!(
            slider
                .handle_event(&pointer(shell::PointerPhase::Down, 50.0))
                .is_handled()
        );
        assert_eq!(slider.take_value(), Some(30000));
        slider.handle_event(&pointer(shell::PointerPhase::Move, 200.0));
        assert_eq!(slider.take_value(), Some(60000));
        slider.project(range, 10000, true).unwrap();
        assert_eq!(slider.take_value(), None);
        assert!(
            !slider
                .handle_event(&pointer(shell::PointerPhase::Move, 20.0))
                .is_handled()
        );
        slider.project(range, 10000, false).unwrap();
        assert!(
            !slider
                .handle_event(&pointer(shell::PointerPhase::Move, 20.0))
                .is_handled()
        );
        assert_eq!(slider.value(), 10000);
    }
    #[test]
    fn keyboard_steps_endpoints_and_disabled_control() {
        let range = IntegerRange {
            min: 0,
            max: 100,
            step: 10,
        };
        let mut slider = NativeSlider::new(range, 50).unwrap();
        let key = |name: &str| {
            UiEvent::Key(shell::KeyEvent {
                phase: shell::KeyPhase::Down,
                key: name.into(),
                key_without_modifiers: name.into(),
                code: name.into(),
                text: None,
                text_with_all_modifiers: None,
                location: Default::default(),
                modifiers: shell::Modifiers::empty(),
                repeat: false,
                synthetic: false,
            })
        };
        assert!(slider.handle_event(&key("ArrowRight")).is_handled());
        assert_eq!(slider.take_value(), Some(60));
        slider.handle_event(&key("Home"));
        assert_eq!(slider.take_value(), Some(0));
        slider.handle_event(&key("ArrowLeft"));
        assert_eq!(slider.take_value(), None);
        slider.handle_event(&key("End"));
        assert_eq!(slider.take_value(), Some(100));
        slider.project(range, 50, true).unwrap();
        assert!(!slider.handle_event(&key("Home")).is_handled());
        assert_eq!(slider.take_value(), None);
    }
    #[test]
    fn range_validation_and_negative_rounding() {
        let range = IntegerRange {
            min: -100,
            max: 100,
            step: 10,
        };
        assert!(range.validate().is_ok());
        assert_eq!(range.normalize(-95), -90);
        assert_eq!(range.normalize(i32::MIN), -100);
        for invalid in [
            IntegerRange { step: 0, ..range },
            IntegerRange { max: -100, ..range },
            IntegerRange { step: 3, ..range },
            IntegerRange {
                max: i32::MAX,
                ..range
            },
        ] {
            assert!(invalid.validate().is_err());
        }
    }
}
