//! Core coordination boundary for SilicaRAW.
//!
//! Phase 4.2 starts the local library command surface.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
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
pub use silica_edit::BasicPreset;
pub use silica_edit::CurveMode;
pub use silica_edit::EditClipboardPayload;
pub use silica_edit::EditClipboardSelection;
pub use silica_edit::HslColorChannel;
pub use silica_edit::WhiteBalance;
pub use silica_storage::ActionLogEntry;
pub use silica_storage::CatalogRebuildDryRunAction;
pub use silica_storage::CatalogRebuildDryRunEntry;
pub use silica_storage::CatalogRebuildDryRunIssue;
pub use silica_storage::CatalogRebuildDryRunIssueKind;
pub use silica_storage::CatalogRebuildDryRunReport;
pub use silica_storage::CatalogRebuildFlagSource;
pub use silica_storage::ExportPreset;
pub use silica_storage::ExportSettings;
pub use silica_storage::ExportSettingsCatalog;
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
pub use silica_storage::NewActionLogEntry;
pub use silica_storage::PhotoFlags;
pub use silica_storage::PhotoHistoryItem;
pub use silica_storage::PhotoHistoryPanel;
pub use silica_storage::PhotoMetadata;
pub use silica_storage::PhotoMetadataField;
pub use silica_storage::PhotoMetadataFieldState;
pub use silica_storage::PhotoSidecarStatus;
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
const LOCAL_ALPHA_BRUSH_MASK_RASTER_EDGE: u32 = 512;

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
/// Default UI scale preference in percent.
pub const DEFAULT_APP_SESSION_UI_SCALE: u16 = 100;
/// Minimum accepted UI scale preference in percent.
pub const MIN_APP_SESSION_UI_SCALE: u16 = 90;
/// Maximum accepted UI scale preference in percent.
pub const MAX_APP_SESSION_UI_SCALE: u16 = 120;
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

/// Supported app theme preferences for the current tokenized shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAppearanceTheme {
    Dark,
    Light,
}

/// Supported app density preferences for the current tokenized shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAppearanceDensity {
    Compact,
    Comfortable,
}

/// App-level appearance preferences persisted outside every library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppAppearancePreferences {
    pub theme: AppAppearanceTheme,
    pub density: AppAppearanceDensity,
    pub ui_scale: u16,
}

impl Default for AppAppearancePreferences {
    fn default() -> Self {
        Self {
            theme: AppAppearanceTheme::Dark,
            density: AppAppearanceDensity::Compact,
            ui_scale: DEFAULT_APP_SESSION_UI_SCALE,
        }
    }
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
    pub appearance: AppAppearancePreferences,
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
            appearance: AppAppearancePreferences::default(),
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

/// Normalized tone curve point exposed by the core command boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoToneCurvePoint {
    pub x: f64,
    pub y: f64,
}

/// Current tone curve state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoToneCurveState {
    pub curve_mode: silica_edit::CurveMode,
    pub rgb_curve: Vec<PhotoToneCurvePoint>,
    pub red_curve: Vec<PhotoToneCurvePoint>,
    pub green_curve: Vec<PhotoToneCurvePoint>,
    pub blue_curve: Vec<PhotoToneCurvePoint>,
}

/// One HSL color mixer channel exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoHslColorChannelState {
    pub hue: f64,
    pub saturation: f64,
    pub luminance: f64,
}

/// Current HSL color mixer state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoHslColorMixerState {
    pub red: PhotoHslColorChannelState,
    pub orange: PhotoHslColorChannelState,
    pub yellow: PhotoHslColorChannelState,
    pub green: PhotoHslColorChannelState,
    pub aqua: PhotoHslColorChannelState,
    pub blue: PhotoHslColorChannelState,
    pub purple: PhotoHslColorChannelState,
    pub magenta: PhotoHslColorChannelState,
}

/// Sharpening state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoDetailSharpeningState {
    pub amount: f64,
    pub radius: f64,
    pub detail: f64,
    pub masking: f64,
}

/// Non-MLX noise reduction state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoDetailNoiseReductionState {
    pub luminance: f64,
    pub detail: f64,
    pub contrast: f64,
    pub color: f64,
    pub color_detail: f64,
}

/// Current Detail state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoDetailState {
    pub sharpening: PhotoDetailSharpeningState,
    pub noise_reduction: PhotoDetailNoiseReductionState,
}

/// Normalized crop state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoGeometryCropState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    pub aspect: Option<String>,
}

/// Perspective/scale transform state exposed for explicit unsupported-state reporting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoGeometryTransformState {
    pub vertical: f64,
    pub horizontal: f64,
    pub aspect: f64,
    pub scale: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

/// Current Geometry state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoGeometryState {
    pub crop: Option<PhotoGeometryCropState>,
    pub rotation: f64,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub transform: PhotoGeometryTransformState,
}

/// Manual mask geometry exposed to local app surfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhotoManualMaskGeometryState {
    LinearGradient {
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    },
    RadialGradient {
        center_x: f64,
        center_y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
    },
}

/// Manual mask state exposed to local app surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoManualMaskState {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub enabled: bool,
    pub invert: bool,
    pub opacity: f64,
    pub feather: f64,
    pub geometry: Option<PhotoManualMaskGeometryState>,
    pub exposure: f64,
    pub contrast: f64,
}

/// Core input point for a sampled manual brush stroke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoManualBrushPointInput {
    pub x: f64,
    pub y: f64,
}

/// Core input stroke for a sampled manual brush mask.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoManualBrushStrokeInput {
    pub id: String,
    pub radius: f64,
    pub points: Vec<PhotoManualBrushPointInput>,
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
    pub white_balance: silica_edit::WhiteBalance,
    pub temperature: f64,
    pub tint: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
    pub vibrance: f64,
    pub saturation: f64,
    pub tone_curve: PhotoToneCurveState,
    pub hsl_color_mixer: PhotoHslColorMixerState,
    pub detail: PhotoDetailState,
    pub geometry: PhotoGeometryState,
    pub masks: Vec<PhotoManualMaskState>,
    pub message: String,
}

impl PhotoEditPreviewSession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nPreview: {:?}\nSource: {}\nExposure: {}\nContrast: {}\nWhite Balance: {:?}\nTemperature: {}\nTint: {}\nHighlights: {}\nShadows: {}\nWhites: {}\nBlacks: {}\nVibrance: {}\nSaturation: {}\nMessage: {}",
            self.photo_id,
            self.status,
            self.source_path,
            self.exposure,
            self.contrast,
            self.white_balance,
            self.temperature,
            self.tint,
            self.highlights,
            self.shadows,
            self.whites,
            self.blacks,
            self.vibrance,
            self.saturation,
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
    pub white_balance: silica_edit::WhiteBalance,
    pub temperature: f64,
    pub tint: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
    pub vibrance: f64,
    pub saturation: f64,
    pub tone_curve: PhotoToneCurveState,
    pub hsl_color_mixer: PhotoHslColorMixerState,
    pub detail: PhotoDetailState,
    pub geometry: PhotoGeometryState,
    pub masks: Vec<PhotoManualMaskState>,
    pub persisted: bool,
    pub message: String,
}

impl PhotoEditCommit {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nExposure: {}\nContrast: {}\nWhite Balance: {:?}\nTemperature: {}\nTint: {}\nHighlights: {}\nShadows: {}\nWhites: {}\nBlacks: {}\nVibrance: {}\nSaturation: {}\nPersisted: {}\nMessage: {}",
            self.photo_id,
            self.exposure,
            self.contrast,
            self.white_balance,
            self.temperature,
            self.tint,
            self.highlights,
            self.shadows,
            self.whites,
            self.blacks,
            self.vibrance,
            self.saturation,
            self.persisted,
            self.message
        )
    }
}

/// Result of applying one edit clipboard payload across multiple catalog photos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncResult {
    pub status: String,
    pub requested_count: usize,
    pub applied_count: usize,
    pub skipped_count: usize,
    pub blocked_count: usize,
    pub failed_count: usize,
    pub commits: Vec<BatchEditClipboardSyncCommit>,
    pub targets: Vec<BatchEditClipboardSyncTarget>,
    pub failures: Vec<BatchEditClipboardSyncFailure>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncCommit {
    pub photo_id: String,
    pub history_id: String,
    pub sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncFailure {
    pub photo_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncPlan {
    pub status: String,
    pub requested_count: usize,
    pub ready_count: usize,
    pub unchanged_count: usize,
    pub blocked_count: usize,
    pub targets: Vec<BatchEditClipboardSyncTarget>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncTarget {
    pub photo_id: String,
    pub status: String,
    pub code: Option<String>,
    pub message: String,
}

struct PreparedEditClipboardSync {
    plan: BatchEditClipboardSyncPlan,
    ready_graphs: Vec<silica_edit::EditGraph>,
}

/// Current committed exposure/contrast edit state for a catalog photo.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoEditState {
    pub photo_id: String,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: silica_edit::WhiteBalance,
    pub temperature: f64,
    pub tint: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
    pub vibrance: f64,
    pub saturation: f64,
    pub tone_curve: PhotoToneCurveState,
    pub hsl_color_mixer: PhotoHslColorMixerState,
    pub detail: PhotoDetailState,
    pub geometry: PhotoGeometryState,
    pub masks: Vec<PhotoManualMaskState>,
    pub persisted: bool,
    pub message: String,
}

impl PhotoEditState {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nExposure: {}\nContrast: {}\nWhite Balance: {:?}\nTemperature: {}\nTint: {}\nHighlights: {}\nShadows: {}\nWhites: {}\nBlacks: {}\nVibrance: {}\nSaturation: {}\nPersisted: {}\nMessage: {}",
            self.photo_id,
            self.exposure,
            self.contrast,
            self.white_balance,
            self.temperature,
            self.tint,
            self.highlights,
            self.shadows,
            self.whites,
            self.blacks,
            self.vibrance,
            self.saturation,
            self.persisted,
            self.message
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

/// Recent export record returned through the core boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoRecentExport {
    pub export_record_id: String,
    pub photo_id: String,
    pub output_path: String,
    pub export_settings_json: String,
    pub created_at: String,
    pub output_exists: bool,
}

/// JPEG output color profile accepted by the core export boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoExportColorProfile {
    Srgb,
    DisplayP3,
}

/// Source metadata policy accepted by the core export boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoExportMetadataPolicy {
    Minimal,
    Preserve,
    RemoveGps,
    RemoveAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PhotoExportFormat {
    Jpeg,
    Png,
    Tiff,
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

/// Histogram readiness for the current supported preview state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoHistogramStatus {
    Ready,
    BlockedByDecode,
    Unsupported,
    Missing,
}

/// Histogram data returned for the current committed Develop state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoHistogramSession {
    pub photo_id: String,
    pub source_path: String,
    pub status: PhotoHistogramStatus,
    pub red: Vec<u32>,
    pub green: Vec<u32>,
    pub blue: Vec<u32>,
    pub luminance: Vec<u32>,
    pub pixel_count: u64,
    pub cache_key: String,
    pub cache_path: String,
    pub message: String,
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
    EditClipboard(silica_edit::EditClipboardError),
    Export(silica_export::ExportError),
    ExportBlocked(String),
    UnsupportedEdit(String),
    AppSession(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "{error}"),
            Self::Decode(error) => write!(formatter, "{error}"),
            Self::RawExport(error) => write!(formatter, "{error}"),
            Self::EditGraph(error) => write!(formatter, "{error}"),
            Self::EditClipboard(error) => write!(formatter, "{error}"),
            Self::Export(error) => write!(formatter, "{error}"),
            Self::ExportBlocked(message) => write!(formatter, "export blocked: {message}"),
            Self::UnsupportedEdit(message) => write!(formatter, "unsupported edit: {message}"),
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
            Self::EditClipboard(error) => Some(error),
            Self::Export(error) => Some(error),
            Self::ExportBlocked(_) => None,
            Self::UnsupportedEdit(_) => None,
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

impl From<silica_edit::EditClipboardError> for CoreError {
    fn from(error: silica_edit::EditClipboardError) -> Self {
        Self::EditClipboard(error)
    }
}

impl From<silica_export::ExportError> for CoreError {
    fn from(error: silica_export::ExportError) -> Self {
        Self::Export(error)
    }
}

/// Copy edit sections through the core boundary without reading or writing catalog state.
pub fn copy_edit_clipboard_payload(
    source: &silica_edit::EditGraph,
    selection: silica_edit::EditClipboardSelection,
) -> Result<silica_edit::EditClipboardPayload, CoreError> {
    silica_edit::copy_edit_clipboard_payload(source, selection).map_err(CoreError::from)
}

/// Apply an edit clipboard payload to a target graph without catalog or sidecar mutation.
pub fn apply_edit_clipboard_payload_to_graph(
    target: &silica_edit::EditGraph,
    payload: &silica_edit::EditClipboardPayload,
    updated_at: impl Into<String>,
) -> Result<silica_edit::EditGraph, CoreError> {
    silica_edit::apply_edit_clipboard_payload(target, payload, updated_at).map_err(CoreError::from)
}

/// Copy edit sections from one catalog photo through the core boundary.
pub fn copy_photo_edit_clipboard_payload(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    selection: silica_edit::EditClipboardSelection,
) -> Result<Option<silica_edit::EditClipboardPayload>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if let Some((code, message)) = edit_clipboard_catalog_target_block(library_root_path, photo_id)?
    {
        if code == "missing_photo" {
            return Ok(None);
        }
        return Err(CoreError::UnsupportedEdit(message));
    }

    let Some(graph) =
        silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)?
    else {
        return Ok(None);
    };
    silica_edit::copy_edit_clipboard_payload(&graph, selection)
        .map(Some)
        .map_err(CoreError::from)
}

