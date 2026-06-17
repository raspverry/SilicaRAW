//! MLX feature boundary for SilicaRAW.
//!
//! ADR 0005 defers MLX from local alpha. This crate remains a boundary only:
//! no MLX dependency, model loading, inference, or AI behavior is present.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::fmt;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-mlx";

/// Stable model manifest schema marker required by `schemas/model_manifest.schema.json`.
pub const MODEL_MANIFEST_SCHEMA: &str = "silica.model";

/// Stable model manifest schema version for v0.1.
pub const MODEL_MANIFEST_VERSION: i64 = 1;

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

/// Model task types accepted by the model manifest schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MlxModelTaskType {
    SubjectMask,
    SkyMask,
    BlurScore,
    QualityScore,
    DuplicateGrouping,
    AutoTone,
    Denoise,
    Upscale,
}

impl MlxModelTaskType {
    fn from_schema_value(value: &str) -> Option<Self> {
        match value {
            "subject_mask" => Some(Self::SubjectMask),
            "sky_mask" => Some(Self::SkyMask),
            "blur_score" => Some(Self::BlurScore),
            "quality_score" => Some(Self::QualityScore),
            "duplicate_grouping" => Some(Self::DuplicateGrouping),
            "auto_tone" => Some(Self::AutoTone),
            "denoise" => Some(Self::Denoise),
            "upscale" => Some(Self::Upscale),
            _ => None,
        }
    }
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

/// Task 24.2 model manifest policy without enabling model loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxModelManifestPolicy {
    pub models_optional: bool,
    pub require_manifest_before_enablement: bool,
}

/// Task 24.3 AI result storage policy without enabling inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxAiResultPolicy {
    pub local_only: bool,
    pub unapproved_by_default: bool,
    pub loads_model: bool,
    pub mutates_edit_graph: bool,
}

/// Task 24.4 first AI review policy without enabling inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MlxAiReviewPolicy {
    pub task_type: MlxModelTaskType,
    pub model_optional: bool,
    pub editor_usable_without_model: bool,
    pub review_information_only: bool,
    pub loads_model: bool,
    pub mutates_edit_graph: bool,
    pub mutates_originals: bool,
}

/// Parsed and validated model manifest metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlxModelManifest {
    pub model_id: String,
    pub model_version: String,
    pub task_type: MlxModelTaskType,
    pub license: String,
    pub source: String,
    pub file_hash: String,
    pub minimum_silica_version: String,
}

/// Error returned by model manifest validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MlxModelManifestValidationError {
    field: Option<&'static str>,
    message: String,
}

impl MlxModelManifestValidationError {
    fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field: Some(field),
            message: message.into(),
        }
    }

    /// Returns the schema field that failed validation when available.
    pub const fn field(&self) -> Option<&'static str> {
        self.field
    }
}

