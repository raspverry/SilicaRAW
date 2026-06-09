//! Storage and persistence boundary for SilicaRAW.
//!
//! Spike 004 selects rusqlite with bundled SQLite and embedded SQL migrations.
//! This crate owns catalog schema creation but does not scan folders, import
//! photos, mutate originals, write sidecars, or manage caches yet.

use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use rusqlite::{params, Connection, OptionalExtension};
pub use silica_catalog::PhotoFlags;
use silica_catalog::{
    is_supported_photo_extension, CatalogFlagError, ImportCandidate,
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

/// Summary returned after importing a folder by reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderImportSummary {
    pub folder_path: PathBuf,
    pub scanned_files: usize,
    pub supported_files: usize,
    pub unsupported_files: usize,
    pub candidates: Vec<ImportCandidate>,
}

/// Catalog row data needed to open a preview for one photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoPreviewCandidate {
    pub photo_id: String,
    pub file_name: String,
    pub path: String,
    pub unsupported: bool,
}

/// Catalog export row written after an export completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRecord {
    pub id: String,
    pub photo_id: String,
    pub output_path: String,
    pub export_settings_json: String,
}

/// Errors returned by local library create/open operations.
#[derive(Debug)]
pub enum LibraryStorageError {
    Io(std::io::Error),
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    CatalogFlag(CatalogFlagError),
    EditGraph(silica_edit::EditGraphValidationError),
    MissingCatalog(PathBuf),
    NotDirectory(PathBuf),
    InvalidPath(PathBuf),
}

impl fmt::Display for LibraryStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "filesystem error: {error}"),
            Self::Sqlite(error) => write!(formatter, "sqlite error: {error}"),
            Self::Json(error) => write!(formatter, "json error: {error}"),
            Self::CatalogFlag(error) => write!(formatter, "catalog flag error: {error}"),
            Self::EditGraph(error) => write!(formatter, "edit graph error: {error}"),
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
            Self::Json(error) => Some(error),
            Self::CatalogFlag(error) => Some(error),
            Self::EditGraph(error) => Some(error),
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

impl From<serde_json::Error> for LibraryStorageError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<CatalogFlagError> for LibraryStorageError {
    fn from(error: CatalogFlagError) -> Self {
        Self::CatalogFlag(error)
    }
}

impl From<silica_edit::EditGraphValidationError> for LibraryStorageError {
    fn from(error: silica_edit::EditGraphValidationError) -> Self {
        Self::EditGraph(error)
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

/// Scan a selected folder and record file candidates by reference.
pub fn import_folder(
    library_root_path: impl AsRef<Path>,
    folder_path: impl AsRef<Path>,
) -> Result<FolderImportSummary, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let folder_path = folder_path.as_ref();
    if !folder_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(folder_path.to_path_buf()));
    }

    let mut candidates = scan_import_candidates(folder_path)?;
    candidates.sort_by(|left, right| left.path.cmp(&right.path));

    let mut connection = open_catalog(&library.catalog_path)?;
    record_import_candidates(&mut connection, folder_path, &candidates)?;

    let unsupported_files = candidates
        .iter()
        .filter(|candidate| candidate.unsupported)
        .count();
    let scanned_files = candidates.len();

    Ok(FolderImportSummary {
        folder_path: folder_path.to_path_buf(),
        scanned_files,
        supported_files: scanned_files - unsupported_files,
        unsupported_files,
        candidates,
    })
}

/// Persist culling and label flags for a photo in the catalog.
pub fn set_photo_flags(
    library_root_path: impl AsRef<Path>,
    photo_id: impl Into<String>,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
) -> Result<PhotoFlags, LibraryStorageError> {
    let flags = PhotoFlags::new(photo_id, rating, picked, rejected, color_label)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;

    connection.execute(
        r#"
        INSERT INTO photo_flags(photo_id, rating, picked, rejected, color_label, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, CURRENT_TIMESTAMP)
        ON CONFLICT(photo_id) DO UPDATE SET
          rating = excluded.rating,
          picked = excluded.picked,
          rejected = excluded.rejected,
          color_label = excluded.color_label,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            flags.photo_id,
            i64::from(flags.rating),
            bool_to_sql(flags.picked),
            bool_to_sql(flags.rejected),
            flags.color_label,
        ],
    )?;

    Ok(flags)
}