/// Plan a batch edit clipboard sync without mutating catalog state.
pub fn plan_edit_clipboard_sync(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<BatchEditClipboardSyncPlan, CoreError> {
    Ok(prepare_edit_clipboard_sync(library_root_path, photo_ids, payload)?.plan)
}

/// Apply one typed edit clipboard payload to multiple photos with per-photo history checkpoints.
pub fn apply_edit_clipboard_sync(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<BatchEditClipboardSyncResult, CoreError> {
    let prepared = prepare_edit_clipboard_sync(library_root_path.as_ref(), photo_ids, payload)?;
    if prepared.plan.status == "empty" {
        return Ok(BatchEditClipboardSyncResult {
            status: "empty".to_string(),
            requested_count: prepared.plan.requested_count,
            applied_count: 0,
            skipped_count: 0,
            blocked_count: 0,
            failed_count: 0,
            commits: Vec::new(),
            targets: prepared.plan.targets,
            failures: Vec::new(),
            message: prepared.plan.message,
        });
    }
    if prepared.plan.blocked_count > 0 {
        let failures = prepared
            .plan
            .targets
            .iter()
            .filter(|target| target.status == "blocked")
            .map(|target| BatchEditClipboardSyncFailure {
                photo_id: target.photo_id.clone(),
                message: target.message.clone(),
            })
            .collect::<Vec<_>>();
        return Ok(BatchEditClipboardSyncResult {
            status: "blocked".to_string(),
            requested_count: prepared.plan.requested_count,
            applied_count: 0,
            skipped_count: prepared.plan.unchanged_count,
            blocked_count: prepared.plan.blocked_count,
            failed_count: prepared.plan.blocked_count,
            commits: Vec::new(),
            targets: prepared.plan.targets,
            failures,
            message: format!(
                "{} target(s) blocked; nothing was written.",
                prepared.plan.blocked_count
            ),
        });
    }
    if prepared.ready_graphs.is_empty() {
        return Ok(BatchEditClipboardSyncResult {
            status: "skipped".to_string(),
            requested_count: prepared.plan.requested_count,
            applied_count: 0,
            skipped_count: prepared.plan.unchanged_count,
            blocked_count: 0,
            failed_count: 0,
            commits: Vec::new(),
            targets: prepared.plan.targets,
            failures: Vec::new(),
            message: "Batch sync skipped; all targets were unchanged.".to_string(),
        });
    }

    let library_root_path = library_root_path.as_ref();
    let storage_result =
        silica_storage::commit_edit_graph_batch(library_root_path, prepared.ready_graphs)?;
    let commits = storage_result
        .commits
        .into_iter()
        .map(|commit| BatchEditClipboardSyncCommit {
            photo_id: commit.photo_id,
            history_id: commit.history_id,
            sequence: commit.sequence,
        })
        .collect::<Vec<_>>();
    let applied_count = commits.len();
    let skipped_count = prepared.plan.unchanged_count + storage_result.skipped_photo_ids.len();
    let status = if applied_count == 0 {
        "skipped"
    } else {
        "applied"
    };

    Ok(BatchEditClipboardSyncResult {
        status: status.to_string(),
        requested_count: prepared.plan.requested_count,
        applied_count,
        skipped_count,
        blocked_count: 0,
        failed_count: 0,
        commits,
        targets: prepared.plan.targets,
        failures: Vec::new(),
        message: format!(
            "Batch sync completed: {applied_count} applied, {skipped_count} unchanged."
        ),
    })
}

/// Backward-compatible name for the batch clipboard apply boundary.
pub fn sync_edit_clipboard_payload_to_photos(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<BatchEditClipboardSyncResult, CoreError> {
    apply_edit_clipboard_sync(library_root_path, photo_ids, payload)
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
        appearance: parse_app_appearance(object.get("appearance"), &mut invalid_values),
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

/// Return the documented default appearance preferences.
pub fn default_app_appearance_preferences() -> AppAppearancePreferences {
    AppAppearancePreferences::default()
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

/// Reset only app appearance preferences in app-level desktop session state.
pub fn reset_app_session_appearance(
    session_path: impl AsRef<Path>,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.appearance = default_app_appearance_preferences();
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

/// Record app appearance preferences in app-level desktop session state.
pub fn record_app_session_appearance(
    session_path: impl AsRef<Path>,
    appearance: AppAppearancePreferences,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.appearance = appearance;
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

/// Append a validated action-log row through the core command boundary.
pub fn append_action_log_entry(
    library_root_path: impl AsRef<Path>,
    entry: NewActionLogEntry,
) -> Result<ActionLogEntry, CoreError> {
    silica_storage::append_action_log_entry(library_root_path, entry).map_err(CoreError::from)
}

/// Read recent action-log rows through the core command boundary.
pub fn list_action_log_entries(
    library_root_path: impl AsRef<Path>,
    limit: u16,
) -> Result<Vec<ActionLogEntry>, CoreError> {
    silica_storage::list_action_log_entries(library_root_path, limit).map_err(CoreError::from)
}

/// Read the library-wide export settings and named presets.
pub fn get_export_settings_catalog(
    library_root_path: impl AsRef<Path>,
) -> Result<ExportSettingsCatalog, CoreError> {
    silica_storage::get_export_settings_catalog(library_root_path).map_err(CoreError::from)
}

/// Create or update a named export preset.
pub fn upsert_export_preset(
    library_root_path: impl AsRef<Path>,
    name: impl AsRef<str>,
    settings: ExportSettings,
) -> Result<ExportPreset, CoreError> {
    silica_storage::upsert_export_preset(library_root_path, name, settings).map_err(CoreError::from)
}

/// Persist the current default export settings.
pub fn set_default_export_settings(
    library_root_path: impl AsRef<Path>,
    preset_id: Option<&str>,
    settings: ExportSettings,
) -> Result<ExportSettingsCatalog, CoreError> {
    silica_storage::set_default_export_settings(library_root_path, preset_id, settings)
        .map_err(CoreError::from)
}

fn append_core_action_log(
    library_root_path: &Path,
    action_type: &str,
    subject_type: Option<&str>,
    subject_id: Option<String>,
    side_effect_category: &str,
    evidence_ref: Option<String>,
    payload: serde_json::Value,
) -> Result<ActionLogEntry, CoreError> {
    let payload_json = serde_json::to_string(&payload).map_err(|error| {
        CoreError::AppSession(format!("action log payload serialization failed: {error}"))
    })?;
    append_action_log_entry(
        library_root_path,
        NewActionLogEntry {
            actor_type: "core".to_string(),
            actor_id: Some("local-alpha".to_string()),
            action_type: action_type.to_string(),
            subject_type: subject_type.map(str::to_string),
            subject_id,
            side_effect_category: side_effect_category.to_string(),
            evidence_ref,
            payload_json,
        },
    )
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
    let folder_path = folder_path.as_ref().to_path_buf();
    let summary =
        silica_storage::import_folder_with_options(&library_root_path, &folder_path, options)?;
    persist_imported_photo_metadata(&library_root_path, &summary)?;
    append_core_action_log(
        &library_root_path,
        "import_reference",
        Some("folder"),
        Some(summary.folder_path.display().to_string()),
        "catalog_reference",
        Some(summary.folder_path.display().to_string()),
        serde_json::json!({
            "scanned_files": summary.scanned_files,
            "supported_files": summary.supported_files,
            "unsupported_files": summary.unsupported_files,
            "recursive": options.recursive,
        }),
    )?;
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

/// List real undoable history checkpoints for one photo through the core boundary.
pub fn list_photo_history(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<silica_storage::PhotoHistoryPanel, CoreError> {
    silica_storage::list_photo_history(library_root_path, photo_id).map_err(CoreError::from)
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
    let library_root_path = library_root_path.as_ref().to_path_buf();
    match silica_storage::write_photo_sidecar(&library_root_path, photo_id, app_version) {
        Ok(result) => {
            append_core_action_log(
                &library_root_path,
                "sidecar_write",
                Some("photo"),
                Some(result.photo_id.clone()),
                "sidecar_write",
                Some(result.sidecar_relative_path.clone()),
                serde_json::json!({
                    "sidecar_relative_path": result.sidecar_relative_path.clone(),
                    "bytes_written": result.bytes_written,
                    "app_version": app_version,
                }),
            )?;
            Ok(Some(result))
        }
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

/// Read catalog-side sidecar sync status through the core command boundary.
pub fn get_photo_sidecar_status(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoSidecarStatus>, CoreError> {
    silica_storage::get_photo_sidecar_status(library_root_path, photo_id).map_err(CoreError::from)
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

/// Build and cache histogram data for the current committed Develop state.
pub fn get_photo_histogram(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoHistogramSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let candidate = match silica_storage::get_photo_preview_candidate(library_root_path, photo_id)?
    {
        Some(candidate) => candidate,
        None => return Ok(None),
    };
    let source_path = PathBuf::from(&candidate.path);
    let empty_bins = || vec![0; 256];

    if candidate.unsupported {
        return Ok(Some(PhotoHistogramSession {
            photo_id: candidate.photo_id,
            source_path: candidate.path,
            status: PhotoHistogramStatus::Unsupported,
            red: empty_bins(),
            green: empty_bins(),
            blue: empty_bins(),
            luminance: empty_bins(),
            pixel_count: 0,
            cache_key: String::new(),
            cache_path: String::new(),
            message: "Histogram unavailable for unsupported files.".to_string(),
        }));
    }
    if !source_path.is_file() {
        return Ok(Some(PhotoHistogramSession {
            photo_id: candidate.photo_id,
            source_path: candidate.path,
            status: PhotoHistogramStatus::Missing,
            red: empty_bins(),
            green: empty_bins(),
            blue: empty_bins(),
            luminance: empty_bins(),
            pixel_count: 0,
            cache_key: String::new(),
            cache_path: String::new(),
            message: "Histogram unavailable because the referenced source file is missing."
                .to_string(),
        }));
    }
    if !is_jpeg_path(&source_path) {
        return Ok(Some(PhotoHistogramSession {
            photo_id: candidate.photo_id,
            source_path: candidate.path,
            status: PhotoHistogramStatus::BlockedByDecode,
            red: empty_bins(),
            green: empty_bins(),
            blue: empty_bins(),
            luminance: empty_bins(),
            pixel_count: 0,
            cache_key: String::new(),
            cache_path: String::new(),
            message: "Histogram blocked until RAW decode provides preview pixels.".to_string(),
        }));
    }

    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let render_geometry = render_geometry_from_graph(&graph);
    if let Some(message) =
        lens_unsupported_message(&graph).or_else(|| geometry_unsupported_message(&render_geometry))
    {
        return Ok(Some(PhotoHistogramSession {
            photo_id: candidate.photo_id,
            source_path: candidate.path,
            status: PhotoHistogramStatus::Unsupported,
            red: empty_bins(),
            green: empty_bins(),
            blue: empty_bins(),
            luminance: empty_bins(),
            pixel_count: 0,
            cache_key: String::new(),
            cache_path: String::new(),
            message,
        }));
    }
    let render_detail = render_detail_from_graph(&graph);
    if !render_detail.is_neutral() {
        return Ok(Some(PhotoHistogramSession {
            photo_id: candidate.photo_id,
            source_path: candidate.path,
            status: PhotoHistogramStatus::Unsupported,
            red: empty_bins(),
            green: empty_bins(),
            blue: empty_bins(),
            luminance: empty_bins(),
            pixel_count: 0,
            cache_key: String::new(),
            cache_path: String::new(),
            message: detail_unsupported_message(),
        }));
    }
    let histogram =
        silica_export::compute_jpeg_develop_histogram(silica_export::JpegHistogramRequest {
            source_path: source_path.clone(),
            exposure: graph.basic.exposure.as_f64().unwrap_or(0.0),
            contrast: graph.basic.contrast.as_f64().unwrap_or(0.0),
            white_balance: export_white_balance_from_render(render_white_balance_from_graph(
                &graph,
            )),
            tone_recovery: export_tone_recovery_from_render(render_tone_recovery_from_graph(
                &graph,
            )),
            color_presence: export_color_presence_from_render(render_color_presence_from_graph(
                &graph,
            )),
            tone_curve: export_tone_curve_from_render(render_tone_curve_from_graph(&graph)),
            hsl_color_mixer: export_hsl_color_mixer_from_render(render_hsl_color_mixer_from_graph(
                &graph,
            )),
            detail: export_detail_from_render(render_detail),
            geometry: export_geometry_from_render(render_geometry),
        })?;
    let pixel_count = histogram.pixel_count;
    let red = histogram.red;
    let green = histogram.green;
    let blue = histogram.blue;
    let luminance = histogram.luminance;
    let cache_key = histogram_cache_key(photo_id, &source_path, &graph);
    let render_cache_root = library_root_path.join("render-cache");
    std::fs::create_dir_all(&render_cache_root)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)?;
    let cache_path = render_cache_root.join(format!("histogram-{photo_id}.json"));
    let cache_value = serde_json::json!({
        "schema": "silica.histogram",
        "version": 1,
        "photo_id": photo_id,
        "source_path": source_path.display().to_string(),
        "cache_key": cache_key,
        "pixel_count": pixel_count,
        "red": &red,
        "green": &green,
        "blue": &blue,
        "luminance": &luminance,
    });
    let cache_bytes = serde_json::to_vec(&cache_value)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)?;
    std::fs::write(&cache_path, &cache_bytes)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)?;
    silica_storage::record_histogram_cache(
        library_root_path,
        photo_id,
        cache_key.clone(),
        &cache_path,
        cache_bytes.len() as i64,
    )?;

    Ok(Some(PhotoHistogramSession {
        photo_id: candidate.photo_id,
        source_path: source_path.display().to_string(),
        status: PhotoHistogramStatus::Ready,
        red,
        green,
        blue,
        luminance,
        pixel_count,
        cache_key,
        cache_path: cache_path.display().to_string(),
        message: "Histogram ready from current committed Develop state.".to_string(),
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
    let mut request = silica_render::plan_exposure_contrast_preview(
        render_plan,
        edited.basic.exposure.as_f64().unwrap_or(exposure),
        edited.basic.contrast.as_f64().unwrap_or(contrast),
    );
    request.white_balance = render_white_balance_from_graph(&graph);
    request.tone_recovery = render_tone_recovery_from_graph(&graph);
    request.color_presence = render_color_presence_from_graph(&graph);
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft white-balance preview request without writing the catalog.
pub fn preview_white_balance_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    white_balance: silica_edit::WhiteBalance,
    temperature: f64,
    tint: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_white_balance_temperature_tint(
        &graph,
        white_balance,
        temperature,
        tint,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let exposure = graph.basic.exposure.as_f64().unwrap_or(0.0);
    let contrast = graph.basic.contrast.as_f64().unwrap_or(0.0);
    let request = silica_render::plan_white_balance_preview(
        render_plan,
        exposure,
        contrast,
        render_white_balance_from_graph(&edited),
    );
    let mut request = request;
    request.tone_recovery = render_tone_recovery_from_graph(&graph);
    request.color_presence = render_color_presence_from_graph(&graph);
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: edited.basic.white_balance,
        temperature: edited.basic.temperature.as_f64().unwrap_or(temperature),
        tint: edited.basic.tint.as_f64().unwrap_or(tint),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft tone-recovery preview request without writing the catalog.
pub fn preview_tone_recovery_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_recovery(
        &graph,
        highlights,
        shadows,
        whites,
        blacks,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_tone_recovery_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&edited),
    );
    let mut request = request;
    request.color_presence = render_color_presence_from_graph(&graph);
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: edited.basic.highlights.as_f64().unwrap_or(highlights),
        shadows: edited.basic.shadows.as_f64().unwrap_or(shadows),
        whites: edited.basic.whites.as_f64().unwrap_or(whites),
        blacks: edited.basic.blacks.as_f64().unwrap_or(blacks),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft tone-curve preview request without writing the catalog.
pub fn preview_tone_curve_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rgb_curve: &[(f64, f64)],
    red_curve: &[(f64, f64)],
    green_curve: &[(f64, f64)],
    blue_curve: &[(f64, f64)],
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_curve(
        &graph,
        silica_edit::CurveMode::Point,
        rgb_curve,
        red_curve,
        green_curve,
        blue_curve,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let mut request = silica_render::plan_tone_curve_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        render_tone_curve_from_graph(&edited),
    );
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &edited)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&edited),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&edited),
        detail: detail_state_from_graph(&edited),
        geometry: geometry_state_from_graph(&edited),
        masks: photo_manual_masks_from_graph(&edited),
        message,
    }))
}

/// Build a draft HSL color mixer preview request without writing the catalog.
pub fn preview_hsl_color_mixer_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    channel: silica_edit::HslColorChannel,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_hsl_color_channel(
        &graph,
        channel,
        hue,
        saturation,
        luminance,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_hsl_color_mixer_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        render_tone_curve_from_graph(&graph),
        render_hsl_color_mixer_from_graph(&edited),
    );
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &edited)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&edited),
        detail: detail_state_from_graph(&edited),
        geometry: geometry_state_from_graph(&edited),
        masks: photo_manual_masks_from_graph(&edited),
        message,
    }))
}

/// Build a draft color-presence preview request without writing the catalog.
pub fn preview_color_presence_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    vibrance: f64,
    saturation: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_color_presence(
        &graph,
        vibrance,
        saturation,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let mut request = silica_render::plan_color_presence_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&edited),
    );
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: edited.basic.vibrance.as_f64().unwrap_or(vibrance),
        saturation: edited.basic.saturation.as_f64().unwrap_or(saturation),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft sharpening preview without writing the catalog.
pub fn preview_detail_sharpening_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_detail_sharpening(
        &graph,
        amount,
        radius,
        detail,
        masking,
        current_timestamp_string(),
    )?;

    preview_detail_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft noise-reduction preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_detail_noise_reduction_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_detail_noise_reduction(
        &graph,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
        current_timestamp_string(),
    )?;

    preview_detail_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft rectangular crop preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_geometry_crop_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<&str>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_crop(
        &graph,
        x,
        y,
        width,
        height,
        angle,
        aspect,
        current_timestamp_string(),
    )?;

    preview_geometry_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft crop-clear preview without writing the catalog.
pub fn preview_clear_geometry_crop(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::clear_geometry_crop(&graph, current_timestamp_string())?;

    preview_geometry_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft rotation/flip preview without writing the catalog.
pub fn preview_geometry_orientation_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_orientation(
        &graph,
        rotation,
        flip_horizontal,
        flip_vertical,
        current_timestamp_string(),
    )?;

    preview_geometry_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft manual linear-gradient mask preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_manual_linear_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_linear_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        start_x,
        start_y,
        end_x,
        end_y,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;

    preview_manual_mask_edit(library_root_path, photo_id, &edited)
}

/// Build a draft manual radial-gradient mask preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_manual_radial_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_radial_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        center_x,
        center_y,
        radius_x,
        radius_y,
        rotation,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;

    preview_manual_mask_edit(library_root_path, photo_id, &edited)
}

/// Build a draft manual brush mask preview without writing durable edit state.
#[allow(clippy::too_many_arguments)]
pub fn preview_manual_brush_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    strokes: Vec<PhotoManualBrushStrokeInput>,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_brush_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        edit_brush_strokes(strokes)?,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;

    preview_manual_mask_edit(library_root_path, photo_id, &edited)
}

/// Persist a manual linear-gradient mask on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_manual_linear_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_linear_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        start_x,
        start_y,
        end_x,
        end_y,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;
    ensure_supported_manual_masks_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Manual linear gradient mask persisted on commit.",
    )))
}

/// Persist a manual radial-gradient mask on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_manual_radial_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_radial_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        center_x,
        center_y,
        radius_x,
        radius_y,
        rotation,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;
    ensure_supported_manual_masks_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Manual radial gradient mask persisted on commit.",
    )))
}

/// Persist a manual brush mask on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_manual_brush_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    strokes: Vec<PhotoManualBrushStrokeInput>,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_brush_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        edit_brush_strokes(strokes)?,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;
    ensure_supported_manual_masks_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Manual brush mask persisted on commit.",
    )))
}

/// Detail commit is intentionally blocked until a real renderer/export path exists.
pub fn commit_detail_sharpening_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let _edited = silica_edit::apply_detail_sharpening(
        &graph,
        amount,
        radius,
        detail,
        masking,
        current_timestamp_string(),
    )?;

    Err(CoreError::UnsupportedEdit(detail_unsupported_message()))
}

/// Detail commit is intentionally blocked until a real renderer/export path exists.
#[allow(clippy::too_many_arguments)]
pub fn commit_detail_noise_reduction_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let _edited = silica_edit::apply_detail_noise_reduction(
        &graph,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
        current_timestamp_string(),
    )?;

    Err(CoreError::UnsupportedEdit(detail_unsupported_message()))
}

/// Persist a rectangular crop edit on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_geometry_crop_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<&str>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_crop(
        &graph,
        x,
        y,
        width,
        height,
        angle,
        aspect,
        current_timestamp_string(),
    )?;
    ensure_supported_lens_geometry_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Geometry crop edit persisted on commit.",
    )))
}

/// Persist a crop-clear edit on commit/release.
pub fn commit_clear_geometry_crop(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::clear_geometry_crop(&graph, current_timestamp_string())?;
    ensure_supported_lens_geometry_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Geometry crop cleared on commit.",
    )))
}

/// Persist a rotation/flip edit on commit/release.
pub fn commit_geometry_orientation_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_orientation(
        &graph,
        rotation,
        flip_horizontal,
        flip_vertical,
        current_timestamp_string(),
    )?;
    ensure_supported_lens_geometry_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Geometry orientation edit persisted on commit.",
    )))
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
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(exposure),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(contrast),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Exposure/contrast edit persisted on commit.".to_string(),
    }))
}

/// Persist a white-balance edit on commit/release.
pub fn commit_white_balance_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    white_balance: silica_edit::WhiteBalance,
    temperature: f64,
    tint: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_white_balance_temperature_tint(
        &graph,
        white_balance,
        temperature,
        tint,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(temperature),
        tint: persisted.basic.tint.as_f64().unwrap_or(tint),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "White balance edit persisted on commit.".to_string(),
    }))
}

/// Persist a tone-recovery edit on commit/release.
pub fn commit_tone_recovery_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_recovery(
        &graph,
        highlights,
        shadows,
        whites,
        blacks,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(highlights),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(shadows),
        whites: persisted.basic.whites.as_f64().unwrap_or(whites),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(blacks),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Tone recovery edit persisted on commit.".to_string(),
    }))
}

/// Persist a tone-curve edit on commit/release.
pub fn commit_tone_curve_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rgb_curve: &[(f64, f64)],
    red_curve: &[(f64, f64)],
    green_curve: &[(f64, f64)],
    blue_curve: &[(f64, f64)],
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_curve(
        &graph,
        silica_edit::CurveMode::Point,
        rgb_curve,
        red_curve,
        green_curve,
        blue_curve,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Tone curve edit persisted on commit.".to_string(),
    }))
}

/// Persist an HSL color mixer edit on commit/release.
pub fn commit_hsl_color_mixer_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    channel: silica_edit::HslColorChannel,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_hsl_color_channel(
        &graph,
        channel,
        hue,
        saturation,
        luminance,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "HSL color mixer edit persisted on commit.".to_string(),
    }))
}

/// Persist a color-presence edit on commit/release.
pub fn commit_color_presence_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    vibrance: f64,
    saturation: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_color_presence(
        &graph,
        vibrance,
        saturation,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(vibrance),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(saturation),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Color presence edit persisted on commit.".to_string(),
    }))
}

/// Persist a full P0 Basic reset as one undoable edit checkpoint.
pub fn commit_p0_basic_reset(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::reset_p0_basic_controls(&graph, current_timestamp_string())?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "P0 Basic reset persisted on commit.".to_string(),
    }))
}

/// Persist a built-in Basic preset as one undoable edit checkpoint.
pub fn commit_basic_preset_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    preset: silica_edit::BasicPreset,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_basic_preset(&graph, preset, current_timestamp_string())?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Basic preset persisted on commit.".to_string(),
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
            photo_id: graph.source.photo_id.clone(),
            exposure: graph.basic.exposure.as_f64().unwrap_or(0.0),
            contrast: graph.basic.contrast.as_f64().unwrap_or(0.0),
            white_balance: graph.basic.white_balance,
            temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
            tint: graph.basic.tint.as_f64().unwrap_or(0.0),
            highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
            shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
            whites: graph.basic.whites.as_f64().unwrap_or(0.0),
            blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
            vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
            saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
            tone_curve: tone_curve_state_from_graph(&graph),
            hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
            detail: detail_state_from_graph(&graph),
            geometry: geometry_state_from_graph(&graph),
            masks: photo_manual_masks_from_graph(&graph),
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
        photo_id: graph.source.photo_id.clone(),
        exposure: graph.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: graph.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
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
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Jpeg,
        color_profile,
        PhotoExportMetadataPolicy::Minimal,
    )
}

/// Export one edited catalog photo as a JPEG file with explicit metadata policy.
pub fn export_photo_jpeg_with_metadata_policy(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    color_profile: PhotoExportColorProfile,
    metadata_policy: PhotoExportMetadataPolicy,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Jpeg,
        color_profile,
        metadata_policy,
    )
}

/// List recent export records with current output file evidence.
pub fn list_recent_exports(
    library_root_path: impl AsRef<Path>,
    limit: usize,
) -> Result<Vec<PhotoRecentExport>, CoreError> {
    let records = silica_storage::list_recent_export_records(library_root_path, limit)?;
    Ok(records
        .into_iter()
        .map(|record| {
            let output_exists = Path::new(&record.output_path).is_file();
            PhotoRecentExport {
                export_record_id: record.id,
                photo_id: record.photo_id,
                output_path: record.output_path,
                export_settings_json: record.export_settings_json,
                created_at: record.created_at,
                output_exists,
            }
        })
        .collect())
}

/// Export one edited catalog photo as a PNG sRGB file and record the export.
pub fn export_photo_png(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Png,
        PhotoExportColorProfile::Srgb,
        PhotoExportMetadataPolicy::Minimal,
    )
}

/// Export one edited catalog photo as a TIFF sRGB file and record the export.
pub fn export_photo_tiff(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Tiff,
        PhotoExportColorProfile::Srgb,
        PhotoExportMetadataPolicy::Minimal,
    )
}

