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
