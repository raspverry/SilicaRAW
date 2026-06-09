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

const LOCAL_ALPHA_JPEG_QUALITY: u8 = 90;

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

/// Build the preview session for one catalog photo.
pub fn open_photo_preview(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoPreviewSession>, CoreError> {
    let (photo_id, file_name, render_plan) = match preview_render_plan(library_root_path, photo_id)?
    {
        Some(plan) => plan,
        None => return Ok(None),
    };

    Ok(Some(PhotoPreviewSession {
        photo_id,
        file_name,
        source_path: render_plan.source_path,
        status: preview_status_from_render(render_plan.status),
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

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        status: preview_status_from_render(request.status),
        exposure: request.exposure,
        contrast: request.contrast,
        message: request.message,
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
    fn opens_preview_session_with_ready_and_blocked_states() {
        let workspace = unique_library_root("core-preview");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");
        let raw_file = import_root.join("sample.dng");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&jpeg_file, b"jpeg placeholder bytes").expect("write jpeg");
        std::fs::write(&raw_file, b"raw placeholder bytes").expect("write raw");
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

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

        let raw_preview = open_photo_preview(&created.root_path, &raw_id)
            .expect("open raw preview")
            .expect("raw preview session");
        assert_eq!(raw_preview.status, PhotoPreviewStatus::BlockedByDecode);
        assert!(raw_preview.message.contains("Core Image RAW preview"));

        let unsupported_preview = open_photo_preview(&created.root_path, &unsupported_id)
            .expect("open unsupported preview")
            .expect("unsupported preview session");
        assert_eq!(unsupported_preview.status, PhotoPreviewStatus::Unsupported);

        assert!(open_photo_preview(&created.root_path, "missing-photo")
            .expect("missing preview lookup")
            .is_none());

        remove_library_root(&workspace);
    }

    #[test]
    fn previews_without_write_and_commits_exposure_contrast_edit() {
        let workspace = unique_library_root("core-edit-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&jpeg_file, b"jpeg placeholder bytes").expect("write jpeg");

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
