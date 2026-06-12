//! Core coordination boundary for SilicaRAW.
//!
//! Phase 4.2 starts the local library command surface.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-core";

pub use silica_decode::RawFullResolutionExportSourceError;
pub use silica_decode::RawPreviewArtifactError;
pub use silica_storage::CatalogRebuildDryRunAction;
pub use silica_storage::CatalogRebuildDryRunEntry;
pub use silica_storage::CatalogRebuildDryRunIssue;
pub use silica_storage::CatalogRebuildDryRunIssueKind;
pub use silica_storage::CatalogRebuildDryRunReport;
pub use silica_storage::CatalogRebuildFlagSource;
pub use silica_storage::FolderImportOptions;
pub use silica_storage::HistoryCommandResult;
pub use silica_storage::ImportIssue;
pub use silica_storage::ImportIssueKind;
pub use silica_storage::LibraryPhotoGridItem;
pub use silica_storage::LibraryQueryFileType;
pub use silica_storage::LibraryQueryFilters;
pub use silica_storage::LibraryQueryMetadataFilter;
pub use silica_storage::LibraryQueryOrderField;
pub use silica_storage::LibraryQueryPage;
pub use silica_storage::LibraryQueryRequest;
pub use silica_storage::LibraryQuerySort;
pub use silica_storage::PhotoFlags;
pub use silica_storage::PhotoMetadata;
pub use silica_storage::PhotoMetadataField;
pub use silica_storage::PhotoMetadataFieldState;
pub use silica_storage::SidecarWriteResult;
pub use silica_storage::ValidatedSidecar;

pub fn plan_product_raw_decode(
    source_path: impl AsRef<str>,
) -> silica_decode::ProductRawDecodePlan {
    silica_decode::plan_product_raw_decode(source_path)
}

pub fn plan_product_raw_decode_from_probe(
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
) -> silica_decode::ProductRawDecodePlan {
    silica_decode::plan_product_raw_decode_from_probe(fixture_class, probe)
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedImageViewerHandoffPlan {
    pub decoded: silica_decode::DecodedImageHandoff,
    pub viewer_input: silica_render::ViewerPreviewInput,
}

impl DecodedImageViewerHandoffPlan {
    pub fn writes_catalog_state(&self) -> bool {
        false
    }

    pub fn writes_sidecars(&self) -> bool {
        false
    }

    pub fn writes_originals(&self) -> bool {
        false
    }

    pub fn writes_exports(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPreviewArtifactSession {
    pub handoff: DecodedImageViewerHandoffPlan,
    pub output_path: PathBuf,
    pub artifact_path: Option<PathBuf>,
    pub cache_record: Option<silica_storage::CacheRecord>,
    pub bytes_written: Option<u64>,
    pub original_hash_unchanged: Option<bool>,
}

pub fn plan_decoded_image_viewer_handoff(
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
    cache_key: impl Into<String>,
) -> DecodedImageViewerHandoffPlan {
    let decoded =
        silica_decode::plan_decoded_image_handoff_from_raw_probe(fixture_class, probe, cache_key);
    let viewer_input = silica_render::ViewerPreviewInput::from_decoded_handoff(&decoded);

    DecodedImageViewerHandoffPlan {
        decoded,
        viewer_input,
    }
}

pub fn write_raw_preview_artifact_for_probe(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
) -> Result<RawPreviewArtifactSession, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let photo_id = photo_id.as_ref();
    let output_path = raw_preview_artifact_output_path(library_root_path, photo_id);
    let cache_key = raw_preview_artifact_cache_key(photo_id, probe);
    let result =
        silica_decode::write_raw_preview_artifact(silica_decode::RawPreviewArtifactRequest {
            fixture_class: fixture_class.as_ref().to_string(),
            probe: probe.clone(),
            cache_key: cache_key.clone(),
            output_path: output_path.clone(),
            max_edge: LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE,
        })?;
    let viewer_input = silica_render::ViewerPreviewInput::from_decoded_handoff(&result.handoff);
    let handoff = DecodedImageViewerHandoffPlan {
        decoded: result.handoff,
        viewer_input,
    };
    let cache_record = match (&result.artifact_path, result.bytes_written) {
        (Some(path), Some(bytes_written)) => {
            let byte_size = i64::try_from(bytes_written).unwrap_or(i64::MAX);
            Some(silica_storage::record_preview_cache(
                library_root_path,
                photo_id,
                cache_key,
                path,
                byte_size,
            )?)
        }
        _ => None,
    };

    Ok(RawPreviewArtifactSession {
        handoff,
        output_path,
        artifact_path: result.artifact_path,
        cache_record,
        bytes_written: result.bytes_written,
        original_hash_unchanged: result.original_hash_unchanged,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn plan_exposure_contrast_metal_draft(
    photo_id: impl Into<String>,
    source_path: impl Into<String>,
    viewer_input: silica_render::ViewerPreviewInput,
    viewport: silica_render::ViewerPreviewViewport,
    request_id: silica_render::ViewerPreviewRenderRequestId,
    edit_graph_revision: u64,
    exposure: f64,
    contrast: f64,
) -> Result<silica_render::ViewerPreviewRenderRequest, CoreError> {
    let photo_id = photo_id.into();
    let source_path = source_path.into();
    let graph = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: photo_id.clone(),
            path: source_path.clone(),
            file_size: 0,
            modified_at: None,
            partial_hash: None,
            full_hash: None,
        },
        current_timestamp_string(),
    );
    let edited = silica_edit::apply_exposure_contrast(
        &graph,
        exposure,
        contrast,
        current_timestamp_string(),
    )?;
    let exposure = edited.basic.exposure.as_f64().unwrap_or(exposure);
    let contrast = edited.basic.contrast.as_f64().unwrap_or(contrast);

    Ok(silica_render::ViewerPreviewRenderRequest::new(
        request_id,
        photo_id,
        source_path,
        viewport,
        viewer_input,
        edit_graph_revision,
    )
    .with_exposure_contrast_draft(exposure, contrast))
}

const LOCAL_ALPHA_JPEG_QUALITY: u8 = 90;
const LOCAL_ALPHA_THUMBNAIL_QUALITY: u8 = 82;
const LOCAL_ALPHA_THUMBNAIL_MAX_EDGE: u32 = 320;
const LOCAL_ALPHA_LOUPE_PREVIEW_QUALITY: u8 = 88;
const LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE: u32 = 2048;
const LOCAL_ALPHA_DEVELOP_PREVIEW_QUALITY: u8 = 86;

/// App-level desktop session schema identifier.
pub const APP_SESSION_SCHEMA: &str = "silica.desktop_session";
/// App-level desktop session schema version.
pub const APP_SESSION_VERSION: i64 = 1;
/// Default Library grid thumbnail size preference in pixels.
pub const DEFAULT_APP_SESSION_THUMBNAIL_SIZE: u16 = 168;
/// Minimum accepted Library grid thumbnail size preference in pixels.
pub const MIN_APP_SESSION_THUMBNAIL_SIZE: u16 = 132;
/// Maximum accepted Library grid thumbnail size preference in pixels.
pub const MAX_APP_SESSION_THUMBNAIL_SIZE: u16 = 220;
/// Maximum number of recent libraries retained in app-level session state.
pub const APP_SESSION_RECENTS_LIMIT: usize = 10;

/// Last active desktop mode persisted outside every library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSessionMode {
    Library,
    Develop,
    Export,
}

/// Library grid sort order persisted in app-level session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLibrarySort {
    ImportedAtDesc,
    FileNameAsc,
    RatingDesc,
}

/// Optional file-type filter persisted in app-level session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppFileTypeFilter {
    Jpeg,
    Raw,
    Unsupported,
}

/// Optional metadata-backed filter persisted in app-level session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMetadataFilter {
    HasDimensions,
}

/// Library grid filters persisted in app-level session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionFilters {
    pub min_rating: Option<u8>,
    pub picked: Option<bool>,
    pub rejected: Option<bool>,
    pub file_type: Option<AppFileTypeFilter>,
    pub metadata: Option<AppMetadataFilter>,
    pub search: String,
}

impl Default for AppSessionFilters {
    fn default() -> Self {
        Self {
            min_rating: None,
            picked: None,
            rejected: None,
            file_type: None,
            metadata: None,
            search: String::new(),
        }
    }
}

/// Workspace layout preferences persisted in app-level session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLayoutPreferences {
    pub sidebar_collapsed: bool,
    pub inspector_collapsed: bool,
    pub filmstrip_visible: bool,
    pub thumbnail_size: u16,
    pub sort: AppLibrarySort,
    pub filters: AppSessionFilters,
}

impl Default for AppLayoutPreferences {
    fn default() -> Self {
        Self {
            sidebar_collapsed: false,
            inspector_collapsed: false,
            filmstrip_visible: true,
            thumbnail_size: DEFAULT_APP_SESSION_THUMBNAIL_SIZE,
            sort: AppLibrarySort::ImportedAtDesc,
            filters: AppSessionFilters::default(),
        }
    }
}

/// One recent library entry persisted after successful create/open only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRecentLibrary {
    pub root_path: PathBuf,
    pub display_name: String,
    pub last_opened_at: String,
}

/// Per-library session state keyed by library root path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPerLibrarySession {
    pub selected_photo_id: Option<String>,
    pub last_mode: AppSessionMode,
    pub last_opened_at: String,
}

impl Default for AppPerLibrarySession {
    fn default() -> Self {
        Self {
            selected_photo_id: None,
            last_mode: AppSessionMode::Library,
            last_opened_at: String::new(),
        }
    }
}

/// Versioned app-level desktop session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSession {
    pub schema: String,
    pub version: i64,
    pub last_library_root_path: Option<PathBuf>,
    pub last_mode: AppSessionMode,
    pub recents: Vec<AppRecentLibrary>,
    pub layout: AppLayoutPreferences,
    pub per_library: BTreeMap<String, AppPerLibrarySession>,
}

impl Default for AppSession {
    fn default() -> Self {
        Self {
            schema: APP_SESSION_SCHEMA.to_string(),
            version: APP_SESSION_VERSION,
            last_library_root_path: None,
            last_mode: AppSessionMode::Library,
            recents: Vec::new(),
            layout: AppLayoutPreferences::default(),
            per_library: BTreeMap::new(),
        }
    }
}

/// Non-fatal app-session load warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppSessionWarning {
    Missing,
    Corrupt,
    UnsupportedVersion,
    InvalidValues,
}

/// Result of loading app-level desktop session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionLoadResult {
    pub session: AppSession,
    pub warnings: Vec<AppSessionWarning>,
}

/// Result of atomically writing app-level desktop session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionWriteResult {
    pub session_path: PathBuf,
    pub bytes_written: u64,
}

/// Relaunch restore state after validating the last app-session library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSessionRestoreStatus {
    NoLastLibrary,
    MissingLibrary,
    MissingCatalog,
    InvalidCatalog,
    Restored,
}

/// Selected-photo restore outcome for the last library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSessionSelectedPhotoStatus {
    None,
    Missing,
    Restored,
}

/// Relaunch restore plan that does not create, migrate, import, or repair libraries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionRestorePlan {
    pub session: AppSession,
    pub warnings: Vec<AppSessionWarning>,
    pub status: AppSessionRestoreStatus,
    pub library_root_path: Option<PathBuf>,
    pub catalog_path: Option<PathBuf>,
    pub schema_version: Option<i64>,
    pub selected_photo_id: Option<String>,
    pub selected_photo_status: AppSessionSelectedPhotoStatus,
    pub requested_mode: AppSessionMode,
    pub resolved_mode: AppSessionMode,
}

/// Local library session returned by core commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrarySession {
    pub root_path: PathBuf,
    pub catalog_path: PathBuf,
    pub schema_version: i64,
}

impl LibrarySession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Library: {}\nCatalog: {}\nSchema: {}",
            self.root_path.display(),
            self.catalog_path.display(),
            self.schema_version
        )
    }
}

impl From<silica_storage::LocalLibrary> for LibrarySession {
    fn from(library: silica_storage::LocalLibrary) -> Self {
        Self {
            root_path: library.root_path,
            catalog_path: library.catalog_path,
            schema_version: library.schema_version,
        }
    }
}

/// Preview state exposed by the core command boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoPreviewStatus {
    Ready,
    BlockedByDecode,
    Unsupported,
}

/// Preview session returned for one catalog photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoPreviewSession {
    pub photo_id: String,
    pub file_name: String,
    pub source_path: String,
    pub preview_bytes: Option<Vec<u8>>,
    pub status: PhotoPreviewStatus,
    pub message: String,
}

impl PhotoPreviewSession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nFile: {}\nPreview: {:?}\nSource: {}\nMessage: {}",
            self.photo_id, self.file_name, self.status, self.source_path, self.message
        )
    }
}

/// Draft preview request returned while an exposure/contrast slider is moving.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoEditPreviewSession {
    pub photo_id: String,
    pub source_path: String,
    pub develop_preview_bytes: Option<Vec<u8>>,
    pub status: PhotoPreviewStatus,
    pub exposure: f64,
    pub contrast: f64,
    pub message: String,
}

impl PhotoEditPreviewSession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nPreview: {:?}\nSource: {}\nExposure: {}\nContrast: {}\nMessage: {}",
            self.photo_id,
            self.status,
            self.source_path,
            self.exposure,
            self.contrast,
            self.message
        )
    }
}

/// Persisted exposure/contrast edit returned on commit/release.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoEditCommit {
    pub photo_id: String,
    pub exposure: f64,
    pub contrast: f64,
    pub persisted: bool,
    pub message: String,
}

impl PhotoEditCommit {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nExposure: {}\nContrast: {}\nPersisted: {}\nMessage: {}",
            self.photo_id, self.exposure, self.contrast, self.persisted, self.message
        )
    }
}

/// Current committed exposure/contrast edit state for a catalog photo.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoEditState {
    pub photo_id: String,
    pub exposure: f64,
    pub contrast: f64,
    pub persisted: bool,
    pub message: String,
}

impl PhotoEditState {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nExposure: {}\nContrast: {}\nPersisted: {}\nMessage: {}",
            self.photo_id, self.exposure, self.contrast, self.persisted, self.message
        )
    }
}

/// Completed JPEG export returned through the core boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoExportSession {
    pub photo_id: String,
    pub source_path: String,
    pub output_path: PathBuf,
    pub format: String,
    pub color_profile: String,
    pub bytes_written: u64,
    pub source_sha256: Option<String>,
    pub output_sha256: String,
    pub icc_profile_embedded: bool,
    pub icc_profile_sha256: String,
    pub decoder_backend: Option<String>,
    pub input_profile: Option<String>,
    pub working_space: Option<String>,
    pub export_record_id: String,
    pub message: String,
}

/// JPEG output color profile accepted by the core export boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoExportColorProfile {
    Srgb,
    DisplayP3,
}

