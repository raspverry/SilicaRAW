use std::path::PathBuf;

use crate::dto::*;

#[tauri::command]
pub(crate) fn export_photo_jpeg_srgb(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> DesktopCommandResponse {
    let command = "export_photo_jpeg_srgb";
    desktop_photo_export_response(
        command,
        silica_core::export_photo_jpeg_srgb(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
pub(crate) fn export_photo_jpeg(
    library_path: String,
    photo_id: String,
    output_path: String,
    color_profile: Option<String>,
    metadata_policy: Option<String>,
) -> DesktopCommandResponse {
    let command = "export_photo_jpeg";
    let requested_profile = match parse_export_color_profile(color_profile.as_deref()) {
        Ok(profile) => profile,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    output_path: Some(output_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    let requested_metadata_policy = match parse_export_metadata_policy(metadata_policy.as_deref()) {
        Ok(policy) => policy,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    output_path: Some(output_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    desktop_photo_export_response(
        command,
        silica_core::export_photo_jpeg_with_metadata_policy(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
            requested_profile,
            requested_metadata_policy,
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
pub(crate) fn export_photo_png(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> DesktopCommandResponse {
    let command = "export_photo_png";
    desktop_photo_export_response(
        command,
        silica_core::export_photo_png(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
pub(crate) fn export_photo_tiff(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> DesktopCommandResponse {
    let command = "export_photo_tiff";
    desktop_photo_export_response(
        command,
        silica_core::export_photo_tiff(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
pub(crate) fn get_export_settings(library_path: String) -> DesktopCommandResponse {
    let command = "get_export_settings";
    match silica_core::get_export_settings_catalog(PathBuf::from(&library_path)) {
        Ok(catalog) => DesktopCommandResponse::ok(
            command,
            "Export settings loaded.",
            export_settings_catalog_data(catalog),
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
pub(crate) fn get_recent_exports(
    library_path: String,
    limit: Option<usize>,
) -> DesktopCommandResponse {
    let command = "get_recent_exports";
    let limit = limit.unwrap_or(10).min(50);
    match silica_core::list_recent_exports(PathBuf::from(&library_path), limit) {
        Ok(exports) => DesktopCommandResponse::ok(
            command,
            "Recent exports loaded.",
            DesktopCommandData::RecentExports {
                exports: exports.into_iter().map(desktop_recent_export).collect(),
                message: "Recent exports loaded.".to_string(),
            },
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
pub(crate) fn save_export_settings(
    library_path: String,
    preset_id: Option<String>,
    format: Option<String>,
    color_profile: Option<String>,
    quality: Option<u8>,
    metadata_policy: Option<String>,
) -> DesktopCommandResponse {
    let command = "save_export_settings";
    let settings = match export_settings_from_request(
        format.as_deref(),
        color_profile.as_deref(),
        quality,
        metadata_policy.as_deref(),
    ) {
        Ok(settings) => settings,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::set_default_export_settings(
        PathBuf::from(&library_path),
        preset_id.as_deref(),
        settings,
    ) {
        Ok(catalog) => DesktopCommandResponse::ok(
            command,
            "Export settings saved.",
            export_settings_catalog_data(catalog),
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
pub(crate) fn save_export_preset(
    library_path: String,
    name: String,
    format: Option<String>,
    color_profile: Option<String>,
    quality: Option<u8>,
    metadata_policy: Option<String>,
) -> DesktopCommandResponse {
    let command = "save_export_preset";
    let settings = match export_settings_from_request(
        format.as_deref(),
        color_profile.as_deref(),
        quality,
        metadata_policy.as_deref(),
    ) {
        Ok(settings) => settings,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::upsert_export_preset(PathBuf::from(&library_path), name, settings.clone()) {
        Ok(preset) => match silica_core::set_default_export_settings(
            PathBuf::from(&library_path),
            Some(&preset.id),
            settings,
        ) {
            Ok(catalog) => DesktopCommandResponse::ok(
                command,
                "Export preset saved.",
                export_settings_catalog_data(catalog),
            ),
            Err(error) => DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            ),
        },
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

fn desktop_photo_export_response(
    command: &'static str,
    export_result: Result<Option<silica_core::PhotoExportSession>, silica_core::CoreError>,
    library_path: String,
    output_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    match export_result {
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
                source_sha256: export.source_sha256,
                output_sha256: export.output_sha256,
                icc_profile_embedded: export.icc_profile_embedded,
                icc_profile_sha256: export.icc_profile_sha256,
                decoder_backend: export.decoder_backend,
                input_profile: export.input_profile,
                working_space: export.working_space,
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

#[tauri::command]
pub(crate) fn get_library_cache_status(library_path: String) -> DesktopCommandResponse {
    let command = "get_library_cache_status";
    match silica_core::get_library_cache_status(PathBuf::from(&library_path)) {
        Ok(status) => DesktopCommandResponse::ok(
            command,
            status.message.clone(),
            DesktopCommandData::CacheStatus {
                library_root_path: status.library_root_path.display().to_string(),
                directories: status
                    .directories
                    .into_iter()
                    .map(DesktopCacheDirectoryStatus::from)
                    .collect(),
                total_bytes: status.total_bytes,
                cache_record_count: status.cache_record_count,
                message: status.message,
            },
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
pub(crate) fn clear_library_cache(library_path: String) -> DesktopCommandResponse {
    let command = "clear_library_cache";
    match silica_core::clear_library_cache(PathBuf::from(&library_path)) {
        Ok(summary) => DesktopCommandResponse::ok(
            command,
            summary.message.clone(),
            DesktopCommandData::CacheClear {
                cleared_directories: summary.cleared_directories,
                recreated_directories: summary.recreated_directories,
                removed_cache_records: summary.removed_cache_records,
                message: summary.message,
            },
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
