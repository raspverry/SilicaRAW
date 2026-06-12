#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
mod platform;

#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
pub use platform::probe_core_image_raw;

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
