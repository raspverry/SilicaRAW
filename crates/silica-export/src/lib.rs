//! Export coordination boundary for SilicaRAW.
//!
//! The local alpha export path accepts already-rendered raster inputs and writes
//! a separate JPEG file. RAW decoding and final color-management fixtures remain
//! outside this crate.

use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use image::ImageEncoder;
use sha2::{Digest, Sha256};

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-export";

/// File format written by the local alpha export path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportImageFormat {
    Jpeg,
}

/// Output color profile target recorded by the export contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportColorProfile {
    Srgb,
    DisplayP3,
}

/// Request to export an already-rendered raster source as JPEG sRGB.
#[derive(Debug, Clone, PartialEq)]
pub struct JpegSrgbExportRequest {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: WhiteBalanceAdjustment,
    pub tone_recovery: ToneRecoveryAdjustment,
    pub quality: u8,
}

/// Request to export an already-rendered raster source as JPEG with an explicit color profile.
#[derive(Debug, Clone, PartialEq)]
pub struct JpegColorExportRequest {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: WhiteBalanceAdjustment,
    pub tone_recovery: ToneRecoveryAdjustment,
    pub quality: u8,
    pub color_profile: ExportColorProfile,
}

/// White balance mode carried through local JPEG preview/export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteBalanceMode {
    AsShot,
    Auto,
    Daylight,
    Cloudy,
    Shade,
    Tungsten,
    Fluorescent,
    Flash,
    Custom,
}

/// White balance values applied to local JPEG preview/export pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WhiteBalanceAdjustment {
    pub mode: WhiteBalanceMode,
    pub temperature: f64,
    pub tint: f64,
}

impl WhiteBalanceAdjustment {
    pub fn neutral() -> Self {
        Self {
            mode: WhiteBalanceMode::AsShot,
            temperature: 5200.0,
            tint: 0.0,
        }
    }
}

/// Tone recovery values applied to local JPEG preview/export pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneRecoveryAdjustment {
    pub highlights: f64,
    pub shadows: f64,
    pub whites: f64,
    pub blacks: f64,
}

impl ToneRecoveryAdjustment {
    pub fn neutral() -> Self {
        Self {
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
        }
    }
}

/// Result returned after a JPEG export is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegExportResult {
    pub output_path: PathBuf,
    pub format: ExportImageFormat,
    pub color_profile: ExportColorProfile,
    pub bytes_written: u64,
    pub source_sha256: String,
    pub output_sha256: String,
    pub icc_profile_embedded: bool,
    pub icc_profile_sha256: String,
}

/// Backwards-compatible result name for the local alpha sRGB export path.
pub type JpegSrgbExportResult = JpegExportResult;

/// ICC inspection result for exported JPEG proof work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegIccProfileInspection {
    pub embedded: bool,
    pub color_profile: Option<ExportColorProfile>,
    pub icc_profile_sha256: Option<String>,
}

/// Request to create a disposable JPEG thumbnail for a raster source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegThumbnailRequest {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub max_edge: u32,
    pub quality: u8,
}

/// Request to create a disposable adjusted JPEG preview for Develop.
#[derive(Debug, Clone, PartialEq)]
pub struct JpegDevelopPreviewRequest {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub max_edge: u32,
    pub quality: u8,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: WhiteBalanceAdjustment,
    pub tone_recovery: ToneRecoveryAdjustment,
}

/// Result returned after a JPEG thumbnail is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JpegThumbnailResult {
    pub output_path: PathBuf,
    pub format: ExportImageFormat,
    pub bytes_written: u64,
}

/// Dimensions read from an existing raster file without writing output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterDimensions {
    pub width: u32,
    pub height: u32,
}

/// Errors returned by the export crate.
#[derive(Debug)]
pub enum ExportError {
    Io(std::io::Error),
    Image(image::ImageError),
    InvalidQuality(u8),
    InvalidThumbnailEdge(u32),
    NonFiniteAdjustment,
    SameSourceAndOutput(PathBuf),
    IccProfileUnavailable {
        profile: ExportColorProfile,
        path: PathBuf,
        message: String,
    },
    InvalidJpegIccProfile(PathBuf),
}