impl fmt::Display for MlxModelManifestValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.field {
            Some(field) => write!(f, "{}: {}", field, self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl std::error::Error for MlxModelManifestValidationError {}

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

/// Task 24.2 manifest gate decision for downstream crates and tests.
pub const TASK_24_2_MODEL_MANIFEST_POLICY: MlxModelManifestPolicy = MlxModelManifestPolicy {
    models_optional: true,
    require_manifest_before_enablement: true,
};

/// Task 24.3 AI result policy for downstream crates and tests.
pub const TASK_24_3_AI_RESULT_POLICY: MlxAiResultPolicy = MlxAiResultPolicy {
    local_only: true,
    unapproved_by_default: true,
    loads_model: false,
    mutates_edit_graph: false,
};

/// Task 24.4 first review policy for downstream crates and tests.
pub const TASK_24_4_AI_REVIEW_POLICY: MlxAiReviewPolicy = MlxAiReviewPolicy {
    task_type: MlxModelTaskType::BlurScore,
    model_optional: true,
    editor_usable_without_model: true,
    review_information_only: true,
    loads_model: false,
    mutates_edit_graph: false,
    mutates_originals: false,
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

/// Computes the deterministic SHA-256 manifest hash for model bytes.
pub fn compute_model_sha256_hash(model_bytes: &[u8]) -> String {
    let digest = Sha256::digest(model_bytes);
    format!("sha256:{digest:x}")
}

/// Validates a model manifest before a model can be enabled.
pub fn validate_model_manifest_json(
    manifest_json: &str,
    model_bytes: &[u8],
) -> Result<MlxModelManifest, MlxModelManifestValidationError> {
    let value: Value = serde_json::from_str(manifest_json).map_err(|error| {
        MlxModelManifestValidationError::new("manifest", format!("invalid JSON: {error}"))
    })?;
    let object = value
        .as_object()
        .ok_or_else(|| MlxModelManifestValidationError::new("manifest", "expected object"))?;

    reject_unknown_top_level_fields(object)?;

    let schema = required_string(object, "schema")?;
    if schema != MODEL_MANIFEST_SCHEMA {
        return Err(MlxModelManifestValidationError::new(
            "schema",
            "expected silica.model",
        ));
    }

    let version = object
        .get("version")
        .and_then(Value::as_i64)
        .ok_or_else(|| MlxModelManifestValidationError::new("version", "missing integer"))?;
    if version != MODEL_MANIFEST_VERSION {
        return Err(MlxModelManifestValidationError::new(
            "version",
            "unsupported manifest version",
        ));
    }

    let model_id = required_string(object, "model_id")?;
    let model_version = required_string(object, "model_version")?;
    let task_type_value = required_string(object, "task_type")?;
    let task_type = MlxModelTaskType::from_schema_value(task_type_value).ok_or_else(|| {
        MlxModelManifestValidationError::new("task_type", "unsupported model task type")
    })?;
    let license = required_string(object, "license")?;
    let source = required_string(object, "source")?;
    let file_hash = required_string(object, "file_hash")?;
    let minimum_silica_version = required_string(object, "minimum_silica_version")?;

    if !is_prefixed_sha256(file_hash) {
        return Err(MlxModelManifestValidationError::new(
            "file_hash",
            "expected sha256:<64 lowercase hex chars>",
        ));
    }

    require_nested_non_empty_object(object, "input", "preprocessing", "input.preprocessing")?;
    require_nested_non_empty_object(object, "output", "metadata", "output.metadata")?;

    let actual_hash = compute_model_sha256_hash(model_bytes);
    if file_hash != actual_hash {
        return Err(MlxModelManifestValidationError::new(
            "file_hash",
            format!("hash mismatch: manifest {file_hash}, actual {actual_hash}"),
        ));
    }

    Ok(MlxModelManifest {
        model_id: model_id.to_string(),
        model_version: model_version.to_string(),
        task_type,
        license: license.to_string(),
        source: source.to_string(),
        file_hash: file_hash.to_string(),
        minimum_silica_version: minimum_silica_version.to_string(),
    })
}

fn reject_unknown_top_level_fields(
    object: &Map<String, Value>,
) -> Result<(), MlxModelManifestValidationError> {
    const ALLOWED_FIELDS: &[&str] = &[
        "schema",
        "version",
        "model_id",
        "model_version",
        "task_type",
        "license",
        "source",
        "file_hash",
        "input",
        "output",
        "minimum_silica_version",
    ];

    for key in object.keys() {
        if !ALLOWED_FIELDS.contains(&key.as_str()) {
            return Err(MlxModelManifestValidationError::new(
                "additionalProperties",
                format!("unknown field {key}"),
            ));
        }
    }

    Ok(())
}

fn required_string<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a str, MlxModelManifestValidationError> {
    let value = object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| MlxModelManifestValidationError::new(field, "missing string"))?;
    if value.trim().is_empty() {
        return Err(MlxModelManifestValidationError::new(
            field,
            "must not be empty",
        ));
    }
    Ok(value)
}

fn require_nested_non_empty_object(
    object: &Map<String, Value>,
    parent: &'static str,
    child: &'static str,
    field: &'static str,
) -> Result<(), MlxModelManifestValidationError> {
    let parent = object
        .get(parent)
        .and_then(Value::as_object)
        .ok_or_else(|| MlxModelManifestValidationError::new(field, "missing object"))?;
    let child = parent
        .get(child)
        .and_then(Value::as_object)
        .ok_or_else(|| MlxModelManifestValidationError::new(field, "missing object"))?;
    if child.is_empty() {
        return Err(MlxModelManifestValidationError::new(
            field,
            "must not be empty",
        ));
    }
    Ok(())
}

fn is_prefixed_sha256(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .chars()
            .all(|char| char.is_ascii_hexdigit() && !char.is_ascii_uppercase())
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

    fn valid_model_manifest(file_hash: &str) -> String {
        format!(
            r#"{{
                "schema": "silica.model",
                "version": 1,
                "model_id": "silicaraw.blur-review.test",
                "model_version": "0.1.0",
                "task_type": "blur_score",
                "license": "MIT-test-fixture",
                "source": "silicaraw-test-fixtures/blur-review-test",
                "file_hash": "{file_hash}",
                "input": {{
                    "preprocessing": {{
                        "color_space": "srgb",
                        "resize": "long_edge_1024"
                    }}
                }},
                "output": {{
                    "metadata": {{
                        "score_kind": "blur",
                        "range": "0..1"
                    }}
                }},
                "minimum_silica_version": "0.1.0"
            }}"#
        )
    }

