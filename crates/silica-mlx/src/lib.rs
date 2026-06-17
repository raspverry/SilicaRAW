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

/// Provisional runtime binding choice recorded by Task 24.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxRuntimeBindingChoice {
    /// Future probe should prefer the official MLX C API from a Rust feature gate.
    MlxCBehindFutureFeatureGate,
}

/// Product behavior when no local model is installed or enabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxNoModelBehavior {
    /// AI features stay unavailable while the non-AI editor remains usable.
    DisableAiFeaturesKeepEditorUsable,
}

/// Model packaging policy recorded before model validation work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxModelPackagingPolicy {
    /// Bundled or user-installed models require a manifest before enablement.
    ManifestRequiredNoBundledWeights,
}

/// Memory behavior selected for the first future runtime probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxMemoryPolicy {
    /// Treat MLX unified memory as app-global pressure and bound background work.
    UnifiedMemoryWithBoundedWorkerQueue,
}

/// Cancellation behavior selected for the first future runtime probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxCancellationPolicy {
    /// Cancellation is cooperative at queued task boundaries, not a hard kernel kill.
    CooperativeTaskBoundaryCancellation,
}

/// Recorded MLX local-alpha decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxGate {
    pub local_alpha: MlxLocalAlphaStatus,
    pub runtime: MlxRuntimeStatus,
}

/// Task 24.1 runtime spike result without enabling a runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxRuntimeSpike {
    pub binding_choice: MlxRuntimeBindingChoice,
    pub runtime_dependency: MlxRuntimeStatus,
    pub no_model_behavior: MlxNoModelBehavior,
    pub model_packaging: MlxModelPackagingPolicy,
    pub memory: MlxMemoryPolicy,
    pub cancellation: MlxCancellationPolicy,
}

/// ADR 0005 decision for downstream crates and tests.
pub const ADR_0005_MLX_GATE: MlxGate = MlxGate {
    local_alpha: MlxLocalAlphaStatus::Deferred,
    runtime: MlxRuntimeStatus::BoundaryOnly,
};

/// Task 24.1 spike decision for downstream crates and tests.
pub const TASK_24_1_MLX_RUNTIME_SPIKE: MlxRuntimeSpike = MlxRuntimeSpike {
    binding_choice: MlxRuntimeBindingChoice::MlxCBehindFutureFeatureGate,
    runtime_dependency: MlxRuntimeStatus::BoundaryOnly,
    no_model_behavior: MlxNoModelBehavior::DisableAiFeaturesKeepEditorUsable,
    model_packaging: MlxModelPackagingPolicy::ManifestRequiredNoBundledWeights,
    memory: MlxMemoryPolicy::UnifiedMemoryWithBoundedWorkerQueue,
    cancellation: MlxCancellationPolicy::CooperativeTaskBoundaryCancellation,
};

impl MlxRuntimeSpike {
    /// Returns true when a model manifest is required before any model can be enabled.
    pub const fn requires_manifest_before_model_enablement(self) -> bool {
        matches!(
            self.model_packaging,
            MlxModelPackagingPolicy::ManifestRequiredNoBundledWeights
        )
    }

    /// Returns true when this spike leaves the crate dependency-free.
    pub const fn adds_runtime_dependency(self) -> bool {
        !matches!(self.runtime_dependency, MlxRuntimeStatus::BoundaryOnly)
    }
}

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

    #[test]
    fn records_task_24_1_runtime_spike_without_enabling_mlx() {
        let spike = super::TASK_24_1_MLX_RUNTIME_SPIKE;

        assert_eq!(
            spike.binding_choice,
            super::MlxRuntimeBindingChoice::MlxCBehindFutureFeatureGate
        );
        assert_eq!(
            spike.no_model_behavior,
            super::MlxNoModelBehavior::DisableAiFeaturesKeepEditorUsable
        );
        assert!(!spike.adds_runtime_dependency());
    }

    #[test]
    fn records_task_24_1_model_manifest_packaging_gate() {
        let spike = super::TASK_24_1_MLX_RUNTIME_SPIKE;

        assert!(spike.requires_manifest_before_model_enablement());
        assert_eq!(
            spike.model_packaging,
            super::MlxModelPackagingPolicy::ManifestRequiredNoBundledWeights
        );
    }
}
