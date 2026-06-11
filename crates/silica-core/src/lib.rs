//! Core coordination boundary for SilicaRAW.
//!
//! Phase 4.2 starts the local library command surface.

use std::error::Error;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-core";

pub use silica_storage::CatalogRebuildDryRunAction;
pub use silica_storage::CatalogRebuildDryRunEntry;
pub use silica_storage::CatalogRebuildDryRunIssue;
pub use silica_storage::CatalogRebuildDryRunIssueKind;
pub use silica_storage::CatalogRebuildDryRunReport;
pub use silica_storage::CatalogRebuildFlagSource;
pub use silica_storage::LibraryPhotoGridItem;
pub use silica_storage::PhotoFlags;
pub use silica_storage::SidecarWriteResult;
pub use silica_storage::ValidatedSidecar;

const LOCAL_ALPHA_JPEG_QUALITY: u8 = 90;
const LOCAL_ALPHA_THUMBNAIL_QUALITY: u8 = 82;
const LOCAL_ALPHA_THUMBNAIL_MAX_EDGE: u32 = 320;
const LOCAL_ALPHA_LOUPE_PREVIEW_QUALITY: u8 = 88;
const LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE: u32 = 2048;
const LOCAL_ALPHA_DEVELOP_PREVIEW_QUALITY: u8 = 86;

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

/// Completed JPEG sRGB export returned through the core boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoExportSession {
    pub photo_id: String,
    pub source_path: String,
    pub output_path: PathBuf,
    pub format: String,
    pub color_profile: String,
    pub bytes_written: u64,
    pub export_record_id: String,
    pub message: String,
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
    EditGraph(silica_edit::EditGraphValidationError),
    Export(silica_export::ExportError),
    ExportBlocked(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "{error}"),
            Self::EditGraph(error) => write!(formatter, "{error}"),
            Self::Export(error) => write!(formatter, "{error}"),
            Self::ExportBlocked(message) => write!(formatter, "export blocked: {message}"),
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            Self::EditGraph(error) => Some(error),
            Self::Export(error) => Some(error),
            Self::ExportBlocked(_) => None,
        }
    }
}

impl From<silica_storage::LibraryStorageError> for CoreError {
    fn from(error: silica_storage::LibraryStorageError) -> Self {
        Self::Storage(error)
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
    silica_storage::import_folder(library_root_path, folder_path).map_err(CoreError::from)
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

    let export_result = silica_export::export_jpeg_srgb(silica_export::JpegSrgbExportRequest {
        source_path: PathBuf::from(&render_request.source_path),
        output_path: output_path.to_path_buf(),
        exposure: render_request.exposure,
        contrast: render_request.contrast,
        quality: render_request.quality,
    })?;
    let format = export_format_string(export_result.format).to_string();
    let color_profile = export_color_profile_string(export_result.color_profile).to_string();
    let settings_json = serde_json::json!({
        "format": format,
        "color_profile": color_profile,
        "quality": render_request.quality,
        "exposure": render_request.exposure,
        "contrast": render_request.contrast,
        "source_path": render_request.source_path,
        "output_path": render_request.output_path,
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
        color_profile,
        bytes_written: export_result.bytes_written,
        export_record_id: export_record.id,
        message: "JPEG sRGB export completed.".to_string(),
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
    let thumbnail_root = library_root_path.join("thumbnails");
    std::fs::create_dir_all(&thumbnail_root)
        .map_err(silica_storage::LibraryStorageError::from)
        .map_err(CoreError::from)?;

    for photo in photos
        .iter()
        .filter(|photo| is_jpeg_thumbnail_candidate(photo))
    {
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

fn export_color_profile_string(profile: silica_export::ExportColorProfile) -> &'static str {
    match profile {
        silica_export::ExportColorProfile::Srgb => "srgb",
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
        assert!(latest.export_settings_json.contains("\"srgb\""));

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
}
