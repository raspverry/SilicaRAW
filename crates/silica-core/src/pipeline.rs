use super::*;

pub(super) fn preview_status_from_render(
    status: silica_render::PreviewRenderStatus,
) -> PhotoPreviewStatus {
    match status {
        silica_render::PreviewRenderStatus::Ready => PhotoPreviewStatus::Ready,
        silica_render::PreviewRenderStatus::BlockedByDecode => PhotoPreviewStatus::BlockedByDecode,
        silica_render::PreviewRenderStatus::Unsupported => PhotoPreviewStatus::Unsupported,
    }
}

pub(super) fn export_format_string(format: silica_export::ExportImageFormat) -> &'static str {
    match format {
        silica_export::ExportImageFormat::Jpeg => "jpeg",
        silica_export::ExportImageFormat::Png => "png",
        silica_export::ExportImageFormat::Tiff => "tiff",
    }
}

pub(super) fn export_raster_format_to_export(
    format: PhotoExportFormat,
) -> silica_export::ExportImageFormat {
    match format {
        PhotoExportFormat::Jpeg => silica_export::ExportImageFormat::Jpeg,
        PhotoExportFormat::Png => silica_export::ExportImageFormat::Png,
        PhotoExportFormat::Tiff => silica_export::ExportImageFormat::Tiff,
    }
}

pub(super) fn export_color_profile_to_export(
    profile: PhotoExportColorProfile,
) -> silica_export::ExportColorProfile {
    match profile {
        PhotoExportColorProfile::Srgb => silica_export::ExportColorProfile::Srgb,
        PhotoExportColorProfile::DisplayP3 => silica_export::ExportColorProfile::DisplayP3,
    }
}

pub(super) fn export_color_profile_string(
    profile: silica_export::ExportColorProfile,
) -> &'static str {
    match profile {
        silica_export::ExportColorProfile::Srgb => "srgb",
        silica_export::ExportColorProfile::DisplayP3 => "display_p3",
    }
}

pub(super) fn export_metadata_policy_to_export(
    policy: PhotoExportMetadataPolicy,
) -> silica_export::ExportMetadataPolicy {
    match policy {
        PhotoExportMetadataPolicy::Minimal => silica_export::ExportMetadataPolicy::Minimal,
        PhotoExportMetadataPolicy::Preserve => silica_export::ExportMetadataPolicy::Preserve,
        PhotoExportMetadataPolicy::RemoveGps => silica_export::ExportMetadataPolicy::RemoveGps,
        PhotoExportMetadataPolicy::RemoveAll => silica_export::ExportMetadataPolicy::RemoveAll,
    }
}

pub(super) fn export_metadata_policy_string(policy: PhotoExportMetadataPolicy) -> &'static str {
    match policy {
        PhotoExportMetadataPolicy::Minimal => "minimal",
        PhotoExportMetadataPolicy::Preserve => "preserve",
        PhotoExportMetadataPolicy::RemoveGps => "remove_gps",
        PhotoExportMetadataPolicy::RemoveAll => "remove_all",
    }
}

pub(super) fn render_white_balance_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::WhiteBalanceRenderAdjustment {
    silica_render::WhiteBalanceRenderAdjustment {
        mode: render_white_balance_mode(graph.basic.white_balance),
        temperature: graph.basic.temperature.as_f64().unwrap_or(5200.0),
        tint: graph.basic.tint.as_f64().unwrap_or(0.0),
    }
}

fn render_white_balance_mode(
    mode: silica_edit::WhiteBalance,
) -> silica_render::WhiteBalanceRenderMode {
    match mode {
        silica_edit::WhiteBalance::AsShot => silica_render::WhiteBalanceRenderMode::AsShot,
        silica_edit::WhiteBalance::Auto => silica_render::WhiteBalanceRenderMode::Auto,
        silica_edit::WhiteBalance::Daylight => silica_render::WhiteBalanceRenderMode::Daylight,
        silica_edit::WhiteBalance::Cloudy => silica_render::WhiteBalanceRenderMode::Cloudy,
        silica_edit::WhiteBalance::Shade => silica_render::WhiteBalanceRenderMode::Shade,
        silica_edit::WhiteBalance::Tungsten => silica_render::WhiteBalanceRenderMode::Tungsten,
        silica_edit::WhiteBalance::Fluorescent => {
            silica_render::WhiteBalanceRenderMode::Fluorescent
        }
        silica_edit::WhiteBalance::Flash => silica_render::WhiteBalanceRenderMode::Flash,
        silica_edit::WhiteBalance::Custom => silica_render::WhiteBalanceRenderMode::Custom,
    }
}

pub(super) fn export_white_balance_from_render(
    white_balance: silica_render::WhiteBalanceRenderAdjustment,
) -> silica_export::WhiteBalanceAdjustment {
    silica_export::WhiteBalanceAdjustment {
        mode: export_white_balance_mode(white_balance.mode),
        temperature: white_balance.temperature,
        tint: white_balance.tint,
    }
}

pub(super) fn render_tone_recovery_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::ToneRecoveryRenderAdjustment {
    silica_render::ToneRecoveryRenderAdjustment {
        highlights: graph.basic.highlights.as_f64().unwrap_or(0.0),
        shadows: graph.basic.shadows.as_f64().unwrap_or(0.0),
        whites: graph.basic.whites.as_f64().unwrap_or(0.0),
        blacks: graph.basic.blacks.as_f64().unwrap_or(0.0),
    }
}

pub(super) fn export_tone_recovery_from_render(
    tone_recovery: silica_render::ToneRecoveryRenderAdjustment,
) -> silica_export::ToneRecoveryAdjustment {
    silica_export::ToneRecoveryAdjustment {
        highlights: tone_recovery.highlights,
        shadows: tone_recovery.shadows,
        whites: tone_recovery.whites,
        blacks: tone_recovery.blacks,
    }
}