fn export_photo_raster(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    format: PhotoExportFormat,
    color_profile: PhotoExportColorProfile,
    metadata_policy: PhotoExportMetadataPolicy,
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
    let render_masks = render_manual_masks_from_graph(&graph)?;
    let exposure = graph.basic.exposure.as_f64().unwrap_or(0.0);
    let contrast = graph.basic.contrast.as_f64().unwrap_or(0.0);
    let render_request = silica_render::plan_jpeg_srgb_export_with_color_presence(
        render_plan.source_path.clone(),
        output_path.display().to_string(),
        exposure,
        contrast,
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        LOCAL_ALPHA_JPEG_QUALITY,
    );
    let render_request = silica_render::plan_jpeg_srgb_export_with_tone_curve(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_tone_curve_from_graph(&graph),
        render_request.quality,
    );
    let render_request = silica_render::plan_jpeg_srgb_export_with_geometry(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_request.tone_curve,
        render_hsl_color_mixer_from_graph(&graph),
        render_detail_from_graph(&graph),
        render_geometry_from_graph(&graph),
        render_request.quality,
    );
    if !render_request.detail.is_neutral() {
        return Err(CoreError::ExportBlocked(render_request.message));
    }
    ensure_supported_lens_geometry_export(&graph, &render_request.geometry)?;
    record_brush_mask_raster_caches(library_root_path, &photo_id, &render_masks)?;
    let export_masks = export_manual_masks_from_render(&render_masks);

    let source_path = PathBuf::from(&render_request.source_path);
    let export_white_balance = export_white_balance_from_render(render_request.white_balance);
    let export_tone_recovery = export_tone_recovery_from_render(render_request.tone_recovery);
    let export_color_presence = export_color_presence_from_render(render_request.color_presence);
    let export_tone_curve = export_tone_curve_from_render(render_request.tone_curve.clone());
    let export_hsl_color_mixer = export_hsl_color_mixer_from_render(render_request.hsl_color_mixer);
    let export_detail = export_detail_from_render(render_request.detail);
    let export_geometry = export_geometry_from_render(render_request.geometry.clone());

    let (
        exported_output_path,
        exported_format,
        exported_color_profile,
        bytes_written,
        source_sha256,
        output_sha256,
        icc_profile_embedded,
        icc_profile_sha256,
        source_metadata_segments,
        output_metadata_segments,
        source_metadata_copied,
        gps_metadata_removed,
    ) = match format {
        PhotoExportFormat::Jpeg => {
            let export_result = silica_export::export_jpeg_with_metadata_policy(
                silica_export::JpegColorExportRequest {
                    source_path,
                    output_path: output_path.to_path_buf(),
                    exposure: render_request.exposure,
                    contrast: render_request.contrast,
                    white_balance: export_white_balance,
                    tone_recovery: export_tone_recovery,
                    color_presence: export_color_presence,
                    tone_curve: export_tone_curve,
                    hsl_color_mixer: export_hsl_color_mixer,
                    detail: export_detail,
                    geometry: export_geometry,
                    masks: export_masks,
                    quality: render_request.quality,
                    color_profile: export_color_profile_to_export(color_profile),
                },
                export_metadata_policy_to_export(metadata_policy),
            )?;
            (
                export_result.output_path,
                export_format_string(export_result.format).to_string(),
                export_color_profile_string(export_result.color_profile).to_string(),
                export_result.bytes_written,
                export_result.source_sha256,
                export_result.output_sha256,
                export_result.icc_profile_embedded,
                Some(export_result.icc_profile_sha256),
                export_result.source_metadata_segments,
                export_result.output_metadata_segments,
                export_result.source_metadata_copied,
                export_result.gps_metadata_removed,
            )
        }
        PhotoExportFormat::Png | PhotoExportFormat::Tiff => {
            let export_result =
                silica_export::export_raster_srgb(silica_export::RasterSrgbExportRequest {
                    source_path,
                    output_path: output_path.to_path_buf(),
                    format: export_raster_format_to_export(format),
                    exposure: render_request.exposure,
                    contrast: render_request.contrast,
                    white_balance: export_white_balance,
                    tone_recovery: export_tone_recovery,
                    color_presence: export_color_presence,
                    tone_curve: export_tone_curve,
                    hsl_color_mixer: export_hsl_color_mixer,
                    detail: export_detail,
                    geometry: export_geometry,
                    masks: export_masks,
                })?;
            (
                export_result.output_path,
                export_format_string(export_result.format).to_string(),
                export_color_profile_string(export_result.color_profile).to_string(),
                export_result.bytes_written,
                export_result.source_sha256,
                export_result.output_sha256,
                export_result.icc_profile_embedded,
                export_result.icc_profile_sha256,
                0,
                0,
                false,
                false,
            )
        }
    };
    let icc_profile_sha256_value = icc_profile_sha256
        .as_deref()
        .map_or(serde_json::Value::Null, serde_json::Value::from);
    let settings_value = serde_json::json!({
        "format": exported_format.clone(),
        "color_profile": exported_color_profile.clone(),
        "quality": render_request.quality,
        "metadata_policy": export_metadata_policy_string(metadata_policy),
        "exposure": render_request.exposure,
        "contrast": render_request.contrast,
        "white_balance": white_balance_render_mode_string(render_request.white_balance.mode),
        "temperature": render_request.white_balance.temperature,
        "tint": render_request.white_balance.tint,
        "highlights": render_request.tone_recovery.highlights,
        "shadows": render_request.tone_recovery.shadows,
        "whites": render_request.tone_recovery.whites,
        "blacks": render_request.tone_recovery.blacks,
        "vibrance": render_request.color_presence.vibrance,
        "saturation": render_request.color_presence.saturation,
        "tone_curve": tone_curve_settings_json(&render_request.tone_curve),
        "hsl_color_mixer": hsl_color_mixer_settings_json(&render_request.hsl_color_mixer),
        "detail": detail_settings_json(&render_request.detail),
        "geometry": geometry_settings_json(&render_request.geometry),
        "masks": manual_mask_settings_json(&render_masks),
        "source_path": render_request.source_path,
        "output_path": render_request.output_path,
        "source_sha256": source_sha256.clone(),
        "output_sha256": output_sha256.clone(),
        "icc_profile_embedded": icc_profile_embedded,
        "icc_profile_sha256": icc_profile_sha256_value,
        "profile_metadata_source": export_profile_metadata_source(format),
        "source_metadata_segments": source_metadata_segments,
        "output_metadata_segments": output_metadata_segments,
        "source_metadata_copied": source_metadata_copied,
        "gps_metadata_removed": gps_metadata_removed,
    });
    let settings_json = settings_value.to_string();
    let export_record = silica_storage::record_export(
        library_root_path,
        &photo_id,
        &exported_output_path,
        settings_json,
    )?;
    let export_record_id = export_record.id;
    append_core_action_log(
        library_root_path,
        "export",
        Some("photo"),
        Some(photo_id.clone()),
        "file_write",
        Some(export_record_id.clone()),
        settings_value,
    )?;

    Ok(Some(PhotoExportSession {
        photo_id,
        source_path: render_plan.source_path,
        output_path: exported_output_path,
        format: exported_format,
        color_profile: exported_color_profile,
        bytes_written,
        source_sha256: Some(source_sha256),
        output_sha256,
        icc_profile_embedded,
        icc_profile_sha256: icc_profile_sha256.unwrap_or_default(),
        decoder_backend: None,
        input_profile: None,
        working_space: None,
        export_record_id,
        message: export_raster_message(format, color_profile).to_string(),
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
    ensure_no_active_manual_masks_for_export(&graph)?;
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
    let render_request = silica_render::plan_raw_derived_jpeg_srgb_export_with_color_presence(
        source_artifact.artifact_path.display().to_string(),
        output_path.display().to_string(),
        exposure,
        contrast,
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        LOCAL_ALPHA_JPEG_QUALITY,
    );
    let render_request = silica_render::plan_raw_derived_jpeg_srgb_export_with_tone_curve(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_tone_curve_from_graph(&graph),
        render_request.quality,
    );
    let mut render_request = silica_render::plan_raw_derived_jpeg_srgb_export_with_hsl_color_mixer(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_request.tone_curve,
        render_hsl_color_mixer_from_graph(&graph),
        render_request.quality,
    );
    render_request.detail = render_detail_from_graph(&graph);
    render_request.geometry = render_geometry_from_graph(&graph);
    if !render_request.detail.is_neutral() {
        render_request.message =
            "Detail export unsupported until renderer support exists.".to_string();
    }
    if !render_request.detail.is_neutral() {
        return Err(CoreError::ExportBlocked(render_request.message));
    }
    ensure_supported_lens_geometry_export(&graph, &render_request.geometry)?;
    let export_result =
        silica_export::export_jpeg_with_color_profile(silica_export::JpegColorExportRequest {
            source_path: PathBuf::from(&render_request.source_path),
            output_path: output_path.to_path_buf(),
            exposure: render_request.exposure,
            contrast: render_request.contrast,
            white_balance: export_white_balance_from_render(render_request.white_balance),
            tone_recovery: export_tone_recovery_from_render(render_request.tone_recovery),
            color_presence: export_color_presence_from_render(render_request.color_presence),
            tone_curve: export_tone_curve_from_render(render_request.tone_curve.clone()),
            hsl_color_mixer: export_hsl_color_mixer_from_render(render_request.hsl_color_mixer),
            detail: export_detail_from_render(render_request.detail),
            geometry: export_geometry_from_render(render_request.geometry.clone()),
            masks: Vec::new(),
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
    let settings_value = serde_json::json!({
        "format": format,
        "color_profile": exported_color_profile,
        "quality": render_request.quality,
        "exposure": render_request.exposure,
        "contrast": render_request.contrast,
        "white_balance": white_balance_render_mode_string(render_request.white_balance.mode),
        "temperature": render_request.white_balance.temperature,
        "tint": render_request.white_balance.tint,
        "highlights": render_request.tone_recovery.highlights,
        "shadows": render_request.tone_recovery.shadows,
        "whites": render_request.tone_recovery.whites,
        "blacks": render_request.tone_recovery.blacks,
        "vibrance": render_request.color_presence.vibrance,
        "saturation": render_request.color_presence.saturation,
        "tone_curve": tone_curve_settings_json(&render_request.tone_curve),
        "hsl_color_mixer": hsl_color_mixer_settings_json(&render_request.hsl_color_mixer),
        "detail": detail_settings_json(&render_request.detail),
        "geometry": geometry_settings_json(&render_request.geometry),
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
    });
    let settings_json = settings_value.to_string();
    let export_record = silica_storage::record_export(
        library_root_path,
        &candidate.photo_id,
        &export_result.output_path,
        settings_json,
    )?;
    let export_record_id = export_record.id;
    append_core_action_log(
        library_root_path,
        "export",
        Some("photo"),
        Some(candidate.photo_id.clone()),
        "file_write",
        Some(export_record_id.clone()),
        settings_value,
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
        export_record_id,
        message: "RAW-derived JPEG sRGB export completed.".to_string(),
    }))
}

/// Clear disposable library cache data without removing catalog or original files.
pub fn clear_library_cache(
    library_root_path: impl AsRef<Path>,
) -> Result<LibraryCacheClearSession, CoreError> {
    let library_root_path = library_root_path.as_ref().to_path_buf();
    let summary = silica_storage::clear_disposable_cache(&library_root_path)?;
    append_core_action_log(
        &library_root_path,
        "cache_clear",
        Some("library"),
        Some(library_root_path.display().to_string()),
        "cache_delete",
        Some("disposable-cache".to_string()),
        serde_json::json!({
            "cleared_directories": summary.cleared_directories.clone(),
            "recreated_directories": summary.recreated_directories.clone(),
            "removed_cache_records": summary.removed_cache_records,
        }),
    )?;
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

fn manual_mask_adjustments(
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> silica_edit::ManualMaskLocalAdjustments {
    silica_edit::ManualMaskLocalAdjustments { exposure, contrast }
}

fn edit_brush_strokes(
    strokes: Vec<PhotoManualBrushStrokeInput>,
) -> Result<Vec<silica_edit::MaskBrushStroke>, CoreError> {
    strokes
        .into_iter()
        .map(|stroke| {
            let points = stroke
                .points
                .into_iter()
                .map(|point| (point.x, point.y))
                .collect::<Vec<_>>();
            silica_edit::manual_brush_stroke(stroke.id, stroke.radius, points)
                .map_err(CoreError::from)
        })
        .collect()
}

fn preview_manual_mask_edit(
    library_root_path: &Path,
    photo_id: &str,
    edited: &silica_edit::EditGraph,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_manual_mask_preview(
        render_plan,
        edited.basic.exposure.as_f64().unwrap_or(0.0),
        edited.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(edited),
        render_tone_recovery_from_graph(edited),
        render_color_presence_from_graph(edited),
        render_tone_curve_from_graph(edited),
        render_hsl_color_mixer_from_graph(edited),
        render_detail_from_graph(edited),
        render_geometry_from_graph(edited),
        render_manual_masks_from_graph(edited)?,
    );
    let request = apply_lens_geometry_preview_boundary(request, edited);
    let request = apply_manual_mask_preview_boundary(request, edited)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry.clone()),
            request.masks.clone(),
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
        white_balance: edited.basic.white_balance,
        temperature: edited.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: edited.basic.tint.as_f64().unwrap_or(0.0),
        highlights: edited.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: edited.basic.shadows.as_f64().unwrap_or(0.0),
        whites: edited.basic.whites.as_f64().unwrap_or(0.0),
        blacks: edited.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: edited.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: edited.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(edited),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(edited),
        detail: detail_state_from_graph(edited),
        geometry: geometry_state_from_graph(edited),
        masks: photo_manual_masks_from_graph(edited),
        message,
    }))
}

fn preview_detail_edit(
    library_root_path: &Path,
    photo_id: &str,
    graph: &silica_edit::EditGraph,
    edited: &silica_edit::EditGraph,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_detail_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(graph),
        render_tone_recovery_from_graph(graph),
        render_color_presence_from_graph(graph),
        render_tone_curve_from_graph(graph),
        render_hsl_color_mixer_from_graph(graph),
        render_detail_from_graph(edited),
    );
    let request = apply_lens_geometry_preview_boundary(request, edited);
    let request = apply_manual_mask_preview_boundary(request, edited)?;
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(graph),
        detail: detail_state_from_graph(edited),
        geometry: geometry_state_from_graph(edited),
        masks: photo_manual_masks_from_graph(edited),
        message,
    }))
}

fn preview_geometry_edit(
    library_root_path: &Path,
    photo_id: &str,
    graph: &silica_edit::EditGraph,
    edited: &silica_edit::EditGraph,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_geometry_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(graph),
        render_tone_recovery_from_graph(graph),
        render_color_presence_from_graph(graph),
        render_tone_curve_from_graph(graph),
        render_hsl_color_mixer_from_graph(graph),
        render_detail_from_graph(graph),
        render_geometry_from_graph(edited),
    );
    let request = apply_lens_geometry_preview_boundary(request, edited);
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
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
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
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(graph),
        detail: detail_state_from_graph(graph),
        geometry: geometry_state_from_graph(edited),
        masks: photo_manual_masks_from_graph(edited),
        message,
    }))
}

fn photo_edit_commit_from_graph(
    persisted: &silica_edit::EditGraph,
    message: impl Into<String>,
) -> PhotoEditCommit {
    PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(persisted),
        detail: detail_state_from_graph(persisted),
        geometry: geometry_state_from_graph(persisted),
        masks: photo_manual_masks_from_graph(persisted),
        persisted: true,
        message: message.into(),
    }
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

fn record_brush_mask_raster_caches(
    library_root_path: &Path,
    photo_id: &str,
    masks: &[silica_render::ManualMaskRenderAdjustment],
) -> Result<(), CoreError> {
    let mask_root = library_root_path.join("render-cache").join("masks");
    for mask in masks {
        let silica_render::ManualMaskRenderGeometry::BrushRaster {
            alpha, cache_key, ..
        } = &mask.geometry
        else {
            continue;
        };
        std::fs::create_dir_all(&mask_root)
            .map_err(silica_storage::LibraryStorageError::from)
            .map_err(CoreError::from)?;
        let file_name = format!(
            "{}-{}-{}.mask8",
            safe_cache_file_component(photo_id),
            safe_cache_file_component(&mask.id),
            safe_cache_file_component(cache_key)
        );
        let path = mask_root.join(file_name);
        std::fs::write(&path, alpha)
            .map_err(silica_storage::LibraryStorageError::from)
            .map_err(CoreError::from)?;
        let byte_size = i64::try_from(alpha.len()).unwrap_or(i64::MAX);
        silica_storage::record_mask_raster_cache(
            library_root_path,
            photo_id,
            cache_key,
            &path,
            byte_size,
        )?;
    }
    Ok(())
}

fn safe_cache_file_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn write_jpeg_develop_preview_bytes(
    library_root_path: &Path,
    photo_id: &str,
    source_path: &str,
    exposure: f64,
    contrast: f64,
    white_balance: silica_export::WhiteBalanceAdjustment,
    tone_recovery: silica_export::ToneRecoveryAdjustment,
    color_presence: silica_export::ColorPresenceAdjustment,
    tone_curve: silica_export::ToneCurveAdjustment,
    hsl_color_mixer: silica_export::HslColorMixerAdjustment,
    detail: silica_export::DetailAdjustment,
    geometry: silica_export::GeometryAdjustment,
    masks: Vec<silica_render::ManualMaskRenderAdjustment>,
) -> Result<Option<Vec<u8>>, CoreError> {
    let source_path = PathBuf::from(source_path);
    if !is_jpeg_path(&source_path) || !source_path.is_file() {
        return Ok(None);
    }

    record_brush_mask_raster_caches(library_root_path, photo_id, &masks)?;
    let masks = export_manual_masks_from_render(&masks);

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
            white_balance,
            tone_recovery,
            color_presence,
            tone_curve,
            hsl_color_mixer,
            detail,
            geometry,
            masks,
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
        "appearance": {
            "theme": app_appearance_theme_string(session.appearance.theme),
            "density": app_appearance_density_string(session.appearance.density),
            "ui_scale": session.appearance.ui_scale,
        },
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

fn parse_app_appearance_theme(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppAppearanceTheme {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("dark") => AppAppearanceTheme::Dark,
        Some("light") => AppAppearanceTheme::Light,
        Some(_) => {
            *invalid_values = true;
            AppAppearanceTheme::Dark
        }
    }
}

fn app_appearance_theme_string(theme: AppAppearanceTheme) -> &'static str {
    match theme {
        AppAppearanceTheme::Dark => "dark",
        AppAppearanceTheme::Light => "light",
    }
}

fn parse_app_appearance_density(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppAppearanceDensity {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("compact") => AppAppearanceDensity::Compact,
        Some("comfortable") => AppAppearanceDensity::Comfortable,
        Some(_) => {
            *invalid_values = true;
            AppAppearanceDensity::Compact
        }
    }
}

