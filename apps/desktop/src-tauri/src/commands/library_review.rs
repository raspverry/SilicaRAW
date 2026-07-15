use std::path::PathBuf;

use crate::dto::*;

use super::session::resolve_app_session_path;

#[tauri::command]
pub(crate) fn create_library(app: tauri::AppHandle, path: String) -> DesktopCommandResponse {
    let session_path = match resolve_app_session_path(&app) {
        Ok(session_path) => Some(session_path),
        Err(error) => {
            return DesktopCommandResponse::error(
                "create_library",
                error,
                DesktopCommandContext {
                    library_path: Some(path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    create_library_at_path(path, session_path)
}

pub(crate) fn create_library_at_path(
    path: String,
    session_path: Option<PathBuf>,
) -> DesktopCommandResponse {
    let command = "create_library";
    match silica_core::create_library(PathBuf::from(&path)) {
        Ok(session) => {
            if let Some(session_path) = session_path {
                if let Err(error) =
                    silica_core::record_app_session_recent_library(&session_path, &session)
                {
                    return DesktopCommandResponse::error(
                        command,
                        error,
                        DesktopCommandContext {
                            library_path: Some(path),
                            ..DesktopCommandContext::default()
                        },
                    );
                }
            }
            DesktopCommandResponse::ok(
                command,
                format!("Library created: {}", session.root_path.display()),
                library_session_data(session),
            )
        }
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
pub(crate) fn open_library(app: tauri::AppHandle, path: String) -> DesktopCommandResponse {
    let session_path = match resolve_app_session_path(&app) {
        Ok(session_path) => Some(session_path),
        Err(error) => {
            return DesktopCommandResponse::error(
                "open_library",
                error,
                DesktopCommandContext {
                    library_path: Some(path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    open_library_at_path(path, session_path)
}

pub(crate) fn open_library_at_path(
    path: String,
    session_path: Option<PathBuf>,
) -> DesktopCommandResponse {
    let command = "open_library";
    match silica_core::open_library(PathBuf::from(&path)) {
        Ok(session) => {
            if let Some(session_path) = session_path {
                if let Err(error) =
                    silica_core::record_app_session_recent_library(&session_path, &session)
                {
                    return DesktopCommandResponse::error(
                        command,
                        error,
                        DesktopCommandContext {
                            library_path: Some(path),
                            ..DesktopCommandContext::default()
                        },
                    );
                }
            }
            DesktopCommandResponse::ok(
                command,
                format!("Library opened: {}", session.root_path.display()),
                library_session_data(session),
            )
        }
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
pub(crate) fn import_folder(
    library_path: String,
    folder_path: String,
    recursive: Option<bool>,
) -> DesktopCommandResponse {
    let command = "import_folder";
    let options = silica_core::FolderImportOptions {
        recursive: recursive.unwrap_or(false),
    };
    match silica_core::import_folder_with_options(
        PathBuf::from(&library_path),
        PathBuf::from(&folder_path),
        options,
    ) {
        Ok(summary) => {
            let message = if summary.originals_unchanged {
                format!(
                    "Imported {} supported file(s) by reference; originals unchanged.",
                    summary.supported_files
                )
            } else {
                format!(
                    "Imported {} supported file(s) by reference; source fingerprints changed during import.",
                    summary.supported_files
                )
            };
            DesktopCommandResponse::ok(
                command,
                message,
                DesktopCommandData::ImportSummary {
                    folder_path: summary.folder_path.display().to_string(),
                    scanned_files: summary.scanned_files,
                    supported_files: summary.supported_files,
                    unsupported_files: summary.unsupported_files,
                    issues: summary
                        .issues
                        .into_iter()
                        .map(desktop_import_issue)
                        .collect(),
                    originals_unchanged: summary.originals_unchanged,
                },
            )
        }
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
pub(crate) fn list_library_photos(library_path: String) -> DesktopCommandResponse {
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
pub(crate) fn query_library_photos(
    library_path: String,
    request: DesktopLibraryQueryRequest,
) -> DesktopCommandResponse {
    let command = "query_library_photos";
    let query = match request.into_core() {
        Ok(query) => query,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            );
        }
    };

    match silica_core::query_library_photos_with_thumbnail_hydration(
        PathBuf::from(&library_path),
        query,
    ) {
        Ok(page) => DesktopCommandResponse::ok(
            command,
            "Library grid page loaded.",
            photo_grid_page_data(page),
        ),
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
pub(crate) fn set_photo_flags(
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
pub(crate) fn get_photo_flags(library_path: String, photo_id: String) -> DesktopCommandResponse {
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
pub(crate) fn get_photo_metadata(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_metadata";
    match silica_core::get_photo_metadata(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(metadata)) => DesktopCommandResponse::ok(
            command,
            "Photo metadata loaded.",
            photo_metadata_data(metadata),
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
pub(crate) fn get_ai_review_panel(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "get_ai_review_panel";
    match silica_core::get_ai_review_panel(PathBuf::from(&library_path), &photo_id) {
        Ok(panel) => {
            DesktopCommandResponse::ok(command, panel.message.clone(), ai_review_panel_data(panel))
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
pub(crate) fn approve_ai_suggestion(
    library_path: String,
    photo_id: String,
    result_id: String,
) -> DesktopCommandResponse {
    let command = "approve_ai_suggestion";
    match silica_core::approve_ai_suggestion(PathBuf::from(&library_path), &photo_id, &result_id) {
        Ok(Some(approval)) => DesktopCommandResponse::ok(
            command,
            "AI suggestion approved as an undoable edit checkpoint.",
            ai_suggestion_approval_data(approval),
        ),
        Ok(None) => DesktopCommandResponse::error_message(
            command,
            "AI suggestion approval skipped because the selected photo is unavailable.",
            "aiReview",
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
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
pub(crate) fn reject_ai_suggestion(
    library_path: String,
    photo_id: String,
    result_id: String,
) -> DesktopCommandResponse {
    let command = "reject_ai_suggestion";
    match silica_core::reject_ai_suggestion(PathBuf::from(&library_path), &photo_id, &result_id) {
        Ok(Some(rejection)) => DesktopCommandResponse::ok(
            command,
            "AI suggestion rejected; edit state is unchanged.",
            ai_suggestion_rejection_data(rejection),
        ),
        Ok(None) => DesktopCommandResponse::error_message(
            command,
            "AI suggestion rejection skipped because the selected photo is unavailable.",
            "aiReview",
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
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
pub(crate) fn open_photo_preview(library_path: String, photo_id: String) -> DesktopCommandResponse {
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
