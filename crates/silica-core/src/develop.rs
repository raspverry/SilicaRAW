use super::*;

#[allow(clippy::too_many_arguments)]
pub fn plan_exposure_contrast_metal_draft(
    photo_id: impl Into<String>,
    source_path: impl Into<String>,
    viewer_input: silica_render::ViewerPreviewInput,
    viewport: silica_render::ViewerPreviewViewport,
    request_id: silica_render::ViewerPreviewRenderRequestId,
    edit_graph_revision: u64,
    exposure: f64,
    contrast: f64,
) -> Result<silica_render::ViewerPreviewRenderRequest, CoreError> {
    let photo_id = photo_id.into();
    let source_path = source_path.into();
    let graph = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: photo_id.clone(),
            path: source_path.clone(),
            file_size: 0,
            modified_at: None,
            partial_hash: None,
            full_hash: None,
        },
        current_timestamp_string(),
    );
    let edited = silica_edit::apply_exposure_contrast(
        &graph,
        exposure,
        contrast,
        current_timestamp_string(),
    )?;
    let exposure = edited.basic.exposure.as_f64().unwrap_or(exposure);
    let contrast = edited.basic.contrast.as_f64().unwrap_or(contrast);

    Ok(silica_render::ViewerPreviewRenderRequest::new(
        request_id,
        photo_id,
        source_path,
        viewport,
        viewer_input,
        edit_graph_revision,
    )
    .with_exposure_contrast_draft(exposure, contrast))
}
/// Normalized tone curve point exposed by the core command boundary.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoToneCurvePoint {
    pub x: f64,
    pub y: f64,
}

/// Current tone curve state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoToneCurveState {
    pub curve_mode: silica_edit::CurveMode,
    pub rgb_curve: Vec<PhotoToneCurvePoint>,
    pub red_curve: Vec<PhotoToneCurvePoint>,
    pub green_curve: Vec<PhotoToneCurvePoint>,
    pub blue_curve: Vec<PhotoToneCurvePoint>,
}

/// One HSL color mixer channel exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoHslColorChannelState {
    pub hue: f64,
    pub saturation: f64,
    pub luminance: f64,
}

/// Current HSL color mixer state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoHslColorMixerState {
    pub red: PhotoHslColorChannelState,
    pub orange: PhotoHslColorChannelState,
    pub yellow: PhotoHslColorChannelState,
    pub green: PhotoHslColorChannelState,
    pub aqua: PhotoHslColorChannelState,
    pub blue: PhotoHslColorChannelState,
    pub purple: PhotoHslColorChannelState,
    pub magenta: PhotoHslColorChannelState,
}

/// Sharpening state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoDetailSharpeningState {
    pub amount: f64,
    pub radius: f64,
    pub detail: f64,
    pub masking: f64,
}

/// Non-MLX noise reduction state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoDetailNoiseReductionState {
    pub luminance: f64,
    pub detail: f64,
    pub contrast: f64,
    pub color: f64,
    pub color_detail: f64,
}

/// Current Detail state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoDetailState {
    pub sharpening: PhotoDetailSharpeningState,
    pub noise_reduction: PhotoDetailNoiseReductionState,
}

/// Normalized crop state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoGeometryCropState {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    pub aspect: Option<String>,
}

/// Perspective/scale transform state exposed for explicit unsupported-state reporting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoGeometryTransformState {
    pub vertical: f64,
    pub horizontal: f64,
    pub aspect: f64,
    pub scale: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

/// Current Geometry state exposed by preview, commit, and edit-state responses.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoGeometryState {
    pub crop: Option<PhotoGeometryCropState>,
    pub rotation: f64,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub transform: PhotoGeometryTransformState,
}

/// Manual mask geometry exposed to local app surfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhotoManualMaskGeometryState {
    LinearGradient {
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    },
    RadialGradient {
        center_x: f64,
        center_y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
    },
}

/// Manual mask state exposed to local app surfaces.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoManualMaskState {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub enabled: bool,
    pub invert: bool,
    pub opacity: f64,
    pub feather: f64,
    pub geometry: Option<PhotoManualMaskGeometryState>,
    pub exposure: f64,
    pub contrast: f64,
}

/// Core input point for a sampled manual brush stroke.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoManualBrushPointInput {
    pub x: f64,
    pub y: f64,
}

