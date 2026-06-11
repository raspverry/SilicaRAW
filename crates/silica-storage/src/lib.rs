//! Storage and persistence boundary for SilicaRAW.
//!
//! Spike 004 selects rusqlite with bundled SQLite and embedded SQL migrations.
//! This crate owns catalog schema creation, library-local sidecars, cache
//! records, and dry-run recovery reports. It does not decode photos, mutate
//! originals, write next-to-original sidecars, or apply restore actions yet.

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

use rusqlite::{named_params, params, Connection, OpenFlags, OptionalExtension};
use silica_catalog::{
    is_supported_photo_extension, CatalogFlagError, ImportCandidate,
    ALPHA_CATALOG_REQUIRED_INDEXES, ALPHA_CATALOG_REQUIRED_TABLES, ALPHA_CATALOG_SCHEMA_VERSION,
    ALPHA_MAX_RATING,
};
pub use silica_catalog::{ImportIssue, ImportIssueKind};
pub use silica_catalog::{
    LibraryQueryFileType, LibraryQueryFilters, LibraryQueryMetadataFilter, LibraryQueryOrderField,
    LibraryQueryPage, LibraryQueryRequest, LibraryQuerySort, PhotoFlags,
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

/// Library-local directory for recovery backup artifacts.
pub const BACKUPS_DIRECTORY: &str = "backups";

/// Stable backup manifest schema marker for Task 10.5 artifacts.
pub const BACKUP_SCHEMA: &str = "silica.backup";

/// Stable backup manifest version for Task 10.5 artifacts.
pub const BACKUP_VERSION: i64 = 1;

/// Manifest filename inside each backup artifact directory.
pub const BACKUP_MANIFEST_FILE: &str = "backup-manifest.json";

/// Library-local directory for portable sidecar JSON files.
pub const SIDECAR_DIRECTORY: &str = "sidecars";

const SIDECAR_FILE_SUFFIX: &str = ".silicaraw.sidecar.json";

/// Stable sidecar schema marker required by `schemas/sidecar.schema.json`.
pub const SIDECAR_SCHEMA: &str = "silica.sidecar";

/// Stable sidecar schema version for v0.1.
pub const SIDECAR_VERSION: i64 = 1;

/// Stable local alpha library row id for single-library catalog databases.
pub const LOCAL_LIBRARY_ID: &str = "local";

/// Disposable cache directories that product cache clearing may delete.
pub const DISPOSABLE_CACHE_DIRECTORIES: &[&str] =
    &["thumbnails", "previews", "render-cache", "ai-cache"];

/// Required support directories inside a SilicaRAW library folder.
pub const REQUIRED_LIBRARY_DIRECTORIES: &[&str] = &[
    "sidecars",
    "thumbnails",
    "previews",
    "render-cache",
    "ai-cache",
    "exports",
    "logs",
    BACKUPS_DIRECTORY,
];

/// Cache record type used for disposable Library grid thumbnails.
pub const THUMBNAIL_CACHE_TYPE: &str = "thumbnail";
/// Cache record type used for disposable Loupe preview images.
pub const PREVIEW_CACHE_TYPE: &str = "preview";

/// Task 11.7.2 policy: no metadata scan runs during library open or session restore.
pub const METADATA_BACKFILL_ON_OPEN_OR_RESTORE: bool = false;
/// Existing imports remain unknown until import-time extraction or explicit scoped backfill.
pub const EXISTING_IMPORTS_WITHOUT_METADATA_STAY_UNKNOWN: bool = true;

/// Source allowed for dimension metadata in the current metadata policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataDimensionSource {
    ExistingRasterPath,
    Unavailable,
}

/// File-level metadata extraction policy selected before the migration task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetadataExtractionPolicy {
    pub dimension_source: MetadataDimensionSource,
    pub raw_decode_supported: bool,
    pub camera_lens_available: bool,
}

/// Normalized metadata values to persist for one imported photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoMetadataUpdate {
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub orientation: Option<String>,
    pub capture_time: Option<String>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub raw_json: String,
}

impl PhotoMetadataUpdate {
    pub fn unavailable() -> Self {
        Self {
            width: None,
            height: None,
            orientation: None,
            capture_time: None,
            camera_make: None,
            camera_model: None,
            lens_model: None,
            raw_json: "{}".to_string(),
        }
    }
}

/// State of one metadata field in a read API response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoMetadataFieldState {
    Known,
    Unknown,
    Unavailable,
}

/// One typed metadata field plus its truth state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoMetadataField<T> {
    pub state: PhotoMetadataFieldState,
    pub value: Option<T>,
}

impl<T> PhotoMetadataField<T> {
    pub fn known(value: T) -> Self {
        Self {
            state: PhotoMetadataFieldState::Known,
            value: Some(value),
        }
    }

    pub fn unknown() -> Self {
        Self {
            state: PhotoMetadataFieldState::Unknown,
            value: None,
        }
    }

    pub fn unavailable() -> Self {
        Self {
            state: PhotoMetadataFieldState::Unavailable,
            value: None,
        }
    }
}

/// Stored metadata and file facts for one catalog photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoMetadata {
    pub photo_id: String,
    pub file_name: String,
    pub source_path: String,
    pub file_type: String,
    pub unsupported: bool,
    pub file_size: PhotoMetadataField<i64>,
    pub modified_at: PhotoMetadataField<String>,
    pub width: PhotoMetadataField<i64>,
    pub height: PhotoMetadataField<i64>,
    pub orientation: PhotoMetadataField<String>,
    pub capture_time: PhotoMetadataField<String>,
    pub camera_make: PhotoMetadataField<String>,
    pub camera_model: PhotoMetadataField<String>,
    pub lens_model: PhotoMetadataField<String>,
}

/// Return the metadata extraction policy for one original path.
pub fn metadata_extraction_policy_for_path(path: &Path) -> MetadataExtractionPolicy {
    let is_jpeg = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg")
        });

    MetadataExtractionPolicy {
        dimension_source: if is_jpeg {
            MetadataDimensionSource::ExistingRasterPath
        } else {
            MetadataDimensionSource::Unavailable
        },
        raw_decode_supported: false,
        camera_lens_available: false,
    }
}

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
    pub issues: Vec<ImportIssue>,
}

/// Catalog row data needed to open a preview for one photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoPreviewCandidate {
    pub photo_id: String,
    pub file_name: String,
    pub path: String,
    pub unsupported: bool,
}

/// Catalog row data needed by the Library grid MVP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryPhotoGridItem {
    pub photo_id: String,
    pub file_name: String,
    pub path: String,
    pub file_type: String,
    pub thumbnail_path: Option<String>,
    pub thumbnail_cache_key: Option<String>,
    pub missing: bool,
    pub unsupported: bool,
    pub rating: u8,
    pub picked: bool,
    pub rejected: bool,
    pub color_label: Option<String>,
}

/// Disposable cache row recorded in the local catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheRecord {
    pub id: String,
    pub photo_id: Option<String>,
    pub cache_type: String,
    pub cache_key: String,
    pub path: String,
    pub byte_size: i64,
}

/// Result of clearing disposable local alpha cache data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheClearSummary {
    pub cleared_directories: Vec<String>,
    pub recreated_directories: Vec<String>,
    pub removed_cache_records: usize,
    pub message: String,
}

/// Result of creating a checkpointed local library backup artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryBackupResult {
    pub backup_id: String,
    pub backup_path: PathBuf,
    pub manifest_path: PathBuf,
    pub created_at: String,
    pub files: Vec<String>,
    pub bytes_copied: u64,
}

/// Result of restoring a local library backup artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryRestoreResult {
    pub restored_library: LocalLibrary,
    pub backup_path: PathBuf,
    pub rollback_path: Option<PathBuf>,
    pub restored_files: Vec<String>,
}

/// Catalog export row written after an export completes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRecord {
    pub id: String,
    pub photo_id: String,
    pub output_path: String,
    pub export_settings_json: String,
}

/// Result returned after a sidecar is written successfully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SidecarWriteResult {
    pub photo_id: String,
    pub sidecar_path: PathBuf,
    pub sidecar_relative_path: String,
    pub written_at: String,
    pub bytes_written: u64,
}

/// Sidecar payload that has passed the v1 sidecar and nested edit graph checks.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedSidecar {
    pub photo_id: String,
    pub sidecar_path: PathBuf,
    pub written_at: String,
    pub flags: PhotoFlags,
    pub edit_graph: silica_edit::EditGraph,
    pub json: serde_json::Value,
}

/// Deterministic report for a catalog rebuild preview from library-local sidecars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRebuildDryRunReport {
    pub sidecars_scanned: usize,
    pub entries: Vec<CatalogRebuildDryRunEntry>,
    pub issues: Vec<CatalogRebuildDryRunIssue>,
}

/// Per-sidecar dry-run result for rebuildable photo flag state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRebuildDryRunEntry {
    pub photo_id: String,
    pub sidecar_relative_path: String,
    pub action: CatalogRebuildDryRunAction,
    pub flag_source: CatalogRebuildFlagSource,
    pub resolved_flags: PhotoFlags,
    pub catalog_flags: Option<PhotoFlags>,
}

/// Rebuild action that would be taken if a later restore task applies the dry-run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogRebuildDryRunAction {
    CreatePhotoFlags,
    UpdatePhotoFlags,
    KeepPhotoFlags,
}

/// Source used to resolve portable culling and label flags for rebuild preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogRebuildFlagSource {
    SidecarFlags,
    EditGraphMetadata,
    Defaults,
}

/// Structured dry-run issue kind for sidecar rebuild reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogRebuildDryRunIssueKind {
    MalformedJson,
    SchemaInvalid,
    InvalidPathIdentity,
    PhotoIdMismatch,
    FlagsMetadataConflict,
    CatalogReconcileConflict,
}

/// Non-fatal issue found while previewing catalog rebuild from sidecars.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogRebuildDryRunIssue {
    pub kind: CatalogRebuildDryRunIssueKind,
    pub photo_id: Option<String>,
    pub sidecar_relative_path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SidecarPhotoRow {
    photo_id: String,
    original_path: String,
    file_name: String,
    file_size: i64,
    modified_at: Option<String>,
    partial_hash: Option<String>,
    full_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SidecarPhotoSnapshot {
    original_path: String,
    file_name: String,
    file_size: Option<i64>,
    modified_at: Option<String>,
    partial_hash: Option<String>,
    full_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackupManifest {
    catalog_schema_version: i64,
    files: Vec<String>,
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
    MissingPhoto(String),
    CatalogSchemaVersion { expected: i64, found: i64 },
    NotDirectory(PathBuf),
    InvalidPath(PathBuf),
    InvalidSidecarPhotoId(String),
    SidecarValidation(String),
    BackupValidation(String),
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
            Self::MissingPhoto(photo_id) => write!(formatter, "missing catalog photo: {photo_id}"),
            Self::CatalogSchemaVersion { expected, found } => write!(
                formatter,
                "catalog schema version mismatch: expected {expected}, found {found}"
            ),
            Self::NotDirectory(path) => write!(formatter, "not a directory: {}", path.display()),
            Self::InvalidPath(path) => {
                write!(formatter, "path is not valid UTF-8: {}", path.display())
            }
            Self::InvalidSidecarPhotoId(photo_id) => {
                write!(formatter, "invalid sidecar photo id: {photo_id:?}")
            }
            Self::SidecarValidation(message) => {
                write!(formatter, "sidecar validation error: {message}")
            }
            Self::BackupValidation(message) => {
                write!(formatter, "backup validation error: {message}")
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
            Self::MissingCatalog(_)
            | Self::MissingPhoto(_)
            | Self::CatalogSchemaVersion { .. }
            | Self::NotDirectory(_)
            | Self::InvalidPath(_)
            | Self::InvalidSidecarPhotoId(_)
            | Self::SidecarValidation(_)
            | Self::BackupValidation(_) => None,
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
    Migration {
        version: 3,
        name: "paged_library_query_indexes",
        sql: PAGED_LIBRARY_QUERY_INDEXES_SQL,
    },
    Migration {
        version: 4,
        name: "photo_metadata_normalized_fields",
        sql: PHOTO_METADATA_NORMALIZED_FIELDS_SQL,
    },
    Migration {
        version: 5,
        name: "photo_metadata_query_indexes",
        sql: PHOTO_METADATA_QUERY_INDEXES_SQL,
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

/// Inspect an existing local library for relaunch restore without migrations or repair.
pub fn inspect_local_library_for_restore(
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

    let connection = Connection::open_with_flags(&catalog_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let schema_version = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    Ok(LocalLibrary {
        root_path: root_path.to_path_buf(),
        catalog_path,
        schema_version,
    })
}

/// Check catalog photo existence for relaunch restore without migrations or repair.
pub fn catalog_photo_exists_for_restore(
    root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<bool, LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    let connection = Connection::open_with_flags(&catalog_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    connection
        .query_row(
            "SELECT 1 FROM photos WHERE id = ?1 LIMIT 1",
            [photo_id],
            |_| Ok(()),
        )
        .optional()
        .map(|result| result.is_some())
        .map_err(LibraryStorageError::from)
}

/// Resolve the library-local sidecar path for a catalog photo id.
pub fn sidecar_path_for_photo(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<PathBuf, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    Ok(library_root_path
        .as_ref()
        .join(SIDECAR_DIRECTORY)
        .join(format!("{photo_id}{SIDECAR_FILE_SUFFIX}")))
}

/// Write a validated sidecar into the library-local sidecars directory.
pub fn write_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<SidecarWriteResult, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_local_library(library_root_path)?;
    let sidecar_path = sidecar_path_for_photo(&library.root_path, photo_id)?;
    let sidecar_relative_path = format!("{SIDECAR_DIRECTORY}/{photo_id}{SIDECAR_FILE_SUFFIX}");
    fs::create_dir_all(library.root_path.join(SIDECAR_DIRECTORY))?;

    let value = build_photo_sidecar_value(&library.root_path, photo_id, app_version)?;
    validate_sidecar_json(&value)?;
    let bytes = serde_json::to_vec_pretty(&value)?;
    let temp_path = sidecar_path.with_extension("json.tmp");
    fs::write(&temp_path, &bytes)?;
    let temp_value: serde_json::Value = serde_json::from_slice(&fs::read(&temp_path)?)?;
    validate_sidecar_json(&temp_value)?;
    fs::rename(&temp_path, &sidecar_path)?;

    let written_at = value
        .get("written_at")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    update_sidecar_status(
        &library.catalog_path,
        photo_id,
        &sidecar_relative_path,
        &written_at,
    )?;

    Ok(SidecarWriteResult {
        photo_id: photo_id.to_string(),
        sidecar_path,
        sidecar_relative_path,
        written_at,
        bytes_written: bytes.len() as u64,
    })
}

/// Read and validate a library-local sidecar without mutating catalog state.
pub fn read_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<ValidatedSidecar>, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let sidecar_path = sidecar_path_for_photo(&library.root_path, photo_id)?;
    if !sidecar_path.is_file() {
        return Ok(None);
    }

    let json: serde_json::Value = serde_json::from_slice(&fs::read(&sidecar_path)?)?;
    validate_sidecar_json(&json)?;
    let sidecar_photo_id = json["photo"]["photo_id"].as_str().ok_or_else(|| {
        LibraryStorageError::SidecarValidation(
            "sidecar.photo.photo_id must be a string".to_string(),
        )
    })?;
    if sidecar_photo_id != photo_id {
        return Err(LibraryStorageError::SidecarValidation(format!(
            "sidecar photo id mismatch: expected {photo_id}, found {sidecar_photo_id}"
        )));
    }

    let flags = parse_sidecar_flags(&json)?;
    let edit_graph: silica_edit::EditGraph = serde_json::from_value(json["edit_graph"].clone())?;
    silica_edit::validate_edit_graph(&edit_graph)?;
    let written_at = json["written_at"].as_str().unwrap_or_default().to_string();

    Ok(Some(ValidatedSidecar {
        photo_id: photo_id.to_string(),
        sidecar_path,
        written_at,
        flags,
        edit_graph,
        json,
    }))
}

/// Preview how the live catalog would rebuild portable flag state from sidecars.
pub fn dry_run_catalog_rebuild_from_sidecars(
    library_root_path: impl AsRef<Path>,
) -> Result<CatalogRebuildDryRunReport, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let sidecars_directory = library.root_path.join(SIDECAR_DIRECTORY);
    let mut report = CatalogRebuildDryRunReport {
        sidecars_scanned: 0,
        entries: Vec::new(),
        issues: Vec::new(),
    };

    if !sidecars_directory.is_dir() {
        return Ok(report);
    }

    let mut sidecar_paths = Vec::new();
    for entry in fs::read_dir(&sidecars_directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if file_name.ends_with(SIDECAR_FILE_SUFFIX) {
            sidecar_paths.push(entry.path());
        }
    }
    sidecar_paths.sort_by(|left, right| {
        left.file_name()
            .cmp(&right.file_name())
            .then_with(|| left.cmp(right))
    });

    let connection = open_catalog(&library.catalog_path)?;
    for sidecar_path in sidecar_paths {
        process_rebuild_dry_run_sidecar(&connection, &sidecar_path, &mut report)?;
    }

    Ok(report)
}

/// Create a checkpointed backup artifact with catalog data, sidecars, and a manifest.
pub fn create_library_backup(
    library_root_path: impl AsRef<Path>,
    app_version: &str,
) -> Result<LibraryBackupResult, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    checkpoint_catalog_for_backup(&library.catalog_path)?;

    let backup_id = current_backup_id();
    let backups_root = library.root_path.join(BACKUPS_DIRECTORY);
    fs::create_dir_all(&backups_root)?;
    let backup_path = backups_root.join(&backup_id);
    if backup_path.exists() {
        return Err(LibraryStorageError::BackupValidation(format!(
            "backup artifact already exists: {}",
            backup_path.display()
        )));
    }
    fs::create_dir(&backup_path)?;

    let mut files = Vec::new();
    let mut bytes_copied = 0_u64;
    bytes_copied += copy_backup_file(
        &library.catalog_path,
        &backup_path.join(CATALOG_DATABASE_FILE),
        CATALOG_DATABASE_FILE,
        &mut files,
    )?;
    bytes_copied += copy_backup_directory(
        &library.root_path.join(SIDECAR_DIRECTORY),
        &backup_path.join(SIDECAR_DIRECTORY),
        SIDECAR_DIRECTORY,
        &mut files,
    )?;
    files.sort();

    let created_at = current_timestamp_string();
    let manifest_files = files.clone();
    let manifest = serde_json::json!({
        "schema": BACKUP_SCHEMA,
        "version": BACKUP_VERSION,
        "app_version": app_version,
        "catalog_schema_version": library.schema_version,
        "created_at": created_at,
        "checkpoint": "wal_checkpoint_truncate",
        "files": manifest_files,
        "excluded": [
            "original referenced photo files",
            "thumbnails/",
            "previews/",
            "render-cache/",
            "ai-cache/",
            "exports/",
            "logs/",
            "backups/"
        ]
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let manifest_path = backup_path.join(BACKUP_MANIFEST_FILE);
    fs::write(&manifest_path, &manifest_bytes)?;
    bytes_copied += manifest_bytes.len() as u64;

    Ok(LibraryBackupResult {
        backup_id,
        backup_path,
        manifest_path,
        created_at,
        files,
        bytes_copied,
    })
}

/// Restore a backup artifact into an empty or rollback-protected library root.
pub fn restore_library_backup(
    backup_path: impl AsRef<Path>,
    target_root_path: impl AsRef<Path>,
) -> Result<LibraryRestoreResult, LibraryStorageError> {
    let backup_path = backup_path.as_ref();
    let target_root_path = target_root_path.as_ref();
    let manifest = read_backup_manifest(backup_path)?;
    let staging_root = restore_staging_path(target_root_path);
    if staging_root.exists() {
        return Err(LibraryStorageError::BackupValidation(format!(
            "restore staging path already exists: {}",
            staging_root.display()
        )));
    }

    copy_backup_payload_to_root(backup_path, &staging_root)?;
    let staging_library = match open_local_library(&staging_root) {
        Ok(library) => library,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_root);
            return Err(error);
        }
    };
    drop(staging_library);

    let rollback_path = if target_has_library_state(target_root_path)? {
        Some(create_restore_rollback(target_root_path)?)
    } else {
        None
    };

    let restored_library = apply_staged_restore(&staging_root, target_root_path)?;

    Ok(LibraryRestoreResult {
        restored_library,
        backup_path: backup_path.to_path_buf(),
        rollback_path,
        restored_files: manifest.files,
    })
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

    let (mut candidates, mut issues) = scan_import_candidates(folder_path)?;
    candidates.sort_by(|left, right| left.path.cmp(&right.path));
    issues.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.kind.as_str().cmp(right.kind.as_str()))
    });

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
        issues,
    })
}

