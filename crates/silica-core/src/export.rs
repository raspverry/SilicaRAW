use std::path::Path;
use std::path::PathBuf;

use crate::{
    append_core_action_log, detail_settings_json, ensure_no_active_manual_masks_for_export,
    ensure_supported_lens_geometry_export, export_color_presence_from_render,
    export_color_profile_string, export_color_profile_to_export, export_detail_from_render,
    export_format_string, export_geometry_from_render, export_hsl_color_mixer_from_render,
    export_manual_masks_from_render, export_metadata_policy_string,
    export_metadata_policy_to_export, export_profile_metadata_source,
    export_raster_format_to_export, export_raster_message, export_tone_curve_from_render,
    export_tone_recovery_from_render, export_white_balance_from_render, geometry_settings_json,
    hsl_color_mixer_settings_json, manual_mask_settings_json, preview_render_plan,
    record_brush_mask_raster_caches, render_color_presence_from_graph, render_detail_from_graph,
    render_geometry_from_graph, render_hsl_color_mixer_from_graph, render_manual_masks_from_graph,
    render_tone_curve_from_graph, render_tone_recovery_from_graph, render_white_balance_from_graph,
    tone_curve_settings_json, white_balance_render_mode_string, CoreError, ExportPreset,
    ExportSettings, ExportSettingsCatalog, LOCAL_ALPHA_JPEG_QUALITY,
};

/// Completed JPEG export returned through the core boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoExportSession {
    pub photo_id: String,
    pub source_path: String,
    pub output_path: PathBuf,
    pub format: String,
    pub color_profile: String,
    pub bytes_written: u64,
    pub source_sha256: Option<String>,
    pub output_sha256: String,
    pub icc_profile_embedded: bool,
    pub icc_profile_sha256: String,
    pub decoder_backend: Option<String>,
    pub input_profile: Option<String>,
    pub working_space: Option<String>,
    pub export_record_id: String,
    pub message: String,
}

/// Recent export record returned through the core boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoRecentExport {
    pub export_record_id: String,
    pub photo_id: String,
    pub output_path: String,
    pub export_settings_json: String,
    pub created_at: String,
    pub output_exists: bool,
}

/// JPEG output color profile accepted by the core export boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoExportColorProfile {
    Srgb,
    DisplayP3,
}

/// Source metadata policy accepted by the core export boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoExportMetadataPolicy {
    Minimal,
    Preserve,
    RemoveGps,
    RemoveAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PhotoExportFormat {
    Jpeg,
    Png,
    Tiff,
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

/// Read the library-wide export settings and named presets.
pub fn get_export_settings_catalog(
    library_root_path: impl AsRef<Path>,
) -> Result<ExportSettingsCatalog, CoreError> {
    silica_storage::get_export_settings_catalog(library_root_path).map_err(CoreError::from)
}

/// Create or update a named export preset.
pub fn upsert_export_preset(
    library_root_path: impl AsRef<Path>,
    name: impl AsRef<str>,
    settings: ExportSettings,
) -> Result<ExportPreset, CoreError> {
    silica_storage::upsert_export_preset(library_root_path, name, settings).map_err(CoreError::from)
}

/// Persist the current default export settings.
pub fn set_default_export_settings(
    library_root_path: impl AsRef<Path>,
    preset_id: Option<&str>,
    settings: ExportSettings,
) -> Result<ExportSettingsCatalog, CoreError> {
    silica_storage::set_default_export_settings(library_root_path, preset_id, settings)
        .map_err(CoreError::from)
}

/// Export one edited catalog photo as a JPEG sRGB file and record the export.
pub fn export_photo_jpeg_srgb(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_jpeg(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportColorProfile::Srgb,
    )
}

/// Export one edited catalog photo as a JPEG file with an explicit output color profile.
pub fn export_photo_jpeg(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    color_profile: PhotoExportColorProfile,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Jpeg,
        color_profile,
        PhotoExportMetadataPolicy::Minimal,
    )
}

