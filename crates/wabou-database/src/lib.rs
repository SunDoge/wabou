//! Versioned, embedded application databases and key-value storage backed by SQLite.
//!
//! This crate owns database lifecycle and schema migrations. Applications own
//! their tables and expose domain-specific repositories. [`KvStore`] provides
//! a separate, explicitly namespaced document store for preferences and other
//! small application records.

#![warn(missing_docs)]

use std::error::Error as StdError;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub use rusqlite::{Connection, Row, Rows, params};

mod kv;

pub use kv::{
    AtomicCommit, KvCheck, KvEntry, KvKey, KvKeyPart, KvListOptions, KvMutation, KvStore,
    Versionstamp,
};

/// A schema change from version `version - 1` to `version`.
#[derive(Clone, Copy, Debug)]
pub struct Migration {
    /// Target schema version after applying this migration.
    pub version: u32,
    /// SQL batch applied atomically before updating `PRAGMA user_version`.
    pub sql: &'static str,
}

impl Migration {
    /// Declare a migration from `version - 1` to `version`.
    pub const fn new(version: u32, sql: &'static str) -> Self {
        Self { version, sql }
    }
}

#[derive(Debug)]
/// Database opening, migration, or query error.
pub enum Error {
    /// Error returned by SQLite.
    Sqlite(rusqlite::Error),
    /// A database task could not run to completion.
    Task(String),
    /// Migration sequence is non-contiguous, empty, or otherwise invalid.
    InvalidMigrations(String),
    /// On-disk schema is newer than this application understands.
    NewerSchema {
        /// Version stored in the database.
        found: u32,
        /// Highest version supplied by this application.
        supported: u32,
    },
    /// A hierarchical key is malformed.
    InvalidKey(String),
    /// A stored JSON document could not be encoded or decoded.
    InvalidDocument(String),
    /// A corrupt negative revision was read from SQLite.
    InvalidVersionstamp(i64),
    /// The SQLite signed integer revision range has been exhausted.
    VersionstampExhausted,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => error.fmt(formatter),
            Self::Task(message) => formatter.write_str(message),
            Self::InvalidMigrations(message) => formatter.write_str(message),
            Self::NewerSchema { found, supported } => write!(
                formatter,
                "database schema version {found} is newer than supported version {supported}"
            ),
            Self::InvalidKey(message) | Self::InvalidDocument(message) => {
                formatter.write_str(message)
            }
            Self::InvalidVersionstamp(value) => {
                write!(formatter, "invalid negative KV versionstamp {value}")
            }
            Self::VersionstampExhausted => formatter.write_str("KV versionstamp exhausted"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Task(_)
            | Self::InvalidMigrations(_)
            | Self::NewerSchema { .. }
            | Self::InvalidKey(_)
            | Self::InvalidDocument(_)
            | Self::InvalidVersionstamp(_)
            | Self::VersionstampExhausted => None,
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

/// Result type returned by database operations.
pub type Result<T> = std::result::Result<T, Error>;

/// An opened local database whose schema is at the requested version.
#[derive(Debug)]
pub struct Database {
    inner: Arc<Mutex<Connection>>,
    version: u32,
}

impl Database {
    /// Opens a local database and atomically applies every missing migration.
    pub async fn open(path: impl AsRef<Path>, migrations: &[Migration]) -> Result<Self> {
        validate_migrations(migrations)?;
        let supported = migrations.last().map_or(0, |migration| migration.version);
        let path = path.as_ref().to_owned();
        let migrations = migrations.to_vec();
        let connection = tokio::task::spawn_blocking(move || -> Result<Connection> {
            let mut connection = Connection::open(path)?;
            let found = schema_version(&connection)?;

            if found > supported {
                return Err(Error::NewerSchema { found, supported });
            }

            if found < supported {
                let transaction = connection.transaction()?;
                for migration in migrations
                    .iter()
                    .filter(|migration| migration.version > found)
                {
                    transaction.execute_batch(migration.sql)?;
                    transaction.pragma_update(None, "user_version", migration.version)?;
                }
                transaction.commit()?;
            }
            Ok(connection)
        })
        .await
        .map_err(task_error)??;

        Ok(Self {
            inner: Arc::new(Mutex::new(connection)),
            version: supported,
        })
    }

    /// Run one serialized database operation outside the async executor thread.
    pub async fn call<T, F>(&self, operation: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T> + Send + 'static,
    {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || {
            let mut connection = inner
                .lock()
                .map_err(|_| Error::Task("SQLite connection lock was poisoned".to_owned()))?;
            operation(&mut connection)
        })
        .await
        .map_err(task_error)?
    }

    /// Highest migration version supported by this database instance.
    pub const fn version(&self) -> u32 {
        self.version
    }
}

fn schema_version(connection: &Connection) -> Result<u32> {
    let version =
        connection.pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))?;
    u32::try_from(version)
        .map_err(|_| Error::InvalidMigrations(format!("invalid database schema version {version}")))
}

fn task_error(error: tokio::task::JoinError) -> Error {
    Error::Task(format!("SQLite task failed: {error}"))
}

