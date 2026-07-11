use std::path::Path;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use super::append_core_action_log;
use super::detail_unsupported_message;
use super::export_color_presence_from_render;
use super::export_detail_from_render;
use super::export_geometry_from_render;
use super::export_hsl_color_mixer_from_render;
use super::export_manual_masks_from_render;
use super::export_tone_curve_from_render;
use super::export_tone_recovery_from_render;
use super::export_white_balance_from_render;
use super::geometry_unsupported_message;
use super::lens_unsupported_message;
use super::mark_runtime_missing_source;
use super::preview_render_plan;
use super::preview_status_from_render;
use super::render_color_presence_from_graph;
use super::render_detail_from_graph;
use super::render_geometry_from_graph;
use super::render_hsl_color_mixer_from_graph;
use super::render_tone_curve_from_graph;
use super::render_tone_recovery_from_graph;
use super::render_white_balance_from_graph;
use super::white_balance_render_mode_string;
use super::CatalogRebuildDryRunReport;
use super::CoreError;
use super::FolderImportOptions;
use super::LibrarySession;
use super::McpPhotoReadRecord;
use super::PhotoSidecarStatus;
use super::SidecarWriteResult;
use super::ValidatedSidecar;
use super::LOCAL_ALPHA_DEVELOP_PREVIEW_QUALITY;
use super::LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE;
use super::LOCAL_ALPHA_LOUPE_PREVIEW_QUALITY;
use super::LOCAL_ALPHA_THUMBNAIL_MAX_EDGE;
use super::LOCAL_ALPHA_THUMBNAIL_QUALITY;

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

/// Read-only status for one disposable cache directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryCacheDirectoryStatus {
    pub name: String,
    pub path: PathBuf,
    pub exists: bool,
    pub byte_size: u64,
    pub file_count: u64,
}

/// Read-only status for all disposable cache directories in one library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryCacheStatusSession {
    pub library_root_path: PathBuf,
    pub directories: Vec<LibraryCacheDirectoryStatus>,
    pub total_bytes: u64,
    pub cache_record_count: usize,
    pub message: String,
}

impl From<silica_storage::CacheStatusSummary> for LibraryCacheStatusSession {
    fn from(summary: silica_storage::CacheStatusSummary) -> Self {
        Self {
            library_root_path: summary.library_root_path,
            directories: summary
                .directories
                .into_iter()
                .map(|directory| LibraryCacheDirectoryStatus {
                    name: directory.name,
                    path: directory.path,
                    exists: directory.exists,
                    byte_size: directory.byte_size,
                    file_count: directory.file_count,
                })
                .collect(),
            total_bytes: summary.total_bytes,
            cache_record_count: summary.cache_record_count,
            message: summary.message,
        }
    }
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
    let photos = silica_storage::list_library_photos(library_root_path)?;
    Ok(photos
        .into_iter()
        .map(mark_runtime_missing_source)
        .collect())
}

/// Query imported catalog photos by bounded page without cache hydration.
/// Legacy catalogs may be migrated by storage before the read-only page query runs.
pub fn query_library_photos(
    library_root_path: impl AsRef<Path>,
    request: silica_storage::LibraryQueryRequest,
) -> Result<silica_storage::LibraryQueryPage<silica_storage::LibraryPhotoGridItem>, CoreError> {
    let mut page = silica_storage::query_library_photos(library_root_path, request)?;
    page.items = page
        .items
        .into_iter()
        .map(mark_runtime_missing_source)
        .collect();
    Ok(page)
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

/// Read one photo row for MCP through Core APIs without hydrating caches.
pub fn get_mcp_photo_read_record(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<McpPhotoReadRecord>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let Some(candidate) = silica_storage::get_photo_preview_candidate(library_root_path, photo_id)?
    else {
        return Ok(None);
    };
    let flags = silica_storage::get_photo_flags(library_root_path, photo_id)?;

    Ok(Some(McpPhotoReadRecord {
        photo_id: candidate.photo_id,
        file_name: candidate.file_name,
        path: candidate.path,
        unsupported: candidate.unsupported,
        rating: flags.as_ref().map(|value| value.rating).unwrap_or(0),
        picked: flags.as_ref().map(|value| value.picked).unwrap_or(false),
        rejected: flags.as_ref().map(|value| value.rejected).unwrap_or(false),
        color_label: flags.and_then(|value| value.color_label),
    }))
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
    if !is_supported_raster_source_path(&source_path) {
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
            message: "Histogram unavailable for source formats outside the supported local alpha raster path.".to_string(),
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

/// Read disposable library cache status without removing catalog or original files.
pub fn get_library_cache_status(
    library_root_path: impl AsRef<Path>,
) -> Result<LibraryCacheStatusSession, CoreError> {
    silica_storage::get_disposable_cache_status(library_root_path)
        .map(LibraryCacheStatusSession::from)
        .map_err(CoreError::from)
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

fn ensure_jpeg_loupe_preview_cache(
    library_root_path: &Path,
    photo_id: &str,
    source_path: &str,
) -> Result<Option<Vec<u8>>, CoreError> {
    let source_path = PathBuf::from(source_path);
    if !is_supported_raster_source_path(&source_path) || !source_path.is_file() {
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

pub(super) fn record_brush_mask_raster_caches(
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

pub(super) fn write_jpeg_develop_preview_bytes(
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
    if !is_supported_raster_source_path(&source_path) || !source_path.is_file() {
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
    !photo.missing
        && !photo.unsupported
        && matches!(photo.file_type.as_str(), "JPG" | "JPEG" | "PNG" | "TIFF")
}

pub(super) fn is_supported_raster_source_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "tif" | "tiff"
            )
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
