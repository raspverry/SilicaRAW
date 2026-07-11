use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use image::{ImageEncoder, ImageFormat};

use crate::metadata::{
    export_icc_profile, insert_jpeg_metadata_segments, inspect_jpeg_icc_profile,
    prepare_jpeg_source_metadata, sha256_file,
};
use crate::model::{
    ExportColorProfile, ExportError, ExportImageFormat, ExportMetadataPolicy,
    JpegColorExportRequest, JpegDevelopPreviewRequest, JpegExportResult, JpegHistogramRequest,
    JpegSrgbExportRequest, JpegSrgbExportResult, JpegThumbnailRequest, JpegThumbnailResult,
    RasterDimensions, RasterExportResult, RasterSrgbExportRequest,
};
use crate::pixels::{
    adjustments_are_finite, apply_color_presence, apply_exposure_contrast, apply_hsl_color_mixer,
    apply_manual_masks, apply_supported_geometry, apply_tone_curve, apply_tone_recovery,
    apply_white_balance, validate_detail_adjustment, validate_geometry_adjustment,
    validate_hsl_color_mixer_adjustment, validate_manual_mask_adjustments,
    validate_tone_curve_adjustment,
};

/// Export an already-rendered raster source as a separate JPEG sRGB file.
pub fn export_jpeg_srgb(
    request: JpegSrgbExportRequest,
) -> Result<JpegSrgbExportResult, ExportError> {
    export_jpeg_with_color_profile(JpegColorExportRequest {
        source_path: request.source_path,
        output_path: request.output_path,
        exposure: request.exposure,
        contrast: request.contrast,
        white_balance: request.white_balance,
        tone_recovery: request.tone_recovery,
        color_presence: request.color_presence,
        tone_curve: request.tone_curve,
        hsl_color_mixer: request.hsl_color_mixer,
        detail: request.detail,
        geometry: request.geometry,
        masks: request.masks,
        quality: request.quality,
        color_profile: ExportColorProfile::Srgb,
    })
}

/// Export an already-rendered raster source as a separate JPEG with an explicit ICC profile.
pub fn export_jpeg_with_color_profile(
    request: JpegColorExportRequest,
) -> Result<JpegExportResult, ExportError> {
    export_jpeg_with_metadata_policy(request, ExportMetadataPolicy::Minimal)
}

/// Export an already-rendered raster source as JPEG with explicit metadata policy.
pub fn export_jpeg_with_metadata_policy(
    request: JpegColorExportRequest,
    metadata_policy: ExportMetadataPolicy,
) -> Result<JpegExportResult, ExportError> {
    if paths_match(&request.source_path, &request.output_path)? {
        return Err(ExportError::SameSourceAndOutput(request.output_path));
    }
    if !(1..=100).contains(&request.quality) {
        return Err(ExportError::InvalidQuality(request.quality));
    }
    if !adjustments_are_finite(
        request.exposure,
        request.contrast,
        request.white_balance,
        request.tone_recovery,
        request.color_presence,
        &request.tone_curve,
        request.hsl_color_mixer,
        request.detail,
        &request.geometry,
    ) {
        return Err(ExportError::NonFiniteAdjustment);
    }
    validate_tone_curve_adjustment(&request.tone_curve)?;
    validate_hsl_color_mixer_adjustment(request.hsl_color_mixer)?;
    validate_detail_adjustment(request.detail)?;
    validate_geometry_adjustment(&request.geometry)?;
    validate_manual_mask_adjustments(&request.masks)?;

    let source_sha256 = sha256_file(&request.source_path)?;
    let source_metadata = prepare_jpeg_source_metadata(&request.source_path, metadata_policy)?;
    let icc_profile = export_icc_profile(request.color_profile)?;
    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let mut rgb = decoded.to_rgb8();
    apply_exposure_contrast(&mut rgb, request.exposure, request.contrast);
    apply_white_balance(&mut rgb, request.white_balance);
    apply_tone_recovery(&mut rgb, request.tone_recovery);
    apply_tone_curve(&mut rgb, &request.tone_curve);
    apply_color_presence(&mut rgb, request.color_presence);
    apply_hsl_color_mixer(&mut rgb, request.hsl_color_mixer);
    apply_manual_masks(&mut rgb, &request.masks);
    let rgb = apply_supported_geometry(rgb, &request.geometry)?;

    let mut output = File::create(&request.output_path)?;
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, request.quality);
    encoder
        .set_icc_profile(icc_profile)
        .map_err(image::ImageError::Unsupported)?;
    encoder.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    drop(output);

    insert_jpeg_metadata_segments(&request.output_path, &source_metadata.output_segments)?;
    let output_sha256 = sha256_file(&request.output_path)?;
    let inspection = inspect_jpeg_icc_profile(&request.output_path)?;
    if inspection.color_profile != Some(request.color_profile) {
        return Err(ExportError::InvalidJpegIccProfile(request.output_path));
    }
    let icc_profile_sha256 = inspection
        .icc_profile_sha256
        .ok_or_else(|| ExportError::InvalidJpegIccProfile(request.output_path.clone()))?;

    Ok(JpegExportResult {
        bytes_written: fs::metadata(&request.output_path)?.len(),
        output_path: request.output_path,
        format: ExportImageFormat::Jpeg,
        color_profile: request.color_profile,
        metadata_policy,
        source_sha256,
        output_sha256,
        icc_profile_embedded: inspection.embedded,
        icc_profile_sha256,
        source_metadata_segments: source_metadata.source_segment_count,
        output_metadata_segments: source_metadata.output_segments.len(),
        source_metadata_copied: !source_metadata.output_segments.is_empty(),
        gps_metadata_removed: source_metadata.gps_removed,
    })
}