/// Insert or update normalized metadata for an imported photo by original path.
pub fn upsert_photo_metadata_by_path(
    library_root_path: impl AsRef<Path>,
    original_path: impl AsRef<Path>,
    metadata: PhotoMetadataUpdate,
) -> Result<(), LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let original_path = path_to_string(original_path.as_ref())?;
    let connection = open_catalog(&library.catalog_path)?;
    connection.execute(
        r#"
        INSERT INTO photo_metadata(
          photo_id,
          camera_make,
          camera_model,
          lens_model,
          capture_time,
          raw_json,
          width,
          height,
          orientation
        )
        SELECT
          photos.id,
          ?2,
          ?3,
          ?4,
          ?5,
          ?6,
          ?7,
          ?8,
          ?9
        FROM photos
        WHERE photos.library_id = ?1
          AND photos.path = ?10
          AND photos.unsupported = 0
        ON CONFLICT(photo_id) DO UPDATE SET
          camera_make = excluded.camera_make,
          camera_model = excluded.camera_model,
          lens_model = excluded.lens_model,
          capture_time = excluded.capture_time,
          raw_json = excluded.raw_json,
          width = excluded.width,
          height = excluded.height,
          orientation = excluded.orientation
        "#,
        params![
            LOCAL_LIBRARY_ID,
            metadata.camera_make,
            metadata.camera_model,
            metadata.lens_model,
            metadata.capture_time,
            metadata.raw_json,
            metadata.width,
            metadata.height,
            metadata.orientation,
            original_path,
        ],
    )?;

    Ok(())
}

/// Read stored metadata for one photo without touching original files.
pub fn get_photo_metadata(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoMetadata>, LibraryStorageError> {
    let (_library, connection) = open_existing_library_for_read_only_query(library_root_path)?;
    connection
        .query_row(
            r#"
            SELECT
              photos.id,
              photos.file_name,
              photos.path,
              photos.file_type,
              photos.unsupported,
              photos.file_size,
              photos.modified_at,
              photo_metadata.photo_id IS NOT NULL,
              photo_metadata.width,
              photo_metadata.height,
              photo_metadata.orientation,
              photo_metadata.capture_time,
              photo_metadata.camera_make,
              photo_metadata.camera_model,
              photo_metadata.lens_model
            FROM photos
            LEFT JOIN photo_metadata ON photo_metadata.photo_id = photos.id
            WHERE photos.library_id = ?1
              AND photos.id = ?2
            "#,
            params![LOCAL_LIBRARY_ID, photo_id],
            photo_metadata_from_row,
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

/// List imported catalog photos for the Library grid without touching originals.
pub fn list_library_photos(
    library_root_path: impl AsRef<Path>,
) -> Result<Vec<LibraryPhotoGridItem>, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let mut statement = connection.prepare(
        r#"
        SELECT
          photos.id,
          photos.file_name,
          photos.path,
          photos.missing,
          photos.unsupported,
          photo_flags.rating,
          photo_flags.picked,
          photo_flags.rejected,
          photo_flags.color_label,
          thumbnail_cache.path,
          thumbnail_cache.cache_key
        FROM photos
        LEFT JOIN photo_flags ON photo_flags.photo_id = photos.id
        LEFT JOIN cache_records AS thumbnail_cache
          ON thumbnail_cache.photo_id = photos.id
          AND thumbnail_cache.cache_type = ?1
        ORDER BY photos.file_name ASC, photos.path ASC
        "#,
    )?;
    let rows = statement.query_map([THUMBNAIL_CACHE_TYPE], library_photo_grid_item_from_row)?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(LibraryStorageError::from)
}

/// Query imported catalog photos by bounded page without mutating catalog state.
pub fn query_library_photos(
    library_root_path: impl AsRef<Path>,
    request: LibraryQueryRequest,
) -> Result<LibraryQueryPage<LibraryPhotoGridItem>, LibraryStorageError> {
    let request = LibraryQueryRequest::new(
        request.offset,
        request.limit,
        request.sort,
        request.filters.clone(),
    );
    let (_library, connection) = open_existing_library_for_read_only_query(library_root_path)?;
    let filter = LibraryQuerySqlFilter::from(&request.filters);
    let order_clause = library_query_order_clause(request.sort);

    let total_count = query_library_photo_count(&connection, &filter)?;
    let limit = i64::from(request.limit);
    let offset = i64::try_from(request.offset).unwrap_or(i64::MAX);
    let query_sql =
        format!("{LIBRARY_QUERY_SELECT_SQL}\n{order_clause}\nLIMIT :limit OFFSET :offset");
    let mut statement = connection.prepare(&query_sql)?;
    let rows = statement.query_map(
        named_params! {
            ":thumbnail_cache_type": THUMBNAIL_CACHE_TYPE,
            ":library_id": LOCAL_LIBRARY_ID,
            ":min_rating": filter.min_rating,
            ":picked": filter.picked,
            ":rejected": filter.rejected,
            ":file_type": filter.file_type,
            ":metadata_filter": filter.metadata,
            ":search": filter.search.as_deref(),
            ":limit": limit,
            ":offset": offset,
        },
        library_photo_grid_item_from_row,
    )?;
    let items = rows.collect::<Result<Vec<_>, _>>()?;
    let has_next_page = request.offset.saturating_add(u64::from(request.limit)) < total_count;

    Ok(LibraryQueryPage {
        items,
        offset: request.offset,
        limit: request.limit,
        total_count,
        has_next_page,
        order_fields: request.order_fields(),
    })
}

/// Record a disposable JPEG thumbnail cache file for a catalog photo.
pub fn record_thumbnail_cache(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    record_photo_cache(
        library_root_path,
        "cache-thumbnail",
        photo_id,
        THUMBNAIL_CACHE_TYPE,
        cache_key,
        path,
        byte_size,
    )
}

/// Record a disposable JPEG Loupe preview cache file for a catalog photo.
pub fn record_preview_cache(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    record_photo_cache(
        library_root_path,
        "cache-preview",
        photo_id,
        PREVIEW_CACHE_TYPE,
        cache_key,
        path,
        byte_size,
    )
}

/// Read a disposable cache record for one catalog photo and cache type.
pub fn get_photo_cache_record(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_type: &str,
) -> Result<Option<CacheRecord>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    connection
        .query_row(
            r#"
            SELECT id, photo_id, cache_type, cache_key, path, byte_size
            FROM cache_records
            WHERE photo_id = ?1 AND cache_type = ?2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            params![photo_id, cache_type],
            |row| {
                Ok(CacheRecord {
                    id: row.get(0)?,
                    photo_id: row.get(1)?,
                    cache_type: row.get(2)?,
                    cache_key: row.get(3)?,
                    path: row.get(4)?,
                    byte_size: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

/// Clear only disposable cache directories and cache record metadata.
pub fn clear_disposable_cache(
    library_root_path: impl AsRef<Path>,
) -> Result<CacheClearSummary, LibraryStorageError> {
    let library = open_local_library(library_root_path)?;
    let mut cleared_directories = Vec::with_capacity(DISPOSABLE_CACHE_DIRECTORIES.len());
    let mut recreated_directories = Vec::with_capacity(DISPOSABLE_CACHE_DIRECTORIES.len());

    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        let path = library.root_path.join(directory);
        if path.exists() {
            fs::remove_dir_all(&path)?;
        }
        fs::create_dir_all(&path)?;
        cleared_directories.push((*directory).to_string());
        recreated_directories.push((*directory).to_string());
    }

    let connection = open_catalog(&library.catalog_path)?;
    let removed_cache_records = connection.execute("DELETE FROM cache_records", [])?;

    Ok(CacheClearSummary {
        cleared_directories,
        recreated_directories,
        removed_cache_records,
        message: "Cache clear removed only disposable library caches.".to_string(),
    })
}

fn record_photo_cache(
    library_root_path: impl AsRef<Path>,
    cache_id_namespace: &str,
    photo_id: &str,
    cache_type: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let cache_id = stable_catalog_id(cache_id_namespace, photo_id);
    let cache_key = cache_key.as_ref().to_string();
    let path = path_to_string(path.as_ref())?;
    let cache_type = cache_type.to_string();

    connection.execute(
        r#"
        INSERT INTO cache_records(
          id,
          photo_id,
          cache_type,
          cache_key,
          path,
          byte_size,
          created_at,
          last_accessed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
          cache_key = excluded.cache_key,
          path = excluded.path,
          byte_size = excluded.byte_size,
          created_at = CURRENT_TIMESTAMP,
          last_accessed_at = CURRENT_TIMESTAMP
        "#,
        params![cache_id, photo_id, cache_type, cache_key, path, byte_size,],
    )?;

    Ok(CacheRecord {
        id: cache_id,
        photo_id: Some(photo_id.to_string()),
        cache_type,
        cache_key,
        path,
        byte_size,
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

/// Load the active committed edit graph for a photo without creating a default draft.
pub fn load_active_edit_graph(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<silica_edit::EditGraph>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;

    connection
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
        .map(|json| {
            let graph: silica_edit::EditGraph = serde_json::from_str(&json)?;
            silica_edit::validate_edit_graph(&graph)?;
            Ok(graph)
        })
        .transpose()
}

/// Load the active edit graph for a photo, or build a default draft without writing it.
pub fn load_active_edit_graph_or_default(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<silica_edit::EditGraph>, LibraryStorageError> {
    let library_root_path = library_root_path.as_ref();
    if let Some(graph) = load_active_edit_graph(library_root_path, photo_id)? {
        return Ok(Some(graph));
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;

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

fn load_sidecar_photo_row(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<SidecarPhotoRow>, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT id, path, file_name, file_size, modified_at, partial_hash, full_hash
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(SidecarPhotoRow {
                    photo_id: row.get(0)?,
                    original_path: row.get(1)?,
                    file_name: row.get(2)?,
                    file_size: row.get(3)?,
                    modified_at: row.get(4)?,
                    partial_hash: row.get(5)?,
                    full_hash: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

fn active_edit_state_id(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<String>, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT id
            FROM edit_states
            WHERE photo_id = ?1 AND active = 1
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
            params![photo_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

fn edit_color_label_from_catalog(
    label: Option<&str>,
) -> Result<Option<silica_edit::ColorLabel>, LibraryStorageError> {
    match label {
        None => Ok(None),
        Some("red") => Ok(Some(silica_edit::ColorLabel::Red)),
        Some("orange") => Ok(Some(silica_edit::ColorLabel::Orange)),
        Some("yellow") => Ok(Some(silica_edit::ColorLabel::Yellow)),
        Some("green") => Ok(Some(silica_edit::ColorLabel::Green)),
        Some("blue") => Ok(Some(silica_edit::ColorLabel::Blue)),
        Some("purple") => Ok(Some(silica_edit::ColorLabel::Purple)),
        Some(other) => Err(LibraryStorageError::SidecarValidation(format!(
            "unsupported sidecar color label: {other}"
        ))),
    }
}

fn color_label_value(label: Option<&str>) -> serde_json::Value {
    match label {
        Some(label) => serde_json::Value::String(label.to_string()),
        None => serde_json::Value::Null,
    }
}

fn build_photo_sidecar_value(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<serde_json::Value, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let photo = load_sidecar_photo_row(&connection, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;
    let flags = get_photo_flags(&library.root_path, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;
    let mut graph = load_active_edit_graph_or_default(&library.root_path, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;

    graph.app_version = Some(app_version.to_string());
    graph.metadata.rating = i64::from(flags.rating);
    graph.metadata.picked = flags.picked;
    graph.metadata.rejected = flags.rejected;
    graph.metadata.color_label = edit_color_label_from_catalog(flags.color_label.as_deref())?;
    silica_edit::validate_edit_graph(&graph)?;
    let edit_graph_json = serde_json::to_value(&graph)?;
    let written_at = current_timestamp_string();
    let catalog_edit_state_id = active_edit_state_id(&connection, photo_id)?;

    let value = serde_json::json!({
        "schema": SIDECAR_SCHEMA,
        "version": SIDECAR_VERSION,
        "app_version": app_version,
        "photo": {
            "photo_id": photo.photo_id,
            "original_path": photo.original_path,
            "file_name": photo.file_name,
            "fingerprint": {
                "file_size": photo.file_size,
                "modified_at": photo.modified_at.unwrap_or_else(|| "unknown".to_string()),
                "partial_hash": photo.partial_hash.unwrap_or_default(),
                "full_hash": photo.full_hash
            }
        },
        "edit_graph": edit_graph_json,
        "flags": {
            "rating": flags.rating,
            "picked": flags.picked,
            "rejected": flags.rejected,
            "color_label": color_label_value(flags.color_label.as_deref())
        },
        "sync": {
            "status": "in_sync",
            "catalog_edit_state_id": catalog_edit_state_id,
            "sidecar_hash": serde_json::Value::Null
        },
        "written_at": written_at
    });

    validate_sidecar_json(&value)?;
    Ok(value)
}

fn validate_sidecar_json(value: &serde_json::Value) -> Result<(), LibraryStorageError> {
    let object = value.as_object().ok_or_else(|| {
        LibraryStorageError::SidecarValidation("sidecar root must be an object".to_string())
    })?;
    if object.get("schema").and_then(serde_json::Value::as_str) != Some(SIDECAR_SCHEMA) {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar schema marker must be silica.sidecar".to_string(),
        ));
    }
    if object.get("version").and_then(serde_json::Value::as_i64) != Some(SIDECAR_VERSION) {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar version must be 1".to_string(),
        ));
    }
    for required in [
        "app_version",
        "photo",
        "edit_graph",
        "flags",
        "sync",
        "written_at",
    ] {
        if !object.contains_key(required) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar missing required field: {required}"
            )));
        }
    }
    let allowed_top_level = [
        "schema",
        "version",
        "app_version",
        "photo",
        "edit_graph",
        "flags",
        "sync",
        "written_at",
    ];
    for key in object.keys() {
        if !allowed_top_level.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "unsupported top-level field: {key}"
            )));
        }
    }
    if !object
        .get("app_version")
        .is_some_and(serde_json::Value::is_string)
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.app_version must be a string".to_string(),
        ));
    }
    if !object
        .get("written_at")
        .is_some_and(serde_json::Value::is_string)
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.written_at must be a string".to_string(),
        ));
    }

    let photo = object
        .get("photo")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.photo must be an object".to_string())
        })?;
    let allowed_photo = ["photo_id", "original_path", "file_name", "fingerprint"];
    for key in photo.keys() {
        if !allowed_photo.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.photo contains unsupported field: {key}"
            )));
        }
    }
    for required in allowed_photo {
        if !photo.contains_key(required) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.photo missing required field: {required}"
            )));
        }
    }
    for key in ["photo_id", "original_path", "file_name"] {
        if !photo.get(key).is_some_and(serde_json::Value::is_string) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.photo.{key} must be a string"
            )));
        }
    }
    if !photo
        .get("fingerprint")
        .is_some_and(serde_json::Value::is_object)
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.photo.fingerprint must be an object".to_string(),
        ));
    }

    let flags = object
        .get("flags")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.flags must be an object".to_string())
        })?;
    let allowed_flags = ["rating", "picked", "rejected", "color_label"];
    for key in flags.keys() {
        if !allowed_flags.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.flags contains unsupported field: {key}"
            )));
        }
    }
    if flags
        .get("rating")
        .and_then(serde_json::Value::as_i64)
        .map_or(true, |rating| !(0..=5).contains(&rating))
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.flags.rating must be 0..=5".to_string(),
        ));
    }
    for key in ["picked", "rejected"] {
        if !flags.get(key).is_some_and(serde_json::Value::is_boolean) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.flags.{key} must be boolean"
            )));
        }
    }
    match flags.get("color_label") {
        Some(value) if value.is_null() => {}
        Some(value) => {
            let label = value.as_str().ok_or_else(|| {
                LibraryStorageError::SidecarValidation(
                    "sidecar.flags.color_label must be string or null".to_string(),
                )
            })?;
            edit_color_label_from_catalog(Some(label))?;
        }
        None => {
            return Err(LibraryStorageError::SidecarValidation(
                "sidecar.flags.color_label is required".to_string(),
            ));
        }
    }

    let edit_graph = object.get("edit_graph").ok_or_else(|| {
        LibraryStorageError::SidecarValidation("sidecar.edit_graph is required".to_string())
    })?;
    silica_edit::validate_edit_graph_json(edit_graph)?;

    let sync = object
        .get("sync")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.sync must be an object".to_string())
        })?;
    let allowed_sync = ["status", "catalog_edit_state_id", "sidecar_hash"];
    for key in sync.keys() {
        if !allowed_sync.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.sync contains unsupported field: {key}"
            )));
        }
    }
    let status = sync
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.sync.status must be a string".to_string(),
            )
        })?;
    if ![
        "in_sync",
        "catalog_newer",
        "sidecar_newer",
        "conflict",
        "missing",
        "disabled",
    ]
    .contains(&status)
    {
        return Err(LibraryStorageError::SidecarValidation(format!(
            "sidecar.sync.status is unsupported: {status}"
        )));
    }

    Ok(())
}

