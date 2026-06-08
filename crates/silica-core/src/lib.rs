//! Core coordination boundary for SilicaRAW.
//!
//! Phase 4.2 starts the local library command surface.

use std::error::Error;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-core";

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

/// Errors returned by core command APIs.
#[derive(Debug)]
pub enum CoreError {
    Storage(silica_storage::LibraryStorageError),
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for CoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
        }
    }
}

impl From<silica_storage::LibraryStorageError> for CoreError {
    fn from(error: silica_storage::LibraryStorageError) -> Self {
        Self::Storage(error)
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
    let candidate = match silica_storage::get_photo_preview_candidate(library_root_path, photo_id)
        .map_err(CoreError::from)?
    {
        Some(candidate) => candidate,
        None => return Ok(None),
    };

    let decode_plan = silica_decode::plan_preview_decode(&candidate.path, candidate.unsupported);
    let render_plan = silica_render::plan_preview_render(decode_plan);
    let status = match render_plan.status {
        silica_render::PreviewRenderStatus::Ready => PhotoPreviewStatus::Ready,
        silica_render::PreviewRenderStatus::BlockedByDecode => PhotoPreviewStatus::BlockedByDecode,
        silica_render::PreviewRenderStatus::Unsupported => PhotoPreviewStatus::Unsupported,
    };

    Ok(Some(PhotoPreviewSession {
        photo_id: candidate.photo_id,
        file_name: candidate.file_name,
        source_path: render_plan.source_path,
        status,
        message: render_plan.message,
    }))
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
}