/// Export one edited catalog photo as a JPEG file with explicit metadata policy.
pub fn export_photo_jpeg_with_metadata_policy(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    color_profile: PhotoExportColorProfile,
    metadata_policy: PhotoExportMetadataPolicy,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Jpeg,
        color_profile,
        metadata_policy,
    )
}

/// List recent export records with current output file evidence.
pub fn list_recent_exports(
    library_root_path: impl AsRef<Path>,
    limit: usize,
) -> Result<Vec<PhotoRecentExport>, CoreError> {
    let records = silica_storage::list_recent_export_records(library_root_path, limit)?;
    Ok(records
        .into_iter()
        .map(|record| {
            let output_exists = Path::new(&record.output_path).is_file();
            PhotoRecentExport {
                export_record_id: record.id,
                photo_id: record.photo_id,
                output_path: record.output_path,
                export_settings_json: record.export_settings_json,
                created_at: record.created_at,
                output_exists,
            }
        })
        .collect())
}

/// Export one edited catalog photo as a PNG sRGB file and record the export.
pub fn export_photo_png(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Png,
        PhotoExportColorProfile::Srgb,
        PhotoExportMetadataPolicy::Minimal,
    )
}

/// Export one edited catalog photo as a TIFF sRGB file and record the export.
pub fn export_photo_tiff(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    export_photo_raster(
        library_root_path,
        photo_id,
        output_path,
        PhotoExportFormat::Tiff,
        PhotoExportColorProfile::Srgb,
        PhotoExportMetadataPolicy::Minimal,
    )
}

