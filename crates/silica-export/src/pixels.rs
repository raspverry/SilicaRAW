use crate::model::{
    ColorPresenceAdjustment, DetailAdjustment, ExportError, GeometryAdjustment,
    GeometryCropAdjustment, HslColorChannelAdjustment, HslColorMixerAdjustment,
    ManualMaskAdjustment, ManualMaskGeometry, ToneCurveAdjustment, ToneCurveMode, ToneCurvePoint,
    ToneRecoveryAdjustment, WhiteBalanceAdjustment,
};

pub(crate) fn apply_exposure_contrast(image: &mut image::RgbImage, exposure: f64, contrast: f64) {
    let exposure_scale = 2.0_f32.powf(exposure as f32);
    let contrast_scale = ((100.0 + contrast as f32) / 100.0).max(0.0);

    for pixel in image.pixels_mut() {
        for channel in &mut pixel.0 {
            let normalized = f32::from(*channel) / 255.0;
            let adjusted =
                ((normalized * exposure_scale - 0.5) * contrast_scale + 0.5).clamp(0.0, 1.0);
            *channel = (adjusted * 255.0).round() as u8;
        }
    }
}

pub(crate) fn apply_white_balance(
    image: &mut image::RgbImage,
    white_balance: WhiteBalanceAdjustment,
) {
    let warmth = ((white_balance.temperature - 5200.0) / 4800.0).clamp(-1.0, 1.0) as f32;
    let tint = (white_balance.tint / 150.0).clamp(-1.0, 1.0) as f32;
    let red_scale = (1.0 + warmth * 0.20 + tint * 0.04).clamp(0.25, 2.0);
    let green_scale = (1.0 + tint * 0.12).clamp(0.25, 2.0);
    let blue_scale = (1.0 - warmth * 0.20 - tint * 0.04).clamp(0.25, 2.0);

    for pixel in image.pixels_mut() {
        pixel.0[0] = scale_channel(pixel.0[0], red_scale);
        pixel.0[1] = scale_channel(pixel.0[1], green_scale);
        pixel.0[2] = scale_channel(pixel.0[2], blue_scale);
    }
}

fn scale_channel(channel: u8, scale: f32) -> u8 {
    (f32::from(channel) * scale).clamp(0.0, 255.0).round() as u8
}

pub(crate) fn apply_tone_recovery(
    image: &mut image::RgbImage,
    tone_recovery: ToneRecoveryAdjustment,
) {
    let highlights = (tone_recovery.highlights / 100.0).clamp(-1.0, 1.0) as f32;
    let shadows = (tone_recovery.shadows / 100.0).clamp(-1.0, 1.0) as f32;
    let whites = (tone_recovery.whites / 100.0).clamp(-1.0, 1.0) as f32;
    let blacks = (tone_recovery.blacks / 100.0).clamp(-1.0, 1.0) as f32;

    for pixel in image.pixels_mut() {
        for channel in &mut pixel.0 {
            let normalized = f32::from(*channel) / 255.0;
            let shadow_weight = (1.0 - normalized).powi(2);
            let highlight_weight = normalized.powi(2);
            let adjusted = (normalized
                + shadows * 0.22 * shadow_weight
                + blacks * 0.14 * shadow_weight
                + highlights * 0.22 * highlight_weight
                + whites * 0.14 * highlight_weight)
                .clamp(0.0, 1.0);
            *channel = (adjusted * 255.0).round() as u8;
        }
    }
}

pub(crate) fn apply_tone_curve(image: &mut image::RgbImage, tone_curve: &ToneCurveAdjustment) {
    if tone_curve.mode == ToneCurveMode::None {
        return;
    }

    for pixel in image.pixels_mut() {
        pixel.0[0] = apply_curve_channel(pixel.0[0], &tone_curve.rgb_curve, &tone_curve.red_curve);
        pixel.0[1] =
            apply_curve_channel(pixel.0[1], &tone_curve.rgb_curve, &tone_curve.green_curve);
        pixel.0[2] = apply_curve_channel(pixel.0[2], &tone_curve.rgb_curve, &tone_curve.blue_curve);
    }
}