/// Export an already-rendered raster source as a separate sRGB raster file.
pub fn export_raster_srgb(
    request: RasterSrgbExportRequest,
) -> Result<RasterExportResult, ExportError> {
    if request.format == ExportImageFormat::Jpeg {
        let result = export_jpeg_srgb(JpegSrgbExportRequest {
            source_path: request.source_path,
            output_path: request.output_path,
            exposure: request.exposure,
            contrast: request.contrast,
            white_balance: request.white_balance,
            tone_recovery: request.tone_recovery,
            color_presence: request.color_presence,
            tone_curve: request.tone_curve,
            hsl_color_mixer: request.hsl_color_mixer,
            detail: request.detail,
            geometry: request.geometry,
            masks: request.masks,
            quality: 90,
        })?;
        return Ok(RasterExportResult {
            output_path: result.output_path,
            format: result.format,
            color_profile: result.color_profile,
            bytes_written: result.bytes_written,
            source_sha256: result.source_sha256,
            output_sha256: result.output_sha256,
            icc_profile_embedded: result.icc_profile_embedded,
            icc_profile_sha256: Some(result.icc_profile_sha256),
        });
    }

    if paths_match(&request.source_path, &request.output_path)? {
        return Err(ExportError::SameSourceAndOutput(request.output_path));
    }
    if !adjustments_are_finite(
        request.exposure,
        request.contrast,
        request.white_balance,
        request.tone_recovery,
        request.color_presence,
        &request.tone_curve,
        request.hsl_color_mixer,
        request.detail,
        &request.geometry,
    ) {
        return Err(ExportError::NonFiniteAdjustment);
    }
    validate_tone_curve_adjustment(&request.tone_curve)?;
    validate_hsl_color_mixer_adjustment(request.hsl_color_mixer)?;
    validate_detail_adjustment(request.detail)?;
    validate_geometry_adjustment(&request.geometry)?;
    validate_manual_mask_adjustments(&request.masks)?;

    let source_sha256 = sha256_file(&request.source_path)?;
    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let mut rgb = decoded.to_rgb8();
    apply_exposure_contrast(&mut rgb, request.exposure, request.contrast);
    apply_white_balance(&mut rgb, request.white_balance);
    apply_tone_recovery(&mut rgb, request.tone_recovery);
    apply_tone_curve(&mut rgb, &request.tone_curve);
    apply_color_presence(&mut rgb, request.color_presence);
    apply_hsl_color_mixer(&mut rgb, request.hsl_color_mixer);
    apply_manual_masks(&mut rgb, &request.masks);
    let rgb = apply_supported_geometry(rgb, &request.geometry)?;

    let image_format = match request.format {
        ExportImageFormat::Png => ImageFormat::Png,
        ExportImageFormat::Tiff => ImageFormat::Tiff,
        ExportImageFormat::Jpeg => unreachable!("JPEG raster export returns above"),
    };
    image::DynamicImage::ImageRgb8(rgb).save_with_format(&request.output_path, image_format)?;

    let output_sha256 = sha256_file(&request.output_path)?;
    Ok(RasterExportResult {
        bytes_written: fs::metadata(&request.output_path)?.len(),
        output_path: request.output_path,
        format: request.format,
        color_profile: ExportColorProfile::Srgb,
        source_sha256,
        output_sha256,
        icc_profile_embedded: false,
        icc_profile_sha256: None,
    })
}

/// Read raster dimensions through the existing image path.
pub fn read_raster_dimensions(path: PathBuf) -> Result<RasterDimensions, ExportError> {
    let (width, height) = image::image_dimensions(path)?;
    Ok(RasterDimensions { width, height })
}

/// Write a disposable JPEG thumbnail for a raster source.
pub fn write_jpeg_thumbnail(
    request: JpegThumbnailRequest,
) -> Result<JpegThumbnailResult, ExportError> {
    if paths_match(&request.source_path, &request.output_path)? {
        return Err(ExportError::SameSourceAndOutput(request.output_path));
    }
    if !(1..=100).contains(&request.quality) {
        return Err(ExportError::InvalidQuality(request.quality));
    }
    if request.max_edge == 0 {
        return Err(ExportError::InvalidThumbnailEdge(request.max_edge));
    }

    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let rgb = decoded
        .thumbnail(request.max_edge, request.max_edge)
        .to_rgb8();

    let mut output = File::create(&request.output_path)?;
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, request.quality);
    encoder.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    drop(output);

    Ok(JpegThumbnailResult {
        bytes_written: fs::metadata(&request.output_path)?.len(),
        output_path: request.output_path,
        format: ExportImageFormat::Jpeg,
    })
}