impl PhotoExportSession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nExport: {}\nFormat: {}\nColor: {}\nBytes: {}\nMessage: {}",
            self.photo_id,
            self.output_path.display(),
            self.format,
            self.color_profile,
            self.bytes_written,
            self.message
        )
    }
}

/// Summary returned when disposable library caches are cleared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryCacheClearSession {
    pub cleared_directories: Vec<String>,
    pub recreated_directories: Vec<String>,
    pub removed_cache_records: usize,
    pub message: String,
}

impl LibraryCacheClearSession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Cleared: {}\nRecreated: {}\nCache records removed: {}\nMessage: {}",
            self.cleared_directories.join(", "),
            self.recreated_directories.join(", "),
            self.removed_cache_records,
            self.message
        )
    }
}

/// Errors returned by core command APIs.
#[derive(Debug)]
pub enum CoreError {
    Storage(silica_storage::LibraryStorageError),
    Decode(silica_decode::RawPreviewArtifactError),
    RawExport(silica_decode::RawFullResolutionExportSourceError),
    EditGraph(silica_edit::EditGraphValidationError),
    Export(silica_export::ExportError),
    ExportBlocked(String),
    AppSession(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "{error}"),
            Self::Decode(error) => write!(formatter, "{error}"),
            Self::RawExport(error) => write!(formatter, "{error}"),
            Self::EditGraph(error) => write!(formatter, "{error}"),
            Self::Export(error) => write!(formatter, "{error}"),
            Self::ExportBlocked(message) => write!(formatter, "export blocked: {message}"),
            Self::AppSession(message) => write!(formatter, "app session error: {message}"),
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::Decode(error) => Some(error),
            Self::RawExport(error) => Some(error),
            Self::EditGraph(error) => Some(error),
            Self::Export(error) => Some(error),
            Self::ExportBlocked(_) => None,
            Self::AppSession(_) => None,
        }
    }
}

impl From<silica_storage::LibraryStorageError> for CoreError {
    fn from(error: silica_storage::LibraryStorageError) -> Self {
        Self::Storage(error)
    }
}

impl From<silica_decode::RawPreviewArtifactError> for CoreError {
    fn from(error: silica_decode::RawPreviewArtifactError) -> Self {
        Self::Decode(error)
    }
}

impl From<silica_decode::RawFullResolutionExportSourceError> for CoreError {
    fn from(error: silica_decode::RawFullResolutionExportSourceError) -> Self {
        Self::RawExport(error)
    }
}

impl From<silica_edit::EditGraphValidationError> for CoreError {
    fn from(error: silica_edit::EditGraphValidationError) -> Self {
        Self::EditGraph(error)
    }
}

impl From<silica_export::ExportError> for CoreError {
    fn from(error: silica_export::ExportError) -> Self {
        Self::Export(error)
    }
}

/// Load app-level desktop session state from a caller-provided path.
pub fn load_app_session(session_path: impl AsRef<Path>) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    if !session_path.exists() {
        return Ok(AppSessionLoadResult {
            session: AppSession::default(),
            warnings: vec![AppSessionWarning::Missing],
        });
    }

    let bytes = std::fs::read(session_path).map_err(|error| {
        CoreError::AppSession(format!("read {}: {error}", session_path.display()))
    })?;
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => {
            return Ok(AppSessionLoadResult {
                session: AppSession::default(),
                warnings: vec![AppSessionWarning::Corrupt],
            });
        }
    };
    let Some(object) = value.as_object() else {
        return Ok(AppSessionLoadResult {
            session: AppSession::default(),
            warnings: vec![AppSessionWarning::Corrupt],
        });
    };

    if object.get("schema").and_then(serde_json::Value::as_str) != Some(APP_SESSION_SCHEMA)
        || object.get("version").and_then(serde_json::Value::as_i64) != Some(APP_SESSION_VERSION)
    {
        return Ok(AppSessionLoadResult {
            session: AppSession::default(),
            warnings: vec![AppSessionWarning::UnsupportedVersion],
        });
    }

    let mut invalid_values = false;
    let session = AppSession {
        schema: APP_SESSION_SCHEMA.to_string(),
        version: APP_SESSION_VERSION,
        last_library_root_path: parse_optional_path(
            object.get("last_library_root_path"),
            &mut invalid_values,
        ),
        last_mode: parse_app_session_mode(object.get("last_mode"), &mut invalid_values),
        recents: parse_app_session_recents(object.get("recents"), &mut invalid_values),
        layout: parse_app_layout(object.get("layout"), &mut invalid_values),
        per_library: parse_app_per_library(object.get("per_library"), &mut invalid_values),
    };

    let warnings = if invalid_values {
        vec![AppSessionWarning::InvalidValues]
    } else {
        Vec::new()
    };

    Ok(AppSessionLoadResult { session, warnings })
}

/// Atomically write app-level desktop session state to a caller-provided path.
pub fn write_app_session(
    session_path: impl AsRef<Path>,
    session: &AppSession,
) -> Result<AppSessionWriteResult, CoreError> {
    let session_path = session_path.as_ref();
    if let Some(parent) = session_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            CoreError::AppSession(format!("create {}: {error}", parent.display()))
        })?;
    }

    let bytes = serde_json::to_vec_pretty(&app_session_to_json(session))
        .map_err(|error| CoreError::AppSession(format!("serialize app session: {error}")))?;
    let temp_path = session_path.with_extension("tmp");
    std::fs::write(&temp_path, &bytes).map_err(|error| {
        CoreError::AppSession(format!("write {}: {error}", temp_path.display()))
    })?;
    std::fs::rename(&temp_path, session_path).map_err(|error| {
        CoreError::AppSession(format!(
            "rename {} to {}: {error}",
            temp_path.display(),
            session_path.display()
        ))
    })?;

    Ok(AppSessionWriteResult {
        session_path: session_path.to_path_buf(),
        bytes_written: bytes.len() as u64,
    })
}

/// Return the documented default workspace layout preferences.
pub fn default_app_layout_preferences() -> AppLayoutPreferences {
    AppLayoutPreferences::default()
}

/// Record a successful library create/open in app-level desktop session state.
pub fn record_app_session_recent_library(
    session_path: impl AsRef<Path>,
    library: &LibrarySession,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    let recent_key = app_session_recent_key(&library.root_path);
    let opened_at = current_timestamp_string();
    session.last_library_root_path = Some(library.root_path.clone());
    session
        .recents
        .retain(|recent| app_session_recent_key(&recent.root_path) != recent_key);
    session.recents.insert(
        0,
        AppRecentLibrary {
            root_path: library.root_path.clone(),
            display_name: app_session_library_display_name(&library.root_path),
            last_opened_at: opened_at,
        },
    );
    session.recents.truncate(APP_SESSION_RECENTS_LIMIT);

    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Reset only workspace layout preferences in app-level desktop session state.
pub fn reset_app_session_layout(
    session_path: impl AsRef<Path>,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.layout = default_app_layout_preferences();
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Record workspace layout preferences in app-level desktop session state.
pub fn record_app_session_layout(
    session_path: impl AsRef<Path>,
    layout: AppLayoutPreferences,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.layout = layout;
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Plan app relaunch restore from app-session state without opening a writable library.
pub fn plan_app_session_restore(
    session_path: impl AsRef<Path>,
) -> Result<AppSessionRestorePlan, CoreError> {
    let loaded = load_app_session(session_path)?;
    let requested_mode = loaded.session.last_mode;
    let Some(last_library_root_path) = loaded.session.last_library_root_path.clone() else {
        return Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::NoLastLibrary,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        });
    };

    match silica_storage::inspect_local_library_for_restore(&last_library_root_path) {
        Ok(library) => {
            let status = if library.schema_version == silica_storage::CURRENT_SCHEMA_VERSION {
                AppSessionRestoreStatus::Restored
            } else {
                AppSessionRestoreStatus::InvalidCatalog
            };
            let restored = status == AppSessionRestoreStatus::Restored;
            let per_library = restored
                .then(|| app_session_recent_key(&library.root_path))
                .and_then(|key| loaded.session.per_library.get(&key));
            let requested_mode = per_library
                .map(|session| session.last_mode)
                .unwrap_or(requested_mode);
            let selected_candidate =
                per_library.and_then(|session| session.selected_photo_id.clone());
            let (selected_photo_id, selected_photo_status) =
                if let Some(photo_id) = selected_candidate {
                    match silica_storage::catalog_photo_exists_for_restore(
                        &library.root_path,
                        &photo_id,
                    ) {
                        Ok(true) => (Some(photo_id), AppSessionSelectedPhotoStatus::Restored),
                        Ok(false) => (None, AppSessionSelectedPhotoStatus::Missing),
                        Err(_) => {
                            return Ok(AppSessionRestorePlan {
                                session: loaded.session,
                                warnings: loaded.warnings,
                                status: AppSessionRestoreStatus::InvalidCatalog,
                                library_root_path: None,
                                catalog_path: None,
                                schema_version: None,
                                selected_photo_id: None,
                                selected_photo_status: AppSessionSelectedPhotoStatus::None,
                                requested_mode,
                                resolved_mode: AppSessionMode::Library,
                            })
                        }
                    }
                } else {
                    (None, AppSessionSelectedPhotoStatus::None)
                };
            let resolved_mode = if requested_mode == AppSessionMode::Library
                || selected_photo_status == AppSessionSelectedPhotoStatus::Restored
            {
                requested_mode
            } else {
                AppSessionMode::Library
            };
            Ok(AppSessionRestorePlan {
                session: loaded.session,
                warnings: loaded.warnings,
                status,
                library_root_path: restored.then_some(library.root_path),
                catalog_path: restored.then_some(library.catalog_path),
                schema_version: restored.then_some(library.schema_version),
                selected_photo_id,
                selected_photo_status,
                requested_mode,
                resolved_mode,
            })
        }
        Err(silica_storage::LibraryStorageError::NotDirectory(_)) => Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::MissingLibrary,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        }),
        Err(silica_storage::LibraryStorageError::MissingCatalog(_)) => Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::MissingCatalog,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        }),
        Err(_) => Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::InvalidCatalog,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        }),
    }
}

/// Record the active library selection and mode in app-level desktop session state.
pub fn record_app_session_library_state(
    session_path: impl AsRef<Path>,
    library_root_path: impl AsRef<Path>,
    selected_photo_id: Option<String>,
    mode: AppSessionMode,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    let library_root_path = library_root_path.as_ref().to_path_buf();
    let key = app_session_recent_key(&library_root_path);
    let opened_at = current_timestamp_string();
    session.last_library_root_path = Some(library_root_path);
    session.last_mode = mode;
    session.per_library.insert(
        key,
        AppPerLibrarySession {
            selected_photo_id,
            last_mode: mode,
            last_opened_at: opened_at,
        },
    );

    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Create a local SilicaRAW library through the core command boundary.
pub fn create_library(root_path: impl AsRef<Path>) -> Result<LibrarySession, CoreError> {
    silica_storage::create_local_library(root_path)
        .map(LibrarySession::from)
        .map_err(CoreError::from)
}

/// Open a local SilicaRAW library through the core command boundary.
pub fn open_library(root_path: impl AsRef<Path>) -> Result<LibrarySession, CoreError> {
    silica_storage::open_local_library(root_path)
        .map(LibrarySession::from)
        .map_err(CoreError::from)
}

/// Scan a folder by reference through the core command boundary.
pub fn import_folder(
    library_root_path: impl AsRef<Path>,
    folder_path: impl AsRef<Path>,
) -> Result<silica_storage::FolderImportSummary, CoreError> {
    import_folder_with_options(
        library_root_path,
        folder_path,
        FolderImportOptions::default(),
    )
}

/// Scan a folder by reference through the core command boundary.
pub fn import_folder_with_options(
    library_root_path: impl AsRef<Path>,
    folder_path: impl AsRef<Path>,
    options: FolderImportOptions,
) -> Result<silica_storage::FolderImportSummary, CoreError> {
    let library_root_path = library_root_path.as_ref().to_path_buf();
    let summary =
        silica_storage::import_folder_with_options(&library_root_path, folder_path, options)?;
    persist_imported_photo_metadata(&library_root_path, &summary)?;
    Ok(summary)
}

/// Return metadata extraction policy without running any backfill work.
pub fn metadata_extraction_policy_for_path(
    path: impl AsRef<Path>,
) -> silica_storage::MetadataExtractionPolicy {
    silica_storage::metadata_extraction_policy_for_path(path.as_ref())
}

fn persist_imported_photo_metadata(
    library_root_path: &Path,
    summary: &silica_storage::FolderImportSummary,
) -> Result<(), CoreError> {
    for candidate in summary
        .candidates
        .iter()
        .filter(|candidate| !candidate.unsupported)
    {
        let path = PathBuf::from(&candidate.path);
        let metadata = metadata_update_for_imported_path(&path);
        silica_storage::upsert_photo_metadata_by_path(library_root_path, &path, metadata)?;
    }
    Ok(())
}

fn metadata_update_for_imported_path(path: &Path) -> silica_storage::PhotoMetadataUpdate {
    let mut metadata = silica_storage::PhotoMetadataUpdate::unavailable();
    let policy = metadata_extraction_policy_for_path(path);
    if policy.dimension_source == silica_storage::MetadataDimensionSource::ExistingRasterPath {
        if let Ok(dimensions) = silica_export::read_raster_dimensions(path.to_path_buf()) {
            metadata.width = Some(i64::from(dimensions.width));
            metadata.height = Some(i64::from(dimensions.height));
        }
    }
    metadata
}

/// List imported catalog photos as JSON for the desktop Library grid.
pub fn list_library_photos_json(library_root_path: impl AsRef<Path>) -> Result<String, CoreError> {
    let photos = list_library_photos(library_root_path)?;
    let rows = photos
        .into_iter()
        .map(|photo| {
            serde_json::json!({
                "photoId": photo.photo_id,
                "fileName": photo.file_name,
                "path": photo.path,
                "fileType": photo.file_type,
                "thumbnailPath": photo.thumbnail_path,
                "missing": photo.missing,
                "unsupported": photo.unsupported,
                "rating": photo.rating,
                "picked": photo.picked,
                "rejected": photo.rejected,
                "colorLabel": photo.color_label,
            })
        })
        .collect::<Vec<_>>();

    Ok(serde_json::Value::Array(rows).to_string())
}

/// List imported catalog photos through the typed core command boundary.
pub fn list_library_photos(
    library_root_path: impl AsRef<Path>,
) -> Result<Vec<silica_storage::LibraryPhotoGridItem>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    ensure_jpeg_thumbnail_cache(library_root_path)?;
    silica_storage::list_library_photos(library_root_path).map_err(CoreError::from)
}

/// Query imported catalog photos by bounded page without cache hydration.
pub fn query_library_photos(
    library_root_path: impl AsRef<Path>,
    request: silica_storage::LibraryQueryRequest,
) -> Result<silica_storage::LibraryQueryPage<silica_storage::LibraryPhotoGridItem>, CoreError> {
    silica_storage::query_library_photos(library_root_path, request).map_err(CoreError::from)
}