fn app_appearance_density_string(density: AppAppearanceDensity) -> &'static str {
    match density {
        AppAppearanceDensity::Compact => "compact",
        AppAppearanceDensity::Comfortable => "comfortable",
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

fn parse_app_appearance(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppAppearancePreferences {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return AppAppearancePreferences::default();
    };

    AppAppearancePreferences {
        theme: parse_app_appearance_theme(object.get("theme"), invalid_values),
        density: parse_app_appearance_density(object.get("density"), invalid_values),
        ui_scale: parse_ui_scale(object.get("ui_scale"), invalid_values),
    }
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

fn parse_ui_scale(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> u16 {
    let Some(value) = value else {
        return DEFAULT_APP_SESSION_UI_SCALE;
    };
    let Some(value) = value.as_i64() else {
        *invalid_values = true;
        return DEFAULT_APP_SESSION_UI_SCALE;
    };
    if value < MIN_APP_SESSION_UI_SCALE as i64 || value > MAX_APP_SESSION_UI_SCALE as i64 {
        *invalid_values = true;
    }
    value.clamp(
        MIN_APP_SESSION_UI_SCALE as i64,
        MAX_APP_SESSION_UI_SCALE as i64,
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

fn histogram_cache_key(
    photo_id: &str,
    source_path: &Path,
    graph: &silica_edit::EditGraph,
) -> String {
    let metadata = std::fs::metadata(source_path).ok();
    let file_size = metadata.as_ref().map(std::fs::Metadata::len).unwrap_or(0);
    let modified = metadata
        .and_then(|metadata| metadata.modified().ok())
        .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!(
        "histogram:v1:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        photo_id,
        source_path.display(),
        file_size,
        modified,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance_render_mode_string(render_white_balance_from_graph(graph).mode),
        graph.basic.temperature.as_f64().unwrap_or(5200.0),
        graph.basic.tint.as_f64().unwrap_or(0.0),
        graph.basic.highlights.as_f64().unwrap_or(0.0),
        graph.basic.shadows.as_f64().unwrap_or(0.0),
        graph.basic.whites.as_f64().unwrap_or(0.0),
        graph.basic.blacks.as_f64().unwrap_or(0.0),
        graph.basic.vibrance.as_f64().unwrap_or(0.0),
        graph.basic.saturation.as_f64().unwrap_or(0.0)
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
        silica_export::ExportImageFormat::Png => "png",
        silica_export::ExportImageFormat::Tiff => "tiff",
    }
}

fn export_raster_format_to_export(format: PhotoExportFormat) -> silica_export::ExportImageFormat {
    match format {
        PhotoExportFormat::Jpeg => silica_export::ExportImageFormat::Jpeg,
        PhotoExportFormat::Png => silica_export::ExportImageFormat::Png,
        PhotoExportFormat::Tiff => silica_export::ExportImageFormat::Tiff,
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

fn export_metadata_policy_to_export(
    policy: PhotoExportMetadataPolicy,
) -> silica_export::ExportMetadataPolicy {
    match policy {
        PhotoExportMetadataPolicy::Minimal => silica_export::ExportMetadataPolicy::Minimal,
        PhotoExportMetadataPolicy::Preserve => silica_export::ExportMetadataPolicy::Preserve,
        PhotoExportMetadataPolicy::RemoveGps => silica_export::ExportMetadataPolicy::RemoveGps,
        PhotoExportMetadataPolicy::RemoveAll => silica_export::ExportMetadataPolicy::RemoveAll,
    }
}

fn export_metadata_policy_string(policy: PhotoExportMetadataPolicy) -> &'static str {
    match policy {
        PhotoExportMetadataPolicy::Minimal => "minimal",
        PhotoExportMetadataPolicy::Preserve => "preserve",
        PhotoExportMetadataPolicy::RemoveGps => "remove_gps",
        PhotoExportMetadataPolicy::RemoveAll => "remove_all",
    }
}

fn render_white_balance_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::WhiteBalanceRenderAdjustment {
    silica_render::WhiteBalanceRenderAdjustment {
        mode: render_white_balance_mode(graph.basic.white_balance),
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
    }
}

fn render_white_balance_mode(
    mode: silica_edit::WhiteBalance,
) -> silica_render::WhiteBalanceRenderMode {
    match mode {
        silica_edit::WhiteBalance::AsShot => silica_render::WhiteBalanceRenderMode::AsShot,
        silica_edit::WhiteBalance::Auto => silica_render::WhiteBalanceRenderMode::Auto,
        silica_edit::WhiteBalance::Daylight => silica_render::WhiteBalanceRenderMode::Daylight,
        silica_edit::WhiteBalance::Cloudy => silica_render::WhiteBalanceRenderMode::Cloudy,
        silica_edit::WhiteBalance::Shade => silica_render::WhiteBalanceRenderMode::Shade,
        silica_edit::WhiteBalance::Tungsten => silica_render::WhiteBalanceRenderMode::Tungsten,
        silica_edit::WhiteBalance::Fluorescent => {
            silica_render::WhiteBalanceRenderMode::Fluorescent
        }
        silica_edit::WhiteBalance::Flash => silica_render::WhiteBalanceRenderMode::Flash,
        silica_edit::WhiteBalance::Custom => silica_render::WhiteBalanceRenderMode::Custom,
    }
}

fn export_white_balance_from_render(
    white_balance: silica_render::WhiteBalanceRenderAdjustment,
) -> silica_export::WhiteBalanceAdjustment {
    silica_export::WhiteBalanceAdjustment {
        mode: export_white_balance_mode(white_balance.mode),
        temperature: white_balance.temperature,
        tint: white_balance.tint,
    }
}

fn render_tone_recovery_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::ToneRecoveryRenderAdjustment {
    silica_render::ToneRecoveryRenderAdjustment {
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
    }
}

fn export_tone_recovery_from_render(
    tone_recovery: silica_render::ToneRecoveryRenderAdjustment,
) -> silica_export::ToneRecoveryAdjustment {
    silica_export::ToneRecoveryAdjustment {
        highlights: tone_recovery.highlights,
        shadows: tone_recovery.shadows,
        whites: tone_recovery.whites,
        blacks: tone_recovery.blacks,
    }
}

fn tone_curve_state_from_graph(graph: &silica_edit::EditGraph) -> PhotoToneCurveState {
    PhotoToneCurveState {
        curve_mode: graph.tone.curve_mode,
        rgb_curve: photo_tone_curve_points_from_edit(&graph.tone.rgb_curve),
        red_curve: photo_tone_curve_points_from_edit(&graph.tone.red_curve),
        green_curve: photo_tone_curve_points_from_edit(&graph.tone.green_curve),
        blue_curve: photo_tone_curve_points_from_edit(&graph.tone.blue_curve),
    }
}

fn photo_tone_curve_points_from_edit(
    points: &[silica_edit::CurvePoint],
) -> Vec<PhotoToneCurvePoint> {
    points
        .iter()
        .map(|point| PhotoToneCurvePoint {
            x: point.x.as_f64().unwrap_or(0.0),
            y: point.y.as_f64().unwrap_or(0.0),
        })
        .collect()
}

fn render_tone_curve_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::ToneCurveRenderAdjustment {
    silica_render::ToneCurveRenderAdjustment {
        mode: match graph.tone.curve_mode {
            silica_edit::CurveMode::None => silica_render::ToneCurveRenderMode::None,
            silica_edit::CurveMode::Point => silica_render::ToneCurveRenderMode::Point,
            silica_edit::CurveMode::Parametric => silica_render::ToneCurveRenderMode::Parametric,
        },
        rgb_curve: render_tone_curve_points_from_edit(&graph.tone.rgb_curve),
        red_curve: render_tone_curve_points_from_edit(&graph.tone.red_curve),
        green_curve: render_tone_curve_points_from_edit(&graph.tone.green_curve),
        blue_curve: render_tone_curve_points_from_edit(&graph.tone.blue_curve),
    }
}

fn render_tone_curve_points_from_edit(
    points: &[silica_edit::CurvePoint],
) -> Vec<silica_render::ToneCurveRenderPoint> {
    points
        .iter()
        .map(|point| silica_render::ToneCurveRenderPoint {
            x: point.x.as_f64().unwrap_or(0.0),
            y: point.y.as_f64().unwrap_or(0.0),
        })
        .collect()
}

fn export_tone_curve_from_render(
    tone_curve: silica_render::ToneCurveRenderAdjustment,
) -> silica_export::ToneCurveAdjustment {
    silica_export::ToneCurveAdjustment {
        mode: match tone_curve.mode {
            silica_render::ToneCurveRenderMode::None => silica_export::ToneCurveMode::None,
            silica_render::ToneCurveRenderMode::Parametric => {
                silica_export::ToneCurveMode::Parametric
            }
            silica_render::ToneCurveRenderMode::Point => silica_export::ToneCurveMode::Point,
        },
        rgb_curve: tone_curve_points_to_export(tone_curve.rgb_curve),
        red_curve: tone_curve_points_to_export(tone_curve.red_curve),
        green_curve: tone_curve_points_to_export(tone_curve.green_curve),
        blue_curve: tone_curve_points_to_export(tone_curve.blue_curve),
    }
}

fn tone_curve_points_to_export(
    points: Vec<silica_render::ToneCurveRenderPoint>,
) -> Vec<silica_export::ToneCurvePoint> {
    points
        .into_iter()
        .map(|point| silica_export::ToneCurvePoint {
            x: point.x,
            y: point.y,
        })
        .collect()
}

fn tone_curve_settings_json(
    tone_curve: &silica_render::ToneCurveRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "curve_mode": match tone_curve.mode {
            silica_render::ToneCurveRenderMode::None => "none",
            silica_render::ToneCurveRenderMode::Parametric => "parametric",
            silica_render::ToneCurveRenderMode::Point => "point",
        },
        "rgb_curve": tone_curve_points_json(&tone_curve.rgb_curve),
        "red_curve": tone_curve_points_json(&tone_curve.red_curve),
        "green_curve": tone_curve_points_json(&tone_curve.green_curve),
        "blue_curve": tone_curve_points_json(&tone_curve.blue_curve),
    })
}

fn tone_curve_points_json(
    points: &[silica_render::ToneCurveRenderPoint],
) -> Vec<serde_json::Value> {
    points
        .iter()
        .map(|point| serde_json::json!({ "x": point.x, "y": point.y }))
        .collect()
}

fn hsl_color_mixer_state_from_graph(graph: &silica_edit::EditGraph) -> PhotoHslColorMixerState {
    PhotoHslColorMixerState {
        red: photo_hsl_color_channel_from_edit(&graph.color.hsl.red),
        orange: photo_hsl_color_channel_from_edit(&graph.color.hsl.orange),
        yellow: photo_hsl_color_channel_from_edit(&graph.color.hsl.yellow),
        green: photo_hsl_color_channel_from_edit(&graph.color.hsl.green),
        aqua: photo_hsl_color_channel_from_edit(&graph.color.hsl.aqua),
        blue: photo_hsl_color_channel_from_edit(&graph.color.hsl.blue),
        purple: photo_hsl_color_channel_from_edit(&graph.color.hsl.purple),
        magenta: photo_hsl_color_channel_from_edit(&graph.color.hsl.magenta),
    }
}

fn photo_hsl_color_channel_from_edit(
    channel: &silica_edit::HslChannel,
) -> PhotoHslColorChannelState {
    PhotoHslColorChannelState {
        hue: channel.hue.as_f64().unwrap_or(0.0),
        saturation: channel.saturation.as_f64().unwrap_or(0.0),
        luminance: channel.luminance.as_f64().unwrap_or(0.0),
    }
}

fn render_hsl_color_mixer_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::HslColorMixerRenderAdjustment {
    silica_render::HslColorMixerRenderAdjustment {
        red: render_hsl_color_channel_from_edit(&graph.color.hsl.red),
        orange: render_hsl_color_channel_from_edit(&graph.color.hsl.orange),
        yellow: render_hsl_color_channel_from_edit(&graph.color.hsl.yellow),
        green: render_hsl_color_channel_from_edit(&graph.color.hsl.green),
        aqua: render_hsl_color_channel_from_edit(&graph.color.hsl.aqua),
        blue: render_hsl_color_channel_from_edit(&graph.color.hsl.blue),
        purple: render_hsl_color_channel_from_edit(&graph.color.hsl.purple),
        magenta: render_hsl_color_channel_from_edit(&graph.color.hsl.magenta),
    }
}

fn render_hsl_color_channel_from_edit(
    channel: &silica_edit::HslChannel,
) -> silica_render::HslColorChannelRenderAdjustment {
    silica_render::HslColorChannelRenderAdjustment {
        hue: channel.hue.as_f64().unwrap_or(0.0),
        saturation: channel.saturation.as_f64().unwrap_or(0.0),
        luminance: channel.luminance.as_f64().unwrap_or(0.0),
    }
}

fn export_hsl_color_mixer_from_render(
    hsl_color_mixer: silica_render::HslColorMixerRenderAdjustment,
) -> silica_export::HslColorMixerAdjustment {
    silica_export::HslColorMixerAdjustment {
        red: hsl_color_channel_to_export(hsl_color_mixer.red),
        orange: hsl_color_channel_to_export(hsl_color_mixer.orange),
        yellow: hsl_color_channel_to_export(hsl_color_mixer.yellow),
        green: hsl_color_channel_to_export(hsl_color_mixer.green),
        aqua: hsl_color_channel_to_export(hsl_color_mixer.aqua),
        blue: hsl_color_channel_to_export(hsl_color_mixer.blue),
        purple: hsl_color_channel_to_export(hsl_color_mixer.purple),
        magenta: hsl_color_channel_to_export(hsl_color_mixer.magenta),
    }
}

fn hsl_color_channel_to_export(
    channel: silica_render::HslColorChannelRenderAdjustment,
) -> silica_export::HslColorChannelAdjustment {
    silica_export::HslColorChannelAdjustment {
        hue: channel.hue,
        saturation: channel.saturation,
        luminance: channel.luminance,
    }
}

fn hsl_color_mixer_settings_json(
    hsl_color_mixer: &silica_render::HslColorMixerRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "red": hsl_color_channel_settings_json(hsl_color_mixer.red),
        "orange": hsl_color_channel_settings_json(hsl_color_mixer.orange),
        "yellow": hsl_color_channel_settings_json(hsl_color_mixer.yellow),
        "green": hsl_color_channel_settings_json(hsl_color_mixer.green),
        "aqua": hsl_color_channel_settings_json(hsl_color_mixer.aqua),
        "blue": hsl_color_channel_settings_json(hsl_color_mixer.blue),
        "purple": hsl_color_channel_settings_json(hsl_color_mixer.purple),
        "magenta": hsl_color_channel_settings_json(hsl_color_mixer.magenta),
    })
}

fn hsl_color_channel_settings_json(
    channel: silica_render::HslColorChannelRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "hue": channel.hue,
        "saturation": channel.saturation,
        "luminance": channel.luminance,
    })
}

fn detail_state_from_graph(graph: &silica_edit::EditGraph) -> PhotoDetailState {
    PhotoDetailState {
        sharpening: PhotoDetailSharpeningState {
            amount: graph.detail.sharpening.amount.as_f64().unwrap_or(0.0),
            radius: graph.detail.sharpening.radius.as_f64().unwrap_or(1.0),
            detail: graph.detail.sharpening.detail.as_f64().unwrap_or(25.0),
            masking: graph.detail.sharpening.masking.as_f64().unwrap_or(0.0),
        },
        noise_reduction: PhotoDetailNoiseReductionState {
            luminance: graph
                .detail
                .noise_reduction
                .luminance
                .as_f64()
                .unwrap_or(0.0),
            detail: graph.detail.noise_reduction.detail.as_f64().unwrap_or(50.0),
            contrast: graph
                .detail
                .noise_reduction
                .contrast
                .as_f64()
                .unwrap_or(0.0),
            color: graph.detail.noise_reduction.color.as_f64().unwrap_or(25.0),
            color_detail: graph
                .detail
                .noise_reduction
                .color_detail
                .as_f64()
                .unwrap_or(50.0),
        },
    }
}

fn render_detail_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::DetailRenderAdjustment {
    silica_render::DetailRenderAdjustment {
        sharpening: silica_render::DetailSharpeningRenderAdjustment {
            amount: graph.detail.sharpening.amount.as_f64().unwrap_or(0.0),
            radius: graph.detail.sharpening.radius.as_f64().unwrap_or(1.0),
            detail: graph.detail.sharpening.detail.as_f64().unwrap_or(25.0),
            masking: graph.detail.sharpening.masking.as_f64().unwrap_or(0.0),
        },
        noise_reduction: silica_render::DetailNoiseReductionRenderAdjustment {
            luminance: graph
                .detail
                .noise_reduction
                .luminance
                .as_f64()
                .unwrap_or(0.0),
            detail: graph.detail.noise_reduction.detail.as_f64().unwrap_or(50.0),
            contrast: graph
                .detail
                .noise_reduction
                .contrast
                .as_f64()
                .unwrap_or(0.0),
            color: graph.detail.noise_reduction.color.as_f64().unwrap_or(25.0),
            color_detail: graph
                .detail
                .noise_reduction
                .color_detail
                .as_f64()
                .unwrap_or(50.0),
        },
    }
}

fn export_detail_from_render(
    detail: silica_render::DetailRenderAdjustment,
) -> silica_export::DetailAdjustment {
    silica_export::DetailAdjustment {
        sharpening: silica_export::DetailSharpeningAdjustment {
            amount: detail.sharpening.amount,
            radius: detail.sharpening.radius,
            detail: detail.sharpening.detail,
            masking: detail.sharpening.masking,
        },
        noise_reduction: silica_export::DetailNoiseReductionAdjustment {
            luminance: detail.noise_reduction.luminance,
            detail: detail.noise_reduction.detail,
            contrast: detail.noise_reduction.contrast,
            color: detail.noise_reduction.color,
            color_detail: detail.noise_reduction.color_detail,
        },
    }
}

fn detail_settings_json(detail: &silica_render::DetailRenderAdjustment) -> serde_json::Value {
    serde_json::json!({
        "sharpening": {
            "amount": detail.sharpening.amount,
            "radius": detail.sharpening.radius,
            "detail": detail.sharpening.detail,
            "masking": detail.sharpening.masking,
        },
        "noise_reduction": {
            "luminance": detail.noise_reduction.luminance,
            "detail": detail.noise_reduction.detail,
            "contrast": detail.noise_reduction.contrast,
            "color": detail.noise_reduction.color,
            "color_detail": detail.noise_reduction.color_detail,
        },
        "mlx_denoise": "deferred",
    })
}

fn geometry_state_from_graph(graph: &silica_edit::EditGraph) -> PhotoGeometryState {
    PhotoGeometryState {
        crop: graph
            .geometry
            .crop
            .as_ref()
            .map(|crop| PhotoGeometryCropState {
                x: crop.x.as_f64().unwrap_or(0.0),
                y: crop.y.as_f64().unwrap_or(0.0),
                width: crop.width.as_f64().unwrap_or(1.0),
                height: crop.height.as_f64().unwrap_or(1.0),
                angle: crop.angle.as_f64().unwrap_or(0.0),
                aspect: crop.aspect.clone(),
            }),
        rotation: graph.geometry.rotation.as_f64().unwrap_or(0.0),
        flip_horizontal: graph.geometry.flip_horizontal,
        flip_vertical: graph.geometry.flip_vertical,
        transform: PhotoGeometryTransformState {
            vertical: graph.geometry.transform.vertical.as_f64().unwrap_or(0.0),
            horizontal: graph.geometry.transform.horizontal.as_f64().unwrap_or(0.0),
            aspect: graph.geometry.transform.aspect.as_f64().unwrap_or(0.0),
            scale: graph.geometry.transform.scale.as_f64().unwrap_or(100.0),
            x_offset: graph.geometry.transform.x_offset.as_f64().unwrap_or(0.0),
            y_offset: graph.geometry.transform.y_offset.as_f64().unwrap_or(0.0),
        },
    }
}

fn render_geometry_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::GeometryRenderAdjustment {
    silica_render::GeometryRenderAdjustment {
        crop: graph.geometry.crop.as_ref().map(|crop| {
            silica_render::GeometryCropRenderAdjustment {
                x: crop.x.as_f64().unwrap_or(0.0),
                y: crop.y.as_f64().unwrap_or(0.0),
                width: crop.width.as_f64().unwrap_or(1.0),
                height: crop.height.as_f64().unwrap_or(1.0),
                angle: crop.angle.as_f64().unwrap_or(0.0),
                aspect: crop.aspect.clone(),
            }
        }),
        rotation: graph.geometry.rotation.as_f64().unwrap_or(0.0),
        flip_horizontal: graph.geometry.flip_horizontal,
        flip_vertical: graph.geometry.flip_vertical,
        transform: silica_render::GeometryTransformRenderAdjustment {
            vertical: graph.geometry.transform.vertical.as_f64().unwrap_or(0.0),
            horizontal: graph.geometry.transform.horizontal.as_f64().unwrap_or(0.0),
            aspect: graph.geometry.transform.aspect.as_f64().unwrap_or(0.0),
            scale: graph.geometry.transform.scale.as_f64().unwrap_or(100.0),
            x_offset: graph.geometry.transform.x_offset.as_f64().unwrap_or(0.0),
            y_offset: graph.geometry.transform.y_offset.as_f64().unwrap_or(0.0),
        },
    }
}

fn export_geometry_from_render(
    geometry: silica_render::GeometryRenderAdjustment,
) -> silica_export::GeometryAdjustment {
    silica_export::GeometryAdjustment {
        crop: geometry
            .crop
            .map(|crop| silica_export::GeometryCropAdjustment {
                x: crop.x,
                y: crop.y,
                width: crop.width,
                height: crop.height,
                angle: crop.angle,
                aspect: crop.aspect,
            }),
        rotation: geometry.rotation,
        flip_horizontal: geometry.flip_horizontal,
        flip_vertical: geometry.flip_vertical,
        transform: silica_export::GeometryTransformAdjustment {
            vertical: geometry.transform.vertical,
            horizontal: geometry.transform.horizontal,
            aspect: geometry.transform.aspect,
            scale: geometry.transform.scale,
            x_offset: geometry.transform.x_offset,
            y_offset: geometry.transform.y_offset,
        },
    }
}

fn geometry_settings_json(geometry: &silica_render::GeometryRenderAdjustment) -> serde_json::Value {
    serde_json::json!({
        "crop": geometry.crop.as_ref().map(|crop| {
            serde_json::json!({
                "x": crop.x,
                "y": crop.y,
                "width": crop.width,
                "height": crop.height,
                "angle": crop.angle,
                "aspect": crop.aspect,
            })
        }),
        "rotation": geometry.rotation,
        "flip_horizontal": geometry.flip_horizontal,
        "flip_vertical": geometry.flip_vertical,
        "transform": {
            "vertical": geometry.transform.vertical,
            "horizontal": geometry.transform.horizontal,
            "aspect": geometry.transform.aspect,
            "scale": geometry.transform.scale,
            "x_offset": geometry.transform.x_offset,
            "y_offset": geometry.transform.y_offset,
        },
    })
}