pub(super) fn tone_curve_state_from_graph(graph: &silica_edit::EditGraph) -> PhotoToneCurveState {
    PhotoToneCurveState {
        curve_mode: graph.tone.curve_mode,
        rgb_curve: photo_tone_curve_points_from_edit(&graph.tone.rgb_curve),
        red_curve: photo_tone_curve_points_from_edit(&graph.tone.red_curve),
        green_curve: photo_tone_curve_points_from_edit(&graph.tone.green_curve),
        blue_curve: photo_tone_curve_points_from_edit(&graph.tone.blue_curve),
    }
}

fn photo_tone_curve_points_from_edit(
    points: &[silica_edit::CurvePoint],
) -> Vec<PhotoToneCurvePoint> {
    points
        .iter()
        .map(|point| PhotoToneCurvePoint {
            x: point.x.as_f64().unwrap_or(0.0),
            y: point.y.as_f64().unwrap_or(0.0),
        })
        .collect()
}

pub(super) fn render_tone_curve_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::ToneCurveRenderAdjustment {
    silica_render::ToneCurveRenderAdjustment {
        mode: match graph.tone.curve_mode {
            silica_edit::CurveMode::None => silica_render::ToneCurveRenderMode::None,
            silica_edit::CurveMode::Point => silica_render::ToneCurveRenderMode::Point,
            silica_edit::CurveMode::Parametric => silica_render::ToneCurveRenderMode::Parametric,
        },
        rgb_curve: render_tone_curve_points_from_edit(&graph.tone.rgb_curve),
        red_curve: render_tone_curve_points_from_edit(&graph.tone.red_curve),
        green_curve: render_tone_curve_points_from_edit(&graph.tone.green_curve),
        blue_curve: render_tone_curve_points_from_edit(&graph.tone.blue_curve),
    }
}

fn render_tone_curve_points_from_edit(
    points: &[silica_edit::CurvePoint],
) -> Vec<silica_render::ToneCurveRenderPoint> {
    points
        .iter()
        .map(|point| silica_render::ToneCurveRenderPoint {
            x: point.x.as_f64().unwrap_or(0.0),
            y: point.y.as_f64().unwrap_or(0.0),
        })
        .collect()
}

pub(super) fn export_tone_curve_from_render(
    tone_curve: silica_render::ToneCurveRenderAdjustment,
) -> silica_export::ToneCurveAdjustment {
    silica_export::ToneCurveAdjustment {
        mode: match tone_curve.mode {
            silica_render::ToneCurveRenderMode::None => silica_export::ToneCurveMode::None,
            silica_render::ToneCurveRenderMode::Parametric => {
                silica_export::ToneCurveMode::Parametric
            }
            silica_render::ToneCurveRenderMode::Point => silica_export::ToneCurveMode::Point,
        },
        rgb_curve: tone_curve_points_to_export(tone_curve.rgb_curve),
        red_curve: tone_curve_points_to_export(tone_curve.red_curve),
        green_curve: tone_curve_points_to_export(tone_curve.green_curve),
        blue_curve: tone_curve_points_to_export(tone_curve.blue_curve),
    }
}

fn tone_curve_points_to_export(
    points: Vec<silica_render::ToneCurveRenderPoint>,
) -> Vec<silica_export::ToneCurvePoint> {
    points
        .into_iter()
        .map(|point| silica_export::ToneCurvePoint {
            x: point.x,
            y: point.y,
        })
        .collect()
}

pub(super) fn tone_curve_settings_json(
    tone_curve: &silica_render::ToneCurveRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "curve_mode": match tone_curve.mode {
            silica_render::ToneCurveRenderMode::None => "none",
            silica_render::ToneCurveRenderMode::Parametric => "parametric",
            silica_render::ToneCurveRenderMode::Point => "point",
        },
        "rgb_curve": tone_curve_points_json(&tone_curve.rgb_curve),
        "red_curve": tone_curve_points_json(&tone_curve.red_curve),
        "green_curve": tone_curve_points_json(&tone_curve.green_curve),
        "blue_curve": tone_curve_points_json(&tone_curve.blue_curve),
    })
}

fn tone_curve_points_json(
    points: &[silica_render::ToneCurveRenderPoint],
) -> Vec<serde_json::Value> {
    points
        .iter()
        .map(|point| serde_json::json!({ "x": point.x, "y": point.y }))
        .collect()
}

pub(super) fn hsl_color_mixer_state_from_graph(
    graph: &silica_edit::EditGraph,
) -> PhotoHslColorMixerState {
    PhotoHslColorMixerState {
        red: photo_hsl_color_channel_from_edit(&graph.color.hsl.red),
        orange: photo_hsl_color_channel_from_edit(&graph.color.hsl.orange),
        yellow: photo_hsl_color_channel_from_edit(&graph.color.hsl.yellow),
        green: photo_hsl_color_channel_from_edit(&graph.color.hsl.green),
        aqua: photo_hsl_color_channel_from_edit(&graph.color.hsl.aqua),
        blue: photo_hsl_color_channel_from_edit(&graph.color.hsl.blue),
        purple: photo_hsl_color_channel_from_edit(&graph.color.hsl.purple),
        magenta: photo_hsl_color_channel_from_edit(&graph.color.hsl.magenta),
    }
}

fn photo_hsl_color_channel_from_edit(
    channel: &silica_edit::HslChannel,
) -> PhotoHslColorChannelState {
    PhotoHslColorChannelState {
        hue: channel.hue.as_f64().unwrap_or(0.0),
        saturation: channel.saturation.as_f64().unwrap_or(0.0),
        luminance: channel.luminance.as_f64().unwrap_or(0.0),
    }
}