fn apply_curve_channel(
    channel: u8,
    rgb_curve: &[ToneCurvePoint],
    channel_curve: &[ToneCurvePoint],
) -> u8 {
    let mut value = f32::from(channel) / 255.0;
    value = evaluate_curve(value, rgb_curve);
    value = evaluate_curve(value, channel_curve);
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn evaluate_curve(value: f32, curve: &[ToneCurvePoint]) -> f32 {
    if curve.is_empty() {
        return value;
    }
    if value <= curve[0].x as f32 {
        return curve[0].y as f32;
    }
    for window in curve.windows(2) {
        let start = window[0];
        let end = window[1];
        let start_x = start.x as f32;
        let end_x = end.x as f32;
        if value <= end_x {
            let span = end_x - start_x;
            if span <= f32::EPSILON {
                return end.y as f32;
            }
            let t = ((value - start_x) / span).clamp(0.0, 1.0);
            return (start.y as f32) + ((end.y - start.y) as f32) * t;
        }
    }
    curve.last().map(|point| point.y as f32).unwrap_or(value)
}

pub(crate) fn apply_color_presence(
    image: &mut image::RgbImage,
    color_presence: ColorPresenceAdjustment,
) {
    let vibrance = (color_presence.vibrance / 100.0).clamp(-1.0, 1.0) as f32;
    let saturation = (color_presence.saturation / 100.0).clamp(-1.0, 1.0) as f32;

    for pixel in image.pixels_mut() {
        let red = f32::from(pixel.0[0]) / 255.0;
        let green = f32::from(pixel.0[1]) / 255.0;
        let blue = f32::from(pixel.0[2]) / 255.0;
        let luma = red * 0.2126 + green * 0.7152 + blue * 0.0722;
        let max_channel = red.max(green).max(blue);
        let min_channel = red.min(green).min(blue);
        let chroma = max_channel - min_channel;
        let factor = (1.0 + saturation * 0.55 + vibrance * 0.45 * (1.0 - chroma)).clamp(0.0, 2.0);

        pixel.0[0] = ((luma + (red - luma) * factor).clamp(0.0, 1.0) * 255.0).round() as u8;
        pixel.0[1] = ((luma + (green - luma) * factor).clamp(0.0, 1.0) * 255.0).round() as u8;
        pixel.0[2] = ((luma + (blue - luma) * factor).clamp(0.0, 1.0) * 255.0).round() as u8;
    }
}

pub(crate) fn apply_manual_masks(image: &mut image::RgbImage, masks: &[ManualMaskAdjustment]) {
    if masks.is_empty() {
        return;
    }

    let width = image.width().max(1) as f32;
    let height = image.height().max(1) as f32;
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let normalized_x = if width <= 1.0 {
            0.0
        } else {
            x as f32 / (width - 1.0)
        };
        let normalized_y = if height <= 1.0 {
            0.0
        } else {
            y as f32 / (height - 1.0)
        };
        for mask in masks {
            if !mask.enabled {
                continue;
            }
            let mut weight = mask_weight(mask, normalized_x, normalized_y);
            if mask.invert {
                weight = 1.0 - weight;
            }
            weight = (weight * (mask.opacity as f32 / 100.0).clamp(0.0, 1.0)).clamp(0.0, 1.0);
            if weight <= 0.0 {
                continue;
            }
            let exposure_scale = 2.0_f32.powf((mask.exposure as f32) * weight);
            let contrast = (mask.contrast as f32) * weight;
            let contrast_scale = ((100.0 + contrast) / 100.0).max(0.0);
            for channel in &mut pixel.0 {
                let normalized = f32::from(*channel) / 255.0;
                let adjusted =
                    ((normalized * exposure_scale - 0.5) * contrast_scale + 0.5).clamp(0.0, 1.0);
                *channel = (adjusted * 255.0).round() as u8;
            }
        }
    }
}