/// Query one catalog page and hydrate JPEG thumbnails only for rows in that page.
pub fn query_library_photos_with_thumbnail_hydration(
    library_root_path: impl AsRef<Path>,
    request: silica_storage::LibraryQueryRequest,
) -> Result<silica_storage::LibraryQueryPage<silica_storage::LibraryPhotoGridItem>, CoreError> {
    let library_root_path = library_root_path.as_ref().to_path_buf();
    let page = query_library_photos(&library_root_path, request.clone())?;
    ensure_jpeg_thumbnail_cache_for_photos(&library_root_path, &page.items)?;
    query_library_photos(&library_root_path, request)
}

/// Persist photo culling and label flags through the core command boundary.
pub fn set_photo_flags(
    library_root_path: impl AsRef<Path>,
    photo_id: impl Into<String>,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
) -> Result<silica_storage::PhotoFlags, CoreError> {
    silica_storage::set_photo_flags(
        library_root_path,
        photo_id,
        rating,
        picked,
        rejected,
        color_label,
    )
    .map_err(CoreError::from)
}

/// Read photo culling and label flags through the core command boundary.
pub fn get_photo_flags(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<silica_storage::PhotoFlags>, CoreError> {
    silica_storage::get_photo_flags(library_root_path, photo_id).map_err(CoreError::from)
}

/// Undo the latest undoable history action for one photo through the core boundary.
pub fn undo_last_history_action(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<silica_storage::HistoryCommandResult, CoreError> {
    silica_storage::undo_last_history_action(library_root_path, photo_id).map_err(CoreError::from)
}

/// Redo the next redoable history action for one photo through the core boundary.
pub fn redo_last_history_action(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<silica_storage::HistoryCommandResult, CoreError> {
    silica_storage::redo_last_history_action(library_root_path, photo_id).map_err(CoreError::from)
}

/// Read stored photo metadata through the core command boundary.
pub fn get_photo_metadata(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<silica_storage::PhotoMetadata>, CoreError> {
    silica_storage::get_photo_metadata(library_root_path, photo_id).map_err(CoreError::from)
}

/// Write a library-local sidecar through the core command boundary.
pub fn write_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<Option<SidecarWriteResult>, CoreError> {
    match silica_storage::write_photo_sidecar(library_root_path, photo_id, app_version) {
        Ok(result) => Ok(Some(result)),
        Err(silica_storage::LibraryStorageError::MissingPhoto(_)) => Ok(None),
        Err(error) => Err(CoreError::from(error)),
    }
}

/// Read a validated library-local sidecar through the core command boundary.
pub fn read_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<ValidatedSidecar>, CoreError> {
    silica_storage::read_photo_sidecar(library_root_path, photo_id).map_err(CoreError::from)
}

/// Dry-run catalog rebuild from library-local sidecars through the core boundary.
pub fn dry_run_catalog_rebuild_from_sidecars(
    library_root_path: impl AsRef<Path>,
) -> Result<CatalogRebuildDryRunReport, CoreError> {
    silica_storage::dry_run_catalog_rebuild_from_sidecars(library_root_path)
        .map_err(CoreError::from)
}

/// Build the preview session for one catalog photo.
pub fn open_photo_preview(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let (photo_id, file_name, render_plan) = match preview_render_plan(library_root_path, photo_id)?
    {
        Some(plan) => plan,
        None => return Ok(None),
    };
    let status = preview_status_from_render(render_plan.status);
    let preview_bytes = if status == PhotoPreviewStatus::Ready {
        ensure_jpeg_loupe_preview_cache(library_root_path, &photo_id, &render_plan.source_path)?
    } else {
        None
    };

    Ok(Some(PhotoPreviewSession {
        photo_id,
        file_name,
        source_path: render_plan.source_path,
        preview_bytes,
        status,
        message: render_plan.message,
    }))
}

/// Build a draft exposure/contrast preview request without writing the catalog.
pub fn preview_exposure_contrast_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    exposure: f64,
    contrast: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_exposure_contrast(
        &graph,
        exposure,
        contrast,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_exposure_contrast_preview(
        render_plan,
        edited.basic.exposure.as_f64().unwrap_or(exposure),
        edited.basic.contrast.as_f64().unwrap_or(contrast),
    );
    let source_is_jpeg = is_jpeg_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_jpeg => {
            message = "JPEG/JPG Develop preview pixels are the only enabled local alpha path."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        message,
    }))
}

/// Persist an exposure/contrast edit on commit/release.
pub fn commit_exposure_contrast_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    exposure: f64,
    contrast: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_exposure_contrast(
        &graph,
        exposure,
        contrast,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id,
        exposure,
        contrast,
        persisted: true,
        message: "Exposure/contrast edit persisted on commit.".to_string(),
    }))
}

/// Read the current exposure/contrast edit state without mutating the catalog.
pub fn get_photo_edit_state(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditState>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if let Some(graph) = silica_storage::load_active_edit_graph(library_root_path, photo_id)? {
        return Ok(Some(PhotoEditState {
            photo_id: graph.source.photo_id,
            exposure: graph.basic.exposure.as_f64().unwrap_or(0.0),
            contrast: graph.basic.contrast.as_f64().unwrap_or(0.0),
            persisted: true,
            message: "Restored committed edit state.".to_string(),
        }));
    }

    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };

    Ok(Some(PhotoEditState {
        photo_id: graph.source.photo_id,
        exposure: graph.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: graph.basic.contrast.as_f64().unwrap_or(0.0),
        persisted: false,
        message: "Default clean edit state loaded.".to_string(),
    }))
}

/// Export one edited catalog photo as a JPEG sRGB file and record the export.
pub fn export_photo_jpeg_srgb(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_jpeg(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportColorProfile::Srgb,
    )
}

/// Export one edited catalog photo as a JPEG file with an explicit output color profile.
pub fn export_photo_jpeg(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    color_profile: PhotoExportColorProfile,
) -> Result<Option<PhotoExportSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let output_path = output_path.as_ref();
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    if render_plan.status != silica_render::PreviewRenderStatus::Ready {
        return Err(CoreError::ExportBlocked(render_plan.message));
    }

    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, &photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let exposure = graph.basic.exposure.as_f64().unwrap_or(0.0);
    let contrast = graph.basic.contrast.as_f64().unwrap_or(0.0);
    let render_request = silica_render::plan_jpeg_srgb_export(
        render_plan.source_path.clone(),
        output_path.display().to_string(),
        exposure,
        contrast,
        LOCAL_ALPHA_JPEG_QUALITY,
    );

    let export_result =
        silica_export::export_jpeg_with_color_profile(silica_export::JpegColorExportRequest {
            source_path: PathBuf::from(&render_request.source_path),
            output_path: output_path.to_path_buf(),
            exposure: render_request.exposure,
            contrast: render_request.contrast,
            quality: render_request.quality,
            color_profile: export_color_profile_to_export(color_profile),
        })?;
    let format = export_format_string(export_result.format).to_string();
    let exported_color_profile =
        export_color_profile_string(export_result.color_profile).to_string();
    let source_sha256 = export_result.source_sha256.clone();
    let output_sha256 = export_result.output_sha256.clone();
    let icc_profile_sha256 = export_result.icc_profile_sha256.clone();
    let settings_json = serde_json::json!({
        "format": format,
        "color_profile": exported_color_profile,
        "quality": render_request.quality,
        "exposure": render_request.exposure,
        "contrast": render_request.contrast,
        "source_path": render_request.source_path,
        "output_path": render_request.output_path,
        "source_sha256": source_sha256.clone(),
        "output_sha256": output_sha256.clone(),
        "icc_profile_embedded": export_result.icc_profile_embedded,
        "icc_profile_sha256": icc_profile_sha256.clone(),
        "profile_metadata_source": "silica-export",
    })
    .to_string();
    let export_record = silica_storage::record_export(
        library_root_path,
        &photo_id,
        &export_result.output_path,
        settings_json,
    )?;

    Ok(Some(PhotoExportSession {
        photo_id,
        source_path: render_plan.source_path,
        output_path: export_result.output_path,
        format,
        color_profile: exported_color_profile,
        bytes_written: export_result.bytes_written,
        source_sha256: Some(source_sha256),
        output_sha256,
        icc_profile_embedded: export_result.icc_profile_embedded,
        icc_profile_sha256,
        decoder_backend: None,
        input_profile: None,
        working_space: None,
        export_record_id: export_record.id,
        message: export_color_profile_message(color_profile).to_string(),
    }))
}

/// Export one fixture-backed RAW catalog photo as JPEG sRGB through a full-resolution source artifact.
pub fn export_raw_photo_jpeg_srgb_from_probe(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let output_path = output_path.as_ref();
    let candidate = match silica_storage::get_photo_preview_candidate(library_root_path, photo_id)?
    {
        Some(candidate) => candidate,
        None => return Ok(None),
    };
    let raw_source_path = PathBuf::from(&probe.source_path);
    if paths_match(&raw_source_path, output_path)? {
        return Err(CoreError::RawExport(
            silica_decode::RawFullResolutionExportSourceError::OutputMatchesSource(
                output_path.to_path_buf(),
            ),
        ));
    }
    if !paths_match(&PathBuf::from(&candidate.path), &raw_source_path)? {
        return Err(CoreError::ExportBlocked(
            "RAW export probe source does not match the catalog photo source.".to_string(),
        ));
    }

    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let exposure = graph.basic.exposure.as_f64().unwrap_or(0.0);
    let contrast = graph.basic.contrast.as_f64().unwrap_or(0.0);
    let source_artifact_path =
        raw_full_resolution_export_source_path(library_root_path, photo_id, probe);
    let source_artifact = silica_decode::write_raw_full_resolution_export_source(
        silica_decode::RawFullResolutionExportSourceRequest {
            fixture_class: fixture_class.as_ref().to_string(),
            probe: probe.clone(),
            output_path: source_artifact_path,
        },
    )?;
    let render_request = silica_render::plan_raw_derived_jpeg_srgb_export(
        source_artifact.artifact_path.display().to_string(),
        output_path.display().to_string(),
        exposure,
        contrast,
        LOCAL_ALPHA_JPEG_QUALITY,
    );
    let export_result =
        silica_export::export_jpeg_with_color_profile(silica_export::JpegColorExportRequest {
            source_path: PathBuf::from(&render_request.source_path),
            output_path: output_path.to_path_buf(),
            exposure: render_request.exposure,
            contrast: render_request.contrast,
            quality: render_request.quality,
            color_profile: silica_export::ExportColorProfile::Srgb,
        })?;
    let format = export_format_string(export_result.format).to_string();
    let exported_color_profile =
        export_color_profile_string(export_result.color_profile).to_string();
    let output_sha256 = export_result.output_sha256.clone();
    let icc_profile_sha256 = export_result.icc_profile_sha256.clone();
    let decoder_backend = source_artifact.decoder_backend.as_str().to_string();
    let input_profile = source_artifact.input_profile.clone();
    let working_space = source_artifact.working_space.clone();
    let settings_json = serde_json::json!({
        "format": format,
        "color_profile": exported_color_profile,
        "quality": render_request.quality,
        "exposure": render_request.exposure,
        "contrast": render_request.contrast,
        "source_path": source_artifact.source_path.clone(),
        "source_sha256": source_artifact.source_sha256.clone(),
        "raw_source_path": source_artifact.source_path.clone(),
        "raw_source_sha256": source_artifact.source_sha256.clone(),
        "raw_export_source_artifact_path": source_artifact.artifact_path.display().to_string(),
        "raw_export_source_artifact_sha256": source_artifact.artifact_sha256.clone(),
        "raw_export_source_artifact_bytes": source_artifact.bytes_written,
        "raw_source_original_hash_unchanged": source_artifact.original_hash_unchanged,
        "output_path": render_request.output_path,
        "output_sha256": output_sha256.clone(),
        "icc_profile_embedded": export_result.icc_profile_embedded,
        "icc_profile_sha256": icc_profile_sha256.clone(),
        "profile_metadata_source": "silica-export",
        "decoder_backend": decoder_backend.clone(),
        "input_profile": input_profile.clone(),
        "working_space": working_space.clone(),
        "export_source_kind": "raw_full_resolution_artifact",
        "viewer_texture_cache_source": render_request.uses_viewer_texture_cache_as_source(),
    })
    .to_string();
    let export_record = silica_storage::record_export(
        library_root_path,
        &candidate.photo_id,
        &export_result.output_path,
        settings_json,
    )?;

    Ok(Some(PhotoExportSession {
        photo_id: candidate.photo_id,
        source_path: source_artifact.source_path,
        output_path: export_result.output_path,
        format,
        color_profile: exported_color_profile,
        bytes_written: export_result.bytes_written,
        source_sha256: Some(source_artifact.source_sha256),
        output_sha256,
        icc_profile_embedded: export_result.icc_profile_embedded,
        icc_profile_sha256,
        decoder_backend: Some(decoder_backend),
        input_profile: Some(input_profile),
        working_space: Some(working_space),
        export_record_id: export_record.id,
        message: "RAW-derived JPEG sRGB export completed.".to_string(),
    }))
}

/// Clear disposable library cache data without removing catalog or original files.
pub fn clear_library_cache(
    library_root_path: impl AsRef<Path>,
) -> Result<LibraryCacheClearSession, CoreError> {
    let summary = silica_storage::clear_disposable_cache(library_root_path)?;
    Ok(LibraryCacheClearSession {
        cleared_directories: summary.cleared_directories,
        recreated_directories: summary.recreated_directories,
        removed_cache_records: summary.removed_cache_records,
        message: summary.message,
    })
}

fn preview_render_plan(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<(String, String, silica_render::PreviewRenderPlan)>, CoreError> {
    let candidate = match silica_storage::get_photo_preview_candidate(library_root_path, photo_id)?
    {
        Some(candidate) => candidate,
        None => return Ok(None),
    };
    let decode_plan = silica_decode::plan_preview_decode(&candidate.path, candidate.unsupported);
    let render_plan = silica_render::plan_preview_render(decode_plan);
    Ok(Some((candidate.photo_id, candidate.file_name, render_plan)))
}

fn ensure_jpeg_loupe_preview_cache(
    library_root_path: &Path,
    photo_id: &str,
    source_path: &str,
) -> Result<Option<Vec<u8>>, CoreError> {
    let source_path = PathBuf::from(source_path);
    if !is_jpeg_path(&source_path) || !source_path.is_file() {
        return Ok(None);
    }

    let preview_root = library_root_path.join("previews");
    std::fs::create_dir_all(&preview_root)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)?;
    let cache_key = preview_cache_key(photo_id, &source_path);

    if let Some(cached) = silica_storage::get_photo_cache_record(
        library_root_path,
        photo_id,
        silica_storage::PREVIEW_CACHE_TYPE,
    )? {
        let cached_path = PathBuf::from(&cached.path);
        if cached.cache_key == cache_key
            && cached_path.starts_with(&preview_root)
            && cached_path.is_file()
        {
            return std::fs::read(cached_path)
                .map(Some)
                .map_err(silica_storage::LibraryStorageError::from)
                .map_err(CoreError::from);
        }
    }

    let output_path = preview_root.join(format!("{photo_id}.jpg"));
    let result = match silica_export::write_jpeg_thumbnail(silica_export::JpegThumbnailRequest {
        source_path: source_path.clone(),
        output_path: output_path.clone(),
        max_edge: LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE,
        quality: LOCAL_ALPHA_LOUPE_PREVIEW_QUALITY,
    }) {
        Ok(result) => result,
        Err(silica_export::ExportError::Image(_)) => return Ok(None),
        Err(error) => return Err(CoreError::from(error)),
    };

    let byte_size = i64::try_from(result.bytes_written).unwrap_or(i64::MAX);
    silica_storage::record_preview_cache(
        library_root_path,
        photo_id,
        cache_key,
        &result.output_path,
        byte_size,
    )?;
    std::fs::read(result.output_path)
        .map(Some)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)
}

