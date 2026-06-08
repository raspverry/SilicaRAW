//! Render request and renderer boundary for SilicaRAW.
//!
//! Spike 003 records the color-managed preview/export gate. This crate still
//! does not render images, apply color transforms, or export files.

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
}
