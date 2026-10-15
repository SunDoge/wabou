//! Built-in demonstration canvas.
use super::WidgetAdapter;
use serde_json::Value;
use wabou_shell_api::widget_implementation::{
    WidgetDraw as Draw, WidgetEvent as Event, WidgetImplementation, WidgetInputs as Inputs,
    WidgetPresentation as Presentation, WidgetUpdate as Update,
};

struct CanvasImplementation;
impl WidgetImplementation for CanvasImplementation {
    fn measure(_: &Inputs) -> Result<Option<[f32; 2]>, String> {
        Ok(None)
    }
    fn presentation(_: &Inputs) -> Result<Presentation, String> {
        Ok(Presentation {
            focusable: false,
            ..Default::default()
        })
    }
    fn paint(inputs: &Inputs) -> Result<Vec<Draw>, String> {
        let [width, height] = inputs.size.map(f64::from);
        if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
            return Ok(Vec::new());
        }
        Ok(vec![
            Draw::Rect {
                rect: [0.0, 0.0, width, height],
                color: [30, 41, 59, 255],
            },
            Draw::Circle {
                center: [width / 2.0, height / 2.0],
                radius: width.min(height) * 0.3,
                color: [99, 102, 241, 128],
            },
            Draw::Line {
                from: [0.0, 0.0],
                to: [width, height],
                width: 2.0,
                color: [34, 211, 238, 255],
            },
        ])
    }
    fn event(inputs: &Inputs, _: Event) -> Result<Update, String> {
        Ok(Update {
            state: inputs.state.clone(),
            handled: false,
        })
    }
}

pub(super) fn adapter() -> WidgetAdapter {
    WidgetAdapter::from_implementation::<CanvasImplementation>(Value::Null)
}

/// Compatibility unit-struct factory for the built-in canvas.
pub struct Canvas;
use wabou_shell::{PaintContext, Widget};

// Standard registration retains the adapter and resource cache. Rust hosts
// using the original unit-struct factory still use the same implementation.
impl Widget for Canvas {
    fn paint(&mut self, cx: &mut PaintContext<'_>) {
        adapter().paint(cx);
    }
}
