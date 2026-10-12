//! Change STEP/colors and save: resident state survives implementation replacement.
use serde_json::json;
use wabou_capability_sdk::{
    export_module,
    widget::{Draw, Event, Inputs, Update, WidgetImplementation, WidgetModule},
};
const STEP: u64 = 1;
struct Counter;
impl WidgetImplementation for Counter {
    const SCHEMA_VERSION: u32 = 1;
    fn measure(_: &Inputs) -> Result<Option<[f32; 2]>, String> {
        Ok(Some([240.0, 80.0]))
    }
    fn paint(inputs: &Inputs) -> Result<Vec<Draw>, String> {
        let count = inputs
            .state
            .as_u64()
            .ok_or("expected integer counter state")?;
        Ok(vec![
            Draw::Rect {
                rect: [0.0, 0.0, inputs.size[0] as f64, inputs.size[1] as f64],
                color: [35, 45, 70, 255],
            },
            Draw::Text {
                origin: [16.0, 20.0],
                text: format!("Count: {count} (step {STEP})"),
                font_size: 20.0,
                color: [240, 245, 255, 255],
            },
        ])
    }
    fn event(inputs: &Inputs, event: Event) -> Result<Update, String> {
        let count = inputs
            .state
            .as_u64()
            .ok_or("expected integer counter state")?;
        let handled = matches!(event, Event::Pointer { down: true, .. });
        Ok(Update {
            state: json!(if handled {
                count.checked_add(STEP).ok_or("counter overflow")?
            } else {
                count
            }),
            handled,
        })
    }
}
export_module!(WidgetModule<Counter>);