/// Read culling and label flags for a photo from the authoritative catalog row.
pub fn get_photo_flags(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoFlags>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let row = connection
        .query_row(
            r#"
            SELECT photo_id, rating, picked, rejected, color_label
            FROM photo_flags
            WHERE photo_id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .optional()?;

    row.map(|(photo_id, rating, picked, rejected, color_label)| {
        photo_flags_from_row(photo_id, rating, picked, rejected, color_label)
    })
    .transpose()
}

/// Read the catalog fields needed to decide whether a photo can open a preview.
pub fn get_photo_preview_candidate(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoPreviewCandidate>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    connection
        .query_row(
            r#"
            SELECT id, file_name, path, unsupported
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(PhotoPreviewCandidate {
                    photo_id: row.get(0)?,
                    file_name: row.get(1)?,
                    path: row.get(2)?,
                    unsupported: sql_to_bool(row.get::<_, i64>(3)?),
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

/// Load the active edit graph for a photo, or build a default draft without writing it.
pub fn load_active_edit_graph_or_default(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<silica_edit::EditGraph>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_local_library(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;

    if let Some(json) = connection
        .query_row(
            r#"
            SELECT edit_graph_json
            FROM edit_states
            WHERE photo_id = ?1 AND active = 1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
            params![photo_id],
            |row| row.get::<_, String>(0),
        )
        .optional()?
    {
        let graph: silica_edit::EditGraph = serde_json::from_str(&json)?;
        silica_edit::validate_edit_graph(&graph)?;
        return Ok(Some(graph));
    }

    let source = connection
        .query_row(
            r#"
            SELECT id, path, file_size, modified_at, partial_hash, full_hash
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(silica_edit::EditGraphSource {
                    photo_id: row.get(0)?,
                    path: row.get(1)?,
                    file_size: row.get(2)?,
                    modified_at: row.get(3)?,
                    partial_hash: row.get(4)?,
                    full_hash: row.get(5)?,
                })
            },
        )
        .optional()?;

    Ok(source.map(|source| silica_edit::default_edit_graph(source, current_timestamp_string())))
}

/// Persist the active edit graph for a photo. Draft preview updates should not call this.
pub fn commit_edit_graph(
    library_root_path: impl AsRef<Path>,
    graph: silica_edit::EditGraph,
) -> Result<silica_edit::EditGraph, LibraryStorageError> {
    silica_edit::validate_edit_graph(&graph)?;

    let library = open_local_library(library_root_path)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    let photo_id = graph.source.photo_id.clone();
    let edit_state_id = stable_catalog_id("edit-state", &photo_id);
    let edit_graph_json = serde_json::to_string(&graph)?;

    let transaction = connection.transaction()?;
    transaction.execute(
        "UPDATE edit_states SET active = 0 WHERE photo_id = ?1 AND id <> ?2",
        params![photo_id, edit_state_id],
    )?;
    transaction.execute(
        r#"
        INSERT INTO edit_states(id, photo_id, active, edit_graph_json, updated_at)
        VALUES (?1, ?2, 1, ?3, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
          active = 1,
          edit_graph_json = excluded.edit_graph_json,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![edit_state_id, photo_id, edit_graph_json],
    )?;
    transaction.execute(
        r#"
        UPDATE photo_flags
        SET edited = 1, updated_at = CURRENT_TIMESTAMP
        WHERE photo_id = ?1
        "#,
        params![graph.source.photo_id],
    )?;
    transaction.commit()?;

    Ok(graph)
}

/// Record a completed export and mark the source photo as exported.
pub fn record_export(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    export_settings_json: impl AsRef<str>,
) -> Result<ExportRecord, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let output_path = path_to_string(output_path.as_ref())?;
    let export_settings_json = export_settings_json.as_ref().to_string();
    serde_json::from_str::<serde_json::Value>(&export_settings_json)?;

    let library = open_local_library(library_root_path)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    let export_id = stable_catalog_id("export", &format!("{photo_id}\n{output_path}"));

    let transaction = connection.transaction()?;
    transaction.execute(
        r#"
        INSERT INTO exports(id, photo_id, output_path, export_settings_json, created_at)
        VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
          output_path = excluded.output_path,
          export_settings_json = excluded.export_settings_json,
          created_at = CURRENT_TIMESTAMP
        "#,
        params![export_id, photo_id, output_path, export_settings_json],
    )?;
    transaction.execute(
        r#"
        INSERT INTO photo_flags(photo_id, exported, updated_at)
        VALUES (?1, 1, CURRENT_TIMESTAMP)
        ON CONFLICT(photo_id) DO UPDATE SET
          exported = 1,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![photo_id],
    )?;
    transaction.commit()?;

    Ok(ExportRecord {
        id: export_id,
        photo_id: photo_id.to_string(),
        output_path,
        export_settings_json,
    })
}

/// Read the most recent export record for one photo.
pub fn get_latest_export_record(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<ExportRecord>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    connection
        .query_row(
            r#"
            SELECT id, photo_id, output_path, export_settings_json
            FROM exports
            WHERE photo_id = ?1
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
            params![photo_id],
            export_record_from_row,
        )
        .optional()
        .map_err(LibraryStorageError::from)
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

fn open_existing_library_for_read(
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

    let connection = open_catalog(&catalog_path)?;
    let schema_version = current_schema_version(&connection)?;

    Ok(LocalLibrary {
        root_path: root_path.to_path_buf(),
        catalog_path,
        schema_version,
    })
}

fn upsert_local_library_row(
    connection: &Connection,
    root_path: &Path,
) -> Result<(), LibraryStorageError> {
    let root_path = path_to_string(root_path)?;

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

fn scan_import_candidates(folder_path: &Path) -> Result<Vec<ImportCandidate>, LibraryStorageError> {
    let mut candidates = Vec::new();

    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let metadata = entry.metadata()?;
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let unsupported = !is_supported_photo_extension(extension);

        candidates.push(ImportCandidate {
            file_name,
            path: path_to_string(&path)?,
            file_size: metadata.len() as i64,
            modified_at: modified_at_string(&metadata),
            partial_hash: partial_file_hash(&path)?,
            unsupported,
        });
    }

    Ok(candidates)
}

fn record_import_candidates(
    connection: &mut Connection,
    folder_path: &Path,
    candidates: &[ImportCandidate],
) -> Result<(), LibraryStorageError> {
    let folder_path = path_to_string(folder_path)?;
    let folder_id = stable_catalog_id("folder", &folder_path);
    let transaction = connection.transaction()?;

    transaction.execute(
        r#"
        INSERT INTO folders(id, library_id, path, scanned_at, missing)
        VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, 0)
        ON CONFLICT(library_id, path) DO UPDATE SET
          scanned_at = CURRENT_TIMESTAMP,
          missing = 0
        "#,
        params![folder_id, LOCAL_LIBRARY_ID, folder_path],
    )?;

    for candidate in candidates {
        let photo_id = stable_catalog_id("photo", &candidate.path);
        transaction.execute(
            r#"
            INSERT INTO photos(
              id,
              library_id,
              folder_id,
              file_name,
              path,
              file_size,
              modified_at,
              missing,
              unsupported,
              partial_hash
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)
            ON CONFLICT(library_id, path) DO UPDATE SET
              folder_id = excluded.folder_id,
              file_name = excluded.file_name,
              file_size = excluded.file_size,
              modified_at = excluded.modified_at,
              missing = 0,
              unsupported = excluded.unsupported,
              partial_hash = excluded.partial_hash
            "#,
            params![
                photo_id,
                LOCAL_LIBRARY_ID,
                folder_id,
                candidate.file_name,
                candidate.path,
                candidate.file_size,
                candidate.modified_at,
                bool_to_sql(candidate.unsupported),
                candidate.partial_hash,
            ],
        )?;

        transaction.execute(
            r#"
            INSERT INTO photo_flags(photo_id)
            VALUES (?1)
            ON CONFLICT(photo_id) DO NOTHING
            "#,
            params![photo_id],
        )?;
    }

    transaction.commit()?;
    Ok(())
}

fn photo_flags_from_row(
    photo_id: String,
    rating: i64,
    picked: i64,
    rejected: i64,
    color_label: Option<String>,
) -> Result<PhotoFlags, LibraryStorageError> {
    let rating = u8::try_from(rating).unwrap_or(u8::MAX);
    PhotoFlags::new(
        photo_id,
        rating,
        sql_to_bool(picked),
        sql_to_bool(rejected),
        color_label,
    )
    .map_err(LibraryStorageError::from)
}

fn export_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportRecord> {
    Ok(ExportRecord {
        id: row.get(0)?,
        photo_id: row.get(1)?,
        output_path: row.get(2)?,
        export_settings_json: row.get(3)?,
    })
}

fn path_to_string(path: &Path) -> Result<String, LibraryStorageError> {
    path.to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LibraryStorageError::InvalidPath(path.to_path_buf()))
}

fn modified_at_string(metadata: &fs::Metadata) -> Option<String> {
    metadata
        .modified()
        .ok()
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| format!("unix:{}", duration.as_secs()))
}

fn current_timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("unix:{}", duration.as_secs()))
        .unwrap_or_else(|_| "unix:0".to_string())
}

fn partial_file_hash(path: &Path) -> Result<String, LibraryStorageError> {
    const MAX_HASH_BYTES: usize = 64 * 1024;

    let mut file = File::open(path)?;
    let mut buffer = [0_u8; 8192];
    let mut remaining = MAX_HASH_BYTES;
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;

    while remaining > 0 {
        let read_limit = remaining.min(buffer.len());
        let read = file.read(&mut buffer[..read_limit])?;
        if read == 0 {
            break;
        }

        for byte in &buffer[..read] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        remaining -= read;
    }

    Ok(format!("{hash:016x}"))
}

fn stable_catalog_id(prefix: &str, value: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{prefix}-{hash:016x}")
}

fn bool_to_sql(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

fn sql_to_bool(value: i64) -> bool {
    value != 0
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

    #[test]
    fn scans_mixed_folder_and_records_photo_candidates_by_reference() {
        let workspace = unique_library_root("import");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");
        let supported_before = std::fs::read(&supported_file).expect("read supported before");
        let unsupported_before = std::fs::read(&unsupported_file).expect("read unsupported before");

        let library = create_local_library(&library_root).expect("create library");
        let summary = import_folder(&library.root_path, &import_root).expect("import folder");

        assert_eq!(summary.scanned_files, 2);
        assert_eq!(summary.supported_files, 1);
        assert_eq!(summary.unsupported_files, 1);
        assert_eq!(summary.candidates.len(), 2);
        assert!(summary
            .candidates
            .iter()
            .any(|candidate| candidate.file_name == "sample.DNG" && !candidate.unsupported));
        assert!(summary
            .candidates
            .iter()
            .any(|candidate| candidate.file_name == "notes.txt" && candidate.unsupported));

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let imported_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))
            .expect("count photos");
        assert_eq!(imported_count, 2);

        let (path, file_size, unsupported, partial_hash): (String, i64, i64, String) = connection
            .query_row(
                "SELECT path, file_size, unsupported, partial_hash FROM photos WHERE file_name = 'sample.DNG'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("supported row");
        assert_eq!(path, supported_file.display().to_string());
        assert_eq!(file_size, supported_before.len() as i64);
        assert_eq!(unsupported, 0);
        assert!(!partial_hash.is_empty());

        let unsupported: i64 = connection
            .query_row(
                "SELECT unsupported FROM photos WHERE file_name = 'notes.txt'",
                [],
                |row| row.get(0),
            )
            .expect("unsupported row");
        assert_eq!(unsupported, 1);

        assert_eq!(
            std::fs::read(&supported_file).expect("read supported after"),
            supported_before
        );
        assert_eq!(
            std::fs::read(&unsupported_file).expect("read unsupported after"),
            unsupported_before
        );
        assert!(!library_root.join("sample.DNG").exists());
        assert!(!library_root.join("notes.txt").exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn persists_photo_flags_across_library_reopen() {
        let workspace = unique_library_root("flags");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.DNG'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");

        let initial_flags = get_photo_flags(&library.root_path, &photo_id)
            .expect("read initial flags")
            .expect("default flags row");
        assert_eq!(
            initial_flags,
            silica_catalog::PhotoFlags::new(photo_id.clone(), 0, false, false, None).unwrap()
        );

        let updated_flags = set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            4,
            true,
            false,
            Some(" green ".to_string()),
        )
        .expect("set flags");
        assert_eq!(
            updated_flags,
            silica_catalog::PhotoFlags::new(
                photo_id.clone(),
                4,
                true,
                false,
                Some("green".to_string())
            )
            .unwrap()
        );

        let (rating, picked, rejected, color_label): (i64, i64, i64, String) = connection
            .query_row(
                "SELECT rating, picked, rejected, color_label FROM photo_flags WHERE photo_id = ?1",
                params![photo_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("authoritative photo_flags row");
        assert_eq!(
            (rating, picked, rejected, color_label.as_str()),
            (4, 1, 0, "green")
        );

        drop(connection);
        let reopened = open_local_library(&library_root).expect("reopen library");
        let persisted_flags = get_photo_flags(&reopened.root_path, &updated_flags.photo_id)
            .expect("read persisted flags")
            .expect("persisted flags row");
        assert_eq!(persisted_flags, updated_flags);

        remove_library_root(&workspace);
    }

    #[test]
    fn reads_photo_preview_candidates_from_catalog() {
        let workspace = unique_library_root("preview-candidate");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let supported_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("supported photo id");
        let unsupported_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'notes.txt'",
                [],
                |row| row.get(0),
            )
            .expect("unsupported photo id");

        let supported = get_photo_preview_candidate(&library.root_path, &supported_id)
            .expect("read supported preview candidate")
            .expect("supported preview candidate");
        assert_eq!(supported.photo_id, supported_id);
        assert_eq!(supported.file_name, "sample.jpg");
        assert_eq!(supported.path, supported_file.display().to_string());
        assert!(!supported.unsupported);

        let unsupported = get_photo_preview_candidate(&library.root_path, &unsupported_id)
            .expect("read unsupported preview candidate")
            .expect("unsupported preview candidate");
        assert_eq!(unsupported.file_name, "notes.txt");
        assert!(unsupported.unsupported);

        assert!(
            get_photo_preview_candidate(&library.root_path, "missing-photo")
                .expect("missing preview candidate lookup")
                .is_none()
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn commits_active_edit_graph_without_draft_write() {
        let workspace = unique_library_root("edit-state");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        assert_eq!(count_edit_states(&connection), 0);
        drop(connection);

        let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
            .expect("load draft edit graph")
            .expect("draft edit graph");
        let edited =
            silica_edit::apply_exposure_contrast(&draft, 0.5, -8.0, "unix:3").expect("apply edit");

        let connection = open_catalog(&library.catalog_path).expect("reopen catalog");
        assert_eq!(
            count_edit_states(&connection),
            0,
            "draft slider update must not write edit_states"
        );
        drop(connection);

        commit_edit_graph(&library.root_path, edited).expect("commit edit graph");

        let reopened = open_local_library(&library_root).expect("reopen library");
        let persisted = load_active_edit_graph_or_default(&reopened.root_path, &photo_id)
            .expect("read active edit graph")
            .expect("active edit graph");
        assert_eq!(persisted.basic.exposure.as_f64(), Some(0.5));
        assert_eq!(persisted.basic.contrast.as_f64(), Some(-8.0));

        let connection = open_catalog(&reopened.catalog_path).expect("open reopened catalog");
        assert_eq!(count_edit_states(&connection), 1);

        remove_library_root(&workspace);
    }

    #[test]
    fn records_export_and_marks_photo_exported() {
        let workspace = unique_library_root("export-record");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        let output_path = workspace.join("Exports").join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(output_path.parent().expect("output parent"))
            .expect("create export directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);

        let record = record_export(
            &library.root_path,
            &photo_id,
            &output_path,
            r#"{"format":"jpeg","color_profile":"srgb"}"#,
        )
        .expect("record export");

        assert_eq!(record.photo_id, photo_id);
        assert_eq!(record.output_path, output_path.display().to_string());
        assert!(record.export_settings_json.contains("\"jpeg\""));

        let connection = open_catalog(&library.catalog_path).expect("reopen catalog");
        let export_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
            .expect("count exports");
        assert_eq!(export_count, 1);

        let exported: i64 = connection
            .query_row(
                "SELECT exported FROM photo_flags WHERE photo_id = ?1",
                params![record.photo_id],
                |row| row.get(0),
            )
            .expect("exported flag");
        assert_eq!(exported, 1);
        drop(connection);

        let latest = get_latest_export_record(&library.root_path, &record.photo_id)
            .expect("read latest export")
            .expect("latest export row");
        assert_eq!(latest, record);

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

    fn count_edit_states(connection: &Connection) -> i64 {
        connection
            .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
            .expect("count edit states")
    }
}