fn write_jpeg_develop_preview_bytes(
    library_root_path: &Path,
    photo_id: &str,
    source_path: &str,
    exposure: f64,
    contrast: f64,
) -> Result<Option<Vec<u8>>, CoreError> {
    let source_path = PathBuf::from(source_path);
    if !is_jpeg_path(&source_path) || !source_path.is_file() {
        return Ok(None);
    }

    let preview_root = library_root_path.join("previews");
    std::fs::create_dir_all(&preview_root)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)?;
    let output_path = preview_root.join(format!("develop-{photo_id}.jpg"));
    let result =
        match silica_export::write_jpeg_develop_preview(silica_export::JpegDevelopPreviewRequest {
            source_path,
            output_path,
            max_edge: LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE,
            quality: LOCAL_ALPHA_DEVELOP_PREVIEW_QUALITY,
            exposure,
            contrast,
        }) {
            Ok(result) => result,
            Err(silica_export::ExportError::Image(_)) => return Ok(None),
            Err(error) => return Err(CoreError::from(error)),
        };

    std::fs::read(result.output_path)
        .map(Some)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)
}

fn ensure_jpeg_thumbnail_cache(library_root_path: &Path) -> Result<(), CoreError> {
    let photos = silica_storage::list_library_photos(library_root_path)?;
    ensure_jpeg_thumbnail_cache_for_photos(library_root_path, &photos)
}

fn ensure_jpeg_thumbnail_cache_for_photos(
    library_root_path: &Path,
    photos: &[silica_storage::LibraryPhotoGridItem],
) -> Result<(), CoreError> {
    let photos = photos
        .iter()
        .filter(|photo| is_jpeg_thumbnail_candidate(photo))
        .collect::<Vec<_>>();
    if photos.is_empty() {
        return Ok(());
    }

    let thumbnail_root = library_root_path.join("thumbnails");
    std::fs::create_dir_all(&thumbnail_root)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)?;

    for photo in photos {
        let source_path = PathBuf::from(&photo.path);
        if !source_path.is_file() {
            continue;
        }

        let cache_key = thumbnail_cache_key(photo, &source_path);
        if has_fresh_jpeg_thumbnail_cache(photo, &cache_key, &thumbnail_root) {
            continue;
        }

        let output_path = thumbnail_root.join(format!("{}.jpg", photo.photo_id));
        let result =
            match silica_export::write_jpeg_thumbnail(silica_export::JpegThumbnailRequest {
                source_path: source_path.clone(),
                output_path: output_path.clone(),
                max_edge: LOCAL_ALPHA_THUMBNAIL_MAX_EDGE,
                quality: LOCAL_ALPHA_THUMBNAIL_QUALITY,
            }) {
                Ok(result) => result,
                Err(silica_export::ExportError::Image(_)) => continue,
                Err(error) => return Err(CoreError::from(error)),
            };
        let byte_size = i64::try_from(result.bytes_written).unwrap_or(i64::MAX);
        silica_storage::record_thumbnail_cache(
            library_root_path,
            &photo.photo_id,
            cache_key,
            &result.output_path,
            byte_size,
        )?;
    }

    Ok(())
}

fn has_fresh_jpeg_thumbnail_cache(
    photo: &silica_storage::LibraryPhotoGridItem,
    cache_key: &str,
    thumbnail_root: &Path,
) -> bool {
    if photo.thumbnail_cache_key.as_deref() != Some(cache_key) {
        return false;
    }

    let Some(thumbnail_path) = photo.thumbnail_path.as_ref() else {
        return false;
    };
    let thumbnail_path = Path::new(thumbnail_path);
    thumbnail_path.starts_with(thumbnail_root) && thumbnail_path.is_file()
}

fn is_jpeg_thumbnail_candidate(photo: &silica_storage::LibraryPhotoGridItem) -> bool {
    !photo.missing && !photo.unsupported && matches!(photo.file_type.as_str(), "JPG" | "JPEG")
}