    #[test]
    fn validates_model_manifest_required_fields_and_model_hash() {
        let model_bytes = b"silicaraw-model-fixture";
        let manifest = valid_model_manifest(
            "sha256:592ecb9eeca3e1fa28682c00e710ec7d386e790c54cf9b69cb38d2e493f86683",
        );

        let parsed = super::validate_model_manifest_json(&manifest, model_bytes)
            .expect("valid manifest and bytes");

        assert_eq!(parsed.model_id, "silicaraw.blur-review.test");
        assert_eq!(parsed.task_type, super::MlxModelTaskType::BlurScore);
        assert_eq!(
            parsed.file_hash,
            "sha256:592ecb9eeca3e1fa28682c00e710ec7d386e790c54cf9b69cb38d2e493f86683"
        );
        assert!(super::TASK_24_2_MODEL_MANIFEST_POLICY.models_optional);
    }

    #[test]
    fn rejects_model_manifest_missing_license_source_hash_preprocessing_or_output_metadata() {
        let model_bytes = b"silicaraw-model-fixture";

        for (field, needle) in [
            ("license", r#""license": "MIT-test-fixture","#),
            (
                "source",
                r#""source": "silicaraw-test-fixtures/blur-review-test","#,
            ),
            (
                "file_hash",
                r#""file_hash": "sha256:592ecb9eeca3e1fa28682c00e710ec7d386e790c54cf9b69cb38d2e493f86683","#,
            ),
            (
                "input.preprocessing",
                r#""preprocessing": {
                        "color_space": "srgb",
                        "resize": "long_edge_1024"
                    }"#,
            ),
            (
                "output.metadata",
                r#""metadata": {
                        "score_kind": "blur",
                        "range": "0..1"
                    }"#,
            ),
        ] {
            let manifest = valid_model_manifest(
                "sha256:592ecb9eeca3e1fa28682c00e710ec7d386e790c54cf9b69cb38d2e493f86683",
            )
            .replace(needle, "");

            let err = super::validate_model_manifest_json(&manifest, model_bytes)
                .expect_err("missing manifest field rejected");
            assert_eq!(err.field(), Some(field));
        }
    }

    #[test]
    fn rejects_model_manifest_hash_mismatch_deterministically() {
        let model_bytes = b"silicaraw-model-fixture";
        let manifest = valid_model_manifest(
            "sha256:0000000000000000000000000000000000000000000000000000000000000000",
        );

        let err = super::validate_model_manifest_json(&manifest, model_bytes)
            .expect_err("hash mismatch rejected");

        assert_eq!(err.field(), Some("file_hash"));
        assert_eq!(
            super::compute_model_sha256_hash(model_bytes),
            "sha256:592ecb9eeca3e1fa28682c00e710ec7d386e790c54cf9b69cb38d2e493f86683"
        );
        assert_eq!(
            super::compute_model_sha256_hash(model_bytes),
            super::compute_model_sha256_hash(model_bytes)
        );
    }

    #[test]
    fn records_task_24_3_ai_result_storage_policy_without_inference() {
        let policy = super::TASK_24_3_AI_RESULT_POLICY;

        assert!(policy.local_only);
        assert!(policy.unapproved_by_default);
        assert!(!policy.loads_model);
        assert!(!policy.mutates_edit_graph);
    }

    #[test]
    fn records_task_24_4_first_review_feature_as_non_mutating_and_optional() {
        let policy = super::TASK_24_4_AI_REVIEW_POLICY;

        assert_eq!(policy.task_type, super::MlxModelTaskType::BlurScore);
        assert!(policy.model_optional);
        assert!(policy.editor_usable_without_model);
        assert!(policy.review_information_only);
        assert!(!policy.loads_model);
        assert!(!policy.mutates_edit_graph);
        assert!(!policy.mutates_originals);
    }
}
