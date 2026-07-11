use std::collections::BTreeSet;
use std::path::Path;

use super::{
    current_timestamp_string, detail_unsupported_message, edit_graphs_equal_ignoring_updated_at,
    geometry_unsupported_message, has_unsupported_basic_runtime, is_supported_raster_file_type,
    lens_unsupported_message, render_detail_from_graph, render_geometry_from_graph, CoreError,
};

/// Result of applying one edit clipboard payload across multiple catalog photos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncResult {
    pub status: String,
    pub requested_count: usize,
    pub applied_count: usize,
    pub skipped_count: usize,
    pub blocked_count: usize,
    pub failed_count: usize,
    pub commits: Vec<BatchEditClipboardSyncCommit>,
    pub targets: Vec<BatchEditClipboardSyncTarget>,
    pub failures: Vec<BatchEditClipboardSyncFailure>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncCommit {
    pub photo_id: String,
    pub history_id: String,
    pub sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncFailure {
    pub photo_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncPlan {
    pub status: String,
    pub requested_count: usize,
    pub ready_count: usize,
    pub unchanged_count: usize,
    pub blocked_count: usize,
    pub targets: Vec<BatchEditClipboardSyncTarget>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchEditClipboardSyncTarget {
    pub photo_id: String,
    pub status: String,
    pub code: Option<String>,
    pub message: String,
}

struct PreparedEditClipboardSync {
    plan: BatchEditClipboardSyncPlan,
    ready_graphs: Vec<silica_edit::EditGraph>,
}

/// Copy edit sections through the core boundary without reading or writing catalog state.
pub fn copy_edit_clipboard_payload(
    source: &silica_edit::EditGraph,
    selection: silica_edit::EditClipboardSelection,
) -> Result<silica_edit::EditClipboardPayload, CoreError> {
    silica_edit::copy_edit_clipboard_payload(source, selection).map_err(CoreError::from)
}

/// Apply an edit clipboard payload to a target graph without catalog or sidecar mutation.
pub fn apply_edit_clipboard_payload_to_graph(
    target: &silica_edit::EditGraph,
    payload: &silica_edit::EditClipboardPayload,
    updated_at: impl Into<String>,
) -> Result<silica_edit::EditGraph, CoreError> {
    silica_edit::apply_edit_clipboard_payload(target, payload, updated_at).map_err(CoreError::from)
}

/// Copy edit sections from one catalog photo through the core boundary.
pub fn copy_photo_edit_clipboard_payload(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    selection: silica_edit::EditClipboardSelection,
) -> Result<Option<silica_edit::EditClipboardPayload>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    if let Some((code, message)) = edit_clipboard_catalog_target_block(library_root_path, photo_id)?
    {
        if code == "missing_photo" {
            return Ok(None);
        }
        return Err(CoreError::UnsupportedEdit(message));
    }

    let Some(graph) =
        silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)?
    else {
        return Ok(None);
    };
    silica_edit::copy_edit_clipboard_payload(&graph, selection)
        .map(Some)
        .map_err(CoreError::from)
}

/// Plan a batch edit clipboard sync without mutating catalog state.
pub fn plan_edit_clipboard_sync(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<BatchEditClipboardSyncPlan, CoreError> {
    Ok(prepare_edit_clipboard_sync(library_root_path, photo_ids, payload)?.plan)
}

/// Apply one typed edit clipboard payload to multiple photos with per-photo history checkpoints.
pub fn apply_edit_clipboard_sync(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<BatchEditClipboardSyncResult, CoreError> {
    let prepared = prepare_edit_clipboard_sync(library_root_path.as_ref(), photo_ids, payload)?;
    if prepared.plan.status == "empty" {
        return Ok(BatchEditClipboardSyncResult {
            status: "empty".to_string(),
            requested_count: prepared.plan.requested_count,
            applied_count: 0,
            skipped_count: 0,
            blocked_count: 0,
            failed_count: 0,
            commits: Vec::new(),
            targets: prepared.plan.targets,
            failures: Vec::new(),
            message: prepared.plan.message,
        });
    }
    if prepared.plan.blocked_count > 0 {
        let failures = prepared
            .plan
            .targets
            .iter()
            .filter(|target| target.status == "blocked")
            .map(|target| BatchEditClipboardSyncFailure {
                photo_id: target.photo_id.clone(),
                message: target.message.clone(),
            })
            .collect::<Vec<_>>();
        return Ok(BatchEditClipboardSyncResult {
            status: "blocked".to_string(),
            requested_count: prepared.plan.requested_count,
            applied_count: 0,
            skipped_count: prepared.plan.unchanged_count,
            blocked_count: prepared.plan.blocked_count,
            failed_count: prepared.plan.blocked_count,
            commits: Vec::new(),
            targets: prepared.plan.targets,
            failures,
            message: format!(
                "{} target(s) blocked; nothing was written.",
                prepared.plan.blocked_count
            ),
        });
    }
    if prepared.ready_graphs.is_empty() {
        return Ok(BatchEditClipboardSyncResult {
            status: "skipped".to_string(),
            requested_count: prepared.plan.requested_count,
            applied_count: 0,
            skipped_count: prepared.plan.unchanged_count,
            blocked_count: 0,
            failed_count: 0,
            commits: Vec::new(),
            targets: prepared.plan.targets,
            failures: Vec::new(),
            message: "Batch sync skipped; all targets were unchanged.".to_string(),
        });
    }

    let library_root_path = library_root_path.as_ref();
    let storage_result =
        silica_storage::commit_edit_graph_batch(library_root_path, prepared.ready_graphs)?;
    let commits = storage_result
        .commits
        .into_iter()
        .map(|commit| BatchEditClipboardSyncCommit {
            photo_id: commit.photo_id,
            history_id: commit.history_id,
            sequence: commit.sequence,
        })
        .collect::<Vec<_>>();
    let applied_count = commits.len();
    let skipped_count = prepared.plan.unchanged_count + storage_result.skipped_photo_ids.len();
    let status = if applied_count == 0 {
        "skipped"
    } else {
        "applied"
    };

    Ok(BatchEditClipboardSyncResult {
        status: status.to_string(),
        requested_count: prepared.plan.requested_count,
        applied_count,
        skipped_count,
        blocked_count: 0,
        failed_count: 0,
        commits,
        targets: prepared.plan.targets,
        failures: Vec::new(),
        message: format!(
            "Batch sync completed: {applied_count} applied, {skipped_count} unchanged."
        ),
    })
}

/// Backward-compatible name for the batch clipboard apply boundary.
pub fn sync_edit_clipboard_payload_to_photos(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<BatchEditClipboardSyncResult, CoreError> {
    apply_edit_clipboard_sync(library_root_path, photo_ids, payload)
}

fn prepare_edit_clipboard_sync(
    library_root_path: impl AsRef<Path>,
    photo_ids: &[String],
    payload: &silica_edit::EditClipboardPayload,
) -> Result<PreparedEditClipboardSync, CoreError> {
    silica_edit::validate_edit_clipboard_payload(payload)?;
    let library_root_path = library_root_path.as_ref();
    let requested_count = photo_ids.len();
    if photo_ids.is_empty() {
        return Ok(PreparedEditClipboardSync {
            plan: BatchEditClipboardSyncPlan {
                status: "empty".to_string(),
                requested_count,
                ready_count: 0,
                unchanged_count: 0,
                blocked_count: 0,
                targets: Vec::new(),
                message: "No target photos requested for batch sync.".to_string(),
            },
            ready_graphs: Vec::new(),
        });
    }

    let mut seen = BTreeSet::new();
    let mut targets = Vec::new();
    let mut ready_graphs = Vec::new();

    for photo_id in photo_ids {
        if photo_id.trim().is_empty() {
            targets.push(batch_sync_target(
                photo_id,
                "blocked",
                Some("empty_target"),
                "Photo id must not be empty.",
            ));
            continue;
        }
        if !seen.insert(photo_id.clone()) {
            targets.push(batch_sync_target(
                photo_id,
                "blocked",
                Some("duplicate_target"),
                "Duplicate target photo id.",
            ));
            continue;
        }

        if let Some((code, message)) =
            edit_clipboard_catalog_target_block(library_root_path, photo_id)?
        {
            targets.push(batch_sync_target(photo_id, "blocked", Some(code), &message));
            continue;
        }

        let target =
            match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
                Some(graph) => graph,
                None => {
                    targets.push(batch_sync_target(
                        photo_id,
                        "blocked",
                        Some("missing_photo"),
                        "Photo not found.",
                    ));
                    continue;
                }
            };
        let edited = match silica_edit::apply_edit_clipboard_payload(
            &target,
            payload,
            current_timestamp_string(),
        ) {
            Ok(graph) => graph,
            Err(error) => {
                targets.push(batch_sync_target(
                    photo_id,
                    "blocked",
                    Some("invalid_payload"),
                    &error.to_string(),
                ));
                continue;
            }
        };
        if let Err((code, message)) = ensure_supported_edit_clipboard_sync(payload, &edited) {
            targets.push(batch_sync_target(photo_id, "blocked", Some(code), &message));
            continue;
        }
        if edit_graphs_equal_ignoring_updated_at(&target, &edited) {
            targets.push(batch_sync_target(
                photo_id,
                "unchanged",
                Some("no_effect"),
                "Clipboard payload does not change this target.",
            ));
            continue;
        }

        targets.push(batch_sync_target(
            photo_id,
            "ready",
            None,
            "Ready for batch sync.",
        ));
        ready_graphs.push(edited);
    }

    let ready_count = targets
        .iter()
        .filter(|target| target.status == "ready")
        .count();
    let unchanged_count = targets
        .iter()
        .filter(|target| target.status == "unchanged")
        .count();
    let blocked_count = targets
        .iter()
        .filter(|target| target.status == "blocked")
        .count();
    let status = if blocked_count > 0 {
        "blocked"
    } else {
        "ready"
    };
    let message = if blocked_count > 0 {
        format!("{blocked_count} target(s) blocked; nothing will be written.")
    } else {
        format!("{ready_count} target(s) ready, {unchanged_count} unchanged.")
    };

    Ok(PreparedEditClipboardSync {
        plan: BatchEditClipboardSyncPlan {
            status: status.to_string(),
            requested_count,
            ready_count,
            unchanged_count,
            blocked_count,
            targets,
            message,
        },
        ready_graphs,
    })
}

fn batch_sync_target(
    photo_id: &str,
    status: &str,
    code: Option<&str>,
    message: &str,
) -> BatchEditClipboardSyncTarget {
    BatchEditClipboardSyncTarget {
        photo_id: photo_id.to_string(),
        status: status.to_string(),
        code: code.map(str::to_string),
        message: message.to_string(),
    }
}

fn ensure_supported_edit_clipboard_sync(
    payload: &silica_edit::EditClipboardPayload,
    graph: &silica_edit::EditGraph,
) -> Result<(), (&'static str, String)> {
    if payload.basic.is_some() && has_unsupported_basic_runtime(graph) {
        return Err((
            "unsupported_basic_runtime",
            "Texture, clarity, and dehaze batch sync are unsupported until runtime support exists."
                .to_string(),
        ));
    }
    if payload.detail.is_some() && !render_detail_from_graph(graph).is_neutral() {
        return Err(("unsupported_detail", detail_unsupported_message()));
    }
    if payload.lens.is_some() || payload.geometry.is_some() {
        if let Some(message) = lens_unsupported_message(graph) {
            return Err(("unsupported_lens", message));
        }
        if let Some(message) = geometry_unsupported_message(&render_geometry_from_graph(graph)) {
            return Err(("unsupported_geometry", message));
        }
    }
    Ok(())
}

fn edit_clipboard_catalog_target_block(
    library_root_path: &Path,
    photo_id: &str,
) -> Result<Option<(&'static str, String)>, CoreError> {
    let Some(metadata) = silica_storage::get_photo_metadata(library_root_path, photo_id)? else {
        return Ok(Some(("missing_photo", "Photo not found.".to_string())));
    };
    if metadata.unsupported || !is_supported_raster_file_type(&metadata.file_type) {
        return Ok(Some((
            "unsupported_target",
            "Edit clipboard copy and sync are limited to supported raster Develop photos in this alpha."
                .to_string(),
        )));
    }
    Ok(None)
}
