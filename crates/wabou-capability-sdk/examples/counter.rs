//! Application behavior is replaceable; `counter.value` is runtime-owned.
use wabou_capability_sdk::{Host, Module, Ready, Task, export_module};

struct Counter;
impl Module for Counter {
    const CONTRACT_VERSION: u32 = 1;

    fn start(host: &Host<'_>, method: &str, request: &[u8]) -> Result<Box<dyn Task>, String> {
        match method {
            "increment" => {
                let amount: u32 = serde_json::from_slice(request).map_err(|e| e.to_string())?;
                let value: u32 = host.call_json("counter.increment", &amount)?;
                Ready::json(&value)
            }
            "label" => {
                let value: u32 = host.call_json("counter.read", &())?;
                // Change this presentation logic without recreating counter state.
                Ready::json(&format!("Count: {value}"))
            }
            _ => Err(format!("unknown counter method: {method}")),
        }
    }
}
export_module!(Counter);
