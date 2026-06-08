//! Render request and renderer boundary for SilicaRAW.
//!
//! Spike 003 records the color-managed preview/export gate. This crate still
//! does not render images, apply color transforms, or export files.

use silica_decode::{PreviewDecodePlan, PreviewDecodeStatus};

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-render";

/// Color-management path selected by the preview/export spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorManagementPath {
    /// Use Core Image/ColorSync-compatible color management first.
    CoreImageColorManagementPrimary,
}

/// Working color space selected for the first implementation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkingColorSpace {
    /// Linear Display P3-compatible wide-gamut RGB.
    LinearDisplayP3,
}

/// Preview color behavior selected by the spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewColorBehavior {
    /// Convert from working space to the active display color space.
    DisplayProfileAware,
}

/// Export color behavior selected by the spike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportColorBehavior {
    /// Export sRGB by default and support Display P3 when explicitly selected.
    SrgbDefaultDisplayP3Supported,
}

/// Status of color-management fixtures in the repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFixtureStatus {
    /// Tagged sRGB and Display P3 raster fixtures are not committed yet.
    MissingTaggedRasterFixtures,
}

/// Recorded output of Spike 003.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorGate {
    pub path: ColorManagementPath,
    pub working_space: WorkingColorSpace,
    pub preview: PreviewColorBehavior,
    pub export: ExportColorBehavior,
    pub fixture_set: ColorFixtureStatus,
}

/// Spike 003 decision for downstream crates and tests.
pub const SPIKE_003_COLOR_GATE: ColorGate = ColorGate {
    path: ColorManagementPath::CoreImageColorManagementPrimary,
    working_space: WorkingColorSpace::LinearDisplayP3,
    preview: PreviewColorBehavior::DisplayProfileAware,
    export: ExportColorBehavior::SrgbDefaultDisplayP3Supported,
    fixture_set: ColorFixtureStatus::MissingTaggedRasterFixtures,
};

/// Tag used in docs and future issue labels for color-dependent work.
pub const COLOR_BLOCKING_TAG: &str = "color-blocking";

/// Render readiness state for the local alpha preview path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewRenderStatus {
    /// Preview source can be opened by the current minimal surface.
    Ready,
    /// Preview is blocked before rendering because decode is not ready.
    BlockedByDecode,
    /// Preview is unsupported for this catalog entry.
    Unsupported,
}

/// Render-side preview plan. This is a routing contract, not a Metal viewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewRenderPlan {
    pub source_path: String,
    pub status: PreviewRenderStatus,
    pub color_behavior: PreviewColorBehavior,
    pub message: String,
}

/// Render request for a draft exposure/contrast preview update.
#[derive(Debug, Clone, PartialEq)]
pub struct ExposureContrastPreviewRequest {
    pub source_path: String,
    pub status: PreviewRenderStatus,
    pub color_behavior: PreviewColorBehavior,
    pub exposure: f64,
    pub contrast: f64,
    pub message: String,
}

/// Build a local alpha render plan from a decode plan.
pub fn plan_preview_render(decode_plan: PreviewDecodePlan) -> PreviewRenderPlan {
    let status = match decode_plan.status {
        PreviewDecodeStatus::Ready => PreviewRenderStatus::Ready,
        PreviewDecodeStatus::Unsupported => PreviewRenderStatus::Unsupported,
        PreviewDecodeStatus::BlockedByMissingRawFixtureProbe => {
            PreviewRenderStatus::BlockedByDecode
        }
    };

    let message = match status {
        PreviewRenderStatus::Ready => {
            "Preview source is ready for a display-profile-aware surface.".to_string()
        }
        PreviewRenderStatus::Unsupported | PreviewRenderStatus::BlockedByDecode => {
            decode_plan.message
        }
    };

    PreviewRenderPlan {
        source_path: decode_plan.source_path,
        status,
        color_behavior: SPIKE_003_COLOR_GATE.preview,
        message,
    }
}

/// Build a render request for a draft exposure/contrast preview update.
pub fn plan_exposure_contrast_preview(
    preview_plan: PreviewRenderPlan,
    exposure: f64,
    contrast: f64,
) -> ExposureContrastPreviewRequest {
    let message = match preview_plan.status {
        PreviewRenderStatus::Ready => {
            "Draft exposure/contrast preview request is ready.".to_string()
        }
        PreviewRenderStatus::BlockedByDecode | PreviewRenderStatus::Unsupported => {
            preview_plan.message.clone()
        }
    };

    ExposureContrastPreviewRequest {
        source_path: preview_plan.source_path,
        status: preview_plan.status,
        color_behavior: preview_plan.color_behavior,
        exposure,
        contrast,
        message,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_crate_name() {
        assert_eq!(super::CRATE_NAME, "silica-render");
    }

    #[test]
    fn records_spike_003_color_gate() {
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.path,
            super::ColorManagementPath::CoreImageColorManagementPrimary
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.working_space,
            super::WorkingColorSpace::LinearDisplayP3
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.preview,
            super::PreviewColorBehavior::DisplayProfileAware
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.export,
            super::ExportColorBehavior::SrgbDefaultDisplayP3Supported
        );
        assert_eq!(
            super::SPIKE_003_COLOR_GATE.fixture_set,
            super::ColorFixtureStatus::MissingTaggedRasterFixtures
        );
        assert_eq!(super::COLOR_BLOCKING_TAG, "color-blocking");
    }

    #[test]
    fn plans_display_aware_preview_from_decode_plan() {
        let decode_plan = silica_decode::plan_preview_decode("/tmp/sample.jpg", false);
        let render_plan = super::plan_preview_render(decode_plan);

        assert_eq!(render_plan.status, super::PreviewRenderStatus::Ready);
        assert_eq!(render_plan.source_path, "/tmp/sample.jpg");
        assert_eq!(
            render_plan.color_behavior,
            super::PreviewColorBehavior::DisplayProfileAware
        );
        assert!(render_plan.message.contains("display-profile-aware"));

        let raw_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.dng",
            false,
        ));
        assert_eq!(raw_plan.status, super::PreviewRenderStatus::BlockedByDecode);
        assert!(raw_plan.message.contains("Core Image RAW preview"));

        let unsupported_plan =
            super::plan_preview_render(silica_decode::plan_preview_decode("/tmp/notes.txt", true));
        assert_eq!(
            unsupported_plan.status,
            super::PreviewRenderStatus::Unsupported
        );
    }

    #[test]
    fn plans_exposure_contrast_preview_request_from_ready_preview() {
        let preview_plan = super::plan_preview_render(silica_decode::plan_preview_decode(
            "/tmp/sample.jpg",
            false,
        ));

        let request = super::plan_exposure_contrast_preview(preview_plan, 0.5, -8.0);

        assert_eq!(request.status, super::PreviewRenderStatus::Ready);
        assert_eq!(request.source_path, "/tmp/sample.jpg");
        assert_eq!(request.exposure, 0.5);
        assert_eq!(request.contrast, -8.0);
        assert_eq!(
            request.color_behavior,
            super::PreviewColorBehavior::DisplayProfileAware
        );
        assert!(request.message.contains("exposure/contrast"));
    }
}