fn export_photo_raster(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    format: PhotoExportFormat,
    color_profile: PhotoExportColorProfile,
    metadata_policy: PhotoExportMetadataPolicy,
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
    let render_masks = render_manual_masks_from_graph(&graph)?;
    let exposure = graph.basic.exposure.as_f64().unwrap_or(0.0);
    let contrast = graph.basic.contrast.as_f64().unwrap_or(0.0);
    let render_request = silica_render::plan_jpeg_srgb_export_with_color_presence(
        render_plan.source_path.clone(),
        output_path.display().to_string(),
        exposure,
        contrast,
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        LOCAL_ALPHA_JPEG_QUALITY,
    );
    let render_request = silica_render::plan_jpeg_srgb_export_with_tone_curve(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_tone_curve_from_graph(&graph),
        render_request.quality,
    );
    let render_request = silica_render::plan_jpeg_srgb_export_with_geometry(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_request.tone_curve,
        render_hsl_color_mixer_from_graph(&graph),
        render_detail_from_graph(&graph),
        render_geometry_from_graph(&graph),
        render_request.quality,
    );
    if !render_request.detail.is_neutral() {
        return Err(CoreError::ExportBlocked(render_request.message));
    }
    ensure_supported_lens_geometry_export(&graph, &render_request.geometry)?;
    record_brush_mask_raster_caches(library_root_path, &photo_id, &render_masks)?;
    let export_masks = export_manual_masks_from_render(&render_masks);

    let source_path = PathBuf::from(&render_request.source_path);
    let export_white_balance = export_white_balance_from_render(render_request.white_balance);
    let export_tone_recovery = export_tone_recovery_from_render(render_request.tone_recovery);
    let export_color_presence = export_color_presence_from_render(render_request.color_presence);
    let export_tone_curve = export_tone_curve_from_render(render_request.tone_curve.clone());
    let export_hsl_color_mixer = export_hsl_color_mixer_from_render(render_request.hsl_color_mixer);
    let export_detail = export_detail_from_render(render_request.detail);
    let export_geometry = export_geometry_from_render(render_request.geometry.clone());

    let (
        exported_output_path,
        exported_format,
        exported_color_profile,
        bytes_written,
        source_sha256,
        output_sha256,
        icc_profile_embedded,
        icc_profile_sha256,
        source_metadata_segments,
        output_metadata_segments,
        source_metadata_copied,
        gps_metadata_removed,
    ) = match format {
        PhotoExportFormat::Jpeg => {
            let export_result = silica_export::export_jpeg_with_metadata_policy(
                silica_export::JpegColorExportRequest {
                    source_path,
                    output_path: output_path.to_path_buf(),
                    exposure: render_request.exposure,
                    contrast: render_request.contrast,
                    white_balance: export_white_balance,
                    tone_recovery: export_tone_recovery,
                    color_presence: export_color_presence,
                    tone_curve: export_tone_curve,
                    hsl_color_mixer: export_hsl_color_mixer,
                    detail: export_detail,
                    geometry: export_geometry,
                    masks: export_masks,
                    quality: render_request.quality,
                    color_profile: export_color_profile_to_export(color_profile),
                },
                export_metadata_policy_to_export(metadata_policy),
            )?;
            (
                export_result.output_path,
                export_format_string(export_result.format).to_string(),
                export_color_profile_string(export_result.color_profile).to_string(),
                export_result.bytes_written,
                export_result.source_sha256,
                export_result.output_sha256,
                export_result.icc_profile_embedded,
                Some(export_result.icc_profile_sha256),
                export_result.source_metadata_segments,
                export_result.output_metadata_segments,
                export_result.source_metadata_copied,
                export_result.gps_metadata_removed,
            )
        }
        PhotoExportFormat::Png | PhotoExportFormat::Tiff => {
            let export_result =
                silica_export::export_raster_srgb(silica_export::RasterSrgbExportRequest {
                    source_path,
                    output_path: output_path.to_path_buf(),
                    format: export_raster_format_to_export(format),
                    exposure: render_request.exposure,
                    contrast: render_request.contrast,
                    white_balance: export_white_balance,
                    tone_recovery: export_tone_recovery,
                    color_presence: export_color_presence,
                    tone_curve: export_tone_curve,
                    hsl_color_mixer: export_hsl_color_mixer,
                    detail: export_detail,
                    geometry: export_geometry,
                    masks: export_masks,
                })?;
            (
                export_result.output_path,
                export_format_string(export_result.format).to_string(),
                export_color_profile_string(export_result.color_profile).to_string(),
                export_result.bytes_written,
                export_result.source_sha256,
                export_result.output_sha256,
                export_result.icc_profile_embedded,
                export_result.icc_profile_sha256,
                0,
                0,
                false,
                false,
            )
        }
    };
    let icc_profile_sha256_value = icc_profile_sha256
        .as_deref()
        .map_or(serde_json::Value::Null, serde_json::Value::from);
    let source_sha256_after_export =
        silica_export::sha256_file(Path::new(&render_request.source_path)).ok();
    let source_original_hash_unchanged =
        source_sha256_after_export.as_deref() == Some(source_sha256.as_str());
    let decoder_backend = "raster".to_string();
    let input_profile = "assume_srgb".to_string();
    let working_space = "srgb".to_string();
    let settings_value = serde_json::json!({
        "format": exported_format.clone(),
        "color_profile": exported_color_profile.clone(),
        "quality": render_request.quality,
        "metadata_policy": export_metadata_policy_string(metadata_policy),
        "exposure": render_request.exposure,
        "contrast": render_request.contrast,
        "white_balance": white_balance_render_mode_string(render_request.white_balance.mode),
        "temperature": render_request.white_balance.temperature,
        "tint": render_request.white_balance.tint,
        "highlights": render_request.tone_recovery.highlights,
        "shadows": render_request.tone_recovery.shadows,
        "whites": render_request.tone_recovery.whites,
        "blacks": render_request.tone_recovery.blacks,
        "vibrance": render_request.color_presence.vibrance,
        "saturation": render_request.color_presence.saturation,
        "tone_curve": tone_curve_settings_json(&render_request.tone_curve),
        "hsl_color_mixer": hsl_color_mixer_settings_json(&render_request.hsl_color_mixer),
        "detail": detail_settings_json(&render_request.detail),
        "geometry": geometry_settings_json(&render_request.geometry),
        "masks": manual_mask_settings_json(&render_masks),
        "source_path": render_request.source_path,
        "output_path": render_request.output_path,
        "source_sha256": source_sha256.clone(),
        "source_sha256_after_export": source_sha256_after_export.clone(),
        "source_original_hash_unchanged": source_original_hash_unchanged,
        "output_sha256": output_sha256.clone(),
        "icc_profile_embedded": icc_profile_embedded,
        "icc_profile_sha256": icc_profile_sha256_value,
        "decoder_backend": decoder_backend.clone(),
        "input_profile": input_profile.clone(),
        "working_space": working_space.clone(),
        "profile_metadata_source": export_profile_metadata_source(format),
        "source_metadata_segments": source_metadata_segments,
        "output_metadata_segments": output_metadata_segments,
        "source_metadata_copied": source_metadata_copied,
        "gps_metadata_removed": gps_metadata_removed,
    });
    let settings_json = settings_value.to_string();
    let export_record = silica_storage::record_export(
        library_root_path,
        &photo_id,
        &exported_output_path,
        settings_json,
    )?;
    let export_record_id = export_record.id;
    append_core_action_log(
        library_root_path,
        "export",
        Some("photo"),
        Some(photo_id.clone()),
        "file_write",
        Some(export_record_id.clone()),
        settings_value,
    )?;

    Ok(Some(PhotoExportSession {
        photo_id,
        source_path: render_plan.source_path,
        output_path: exported_output_path,
        format: exported_format,
        color_profile: exported_color_profile,
        bytes_written,
        source_sha256: Some(source_sha256),
        output_sha256,
        icc_profile_embedded,
        icc_profile_sha256: icc_profile_sha256.unwrap_or_default(),
        decoder_backend: Some(decoder_backend),
        input_profile: Some(input_profile),
        working_space: Some(working_space),
        export_record_id,
        message: export_raster_message(format, color_profile).to_string(),
    }))
}

