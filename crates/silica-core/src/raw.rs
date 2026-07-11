use std::path::{Path, PathBuf};

use crate::{CoreError, LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE};

pub fn plan_product_raw_decode(
    source_path: impl AsRef<str>,
) -> silica_decode::ProductRawDecodePlan {
    silica_decode::plan_product_raw_decode(source_path)
}

pub fn plan_product_raw_decode_from_probe(
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
) -> silica_decode::ProductRawDecodePlan {
    silica_decode::plan_product_raw_decode_from_probe(fixture_class, probe)
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedImageViewerHandoffPlan {
    pub decoded: silica_decode::DecodedImageHandoff,
    pub viewer_input: silica_render::ViewerPreviewInput,
}

impl DecodedImageViewerHandoffPlan {
    pub fn writes_catalog_state(&self) -> bool {
        false
    }

    pub fn writes_sidecars(&self) -> bool {
        false
    }

    pub fn writes_originals(&self) -> bool {
        false
    }

    pub fn writes_exports(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPreviewArtifactSession {
    pub handoff: DecodedImageViewerHandoffPlan,
    pub output_path: PathBuf,
    pub artifact_path: Option<PathBuf>,
    pub cache_record: Option<silica_storage::CacheRecord>,
    pub bytes_written: Option<u64>,
    pub original_hash_unchanged: Option<bool>,
}

pub fn plan_decoded_image_viewer_handoff(
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
    cache_key: impl Into<String>,
) -> DecodedImageViewerHandoffPlan {
    let decoded =
        silica_decode::plan_decoded_image_handoff_from_raw_probe(fixture_class, probe, cache_key);
    let viewer_input = silica_render::ViewerPreviewInput::from_decoded_handoff(&decoded);

    DecodedImageViewerHandoffPlan {
        decoded,
        viewer_input,
    }
}

pub fn write_raw_preview_artifact_for_probe(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
    fixture_class: impl AsRef<str>,
    probe: &silica_decode::RawProbeResult,
) -> Result<RawPreviewArtifactSession, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let photo_id = photo_id.as_ref();
    let output_path = raw_preview_artifact_output_path(library_root_path, photo_id);
    let cache_key = raw_preview_artifact_cache_key(photo_id, probe);
    let result =
        silica_decode::write_raw_preview_artifact(silica_decode::RawPreviewArtifactRequest {
            fixture_class: fixture_class.as_ref().to_string(),
            probe: probe.clone(),
            cache_key: cache_key.clone(),
            output_path: output_path.clone(),
            max_edge: LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE,
        })?;
    let viewer_input = silica_render::ViewerPreviewInput::from_decoded_handoff(&result.handoff);
    let handoff = DecodedImageViewerHandoffPlan {
        decoded: result.handoff,
        viewer_input,
    };
    let cache_record = match (&result.artifact_path, result.bytes_written) {
        (Some(path), Some(bytes_written)) => {
            let byte_size = i64::try_from(bytes_written).unwrap_or(i64::MAX);
            Some(silica_storage::record_preview_cache(
                library_root_path,
                photo_id,
                cache_key,
                path,
                byte_size,
            )?)
        }
        _ => None,
    };

    Ok(RawPreviewArtifactSession {
        handoff,
        output_path,
        artifact_path: result.artifact_path,
        cache_record,
        bytes_written: result.bytes_written,
        original_hash_unchanged: result.original_hash_unchanged,
    })
}

pub(super) fn raw_preview_artifact_output_path(
    library_root_path: &Path,
    photo_id: &str,
) -> PathBuf {
    library_root_path
        .join("previews")
        .join(format!("raw-{photo_id}.jpg"))
}

pub(super) fn raw_preview_artifact_cache_key(
    photo_id: &str,
    probe: &silica_decode::RawProbeResult,
) -> String {
    let backend = match probe.backend {
        silica_decode::RawProbeBackend::CoreImageRaw => "core-image-raw",
    };
    let source_sha = probe.source_sha256.as_deref().unwrap_or("missing-sha256");
    format!(
        "raw-preview:v1:{photo_id}:{backend}:{source_sha}:{}:{}x{}",
        LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE,
        probe.width.unwrap_or(0),
        probe.height.unwrap_or(0)
    )
}
