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
    pub color_presence: ColorPresenceAdjustment,
    pub tone_curve: ToneCurveAdjustment,
    pub hsl_color_mixer: HslColorMixerAdjustment,
    pub detail: DetailAdjustment,
    pub geometry: GeometryAdjustment,
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
    pub color_presence: ColorPresenceAdjustment,
    pub tone_curve: ToneCurveAdjustment,
    pub hsl_color_mixer: HslColorMixerAdjustment,
    pub detail: DetailAdjustment,
    pub geometry: GeometryAdjustment,
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

/// Tone curve mode applied to local JPEG preview/export pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneCurveMode {
    None,
    Parametric,
    Point,
}

/// One normalized point in a tone curve.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToneCurvePoint {
    pub x: f64,
    pub y: f64,
}

/// Tone curve values applied to local JPEG preview/export pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct ToneCurveAdjustment {
    pub mode: ToneCurveMode,
    pub rgb_curve: Vec<ToneCurvePoint>,
    pub red_curve: Vec<ToneCurvePoint>,
    pub green_curve: Vec<ToneCurvePoint>,
    pub blue_curve: Vec<ToneCurvePoint>,
}

impl ToneCurveAdjustment {
    pub fn neutral() -> Self {
        Self {
            mode: ToneCurveMode::None,
            rgb_curve: Vec::new(),
            red_curve: Vec::new(),
            green_curve: Vec::new(),
            blue_curve: Vec::new(),
        }
    }
}

/// One HSL color mixer channel applied to local JPEG preview/export pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HslColorChannelAdjustment {
    pub hue: f64,
    pub saturation: f64,
    pub luminance: f64,
}

impl HslColorChannelAdjustment {
    pub fn neutral() -> Self {
        Self {
            hue: 0.0,
            saturation: 0.0,
            luminance: 0.0,
        }
    }

    fn is_neutral(self) -> bool {
        self.hue == 0.0 && self.saturation == 0.0 && self.luminance == 0.0
    }
}

/// HSL color mixer values applied to local JPEG preview/export pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HslColorMixerAdjustment {
    pub red: HslColorChannelAdjustment,
    pub orange: HslColorChannelAdjustment,
    pub yellow: HslColorChannelAdjustment,
    pub green: HslColorChannelAdjustment,
    pub aqua: HslColorChannelAdjustment,
    pub blue: HslColorChannelAdjustment,
    pub purple: HslColorChannelAdjustment,
    pub magenta: HslColorChannelAdjustment,
}

impl HslColorMixerAdjustment {
    pub fn neutral() -> Self {
        let neutral = HslColorChannelAdjustment::neutral();
        Self {
            red: neutral,
            orange: neutral,
            yellow: neutral,
            green: neutral,
            aqua: neutral,
            blue: neutral,
            purple: neutral,
            magenta: neutral,
        }
    }

    fn is_neutral(self) -> bool {
        [
            self.red,
            self.orange,
            self.yellow,
            self.green,
            self.aqua,
            self.blue,
            self.purple,
            self.magenta,
        ]
        .iter()
        .all(|channel| channel.is_neutral())
    }
}

/// Sharpening values for local JPEG preview/export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailSharpeningAdjustment {
    pub amount: f64,
    pub radius: f64,
    pub detail: f64,
    pub masking: f64,
}

impl DetailSharpeningAdjustment {
    pub fn neutral() -> Self {
        Self {
            amount: 0.0,
            radius: 1.0,
            detail: 25.0,
            masking: 0.0,
        }
    }

    fn is_neutral(self) -> bool {
        self == Self::neutral()
    }
}

/// Non-MLX noise reduction values for local JPEG preview/export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailNoiseReductionAdjustment {
    pub luminance: f64,
    pub detail: f64,
    pub contrast: f64,
    pub color: f64,
    pub color_detail: f64,
}

impl DetailNoiseReductionAdjustment {
    pub fn neutral() -> Self {
        Self {
            luminance: 0.0,
            detail: 50.0,
            contrast: 0.0,
            color: 25.0,
            color_detail: 50.0,
        }
    }

    fn is_neutral(self) -> bool {
        self == Self::neutral()
    }
}

/// Detail values for local JPEG preview/export.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DetailAdjustment {
    pub sharpening: DetailSharpeningAdjustment,
    pub noise_reduction: DetailNoiseReductionAdjustment,
}

