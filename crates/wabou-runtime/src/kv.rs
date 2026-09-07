//! Built-in typed bridge for Wabou's SQLite-backed hierarchical KV store.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use wabou_bindgen::HostMethod;
use wabou_database::{KvCheck, KvKey, KvKeyPart, KvListOptions, KvMutation, KvStore, Versionstamp};

use crate::NativeCapability;

type LazyStore = Arc<tokio::sync::OnceCell<Arc<KvStore>>>;

const GET: HostMethod<KeyRequest, Option<EntryResponse>> = HostMethod::new("get");
const SET: HostMethod<SetRequest, CommitResponse> = HostMethod::new("set");
const DELETE: HostMethod<KeyRequest, CommitResponse> = HostMethod::new("delete");
const LIST: HostMethod<ListRequest, Vec<EntryResponse>> = HostMethod::new("list");
const ATOMIC: HostMethod<AtomicRequest, AtomicResponse> = HostMethod::new("atomic");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
enum WireKeyPart {
    String(String),
    I64(String),
    U64(String),
    Bytes(Vec<u8>),
    Bool(bool),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct KeyRequest {
    key: Vec<WireKeyPart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SetRequest {
    key: Vec<WireKeyPart>,
    value: serde_json::Value,
    expire_in: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ListRequest {
    #[serde(default)]
    prefix: Vec<WireKeyPart>,
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    reverse: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CheckRequest {
    key: Vec<WireKeyPart>,
    versionstamp: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum MutationRequest {
    Set {
        key: Vec<WireKeyPart>,
        value: serde_json::Value,
        #[serde(default)]
        expire_in: Option<u64>,
    },
    MergePatch {
        key: Vec<WireKeyPart>,
        patch: serde_json::Value,
    },
    Delete {
        key: Vec<WireKeyPart>,
    },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AtomicRequest {
    #[serde(default)]
    checks: Vec<CheckRequest>,
    #[serde(default)]
    mutations: Vec<MutationRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EntryResponse {
    key: Vec<WireKeyPart>,
    value: serde_json::Value,
    versionstamp: String,
    expires_at: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CommitResponse {
    versionstamp: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AtomicResponse {
    committed: bool,
    versionstamp: Option<String>,
}

/// Mount the standard KV methods into an alternate Wabou host capability.
#[doc(hidden)]
pub fn mount_kv_methods(capability: NativeCapability<'_>, path: PathBuf) -> rquickjs::Result<()> {
    mount_kv_methods_with_state(capability, Arc::new(tokio::sync::OnceCell::new()), path)
}

fn mount_kv_methods_with_state(
    capability: NativeCapability<'_>,
    state: LazyStore,
    path: PathBuf,
) -> rquickjs::Result<()> {
    let get_state = state.clone();
    let get_path = path.clone();
    capability.method(GET, move |request: KeyRequest| {
        let state = get_state.clone();
        let path = get_path.clone();
        async move {
            let store = open_store(&state, &path).await?;
            store
                .get(&decode_key(request.key)?)
                .await
                .map_err(|error| error.to_string())?
                .map(EntryResponse::try_from)
                .transpose()
        }
    })?;

    let set_state = state.clone();
    let set_path = path.clone();
    capability.method(SET, move |request: SetRequest| {
        let state = set_state.clone();
        let path = set_path.clone();
        async move {
            let store = open_store(&state, &path).await?;
            let versionstamp = store
                .set(
                    decode_key(request.key)?,
                    request.value,
                    expiry_from_duration(request.expire_in)?,
                )
                .await
                .map_err(|error| error.to_string())?;
            Ok::<_, String>(CommitResponse {
                versionstamp: versionstamp.0.to_string(),
            })
        }
    })?;

    let delete_state = state.clone();
    let delete_path = path.clone();
    capability.method(DELETE, move |request: KeyRequest| {
        let state = delete_state.clone();
        let path = delete_path.clone();
        async move {
            let store = open_store(&state, &path).await?;
            let versionstamp = store
                .delete(decode_key(request.key)?)
                .await
                .map_err(|error| error.to_string())?;
            Ok::<_, String>(CommitResponse {
                versionstamp: versionstamp.0.to_string(),
            })
        }
    })?;

    let list_state = state.clone();
    let list_path = path.clone();
    capability.method(LIST, move |request: ListRequest| {
        let state = list_state.clone();
        let path = list_path.clone();
        async move {
            let store = open_store(&state, &path).await?;
            store
                .list(&KvListOptions {
                    prefix: decode_prefix(request.prefix)?,
                    limit: request.limit,
                    reverse: request.reverse,
                })
                .await
                .map_err(|error| error.to_string())?
                .into_iter()
                .map(EntryResponse::try_from)
                .collect()
        }
    })?;

    capability.method(ATOMIC, move |request: AtomicRequest| {
        let state = state.clone();
        let path = path.clone();
        async move {
            let store = open_store(&state, &path).await?;
            let checks = request
                .checks
                .into_iter()
                .map(|check| {
                    Ok(KvCheck {
                        key: decode_key(check.key)?,
                        versionstamp: check
                            .versionstamp
                            .map(|value| parse_versionstamp(&value))
                            .transpose()?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            let mutations = request
                .mutations
                .into_iter()
                .map(decode_mutation)
                .collect::<Result<Vec<_>, String>>()?;
            let result = store
                .atomic(&checks, &mutations)
                .await
                .map_err(|error| error.to_string())?;
            Ok::<_, String>(AtomicResponse {
                committed: result.committed,
                versionstamp: result.versionstamp.map(|value| value.0.to_string()),
            })
        }
    })
}

async fn open_store(state: &LazyStore, path: &Path) -> Result<Arc<KvStore>, String> {
    state
        .get_or_try_init(|| async {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .map_err(|error| format!("cannot create KV directory: {error}"))?;
            }
            KvStore::open(path)
                .await
                .map(Arc::new)
                .map_err(|error| error.to_string())
        })
        .await
        .cloned()
}

fn decode_key(parts: Vec<WireKeyPart>) -> Result<KvKey, String> {
    if parts.is_empty() {
        return Err("KV keys must contain at least one part".into());
    }
    decode_prefix(parts)
}

fn decode_prefix(parts: Vec<WireKeyPart>) -> Result<KvKey, String> {
    parts
        .into_iter()
        .map(|part| match part {
            WireKeyPart::String(value) => Ok(KvKeyPart::String(value)),
            WireKeyPart::I64(value) => value
                .parse()
                .map(KvKeyPart::I64)
                .map_err(|_| format!("invalid signed KV key integer `{value}`")),
            WireKeyPart::U64(value) => value
                .parse()
                .map(KvKeyPart::U64)
                .map_err(|_| format!("invalid unsigned KV key integer `{value}`")),
            WireKeyPart::Bytes(value) => Ok(KvKeyPart::Bytes(value)),
            WireKeyPart::Bool(value) => Ok(KvKeyPart::Bool(value)),
        })
        .collect()
}

fn encode_key(parts: KvKey) -> Vec<WireKeyPart> {
    parts
        .into_iter()
        .map(|part| match part {
            KvKeyPart::String(value) => WireKeyPart::String(value),
            KvKeyPart::I64(value) => WireKeyPart::I64(value.to_string()),
            KvKeyPart::U64(value) => WireKeyPart::U64(value.to_string()),
            KvKeyPart::Bytes(value) => WireKeyPart::Bytes(value),
            KvKeyPart::Bool(value) => WireKeyPart::Bool(value),
        })
        .collect()
}

fn decode_mutation(request: MutationRequest) -> Result<KvMutation, String> {
    match request {
        MutationRequest::Set {
            key,
            value,
            expire_in,
        } => Ok(KvMutation::Set {
            key: decode_key(key)?,
            value,
            expires_at: expiry_from_duration(expire_in)?,
        }),
        MutationRequest::MergePatch { key, patch } => Ok(KvMutation::MergePatch {
            key: decode_key(key)?,
            patch,
        }),
        MutationRequest::Delete { key } => Ok(KvMutation::Delete {
            key: decode_key(key)?,
        }),
    }
}

fn expiry_from_duration(expire_in: Option<u64>) -> Result<Option<i64>, String> {
    expire_in
        .map(|duration| {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let expiry = now
                .checked_add(u128::from(duration))
                .ok_or_else(|| "KV expiry exceeds the supported timestamp range".to_owned())?;
            i64::try_from(expiry)
                .map_err(|_| "KV expiry exceeds the supported timestamp range".to_owned())
        })
        .transpose()
}

fn parse_versionstamp(value: &str) -> Result<Versionstamp, String> {
    value
        .parse::<u64>()
        .map(Versionstamp)
        .map_err(|_| format!("invalid KV versionstamp `{value}`"))
}

fn default_limit() -> usize {
    100
}

impl TryFrom<wabou_database::KvEntry> for EntryResponse {
    type Error = String;

    fn try_from(entry: wabou_database::KvEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            key: encode_key(entry.key),
            value: entry.value,
            versionstamp: entry.versionstamp.0.to_string(),
            expires_at: entry.expires_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_wire_preserves_large_integers_without_javascript_numbers() {
        let decoded = decode_key(vec![
            WireKeyPart::I64(i64::MIN.to_string()),
            WireKeyPart::U64(u64::MAX.to_string()),
        ])
        .unwrap();
        assert_eq!(decoded[0], KvKeyPart::I64(i64::MIN));
        assert_eq!(decoded[1], KvKeyPart::U64(u64::MAX));
        assert_eq!(encode_key(decoded).len(), 2);
    }
}
