use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};

use crate::{Database, Error, Migration, Result};

const SCHEMA: Migration = Migration::new(
    1,
    "CREATE TABLE kv_meta (\
        singleton INTEGER PRIMARY KEY CHECK (singleton = 1),\
        versionstamp INTEGER NOT NULL\
    ) STRICT;\
    INSERT INTO kv_meta (singleton, versionstamp) VALUES (1, 0);\
    CREATE TABLE kv_entries (\
        key BLOB PRIMARY KEY,\
        key_json TEXT NOT NULL,\
        value_json TEXT NOT NULL,\
        versionstamp INTEGER NOT NULL,\
        expires_at INTEGER\
    ) STRICT;\
    CREATE INDEX kv_entries_expiry ON kv_entries (expires_at)\
        WHERE expires_at IS NOT NULL;",
);

/// A monotonically increasing revision assigned to one atomic commit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Versionstamp(pub u64);

/// One typed component in a hierarchical KV key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
pub enum KvKeyPart {
    /// UTF-8 text component.
    String(String),
    /// Signed integer component.
    I64(i64),
    /// Unsigned integer component.
    U64(u64),
    /// Arbitrary byte component.
    Bytes(Vec<u8>),
    /// Boolean component.
    Bool(bool),
}

/// A non-empty hierarchical key. Prefixes are defined by whole components.
pub type KvKey = Vec<KvKeyPart>;

/// A stored JSON document and its concurrency metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KvEntry {
    /// Original typed key.
    pub key: KvKey,
    /// Structured JSON value.
    pub value: serde_json::Value,
    /// Commit which last changed this entry.
    pub versionstamp: Versionstamp,
    /// Unix timestamp in milliseconds after which the entry is absent.
    pub expires_at: Option<i64>,
}

/// Options for a lexicographically ordered prefix query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KvListOptions {
    /// Whole-component prefix. An empty prefix lists the entire store.
    pub prefix: KvKey,
    /// Maximum number of entries returned.
    pub limit: usize,
    /// Return entries in descending key order.
    pub reverse: bool,
}

impl Default for KvListOptions {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            limit: 100,
            reverse: false,
        }
    }
}

/// Optimistic concurrency condition evaluated inside an atomic transaction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KvCheck {
    /// Entry to inspect.
    pub key: KvKey,
    /// Required revision. `None` requires the key to be absent or expired.
    pub versionstamp: Option<Versionstamp>,
}

/// One mutation in an atomic commit.
#[derive(Clone, Debug, PartialEq)]
pub enum KvMutation {
    /// Store a JSON value, optionally expiring at a Unix millisecond timestamp.
    Set {
        /// Destination key.
        key: KvKey,
        /// Structured JSON value.
        value: serde_json::Value,
        /// Optional absolute expiry time.
        expires_at: Option<i64>,
    },
    /// Apply an RFC 7396 JSON Merge Patch while preserving the entry expiry.
    ///
    /// Missing or expired entries are treated as an empty JSON object.
    MergePatch {
        /// Destination key.
        key: KvKey,
        /// JSON Merge Patch document.
        patch: serde_json::Value,
    },
    /// Remove a key if it exists.
    Delete {
        /// Key to remove.
        key: KvKey,
    },
}

/// Result of an optimistic atomic transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtomicCommit {
    /// Whether every check matched and the mutations committed.
    pub committed: bool,
    /// Shared revision assigned to every set in the successful commit.
    pub versionstamp: Option<Versionstamp>,
}

/// SQLite-backed hierarchical key-value store.
#[derive(Debug)]
pub struct KvStore {
    database: Database,
}