/// Core input stroke for a sampled manual brush mask.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoManualBrushStrokeInput {
    pub id: String,
    pub radius: f64,
    pub points: Vec<PhotoManualBrushPointInput>,
}

/// Draft preview request returned while an exposure/contrast slider is moving.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoEditPreviewSession {
    pub photo_id: String,
    pub source_path: String,
    pub develop_preview_bytes: Option<Vec<u8>>,
    pub status: PhotoPreviewStatus,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: silica_edit::WhiteBalance,
    pub temperature: f64,
    pub tint: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
    pub vibrance: f64,
    pub saturation: f64,
    pub tone_curve: PhotoToneCurveState,
    pub hsl_color_mixer: PhotoHslColorMixerState,
    pub detail: PhotoDetailState,
    pub geometry: PhotoGeometryState,
    pub masks: Vec<PhotoManualMaskState>,
    pub message: String,
}

impl PhotoEditPreviewSession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nPreview: {:?}\nSource: {}\nExposure: {}\nContrast: {}\nWhite Balance: {:?}\nTemperature: {}\nTint: {}\nHighlights: {}\nShadows: {}\nWhites: {}\nBlacks: {}\nVibrance: {}\nSaturation: {}\nMessage: {}",
            self.photo_id,
            self.status,
            self.source_path,
            self.exposure,
            self.contrast,
            self.white_balance,
            self.temperature,
            self.tint,
            self.highlights,
            self.shadows,
            self.whites,
            self.blacks,
            self.vibrance,
            self.saturation,
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
    pub white_balance: silica_edit::WhiteBalance,
    pub temperature: f64,
    pub tint: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
    pub vibrance: f64,
    pub saturation: f64,
    pub tone_curve: PhotoToneCurveState,
    pub hsl_color_mixer: PhotoHslColorMixerState,
    pub detail: PhotoDetailState,
    pub geometry: PhotoGeometryState,
    pub masks: Vec<PhotoManualMaskState>,
    pub persisted: bool,
    pub message: String,
}

impl PhotoEditCommit {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nExposure: {}\nContrast: {}\nWhite Balance: {:?}\nTemperature: {}\nTint: {}\nHighlights: {}\nShadows: {}\nWhites: {}\nBlacks: {}\nVibrance: {}\nSaturation: {}\nPersisted: {}\nMessage: {}",
            self.photo_id,
            self.exposure,
            self.contrast,
            self.white_balance,
            self.temperature,
            self.tint,
            self.highlights,
            self.shadows,
            self.whites,
            self.blacks,
            self.vibrance,
            self.saturation,
            self.persisted,
            self.message
        )
    }
}
/// Current committed exposure/contrast edit state for a catalog photo.
#[derive(Debug, Clone, PartialEq)]
pub struct PhotoEditState {
    pub photo_id: String,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: silica_edit::WhiteBalance,
    pub temperature: f64,
    pub tint: f64,
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
    pub vibrance: f64,
    pub saturation: f64,
    pub tone_curve: PhotoToneCurveState,
    pub hsl_color_mixer: PhotoHslColorMixerState,
    pub detail: PhotoDetailState,
    pub geometry: PhotoGeometryState,
    pub masks: Vec<PhotoManualMaskState>,
    pub persisted: bool,
    pub message: String,
}

