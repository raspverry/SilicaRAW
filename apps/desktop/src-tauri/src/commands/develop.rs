use std::path::PathBuf;

use crate::dto::*;

#[tauri::command]
pub(crate) fn preview_exposure_contrast_edit(
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_white_balance_edit(
    library_path: String,
    photo_id: String,
    white_balance: String,
    temperature: f64,
    tint: f64,
) -> DesktopCommandResponse {
    let command = "preview_white_balance_edit";
    let white_balance_mode = match parse_white_balance(&white_balance) {
        Ok(mode) => mode,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    match silica_core::preview_white_balance_edit(
        PathBuf::from(&library_path),
        &photo_id,
        white_balance_mode,
        temperature,
        tint,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_tone_recovery_edit(
    library_path: String,
    photo_id: String,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> DesktopCommandResponse {
    let command = "preview_tone_recovery_edit";
    match silica_core::preview_tone_recovery_edit(
        PathBuf::from(&library_path),
        &photo_id,
        highlights,
        shadows,
        whites,
        blacks,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_color_presence_edit(
    library_path: String,
    photo_id: String,
    vibrance: f64,
    saturation: f64,
) -> DesktopCommandResponse {
    let command = "preview_color_presence_edit";
    match silica_core::preview_color_presence_edit(
        PathBuf::from(&library_path),
        &photo_id,
        vibrance,
        saturation,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_tone_curve_edit(
    library_path: String,
    photo_id: String,
    rgb_curve: Vec<DesktopToneCurvePoint>,
    red_curve: Vec<DesktopToneCurvePoint>,
    green_curve: Vec<DesktopToneCurvePoint>,
    blue_curve: Vec<DesktopToneCurvePoint>,
) -> DesktopCommandResponse {
    let command = "preview_tone_curve_edit";
    let rgb_curve = tone_curve_pairs(&rgb_curve);
    let red_curve = tone_curve_pairs(&red_curve);
    let green_curve = tone_curve_pairs(&green_curve);
    let blue_curve = tone_curve_pairs(&blue_curve);

    match silica_core::preview_tone_curve_edit(
        PathBuf::from(&library_path),
        &photo_id,
        &rgb_curve,
        &red_curve,
        &green_curve,
        &blue_curve,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_hsl_color_mixer_edit(
    library_path: String,
    photo_id: String,
    channel: String,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> DesktopCommandResponse {
    let command = "preview_hsl_color_mixer_edit";
    let hsl_channel = match parse_hsl_color_channel(&channel) {
        Ok(channel) => channel,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::preview_hsl_color_mixer_edit(
        PathBuf::from(&library_path),
        &photo_id,
        hsl_channel,
        hue,
        saturation,
        luminance,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_detail_sharpening_edit(
    library_path: String,
    photo_id: String,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> DesktopCommandResponse {
    let command = "preview_detail_sharpening_edit";
    match silica_core::preview_detail_sharpening_edit(
        PathBuf::from(&library_path),
        &photo_id,
        amount,
        radius,
        detail,
        masking,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn preview_detail_noise_reduction_edit(
    library_path: String,
    photo_id: String,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> DesktopCommandResponse {
    let command = "preview_detail_noise_reduction_edit";
    match silica_core::preview_detail_noise_reduction_edit(
        PathBuf::from(&library_path),
        &photo_id,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn preview_geometry_crop_edit(
    library_path: String,
    photo_id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<String>,
) -> DesktopCommandResponse {
    let command = "preview_geometry_crop_edit";
    match silica_core::preview_geometry_crop_edit(
        PathBuf::from(&library_path),
        &photo_id,
        x,
        y,
        width,
        height,
        angle,
        aspect.as_deref(),
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_clear_geometry_crop(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "preview_clear_geometry_crop";
    match silica_core::preview_clear_geometry_crop(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn preview_geometry_orientation_edit(
    library_path: String,
    photo_id: String,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> DesktopCommandResponse {
    let command = "preview_geometry_orientation_edit";
    match silica_core::preview_geometry_orientation_edit(
        PathBuf::from(&library_path),
        &photo_id,
        rotation,
        flip_horizontal,
        flip_vertical,
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
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
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
pub(crate) fn commit_exposure_contrast_edit(
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
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_white_balance_edit(
    library_path: String,
    photo_id: String,
    white_balance: String,
    temperature: f64,
    tint: f64,
) -> DesktopCommandResponse {
    let command = "commit_white_balance_edit";
    let white_balance_mode = match parse_white_balance(&white_balance) {
        Ok(mode) => mode,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    match silica_core::commit_white_balance_edit(
        PathBuf::from(&library_path),
        &photo_id,
        white_balance_mode,
        temperature,
        tint,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_tone_recovery_edit(
    library_path: String,
    photo_id: String,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> DesktopCommandResponse {
    let command = "commit_tone_recovery_edit";
    match silica_core::commit_tone_recovery_edit(
        PathBuf::from(&library_path),
        &photo_id,
        highlights,
        shadows,
        whites,
        blacks,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_color_presence_edit(
    library_path: String,
    photo_id: String,
    vibrance: f64,
    saturation: f64,
) -> DesktopCommandResponse {
    let command = "commit_color_presence_edit";
    match silica_core::commit_color_presence_edit(
        PathBuf::from(&library_path),
        &photo_id,
        vibrance,
        saturation,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_tone_curve_edit(
    library_path: String,
    photo_id: String,
    rgb_curve: Vec<DesktopToneCurvePoint>,
    red_curve: Vec<DesktopToneCurvePoint>,
    green_curve: Vec<DesktopToneCurvePoint>,
    blue_curve: Vec<DesktopToneCurvePoint>,
) -> DesktopCommandResponse {
    let command = "commit_tone_curve_edit";
    let rgb_curve = tone_curve_pairs(&rgb_curve);
    let red_curve = tone_curve_pairs(&red_curve);
    let green_curve = tone_curve_pairs(&green_curve);
    let blue_curve = tone_curve_pairs(&blue_curve);

    match silica_core::commit_tone_curve_edit(
        PathBuf::from(&library_path),
        &photo_id,
        &rgb_curve,
        &red_curve,
        &green_curve,
        &blue_curve,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_hsl_color_mixer_edit(
    library_path: String,
    photo_id: String,
    channel: String,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> DesktopCommandResponse {
    let command = "commit_hsl_color_mixer_edit";
    let hsl_channel = match parse_hsl_color_channel(&channel) {
        Ok(channel) => channel,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::commit_hsl_color_mixer_edit(
        PathBuf::from(&library_path),
        &photo_id,
        hsl_channel,
        hue,
        saturation,
        luminance,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_detail_sharpening_edit(
    library_path: String,
    photo_id: String,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> DesktopCommandResponse {
    let command = "commit_detail_sharpening_edit";
    match silica_core::commit_detail_sharpening_edit(
        PathBuf::from(&library_path),
        &photo_id,
        amount,
        radius,
        detail,
        masking,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn commit_detail_noise_reduction_edit(
    library_path: String,
    photo_id: String,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> DesktopCommandResponse {
    let command = "commit_detail_noise_reduction_edit";
    match silica_core::commit_detail_noise_reduction_edit(
        PathBuf::from(&library_path),
        &photo_id,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
#[allow(clippy::too_many_arguments)]
pub(crate) fn commit_geometry_crop_edit(
    library_path: String,
    photo_id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<String>,
) -> DesktopCommandResponse {
    let command = "commit_geometry_crop_edit";
    match silica_core::commit_geometry_crop_edit(
        PathBuf::from(&library_path),
        &photo_id,
        x,
        y,
        width,
        height,
        angle,
        aspect.as_deref(),
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_clear_geometry_crop(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "commit_clear_geometry_crop";
    match silica_core::commit_clear_geometry_crop(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_geometry_orientation_edit(
    library_path: String,
    photo_id: String,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> DesktopCommandResponse {
    let command = "commit_geometry_orientation_edit";
    match silica_core::commit_geometry_orientation_edit(
        PathBuf::from(&library_path),
        &photo_id,
        rotation,
        flip_horizontal,
        flip_vertical,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_p0_basic_reset(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "commit_p0_basic_reset";
    match silica_core::commit_p0_basic_reset(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn commit_basic_preset_edit(
    library_path: String,
    photo_id: String,
    preset: String,
) -> DesktopCommandResponse {
    let command = "commit_basic_preset_edit";
    let preset = match parse_basic_preset(&preset) {
        Ok(preset) => preset,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    match silica_core::commit_basic_preset_edit(PathBuf::from(&library_path), &photo_id, preset) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
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
pub(crate) fn get_photo_edit_state(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "get_photo_edit_state";
    match silica_core::get_photo_edit_state(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(state)) => DesktopCommandResponse::ok(
            command,
            state.message.clone(),
            DesktopCommandData::EditState {
                photo_id: state.photo_id,
                exposure: state.exposure,
                contrast: state.contrast,
                white_balance: white_balance_text(state.white_balance),
                temperature: state.temperature,
                tint: state.tint,
                highlights: state.highlights,
                shadows: state.shadows,
                whites: state.whites,
                blacks: state.blacks,
                vibrance: state.vibrance,
                saturation: state.saturation,
                tone_curve: tone_curve_data(state.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(state.hsl_color_mixer),
                detail: detail_data(state.detail),
                geometry: geometry_data(state.geometry),
                masks: manual_mask_data(state.masks),
                persisted: state.persisted,
                message: state.message,
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
pub(crate) fn copy_edit_clipboard_payload(
    library_path: String,
    photo_id: String,
    selection: silica_core::EditClipboardSelection,
) -> DesktopCommandResponse {
    let command = "copy_edit_clipboard_payload";
    match silica_core::copy_photo_edit_clipboard_payload(
        PathBuf::from(&library_path),
        &photo_id,
        selection,
    ) {
        Ok(Some(payload)) => DesktopCommandResponse::ok(
            command,
            "Copied selected edit sections.",
            edit_clipboard_data(photo_id, selection, payload),
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
pub(crate) fn plan_edit_clipboard_sync(
    library_path: String,
    photo_ids: Vec<String>,
    payload: silica_core::EditClipboardPayload,
) -> DesktopCommandResponse {
    let command = "plan_edit_clipboard_sync";
    match silica_core::plan_edit_clipboard_sync(PathBuf::from(&library_path), &photo_ids, &payload)
    {
        Ok(plan) => DesktopCommandResponse::ok(
            command,
            plan.message.clone(),
            edit_clipboard_plan_data(plan),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: photo_ids.first().cloned(),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
pub(crate) fn apply_edit_clipboard_sync(
    library_path: String,
    photo_ids: Vec<String>,
    payload: silica_core::EditClipboardPayload,
) -> DesktopCommandResponse {
    let command = "apply_edit_clipboard_sync";
    match silica_core::apply_edit_clipboard_sync(PathBuf::from(&library_path), &photo_ids, &payload)
    {
        Ok(result) => DesktopCommandResponse::ok(
            command,
            result.message.clone(),
            edit_clipboard_sync_data(result),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: photo_ids.first().cloned(),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
pub(crate) fn get_photo_histogram(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "get_photo_histogram";
    match silica_core::get_photo_histogram(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(histogram)) => DesktopCommandResponse::ok(
            command,
            histogram.message.clone(),
            DesktopCommandData::Histogram {
                photo_id: histogram.photo_id,
                source_path: histogram.source_path,
                status: histogram_status_text(histogram.status),
                red: histogram.red,
                green: histogram.green,
                blue: histogram.blue,
                luminance: histogram.luminance,
                pixel_count: histogram.pixel_count,
                cache_key: histogram.cache_key,
                cache_path: histogram.cache_path,
                message: histogram.message,
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
pub(crate) fn undo_last_history_action(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "undo_last_history_action";
    match silica_core::undo_last_history_action(PathBuf::from(&library_path), &photo_id) {
        Ok(result) => DesktopCommandResponse::ok(
            command,
            result.message.clone(),
            history_command_data(result),
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
pub(crate) fn redo_last_history_action(
    library_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    let command = "redo_last_history_action";
    match silica_core::redo_last_history_action(PathBuf::from(&library_path), &photo_id) {
        Ok(result) => DesktopCommandResponse::ok(
            command,
            result.message.clone(),
            history_command_data(result),
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
pub(crate) fn get_photo_history(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_history";
    match silica_core::list_photo_history(PathBuf::from(&library_path), &photo_id) {
        Ok(panel) => DesktopCommandResponse::ok(
            command,
            panel.message.clone(),
            photo_history_panel_data(panel),
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
