use std::error::Error;
use std::fmt;
use std::path::PathBuf;

/// File format written by the local alpha export path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportImageFormat {
    Jpeg,
    Png,
    Tiff,
}

/// Output color profile target recorded by the export contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportColorProfile {
    Srgb,
    DisplayP3,
}

/// Source metadata handling policy for raster exports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportMetadataPolicy {
    Minimal,
    Preserve,
    RemoveGps,
    RemoveAll,
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
    pub masks: Vec<ManualMaskAdjustment>,
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
    pub masks: Vec<ManualMaskAdjustment>,
    pub quality: u8,
    pub color_profile: ExportColorProfile,
}

/// Request to export an already-rendered raster source as sRGB PNG/TIFF.
#[derive(Debug, Clone, PartialEq)]
pub struct RasterSrgbExportRequest {
    pub source_path: PathBuf,
    pub output_path: PathBuf,
    pub format: ExportImageFormat,
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

    pub(crate) fn is_neutral(self) -> bool {
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

    pub(crate) fn is_neutral(self) -> bool {
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

/// Manual mask geometry applied to disposable preview output.
#[derive(Debug, Clone, PartialEq)]
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
    RasterAlphaPlane {
        width: u32,
        height: u32,
        alpha: Vec<u8>,
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
    pub metadata_policy: ExportMetadataPolicy,
    pub bytes_written: u64,
    pub source_sha256: String,
    pub output_sha256: String,
    pub icc_profile_embedded: bool,
    pub icc_profile_sha256: String,
    pub source_metadata_segments: usize,
    pub output_metadata_segments: usize,
    pub source_metadata_copied: bool,
    pub gps_metadata_removed: bool,
}

/// Backwards-compatible result name for the local alpha sRGB export path.
pub type JpegSrgbExportResult = JpegExportResult;

/// Result returned after a raster export is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RasterExportResult {
    pub output_path: PathBuf,
    pub format: ExportImageFormat,
    pub color_profile: ExportColorProfile,
    pub bytes_written: u64,
    pub source_sha256: String,
    pub output_sha256: String,
    pub icc_profile_embedded: bool,
    pub icc_profile_sha256: Option<String>,
}

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

/// Request to compute Develop histogram data from a supported raster source.
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
    InvalidManualMaskAdjustment(String),
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
            Self::InvalidManualMaskAdjustment(message) => {
                write!(formatter, "invalid manual mask adjustment: {message}")
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
            | Self::InvalidManualMaskAdjustment(_)
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

fn export_color_profile_label(profile: ExportColorProfile) -> &'static str {
    match profile {
        ExportColorProfile::Srgb => "sRGB",
        ExportColorProfile::DisplayP3 => "Display P3",
    }
}
