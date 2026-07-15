use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopExportSettings {
    pub(crate) format: String,
    pub(crate) color_profile: String,
    pub(crate) quality: u8,
    pub(crate) metadata_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopExportPreset {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) settings: DesktopExportSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopRecentExport {
    pub(crate) export_record_id: String,
    pub(crate) photo_id: String,
    pub(crate) output_path: String,
    pub(crate) export_settings_json: String,
    pub(crate) created_at: String,
    pub(crate) output_exists: bool,
}

pub(crate) fn parse_export_color_profile(
    color_profile: Option<&str>,
) -> Result<silica_core::PhotoExportColorProfile, silica_core::CoreError> {
    match color_profile.unwrap_or("srgb") {
        "srgb" => Ok(silica_core::PhotoExportColorProfile::Srgb),
        "display_p3" => Ok(silica_core::PhotoExportColorProfile::DisplayP3),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported export color profile: {unsupported}. Supported profiles: srgb, display_p3."
        ))),
    }
}

pub(crate) fn parse_export_format(
    format: Option<&str>,
) -> Result<&'static str, silica_core::CoreError> {
    match format.unwrap_or("jpeg") {
        "jpeg" => Ok("jpeg"),
        "png" => Ok("png"),
        "tiff" => Ok("tiff"),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported export format: {unsupported}. Supported formats: jpeg, png, tiff."
        ))),
    }
}

pub(crate) fn parse_export_metadata_policy(
    metadata_policy: Option<&str>,
) -> Result<silica_core::PhotoExportMetadataPolicy, silica_core::CoreError> {
    match metadata_policy.unwrap_or("minimal") {
        "minimal" => Ok(silica_core::PhotoExportMetadataPolicy::Minimal),
        "preserve" => Ok(silica_core::PhotoExportMetadataPolicy::Preserve),
        "remove_gps" => Ok(silica_core::PhotoExportMetadataPolicy::RemoveGps),
        "remove_all" => Ok(silica_core::PhotoExportMetadataPolicy::RemoveAll),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported export metadata policy: {unsupported}. Supported policies: minimal, preserve, remove_gps, remove_all."
        ))),
    }
}

pub(crate) fn export_metadata_policy_request_string(
    metadata_policy: silica_core::PhotoExportMetadataPolicy,
) -> &'static str {
    match metadata_policy {
        silica_core::PhotoExportMetadataPolicy::Minimal => "minimal",
        silica_core::PhotoExportMetadataPolicy::Preserve => "preserve",
        silica_core::PhotoExportMetadataPolicy::RemoveGps => "remove_gps",
        silica_core::PhotoExportMetadataPolicy::RemoveAll => "remove_all",
    }
}

pub(crate) fn export_settings_from_request(
    format: Option<&str>,
    color_profile: Option<&str>,
    quality: Option<u8>,
    metadata_policy: Option<&str>,
) -> Result<silica_core::ExportSettings, silica_core::CoreError> {
    let format = parse_export_format(format)?;
    let color_profile = parse_export_color_profile(color_profile)?;
    let metadata_policy = parse_export_metadata_policy(metadata_policy)?;
    if format != "jpeg" && color_profile != silica_core::PhotoExportColorProfile::Srgb {
        return Err(silica_core::CoreError::ExportBlocked(
            "PNG and TIFF export settings currently require sRGB color profile.".to_string(),
        ));
    }
    Ok(silica_core::ExportSettings {
        format: format.to_string(),
        color_profile: export_color_profile_request_string(color_profile).to_string(),
        quality: quality.unwrap_or(90),
        metadata_policy: export_metadata_policy_request_string(metadata_policy).to_string(),
    })
}

pub(crate) fn export_color_profile_request_string(
    color_profile: silica_core::PhotoExportColorProfile,
) -> &'static str {
    match color_profile {
        silica_core::PhotoExportColorProfile::Srgb => "srgb",
        silica_core::PhotoExportColorProfile::DisplayP3 => "display_p3",
    }
}

pub(crate) fn desktop_export_preset(preset: silica_core::ExportPreset) -> DesktopExportPreset {
    DesktopExportPreset {
        id: preset.id,
        name: preset.name,
        settings: desktop_export_settings(preset.settings),
    }
}

pub(crate) fn desktop_export_settings(
    settings: silica_core::ExportSettings,
) -> DesktopExportSettings {
    DesktopExportSettings {
        format: settings.format,
        color_profile: settings.color_profile,
        quality: settings.quality,
        metadata_policy: settings.metadata_policy,
    }
}

pub(crate) fn desktop_recent_export(export: silica_core::PhotoRecentExport) -> DesktopRecentExport {
    DesktopRecentExport {
        export_record_id: export.export_record_id,
        photo_id: export.photo_id,
        output_path: export.output_path,
        export_settings_json: export.export_settings_json,
        created_at: export.created_at,
        output_exists: export.output_exists,
    }
}