fn manual_mask_settings_json(
    masks: &[silica_render::ManualMaskRenderAdjustment],
) -> serde_json::Value {
    serde_json::Value::Array(masks.iter().map(manual_mask_setting_json).collect())
}

fn manual_mask_setting_json(mask: &silica_render::ManualMaskRenderAdjustment) -> serde_json::Value {
    serde_json::json!({
        "id": mask.id,
        "kind": manual_mask_render_kind(&mask.geometry),
        "enabled": mask.enabled,
        "invert": mask.invert,
        "opacity": mask.opacity,
        "feather": mask.feather,
        "geometry": manual_mask_geometry_settings_json(&mask.geometry),
        "exposure": mask.exposure,
        "contrast": mask.contrast,
    })
}

fn manual_mask_render_kind(geometry: &silica_render::ManualMaskRenderGeometry) -> &'static str {
    match geometry {
        silica_render::ManualMaskRenderGeometry::LinearGradient { .. } => "linear_gradient",
        silica_render::ManualMaskRenderGeometry::RadialGradient { .. } => "radial_gradient",
        silica_render::ManualMaskRenderGeometry::BrushRaster { .. } => "brush",
    }
}

fn manual_mask_geometry_settings_json(
    geometry: &silica_render::ManualMaskRenderGeometry,
) -> serde_json::Value {
    match geometry {
        silica_render::ManualMaskRenderGeometry::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => serde_json::json!({
            "kind": "linear_gradient",
            "start_x": start_x,
            "start_y": start_y,
            "end_x": end_x,
            "end_y": end_y,
        }),
        silica_render::ManualMaskRenderGeometry::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        } => serde_json::json!({
            "kind": "radial_gradient",
            "center_x": center_x,
            "center_y": center_y,
            "radius_x": radius_x,
            "radius_y": radius_y,
            "rotation": rotation,
        }),
        silica_render::ManualMaskRenderGeometry::BrushRaster {
            width,
            height,
            cache_key,
            ..
        } => serde_json::json!({
            "kind": "brush_raster",
            "width": width,
            "height": height,
            "cache_key": cache_key,
        }),
    }
}

fn detail_unsupported_message() -> String {
    "Detail preview/export is unsupported until renderer support exists.".to_string()
}

fn prepare_edit_clipboard_sync(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<PreparedEditClipboardSync, CoreError> {
    silica_edit::validate_edit_clipboard_payload(payload)?;
    let library_root_path = library_root_path.as_ref();
    let requested_count = photo_ids.len();
    if photo_ids.is_empty() {
        return Ok(PreparedEditClipboardSync {
            plan: BatchEditClipboardSyncPlan {
                status: "empty".to_string(),
                requested_count,
                ready_count: 0,
                unchanged_count: 0,
                blocked_count: 0,
                targets: Vec::new(),
                message: "No target photos requested for batch sync.".to_string(),
            },
            ready_graphs: Vec::new(),
        });
    }

    let mut seen = BTreeSet::new();
    let mut targets = Vec::new();
    let mut ready_graphs = Vec::new();

    for photo_id in photo_ids {
        if photo_id.trim().is_empty() {
            targets.push(batch_sync_target(
                photo_id,
                "blocked",
                Some("empty_target"),
                "Photo id must not be empty.",
            ));
            continue;
        }
        if !seen.insert(photo_id.clone()) {
            targets.push(batch_sync_target(
                photo_id,
                "blocked",
                Some("duplicate_target"),
                "Duplicate target photo id.",
            ));
            continue;
        }

        if let Some((code, message)) =
            edit_clipboard_catalog_target_block(library_root_path, photo_id)?
        {
            targets.push(batch_sync_target(photo_id, "blocked", Some(code), &message));
            continue;
        }

        let target =
            match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
                Some(graph) => graph,
                None => {
                    targets.push(batch_sync_target(
                        photo_id,
                        "blocked",
                        Some("missing_photo"),
                        "Photo not found.",
                    ));
                    continue;
                }
            };
        let edited = match silica_edit::apply_edit_clipboard_payload(
            &target,
            payload,
            current_timestamp_string(),
        ) {
            Ok(graph) => graph,
            Err(error) => {
                targets.push(batch_sync_target(
                    photo_id,
                    "blocked",
                    Some("invalid_payload"),
                    &error.to_string(),
                ));
                continue;
            }
        };
        if let Err((code, message)) = ensure_supported_edit_clipboard_sync(payload, &edited) {
            targets.push(batch_sync_target(photo_id, "blocked", Some(code), &message));
            continue;
        }
        if edit_graphs_equal_ignoring_updated_at(&target, &edited) {
            targets.push(batch_sync_target(
                photo_id,
                "unchanged",
                Some("no_effect"),
                "Clipboard payload does not change this target.",
            ));
            continue;
        }

        targets.push(batch_sync_target(
            photo_id,
            "ready",
            None,
            "Ready for batch sync.",
        ));
        ready_graphs.push(edited);
    }

    let ready_count = targets
        .iter()
        .filter(|target| target.status == "ready")
        .count();
    let unchanged_count = targets
        .iter()
        .filter(|target| target.status == "unchanged")
        .count();
    let blocked_count = targets
        .iter()
        .filter(|target| target.status == "blocked")
        .count();
    let status = if blocked_count > 0 {
        "blocked"
    } else {
        "ready"
    };
    let message = if blocked_count > 0 {
        format!("{blocked_count} target(s) blocked; nothing will be written.")
    } else {
        format!("{ready_count} target(s) ready, {unchanged_count} unchanged.")
    };

    Ok(PreparedEditClipboardSync {
        plan: BatchEditClipboardSyncPlan {
            status: status.to_string(),
            requested_count,
            ready_count,
            unchanged_count,
            blocked_count,
            targets,
            message,
        },
        ready_graphs,
    })
}

fn batch_sync_target(
    photo_id: &str,
    status: &str,
    code: Option<&str>,
    message: &str,
) -> BatchEditClipboardSyncTarget {
    BatchEditClipboardSyncTarget {
        photo_id: photo_id.to_string(),
        status: status.to_string(),
        code: code.map(str::to_string),
        message: message.to_string(),
    }
}

fn ensure_supported_edit_clipboard_sync(
    payload: &silica_edit::EditClipboardPayload,
    graph: &silica_edit::EditGraph,
) -> Result<(), (&'static str, String)> {
    if payload.basic.is_some() && has_unsupported_basic_runtime(graph) {
        return Err((
            "unsupported_basic_runtime",
            "Texture, clarity, and dehaze batch sync are unsupported until runtime support exists."
                .to_string(),
        ));
    }
    if payload.detail.is_some() && !render_detail_from_graph(graph).is_neutral() {
        return Err(("unsupported_detail", detail_unsupported_message()));
    }
    if payload.lens.is_some() || payload.geometry.is_some() {
        if let Some(message) = lens_unsupported_message(graph) {
            return Err(("unsupported_lens", message));
        }
        if let Some(message) = geometry_unsupported_message(&render_geometry_from_graph(graph)) {
            return Err(("unsupported_geometry", message));
        }
    }
    Ok(())
}

fn edit_clipboard_catalog_target_block(
    library_root_path: &Path,
    photo_id: &str,
) -> Result<Option<(&'static str, String)>, CoreError> {
    let Some(metadata) = silica_storage::get_photo_metadata(library_root_path, photo_id)? else {
        return Ok(Some(("missing_photo", "Photo not found.".to_string())));
    };
    if metadata.unsupported || metadata.file_type != "jpeg" {
        return Ok(Some((
            "unsupported_target",
            "Edit clipboard copy and sync are limited to JPEG/JPG Develop photos in this alpha."
                .to_string(),
        )));
    }
    Ok(None)
}

fn has_unsupported_basic_runtime(graph: &silica_edit::EditGraph) -> bool {
    graph.basic.texture.as_f64().unwrap_or(0.0) != 0.0
        || graph.basic.clarity.as_f64().unwrap_or(0.0) != 0.0
        || graph.basic.dehaze.as_f64().unwrap_or(0.0) != 0.0
}

fn edit_graphs_equal_ignoring_updated_at(
    left: &silica_edit::EditGraph,
    right: &silica_edit::EditGraph,
) -> bool {
    let mut normalized_left = left.clone();
    let mut normalized_right = right.clone();
    normalized_left.updated_at.clear();
    normalized_right.updated_at.clear();
    normalized_left == normalized_right
}

fn apply_detail_preview_boundary(
    mut request: silica_render::ExposureContrastPreviewRequest,
    detail: silica_render::DetailRenderAdjustment,
) -> silica_render::ExposureContrastPreviewRequest {
    request.detail = detail;
    if request.status == silica_render::PreviewRenderStatus::Ready && !detail.is_neutral() {
        request.status = silica_render::PreviewRenderStatus::Unsupported;
        request.message = detail_unsupported_message();
    }
    request
}

fn apply_manual_mask_preview_boundary(
    mut request: silica_render::ExposureContrastPreviewRequest,
    graph: &silica_edit::EditGraph,
) -> Result<silica_render::ExposureContrastPreviewRequest, CoreError> {
    request.masks = render_manual_masks_from_graph(graph)?;
    Ok(request)
}

fn apply_lens_geometry_preview_boundary(
    mut request: silica_render::ExposureContrastPreviewRequest,
    graph: &silica_edit::EditGraph,
) -> silica_render::ExposureContrastPreviewRequest {
    request.geometry = render_geometry_from_graph(graph);
    if request.status == silica_render::PreviewRenderStatus::Ready {
        if let Some(message) = lens_unsupported_message(graph)
            .or_else(|| geometry_unsupported_message(&request.geometry))
        {
            request.status = silica_render::PreviewRenderStatus::Unsupported;
            request.message = message;
        }
    }
    request
}

fn lens_unsupported_message(graph: &silica_edit::EditGraph) -> Option<String> {
    let distortion = graph.lens.distortion.as_f64().unwrap_or(0.0);
    let vignetting = graph.lens.vignetting.as_f64().unwrap_or(0.0);
    if graph.lens.profile_correction
        || graph.lens.profile_id.is_some()
        || graph.lens.chromatic_aberration
        || distortion != 0.0
        || vignetting != 0.0
    {
        return Some(
            "Lens correction preview/export is unsupported until lens-profile support exists."
                .to_string(),
        );
    }
    None
}

fn geometry_unsupported_message(
    geometry: &silica_render::GeometryRenderAdjustment,
) -> Option<String> {
    if !geometry.transform.is_neutral() {
        return Some(
            "Geometry transform preview/export is unsupported until renderer support exists."
                .to_string(),
        );
    }
    if let Some(crop) = &geometry.crop {
        if crop.angle != 0.0 {
            return Some(
                "Angled crop preview/export is unsupported until renderer support exists."
                    .to_string(),
            );
        }
    }
    if !is_supported_quarter_turn(geometry.rotation) {
        return Some(
            "Arbitrary rotation preview/export is unsupported until renderer support exists."
                .to_string(),
        );
    }
    None
}

fn ensure_supported_lens_geometry_export(
    graph: &silica_edit::EditGraph,
    geometry: &silica_render::GeometryRenderAdjustment,
) -> Result<(), CoreError> {
    if let Some(message) = lens_unsupported_message(graph) {
        return Err(CoreError::ExportBlocked(message));
    }
    if let Some(message) = geometry_unsupported_message(geometry) {
        return Err(CoreError::ExportBlocked(message));
    }
    Ok(())
}

fn ensure_supported_lens_geometry_commit(graph: &silica_edit::EditGraph) -> Result<(), CoreError> {
    if let Some(message) = lens_unsupported_message(graph) {
        return Err(CoreError::UnsupportedEdit(message));
    }
    if let Some(message) = geometry_unsupported_message(&render_geometry_from_graph(graph)) {
        return Err(CoreError::UnsupportedEdit(message));
    }
    Ok(())
}

fn ensure_supported_manual_masks_commit(graph: &silica_edit::EditGraph) -> Result<(), CoreError> {
    render_manual_masks_from_graph(graph).map(|_| ())
}

fn ensure_no_active_manual_masks_for_export(
    graph: &silica_edit::EditGraph,
) -> Result<(), CoreError> {
    if graph.masks.iter().any(|mask| mask.enabled) {
        return Err(CoreError::ExportBlocked(masked_export_blocked_message()));
    }
    Ok(())
}

fn masked_export_blocked_message() -> String {
    "Manual mask export is unsupported for RAW-derived export in the local alpha.".to_string()
}

fn manual_mask_unsupported_message(message: impl AsRef<str>) -> CoreError {
    CoreError::UnsupportedEdit(format!("Manual mask unsupported: {}", message.as_ref()))
}

fn manual_mask_type_string(mask_type: silica_edit::MaskType) -> &'static str {
    match mask_type {
        silica_edit::MaskType::Brush => "brush",
        silica_edit::MaskType::LinearGradient => "linear_gradient",
        silica_edit::MaskType::RadialGradient => "radial_gradient",
        silica_edit::MaskType::Subject => "subject",
        silica_edit::MaskType::Sky => "sky",
        silica_edit::MaskType::Background => "background",
        silica_edit::MaskType::ColorRange => "color_range",
        silica_edit::MaskType::LuminanceRange => "luminance_range",
    }
}

fn photo_manual_masks_from_graph(graph: &silica_edit::EditGraph) -> Vec<PhotoManualMaskState> {
    graph
        .masks
        .iter()
        .filter_map(photo_manual_mask_from_edit)
        .collect()
}

fn photo_manual_mask_from_edit(mask: &silica_edit::Mask) -> Option<PhotoManualMaskState> {
    let geometry = match (&mask.mask_type, mask.geometry.as_ref()) {
        (
            silica_edit::MaskType::LinearGradient,
            Some(silica_edit::MaskGeometry::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
            }),
        ) => Some(PhotoManualMaskGeometryState::LinearGradient {
            start_x: start_x.as_f64().unwrap_or(0.0),
            start_y: start_y.as_f64().unwrap_or(0.0),
            end_x: end_x.as_f64().unwrap_or(1.0),
            end_y: end_y.as_f64().unwrap_or(1.0),
        }),
        (
            silica_edit::MaskType::RadialGradient,
            Some(silica_edit::MaskGeometry::RadialGradient {
                center_x,
                center_y,
                radius_x,
                radius_y,
                rotation,
            }),
        ) => Some(PhotoManualMaskGeometryState::RadialGradient {
            center_x: center_x.as_f64().unwrap_or(0.5),
            center_y: center_y.as_f64().unwrap_or(0.5),
            radius_x: radius_x.as_f64().unwrap_or(0.25),
            radius_y: radius_y.as_f64().unwrap_or(0.25),
            rotation: rotation.as_f64().unwrap_or(0.0),
        }),
        (silica_edit::MaskType::Brush, None) if mask.brush.is_some() => None,
        _ => return None,
    };

    Some(PhotoManualMaskState {
        id: mask.id.clone(),
        kind: manual_mask_type_string(mask.mask_type).to_string(),
        name: mask.name.clone(),
        enabled: mask.enabled,
        invert: mask.invert,
        opacity: mask.opacity.as_f64().unwrap_or(100.0),
        feather: mask.feather.as_f64().unwrap_or(0.0),
        geometry,
        exposure: mask
            .local_adjustments
            .get("exposure")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        contrast: mask
            .local_adjustments
            .get("contrast")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
    })
}

fn render_manual_masks_from_graph(
    graph: &silica_edit::EditGraph,
) -> Result<Vec<silica_render::ManualMaskRenderAdjustment>, CoreError> {
    graph
        .masks
        .iter()
        .map(render_manual_mask_from_edit)
        .collect()
}

fn render_manual_mask_from_edit(
    mask: &silica_edit::Mask,
) -> Result<silica_render::ManualMaskRenderAdjustment, CoreError> {
    if mask.source.kind != silica_edit::MaskSourceKind::Manual {
        return Err(manual_mask_unsupported_message(
            "only manual mask source is supported",
        ));
    }
    for key in mask.local_adjustments.keys() {
        if key != "exposure" && key != "contrast" {
            return Err(manual_mask_unsupported_message(format!(
                "local adjustment `{key}` is unsupported"
            )));
        }
    }

    let geometry = match (&mask.mask_type, &mask.geometry) {
        (silica_edit::MaskType::Brush, None) => {
            let brush = mask.brush.as_ref().ok_or_else(|| {
                manual_mask_unsupported_message("brush masks require durable brush strokes")
            })?;
            let strokes = render_brush_strokes_from_edit(brush);
            let raster = silica_render::rasterize_brush_mask(
                &mask.id,
                &strokes,
                LOCAL_ALPHA_BRUSH_MASK_RASTER_EDGE,
                LOCAL_ALPHA_BRUSH_MASK_RASTER_EDGE,
            )
            .map_err(|error| manual_mask_unsupported_message(error.to_string()))?;
            silica_render::ManualMaskRenderGeometry::BrushRaster {
                width: raster.width,
                height: raster.height,
                alpha: raster.alpha,
                cache_key: raster.cache_key,
            }
        }
        (
            silica_edit::MaskType::LinearGradient,
            Some(silica_edit::MaskGeometry::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
            }),
        ) => silica_render::ManualMaskRenderGeometry::LinearGradient {
            start_x: start_x.as_f64().unwrap_or(0.0),
            start_y: start_y.as_f64().unwrap_or(0.0),
            end_x: end_x.as_f64().unwrap_or(1.0),
            end_y: end_y.as_f64().unwrap_or(1.0),
        },
        (
            silica_edit::MaskType::RadialGradient,
            Some(silica_edit::MaskGeometry::RadialGradient {
                center_x,
                center_y,
                radius_x,
                radius_y,
                rotation,
            }),
        ) => silica_render::ManualMaskRenderGeometry::RadialGradient {
            center_x: center_x.as_f64().unwrap_or(0.5),
            center_y: center_y.as_f64().unwrap_or(0.5),
            radius_x: radius_x.as_f64().unwrap_or(0.25),
            radius_y: radius_y.as_f64().unwrap_or(0.25),
            rotation: rotation.as_f64().unwrap_or(0.0),
        },
        _ => {
            return Err(manual_mask_unsupported_message(
                "mask type and geometry payload must match",
            ))
        }
    };
    let exposure = manual_mask_local_value(mask, "exposure", -5.0, 5.0)?;
    let contrast = manual_mask_local_value(mask, "contrast", -100.0, 100.0)?;

    Ok(silica_render::ManualMaskRenderAdjustment {
        id: mask.id.clone(),
        enabled: mask.enabled,
        invert: mask.invert,
        opacity: mask.opacity.as_f64().unwrap_or(100.0),
        feather: mask.feather.as_f64().unwrap_or(0.0),
        geometry,
        exposure,
        contrast,
    })
}

fn render_brush_strokes_from_edit(
    brush: &silica_edit::MaskBrush,
) -> Vec<silica_render::BrushMaskRasterStroke> {
    brush
        .strokes
        .iter()
        .map(|stroke| silica_render::BrushMaskRasterStroke {
            id: stroke.id.clone(),
            radius: stroke.radius.as_f64().unwrap_or(0.0),
            points: stroke
                .points
                .iter()
                .map(|point| silica_render::BrushMaskRasterPoint {
                    x: point.x.as_f64().unwrap_or(0.0),
                    y: point.y.as_f64().unwrap_or(0.0),
                })
                .collect(),
        })
        .collect()
}

fn manual_mask_local_value(
    mask: &silica_edit::Mask,
    key: &str,
    min: f64,
    max: f64,
) -> Result<f64, CoreError> {
    let Some(value) = mask.local_adjustments.get(key) else {
        return Ok(0.0);
    };
    let value = value
        .as_f64()
        .ok_or_else(|| manual_mask_unsupported_message(format!("`{key}` must be finite")))?;
    if !(min..=max).contains(&value) {
        return Err(manual_mask_unsupported_message(format!(
            "`{key}` must be between {min} and {max}"
        )));
    }
    Ok(value)
}

fn export_manual_masks_from_render(
    masks: &[silica_render::ManualMaskRenderAdjustment],
) -> Vec<silica_export::ManualMaskAdjustment> {
    masks
        .iter()
        .map(|mask| silica_export::ManualMaskAdjustment {
            id: mask.id.clone(),
            enabled: mask.enabled,
            invert: mask.invert,
            opacity: mask.opacity,
            feather: mask.feather,
            geometry: match &mask.geometry {
                silica_render::ManualMaskRenderGeometry::LinearGradient {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                } => silica_export::ManualMaskGeometry::LinearGradient {
                    start_x: *start_x,
                    start_y: *start_y,
                    end_x: *end_x,
                    end_y: *end_y,
                },
                silica_render::ManualMaskRenderGeometry::RadialGradient {
                    center_x,
                    center_y,
                    radius_x,
                    radius_y,
                    rotation,
                } => silica_export::ManualMaskGeometry::RadialGradient {
                    center_x: *center_x,
                    center_y: *center_y,
                    radius_x: *radius_x,
                    radius_y: *radius_y,
                    rotation: *rotation,
                },
                silica_render::ManualMaskRenderGeometry::BrushRaster {
                    width,
                    height,
                    alpha,
                    ..
                } => silica_export::ManualMaskGeometry::RasterAlphaPlane {
                    width: *width,
                    height: *height,
                    alpha: alpha.clone(),
                },
            },
            exposure: mask.exposure,
            contrast: mask.contrast,
        })
        .collect()
}