impl fmt::Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "filesystem error: {error}"),
            Self::Image(error) => write!(formatter, "image error: {error}"),
            Self::InvalidQuality(quality) => {
                write!(
                    formatter,
                    "jpeg quality must be between 1 and 100, got {quality}"
                )
            }
            Self::InvalidThumbnailEdge(edge) => {
                write!(
                    formatter,
                    "thumbnail max edge must be greater than 0, got {edge}"
                )
            }
            Self::NonFiniteAdjustment => {
                write!(
                    formatter,
                    "exposure and contrast adjustments must be finite"
                )
            }
            Self::SameSourceAndOutput(path) => {
                write!(
                    formatter,
                    "output path must differ from original source path: {}",
                    path.display()
                )
            }
            Self::IccProfileUnavailable {
                profile,
                path,
                message,
            } => {
                write!(
                    formatter,
                    "{} ICC profile is unavailable at {}: {message}",
                    export_color_profile_label(*profile),
                    path.display()
                )
            }
            Self::InvalidJpegIccProfile(path) => {
                write!(
                    formatter,
                    "jpeg ICC profile could not be inspected: {}",
                    path.display()
                )
            }
        }
    }
}

impl Error for ExportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Image(error) => Some(error),
            Self::InvalidQuality(_)
            | Self::InvalidThumbnailEdge(_)
            | Self::NonFiniteAdjustment
            | Self::SameSourceAndOutput(_)
            | Self::IccProfileUnavailable { .. }
            | Self::InvalidJpegIccProfile(_) => None,
        }
    }
}

impl From<std::io::Error> for ExportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<image::ImageError> for ExportError {
    fn from(error: image::ImageError) -> Self {
        Self::Image(error)
    }
}

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
        quality: request.quality,
        color_profile: ExportColorProfile::Srgb,
    })
}

/// Export an already-rendered raster source as a separate JPEG with an explicit ICC profile.
pub fn export_jpeg_with_color_profile(
    request: JpegColorExportRequest,
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
    ) {
        return Err(ExportError::NonFiniteAdjustment);
    }

    let source_sha256 = sha256_file(&request.source_path)?;
    let icc_profile = export_icc_profile(request.color_profile)?;
    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let mut rgb = decoded.to_rgb8();
    apply_exposure_contrast(&mut rgb, request.exposure, request.contrast);
    apply_white_balance(&mut rgb, request.white_balance);
    apply_tone_recovery(&mut rgb, request.tone_recovery);

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
        source_sha256,
        output_sha256,
        icc_profile_embedded: inspection.embedded,
        icc_profile_sha256,
    })
}

