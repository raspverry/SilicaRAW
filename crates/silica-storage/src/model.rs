use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use silica_catalog::{
    CatalogFlagError, ImportCandidate, ImportIssue, PhotoFlags, ALPHA_CATALOG_REQUIRED_INDEXES,
    ALPHA_CATALOG_REQUIRED_TABLES, ALPHA_CATALOG_SCHEMA_VERSION,
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

/// Stable sidecar schema marker required by `schemas/sidecar.schema.json`.
pub const SIDECAR_SCHEMA: &str = "silica.sidecar";

/// Stable sidecar schema version for v0.1.
pub const SIDECAR_VERSION: i64 = 1;

/// Stable AI result schema marker for Task 24.3 local result records.
pub const AI_RESULT_SCHEMA: &str = "silica.ai_result";

/// Stable AI result schema version for Task 24.3.
pub const AI_RESULT_VERSION: i64 = 1;

/// Permission required before a future model can propose local AI result data.
pub const AI_RESULT_PROPOSE_PERMISSION_ID: &str = "ai_result:propose";

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
/// Cache record type used for disposable Develop histogram data.
pub const HISTOGRAM_CACHE_TYPE: &str = "histogram";
/// Cache record type used for disposable manual brush alpha rasters.
pub const MASK_RASTER_CACHE_TYPE: &str = "mask_raster";

/// Singleton row id for library-wide export defaults.
pub const DEFAULT_EXPORT_SETTINGS_ID: &str = "default";
/// Built-in conservative JPEG sRGB preset id.
pub const DEFAULT_EXPORT_PRESET_ID: &str = "jpeg-srgb-90";

/// Stable action payload schema marker for undo/history records.
pub const ACTION_SCHEMA: &str = "silica.action";

/// Stable action payload version for Phase 16 history records.
pub const ACTION_VERSION: i64 = 1;

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
    pub originals_unchanged: bool,
    pub candidates: Vec<ImportCandidate>,
    pub issues: Vec<ImportIssue>,
}

/// Options for folder import scanning.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FolderImportOptions {
    pub recursive: bool,
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

/// Read-only status for one disposable cache directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheDirectoryStatus {
    pub name: String,
    pub path: PathBuf,
    pub exists: bool,
    pub byte_size: u64,
    pub file_count: u64,
}

/// Read-only status for all disposable cache directories in one library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStatusSummary {
    pub library_root_path: PathBuf,
    pub directories: Vec<CacheDirectoryStatus>,
    pub total_bytes: u64,
    pub cache_record_count: usize,
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

/// Catalog export row used by recent export views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentExportRecord {
    pub id: String,
    pub photo_id: String,
    pub output_path: String,
    pub export_settings_json: String,
    pub created_at: String,
}

/// Export-owned settings that are separate from edit graph state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSettings {
    pub format: String,
    pub color_profile: String,
    pub quality: u8,
    pub metadata_policy: String,
}

impl ExportSettings {
    pub fn jpeg_srgb_default() -> Self {
        Self {
            format: "jpeg".to_string(),
            color_profile: "srgb".to_string(),
            quality: 90,
            metadata_policy: "minimal".to_string(),
        }
    }
}

/// Named export preset row stored in the catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportPreset {
    pub id: String,
    pub name: String,
    pub settings: ExportSettings,
}

/// Library export settings state shown by the desktop export UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportSettingsCatalog {
    pub default_preset_id: Option<String>,
    pub default_settings: ExportSettings,
    pub presets: Vec<ExportPreset>,
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

/// Current catalog-side sync state for one library-local sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoSidecarStatus {
    pub photo_id: String,
    pub sidecar_path: Option<String>,
    pub last_written_at: Option<String>,
    pub conflict_state: String,
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
pub struct HistoryCommandResult {
    pub photo_id: String,
    pub command: String,
    pub applied: bool,
    pub action_kind: Option<String>,
    pub history_id: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditGraphCommitResult {
    pub commits: Vec<BatchEditGraphCommit>,
    pub skipped_photo_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditGraphCommit {
    pub photo_id: String,
    pub edit_state_id: String,
    pub history_id: String,
    pub sequence: i64,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoHistoryPanel {
    pub photo_id: String,
    pub items: Vec<PhotoHistoryItem>,
    pub can_undo: bool,
    pub can_redo: bool,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoHistoryItem {
    pub history_id: String,
    pub photo_id: String,
    pub sequence: i64,
    pub action_kind: String,
    pub label: String,
    pub history_state: String,
    pub can_undo: bool,
    pub can_redo: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewActionLogEntry {
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub action_type: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub side_effect_category: String,
    pub evidence_ref: Option<String>,
    pub payload_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionLogEntry {
    pub id: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub action_type: String,
    pub subject_type: Option<String>,
    pub subject_id: Option<String>,
    pub side_effect_category: String,
    pub evidence_ref: Option<String>,
    pub payload_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAiResult {
    pub photo_id: String,
    pub task_type: String,
    pub model_id: String,
    pub permission_id: String,
    pub output_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiResult {
    pub id: String,
    pub photo_id: String,
    pub task_type: String,
    pub model_id: String,
    pub result_json: String,
    pub approved: bool,
    pub created_at: String,
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
    CacheValidation(String),
    SidecarValidation(String),
    BackupValidation(String),
    HistoryValidation(String),
    ActionLogValidation(String),
    AiResultValidation(String),
    ExportSettingsValidation(String),
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
            Self::CacheValidation(message) => {
                write!(formatter, "cache validation error: {message}")
            }
            Self::SidecarValidation(message) => {
                write!(formatter, "sidecar validation error: {message}")
            }
            Self::BackupValidation(message) => {
                write!(formatter, "backup validation error: {message}")
            }
            Self::HistoryValidation(message) => {
                write!(formatter, "history validation error: {message}")
            }
            Self::ActionLogValidation(message) => {
                write!(formatter, "action log validation error: {message}")
            }
            Self::AiResultValidation(message) => {
                write!(formatter, "AI result validation error: {message}")
            }
            Self::ExportSettingsValidation(message) => {
                write!(formatter, "export settings validation error: {message}")
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
            | Self::CacheValidation(_)
            | Self::SidecarValidation(_)
            | Self::BackupValidation(_)
            | Self::HistoryValidation(_)
            | Self::ActionLogValidation(_)
            | Self::AiResultValidation(_)
            | Self::ExportSettingsValidation(_) => None,
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