impl PhotoEditState {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Photo: {}\nExposure: {}\nContrast: {}\nWhite Balance: {:?}\nTemperature: {}\nTint: {}\nHighlights: {}\nShadows: {}\nWhites: {}\nBlacks: {}\nVibrance: {}\nSaturation: {}\nPersisted: {}\nMessage: {}",
            self.photo_id,
            self.exposure,
            self.contrast,
            self.white_balance,
            self.temperature,
            self.tint,
            self.highlights,
            self.shadows,
            self.whites,
            self.blacks,
            self.vibrance,
            self.saturation,
            self.persisted,
            self.message
        )
    }
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
    let mut request = silica_render::plan_exposure_contrast_preview(
        render_plan,
        edited.basic.exposure.as_f64().unwrap_or(exposure),
        edited.basic.contrast.as_f64().unwrap_or(contrast),
    );
    request.white_balance = render_white_balance_from_graph(&graph);
    request.tone_recovery = render_tone_recovery_from_graph(&graph);
    request.color_presence = render_color_presence_from_graph(&graph);
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft white-balance preview request without writing the catalog.
pub fn preview_white_balance_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    white_balance: silica_edit::WhiteBalance,
    temperature: f64,
    tint: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_white_balance_temperature_tint(
        &graph,
        white_balance,
        temperature,
        tint,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let exposure = graph.basic.exposure.as_f64().unwrap_or(0.0);
    let contrast = graph.basic.contrast.as_f64().unwrap_or(0.0);
    let request = silica_render::plan_white_balance_preview(
        render_plan,
        exposure,
        contrast,
        render_white_balance_from_graph(&edited),
    );
    let mut request = request;
    request.tone_recovery = render_tone_recovery_from_graph(&graph);
    request.color_presence = render_color_presence_from_graph(&graph);
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: edited.basic.white_balance,
        temperature: edited.basic.temperature.as_f64().unwrap_or(temperature),
        tint: edited.basic.tint.as_f64().unwrap_or(tint),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft tone-recovery preview request without writing the catalog.
pub fn preview_tone_recovery_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_recovery(
        &graph,
        highlights,
        shadows,
        whites,
        blacks,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_tone_recovery_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&edited),
    );
    let mut request = request;
    request.color_presence = render_color_presence_from_graph(&graph);
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: edited.basic.highlights.as_f64().unwrap_or(highlights),
        shadows: edited.basic.shadows.as_f64().unwrap_or(shadows),
        whites: edited.basic.whites.as_f64().unwrap_or(whites),
        blacks: edited.basic.blacks.as_f64().unwrap_or(blacks),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft tone-curve preview request without writing the catalog.
pub fn preview_tone_curve_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rgb_curve: &[(f64, f64)],
    red_curve: &[(f64, f64)],
    green_curve: &[(f64, f64)],
    blue_curve: &[(f64, f64)],
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_curve(
        &graph,
        silica_edit::CurveMode::Point,
        rgb_curve,
        red_curve,
        green_curve,
        blue_curve,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let mut request = silica_render::plan_tone_curve_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        render_tone_curve_from_graph(&edited),
    );
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &edited)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&edited),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&edited),
        detail: detail_state_from_graph(&edited),
        geometry: geometry_state_from_graph(&edited),
        masks: photo_manual_masks_from_graph(&edited),
        message,
    }))
}

/// Build a draft HSL color mixer preview request without writing the catalog.
pub fn preview_hsl_color_mixer_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    channel: silica_edit::HslColorChannel,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_hsl_color_channel(
        &graph,
        channel,
        hue,
        saturation,
        luminance,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_hsl_color_mixer_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&graph),
        render_tone_curve_from_graph(&graph),
        render_hsl_color_mixer_from_graph(&edited),
    );
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &edited)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&edited),
        detail: detail_state_from_graph(&edited),
        geometry: geometry_state_from_graph(&edited),
        masks: photo_manual_masks_from_graph(&edited),
        message,
    }))
}

/// Build a draft color-presence preview request without writing the catalog.
pub fn preview_color_presence_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    vibrance: f64,
    saturation: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_color_presence(
        &graph,
        vibrance,
        saturation,
        current_timestamp_string(),
    )?;
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let mut request = silica_render::plan_color_presence_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(&graph),
        render_tone_recovery_from_graph(&graph),
        render_color_presence_from_graph(&edited),
    );
    request.tone_curve = render_tone_curve_from_graph(&graph);
    request.hsl_color_mixer = render_hsl_color_mixer_from_graph(&graph);
    let request = apply_detail_preview_boundary(request, render_detail_from_graph(&graph));
    let request = apply_lens_geometry_preview_boundary(request, &graph);
    let request = apply_manual_mask_preview_boundary(request, &graph)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: edited.basic.vibrance.as_f64().unwrap_or(vibrance),
        saturation: edited.basic.saturation.as_f64().unwrap_or(saturation),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        message,
    }))
}

/// Build a draft sharpening preview without writing the catalog.
pub fn preview_detail_sharpening_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_detail_sharpening(
        &graph,
        amount,
        radius,
        detail,
        masking,
        current_timestamp_string(),
    )?;

    preview_detail_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft noise-reduction preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_detail_noise_reduction_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_detail_noise_reduction(
        &graph,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
        current_timestamp_string(),
    )?;

    preview_detail_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft rectangular crop preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_geometry_crop_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<&str>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_crop(
        &graph,
        x,
        y,
        width,
        height,
        angle,
        aspect,
        current_timestamp_string(),
    )?;

    preview_geometry_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft crop-clear preview without writing the catalog.
