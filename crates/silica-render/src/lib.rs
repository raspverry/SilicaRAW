//! Render request and renderer boundary for SilicaRAW.
//!
//! Spike 003 records the color-managed preview/export gate. This crate still
//! does not render images, apply color transforms, or export files.

#[cfg(feature = "color-probe")]
use std::fs;
#[cfg(feature = "color-probe")]
use std::path::PathBuf;

#[cfg(feature = "color-probe")]
use sha2::{Digest, Sha256};

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

/// Render-side export request for the local alpha JPEG sRGB path.
#[derive(Debug, Clone, PartialEq)]
pub struct JpegSrgbExportRenderRequest {
    pub source_path: String,
    pub output_path: String,
    pub color_behavior: ExportColorBehavior,
    pub exposure: f64,
    pub contrast: f64,
    pub quality: u8,
    pub message: String,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorProbeRequest {
    pub source_path: String,
}

#[cfg(feature = "color-probe")]
impl ColorProbeRequest {
    pub fn new(source_path: impl AsRef<str>) -> Self {
        Self {
            source_path: source_path.as_ref().to_string(),
        }
    }
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbePlatform {
    Macos,
    UnsupportedPlatform,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeStatus {
    Success,
    Failed,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeInputProfile {
    Srgb,
    DisplayP3,
    None,
    Unknown,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeOutputProfile {
    Srgb,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeTransformPath {
    EmbeddedIccToLinearDisplayP3ToSrgb,
    AssumeSrgbToLinearDisplayP3ToSrgb,
    Unavailable,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorProbeErrorCategory {
    UnsupportedPlatform,
    MissingFile,
    NotAFile,
    ReadFailed,
    InvalidJpeg,
}

#[cfg(feature = "color-probe")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorProbeResult {
    pub platform: ColorProbePlatform,
    pub source_path: String,
    pub source_sha256: Option<String>,
    pub status: ColorProbeStatus,
    pub input_profile: ColorProbeInputProfile,
    pub embedded_icc: bool,
    pub working_space: WorkingColorSpace,
    pub output_profile: ColorProbeOutputProfile,
    pub transform_path: ColorProbeTransformPath,
    pub error_category: Option<ColorProbeErrorCategory>,
    pub message: String,
}

#[cfg(feature = "color-probe")]
pub fn probe_color_profile(request: ColorProbeRequest) -> ColorProbeResult {
    let source_path = request.source_path;
    let path = PathBuf::from(&source_path);
    let platform = current_color_probe_platform();

    if platform == ColorProbePlatform::UnsupportedPlatform {
        return failed_color_probe(
            platform,
            source_path,
            None,
            ColorProbeErrorCategory::UnsupportedPlatform,
            "Color probe is available only on macOS for Phase 13 proof work.",
        );
    }

    let metadata = match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => metadata,
        Ok(_) => {
            return failed_color_probe(
                platform,
                source_path,
                None,
                ColorProbeErrorCategory::NotAFile,
                "Color probe source is not a file.",
            );
        }
        Err(error) => {
            let category = if error.kind() == std::io::ErrorKind::NotFound {
                ColorProbeErrorCategory::MissingFile
            } else {
                ColorProbeErrorCategory::ReadFailed
            };
            return failed_color_probe(
                platform,
                source_path,
                None,
                category,
                "Color probe source could not be read.",
            );
        }
    };

    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return failed_color_probe(
                platform,
                source_path,
                None,
                ColorProbeErrorCategory::ReadFailed,
                "Color probe source could not be read.",
            );
        }
    };
    let source_sha256 = sha256_hex(&bytes);

    let icc_profile = match first_icc_profile(&bytes) {
        Ok(profile) => profile,
        Err(category) => {
            return failed_color_probe(
                platform,
                source_path,
                Some(source_sha256),
                category,
                "Color probe source is not a readable JPEG marker stream.",
            );
        }
    };

    let input_profile = match icc_profile.as_deref() {
        Some(profile) => classify_icc_profile(profile),
        None => ColorProbeInputProfile::None,
    };
    let transform_path = if icc_profile.is_some() {
        ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb
    } else {
        ColorProbeTransformPath::AssumeSrgbToLinearDisplayP3ToSrgb
    };

    ColorProbeResult {
        platform,
        source_path,
        source_sha256: Some(source_sha256),
        status: ColorProbeStatus::Success,
        input_profile,
        embedded_icc: icc_profile.is_some(),
        working_space: SPIKE_003_COLOR_GATE.working_space,
        output_profile: ColorProbeOutputProfile::Srgb,
        transform_path,
        error_category: None,
        message: format!(
            "Color probe recorded profile metadata for {} bytes.",
            metadata.len()
        ),
    }
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

#[cfg(feature = "color-probe")]
fn failed_color_probe(
    platform: ColorProbePlatform,
    source_path: String,
    source_sha256: Option<String>,
    category: ColorProbeErrorCategory,
    message: &str,
) -> ColorProbeResult {
    ColorProbeResult {
        platform,
        source_path,
        source_sha256,
        status: ColorProbeStatus::Failed,
        input_profile: ColorProbeInputProfile::Unknown,
        embedded_icc: false,
        working_space: SPIKE_003_COLOR_GATE.working_space,
        output_profile: ColorProbeOutputProfile::Srgb,
        transform_path: ColorProbeTransformPath::Unavailable,
        error_category: Some(category),
        message: message.to_string(),
    }
}

#[cfg(feature = "color-probe")]
fn current_color_probe_platform() -> ColorProbePlatform {
    if cfg!(target_os = "macos") {
        ColorProbePlatform::Macos
    } else {
        ColorProbePlatform::UnsupportedPlatform
    }
}

#[cfg(feature = "color-probe")]
fn first_icc_profile(bytes: &[u8]) -> Result<Option<Vec<u8>>, ColorProbeErrorCategory> {
    if bytes.len() < 2 || bytes[0..2] != [0xff, 0xd8] {
        return Err(ColorProbeErrorCategory::InvalidJpeg);
    }

    let mut index = 2;
    while index + 4 <= bytes.len() {
        if bytes[index] != 0xff {
            return Err(ColorProbeErrorCategory::InvalidJpeg);
        }

        let marker = bytes[index + 1];
        if marker == 0xd9 || marker == 0xda {
            return Ok(None);
        }

        let length = u16::from_be_bytes([bytes[index + 2], bytes[index + 3]]) as usize;
        if length < 2 || index + 2 + length > bytes.len() {
            return Err(ColorProbeErrorCategory::InvalidJpeg);
        }

        let marker_payload = &bytes[index + 4..index + 2 + length];
        if marker == 0xe2
            && marker_payload.starts_with(b"ICC_PROFILE\0")
            && marker_payload.len() >= 14
        {
            return Ok(Some(marker_payload[14..].to_vec()));
        }

        index += 2 + length;
    }

    Ok(None)
}

#[cfg(feature = "color-probe")]
fn classify_icc_profile(profile: &[u8]) -> ColorProbeInputProfile {
    match sha256_hex(profile).as_str() {
        "2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e" => {
            ColorProbeInputProfile::Srgb
        }
        "0ff6958f98684c61f6bbdce1368ddeaf3873baf84545baba482e920d92a914c0" => {
            ColorProbeInputProfile::DisplayP3
        }
        _ if profile
            .windows(b"sRGB".len())
            .any(|window| window.eq_ignore_ascii_case(b"sRGB")) =>
        {
            ColorProbeInputProfile::Srgb
        }
        _ => ColorProbeInputProfile::Unknown,
    }
}

#[cfg(feature = "color-probe")]
fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
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

/// Build a render-side request for exporting an edited raster source as sRGB JPEG.
pub fn plan_jpeg_srgb_export(
    source_path: impl Into<String>,
    output_path: impl Into<String>,
    exposure: f64,
    contrast: f64,
    quality: u8,
) -> JpegSrgbExportRenderRequest {
    JpegSrgbExportRenderRequest {
        source_path: source_path.into(),
        output_path: output_path.into(),
        color_behavior: SPIKE_003_COLOR_GATE.export,
        exposure,
        contrast,
        quality,
        message: "JPEG sRGB export request is ready.".to_string(),
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

    #[test]
    fn plans_jpeg_srgb_export_request() {
        let request =
            super::plan_jpeg_srgb_export("/tmp/original.jpg", "/tmp/exported.jpg", 0.5, -8.0, 90);

        assert_eq!(request.source_path, "/tmp/original.jpg");
        assert_eq!(request.output_path, "/tmp/exported.jpg");
        assert_eq!(request.exposure, 0.5);
        assert_eq!(request.contrast, -8.0);
        assert_eq!(request.quality, 90);
        assert_eq!(
            request.color_behavior,
            super::ExportColorBehavior::SrgbDefaultDisplayP3Supported
        );
        assert!(request.message.contains("JPEG sRGB export"));
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_classifies_embedded_srgb_profile() {
        let path = write_color_probe_fixture(
            "srgb",
            jpeg_with_icc_profile(b"header IEC sRGB profile bytes"),
        );

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.source_path, path.to_string_lossy());
        assert_eq!(result.status, super::ColorProbeStatus::Success);
        assert_eq!(result.input_profile, super::ColorProbeInputProfile::Srgb);
        assert!(result.embedded_icc);
        assert!(result.source_sha256.is_some());
        assert_eq!(
            result.working_space,
            super::WorkingColorSpace::LinearDisplayP3
        );
        assert_eq!(result.output_profile, super::ColorProbeOutputProfile::Srgb);
        assert_eq!(
            result.transform_path,
            super::ColorProbeTransformPath::EmbeddedIccToLinearDisplayP3ToSrgb
        );
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_classifies_local_display_p3_profile() {
        let profile = std::fs::read("/System/Library/ColorSync/Profiles/Display P3.icc")
            .expect("local Display P3 profile");
        let path = write_color_probe_fixture("display-p3", jpeg_with_icc_profile(&profile));

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.status, super::ColorProbeStatus::Success);
        assert_eq!(
            result.input_profile,
            super::ColorProbeInputProfile::DisplayP3
        );
        assert!(result.embedded_icc);
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_records_untagged_raster_as_assume_srgb() {
        let path = write_color_probe_fixture("untagged", minimal_jpeg_without_icc());

        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.status, super::ColorProbeStatus::Success);
        assert_eq!(result.input_profile, super::ColorProbeInputProfile::None);
        assert!(!result.embedded_icc);
        assert_eq!(
            result.transform_path,
            super::ColorProbeTransformPath::AssumeSrgbToLinearDisplayP3ToSrgb
        );
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    #[test]
    fn color_probe_reports_missing_file_without_panicking() {
        let path = std::env::temp_dir().join(unique_color_probe_name("missing"));
        let result =
            super::probe_color_profile(super::ColorProbeRequest::new(path.to_string_lossy()));

        assert_eq!(result.status, super::ColorProbeStatus::Failed);
        assert_eq!(
            result.error_category,
            Some(super::ColorProbeErrorCategory::MissingFile)
        );
        assert_eq!(result.source_sha256, None);
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn write_color_probe_fixture(name: &str, bytes: Vec<u8>) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(unique_color_probe_name(name));
        std::fs::write(&path, bytes).expect("write color probe fixture");
        path
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn unique_color_probe_name(name: &str) -> String {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        format!("silicaraw-color-probe-{name}-{nanos}.jpg")
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn jpeg_with_icc_profile(profile: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0xff, 0xd8];
        let mut payload = b"ICC_PROFILE\0\x01\x01".to_vec();
        payload.extend_from_slice(profile);
        bytes.extend_from_slice(&[0xff, 0xe2]);
        bytes.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
        bytes.extend_from_slice(&payload);
        bytes.extend_from_slice(&minimal_jpeg_without_icc()[2..]);
        bytes
    }

    #[cfg(all(feature = "color-probe", target_os = "macos"))]
    fn minimal_jpeg_without_icc() -> Vec<u8> {
        vec![0xff, 0xd8, 0xff, 0xd9]
    }
}