pub(super) fn render_hsl_color_mixer_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::HslColorMixerRenderAdjustment {
    silica_render::HslColorMixerRenderAdjustment {
        red: render_hsl_color_channel_from_edit(&graph.color.hsl.red),
        orange: render_hsl_color_channel_from_edit(&graph.color.hsl.orange),
        yellow: render_hsl_color_channel_from_edit(&graph.color.hsl.yellow),
        green: render_hsl_color_channel_from_edit(&graph.color.hsl.green),
        aqua: render_hsl_color_channel_from_edit(&graph.color.hsl.aqua),
        blue: render_hsl_color_channel_from_edit(&graph.color.hsl.blue),
        purple: render_hsl_color_channel_from_edit(&graph.color.hsl.purple),
        magenta: render_hsl_color_channel_from_edit(&graph.color.hsl.magenta),
    }
}

fn render_hsl_color_channel_from_edit(
    channel: &silica_edit::HslChannel,
) -> silica_render::HslColorChannelRenderAdjustment {
    silica_render::HslColorChannelRenderAdjustment {
        hue: channel.hue.as_f64().unwrap_or(0.0),
        saturation: channel.saturation.as_f64().unwrap_or(0.0),
        luminance: channel.luminance.as_f64().unwrap_or(0.0),
    }
}

pub(super) fn export_hsl_color_mixer_from_render(
    hsl_color_mixer: silica_render::HslColorMixerRenderAdjustment,
) -> silica_export::HslColorMixerAdjustment {
    silica_export::HslColorMixerAdjustment {
        red: hsl_color_channel_to_export(hsl_color_mixer.red),
        orange: hsl_color_channel_to_export(hsl_color_mixer.orange),
        yellow: hsl_color_channel_to_export(hsl_color_mixer.yellow),
        green: hsl_color_channel_to_export(hsl_color_mixer.green),
        aqua: hsl_color_channel_to_export(hsl_color_mixer.aqua),
        blue: hsl_color_channel_to_export(hsl_color_mixer.blue),
        purple: hsl_color_channel_to_export(hsl_color_mixer.purple),
        magenta: hsl_color_channel_to_export(hsl_color_mixer.magenta),
    }
}

fn hsl_color_channel_to_export(
    channel: silica_render::HslColorChannelRenderAdjustment,
) -> silica_export::HslColorChannelAdjustment {
    silica_export::HslColorChannelAdjustment {
        hue: channel.hue,
        saturation: channel.saturation,
        luminance: channel.luminance,
    }
}

pub(super) fn hsl_color_mixer_settings_json(
    hsl_color_mixer: &silica_render::HslColorMixerRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "red": hsl_color_channel_settings_json(hsl_color_mixer.red),
        "orange": hsl_color_channel_settings_json(hsl_color_mixer.orange),
        "yellow": hsl_color_channel_settings_json(hsl_color_mixer.yellow),
        "green": hsl_color_channel_settings_json(hsl_color_mixer.green),
        "aqua": hsl_color_channel_settings_json(hsl_color_mixer.aqua),
        "blue": hsl_color_channel_settings_json(hsl_color_mixer.blue),
        "purple": hsl_color_channel_settings_json(hsl_color_mixer.purple),
        "magenta": hsl_color_channel_settings_json(hsl_color_mixer.magenta),
    })
}

fn hsl_color_channel_settings_json(
    channel: silica_render::HslColorChannelRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "hue": channel.hue,
        "saturation": channel.saturation,
        "luminance": channel.luminance,
    })
}

pub(super) fn detail_state_from_graph(graph: &silica_edit::EditGraph) -> PhotoDetailState {
    PhotoDetailState {
        sharpening: PhotoDetailSharpeningState {
            amount: graph.detail.sharpening.amount.as_f64().unwrap_or(0.0),
            radius: graph.detail.sharpening.radius.as_f64().unwrap_or(1.0),
            detail: graph.detail.sharpening.detail.as_f64().unwrap_or(25.0),
            masking: graph.detail.sharpening.masking.as_f64().unwrap_or(0.0),
        },
        noise_reduction: PhotoDetailNoiseReductionState {
            luminance: graph
                .detail
                .noise_reduction
                .luminance
                .as_f64()
                .unwrap_or(0.0),
            detail: graph.detail.noise_reduction.detail.as_f64().unwrap_or(50.0),
            contrast: graph
                .detail
                .noise_reduction
                .contrast
                .as_f64()
                .unwrap_or(0.0),
            color: graph.detail.noise_reduction.color.as_f64().unwrap_or(25.0),
            color_detail: graph
                .detail
                .noise_reduction
                .color_detail
                .as_f64()
                .unwrap_or(50.0),
        },
    }
}

pub(super) fn render_detail_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::DetailRenderAdjustment {
    silica_render::DetailRenderAdjustment {
        sharpening: silica_render::DetailSharpeningRenderAdjustment {
            amount: graph.detail.sharpening.amount.as_f64().unwrap_or(0.0),
            radius: graph.detail.sharpening.radius.as_f64().unwrap_or(1.0),
            detail: graph.detail.sharpening.detail.as_f64().unwrap_or(25.0),
            masking: graph.detail.sharpening.masking.as_f64().unwrap_or(0.0),
        },
        noise_reduction: silica_render::DetailNoiseReductionRenderAdjustment {
            luminance: graph
                .detail
                .noise_reduction
                .luminance
                .as_f64()
                .unwrap_or(0.0),
            detail: graph.detail.noise_reduction.detail.as_f64().unwrap_or(50.0),
            contrast: graph
                .detail
                .noise_reduction
                .contrast
                .as_f64()
                .unwrap_or(0.0),
            color: graph.detail.noise_reduction.color.as_f64().unwrap_or(25.0),
            color_detail: graph
                .detail
                .noise_reduction
                .color_detail
                .as_f64()
                .unwrap_or(50.0),
        },
    }
}

