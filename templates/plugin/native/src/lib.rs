//! One small Rust crate providing capabilities and native widgets.
use serde_json::json;
use std::task::Poll;
use wabou_capability_sdk::{
    Host, Module, Ready, Task, export_plugin,
    widget::{Draw, Event, Inputs, Update, WidgetImplementation},
};
const STEP: u64 = 1;
struct Counter;
impl Module for Counter {
    const CONTRACT_VERSION: u32 = 1;
    fn start(host: &Host<'_>, method: &str, _: &[u8]) -> Result<Box<dyn Task>, String> {
        if method == "later" {
            return Ok(Box::new(Deferred(false)));
        }
        let state = host.state::<u64>();
        let mut value = state.get()?.unwrap_or(0);
        match method {
            "increment" => {
                value = value.checked_add(STEP).ok_or("counter overflow")?;
                state.set(&value)?;
            }
            "read" => {}
            _ => return Err(format!("unknown counter method: {method}")),
        }
        Ready::json(&value)
    }
}
struct Deferred(bool);
impl Task for Deferred {
    fn poll(&mut self, _: &Host<'_>) -> Poll<Result<Vec<u8>, String>> {
        if !std::mem::replace(&mut self.0, true) {
            return Poll::Pending;
        }
        Poll::Ready(serde_json::to_vec(&STEP).map_err(|e| e.to_string()))
    }
}
struct CounterWidget;
impl WidgetImplementation for CounterWidget {
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
export_plugin! {
    capabilities { "counter" => Counter, "secondary" => Counter }
    widgets { "reload-counter" => CounterWidget, "secondary-counter" => CounterWidget }
}
