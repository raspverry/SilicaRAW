//! Storage and persistence boundary for SilicaRAW.
//!
//! Spike 004 selects rusqlite with bundled SQLite and embedded SQL migrations.
//! This crate owns catalog schema creation but does not scan folders, import
//! photos, mutate originals, write sidecars, or manage caches yet.

use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};
use silica_catalog::{
    ALPHA_CATALOG_REQUIRED_INDEXES, ALPHA_CATALOG_REQUIRED_TABLES, ALPHA_CATALOG_SCHEMA_VERSION,
};

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-storage";

/// SQLite binding selected by the persistence spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqliteBinding {
    /// rusqlite with bundled SQLite for deterministic local distribution builds.
    RusqliteBundled,
}

/// Migration approach selected by the persistence spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationApproach {
    /// Ordered SQL strings compiled into `silica-storage`.
    EmbeddedSqlMigrations,
}

/// Catalog journal mode selected for normal writable local libraries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogJournalMode {
    /// Use SQLite write-ahead logging for local writable catalogs.
    Wal,
}

/// Recorded output of Spike 004.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageGate {
    pub sqlite_binding: SqliteBinding,
    pub migration_approach: MigrationApproach,
    pub journal_mode: CatalogJournalMode,
    pub current_schema_version: i64,
}

/// Spike 004 decision for downstream crates and tests.
pub const SPIKE_004_STORAGE_GATE: StorageGate = StorageGate {
    sqlite_binding: SqliteBinding::RusqliteBundled,
    migration_approach: MigrationApproach::EmbeddedSqlMigrations,
    journal_mode: CatalogJournalMode::Wal,
    current_schema_version: CURRENT_SCHEMA_VERSION,
};

/// Current catalog schema version after all embedded migrations run.
pub const CURRENT_SCHEMA_VERSION: i64 = ALPHA_CATALOG_SCHEMA_VERSION;

/// Required initial tables from `docs/10_Data_Model_and_Storage_Specification.md`.
pub const REQUIRED_TABLES: &[&str] = ALPHA_CATALOG_REQUIRED_TABLES;

/// Required initial indexes from `docs/10_Data_Model_and_Storage_Specification.md`.
pub const REQUIRED_INDEXES: &[&str] = ALPHA_CATALOG_REQUIRED_INDEXES;

/// Catalog database filename inside a SilicaRAW library folder.
pub const CATALOG_DATABASE_FILE: &str = "catalog.db";

/// Stable local alpha library row id for single-library catalog databases.
pub const LOCAL_LIBRARY_ID: &str = "local";

/// Required support directories inside a SilicaRAW library folder.
pub const REQUIRED_LIBRARY_DIRECTORIES: &[&str] = &[
    "sidecars",
    "thumbnails",
    "previews",
    "render-cache",
    "ai-cache",
    "exports",
    "logs",
    "backups",
];

/// Opened or newly created local library paths and schema state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalLibrary {
    pub root_path: PathBuf,
    pub catalog_path: PathBuf,
    pub schema_version: i64,
}

/// Errors returned by local library create/open operations.
#[derive(Debug)]
pub enum LibraryStorageError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    MissingCatalog(PathBuf),
    NotDirectory(PathBuf),
    InvalidPath(PathBuf),
}

impl fmt::Display for LibraryStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "filesystem error: {error}"),
            Self::Sqlite(error) => write!(formatter, "sqlite error: {error}"),
            Self::MissingCatalog(path) => {
                write!(formatter, "missing catalog database at {}", path.display())
            }
            Self::NotDirectory(path) => write!(formatter, "not a directory: {}", path.display()),
            Self::InvalidPath(path) => {
                write!(formatter, "path is not valid UTF-8: {}", path.display())
            }
        }
    }
}

impl Error for LibraryStorageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Sqlite(error) => Some(error),
            Self::MissingCatalog(_) | Self::NotDirectory(_) | Self::InvalidPath(_) => None,
        }
    }
}

impl From<std::io::Error> for LibraryStorageError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<rusqlite::Error> for LibraryStorageError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

#[derive(Debug, Clone, Copy)]
struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_catalog_schema",
        sql: INITIAL_CATALOG_SCHEMA_SQL,
    },
    Migration {
        version: 2,
        name: "required_catalog_indexes",
        sql: REQUIRED_INDEXES_SQL,
    },
];

/// Open a catalog database and apply all embedded migrations.
pub fn open_catalog(path: impl AsRef<Path>) -> rusqlite::Result<Connection> {
    let mut connection = Connection::open(path)?;
    configure_connection(&connection)?;
    run_migrations(&mut connection)?;
    Ok(connection)
}