pub(super) fn export_detail_from_render(
    detail: silica_render::DetailRenderAdjustment,
) -> silica_export::DetailAdjustment {
    silica_export::DetailAdjustment {
        sharpening: silica_export::DetailSharpeningAdjustment {
            amount: detail.sharpening.amount,
            radius: detail.sharpening.radius,
            detail: detail.sharpening.detail,
            masking: detail.sharpening.masking,
        },
        noise_reduction: silica_export::DetailNoiseReductionAdjustment {
            luminance: detail.noise_reduction.luminance,
            detail: detail.noise_reduction.detail,
            contrast: detail.noise_reduction.contrast,
            color: detail.noise_reduction.color,
            color_detail: detail.noise_reduction.color_detail,
        },
    }
}

pub(super) fn detail_settings_json(
    detail: &silica_render::DetailRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "sharpening": {
            "amount": detail.sharpening.amount,
            "radius": detail.sharpening.radius,
            "detail": detail.sharpening.detail,
            "masking": detail.sharpening.masking,
        },
        "noise_reduction": {
            "luminance": detail.noise_reduction.luminance,
            "detail": detail.noise_reduction.detail,
            "contrast": detail.noise_reduction.contrast,
            "color": detail.noise_reduction.color,
            "color_detail": detail.noise_reduction.color_detail,
        },
        "mlx_denoise": "deferred",
    })
}

pub(super) fn geometry_state_from_graph(graph: &silica_edit::EditGraph) -> PhotoGeometryState {
    PhotoGeometryState {
        crop: graph
            .geometry
            .crop
            .as_ref()
            .map(|crop| PhotoGeometryCropState {
                x: crop.x.as_f64().unwrap_or(0.0),
                y: crop.y.as_f64().unwrap_or(0.0),
                width: crop.width.as_f64().unwrap_or(1.0),
                height: crop.height.as_f64().unwrap_or(1.0),
                angle: crop.angle.as_f64().unwrap_or(0.0),
                aspect: crop.aspect.clone(),
            }),
        rotation: graph.geometry.rotation.as_f64().unwrap_or(0.0),
        flip_horizontal: graph.geometry.flip_horizontal,
        flip_vertical: graph.geometry.flip_vertical,
        transform: PhotoGeometryTransformState {
            vertical: graph.geometry.transform.vertical.as_f64().unwrap_or(0.0),
            horizontal: graph.geometry.transform.horizontal.as_f64().unwrap_or(0.0),
            aspect: graph.geometry.transform.aspect.as_f64().unwrap_or(0.0),
            scale: graph.geometry.transform.scale.as_f64().unwrap_or(100.0),
            x_offset: graph.geometry.transform.x_offset.as_f64().unwrap_or(0.0),
            y_offset: graph.geometry.transform.y_offset.as_f64().unwrap_or(0.0),
        },
    }
}

pub(super) fn render_geometry_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::GeometryRenderAdjustment {
    silica_render::GeometryRenderAdjustment {
        crop: graph.geometry.crop.as_ref().map(|crop| {
            silica_render::GeometryCropRenderAdjustment {
                x: crop.x.as_f64().unwrap_or(0.0),
                y: crop.y.as_f64().unwrap_or(0.0),
                width: crop.width.as_f64().unwrap_or(1.0),
                height: crop.height.as_f64().unwrap_or(1.0),
                angle: crop.angle.as_f64().unwrap_or(0.0),
                aspect: crop.aspect.clone(),
            }
        }),
        rotation: graph.geometry.rotation.as_f64().unwrap_or(0.0),
        flip_horizontal: graph.geometry.flip_horizontal,
        flip_vertical: graph.geometry.flip_vertical,
        transform: silica_render::GeometryTransformRenderAdjustment {
            vertical: graph.geometry.transform.vertical.as_f64().unwrap_or(0.0),
            horizontal: graph.geometry.transform.horizontal.as_f64().unwrap_or(0.0),
            aspect: graph.geometry.transform.aspect.as_f64().unwrap_or(0.0),
            scale: graph.geometry.transform.scale.as_f64().unwrap_or(100.0),
            x_offset: graph.geometry.transform.x_offset.as_f64().unwrap_or(0.0),
            y_offset: graph.geometry.transform.y_offset.as_f64().unwrap_or(0.0),
        },
    }
}

pub(super) fn export_geometry_from_render(
    geometry: silica_render::GeometryRenderAdjustment,
) -> silica_export::GeometryAdjustment {
    silica_export::GeometryAdjustment {
        crop: geometry
            .crop
            .map(|crop| silica_export::GeometryCropAdjustment {
                x: crop.x,
                y: crop.y,
                width: crop.width,
                height: crop.height,
                angle: crop.angle,
                aspect: crop.aspect,
            }),
        rotation: geometry.rotation,
        flip_horizontal: geometry.flip_horizontal,
        flip_vertical: geometry.flip_vertical,
        transform: silica_export::GeometryTransformAdjustment {
            vertical: geometry.transform.vertical,
            horizontal: geometry.transform.horizontal,
            aspect: geometry.transform.aspect,
            scale: geometry.transform.scale,
            x_offset: geometry.transform.x_offset,
            y_offset: geometry.transform.y_offset,
        },
    }
}

