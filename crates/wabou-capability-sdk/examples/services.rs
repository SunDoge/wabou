//! Integration probe for services supplied by a precompiled Wabou runtime.
use serde_json::{Value, json};
use std::{path::PathBuf, task::Poll, time::Duration};
use wabou_capability_sdk::{
    ConnectionKey, FileKey, Host, Module, Ready, Task, TaskKey, export_module,
};

struct Services;
impl Module for Services {
    const CONTRACT_VERSION: u32 = 1;
    fn start(host: &Host<'_>, method: &str, request: &[u8]) -> Result<Box<dyn Task>, String> {
        fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
            serde_json::from_slice(bytes).map_err(|e| e.to_string())
        }
        let value = match method {
            "increment" => {
                let amount: u64 = decode(request)?;
                let slot = host.state::<u64>();
                let value = slot.get()?.unwrap_or_default() + amount;
                slot.set(&value)?;
                json!(value)
            }
            "sleep" => json!(host.sleep(Duration::from_millis(decode(request)?))?),
            "poll" => match host.task_poll::<Value>(decode::<TaskKey>(request)?) {
                Poll::Pending => json!({"status":"pending"}),
                Poll::Ready(Ok(value)) => json!({"status":"ready", "value":value}),
                Poll::Ready(Err(error)) => json!({"status":"failed", "error":error}),
            },
            "release" => {
                host.task_release(decode(request)?)?;
                Value::Null
            }
            "openFile" => json!(host.file_open(&decode::<PathBuf>(request)?)?),
            "readFile" => {
                let (file, length): (FileKey, usize) = decode(request)?;
                json!(host.file_read(file, length)?)
            }
            "closeFile" => {
                host.file_close(decode(request)?)?;
                Value::Null
            }
            "connect" => json!(host.tcp_connect(&decode::<String>(request)?)?),
            "readSocket" => {
                let (connection, length): (ConnectionKey, usize) = decode(request)?;
                json!(host.tcp_read(connection, length)?)
            }
            "writeSocket" => {
                let (connection, bytes): (ConnectionKey, Vec<u8>) = decode(request)?;
                json!(host.tcp_write(connection, &bytes)?)
            }
            "closeSocket" => {
                host.tcp_close(decode(request)?)?;
                Value::Null
            }
            _ => return Err(format!("unknown probe method: {method}")),
        };
        Ready::json(&value)
    }
}
export_module!(Services);
