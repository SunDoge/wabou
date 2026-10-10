//! Resident JSON state for one capability namespace, independent of loaded code.
use crate::capability_module::HostOperations;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Cloneable host-owned storage. Values survive implementation replacement,
/// but are volatile and do not survive process exit. Use one store per namespace.
#[derive(Clone, Default)]
pub struct CapabilityState {
    values: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}
impl CapabilityState {
    /// Stable operations understood by the capability SDK's typed state slots.
    pub fn operations(&self) -> Arc<HostOperations> {
        let store = self.clone();
        Arc::new(move |operation, request| store.call(operation, request))
    }
    /// Dispatch one resident state operation with copied JSON data.
    pub fn call(&self, operation: &str, request: &[u8]) -> Result<Vec<u8>, String> {
        let request: serde_json::Value =
            serde_json::from_slice(request).map_err(|e| e.to_string())?;
        let mut state = self.values.lock().map_err(|e| e.to_string())?;
        let response = match operation {
            "state.snapshot" | "state.read" => {
                let key = request.as_str().ok_or("state read expects a key string")?;
                match (operation, state.get(key)) {
                    ("state.snapshot", Some(value)) => {
                        serde_json::json!({"present": true, "value": value})
                    }
                    ("state.snapshot", None) => serde_json::json!({"present": false}),
                    (_, value) => value.cloned().unwrap_or(serde_json::Value::Null),
                }
            }
            "state.write" => {
                let key = request["key"]
                    .as_str()
                    .ok_or("state.write expects key and value")?;
                let value = request
                    .get("value")
                    .ok_or("state.write expects value")?
                    .clone();
                state.insert(key.to_owned(), value);
                serde_json::Value::Null
            }
            _ => return Err(format!("unknown resident host operation: {operation}")),
        };
        serde_json::to_vec(&response).map_err(|e| e.to_string())
    }
}