fn mask_weight(mask: &ManualMaskAdjustment, x: f32, y: f32) -> f32 {
    match &mask.geometry {
        ManualMaskGeometry::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => {
            let start_x = *start_x as f32;
            let start_y = *start_y as f32;
            let vector_x = *end_x as f32 - start_x;
            let vector_y = *end_y as f32 - start_y;
            let length_squared = (vector_x * vector_x + vector_y * vector_y).max(f32::EPSILON);
            let projection = ((x - start_x) * vector_x + (y - start_y) * vector_y) / length_squared;
            smooth_mask_edge(projection.clamp(0.0, 1.0), mask.feather)
        }
        ManualMaskGeometry::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        } => {
            let angle = -(*rotation as f32).to_radians();
            let cos = angle.cos();
            let sin = angle.sin();
            let dx = x - *center_x as f32;
            let dy = y - *center_y as f32;
            let rotated_x = (dx * cos - dy * sin) / (*radius_x as f32).max(f32::EPSILON);
            let rotated_y = (dx * sin + dy * cos) / (*radius_y as f32).max(f32::EPSILON);
            let distance = (rotated_x * rotated_x + rotated_y * rotated_y).sqrt();
            smooth_mask_edge((1.0 - distance).clamp(0.0, 1.0), mask.feather)
        }
        ManualMaskGeometry::RasterAlphaPlane {
            width,
            height,
            alpha,
        } => {
            let sample_x = (x * width.saturating_sub(1) as f32)
                .round()
                .clamp(0.0, width.saturating_sub(1) as f32) as usize;
            let sample_y = (y * height.saturating_sub(1) as f32)
                .round()
                .clamp(0.0, height.saturating_sub(1) as f32) as usize;
            let index = sample_y * (*width as usize) + sample_x;
            f32::from(alpha[index]) / 255.0
        }
    }
}

fn smooth_mask_edge(weight: f32, feather: f64) -> f32 {
    let feather = (feather as f32 / 100.0).clamp(0.0, 1.0);
    if feather <= f32::EPSILON {
        return weight;
    }
    let lower = (0.5 - feather * 0.5).clamp(0.0, 1.0);
    let upper = (0.5 + feather * 0.5).clamp(0.0, 1.0);
    if weight <= lower {
        0.0
    } else if weight >= upper {
        1.0
    } else {
        ((weight - lower) / (upper - lower).max(f32::EPSILON)).clamp(0.0, 1.0)
    }
}

pub(crate) fn apply_hsl_color_mixer(
    image: &mut image::RgbImage,
    hsl_color_mixer: HslColorMixerAdjustment,
) {
    if hsl_color_mixer.is_neutral() {
        return;
    }

    for pixel in image.pixels_mut() {
        let (mut hue, mut saturation, mut luminance) = rgb_to_hsl(pixel.0);
        let mut hue_shift = 0.0_f32;
        let mut saturation_delta = 0.0_f32;
        let mut luminance_delta = 0.0_f32;

        for (center, channel) in hsl_channel_centers(hsl_color_mixer) {
            let weight = hsl_channel_weight(hue, center);
            if weight <= 0.0 {
                continue;
            }
            hue_shift += (channel.hue as f32 / 100.0) * 30.0 * weight;
            saturation_delta += (channel.saturation as f32 / 100.0) * 0.65 * weight;
            luminance_delta += (channel.luminance as f32 / 100.0) * 0.35 * weight;
        }

        hue = wrap_hue_degrees(hue + hue_shift);
        saturation = (saturation * (1.0 + saturation_delta)).clamp(0.0, 1.0);
        luminance = (luminance + luminance_delta).clamp(0.0, 1.0);
        pixel.0 = hsl_to_rgb(hue, saturation, luminance);
    }
}

pub(crate) fn apply_supported_geometry(
    mut image: image::RgbImage,
    geometry: &GeometryAdjustment,
) -> Result<image::RgbImage, ExportError> {
    validate_geometry_adjustment(geometry)?;

    if let Some(crop) = &geometry.crop {
        let (x, y, width, height) = normalized_crop_bounds(crop, image.width(), image.height())?;
        image = image::imageops::crop_imm(&image, x, y, width, height).to_image();
    }
    if geometry.flip_horizontal {
        image = image::imageops::flip_horizontal(&image);
    }
    if geometry.flip_vertical {
        image = image::imageops::flip_vertical(&image);
    }
    image = match normalized_quarter_turn(geometry.rotation)? {
        0 => image,
        90 => image::imageops::rotate90(&image),
        180 | -180 => image::imageops::rotate180(&image),
        -90 => image::imageops::rotate270(&image),
        _ => unreachable!("validated quarter turn"),
    };
    Ok(image)
}

