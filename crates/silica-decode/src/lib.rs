//! RAW decode abstraction boundary for SilicaRAW.
//!
//! Spike 002 records the decoder path gate. This crate still does not decode RAW
//! files or link decoder backends.

use std::path::Path;

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

    PreviewDecodePlan {
        source_path,
        backend: PreviewDecodeBackend::CoreImageRaw,
        status: PreviewDecodeStatus::BlockedByMissingRawFixtureProbe,
        message: "Core Image RAW preview is selected but not implemented until fixture-backed probe coverage exists.".to_string(),
    }
}

fn is_raster_preview_extension(extension: &str) -> bool {
    ["jpg", "jpeg", "png", "heic", "tif", "tiff"]
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

    #[cfg(feature = "core-image-raw-probe")]
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