impl KvStore {
    /// Open or create a KV database at `path`.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            database: Database::open(path, &[SCHEMA]).await?,
        })
    }

    /// Read one unexpired entry.
    pub async fn get(&self, key: &[KvKeyPart]) -> Result<Option<KvEntry>> {
        let encoded = encode_key(key)?;
        self.database
            .call(move |connection| read_entry(connection, &encoded, now_millis()))
            .await
    }

    /// Store one value and return its new revision.
    pub async fn set(
        &self,
        key: KvKey,
        value: serde_json::Value,
        expires_at: Option<i64>,
    ) -> Result<Versionstamp> {
        let result = self
            .atomic(
                &[],
                &[KvMutation::Set {
                    key,
                    value,
                    expires_at,
                }],
            )
            .await?;
        Ok(result.versionstamp.expect("unconditional commit succeeds"))
    }

    /// Delete one key and return the commit revision.
    pub async fn delete(&self, key: KvKey) -> Result<Versionstamp> {
        let result = self.atomic(&[], &[KvMutation::Delete { key }]).await?;
        Ok(result.versionstamp.expect("unconditional commit succeeds"))
    }

    /// Apply an RFC 7396 JSON Merge Patch and return the new revision.
    ///
    /// The update is performed inside SQLite without reading and rewriting the
    /// complete document through Rust. Missing or expired entries use `{}` as
    /// their initial document.
    pub async fn merge_patch(&self, key: KvKey, patch: serde_json::Value) -> Result<Versionstamp> {
        let result = self
            .atomic(&[], &[KvMutation::MergePatch { key, patch }])
            .await?;
        Ok(result.versionstamp.expect("unconditional commit succeeds"))
    }

    /// List unexpired entries whose keys begin with `options.prefix`.
    pub async fn list(&self, options: &KvListOptions) -> Result<Vec<KvEntry>> {
        if options.limit == 0 {
            return Ok(Vec::new());
        }
        let lower = encode_prefix(&options.prefix);
        let upper = prefix_successor(&lower);
        let direction = if options.reverse { "DESC" } else { "ASC" };
        let sql = if upper.is_some() {
            format!(
                "SELECT key_json, value_json, versionstamp, expires_at FROM kv_entries \
                 WHERE key >= ?1 AND key < ?2 AND (expires_at IS NULL OR expires_at > ?3) \
                 ORDER BY key {direction} LIMIT ?4"
            )
        } else {
            format!(
                "SELECT key_json, value_json, versionstamp, expires_at FROM kv_entries \
                 WHERE key >= ?1 AND (expires_at IS NULL OR expires_at > ?2) \
                 ORDER BY key {direction} LIMIT ?3"
            )
        };
        let limit = i64::try_from(options.limit.min(10_000)).unwrap_or(10_000);
        self.database
            .call(move |connection| {
                let mut statement = connection.prepare(&sql)?;
                let mut entries = Vec::new();
                match upper {
                    Some(upper) => {
                        let rows = statement.query_map(
                            (lower, upper, now_millis(), limit),
                            decode_entry_row_sqlite,
                        )?;
                        for row in rows {
                            entries.push(row??);
                        }
                    }
                    None => {
                        let rows = statement
                            .query_map((lower, now_millis(), limit), decode_entry_row_sqlite)?;
                        for row in rows {
                            entries.push(row??);
                        }
                    }
                }
                Ok(entries)
            })
            .await
    }

    /// Check revisions and apply mutations in one SQLite transaction.
    pub async fn atomic(
        &self,
        checks: &[KvCheck],
        mutations: &[KvMutation],
    ) -> Result<AtomicCommit> {
        for check in checks {
            encode_key(&check.key)?;
        }
        for mutation in mutations {
            match mutation {
                KvMutation::Set { key, .. }
                | KvMutation::MergePatch { key, .. }
                | KvMutation::Delete { key } => {
                    encode_key(key)?;
                }
            }
        }

        let checks = checks.to_vec();
        let mutations = mutations.to_vec();
        self.database
            .call(move |connection| {
                let transaction = connection.transaction()?;
                let now = now_millis();
                for check in &checks {
                    let encoded = encode_key(&check.key)?;
                    let found = read_versionstamp(&transaction, &encoded, now)?;
                    if found != check.versionstamp {
                        transaction.rollback()?;
                        return Ok(AtomicCommit {
                            committed: false,
                            versionstamp: None,
                        });
                    }
                }

                transaction.execute(
                    "UPDATE kv_meta SET versionstamp = versionstamp + 1 WHERE singleton = 1",
                    (),
                )?;
                let versionstamp = current_versionstamp(&transaction)?;
                for mutation in &mutations {
                    match mutation {
                        KvMutation::Set {
                            key,
                            value,
                            expires_at,
                        } => {
                            let encoded = encode_key(key)?;
                            let key_json = serde_json::to_string(key).map_err(json_error)?;
                            let value_json = serde_json::to_string(value).map_err(json_error)?;
                            transaction.execute(
                                "INSERT INTO kv_entries \
                                 (key, key_json, value_json, versionstamp, expires_at) \
                                 VALUES (?1, ?2, ?3, ?4, ?5) \
                                 ON CONFLICT(key) DO UPDATE SET \
                                 key_json = excluded.key_json, value_json = excluded.value_json, \
                                 versionstamp = excluded.versionstamp, expires_at = excluded.expires_at",
                                (
                                    encoded,
                                    key_json,
                                    value_json,
                                    versionstamp_to_i64(versionstamp)?,
                                    *expires_at,
                                ),
                            )?;
                        }
                        KvMutation::MergePatch { key, patch } => {
                            let encoded = encode_key(key)?;
                            let key_json = serde_json::to_string(key).map_err(json_error)?;
                            let patch_json = serde_json::to_string(patch).map_err(json_error)?;
                            let revision = versionstamp_to_i64(versionstamp)?;
                            transaction.execute(
                                "DELETE FROM kv_entries \
                                 WHERE key = ?1 AND expires_at IS NOT NULL AND expires_at <= ?2",
                                (&encoded, now),
                            )?;
                            transaction.execute(
                                "INSERT INTO kv_entries \
                                 (key, key_json, value_json, versionstamp, expires_at) \
                                 VALUES (?1, ?2, json_patch('{}', ?3), ?4, NULL) \
                                 ON CONFLICT(key) DO UPDATE SET \
                                 key_json = excluded.key_json, \
                                 value_json = json_patch(kv_entries.value_json, ?3), \
                                 versionstamp = excluded.versionstamp",
                                (encoded, key_json, patch_json, revision),
                            )?;
                        }
                        KvMutation::Delete { key } => {
                            transaction.execute(
                                "DELETE FROM kv_entries WHERE key = ?1",
                                [encode_key(key)?],
                            )?;
                        }
                    }
                }
                transaction.commit()?;
                Ok(AtomicCommit {
                    committed: true,
                    versionstamp: Some(versionstamp),
                })
            })
            .await
    }
}