fn normalized_crop_bounds(
    crop: &GeometryCropAdjustment,
    image_width: u32,
    image_height: u32,
) -> Result<(u32, u32, u32, u32), ExportError> {
    if image_width == 0 || image_height == 0 {
        return Err(ExportError::UnsupportedGeometryAdjustment(
            "Geometry crop requires a non-empty raster source".to_string(),
        ));
    }
    let x = crop.x;
    let y = crop.y;
    let width = crop.width;
    let height = crop.height;
    if !(0.0..=1.0).contains(&x)
        || !(0.0..=1.0).contains(&y)
        || !(0.0..=1.0).contains(&width)
        || !(0.0..=1.0).contains(&height)
        || width <= 0.0
        || height <= 0.0
        || x + width > 1.0
        || y + height > 1.0
    {
        return Err(ExportError::UnsupportedGeometryAdjustment(
            "Geometry crop must stay within the normalized source bounds".to_string(),
        ));
    }

    let x_px = ((x * f64::from(image_width)).floor() as u32).min(image_width - 1);
    let y_px = ((y * f64::from(image_height)).floor() as u32).min(image_height - 1);
    let width_px = ((width * f64::from(image_width)).round() as u32)
        .max(1)
        .min(image_width - x_px);
    let height_px = ((height * f64::from(image_height)).round() as u32)
        .max(1)
        .min(image_height - y_px);

    Ok((x_px, y_px, width_px, height_px))
}

fn hsl_channel_centers(
    hsl_color_mixer: HslColorMixerAdjustment,
) -> [(f32, HslColorChannelAdjustment); 8] {
    [
        (0.0, hsl_color_mixer.red),
        (30.0, hsl_color_mixer.orange),
        (60.0, hsl_color_mixer.yellow),
        (120.0, hsl_color_mixer.green),
        (180.0, hsl_color_mixer.aqua),
        (240.0, hsl_color_mixer.blue),
        (280.0, hsl_color_mixer.purple),
        (320.0, hsl_color_mixer.magenta),
    ]
}

fn hsl_channel_weight(hue: f32, center: f32) -> f32 {
    let distance = hue_distance_degrees(hue, center);
    if distance >= 45.0 {
        0.0
    } else {
        1.0 - distance / 45.0
    }
}

fn hue_distance_degrees(a: f32, b: f32) -> f32 {
    let distance = (a - b).abs().rem_euclid(360.0);
    distance.min(360.0 - distance)
}

fn wrap_hue_degrees(hue: f32) -> f32 {
    hue.rem_euclid(360.0)
}

fn rgb_to_hsl(rgb: [u8; 3]) -> (f32, f32, f32) {
    let red = f32::from(rgb[0]) / 255.0;
    let green = f32::from(rgb[1]) / 255.0;
    let blue = f32::from(rgb[2]) / 255.0;
    let max_channel = red.max(green).max(blue);
    let min_channel = red.min(green).min(blue);
    let luminance = (max_channel + min_channel) / 2.0;
    let delta = max_channel - min_channel;

    if delta <= f32::EPSILON {
        return (0.0, 0.0, luminance);
    }

    let saturation = delta / (1.0 - (2.0 * luminance - 1.0).abs());
    let hue = if max_channel == red {
        60.0 * ((green - blue) / delta).rem_euclid(6.0)
    } else if max_channel == green {
        60.0 * (((blue - red) / delta) + 2.0)
    } else {
        60.0 * (((red - green) / delta) + 4.0)
    };

    (wrap_hue_degrees(hue), saturation.clamp(0.0, 1.0), luminance)
}