pub fn preview_clear_geometry_crop(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::clear_geometry_crop(&graph, current_timestamp_string())?;

    preview_geometry_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft rotation/flip preview without writing the catalog.
pub fn preview_geometry_orientation_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_orientation(
        &graph,
        rotation,
        flip_horizontal,
        flip_vertical,
        current_timestamp_string(),
    )?;

    preview_geometry_edit(library_root_path, photo_id, &graph, &edited)
}

/// Build a draft manual linear-gradient mask preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_manual_linear_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_linear_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        start_x,
        start_y,
        end_x,
        end_y,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;

    preview_manual_mask_edit(library_root_path, photo_id, &edited)
}

/// Build a draft manual radial-gradient mask preview without writing the catalog.
#[allow(clippy::too_many_arguments)]
pub fn preview_manual_radial_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_radial_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        center_x,
        center_y,
        radius_x,
        radius_y,
        rotation,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;

    preview_manual_mask_edit(library_root_path, photo_id, &edited)
}

/// Build a draft manual brush mask preview without writing durable edit state.
#[allow(clippy::too_many_arguments)]
pub fn preview_manual_brush_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    strokes: Vec<PhotoManualBrushStrokeInput>,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_brush_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        edit_brush_strokes(strokes)?,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;

    preview_manual_mask_edit(library_root_path, photo_id, &edited)
}

/// Persist a manual linear-gradient mask on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_manual_linear_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_linear_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        start_x,
        start_y,
        end_x,
        end_y,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;
    ensure_supported_manual_masks_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Manual linear gradient mask persisted on commit.",
    )))
}

/// Persist a manual radial-gradient mask on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_manual_radial_gradient_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    center_x: f64,
    center_y: f64,
    radius_x: f64,
    radius_y: f64,
    rotation: f64,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_radial_gradient_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        center_x,
        center_y,
        radius_x,
        radius_y,
        rotation,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;
    ensure_supported_manual_masks_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Manual radial gradient mask persisted on commit.",
    )))
}

/// Persist a manual brush mask on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_manual_brush_mask(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    id: &str,
    name: &str,
    opacity: f64,
    feather: f64,
    invert: bool,
    strokes: Vec<PhotoManualBrushStrokeInput>,
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mask = silica_edit::manual_brush_mask(
        id,
        name,
        opacity,
        feather,
        invert,
        edit_brush_strokes(strokes)?,
        manual_mask_adjustments(exposure, contrast),
    )?;
    let edited = silica_edit::append_manual_mask(&graph, mask, current_timestamp_string())?;
    ensure_supported_manual_masks_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Manual brush mask persisted on commit.",
    )))
}

/// Detail commit is intentionally blocked until a real renderer/export path exists.
pub fn commit_detail_sharpening_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let _edited = silica_edit::apply_detail_sharpening(
        &graph,
        amount,
        radius,
        detail,
        masking,
        current_timestamp_string(),
    )?;

    Err(CoreError::UnsupportedEdit(detail_unsupported_message()))
}

/// Detail commit is intentionally blocked until a real renderer/export path exists.
#[allow(clippy::too_many_arguments)]
pub fn commit_detail_noise_reduction_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let _edited = silica_edit::apply_detail_noise_reduction(
        &graph,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
        current_timestamp_string(),
    )?;

    Err(CoreError::UnsupportedEdit(detail_unsupported_message()))
}

/// Persist a rectangular crop edit on commit/release.
#[allow(clippy::too_many_arguments)]
pub fn commit_geometry_crop_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<&str>,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_crop(
        &graph,
        x,
        y,
        width,
        height,
        angle,
        aspect,
        current_timestamp_string(),
    )?;
    ensure_supported_lens_geometry_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Geometry crop edit persisted on commit.",
    )))
}

/// Persist a crop-clear edit on commit/release.
pub fn commit_clear_geometry_crop(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::clear_geometry_crop(&graph, current_timestamp_string())?;
    ensure_supported_lens_geometry_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Geometry crop cleared on commit.",
    )))
}

