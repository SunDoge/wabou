//! Built-in typed bridge for Wabou's SQLite-backed hierarchical KV store.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use wabou_database::{KvCheck, KvKey, KvKeyPart, KvListOptions, KvMutation, KvStore, Versionstamp};

use crate::{HostModuleState, JsValue};

type LazyStore = Arc<tokio::sync::OnceCell<Arc<KvStore>>>;

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
pub(crate) struct KeyRequest {
    key: Vec<WireKeyPart>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetRequest {
    key: Vec<WireKeyPart>,
    value: serde_json::Value,
    expire_in: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ListRequest {
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
pub(crate) struct AtomicRequest {
    #[serde(default)]
    checks: Vec<CheckRequest>,
    #[serde(default)]
    mutations: Vec<MutationRequest>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EntryResponse {
    key: Vec<WireKeyPart>,
    value: serde_json::Value,
    versionstamp: String,
    expires_at: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitResponse {
    versionstamp: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AtomicResponse {
    committed: bool,
    versionstamp: Option<String>,
}

/// Runtime-owned state for the built-in KV host module.
#[doc(hidden)]
#[derive(Clone)]
pub struct KvModuleState {
    store: LazyStore,
    path: PathBuf,
}

impl KvModuleState {
    /// Create an unopened store at `path`.
    #[doc(hidden)]
    pub fn new(path: PathBuf) -> Self {
        Self {
            store: Arc::new(tokio::sync::OnceCell::new()),
            path,
        }
    }
}

/// QuickJS exports for the built-in KV store.
#[rquickjs::module]
pub mod kv_host {
    use super::*;

    #[rquickjs::function]
    pub async fn get(
        ctx: rquickjs::Ctx<'_>,
        JsValue(request): JsValue<KeyRequest>,
    ) -> rquickjs::Result<JsValue<Option<EntryResponse>>> {
        let state = state(&ctx)?;
        let store = open_store(&state.store, &state.path)
            .await
            .map_err(host_error)?;
        let response = store
            .get(&decode_key(request.key).map_err(host_error)?)
            .await
            .map_err(|error| host_error(error.to_string()))?
            .map(EntryResponse::try_from)
            .transpose()
            .map_err(host_error)?;
        Ok(JsValue(response))
    }

    #[rquickjs::function]
    pub async fn set(
        ctx: rquickjs::Ctx<'_>,
        JsValue(request): JsValue<SetRequest>,
    ) -> rquickjs::Result<JsValue<CommitResponse>> {
        let state = state(&ctx)?;
        let store = open_store(&state.store, &state.path)
            .await
            .map_err(host_error)?;
        let versionstamp = store
            .set(
                decode_key(request.key).map_err(host_error)?,
                request.value,
                expiry_from_duration(request.expire_in).map_err(host_error)?,
            )
            .await
            .map_err(|error| host_error(error.to_string()))?;
        Ok(JsValue(CommitResponse {
            versionstamp: versionstamp.0.to_string(),
        }))
    }

    #[rquickjs::function]
    pub async fn delete(
        ctx: rquickjs::Ctx<'_>,
        JsValue(request): JsValue<KeyRequest>,
    ) -> rquickjs::Result<JsValue<CommitResponse>> {
        let state = state(&ctx)?;
        let store = open_store(&state.store, &state.path)
            .await
            .map_err(host_error)?;
        let versionstamp = store
            .delete(decode_key(request.key).map_err(host_error)?)
            .await
            .map_err(|error| host_error(error.to_string()))?;
        Ok(JsValue(CommitResponse {
            versionstamp: versionstamp.0.to_string(),
        }))
    }

    #[rquickjs::function]
    pub async fn list(
        ctx: rquickjs::Ctx<'_>,
        JsValue(request): JsValue<ListRequest>,
    ) -> rquickjs::Result<JsValue<Vec<EntryResponse>>> {
        let state = state(&ctx)?;
        let store = open_store(&state.store, &state.path)
            .await
            .map_err(host_error)?;
        let entries = store
            .list(&KvListOptions {
                prefix: decode_prefix(request.prefix).map_err(host_error)?,
                limit: request.limit,
                reverse: request.reverse,
            })
            .await
            .map_err(|error| host_error(error.to_string()))?
            .into_iter()
            .map(EntryResponse::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(host_error)?;
        Ok(JsValue(entries))
    }

    #[rquickjs::function]
    pub async fn atomic(
        ctx: rquickjs::Ctx<'_>,
        JsValue(request): JsValue<AtomicRequest>,
    ) -> rquickjs::Result<JsValue<AtomicResponse>> {
        let state = state(&ctx)?;
        let store = open_store(&state.store, &state.path)
            .await
            .map_err(host_error)?;
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
            .collect::<Result<Vec<_>, String>>()
            .map_err(host_error)?;
        let mutations = request
            .mutations
            .into_iter()
            .map(decode_mutation)
            .collect::<Result<Vec<_>, String>>()
            .map_err(host_error)?;
        let result = store
            .atomic(&checks, &mutations)
            .await
            .map_err(|error| host_error(error.to_string()))?;
        Ok(JsValue(AtomicResponse {
            committed: result.committed,
            versionstamp: result.versionstamp.map(|value| value.0.to_string()),
        }))
    }

    fn state(ctx: &rquickjs::Ctx<'_>) -> rquickjs::Result<KvModuleState> {
        ctx.userdata::<HostModuleState<KvModuleState>>()
            .map(|state| state.0.clone())
            .ok_or_else(|| rquickjs::Exception::throw_type(ctx, "KV state is unavailable"))
    }

    fn host_error(error: impl std::fmt::Display) -> rquickjs::Error {
        rquickjs::Error::new_into_js_message("KV request", "KV response", error.to_string())
    }
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