pub(super) fn geometry_settings_json(
    geometry: &silica_render::GeometryRenderAdjustment,
) -> serde_json::Value {
    serde_json::json!({
        "crop": geometry.crop.as_ref().map(|crop| {
            serde_json::json!({
                "x": crop.x,
                "y": crop.y,
                "width": crop.width,
                "height": crop.height,
                "angle": crop.angle,
                "aspect": crop.aspect,
            })
        }),
        "rotation": geometry.rotation,
        "flip_horizontal": geometry.flip_horizontal,
        "flip_vertical": geometry.flip_vertical,
        "transform": {
            "vertical": geometry.transform.vertical,
            "horizontal": geometry.transform.horizontal,
            "aspect": geometry.transform.aspect,
            "scale": geometry.transform.scale,
            "x_offset": geometry.transform.x_offset,
            "y_offset": geometry.transform.y_offset,
        },
    })
}

pub(super) fn manual_mask_settings_json(
    masks: &[silica_render::ManualMaskRenderAdjustment],
) -> serde_json::Value {
    serde_json::Value::Array(masks.iter().map(manual_mask_setting_json).collect())
}

fn manual_mask_setting_json(mask: &silica_render::ManualMaskRenderAdjustment) -> serde_json::Value {
    serde_json::json!({
        "id": mask.id,
        "kind": manual_mask_render_kind(&mask.geometry),
        "enabled": mask.enabled,
        "invert": mask.invert,
        "opacity": mask.opacity,
        "feather": mask.feather,
        "geometry": manual_mask_geometry_settings_json(&mask.geometry),
        "exposure": mask.exposure,
        "contrast": mask.contrast,
    })
}

fn manual_mask_render_kind(geometry: &silica_render::ManualMaskRenderGeometry) -> &'static str {
    match geometry {
        silica_render::ManualMaskRenderGeometry::LinearGradient { .. } => "linear_gradient",
        silica_render::ManualMaskRenderGeometry::RadialGradient { .. } => "radial_gradient",
        silica_render::ManualMaskRenderGeometry::BrushRaster { .. } => "brush",
    }
}

fn manual_mask_geometry_settings_json(
    geometry: &silica_render::ManualMaskRenderGeometry,
) -> serde_json::Value {
    match geometry {
        silica_render::ManualMaskRenderGeometry::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => serde_json::json!({
            "kind": "linear_gradient",
            "start_x": start_x,
            "start_y": start_y,
            "end_x": end_x,
            "end_y": end_y,
        }),
        silica_render::ManualMaskRenderGeometry::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        } => serde_json::json!({
            "kind": "radial_gradient",
            "center_x": center_x,
            "center_y": center_y,
            "radius_x": radius_x,
            "radius_y": radius_y,
            "rotation": rotation,
        }),
        silica_render::ManualMaskRenderGeometry::BrushRaster {
            width,
            height,
            cache_key,
            ..
        } => serde_json::json!({
            "kind": "brush_raster",
            "width": width,
            "height": height,
            "cache_key": cache_key,
        }),
    }
}

pub(super) fn detail_unsupported_message() -> String {
    "Detail preview/export is unsupported until renderer support exists.".to_string()
}
fn local_alpha_develop_source_block(
    library_root_path: &Path,
    photo_id: &str,
) -> Result<Option<(&'static str, String)>, CoreError> {
    let Some(metadata) = silica_storage::get_photo_metadata(library_root_path, photo_id)? else {
        return Ok(Some(("missing_photo", "Photo not found.".to_string())));
    };
    if metadata.unsupported || !is_supported_raster_file_type(&metadata.file_type) {
        return Ok(Some((
            "unsupported_source",
            "Develop edits are limited to supported raster source photos in this alpha."
                .to_string(),
        )));
    }
    if !Path::new(&metadata.source_path).is_file() {
        return Ok(Some((
            "missing_source",
            "Develop edits are blocked because the referenced source file is missing.".to_string(),
        )));
    }
    Ok(None)
}

pub(super) fn is_supported_raster_file_type(file_type: &str) -> bool {
    matches!(file_type, "jpeg" | "png" | "tiff")
}

pub(super) fn ensure_supported_develop_source(
    library_root_path: &Path,
    photo_id: &str,
) -> Result<bool, CoreError> {
    match local_alpha_develop_source_block(library_root_path, photo_id)? {
        Some(("missing_photo", _)) => Ok(false),
        Some((_, message)) => Err(CoreError::UnsupportedEdit(message)),
        None => Ok(true),
    }
}

pub(super) fn has_unsupported_basic_runtime(graph: &silica_edit::EditGraph) -> bool {
    graph.basic.texture.as_f64().unwrap_or(0.0) != 0.0
        || graph.basic.clarity.as_f64().unwrap_or(0.0) != 0.0
        || graph.basic.dehaze.as_f64().unwrap_or(0.0) != 0.0
}

pub(super) fn edit_graphs_equal_ignoring_updated_at(
    left: &silica_edit::EditGraph,
    right: &silica_edit::EditGraph,
) -> bool {
    let mut normalized_left = left.clone();
    let mut normalized_right = right.clone();
    normalized_left.updated_at.clear();
    normalized_right.updated_at.clear();
    normalized_left == normalized_right
}

pub(super) fn apply_detail_preview_boundary(
    mut request: silica_render::ExposureContrastPreviewRequest,
    detail: silica_render::DetailRenderAdjustment,
) -> silica_render::ExposureContrastPreviewRequest {
    request.detail = detail;
    if request.status == silica_render::PreviewRenderStatus::Ready && !detail.is_neutral() {
        request.status = silica_render::PreviewRenderStatus::Unsupported;
        request.message = detail_unsupported_message();
    }
    request
}

