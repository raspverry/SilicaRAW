//! RAW decode abstraction boundary for SilicaRAW.
//!
//! Spike 002 records the decoder path gate. This crate still does not decode RAW
//! files or link decoder backends.

use std::path::Path;

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
}
