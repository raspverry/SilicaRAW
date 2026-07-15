use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopToneCurveState {
    pub(crate) curve_mode: &'static str,
    pub(crate) rgb_curve: Vec<DesktopToneCurvePoint>,
    pub(crate) red_curve: Vec<DesktopToneCurvePoint>,
    pub(crate) green_curve: Vec<DesktopToneCurvePoint>,
    pub(crate) blue_curve: Vec<DesktopToneCurvePoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopHslColorChannelState {
    pub(crate) hue: f64,
    pub(crate) saturation: f64,
    pub(crate) luminance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopHslColorMixerState {
    pub(crate) red: DesktopHslColorChannelState,
    pub(crate) orange: DesktopHslColorChannelState,
    pub(crate) yellow: DesktopHslColorChannelState,
    pub(crate) green: DesktopHslColorChannelState,
    pub(crate) aqua: DesktopHslColorChannelState,
    pub(crate) blue: DesktopHslColorChannelState,
    pub(crate) purple: DesktopHslColorChannelState,
    pub(crate) magenta: DesktopHslColorChannelState,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopDetailSharpeningState {
    pub(crate) amount: f64,
    pub(crate) radius: f64,
    pub(crate) detail: f64,
    pub(crate) masking: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopDetailNoiseReductionState {
    pub(crate) luminance: f64,
    pub(crate) detail: f64,
    pub(crate) contrast: f64,
    pub(crate) color: f64,
    pub(crate) color_detail: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopDetailState {
    pub(crate) sharpening: DesktopDetailSharpeningState,
    pub(crate) noise_reduction: DesktopDetailNoiseReductionState,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopGeometryCropState {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) angle: f64,
    pub(crate) aspect: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopGeometryTransformState {
    pub(crate) vertical: f64,
    pub(crate) horizontal: f64,
    pub(crate) aspect: f64,
    pub(crate) scale: f64,
    pub(crate) x_offset: f64,
    pub(crate) y_offset: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopGeometryState {
    pub(crate) crop: Option<DesktopGeometryCropState>,
    pub(crate) rotation: f64,
    pub(crate) flip_horizontal: bool,
    pub(crate) flip_vertical: bool,
    pub(crate) transform: DesktopGeometryTransformState,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    tag = "kind",
    rename_all_fields = "camelCase"
)]
pub(crate) enum DesktopManualMaskGeometryState {
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

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopManualMaskState {
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) invert: bool,
    pub(crate) opacity: f64,
    pub(crate) feather: f64,
    pub(crate) geometry: Option<DesktopManualMaskGeometryState>,
    pub(crate) exposure: f64,
    pub(crate) contrast: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopToneCurvePoint {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

pub(crate) fn parse_white_balance(
    white_balance: &str,
) -> Result<silica_core::WhiteBalance, silica_core::CoreError> {
    match white_balance {
        "as_shot" => Ok(silica_core::WhiteBalance::AsShot),
        "auto" => Ok(silica_core::WhiteBalance::Auto),
        "daylight" => Ok(silica_core::WhiteBalance::Daylight),
        "cloudy" => Ok(silica_core::WhiteBalance::Cloudy),
        "shade" => Ok(silica_core::WhiteBalance::Shade),
        "tungsten" => Ok(silica_core::WhiteBalance::Tungsten),
        "fluorescent" => Ok(silica_core::WhiteBalance::Fluorescent),
        "flash" => Ok(silica_core::WhiteBalance::Flash),
        "custom" => Ok(silica_core::WhiteBalance::Custom),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported white balance mode: {unsupported}."
        ))),
    }
}

pub(crate) fn parse_basic_preset(
    preset: &str,
) -> Result<silica_core::BasicPreset, silica_core::CoreError> {
    match preset {
        "silica_neutral" => Ok(silica_core::BasicPreset::SilicaNeutral),
        "warm_contrast" => Ok(silica_core::BasicPreset::WarmContrast),
        "soft_matte" => Ok(silica_core::BasicPreset::SoftMatte),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported basic preset: {unsupported}."
        ))),
    }
}

pub(crate) fn white_balance_text(white_balance: silica_core::WhiteBalance) -> &'static str {
    match white_balance {
        silica_core::WhiteBalance::AsShot => "as_shot",
        silica_core::WhiteBalance::Auto => "auto",
        silica_core::WhiteBalance::Daylight => "daylight",
        silica_core::WhiteBalance::Cloudy => "cloudy",
        silica_core::WhiteBalance::Shade => "shade",
        silica_core::WhiteBalance::Tungsten => "tungsten",
        silica_core::WhiteBalance::Fluorescent => "fluorescent",
        silica_core::WhiteBalance::Flash => "flash",
        silica_core::WhiteBalance::Custom => "custom",
    }
}

pub(crate) fn tone_curve_data(
    tone_curve: silica_core::PhotoToneCurveState,
) -> DesktopToneCurveState {
    DesktopToneCurveState {
        curve_mode: curve_mode_text(tone_curve.curve_mode),
        rgb_curve: tone_curve_points_data(tone_curve.rgb_curve),
        red_curve: tone_curve_points_data(tone_curve.red_curve),
        green_curve: tone_curve_points_data(tone_curve.green_curve),
        blue_curve: tone_curve_points_data(tone_curve.blue_curve),
    }
}

pub(crate) fn curve_mode_text(curve_mode: silica_core::CurveMode) -> &'static str {
    match curve_mode {
        silica_core::CurveMode::None => "none",
        silica_core::CurveMode::Parametric => "parametric",
        silica_core::CurveMode::Point => "point",
    }
}