/// Export one fixture-backed RAW catalog photo as JPEG sRGB through a full-resolution source artifact.
pub fn export_raw_photo_jpeg_srgb_from_probe(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
    output_path: impl AsRef<Path>,
) -> Result<Option<PhotoExportSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let output_path = output_path.as_ref();
    let candidate = match silica_storage::get_photo_preview_candidate(library_root_path, photo_id)?
    {
        Some(candidate) => candidate,
        None => return Ok(None),
    };
    let raw_source_path = PathBuf::from(&probe.source_path);
    if paths_match(&raw_source_path, output_path)? {
        return Err(CoreError::RawExport(
            silica_decode::RawFullResolutionExportSourceError::OutputMatchesSource(
                output_path.to_path_buf(),
            ),
        ));
    }
    if !paths_match(&PathBuf::from(&candidate.path), &raw_source_path)? {
        return Err(CoreError::ExportBlocked(
            "RAW export probe source does not match the catalog photo source.".to_string(),
        ));
    }

    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    ensure_no_active_manual_masks_for_export(&graph)?;
    let exposure = graph.basic.exposure.as_f64().unwrap_or(0.0);
    let contrast = graph.basic.contrast.as_f64().unwrap_or(0.0);
    let source_artifact_path =
        raw_full_resolution_export_source_path(library_root_path, photo_id, probe);
    let source_artifact = silica_decode::write_raw_full_resolution_export_source(
        silica_decode::RawFullResolutionExportSourceRequest {
            fixture_class: fixture_class.as_ref().to_string(),
            probe: probe.clone(),
            output_path: source_artifact_path,
        },
    )?;
    let render_request = silica_render::plan_raw_derived_jpeg_srgb_export_with_color_presence(
        source_artifact.artifact_path.display().to_string(),
        output_path.display().to_string(),
        exposure,
        contrast,
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        LOCAL_ALPHA_JPEG_QUALITY,
    );
    let render_request = silica_render::plan_raw_derived_jpeg_srgb_export_with_tone_curve(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_tone_curve_from_graph(&graph),
        render_request.quality,
    );
    let mut render_request = silica_render::plan_raw_derived_jpeg_srgb_export_with_hsl_color_mixer(
        render_request.source_path,
        render_request.output_path,
        render_request.exposure,
        render_request.contrast,
        render_request.white_balance,
        render_request.tone_recovery,
        render_request.color_presence,
        render_request.tone_curve,
        render_hsl_color_mixer_from_graph(&graph),
        render_request.quality,
    );
    render_request.detail = render_detail_from_graph(&graph);
    render_request.geometry = render_geometry_from_graph(&graph);
    if !render_request.detail.is_neutral() {
        render_request.message =
            "Detail export unsupported until renderer support exists.".to_string();
    }
    if !render_request.detail.is_neutral() {
        return Err(CoreError::ExportBlocked(render_request.message));
    }
    ensure_supported_lens_geometry_export(&graph, &render_request.geometry)?;
    let export_result =
        silica_export::export_jpeg_with_color_profile(silica_export::JpegColorExportRequest {
            source_path: PathBuf::from(&render_request.source_path),
            output_path: output_path.to_path_buf(),
            exposure: render_request.exposure,
            contrast: render_request.contrast,
            white_balance: export_white_balance_from_render(render_request.white_balance),
            tone_recovery: export_tone_recovery_from_render(render_request.tone_recovery),
            color_presence: export_color_presence_from_render(render_request.color_presence),
            tone_curve: export_tone_curve_from_render(render_request.tone_curve.clone()),
            hsl_color_mixer: export_hsl_color_mixer_from_render(render_request.hsl_color_mixer),
            detail: export_detail_from_render(render_request.detail),
            geometry: export_geometry_from_render(render_request.geometry.clone()),
            masks: Vec::new(),
            quality: render_request.quality,
            color_profile: silica_export::ExportColorProfile::Srgb,
        })?;
    let format = export_format_string(export_result.format).to_string();
    let exported_color_profile =
        export_color_profile_string(export_result.color_profile).to_string();
    let output_sha256 = export_result.output_sha256.clone();
    let icc_profile_sha256 = export_result.icc_profile_sha256.clone();
    let decoder_backend = source_artifact.decoder_backend.as_str().to_string();
    let input_profile = source_artifact.input_profile.clone();
    let working_space = source_artifact.working_space.clone();
    let settings_value = serde_json::json!({
        "format": format,
        "color_profile": exported_color_profile,
        "quality": render_request.quality,
        "exposure": render_request.exposure,
        "contrast": render_request.contrast,
        "white_balance": white_balance_render_mode_string(render_request.white_balance.mode),
        "temperature": render_request.white_balance.temperature,
        "tint": render_request.white_balance.tint,
        "highlights": render_request.tone_recovery.highlights,
        "shadows": render_request.tone_recovery.shadows,
        "whites": render_request.tone_recovery.whites,
        "blacks": render_request.tone_recovery.blacks,
        "vibrance": render_request.color_presence.vibrance,
        "saturation": render_request.color_presence.saturation,
        "tone_curve": tone_curve_settings_json(&render_request.tone_curve),
        "hsl_color_mixer": hsl_color_mixer_settings_json(&render_request.hsl_color_mixer),
        "detail": detail_settings_json(&render_request.detail),
        "geometry": geometry_settings_json(&render_request.geometry),
        "source_path": source_artifact.source_path.clone(),
        "source_sha256": source_artifact.source_sha256.clone(),
        "raw_source_path": source_artifact.source_path.clone(),
        "raw_source_sha256": source_artifact.source_sha256.clone(),
        "raw_export_source_artifact_path": source_artifact.artifact_path.display().to_string(),
        "raw_export_source_artifact_sha256": source_artifact.artifact_sha256.clone(),
        "raw_export_source_artifact_bytes": source_artifact.bytes_written,
        "raw_source_original_hash_unchanged": source_artifact.original_hash_unchanged,
        "output_path": render_request.output_path,
        "output_sha256": output_sha256.clone(),
        "icc_profile_embedded": export_result.icc_profile_embedded,
        "icc_profile_sha256": icc_profile_sha256.clone(),
        "profile_metadata_source": "silica-export",
        "decoder_backend": decoder_backend.clone(),
        "input_profile": input_profile.clone(),
        "working_space": working_space.clone(),
        "export_source_kind": "raw_full_resolution_artifact",
        "viewer_texture_cache_source": render_request.uses_viewer_texture_cache_as_source(),
    });
    let settings_json = settings_value.to_string();
    let export_record = silica_storage::record_export(
        library_root_path,
        &candidate.photo_id,
        &export_result.output_path,
        settings_json,
    )?;
    let export_record_id = export_record.id;
    append_core_action_log(
        library_root_path,
        "export",
        Some("photo"),
        Some(candidate.photo_id.clone()),
        "file_write",
        Some(export_record_id.clone()),
        settings_value,
    )?;

    Ok(Some(PhotoExportSession {
        photo_id: candidate.photo_id,
        source_path: source_artifact.source_path,
        output_path: export_result.output_path,
        format,
        color_profile: exported_color_profile,
        bytes_written: export_result.bytes_written,
        source_sha256: Some(source_artifact.source_sha256),
        output_sha256,
        icc_profile_embedded: export_result.icc_profile_embedded,
        icc_profile_sha256,
        decoder_backend: Some(decoder_backend),
        input_profile: Some(input_profile),
        working_space: Some(working_space),
        export_record_id,
        message: "RAW-derived JPEG sRGB export completed.".to_string(),
    }))
}