fn is_supported_quarter_turn(rotation: f64) -> bool {
    [0.0, 90.0, -90.0, 180.0, -180.0]
        .iter()
        .any(|supported| (rotation - supported).abs() <= f64::EPSILON)
}

fn render_color_presence_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::ColorPresenceRenderAdjustment {
    silica_render::ColorPresenceRenderAdjustment {
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
    }
}

fn export_color_presence_from_render(
    color_presence: silica_render::ColorPresenceRenderAdjustment,
) -> silica_export::ColorPresenceAdjustment {
    silica_export::ColorPresenceAdjustment {
        vibrance: color_presence.vibrance,
        saturation: color_presence.saturation,
    }
}

fn export_white_balance_mode(
    mode: silica_render::WhiteBalanceRenderMode,
) -> silica_export::WhiteBalanceMode {
    match mode {
        silica_render::WhiteBalanceRenderMode::AsShot => silica_export::WhiteBalanceMode::AsShot,
        silica_render::WhiteBalanceRenderMode::Auto => silica_export::WhiteBalanceMode::Auto,
        silica_render::WhiteBalanceRenderMode::Daylight => {
            silica_export::WhiteBalanceMode::Daylight
        }
        silica_render::WhiteBalanceRenderMode::Cloudy => silica_export::WhiteBalanceMode::Cloudy,
        silica_render::WhiteBalanceRenderMode::Shade => silica_export::WhiteBalanceMode::Shade,
        silica_render::WhiteBalanceRenderMode::Tungsten => {
            silica_export::WhiteBalanceMode::Tungsten
        }
        silica_render::WhiteBalanceRenderMode::Fluorescent => {
            silica_export::WhiteBalanceMode::Fluorescent
        }
        silica_render::WhiteBalanceRenderMode::Flash => silica_export::WhiteBalanceMode::Flash,
        silica_render::WhiteBalanceRenderMode::Custom => silica_export::WhiteBalanceMode::Custom,
    }
}

fn white_balance_render_mode_string(mode: silica_render::WhiteBalanceRenderMode) -> &'static str {
    match mode {
        silica_render::WhiteBalanceRenderMode::AsShot => "as_shot",
        silica_render::WhiteBalanceRenderMode::Auto => "auto",
        silica_render::WhiteBalanceRenderMode::Daylight => "daylight",
        silica_render::WhiteBalanceRenderMode::Cloudy => "cloudy",
        silica_render::WhiteBalanceRenderMode::Shade => "shade",
        silica_render::WhiteBalanceRenderMode::Tungsten => "tungsten",
        silica_render::WhiteBalanceRenderMode::Fluorescent => "fluorescent",
        silica_render::WhiteBalanceRenderMode::Flash => "flash",
        silica_render::WhiteBalanceRenderMode::Custom => "custom",
    }
}

fn export_color_profile_message(profile: PhotoExportColorProfile) -> &'static str {
    match profile {
        PhotoExportColorProfile::Srgb => "JPEG sRGB export completed.",
        PhotoExportColorProfile::DisplayP3 => "JPEG Display P3 export completed.",
    }
}

fn export_raster_message(
    format: PhotoExportFormat,
    color_profile: PhotoExportColorProfile,
) -> &'static str {
    match format {
        PhotoExportFormat::Jpeg => export_color_profile_message(color_profile),
        PhotoExportFormat::Png => "PNG sRGB export completed.",
        PhotoExportFormat::Tiff => "TIFF sRGB export completed.",
    }
}