/// Persist a rotation/flip edit on commit/release.
pub fn commit_geometry_orientation_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_geometry_orientation(
        &graph,
        rotation,
        flip_horizontal,
        flip_vertical,
        current_timestamp_string(),
    )?;
    ensure_supported_lens_geometry_commit(&edited)?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(photo_edit_commit_from_graph(
        &persisted,
        "Geometry orientation edit persisted on commit.",
    )))
}

/// Persist an exposure/contrast edit on commit/release.
pub fn commit_exposure_contrast_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    exposure: f64,
    contrast: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
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
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(exposure),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(contrast),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Exposure/contrast edit persisted on commit.".to_string(),
    }))
}

/// Persist a white-balance edit on commit/release.
pub fn commit_white_balance_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    white_balance: silica_edit::WhiteBalance,
    temperature: f64,
    tint: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_white_balance_temperature_tint(
        &graph,
        white_balance,
        temperature,
        tint,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(temperature),
        tint: persisted.basic.tint.as_f64().unwrap_or(tint),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "White balance edit persisted on commit.".to_string(),
    }))
}

/// Persist a tone-recovery edit on commit/release.
pub fn commit_tone_recovery_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_recovery(
        &graph,
        highlights,
        shadows,
        whites,
        blacks,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(highlights),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(shadows),
        whites: persisted.basic.whites.as_f64().unwrap_or(whites),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(blacks),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Tone recovery edit persisted on commit.".to_string(),
    }))
}

/// Persist a tone-curve edit on commit/release.
pub fn commit_tone_curve_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    rgb_curve: &[(f64, f64)],
    red_curve: &[(f64, f64)],
    green_curve: &[(f64, f64)],
    blue_curve: &[(f64, f64)],
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_tone_curve(
        &graph,
        silica_edit::CurveMode::Point,
        rgb_curve,
        red_curve,
        green_curve,
        blue_curve,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Tone curve edit persisted on commit.".to_string(),
    }))
}

/// Persist an HSL color mixer edit on commit/release.
pub fn commit_hsl_color_mixer_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    channel: silica_edit::HslColorChannel,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_hsl_color_channel(
        &graph,
        channel,
        hue,
        saturation,
        luminance,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "HSL color mixer edit persisted on commit.".to_string(),
    }))
}

/// Persist a color-presence edit on commit/release.
pub fn commit_color_presence_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    vibrance: f64,
    saturation: f64,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_color_presence(
        &graph,
        vibrance,
        saturation,
        current_timestamp_string(),
    )?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(vibrance),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(saturation),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Color presence edit persisted on commit.".to_string(),
    }))
}

/// Persist a full P0 Basic reset as one undoable edit checkpoint.
pub fn commit_p0_basic_reset(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::reset_p0_basic_controls(&graph, current_timestamp_string())?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "P0 Basic reset persisted on commit.".to_string(),
    }))
}

/// Persist a built-in Basic preset as one undoable edit checkpoint.
pub fn commit_basic_preset_edit(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    preset: silica_edit::BasicPreset,
) -> Result<Option<PhotoEditCommit>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let edited = silica_edit::apply_basic_preset(&graph, preset, current_timestamp_string())?;
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;

    Ok(Some(PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&persisted),
        detail: detail_state_from_graph(&persisted),
        geometry: geometry_state_from_graph(&persisted),
        masks: photo_manual_masks_from_graph(&persisted),
        persisted: true,
        message: "Basic preset persisted on commit.".to_string(),
    }))
}