/// Write a disposable adjusted JPEG preview for the Develop surface.
pub fn write_jpeg_develop_preview(
    request: JpegDevelopPreviewRequest,
) -> Result<JpegThumbnailResult, ExportError> {
    if paths_match(&request.source_path, &request.output_path)? {
        return Err(ExportError::SameSourceAndOutput(request.output_path));
    }
    if !(1..=100).contains(&request.quality) {
        return Err(ExportError::InvalidQuality(request.quality));
    }
    if request.max_edge == 0 {
        return Err(ExportError::InvalidThumbnailEdge(request.max_edge));
    }
    if !adjustments_are_finite(
        request.exposure,
        request.contrast,
        request.white_balance,
        request.tone_recovery,
        request.color_presence,
        &request.tone_curve,
        request.hsl_color_mixer,
        request.detail,
        &request.geometry,
    ) {
        return Err(ExportError::NonFiniteAdjustment);
    }
    validate_tone_curve_adjustment(&request.tone_curve)?;
    validate_hsl_color_mixer_adjustment(request.hsl_color_mixer)?;
    validate_detail_adjustment(request.detail)?;
    validate_geometry_adjustment(&request.geometry)?;
    validate_manual_mask_adjustments(&request.masks)?;

    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let mut rgb = decoded
        .thumbnail(request.max_edge, request.max_edge)
        .to_rgb8();
    apply_exposure_contrast(&mut rgb, request.exposure, request.contrast);
    apply_white_balance(&mut rgb, request.white_balance);
    apply_tone_recovery(&mut rgb, request.tone_recovery);
    apply_tone_curve(&mut rgb, &request.tone_curve);
    apply_color_presence(&mut rgb, request.color_presence);
    apply_hsl_color_mixer(&mut rgb, request.hsl_color_mixer);
    apply_manual_masks(&mut rgb, &request.masks);
    let rgb = apply_supported_geometry(rgb, &request.geometry)?;

    let mut output = File::create(&request.output_path)?;
    let mut encoder =
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, request.quality);
    encoder.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        image::ExtendedColorType::Rgb8,
    )?;
    drop(output);

    Ok(JpegThumbnailResult {
        bytes_written: fs::metadata(&request.output_path)?.len(),
        output_path: request.output_path,
        format: ExportImageFormat::Jpeg,
    })
}

/// Compute real histogram data from the same local JPEG adjustment path used for Develop preview.
pub fn compute_jpeg_develop_histogram(
    request: JpegHistogramRequest,
) -> Result<silica_render::RgbHistogram, ExportError> {
    if !adjustments_are_finite(
        request.exposure,
        request.contrast,
        request.white_balance,
        request.tone_recovery,
        request.color_presence,
        &request.tone_curve,
        request.hsl_color_mixer,
        request.detail,
        &request.geometry,
    ) {
        return Err(ExportError::NonFiniteAdjustment);
    }
    validate_tone_curve_adjustment(&request.tone_curve)?;
    validate_hsl_color_mixer_adjustment(request.hsl_color_mixer)?;
    validate_detail_adjustment(request.detail)?;
    validate_geometry_adjustment(&request.geometry)?;

    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let mut rgb = decoded.to_rgb8();
    apply_exposure_contrast(&mut rgb, request.exposure, request.contrast);
    apply_white_balance(&mut rgb, request.white_balance);
    apply_tone_recovery(&mut rgb, request.tone_recovery);
    apply_tone_curve(&mut rgb, &request.tone_curve);
    apply_color_presence(&mut rgb, request.color_presence);
    apply_hsl_color_mixer(&mut rgb, request.hsl_color_mixer);
    let rgb = apply_supported_geometry(rgb, &request.geometry)?;
    silica_render::compute_rgb_histogram(rgb.as_raw()).map_err(|error| {
        ExportError::Image(image::ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            error.to_string(),
        )))
    })
}

fn paths_match(source_path: &PathBuf, output_path: &PathBuf) -> Result<bool, ExportError> {
    if source_path == output_path {
        return Ok(true);
    }
    if !output_path.exists() {
        return Ok(false);
    }

    if fs::canonicalize(source_path)? == fs::canonicalize(output_path)? {
        return Ok(true);
    }

    paths_share_file_identity(source_path, output_path)
}

#[cfg(unix)]
fn paths_share_file_identity(source_path: &Path, output_path: &Path) -> Result<bool, ExportError> {
    use std::os::unix::fs::MetadataExt;

    let source = fs::metadata(source_path)?;
    let output = fs::metadata(output_path)?;
    Ok(source.dev() == output.dev() && source.ino() == output.ino())
}

#[cfg(not(unix))]
fn paths_share_file_identity(
    _source_path: &Path,
    _output_path: &Path,
) -> Result<bool, ExportError> {
    Ok(false)
}