fn export_profile_metadata_source(format: PhotoExportFormat) -> &'static str {
    match format {
        PhotoExportFormat::Jpeg => "silica-export",
        PhotoExportFormat::Png | PhotoExportFormat::Tiff => "none",
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
    fn edit_clipboard_contract_through_core_preserves_target_identity_without_catalog_write() {
        let mut source = silica_edit::default_edit_graph(
            silica_edit::EditGraphSource {
                photo_id: "source-photo".to_string(),
                path: "/tmp/source.raw".to_string(),
                file_size: 2048,
                modified_at: Some("unix:11".to_string()),
                partial_hash: Some("source-partial".to_string()),
                full_hash: Some("source-full".to_string()),
            },
            "unix:12",
        );
        source.profile.input_profile = "source-profile".to_string();
        source.metadata.rating = 5;
        source.extensions.insert(
            "com.example.source".to_string(),
            serde_json::json!({ "owned_by": "source" }),
        );
        let source = silica_edit::apply_exposure_contrast(&source, 0.8, 12.0, "unix:13")
            .expect("source basic edit");
        let source = silica_edit::apply_geometry_orientation(&source, 90.0, true, false, "unix:14")
            .expect("source geometry edit");

        let mut target = silica_edit::default_edit_graph(
            silica_edit::EditGraphSource {
                photo_id: "target-photo".to_string(),
                path: "/tmp/target.raw".to_string(),
                file_size: 4096,
                modified_at: Some("unix:21".to_string()),
                partial_hash: Some("target-partial".to_string()),
                full_hash: Some("target-full".to_string()),
            },
            "unix:22",
        );
        target.profile.input_profile = "target-profile".to_string();
        target.metadata.rejected = true;
        target.extensions.insert(
            "com.example.target".to_string(),
            serde_json::json!({ "owned_by": "target" }),
        );

        let payload = copy_edit_clipboard_payload(
            &source,
            silica_edit::EditClipboardSelection {
                basic: true,
                geometry: true,
                ..Default::default()
            },
        )
        .expect("copy clipboard through core");
        let pasted = apply_edit_clipboard_payload_to_graph(&target, &payload, "unix:30")
            .expect("paste clipboard through core");

        assert_eq!(pasted.source, target.source);
        assert_eq!(pasted.profile, target.profile);
        assert_eq!(pasted.metadata, target.metadata);
        assert_eq!(pasted.extensions, target.extensions);
        assert_eq!(pasted.masks, target.masks);
        assert_eq!(pasted.basic, source.basic);
        assert_eq!(pasted.geometry, source.geometry);
        assert_eq!(pasted.tone, target.tone);
        assert_eq!(pasted.color, target.color);
        assert_eq!(pasted.detail, target.detail);
        assert_eq!(pasted.lens, target.lens);
    }

    #[test]
    fn copies_photo_edit_clipboard_payload_from_catalog_state() {
        let workspace = unique_library_root("core-copy-photo-edit-clipboard");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import root");
        write_source_jpeg(&supported_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");
        let photo = list_library_photos(&created.root_path)
            .expect("list photos")
            .into_iter()
            .find(|photo| photo.file_name == "sample.jpg")
            .expect("sample photo");
        commit_exposure_contrast_edit(&created.root_path, &photo.photo_id, 0.65, 9.0)
            .expect("commit source edit")
            .expect("committed edit");

        let payload = copy_photo_edit_clipboard_payload(
            &created.root_path,
            &photo.photo_id,
            silica_edit::EditClipboardSelection {
                basic: true,
                ..Default::default()
            },
        )
        .expect("copy catalog clipboard payload")
        .expect("payload");

        assert_eq!(payload.schema, silica_edit::EDIT_CLIPBOARD_SCHEMA);
        assert!(payload.basic.is_some());
        assert_eq!(
            payload
                .basic
                .as_ref()
                .and_then(|basic| basic.exposure.as_f64()),
            Some(0.65)
        );
        assert_eq!(
            payload
                .basic
                .as_ref()
                .and_then(|basic| basic.contrast.as_f64()),
            Some(9.0)
        );
        assert!(payload.tone.is_none());
        assert!(payload.color.is_none());
        assert!(payload.detail.is_none());
        assert!(payload.lens.is_none());
        assert!(payload.geometry.is_none());

        let missing = copy_photo_edit_clipboard_payload(
            &created.root_path,
            "missing-photo",
            silica_edit::EditClipboardSelection {
                basic: true,
                ..Default::default()
            },
        )
        .expect("missing photo is not an error");
        assert!(missing.is_none());

        remove_library_root(&workspace);
    }

    #[test]
    fn batch_sync_edit_clipboard_applies_payload_with_per_photo_history() {
        let workspace = unique_library_root("core-batch-sync");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let first_file = import_root.join("first.jpg");
        let second_file = import_root.join("second.jpg");

        std::fs::create_dir_all(&import_root).expect("create import root");
        write_source_jpeg(&first_file);
        write_source_jpeg(&second_file);
        let first_hash = file_hash(&first_file);
        let second_hash = file_hash(&second_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");
        let photos = list_library_photos(&created.root_path).expect("list photos");
        assert_eq!(photos.len(), 2);
        let target_ids: Vec<String> = photos.iter().map(|photo| photo.photo_id.clone()).collect();

        let source = silica_edit::default_edit_graph(
            silica_edit::EditGraphSource {
                photo_id: "source-photo".to_string(),
                path: "/tmp/source.jpg".to_string(),
                file_size: 256,
                modified_at: Some("unix:source".to_string()),
                partial_hash: Some("source-partial".to_string()),
                full_hash: None,
            },
            "unix:10",
        );
        let source = silica_edit::apply_exposure_contrast(&source, 0.9, 16.0, "unix:11")
            .expect("source basic edit");
        let payload = copy_edit_clipboard_payload(
            &source,
            silica_edit::EditClipboardSelection {
                basic: true,
                ..Default::default()
            },
        )
        .expect("copy clipboard payload");

        let plan = plan_edit_clipboard_sync(&created.root_path, &target_ids, &payload)
            .expect("plan batch sync payload");
        assert_eq!(plan.status, "ready");
        assert_eq!(plan.ready_count, 2);
        assert_eq!(plan.unchanged_count, 0);
        assert_eq!(plan.blocked_count, 0);

        let result = apply_edit_clipboard_sync(&created.root_path, &target_ids, &payload)
            .expect("batch sync payload");

        assert_eq!(result.status, "applied");
        assert_eq!(result.requested_count, 2);
        assert_eq!(result.applied_count, 2);
        assert_eq!(result.failed_count, 0);
        assert_eq!(result.blocked_count, 0);
        assert_eq!(result.commits.len(), 2);
        assert!(result.failures.is_empty());

        for photo in &photos {
            let graph = silica_storage::load_active_edit_graph(&created.root_path, &photo.photo_id)
                .expect("load active graph")
                .expect("active graph");
            assert_eq!(graph.source.photo_id, photo.photo_id);
            assert_eq!(graph.source.path, photo.path);
            assert_eq!(
                graph.profile.input_profile,
                silica_edit::INPUT_PROFILE_UNKNOWN
            );
            assert_eq!(graph.basic.exposure.as_f64(), Some(0.9));
            assert_eq!(graph.basic.contrast.as_f64(), Some(16.0));

            let history =
                list_photo_history(&created.root_path, &photo.photo_id).expect("read history");
            assert_eq!(history.items.len(), 1);
            assert_eq!(history.items[0].action_kind, "edit_commit");
            assert_eq!(history.items[0].sequence, 1);
        }

        assert_original_hash(&first_file, &first_hash, "batch sync first original");
        assert_original_hash(&second_file, &second_hash, "batch sync second original");
        remove_library_root(&workspace);
    }

    #[test]
    fn batch_sync_edit_clipboard_preflight_failure_writes_no_history() {
        let workspace = unique_library_root("core-batch-sync-preflight");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import root");
        write_source_jpeg(&supported_file);
        let original_hash = file_hash(&supported_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");
        let photo = list_library_photos(&created.root_path)
            .expect("list photos")
            .into_iter()
            .find(|photo| photo.file_name == "sample.jpg")
            .expect("sample photo");

        let source = silica_edit::default_edit_graph(
            silica_edit::EditGraphSource {
                photo_id: "source-photo".to_string(),
                path: "/tmp/source.jpg".to_string(),
                file_size: 256,
                modified_at: Some("unix:source".to_string()),
                partial_hash: Some("source-partial".to_string()),
                full_hash: None,
            },
            "unix:10",
        );
        let source = silica_edit::apply_exposure_contrast(&source, 0.4, 8.0, "unix:11")
            .expect("source basic edit");
        let payload = copy_edit_clipboard_payload(
            &source,
            silica_edit::EditClipboardSelection {
                basic: true,
                ..Default::default()
            },
        )
        .expect("copy clipboard payload");

        let plan = plan_edit_clipboard_sync(
            &created.root_path,
            &[photo.photo_id.clone(), "missing-photo".to_string()],
            &payload,
        )
        .expect("plan preflight failure");

        assert_eq!(plan.status, "blocked");
        assert_eq!(plan.requested_count, 2);
        assert_eq!(plan.ready_count, 1);
        assert_eq!(plan.blocked_count, 1);
        assert_eq!(plan.targets[1].photo_id, "missing-photo");
        assert_eq!(plan.targets[1].code.as_deref(), Some("missing_photo"));

        let result = apply_edit_clipboard_sync(
            &created.root_path,
            &[photo.photo_id.clone(), "missing-photo".to_string()],
            &payload,
        )
        .expect("preflight failure returns result");

        assert_eq!(result.status, "blocked");
        assert_eq!(result.requested_count, 2);
        assert_eq!(result.applied_count, 0);
        assert_eq!(result.blocked_count, 1);
        assert_eq!(result.failed_count, 1);
        assert_eq!(result.failures[0].photo_id, "missing-photo");
        assert_eq!(result.targets[1].code.as_deref(), Some("missing_photo"));
        assert!(result.failures[0].message.contains("not found"));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo.photo_id)
                .expect("load active graph")
                .is_none(),
            "failed batch must not write active edit graph"
        );
        let history = list_photo_history(&created.root_path, &photo.photo_id)
            .expect("read history after failed batch");
        assert!(history.items.is_empty());
        assert_original_hash(
            &supported_file,
            &original_hash,
            "failed batch sync original",
        );
        remove_library_root(&workspace);
    }

    #[test]
    fn batch_sync_edit_clipboard_blocks_unsupported_detail_without_writes() {
        let workspace = unique_library_root("core-batch-sync-detail-blocked");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import root");
        write_source_jpeg(&supported_file);
        let original_hash = file_hash(&supported_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");
        let photo = list_library_photos(&created.root_path)
            .expect("list photos")
            .into_iter()
            .find(|photo| photo.file_name == "sample.jpg")
            .expect("sample photo");

        let source = silica_edit::default_edit_graph(
            silica_edit::EditGraphSource {
                photo_id: "source-photo".to_string(),
                path: "/tmp/source.jpg".to_string(),
                file_size: 256,
                modified_at: Some("unix:source".to_string()),
                partial_hash: Some("source-partial".to_string()),
                full_hash: None,
            },
            "unix:10",
        );
        let source =
            silica_edit::apply_detail_sharpening(&source, 40.0, 1.2, 35.0, 10.0, "unix:11")
                .expect("source detail edit");
        let payload = copy_edit_clipboard_payload(
            &source,
            silica_edit::EditClipboardSelection {
                detail: true,
                ..Default::default()
            },
        )
        .expect("copy detail clipboard payload");

        let result =
            apply_edit_clipboard_sync(&created.root_path, &[photo.photo_id.clone()], &payload)
                .expect("unsupported detail returns blocked result");

        assert_eq!(result.status, "blocked");
        assert_eq!(result.applied_count, 0);
        assert_eq!(result.blocked_count, 1);
        assert_eq!(
            result.targets[0].code.as_deref(),
            Some("unsupported_detail")
        );
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo.photo_id)
                .expect("load active graph")
                .is_none(),
            "blocked detail batch must not write active edit graph"
        );
        let history = list_photo_history(&created.root_path, &photo.photo_id)
            .expect("read history after blocked detail batch");
        assert!(history.items.is_empty());
        assert_original_hash(
            &supported_file,
            &original_hash,
            "blocked detail batch original",
        );
        remove_library_root(&workspace);
    }

    #[test]
    fn edit_clipboard_blocks_raw_copy_and_batch_target_without_writes() {
        let workspace = unique_library_root("core-edit-clipboard-raw-blocked");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let source_file = import_root.join("source.jpg");
        let raw_file = import_root.join("target.DNG");

        std::fs::create_dir_all(&import_root).expect("create import root");
        write_source_jpeg(&source_file);
        std::fs::write(&raw_file, b"raw target placeholder").expect("write raw target");
        let raw_hash = file_hash(&raw_file);

        let created = create_library(&library_root).expect("create library through core");
        import_folder(&created.root_path, &import_root).expect("import folder through core");
        let photos = list_library_photos(&created.root_path).expect("list photos");
        let source_photo = photos
            .iter()
            .find(|photo| photo.file_name == "source.jpg")
            .expect("source photo");
        let raw_photo = photos
            .iter()
            .find(|photo| photo.file_name == "target.DNG")
            .expect("raw target photo");
        assert_eq!(raw_photo.file_type, "DNG");
        assert!(!raw_photo.unsupported);

        let raw_copy = copy_photo_edit_clipboard_payload(
            &created.root_path,
            &raw_photo.photo_id,
            silica_edit::EditClipboardSelection {
                basic: true,
                ..Default::default()
            },
        )
        .expect_err("RAW copy must be blocked");
        assert!(matches!(raw_copy, CoreError::UnsupportedEdit(_)));
        assert!(raw_copy.to_string().contains("JPEG/JPG"));

        commit_exposure_contrast_edit(&created.root_path, &source_photo.photo_id, 0.8, 12.0)
            .expect("commit source edit");
        let payload = copy_photo_edit_clipboard_payload(
            &created.root_path,
            &source_photo.photo_id,
            silica_edit::EditClipboardSelection {
                basic: true,
                ..Default::default()
            },
        )
        .expect("copy source payload")
        .expect("source payload");

        let plan = plan_edit_clipboard_sync(
            &created.root_path,
            std::slice::from_ref(&raw_photo.photo_id),
            &payload,
        )
        .expect("plan raw target");
        assert_eq!(plan.status, "blocked");
        assert_eq!(plan.ready_count, 0);
        assert_eq!(plan.blocked_count, 1);
        assert_eq!(plan.targets[0].code.as_deref(), Some("unsupported_target"));
        assert!(plan.targets[0].message.contains("JPEG/JPG"));

        let result = apply_edit_clipboard_sync(
            &created.root_path,
            std::slice::from_ref(&raw_photo.photo_id),
            &payload,
        )
        .expect("raw target returns blocked result");
        assert_eq!(result.status, "blocked");
        assert_eq!(result.applied_count, 0);
        assert_eq!(result.blocked_count, 1);
        assert_eq!(
            result.targets[0].code.as_deref(),
            Some("unsupported_target")
        );
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &raw_photo.photo_id)
                .expect("load raw active graph")
                .is_none(),
            "RAW target batch must not write active edit graph"
        );
        let history = list_photo_history(&created.root_path, &raw_photo.photo_id)
            .expect("read raw history after blocked batch");
        assert!(history.items.is_empty());
        assert_original_hash(&raw_file, &raw_hash, "blocked raw batch target");
        remove_library_root(&workspace);
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
              "appearance": {
                "theme": "neon",
                "density": "wide",
                "ui_scale": 1000
              },
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
        assert_eq!(loaded.session.appearance.theme, AppAppearanceTheme::Dark);
        assert_eq!(
            loaded.session.appearance.density,
            AppAppearanceDensity::Compact
        );
        assert_eq!(loaded.session.appearance.ui_scale, MAX_APP_SESSION_UI_SCALE);
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
    fn appearance_preferences_defaults_and_reset_are_stable() {
        let workspace = unique_library_root("appearance-preferences-reset");
        let session_path = workspace.join("app-session.json");
        let library_root = workspace.join("SilicaRAW Library");
        let defaults = default_app_appearance_preferences();

        assert_eq!(defaults.theme, AppAppearanceTheme::Dark);
        assert_eq!(defaults.density, AppAppearanceDensity::Compact);
        assert_eq!(defaults.ui_scale, DEFAULT_APP_SESSION_UI_SCALE);

        let mut session = AppSession::default();
        session.last_library_root_path = Some(library_root.clone());
        write_app_session(&session_path, &session).expect("write app session");

        let changed = AppAppearancePreferences {
            theme: AppAppearanceTheme::Light,
            density: AppAppearanceDensity::Comfortable,
            ui_scale: MAX_APP_SESSION_UI_SCALE,
        };
        let recorded = record_app_session_appearance(&session_path, changed.clone())
            .expect("record appearance");
        assert_eq!(recorded.session.appearance, changed);
        assert_eq!(
            recorded.session.last_library_root_path.as_deref(),
            Some(library_root.as_path())
        );

        let reset = reset_app_session_appearance(&session_path).expect("reset appearance");

        assert!(reset.warnings.is_empty());
        assert_eq!(reset.session.appearance, defaults);
        assert_eq!(
            reset.session.last_library_root_path.as_deref(),
            Some(library_root.as_path())
        );
        let loaded = load_app_session(&session_path).expect("reload reset appearance");
        assert_eq!(loaded.session.appearance, defaults);

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
    fn computes_and_caches_histogram_without_mutating_original() {
        let workspace = unique_library_root("core-histogram-flow");
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

        commit_color_presence_edit(&created.root_path, &photo_id, 24.0, -8.5)
            .expect("commit color presence")
            .expect("commit result");
        let histogram = get_photo_histogram(&created.root_path, &photo_id)
            .expect("get histogram")
            .expect("histogram result");

        assert_eq!(histogram.status, PhotoHistogramStatus::Ready);
        assert_eq!(histogram.pixel_count, 4);
        assert_eq!(histogram.red.len(), 256);
        assert_eq!(histogram.green.len(), 256);
        assert_eq!(histogram.blue.len(), 256);
        assert_eq!(histogram.luminance.len(), 256);
        assert!(histogram.cache_path.contains("render-cache"));
        assert_original_hash(&jpeg_file, &original_hash, "histogram generation");

        let cached = silica_storage::get_photo_cache_record(
            &created.root_path,
            &photo_id,
            silica_storage::HISTOGRAM_CACHE_TYPE,
        )
        .expect("read histogram cache")
        .expect("histogram cache row");
        assert_eq!(cached.path, histogram.cache_path);

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
    fn previews_without_write_and_commits_manual_linear_gradient_mask() {
        let workspace = unique_library_root("core-manual-mask-flow");
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

        let before_preview = durable_catalog_counts(&created.catalog_path);
        let preview = preview_manual_linear_gradient_mask(
            &created.root_path,
            &photo_id,
            "mask-linear-1",
            "Top burn",
            100.0,
            0.0,
            false,
            0.0,
            0.0,
            1.0,
            1.0,
            Some(0.75),
            Some(0.0),
        )
        .expect("preview manual mask")
        .expect("preview result");

        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.masks.len(), 1);
        assert_eq!(preview.masks[0].id, "mask-linear-1");
        assert_eq!(preview.masks[0].exposure, 0.75);
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert_original_hash(&jpeg_file, &original_hash, "manual mask preview");
        assert_eq!(
            durable_catalog_counts(&created.catalog_path),
            before_preview,
            "manual mask preview must not write durable catalog state"
        );
        let unmasked_followup_preview =
            preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.0, 0.0)
                .expect("preview without committed masks")
                .expect("unmasked preview result");
        assert!(unmasked_followup_preview.masks.is_empty());
        let unmasked_followup_bytes = unmasked_followup_preview
            .develop_preview_bytes
            .expect("unmasked preview bytes");

        let committed = commit_manual_linear_gradient_mask(
            &created.root_path,
            &photo_id,
            "mask-linear-1",
            "Top burn",
            100.0,
            0.0,
            false,
            0.0,
            0.0,
            1.0,
            1.0,
            Some(0.75),
            Some(0.0),
        )
        .expect("commit manual mask")
        .expect("commit result");
        assert!(committed.persisted);
        assert_eq!(committed.masks.len(), 1);
        assert_eq!(committed.masks[0].name, "Top burn");

        let restored = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read manual mask state")
            .expect("edit state");
        assert_eq!(restored.masks.len(), 1);
        assert_eq!(
            restored.masks[0].geometry,
            Some(PhotoManualMaskGeometryState::LinearGradient {
                start_x: 0.0,
                start_y: 0.0,
                end_x: 1.0,
                end_y: 1.0,
            })
        );
        let masked_followup_preview =
            preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.0, 0.0)
                .expect("preview with committed mask")
                .expect("masked preview result");
        assert_eq!(masked_followup_preview.masks.len(), 1);
        assert_ne!(
            masked_followup_preview
                .develop_preview_bytes
                .expect("masked preview bytes"),
            unmasked_followup_bytes,
            "committed mask must affect later Develop previews"
        );

        let undo = undo_last_history_action(&created.root_path, &photo_id).expect("undo");
        assert!(undo.applied);
        let undone = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read undone state")
            .expect("edit state");
        assert!(undone.masks.is_empty());

        let redo = redo_last_history_action(&created.root_path, &photo_id).expect("redo");
        assert!(redo.applied);
        let redone = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read redone state")
            .expect("edit state");
        assert_eq!(redone.masks.len(), 1);
        assert_original_hash(&jpeg_file, &original_hash, "manual mask commit");

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_brush_mask_cache_without_durable_edit_writes_and_commits_strokes() {
        let workspace = unique_library_root("core-brush-mask-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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

        let stroke = PhotoManualBrushStrokeInput {
            id: "stroke-1".to_string(),
            radius: 0.20,
            points: vec![PhotoManualBrushPointInput { x: 0.5, y: 0.5 }],
        };
        let before_preview = durable_catalog_counts(&created.catalog_path);
        let preview = preview_manual_brush_mask(
            &created.root_path,
            &photo_id,
            "mask-brush-1",
            "Center dodge",
            100.0,
            0.0,
            false,
            vec![stroke.clone()],
            Some(0.75),
            Some(0.0),
        )
        .expect("preview brush mask")
        .expect("preview result");

        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.masks.len(), 1);
        assert_eq!(preview.masks[0].kind, "brush");
        assert!(preview.masks[0].geometry.is_none());
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert_original_hash(&jpeg_file, &original_hash, "brush mask preview");
        let after_preview = durable_catalog_counts(&created.catalog_path);
        assert_eq!(after_preview.edit_states, before_preview.edit_states);
        assert_eq!(after_preview.edit_history, before_preview.edit_history);
        assert_eq!(after_preview.action_log, before_preview.action_log);
        assert_eq!(after_preview.exports, before_preview.exports);
        assert_eq!(
            after_preview.cache_records,
            before_preview.cache_records + 1
        );
        let mask_cache = silica_storage::get_photo_cache_record(
            &created.root_path,
            &photo_id,
            silica_storage::MASK_RASTER_CACHE_TYPE,
        )
        .expect("read mask raster cache")
        .expect("mask raster cache row");
        assert!(mask_cache.path.contains("render-cache/masks"));
        assert!(Path::new(&mask_cache.path).is_file());

        silica_storage::clear_disposable_cache(&created.root_path).expect("clear cache");
        assert!(!Path::new(&mask_cache.path).exists());
        assert!(silica_storage::get_photo_cache_record(
            &created.root_path,
            &photo_id,
            silica_storage::MASK_RASTER_CACHE_TYPE,
        )
        .expect("read mask raster cache after clear")
        .is_none());

        let committed = commit_manual_brush_mask(
            &created.root_path,
            &photo_id,
            "mask-brush-1",
            "Center dodge",
            100.0,
            0.0,
            false,
            vec![stroke],
            Some(0.75),
            Some(0.0),
        )
        .expect("commit brush mask")
        .expect("commit result");
        assert!(committed.persisted);
        assert_eq!(committed.masks.len(), 1);
        assert_eq!(committed.masks[0].kind, "brush");
        assert!(committed.masks[0].geometry.is_none());

        let restored = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read brush mask state")
            .expect("edit state");
        assert_eq!(restored.masks.len(), 1);
        assert_eq!(restored.masks[0].kind, "brush");
        assert!(restored.masks[0].geometry.is_none());

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export brush masked photo")
            .expect("export result");
        assert!(exported.bytes_written > 0);
        assert!(output_path.exists());
        let export_cache = silica_storage::get_photo_cache_record(
            &created.root_path,
            &photo_id,
            silica_storage::MASK_RASTER_CACHE_TYPE,
        )
        .expect("read mask raster cache after export")
        .expect("mask raster cache row after export");
        assert!(Path::new(&export_cache.path).is_file());
        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest brush masked export")
            .expect("latest brush masked export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["masks"][0]["kind"], "brush");
        assert_eq!(settings["masks"][0]["geometry"]["kind"], "brush_raster");
        assert!(settings["masks"][0]["geometry"]["cache_key"]
            .as_str()
            .is_some_and(|value| !value.is_empty()));
        assert_original_hash(&jpeg_file, &original_hash, "brush mask export");

        remove_library_root(&workspace);
    }

    #[test]
    fn exports_committed_manual_mask_and_records_mask_evidence() {
        let workspace = unique_library_root("core-manual-mask-export");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let neutral_output_path = export_root.join("sample-neutral.jpg");
        let masked_output_path = export_root.join("sample-masked.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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

        let neutral_export =
            export_photo_jpeg_srgb(&created.root_path, &photo_id, &neutral_output_path)
                .expect("export neutral photo")
                .expect("neutral export result");
        assert!(neutral_output_path.exists());
        assert_original_hash(&jpeg_file, &original_hash, "neutral export before mask");

        commit_manual_linear_gradient_mask(
            &created.root_path,
            &photo_id,
            "mask-linear-1",
            "Diagonal lift",
            100.0,
            0.0,
            false,
            0.0,
            0.0,
            1.0,
            1.0,
            Some(1.0),
            Some(0.0),
        )
        .expect("commit linear mask")
        .expect("commit result");

        let masked_preview =
            preview_exposure_contrast_edit(&created.root_path, &photo_id, 0.0, 0.0)
                .expect("preview committed mask")
                .expect("masked preview result");
        assert_eq!(masked_preview.status, PhotoPreviewStatus::Ready);
        assert!(masked_preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        let masked_export =
            export_photo_jpeg_srgb(&created.root_path, &photo_id, &masked_output_path)
                .expect("export masked photo")
                .expect("masked export result");
        assert!(masked_output_path.exists());
        assert_ne!(neutral_export.output_sha256, masked_export.output_sha256);
        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest masked export")
            .expect("latest masked export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["masks"][0]["kind"], "linear_gradient");
        assert_eq!(settings["masks"][0]["geometry"]["kind"], "linear_gradient");
        assert_eq!(settings["masks"][0]["geometry"]["start_x"], 0.0);
        assert_eq!(settings["masks"][0]["exposure"], 1.0);
        assert_original_hash(&jpeg_file, &original_hash, "masked export");

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
    fn photo_history_through_core_lists_real_checkpoints() {
        let workspace = unique_library_root("core-history-panel");
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

        let history = list_photo_history(&created.root_path, &photo_id).expect("read history");
        assert_eq!(history.photo_id, photo_id);
        assert_eq!(history.status, "ready");
        assert!(history.can_undo);
        assert!(!history.can_redo);
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].action_kind, "edit_commit");
        assert_eq!(history.items[0].label, "Exposure / contrast");
        assert_eq!(history.items[0].history_state, "applied");

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
    fn exports_edited_photo_to_png_and_records_catalog_row() {
        let workspace = unique_library_root("core-export-png");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.png");

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

        let exported = export_photo_png(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");

        assert_eq!(exported.photo_id, photo_id);
        assert_eq!(exported.output_path, output_path);
        assert_eq!(exported.format, "png");
        assert_eq!(exported.color_profile, "srgb");
        assert!(exported.bytes_written > 0);
        assert_eq!(exported.output_sha256.len(), 64);
        assert!(!exported.icc_profile_embedded);
        assert_eq!(exported.icc_profile_sha256, "");
        assert_eq!(
            std::fs::read(&jpeg_file).expect("read original after"),
            original_before
        );

        let decoded = image::ImageReader::open(&exported.output_path)
            .expect("open exported png")
            .with_guessed_format()
            .expect("guess exported format")
            .decode()
            .expect("decode exported png");
        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 2);

        let latest =
            silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
                .expect("read latest export")
                .expect("latest export");
        assert_eq!(latest.id, exported.export_record_id);
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["format"], "png");
        assert_eq!(settings["color_profile"], "srgb");
        assert_eq!(settings["icc_profile_embedded"], false);
        assert_eq!(settings["icc_profile_sha256"], serde_json::Value::Null);
        assert_eq!(settings["output_sha256"], exported.output_sha256);

        remove_library_root(&workspace);
    }

    #[test]
    fn exports_edited_photo_to_tiff_and_records_catalog_row() {
        let workspace = unique_library_root("core-export-tiff");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.tiff");

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

        let exported = export_photo_tiff(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");

        assert_eq!(exported.photo_id, photo_id);
        assert_eq!(exported.output_path, output_path);
        assert_eq!(exported.format, "tiff");
        assert_eq!(exported.color_profile, "srgb");
        assert!(exported.bytes_written > 0);
        assert_eq!(exported.output_sha256.len(), 64);
        assert!(!exported.icc_profile_embedded);
        assert_eq!(exported.icc_profile_sha256, "");
        assert_eq!(
            std::fs::read(&jpeg_file).expect("read original after"),
            original_before
        );

        let decoded = image::ImageReader::open(&exported.output_path)
            .expect("open exported tiff")
            .with_guessed_format()
            .expect("guess exported format")
            .decode()
            .expect("decode exported tiff");
        assert_eq!(decoded.width(), 2);
        assert_eq!(decoded.height(), 2);

        let latest =
            silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
                .expect("read latest export")
                .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["format"], "tiff");
        assert_eq!(settings["color_profile"], "srgb");
        assert_eq!(settings["icc_profile_embedded"], false);
        assert_eq!(settings["icc_profile_sha256"], serde_json::Value::Null);
        assert_eq!(settings["output_sha256"], exported.output_sha256);

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_commits_and_exports_white_balance_through_core() {
        let workspace = unique_library_root("core-white-balance");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph before preview")
                .is_none()
        );
        assert!(list_photo_history(&created.root_path, &photo_id)
            .expect("read history before preview")
            .items
            .is_empty());

        let preview = preview_white_balance_edit(
            &created.root_path,
            &photo_id,
            silica_edit::WhiteBalance::Custom,
            6500.0,
            20.0,
        )
        .expect("preview white balance")
        .expect("preview result");

        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.white_balance, silica_edit::WhiteBalance::Custom);
        assert_eq!(preview.temperature, 6500.0);
        assert_eq!(preview.tint, 20.0);
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));

        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after preview")
                .is_none(),
            "white balance preview must not write edit state"
        );
        assert!(
            list_photo_history(&created.root_path, &photo_id)
                .expect("read history after preview")
                .items
                .is_empty(),
            "white balance preview must not write edit history"
        );

        let committed = commit_white_balance_edit(
            &created.root_path,
            &photo_id,
            silica_edit::WhiteBalance::Custom,
            6500.0,
            20.0,
        )
        .expect("commit white balance")
        .expect("commit result");
        assert_eq!(committed.white_balance, silica_edit::WhiteBalance::Custom);
        assert_eq!(committed.temperature, 6500.0);
        assert_eq!(committed.tint, 20.0);
        assert!(committed.persisted);

        let persisted =
            silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
                .expect("load active graph")
                .expect("active graph");
        assert_eq!(
            persisted.basic.white_balance,
            silica_edit::WhiteBalance::Custom
        );
        assert_eq!(persisted.basic.temperature.as_f64(), Some(6500.0));
        assert_eq!(persisted.basic.tint.as_f64(), Some(20.0));

        let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].label, "White balance");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");
        assert!(exported.bytes_written > 0);

        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["white_balance"], "custom");
        assert_eq!(settings["temperature"], 6500.0);
        assert_eq!(settings["tint"], 20.0);

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_commits_and_exports_tone_recovery_through_core() {
        let workspace = unique_library_root("core-tone-recovery");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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

        let preview =
            preview_tone_recovery_edit(&created.root_path, &photo_id, -35.0, 42.0, 10.0, -12.0)
                .expect("preview tone recovery")
                .expect("preview result");

        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.highlights, -35.0);
        assert_eq!(preview.shadows, 42.0);
        assert_eq!(preview.whites, 10.0);
        assert_eq!(preview.blacks, -12.0);
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after preview")
                .is_none(),
            "tone recovery preview must not write edit state"
        );

        let committed =
            commit_tone_recovery_edit(&created.root_path, &photo_id, -35.0, 42.0, 10.0, -12.0)
                .expect("commit tone recovery")
                .expect("commit result");
        assert_eq!(committed.highlights, -35.0);
        assert_eq!(committed.shadows, 42.0);
        assert_eq!(committed.whites, 10.0);
        assert_eq!(committed.blacks, -12.0);
        assert!(committed.persisted);

        let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].label, "Tone recovery");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");
        assert!(exported.bytes_written > 0);

        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["highlights"], -35.0);
        assert_eq!(settings["shadows"], 42.0);
        assert_eq!(settings["whites"], 10.0);
        assert_eq!(settings["blacks"], -12.0);

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_commits_and_exports_tone_curve_through_core() {
        let workspace = unique_library_root("core-tone-curve");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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
        let rgb_curve = [(0.0, 0.0), (0.5, 0.28), (1.0, 1.0)];

        let preview =
            preview_tone_curve_edit(&created.root_path, &photo_id, &rgb_curve, &[], &[], &[])
                .expect("preview tone curve")
                .expect("preview result");

        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.tone_curve.curve_mode, silica_edit::CurveMode::Point);
        assert_eq!(preview.tone_curve.rgb_curve.len(), 3);
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after preview")
                .is_none(),
            "tone curve preview must not write edit state"
        );

        let committed =
            commit_tone_curve_edit(&created.root_path, &photo_id, &rgb_curve, &[], &[], &[])
                .expect("commit tone curve")
                .expect("commit result");
        assert_eq!(
            committed.tone_curve.curve_mode,
            silica_edit::CurveMode::Point
        );
        assert_eq!(committed.tone_curve.rgb_curve[1].y, 0.28);
        assert!(committed.persisted);

        let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].label, "Tone curve");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");
        assert!(exported.bytes_written > 0);

        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["tone_curve"]["curve_mode"], "point");
        assert_eq!(settings["tone_curve"]["rgb_curve"][1]["x"], 0.5);
        assert_eq!(settings["tone_curve"]["rgb_curve"][1]["y"], 0.28);

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_commits_and_exports_hsl_color_mixer_through_core() {
        let workspace = unique_library_root("core-hsl-color-mixer");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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

        let preview = preview_hsl_color_mixer_edit(
            &created.root_path,
            &photo_id,
            silica_edit::HslColorChannel::Blue,
            -12.0,
            24.0,
            -8.5,
        )
        .expect("preview hsl color mixer")
        .expect("preview result");

        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.hsl_color_mixer.blue.hue, -12.0);
        assert_eq!(preview.hsl_color_mixer.blue.saturation, 24.0);
        assert_eq!(preview.hsl_color_mixer.blue.luminance, -8.5);
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after preview")
                .is_none(),
            "HSL color mixer preview must not write edit state"
        );

        let committed = commit_hsl_color_mixer_edit(
            &created.root_path,
            &photo_id,
            silica_edit::HslColorChannel::Blue,
            -12.0,
            24.0,
            -8.5,
        )
        .expect("commit hsl color mixer")
        .expect("commit result");
        assert_eq!(committed.hsl_color_mixer.blue.hue, -12.0);
        assert_eq!(committed.hsl_color_mixer.blue.saturation, 24.0);
        assert_eq!(committed.hsl_color_mixer.blue.luminance, -8.5);
        assert!(committed.persisted);

        let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].label, "HSL color mixer");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");
        assert!(exported.bytes_written > 0);

        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["hsl_color_mixer"]["blue"]["hue"], -12.0);
        assert_eq!(settings["hsl_color_mixer"]["blue"]["saturation"], 24.0);
        assert_eq!(settings["hsl_color_mixer"]["blue"]["luminance"], -8.5);

        remove_library_root(&workspace);
    }

    #[test]
    fn blocks_detail_preview_commit_and_export_until_renderer_support_exists() {
        let workspace = unique_library_root("core-detail-boundary");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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

        let preview =
            preview_detail_sharpening_edit(&created.root_path, &photo_id, 42.0, 1.2, 35.0, 10.0)
                .expect("preview detail sharpening")
                .expect("preview result");
        assert_eq!(preview.status, PhotoPreviewStatus::Unsupported);
        assert_eq!(preview.detail.sharpening.amount, 42.0);
        assert!(preview.develop_preview_bytes.is_none());
        assert!(preview.message.contains("Detail"));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after blocked detail preview")
                .is_none(),
            "blocked detail preview must not write edit state"
        );

        let commit_error =
            commit_detail_sharpening_edit(&created.root_path, &photo_id, 42.0, 1.2, 35.0, 10.0)
                .expect_err("detail commit unsupported");
        assert!(matches!(commit_error, CoreError::UnsupportedEdit(_)));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after blocked detail commit")
                .is_none(),
            "blocked detail commit must not persist an edit graph"
        );

        let graph =
            silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
                .expect("load default graph")
                .expect("default graph");
        let detail_graph =
            silica_edit::apply_detail_sharpening(&graph, 42.0, 1.2, 35.0, 10.0, "unix:detail")
                .expect("build detail graph");
        silica_storage::commit_edit_graph(&created.root_path, detail_graph)
            .expect("seed unsupported committed detail state");
        let export_error = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect_err("active detail export unsupported");
        assert!(matches!(export_error, CoreError::ExportBlocked(_)));
        assert!(!output_path.exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_commits_and_exports_geometry_through_core() {
        let workspace = unique_library_root("core-geometry");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_geometry_source_jpeg(&jpeg_file);
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
        let orientation_before = get_photo_metadata(&created.root_path, &photo_id)
            .expect("read metadata before")
            .expect("metadata before")
            .orientation;

        let preview = preview_geometry_crop_edit(
            &created.root_path,
            &photo_id,
            0.0,
            0.0,
            0.5,
            1.0,
            0.0,
            None,
        )
        .expect("preview geometry crop")
        .expect("preview result");
        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(
            preview.geometry.crop.as_ref().map(|crop| crop.width),
            Some(0.5)
        );
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after preview")
                .is_none(),
            "geometry preview must not write edit state"
        );

        let crop_commit =
            commit_geometry_crop_edit(&created.root_path, &photo_id, 0.0, 0.0, 0.5, 1.0, 0.0, None)
                .expect("commit geometry crop")
                .expect("crop commit");
        assert!(crop_commit.persisted);
        assert_eq!(
            crop_commit.geometry.crop.as_ref().map(|crop| crop.height),
            Some(1.0)
        );

        let orientation_preview =
            preview_geometry_orientation_edit(&created.root_path, &photo_id, 90.0, true, false)
                .expect("preview geometry orientation")
                .expect("orientation preview");
        assert_eq!(orientation_preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(orientation_preview.geometry.rotation, 90.0);
        assert!(orientation_preview.geometry.flip_horizontal);

        let orientation_commit =
            commit_geometry_orientation_edit(&created.root_path, &photo_id, 90.0, true, false)
                .expect("commit geometry orientation")
                .expect("orientation commit");
        assert_eq!(orientation_commit.geometry.rotation, 90.0);
        assert!(orientation_commit.geometry.flip_horizontal);
        assert!(orientation_commit.persisted);

        let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
        assert_eq!(history.items.len(), 2);
        assert_eq!(history.items[0].label, "Geometry orientation");
        assert_eq!(history.items[1].label, "Geometry crop");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export geometry photo")
            .expect("export result");
        let decoded = image::ImageReader::open(&exported.output_path)
            .expect("open geometry export")
            .with_guessed_format()
            .expect("guess geometry export")
            .decode()
            .expect("decode geometry export");
        assert_eq!(decoded.width(), 3);
        assert_eq!(decoded.height(), 2);
        assert_original_hash(&jpeg_file, &original_hash, "geometry preview/export");
        let orientation_after = get_photo_metadata(&created.root_path, &photo_id)
            .expect("read metadata after")
            .expect("metadata after")
            .orientation;
        assert_eq!(orientation_after, orientation_before);

        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["geometry"]["crop"]["width"], 0.5);
        assert_eq!(settings["geometry"]["rotation"], 90.0);
        assert_eq!(settings["geometry"]["flip_horizontal"], true);

        remove_library_root(&workspace);
    }

    #[test]
    fn blocks_unsupported_lens_and_geometry_export_states() {
        let workspace = unique_library_root("core-unsupported-geometry");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let lens_output_path = export_root.join("lens-export.jpg");
        let transform_output_path = export_root.join("transform-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_geometry_source_jpeg(&jpeg_file);
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

        let graph =
            silica_storage::load_active_edit_graph_or_default(&created.root_path, &photo_id)
                .expect("load default graph")
                .expect("default graph");
        let lens_graph =
            silica_edit::apply_lens_adjustments(&graph, true, false, 0.0, 0.0, "unix:lens")
                .expect("build unsupported lens graph");
        silica_storage::commit_edit_graph(&created.root_path, lens_graph)
            .expect("seed unsupported lens state");
        let lens_error = export_photo_jpeg_srgb(&created.root_path, &photo_id, &lens_output_path)
            .expect_err("active lens export unsupported");
        assert!(matches!(lens_error, CoreError::ExportBlocked(_)));
        assert!(!lens_output_path.exists());

        let transform_graph = silica_edit::apply_geometry_transform(
            &graph,
            0.0,
            0.0,
            0.0,
            125.0,
            0.0,
            0.0,
            "unix:transform",
        )
        .expect("build unsupported transform graph");
        silica_storage::commit_edit_graph(&created.root_path, transform_graph)
            .expect("seed unsupported transform state");
        let transform_error =
            export_photo_jpeg_srgb(&created.root_path, &photo_id, &transform_output_path)
                .expect_err("active transform export unsupported");
        assert!(matches!(transform_error, CoreError::ExportBlocked(_)));
        assert!(!transform_output_path.exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_commits_and_exports_color_presence_through_core() {
        let workspace = unique_library_root("core-color-presence");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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

        let preview = preview_color_presence_edit(&created.root_path, &photo_id, 24.0, -8.5)
            .expect("preview color presence")
            .expect("preview result");

        assert_eq!(preview.status, PhotoPreviewStatus::Ready);
        assert_eq!(preview.vibrance, 24.0);
        assert_eq!(preview.saturation, -8.5);
        assert!(preview
            .develop_preview_bytes
            .as_ref()
            .is_some_and(|bytes| bytes.len() > 2));
        assert!(
            silica_storage::load_active_edit_graph(&created.root_path, &photo_id)
                .expect("load active graph after preview")
                .is_none(),
            "color presence preview must not write edit state"
        );

        let committed = commit_color_presence_edit(&created.root_path, &photo_id, 24.0, -8.5)
            .expect("commit color presence")
            .expect("commit result");
        assert_eq!(committed.vibrance, 24.0);
        assert_eq!(committed.saturation, -8.5);
        assert!(committed.persisted);

        let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
        assert_eq!(history.items.len(), 1);
        assert_eq!(history.items[0].label, "Color presence");

        let exported = export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");
        assert!(exported.bytes_written > 0);

        let latest = silica_storage::get_latest_export_record(&created.root_path, &photo_id)
            .expect("read latest export")
            .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["vibrance"], 24.0);
        assert_eq!(settings["saturation"], -8.5);

        remove_library_root(&workspace);
    }

    #[test]
    fn reset_and_basic_preset_commits_are_undoable_without_mutating_original() {
        let workspace = unique_library_root("core-basic-preset-reset");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
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

        let preset = commit_basic_preset_edit(
            &created.root_path,
            &photo_id,
            silica_edit::BasicPreset::WarmContrast,
        )
        .expect("commit preset")
        .expect("preset result");
        assert_eq!(preset.white_balance, silica_edit::WhiteBalance::Custom);
        assert_eq!(preset.temperature, 6200.0);
        assert_eq!(preset.contrast, 18.0);
        assert_eq!(preset.vibrance, 12.0);
        assert!(preset.persisted);

        let reset = commit_p0_basic_reset(&created.root_path, &photo_id)
            .expect("commit reset")
            .expect("reset result");
        assert_eq!(reset.white_balance, silica_edit::WhiteBalance::AsShot);
        assert_eq!(reset.temperature, 5200.0);
        assert_eq!(reset.tint, 0.0);
        assert_eq!(reset.exposure, 0.0);
        assert_eq!(reset.contrast, 0.0);
        assert_eq!(reset.highlights, 0.0);
        assert_eq!(reset.shadows, 0.0);
        assert_eq!(reset.whites, 0.0);
        assert_eq!(reset.blacks, 0.0);
        assert_eq!(reset.vibrance, 0.0);
        assert_eq!(reset.saturation, 0.0);

        let history = list_photo_history(&created.root_path, &photo_id).expect("history panel");
        assert_eq!(history.items.len(), 2);
        assert!(history.can_undo);

        undo_last_history_action(&created.root_path, &photo_id).expect("undo reset");
        let restored = get_photo_edit_state(&created.root_path, &photo_id)
            .expect("read restored preset")
            .expect("edit state");
        assert_eq!(restored.temperature, 6200.0);
        assert_eq!(restored.contrast, 18.0);
        assert_eq!(restored.vibrance, 12.0);
        assert_eq!(
            std::fs::read(&jpeg_file).expect("read original after"),
            original_before
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn sensitive_core_actions_append_action_log_entries() {
        let workspace = unique_library_root("core-action-log");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
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

        write_photo_sidecar(&created.root_path, &photo_id, "test")
            .expect("write sidecar")
            .expect("sidecar result");
        export_photo_jpeg_srgb(&created.root_path, &photo_id, &output_path)
            .expect("export photo")
            .expect("export result");
        clear_library_cache(&created.root_path).expect("clear cache");

        let entries = list_action_log_entries(&created.root_path, 20).expect("list action log");
        assert!(entries
            .iter()
            .any(|entry| entry.action_type == "import_reference"
                && entry.side_effect_category == "catalog_reference"));
        assert!(entries
            .iter()
            .any(|entry| entry.action_type == "sidecar_write"
                && entry.side_effect_category == "sidecar_write"
                && entry.subject_id.as_deref() == Some(photo_id.as_str())));
        assert!(entries.iter().any(|entry| entry.action_type == "export"
            && entry.side_effect_category == "file_write"
            && entry.subject_id.as_deref() == Some(photo_id.as_str())));
        assert!(entries
            .iter()
            .any(|entry| entry.action_type == "cache_clear"
                && entry.side_effect_category == "cache_delete"));

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

    #[test]
    fn raw_derived_jpeg_srgb_export_blocks_committed_manual_masks_before_output() {
        let workspace = unique_library_root("core-raw-export-mask-block");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let raw_file = import_root.join("sample.cr2");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        std::fs::write(&raw_file, b"raw placeholder").expect("write raw placeholder");
        let original_hash = file_hash(&raw_file);
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

        commit_manual_linear_gradient_mask(
            &created.root_path,
            &photo_id,
            "mask-linear-1",
            "Diagonal lift",
            100.0,
            0.0,
            false,
            0.0,
            0.0,
            1.0,
            1.0,
            Some(1.0),
            Some(0.0),
        )
        .expect("commit RAW manual mask")
        .expect("commit result");
        let probe = successful_raw_probe(&raw_file.display().to_string(), Some(5184), Some(3456));

        let error = export_raw_photo_jpeg_srgb_from_probe(
            &created.root_path,
            &photo_id,
            "A",
            &probe,
            &output_path,
        )
        .expect_err("RAW-derived masked export should block before output");

        assert!(matches!(error, CoreError::ExportBlocked(_)));
        assert!(error.to_string().contains("RAW-derived export"));
        assert!(!output_path.exists());
        assert!(!created
            .root_path
            .join("render-cache")
            .join("raw-export-sources")
            .exists());
        assert_original_hash(&raw_file, &original_hash, "blocked RAW mask export");

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
        assert_eq!(exported.icc_profile_sha256.len(), 64);
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
        assert_eq!(settings["icc_profile_embedded"], true);
        assert_eq!(settings["icc_profile_sha256"], exported.icc_profile_sha256);

        remove_library_root(&workspace);
    }

    #[test]
    fn export_metadata_policy_removes_gps_and_records_evidence() {
        let workspace = unique_library_root("core-export-metadata-policy");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-remove-gps.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg_with_exif(&jpeg_file);
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

        let exported = export_photo_jpeg_with_metadata_policy(
            &created.root_path,
            &photo_id,
            &output_path,
            PhotoExportColorProfile::Srgb,
            PhotoExportMetadataPolicy::RemoveGps,
        )
        .expect("export photo")
        .expect("export result");

        assert_eq!(exported.format, "jpeg");
        assert_eq!(
            std::fs::read(&jpeg_file).expect("read original after"),
            original_before
        );
        assert!(jpeg_contains_exif_make(&exported.output_path));
        assert!(!jpeg_has_exif_gps_ifd(&exported.output_path));

        let latest =
            silica_storage::get_latest_export_record(&created.root_path, &exported.photo_id)
                .expect("read latest export")
                .expect("latest export");
        let settings: serde_json::Value =
            serde_json::from_str(&latest.export_settings_json).expect("parse export settings");
        assert_eq!(settings["metadata_policy"], "remove_gps");
        assert_eq!(settings["source_metadata_segments"], 1);
        assert_eq!(settings["output_metadata_segments"], 1);
        assert_eq!(settings["source_metadata_copied"], true);
        assert_eq!(settings["gps_metadata_removed"], true);

        remove_library_root(&workspace);
    }

    #[test]
    fn recent_exports_report_missing_output_evidence() {
        let workspace = unique_library_root("core-recent-exports");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let jpeg_file = import_root.join("sample.jpg");
        let existing_output = export_root.join("sample-export.jpg");
        let missing_output = export_root.join("missing-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&existing_output, b"export bytes").expect("write export output");

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

        silica_storage::record_export(
            &created.root_path,
            &photo_id,
            &existing_output,
            r#"{"format":"jpeg"}"#,
        )
        .expect("record existing export");
        silica_storage::record_export(
            &created.root_path,
            &photo_id,
            &missing_output,
            r#"{"format":"png"}"#,
        )
        .expect("record missing export");

        let recent = list_recent_exports(&created.root_path, 2).expect("list recent exports");

        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].output_path, missing_output.display().to_string());
        assert!(!recent[0].output_exists);
        assert_eq!(recent[1].output_path, existing_output.display().to_string());
        assert!(recent[1].output_exists);
        assert!(!recent[0].created_at.is_empty());

        remove_library_root(&workspace);
    }

    #[test]
    fn export_settings_defaults_and_presets_flow_through_core_without_edit_history() {
        let workspace = unique_library_root("core-export-settings");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);

        let created = create_library(&library_root).expect("create library");
        import_folder(&created.root_path, &import_root).expect("import folder");

        let initial_catalog =
            get_export_settings_catalog(&created.root_path).expect("read export settings");
        assert_eq!(
            initial_catalog.default_settings,
            ExportSettings::jpeg_srgb_default()
        );

        let display_p3_settings = ExportSettings {
            color_profile: "display_p3".to_string(),
            ..ExportSettings::jpeg_srgb_default()
        };
        let preset = upsert_export_preset(
            &created.root_path,
            "Core Display P3 Review",
            display_p3_settings.clone(),
        )
        .expect("upsert preset through core");
        let updated_catalog = set_default_export_settings(
            &created.root_path,
            Some(&preset.id),
            display_p3_settings.clone(),
        )
        .expect("set default export settings through core");
        assert_eq!(updated_catalog.default_settings, display_p3_settings);
        assert_eq!(
            updated_catalog.default_preset_id.as_deref(),
            Some(preset.id.as_str())
        );

        let counts = durable_catalog_counts(&created.catalog_path);
        assert_eq!(counts.edit_states, 0);
        assert_eq!(counts.edit_history, 0);

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
    fn sidecar_status_after_history_is_exposed_through_core() {
        let workspace = unique_library_root("core-sidecar-status-history");
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

        write_photo_sidecar(&created.root_path, &photo_id, "0.1.0-alpha.1")
            .expect("write sidecar")
            .expect("sidecar result");
        let clean_status = get_photo_sidecar_status(&created.root_path, &photo_id)
            .expect("read clean status")
            .expect("clean status");
        assert_eq!(clean_status.conflict_state, "clean");

        commit_exposure_contrast_edit(&created.root_path, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("commit result");
        let stale_status = get_photo_sidecar_status(&created.root_path, &photo_id)
            .expect("read stale status")
            .expect("stale status");
        assert_eq!(stale_status.conflict_state, "catalog_newer");

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

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DurableCatalogCounts {
        edit_states: i64,
        edit_history: i64,
        action_log: i64,
        exports: i64,
        cache_records: i64,
    }

    fn durable_catalog_counts(catalog_path: &Path) -> DurableCatalogCounts {
        let connection = silica_storage::open_catalog(catalog_path).expect("open catalog");
        DurableCatalogCounts {
            edit_states: connection
                .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
                .expect("count edit states"),
            edit_history: connection
                .query_row("SELECT COUNT(*) FROM edit_history", [], |row| row.get(0))
                .expect("count edit history"),
            action_log: connection
                .query_row("SELECT COUNT(*) FROM action_log", [], |row| row.get(0))
                .expect("count action log"),
            exports: connection
                .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
                .expect("count exports"),
            cache_records: connection
                .query_row("SELECT COUNT(*) FROM cache_records", [], |row| row.get(0))
                .expect("count cache records"),
        }
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

    fn write_source_jpeg_with_exif(path: &Path) {
        write_source_jpeg(path);
        let bytes = std::fs::read(path).expect("read source jpeg");
        let with_exif = insert_app1_exif_segment(&bytes, &minimal_exif_with_gps());
        std::fs::write(path, with_exif).expect("write source jpeg exif");
    }

    fn insert_app1_exif_segment(jpeg: &[u8], exif: &[u8]) -> Vec<u8> {
        assert!(jpeg.starts_with(&[0xFF, 0xD8]));
        let segment_len = exif.len() + 2;
        let mut output = Vec::with_capacity(jpeg.len() + segment_len + 2);
        output.extend_from_slice(&jpeg[..2]);
        output.extend_from_slice(&[0xFF, 0xE1]);
        output.extend_from_slice(&(segment_len as u16).to_be_bytes());
        output.extend_from_slice(exif);
        output.extend_from_slice(&jpeg[2..]);
        output
    }

    fn minimal_exif_with_gps() -> Vec<u8> {
        let make_offset = 38_u32;
        let gps_offset = 48_u32;
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&42_u16.to_le_bytes());
        tiff.extend_from_slice(&8_u32.to_le_bytes());
        tiff.extend_from_slice(&2_u16.to_le_bytes());
        tiff.extend_from_slice(&0x010F_u16.to_le_bytes());
        tiff.extend_from_slice(&2_u16.to_le_bytes());
        tiff.extend_from_slice(&10_u32.to_le_bytes());
        tiff.extend_from_slice(&make_offset.to_le_bytes());
        tiff.extend_from_slice(&0x8825_u16.to_le_bytes());
        tiff.extend_from_slice(&4_u16.to_le_bytes());
        tiff.extend_from_slice(&1_u32.to_le_bytes());
        tiff.extend_from_slice(&gps_offset.to_le_bytes());
        tiff.extend_from_slice(&0_u32.to_le_bytes());
        tiff.extend_from_slice(b"SilicaCam\0");
        tiff.extend_from_slice(&1_u16.to_le_bytes());
        tiff.extend_from_slice(&1_u16.to_le_bytes());
        tiff.extend_from_slice(&2_u16.to_le_bytes());
        tiff.extend_from_slice(&2_u32.to_le_bytes());
        tiff.extend_from_slice(b"N\0\0\0");
        tiff.extend_from_slice(&0_u32.to_le_bytes());

        let mut exif = b"Exif\0\0".to_vec();
        exif.extend_from_slice(&tiff);
        exif
    }

    fn jpeg_contains_exif_make(path: &Path) -> bool {
        std::fs::read(path)
            .expect("read jpeg")
            .windows(b"SilicaCam".len())
            .any(|window| window == b"SilicaCam")
    }

    fn jpeg_has_exif_gps_ifd(path: &Path) -> bool {
        let bytes = std::fs::read(path).expect("read jpeg");
        bytes.windows(2).any(|window| window == [0x25, 0x88])
            || bytes.windows(2).any(|window| window == [0x88, 0x25])
    }

    fn write_geometry_source_jpeg(path: &Path) {
        let image = image::RgbImage::from_fn(4, 3, |x, y| {
            image::Rgb([
                (32 + (x * 40)) as u8,
                (48 + (y * 50)) as u8,
                (96 + ((x + y) * 10)) as u8,
            ])
        });
        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write geometry source jpeg");
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
