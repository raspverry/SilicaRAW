#[cfg(feature = "core-image-raw-probe")]
fn main() {
    let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
        .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
    let report = silica_decode::probe_raw_fixture_manifest(&manifest)
        .expect("probe legal RAW fixture manifest");

    let results: Vec<_> = report
        .results
        .iter()
        .map(|result| {
            let probe = &result.probe;
            serde_json::json!({
                "fixture_id": result.fixture_id,
                "fixture_class": result.fixture_class,
                "relative_path": result.relative_path,
                "original_hash_unchanged": result.original_hash_unchanged,
                "backend": raw_probe_backend(probe.backend),
                "platform": raw_probe_platform(probe.platform),
                "macos_version": probe.macos_version,
                "source_path": probe.source_path,
                "source_sha256": probe.source_sha256,
                "original_file_size": probe.original_file_size,
                "original_modified_at": probe.original_modified_at,
                "status": raw_probe_status(probe.status),
                "width": probe.width,
                "height": probe.height,
                "orientation": probe.orientation,
                "error_category": probe.error_category.map(raw_probe_error_category),
                "message": probe.message,
            })
        })
        .collect();

    let output = serde_json::json!({
        "manifest_path": report.manifest_path,
        "results": results,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&output).expect("serialize RAW probe report")
    );
}

#[cfg(not(feature = "core-image-raw-probe"))]
fn main() {
    eprintln!("raw_probe_report requires the core-image-raw-probe feature");
    std::process::exit(2);
}

#[cfg(feature = "core-image-raw-probe")]
fn raw_probe_backend(backend: silica_decode::RawProbeBackend) -> &'static str {
    match backend {
        silica_decode::RawProbeBackend::CoreImageRaw => "core_image_raw",
    }
}

#[cfg(feature = "core-image-raw-probe")]
fn raw_probe_platform(platform: silica_decode::RawProbePlatform) -> &'static str {
    match platform {
        silica_decode::RawProbePlatform::Macos => "macos",
        silica_decode::RawProbePlatform::UnsupportedPlatform => "unsupported_platform",
    }
}

#[cfg(feature = "core-image-raw-probe")]
fn raw_probe_status(status: silica_decode::RawProbeStatus) -> &'static str {
    match status {
        silica_decode::RawProbeStatus::Success => "success",
        silica_decode::RawProbeStatus::Unsupported => "unsupported",
        silica_decode::RawProbeStatus::Failed => "failed",
        silica_decode::RawProbeStatus::Unavailable => "unavailable",
    }
}

#[cfg(feature = "core-image-raw-probe")]
fn raw_probe_error_category(category: silica_decode::RawProbeErrorCategory) -> &'static str {
    match category {
        silica_decode::RawProbeErrorCategory::UnsupportedPlatform => "unsupported_platform",
        silica_decode::RawProbeErrorCategory::MissingFile => "missing_file",
        silica_decode::RawProbeErrorCategory::SourceHashMismatch => "source_hash_mismatch",
        silica_decode::RawProbeErrorCategory::CoreImageUnavailable => "core_image_unavailable",
        silica_decode::RawProbeErrorCategory::CoreImageOpenFailed => "core_image_open_failed",
        silica_decode::RawProbeErrorCategory::CoreImageMetadataMissing => {
            "core_image_metadata_missing"
        }
        silica_decode::RawProbeErrorCategory::PermissionDenied => "permission_denied",
        silica_decode::RawProbeErrorCategory::InvalidFixture => "invalid_fixture",
        silica_decode::RawProbeErrorCategory::Unknown => "unknown",
    }
}