fn app_session_to_json(session: &AppSession) -> serde_json::Value {
    let recents = session
        .recents
        .iter()
        .map(|recent| {
            serde_json::json!({
                "root_path": recent.root_path.display().to_string(),
                "display_name": recent.display_name,
                "last_opened_at": recent.last_opened_at,
            })
        })
        .collect::<Vec<_>>();
    let per_library = session
        .per_library
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                serde_json::json!({
                    "selected_photo_id": value.selected_photo_id,
                    "last_mode": app_session_mode_string(value.last_mode),
                    "last_opened_at": value.last_opened_at,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    serde_json::json!({
        "schema": APP_SESSION_SCHEMA,
        "version": APP_SESSION_VERSION,
        "last_library_root_path": session.last_library_root_path.as_ref().map(|path| path.display().to_string()),
        "last_mode": app_session_mode_string(session.last_mode),
        "recents": recents,
        "layout": {
            "sidebar_collapsed": session.layout.sidebar_collapsed,
            "inspector_collapsed": session.layout.inspector_collapsed,
            "filmstrip_visible": session.layout.filmstrip_visible,
            "thumbnail_size": session.layout.thumbnail_size,
            "sort": app_library_sort_string(session.layout.sort),
            "filters": {
                "min_rating": session.layout.filters.min_rating,
                "picked": session.layout.filters.picked,
                "rejected": session.layout.filters.rejected,
                "file_type": session.layout.filters.file_type.map(app_file_type_filter_string),
                "metadata": session.layout.filters.metadata.map(app_metadata_filter_string),
                "search": session.layout.filters.search,
            }
        },
        "per_library": per_library,
    })
}

fn parse_optional_path(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<PathBuf> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_str() {
            Some(path) => Some(PathBuf::from(path)),
            None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn parse_app_session_mode(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppSessionMode {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("library") => AppSessionMode::Library,
        Some("develop") => AppSessionMode::Develop,
        Some("export") => AppSessionMode::Export,
        Some(_) => {
            *invalid_values = true;
            AppSessionMode::Library
        }
    }
}

fn app_session_mode_string(mode: AppSessionMode) -> &'static str {
    match mode {
        AppSessionMode::Library => "library",
        AppSessionMode::Develop => "develop",
        AppSessionMode::Export => "export",
    }
}

fn parse_app_library_sort(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppLibrarySort {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("imported_at_desc") => AppLibrarySort::ImportedAtDesc,
        Some("file_name_asc") => AppLibrarySort::FileNameAsc,
        Some("rating_desc") => AppLibrarySort::RatingDesc,
        Some(_) => {
            *invalid_values = true;
            AppLibrarySort::ImportedAtDesc
        }
    }
}

fn app_library_sort_string(sort: AppLibrarySort) -> &'static str {
    match sort {
        AppLibrarySort::ImportedAtDesc => "imported_at_desc",
        AppLibrarySort::FileNameAsc => "file_name_asc",
        AppLibrarySort::RatingDesc => "rating_desc",
    }
}

fn parse_app_file_type_filter(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<AppFileTypeFilter> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_str() {
            Some("jpeg") => Some(AppFileTypeFilter::Jpeg),
            Some("raw") => Some(AppFileTypeFilter::Raw),
            Some("unsupported") => Some(AppFileTypeFilter::Unsupported),
            Some(_) | None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn app_file_type_filter_string(filter: AppFileTypeFilter) -> &'static str {
    match filter {
        AppFileTypeFilter::Jpeg => "jpeg",
        AppFileTypeFilter::Raw => "raw",
        AppFileTypeFilter::Unsupported => "unsupported",
    }
}

fn parse_app_metadata_filter(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<AppMetadataFilter> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_str() {
            Some("has_dimensions") => Some(AppMetadataFilter::HasDimensions),
            Some(_) | None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn app_metadata_filter_string(filter: AppMetadataFilter) -> &'static str {
    match filter {
        AppMetadataFilter::HasDimensions => "has_dimensions",
    }
}

fn parse_app_session_recents(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Vec<AppRecentLibrary> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(entries) = value.as_array() else {
        *invalid_values = true;
        return Vec::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            let object = match entry.as_object() {
                Some(object) => object,
                None => {
                    *invalid_values = true;
                    return None;
                }
            };
            let root_path = match object.get("root_path").and_then(serde_json::Value::as_str) {
                Some(root_path) => PathBuf::from(root_path),
                None => {
                    *invalid_values = true;
                    return None;
                }
            };
            let display_name = object
                .get("display_name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let last_opened_at = object
                .get("last_opened_at")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();

            Some(AppRecentLibrary {
                root_path,
                display_name,
                last_opened_at,
            })
        })
        .collect()
}

fn parse_app_layout(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppLayoutPreferences {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return AppLayoutPreferences::default();
    };
    let defaults = AppLayoutPreferences::default();

    AppLayoutPreferences {
        sidebar_collapsed: parse_bool_or_default(
            object.get("sidebar_collapsed"),
            defaults.sidebar_collapsed,
            invalid_values,
        ),
        inspector_collapsed: parse_bool_or_default(
            object.get("inspector_collapsed"),
            defaults.inspector_collapsed,
            invalid_values,
        ),
        filmstrip_visible: parse_bool_or_default(
            object.get("filmstrip_visible"),
            defaults.filmstrip_visible,
            invalid_values,
        ),
        thumbnail_size: parse_thumbnail_size(object.get("thumbnail_size"), invalid_values),
        sort: parse_app_library_sort(object.get("sort"), invalid_values),
        filters: parse_app_session_filters(object.get("filters"), invalid_values),
    }
}

fn parse_app_session_filters(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppSessionFilters {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return AppSessionFilters::default();
    };

    AppSessionFilters {
        min_rating: parse_min_rating(object.get("min_rating"), invalid_values),
        picked: parse_optional_bool(object.get("picked"), invalid_values),
        rejected: parse_optional_bool(object.get("rejected"), invalid_values),
        file_type: parse_app_file_type_filter(object.get("file_type"), invalid_values),
        metadata: parse_app_metadata_filter(object.get("metadata"), invalid_values),
        search: parse_search_string(object.get("search"), invalid_values),
    }
}

fn parse_app_per_library(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> BTreeMap<String, AppPerLibrarySession> {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return BTreeMap::new();
    };

    object
        .iter()
        .filter_map(|(key, value)| {
            let entry = match value.as_object() {
                Some(entry) => entry,
                None => {
                    *invalid_values = true;
                    return None;
                }
            };
            let selected_photo_id = match entry.get("selected_photo_id") {
                None | Some(serde_json::Value::Null) => None,
                Some(value) => match value.as_str() {
                    Some(photo_id) => Some(photo_id.to_string()),
                    None => {
                        *invalid_values = true;
                        None
                    }
                },
            };
            let last_opened_at = entry
                .get("last_opened_at")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            Some((
                key.clone(),
                AppPerLibrarySession {
                    selected_photo_id,
                    last_mode: parse_app_session_mode(entry.get("last_mode"), invalid_values),
                    last_opened_at,
                },
            ))
        })
        .collect()
}

fn parse_bool_or_default(
    value: Option<&serde_json::Value>,
    default: bool,
    invalid_values: &mut bool,
) -> bool {
    match value {
        None => default,
        Some(value) => match value.as_bool() {
            Some(value) => value,
            None => {
                *invalid_values = true;
                default
            }
        },
    }
}

fn parse_optional_bool(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<bool> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_bool() {
            Some(value) => Some(value),
            None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn parse_min_rating(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> Option<u8> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => {
            if let Some(value) = value.as_i64() {
                if !(0..=5).contains(&value) {
                    *invalid_values = true;
                }
                Some(value.clamp(0, 5) as u8)
            } else {
                *invalid_values = true;
                None
            }
        }
    }
}

fn parse_search_string(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> String {
    match value {
        None => String::new(),
        Some(value) => match value.as_str() {
            Some(value) => value.to_string(),
            None => {
                *invalid_values = true;
                String::new()
            }
        },
    }
}

fn parse_thumbnail_size(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> u16 {
    let Some(value) = value else {
        return DEFAULT_APP_SESSION_THUMBNAIL_SIZE;
    };
    let Some(value) = value.as_i64() else {
        *invalid_values = true;
        return DEFAULT_APP_SESSION_THUMBNAIL_SIZE;
    };
    if value < MIN_APP_SESSION_THUMBNAIL_SIZE as i64
        || value > MAX_APP_SESSION_THUMBNAIL_SIZE as i64
    {
        *invalid_values = true;
    }
    value.clamp(
        MIN_APP_SESSION_THUMBNAIL_SIZE as i64,
        MAX_APP_SESSION_THUMBNAIL_SIZE as i64,
    ) as u16
}

fn app_session_recent_key(root_path: &Path) -> String {
    std::fs::canonicalize(root_path)
        .unwrap_or_else(|_| root_path.to_path_buf())
        .display()
        .to_string()
}

fn app_session_library_display_name(root_path: &Path) -> String {
    root_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("SilicaRAW Library")
        .to_string()
}

fn is_jpeg_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg")
        })
}

fn thumbnail_cache_key(photo: &silica_storage::LibraryPhotoGridItem, source_path: &Path) -> String {
    let metadata = std::fs::metadata(source_path).ok();
    let file_size = metadata.as_ref().map(std::fs::Metadata::len).unwrap_or(0);
    let modified = metadata
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(
        "thumbnail:v1:{}:{}:{}:{}",
        photo.photo_id, photo.path, file_size, modified
    )
}

fn preview_cache_key(photo_id: &str, source_path: &Path) -> String {
    let metadata = std::fs::metadata(source_path).ok();
    let file_size = metadata.as_ref().map(std::fs::Metadata::len).unwrap_or(0);
    let modified = metadata
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(
        "preview:v1:{}:{}:{}:{}:{}:{}",
        photo_id,
        source_path.display(),
        file_size,
        modified,
        LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE,
        LOCAL_ALPHA_LOUPE_PREVIEW_QUALITY
    )
}

fn raw_preview_artifact_output_path(library_root_path: &Path, photo_id: &str) -> PathBuf {
    library_root_path
        .join("previews")
        .join(format!("raw-{photo_id}.jpg"))
}

fn raw_preview_artifact_cache_key(photo_id: &str, probe: &silica_decode::RawProbeResult) -> String {
    let backend = match probe.backend {
        silica_decode::RawProbeBackend::CoreImageRaw => "core-image-raw",
    };
    let source_sha = probe.source_sha256.as_deref().unwrap_or("missing-sha256");
    format!(
        "raw-preview:v1:{photo_id}:{backend}:{source_sha}:{}:{}x{}",
        LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE,
        probe.width.unwrap_or(0),
        probe.height.unwrap_or(0)
    )
}

fn raw_full_resolution_export_source_path(
    library_root_path: &Path,
    photo_id: &str,
    probe: &silica_decode::RawProbeResult,
) -> PathBuf {
    let source_sha = probe.source_sha256.as_deref().unwrap_or("missing-sha256");
    library_root_path
        .join("render-cache")
        .join("raw-export-sources")
        .join(format!("raw-export-{photo_id}-{source_sha}.jpg"))
}

fn paths_match(source_path: &PathBuf, output_path: &Path) -> Result<bool, CoreError> {
    if source_path == output_path {
        return Ok(true);
    }
    if !output_path.exists() {
        return Ok(false);
    }

    let source_path = match std::fs::canonicalize(source_path) {
        Ok(path) => path,
        Err(error) => {
            return Err(CoreError::Storage(
                silica_storage::LibraryStorageError::from(error),
            ))
        }
    };
    let output_path = match std::fs::canonicalize(output_path) {
        Ok(path) => path,
        Err(error) => {
            return Err(CoreError::Storage(
                silica_storage::LibraryStorageError::from(error),
            ))
        }
    };

    Ok(source_path == output_path)
}

fn preview_status_from_render(status: silica_render::PreviewRenderStatus) -> PhotoPreviewStatus {
    match status {
        silica_render::PreviewRenderStatus::Ready => PhotoPreviewStatus::Ready,
        silica_render::PreviewRenderStatus::BlockedByDecode => PhotoPreviewStatus::BlockedByDecode,
        silica_render::PreviewRenderStatus::Unsupported => PhotoPreviewStatus::Unsupported,
    }
}

fn export_format_string(format: silica_export::ExportImageFormat) -> &'static str {
    match format {
        silica_export::ExportImageFormat::Jpeg => "jpeg",
    }
}

fn export_color_profile_to_export(
    profile: PhotoExportColorProfile,
) -> silica_export::ExportColorProfile {
    match profile {
        PhotoExportColorProfile::Srgb => silica_export::ExportColorProfile::Srgb,
        PhotoExportColorProfile::DisplayP3 => silica_export::ExportColorProfile::DisplayP3,
    }
}

fn export_color_profile_string(profile: silica_export::ExportColorProfile) -> &'static str {
    match profile {
        silica_export::ExportColorProfile::Srgb => "srgb",
        silica_export::ExportColorProfile::DisplayP3 => "display_p3",
    }
}

fn export_color_profile_message(profile: PhotoExportColorProfile) -> &'static str {
    match profile {
        PhotoExportColorProfile::Srgb => "JPEG sRGB export completed.",
        PhotoExportColorProfile::DisplayP3 => "JPEG Display P3 export completed.",
    }
}

fn current_timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| format!("unix:{}", duration.as_secs()))
        .unwrap_or_else(|_| "unix:0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn exposes_crate_name() {
        assert_eq!(CRATE_NAME, "silica-core");
    }

    #[test]
    fn product_raw_decode_plan_wraps_decode_contract_without_side_effects() {
        let plan = plan_product_raw_decode("/tmp/sample.dng");

        assert_eq!(plan.source_path, "/tmp/sample.dng");
        assert_eq!(
            plan.status,
            silica_decode::ProductRawDecodeStatus::BlockedPendingEvidence
        );
        assert_ne!(
            plan.status,
            silica_decode::ProductRawDecodeStatus::Supported
        );
    }

    #[test]
    fn product_raw_decode_probe_plan_wraps_supported_fixture_evidence() {
        let probe = silica_decode::RawProbeResult {
            backend: silica_decode::RawProbeBackend::CoreImageRaw,
            platform: silica_decode::RawProbePlatform::Macos,
            macos_version: Some("26.4".to_string()),
            source_path: "/tmp/sample.cr2".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            original_file_size: Some(1024),
            original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
            status: silica_decode::RawProbeStatus::Success,
            width: Some(5184),
            height: Some(3456),
            orientation: None,
            error_category: None,
            message: "Core Image opened the RAW source.".to_string(),
        };

        let plan = plan_product_raw_decode_from_probe("A", &probe);

        assert_eq!(plan.source_path, "/tmp/sample.cr2");
        assert_eq!(
            plan.status,
            silica_decode::ProductRawDecodeStatus::Supported
        );
        assert_eq!(plan.width, Some(5184));
        assert_eq!(plan.height, Some(3456));
    }

    #[test]
    fn decoded_image_viewer_handoff_wraps_decode_and_render_without_state_writes() {
        let probe = silica_decode::RawProbeResult {
            backend: silica_decode::RawProbeBackend::CoreImageRaw,
            platform: silica_decode::RawProbePlatform::Macos,
            macos_version: Some("26.4".to_string()),
            source_path: "/tmp/sample.cr2".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            original_file_size: Some(1024),
            original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
            status: silica_decode::RawProbeStatus::Success,
            width: Some(5184),
            height: Some(3456),
            orientation: None,
            error_category: None,
            message: "Core Image opened the RAW source.".to_string(),
        };

        let plan = plan_decoded_image_viewer_handoff("A", &probe, "previews/raw/photo-1");

        assert_eq!(
            plan.decoded.status,
            silica_decode::DecodedImageHandoffStatus::Ready
        );
        assert!(!plan.writes_catalog_state());
        assert!(!plan.writes_sidecars());
        assert!(!plan.writes_originals());
        assert!(!plan.writes_exports());
        match plan.viewer_input {
            silica_render::ViewerPreviewInput::DecodedImageArtifact { cache_key, .. } => {
                assert_eq!(cache_key, "previews/raw/photo-1");
            }
            other => panic!("expected decoded image artifact input, got {other:?}"),
        }
    }

    #[test]
    fn raw_preview_artifact_path_stays_under_library_previews() {
        let library_root = PathBuf::from("/tmp/SilicaRAW Library");
        let output_path = raw_preview_artifact_output_path(&library_root, "photo-1");

        assert_eq!(
            output_path,
            library_root.join("previews").join("raw-photo-1.jpg")
        );
        assert!(output_path.starts_with(library_root.join("previews")));
    }

    #[test]
    fn raw_preview_artifact_cache_key_uses_source_hash_and_decode_settings() {
        let probe = silica_decode::RawProbeResult {
            backend: silica_decode::RawProbeBackend::CoreImageRaw,
            platform: silica_decode::RawProbePlatform::Macos,
            macos_version: Some("26.4".to_string()),
            source_path: "/tmp/sample.cr2".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            original_file_size: Some(1024),
            original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
            status: silica_decode::RawProbeStatus::Success,
            width: Some(5184),
            height: Some(3456),
            orientation: None,
            error_category: None,
            message: "Core Image opened the RAW source.".to_string(),
        };

        let cache_key = raw_preview_artifact_cache_key("photo-1", &probe);

        assert!(cache_key.contains("raw-preview:v1:photo-1"));
        assert!(cache_key.contains("fixture-hash"));
        assert!(cache_key.contains("core-image-raw"));
        assert!(cache_key.contains("2048"));
    }

    #[test]
    fn raw_preview_artifact_wrapper_keeps_blocked_classes_reviewable_without_cache_write() {
        let workspace = unique_library_root("raw-preview-wrapper-blocked");
        let library_root = workspace.join("SilicaRAW Library");
        let created = create_library(&library_root).expect("create library through core");
        let source_path = workspace.join("sample.cr2");
        std::fs::write(&source_path, b"raw placeholder").expect("write raw placeholder");
        let probe = silica_decode::RawProbeResult {
            backend: silica_decode::RawProbeBackend::CoreImageRaw,
            platform: silica_decode::RawProbePlatform::Macos,
            macos_version: Some("26.4".to_string()),
            source_path: source_path.display().to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            original_file_size: Some(1024),
            original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
            status: silica_decode::RawProbeStatus::Success,
            width: Some(1200),
            height: Some(800),
            orientation: None,
            error_category: None,
            message: "Core Image opened the RAW source.".to_string(),
        };

        let session =
            write_raw_preview_artifact_for_probe(&created.root_path, "photo-1", "E", &probe)
                .expect("blocked class remains reviewable");

        assert_eq!(
            session.handoff.decoded.status,
            silica_decode::DecodedImageHandoffStatus::BlockedPendingEvidence
        );
        assert_eq!(session.artifact_path, None);
        assert_eq!(session.cache_record, None);
        assert!(session
            .output_path
            .starts_with(created.root_path.join("previews")));
        assert!(!session.output_path.exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn metal_draft_preview_request_validates_exposure_contrast_without_state_writes() {
        let viewer_input = silica_render::ViewerPreviewInput::DecodedImageArtifact {
            cache_key: "raw-preview:v1:photo-1".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            width_px: 2048,
            height_px: 1365,
            pixel_format: silica_render::ViewerPreviewPixelFormat::JpegSrgb8,
            decoder_backend: "core_image_raw".to_string(),
            input_profile: "core_image_raw".to_string(),
            working_space: "srgb".to_string(),
        };

        let request = plan_exposure_contrast_metal_draft(
            "photo-1",
            "/tmp/sample.cr2",
            viewer_input,
            silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
            silica_render::ViewerPreviewRenderRequestId(51),
            3,
            0.5,
            -8.0,
        )
        .expect("valid metal draft request");

        assert_eq!(request.photo_id, "photo-1");
        assert_eq!(
            request.exposure_contrast_draft,
            Some(silica_render::ViewerExposureContrastDraft {
                exposure: 0.5,
                contrast: -8.0
            })
        );
        assert!(!request.writes_catalog_state());
        assert!(!request.contains_image_pixels());
    }

    #[test]
    fn metal_draft_preview_request_rejects_invalid_edit_values() {
        let viewer_input = silica_render::ViewerPreviewInput::DecodedImageArtifact {
            cache_key: "raw-preview:v1:photo-1".to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            width_px: 2048,
            height_px: 1365,
            pixel_format: silica_render::ViewerPreviewPixelFormat::JpegSrgb8,
            decoder_backend: "core_image_raw".to_string(),
            input_profile: "core_image_raw".to_string(),
            working_space: "srgb".to_string(),
        };

        let error = plan_exposure_contrast_metal_draft(
            "photo-1",
            "/tmp/sample.cr2",
            viewer_input,
            silica_render::ViewerPreviewViewport::new(1200, 675, 1.5),
            silica_render::ViewerPreviewRenderRequestId(52),
            3,
            8.0,
            0.0,
        )
        .expect_err("invalid exposure must fail edit graph validation");

        assert!(matches!(error, CoreError::EditGraph(_)));
    }

    #[test]
    fn app_session_missing_file_returns_safe_defaults() {
        let workspace = unique_library_root("app-session-missing");
        let session_path = workspace
            .join("Application Support")
            .join("dev.silicaraw.desktop")
            .join("app-session.json");

        let loaded = load_app_session(&session_path).expect("load missing session");

        assert_eq!(loaded.session.schema, APP_SESSION_SCHEMA);
        assert_eq!(loaded.session.version, APP_SESSION_VERSION);
        assert_eq!(loaded.session.last_mode, AppSessionMode::Library);
        assert!(loaded.session.last_library_root_path.is_none());
        assert!(loaded.session.recents.is_empty());
        assert!(loaded.session.per_library.is_empty());
        assert_eq!(
            loaded.session.layout.thumbnail_size,
            DEFAULT_APP_SESSION_THUMBNAIL_SIZE
        );
        assert_eq!(loaded.warnings, vec![AppSessionWarning::Missing]);
        assert!(!session_path.exists());
        assert!(!workspace.join("catalog.db").exists());
        assert!(!workspace.join("sidecars").exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn app_session_round_trips_typed_state_with_atomic_write() {
        let workspace = unique_library_root("app-session-roundtrip");
        let session_path = workspace
            .join("Application Support")
            .join("dev.silicaraw.desktop")
            .join("app-session.json");
        let library_root = workspace.join("SilicaRAW Library");

        let mut session = AppSession::default();
        session.last_library_root_path = Some(library_root.clone());
        session.last_mode = AppSessionMode::Develop;
        session.recents.push(AppRecentLibrary {
            root_path: library_root.clone(),
            display_name: "SilicaRAW Library".to_string(),
            last_opened_at: "unix:42".to_string(),
        });
        session.per_library.insert(
            library_root.display().to_string(),
            AppPerLibrarySession {
                selected_photo_id: Some("photo-1".to_string()),
                last_mode: AppSessionMode::Develop,
                last_opened_at: "unix:42".to_string(),
            },
        );

        let written = write_app_session(&session_path, &session).expect("write app session");
        assert_eq!(written.session_path, session_path);
        assert!(written.bytes_written > 0);
        assert!(session_path.is_file());
        assert!(!session_path.with_extension("tmp").exists());

        let loaded = load_app_session(&session_path).expect("load written session");
        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.session, session);

        let raw = std::fs::read_to_string(&session_path).expect("read app session json");
        assert!(raw.contains("\"schema\": \"silica.desktop_session\""));
        assert!(raw.contains("\"last_mode\": \"develop\""));

        remove_library_root(&workspace);
    }

    #[test]
    fn app_session_corrupt_or_newer_files_return_defaults_with_warnings() {
        let workspace = unique_library_root("app-session-invalid");
        let session_path = workspace.join("app-session.json");
        std::fs::create_dir_all(&workspace).expect("create session workspace");

        std::fs::write(&session_path, b"{not json").expect("write corrupt session");
        let corrupt = load_app_session(&session_path).expect("load corrupt session");
        assert_eq!(corrupt.session, AppSession::default());
        assert_eq!(corrupt.warnings, vec![AppSessionWarning::Corrupt]);

        std::fs::write(
            &session_path,
            r#"{"schema":"silica.desktop_session","version":999,"recents":[],"per_library":{}}"#,
        )
        .expect("write newer session");
        let newer = load_app_session(&session_path).expect("load newer session");
        assert_eq!(newer.session, AppSession::default());
        assert_eq!(newer.warnings, vec![AppSessionWarning::UnsupportedVersion]);

        remove_library_root(&workspace);
    }

    #[test]
    fn app_session_invalid_values_are_clamped_to_safe_defaults() {
        let workspace = unique_library_root("app-session-clamp");
        let session_path = workspace.join("app-session.json");
        std::fs::create_dir_all(&workspace).expect("create session workspace");
        std::fs::write(
            &session_path,
            r#"{
              "schema": "silica.desktop_session",
              "version": 1,
              "last_library_root_path": "/tmp/SilicaRAW Library",
              "last_mode": "unknown-mode",
              "recents": [],
              "layout": {
                "sidebar_collapsed": true,
                "inspector_collapsed": true,
                "filmstrip_visible": false,
                "thumbnail_size": 9999,
                "sort": "unknown-sort",
                "filters": {
                  "min_rating": 99,
                  "picked": true,
                  "rejected": false,
                  "file_type": "unsupported",
                  "metadata": "not-indexed",
                  "search": 123
                }
              },
              "per_library": {
                "/tmp/SilicaRAW Library": {
                  "selected_photo_id": "photo-2",
                  "last_mode": "not-real",
                  "last_opened_at": "unix:44"
                }
              }
            }"#,
        )
        .expect("write invalid value session");

        let loaded = load_app_session(&session_path).expect("load invalid value session");

        assert_eq!(loaded.session.last_mode, AppSessionMode::Library);
        assert_eq!(
            loaded.session.layout.thumbnail_size,
            MAX_APP_SESSION_THUMBNAIL_SIZE
        );
        assert_eq!(loaded.session.layout.sort, AppLibrarySort::ImportedAtDesc);
        assert_eq!(loaded.session.layout.filters.min_rating, Some(5));
        assert_eq!(loaded.session.layout.filters.search, "");
        let per_library = loaded
            .session
            .per_library
            .get("/tmp/SilicaRAW Library")
            .expect("per-library state");
        assert_eq!(per_library.last_mode, AppSessionMode::Library);
        assert_eq!(per_library.selected_photo_id.as_deref(), Some("photo-2"));
        assert_eq!(loaded.warnings, vec![AppSessionWarning::InvalidValues]);

        remove_library_root(&workspace);
    }

    #[test]
    fn layout_preferences_defaults_and_reset_are_stable() {
        let workspace = unique_library_root("layout-preferences-reset");
        let session_path = workspace.join("app-session.json");
        let library_root = workspace.join("SilicaRAW Library");
        let defaults = default_app_layout_preferences();

        assert!(!defaults.sidebar_collapsed);
        assert!(!defaults.inspector_collapsed);
        assert!(defaults.filmstrip_visible);
        assert_eq!(defaults.thumbnail_size, DEFAULT_APP_SESSION_THUMBNAIL_SIZE);
        assert_eq!(defaults.sort, AppLibrarySort::ImportedAtDesc);
        assert_eq!(defaults.filters, AppSessionFilters::default());

        let mut session = AppSession::default();
        session.last_library_root_path = Some(library_root.clone());
        write_app_session(&session_path, &session).expect("write app session");

        let mut changed_layout = default_app_layout_preferences();
        changed_layout.sidebar_collapsed = true;
        changed_layout.inspector_collapsed = true;
        changed_layout.filmstrip_visible = false;
        changed_layout.thumbnail_size = MAX_APP_SESSION_THUMBNAIL_SIZE;
        changed_layout.sort = AppLibrarySort::RatingDesc;
        changed_layout.filters.min_rating = Some(4);
        changed_layout.filters.metadata = Some(AppMetadataFilter::HasDimensions);
        changed_layout.filters.search = "portrait".to_string();
        let recorded = record_app_session_layout(&session_path, changed_layout.clone())
            .expect("record layout");
        assert_eq!(recorded.session.layout, changed_layout);

        let reset = reset_app_session_layout(&session_path).expect("reset layout");

        assert!(reset.warnings.is_empty());
        assert_eq!(reset.session.layout, defaults);
        assert_eq!(
            reset.session.last_library_root_path.as_deref(),
            Some(library_root.as_path())
        );
        let loaded = load_app_session(&session_path).expect("reload reset layout");
        assert_eq!(loaded.session.layout, defaults);

        remove_library_root(&workspace);
    }

    #[test]
    fn app_session_records_recents_with_dedupe_and_cap() {
        let workspace = unique_library_root("app-session-recents");
        let session_path = workspace.join("app-session.json");

        for index in 0..12 {
            let root_path = workspace.join(format!("Library {index}"));
            let session = LibrarySession {
                root_path: root_path.clone(),
                catalog_path: root_path.join("catalog.db"),
                schema_version: 1,
            };
            record_app_session_recent_library(&session_path, &session).expect("record app recent");
        }

        let loaded = load_app_session(&session_path).expect("load recents");
        assert!(loaded.warnings.is_empty());
        assert_eq!(loaded.session.recents.len(), APP_SESSION_RECENTS_LIMIT);
        assert_eq!(
            loaded.session.last_library_root_path.as_deref(),
            Some(workspace.join("Library 11").as_path())
        );
        assert_eq!(
            loaded
                .session
                .recents
                .first()
                .map(|recent| recent.root_path.as_path()),
            Some(workspace.join("Library 11").as_path())
        );
        assert!(!loaded
            .session
            .recents
            .iter()
            .any(|recent| recent.root_path == workspace.join("Library 0")));

        let repeated = LibrarySession {
            root_path: workspace.join("Library 5"),
            catalog_path: workspace.join("Library 5").join("catalog.db"),
            schema_version: 1,
        };
        record_app_session_recent_library(&session_path, &repeated)
            .expect("record repeated recent");
        let loaded = load_app_session(&session_path).expect("reload recents");

        assert_eq!(loaded.session.recents.len(), APP_SESSION_RECENTS_LIMIT);
        assert_eq!(
            loaded
                .session
                .recents
                .first()
                .map(|recent| recent.root_path.as_path()),
            Some(workspace.join("Library 5").as_path())
        );
        assert_eq!(
            loaded
                .session
                .recents
                .iter()
                .filter(|recent| recent.root_path == workspace.join("Library 5"))
                .count(),
            1
        );
        assert!(!workspace.join("catalog.db").exists());
        assert!(!workspace.join("sidecars").exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn app_session_restore_plans_existing_library_without_support_dir_repair() {
        let workspace = unique_library_root("app-session-restore-existing");
        let session_path = workspace.join("app-session.json");
        let library_root = workspace.join("restore-library");
        create_library(&library_root).expect("create library");
        std::fs::remove_dir_all(library_root.join("thumbnails")).expect("remove thumbnails");

        let mut session = AppSession::default();
        session.last_library_root_path = Some(library_root.clone());
        session.last_mode = AppSessionMode::Develop;
        write_app_session(&session_path, &session).expect("write app session");

        let restored = plan_app_session_restore(&session_path).expect("plan restore");

        assert_eq!(restored.status, AppSessionRestoreStatus::Restored);
        assert_eq!(restored.requested_mode, AppSessionMode::Develop);
        assert_eq!(restored.resolved_mode, AppSessionMode::Library);
        assert_eq!(
            restored.library_root_path.as_deref(),
            Some(library_root.as_path())
        );
        assert_eq!(
            restored.catalog_path.as_deref(),
            Some(library_root.join("catalog.db").as_path())
        );
        assert!(!library_root.join("thumbnails").exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn app_session_restore_falls_back_for_missing_library_or_catalog() {
        let workspace = unique_library_root("app-session-restore-missing");
        let session_path = workspace.join("app-session.json");

        let mut session = AppSession::default();
        session.last_library_root_path = Some(workspace.join("missing-library"));
        session.last_mode = AppSessionMode::Export;
        write_app_session(&session_path, &session).expect("write missing library app session");

        let missing_library_restore =
            plan_app_session_restore(&session_path).expect("plan missing library restore");
        assert_eq!(
            missing_library_restore.status,
            AppSessionRestoreStatus::MissingLibrary
        );
        assert_eq!(
            missing_library_restore.requested_mode,
            AppSessionMode::Export
        );
        assert_eq!(
            missing_library_restore.resolved_mode,
            AppSessionMode::Library
        );
        assert!(missing_library_restore.library_root_path.is_none());

        let library_without_catalog = workspace.join("library-without-catalog");
        std::fs::create_dir_all(&library_without_catalog).expect("create library dir");
        let mut session = AppSession::default();
        session.last_library_root_path = Some(library_without_catalog);
        write_app_session(&session_path, &session).expect("write missing catalog app session");

        let missing_catalog_restore =
            plan_app_session_restore(&session_path).expect("plan missing catalog restore");
        assert_eq!(
            missing_catalog_restore.status,
            AppSessionRestoreStatus::MissingCatalog
        );
        assert_eq!(
            missing_catalog_restore.requested_mode,
            AppSessionMode::Library
        );
        assert_eq!(
            missing_catalog_restore.resolved_mode,
            AppSessionMode::Library
        );
        assert!(missing_catalog_restore.catalog_path.is_none());

        remove_library_root(&workspace);
    }

    #[test]
    fn selected_photo_restore_keeps_existing_photo_and_clears_missing_photo() {
        let workspace = unique_library_root("selected-photo-restore");
        let session_path = workspace.join("app-session.json");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder");
        let photo_id = list_library_photos(&created.root_path)
            .expect("list photos")
            .into_iter()
            .find(|photo| photo.file_name == "sample.jpg")
            .map(|photo| photo.photo_id)
            .expect("imported photo id");

        record_app_session_library_state(
            &session_path,
            &created.root_path,
            Some(photo_id.clone()),
            AppSessionMode::Develop,
        )
        .expect("record selected photo");

        let restored = plan_app_session_restore(&session_path).expect("restore selected photo");
        assert_eq!(restored.status, AppSessionRestoreStatus::Restored);
        assert_eq!(
            restored.selected_photo_status,
            AppSessionSelectedPhotoStatus::Restored
        );
        assert_eq!(
            restored.selected_photo_id.as_deref(),
            Some(photo_id.as_str())
        );
        assert_eq!(restored.requested_mode, AppSessionMode::Develop);
        assert_eq!(restored.resolved_mode, AppSessionMode::Develop);

        record_app_session_library_state(
            &session_path,
            &created.root_path,
            Some("missing-photo".to_string()),
            AppSessionMode::Export,
        )
        .expect("record missing selected photo");

        let restored = plan_app_session_restore(&session_path).expect("restore missing selection");
        assert_eq!(restored.status, AppSessionRestoreStatus::Restored);
        assert_eq!(
            restored.selected_photo_status,
            AppSessionSelectedPhotoStatus::Missing
        );
        assert!(restored.selected_photo_id.is_none());
        assert_eq!(restored.requested_mode, AppSessionMode::Export);
        assert_eq!(restored.resolved_mode, AppSessionMode::Library);

        remove_library_root(&workspace);
    }

    #[test]
    fn exposes_metadata_policy_without_raw_decode_claim() {
        let jpeg_policy = metadata_extraction_policy_for_path(Path::new("sample.jpeg"));
        assert_eq!(
            jpeg_policy.dimension_source,
            silica_storage::MetadataDimensionSource::ExistingRasterPath
        );
        assert!(!jpeg_policy.raw_decode_supported);

        let raw_policy = metadata_extraction_policy_for_path(Path::new("sample.ARW"));
        assert_eq!(
            raw_policy.dimension_source,
            silica_storage::MetadataDimensionSource::Unavailable
        );
        assert!(!raw_policy.raw_decode_supported);
        assert!(!raw_policy.camera_lens_available);
    }

    #[test]
    fn imports_jpeg_metadata_without_mutating_original() {
        let workspace = unique_library_root("jpeg-metadata");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");
        let raw_file = import_root.join("sample.DNG");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&raw_file, b"raw placeholder bytes").expect("write raw");
        std::fs::write(&unsupported_file, b"unsupported note").expect("write unsupported");
        let jpeg_hash = file_hash(&jpeg_file);
        let raw_hash = file_hash(&raw_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let (width, height, camera_make, lens_model): (
            Option<i64>,
            Option<i64>,
            Option<String>,
            Option<String>,
        ) = connection
            .query_row(
                r#"
                SELECT photo_metadata.width,
                       photo_metadata.height,
                       photo_metadata.camera_make,
                       photo_metadata.lens_model
                FROM photo_metadata
                JOIN photos ON photos.id = photo_metadata.photo_id
                WHERE photos.file_name = 'sample.jpg'
                "#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("jpeg metadata row");
        assert_eq!(width, Some(2));
        assert_eq!(height, Some(2));
        assert_eq!(camera_make, None);
        assert_eq!(lens_model, None);

        let raw_metadata_count: i64 = connection
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM photo_metadata
                JOIN photos ON photos.id = photo_metadata.photo_id
                WHERE photos.file_name = 'sample.DNG'
                  AND photo_metadata.width IS NULL
                  AND photo_metadata.height IS NULL
                  AND photo_metadata.camera_make IS NULL
                  AND photo_metadata.lens_model IS NULL
                "#,
                [],
                |row| row.get(0),
            )
            .expect("raw metadata count");
        assert_eq!(raw_metadata_count, 1);

        let unsupported_metadata_count: i64 = connection
            .query_row(
                r#"
                SELECT COUNT(*)
                FROM photo_metadata
                JOIN photos ON photos.id = photo_metadata.photo_id
                WHERE photos.file_name = 'notes.txt'
                "#,
                [],
                |row| row.get(0),
            )
            .expect("unsupported metadata count");
        assert_eq!(unsupported_metadata_count, 0);

        assert_original_hash(&jpeg_file, &jpeg_hash, "JPEG metadata extraction");
        assert_original_hash(&raw_file, &raw_hash, "RAW metadata policy");

        remove_library_root(&workspace);
    }

    #[test]
    fn queries_photo_metadata_without_reopening_original() {
        let workspace = unique_library_root("core-metadata-query");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");
        std::fs::remove_file(&jpeg_file).expect("remove original before metadata query");

        let photo_id = list_library_photos(&created.root_path)
            .expect("list imported photos")
            .into_iter()
            .find(|photo| photo.file_name == "sample.jpg")
            .expect("sample photo")
            .photo_id;
        let metadata = get_photo_metadata(&created.root_path, &photo_id)
            .expect("query metadata through core")
            .expect("photo metadata");
        assert_eq!(metadata.width.state, PhotoMetadataFieldState::Known);
        assert_eq!(metadata.width.value, Some(2));
        assert_eq!(metadata.height.state, PhotoMetadataFieldState::Known);
        assert_eq!(metadata.height.value, Some(2));
        assert_eq!(
            metadata.capture_time.state,
            PhotoMetadataFieldState::Unavailable
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn metadata_filter_returns_only_photos_with_dimensions() {
        let workspace = unique_library_root("core-metadata-filter");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");
        let raw_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&raw_file, b"raw placeholder bytes").expect("write raw");

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");
        std::fs::remove_file(&jpeg_file).expect("remove original after dimension import");
        std::fs::remove_file(&raw_file).expect("remove raw original before metadata filter query");

        let page = query_library_photos(
            &created.root_path,
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
        assert_eq!(page.items[0].file_name, "sample.jpg");

        remove_library_root(&workspace);
    }

    #[test]
    fn import_error_summary_survives_core_metadata_step() {
        let workspace = unique_library_root("core-import-errors");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");
        let hidden_file = import_root.join(".hidden.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");
        std::fs::write(&hidden_file, b"hidden jpeg").expect("write hidden");

        let created = create_library(&library_root).expect("create library through core");
        let summary = import_folder(&created.root_path, &import_root).expect("import through core");

        assert_eq!(summary.supported_files, 1);
        assert_eq!(summary.unsupported_files, 1);
        assert!(summary
            .issues
            .iter()
            .any(|issue| issue.kind == ImportIssueKind::UnsupportedFile));
        assert!(summary
            .issues
            .iter()
            .any(|issue| issue.kind == ImportIssueKind::HiddenEntrySkipped));

        let rows = list_library_photos(&created.root_path).expect("browse after import issues");
        assert_eq!(rows.len(), 2);

        remove_library_root(&workspace);
    }

    #[test]
    fn recursive_import_opt_in_through_core_imports_nested_rows() {
        let workspace = unique_library_root("core-recursive-import");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let nested_root = import_root.join("Nested");
        let nested_file = nested_root.join("child.jpg");
        let unsupported_file = nested_root.join("notes.txt");

        std::fs::create_dir_all(&nested_root).expect("create nested import directory");
        write_source_jpeg(&nested_file);
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        let created = create_library(&library_root).expect("create library through core");
        let default_summary =
            import_folder(&created.root_path, &import_root).expect("default import");
        assert_eq!(default_summary.scanned_files, 0);

        let summary = import_folder_with_options(
            &created.root_path,
            &import_root,
            FolderImportOptions { recursive: true },
        )
        .expect("recursive import through core");

        assert_eq!(summary.supported_files, 1);
        assert_eq!(summary.unsupported_files, 1);
        assert!(summary
            .issues
            .iter()
            .any(|issue| issue.kind == ImportIssueKind::UnsupportedFile));

        let rows = list_library_photos(&created.root_path).expect("browse recursive rows");
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|row| row.file_name == "child.jpg"));
        assert!(rows
            .iter()
            .any(|row| row.file_name == "notes.txt" && row.unsupported));

        remove_library_root(&workspace);
    }

    #[test]
    fn creates_and_reopens_local_library_session() {
        let root = unique_library_root("core");

        let created = create_library(&root).expect("create library through core");
        let reopened = open_library(&root).expect("open library through core");

        assert_eq!(created.root_path, root);
        assert_eq!(reopened.root_path, created.root_path);
        assert_eq!(reopened.catalog_path, created.catalog_path);
        assert_eq!(reopened.schema_version, created.schema_version);
        assert!(created.catalog_path.is_file());
        assert!(created.status_text().contains("Library:"));
        assert!(created.status_text().contains("catalog.db"));

        remove_library_root(&root);
    }

    #[test]
    fn imports_and_persists_photo_flags_through_core() {
        let workspace = unique_library_root("core-flags");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");

        let created = create_library(&library_root).expect("create library through core");
        let summary = import_folder(&created.root_path, &import_root).expect("import through core");
        assert_eq!(summary.supported_files, 1);

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.DNG'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");

        let updated = set_photo_flags(
            &created.root_path,
            photo_id,
            3,
            false,
            true,
            Some("red".to_string()),
        )
        .expect("set flags through core");

        let reopened = open_library(&library_root).expect("reopen library through core");
        let persisted = get_photo_flags(&reopened.root_path, &updated.photo_id)
            .expect("read flags through core")
            .expect("flags row");

        assert_eq!(persisted, updated);

        remove_library_root(&workspace);
    }

    #[test]
    fn serializes_library_photo_grid_rows_for_desktop() {
        let workspace = unique_library_root("core-grid");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.DNG'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        set_photo_flags(
            &created.root_path,
            photo_id,
            4,
            true,
            false,
            Some("green".to_string()),
        )
        .expect("set grid flags through core");

        let rows = list_library_photos_json(&created.root_path).expect("list grid rows as json");
        let rows: serde_json::Value = serde_json::from_str(&rows).expect("parse grid rows json");
        let rows = rows.as_array().expect("grid rows array");

        assert_eq!(rows.len(), 2);
        assert!(rows.iter().any(|row| {
            row["fileName"] == "sample.DNG"
                && row["fileType"] == "DNG"
                && row["rating"] == 4
                && row["picked"] == true
                && row["colorLabel"] == "green"
        }));
        assert!(rows.iter().any(|row| {
            row["fileName"] == "notes.txt" && row["fileType"] == "TXT" && row["unsupported"] == true
        }));

        remove_library_root(&workspace);
    }

    #[test]
    fn library_query_returns_page_without_cache_hydration() {
        let workspace = unique_library_root("core-library-query");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");
        let raw_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&raw_file, b"raw candidate").expect("write raw");

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");

        let page = query_library_photos(
            &created.root_path,
            LibraryQueryRequest::new(
                0,
                1,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters::default(),
            ),
        )
        .expect("query page through core");

        assert_eq!(page.items.len(), 1);
        assert_eq!(page.total_count, 2);
        assert!(page.has_next_page);

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let cache_records: i64 = connection
            .query_row("SELECT COUNT(*) FROM cache_records", [], |row| row.get(0))
            .expect("count cache records");
        assert_eq!(cache_records, 0);

        remove_library_root(&workspace);
    }

    #[test]
    fn creates_jpeg_thumbnail_cache_for_grid_without_mutating_original() {
        let workspace = unique_library_root("core-thumbnail-grid");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");
        let raw_file = import_root.join("sample.DNG");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&raw_file, b"supported raw candidate").expect("write raw candidate");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        let original_hash = file_hash(&jpeg_file);
        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");

        let rows = list_library_photos(&created.root_path).expect("list grid rows");

        let jpeg = rows
            .iter()
            .find(|row| row.file_name == "sample.jpg")
            .expect("jpeg grid row");
        let thumbnail_path = PathBuf::from(
            jpeg.thumbnail_path
                .as_ref()
                .expect("jpeg row exposes thumbnail path"),
        );
        assert!(thumbnail_path.starts_with(created.root_path.join("thumbnails")));
        assert!(thumbnail_path.is_file());
        let decoded = image::ImageReader::open(&thumbnail_path)
            .expect("open thumbnail")
            .with_guessed_format()
            .expect("guess thumbnail format")
            .decode()
            .expect("decode thumbnail");
        assert!(decoded.width() <= 320);
        assert!(decoded.height() <= 320);
        assert_original_hash(&jpeg_file, &original_hash, "thumbnail cache generation");

        let raw = rows
            .iter()
            .find(|row| row.file_name == "sample.DNG")
            .expect("raw grid row");
        assert!(raw.thumbnail_path.is_none());
        let unsupported = rows
            .iter()
            .find(|row| row.file_name == "notes.txt")
            .expect("unsupported grid row");
        assert!(unsupported.thumbnail_path.is_none());

        let cached_rows = list_library_photos(&created.root_path).expect("list cached grid rows");
        let cached_jpeg = cached_rows
            .iter()
            .find(|row| row.file_name == "sample.jpg")
            .expect("cached jpeg grid row");
        assert_eq!(
            cached_jpeg.thumbnail_path.as_deref(),
            jpeg.thumbnail_path.as_deref()
        );
        assert_eq!(
            cached_jpeg.thumbnail_cache_key.as_deref(),
            jpeg.thumbnail_cache_key.as_deref()
        );

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
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
    fn hydrates_thumbnails_only_for_queried_page() {
        let workspace = unique_library_root("core-thumbnail-page");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let first_jpeg = import_root.join("a-first.jpg");
        let second_jpeg = import_root.join("b-second.jpg");
        let raw_file = import_root.join("c-raw.DNG");
        let unsupported_file = import_root.join("d-notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&first_jpeg);
        write_source_jpeg(&second_jpeg);
        std::fs::write(&raw_file, b"raw candidate").expect("write raw");
        std::fs::write(&unsupported_file, b"unsupported").expect("write unsupported");
        let first_hash = file_hash(&first_jpeg);
        let second_hash = file_hash(&second_jpeg);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");

        let page = query_library_photos_with_thumbnail_hydration(
            &created.root_path,
            LibraryQueryRequest::new(
                0,
                1,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters::default(),
            ),
        )
        .expect("query hydrated page");

        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].file_name, "a-first.jpg");
        assert!(page.items[0].thumbnail_path.is_some());
        assert!(page.items[0].thumbnail_cache_key.is_some());
        assert_original_hash(&first_jpeg, &first_hash, "page thumbnail hydration");
        assert_original_hash(&second_jpeg, &second_hash, "page thumbnail hydration");

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let thumbnail_records: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'thumbnail'",
                [],
                |row| row.get(0),
            )
            .expect("count thumbnail records");
        assert_eq!(thumbnail_records, 1);

        let second_page = query_library_photos(
            &created.root_path,
            LibraryQueryRequest::new(
                1,
                1,
                LibraryQuerySort::FileNameAsc,
                LibraryQueryFilters::default(),
            ),
        )
        .expect("query second page without hydration");
        assert_eq!(second_page.items[0].file_name, "b-second.jpg");
        assert!(second_page.items[0].thumbnail_path.is_none());

        remove_library_root(&workspace);
    }

    #[test]
    fn opens_preview_session_with_ready_and_blocked_states() {
        let workspace = unique_library_root("core-preview");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");
        let raw_file = import_root.join("sample.dng");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&raw_file, b"raw placeholder bytes").expect("write raw");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        let original_hash = file_hash(&jpeg_file);
        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let jpeg_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("jpeg photo id");
        let raw_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.dng'",
                [],
                |row| row.get(0),
            )
            .expect("raw photo id");
        let unsupported_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'notes.txt'",
                [],
                |row| row.get(0),
            )
            .expect("unsupported photo id");

        let jpeg_preview = open_photo_preview(&created.root_path, &jpeg_id)
            .expect("open jpeg preview")
            .expect("jpeg preview session");
        assert_eq!(jpeg_preview.file_name, "sample.jpg");
        assert_eq!(jpeg_preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(jpeg_preview.source_path, jpeg_file.display().to_string());
        assert!(jpeg_preview
            .preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert_original_hash(&jpeg_file, &original_hash, "loupe preview cache generation");

        let jpeg_preview_again = open_photo_preview(&created.root_path, &jpeg_id)
            .expect("reopen jpeg preview")
            .expect("cached jpeg preview session");
        assert_eq!(jpeg_preview_again.preview_bytes, jpeg_preview.preview_bytes);

        let raw_preview = open_photo_preview(&created.root_path, &raw_id)
            .expect("open raw preview")
            .expect("raw preview session");
        assert_eq!(raw_preview.status, PhotoPreviewStatus::BlockedByDecode);
        assert!(raw_preview.message.contains("Core Image RAW preview"));
        assert!(raw_preview.preview_bytes.is_none());

        let unsupported_preview = open_photo_preview(&created.root_path, &unsupported_id)
            .expect("open unsupported preview")
            .expect("unsupported preview session");
        assert_eq!(unsupported_preview.status, PhotoPreviewStatus::Unsupported);

        assert!(open_photo_preview(&created.root_path, "missing-photo")
            .expect("missing preview lookup")
            .is_none());

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
    fn previews_without_write_and_commits_exposure_contrast_edit() {
        let workspace = unique_library_root("core-edit-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);

        let original_hash = file_hash(&jpeg_file);
        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        let edit_state_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
            .expect("count edit states");
        assert_eq!(edit_state_count, 0);
        drop(connection);

        let preview = preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("preview edit")
            .expect("preview edit request");

        assert_eq!(preview.photo_id, photo_id);
        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.exposure, 0.5);
        assert_eq!(preview.contrast, -8.0);
        assert!(preview.message.contains("exposure/contrast"));
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert_original_hash(&jpeg_file, &original_hash, "develop preview generation");

        let default_edit_state = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read default edit state")
            .expect("default edit state");
        assert_eq!(default_edit_state.exposure, 0.0);
        assert_eq!(default_edit_state.contrast, 0.0);
        assert!(!default_edit_state.persisted);

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let edit_state_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
            .expect("count edit states");
        assert_eq!(
            edit_state_count, 0,
            "preview edit must not write edit_states"
        );
        drop(connection);

        let committed = commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("committed edit");
        assert_eq!(committed.photo_id, photo_id);
        assert_eq!(committed.exposure, 0.5);
        assert_eq!(committed.contrast, -8.0);
        assert!(committed.persisted);

        let reopened = open_library(&library_root).expect("reopen library through core");
        let persisted = silica_storage::load_active_edit_graph_or_default(
            &reopened.root_path,
            &committed.photo_id,
        )
        .expect("load active graph")
        .expect("active graph");
        assert_eq!(persisted.basic.exposure.as_f64(), Some(0.5));
        assert_eq!(persisted.basic.contrast.as_f64(), Some(-8.0));

        let restored = get_photo_edit_state(&reopened.root_path, &committed.photo_id)
            .expect("read restored edit state")
            .expect("restored edit state");
        assert_eq!(restored.exposure, 0.5);
        assert_eq!(restored.contrast, -8.0);
        assert!(restored.persisted);

        remove_library_root(&workspace);
    }

    #[test]
    fn undo_and_redo_edit_history_through_core() {
        let workspace = unique_library_root("core-undo-redo");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        let created = create_library(&library_root).expect("create library");
        import_folder(&created.root_path, &import_root).expect("import folder");
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);

        commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("commit result");

        let undo = undo_last_history_action(&created.root_path, &photo_id).expect("undo");
        assert!(undo.applied);
        let undone = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read undone edit")
            .expect("edit state");
        assert_eq!(undone.exposure, 0.0);
        assert_eq!(undone.contrast, 0.0);

        let redo = redo_last_history_action(&created.root_path, &photo_id).expect("redo");
        assert!(redo.applied);
        let redone = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read redone edit")
            .expect("edit state");
        assert_eq!(redone.exposure, 0.5);
        assert_eq!(redone.contrast, -8.0);

        remove_library_root(&workspace);
    }

    #[test]
    fn exports_edited_photo_to_jpeg_srgb_and_records_catalog_row() {
        let workspace = unique_library_root("core-export");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&jpeg_file);
        let original_before = std::fs::read(&jpeg_file).expect("read original before");

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);

        commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("edit commit");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");

        assert_eq!(exported.photo_id, photo_id);
        assert_eq!(exported.output_path, output_path);
        assert_eq!(exported.format, "jpeg");
        assert_eq!(exported.color_profile, "srgb");
        assert!(exported.bytes_written > 0);
        assert_eq!(
            exported
                .source_sha256
                .as_deref()
                .expect("source SHA-256 evidence")
                .len(),
            64
        );
        assert_eq!(exported.output_sha256.len(), 64);
        assert!(exported.icc_profile_embedded);
        assert_eq!(
            exported.icc_profile_sha256,
            "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e"
        );
        assert_eq!(
            std::fs::read(&jpeg_file).expect("read original after"),
            original_before
        );

        let decoded = image::ImageReader::open(&exported.output_path)
            .expect("open exported jpeg")
            .with_guessed_format()
            .expect("guess exported format")
            .decode()
            .expect("decode exported jpeg");
        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 2);

        let latest =
            silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
                .expect("read latest export")
                .expect("latest export");
        assert_eq!(latest.id, exported.export_record_id);
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["color_profile"], "srgb");
        assert_eq!(settings["icc_profile_embedded"], true);
        assert_eq!(
            settings["icc_profile_sha256"],
            "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e"
        );
        assert_eq!(
            settings["output_sha256"]
                .as_str()
                .expect("output hash string")
                .len(),
            64
        );
        assert_eq!(
            settings["source_sha256"].as_str(),
            exported.source_sha256.as_deref()
        );

        let flags = get_photo_flags(&created.root_path, &exported.photo_id)
            .expect("read flags")
            .expect("flags row");
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let exported_flag: i64 = connection
            .query_row(
                "SELECT exported FROM photo_flags WHERE photo_id = ?1",
                [&flags.photo_id],
                |row| row.get(0),
            )
            .expect("exported flag");
        assert_eq!(exported_flag, 1);

        remove_library_root(&workspace);
    }

    #[test]
    fn raw_derived_jpeg_srgb_export_rejects_original_overwrite_before_decode() {
        let workspace = unique_library_root("core-raw-export-overwrite");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let raw_file = import_root.join("sample.cr2");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&raw_file, b"raw placeholder").expect("write raw placeholder");
        let created = create_library(&library_root).expect("create library");
        import_folder(&created.root_path, &import_root).expect("import folder");
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.cr2'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);
        let probe = successful_raw_probe(&raw_file.display().to_string(), Some(5184), Some(3456));

        let error = export_raw_photo_jpeg_srgb_from_probe(
            &created.root_path,
            &photo_id,
            "A",
            &probe,
            &raw_file,
        )
        .expect_err("RAW export cannot overwrite original");

        assert!(matches!(
            error,
            CoreError::RawExport(
                silica_decode::RawFullResolutionExportSourceError::OutputMatchesSource(_)
            )
        ));
        assert_original_hash(&raw_file, &file_hash(&raw_file), "RAW overwrite rejection");

        remove_library_root(&workspace);
    }

    #[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
    #[test]
    #[ignore]
    fn raw_derived_jpeg_srgb_export_from_fixture_records_evidence_without_preview_cache() {
        let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
            .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
        let report =
            silica_decode::probe_raw_fixture_manifest(manifest).expect("probe RAW fixtures");
        let fixture = report
            .results
            .iter()
            .find(|result| result.fixture_class == "A")
            .expect("Class A fixture evidence");
        let raw_path = PathBuf::from(&fixture.probe.source_path);
        let import_root = raw_path.parent().expect("fixture parent");
        let workspace = unique_library_root("core-raw-export-fixture");
        let library_root = workspace.join("SilicaRAW Library");
        let export_root = workspace.join("Exports");
        let baseline_output = export_root.join("baseline.jpg");
        let adjusted_output = export_root.join("adjusted.jpg");

        std::fs::create_dir_all(&export_root).expect("create export directory");
        let created = create_library(&library_root).expect("create library");
        import_folder(&created.root_path, import_root).expect("import RAW fixture folder");
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE path = ?1",
                [&fixture.probe.source_path],
                |row| row.get(0),
            )
            .expect("fixture photo id");
        drop(connection);

        let baseline = export_raw_photo_jpeg_srgb_from_probe(
            &created.root_path,
            &photo_id,
            &fixture.fixture_class,
            &fixture.probe,
            &baseline_output,
        )
        .expect("export baseline RAW photo")
        .expect("baseline export result");
        commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("commit exposure/contrast")
            .expect("commit result");
        let adjusted = export_raw_photo_jpeg_srgb_from_probe(
            &created.root_path,
            &photo_id,
            &fixture.fixture_class,
            &fixture.probe,
            &adjusted_output,
        )
        .expect("export adjusted RAW photo")
        .expect("adjusted export result");

        assert_eq!(
            adjusted.source_sha256.as_deref(),
            fixture.probe.source_sha256.as_deref()
        );
        assert_ne!(baseline.output_sha256, adjusted.output_sha256);
        assert!(adjusted.icc_profile_embedded);
        assert_eq!(adjusted.decoder_backend.as_deref(), Some("core_image_raw"));
        assert_eq!(adjusted.input_profile.as_deref(), Some("core_image_raw"));
        assert_eq!(adjusted.working_space.as_deref(), Some("srgb"));
        assert!(adjusted.output_path.is_file());
        assert_ne!(adjusted.output_path, raw_path);
        assert!(silica_storage::get_photo_cache_record(
            &created.root_path,
            &photo_id,
            silica_storage::PREVIEW_CACHE_TYPE,
        )
        .expect("preview cache lookup")
        .is_none());

        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(
            settings["source_sha256"],
            fixture.probe.source_sha256.clone().unwrap()
        );
        assert_eq!(settings["output_sha256"], adjusted.output_sha256);
        assert_eq!(settings["icc_profile_embedded"], true);
        assert_eq!(settings["icc_profile_sha256"], adjusted.icc_profile_sha256);
        assert_eq!(settings["decoder_backend"], "core_image_raw");
        assert_eq!(settings["input_profile"], "core_image_raw");
        assert_eq!(settings["working_space"], "srgb");
        assert_eq!(settings["profile_metadata_source"], "silica-export");
        assert_eq!(
            settings["export_source_kind"],
            "raw_full_resolution_artifact"
        );
        assert_eq!(settings["viewer_texture_cache_source"], false);
        assert_eq!(settings["raw_source_original_hash_unchanged"], true);
        let artifact_path = settings["raw_export_source_artifact_path"]
            .as_str()
            .expect("artifact path");
        assert!(artifact_path.contains("render-cache/raw-export-sources"));
        assert!(!artifact_path.contains("/previews/"));

        if let Ok(qa_dir) = std::env::var("SILICARAW_RAW_EXPORT_QA_DIR") {
            let qa_dir = PathBuf::from(qa_dir);
            std::fs::create_dir_all(&qa_dir).expect("create RAW export QA directory");
            let qa_output = qa_dir.join(format!("{}-adjusted-srgb.jpg", fixture.fixture_id));
            std::fs::copy(&adjusted.output_path, &qa_output).expect("copy adjusted QA export");
            let qa_evidence = serde_json::json!({
                "task": "15.6",
                "fixture_id": fixture.fixture_id,
                "fixture_class": fixture.fixture_class,
                "source_path": fixture.probe.source_path,
                "source_sha256": fixture.probe.source_sha256,
                "output_path": qa_output.display().to_string(),
                "output_sha256": adjusted.output_sha256,
                "icc_profile_embedded": adjusted.icc_profile_embedded,
                "icc_profile_sha256": adjusted.icc_profile_sha256,
                "decoder_backend": adjusted.decoder_backend,
                "input_profile": adjusted.input_profile,
                "working_space": adjusted.working_space,
                "export_settings": settings,
            })
            .to_string();
            std::fs::write(qa_dir.join("raw-export-qa-evidence.json"), qa_evidence)
                .expect("write RAW export QA evidence");
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn exports_edited_photo_to_jpeg_display_p3_when_explicit() {
        let workspace = unique_library_root("core-export-display-p3");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-display-p3.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&jpeg_file);
        let original_before = std::fs::read(&jpeg_file).expect("read original before");

        let created = create_library(&library_root).expect("create library");
        import_folder(&created.root_path, &import_root).expect("import folder");
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);

        let exported = export_photo_jpeg(
            &created.root_path,
            &photo_id,
            &output_path,
            PhotoExportColorProfile::DisplayP3,
        )
        .expect("export photo")
        .expect("export result");

        assert_eq!(exported.output_path, output_path);
        assert_eq!(exported.format, "jpeg");
        assert_eq!(exported.color_profile, "display_p3");
        assert!(exported.bytes_written > 0);
        assert_eq!(
            std::fs::read(&jpeg_file).expect("read original after"),
            original_before
        );
        let latest =
            silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
                .expect("read latest export")
                .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["color_profile"], "display_p3");
        assert_eq!(
            settings["icc_profile_sha256"],
            "0ff6958f98684c61f6bbdce1368ddeaf3873baf84545baba482e920d92a914c0"
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn writes_and_reads_photo_sidecar_through_core() {
        let workspace = unique_library_root("core-sidecar");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);
        let original_hash = file_hash(&jpeg_file);

        let created = create_library(&library_root).expect("create library");
        import_folder(&created.root_path, &import_root).expect("import folder");
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);
        set_photo_flags(
            &created.root_path,
            photo_id.clone(),
            2,
            true,
            false,
            Some("blue".to_string()),
        )
        .expect("set flags");

        let written = write_photo_sidecar(&created.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write sidecar")
            .expect("sidecar write result");
        assert_eq!(written.photo_id, photo_id);
        assert!(written.sidecar_path.is_file());
        assert_original_hash(&jpeg_file, &original_hash, "core sidecar write");

        let read = read_photo_sidecar(&created.root_path, &photo_id)
            .expect("read sidecar")
            .expect("sidecar exists");
        assert_eq!(read.photo_id, photo_id);
        assert_eq!(read.flags.rating, 2);
        assert_eq!(read.flags.color_label.as_deref(), Some("blue"));
        assert_original_hash(&jpeg_file, &original_hash, "core sidecar read");

        remove_library_root(&workspace);
    }

    #[test]
    fn dry_runs_sidecar_rebuild_through_core_without_mutating_flags() {
        let workspace = unique_library_root("core-sidecar-rebuild");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);

        let created = create_library(&library_root).expect("create library");
        import_folder(&created.root_path, &import_root).expect("import folder");
        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);

        set_photo_flags(
            &created.root_path,
            photo_id.clone(),
            5,
            true,
            false,
            Some("green".to_string()),
        )
        .expect("set sidecar flags");
        write_photo_sidecar(&created.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write sidecar")
            .expect("sidecar write result");
        set_photo_flags(&created.root_path, photo_id.clone(), 1, false, true, None)
            .expect("change live catalog flags");

        let report =
            dry_run_catalog_rebuild_from_sidecars(&created.root_path).expect("dry-run rebuild");

        assert_eq!(report.sidecars_scanned, 1);
        assert!(report.issues.is_empty());
        assert_eq!(report.entries.len(), 1);
        assert_eq!(
            report.entries[0].action,
            CatalogRebuildDryRunAction::UpdatePhotoFlags
        );
        assert_eq!(
            report.entries[0].flag_source,
            CatalogRebuildFlagSource::SidecarFlags
        );
        assert_eq!(report.entries[0].resolved_flags.rating, 5);

        let live_flags = get_photo_flags(&created.root_path, &photo_id)
            .expect("read live flags")
            .expect("live flags");
        assert_eq!(live_flags.rating, 1);
        assert!(!live_flags.picked);
        assert!(live_flags.rejected);

        remove_library_root(&workspace);
    }

    #[test]
    fn local_alpha_workflow_preserves_original_file_hash() {
        let workspace = unique_library_root("core-original-safety");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&jpeg_file);
        let original_hash = file_hash(&jpeg_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import through core");
        assert_original_hash(&jpeg_file, &original_hash, "import by reference");

        let connection = silica_storage::open_catalog(&created.catalog_path).expect("open catalog");
        let photo_id: String = connection
            .query_row(
                "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
                [],
                |row| row.get(0),
            )
            .expect("photo id");
        drop(connection);

        set_photo_flags(
            &created.root_path,
            photo_id.clone(),
            5,
            true,
            false,
            Some("green".to_string()),
        )
        .expect("set flags through core");
        assert_original_hash(&jpeg_file, &original_hash, "rating and pick update");

        let preview = open_photo_preview(&created.root_path, &photo_id)
            .expect("open preview")
            .expect("preview session");
        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_original_hash(&jpeg_file, &original_hash, "preview open");

        preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("preview edit")
            .expect("preview edit request");
        assert_original_hash(&jpeg_file, &original_hash, "draft edit preview");

        commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("edit commit");
        assert_original_hash(&jpeg_file, &original_hash, "edit commit");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");
        assert_eq!(exported.source_path, jpeg_file.display().to_string());
        assert_eq!(exported.output_path, output_path);
        assert!(exported.output_path.is_file());
        assert_ne!(exported.output_path, jpeg_file);
        assert_original_hash(&jpeg_file, &original_hash, "JPEG sRGB export");

        let cache_clear = clear_library_cache(&created.root_path).expect("clear library cache");
        assert_eq!(cache_clear.removed_cache_records, 1);
        assert_eq!(
            cache_clear.cleared_directories,
            vec!["thumbnails", "previews", "render-cache", "ai-cache"]
        );
        for directory in &cache_clear.recreated_directories {
            assert!(created.root_path.join(directory).is_dir());
        }
        assert_original_hash(&jpeg_file, &original_hash, "cache directory clear");

        let reopened = open_library(&library_root).expect("reopen library through core");
        assert_original_hash(&jpeg_file, &original_hash, "library restart and reopen");

        let flags = get_photo_flags(&reopened.root_path, &photo_id)
            .expect("read flags")
            .expect("flags row");
        assert_eq!(flags.rating, 5);
        assert!(flags.picked);
        assert!(!flags.rejected);

        let persisted =
            silica_storage::load_active_edit_graph_or_default(&reopened.root_path, &photo_id)
                .expect("load active graph")
                .expect("active graph");
        assert_eq!(persisted.basic.exposure.as_f64(), Some(0.5));
        assert_eq!(persisted.basic.contrast.as_f64(), Some(-8.0));

        let latest = silica_storage::get_latest_export_record(&reopened.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        assert_eq!(
            latest.output_path,
            exported.output_path.display().to_string()
        );

        remove_library_root(&workspace);
    }

    fn unique_library_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "silicaraw-core-library-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn remove_library_root(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }

    fn file_hash(path: &Path) -> String {
        let bytes = std::fs::read(path).expect("read file for hash");
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in bytes {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{hash:016x}")
    }

    fn assert_original_hash(path: &Path, expected_hash: &str, stage: &str) {
        assert_eq!(
            file_hash(path),
            expected_hash,
            "original file hash changed after {stage}"
        );
    }

    fn write_source_jpeg(path: &Path) {
        let image = image::RgbImage::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgb([64, 128, 192])
            } else {
                image::Rgb([192, 128, 64])
            }
        });
        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write source jpeg");
    }

    fn successful_raw_probe(
        source_path: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> silica_decode::RawProbeResult {
        silica_decode::RawProbeResult {
            backend: silica_decode::RawProbeBackend::CoreImageRaw,
            platform: silica_decode::RawProbePlatform::Macos,
            macos_version: Some("26.4".to_string()),
            source_path: source_path.to_string(),
            source_sha256: Some(file_hash(Path::new(source_path))),
            original_file_size: Some(1024),
            original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
            status: silica_decode::RawProbeStatus::Success,
            width,
            height,
            orientation: None,
            error_category: None,
            message: "Core Image opened the RAW source.".to_string(),
        }
    }
}