fn hsl_to_rgb(hue: f32, saturation: f32, luminance: f32) -> [u8; 3] {
    let chroma = (1.0 - (2.0 * luminance - 1.0).abs()) * saturation;
    let hue_prime = hue / 60.0;
    let x = chroma * (1.0 - (hue_prime.rem_euclid(2.0) - 1.0).abs());
    let (red1, green1, blue1) = if hue_prime < 1.0 {
        (chroma, x, 0.0)
    } else if hue_prime < 2.0 {
        (x, chroma, 0.0)
    } else if hue_prime < 3.0 {
        (0.0, chroma, x)
    } else if hue_prime < 4.0 {
        (0.0, x, chroma)
    } else if hue_prime < 5.0 {
        (x, 0.0, chroma)
    } else {
        (chroma, 0.0, x)
    };
    let match_value = luminance - chroma / 2.0;

    [
        float_channel_to_u8(red1 + match_value),
        float_channel_to_u8(green1 + match_value),
        float_channel_to_u8(blue1 + match_value),
    ]
}

fn float_channel_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub(crate) fn adjustments_are_finite(
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceAdjustment,
    tone_recovery: ToneRecoveryAdjustment,
    color_presence: ColorPresenceAdjustment,
    tone_curve: &ToneCurveAdjustment,
    hsl_color_mixer: HslColorMixerAdjustment,
    detail: DetailAdjustment,
    geometry: &GeometryAdjustment,
) -> bool {
    exposure.is_finite()
        && contrast.is_finite()
        && white_balance.temperature.is_finite()
        && white_balance.tint.is_finite()
        && tone_recovery.highlights.is_finite()
        && tone_recovery.shadows.is_finite()
        && tone_recovery.whites.is_finite()
        && tone_recovery.blacks.is_finite()
        && color_presence.vibrance.is_finite()
        && color_presence.saturation.is_finite()
        && tone_curve_points_are_finite(&tone_curve.rgb_curve)
        && tone_curve_points_are_finite(&tone_curve.red_curve)
        && tone_curve_points_are_finite(&tone_curve.green_curve)
        && tone_curve_points_are_finite(&tone_curve.blue_curve)
        && hsl_color_mixer_is_finite(hsl_color_mixer)
        && detail_is_finite(detail)
        && geometry_is_finite(geometry)
}

fn tone_curve_points_are_finite(points: &[ToneCurvePoint]) -> bool {
    points
        .iter()
        .all(|point| point.x.is_finite() && point.y.is_finite())
}

fn hsl_color_mixer_is_finite(hsl_color_mixer: HslColorMixerAdjustment) -> bool {
    hsl_channel_centers(hsl_color_mixer)
        .iter()
        .all(|(_, channel)| {
            channel.hue.is_finite()
                && channel.saturation.is_finite()
                && channel.luminance.is_finite()
        })
}

fn detail_is_finite(detail: DetailAdjustment) -> bool {
    detail.sharpening.amount.is_finite()
        && detail.sharpening.radius.is_finite()
        && detail.sharpening.detail.is_finite()
        && detail.sharpening.masking.is_finite()
        && detail.noise_reduction.luminance.is_finite()
        && detail.noise_reduction.detail.is_finite()
        && detail.noise_reduction.contrast.is_finite()
        && detail.noise_reduction.color.is_finite()
        && detail.noise_reduction.color_detail.is_finite()
}

fn geometry_is_finite(geometry: &GeometryAdjustment) -> bool {
    geometry.rotation.is_finite()
        && geometry.transform.vertical.is_finite()
        && geometry.transform.horizontal.is_finite()
        && geometry.transform.aspect.is_finite()
        && geometry.transform.scale.is_finite()
        && geometry.transform.x_offset.is_finite()
        && geometry.transform.y_offset.is_finite()
        && geometry.crop.as_ref().map_or(true, |crop| {
            crop.x.is_finite()
                && crop.y.is_finite()
                && crop.width.is_finite()
                && crop.height.is_finite()
                && crop.angle.is_finite()
        })
}

