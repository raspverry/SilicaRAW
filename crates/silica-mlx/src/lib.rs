//! MLX feature boundary for SilicaRAW.
//!
//! ADR 0005 defers MLX from local alpha. This crate remains a boundary only:
//! no MLX dependency, model loading, inference, or AI behavior is present.

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-mlx";

/// MLX status for the local DMG alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxLocalAlphaStatus {
    /// MLX is intentionally deferred from local alpha.
    Deferred,
}

/// Runtime implementation status for this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxRuntimeStatus {
    /// The crate is an architecture boundary only.
    BoundaryOnly,
}

/// Recorded MLX local-alpha decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxGate {
    pub local_alpha: MlxLocalAlphaStatus,
    pub runtime: MlxRuntimeStatus,
}

/// ADR 0005 decision for downstream crates and tests.
pub const ADR_0005_MLX_GATE: MlxGate = MlxGate {
    local_alpha: MlxLocalAlphaStatus::Deferred,
    runtime: MlxRuntimeStatus::BoundaryOnly,
};

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-mlx");
    }

    #[test]
    fn records_adr_0005_mlx_deferral() {
        assert_eq!(
            super::ADR_0005_MLX_GATE.local_alpha,
            super::MlxLocalAlphaStatus::Deferred
        );
        assert_eq!(
            super::ADR_0005_MLX_GATE.runtime,
            super::MlxRuntimeStatus::BoundaryOnly
        );
    }
}