impl DetailAdjustment {
    pub fn neutral() -> Self {
        Self {
            sharpening: DetailSharpeningAdjustment::neutral(),
            noise_reduction: DetailNoiseReductionAdjustment::neutral(),
        }
    }

    fn is_neutral(self) -> bool {
        self.sharpening.is_neutral() && self.noise_reduction.is_neutral()
    }
}

/// Normalized crop rectangle applied by local JPEG preview/export.
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryCropAdjustment {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub angle: f64,
    pub aspect: Option<String>,
}

/// Perspective/scale transform controls carried for explicit unsupported-state handling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeometryTransformAdjustment {
    pub vertical: f64,
    pub horizontal: f64,
    pub aspect: f64,
    pub scale: f64,
    pub x_offset: f64,
    pub y_offset: f64,
}

impl GeometryTransformAdjustment {
    pub fn neutral() -> Self {
        Self {
            vertical: 0.0,
            horizontal: 0.0,
            aspect: 0.0,
            scale: 100.0,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    pub fn is_neutral(self) -> bool {
        self == Self::neutral()
    }
}

/// Supported non-destructive geometry applied to disposable preview/export output.
#[derive(Debug, Clone, PartialEq)]
pub struct GeometryAdjustment {
    pub crop: Option<GeometryCropAdjustment>,
    pub rotation: f64,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub transform: GeometryTransformAdjustment,
}

impl GeometryAdjustment {
    pub fn neutral() -> Self {
        Self {
            crop: None,
            rotation: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
            transform: GeometryTransformAdjustment::neutral(),
        }
    }
}

/// Manual gradient mask geometry applied to disposable preview output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ManualMaskGeometry {
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

/// Supported manual mask adjustment applied to disposable preview output.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMaskAdjustment {
    pub id: String,
    pub enabled: bool,
    pub invert: bool,
    pub opacity: f64,
    pub feather: f64,
    pub geometry: ManualMaskGeometry,
    pub exposure: f64,
    pub contrast: f64,
}

/// Color presence values applied to local JPEG preview/export pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorPresenceAdjustment {
    pub vibrance: f64,
    pub saturation: f64,
}