/// Read the current exposure/contrast edit state without mutating the catalog.
pub fn get_photo_edit_state(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoEditState>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if let Some(graph) = silica_storage::load_active_edit_graph(library_root_path, photo_id)? {
        return Ok(Some(PhotoEditState {
            photo_id: graph.source.photo_id.clone(),
            exposure: graph.basic.exposure.as_f64().unwrap_or(0.0),
            contrast: graph.basic.contrast.as_f64().unwrap_or(0.0),
            white_balance: graph.basic.white_balance,
            temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
            tint: graph.basic.tint.as_f64().unwrap_or(0.0),
            highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
            shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
            whites: graph.basic.whites.as_f64().unwrap_or(0.0),
            blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
            vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
            saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
            tone_curve: tone_curve_state_from_graph(&graph),
            hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
            detail: detail_state_from_graph(&graph),
            geometry: geometry_state_from_graph(&graph),
            masks: photo_manual_masks_from_graph(&graph),
            persisted: true,
            message: "Restored committed edit state.".to_string(),
        }));
    }

    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };

    Ok(Some(PhotoEditState {
        photo_id: graph.source.photo_id.clone(),
        exposure: graph.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: graph.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(&graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(&graph),
        detail: detail_state_from_graph(&graph),
        geometry: geometry_state_from_graph(&graph),
        masks: photo_manual_masks_from_graph(&graph),
        persisted: false,
        message: "Default clean edit state loaded.".to_string(),
    }))
}
pub(super) fn preview_render_plan(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<(String, String, silica_render::PreviewRenderPlan)>, CoreError> {
    let candidate = match silica_storage::get_photo_preview_candidate(library_root_path, photo_id)?
    {
        Some(candidate) => candidate,
        None => return Ok(None),
    };
    let source_path = PathBuf::from(&candidate.path);
    if !source_path.is_file() {
        return Ok(Some((
            candidate.photo_id,
            candidate.file_name,
            silica_render::PreviewRenderPlan {
                source_path: candidate.path,
                status: silica_render::PreviewRenderStatus::BlockedByDecode,
                color_behavior: silica_render::PreviewColorBehavior::DisplayProfileAware,
                message: "Preview unavailable because the referenced source file is missing."
                    .to_string(),
            },
        )));
    }
    let decode_plan = silica_decode::plan_preview_decode(&candidate.path, candidate.unsupported);
    let render_plan = silica_render::plan_preview_render(decode_plan);
    Ok(Some((candidate.photo_id, candidate.file_name, render_plan)))
}

pub(super) fn mark_runtime_missing_source(
    mut photo: silica_storage::LibraryPhotoGridItem,
) -> silica_storage::LibraryPhotoGridItem {
    if !Path::new(&photo.path).is_file() {
        photo.missing = true;
        photo.thumbnail_path = None;
        photo.thumbnail_cache_key = None;
    }
    photo
}

fn manual_mask_adjustments(
    exposure: Option<f64>,
    contrast: Option<f64>,
) -> silica_edit::ManualMaskLocalAdjustments {
    silica_edit::ManualMaskLocalAdjustments { exposure, contrast }
}

fn edit_brush_strokes(
    strokes: Vec<PhotoManualBrushStrokeInput>,
) -> Result<Vec<silica_edit::MaskBrushStroke>, CoreError> {
    strokes
        .into_iter()
        .map(|stroke| {
            let points = stroke
                .points
                .into_iter()
                .map(|point| (point.x, point.y))
                .collect::<Vec<_>>();
            silica_edit::manual_brush_stroke(stroke.id, stroke.radius, points)
                .map_err(CoreError::from)
        })
        .collect()
}

fn preview_manual_mask_edit(
    library_root_path: &Path,
    photo_id: &str,
    edited: &silica_edit::EditGraph,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_manual_mask_preview(
        render_plan,
        edited.basic.exposure.as_f64().unwrap_or(0.0),
        edited.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(edited),
        render_tone_recovery_from_graph(edited),
        render_color_presence_from_graph(edited),
        render_tone_curve_from_graph(edited),
        render_hsl_color_mixer_from_graph(edited),
        render_detail_from_graph(edited),
        render_geometry_from_graph(edited),
        render_manual_masks_from_graph(edited)?,
    );
    let request = apply_lens_geometry_preview_boundary(request, edited);
    let request = apply_manual_mask_preview_boundary(request, edited)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry.clone()),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: edited.basic.white_balance,
        temperature: edited.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: edited.basic.tint.as_f64().unwrap_or(0.0),
        highlights: edited.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: edited.basic.shadows.as_f64().unwrap_or(0.0),
        whites: edited.basic.whites.as_f64().unwrap_or(0.0),
        blacks: edited.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: edited.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: edited.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(edited),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(edited),
        detail: detail_state_from_graph(edited),
        geometry: geometry_state_from_graph(edited),
        masks: photo_manual_masks_from_graph(edited),
        message,
    }))
}