fn parse_sidecar_flags(value: &serde_json::Value) -> Result<PhotoFlags, LibraryStorageError> {
    let photo_id = value["photo"]["photo_id"].as_str().ok_or_else(|| {
        LibraryStorageError::SidecarValidation(
            "sidecar.photo.photo_id must be a string".to_string(),
        )
    })?;
    let flags = value
        .get("flags")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.flags must be an object".to_string())
        })?;
    let rating = flags
        .get("rating")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.flags.rating must be an integer".to_string(),
            )
        })?;
    let picked = flags
        .get("picked")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.flags.picked must be boolean".to_string(),
            )
        })?;
    let rejected = flags
        .get("rejected")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.flags.rejected must be boolean".to_string(),
            )
        })?;
    let color_label = match flags.get("color_label") {
        Some(value) if value.is_null() => None,
        Some(value) => Some(
            value
                .as_str()
                .ok_or_else(|| {
                    LibraryStorageError::SidecarValidation(
                        "sidecar.flags.color_label must be string or null".to_string(),
                    )
                })?
                .to_string(),
        ),
        None => None,
    };

    PhotoFlags::new(
        photo_id.to_string(),
        rating as u8,
        picked,
        rejected,
        color_label,
    )
    .map_err(LibraryStorageError::from)
}

fn process_rebuild_dry_run_sidecar(
    connection: &Connection,
    sidecar_path: &Path,
    report: &mut CatalogRebuildDryRunReport,
) -> Result<(), LibraryStorageError> {
    report.sidecars_scanned += 1;

    let file_name = sidecar_path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sidecar_relative_path = format!("{SIDECAR_DIRECTORY}/{file_name}");
    let expected_photo_id = file_name
        .strip_suffix(SIDECAR_FILE_SUFFIX)
        .unwrap_or_default()
        .to_string();

    if let Err(error) = validate_sidecar_photo_id(&expected_photo_id) {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::InvalidPathIdentity,
            Some(expected_photo_id),
            sidecar_relative_path,
            error.to_string(),
        );
        return Ok(());
    }

    let bytes = fs::read(sidecar_path)?;
    let json: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            push_rebuild_issue(
                report,
                CatalogRebuildDryRunIssueKind::MalformedJson,
                Some(expected_photo_id),
                sidecar_relative_path,
                error.to_string(),
            );
            return Ok(());
        }
    };

    if json.get("schema").and_then(serde_json::Value::as_str) != Some(SIDECAR_SCHEMA)
        || json.get("version").and_then(serde_json::Value::as_i64) != Some(SIDECAR_VERSION)
    {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::SchemaInvalid,
            Some(expected_photo_id),
            sidecar_relative_path,
            "sidecar schema marker or version is unsupported".to_string(),
        );
        return Ok(());
    }

    if let Err(error) = validate_sidecar_json(&json) {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::SchemaInvalid,
            Some(expected_photo_id.clone()),
            sidecar_relative_path.clone(),
            error.to_string(),
        );
    }

    let sidecar_photo_id = match json["photo"]["photo_id"].as_str() {
        Some(photo_id) => photo_id,
        None => {
            push_rebuild_issue(
                report,
                CatalogRebuildDryRunIssueKind::PhotoIdMismatch,
                Some(expected_photo_id),
                sidecar_relative_path,
                "sidecar.photo.photo_id is missing".to_string(),
            );
            return Ok(());
        }
    };

    if sidecar_photo_id != expected_photo_id {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::PhotoIdMismatch,
            Some(sidecar_photo_id.to_string()),
            sidecar_relative_path,
            format!(
                "sidecar path identity {expected_photo_id} does not match payload {sidecar_photo_id}"
            ),
        );
        return Ok(());
    }

    let sidecar_flags = parse_valid_rebuild_sidecar_flags(&json).ok();
    let metadata_flags = parse_valid_edit_graph_metadata_flags(&expected_photo_id, &json).ok();
    if let (Some(sidecar_flags), Some(metadata_flags)) = (&sidecar_flags, &metadata_flags) {
        if sidecar_flags != metadata_flags {
            push_rebuild_issue(
                report,
                CatalogRebuildDryRunIssueKind::FlagsMetadataConflict,
                Some(expected_photo_id.clone()),
                sidecar_relative_path.clone(),
                "sidecar.flags and edit_graph.metadata disagree; sidecar.flags would win"
                    .to_string(),
            );
        }
    }

    if let Some(snapshot) = parse_sidecar_photo_snapshot(&json) {
        report_catalog_reconcile_issues(
            connection,
            &expected_photo_id,
            &sidecar_relative_path,
            &snapshot,
            report,
        )?;
    }

    let (flag_source, resolved_flags) = match (sidecar_flags, metadata_flags) {
        (Some(flags), _) => (CatalogRebuildFlagSource::SidecarFlags, flags),
        (None, Some(flags)) => (CatalogRebuildFlagSource::EditGraphMetadata, flags),
        (None, None) => (
            CatalogRebuildFlagSource::Defaults,
            default_rebuild_flags(&expected_photo_id),
        ),
    };
    let catalog_flags = get_photo_flags_from_connection(connection, &expected_photo_id)?;
    let action = match &catalog_flags {
        None => CatalogRebuildDryRunAction::CreatePhotoFlags,
        Some(flags) if flags != &resolved_flags => CatalogRebuildDryRunAction::UpdatePhotoFlags,
        Some(_) => CatalogRebuildDryRunAction::KeepPhotoFlags,
    };

    report.entries.push(CatalogRebuildDryRunEntry {
        photo_id: expected_photo_id,
        sidecar_relative_path,
        action,
        flag_source,
        resolved_flags,
        catalog_flags,
    });

    Ok(())
}

fn push_rebuild_issue(
    report: &mut CatalogRebuildDryRunReport,
    kind: CatalogRebuildDryRunIssueKind,
    photo_id: Option<String>,
    sidecar_relative_path: String,
    message: String,
) {
    report.issues.push(CatalogRebuildDryRunIssue {
        kind,
        photo_id,
        sidecar_relative_path,
        message,
    });
}

fn parse_valid_rebuild_sidecar_flags(
    value: &serde_json::Value,
) -> Result<PhotoFlags, LibraryStorageError> {
    let flags = parse_sidecar_flags(value)?;
    if let Some(label) = flags.color_label.as_deref() {
        edit_color_label_from_catalog(Some(label))?;
    }
    Ok(flags)
}

fn parse_valid_edit_graph_metadata_flags(
    photo_id: &str,
    value: &serde_json::Value,
) -> Result<PhotoFlags, LibraryStorageError> {
    let metadata = value
        .get("edit_graph")
        .and_then(|edit_graph| edit_graph.get("metadata"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.edit_graph.metadata must be an object".to_string(),
            )
        })?;

    let rating = metadata
        .get("rating")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "edit_graph.metadata.rating must be an integer".to_string(),
            )
        })?;
    let rating = u8::try_from(rating).map_err(|_| {
        LibraryStorageError::SidecarValidation(
            "edit_graph.metadata.rating must be 0..=5".to_string(),
        )
    })?;

    let picked = metadata
        .get("picked")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "edit_graph.metadata.picked must be boolean".to_string(),
            )
        })?;
    let rejected = metadata
        .get("rejected")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "edit_graph.metadata.rejected must be boolean".to_string(),
            )
        })?;
    let color_label = match metadata.get("color_label") {
        Some(value) if value.is_null() => None,
        Some(value) => {
            let label = value.as_str().ok_or_else(|| {
                LibraryStorageError::SidecarValidation(
                    "edit_graph.metadata.color_label must be string or null".to_string(),
                )
            })?;
            edit_color_label_from_catalog(Some(label))?;
            Some(label.to_string())
        }
        None => None,
    };

    PhotoFlags::new(photo_id.to_string(), rating, picked, rejected, color_label)
        .map_err(LibraryStorageError::from)
}

fn parse_sidecar_photo_snapshot(value: &serde_json::Value) -> Option<SidecarPhotoSnapshot> {
    let photo = value.get("photo")?.as_object()?;
    let fingerprint = photo.get("fingerprint")?.as_object()?;

    Some(SidecarPhotoSnapshot {
        original_path: photo.get("original_path")?.as_str()?.to_string(),
        file_name: photo.get("file_name")?.as_str()?.to_string(),
        file_size: fingerprint
            .get("file_size")
            .and_then(serde_json::Value::as_i64),
        modified_at: fingerprint
            .get("modified_at")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        partial_hash: fingerprint
            .get("partial_hash")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        full_hash: fingerprint.get("full_hash").and_then(|value| {
            if value.is_null() {
                None
            } else {
                value.as_str().map(ToOwned::to_owned)
            }
        }),
    })
}

