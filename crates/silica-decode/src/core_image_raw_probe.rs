use crate::{
    RawProbeBackend, RawProbeErrorCategory, RawProbePlatform, RawProbeRequest, RawProbeResult,
    RawProbeStatus,
};

pub fn probe_core_image_raw(request: RawProbeRequest) -> RawProbeResult {
    RawProbeResult {
        backend: RawProbeBackend::CoreImageRaw,
        platform: RawProbePlatform::UnsupportedPlatform,
        macos_version: None,
        source_path: request.source_path,
        source_sha256: None,
        original_file_size: None,
        original_modified_at: None,
        status: RawProbeStatus::Unavailable,
        width: None,
        height: None,
        orientation: None,
        error_category: Some(RawProbeErrorCategory::UnsupportedPlatform),
        message: "Core Image RAW probe is unavailable on this platform or feature build."
            .to_string(),
    }
}