fn preview_detail_edit(
    library_root_path: &Path,
    photo_id: &str,
    graph: &silica_edit::EditGraph,
    edited: &silica_edit::EditGraph,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_detail_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(graph),
        render_tone_recovery_from_graph(graph),
        render_color_presence_from_graph(graph),
        render_tone_curve_from_graph(graph),
        render_hsl_color_mixer_from_graph(graph),
        render_detail_from_graph(edited),
    );
    let request = apply_lens_geometry_preview_boundary(request, edited);
    let request = apply_manual_mask_preview_boundary(request, edited)?;
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(graph),
        detail: detail_state_from_graph(edited),
        geometry: geometry_state_from_graph(edited),
        masks: photo_manual_masks_from_graph(edited),
        message,
    }))
}

fn preview_geometry_edit(
    library_root_path: &Path,
    photo_id: &str,
    graph: &silica_edit::EditGraph,
    edited: &silica_edit::EditGraph,
) -> Result<Option<PhotoEditPreviewSession>, CoreError> {
    let (photo_id, _file_name, render_plan) =
        match preview_render_plan(library_root_path, photo_id)? {
            Some(plan) => plan,
            None => return Ok(None),
        };
    let request = silica_render::plan_geometry_preview(
        render_plan,
        graph.basic.exposure.as_f64().unwrap_or(0.0),
        graph.basic.contrast.as_f64().unwrap_or(0.0),
        render_white_balance_from_graph(graph),
        render_tone_recovery_from_graph(graph),
        render_color_presence_from_graph(graph),
        render_tone_curve_from_graph(graph),
        render_hsl_color_mixer_from_graph(graph),
        render_detail_from_graph(graph),
        render_geometry_from_graph(edited),
    );
    let request = apply_lens_geometry_preview_boundary(request, edited);
    let source_is_supported_raster =
        is_supported_raster_source_path(Path::new(&request.source_path));
    let mut message = request.message;
    let status = match preview_status_from_render(request.status) {
        PhotoPreviewStatus::Ready if !source_is_supported_raster => {
            message = "JPEG/JPG/PNG/TIFF Develop preview pixels are enabled for local alpha raster sources."
                .to_string();
            PhotoPreviewStatus::BlockedByDecode
        }
        status => status,
    };
    let develop_preview_bytes = if status == PhotoPreviewStatus::Ready {
        write_jpeg_develop_preview_bytes(
            library_root_path,
            &photo_id,
            &request.source_path,
            request.exposure,
            request.contrast,
            export_white_balance_from_render(request.white_balance),
            export_tone_recovery_from_render(request.tone_recovery),
            export_color_presence_from_render(request.color_presence),
            export_tone_curve_from_render(request.tone_curve.clone()),
            export_hsl_color_mixer_from_render(request.hsl_color_mixer),
            export_detail_from_render(request.detail),
            export_geometry_from_render(request.geometry),
            request.masks.clone(),
        )?
    } else {
        None
    };

    Ok(Some(PhotoEditPreviewSession {
        photo_id,
        source_path: request.source_path,
        develop_preview_bytes,
        status,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: graph.basic.white_balance,
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(graph),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(graph),
        detail: detail_state_from_graph(graph),
        geometry: geometry_state_from_graph(edited),
        masks: photo_manual_masks_from_graph(edited),
        message,
    }))
}

pub(super) fn photo_edit_commit_from_graph(
    persisted: &silica_edit::EditGraph,
    message: impl Into<String>,
) -> PhotoEditCommit {
    PhotoEditCommit {
        photo_id: persisted.source.photo_id.clone(),
        exposure: persisted.basic.exposure.as_f64().unwrap_or(0.0),
        contrast: persisted.basic.contrast.as_f64().unwrap_or(0.0),
        white_balance: persisted.basic.white_balance,
        temperature: persisted.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: persisted.basic.tint.as_f64().unwrap_or(0.0),
        highlights: persisted.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: persisted.basic.shadows.as_f64().unwrap_or(0.0),
        whites: persisted.basic.whites.as_f64().unwrap_or(0.0),
        blacks: persisted.basic.blacks.as_f64().unwrap_or(0.0),
        vibrance: persisted.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: persisted.basic.saturation.as_f64().unwrap_or(0.0),
        tone_curve: tone_curve_state_from_graph(persisted),
        hsl_color_mixer: hsl_color_mixer_state_from_graph(persisted),
        detail: detail_state_from_graph(persisted),
        geometry: geometry_state_from_graph(persisted),
        masks: photo_manual_masks_from_graph(persisted),
        persisted: true,
        message: message.into(),
    }
}