fn report_catalog_reconcile_issues(
    connection: &Connection,
    photo_id: &str,
    sidecar_relative_path: &str,
    snapshot: &SidecarPhotoSnapshot,
    report: &mut CatalogRebuildDryRunReport,
) -> Result<(), LibraryStorageError> {
    let Some(catalog_photo) = load_sidecar_photo_row(connection, photo_id)? else {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::CatalogReconcileConflict,
            Some(photo_id.to_string()),
            sidecar_relative_path.to_string(),
            "catalog photo is missing; rebuild would depend on sidecar photo data".to_string(),
        );
        return Ok(());
    };

    let mut mismatches = Vec::new();
    if catalog_photo.original_path != snapshot.original_path {
        mismatches.push("original_path");
    }
    if catalog_photo.file_name != snapshot.file_name {
        mismatches.push("file_name");
    }
    if snapshot
        .file_size
        .is_some_and(|file_size| file_size != catalog_photo.file_size)
    {
        mismatches.push("file_size");
    }
    if snapshot
        .modified_at
        .as_ref()
        .is_some_and(|modified_at| Some(modified_at) != catalog_photo.modified_at.as_ref())
    {
        mismatches.push("modified_at");
    }
    if snapshot
        .partial_hash
        .as_ref()
        .is_some_and(|partial_hash| Some(partial_hash) != catalog_photo.partial_hash.as_ref())
    {
        mismatches.push("partial_hash");
    }
    if snapshot
        .full_hash
        .as_ref()
        .is_some_and(|full_hash| Some(full_hash) != catalog_photo.full_hash.as_ref())
    {
        mismatches.push("full_hash");
    }

    if !mismatches.is_empty() {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::CatalogReconcileConflict,
            Some(photo_id.to_string()),
            sidecar_relative_path.to_string(),
            format!(
                "catalog photo differs from sidecar fields: {}",
                mismatches.join(", ")
            ),
        );
    }

    Ok(())
}

fn get_photo_flags_from_connection(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<PhotoFlags>, LibraryStorageError> {
    connection
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
        .optional()?
        .map(|(photo_id, rating, picked, rejected, color_label)| {
            photo_flags_from_row(photo_id, rating, picked, rejected, color_label)
        })
        .transpose()
}

fn default_rebuild_flags(photo_id: &str) -> PhotoFlags {
    PhotoFlags::new(photo_id.to_string(), 0, false, false, None)
        .expect("default rebuild flags are valid")
}

fn checkpoint_catalog_for_backup(catalog_path: &Path) -> Result<(), LibraryStorageError> {
    let connection = open_catalog(catalog_path)?;
    let (busy, _log_frames, _checkpointed_frames): (i64, i64, i64) =
        connection.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
    if busy != 0 {
        return Err(LibraryStorageError::BackupValidation(
            "catalog WAL checkpoint could not complete cleanly".to_string(),
        ));
    }
    Ok(())
}

fn read_backup_manifest(backup_path: &Path) -> Result<BackupManifest, LibraryStorageError> {
    if !backup_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(backup_path.to_path_buf()));
    }

    let manifest_path = backup_path.join(BACKUP_MANIFEST_FILE);
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;

    if manifest.get("schema").and_then(serde_json::Value::as_str) != Some(BACKUP_SCHEMA) {
        return Err(LibraryStorageError::BackupValidation(
            "backup schema marker must be silica.backup".to_string(),
        ));
    }
    if manifest.get("version").and_then(serde_json::Value::as_i64) != Some(BACKUP_VERSION) {
        return Err(LibraryStorageError::BackupValidation(
            "backup version must be 1".to_string(),
        ));
    }
    let catalog_schema_version = manifest
        .get("catalog_schema_version")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            LibraryStorageError::BackupValidation(
                "backup catalog_schema_version must be an integer".to_string(),
            )
        })?;
    if catalog_schema_version > CURRENT_SCHEMA_VERSION {
        return Err(LibraryStorageError::BackupValidation(format!(
            "backup uses newer catalog schema {catalog_schema_version}; app supports {CURRENT_SCHEMA_VERSION}"
        )));
    }

    let files = manifest
        .get("files")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            LibraryStorageError::BackupValidation("backup files must be an array".to_string())
        })?
        .iter()
        .map(|value| {
            let file = value.as_str().ok_or_else(|| {
                LibraryStorageError::BackupValidation(
                    "backup files entries must be strings".to_string(),
                )
            })?;
            validate_backup_relative_path(file)?;
            Ok(file.to_string())
        })
        .collect::<Result<Vec<_>, LibraryStorageError>>()?;

    if !files.iter().any(|file| file == CATALOG_DATABASE_FILE) {
        return Err(LibraryStorageError::BackupValidation(
            "backup files must include catalog.db".to_string(),
        ));
    }

    Ok(BackupManifest {
        catalog_schema_version,
        files,
    })
}

fn validate_backup_relative_path(path: &str) -> Result<(), LibraryStorageError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.split('/').any(|part| part == "." || part == "..")
    {
        return Err(LibraryStorageError::BackupValidation(format!(
            "backup contains unsafe relative path: {path:?}"
        )));
    }
    Ok(())
}

fn copy_backup_file(
    source: &Path,
    destination: &Path,
    relative_path: &str,
    files: &mut Vec<String>,
) -> Result<u64, LibraryStorageError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = fs::copy(source, destination)?;
    files.push(relative_path.to_string());
    Ok(bytes)
}

fn copy_backup_directory(
    source_root: &Path,
    destination_root: &Path,
    relative_root: &str,
    files: &mut Vec<String>,
) -> Result<u64, LibraryStorageError> {
    fs::create_dir_all(destination_root)?;
    if !source_root.is_dir() {
        return Ok(0);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(source_root)? {
        entries.push(entry?);
    }
    entries.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    let mut bytes = 0_u64;
    for entry in entries {
        let file_type = entry.file_type()?;
        let name = entry
            .file_name()
            .to_str()
            .map(ToOwned::to_owned)
            .ok_or_else(|| LibraryStorageError::InvalidPath(entry.path()))?;
        let relative_path = format!("{relative_root}/{name}");
        let destination = destination_root.join(&name);

        if file_type.is_symlink() {
            return Err(LibraryStorageError::BackupValidation(format!(
                "backup refuses symlink in durable data: {relative_path}"
            )));
        }
        if file_type.is_file() && name.ends_with(".tmp") {
            continue;
        }
        if file_type.is_dir() {
            bytes += copy_backup_directory(&entry.path(), &destination, &relative_path, files)?;
        } else if file_type.is_file() {
            bytes += copy_backup_file(&entry.path(), &destination, &relative_path, files)?;
        }
    }

    Ok(bytes)
}

fn copy_backup_payload_to_root(
    backup_path: &Path,
    target_root: &Path,
) -> Result<(), LibraryStorageError> {
    fs::create_dir_all(target_root)?;
    copy_backup_file(
        &backup_path.join(CATALOG_DATABASE_FILE),
        &target_root.join(CATALOG_DATABASE_FILE),
        CATALOG_DATABASE_FILE,
        &mut Vec::new(),
    )?;
    copy_backup_directory(
        &backup_path.join(SIDECAR_DIRECTORY),
        &target_root.join(SIDECAR_DIRECTORY),
        SIDECAR_DIRECTORY,
        &mut Vec::new(),
    )?;
    Ok(())
}

fn target_has_library_state(target_root: &Path) -> Result<bool, LibraryStorageError> {
    if !target_root.exists() {
        return Ok(false);
    }
    if !target_root.is_dir() {
        return Err(LibraryStorageError::NotDirectory(target_root.to_path_buf()));
    }
    Ok(target_root.join(CATALOG_DATABASE_FILE).exists()
        || target_root.join(SIDECAR_DIRECTORY).exists())
}

fn create_restore_rollback(target_root: &Path) -> Result<PathBuf, LibraryStorageError> {
    let rollback_path = target_root
        .join(BACKUPS_DIRECTORY)
        .join(current_restore_rollback_id());
    if rollback_path.exists() {
        return Err(LibraryStorageError::BackupValidation(format!(
            "restore rollback path already exists: {}",
            rollback_path.display()
        )));
    }
    fs::create_dir_all(&rollback_path)?;

    if target_root.join(CATALOG_DATABASE_FILE).is_file() {
        copy_backup_file(
            &target_root.join(CATALOG_DATABASE_FILE),
            &rollback_path.join(CATALOG_DATABASE_FILE),
            CATALOG_DATABASE_FILE,
            &mut Vec::new(),
        )?;
    }
    if target_root.join(SIDECAR_DIRECTORY).is_dir() {
        copy_backup_directory(
            &target_root.join(SIDECAR_DIRECTORY),
            &rollback_path.join(SIDECAR_DIRECTORY),
            SIDECAR_DIRECTORY,
            &mut Vec::new(),
        )?;
    }

    Ok(rollback_path)
}

fn apply_staged_restore(
    staging_root: &Path,
    target_root: &Path,
) -> Result<LocalLibrary, LibraryStorageError> {
    if !target_root.exists() {
        fs::rename(staging_root, target_root)?;
    } else if is_empty_directory(target_root)? {
        fs::remove_dir(target_root)?;
        fs::rename(staging_root, target_root)?;
    } else {
        fs::copy(
            staging_root.join(CATALOG_DATABASE_FILE),
            target_root.join(CATALOG_DATABASE_FILE),
        )?;
        let target_sidecars = target_root.join(SIDECAR_DIRECTORY);
        if target_sidecars.exists() {
            fs::remove_dir_all(&target_sidecars)?;
        }
        copy_backup_directory(
            &staging_root.join(SIDECAR_DIRECTORY),
            &target_sidecars,
            SIDECAR_DIRECTORY,
            &mut Vec::new(),
        )?;
        fs::remove_dir_all(staging_root)?;
    }

    open_local_library(target_root)
}

fn is_empty_directory(path: &Path) -> Result<bool, LibraryStorageError> {
    if !path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(path.to_path_buf()));
    }
    Ok(fs::read_dir(path)?.next().is_none())
}

fn current_backup_id() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    format!(
        "backup-unix-{}-{}",
        duration.as_secs(),
        duration.subsec_nanos()
    )
}

fn current_restore_rollback_id() -> String {
    format!("restore-rollback-{}", current_backup_id())
}

fn restore_staging_path(target_root: &Path) -> PathBuf {
    let parent = target_root.parent().unwrap_or_else(|| Path::new("."));
    let name = target_root
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "library".to_string());
    parent.join(format!(".{name}.restore-staging-{}", current_backup_id()))
}

fn update_sidecar_status(
    catalog_path: &Path,
    photo_id: &str,
    sidecar_relative_path: &str,
    written_at: &str,
) -> Result<(), LibraryStorageError> {
    let connection = open_catalog(catalog_path)?;
    connection.execute(
        r#"
        INSERT INTO sidecar_status(photo_id, sidecar_path, last_written_at, conflict_state)
        VALUES (?1, ?2, ?3, 'clean')
        ON CONFLICT(photo_id) DO UPDATE SET
          sidecar_path = excluded.sidecar_path,
          last_written_at = excluded.last_written_at,
          conflict_state = 'clean'
        "#,
        params![photo_id, sidecar_relative_path, written_at],
    )?;
    Ok(())
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

fn open_existing_library_for_read_only_query(
    root_path: impl AsRef<Path>,
) -> Result<(LocalLibrary, Connection), LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    let connection = Connection::open_with_flags(&catalog_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    let schema_version = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    if schema_version != CURRENT_SCHEMA_VERSION {
        return Err(LibraryStorageError::CatalogSchemaVersion {
            expected: CURRENT_SCHEMA_VERSION,
            found: schema_version,
        });
    }

    Ok((
        LocalLibrary {
            root_path: root_path.to_path_buf(),
            catalog_path,
            schema_version,
        },
        connection,
    ))
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

fn scan_import_candidates(
    folder_path: &Path,
) -> Result<(Vec<ImportCandidate>, Vec<ImportIssue>), LibraryStorageError> {
    let mut candidates = Vec::new();
    let mut issues = Vec::new();

    for entry in fs::read_dir(folder_path)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    folder_path,
                    None,
                    format!("failed to read directory entry: {error}"),
                ));
                continue;
            }
        };
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    &path,
                    Some(file_name),
                    format!("failed to read entry type: {error}"),
                ));
                continue;
            }
        };

        if file_type.is_symlink() {
            issues.push(import_issue(
                ImportIssueKind::SymlinkEntrySkipped,
                &path,
                Some(file_name),
                "symbolic links are skipped by import policy",
            ));
            continue;
        }

        if is_hidden_entry(&file_name) {
            issues.push(import_issue(
                ImportIssueKind::HiddenEntrySkipped,
                &path,
                Some(file_name),
                "hidden entries are skipped by import policy",
            ));
            continue;
        }

        if file_type.is_dir() {
            if is_package_directory(&path) {
                issues.push(import_issue(
                    ImportIssueKind::PackageDirectorySkipped,
                    &path,
                    Some(file_name),
                    "package directories are skipped by import policy",
                ));
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    &path,
                    Some(file_name),
                    format!("failed to read file metadata: {error}"),
                ));
                continue;
            }
        };
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let unsupported = !is_supported_photo_extension(extension);
        let partial_hash = match partial_file_hash(&path) {
            Ok(hash) => hash,
            Err(error) => {
                issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    &path,
                    Some(file_name),
                    format!("failed to read file fingerprint: {error}"),
                ));
                continue;
            }
        };

        if unsupported {
            issues.push(import_issue(
                ImportIssueKind::UnsupportedFile,
                &path,
                Some(file_name.clone()),
                "file extension is unsupported by the local alpha",
            ));
        }

        candidates.push(ImportCandidate {
            file_name,
            path: path_to_string(&path)?,
            file_size: metadata.len() as i64,
            modified_at: modified_at_string(&metadata),
            partial_hash,
            unsupported,
        });
    }

    Ok((candidates, issues))
}

fn import_issue(
    kind: ImportIssueKind,
    path: &Path,
    file_name: Option<String>,
    message: impl Into<String>,
) -> ImportIssue {
    ImportIssue {
        kind,
        path: path.display().to_string(),
        file_name,
        message: message.into(),
    }
}

fn is_hidden_entry(file_name: &str) -> bool {
    file_name.starts_with('.') && file_name != "." && file_name != ".."
}

fn is_package_directory(path: &Path) -> bool {
    const PACKAGE_EXTENSIONS: &[&str] = &[
        "app",
        "aplibrary",
        "framework",
        "library",
        "lrdata",
        "photoslibrary",
        "plugin",
    ];

    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            PACKAGE_EXTENSIONS
                .iter()
                .any(|package| extension.eq_ignore_ascii_case(package))
        })
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
              file_type,
              partial_hash
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9, ?10)
            ON CONFLICT(library_id, path) DO UPDATE SET
              folder_id = excluded.folder_id,
              file_name = excluded.file_name,
              file_size = excluded.file_size,
              modified_at = excluded.modified_at,
              missing = 0,
              unsupported = excluded.unsupported,
              file_type = excluded.file_type,
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
                catalog_file_type_for_path(&candidate.path, candidate.unsupported),
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

fn catalog_file_type_for_path(path: &str, unsupported: bool) -> &'static str {
    if unsupported {
        return "unsupported";
    }

    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
        "jpeg"
    } else if is_supported_photo_extension(extension) {
        "raw"
    } else {
        "unsupported"
    }
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