/// Create a local SilicaRAW library folder and initialize its catalog.
pub fn create_local_library(
    root_path: impl AsRef<Path>,
) -> Result<LocalLibrary, LibraryStorageError> {
    let root_path = root_path.as_ref();
    fs::create_dir_all(root_path)?;
    ensure_library_directories(root_path)?;

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    open_library_catalog(root_path, &catalog_path)
}

/// Open an existing local SilicaRAW library folder and migrate its catalog.
pub fn open_local_library(
    root_path: impl AsRef<Path>,
) -> Result<LocalLibrary, LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    ensure_library_directories(root_path)?;
    open_library_catalog(root_path, &catalog_path)
}

/// Apply connection-local safety and durability settings.
pub fn configure_connection(connection: &Connection) -> rusqlite::Result<()> {
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

/// Apply every embedded migration that has not already run.
pub fn run_migrations(connection: &mut Connection) -> rusqlite::Result<()> {
    run_migrations_through(connection, CURRENT_SCHEMA_VERSION)
}

/// Apply embedded migrations up to a target version.
pub fn run_migrations_through(
    connection: &mut Connection,
    target_version: i64,
) -> rusqlite::Result<()> {
    ensure_migration_table(connection)?;
    let applied_version = applied_schema_version(connection)?;

    for migration in MIGRATIONS.iter().filter(|migration| {
        migration.version > applied_version && migration.version <= target_version
    }) {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations(version, name) VALUES (?1, ?2)",
            params![migration.version, migration.name],
        )?;
        transaction.commit()?;
    }

    Ok(())
}

/// Return the highest migration version applied to the catalog.
pub fn current_schema_version(connection: &Connection) -> rusqlite::Result<i64> {
    ensure_migration_table(connection)?;
    applied_schema_version(connection)
}

/// Check whether a required table or index exists in `sqlite_master`.
pub fn catalog_object_exists(connection: &Connection, name: &str) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE name = ?1 LIMIT 1",
            params![name],
            |_| Ok(()),
        )
        .optional()
        .map(|result| result.is_some())
}

fn ensure_library_directories(root_path: &Path) -> Result<(), LibraryStorageError> {
    for directory in REQUIRED_LIBRARY_DIRECTORIES {
        fs::create_dir_all(root_path.join(directory))?;
    }
    Ok(())
}

fn open_library_catalog(
    root_path: &Path,
    catalog_path: &Path,
) -> Result<LocalLibrary, LibraryStorageError> {
    let connection = open_catalog(catalog_path)?;
    upsert_local_library_row(&connection, root_path)?;
    let schema_version = current_schema_version(&connection)?;

    Ok(LocalLibrary {
        root_path: root_path.to_path_buf(),
        catalog_path: catalog_path.to_path_buf(),
        schema_version,
    })
}

fn upsert_local_library_row(
    connection: &Connection,
    root_path: &Path,
) -> Result<(), LibraryStorageError> {
    let root_path = root_path
        .to_str()
        .ok_or_else(|| LibraryStorageError::InvalidPath(root_path.to_path_buf()))?;

    connection.execute(
        r#"
        INSERT INTO libraries(id, root_path)
        VALUES (?1, ?2)
        ON CONFLICT(id) DO UPDATE SET
          root_path = excluded.root_path,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![LOCAL_LIBRARY_ID, root_path],
    )?;

    Ok(())
}

fn ensure_migration_table(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
          version INTEGER PRIMARY KEY,
          name TEXT NOT NULL,
          applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
}

fn applied_schema_version(connection: &Connection) -> rusqlite::Result<i64> {
    connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
}