fn validate_migrations(migrations: &[Migration]) -> Result<()> {
    for (index, migration) in migrations.iter().enumerate() {
        let expected = u32::try_from(index + 1)
            .map_err(|_| Error::InvalidMigrations("too many database migrations".to_string()))?;
        if migration.version != expected {
            return Err(Error::InvalidMigrations(format!(
                "migration versions must be contiguous from 1; expected {expected}, found {}",
                migration.version
            )));
        }
        if migration.sql.trim().is_empty() {
            return Err(Error::InvalidMigrations(format!(
                "migration {} is empty",
                migration.version
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    const V1: Migration = Migration::new(
        1,
        "CREATE TABLE preferences (\
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1), \
            theme TEXT NOT NULL CHECK (theme IN ('light', 'dark'))\
        ) STRICT;",
    );
    const V2: Migration = Migration::new(
        2,
        "ALTER TABLE preferences ADD COLUMN locale TEXT NOT NULL DEFAULT 'en';",
    );

    fn temporary_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "wabou-database-{name}-{}-{}.db",
            std::process::id(),
            uuid::Uuid::new_v4()
        ))
    }

    #[tokio::test]
    async fn creates_and_reopens_a_versioned_database() {
        let path = temporary_path("reopen");
        let database = Database::open(&path, &[V1]).await.unwrap();
        database
            .call(|connection| {
                connection.execute(
                    "INSERT INTO preferences (singleton, theme) VALUES (1, ?1)",
                    ["dark"],
                )?;
                Ok(())
            })
            .await
            .unwrap();
        drop(database);

        let reopened = Database::open(&path, &[V1]).await.unwrap();
        let (theme, version) = reopened
            .call(|connection| {
                let theme = connection.query_row(
                    "SELECT theme FROM preferences WHERE singleton = 1",
                    (),
                    |row| row.get::<_, String>(0),
                )?;
                Ok((theme, schema_version(connection)?))
            })
            .await
            .unwrap();
        assert_eq!(theme, "dark");
        assert_eq!(version, 1);
        drop(reopened);
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn upgrades_existing_data_atomically() {
        let path = temporary_path("upgrade");
        let database = Database::open(&path, &[V1]).await.unwrap();
        database
            .call(|connection| {
                connection.execute(
                    "INSERT INTO preferences (singleton, theme) VALUES (1, 'light')",
                    (),
                )?;
                Ok(())
            })
            .await
            .unwrap();
        drop(database);

        let upgraded = Database::open(&path, &[V1, V2]).await.unwrap();
        let (theme, locale, version) = upgraded
            .call(|connection| {
                let (theme, locale) =
                    connection.query_row("SELECT theme, locale FROM preferences", (), |row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    })?;
                Ok((theme, locale, schema_version(connection)?))
            })
            .await
            .unwrap();
        assert_eq!(theme, "light");
        assert_eq!(locale, "en");
        assert_eq!(version, 2);
        drop(upgraded);
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn failed_upgrade_rolls_back_schema_and_version() {
        let path = temporary_path("rollback");
        let database = Database::open(&path, &[V1]).await.unwrap();
        drop(database);
        let broken = Migration::new(
            2,
            "ALTER TABLE preferences ADD COLUMN locale TEXT; INVALID SQL;",
        );

        assert!(Database::open(&path, &[V1, broken]).await.is_err());

        let reopened = Database::open(&path, &[V1]).await.unwrap();
        reopened
            .call(|connection| {
                assert_eq!(schema_version(connection).unwrap(), 1);
                assert!(
                    connection
                        .prepare("SELECT locale FROM preferences")
                        .is_err()
                );
                Ok(())
            })
            .await
            .unwrap();
        drop(reopened);
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn rejects_non_contiguous_and_newer_schemas() {
        let skipped = Migration::new(2, "CREATE TABLE skipped (id INTEGER);");
        assert!(matches!(
            Database::open(":memory:", &[skipped]).await,
            Err(Error::InvalidMigrations(_))
        ));

        let path = temporary_path("newer");
        let database = Database::open(&path, &[V1, V2]).await.unwrap();
        drop(database);
        assert!(matches!(
            Database::open(&path, &[V1]).await,
            Err(Error::NewerSchema {
                found: 2,
                supported: 1
            })
        ));
        let _ = fs::remove_file(path);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn supports_concurrent_connections() {
        let path = temporary_path("concurrent");
        let database = Arc::new(Database::open(&path, &[V1]).await.unwrap());
        let mut tasks = Vec::new();

        for worker in 0..4 {
            let database = database.clone();
            tasks.push(tokio::spawn(async move {
                database
                    .call(move |connection| {
                        connection.execute(
                            "CREATE TABLE IF NOT EXISTS events (\
                            worker INTEGER NOT NULL, sequence INTEGER NOT NULL, \
                            PRIMARY KEY (worker, sequence)\
                        ) STRICT",
                            (),
                        )?;
                        for sequence in 0..25 {
                            connection.execute(
                                "INSERT INTO events (worker, sequence) VALUES (?1, ?2)",
                                params![worker, sequence],
                            )?;
                        }
                        Ok(())
                    })
                    .await
                    .unwrap();
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }
        let count = database
            .call(|connection| {
                connection
                    .query_row("SELECT COUNT(*) FROM events", (), |row| {
                        row.get::<_, i64>(0)
                    })
                    .map_err(Into::into)
            })
            .await
            .unwrap();
        assert_eq!(count, 100);
        drop(database);
        let _ = fs::remove_file(path);
    }
}
