//! RAW decode abstraction boundary for SilicaRAW.
//!
//! Spike 002 records the decoder path gate. This crate still does not decode RAW
//! files or link decoder backends.

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
}
