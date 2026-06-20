//! RAW decode abstraction boundary for SilicaRAW.
//!
//! Spike 002 records the decoder path gate. This crate still does not decode RAW
//! files or link decoder backends.

use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

mod core_image_raw_probe;
mod raw_probe_fixture;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-decode";

/// Decoder path selected by the RAW decoder spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawDecoderPath {
    /// Use Apple's Core Image RAW path as the first implementation target.
    CoreImageRawPrimary,
    /// Use LibRaw as the first implementation target.
    LibRawPrimary,
    /// Maintain Core Image and LibRaw as first-class parallel backends.
    HybridCoreImageAndLibRaw,
}

/// Status of the LibRaw fallback after Spike 002.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibRawFallbackStatus {
    /// Do not add the dependency until Core Image fixture coverage fails a real need.
    DeferredUntilFixtureGap,
}

/// Status of legally usable RAW fixtures in the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureSetStatus {
    /// No legal RAW fixtures are committed yet.
    MissingLegalRawFixtures,
}

/// Recorded output of Spike 002.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoderGate {
    pub path: RawDecoderPath,
    pub libraw_fallback: LibRawFallbackStatus,
    pub fixture_set: FixtureSetStatus,
}

/// Spike 002 decision for downstream crates and tests.
pub const SPIKE_002_DECODER_GATE: DecoderGate = DecoderGate {
    path: RawDecoderPath::CoreImageRawPrimary,
    libraw_fallback: LibRawFallbackStatus::DeferredUntilFixtureGap,
    fixture_set: FixtureSetStatus::MissingLegalRawFixtures,
};

/// Tag used in docs and future issue labels for decoder-dependent work.
pub const DECODER_BLOCKING_TAG: &str = "decoder-blocking";

/// Preview decode backend selected for a photo candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewDecodeBackend {
    /// No backend should run for unsupported catalog entries.
    None,
    /// Browser/native raster preview path for simple already-rendered images.
    Raster,
    /// Future Core Image RAW backend selected by Spike 002.
    CoreImageRaw,
}

/// Readiness state for opening a preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewDecodeStatus {
    /// The candidate can be handed to the preview surface without RAW decoding.
    Ready,
    /// The catalog entry is unsupported for preview.
    Unsupported,
    /// RAW preview is intentionally blocked until the Core Image probe is fixture-backed.
    BlockedByMissingRawFixtureProbe,
}

/// Decode-side preview plan. This is a routing contract, not decoded pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewDecodePlan {
    pub source_path: String,
    pub backend: PreviewDecodeBackend,
    pub status: PreviewDecodeStatus,
    pub message: String,
}

/// Request for a proof-only Core Image RAW probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawProbeRequest {
    pub source_path: String,
    pub expected_sha256: Option<String>,
}

impl RawProbeRequest {
    pub fn new(source_path: impl AsRef<str>) -> Self {
        Self {
            source_path: source_path.as_ref().to_string(),
            expected_sha256: None,
        }
    }
}

/// Backend used by a RAW probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbeBackend {
    CoreImageRaw,
}

/// Platform path used by a RAW probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbePlatform {
    Macos,
    UnsupportedPlatform,
}

/// High-level status of a RAW probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbeStatus {
    Success,
    Unsupported,
    Failed,
    Unavailable,
}

/// Recoverable category for failed or unavailable RAW probes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbeErrorCategory {
    UnsupportedPlatform,
    MissingFile,
    SourceHashMismatch,
    CoreImageUnavailable,
    CoreImageOpenFailed,
    CoreImageMetadataMissing,
    PermissionDenied,
    InvalidFixture,
    Unknown,
}

/// Structured proof result for Core Image RAW probing. This is not decoded pixels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawProbeResult {
    pub backend: RawProbeBackend,
    pub platform: RawProbePlatform,
    pub macos_version: Option<String>,
    pub source_path: String,
    pub source_sha256: Option<String>,
    pub original_file_size: Option<u64>,
    pub original_modified_at: Option<String>,
    pub status: RawProbeStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<i32>,
    pub error_category: Option<RawProbeErrorCategory>,
    pub message: String,
}