impl ColorPresenceAdjustment {
    pub fn neutral() -> Self {
        Self {
            vibrance: 0.0,
            saturation: 0.0,
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
    pub color_presence: ColorPresenceAdjustment,
    pub tone_curve: ToneCurveAdjustment,
    pub hsl_color_mixer: HslColorMixerAdjustment,
    pub detail: DetailAdjustment,
    pub geometry: GeometryAdjustment,
    pub masks: Vec<ManualMaskAdjustment>,
}

/// Request to compute Develop histogram data from a supported JPEG source.
#[derive(Debug, Clone, PartialEq)]
pub struct JpegHistogramRequest {
    pub source_path: PathBuf,
    pub exposure: f64,
    pub contrast: f64,
    pub white_balance: WhiteBalanceAdjustment,
    pub tone_recovery: ToneRecoveryAdjustment,
    pub color_presence: ColorPresenceAdjustment,
    pub tone_curve: ToneCurveAdjustment,
    pub hsl_color_mixer: HslColorMixerAdjustment,
    pub detail: DetailAdjustment,
    pub geometry: GeometryAdjustment,
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
    InvalidToneCurveAdjustment(String),
    InvalidHslColorMixerAdjustment(String),
    UnsupportedDetailAdjustment(String),
    UnsupportedGeometryAdjustment(String),
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
            Self::InvalidToneCurveAdjustment(message) => {
                write!(formatter, "invalid tone curve adjustment: {message}")
            }
            Self::InvalidHslColorMixerAdjustment(message) => {
                write!(formatter, "invalid HSL color mixer adjustment: {message}")
            }
            Self::UnsupportedDetailAdjustment(message) => {
                write!(formatter, "unsupported detail adjustment: {message}")
            }
            Self::UnsupportedGeometryAdjustment(message) => {
                write!(formatter, "unsupported geometry adjustment: {message}")
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
            | Self::InvalidToneCurveAdjustment(_)
            | Self::InvalidHslColorMixerAdjustment(_)
            | Self::UnsupportedDetailAdjustment(_)
            | Self::UnsupportedGeometryAdjustment(_)
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
        color_presence: request.color_presence,
        tone_curve: request.tone_curve,
        hsl_color_mixer: request.hsl_color_mixer,
        detail: request.detail,
        geometry: request.geometry,
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

    let source_sha256 = sha256_file(&request.source_path)?;
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

fn apply_tone_curve(image: &mut image::RgbImage, tone_curve: &ToneCurveAdjustment) {
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

fn apply_color_presence(image: &mut image::RgbImage, color_presence: ColorPresenceAdjustment) {
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

fn apply_manual_masks(image: &mut image::RgbImage, masks: &[ManualMaskAdjustment]) {
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
    match mask.geometry {
        ManualMaskGeometry::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => {
            let start_x = start_x as f32;
            let start_y = start_y as f32;
            let vector_x = end_x as f32 - start_x;
            let vector_y = end_y as f32 - start_y;
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
            let angle = -(rotation as f32).to_radians();
            let cos = angle.cos();
            let sin = angle.sin();
            let dx = x - center_x as f32;
            let dy = y - center_y as f32;
            let rotated_x = (dx * cos - dy * sin) / (radius_x as f32).max(f32::EPSILON);
            let rotated_y = (dx * sin + dy * cos) / (radius_y as f32).max(f32::EPSILON);
            let distance = (rotated_x * rotated_x + rotated_y * rotated_y).sqrt();
            smooth_mask_edge((1.0 - distance).clamp(0.0, 1.0), mask.feather)
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

fn apply_hsl_color_mixer(image: &mut image::RgbImage, hsl_color_mixer: HslColorMixerAdjustment) {
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

fn apply_supported_geometry(
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

fn adjustments_are_finite(
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

fn validate_tone_curve_adjustment(tone_curve: &ToneCurveAdjustment) -> Result<(), ExportError> {
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

fn validate_hsl_color_mixer_adjustment(
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

fn validate_detail_adjustment(detail: DetailAdjustment) -> Result<(), ExportError> {
    if detail.is_neutral() {
        Ok(())
    } else {
        Err(ExportError::UnsupportedDetailAdjustment(
            "Detail preview/export is unsupported until renderer support exists".to_string(),
        ))
    }
}

fn validate_geometry_adjustment(geometry: &GeometryAdjustment) -> Result<(), ExportError> {
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
        _ if icc_rgb_primaries_match(profile, DISPLAY_P3_D50_XYZ) => {
            Some(ExportColorProfile::DisplayP3)
        }
        _ if icc_rgb_primaries_match(profile, SRGB_D50_XYZ) => Some(ExportColorProfile::Srgb),
        _ if profile_contains_ascii(profile, b"sRGB") => Some(ExportColorProfile::Srgb),
        _ => None,
    }
}

const DISPLAY_P3_D50_XYZ: [[f64; 3]; 3] = [
    [0.5151214599609375, 0.2411956787109375, -0.0010528564453125],
    [0.2919769287109375, 0.6922454833984375, 0.0418853759765625],
    [0.1571044921875, 0.0665740966796875, 0.7840728759765625],
];

const SRGB_D50_XYZ: [[f64; 3]; 3] = [
    [0.436065673828125, 0.2224884033203125, 0.013916015625],
    [0.3851470947265625, 0.7168731689453125, 0.097076416015625],
    [0.14306640625, 0.06060791015625, 0.7140960693359375],
];

fn icc_rgb_primaries_match(profile: &[u8], expected: [[f64; 3]; 3]) -> bool {
    let Some(actual) = icc_rgb_primaries(profile) else {
        return false;
    };
    actual
        .iter()
        .flatten()
        .zip(expected.iter().flatten())
        .all(|(actual, expected)| (actual - expected).abs() <= 0.02)
}

fn icc_rgb_primaries(profile: &[u8]) -> Option<[[f64; 3]; 3]> {
    if profile.len() < 132 || &profile[36..40] != b"acsp" {
        return None;
    }
    let tag_count = read_u32_be(profile, 128)? as usize;
    let tag_table_end = 132_usize.checked_add(tag_count.checked_mul(12)?)?;
    if tag_table_end > profile.len() {
        return None;
    }

    Some([
        icc_xyz_tag(profile, tag_count, b"rXYZ")?,
        icc_xyz_tag(profile, tag_count, b"gXYZ")?,
        icc_xyz_tag(profile, tag_count, b"bXYZ")?,
    ])
}

fn icc_xyz_tag(profile: &[u8], tag_count: usize, signature: &[u8; 4]) -> Option<[f64; 3]> {
    for index in 0..tag_count {
        let record_offset = 132 + (index * 12);
        if &profile[record_offset..record_offset + 4] != signature {
            continue;
        }
        let tag_offset = read_u32_be(profile, record_offset + 4)? as usize;
        let tag_size = read_u32_be(profile, record_offset + 8)? as usize;
        if tag_size < 20 || tag_offset.checked_add(20)? > profile.len() {
            return None;
        }
        if &profile[tag_offset..tag_offset + 4] != b"XYZ " {
            return None;
        }
        return Some([
            read_s15_fixed_16(profile, tag_offset + 8)?,
            read_s15_fixed_16(profile, tag_offset + 12)?,
            read_s15_fixed_16(profile, tag_offset + 16)?,
        ]);
    }
    None
}

fn read_u32_be(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_s15_fixed_16(bytes: &[u8], offset: usize) -> Option<f64> {
    let raw = i32::from_be_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
    Some(f64::from(raw) / 65536.0)
}

fn profile_contains_ascii(profile: &[u8], needle: &[u8]) -> bool {
    profile
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle))
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
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
    fn classifies_display_p3_icc_profile_by_xyz_tags_when_hash_differs() {
        let profile = synthetic_rgb_icc_profile([
            [0.5151214599609375, 0.2411956787109375, -0.0010528564453125],
            [0.2919769287109375, 0.6922454833984375, 0.0418853759765625],
            [0.1571044921875, 0.0665740966796875, 0.7840728759765625],
        ]);

        assert_eq!(
            super::classify_icc_profile(&profile),
            Some(super::ExportColorProfile::DisplayP3)
        );
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
    fn writes_masked_jpeg_preview_without_mutating_original() {
        let root = unique_export_root("masked-develop-preview");
        let source_path = root.join("source.jpg");
        let neutral_path = root.join("previews").join("source-neutral.jpg");
        let masked_path = root.join("previews").join("source-masked.jpg");
        std::fs::create_dir_all(neutral_path.parent().expect("preview parent"))
            .expect("create preview directory");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");
        let mask = super::ManualMaskAdjustment {
            id: "mask-linear-1".to_string(),
            enabled: true,
            invert: false,
            opacity: 100.0,
            feather: 0.0,
            geometry: super::ManualMaskGeometry::LinearGradient {
                start_x: 0.0,
                start_y: 0.0,
                end_x: 1.0,
                end_y: 1.0,
            },
            exposure: 1.0,
            contrast: 0.0,
        };

        let neutral = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: neutral_path,
            max_edge: 2,
            quality: 95,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
        })
        .expect("write neutral preview");
        let masked = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: masked_path,
            max_edge: 2,
            quality: 95,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: vec![mask],
        })
        .expect("write masked preview");

        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );
        assert_ne!(
            std::fs::read(neutral.output_path).expect("read neutral preview"),
            std::fs::read(masked.output_path).expect("read masked preview")
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
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
    fn applies_supported_geometry_preview_and_export_without_mutating_original() {
        let root = unique_export_root("geometry");
        let source_path = root.join("source.jpg");
        let preview_path = root.join("previews").join("geometry.jpg");
        let export_path = root.join("export").join("geometry.jpg");
        std::fs::create_dir_all(export_path.parent().expect("export parent"))
            .expect("create export directory");
        std::fs::create_dir_all(preview_path.parent().expect("preview parent"))
            .expect("create preview directory");
        write_geometry_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");
        let geometry = super::GeometryAdjustment {
            crop: Some(super::GeometryCropAdjustment {
                x: 0.0,
                y: 0.0,
                width: 0.5,
                height: 1.0,
                angle: 0.0,
                aspect: None,
            }),
            rotation: 90.0,
            flip_horizontal: true,
            flip_vertical: false,
            ..super::GeometryAdjustment::neutral()
        };

        let preview = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: preview_path,
            max_edge: 4,
            quality: 95,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: geometry.clone(),
            masks: Vec::new(),
        })
        .expect("write geometry preview");
        let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: export_path,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry,
            quality: 95,
            color_profile: super::ExportColorProfile::Srgb,
        })
        .expect("export geometry jpeg");

        assert_eq!(
            std::fs::read(&source_path).expect("read original after"),
            original_before
        );
        for path in [&preview.output_path, &exported.output_path] {
            let decoded = image::ImageReader::open(path)
                .expect("open geometry output")
                .with_guessed_format()
                .expect("guess geometry format")
                .decode()
                .expect("decode geometry output");
            assert_eq!(decoded.width(), 3);
            assert_eq!(decoded.height(), 2);
        }

        remove_export_root(&root);
    }

    #[test]
    fn blocks_unsupported_geometry_without_writing_output() {
        let root = unique_export_root("unsupported-geometry");
        let source_path = root.join("source.jpg");
        let output_path = root.join("export").join("unsupported.jpg");
        std::fs::create_dir_all(output_path.parent().expect("export parent"))
            .expect("create export directory");
        write_geometry_source_jpeg(&source_path);
        let geometry = super::GeometryAdjustment {
            rotation: 13.0,
            ..super::GeometryAdjustment::neutral()
        };

        let error = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path,
            output_path: output_path.clone(),
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry,
            quality: 95,
            color_profile: super::ExportColorProfile::Srgb,
        })
        .expect_err("unsupported arbitrary rotation should fail");

        assert!(matches!(
            error,
            super::ExportError::UnsupportedGeometryAdjustment(_)
        ));
        assert!(!output_path.exists());
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
        })
        .expect("write tone recovery preview");
        let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: adjusted_export_path,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery,
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
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

    #[test]
    fn writes_tone_curve_adjusted_preview_and_export_without_mutating_original() {
        let root = unique_export_root("tone-curve");
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
        let tone_curve = super::ToneCurveAdjustment {
            mode: super::ToneCurveMode::Point,
            rgb_curve: vec![
                super::ToneCurvePoint { x: 0.0, y: 0.0 },
                super::ToneCurvePoint { x: 0.5, y: 0.28 },
                super::ToneCurvePoint { x: 1.0, y: 1.0 },
            ],
            red_curve: Vec::new(),
            green_curve: Vec::new(),
            blue_curve: Vec::new(),
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: tone_curve.clone(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
        })
        .expect("write tone curve preview");
        let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: adjusted_export_path,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve,
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            quality: 90,
            color_profile: super::ExportColorProfile::Srgb,
        })
        .expect("export tone curve jpeg");

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
    fn writes_color_presence_adjusted_preview_and_export_without_mutating_original() {
        let root = unique_export_root("color-presence");
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
        let color_presence = super::ColorPresenceAdjustment {
            vibrance: 24.0,
            saturation: -8.5,
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence,
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
        })
        .expect("write color presence preview");
        let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: adjusted_export_path,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence,
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            quality: 90,
            color_profile: super::ExportColorProfile::Srgb,
        })
        .expect("export color presence jpeg");

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
    fn writes_hsl_color_mixer_preview_and_export_without_mutating_original() {
        let root = unique_export_root("hsl-color-mixer");
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
        let hsl_color_mixer = super::HslColorMixerAdjustment {
            blue: super::HslColorChannelAdjustment {
                hue: -12.0,
                saturation: 24.0,
                luminance: -8.5,
            },
            ..super::HslColorMixerAdjustment::neutral()
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
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
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
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer,
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
        })
        .expect("write hsl preview");
        let exported = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: adjusted_export_path,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer,
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
            quality: 90,
            color_profile: super::ExportColorProfile::Srgb,
        })
        .expect("export hsl jpeg");

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
    fn rejects_non_neutral_detail_preview_and_export_until_renderer_support_exists() {
        let root = unique_export_root("detail-boundary");
        let source_path = root.join("source.jpg");
        let preview_path = root.join("previews").join("detail.jpg");
        let export_path = root.join("export").join("detail.jpg");
        std::fs::create_dir_all(preview_path.parent().expect("preview parent"))
            .expect("create preview directory");
        std::fs::create_dir_all(export_path.parent().expect("export parent"))
            .expect("create export directory");
        write_source_jpeg(&source_path);
        let detail = super::DetailAdjustment {
            sharpening: super::DetailSharpeningAdjustment {
                amount: 42.0,
                radius: 1.2,
                detail: 35.0,
                masking: 10.0,
            },
            ..super::DetailAdjustment::neutral()
        };

        let preview_error = super::write_jpeg_develop_preview(super::JpegDevelopPreviewRequest {
            source_path: source_path.clone(),
            output_path: preview_path.clone(),
            max_edge: 2,
            quality: 82,
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail,
            geometry: super::GeometryAdjustment::neutral(),
            masks: Vec::new(),
        })
        .expect_err("detail preview unsupported");
        let export_error = super::export_jpeg_with_color_profile(super::JpegColorExportRequest {
            source_path: source_path.clone(),
            output_path: export_path.clone(),
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment::neutral(),
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail,
            geometry: super::GeometryAdjustment::neutral(),
            quality: 90,
            color_profile: super::ExportColorProfile::Srgb,
        })
        .expect_err("detail export unsupported");

        assert!(matches!(
            preview_error,
            super::ExportError::UnsupportedDetailAdjustment(_)
        ));
        assert!(matches!(
            export_error,
            super::ExportError::UnsupportedDetailAdjustment(_)
        ));
        assert!(!preview_path.exists());
        assert!(!export_path.exists());

        remove_export_root(&root);
    }

    #[test]
    fn computes_adjusted_jpeg_histogram_without_mutating_original() {
        let root = unique_export_root("histogram");
        let source_path = root.join("source.jpg");
        std::fs::create_dir_all(&root).expect("create histogram root");
        write_source_jpeg(&source_path);
        let original_before = std::fs::read(&source_path).expect("read original before");

        let histogram = super::compute_jpeg_develop_histogram(super::JpegHistogramRequest {
            source_path: source_path.clone(),
            exposure: 0.0,
            contrast: 0.0,
            white_balance: super::WhiteBalanceAdjustment::neutral(),
            tone_recovery: super::ToneRecoveryAdjustment::neutral(),
            color_presence: super::ColorPresenceAdjustment {
                vibrance: 24.0,
                saturation: -8.5,
            },
            tone_curve: super::ToneCurveAdjustment::neutral(),
            hsl_color_mixer: super::HslColorMixerAdjustment::neutral(),
            detail: super::DetailAdjustment::neutral(),
            geometry: super::GeometryAdjustment::neutral(),
        })
        .expect("compute histogram");

        assert_eq!(histogram.pixel_count, 4);
        assert_eq!(histogram.red.len(), 256);
        assert_eq!(histogram.green.len(), 256);
        assert_eq!(histogram.blue.len(), 256);
        assert_eq!(histogram.luminance.len(), 256);
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

    fn write_geometry_source_jpeg(path: &Path) {
        let image = image::RgbImage::from_fn(4, 3, |x, y| {
            image::Rgb([
                (32 + (x * 40)) as u8,
                (48 + (y * 50)) as u8,
                (96 + ((x + y) * 10)) as u8,
            ])
        });
        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write geometry source jpeg");
    }

    fn synthetic_rgb_icc_profile(primaries: [[f64; 3]; 3]) -> Vec<u8> {
        let tag_table_start = 128;
        let tag_count_size = 4;
        let tag_record_size = 12;
        let tag_data_start = tag_table_start + tag_count_size + (3 * tag_record_size);
        let tag_data_size = 20;
        let profile_size = tag_data_start + (3 * tag_data_size);
        let mut profile = vec![0_u8; profile_size];
        profile[0..4].copy_from_slice(&(profile_size as u32).to_be_bytes());
        profile[36..40].copy_from_slice(b"acsp");
        profile[128..132].copy_from_slice(&3_u32.to_be_bytes());

        for (index, (signature, values)) in [
            (b"rXYZ", primaries[0]),
            (b"gXYZ", primaries[1]),
            (b"bXYZ", primaries[2]),
        ]
        .into_iter()
        .enumerate()
        {
            let record_offset = 132 + (index * tag_record_size);
            let data_offset = tag_data_start + (index * tag_data_size);
            profile[record_offset..record_offset + 4].copy_from_slice(signature);
            profile[record_offset + 4..record_offset + 8]
                .copy_from_slice(&(data_offset as u32).to_be_bytes());
            profile[record_offset + 8..record_offset + 12]
                .copy_from_slice(&(tag_data_size as u32).to_be_bytes());
            profile[data_offset..data_offset + 4].copy_from_slice(b"XYZ ");
            for (component, value) in values.iter().enumerate() {
                let fixed = (value * 65536.0).round() as i32;
                let component_offset = data_offset + 8 + (component * 4);
                profile[component_offset..component_offset + 4]
                    .copy_from_slice(&fixed.to_be_bytes());
            }
        }

        profile
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