/// Inspect the first embedded ICC profile in a JPEG file.
pub fn inspect_jpeg_icc_profile(
    path: impl AsRef<Path>,
) -> Result<JpegIccProfileInspection, ExportError> {
    let path = path.as_ref();
    let bytes = fs::read(path)?;
    let profile = first_icc_profile(&bytes)
        .map_err(|_| ExportError::InvalidJpegIccProfile(path.to_path_buf()))?;
    let Some(profile) = profile else {
        return Ok(JpegIccProfileInspection {
            embedded: false,
            color_profile: None,
            icc_profile_sha256: None,
        });
    };

    let icc_profile_sha256 = sha256_hex(&profile);
    Ok(JpegIccProfileInspection {
        embedded: true,
        color_profile: classify_icc_profile(&profile),
        icc_profile_sha256: Some(icc_profile_sha256),
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
    ) {
        return Err(ExportError::NonFiniteAdjustment);
    }

    let decoded = image::ImageReader::open(&request.source_path)?
        .with_guessed_format()?
        .decode()?;
    let mut rgb = decoded
        .thumbnail(request.max_edge, request.max_edge)
        .to_rgb8();
    apply_exposure_contrast(&mut rgb, request.exposure, request.contrast);
    apply_white_balance(&mut rgb, request.white_balance);
    apply_tone_recovery(&mut rgb, request.tone_recovery);

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

fn paths_match(source_path: &PathBuf, output_path: &PathBuf) -> Result<bool, ExportError> {
    if source_path == output_path {
        return Ok(true);
    }
    if !output_path.exists() {
        return Ok(false);
    }

    Ok(fs::canonicalize(source_path)? == fs::canonicalize(output_path)?)
}

fn apply_exposure_contrast(image: &mut image::RgbImage, exposure: f64, contrast: f64) {
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

fn apply_white_balance(image: &mut image::RgbImage, white_balance: WhiteBalanceAdjustment) {
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

fn apply_tone_recovery(image: &mut image::RgbImage, tone_recovery: ToneRecoveryAdjustment) {
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

fn adjustments_are_finite(
    exposure: f64,
    contrast: f64,
    white_balance: WhiteBalanceAdjustment,
    tone_recovery: ToneRecoveryAdjustment,
) -> bool {
    exposure.is_finite()
        && contrast.is_finite()
        && white_balance.temperature.is_finite()
        && white_balance.tint.is_finite()
        && tone_recovery.highlights.is_finite()
        && tone_recovery.shadows.is_finite()
        && tone_recovery.whites.is_finite()
        && tone_recovery.blacks.is_finite()
}

fn export_icc_profile(profile: ExportColorProfile) -> Result<Vec<u8>, ExportError> {
    let path = export_icc_profile_path(profile);
    fs::read(&path).map_err(|error| ExportError::IccProfileUnavailable {
        profile,
        path,
        message: error.to_string(),
    })
}

fn export_icc_profile_path(profile: ExportColorProfile) -> PathBuf {
    match profile {
        ExportColorProfile::Srgb => {
            PathBuf::from("/System/Library/ColorSync/Profiles/sRGB Profile.icc")
        }
        ExportColorProfile::DisplayP3 => {
            PathBuf::from("/System/Library/ColorSync/Profiles/Display P3.icc")
        }
    }
}

fn first_icc_profile(bytes: &[u8]) -> Result<Option<Vec<u8>>, ()> {
    if bytes.len() < 2 || bytes[0..2] != [0xff, 0xd8] {
        return Err(());
    }

    let mut index = 2;
    while index + 4 <= bytes.len() {
        if bytes[index] != 0xff {
            return Err(());
        }

        let marker = bytes[index + 1];
        if marker == 0xd9 || marker == 0xda {
            return Ok(None);
        }

        let length = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]) as usize;
        if length < 2 || index + 2 + length > bytes.len() {
            return Err(());
        }

        let marker_payload = &bytes[index + 4..index + 2 + length];
        if marker == 0xe2
            && marker_payload.starts_with(b"ICC_PROFILE\0")
            && marker_payload.len() >= 14
        {
            return Ok(Some(marker_payload[14..].to_vec()));
        }

        index += 2 + length;
    }

    Ok(None)
}

fn classify_icc_profile(profile: &[u8]) -> Option<ExportColorProfile> {
    match sha256_hex(profile).as_str() {
        "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e" => {
            Some(ExportColorProfile::Srgb)
        }
        "0ff6958f98684c61f6bbdce1368ddeaf3873baf84545baba482e920d92a914c0" => {
            Some(ExportColorProfile::DisplayP3)
        }
        _ if profile
            .windows(b"sRGB".len())
            .any(|window| window.eq_ignore_ascii_case(b"sRGB")) =>
        {
            Some(ExportColorProfile::Srgb)
        }
        _ => None,
    }
}

fn sha256_file(path: &Path) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }

    Ok(format!("{:x}", digest.finalize()))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn export_color_profile_label(profile: ExportColorProfile) -> &'static str {
    match profile {
        ExportColorProfile::Srgb => "sRGB",
        ExportColorProfile::DisplayP3 => "Display P3",
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-export");
    }

    #[test]
    fn exports_jpeg_srgb_without_mutating_original() {
        let root = unique_export_root("jpeg");
        let source_path = root.join("source.jpg");
        let output_path = root.join("export").join("edited.jpg");
        std::fs::create_dir_all(output_path.parent().expect("output parent"))
            .expect("create output directory");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");

        let result = super::export_jpeg_srgb(super::JpegSrgbExportRequest {
            source_path: source_path.clone(),
            output_path: output_path.clone(),
            exposure: 0.5,
            contrast: -8.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            quality: 90,
        })
        .expect("export jpeg srgb");

        assert_eq!(result.output_path, output_path);
        assert_eq!(result.format, super::ExportImageFormat::Jpeg);
        assert_eq!(result.color_profile, super::ExportColorProfile::Srgb);
        assert!(result.bytes_written > 0);
        assert!(result.icc_profile_embedded);
        assert_eq!(
            result.output_sha256,
            super::sha256_file(&result.output_path).expect("hash exported jpeg")
        );
        assert_eq!(
            result.source_sha256,
            super::sha256_file(&source_path).expect("hash source jpeg")
        );
        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );
        let inspection =
            super::inspect_jpeg_icc_profile(&result.output_path).expect("inspect exported ICC");
        assert!(inspection.embedded);
        assert_eq!(
            inspection.color_profile,
            Some(super::ExportColorProfile::Srgb)
        );
        assert_eq!(
            inspection.icc_profile_sha256.as_deref(),
            Some(result.icc_profile_sha256.as_str())
        );

        let exported = image::ImageReader::open(&result.output_path)
            .expect("open exported jpeg")
            .with_guessed_format()
            .expect("guess jpeg format")
            .decode()
            .expect("decode exported jpeg");
        assert_eq!(exported.width(), 2);
        assert_eq!(exported.height(), 2);

        remove_export_root(&root);
    }

    #[test]
    fn exports_display_p3_jpeg_only_when_explicitly_requested() {
        let root = unique_export_root("display-p3");
        let source_path = root.join("source.jpg");
        let output_path = root.join("export").join("display-p3.jpg");
        std::fs::create_dir_all(output_path.parent().expect("output parent"))
            .expect("create output directory");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");

        let result = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: output_path.clone(),
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            quality: 90,
            color_profile: super::ExportColorProfile::DisplayP3,
        })
        .expect("export display p3 jpeg");

        assert_eq!(result.output_path, output_path);
        assert_eq!(result.format, super::ExportImageFormat::Jpeg);
        assert_eq!(result.color_profile, super::ExportColorProfile::DisplayP3);
        assert!(result.icc_profile_embedded);
        assert_eq!(
            result.output_sha256,
            super::sha256_file(&result.output_path).expect("hash exported jpeg")
        );
        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );

        let inspection =
            super::inspect_jpeg_icc_profile(&result.output_path).expect("inspect exported ICC");
        assert!(inspection.embedded);
        assert_eq!(
            inspection.color_profile,
            Some(super::ExportColorProfile::DisplayP3)
        );
        assert_eq!(
            inspection.icc_profile_sha256.as_deref(),
            Some(result.icc_profile_sha256.as_str())
        );

        remove_export_root(&root);
    }

    #[test]
    fn refuses_to_export_over_original_path() {
        let root = unique_export_root("same-path");
        let source_path = root.join("source.jpg");
        std::fs::create_dir_all(&root).expect("create export root");
        write_source_jpeg(&source_path);

        let error = super::export_jpeg_srgb(super::JpegSrgbExportRequest {
            source_path: source_path.clone(),
            output_path: source_path.clone(),
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            quality: 90,
        })
        .expect_err("same source/output path should fail");

        assert!(error.to_string().contains("output path must differ"));
        remove_export_root(&root);
    }

    #[test]
    fn writes_jpeg_thumbnail_without_mutating_original() {
        let root = unique_export_root("thumbnail");
        let source_path = root.join("source.jpg");
        let output_path = root.join("thumbs").join("source-thumb.jpg");
        std::fs::create_dir_all(output_path.parent().expect("output parent"))
            .expect("create output directory");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");

        let result = super::write_jpeg_thumbnail(super::JpegThumbnailRequest {
            source_path: source_path.clone(),
            output_path: output_path.clone(),
            max_edge: 2,
            quality: 80,
        })
        .expect("write thumbnail");

        assert_eq!(result.output_path, output_path);
        assert_eq!(result.format, super::ExportImageFormat::Jpeg);
        assert!(result.bytes_written > 0);
        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );
        let decoded = image::ImageReader::open(&result.output_path)
            .expect("open thumbnail")
            .with_guessed_format()
            .expect("guess thumbnail format")
            .decode()
            .expect("decode thumbnail");
        assert!(decoded.width() <= 2);
        assert!(decoded.height() <= 2);

        remove_export_root(&root);
    }

    #[test]
    fn writes_adjusted_jpeg_preview_without_mutating_original() {
        let root = unique_export_root("develop-preview");
        let source_path = root.join("source.jpg");
        let neutral_path = root.join("previews").join("source-neutral.jpg");
        let adjusted_path = root.join("previews").join("source-adjusted.jpg");
        std::fs::create_dir_all(neutral_path.parent().expect("preview parent"))
            .expect("create preview directory");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");

        let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: neutral_path,
            max_edge: 2,
            quality: 82,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        })
        .expect("write neutral preview");
        let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: adjusted_path,
            max_edge: 2,
            quality: 82,
            exposure: 1.0,
            contrast: 20.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        })
        .expect("write adjusted preview");

        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );
        assert_ne!(
            std::fs::read(neutral.output_path).expect("read neutral preview"),
            std::fs::read(adjusted.output_path).expect("read adjusted preview")
        );

        remove_export_root(&root);
    }

    #[test]
    fn writes_white_balance_adjusted_preview_and_export_without_mutating_original() {
        let root = unique_export_root("white-balance");
        let source_path = root.join("source.jpg");
        let neutral_preview_path = root.join("previews").join("neutral.jpg");
        let adjusted_preview_path = root.join("previews").join("adjusted.jpg");
        let adjusted_export_path = root.join("export").join("adjusted.jpg");
        std::fs::create_dir_all(adjusted_export_path.parent().expect("export parent"))
            .expect("create export directory");
        std::fs::create_dir_all(neutral_preview_path.parent().expect("preview parent"))
            .expect("create preview directory");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");
        let white_balance = super::WhiteBalanceAdjustment {
            mode: super::WhiteBalanceMode::Custom,
            temperature: 6500.0,
            tint: 20.0,
        };

        let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: neutral_preview_path,
            max_edge: 2,
            quality: 82,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        })
        .expect("write neutral preview");
        let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: adjusted_preview_path,
            max_edge: 2,
            quality: 82,
            exposure: 0.0,
            contrast: 0.0,
            white_balance,
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        })
        .expect("write white balance preview");
        let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: adjusted_export_path,
            exposure: 0.0,
            contrast: 0.0,
            quality: 90,
            color_profile: super::ExportColorProfile::Srgb,
            white_balance,
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        })
        .expect("export white balance jpeg");

        assert_ne!(
            std::fs::read(neutral.output_path).expect("read neutral preview"),
            std::fs::read(adjusted.output_path).expect("read adjusted preview")
        );
        assert!(exported.bytes_written > 0);
        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );

        remove_export_root(&root);
    }

    #[test]
    fn writes_tone_recovery_adjusted_preview_and_export_without_mutating_original() {
        let root = unique_export_root("tone-recovery");
        let source_path = root.join("source.jpg");
        let neutral_preview_path = root.join("previews").join("neutral.jpg");
        let adjusted_preview_path = root.join("previews").join("adjusted.jpg");
        let adjusted_export_path = root.join("export").join("adjusted.jpg");
        std::fs::create_dir_all(adjusted_export_path.parent().expect("export parent"))
            .expect("create export directory");
        std::fs::create_dir_all(neutral_preview_path.parent().expect("preview parent"))
            .expect("create preview directory");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");
        let tone_recovery = super::ToneRecoveryAdjustment {
            highlights: -35.0,
            shadows: 42.0,
            whites: 10.0,
            blacks: -12.0,
        };

        let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: neutral_preview_path,
            max_edge: 2,
            quality: 82,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
        })
        .expect("write neutral preview");
        let adjusted = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: adjusted_preview_path,
            max_edge: 2,
            quality: 82,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery,
        })
        .expect("write tone recovery preview");
        let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: adjusted_export_path,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery,
            quality: 90,
            color_profile: super::ExportColorProfile::Srgb,
        })
        .expect("export tone recovery jpeg");

        assert_ne!(
            std::fs::read(neutral.output_path).expect("read neutral preview"),
            std::fs::read(adjusted.output_path).expect("read adjusted preview")
        );
        assert!(exported.bytes_written > 0);
        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );

        remove_export_root(&root);
    }

    fn write_source_jpeg(path: &Path) {
        let image = image::RgbImage::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgb([64, 128, 192])
            } else {
                image::Rgb([192, 128, 64])
            }
        });
        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write source jpeg");
    }

    fn unique_export_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "silicaraw-export-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn remove_export_root(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }
}