/// Run the proof-only Core Image RAW probe route.
pub fn probe_core_image_raw(request: RawProbeRequest) -> RawProbeResult {
    core_image_raw_probe::probe_core_image_raw(request)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFixtureProbeReport {
    pub manifest_path: String,
    pub results: Vec<RawFixtureProbeResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFixtureProbeResult {
    pub fixture_id: String,
    pub fixture_class: String,
    pub relative_path: String,
    pub probe: RawProbeResult,
    pub original_hash_unchanged: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawFixtureProbeError {
    FeatureDisabled,
    ReadManifest { path: String, message: String },
    InvalidManifest { path: String, message: String },
    InvalidFixture { fixture_id: String, message: String },
}

pub fn probe_raw_fixture_manifest(
    manifest_path: impl AsRef<str>,
) -> Result<RawFixtureProbeReport, RawFixtureProbeError> {
    raw_probe_fixture::probe_raw_fixture_manifest(manifest_path.as_ref())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductRawDecodeStatus {
    Supported,
    BlockedPendingEvidence,
    BlockedCoreImageFailed,
    BlockedUnsupportedClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductRawDecodePlan {
    pub source_path: String,
    pub backend: RawProbeBackend,
    pub status: ProductRawDecodeStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<i32>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedImageDecoderBackend {
    CoreImageRaw,
}

impl DecodedImageDecoderBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CoreImageRaw => "core_image_raw",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedImageHandoffStatus {
    Ready,
    BlockedPendingEvidence,
    BlockedCoreImageFailed,
    BlockedUnsupportedClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImageCacheIdentity {
    pub cache_key: String,
    pub disposable: bool,
}

impl DecodedImageCacheIdentity {
    pub fn disposable(cache_key: impl Into<String>) -> Self {
        Self {
            cache_key: cache_key.into(),
            disposable: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodedImagePixelFormat {
    Rgba16Float,
    JpegSrgb8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedImageHandoff {
    pub source_path: String,
    pub source_sha256: Option<String>,
    pub decoder_backend: DecodedImageDecoderBackend,
    pub status: DecodedImageHandoffStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<i32>,
    pub input_profile: String,
    pub working_space: String,
    pub cache_identity: Option<DecodedImageCacheIdentity>,
    pub pixel_format: Option<DecodedImagePixelFormat>,
    pub message: String,
}

impl DecodedImageHandoff {
    pub fn contains_image_pixels(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPreviewArtifactRequest {
    pub fixture_class: String,
    pub probe: RawProbeResult,
    pub cache_key: String,
    pub output_path: PathBuf,
    pub max_edge: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPreviewArtifactResult {
    pub handoff: DecodedImageHandoff,
    pub artifact_path: Option<PathBuf>,
    pub bytes_written: Option<u64>,
    pub original_hash_unchanged: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFullResolutionExportSourceRequest {
    pub fixture_class: String,
    pub probe: RawProbeResult,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFullResolutionExportSourceResult {
    pub source_path: String,
    pub artifact_path: PathBuf,
    pub bytes_written: u64,
    pub source_sha256: String,
    pub artifact_sha256: String,
    pub decoder_backend: DecodedImageDecoderBackend,
    pub input_profile: String,
    pub working_space: String,
    pub pixel_format: DecodedImagePixelFormat,
    pub original_hash_unchanged: bool,
}

#[derive(Debug)]
pub enum RawPreviewArtifactError {
    OutputMatchesSource(PathBuf),
    SourceHashMismatch { expected: String, actual: String },
    InvalidRequest(String),
    CoreImageUnavailable(String),
    CoreImageWrite(String),
    Io(std::io::Error),
}

impl fmt::Display for RawPreviewArtifactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutputMatchesSource(path) => {
                write!(
                    formatter,
                    "RAW preview artifact output matches source: {}",
                    path.display()
                )
            }
            Self::SourceHashMismatch { expected, actual } => write!(
                formatter,
                "RAW preview artifact source hash mismatch: expected {expected}, actual {actual}"
            ),
            Self::InvalidRequest(message) => {
                write!(formatter, "invalid RAW preview request: {message}")
            }
            Self::CoreImageUnavailable(message) => {
                write!(formatter, "Core Image unavailable: {message}")
            }
            Self::CoreImageWrite(message) => {
                write!(formatter, "Core Image RAW preview write failed: {message}")
            }
            Self::Io(error) => write!(formatter, "RAW preview artifact filesystem error: {error}"),
        }
    }
}

impl Error for RawPreviewArtifactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::OutputMatchesSource(_)
            | Self::SourceHashMismatch { .. }
            | Self::InvalidRequest(_)
            | Self::CoreImageUnavailable(_)
            | Self::CoreImageWrite(_) => None,
        }
    }
}

impl From<std::io::Error> for RawPreviewArtifactError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug)]
pub enum RawFullResolutionExportSourceError {
    OutputMatchesSource(PathBuf),
    SourceHashMismatch { expected: String, actual: String },
    MissingSourceHash(String),
    UnsupportedFixtureClass(String),
    InvalidProbeEvidence(String),
    CoreImageUnavailable(String),
    CoreImageWrite(String),
    Io(std::io::Error),
}

impl fmt::Display for RawFullResolutionExportSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutputMatchesSource(path) => {
                write!(
                    formatter,
                    "RAW full-resolution export source output matches original source: {}",
                    path.display()
                )
            }
            Self::SourceHashMismatch { expected, actual } => write!(
                formatter,
                "RAW full-resolution export source hash mismatch: expected {expected}, actual {actual}"
            ),
            Self::MissingSourceHash(path) => {
                write!(
                    formatter,
                    "RAW full-resolution export requires fixture source SHA-256 evidence: {path}"
                )
            }
            Self::UnsupportedFixtureClass(fixture_class) => {
                write!(
                    formatter,
                    "RAW fixture class {fixture_class} is not enabled for full-resolution export"
                )
            }
            Self::InvalidProbeEvidence(message) => {
                write!(formatter, "invalid RAW export probe evidence: {message}")
            }
            Self::CoreImageUnavailable(message) => {
                write!(formatter, "Core Image unavailable: {message}")
            }
            Self::CoreImageWrite(message) => {
                write!(
                    formatter,
                    "Core Image RAW full-resolution export source write failed: {message}"
                )
            }
            Self::Io(error) => {
                write!(
                    formatter,
                    "RAW full-resolution export source filesystem error: {error}"
                )
            }
        }
    }
}

impl Error for RawFullResolutionExportSourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::OutputMatchesSource(_)
            | Self::SourceHashMismatch { .. }
            | Self::MissingSourceHash(_)
            | Self::UnsupportedFixtureClass(_)
            | Self::InvalidProbeEvidence(_)
            | Self::CoreImageUnavailable(_)
            | Self::CoreImageWrite(_) => None,
        }
    }
}

impl From<std::io::Error> for RawFullResolutionExportSourceError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<RawPreviewArtifactError> for RawFullResolutionExportSourceError {
    fn from(error: RawPreviewArtifactError) -> Self {
        match error {
            RawPreviewArtifactError::OutputMatchesSource(path) => Self::OutputMatchesSource(path),
            RawPreviewArtifactError::SourceHashMismatch { expected, actual } => {
                Self::SourceHashMismatch { expected, actual }
            }
            RawPreviewArtifactError::InvalidRequest(message) => Self::InvalidProbeEvidence(message),
            RawPreviewArtifactError::CoreImageUnavailable(message) => {
                Self::CoreImageUnavailable(message)
            }
            RawPreviewArtifactError::CoreImageWrite(message) => Self::CoreImageWrite(message),
            RawPreviewArtifactError::Io(error) => Self::Io(error),
        }
    }
}

pub fn write_raw_preview_artifact(
    request: RawPreviewArtifactRequest,
) -> Result<RawPreviewArtifactResult, RawPreviewArtifactError> {
    let source_path = PathBuf::from(&request.probe.source_path);
    if paths_refer_to_same_file(&source_path, &request.output_path) {
        return Err(RawPreviewArtifactError::OutputMatchesSource(
            request.output_path,
        ));
    }
    if request.max_edge == 0 {
        return Err(RawPreviewArtifactError::InvalidRequest(
            "max_edge must be greater than zero".to_string(),
        ));
    }
    let mut handoff = plan_decoded_image_handoff_from_raw_probe(
        &request.fixture_class,
        &request.probe,
        request.cache_key,
    );
    if handoff.status != DecodedImageHandoffStatus::Ready {
        return Ok(RawPreviewArtifactResult {
            handoff,
            artifact_path: None,
            bytes_written: None,
            original_hash_unchanged: None,
        });
    }
    if let (Some(width), Some(height)) = (request.probe.width, request.probe.height) {
        let (bounded_width, bounded_height) =
            bounded_preview_dimensions(width, height, request.max_edge);
        handoff.width = Some(bounded_width);
        handoff.height = Some(bounded_height);
    }
    handoff.input_profile = "core_image_raw".to_string();
    handoff.working_space = "srgb".to_string();
    handoff.pixel_format = Some(DecodedImagePixelFormat::JpegSrgb8);

    let write_result = core_image_raw_probe::write_core_image_raw_preview_artifact(
        &request.probe,
        &request.output_path,
        request.max_edge,
    )?;

    Ok(RawPreviewArtifactResult {
        handoff,
        artifact_path: Some(write_result.output_path),
        bytes_written: Some(write_result.bytes_written),
        original_hash_unchanged: Some(write_result.original_hash_unchanged),
    })
}

pub fn write_raw_full_resolution_export_source(
    request: RawFullResolutionExportSourceRequest,
) -> Result<RawFullResolutionExportSourceResult, RawFullResolutionExportSourceError> {
    let source_path = PathBuf::from(&request.probe.source_path);
    if paths_refer_to_same_file(&source_path, &request.output_path) {
        return Err(RawFullResolutionExportSourceError::OutputMatchesSource(
            request.output_path,
        ));
    }
    let raw_plan = plan_product_raw_decode_from_probe(&request.fixture_class, &request.probe);
    if raw_plan.status != ProductRawDecodeStatus::Supported {
        if !is_core_image_supported_fixture_class(request.fixture_class.trim()) {
            return Err(RawFullResolutionExportSourceError::UnsupportedFixtureClass(
                request.fixture_class,
            ));
        }
        return Err(RawFullResolutionExportSourceError::InvalidProbeEvidence(
            raw_plan.message,
        ));
    }
    let Some(expected_source_sha256) = request.probe.source_sha256.as_deref() else {
        return Err(RawFullResolutionExportSourceError::MissingSourceHash(
            request.probe.source_path,
        ));
    };

    let write_result = core_image_raw_probe::write_core_image_raw_full_resolution_export_source(
        &request.probe,
        &request.output_path,
    )?;
    if !expected_source_sha256.eq_ignore_ascii_case(&write_result.source_sha256) {
        return Err(RawFullResolutionExportSourceError::SourceHashMismatch {
            expected: expected_source_sha256.to_string(),
            actual: write_result.source_sha256,
        });
    }

    Ok(RawFullResolutionExportSourceResult {
        source_path: raw_plan.source_path,
        artifact_path: write_result.output_path,
        bytes_written: write_result.bytes_written,
        source_sha256: expected_source_sha256.to_string(),
        artifact_sha256: write_result.artifact_sha256,
        decoder_backend: DecodedImageDecoderBackend::CoreImageRaw,
        input_profile: "core_image_raw".to_string(),
        working_space: "srgb".to_string(),
        pixel_format: DecodedImagePixelFormat::JpegSrgb8,
        original_hash_unchanged: write_result.original_hash_unchanged,
    })
}

fn paths_refer_to_same_file(source_path: &Path, output_path: &Path) -> bool {
    if source_path == output_path {
        return true;
    }

    match (
        std::fs::canonicalize(source_path),
        std::fs::canonicalize(output_path),
    ) {
        (Ok(source_path), Ok(output_path)) => source_path == output_path,
        _ => false,
    }
}

pub fn plan_product_raw_decode(source_path: impl AsRef<str>) -> ProductRawDecodePlan {
    let source_path = source_path.as_ref().to_string();
    let extension = Path::new(&source_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");

    if !is_raw_candidate_extension(extension) {
        return ProductRawDecodePlan {
            source_path,
            backend: RawProbeBackend::CoreImageRaw,
            status: ProductRawDecodeStatus::BlockedUnsupportedClass,
            width: None,
            height: None,
            orientation: None,
            message: "Product RAW decode is blocked because the source is not a RAW candidate."
                .to_string(),
        };
    }

    ProductRawDecodePlan {
        source_path,
        backend: RawProbeBackend::CoreImageRaw,
        status: ProductRawDecodeStatus::BlockedPendingEvidence,
        width: None,
        height: None,
        orientation: None,
        message: "Product RAW decode is blocked until legal fixture probe evidence marks this class as supported.".to_string(),
    }
}

pub fn plan_product_raw_decode_from_probe(
    fixture_class: impl AsRef<str>,
    probe: &RawProbeResult,
) -> ProductRawDecodePlan {
    let fixture_class = fixture_class.as_ref().trim();

    if !is_core_image_supported_fixture_class(fixture_class) {
        return ProductRawDecodePlan {
            source_path: probe.source_path.clone(),
            backend: RawProbeBackend::CoreImageRaw,
            status: ProductRawDecodeStatus::BlockedPendingEvidence,
            width: None,
            height: None,
            orientation: None,
            message: "Product RAW decode remains blocked because this fixture class is not marked Core Image supported.".to_string(),
        };
    }

    if probe.status != RawProbeStatus::Success {
        return ProductRawDecodePlan {
            source_path: probe.source_path.clone(),
            backend: RawProbeBackend::CoreImageRaw,
            status: match probe.status {
                RawProbeStatus::Unsupported => ProductRawDecodeStatus::BlockedUnsupportedClass,
                _ => ProductRawDecodeStatus::BlockedCoreImageFailed,
            },
            width: probe.width,
            height: probe.height,
            orientation: probe.orientation,
            message: "Product RAW decode is blocked because the Core Image probe did not succeed."
                .to_string(),
        };
    }

    if probe.platform != RawProbePlatform::Macos || probe.source_sha256.is_none() {
        return ProductRawDecodePlan {
            source_path: probe.source_path.clone(),
            backend: RawProbeBackend::CoreImageRaw,
            status: ProductRawDecodeStatus::BlockedCoreImageFailed,
            width: probe.width,
            height: probe.height,
            orientation: probe.orientation,
            message: "Product RAW decode is blocked because the probe evidence is incomplete."
                .to_string(),
        };
    }

    if probe.width.is_none() || probe.height.is_none() {
        return ProductRawDecodePlan {
            source_path: probe.source_path.clone(),
            backend: RawProbeBackend::CoreImageRaw,
            status: ProductRawDecodeStatus::BlockedCoreImageFailed,
            width: probe.width,
            height: probe.height,
            orientation: probe.orientation,
            message: "Product RAW decode is blocked because the Core Image probe did not report dimensions.".to_string(),
        };
    }

    ProductRawDecodePlan {
        source_path: probe.source_path.clone(),
        backend: RawProbeBackend::CoreImageRaw,
        status: ProductRawDecodeStatus::Supported,
        width: probe.width,
        height: probe.height,
        orientation: probe.orientation,
        message: "Product RAW decode is supported for this fixture-backed Core Image class as a metadata-only plan.".to_string(),
    }
}

pub fn plan_decoded_image_handoff_from_raw_probe(
    fixture_class: impl AsRef<str>,
    probe: &RawProbeResult,
    cache_key: impl Into<String>,
) -> DecodedImageHandoff {
    let raw_plan = plan_product_raw_decode_from_probe(fixture_class, probe);
    let status = decoded_handoff_status(raw_plan.status);
    let ready = status == DecodedImageHandoffStatus::Ready;

    DecodedImageHandoff {
        source_path: raw_plan.source_path,
        source_sha256: probe.source_sha256.clone(),
        decoder_backend: DecodedImageDecoderBackend::CoreImageRaw,
        status,
        width: if ready { raw_plan.width } else { None },
        height: if ready { raw_plan.height } else { None },
        orientation: if ready { raw_plan.orientation } else { None },
        input_profile: "unknown".to_string(),
        working_space: "linear_display_p3".to_string(),
        cache_identity: if ready {
            Some(DecodedImageCacheIdentity::disposable(cache_key))
        } else {
            None
        },
        pixel_format: if ready {
            Some(DecodedImagePixelFormat::Rgba16Float)
        } else {
            None
        },
        message: raw_plan.message,
    }
}

fn decoded_handoff_status(status: ProductRawDecodeStatus) -> DecodedImageHandoffStatus {
    match status {
        ProductRawDecodeStatus::Supported => DecodedImageHandoffStatus::Ready,
        ProductRawDecodeStatus::BlockedPendingEvidence => {
            DecodedImageHandoffStatus::BlockedPendingEvidence
        }
        ProductRawDecodeStatus::BlockedCoreImageFailed => {
            DecodedImageHandoffStatus::BlockedCoreImageFailed
        }
        ProductRawDecodeStatus::BlockedUnsupportedClass => {
            DecodedImageHandoffStatus::BlockedUnsupportedClass
        }
    }
}

fn bounded_preview_dimensions(width: u32, height: u32, max_edge: u32) -> (u32, u32) {
    let longest_edge = width.max(height);
    if longest_edge <= max_edge {
        return (width, height);
    }

    let scale = max_edge as f64 / longest_edge as f64;
    let bounded_width = ((width as f64 * scale).round() as u32).max(1);
    let bounded_height = ((height as f64 * scale).round() as u32).max(1);
    (bounded_width, bounded_height)
}

fn is_core_image_supported_fixture_class(fixture_class: &str) -> bool {
    matches!(fixture_class, "A" | "B" | "C" | "D")
}

/// Build the local alpha preview decode plan for a catalog photo path.
pub fn plan_preview_decode(source_path: impl AsRef<str>, unsupported: bool) -> PreviewDecodePlan {
    let source_path = source_path.as_ref().to_string();
    if unsupported {
        return PreviewDecodePlan {
            source_path,
            backend: PreviewDecodeBackend::None,
            status: PreviewDecodeStatus::Unsupported,
            message: "Unsupported file type for SilicaRAW preview.".to_string(),
        };
    }

    let extension = Path::new(&source_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("");

    if is_raster_preview_extension(extension) {
        return PreviewDecodePlan {
            source_path,
            backend: PreviewDecodeBackend::Raster,
            status: PreviewDecodeStatus::Ready,
            message: "Raster preview can be opened by reference.".to_string(),
        };
    }

    if is_raw_candidate_extension(extension) {
        return PreviewDecodePlan {
            source_path,
            backend: PreviewDecodeBackend::CoreImageRaw,
            status: PreviewDecodeStatus::BlockedByMissingRawFixtureProbe,
            message: "Core Image RAW preview is selected but not implemented until fixture-backed probe coverage exists.".to_string(),
        };
    }

    PreviewDecodePlan {
        source_path,
        backend: PreviewDecodeBackend::None,
        status: PreviewDecodeStatus::Unsupported,
        message: "Unsupported file type for SilicaRAW preview.".to_string(),
    }
}

fn is_raster_preview_extension(extension: &str) -> bool {
    ["jpg", "jpeg"]
        .iter()
        .any(|supported| extension.eq_ignore_ascii_case(supported))
}

fn is_raw_candidate_extension(extension: &str) -> bool {
    [
        "arw", "cr2", "cr3", "dng", "nef", "orf", "raf", "rw2", "raw",
    ]
    .iter()
    .any(|supported| extension.eq_ignore_ascii_case(supported))
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-decode");
    }

    #[test]
    fn records_spike_002_decoder_gate() {
        assert_eq!(
            super::SPIKE_002_DECODER_GATE.path,
            super::RawDecoderPath::CoreImageRawPrimary
        );
        assert_eq!(
            super::SPIKE_002_DECODER_GATE.libraw_fallback,
            super::LibRawFallbackStatus::DeferredUntilFixtureGap
        );
        assert_eq!(
            super::SPIKE_002_DECODER_GATE.fixture_set,
            super::FixtureSetStatus::MissingLegalRawFixtures
        );
        assert_eq!(super::DECODER_BLOCKING_TAG, "decoder-blocking");
    }

    #[test]
    fn plans_preview_decode_readiness_by_file_type() {
        let jpeg_plan = super::plan_preview_decode("/tmp/sample.JPG", false);
        assert_eq!(jpeg_plan.status, super::PreviewDecodeStatus::Ready);
        assert_eq!(jpeg_plan.backend, super::PreviewDecodeBackend::Raster);
        assert_eq!(jpeg_plan.source_path, "/tmp/sample.JPG");

        let raw_plan = super::plan_preview_decode("/tmp/sample.dng", false);
        assert_eq!(
            raw_plan.status,
            super::PreviewDecodeStatus::BlockedByMissingRawFixtureProbe
        );
        assert_eq!(raw_plan.backend, super::PreviewDecodeBackend::CoreImageRaw);
        assert!(raw_plan
            .message
            .contains("Core Image RAW preview is selected but not implemented"));

        let unsupported_plan = super::plan_preview_decode("/tmp/notes.txt", true);
        assert_eq!(
            unsupported_plan.status,
            super::PreviewDecodeStatus::Unsupported
        );
        assert_eq!(unsupported_plan.backend, super::PreviewDecodeBackend::None);

        for source_path in ["/tmp/sample.png", "/tmp/sample.tiff", "/tmp/sample.heic"] {
            let unsupported_raster_plan = super::plan_preview_decode(source_path, false);
            assert_eq!(
                unsupported_raster_plan.status,
                super::PreviewDecodeStatus::Unsupported
            );
            assert_eq!(
                unsupported_raster_plan.backend,
                super::PreviewDecodeBackend::None
            );
        }
    }

    #[test]
    fn core_image_raw_probe_contract_does_not_change_preview_readiness() {
        let unavailable =
            super::probe_core_image_raw(super::RawProbeRequest::new("/tmp/missing.dng"));
        assert_eq!(unavailable.backend, super::RawProbeBackend::CoreImageRaw);
        assert!(matches!(
            unavailable.status,
            super::RawProbeStatus::Unavailable | super::RawProbeStatus::Failed
        ));

        let raw_plan = super::plan_preview_decode("/tmp/sample.dng", false);
        assert_eq!(
            raw_plan.status,
            super::PreviewDecodeStatus::BlockedByMissingRawFixtureProbe
        );
    }

    #[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
    #[test]
    fn core_image_raw_probe_reports_missing_files_on_macos_feature() {
        let path = unique_temp_probe_path("missing.dng");
        let _ = std::fs::remove_file(&path);

        let result =
            super::probe_core_image_raw(super::RawProbeRequest::new(path.to_string_lossy()));

        assert_eq!(result.platform, super::RawProbePlatform::Macos);
        assert_eq!(result.status, super::RawProbeStatus::Failed);
        assert_eq!(
            result.error_category,
            Some(super::RawProbeErrorCategory::MissingFile)
        );
        assert_eq!(result.source_sha256, None);
        assert_eq!(result.original_file_size, None);
    }

    #[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
    #[test]
    fn core_image_raw_probe_records_hash_mismatch_before_core_image_open() {
        let path = unique_temp_probe_path("hash-mismatch.dng");
        std::fs::write(&path, b"not a raw file\n").expect("write probe fixture");
        let before = std::fs::read(&path).expect("read probe fixture before probe");

        let result = super::probe_core_image_raw(super::RawProbeRequest {
            source_path: path.to_string_lossy().to_string(),
            expected_sha256: Some("00000000000000000000000000000000".to_string()),
        });

        let after = std::fs::read(&path).expect("read probe fixture after probe");
        let _ = std::fs::remove_file(&path);

        assert_eq!(before, after);
        assert_eq!(result.platform, super::RawProbePlatform::Macos);
        assert_eq!(result.status, super::RawProbeStatus::Failed);
        assert_eq!(
            result.error_category,
            Some(super::RawProbeErrorCategory::SourceHashMismatch)
        );
        assert_eq!(
            result.source_sha256.as_deref(),
            Some("8f48f233d3b6daa5e4735c8b695ec2754d9bffa3876e9ef5f541eef7b5e6c9fc")
        );
        assert_eq!(result.original_file_size, Some(15));
        assert_eq!(result.width, None);
        assert_eq!(result.height, None);
    }

    fn unique_temp_probe_path(label: &str) -> std::path::PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("silicaraw-core-image-raw-probe-{nonce}-{label}"))
    }

    #[cfg(feature = "core-image-raw-probe")]
    #[test]
    #[ignore]
    fn probes_raw_fixture_manifest_without_mutating_originals() {
        let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
            .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
        let report =
            super::probe_raw_fixture_manifest(manifest).expect("probe legal RAW fixture manifest");
        assert!(!report.results.is_empty());
        assert!(report
            .results
            .iter()
            .all(|result| result.original_hash_unchanged));
    }

    #[cfg(feature = "core-image-raw-probe")]
    #[test]
    fn raw_fixture_manifest_rejects_entries_that_cannot_be_legal_raw_probes() {
        let cases = [
            ("absolute-path", "\"/tmp/sample.dng\"", "\"raw\"", true),
            ("parent-dir", "\"../sample.dng\"", "\"raw\"", true),
            ("missing-hash", "\"sample.dng\"", "\"raw\"", false),
            ("non-raw-kind", "\"sample.dng\"", "\"tagged_raster\"", true),
        ];

        for (case, relative_path, kind, include_hash) in cases {
            let manifest_path = unique_temp_probe_path(&format!("{case}.json"));
            let expected_hashes = if include_hash {
                "\"sample.dng\": \"8f48f233d3b6daa5e4735c8b695ec2754d9bffa3876e9ef5f541eef7b5e6c9fc\""
            } else {
                ""
            };
            let manifest = format!(
                r#"{{
                  "schema": "silica.fixture_manifest",
                  "version": 1,
                  "manifest_kind": "raw-fixtures",
                  "expected_source_hashes": {{{expected_hashes}}},
                  "fixtures": [{{
                    "id": "{case}",
                    "class": "A",
                    "kind": {kind},
                    "relative_path": {relative_path},
                    "integrity": {{
                      "sha256": "8f48f233d3b6daa5e4735c8b695ec2754d9bffa3876e9ef5f541eef7b5e6c9fc"
                    }}
                  }}]
                }}"#
            );
            std::fs::write(&manifest_path, manifest).expect("write invalid manifest");

            let error = super::probe_raw_fixture_manifest(manifest_path.to_string_lossy())
                .expect_err("invalid fixture manifest should fail before probing");
            let _ = std::fs::remove_file(&manifest_path);

            assert!(
                matches!(
                    error,
                    super::RawFixtureProbeError::InvalidFixture { .. }
                        | super::RawFixtureProbeError::InvalidManifest { .. }
                ),
                "{case} returned {error:?}"
            );
        }
    }

    #[test]
    fn product_raw_decode_plan_blocks_until_fixture_evidence_exists() {
        let plan = super::plan_product_raw_decode("/tmp/sample.dng");

        assert_eq!(plan.source_path, "/tmp/sample.dng");
        assert_eq!(plan.backend, super::RawProbeBackend::CoreImageRaw);
        assert_ne!(plan.status, super::ProductRawDecodeStatus::Supported);
        assert_eq!(
            plan.status,
            super::ProductRawDecodeStatus::BlockedPendingEvidence
        );
        assert_eq!(plan.width, None);
        assert_eq!(plan.height, None);
        assert_eq!(plan.orientation, None);
    }

    #[test]
    fn product_raw_decode_plan_supports_successful_fixture_probe() {
        let probe = successful_raw_probe("/tmp/sample.cr3", Some(6960), Some(4640));
        let plan = super::plan_product_raw_decode_from_probe("B", &probe);

        assert_eq!(plan.source_path, "/tmp/sample.cr3");
        assert_eq!(plan.backend, super::RawProbeBackend::CoreImageRaw);
        assert_eq!(plan.status, super::ProductRawDecodeStatus::Supported);
        assert_eq!(plan.width, Some(6960));
        assert_eq!(plan.height, Some(4640));
        assert_eq!(plan.orientation, None);
    }

    #[test]
    fn decoded_image_handoff_records_supported_fixture_identity() {
        let probe = successful_raw_probe("/tmp/sample.cr2", Some(5184), Some(3456));
        let handoff =
            super::plan_decoded_image_handoff_from_raw_probe("A", &probe, "previews/raw/photo-1");

        assert_eq!(handoff.source_path, "/tmp/sample.cr2");
        assert_eq!(handoff.source_sha256.as_deref(), Some("fixture-hash"));
        assert_eq!(
            handoff.decoder_backend,
            super::DecodedImageDecoderBackend::CoreImageRaw
        );
        assert_eq!(handoff.status, super::DecodedImageHandoffStatus::Ready);
        assert_eq!(handoff.width, Some(5184));
        assert_eq!(handoff.height, Some(3456));
        assert_eq!(handoff.orientation, None);
        assert_eq!(handoff.input_profile, "unknown");
        assert_eq!(handoff.working_space, "linear_display_p3");
        assert_eq!(
            handoff.pixel_format,
            Some(super::DecodedImagePixelFormat::Rgba16Float)
        );
        let cache_identity = handoff.cache_identity.as_ref().expect("cache identity");
        assert_eq!(cache_identity.cache_key, "previews/raw/photo-1");
        assert!(cache_identity.disposable);
        assert!(!handoff.contains_image_pixels());
    }

    #[test]
    fn decoded_image_handoff_keeps_blocked_classes_without_cache_identity() {
        let probe = successful_raw_probe("/tmp/sample.raw", Some(1200), Some(800));
        let handoff =
            super::plan_decoded_image_handoff_from_raw_probe("E", &probe, "previews/raw/photo-e");

        assert_eq!(
            handoff.status,
            super::DecodedImageHandoffStatus::BlockedPendingEvidence
        );
        assert_eq!(handoff.source_sha256.as_deref(), Some("fixture-hash"));
        assert_eq!(handoff.width, None);
        assert_eq!(handoff.height, None);
        assert_eq!(handoff.cache_identity, None);
        assert_eq!(handoff.pixel_format, None);
        assert!(!handoff.contains_image_pixels());
    }

    #[test]
    fn raw_preview_artifact_refuses_to_write_over_original_source() {
        let probe = successful_raw_probe("/tmp/sample.cr2", Some(5184), Some(3456));

        let error = super::write_raw_preview_artifact(super::RawPreviewArtifactRequest {
            fixture_class: "A".to_string(),
            probe,
            cache_key: "raw-preview:test".to_string(),
            output_path: std::path::PathBuf::from("/tmp/sample.cr2"),
            max_edge: 2048,
        })
        .expect_err("preview artifact must not overwrite source");

        assert!(matches!(
            error,
            super::RawPreviewArtifactError::OutputMatchesSource(_)
        ));
    }

    #[test]
    fn raw_preview_artifact_refuses_canonical_source_output_match() {
        let source_path = unique_temp_probe_path("raw-preview-source.cr2");
        std::fs::write(&source_path, b"raw placeholder").expect("write source");
        let output_path = unique_temp_probe_path("raw-preview-source-link.cr2");
        std::os::unix::fs::symlink(&source_path, &output_path).expect("create source symlink");
        let probe = successful_raw_probe(&source_path.display().to_string(), Some(1200), Some(800));

        let error = super::write_raw_preview_artifact(super::RawPreviewArtifactRequest {
            fixture_class: "A".to_string(),
            probe,
            cache_key: "raw-preview:test".to_string(),
            output_path: output_path.clone(),
            max_edge: 2048,
        })
        .expect_err("canonical source/output match must be rejected");

        assert!(matches!(
            error,
            super::RawPreviewArtifactError::OutputMatchesSource(_)
        ));
        let _ = std::fs::remove_file(source_path);
        let _ = std::fs::remove_file(output_path);
    }

    #[test]
    fn raw_preview_artifact_keeps_unproven_classes_blocked() {
        let probe = successful_raw_probe("/tmp/sample.raw", Some(1200), Some(800));

        let result = super::write_raw_preview_artifact(super::RawPreviewArtifactRequest {
            fixture_class: "E".to_string(),
            probe,
            cache_key: "raw-preview:test".to_string(),
            output_path: std::path::PathBuf::from("/tmp/preview.jpg"),
            max_edge: 2048,
        })
        .expect("blocked handoff should be reviewable");

        assert_eq!(
            result.handoff.status,
            super::DecodedImageHandoffStatus::BlockedPendingEvidence
        );
        assert_eq!(result.artifact_path, None);
        assert_eq!(result.bytes_written, None);
        assert_eq!(result.original_hash_unchanged, None);
    }

    #[test]
    fn raw_full_resolution_export_source_rejects_unproven_classes_without_file() {
        let output_path = unique_temp_probe_path("raw-export-source.jpg");
        let probe = successful_raw_probe("/tmp/sample.raw", Some(1200), Some(800));

        let error = super::write_raw_full_resolution_export_source(
            super::RawFullResolutionExportSourceRequest {
                fixture_class: "E".to_string(),
                probe,
                output_path: output_path.clone(),
            },
        )
        .expect_err("unproven RAW fixture classes must not export");

        assert!(matches!(
            error,
            super::RawFullResolutionExportSourceError::UnsupportedFixtureClass(_)
        ));
        assert!(!output_path.exists());
    }

    #[test]
    fn raw_full_resolution_export_source_refuses_to_write_over_original_source() {
        let probe = successful_raw_probe("/tmp/sample.cr2", Some(5184), Some(3456));

        let error = super::write_raw_full_resolution_export_source(
            super::RawFullResolutionExportSourceRequest {
                fixture_class: "A".to_string(),
                probe,
                output_path: std::path::PathBuf::from("/tmp/sample.cr2"),
            },
        )
        .expect_err("RAW export source artifact must not overwrite original source");

        assert!(matches!(
            error,
            super::RawFullResolutionExportSourceError::OutputMatchesSource(_)
        ));
    }

    #[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
    #[test]
    #[ignore]
    fn writes_raw_preview_artifact_from_fixture_manifest_without_mutating_originals() {
        let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
            .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
        let report =
            super::probe_raw_fixture_manifest(manifest).expect("probe legal RAW fixture manifest");
        let required = report
            .results
            .iter()
            .find(|result| result.fixture_class == "A")
            .expect("Class A fixture evidence");
        let higher_risk = report
            .results
            .iter()
            .find(|result| matches!(result.fixture_class.as_str(), "C" | "D"))
            .expect("Class C or D fixture evidence");
        let output_root = unique_temp_probe_path("raw-preview-artifacts");
        std::fs::create_dir_all(&output_root).expect("create output root");

        for fixture in [required, higher_risk] {
            let output_path = output_root.join(format!("{}.jpg", fixture.fixture_id));
            let expected_cache_key = format!("raw-preview:{}", fixture.fixture_id);
            let result = super::write_raw_preview_artifact(super::RawPreviewArtifactRequest {
                fixture_class: fixture.fixture_class.clone(),
                probe: fixture.probe.clone(),
                cache_key: expected_cache_key.clone(),
                output_path: output_path.clone(),
                max_edge: 2048,
            })
            .expect("write RAW preview artifact");

            assert_eq!(
                result.handoff.status,
                super::DecodedImageHandoffStatus::Ready
            );
            assert_eq!(result.artifact_path.as_deref(), Some(output_path.as_path()));
            assert_eq!(
                result
                    .handoff
                    .cache_identity
                    .as_ref()
                    .map(|identity| identity.cache_key.as_str()),
                Some(expected_cache_key.as_str())
            );
            assert_eq!(
                result.handoff.pixel_format,
                Some(super::DecodedImagePixelFormat::JpegSrgb8)
            );
            assert_eq!(result.handoff.working_space, "srgb");
            assert!(result.handoff.width.unwrap_or_default() <= 2048);
            assert!(result.handoff.height.unwrap_or_default() <= 2048);
            assert!(result.bytes_written.unwrap_or_default() > 0);
            assert_eq!(result.original_hash_unchanged, Some(true));
            assert!(output_path.starts_with(&output_root));
            assert_ne!(
                output_path,
                std::path::PathBuf::from(&fixture.probe.source_path)
            );
            assert!(output_path.is_file());
            let _ = std::fs::remove_file(output_path);
        }
        let _ = std::fs::remove_dir_all(output_root);
    }

    #[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
    #[test]
    #[ignore]
    fn raw_preview_artifact_rejects_stale_probe_hash_before_writing() {
        let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
            .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
        let report =
            super::probe_raw_fixture_manifest(manifest).expect("probe legal RAW fixture manifest");
        let fixture = report
            .results
            .iter()
            .find(|result| result.fixture_class == "A")
            .expect("Class A fixture evidence");
        let mut stale_probe = fixture.probe.clone();
        stale_probe.source_sha256 = Some("stale-fixture-hash".to_string());
        let output_path = unique_temp_probe_path("stale-raw-preview.jpg");

        let error = super::write_raw_preview_artifact(super::RawPreviewArtifactRequest {
            fixture_class: fixture.fixture_class.clone(),
            probe: stale_probe,
            cache_key: "raw-preview:stale".to_string(),
            output_path: output_path.clone(),
            max_edge: 2048,
        })
        .expect_err("stale probe evidence must be rejected before writing");

        assert!(matches!(
            error,
            super::RawPreviewArtifactError::SourceHashMismatch { .. }
        ));
        assert!(!output_path.exists());
    }

    #[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
    #[test]
    #[ignore]
    fn writes_raw_full_resolution_export_source_from_fixture_manifest_without_preview_cache() {
        let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
            .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
        let report =
            super::probe_raw_fixture_manifest(manifest).expect("probe legal RAW fixture manifest");
        let fixture = report
            .results
            .iter()
            .find(|result| result.fixture_class == "A")
            .expect("Class A fixture evidence");
        let output_root = unique_temp_probe_path("raw-full-resolution-export-sources");
        std::fs::create_dir_all(&output_root).expect("create output root");
        let output_path = output_root.join(format!("{}-source.jpg", fixture.fixture_id));

        let result = super::write_raw_full_resolution_export_source(
            super::RawFullResolutionExportSourceRequest {
                fixture_class: fixture.fixture_class.clone(),
                probe: fixture.probe.clone(),
                output_path: output_path.clone(),
            },
        )
        .expect("write full-resolution RAW export source");

        assert_eq!(result.artifact_path, output_path);
        assert_eq!(
            result.source_sha256,
            fixture.probe.source_sha256.clone().unwrap()
        );
        assert_eq!(
            result.decoder_backend,
            super::DecodedImageDecoderBackend::CoreImageRaw
        );
        assert_eq!(result.input_profile, "core_image_raw");
        assert_eq!(result.working_space, "srgb");
        assert_eq!(
            result.pixel_format,
            super::DecodedImagePixelFormat::JpegSrgb8
        );
        assert!(result.bytes_written > 0);
        assert_eq!(result.artifact_sha256.len(), 64);
        assert!(result.original_hash_unchanged);
        assert!(output_path.is_file());
        assert!(!output_path
            .components()
            .any(|component| { component.as_os_str() == std::ffi::OsStr::new("previews") }));

        let _ = std::fs::remove_file(output_path);
        let _ = std::fs::remove_dir_all(output_root);
    }

    #[test]
    fn product_raw_decode_plan_keeps_unproven_fixture_classes_blocked() {
        let probe = successful_raw_probe("/tmp/sample.raw", Some(1200), Some(800));
        let plan = super::plan_product_raw_decode_from_probe("E", &probe);

        assert_eq!(
            plan.status,
            super::ProductRawDecodeStatus::BlockedPendingEvidence
        );
        assert_eq!(plan.width, None);
        assert_eq!(plan.height, None);
    }

    #[test]
    fn product_raw_decode_plan_blocks_failed_fixture_probe() {
        let mut probe = successful_raw_probe("/tmp/sample.raf", Some(6240), Some(4160));
        probe.status = super::RawProbeStatus::Failed;
        probe.error_category = Some(super::RawProbeErrorCategory::CoreImageOpenFailed);
        probe.message = "Core Image failed.".to_string();

        let plan = super::plan_product_raw_decode_from_probe("C", &probe);

        assert_eq!(
            plan.status,
            super::ProductRawDecodeStatus::BlockedCoreImageFailed
        );
        assert_eq!(plan.width, Some(6240));
        assert_eq!(plan.height, Some(4160));
    }

    #[test]
    fn product_raw_decode_plan_blocks_success_without_dimensions() {
        let probe = successful_raw_probe("/tmp/sample.dng", None, Some(3024));
        let plan = super::plan_product_raw_decode_from_probe("D", &probe);

        assert_eq!(
            plan.status,
            super::ProductRawDecodeStatus::BlockedCoreImageFailed
        );
        assert_eq!(plan.width, None);
        assert_eq!(plan.height, Some(3024));
    }

    #[test]
    fn product_raw_decode_plan_blocks_incomplete_probe_evidence() {
        let mut probe = successful_raw_probe("/tmp/sample.cr2", Some(5184), Some(3456));
        probe.source_sha256 = None;
        let plan = super::plan_product_raw_decode_from_probe("A", &probe);

        assert_eq!(
            plan.status,
            super::ProductRawDecodeStatus::BlockedCoreImageFailed
        );
    }

    #[test]
    fn product_raw_decode_plan_blocks_non_raw_candidates_as_unsupported_class() {
        let plan = super::plan_product_raw_decode("/tmp/sample.txt");

        assert_eq!(
            plan.status,
            super::ProductRawDecodeStatus::BlockedUnsupportedClass
        );
        assert_eq!(plan.width, None);
        assert_eq!(plan.height, None);
        assert_eq!(plan.orientation, None);
    }

    fn successful_raw_probe(
        source_path: &str,
        width: Option<u32>,
        height: Option<u32>,
    ) -> super::RawProbeResult {
        super::RawProbeResult {
            backend: super::RawProbeBackend::CoreImageRaw,
            platform: super::RawProbePlatform::Macos,
            macos_version: Some("26.4".to_string()),
            source_path: source_path.to_string(),
            source_sha256: Some("fixture-hash".to_string()),
            original_file_size: Some(1024),
            original_modified_at: Some("2026-06-12T00:00:00Z".to_string()),
            status: super::RawProbeStatus::Success,
            width,
            height,
            orientation: None,
            error_category: None,
            message: "Core Image opened the RAW source.".to_string(),
        }
    }
}