pub(crate) fn tone_curve_points_data(
    points: Vec<silica_core::PhotoToneCurvePoint>,
) -> Vec<DesktopToneCurvePoint> {
    points
        .into_iter()
        .map(|point| DesktopToneCurvePoint {
            x: point.x,
            y: point.y,
        })
        .collect()
}

pub(crate) fn tone_curve_pairs(points: &[DesktopToneCurvePoint]) -> Vec<(f64, f64)> {
    points.iter().map(|point| (point.x, point.y)).collect()
}

pub(crate) fn hsl_color_mixer_data(
    hsl_color_mixer: silica_core::PhotoHslColorMixerState,
) -> DesktopHslColorMixerState {
    DesktopHslColorMixerState {
        red: hsl_color_channel_data(hsl_color_mixer.red),
        orange: hsl_color_channel_data(hsl_color_mixer.orange),
        yellow: hsl_color_channel_data(hsl_color_mixer.yellow),
        green: hsl_color_channel_data(hsl_color_mixer.green),
        aqua: hsl_color_channel_data(hsl_color_mixer.aqua),
        blue: hsl_color_channel_data(hsl_color_mixer.blue),
        purple: hsl_color_channel_data(hsl_color_mixer.purple),
        magenta: hsl_color_channel_data(hsl_color_mixer.magenta),
    }
}

pub(crate) fn hsl_color_channel_data(
    channel: silica_core::PhotoHslColorChannelState,
) -> DesktopHslColorChannelState {
    DesktopHslColorChannelState {
        hue: channel.hue,
        saturation: channel.saturation,
        luminance: channel.luminance,
    }
}

pub(crate) fn detail_data(detail: silica_core::PhotoDetailState) -> DesktopDetailState {
    DesktopDetailState {
        sharpening: DesktopDetailSharpeningState {
            amount: detail.sharpening.amount,
            radius: detail.sharpening.radius,
            detail: detail.sharpening.detail,
            masking: detail.sharpening.masking,
        },
        noise_reduction: DesktopDetailNoiseReductionState {
            luminance: detail.noise_reduction.luminance,
            detail: detail.noise_reduction.detail,
            contrast: detail.noise_reduction.contrast,
            color: detail.noise_reduction.color,
            color_detail: detail.noise_reduction.color_detail,
        },
    }
}

pub(crate) fn geometry_data(geometry: silica_core::PhotoGeometryState) -> DesktopGeometryState {
    DesktopGeometryState {
        crop: geometry.crop.map(|crop| DesktopGeometryCropState {
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
        transform: DesktopGeometryTransformState {
            vertical: geometry.transform.vertical,
            horizontal: geometry.transform.horizontal,
            aspect: geometry.transform.aspect,
            scale: geometry.transform.scale,
            x_offset: geometry.transform.x_offset,
            y_offset: geometry.transform.y_offset,
        },
    }
}

pub(crate) fn manual_mask_geometry_data(
    geometry: Option<silica_core::PhotoManualMaskGeometryState>,
) -> Option<DesktopManualMaskGeometryState> {
    geometry.map(|geometry| match geometry {
        silica_core::PhotoManualMaskGeometryState::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => DesktopManualMaskGeometryState::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        },
        silica_core::PhotoManualMaskGeometryState::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        } => DesktopManualMaskGeometryState::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        },
    })
}

pub(crate) fn manual_mask_data(
    masks: Vec<silica_core::PhotoManualMaskState>,
) -> Vec<DesktopManualMaskState> {
    masks
        .into_iter()
        .map(|mask| DesktopManualMaskState {
            id: mask.id,
            kind: mask.kind,
            name: mask.name,
            enabled: mask.enabled,
            invert: mask.invert,
            opacity: mask.opacity,
            feather: mask.feather,
            geometry: manual_mask_geometry_data(mask.geometry),
            exposure: mask.exposure,
            contrast: mask.contrast,
        })
        .collect()
}

pub(crate) fn parse_hsl_color_channel(
    channel: &str,
) -> Result<silica_core::HslColorChannel, silica_core::CoreError> {
    silica_core::HslColorChannel::try_from(channel).map_err(silica_core::CoreError::from)
}
