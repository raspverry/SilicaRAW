#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
mod platform;

#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
pub use platform::probe_core_image_raw;

#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
pub use platform::write_core_image_raw_preview_artifact;

#[cfg(not(all(target_os = "macos", feature = "core-image-raw-probe")))]
pub fn probe_core_image_raw(request: crate::RawProbeRequest) -> crate::RawProbeResult {
    crate::RawProbeResult {
        backend: crate::RawProbeBackend::CoreImageRaw,
        platform: crate::RawProbePlatform::UnsupportedPlatform,
        macos_version: None,
        source_path: request.source_path,
        source_sha256: None,
        original_file_size: None,
        original_modified_at: None,
        status: crate::RawProbeStatus::Unavailable,
        width: None,
        height: None,
        orientation: None,
        error_category: Some(crate::RawProbeErrorCategory::UnsupportedPlatform),
        message: "Core Image RAW probe is unavailable on this platform or feature build."
            .to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreImageRawPreviewArtifact {
    pub output_path: std::path::PathBuf,
    pub bytes_written: u64,
    pub original_hash_unchanged: bool,
}

#[cfg(not(all(target_os = "macos", feature = "core-image-raw-probe")))]
pub fn write_core_image_raw_preview_artifact(
    _probe: &crate::RawProbeResult,
    _output_path: &std::path::Path,
    _max_edge: u32,
) -> Result<CoreImageRawPreviewArtifact, crate::RawPreviewArtifactError> {
    Err(crate::RawPreviewArtifactError::CoreImageUnavailable(
        "Core Image RAW preview artifact writing requires macOS with the core-image-raw-probe feature.".to_string(),
    ))
}