fn library_photo_grid_item_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<LibraryPhotoGridItem> {
    let photo_id: String = row.get(0)?;
    let file_name: String = row.get(1)?;
    let path: String = row.get(2)?;
    let file_type = Path::new(&path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();
    let rating = u8::try_from(row.get::<_, Option<i64>>(5)?.unwrap_or(0))
        .unwrap_or(0)
        .min(ALPHA_MAX_RATING);

    Ok(LibraryPhotoGridItem {
        photo_id,
        file_name,
        path,
        file_type,
        thumbnail_path: row.get(9)?,
        thumbnail_cache_key: row.get(10)?,
        missing: sql_to_bool(row.get::<_, i64>(3)?),
        unsupported: sql_to_bool(row.get::<_, i64>(4)?),
        rating,
        picked: sql_to_bool(row.get::<_, Option<i64>>(6)?.unwrap_or(0)),
        rejected: sql_to_bool(row.get::<_, Option<i64>>(7)?.unwrap_or(0)),
        color_label: row.get(8)?,
    })
}

fn photo_metadata_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PhotoMetadata> {
    let unsupported = sql_to_bool(row.get::<_, i64>(4)?);
    let metadata_present = row.get::<_, i64>(7)? != 0;

    Ok(PhotoMetadata {
        photo_id: row.get(0)?,
        file_name: row.get(1)?,
        source_path: row.get(2)?,
        file_type: row.get(3)?,
        unsupported,
        file_size: PhotoMetadataField::known(row.get(5)?),
        modified_at: match row.get(6)? {
            Some(value) => PhotoMetadataField::known(value),
            None => PhotoMetadataField::unavailable(),
        },
        width: stored_metadata_field(row.get(8)?, metadata_present, unsupported),
        height: stored_metadata_field(row.get(9)?, metadata_present, unsupported),
        orientation: stored_metadata_field(row.get(10)?, metadata_present, unsupported),
        capture_time: stored_metadata_field(row.get(11)?, metadata_present, unsupported),
        camera_make: stored_metadata_field(row.get(12)?, metadata_present, unsupported),
        camera_model: stored_metadata_field(row.get(13)?, metadata_present, unsupported),
        lens_model: stored_metadata_field(row.get(14)?, metadata_present, unsupported),
    })
}

fn stored_metadata_field<T>(
    value: Option<T>,
    metadata_present: bool,
    unsupported: bool,
) -> PhotoMetadataField<T> {
    match value {
        Some(value) => PhotoMetadataField::known(value),
        None if metadata_present || unsupported => PhotoMetadataField::unavailable(),
        None => PhotoMetadataField::unknown(),
    }
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

fn validate_sidecar_photo_id(photo_id: &str) -> Result<(), LibraryStorageError> {
    if photo_id.is_empty()
        || photo_id == "."
        || photo_id == ".."
        || photo_id.contains('/')
        || photo_id.contains('\\')
        || photo_id.contains("..")
        || photo_id.chars().any(|character| {
            !(character.is_ascii_alphanumeric()
                || character == '-'
                || character == '_'
                || character == '.')
        })
    {
        return Err(LibraryStorageError::InvalidSidecarPhotoId(
            photo_id.to_string(),
        ));
    }

    Ok(())
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

const PAGED_LIBRARY_QUERY_INDEXES_SQL: &str = r#"
ALTER TABLE photos
  ADD COLUMN file_type TEXT NOT NULL DEFAULT 'unsupported'
  CHECK (file_type IN ('jpeg', 'raw', 'unsupported'));

UPDATE photos
SET file_type = CASE
  WHEN unsupported = 1 THEN 'unsupported'
  WHEN lower(file_name) GLOB '*.jpg'
    OR lower(file_name) GLOB '*.jpeg'
    OR lower(path) GLOB '*.jpg'
    OR lower(path) GLOB '*.jpeg'
    THEN 'jpeg'
  WHEN lower(file_name) GLOB '*.dng'
    OR lower(file_name) GLOB '*.cr2'
    OR lower(file_name) GLOB '*.cr3'
    OR lower(file_name) GLOB '*.nef'
    OR lower(file_name) GLOB '*.arw'
    OR lower(file_name) GLOB '*.raf'
    OR lower(file_name) GLOB '*.orf'
    OR lower(file_name) GLOB '*.rw2'
    OR lower(file_name) GLOB '*.pef'
    OR lower(file_name) GLOB '*.srw'
    OR lower(file_name) GLOB '*.raw'
    OR lower(file_name) GLOB '*.tif'
    OR lower(file_name) GLOB '*.tiff'
    OR lower(file_name) GLOB '*.heic'
    OR lower(path) GLOB '*.dng'
    OR lower(path) GLOB '*.cr2'
    OR lower(path) GLOB '*.cr3'
    OR lower(path) GLOB '*.nef'
    OR lower(path) GLOB '*.arw'
    OR lower(path) GLOB '*.raf'
    OR lower(path) GLOB '*.orf'
    OR lower(path) GLOB '*.rw2'
    OR lower(path) GLOB '*.pef'
    OR lower(path) GLOB '*.srw'
    OR lower(path) GLOB '*.raw'
    OR lower(path) GLOB '*.tif'
    OR lower(path) GLOB '*.tiff'
    OR lower(path) GLOB '*.heic'
    THEN 'raw'
  ELSE 'unsupported'
END;

CREATE INDEX IF NOT EXISTS idx_photos_library_imported_id
  ON photos(library_id, imported_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_photos_library_file_name_path_id
  ON photos(library_id, file_name ASC, path ASC, id ASC);

CREATE INDEX IF NOT EXISTS idx_photos_library_file_type_id
  ON photos(library_id, file_type, id ASC);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rating_photo_id
  ON photo_flags(rating DESC, photo_id ASC);
"#;

const PHOTO_METADATA_NORMALIZED_FIELDS_SQL: &str = r#"
ALTER TABLE photo_metadata
  ADD COLUMN width INTEGER;

ALTER TABLE photo_metadata
  ADD COLUMN height INTEGER;

ALTER TABLE photo_metadata
  ADD COLUMN orientation TEXT;
"#;

const PHOTO_METADATA_QUERY_INDEXES_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_photo_metadata_dimensions_photo_id
  ON photo_metadata(width, height, photo_id);
"#;

const LIBRARY_QUERY_COUNT_SQL: &str = r#"
SELECT COUNT(*)
FROM photos
LEFT JOIN photo_flags ON photo_flags.photo_id = photos.id
LEFT JOIN photo_metadata ON photo_metadata.photo_id = photos.id
WHERE photos.library_id = :library_id
  AND (:min_rating IS NULL OR COALESCE(photo_flags.rating, 0) >= :min_rating)
  AND (:picked IS NULL OR COALESCE(photo_flags.picked, 0) = :picked)
  AND (:rejected IS NULL OR COALESCE(photo_flags.rejected, 0) = :rejected)
  AND (:file_type IS NULL OR photos.file_type = :file_type)
  AND (
    :metadata_filter IS NULL
    OR (
      :metadata_filter = 'has_dimensions'
      AND photo_metadata.width IS NOT NULL
      AND photo_metadata.height IS NOT NULL
    )
  )
  AND (
    :search IS NULL
    OR lower(photos.file_name) LIKE :search ESCAPE '\'
    OR lower(photos.path) LIKE :search ESCAPE '\'
  )
"#;

const LIBRARY_QUERY_SELECT_SQL: &str = r#"
SELECT
  photos.id,
  photos.file_name,
  photos.path,
  photos.missing,
  photos.unsupported,
  photo_flags.rating,
  photo_flags.picked,
  photo_flags.rejected,
  photo_flags.color_label,
  thumbnail_cache.path,
  thumbnail_cache.cache_key
FROM photos
LEFT JOIN photo_flags ON photo_flags.photo_id = photos.id
LEFT JOIN photo_metadata ON photo_metadata.photo_id = photos.id
LEFT JOIN cache_records AS thumbnail_cache
  ON thumbnail_cache.photo_id = photos.id
  AND thumbnail_cache.cache_type = :thumbnail_cache_type
WHERE photos.library_id = :library_id
  AND (:min_rating IS NULL OR COALESCE(photo_flags.rating, 0) >= :min_rating)
  AND (:picked IS NULL OR COALESCE(photo_flags.picked, 0) = :picked)
  AND (:rejected IS NULL OR COALESCE(photo_flags.rejected, 0) = :rejected)
  AND (:file_type IS NULL OR photos.file_type = :file_type)
  AND (
    :metadata_filter IS NULL
    OR (
      :metadata_filter = 'has_dimensions'
      AND photo_metadata.width IS NOT NULL
      AND photo_metadata.height IS NOT NULL
    )
  )
  AND (
    :search IS NULL
    OR lower(photos.file_name) LIKE :search ESCAPE '\'
    OR lower(photos.path) LIKE :search ESCAPE '\'
  )
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LibraryQuerySqlFilter {
    min_rating: Option<i64>,
    picked: Option<i64>,
    rejected: Option<i64>,
    file_type: Option<&'static str>,
    metadata: Option<&'static str>,
    search: Option<String>,
}

impl From<&LibraryQueryFilters> for LibraryQuerySqlFilter {
    fn from(filters: &LibraryQueryFilters) -> Self {
        Self {
            min_rating: filters.min_rating.map(i64::from),
            picked: filters.picked.map(bool_to_sql),
            rejected: filters.rejected.map(bool_to_sql),
            file_type: filters.file_type.map(library_query_file_type_value),
            metadata: filters.metadata.map(library_query_metadata_filter_value),
            search: library_query_search_pattern(&filters.search),
        }
    }
}

fn query_library_photo_count(
    connection: &Connection,
    filter: &LibraryQuerySqlFilter,
) -> rusqlite::Result<u64> {
    let count: i64 = connection.query_row(
        LIBRARY_QUERY_COUNT_SQL,
        named_params! {
            ":library_id": LOCAL_LIBRARY_ID,
            ":min_rating": filter.min_rating,
            ":picked": filter.picked,
            ":rejected": filter.rejected,
            ":file_type": filter.file_type,
            ":metadata_filter": filter.metadata,
            ":search": filter.search.as_deref(),
        },
        |row| row.get(0),
    )?;

    Ok(u64::try_from(count).unwrap_or(0))
}

fn library_query_order_clause(sort: LibraryQuerySort) -> &'static str {
    match sort {
        LibraryQuerySort::ImportedAtDesc => "ORDER BY photos.imported_at DESC, photos.id ASC",
        LibraryQuerySort::FileNameAsc => {
            "ORDER BY photos.file_name ASC, photos.path ASC, photos.id ASC"
        }
        LibraryQuerySort::RatingDesc => {
            "ORDER BY COALESCE(photo_flags.rating, 0) DESC, photos.id ASC"
        }
    }
}

fn library_query_file_type_value(file_type: LibraryQueryFileType) -> &'static str {
    match file_type {
        LibraryQueryFileType::Jpeg => "jpeg",
        LibraryQueryFileType::Raw => "raw",
        LibraryQueryFileType::Unsupported => "unsupported",
    }
}

fn library_query_metadata_filter_value(metadata: LibraryQueryMetadataFilter) -> &'static str {
    match metadata {
        LibraryQueryMetadataFilter::HasDimensions => "has_dimensions",
    }
}

fn library_query_search_pattern(search: &str) -> Option<String> {
    let search = search.trim();
    if search.is_empty() {
        return None;
    }

    let mut pattern = String::from("%");
    for character in search.to_ascii_lowercase().chars() {
        if matches!(character, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern.push('%');
    Some(pattern)
}

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
    fn records_metadata_backfill_policy_without_automatic_open_work() {
        assert!(!METADATA_BACKFILL_ON_OPEN_OR_RESTORE);
        assert!(EXISTING_IMPORTS_WITHOUT_METADATA_STAY_UNKNOWN);

        let jpeg_policy = metadata_extraction_policy_for_path(Path::new("sample.jpg"));
        assert_eq!(
            jpeg_policy.dimension_source,
            MetadataDimensionSource::ExistingRasterPath
        );
        assert!(!jpeg_policy.raw_decode_supported);

        let raw_policy = metadata_extraction_policy_for_path(Path::new("sample.DNG"));
        assert_eq!(
            raw_policy.dimension_source,
            MetadataDimensionSource::Unavailable
        );
        assert!(!raw_policy.raw_decode_supported);
        assert!(!raw_policy.camera_lens_available);

        let workspace = unique_library_root("metadata-policy");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let before_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM photo_metadata", [], |row| row.get(0))
            .expect("count metadata before reopen");
        assert_eq!(before_count, 0);
        drop(connection);

        open_local_library(&library.root_path).expect("reopen library");
        let connection = open_catalog(&library.catalog_path).expect("open reopened catalog");
        let after_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM photo_metadata", [], |row| row.get(0))
            .expect("count metadata after reopen");
        assert_eq!(after_count, 0);

        remove_library_root(&workspace);
    }

    #[test]
    fn metadata_migration_adds_normalized_columns_and_upsert() {
        let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
        configure_connection(&connection).expect("configure sqlite");
        run_migrations(&mut connection).expect("run migrations");

        let columns: Vec<String> = {
            let mut statement = connection
                .prepare("SELECT name FROM pragma_table_info('photo_metadata')")
                .expect("prepare metadata column query");
            statement
                .query_map([], |row| row.get::<_, String>(0))
                .expect("query metadata columns")
                .map(|row| row.expect("metadata column"))
                .collect()
        };
        for column in ["width", "height", "orientation"] {
            assert!(
                columns.contains(&column.to_string()),
                "missing metadata column {column}"
            );
        }

        let workspace = unique_library_root("metadata-upsert");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&jpeg_file, b"jpeg placeholder bytes").expect("write supported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        upsert_photo_metadata_by_path(
            &library.root_path,
            &jpeg_file,
            PhotoMetadataUpdate {
                width: Some(2),
                height: Some(3),
                ..PhotoMetadataUpdate::unavailable()
            },
        )
        .expect("upsert metadata");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let (width, height, camera_make): (Option<i64>, Option<i64>, Option<String>) = connection
            .query_row(
                r#"
                SELECT photo_metadata.width, photo_metadata.height, photo_metadata.camera_make
                FROM photo_metadata
                JOIN photos ON photos.id = photo_metadata.photo_id
                WHERE photos.file_name = 'sample.jpg'
                "#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read metadata row");
        assert_eq!(width, Some(2));
        assert_eq!(height, Some(3));
        assert_eq!(camera_make, None);

        remove_library_root(&workspace);
    }

    #[test]
    fn metadata_filter_index_and_query_use_stored_dimensions_only() {
        let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
        configure_connection(&connection).expect("configure sqlite");
        run_migrations(&mut connection).expect("run migrations");
        assert!(
            catalog_object_exists(&connection, "idx_photo_metadata_dimensions_photo_id")
                .expect("index lookup"),
            "missing metadata dimension filter index"
        );

        let workspace = unique_library_root("metadata-filter");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let known_file = import_root.join("known.jpg");
        let unknown_file = import_root.join("unknown.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&known_file, b"known jpeg bytes").expect("write known");
        std::fs::write(&unknown_file, b"unknown jpeg bytes").expect("write unknown");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        upsert_photo_metadata_by_path(
            &library.root_path,
            &known_file,
            PhotoMetadataUpdate {
                width: Some(24),
                height: Some(36),
                ..PhotoMetadataUpdate::unavailable()
            },
        )
        .expect("upsert known metadata");
        std::fs::remove_file(&known_file).expect("remove original after metadata persist");

        let page = query_library_photos(
            &library.root_path,
            LibraryQueryRequest::new(
                0,
                100,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters {
                    metadata: Some(LibraryQueryMetadataFilter::HasDimensions),
                    ..LibraryQueryFilters::default()
                },
            ),
        )
        .expect("query metadata-backed filter");

        assert_eq!(page.total_count, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].file_name, "known.jpg");

        remove_library_root(&workspace);
    }

    #[test]
    fn metadata_query_returns_stored_states_without_original_reads() {
        let workspace = unique_library_root("metadata-query");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let known_file = import_root.join("known.jpg");
        let unknown_file = import_root.join("unknown.jpg");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&known_file, b"known jpeg bytes").expect("write known");
        std::fs::write(&unknown_file, b"unknown jpeg bytes").expect("write unknown");
        std::fs::write(&unsupported_file, b"unsupported").expect("write unsupported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        upsert_photo_metadata_by_path(
            &library.root_path,
            &known_file,
            PhotoMetadataUpdate {
                width: Some(24),
                height: Some(36),
                ..PhotoMetadataUpdate::unavailable()
            },
        )
        .expect("upsert known metadata");

        std::fs::remove_file(&known_file).expect("remove original after metadata persist");

        let known_id = stable_catalog_id("photo", &known_file.display().to_string());
        let known = get_photo_metadata(&library.root_path, &known_id)
            .expect("query known metadata")
            .expect("known photo metadata");
        assert_eq!(known.width.state, PhotoMetadataFieldState::Known);
        assert_eq!(known.width.value, Some(24));
        assert_eq!(known.height.state, PhotoMetadataFieldState::Known);
        assert_eq!(known.height.value, Some(36));
        assert_eq!(
            known.camera_make.state,
            PhotoMetadataFieldState::Unavailable
        );
        assert_eq!(known.camera_make.value, None);
        assert_eq!(known.file_size.state, PhotoMetadataFieldState::Known);

        let unknown_id = stable_catalog_id("photo", &unknown_file.display().to_string());
        let unknown = get_photo_metadata(&library.root_path, &unknown_id)
            .expect("query unknown metadata")
            .expect("unknown photo metadata");
        assert_eq!(unknown.width.state, PhotoMetadataFieldState::Unknown);
        assert_eq!(unknown.width.value, None);
        assert_eq!(unknown.camera_model.state, PhotoMetadataFieldState::Unknown);

        let unsupported_id = stable_catalog_id("photo", &unsupported_file.display().to_string());
        let unsupported = get_photo_metadata(&library.root_path, &unsupported_id)
            .expect("query unsupported metadata")
            .expect("unsupported photo metadata");
        assert!(unsupported.unsupported);
        assert_eq!(
            unsupported.width.state,
            PhotoMetadataFieldState::Unavailable
        );
        assert_eq!(
            unsupported.lens_model.state,
            PhotoMetadataFieldState::Unavailable
        );

        assert!(get_photo_metadata(&library.root_path, "missing-photo")
            .expect("query missing metadata")
            .is_none());

        remove_library_root(&workspace);
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
    fn query_index_migration_adds_normalized_file_type_and_indexes() {
        let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
        configure_connection(&connection).expect("configure sqlite");
        run_migrations(&mut connection).expect("run migrations");

        let file_type_columns: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('photos') WHERE name = 'file_type'",
                [],
                |row| row.get(0),
            )
            .expect("file_type column count");
        assert_eq!(file_type_columns, 1);

        for index_name in [
            "idx_photos_library_imported_id",
            "idx_photos_library_file_name_path_id",
            "idx_photos_library_file_type_id",
            "idx_photo_flags_rating_photo_id",
        ] {
            assert!(
                catalog_object_exists(&connection, index_name).expect("index lookup"),
                "missing paged query index {index_name}"
            );
        }

        run_migrations(&mut connection).expect("rerun migrations");
        assert_eq!(
            current_schema_version(&connection).expect("schema version"),
            CURRENT_SCHEMA_VERSION
        );
    }

    #[test]
    fn query_index_migration_backfills_photo_file_type() {
        let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
        configure_connection(&connection).expect("configure sqlite");
        run_migrations_through(&mut connection, 2).expect("run v2 migrations");

        connection
            .execute(
                "INSERT INTO libraries(id, root_path) VALUES ('local', '/tmp/library')",
                [],
            )
            .expect("insert library");
        connection
            .execute(
                "INSERT INTO folders(id, library_id, path) VALUES ('folder', 'local', '/tmp/import')",
                [],
            )
            .expect("insert folder");
        for (id, file_name, unsupported) in [
            ("photo-jpeg", "portrait.JPG", 0_i64),
            ("photo-raw", "sample.DNG", 0_i64),
            ("photo-unsupported", "notes.txt", 1_i64),
        ] {
            connection
                .execute(
                    r#"
                    INSERT INTO photos(
                      id, library_id, folder_id, file_name, path, unsupported
                    )
                    VALUES (?1, 'local', 'folder', ?2, ?3, ?4)
                    "#,
                    params![
                        id,
                        file_name,
                        format!("/tmp/import/{file_name}"),
                        unsupported
                    ],
                )
                .expect("insert v2 photo");
        }

        run_migrations(&mut connection).expect("upgrade to current");

        for (id, expected) in [
            ("photo-jpeg", "jpeg"),
            ("photo-raw", "raw"),
            ("photo-unsupported", "unsupported"),
        ] {
            let actual: String = connection
                .query_row(
                    "SELECT file_type FROM photos WHERE id = ?1",
                    params![id],
                    |row| row.get(0),
                )
                .expect("file type");
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn library_query_returns_bounded_pages_and_normalized_filters() {
        let workspace = unique_library_root("library-query");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("portrait.jpg");
        let raw_file = import_root.join("sample.DNG");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&jpeg_file, b"jpeg candidate").expect("write jpeg");
        std::fs::write(&raw_file, b"raw candidate").expect("write raw");
        std::fs::write(&unsupported_file, b"unsupported").expect("write unsupported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let raw_id = stable_catalog_id("photo", &raw_file.display().to_string());
        set_photo_flags(&library.root_path, raw_id.clone(), 4, true, false, None)
            .expect("set raw flags");

        let first_page = query_library_photos(
            &library.root_path,
            LibraryQueryRequest::new(
                0,
                2,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters::default(),
            ),
        )
        .expect("query first page");

        assert_eq!(first_page.offset, 0);
        assert_eq!(first_page.limit, 2);
        assert_eq!(first_page.total_count, 3);
        assert!(first_page.has_next_page);
        assert_eq!(
            first_page.order_fields,
            LibraryQuerySort::FileNameAsc.order_fields()
        );
        assert_eq!(first_page.items.len(), 2);
        assert_eq!(first_page.items[0].file_name, "notes.txt");
        assert_eq!(first_page.items[1].file_name, "portrait.jpg");

        let raw_page = query_library_photos(
            &library.root_path,
            LibraryQueryRequest::new(
                0,
                10,
                LibraryQuerySort::RatingDesc,
                LibraryQueryFilters {
                    min_rating: Some(4),
                    picked: Some(true),
                    rejected: Some(false),
                    file_type: Some(LibraryQueryFileType::Raw),
                    search: "sample".to_string(),
                    ..LibraryQueryFilters::default()
                },
            ),
        )
        .expect("query filtered page");
        assert_eq!(raw_page.total_count, 1);
        assert_eq!(raw_page.items.len(), 1);
        assert_eq!(raw_page.items[0].photo_id, raw_id);
        assert_eq!(raw_page.items[0].rating, 4);
        assert!(raw_page.items[0].picked);
        assert!(!raw_page.has_next_page);

        let empty_page = query_library_photos(
            &library.root_path,
            LibraryQueryRequest::new(
                99,
                10,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters::default(),
            ),
        )
        .expect("query empty page");
        assert!(empty_page.items.is_empty());
        assert_eq!(empty_page.offset, 99);
        assert_eq!(empty_page.total_count, 3);
        assert!(!empty_page.has_next_page);

        remove_library_root(&workspace);
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
    fn resolves_sidecar_paths_under_library_sidecars_only() {
        let root = unique_library_root("sidecar-path");
        let path = sidecar_path_for_photo(&root, "photo_ABC-123.ok").expect("valid sidecar path");

        assert_eq!(
            path,
            root.join("sidecars")
                .join("photo_ABC-123.ok.silicaraw.sidecar.json")
        );
        assert!(path.starts_with(root.join("sidecars")));
    }

    #[test]
    fn rejects_unsafe_sidecar_photo_ids() {
        let root = unique_library_root("sidecar-invalid-id");
        for invalid in [
            "",
            "../photo",
            "folder/photo",
            "folder\\photo",
            "/tmp/photo",
            "photo\nid",
        ] {
            assert!(
                sidecar_path_for_photo(&root, invalid).is_err(),
                "invalid photo id should be rejected: {invalid:?}"
            );
        }
    }

    #[test]
    fn builds_valid_sidecar_payload_with_flags_and_metadata_mirror() {
        let workspace = unique_library_root("sidecar-payload");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            5,
            true,
            false,
            Some("purple".to_string()),
        )
        .expect("set flags");

        let sidecar = build_photo_sidecar_value(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("build sidecar");

        validate_sidecar_json(&sidecar).expect("validate sidecar");
        assert_eq!(sidecar["schema"], SIDECAR_SCHEMA);
        assert_eq!(sidecar["version"], SIDECAR_VERSION);
        assert_eq!(sidecar["photo"]["photo_id"], photo_id);
        assert_eq!(sidecar["flags"]["rating"], 5);
        assert_eq!(sidecar["flags"]["picked"], true);
        assert_eq!(sidecar["flags"]["rejected"], false);
        assert_eq!(sidecar["flags"]["color_label"], "purple");
        assert_eq!(sidecar["edit_graph"]["metadata"]["rating"], 5);
        assert_eq!(sidecar["edit_graph"]["metadata"]["picked"], true);
        assert_eq!(sidecar["edit_graph"]["metadata"]["rejected"], false);
        assert_eq!(sidecar["edit_graph"]["metadata"]["color_label"], "purple");
        assert!(sidecar["sync"]["sidecar_hash"].is_null());
        assert!(sidecar["flags"].get("edited").is_none());
        assert!(sidecar["flags"].get("exported").is_none());

        let connection = open_catalog(library.catalog_path).expect("open catalog");
        assert_eq!(
            count_edit_states(&connection),
            0,
            "building a sidecar for an unedited photo must not write edit_states"
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn rejects_sidecar_payload_for_invalid_color_label() {
        let workspace = unique_library_root("sidecar-invalid-label");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        connection
            .execute(
                "UPDATE photo_flags SET color_label = 'cyan' WHERE photo_id = ?1",
                params![photo_id],
            )
            .expect("force invalid catalog label");
        drop(connection);

        let error = build_photo_sidecar_value(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect_err("invalid sidecar label should fail");
        assert!(error
            .to_string()
            .contains("unsupported sidecar color label"));

        remove_library_root(&workspace);
    }

    #[test]
    fn writes_sidecar_under_library_and_updates_status_after_success() {
        let workspace = unique_library_root("sidecar-write");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");
        let original_before = std::fs::read(&supported_file).expect("read original before");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            3,
            false,
            true,
            Some("red".to_string()),
        )
        .expect("set flags");

        let result = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write sidecar");

        assert_eq!(result.photo_id, photo_id);
        assert_eq!(
            result.sidecar_relative_path,
            format!("sidecars/{photo_id}.silicaraw.sidecar.json")
        );
        assert!(result.sidecar_path.is_file());
        assert!(result
            .sidecar_path
            .starts_with(library.root_path.join(SIDECAR_DIRECTORY)));
        assert!(result.bytes_written > 0);
        assert_eq!(
            std::fs::read(&supported_file).expect("read original after"),
            original_before
        );

        let json: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&result.sidecar_path).expect("read sidecar"),
        )
        .expect("parse sidecar");
        validate_sidecar_json(&json).expect("validate written sidecar");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let (sidecar_path, conflict_state): (String, String) = connection
            .query_row(
                "SELECT sidecar_path, conflict_state FROM sidecar_status WHERE photo_id = ?1",
                params![&result.photo_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("sidecar status");
        assert_eq!(sidecar_path, result.sidecar_relative_path);
        assert_eq!(conflict_state, "clean");

        remove_library_root(&workspace);
    }

    #[test]
    fn failed_sidecar_write_does_not_replace_existing_valid_sidecar() {
        let workspace = unique_library_root("sidecar-write-failure");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let first = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("first write");
        let first_bytes = std::fs::read(&first.sidecar_path).expect("read first sidecar");

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        connection
            .execute(
                "UPDATE photo_flags SET color_label = 'cyan' WHERE photo_id = ?1",
                params![photo_id],
            )
            .expect("force invalid catalog label");
        drop(connection);

        let error = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect_err("invalid write should fail");
        assert!(error
            .to_string()
            .contains("unsupported sidecar color label"));
        assert_eq!(
            std::fs::read(&first.sidecar_path).expect("read preserved sidecar"),
            first_bytes
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn reads_valid_sidecar_without_mutating_catalog_flags() {
        let workspace = unique_library_root("sidecar-read");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            4,
            true,
            false,
            Some("green".to_string()),
        )
        .expect("set sidecar flags");
        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");
        set_photo_flags(&library.root_path, photo_id.clone(), 1, false, true, None)
            .expect("change catalog flags after write");

        let sidecar = read_photo_sidecar(&library.root_path, &photo_id)
            .expect("read sidecar")
            .expect("sidecar exists");
        assert_eq!(sidecar.photo_id, photo_id);
        assert_eq!(sidecar.flags.rating, 4);
        assert!(sidecar.flags.picked);
        assert!(!sidecar.flags.rejected);
        assert_eq!(sidecar.flags.color_label.as_deref(), Some("green"));
        assert_eq!(sidecar.edit_graph.metadata.rating, 4);

        let live_flags = get_photo_flags(&library.root_path, &sidecar.photo_id)
            .expect("read live flags")
            .expect("live flags");
        assert_eq!(live_flags.rating, 1);
        assert!(!live_flags.picked);
        assert!(live_flags.rejected);
        assert_eq!(live_flags.color_label, None);

        remove_library_root(&workspace);
    }

    #[test]
    fn sidecar_read_rejects_malformed_and_mismatched_payloads() {
        let workspace = unique_library_root("sidecar-read-invalid");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let sidecar_path =
            sidecar_path_for_photo(&library.root_path, &photo_id).expect("sidecar path");
        std::fs::write(&sidecar_path, b"{not json").expect("write malformed");
        assert!(
            read_photo_sidecar(&library.root_path, &photo_id).is_err(),
            "malformed sidecar must fail"
        );

        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write valid sidecar");
        let mut value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&sidecar_path).expect("read sidecar"))
                .expect("parse sidecar");
        value["photo"]["photo_id"] = serde_json::Value::String("other-photo".to_string());
        std::fs::write(
            &sidecar_path,
            serde_json::to_vec_pretty(&value).expect("serialize mismatch"),
        )
        .expect("write mismatch");
        let error = read_photo_sidecar(&library.root_path, &photo_id)
            .expect_err("photo id mismatch must fail");
        assert!(error.to_string().contains("sidecar photo id mismatch"));

        remove_library_root(&workspace);
    }

    #[test]
    fn sidecar_read_rejects_schema_invalid_fields() {
        let workspace = unique_library_root("sidecar-read-schema-invalid");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let result = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write valid sidecar");
        let mut value: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&result.sidecar_path).expect("read sidecar"),
        )
        .expect("parse sidecar");

        value["unexpected"] = serde_json::Value::String("not allowed".to_string());
        std::fs::write(
            &result.sidecar_path,
            serde_json::to_vec_pretty(&value).expect("serialize top-level extra"),
        )
        .expect("write top-level extra");
        let error = read_photo_sidecar(&library.root_path, &photo_id)
            .expect_err("top-level extra field must fail");
        assert!(error.to_string().contains("unsupported top-level field"));

        value
            .as_object_mut()
            .expect("sidecar object")
            .remove("unexpected");
        value["sync"]["status"] = serde_json::Value::String("proof_claim".to_string());
        std::fs::write(
            &result.sidecar_path,
            serde_json::to_vec_pretty(&value).expect("serialize bad sync"),
        )
        .expect("write bad sync");
        let error = read_photo_sidecar(&library.root_path, &photo_id)
            .expect_err("bad sync status must fail");
        assert!(error.to_string().contains("sidecar.sync.status"));

        remove_library_root(&workspace);
    }

    #[test]
    fn rebuild_dry_run_reports_deterministic_updates_without_mutating_catalog() {
        let workspace = unique_library_root("sidecar-rebuild-dry-run");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            5,
            true,
            false,
            Some("purple".to_string()),
        )
        .expect("set sidecar flags");
        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");
        set_photo_flags(&library.root_path, photo_id.clone(), 1, false, true, None)
            .expect("change live catalog flags");

        let report = dry_run_catalog_rebuild_from_sidecars(&library.root_path)
            .expect("dry-run sidecar rebuild");
        let second_report = dry_run_catalog_rebuild_from_sidecars(&library.root_path)
            .expect("repeat dry-run sidecar rebuild");

        assert_eq!(report, second_report, "dry-run output must be stable");
        assert_eq!(report.sidecars_scanned, 1);
        assert!(report.issues.is_empty());
        assert_eq!(report.entries.len(), 1);
        let entry = &report.entries[0];
        assert_eq!(entry.photo_id, photo_id);
        assert_eq!(entry.action, CatalogRebuildDryRunAction::UpdatePhotoFlags);
        assert_eq!(entry.flag_source, CatalogRebuildFlagSource::SidecarFlags);
        assert_eq!(entry.resolved_flags.rating, 5);
        assert!(entry.resolved_flags.picked);
        assert!(!entry.resolved_flags.rejected);
        assert_eq!(entry.resolved_flags.color_label.as_deref(), Some("purple"));

        let live_flags = get_photo_flags(&library.root_path, &photo_id)
            .expect("read live flags")
            .expect("live flags");
        assert_eq!(live_flags.rating, 1);
        assert!(!live_flags.picked);
        assert!(live_flags.rejected);
        assert_eq!(live_flags.color_label, None);

        remove_library_root(&workspace);
    }

    #[test]
    fn rebuild_dry_run_uses_metadata_fallback_and_defaults() {
        let workspace = unique_library_root("sidecar-rebuild-precedence");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let metadata_file = import_root.join("metadata.jpg");
        let defaults_file = import_root.join("defaults.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&metadata_file, b"metadata jpeg bytes").expect("write metadata original");
        std::fs::write(&defaults_file, b"defaults jpeg bytes").expect("write defaults original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let metadata_photo_id = stable_catalog_id("photo", &metadata_file.display().to_string());
        let defaults_photo_id = stable_catalog_id("photo", &defaults_file.display().to_string());

        let metadata_sidecar =
            write_photo_sidecar(&library.root_path, &metadata_photo_id, "0.1.0-alpha.1")
                .expect("write metadata sidecar");
        let mut metadata_value: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&metadata_sidecar.sidecar_path).expect("read sidecar"),
        )
        .expect("parse metadata sidecar");
        metadata_value
            .as_object_mut()
            .expect("sidecar object")
            .remove("flags");
        metadata_value["edit_graph"]["metadata"]["rating"] = serde_json::json!(2);
        metadata_value["edit_graph"]["metadata"]["picked"] = serde_json::json!(true);
        metadata_value["edit_graph"]["metadata"]["rejected"] = serde_json::json!(false);
        metadata_value["edit_graph"]["metadata"]["color_label"] = serde_json::json!("blue");
        std::fs::write(
            &metadata_sidecar.sidecar_path,
            serde_json::to_vec_pretty(&metadata_value).expect("serialize metadata fallback"),
        )
        .expect("write metadata fallback sidecar");

        let defaults_sidecar =
            write_photo_sidecar(&library.root_path, &defaults_photo_id, "0.1.0-alpha.1")
                .expect("write defaults sidecar");
        let mut defaults_value: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&defaults_sidecar.sidecar_path).expect("read sidecar"),
        )
        .expect("parse defaults sidecar");
        defaults_value["flags"]["rating"] = serde_json::json!(99);
        defaults_value["edit_graph"]["metadata"]["rating"] = serde_json::json!(9);
        std::fs::write(
            &defaults_sidecar.sidecar_path,
            serde_json::to_vec_pretty(&defaults_value).expect("serialize defaults fallback"),
        )
        .expect("write defaults fallback sidecar");

        let report = dry_run_catalog_rebuild_from_sidecars(&library.root_path)
            .expect("dry-run sidecar rebuild");

        assert_eq!(report.sidecars_scanned, 2);
        assert_eq!(report.entries.len(), 2);
        assert!(
            report
                .issues
                .iter()
                .filter(|issue| issue.kind == CatalogRebuildDryRunIssueKind::SchemaInvalid)
                .count()
                >= 2,
            "schema-invalid sidecars must be reported"
        );

        let metadata_entry = report
            .entries
            .iter()
            .find(|entry| entry.photo_id == metadata_photo_id)
            .expect("metadata entry");
        assert_eq!(
            metadata_entry.flag_source,
            CatalogRebuildFlagSource::EditGraphMetadata
        );
        assert_eq!(metadata_entry.resolved_flags.rating, 2);
        assert!(metadata_entry.resolved_flags.picked);
        assert_eq!(
            metadata_entry.resolved_flags.color_label.as_deref(),
            Some("blue")
        );

        let defaults_entry = report
            .entries
            .iter()
            .find(|entry| entry.photo_id == defaults_photo_id)
            .expect("defaults entry");
        assert_eq!(
            defaults_entry.flag_source,
            CatalogRebuildFlagSource::Defaults
        );
        assert_eq!(defaults_entry.resolved_flags.rating, 0);
        assert!(!defaults_entry.resolved_flags.picked);
        assert!(!defaults_entry.resolved_flags.rejected);
        assert_eq!(defaults_entry.resolved_flags.color_label, None);

        remove_library_root(&workspace);
    }

    #[test]
    fn rebuild_dry_run_reports_conflicts_and_identity_mismatch() {
        let workspace = unique_library_root("sidecar-rebuild-conflicts");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            4,
            true,
            false,
            Some("green".to_string()),
        )
        .expect("set flags");
        let sidecar = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write sidecar");
        let mut value: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&sidecar.sidecar_path).expect("read sidecar"),
        )
        .expect("parse sidecar");
        value["flags"]["rating"] = serde_json::json!(3);
        value["photo"]["original_path"] = serde_json::json!("/tmp/elsewhere/sample.jpg");
        std::fs::write(
            &sidecar.sidecar_path,
            serde_json::to_vec_pretty(&value).expect("serialize conflict sidecar"),
        )
        .expect("write conflict sidecar");

        let mismatch_path =
            sidecar_path_for_photo(&library.root_path, "mismatch-photo").expect("mismatch path");
        std::fs::write(
            &mismatch_path,
            serde_json::to_vec_pretty(&value).expect("serialize mismatch sidecar"),
        )
        .expect("write mismatch sidecar");

        let report = dry_run_catalog_rebuild_from_sidecars(&library.root_path)
            .expect("dry-run sidecar rebuild");

        assert_eq!(report.sidecars_scanned, 2);
        let entry = report
            .entries
            .iter()
            .find(|entry| entry.photo_id == photo_id)
            .expect("entry");
        assert_eq!(entry.flag_source, CatalogRebuildFlagSource::SidecarFlags);
        assert_eq!(entry.resolved_flags.rating, 3);
        assert!(report.issues.iter().any(|issue| {
            issue.kind == CatalogRebuildDryRunIssueKind::FlagsMetadataConflict
                && issue.photo_id.as_deref() == Some(&photo_id)
        }));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == CatalogRebuildDryRunIssueKind::CatalogReconcileConflict
                && issue.photo_id.as_deref() == Some(&photo_id)
        }));
        assert!(report.issues.iter().any(|issue| {
            issue.kind == CatalogRebuildDryRunIssueKind::PhotoIdMismatch
                && issue.sidecar_relative_path
                    == format!("{SIDECAR_DIRECTORY}/mismatch-photo.silicaraw.sidecar.json")
        }));

        remove_library_root(&workspace);
    }

    #[test]
    fn rebuild_dry_run_reports_malformed_sidecars_without_entries() {
        let workspace = unique_library_root("sidecar-rebuild-malformed");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let sidecar_path =
            sidecar_path_for_photo(&library.root_path, &photo_id).expect("sidecar path");
        std::fs::write(&sidecar_path, b"{not json").expect("write malformed sidecar");

        let report = dry_run_catalog_rebuild_from_sidecars(&library.root_path)
            .expect("dry-run sidecar rebuild");

        assert_eq!(report.sidecars_scanned, 1);
        assert!(report.entries.is_empty());
        assert_eq!(report.issues.len(), 1);
        assert_eq!(
            report.issues[0].kind,
            CatalogRebuildDryRunIssueKind::MalformedJson
        );
        assert_eq!(
            report.issues[0].photo_id.as_deref(),
            Some(photo_id.as_str())
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn creates_backup_with_catalog_sidecars_manifest_and_excludes_disposable_data() {
        let workspace = unique_library_root("backup-boundaries");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("External Exports");
        let supported_file = import_root.join("sample.jpg");
        let export_file = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");
        std::fs::write(&export_file, b"exported jpeg bytes").expect("write export output");
        let original_bytes = std::fs::read(&supported_file).expect("read original before");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            4,
            true,
            false,
            Some("green".to_string()),
        )
        .expect("set flags");
        let sidecar = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write sidecar");
        record_export(
            &library.root_path,
            &photo_id,
            &export_file,
            r#"{"format":"jpeg","color_profile":"srgb"}"#,
        )
        .expect("record export");

        for directory in DISPOSABLE_CACHE_DIRECTORIES {
            let path = library.root_path.join(directory);
            std::fs::write(path.join("sentinel.cache"), b"cache bytes").expect("write cache");
        }
        std::fs::write(
            library.root_path.join("backups").join("old-backup.txt"),
            b"old backup bytes",
        )
        .expect("write old backup sentinel");
        std::fs::write(
            library.root_path.join("logs").join("runtime.log"),
            b"runtime log",
        )
        .expect("write log sentinel");
        let sidecar_temp = library
            .root_path
            .join(SIDECAR_DIRECTORY)
            .join("partial.silicaraw.sidecar.json.tmp");
        std::fs::write(&sidecar_temp, b"partial sidecar temp").expect("write sidecar temp");

        let backup = create_library_backup(&library.root_path, "0.1.0-alpha.1")
            .expect("create library backup");

        assert!(backup
            .backup_path
            .starts_with(library.root_path.join("backups")));
        assert!(backup.backup_path.join(CATALOG_DATABASE_FILE).is_file());
        assert!(backup.manifest_path.is_file());
        assert!(backup
            .backup_path
            .join(&sidecar.sidecar_relative_path)
            .is_file());
        assert!(
            !backup
                .backup_path
                .join(SIDECAR_DIRECTORY)
                .join("partial.silicaraw.sidecar.json.tmp")
                .exists(),
            "sidecar temp files must not be copied into backup"
        );
        for directory in [
            "thumbnails",
            "previews",
            "render-cache",
            "ai-cache",
            "exports",
            "logs",
            "backups",
        ] {
            assert!(
                !backup.backup_path.join(directory).exists(),
                "{directory} must not be copied into backup"
            );
        }
        assert!(
            !backup
                .backup_path
                .join(export_file.file_name().expect("export file name"))
                .exists(),
            "export output files must not be followed into backup"
        );
        assert_eq!(
            std::fs::read(&supported_file).expect("read original after"),
            original_bytes
        );

        let manifest: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(&backup.manifest_path).expect("read manifest"),
        )
        .expect("parse manifest");
        assert_eq!(manifest["schema"], BACKUP_SCHEMA);
        assert_eq!(manifest["version"], BACKUP_VERSION);
        assert_eq!(manifest["app_version"], "0.1.0-alpha.1");
        assert_eq!(manifest["catalog_schema_version"], CURRENT_SCHEMA_VERSION);
        assert!(manifest["files"]
            .as_array()
            .expect("manifest file list")
            .contains(&serde_json::Value::String(
                CATALOG_DATABASE_FILE.to_string()
            )));
        assert!(manifest["files"]
            .as_array()
            .expect("manifest file list")
            .contains(&serde_json::Value::String(sidecar.sidecar_relative_path)));

        let backup_connection =
            open_catalog(backup.backup_path.join(CATALOG_DATABASE_FILE)).expect("open backup db");
        let export_count: i64 = backup_connection
            .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
            .expect("count backup exports");
        let sidecar_status_count: i64 = backup_connection
            .query_row("SELECT COUNT(*) FROM sidecar_status", [], |row| row.get(0))
            .expect("count backup sidecar status");
        assert_eq!(export_count, 1);
        assert_eq!(sidecar_status_count, 1);

        remove_library_root(&workspace);
    }

    #[test]
    fn backup_checkpoint_copies_latest_wal_state_without_wal_files() {
        let workspace = unique_library_root("backup-wal");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

        let connection = open_catalog(&library.catalog_path).expect("open writer connection");
        connection
            .execute(
                "UPDATE photo_flags SET rating = 5, picked = 1 WHERE photo_id = ?1",
                params![photo_id],
            )
            .expect("write uncheckpointed flag state");

        let backup = create_library_backup(&library.root_path, "0.1.0-alpha.1")
            .expect("create library backup");

        assert!(backup.backup_path.join(CATALOG_DATABASE_FILE).is_file());
        assert!(!backup.backup_path.join("catalog.db-wal").exists());
        assert!(!backup.backup_path.join("catalog.db-shm").exists());

        let backup_connection =
            open_catalog(backup.backup_path.join(CATALOG_DATABASE_FILE)).expect("open backup db");
        let (rating, picked): (i64, i64) = backup_connection
            .query_row(
                "SELECT rating, picked FROM photo_flags WHERE photo_id = ?1",
                params![stable_catalog_id(
                    "photo",
                    &supported_file.display().to_string()
                )],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read backup flags");
        assert_eq!(rating, 5);
        assert_eq!(picked, 1);

        drop(connection);
        remove_library_root(&workspace);
    }

    #[test]
    fn restores_backup_to_empty_directory_preserving_catalog_state_and_originals() {
        let workspace = unique_library_root("restore-empty");
        let library_root = workspace.join("SilicaRAW Library");
        let restore_root = workspace.join("Restored Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("External Exports");
        let supported_file = import_root.join("sample.jpg");
        let export_file = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");
        std::fs::write(&export_file, b"exported jpeg bytes").expect("write export output");
        let original_bytes = std::fs::read(&supported_file).expect("read original before");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            photo_id.clone(),
            5,
            true,
            false,
            Some("purple".to_string()),
        )
        .expect("set flags");
        let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
            .expect("load draft")
            .expect("draft");
        let edited =
            silica_edit::apply_exposure_contrast(&draft, 0.75, -12.0, "unix:7").expect("edit");
        commit_edit_graph(&library.root_path, edited).expect("commit edit");
        let sidecar = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write sidecar");
        record_export(
            &library.root_path,
            &photo_id,
            &export_file,
            r#"{"format":"jpeg","color_profile":"srgb"}"#,
        )
        .expect("record export");
        let backup =
            create_library_backup(&library.root_path, "0.1.0-alpha.1").expect("create backup");

        let restored =
            restore_library_backup(&backup.backup_path, &restore_root).expect("restore backup");

        assert_eq!(restored.restored_library.root_path, restore_root);
        assert!(restored.rollback_path.is_none());
        assert!(restore_root.join(CATALOG_DATABASE_FILE).is_file());
        assert!(restore_root.join(&sidecar.sidecar_relative_path).is_file());
        assert_eq!(
            std::fs::read(&supported_file).expect("read original after"),
            original_bytes
        );

        let restored_library = open_local_library(&restore_root).expect("open restored library");
        let restored_flags = get_photo_flags(&restored_library.root_path, &photo_id)
            .expect("read restored flags")
            .expect("restored flags");
        assert_eq!(restored_flags.rating, 5);
        assert!(restored_flags.picked);
        assert_eq!(restored_flags.color_label.as_deref(), Some("purple"));

        let restored_graph =
            load_active_edit_graph_or_default(&restored_library.root_path, &photo_id)
                .expect("read restored edit")
                .expect("restored edit");
        assert_eq!(restored_graph.basic.exposure.as_f64(), Some(0.75));
        assert_eq!(restored_graph.basic.contrast.as_f64(), Some(-12.0));

        let connection = open_catalog(&restored_library.catalog_path).expect("open restored db");
        assert_eq!(count_edit_states(&connection), 1);
        let export_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
            .expect("count exports");
        let sidecar_status_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM sidecar_status", [], |row| row.get(0))
            .expect("count sidecar status");
        let schema_version: i64 = connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .expect("schema version");
        let (edited_flag, exported_flag): (i64, i64) = connection
            .query_row(
                "SELECT edited, exported FROM photo_flags WHERE photo_id = ?1",
                params![photo_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("edited/exported flags");
        assert_eq!(export_count, 1);
        assert_eq!(sidecar_status_count, 1);
        assert_eq!(schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(edited_flag, 1);
        assert_eq!(exported_flag, 1);

        remove_library_root(&workspace);
    }

    #[test]
    fn restore_into_existing_library_creates_rollback_before_replacing_state() {
        let workspace = unique_library_root("restore-existing");
        let source_root = workspace.join("Source Library");
        let target_root = workspace.join("Target Library");
        let source_import = workspace.join("Source Originals");
        let target_import = workspace.join("Target Originals");
        let source_file = source_import.join("source.jpg");
        let target_file = target_import.join("target.jpg");

        std::fs::create_dir_all(&source_import).expect("create source import");
        std::fs::create_dir_all(&target_import).expect("create target import");
        std::fs::write(&source_file, b"source jpeg bytes").expect("write source");
        std::fs::write(&target_file, b"target jpeg bytes").expect("write target");

        let source_library = create_local_library(&source_root).expect("create source library");
        import_folder(&source_library.root_path, &source_import).expect("import source");
        let source_photo_id = stable_catalog_id("photo", &source_file.display().to_string());
        set_photo_flags(
            &source_library.root_path,
            source_photo_id.clone(),
            4,
            true,
            false,
            None,
        )
        .expect("set source flags");
        write_photo_sidecar(&source_library.root_path, &source_photo_id, "0.1.0-alpha.1")
            .expect("write source sidecar");
        let backup =
            create_library_backup(&source_library.root_path, "0.1.0-alpha.1").expect("backup");

        let target_library = create_local_library(&target_root).expect("create target library");
        import_folder(&target_library.root_path, &target_import).expect("import target");
        let target_photo_id = stable_catalog_id("photo", &target_file.display().to_string());
        set_photo_flags(
            &target_library.root_path,
            target_photo_id.clone(),
            1,
            false,
            true,
            None,
        )
        .expect("set target flags");
        write_photo_sidecar(&target_library.root_path, &target_photo_id, "0.1.0-alpha.1")
            .expect("write target sidecar");

        let restored = restore_library_backup(&backup.backup_path, &target_root)
            .expect("restore over existing target");
        let rollback_path = restored.rollback_path.expect("rollback path");
        assert!(rollback_path.starts_with(target_root.join(BACKUPS_DIRECTORY)));
        assert!(rollback_path.join(CATALOG_DATABASE_FILE).is_file());
        assert!(
            rollback_path
                .join(SIDECAR_DIRECTORY)
                .join(format!("{target_photo_id}{SIDECAR_FILE_SUFFIX}"))
                .is_file(),
            "rollback must preserve previous target sidecar"
        );

        let restored_items = list_library_photos(&target_root).expect("list restored target");
        assert!(restored_items
            .iter()
            .any(|item| item.photo_id == source_photo_id));
        assert!(!restored_items
            .iter()
            .any(|item| item.photo_id == target_photo_id));

        let rollback_connection =
            open_catalog(rollback_path.join(CATALOG_DATABASE_FILE)).expect("open rollback db");
        let rollback_photo_count: i64 = rollback_connection
            .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))
            .expect("count rollback photos");
        let rollback_target_count: i64 = rollback_connection
            .query_row(
                "SELECT COUNT(*) FROM photos WHERE id = ?1",
                params![target_photo_id],
                |row| row.get(0),
            )
            .expect("count rollback target photo");
        assert_eq!(rollback_photo_count, 1);
        assert_eq!(rollback_target_count, 1);

        remove_library_root(&workspace);
    }

    #[test]
    fn restore_rejects_newer_schema_backup_without_mutating_existing_library() {
        let workspace = unique_library_root("restore-newer-schema");
        let source_root = workspace.join("Source Library");
        let target_root = workspace.join("Target Library");
        let source_import = workspace.join("Source Originals");
        let target_import = workspace.join("Target Originals");
        let source_file = source_import.join("source.jpg");
        let target_file = target_import.join("target.jpg");

        std::fs::create_dir_all(&source_import).expect("create source import");
        std::fs::create_dir_all(&target_import).expect("create target import");
        std::fs::write(&source_file, b"source jpeg bytes").expect("write source");
        std::fs::write(&target_file, b"target jpeg bytes").expect("write target");

        let source_library = create_local_library(&source_root).expect("create source library");
        import_folder(&source_library.root_path, &source_import).expect("import source");
        let backup =
            create_library_backup(&source_library.root_path, "0.1.0-alpha.1").expect("backup");
        let manifest_path = backup.backup_path.join(BACKUP_MANIFEST_FILE);
        let mut manifest: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("read manifest"))
                .expect("parse manifest");
        manifest["catalog_schema_version"] = serde_json::json!(CURRENT_SCHEMA_VERSION + 1);
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).expect("serialize newer manifest"),
        )
        .expect("write newer manifest");

        let target_library = create_local_library(&target_root).expect("create target library");
        import_folder(&target_library.root_path, &target_import).expect("import target");
        let target_photo_id = stable_catalog_id("photo", &target_file.display().to_string());
        set_photo_flags(
            &target_library.root_path,
            target_photo_id.clone(),
            2,
            false,
            true,
            None,
        )
        .expect("set target flags");

        let error = restore_library_backup(&backup.backup_path, &target_root)
            .expect_err("newer schema restore must fail");
        assert!(error.to_string().contains("newer catalog schema"));

        let target_flags = get_photo_flags(&target_root, &target_photo_id)
            .expect("read target flags")
            .expect("target flags still present");
        assert_eq!(target_flags.rating, 2);
        assert!(target_flags.rejected);
        assert!(!target_root
            .join(BACKUPS_DIRECTORY)
            .join("restore-rollback")
            .exists());

        remove_library_root(&workspace);
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

        let (path, file_size, unsupported, file_type, partial_hash): (
            String,
            i64,
            i64,
            String,
            String,
        ) = connection
            .query_row(
                "SELECT path, file_size, unsupported, file_type, partial_hash FROM photos WHERE file_name = 'sample.DNG'",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .expect("supported row");
        assert_eq!(path, supported_file.display().to_string());
        assert_eq!(file_size, supported_before.len() as i64);
        assert_eq!(unsupported, 0);
        assert_eq!(file_type, "raw");
        assert!(!partial_hash.is_empty());

        let (unsupported, file_type): (i64, String) = connection
            .query_row(
                "SELECT unsupported, file_type FROM photos WHERE file_name = 'notes.txt'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("unsupported row");
        assert_eq!(unsupported, 1);
        assert_eq!(file_type, "unsupported");

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
    fn import_error_summary_reports_recoverable_issues_without_blocking_browse() {
        let workspace = unique_library_root("import-errors");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");
        let hidden_file = import_root.join(".hidden.jpg");
        let package_dir = import_root.join("Archive.photoslibrary");
        let symlink_path = import_root.join("linked");

        std::fs::create_dir_all(&package_dir).expect("create package directory");
        std::fs::write(&supported_file, b"supported jpeg candidate").expect("write supported");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");
        std::fs::write(&hidden_file, b"hidden jpeg candidate").expect("write hidden");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&supported_file, &symlink_path).expect("create symlink");

        let library = create_local_library(&library_root).expect("create library");
        let summary = import_folder(&library.root_path, &import_root).expect("import folder");

        assert_eq!(summary.scanned_files, 2);
        assert_eq!(summary.supported_files, 1);
        assert_eq!(summary.unsupported_files, 1);
        assert_issue_kind(
            &summary.issues,
            ImportIssueKind::UnsupportedFile,
            "notes.txt",
        );
        assert_issue_kind(
            &summary.issues,
            ImportIssueKind::HiddenEntrySkipped,
            ".hidden.jpg",
        );
        assert_issue_kind(
            &summary.issues,
            ImportIssueKind::PackageDirectorySkipped,
            "Archive.photoslibrary",
        );
        #[cfg(unix)]
        assert_issue_kind(
            &summary.issues,
            ImportIssueKind::SymlinkEntrySkipped,
            "linked",
        );

        let items = list_library_photos(&library.root_path).expect("list imported rows");
        assert!(items.iter().any(|item| item.file_name == "sample.jpg"));
        assert!(items
            .iter()
            .any(|item| item.file_name == "notes.txt" && item.unsupported));
        assert!(!items.iter().any(|item| item.file_name == ".hidden.jpg"));

        remove_library_root(&workspace);
    }

    #[test]
    fn lists_library_photo_grid_items_with_flags_and_states() {
        let workspace = unique_library_root("grid-items");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let supported_id = stable_catalog_id("photo", &supported_file.display().to_string());
        set_photo_flags(
            &library.root_path,
            supported_id.clone(),
            4,
            true,
            false,
            Some("green".to_string()),
        )
        .expect("set grid flags");

        let items = list_library_photos(&library.root_path).expect("list library photos");

        assert_eq!(items.len(), 2);
        let supported = items
            .iter()
            .find(|item| item.file_name == "sample.DNG")
            .expect("supported grid item");
        assert_eq!(supported.photo_id, supported_id);
        assert_eq!(supported.file_type, "DNG");
        assert_eq!(supported.rating, 4);
        assert!(supported.picked);
        assert!(!supported.rejected);
        assert_eq!(supported.color_label.as_deref(), Some("green"));
        assert!(!supported.missing);
        assert!(!supported.unsupported);

        let unsupported = items
            .iter()
            .find(|item| item.file_name == "notes.txt")
            .expect("unsupported grid item");
        assert_eq!(unsupported.file_type, "TXT");
        assert!(unsupported.unsupported);
        assert_eq!(unsupported.rating, 0);

        remove_library_root(&workspace);
    }

    #[test]
    fn records_thumbnail_cache_and_exposes_grid_path() {
        let workspace = unique_library_root("thumbnail-cache");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let thumbnail_path = library
            .root_path
            .join("thumbnails")
            .join("sample-thumb.jpg");
        std::fs::write(&thumbnail_path, b"thumbnail bytes").expect("write thumbnail");

        let record = record_thumbnail_cache(
            &library.root_path,
            &photo_id,
            "thumbnail-key",
            &thumbnail_path,
            15,
        )
        .expect("record thumbnail cache");
        assert_eq!(record.photo_id.as_deref(), Some(photo_id.as_str()));
        assert_eq!(record.cache_type, THUMBNAIL_CACHE_TYPE);
        assert_eq!(record.path, thumbnail_path.display().to_string());
        assert_eq!(record.byte_size, 15);

        let items = list_library_photos(&library.root_path).expect("list grid items");
        let item = items
            .iter()
            .find(|item| item.file_name == "sample.jpg")
            .expect("jpeg grid item");
        assert_eq!(
            item.thumbnail_path.as_deref(),
            Some(thumbnail_path.display().to_string().as_str())
        );
        assert_eq!(item.thumbnail_cache_key.as_deref(), Some("thumbnail-key"));

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let cache_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'thumbnail'",
                [],
                |row| row.get(0),
            )
            .expect("count thumbnail cache rows");
        assert_eq!(cache_count, 1);

        remove_library_root(&workspace);
    }

    #[test]
    fn records_preview_cache_and_reads_it_by_photo_type() {
        let workspace = unique_library_root("preview-cache");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview_path = library
            .root_path
            .join("previews")
            .join("sample-preview.jpg");
        std::fs::write(&preview_path, b"preview bytes").expect("write preview");

        let record = record_preview_cache(
            &library.root_path,
            &photo_id,
            "preview-key",
            &preview_path,
            13,
        )
        .expect("record preview cache");
        assert_eq!(record.cache_type, PREVIEW_CACHE_TYPE);

        let cached = get_photo_cache_record(&library.root_path, &photo_id, PREVIEW_CACHE_TYPE)
            .expect("read preview cache")
            .expect("preview cache row");
        assert_eq!(cached.path, preview_path.display().to_string());
        assert_eq!(cached.cache_key, "preview-key");
        assert_eq!(cached.byte_size, 13);

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let cache_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'preview'",
                [],
                |row| row.get(0),
            )
            .expect("count preview cache rows");
        assert_eq!(cache_count, 1);

        remove_library_root(&workspace);
    }

    #[test]
    fn clears_only_disposable_cache_directories() {
        let workspace = unique_library_root("clear-cache");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
        let original_bytes = std::fs::read(&supported_file).expect("read original before");

        let library = create_local_library(&library_root).expect("create library");
        import_folder(&library.root_path, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

        for directory in DISPOSABLE_CACHE_DIRECTORIES {
            let path = library.root_path.join(directory);
            std::fs::create_dir_all(&path).expect("create cache directory");
            std::fs::write(path.join("sentinel.cache"), b"disposable cache bytes")
                .expect("write cache sentinel");
        }
        for directory in ["sidecars", "exports", "logs", "backups"] {
            let path = library.root_path.join(directory);
            std::fs::create_dir_all(&path).expect("create protected directory");
            std::fs::write(path.join("keep.txt"), b"preserve this").expect("write protected file");
        }

        let thumbnail_path = library
            .root_path
            .join("thumbnails")
            .join("sample-thumb.jpg");
        let preview_path = library
            .root_path
            .join("previews")
            .join("sample-preview.jpg");
        std::fs::write(&thumbnail_path, b"thumbnail bytes").expect("write thumbnail cache");
        std::fs::write(&preview_path, b"preview bytes").expect("write preview cache");
        record_thumbnail_cache(
            &library.root_path,
            &photo_id,
            "thumbnail-key",
            &thumbnail_path,
            15,
        )
        .expect("record thumbnail cache");
        record_preview_cache(
            &library.root_path,
            &photo_id,
            "preview-key",
            &preview_path,
            13,
        )
        .expect("record preview cache");

        let summary = clear_disposable_cache(&library.root_path).expect("clear cache");

        assert_eq!(
            summary.cleared_directories,
            DISPOSABLE_CACHE_DIRECTORIES
                .iter()
                .map(|directory| directory.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(summary.removed_cache_records, 2);
        for directory in DISPOSABLE_CACHE_DIRECTORIES {
            let path = library.root_path.join(directory);
            assert!(path.is_dir(), "{directory} should be recreated");
            assert!(
                !path.join("sentinel.cache").exists(),
                "{directory} sentinel should be removed"
            );
        }
        for directory in ["sidecars", "exports", "logs", "backups"] {
            assert!(
                library.root_path.join(directory).join("keep.txt").is_file(),
                "{directory} should be preserved"
            );
        }

        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        let photo_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))
            .expect("count photos");
        let cache_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM cache_records", [], |row| row.get(0))
            .expect("count cache records");
        assert_eq!(photo_count, 1);
        assert_eq!(cache_count, 0);
        assert_eq!(
            std::fs::read(&supported_file).expect("read original after"),
            original_bytes
        );

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

    fn assert_issue_kind(issues: &[ImportIssue], kind: ImportIssueKind, file_name: &str) {
        assert!(
            issues
                .iter()
                .any(|issue| issue.kind == kind && issue.file_name.as_deref() == Some(file_name)),
            "missing {kind:?} for {file_name}: {issues:?}"
        );
    }

    fn count_edit_states(connection: &Connection) -> i64 {
        connection
            .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
            .expect("count edit states")
    }
}