const INITIAL_CATALOG_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS libraries (
  id TEXT PRIMARY KEY,
  root_path TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS folders (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  path TEXT NOT NULL,
  scanned_at TEXT,
  missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
  FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
  UNIQUE (library_id, path)
);

CREATE TABLE IF NOT EXISTS photos (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  folder_id TEXT NOT NULL,
  file_name TEXT NOT NULL,
  path TEXT NOT NULL,
  file_size INTEGER NOT NULL DEFAULT 0,
  modified_at TEXT,
  capture_time TEXT,
  imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
  unsupported INTEGER NOT NULL DEFAULT 0 CHECK (unsupported IN (0, 1)),
  partial_hash TEXT,
  full_hash TEXT,
  FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
  FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE,
  UNIQUE (library_id, path)
);

CREATE TABLE IF NOT EXISTS photo_metadata (
  photo_id TEXT PRIMARY KEY,
  camera_make TEXT,
  camera_model TEXT,
  lens_model TEXT,
  capture_time TEXT,
  raw_json TEXT NOT NULL DEFAULT '{}',
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS photo_flags (
  photo_id TEXT PRIMARY KEY,
  rating INTEGER NOT NULL DEFAULT 0 CHECK (rating BETWEEN 0 AND 5),
  rejected INTEGER NOT NULL DEFAULT 0 CHECK (rejected IN (0, 1)),
  picked INTEGER NOT NULL DEFAULT 0 CHECK (picked IN (0, 1)),
  color_label TEXT,
  edited INTEGER NOT NULL DEFAULT 0 CHECK (edited IN (0, 1)),
  exported INTEGER NOT NULL DEFAULT 0 CHECK (exported IN (0, 1)),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS collections (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
  UNIQUE (library_id, name)
);

CREATE TABLE IF NOT EXISTS collection_photos (
  collection_id TEXT NOT NULL,
  photo_id TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (collection_id, photo_id),
  FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS edit_states (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  edit_graph_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS edit_history (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  edit_state_id TEXT,
  action_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE,
  FOREIGN KEY (edit_state_id) REFERENCES edit_states(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  preset_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sidecar_status (
  photo_id TEXT PRIMARY KEY,
  sidecar_path TEXT,
  last_written_at TEXT,
  conflict_state TEXT NOT NULL DEFAULT 'clean',
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS cache_records (
  id TEXT PRIMARY KEY,
  photo_id TEXT,
  cache_type TEXT NOT NULL,
  cache_key TEXT NOT NULL UNIQUE,
  path TEXT NOT NULL,
  byte_size INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_accessed_at TEXT,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ai_results (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  task_type TEXT NOT NULL,
  model_id TEXT NOT NULL,
  result_json TEXT NOT NULL,
  approved INTEGER NOT NULL DEFAULT 0 CHECK (approved IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS exports (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  output_path TEXT NOT NULL,
  export_settings_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS action_log (
  id TEXT PRIMARY KEY,
  actor_type TEXT NOT NULL,
  actor_id TEXT,
  action_type TEXT NOT NULL,
  subject_type TEXT,
  subject_id TEXT,
  payload_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

const REQUIRED_INDEXES_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_folders_library_id
  ON folders(library_id);

CREATE INDEX IF NOT EXISTS idx_photos_library_id
  ON photos(library_id);

CREATE INDEX IF NOT EXISTS idx_photos_folder_id
  ON photos(folder_id);

CREATE INDEX IF NOT EXISTS idx_photos_capture_time
  ON photos(capture_time);

CREATE INDEX IF NOT EXISTS idx_photos_imported_at
  ON photos(imported_at);

CREATE INDEX IF NOT EXISTS idx_photos_missing
  ON photos(missing);

CREATE INDEX IF NOT EXISTS idx_photos_unsupported
  ON photos(unsupported);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rating
  ON photo_flags(rating);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rejected
  ON photo_flags(rejected);

CREATE INDEX IF NOT EXISTS idx_photo_flags_picked
  ON photo_flags(picked);

CREATE INDEX IF NOT EXISTS idx_photo_flags_label
  ON photo_flags(color_label);

CREATE INDEX IF NOT EXISTS idx_collections_library_id
  ON collections(library_id);

CREATE INDEX IF NOT EXISTS idx_collection_photos_photo_id
  ON collection_photos(photo_id);

CREATE INDEX IF NOT EXISTS idx_edit_states_photo_id
  ON edit_states(photo_id);

CREATE INDEX IF NOT EXISTS idx_edit_states_photo_active
  ON edit_states(photo_id, active);

CREATE INDEX IF NOT EXISTS idx_edit_history_photo_id
  ON edit_history(photo_id);

CREATE INDEX IF NOT EXISTS idx_cache_records_photo_type
  ON cache_records(photo_id, cache_type);

CREATE INDEX IF NOT EXISTS idx_cache_records_key
  ON cache_records(cache_key);

CREATE INDEX IF NOT EXISTS idx_ai_results_photo_task
  ON ai_results(photo_id, task_type);

CREATE INDEX IF NOT EXISTS idx_ai_results_model
  ON ai_results(model_id, task_type);

CREATE INDEX IF NOT EXISTS idx_exports_photo_id
  ON exports(photo_id);

CREATE INDEX IF NOT EXISTS idx_action_log_actor
  ON action_log(actor_type, actor_id);

CREATE INDEX IF NOT EXISTS idx_action_log_created_at
  ON action_log(created_at);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn exposes_crate_name() {
        assert_eq!(CRATE_NAME, "silica-storage");
    }

    #[test]
    fn records_spike_004_storage_gate() {
        assert_eq!(
            SPIKE_004_STORAGE_GATE.sqlite_binding,
            SqliteBinding::RusqliteBundled
        );
        assert_eq!(
            SPIKE_004_STORAGE_GATE.migration_approach,
            MigrationApproach::EmbeddedSqlMigrations
        );
        assert_eq!(SPIKE_004_STORAGE_GATE.journal_mode, CatalogJournalMode::Wal);
        assert_eq!(
            SPIKE_004_STORAGE_GATE.current_schema_version,
            CURRENT_SCHEMA_VERSION
        );
    }

    #[test]
    fn creates_empty_catalog_schema_and_required_indexes() {
        let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
        configure_connection(&connection).expect("configure sqlite");
        run_migrations(&mut connection).expect("run migrations");

        assert_eq!(
            current_schema_version(&connection).expect("schema version"),
            CURRENT_SCHEMA_VERSION
        );
        assert!(catalog_object_exists(&connection, "libraries").expect("libraries table exists"));
        assert!(catalog_object_exists(&connection, "photos").expect("photos table exists"));
        assert!(catalog_object_exists(&connection, "schema_migrations").expect("migrations table"));

        for table_name in REQUIRED_TABLES {
            assert!(
                catalog_object_exists(&connection, table_name).expect("table lookup"),
                "missing required table {table_name}"
            );
        }

        for index_name in REQUIRED_INDEXES {
            assert!(
                catalog_object_exists(&connection, index_name).expect("index lookup"),
                "missing required index {index_name}"
            );
        }
    }

    #[test]
    fn upgrades_empty_catalog_from_first_migration() {
        let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
        configure_connection(&connection).expect("configure sqlite");

        run_migrations_through(&mut connection, 1).expect("run first migration");
        assert_eq!(current_schema_version(&connection).expect("version"), 1);
        assert!(
            !catalog_object_exists(&connection, "idx_photos_library_id").expect("index lookup"),
            "index migration should not have run yet"
        );

        run_migrations(&mut connection).expect("upgrade to latest");
        assert_eq!(
            current_schema_version(&connection).expect("version"),
            CURRENT_SCHEMA_VERSION
        );
        assert!(catalog_object_exists(&connection, "idx_photos_library_id").expect("index lookup"));
    }

    #[test]
    fn enforces_foreign_keys_for_catalog_rows() {
        let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
        configure_connection(&connection).expect("configure sqlite");
        run_migrations(&mut connection).expect("run migrations");

        let result = connection.execute(
            r#"
            INSERT INTO photos(id, library_id, folder_id, file_name, path)
            VALUES ('photo-1', 'missing-library', 'missing-folder', 'sample.dng', '/tmp/sample.dng')
            "#,
            [],
        );

        assert!(result.is_err(), "foreign key violation should fail");
    }

    #[test]
    fn opens_file_backed_catalog_with_wal_and_foreign_keys() {
        let path = unique_catalog_path("wal");

        {
            let connection = open_catalog(&path).expect("open file-backed catalog");
            assert_eq!(
                current_schema_version(&connection).expect("version"),
                CURRENT_SCHEMA_VERSION
            );

            let journal_mode: String = connection
                .query_row("PRAGMA journal_mode", [], |row| row.get(0))
                .expect("journal mode");
            assert_eq!(journal_mode.to_ascii_lowercase(), "wal");

            let foreign_keys: i64 = connection
                .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                .expect("foreign keys");
            assert_eq!(foreign_keys, 1);
        }

        remove_catalog_files(&path);
    }

    #[test]
    fn creates_library_folder_with_catalog_and_support_directories() {
        let root = unique_library_root("create");

        {
            let library = create_local_library(&root).expect("create local library");
            assert_eq!(library.root_path, root);
            assert_eq!(library.catalog_path, root.join(CATALOG_DATABASE_FILE));
            assert_eq!(library.schema_version, CURRENT_SCHEMA_VERSION);
            assert!(library.catalog_path.is_file());

            for directory in REQUIRED_LIBRARY_DIRECTORIES {
                assert!(
                    root.join(directory).is_dir(),
                    "missing library directory {directory}"
                );
            }
        }

        remove_library_root(&root);
    }

    #[test]
    fn reopens_existing_library_without_recreating_original_photo_directory() {
        let workspace = unique_library_root("workspace");
        let original_dir = workspace.join("originals");
        let original_file = original_dir.join("sample.dng");
        let library_root = workspace.join("SilicaRAW Library");

        std::fs::create_dir_all(&original_dir).expect("create original directory");
        std::fs::write(&original_file, b"original raw bytes").expect("write original sentinel");
        let original_before = std::fs::read(&original_file).expect("read original before");

        let created = create_local_library(&library_root).expect("create library");
        let reopened = open_local_library(&library_root).expect("reopen library");

        assert_eq!(reopened.root_path, created.root_path);
        assert_eq!(reopened.catalog_path, created.catalog_path);
        assert_eq!(reopened.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(
            std::fs::read(&original_file).expect("read original after"),
            original_before
        );
        assert!(original_dir.is_dir());

        remove_library_root(&workspace);
    }

    fn unique_catalog_path(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "silicaraw-storage-{label}-{}-{nanos}.db",
            std::process::id()
        ))
    }

    fn unique_library_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "silicaraw-library-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn remove_catalog_files(path: &Path) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{}", path.display(), suffix));
        }
    }

    fn remove_library_root(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }
}