pub(crate) fn validate_tone_curve_adjustment(
    tone_curve: &ToneCurveAdjustment,
) -> Result<(), ExportError> {
    match tone_curve.mode {
        ToneCurveMode::None => {
            if tone_curve.rgb_curve.is_empty()
                && tone_curve.red_curve.is_empty()
                && tone_curve.green_curve.is_empty()
                && tone_curve.blue_curve.is_empty()
            {
                Ok(())
            } else {
                Err(ExportError::InvalidToneCurveAdjustment(
                    "none mode must not carry curve points".to_string(),
                ))
            }
        }
        ToneCurveMode::Parametric => Err(ExportError::InvalidToneCurveAdjustment(
            "parametric curves have no schema-owned parameters yet".to_string(),
        )),
        ToneCurveMode::Point => {
            validate_tone_curve_points("rgb_curve", &tone_curve.rgb_curve)?;
            validate_tone_curve_points("red_curve", &tone_curve.red_curve)?;
            validate_tone_curve_points("green_curve", &tone_curve.green_curve)?;
            validate_tone_curve_points("blue_curve", &tone_curve.blue_curve)
        }
    }
}

fn validate_tone_curve_points(path: &str, points: &[ToneCurvePoint]) -> Result<(), ExportError> {
    if points.is_empty() {
        return Ok(());
    }
    if points.len() < 2 {
        return Err(ExportError::InvalidToneCurveAdjustment(format!(
            "{path} must include endpoints"
        )));
    }
    for (index, point) in points.iter().enumerate() {
        if !(0.0..=1.0).contains(&point.x) || !(0.0..=1.0).contains(&point.y) {
            return Err(ExportError::InvalidToneCurveAdjustment(format!(
                "{path}.{index} must be between 0 and 1"
            )));
        }
        if index > 0 && point.x <= points[index - 1].x {
            return Err(ExportError::InvalidToneCurveAdjustment(format!(
                "{path}.{index}.x must be strictly increasing"
            )));
        }
    }
    let first = points.first().expect("non-empty curve checked");
    let last = points.last().expect("non-empty curve checked");
    if first.x != 0.0 || first.y != 0.0 || last.x != 1.0 || last.y != 1.0 {
        return Err(ExportError::InvalidToneCurveAdjustment(format!(
            "{path} must start at (0, 0) and end at (1, 1)"
        )));
    }
    Ok(())
}

pub(crate) fn validate_hsl_color_mixer_adjustment(
    hsl_color_mixer: HslColorMixerAdjustment,
) -> Result<(), ExportError> {
    for (name, channel) in [
        ("red", hsl_color_mixer.red),
        ("orange", hsl_color_mixer.orange),
        ("yellow", hsl_color_mixer.yellow),
        ("green", hsl_color_mixer.green),
        ("aqua", hsl_color_mixer.aqua),
        ("blue", hsl_color_mixer.blue),
        ("purple", hsl_color_mixer.purple),
        ("magenta", hsl_color_mixer.magenta),
    ] {
        validate_hsl_channel_adjustment(name, channel)?;
    }
    Ok(())
}

pub(crate) fn validate_detail_adjustment(detail: DetailAdjustment) -> Result<(), ExportError> {
    if detail.is_neutral() {
        Ok(())
    } else {
        Err(ExportError::UnsupportedDetailAdjustment(
            "Detail preview/export is unsupported until renderer support exists".to_string(),
        ))
    }
}

pub(crate) fn validate_geometry_adjustment(
    geometry: &GeometryAdjustment,
) -> Result<(), ExportError> {
    if !geometry.transform.is_neutral() {
        return Err(ExportError::UnsupportedGeometryAdjustment(
            "Geometry transform preview/export is unsupported until renderer support exists"
                .to_string(),
        ));
    }
    if let Some(crop) = &geometry.crop {
        if crop.angle != 0.0 {
            return Err(ExportError::UnsupportedGeometryAdjustment(
                "Angled crop preview/export is unsupported until renderer support exists"
                    .to_string(),
            ));
        }
        normalized_crop_bounds(crop, 1, 1)?;
    }
    normalized_quarter_turn(geometry.rotation)?;
    Ok(())
}

