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
}
