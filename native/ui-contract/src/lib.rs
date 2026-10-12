//! Shared data contracts, not shared Rust objects or callback pointers.
pub use serde;
use serde::de::DeserializeOwned;
pub use serde::{Deserialize, Serialize};

/// One application operation. Its Rust type provides its identity automatically.
/// Keep this shared contract unchanged during UI reload; changing it restarts host.
pub trait Request: Serialize + DeserializeOwned {
    type Response: Serialize + DeserializeOwned;
    fn id() -> u64 {
        blockui_abi::key(std::any::type_name::<Self>())
    }
}
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() > blockui_abi::MAX_MESSAGE_TEXT_BYTES {
        return Err("application service payload exceeds 64 KiB".into());
    }
    Ok(bytes)
}
pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    if bytes.len() > blockui_abi::MAX_MESSAGE_TEXT_BYTES {
        return Err("application service payload exceeds 64 KiB".into());
    }
    serde_json::from_slice(bytes).map_err(|e| e.to_string())
}