pub(super) fn apply_manual_mask_preview_boundary(
    mut request: silica_render::ExposureContrastPreviewRequest,
    graph: &silica_edit::EditGraph,
) -> Result<silica_render::ExposureContrastPreviewRequest, CoreError> {
    request.masks = render_manual_masks_from_graph(graph)?;
    Ok(request)
}

pub(super) fn apply_lens_geometry_preview_boundary(
    mut request: silica_render::ExposureContrastPreviewRequest,
    graph: &silica_edit::EditGraph,
) -> silica_render::ExposureContrastPreviewRequest {
    request.geometry = render_geometry_from_graph(graph);
    if request.status == silica_render::PreviewRenderStatus::Ready {
        if let Some(message) = lens_unsupported_message(graph)
            .or_else(|| geometry_unsupported_message(&request.geometry))
        {
            request.status = silica_render::PreviewRenderStatus::Unsupported;
            request.message = message;
        }
    }
    request
}

pub(super) fn lens_unsupported_message(graph: &silica_edit::EditGraph) -> Option<String> {
    let distortion = graph.lens.distortion.as_f64().unwrap_or(0.0);
    let vignetting = graph.lens.vignetting.as_f64().unwrap_or(0.0);
    if graph.lens.profile_correction
        || graph.lens.profile_id.is_some()
        || graph.lens.chromatic_aberration
        || distortion != 0.0
        || vignetting != 0.0
    {
        return Some(
            "Lens correction preview/export is unsupported until lens-profile support exists."
                .to_string(),
        );
    }
    None
}

pub(super) fn geometry_unsupported_message(
    geometry: &silica_render::GeometryRenderAdjustment,
) -> Option<String> {
    if !geometry.transform.is_neutral() {
        return Some(
            "Geometry transform preview/export is unsupported until renderer support exists."
                .to_string(),
        );
    }
    if let Some(crop) = &geometry.crop {
        if crop.angle != 0.0 {
            return Some(
                "Angled crop preview/export is unsupported until renderer support exists."
                    .to_string(),
            );
        }
    }
    if !is_supported_quarter_turn(geometry.rotation) {
        return Some(
            "Arbitrary rotation preview/export is unsupported until renderer support exists."
                .to_string(),
        );
    }
    None
}

pub(super) fn ensure_supported_lens_geometry_export(
    graph: &silica_edit::EditGraph,
    geometry: &silica_render::GeometryRenderAdjustment,
) -> Result<(), CoreError> {
    if let Some(message) = lens_unsupported_message(graph) {
        return Err(CoreError::ExportBlocked(message));
    }
    if let Some(message) = geometry_unsupported_message(geometry) {
        return Err(CoreError::ExportBlocked(message));
    }
    Ok(())
}

pub(super) fn ensure_supported_lens_geometry_commit(
    graph: &silica_edit::EditGraph,
) -> Result<(), CoreError> {
    if let Some(message) = lens_unsupported_message(graph) {
        return Err(CoreError::UnsupportedEdit(message));
    }
    if let Some(message) = geometry_unsupported_message(&render_geometry_from_graph(graph)) {
        return Err(CoreError::UnsupportedEdit(message));
    }
    Ok(())
}

pub(super) fn ensure_supported_manual_masks_commit(
    graph: &silica_edit::EditGraph,
) -> Result<(), CoreError> {
    render_manual_masks_from_graph(graph).map(|_| ())
}

pub(super) fn ensure_no_active_manual_masks_for_export(
    graph: &silica_edit::EditGraph,
) -> Result<(), CoreError> {
    if graph.masks.iter().any(|mask| mask.enabled) {
        return Err(CoreError::ExportBlocked(masked_export_blocked_message()));
    }
    Ok(())
}

fn masked_export_blocked_message() -> String {
    "Manual mask export is unsupported for RAW-derived export in the local alpha.".to_string()
}

fn manual_mask_unsupported_message(message: impl AsRef<str>) -> CoreError {
    CoreError::UnsupportedEdit(format!("Manual mask unsupported: {}", message.as_ref()))
}

fn manual_mask_type_string(mask_type: silica_edit::MaskType) -> &'static str {
    match mask_type {
        silica_edit::MaskType::Brush => "brush",
        silica_edit::MaskType::LinearGradient => "linear_gradient",
        silica_edit::MaskType::RadialGradient => "radial_gradient",
        silica_edit::MaskType::Subject => "subject",
        silica_edit::MaskType::Sky => "sky",
        silica_edit::MaskType::Background => "background",
        silica_edit::MaskType::ColorRange => "color_range",
        silica_edit::MaskType::LuminanceRange => "luminance_range",
    }
}

pub(super) fn photo_manual_masks_from_graph(
    graph: &silica_edit::EditGraph,
) -> Vec<PhotoManualMaskState> {
    graph
        .masks
        .iter()
        .filter_map(photo_manual_mask_from_edit)
        .collect()
}