fn read_entry(connection: &crate::Connection, encoded: &[u8], now: i64) -> Result<Option<KvEntry>> {
    connection
        .query_row(
            "SELECT key_json, value_json, versionstamp, expires_at FROM kv_entries \
             WHERE key = ?1 AND (expires_at IS NULL OR expires_at > ?2)",
            (encoded, now),
            decode_entry_row_sqlite,
        )
        .optional()?
        .transpose()
}

fn read_versionstamp(
    connection: &crate::Connection,
    encoded: &[u8],
    now: i64,
) -> Result<Option<Versionstamp>> {
    connection
        .query_row(
            "SELECT versionstamp FROM kv_entries \
             WHERE key = ?1 AND (expires_at IS NULL OR expires_at > ?2)",
            (encoded, now),
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .map(i64_to_versionstamp)
        .transpose()
}

fn current_versionstamp(connection: &crate::Connection) -> Result<Versionstamp> {
    let value = connection
        .query_row(
            "SELECT versionstamp FROM kv_meta WHERE singleton = 1",
            (),
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .ok_or_else(|| Error::InvalidMigrations("KV metadata row is missing".to_owned()))?;
    i64_to_versionstamp(value)
}

fn decode_entry_row(row: &crate::Row) -> Result<KvEntry> {
    let key_json = row.get::<_, String>(0)?;
    let value_json = row.get::<_, String>(1)?;
    Ok(KvEntry {
        key: serde_json::from_str(&key_json).map_err(json_error)?,
        value: serde_json::from_str(&value_json).map_err(json_error)?,
        versionstamp: i64_to_versionstamp(row.get::<_, i64>(2)?)?,
        expires_at: row.get::<_, Option<i64>>(3)?,
    })
}

fn decode_entry_row_sqlite(
    row: &crate::Row<'_>,
) -> std::result::Result<Result<KvEntry>, rusqlite::Error> {
    Ok(decode_entry_row(row))
}

fn encode_key(key: &[KvKeyPart]) -> Result<Vec<u8>> {
    if key.is_empty() {
        return Err(Error::InvalidKey(
            "KV keys must contain at least one part".into(),
        ));
    }
    Ok(encode_prefix(key))
}

fn encode_prefix(key: &[KvKeyPart]) -> Vec<u8> {
    let mut encoded = Vec::new();
    for part in key {
        match part {
            KvKeyPart::Bool(value) => encoded.extend([0x10, u8::from(*value)]),
            KvKeyPart::I64(value) => {
                encoded.push(0x20);
                encoded.extend(((*value as u64) ^ (1_u64 << 63)).to_be_bytes());
            }
            KvKeyPart::U64(value) => {
                encoded.push(0x30);
                encoded.extend(value.to_be_bytes());
            }
            KvKeyPart::String(value) => {
                encoded.push(0x40);
                encode_variable(value.as_bytes(), &mut encoded);
            }
            KvKeyPart::Bytes(value) => {
                encoded.push(0x50);
                encode_variable(value, &mut encoded);
            }
        }
    }
    encoded
}

fn encode_variable(value: &[u8], output: &mut Vec<u8>) {
    for byte in value {
        if *byte == 0 {
            output.extend([0, 0xff]);
        } else {
            output.push(*byte);
        }
    }
    output.extend([0, 0]);
}

fn prefix_successor(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut upper = prefix.to_vec();
    for index in (0..upper.len()).rev() {
        if upper[index] != u8::MAX {
            upper[index] += 1;
            upper.truncate(index + 1);
            return Some(upper);
        }
    }
    None
}

fn now_millis() -> i64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    i64::try_from(millis).unwrap_or(i64::MAX)
}

fn i64_to_versionstamp(value: i64) -> Result<Versionstamp> {
    u64::try_from(value)
        .map(Versionstamp)
        .map_err(|_| Error::InvalidVersionstamp(value))
}

fn versionstamp_to_i64(value: Versionstamp) -> Result<i64> {
    i64::try_from(value.0).map_err(|_| Error::VersionstampExhausted)
}

fn json_error(error: serde_json::Error) -> Error {
    Error::InvalidDocument(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;

    fn temporary_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "wabou-kv-{name}-{}-{}.db",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    fn key(parts: &[&str]) -> KvKey {
        parts
            .iter()
            .map(|part| KvKeyPart::String((*part).to_owned()))
            .collect()
    }

    #[tokio::test]
    async fn persists_structured_values_and_versions() {
        let path = temporary_path("persist");
        let store = KvStore::open(&path).await.unwrap();
        let version = store
            .set(key(&["settings", "theme"]), json!({ "mode": "dark" }), None)
            .await
            .unwrap();
        assert_eq!(version, Versionstamp(1));
        drop(store);

        let reopened = KvStore::open(&path).await.unwrap();
        assert_eq!(
            reopened.get(&key(&["settings", "theme"])).await.unwrap(),
            Some(KvEntry {
                key: key(&["settings", "theme"]),
                value: json!({ "mode": "dark" }),
                versionstamp: Versionstamp(1),
                expires_at: None,
            })
        );
        drop(reopened);
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn lists_whole_component_prefixes_in_key_order() {
        let store = KvStore::open(":memory:").await.unwrap();
        for (parts, value) in [
            (("projects", "b"), 2),
            (("settings", "theme"), 3),
            (("projects", "a"), 1),
        ] {
            store
                .set(key(&[parts.0, parts.1]), json!(value), None)
                .await
                .unwrap();
        }
        let entries = store
            .list(&KvListOptions {
                prefix: key(&["projects"]),
                limit: 10,
                reverse: false,
            })
            .await
            .unwrap();
        assert_eq!(
            entries.iter().map(|entry| &entry.value).collect::<Vec<_>>(),
            vec![&json!(1), &json!(2)]
        );
    }

    #[tokio::test]
    async fn atomic_checks_prevent_lost_updates() {
        let store = KvStore::open(":memory:").await.unwrap();
        let record = key(&["drafts", "one"]);
        let first = store
            .set(record.clone(), json!("first"), None)
            .await
            .unwrap();
        let committed = store
            .atomic(
                &[KvCheck {
                    key: record.clone(),
                    versionstamp: Some(first),
                }],
                &[KvMutation::Set {
                    key: record.clone(),
                    value: json!("second"),
                    expires_at: None,
                }],
            )
            .await
            .unwrap();
        assert_eq!(committed.versionstamp, Some(Versionstamp(2)));

        let stale = store
            .atomic(
                &[KvCheck {
                    key: record.clone(),
                    versionstamp: Some(first),
                }],
                &[KvMutation::Set {
                    key: record.clone(),
                    value: json!("lost"),
                    expires_at: None,
                }],
            )
            .await
            .unwrap();
        assert!(!stale.committed);
        assert_eq!(
            store.get(&record).await.unwrap().unwrap().value,
            json!("second")
        );
    }

    #[tokio::test]
    async fn merge_patches_documents_inside_atomic_commits() {
        let store = KvStore::open(":memory:").await.unwrap();
        let record = key(&["profiles", "one"]);
        let expiry = now_millis() + 60_000;
        let first = store
            .set(
                record.clone(),
                json!({
                    "name": "Photos",
                    "settings": { "compression": 3, "verify": true },
                    "obsolete": true
                }),
                Some(expiry),
            )
            .await
            .unwrap();

        let commit = store
            .atomic(
                &[KvCheck {
                    key: record.clone(),
                    versionstamp: Some(first),
                }],
                &[KvMutation::MergePatch {
                    key: record.clone(),
                    patch: json!({
                        "settings": { "compression": 7 },
                        "obsolete": null
                    }),
                }],
            )
            .await
            .unwrap();
        assert!(commit.committed);
        let entry = store.get(&record).await.unwrap().unwrap();
        assert_eq!(
            entry.value,
            json!({
                "name": "Photos",
                "settings": { "compression": 7, "verify": true }
            })
        );
        assert_eq!(entry.expires_at, Some(expiry));

        let missing = key(&["profiles", "two"]);
        store
            .merge_patch(
                missing.clone(),
                json!({ "name": "Documents", "ignored": null }),
            )
            .await
            .unwrap();
        assert_eq!(
            store.get(&missing).await.unwrap().unwrap().value,
            json!({ "name": "Documents" })
        );
    }

    #[tokio::test]
    async fn treats_expired_entries_as_absent() {
        let store = KvStore::open(":memory:").await.unwrap();
        let record = key(&["ephemeral"]);
        store
            .set(record.clone(), json!(true), Some(now_millis() - 1))
            .await
            .unwrap();
        assert_eq!(store.get(&record).await.unwrap(), None);
        let result = store
            .atomic(
                &[KvCheck {
                    key: record.clone(),
                    versionstamp: None,
                }],
                &[KvMutation::Set {
                    key: record.clone(),
                    value: json!(false),
                    expires_at: None,
                }],
            )
            .await
            .unwrap();
        assert!(result.committed);
    }

    #[test]
    fn key_encoding_is_unambiguous_and_prefix_preserving() {
        let namespace = encode_key(&key(&["a"])).unwrap();
        let child = encode_key(&key(&["a", "b"])).unwrap();
        let escaped = encode_key(&[KvKeyPart::String("a\0b".into())]).unwrap();
        assert!(child.starts_with(&namespace));
        assert_ne!(namespace, escaped);
        assert!(encode_key(&[]).is_err());
    }
}