fn raw_full_resolution_export_source_path(
    library_root_path: &Path,
    photo_id: &str,
    probe: &silica_decode::RawProbeResult,
) -> PathBuf {
    let source_sha = probe.source_sha256.as_deref().unwrap_or("missing-sha256");
    library_root_path
        .join("render-cache")
        .join("raw-export-sources")
        .join(format!("raw-export-{photo_id}-{source_sha}.jpg"))
}

fn paths_match(source_path: &PathBuf, output_path: &Path) -> Result<bool, CoreError> {
    if source_path == output_path {
        return Ok(true);
    }
    if !output_path.exists() {
        return Ok(false);
    }

    let source_path = match std::fs::canonicalize(source_path) {
        Ok(path) => path,
        Err(error) => {
            return Err(CoreError::Storage(
                silica_storage::LibraryStorageError::from(error),
            ))
        }
    };
    let output_path = match std::fs::canonicalize(output_path) {
        Ok(path) => path,
        Err(error) => {
            return Err(CoreError::Storage(
                silica_storage::LibraryStorageError::from(error),
            ))
        }
    };

    if source_path == output_path {
        return Ok(true);
    }

    paths_share_file_identity(&source_path, &output_path)
}

#[cfg(unix)]
fn paths_share_file_identity(source_path: &Path, output_path: &Path) -> Result<bool, CoreError> {
    use std::os::unix::fs::MetadataExt;

    let source = std::fs::metadata(source_path)
        .map_err(|error| CoreError::Storage(silica_storage::LibraryStorageError::from(error)))?;
    let output = std::fs::metadata(output_path)
        .map_err(|error| CoreError::Storage(silica_storage::LibraryStorageError::from(error)))?;
    Ok(source.dev() == output.dev() && source.ino() == output.ino())
}

#[cfg(not(unix))]
fn paths_share_file_identity(_source_path: &Path, _output_path: &Path) -> Result<bool, CoreError> {
    Ok(false)
}