pub(crate) fn validate_manual_mask_adjustments(
    masks: &[ManualMaskAdjustment],
) -> Result<(), ExportError> {
    for mask in masks {
        if mask.id.trim().is_empty() {
            return Err(ExportError::InvalidManualMaskAdjustment(
                "id must not be empty".to_string(),
            ));
        }
        if !mask.opacity.is_finite() || !(0.0..=100.0).contains(&mask.opacity) {
            return Err(ExportError::InvalidManualMaskAdjustment(format!(
                "{} opacity must be finite and between 0 and 100",
                mask.id
            )));
        }
        if !mask.feather.is_finite() || !(0.0..=100.0).contains(&mask.feather) {
            return Err(ExportError::InvalidManualMaskAdjustment(format!(
                "{} feather must be finite and between 0 and 100",
                mask.id
            )));
        }
        if !mask.exposure.is_finite() || !mask.contrast.is_finite() {
            return Err(ExportError::InvalidManualMaskAdjustment(format!(
                "{} local adjustments must be finite",
                mask.id
            )));
        }
        validate_manual_mask_geometry(mask)?;
    }
    Ok(())
}

fn validate_manual_mask_geometry(mask: &ManualMaskAdjustment) -> Result<(), ExportError> {
    match &mask.geometry {
        ManualMaskGeometry::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => {
            for (path, value) in [
                ("start_x", start_x),
                ("start_y", start_y),
                ("end_x", end_x),
                ("end_y", end_y),
            ] {
                if !value.is_finite() || !(0.0..=1.0).contains(value) {
                    return Err(ExportError::InvalidManualMaskAdjustment(format!(
                        "{} geometry.{path} must be finite and between 0 and 1",
                        mask.id
                    )));
                }
            }
            if start_x == end_x && start_y == end_y {
                return Err(ExportError::InvalidManualMaskAdjustment(format!(
                    "{} linear gradient endpoints must differ",
                    mask.id
                )));
            }
        }
        ManualMaskGeometry::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        } => {
            for (path, value) in [("center_x", center_x), ("center_y", center_y)] {
                if !value.is_finite() || !(0.0..=1.0).contains(value) {
                    return Err(ExportError::InvalidManualMaskAdjustment(format!(
                        "{} geometry.{path} must be finite and between 0 and 1",
                        mask.id
                    )));
                }
            }
            for (path, value) in [("radius_x", radius_x), ("radius_y", radius_y)] {
                if !value.is_finite() || *value <= 0.0 || *value > 1.0 {
                    return Err(ExportError::InvalidManualMaskAdjustment(format!(
                        "{} geometry.{path} must be finite and in the range (0, 1]",
                        mask.id
                    )));
                }
            }
            if !rotation.is_finite() || !(-180.0..=180.0).contains(rotation) {
                return Err(ExportError::InvalidManualMaskAdjustment(format!(
                    "{} geometry.rotation must be finite and between -180 and 180",
                    mask.id
                )));
            }
        }
        ManualMaskGeometry::RasterAlphaPlane {
            width,
            height,
            alpha,
        } => {
            if *width == 0 || *height == 0 {
                return Err(ExportError::InvalidManualMaskAdjustment(format!(
                    "{} raster alpha plane dimensions must be greater than zero",
                    mask.id
                )));
            }
            let expected_len = (*width as usize) * (*height as usize);
            if alpha.len() != expected_len {
                return Err(ExportError::InvalidManualMaskAdjustment(format!(
                    "{} raster alpha plane length must equal width * height",
                    mask.id
                )));
            }
        }
    }
    Ok(())
}

fn normalized_quarter_turn(rotation: f64) -> Result<i16, ExportError> {
    for supported in [0_i16, 90, -90, 180, -180] {
        if (rotation - f64::from(supported)).abs() <= f64::EPSILON {
            return Ok(supported);
        }
    }
    Err(ExportError::UnsupportedGeometryAdjustment(
        "Arbitrary rotation preview/export is unsupported until renderer support exists"
            .to_string(),
    ))
}

fn validate_hsl_channel_adjustment(
    name: &str,
    channel: HslColorChannelAdjustment,
) -> Result<(), ExportError> {
    for (field, value) in [
        ("hue", channel.hue),
        ("saturation", channel.saturation),
        ("luminance", channel.luminance),
    ] {
        if !(-100.0..=100.0).contains(&value) {
            return Err(ExportError::InvalidHslColorMixerAdjustment(format!(
                "{name}.{field} must be between -100 and 100"
            )));
        }
    }
    Ok(())
}
