use std::path::PathBuf;

use tauri::{path::BaseDirectory, Manager};

use crate::dto::*;

#[tauri::command]
pub(crate) fn read_app_session(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => read_app_session_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "read_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn write_app_session(
    app: tauri::AppHandle,
    session: DesktopAppSession,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => write_app_session_at_path(session_path, session),
        Err(error) => DesktopCommandResponse::error(
            "write_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn reset_app_session(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn record_app_session_layout(
    app: tauri::AppHandle,
    layout: DesktopLayoutPreferences,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_layout_at_path(session_path, layout),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_layout",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn reset_app_session_layout(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_layout_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session_layout",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn record_app_session_appearance(
    app: tauri::AppHandle,
    appearance: DesktopAppearancePreferences,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_appearance_at_path(session_path, appearance),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_appearance",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn reset_app_session_appearance(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_appearance_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session_appearance",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn record_app_session_library_preferences(
    app: tauri::AppHandle,
    library: DesktopLibraryPreferences,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_library_preferences_at_path(session_path, library),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_library_preferences",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn reset_app_session_library_preferences(
    app: tauri::AppHandle,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_library_preferences_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session_library_preferences",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn inspect_app_session(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => inspect_app_session_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "inspect_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn resolve_launch_restore(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => resolve_launch_restore_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "resolve_launch_restore",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
pub(crate) fn record_app_session_selection(
    app: tauri::AppHandle,
    library_path: String,
    selected_photo_id: Option<String>,
    mode: String,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_selection_at_path(
            session_path,
            library_path,
            selected_photo_id,
            mode,
        ),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_selection",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

pub(crate) fn resolve_app_session_path(
    app: &tauri::AppHandle,
) -> Result<PathBuf, silica_core::CoreError> {
    app.path()
        .resolve("app-session.json", BaseDirectory::AppConfig)
        .map_err(|error| {
            silica_core::CoreError::AppSession(format!("resolve app session path: {error}"))
        })
}

pub(crate) fn read_app_session_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "read_app_session";
    match silica_core::load_app_session(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session loaded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn write_app_session_at_path(
    session_path: PathBuf,
    session: DesktopAppSession,
) -> DesktopCommandResponse {
    let command = "write_app_session";
    let session = match session.into_core() {
        Ok(session) => session,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };

    match silica_core::write_app_session(&session_path, &session) {
        Ok(written) => DesktopCommandResponse::ok(
            command,
            "App session written.",
            DesktopCommandData::AppSessionWrite {
                session_path: written.session_path.display().to_string(),
                bytes_written: written.bytes_written,
            },
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn reset_app_session_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "reset_app_session";
    let session = silica_core::AppSession::default();
    match silica_core::write_app_session(&session_path, &session) {
        Ok(_) => DesktopCommandResponse::ok(
            command,
            "App session reset.",
            DesktopCommandData::AppSession {
                session_path: session_path.display().to_string(),
                session: DesktopAppSession::from_core(session),
                warnings: Vec::new(),
            },
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn record_app_session_layout_at_path(
    session_path: PathBuf,
    layout: DesktopLayoutPreferences,
) -> DesktopCommandResponse {
    let command = "record_app_session_layout";
    let layout = match layout.into_core() {
        Ok(layout) => layout,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };

    match silica_core::record_app_session_layout(&session_path, layout) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session layout recorded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn reset_app_session_layout_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "reset_app_session_layout";
    match silica_core::reset_app_session_layout(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session layout reset.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn record_app_session_appearance_at_path(
    session_path: PathBuf,
    appearance: DesktopAppearancePreferences,
) -> DesktopCommandResponse {
    let command = "record_app_session_appearance";
    let appearance = match appearance.into_core() {
        Ok(appearance) => appearance,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };

    match silica_core::record_app_session_appearance(&session_path, appearance) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session appearance recorded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn reset_app_session_appearance_at_path(
    session_path: PathBuf,
) -> DesktopCommandResponse {
    let command = "reset_app_session_appearance";
    match silica_core::reset_app_session_appearance(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session appearance reset.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn record_app_session_library_preferences_at_path(
    session_path: PathBuf,
    library: DesktopLibraryPreferences,
) -> DesktopCommandResponse {
    let command = "record_app_session_library_preferences";
    match silica_core::record_app_session_library_preferences(&session_path, library.into_core()) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session library preferences recorded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn reset_app_session_library_preferences_at_path(
    session_path: PathBuf,
) -> DesktopCommandResponse {
    let command = "reset_app_session_library_preferences";
    match silica_core::reset_app_session_library_preferences(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session library preferences reset.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn inspect_app_session_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "inspect_app_session";
    let exists = session_path.is_file();
    match silica_core::load_app_session(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session inspected.",
            DesktopCommandData::AppSessionInspection {
                session_path: session_path.display().to_string(),
                exists,
                warnings: app_session_warning_strings(&loaded.warnings),
            },
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn resolve_launch_restore_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "resolve_launch_restore";
    match silica_core::plan_app_session_restore(&session_path) {
        Ok(plan) => {
            let status = app_session_restore_status_string(plan.status).to_string();
            let state = if plan.status == silica_core::AppSessionRestoreStatus::Restored {
                "library".to_string()
            } else {
                "welcome".to_string()
            };
            let fallback_reason = if state == "welcome" {
                Some(status.clone())
            } else {
                None
            };
            DesktopCommandResponse::ok(
                command,
                "Launch restore resolved.",
                DesktopCommandData::LaunchRestore {
                    session_path: session_path.display().to_string(),
                    session: DesktopAppSession::from_core(plan.session),
                    warnings: app_session_warning_strings(&plan.warnings),
                    status,
                    state,
                    fallback_reason,
                    requested_mode: app_session_mode_string(plan.requested_mode).to_string(),
                    resolved_mode: app_session_mode_string(plan.resolved_mode).to_string(),
                    selected_photo_id: plan.selected_photo_id,
                    selected_photo_status: app_session_selected_photo_status_string(
                        plan.selected_photo_status,
                    )
                    .to_string(),
                    library_root_path: plan
                        .library_root_path
                        .map(|path| path.display().to_string()),
                    catalog_path: plan.catalog_path.map(|path| path.display().to_string()),
                    schema_version: plan.schema_version,
                },
            )
        }
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

pub(crate) fn record_app_session_selection_at_path(
    session_path: PathBuf,
    library_path: String,
    selected_photo_id: Option<String>,
    mode: String,
) -> DesktopCommandResponse {
    let command = "record_app_session_selection";
    let mode = match parse_desktop_app_session_mode(&mode) {
        Ok(mode) => mode,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };
    let selected_photo_id =
        selected_photo_id.and_then(|photo_id| (!photo_id.trim().is_empty()).then_some(photo_id));

    match silica_core::record_app_session_library_state(
        &session_path,
        PathBuf::from(&library_path),
        selected_photo_id,
        mode,
    ) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session selection recorded.",
            app_session_data(session_path, loaded),
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