fn photo_manual_mask_from_edit(mask: &silica_edit::Mask) -> Option<PhotoManualMaskState> {
    let geometry = match (&mask.mask_type, mask.geometry.as_ref()) {
        (
            silica_edit::MaskType::LinearGradient,
            Some(silica_edit::MaskGeometry::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
            }),
        ) => Some(PhotoManualMaskGeometryState::LinearGradient {
            start_x: start_x.as_f64().unwrap_or(0.0),
            start_y: start_y.as_f64().unwrap_or(0.0),
            end_x: end_x.as_f64().unwrap_or(1.0),
            end_y: end_y.as_f64().unwrap_or(1.0),
        }),
        (
            silica_edit::MaskType::RadialGradient,
            Some(silica_edit::MaskGeometry::RadialGradient {
                center_x,
                center_y,
                radius_x,
                radius_y,
                rotation,
            }),
        ) => Some(PhotoManualMaskGeometryState::RadialGradient {
            center_x: center_x.as_f64().unwrap_or(0.5),
            center_y: center_y.as_f64().unwrap_or(0.5),
            radius_x: radius_x.as_f64().unwrap_or(0.25),
            radius_y: radius_y.as_f64().unwrap_or(0.25),
            rotation: rotation.as_f64().unwrap_or(0.0),
        }),
        (silica_edit::MaskType::Brush, None) if mask.brush.is_some() => None,
        _ => return None,
    };

    Some(PhotoManualMaskState {
        id: mask.id.clone(),
        kind: manual_mask_type_string(mask.mask_type).to_string(),
        name: mask.name.clone(),
        enabled: mask.enabled,
        invert: mask.invert,
        opacity: mask.opacity.as_f64().unwrap_or(100.0),
        feather: mask.feather.as_f64().unwrap_or(0.0),
        geometry,
        exposure: mask
            .local_adjustments
            .get("exposure")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
        contrast: mask
            .local_adjustments
            .get("contrast")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0),
    })
}

pub(super) fn render_manual_masks_from_graph(
    graph: &silica_edit::EditGraph,
) -> Result<Vec<silica_render::ManualMaskRenderAdjustment>, CoreError> {
    graph
        .masks
        .iter()
        .map(render_manual_mask_from_edit)
        .collect()
}

fn render_manual_mask_from_edit(
    mask: &silica_edit::Mask,
) -> Result<silica_render::ManualMaskRenderAdjustment, CoreError> {
    if mask.source.kind != silica_edit::MaskSourceKind::Manual {
        return Err(manual_mask_unsupported_message(
            "only manual mask source is supported",
        ));
    }
    for key in mask.local_adjustments.keys() {
        if key != "exposure" && key != "contrast" {
            return Err(manual_mask_unsupported_message(format!(
                "local adjustment `{key}` is unsupported"
            )));
        }
    }

    let geometry = match (&mask.mask_type, &mask.geometry) {
        (silica_edit::MaskType::Brush, None) => {
            let brush = mask.brush.as_ref().ok_or_else(|| {
                manual_mask_unsupported_message("brush masks require durable brush strokes")
            })?;
            let strokes = render_brush_strokes_from_edit(brush);
            let raster = silica_render::rasterize_brush_mask(
                &mask.id,
                &strokes,
                LOCAL_ALPHA_BRUSH_MASK_RASTER_EDGE,
                LOCAL_ALPHA_BRUSH_MASK_RASTER_EDGE,
            )
            .map_err(|error| manual_mask_unsupported_message(error.to_string()))?;
            silica_render::ManualMaskRenderGeometry::BrushRaster {
                width: raster.width,
                height: raster.height,
                alpha: raster.alpha,
                cache_key: raster.cache_key,
            }
        }
        (
            silica_edit::MaskType::LinearGradient,
            Some(silica_edit::MaskGeometry::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
            }),
        ) => silica_render::ManualMaskRenderGeometry::LinearGradient {
            start_x: start_x.as_f64().unwrap_or(0.0),
            start_y: start_y.as_f64().unwrap_or(0.0),
            end_x: end_x.as_f64().unwrap_or(1.0),
            end_y: end_y.as_f64().unwrap_or(1.0),
        },
        (
            silica_edit::MaskType::RadialGradient,
            Some(silica_edit::MaskGeometry::RadialGradient {
                center_x,
                center_y,
                radius_x,
                radius_y,
                rotation,
            }),
        ) => silica_render::ManualMaskRenderGeometry::RadialGradient {
            center_x: center_x.as_f64().unwrap_or(0.5),
            center_y: center_y.as_f64().unwrap_or(0.5),
            radius_x: radius_x.as_f64().unwrap_or(0.25),
            radius_y: radius_y.as_f64().unwrap_or(0.25),
            rotation: rotation.as_f64().unwrap_or(0.0),
        },
        _ => {
            return Err(manual_mask_unsupported_message(
                "mask type and geometry payload must match",
            ))
        }
    };
    let exposure = manual_mask_local_value(mask, "exposure", -5.0, 5.0)?;
    let contrast = manual_mask_local_value(mask, "contrast", -100.0, 100.0)?;

    Ok(silica_render::ManualMaskRenderAdjustment {
        id: mask.id.clone(),
        enabled: mask.enabled,
        invert: mask.invert,
        opacity: mask.opacity.as_f64().unwrap_or(100.0),
        feather: mask.feather.as_f64().unwrap_or(0.0),
        geometry,
        exposure,
        contrast,
    })
}

fn render_brush_strokes_from_edit(
    brush: &silica_edit::MaskBrush,
) -> Vec<silica_render::BrushMaskRasterStroke> {
    brush
        .strokes
        .iter()
        .map(|stroke| silica_render::BrushMaskRasterStroke {
            id: stroke.id.clone(),
            radius: stroke.radius.as_f64().unwrap_or(0.0),
            points: stroke
                .points
                .iter()
                .map(|point| silica_render::BrushMaskRasterPoint {
                    x: point.x.as_f64().unwrap_or(0.0),
                    y: point.y.as_f64().unwrap_or(0.0),
                })
                .collect(),
        })
        .collect()
}

fn manual_mask_local_value(
    mask: &silica_edit::Mask,
    key: &str,
    min: f64,
    max: f64,
) -> Result<f64, CoreError> {
    let Some(value) = mask.local_adjustments.get(key) else {
        return Ok(0.0);
    };
    let value = value
        .as_f64()
        .ok_or_else(|| manual_mask_unsupported_message(format!("`{key}` must be finite")))?;
    if !(min..=max).contains(&value) {
        return Err(manual_mask_unsupported_message(format!(
            "`{key}` must be between {min} and {max}"
        )));
    }
    Ok(value)
}

