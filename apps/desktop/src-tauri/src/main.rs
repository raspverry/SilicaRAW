#[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
mod metal_host_spike;

use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCommandResponse {
    ok: bool,
    command: &'static str,
    message: String,
    data: Option<DesktopCommandData>,
    error: Option<DesktopCommandError>,
}

impl DesktopCommandResponse {
    fn ok(command: &'static str, message: impl Into<String>, data: DesktopCommandData) -> Self {
        Self {
            ok: true,
            command,
            message: message.into(),
            data: Some(data),
            error: None,
        }
    }

    fn empty(command: &'static str, message: impl Into<String>) -> Self {
        Self {
            ok: true,
            command,
            message: message.into(),
            data: None,
            error: None,
        }
    }

    fn error(
        command: &'static str,
        error: silica_core::CoreError,
        context: DesktopCommandContext,
    ) -> Self {
        let kind = core_error_kind(&error).to_string();
        let message = error.to_string();
        Self {
            ok: false,
            command,
            message: message.clone(),
            data: None,
            error: Some(DesktopCommandError {
                kind,
                message,
                context,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    tag = "kind",
    rename_all_fields = "camelCase"
)]
enum DesktopCommandData {
    LibrarySession {
        root_path: String,
        catalog_path: String,
        schema_version: i64,
    },
    ImportSummary {
        folder_path: String,
        scanned_files: usize,
        supported_files: usize,
        unsupported_files: usize,
        originals_unchanged: bool,
    },
    PhotoGrid {
        photos: Vec<DesktopPhotoGridItem>,
    },
    PhotoFlags {
        photo_id: String,
        rating: u8,
        picked: bool,
        rejected: bool,
        color_label: Option<String>,
    },
    PhotoPreview {
        photo_id: String,
        file_name: String,
        source_path: String,
        preview_bytes: Option<Vec<u8>>,
        status: &'static str,
        message: String,
    },
    EditPreview {
        photo_id: String,
        source_path: String,
        status: &'static str,
        exposure: f64,
        contrast: f64,
        develop_preview_bytes: Option<Vec<u8>>,
        message: String,
    },
    EditCommit {
        photo_id: String,
        exposure: f64,
        contrast: f64,
        persisted: bool,
        message: String,
    },
    Export {
        photo_id: String,
        source_path: String,
        output_path: String,
        format: String,
        color_profile: String,
        bytes_written: u64,
        export_record_id: String,
        message: String,
    },
}

impl DesktopCommandData {
    #[cfg(test)]
    fn kind(&self) -> &'static str {
        match self {
            Self::LibrarySession { .. } => "librarySession",
            Self::ImportSummary { .. } => "importSummary",
            Self::PhotoGrid { .. } => "photoGrid",
            Self::PhotoFlags { .. } => "photoFlags",
            Self::PhotoPreview { .. } => "photoPreview",
            Self::EditPreview { .. } => "editPreview",
            Self::EditCommit { .. } => "editCommit",
            Self::Export { .. } => "export",
        }
    }

    #[cfg(test)]
    fn root_path(&self) -> Option<String> {
        match self {
            Self::LibrarySession { root_path, .. } => Some(root_path.clone()),
            _ => None,
        }
    }

    #[cfg(test)]
    fn catalog_path(&self) -> Option<String> {
        match self {
            Self::LibrarySession { catalog_path, .. } => Some(catalog_path.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopPhotoGridItem {
    photo_id: String,
    file_name: String,
    path: String,
    file_type: String,
    thumbnail_path: Option<String>,
    thumbnail_bytes: Option<Vec<u8>>,
    missing: bool,
    unsupported: bool,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
}

impl From<silica_core::LibraryPhotoGridItem> for DesktopPhotoGridItem {
    fn from(photo: silica_core::LibraryPhotoGridItem) -> Self {
        Self {
            photo_id: photo.photo_id,
            file_name: photo.file_name,
            path: photo.path,
            file_type: photo.file_type,
            thumbnail_bytes: photo
                .thumbnail_path
                .as_ref()
                .and_then(|path| std::fs::read(path).ok()),
            thumbnail_path: photo.thumbnail_path,
            missing: photo.missing,
            unsupported: photo.unsupported,
            rating: photo.rating,
            picked: photo.picked,
            rejected: photo.rejected,
            color_label: photo.color_label,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCommandContext {
    library_path: Option<String>,
    folder_path: Option<String>,
    output_path: Option<String>,
    photo_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCommandError {
    kind: String,
    message: String,
    context: DesktopCommandContext,
}

#[tauri::command]
fn create_library(path: String) -> DesktopCommandResponse {
    let command = "create_library";
    match silica_core::create_library(PathBuf::from(&path)) {
        Ok(session) => DesktopCommandResponse::ok(
            command,
            format!("Library created: {}", session.root_path.display()),
            library_session_data(session),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn open_library(path: String) -> DesktopCommandResponse {
    let command = "open_library";
    match silica_core::open_library(PathBuf::from(&path)) {
        Ok(session) => DesktopCommandResponse::ok(
            command,
            format!("Library opened: {}", session.root_path.display()),
            library_session_data(session),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn import_folder(library_path: String, folder_path: String) -> DesktopCommandResponse {
    let command = "import_folder";
    match silica_core::import_folder(PathBuf::from(&library_path), PathBuf::from(&folder_path)) {
        Ok(summary) => DesktopCommandResponse::ok(
            command,
            format!(
                "Imported {} supported file(s) by reference; originals unchanged.",
                summary.supported_files
            ),
            DesktopCommandData::ImportSummary {
                folder_path: summary.folder_path.display().to_string(),
                scanned_files: summary.scanned_files,
                supported_files: summary.supported_files,
                unsupported_files: summary.unsupported_files,
                originals_unchanged: true,
            },
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                folder_path: Some(folder_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn list_library_photos(library_path: String) -> DesktopCommandResponse {
    let command = "list_library_photos";
    match silica_core::list_library_photos(PathBuf::from(&library_path)) {
        Ok(photos) => {
            let photos = photos.into_iter().map(DesktopPhotoGridItem::from).collect();
            DesktopCommandResponse::ok(
                command,
                "Library grid loaded.",
                DesktopCommandData::PhotoGrid { photos },
            )
        }
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn set_photo_flags(
    library_path: String,
    photo_id: String,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
) -> DesktopCommandResponse {
    let command = "set_photo_flags";
    match silica_core::set_photo_flags(
        PathBuf::from(&library_path),
        photo_id.clone(),
        rating,
        picked,
        rejected,
        color_label,
    ) {
        Ok(flags) => {
            DesktopCommandResponse::ok(command, "Photo flags updated.", photo_flags_data(flags))
        }
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_photo_flags(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_flags";
    match silica_core::get_photo_flags(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(flags)) => {
            DesktopCommandResponse::ok(command, "Photo flags loaded.", photo_flags_data(flags))
        }
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn open_photo_preview(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "open_photo_preview";
    match silica_core::open_photo_preview(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::PhotoPreview {
                photo_id: preview.photo_id,
                file_name: preview.file_name,
                source_path: preview.source_path,
                preview_bytes: preview.preview_bytes,
                status: preview_status_text(preview.status),
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_exposure_contrast_edit(
    library_path: String,
    photo_id: String,
    exposure: f64,
    contrast: f64,
) -> DesktopCommandResponse {
    let command = "preview_exposure_contrast_edit";
    match silica_core::preview_exposure_contrast_edit(
        PathBuf::from(&library_path),
        &photo_id,
        exposure,
        contrast,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_exposure_contrast_edit(
    library_path: String,
    photo_id: String,
    exposure: f64,
    contrast: f64,
) -> DesktopCommandResponse {
    let command = "commit_exposure_contrast_edit";
    match silica_core::commit_exposure_contrast_edit(
        PathBuf::from(&library_path),
        &photo_id,
        exposure,
        contrast,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn export_photo_jpeg_srgb(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> DesktopCommandResponse {
    let command = "export_photo_jpeg_srgb";
    match silica_core::export_photo_jpeg_srgb(
        PathBuf::from(&library_path),
        &photo_id,
        PathBuf::from(&output_path),
    ) {
        Ok(Some(export)) => DesktopCommandResponse::ok(
            command,
            export.message.clone(),
            DesktopCommandData::Export {
                photo_id: export.photo_id,
                source_path: export.source_path,
                output_path: export.output_path.display().to_string(),
                format: export.format,
                color_profile: export.color_profile,
                bytes_written: export.bytes_written,
                export_record_id: export.export_record_id,
                message: export.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                output_path: Some(output_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

fn library_session_data(session: silica_core::LibrarySession) -> DesktopCommandData {
    DesktopCommandData::LibrarySession {
        root_path: session.root_path.display().to_string(),
        catalog_path: session.catalog_path.display().to_string(),
        schema_version: session.schema_version,
    }
}

fn photo_flags_data(flags: silica_core::PhotoFlags) -> DesktopCommandData {
    DesktopCommandData::PhotoFlags {
        photo_id: flags.photo_id,
        rating: flags.rating,
        picked: flags.picked,
        rejected: flags.rejected,
        color_label: flags.color_label,
    }
}

fn preview_status_text(status: silica_core::PhotoPreviewStatus) -> &'static str {
    match status {
        silica_core::PhotoPreviewStatus::Ready => "Ready",
        silica_core::PhotoPreviewStatus::BlockedByDecode => "BlockedByDecode",
        silica_core::PhotoPreviewStatus::Unsupported => "Unsupported",
    }
}

fn core_error_kind(error: &silica_core::CoreError) -> &'static str {
    match error {
        silica_core::CoreError::Storage(_) => "storage",
        silica_core::CoreError::EditGraph(_) => "editGraph",
        silica_core::CoreError::Export(_) => "export",
        silica_core::CoreError::ExportBlocked(_) => "exportBlocked",
    }
}

fn main() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
    let builder = builder.setup(metal_host_spike::install);

    builder
        .invoke_handler(tauri::generate_handler![
            create_library,
            open_library,
            import_folder,
            list_library_photos,
            set_photo_flags,
            get_photo_flags,
            open_photo_preview,
            preview_exposure_contrast_edit,
            commit_exposure_contrast_edit,
            export_photo_jpeg_srgb
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SilicaRAW desktop shell");
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn desktop_commands_create_and_open_library() {
        let root = unique_library_root("desktop");

        let created = super::create_library(root.display().to_string());
        let opened = super::open_library(root.display().to_string());

        assert!(created.ok);
        assert!(created.error.is_none());
        assert_eq!(created.command, "create_library");
        assert_eq!(response_data(&created).kind(), "librarySession");
        assert_eq!(
            response_data(&created).root_path(),
            Some(root.display().to_string())
        );
        assert!(created.message.contains("Library created"));

        assert!(opened.ok);
        assert_eq!(opened.command, "open_library");
        assert_eq!(response_data(&opened).kind(), "librarySession");
        assert_eq!(
            response_data(&opened).catalog_path(),
            response_data(&created).catalog_path()
        );

        let missing = super::open_library(root.join("missing").display().to_string());
        assert!(!missing.ok);
        assert_eq!(missing.command, "open_library");
        let error = missing.error.as_ref().expect("structured error");
        assert_eq!(error.kind, "storage");
        assert_eq!(
            error.context.library_path,
            Some(root.join("missing").display().to_string())
        );
        assert!(error.message.contains("not a directory"));

        remove_library_root(&root);
    }

    #[test]
    fn desktop_commands_set_and_get_photo_flags() {
        let workspace = unique_library_root("desktop-flags");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let updated = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            2,
            true,
            false,
            Some("blue".to_string()),
        );
        assert!(updated.ok);
        match response_data(&updated) {
            super::DesktopCommandData::PhotoFlags {
                rating,
                picked,
                color_label,
                ..
            } => {
                assert_eq!(*rating, 2);
                assert!(*picked);
                assert_eq!(color_label.as_deref(), Some("blue"));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let reopened = super::get_photo_flags(library_root.display().to_string(), photo_id);
        assert!(reopened.ok);
        assert_eq!(response_data(&reopened), response_data(&updated));

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_imports_folder_by_reference() {
        let workspace = unique_library_root("desktop-import");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
        std::fs::write(&unsupported_file, b"not a photo").expect("write unsupported");

        silica_core::create_library(&library_root).expect("create library");

        let imported = super::import_folder(
            library_root.display().to_string(),
            import_root.display().to_string(),
        );

        assert!(imported.ok);
        match response_data(&imported) {
            super::DesktopCommandData::ImportSummary {
                scanned_files,
                supported_files,
                unsupported_files,
                originals_unchanged,
                ..
            } => {
                assert_eq!(*scanned_files, 2);
                assert_eq!(*supported_files, 1);
                assert_eq!(*unsupported_files, 1);
                assert!(*originals_unchanged);
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(supported_file.is_file());
        assert!(unsupported_file.is_file());

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_lists_library_photos_for_grid() {
        let workspace = unique_library_root("desktop-grid");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");
        let jpeg_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let flags = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id,
            4,
            true,
            false,
            Some("green".to_string()),
        );
        assert!(flags.ok);

        let grid = super::list_library_photos(library_root.display().to_string());
        assert!(grid.ok);
        match response_data(&grid) {
            super::DesktopCommandData::PhotoGrid { photos } => {
                assert_eq!(photos.len(), 3);
                assert!(photos.iter().any(|photo| photo.file_name == "sample.DNG"
                    && photo.rating == 4
                    && photo.picked
                    && photo.color_label.as_deref() == Some("green")));
                assert!(photos.iter().any(|photo| {
                    photo.file_name == "sample.jpg"
                        && photo.thumbnail_path.is_some()
                        && photo
                            .thumbnail_bytes
                            .as_ref()
                            .is_some_and(|bytes| !bytes.is_empty())
                }));
                assert!(photos
                    .iter()
                    .any(|photo| photo.file_name == "notes.txt" && photo.unsupported));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_opens_photo_preview_status() {
        let workspace = unique_library_root("desktop-preview");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::open_photo_preview(library_root.display().to_string(), photo_id);

        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::PhotoPreview {
                file_name,
                status,
                message,
                preview_bytes,
                ..
            } => {
                assert_eq!(file_name, "sample.jpg");
                assert_eq!(*status, "Ready");
                assert!(message.contains("display-profile-aware"));
                assert!(preview_bytes.as_ref().is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_exposure_contrast_edit() {
        let workspace = unique_library_root("desktop-edit-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.5,
            -8.0,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                exposure,
                contrast,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(*exposure, 0.5);
                assert_eq!(*contrast, -8.0);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id,
            0.5,
            -8.0,
        );
        assert!(committed.ok);
        match response_data(&committed) {
            super::DesktopCommandData::EditCommit {
                exposure,
                contrast,
                persisted,
                ..
            } => {
                assert_eq!(*exposure, 0.5);
                assert_eq!(*contrast, -8.0);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_exports_photo_jpeg_srgb() {
        let workspace = unique_library_root("desktop-export");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let supported_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        silica_core::commit_exposure_contrast_edit(&library_root, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("committed edit");

        let export = super::export_photo_jpeg_srgb(
            library_root.display().to_string(),
            photo_id,
            output_path.display().to_string(),
        );

        assert!(export.ok);
        match response_data(&export) {
            super::DesktopCommandData::Export {
                format,
                color_profile,
                output_path: actual_output_path,
                ..
            } => {
                assert_eq!(format, "jpeg");
                assert_eq!(color_profile, "srgb");
                assert_eq!(actual_output_path, &output_path.display().to_string());
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(output_path.is_file());

        remove_library_root(&workspace);
    }

    fn response_data(response: &super::DesktopCommandResponse) -> &super::DesktopCommandData {
        response.data.as_ref().expect("response data")
    }

    fn stable_catalog_id(prefix: &str, value: &str) -> String {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in value.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{prefix}-{hash:016x}")
    }

    fn unique_library_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "silicaraw-desktop-library-{label}-{}-{nanos}",
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
