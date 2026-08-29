//! Versioned, embedded application databases and key-value storage backed by SQLite.
//!
//! This crate owns database lifecycle and schema migrations. Applications own
//! their tables and expose domain-specific repositories. [`KvStore`] provides
//! a separate, explicitly namespaced document store for preferences and other
//! small application records.

#![warn(missing_docs)]

use std::error::Error as StdError;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use tokio::sync::{Mutex, MutexGuard};
pub use turso::{Connection, Row, Rows, Value, params};

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
    /// Error returned by the embedded Turso engine.
    Turso(turso::Error),
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
            Self::Turso(error) => error.fmt(formatter),
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
            Self::Turso(error) => Some(error),
            Self::InvalidMigrations(_)
            | Self::NewerSchema { .. }
            | Self::InvalidKey(_)
            | Self::InvalidDocument(_)
            | Self::InvalidVersionstamp(_)
            | Self::VersionstampExhausted => None,
        }
    }
}

impl From<turso::Error> for Error {
    fn from(error: turso::Error) -> Self {
        Self::Turso(error)
    }
}

/// Result type returned by database operations.
pub type Result<T> = std::result::Result<T, Error>;

/// An opened local database whose schema is at the requested version.
#[derive(Debug)]
pub struct Database {
    inner: turso::Database,
    version: u32,
    write_gate: Mutex<()>,
}

/// A connection holding this database's single-writer gate.
///
/// Turso's default journal mode can return `database is locked` when separate
/// connections write concurrently. Wabou serializes writes by default instead
/// of enabling the still-evolving MVCC journal mode implicitly.
pub struct WriteConnection<'database> {
    connection: Connection,
    _guard: MutexGuard<'database, ()>,
}

impl Deref for WriteConnection<'_> {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl DerefMut for WriteConnection<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

impl Database {
    /// Opens a local database and atomically applies every missing migration.
    pub async fn open(path: impl AsRef<Path>, migrations: &[Migration]) -> Result<Self> {
        validate_migrations(migrations)?;
        let supported = migrations.last().map_or(0, |migration| migration.version);
        let path = path.as_ref().to_string_lossy();
        let inner = turso::Builder::new_local(&path).build().await?;
        let mut connection = inner.connect()?;
        let found = schema_version(&connection).await?;

        if found > supported {
            return Err(Error::NewerSchema { found, supported });
        }

        if found < supported {
            let transaction = connection.transaction().await?;
            for migration in migrations
                .iter()
                .filter(|migration| migration.version > found)
            {
                transaction.execute_batch(migration.sql).await?;
                transaction
                    .execute(format!("PRAGMA user_version = {}", migration.version), ())
                    .await?;
            }
            transaction.commit().await?;
        }

        Ok(Self {
            inner,
            version: supported,
            write_gate: Mutex::new(()),
        })
    }

    /// Opens an independent connection, suitable for reads.
    ///
    /// Use [`Self::write_connection`] for mutations so writers are serialized.
    pub fn connect(&self) -> Result<Connection> {
        self.inner.connect().map_err(Into::into)
    }

    /// Opens a connection after waiting for this database's writer gate.
    pub async fn write_connection(&self) -> Result<WriteConnection<'_>> {
        let guard = self.write_gate.lock().await;
        let connection = self.inner.connect()?;
        Ok(WriteConnection {
            connection,
            _guard: guard,
        })
    }

    /// Highest migration version supported by this database instance.
    pub const fn version(&self) -> u32 {
        self.version
    }
}

async fn schema_version(connection: &Connection) -> Result<u32> {
    let mut rows = connection.query("PRAGMA user_version", ()).await?;
    let row = rows.next().await?.ok_or_else(|| {
        Error::InvalidMigrations("PRAGMA user_version returned no row".to_string())
    })?;
    let version = row.get::<i64>(0)?;
    u32::try_from(version)
        .map_err(|_| Error::InvalidMigrations(format!("invalid database schema version {version}")))
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

    async fn stored_version(connection: &Connection) -> u32 {
        schema_version(connection).await.unwrap()
    }

    #[tokio::test]
    async fn creates_and_reopens_a_versioned_database() {
        let path = temporary_path("reopen");
        let database = Database::open(&path, &[V1]).await.unwrap();
        let connection = database.connect().unwrap();
        connection
            .execute(
                "INSERT INTO preferences (singleton, theme) VALUES (1, ?1)",
                ["dark"],
            )
            .await
            .unwrap();
        drop(connection);
        drop(database);

        let reopened = Database::open(&path, &[V1]).await.unwrap();
        let connection = reopened.connect().unwrap();
        let mut rows = connection
            .query("SELECT theme FROM preferences WHERE singleton = 1", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next()
                .await
                .unwrap()
                .unwrap()
                .get::<String>(0)
                .unwrap(),
            "dark"
        );
        assert_eq!(stored_version(&connection).await, 1);
        drop(rows);
        drop(connection);
        drop(reopened);
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn upgrades_existing_data_atomically() {
        let path = temporary_path("upgrade");
        let database = Database::open(&path, &[V1]).await.unwrap();
        let connection = database.connect().unwrap();
        connection
            .execute(
                "INSERT INTO preferences (singleton, theme) VALUES (1, 'light')",
                (),
            )
            .await
            .unwrap();
        drop(connection);
        drop(database);

        let upgraded = Database::open(&path, &[V1, V2]).await.unwrap();
        let connection = upgraded.connect().unwrap();
        let mut rows = connection
            .query("SELECT theme, locale FROM preferences", ())
            .await
            .unwrap();
        let row = rows.next().await.unwrap().unwrap();
        assert_eq!(row.get::<String>(0).unwrap(), "light");
        assert_eq!(row.get::<String>(1).unwrap(), "en");
        assert_eq!(stored_version(&connection).await, 2);
        drop(rows);
        drop(connection);
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
        let connection = reopened.connect().unwrap();
        assert_eq!(stored_version(&connection).await, 1);
        assert!(
            connection
                .query("SELECT locale FROM preferences", ())
                .await
                .is_err()
        );
        drop(connection);
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
                let connection = database.write_connection().await.unwrap();
                connection
                    .execute(
                        "CREATE TABLE IF NOT EXISTS events (\
                            worker INTEGER NOT NULL, sequence INTEGER NOT NULL, \
                            PRIMARY KEY (worker, sequence)\
                        ) STRICT",
                        (),
                    )
                    .await
                    .unwrap();
                for sequence in 0..25 {
                    connection
                        .execute(
                            "INSERT INTO events (worker, sequence) VALUES (?1, ?2)",
                            turso::params![worker, sequence],
                        )
                        .await
                        .unwrap();
                }
            }));
        }

        for task in tasks {
            task.await.unwrap();
        }
        let connection = database.connect().unwrap();
        let mut rows = connection
            .query("SELECT COUNT(*) FROM events", ())
            .await
            .unwrap();
        assert_eq!(
            rows.next().await.unwrap().unwrap().get::<i64>(0).unwrap(),
            100
        );
        drop(rows);
        drop(connection);
        drop(database);
        let _ = fs::remove_file(path);
    }
}