pub(super) fn export_manual_masks_from_render(
    masks: &[silica_render::ManualMaskRenderAdjustment],
) -> Vec<silica_export::ManualMaskAdjustment> {
    masks
        .iter()
        .map(|mask| silica_export::ManualMaskAdjustment {
            id: mask.id.clone(),
            enabled: mask.enabled,
            invert: mask.invert,
            opacity: mask.opacity,
            feather: mask.feather,
            geometry: match &mask.geometry {
                silica_render::ManualMaskRenderGeometry::LinearGradient {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                } => silica_export::ManualMaskGeometry::LinearGradient {
                    start_x: *start_x,
                    start_y: *start_y,
                    end_x: *end_x,
                    end_y: *end_y,
                },
                silica_render::ManualMaskRenderGeometry::RadialGradient {
                    center_x,
                    center_y,
                    radius_x,
                    radius_y,
                    rotation,
                } => silica_export::ManualMaskGeometry::RadialGradient {
                    center_x: *center_x,
                    center_y: *center_y,
                    radius_x: *radius_x,
                    radius_y: *radius_y,
                    rotation: *rotation,
                },
                silica_render::ManualMaskRenderGeometry::BrushRaster {
                    width,
                    height,
                    alpha,
                    ..
                } => silica_export::ManualMaskGeometry::RasterAlphaPlane {
                    width: *width,
                    height: *height,
                    alpha: alpha.clone(),
                },
            },
            exposure: mask.exposure,
            contrast: mask.contrast,
        })
        .collect()
}

fn is_supported_quarter_turn(rotation: f64) -> bool {
    [0.0, 90.0, -90.0, 180.0, -180.0]
        .iter()
        .any(|supported| (rotation - supported).abs() <= f64::EPSILON)
}

pub(super) fn render_color_presence_from_graph(
    graph: &silica_edit::EditGraph,
) -> silica_render::ColorPresenceRenderAdjustment {
    silica_render::ColorPresenceRenderAdjustment {
        vibrance: graph.basic.vibrance.as_f64().unwrap_or(0.0),
        saturation: graph.basic.saturation.as_f64().unwrap_or(0.0),
    }
}

pub(super) fn export_color_presence_from_render(
    color_presence: silica_render::ColorPresenceRenderAdjustment,
) -> silica_export::ColorPresenceAdjustment {
    silica_export::ColorPresenceAdjustment {
        vibrance: color_presence.vibrance,
        saturation: color_presence.saturation,
    }
}

fn export_white_balance_mode(
    mode: silica_render::WhiteBalanceRenderMode,
) -> silica_export::WhiteBalanceMode {
    match mode {
        silica_render::WhiteBalanceRenderMode::AsShot => silica_export::WhiteBalanceMode::AsShot,
        silica_render::WhiteBalanceRenderMode::Auto => silica_export::WhiteBalanceMode::Auto,
        silica_render::WhiteBalanceRenderMode::Daylight => {
            silica_export::WhiteBalanceMode::Daylight
        }
        silica_render::WhiteBalanceRenderMode::Cloudy => silica_export::WhiteBalanceMode::Cloudy,
        silica_render::WhiteBalanceRenderMode::Shade => silica_export::WhiteBalanceMode::Shade,
        silica_render::WhiteBalanceRenderMode::Tungsten => {
            silica_export::WhiteBalanceMode::Tungsten
        }
        silica_render::WhiteBalanceRenderMode::Fluorescent => {
            silica_export::WhiteBalanceMode::Fluorescent
        }
        silica_render::WhiteBalanceRenderMode::Flash => silica_export::WhiteBalanceMode::Flash,
        silica_render::WhiteBalanceRenderMode::Custom => silica_export::WhiteBalanceMode::Custom,
    }
}

pub(super) fn white_balance_render_mode_string(
    mode: silica_render::WhiteBalanceRenderMode,
) -> &'static str {
    match mode {
        silica_render::WhiteBalanceRenderMode::AsShot => "as_shot",
        silica_render::WhiteBalanceRenderMode::Auto => "auto",
        silica_render::WhiteBalanceRenderMode::Daylight => "daylight",
        silica_render::WhiteBalanceRenderMode::Cloudy => "cloudy",
        silica_render::WhiteBalanceRenderMode::Shade => "shade",
        silica_render::WhiteBalanceRenderMode::Tungsten => "tungsten",
        silica_render::WhiteBalanceRenderMode::Fluorescent => "fluorescent",
        silica_render::WhiteBalanceRenderMode::Flash => "flash",
        silica_render::WhiteBalanceRenderMode::Custom => "custom",
    }
}

fn export_color_profile_message(profile: PhotoExportColorProfile) -> &'static str {
    match profile {
        PhotoExportColorProfile::Srgb => "JPEG sRGB export completed.",
        PhotoExportColorProfile::DisplayP3 => "JPEG Display P3 export completed.",
    }
}

pub(super) fn export_raster_message(
    format: PhotoExportFormat,
    color_profile: PhotoExportColorProfile,
) -> &'static str {
    match format {
        PhotoExportFormat::Jpeg => export_color_profile_message(color_profile),
        PhotoExportFormat::Png => "PNG sRGB export completed.",
        PhotoExportFormat::Tiff => "TIFF sRGB export completed.",
    }
}

pub(super) fn export_profile_metadata_source(format: PhotoExportFormat) -> &'static str {
    match format {
        PhotoExportFormat::Jpeg => "silica-export",
        PhotoExportFormat::Png | PhotoExportFormat::Tiff => "none",
    }
}
