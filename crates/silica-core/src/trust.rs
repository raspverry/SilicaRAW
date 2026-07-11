use std::path::Path;

use super::{
    current_timestamp_string, ensure_supported_develop_source, photo_edit_commit_from_graph,
    ActionLogEntry, AiResult, CoreError, ExtensionPermission, NewActionLogEntry, PhotoEditCommit,
};

/// Append a validated action-log row through the core command boundary.
pub fn append_action_log_entry(
    library_root_path: impl AsRef<Path>,
    entry: NewActionLogEntry,
) -> Result<ActionLogEntry, CoreError> {
    silica_storage::append_action_log_entry(library_root_path, entry).map_err(CoreError::from)
}

/// Read recent action-log rows through the core command boundary.
pub fn list_action_log_entries(
    library_root_path: impl AsRef<Path>,
    limit: u16,
) -> Result<Vec<ActionLogEntry>, CoreError> {
    silica_storage::list_action_log_entries(library_root_path, limit).map_err(CoreError::from)
}

/// Store local AI result data without loading a model or mutating edit state.
pub fn store_ai_result(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
    task_type: impl AsRef<str>,
    model_id: impl AsRef<str>,
    output_json: impl AsRef<str>,
) -> Result<AiResult, CoreError> {
    silica_storage::append_ai_result(
        library_root_path,
        silica_storage::NewAiResult {
            photo_id: photo_id.as_ref().to_string(),
            task_type: task_type.as_ref().to_string(),
            model_id: model_id.as_ref().to_string(),
            permission_id: ExtensionPermission::AiResultPropose.stable_id().to_string(),
            output_json: output_json.as_ref().to_string(),
        },
    )
    .map_err(CoreError::from)
}

/// Read local AI result rows for one photo through the Core boundary.
pub fn list_ai_results_for_photo(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
    limit: u16,
) -> Result<Vec<AiResult>, CoreError> {
    silica_storage::list_ai_results_for_photo(library_root_path, photo_id.as_ref(), limit)
        .map_err(CoreError::from)
}

pub const AI_REVIEW_BLUR_TASK_TYPE: &str = "blur_score";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiReviewPanelStatus {
    ModelUnavailable,
    ReviewAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiReviewPanel {
    pub photo_id: String,
    pub task_type: String,
    pub status: AiReviewPanelStatus,
    pub message: String,
    pub editor_remains_usable: bool,
    pub requires_explicit_approval: bool,
    pub writes_edit_graph: bool,
    pub writes_photo_flags: bool,
    pub items: Vec<AiReviewItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiReviewItem {
    pub result_id: String,
    pub model_id: String,
    pub label: String,
    pub recommendation: String,
    pub approvable: bool,
    pub confidence_percent: Option<u8>,
    pub approved: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiSuggestionApproval {
    pub photo_id: String,
    pub result_id: String,
    pub model_id: String,
    pub task_type: String,
    pub suggestion_kind: String,
    pub action_log_id: String,
    pub commit: PhotoEditCommit,
    pub writes_edit_graph: bool,
    pub writes_photo_flags: bool,
    pub writes_original: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AiSuggestionRejection {
    pub photo_id: String,
    pub result_id: String,
    pub model_id: String,
    pub task_type: String,
    pub action_log_id: String,
    pub edit_state_unchanged: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PluginPresetApproval {
    pub plugin_id: String,
    pub preset_id: String,
    pub preset_name: String,
    pub action_log_id: String,
    pub commit: PhotoEditCommit,
    pub writes_edit_graph: bool,
    pub writes_photo_flags: bool,
    pub writes_original: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginPermissionReview {
    pub plugin_id: String,
    pub permission_id: String,
    pub granted: bool,
    pub action_log_id: String,
    pub runtime_started: bool,
    pub permission_persisted: bool,
    pub writes_edit_graph: bool,
    pub writes_original: bool,
}

/// Photo row subset exposed to read-only MCP adapters through Core only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpPhotoReadRecord {
    pub photo_id: String,
    pub file_name: String,
    pub path: String,
    pub unsupported: bool,
    pub rating: u8,
    pub picked: bool,
    pub rejected: bool,
    pub color_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
struct AiApprovalSuggestion {
    kind: String,
    exposure: f64,
    contrast: f64,
    summary: String,
}

/// Build the first non-mutating AI review panel from stored local AI result rows only.
pub fn get_ai_review_panel(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
) -> Result<AiReviewPanel, CoreError> {
    let photo_id = photo_id.as_ref();
    let results = silica_storage::list_ai_results_for_photo(library_root_path, photo_id, 20)?;
    let items = results
        .into_iter()
        .filter(|result| result.task_type == AI_REVIEW_BLUR_TASK_TYPE)
        .map(ai_review_item_from_result)
        .collect::<Result<Vec<_>, _>>()?;
    let status = if items.is_empty() {
        AiReviewPanelStatus::ModelUnavailable
    } else {
        AiReviewPanelStatus::ReviewAvailable
    };
    let message = match status {
        AiReviewPanelStatus::ModelUnavailable => {
            "No local blur review model or stored result is available."
        }
        AiReviewPanelStatus::ReviewAvailable => {
            "Blur review suggestions are information only until explicit approval is implemented."
        }
    };

    Ok(AiReviewPanel {
        photo_id: photo_id.to_string(),
        task_type: AI_REVIEW_BLUR_TASK_TYPE.to_string(),
        status,
        message: message.to_string(),
        editor_remains_usable: true,
        requires_explicit_approval: true,
        writes_edit_graph: false,
        writes_photo_flags: false,
        items,
    })
}

fn ai_review_item_from_result(result: AiResult) -> Result<AiReviewItem, CoreError> {
    let payload: serde_json::Value = serde_json::from_str(&result.result_json)
        .map_err(|error| CoreError::AiReview(format!("invalid result JSON: {error}")))?;
    let output = payload
        .get("output")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| CoreError::AiReview("stored result missing output object".to_string()))?;
    let review = output
        .get("review")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| CoreError::AiReview("stored result missing review object".to_string()))?;
    let label = review
        .get("label")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Review available")
        .to_string();
    let recommendation = review
        .get("recommendation")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("review")
        .to_string();
    let confidence_percent = review
        .get("confidence")
        .and_then(serde_json::Value::as_f64)
        .map(|confidence| (confidence.clamp(0.0, 1.0) * 100.0).round() as u8);
    let approvable = output.get("approval_suggestion").is_some();

    Ok(AiReviewItem {
        result_id: result.id,
        model_id: result.model_id,
        label,
        recommendation,
        approvable,
        confidence_percent,
        approved: result.approved,
        created_at: result.created_at,
    })
}

pub fn approve_ai_suggestion(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
    result_id: impl AsRef<str>,
) -> Result<Option<AiSuggestionApproval>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let photo_id = photo_id.as_ref();
    let result_id = result_id.as_ref();
    let result = silica_storage::get_ai_result(library_root_path, result_id)?;
    ensure_ai_result_targets_photo(&result, photo_id)?;
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    if result.approved {
        return Err(CoreError::AiReview(
            "AI result is already approved.".to_string(),
        ));
    }

    let suggestion = ai_approval_suggestion_from_result(&result)?;
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let mut edited = match suggestion.kind.as_str() {
        "basic_exposure_contrast" => silica_edit::apply_exposure_contrast(
            &graph,
            suggestion.exposure,
            suggestion.contrast,
            current_timestamp_string(),
        )?,
        other => {
            return Err(CoreError::AiReview(format!(
                "unsupported AI approval suggestion kind: {other}"
            )));
        }
    };
    let provenance = ai_suggestion_provenance(&result, &suggestion, "approved");
    edited
        .extensions
        .insert("silica.ai_provenance".to_string(), provenance.clone());

    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;
    let approved_result = silica_storage::approve_ai_result(library_root_path, result_id)?;
    let action_log = append_permissioned_action_log(
        library_root_path,
        "ai",
        Some(&approved_result.model_id),
        "ai_approval",
        Some("photo"),
        Some(photo_id),
        "ai_result",
        Some(format!(
            "ai:{}:approval:{result_id}",
            approved_result.model_id
        )),
        serde_json::json!({
            "model_id": approved_result.model_id,
            "result_id": approved_result.id,
            "task_type": approved_result.task_type,
            "permission_id": ExtensionPermission::EditSuggestionApply.stable_id(),
            "suggestion_kind": suggestion.kind,
            "writes_edit_graph": true,
            "writes_photo_flags": false,
            "writes_original": false,
            "provenance": provenance,
        }),
    )?;

    Ok(Some(AiSuggestionApproval {
        photo_id: persisted.source.photo_id.clone(),
        result_id: approved_result.id,
        model_id: approved_result.model_id,
        task_type: approved_result.task_type,
        suggestion_kind: suggestion.kind,
        action_log_id: action_log.id,
        commit: photo_edit_commit_from_graph(&persisted, "AI suggestion approved."),
        writes_edit_graph: true,
        writes_photo_flags: false,
        writes_original: false,
    }))
}

pub fn reject_ai_suggestion(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
    result_id: impl AsRef<str>,
) -> Result<Option<AiSuggestionRejection>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let photo_id = photo_id.as_ref();
    let result_id = result_id.as_ref();
    let result = silica_storage::get_ai_result(library_root_path, result_id)?;
    ensure_ai_result_targets_photo(&result, photo_id)?;
    if result.approved {
        return Err(CoreError::AiReview(
            "AI result is already approved.".to_string(),
        ));
    }
    let suggestion = ai_approval_suggestion_from_result(&result)?;
    let action_log = append_permissioned_action_log(
        library_root_path,
        "ai",
        Some(&result.model_id),
        "ai_rejection",
        Some("photo"),
        Some(photo_id),
        "ai_result",
        Some(format!("ai:{}:rejection:{result_id}", result.model_id)),
        serde_json::json!({
            "model_id": result.model_id,
            "result_id": result.id,
            "task_type": result.task_type,
            "permission_id": ExtensionPermission::EditSuggestionApply.stable_id(),
            "suggestion_kind": suggestion.kind,
            "writes_edit_graph": false,
            "writes_photo_flags": false,
            "writes_original": false,
        }),
    )?;

    Ok(Some(AiSuggestionRejection {
        photo_id: photo_id.to_string(),
        result_id: result.id,
        model_id: result.model_id,
        task_type: result.task_type,
        action_log_id: action_log.id,
        edit_state_unchanged: true,
    }))
}

fn ensure_ai_result_targets_photo(result: &AiResult, photo_id: &str) -> Result<(), CoreError> {
    if result.photo_id != photo_id {
        return Err(CoreError::AiReview(
            "AI result does not belong to selected photo.".to_string(),
        ));
    }
    Ok(())
}

fn ai_approval_suggestion_from_result(
    result: &AiResult,
) -> Result<AiApprovalSuggestion, CoreError> {
    let payload: serde_json::Value = serde_json::from_str(&result.result_json)
        .map_err(|error| CoreError::AiReview(format!("invalid result JSON: {error}")))?;
    let output = payload
        .get("output")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| CoreError::AiReview("stored result missing output object".to_string()))?;
    let suggestion = output
        .get("approval_suggestion")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            CoreError::AiReview("stored result missing approval suggestion".to_string())
        })?;
    let kind = suggestion
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| CoreError::AiReview("approval suggestion missing kind".to_string()))?;
    if kind != "basic_exposure_contrast" {
        return Err(CoreError::AiReview(format!(
            "unsupported AI approval suggestion kind: {kind}"
        )));
    }
    let exposure = suggestion
        .get("exposure")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| CoreError::AiReview("approval suggestion missing exposure".to_string()))?;
    let contrast = suggestion
        .get("contrast")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| CoreError::AiReview("approval suggestion missing contrast".to_string()))?;
    let summary = suggestion
        .get("summary")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("AI exposure and contrast suggestion")
        .to_string();

    Ok(AiApprovalSuggestion {
        kind: kind.to_string(),
        exposure,
        contrast,
        summary,
    })
}

fn ai_suggestion_provenance(
    result: &AiResult,
    suggestion: &AiApprovalSuggestion,
    decision: &str,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "silica.ai_provenance",
        "version": 1,
        "decision": decision,
        "result_id": result.id,
        "model_id": result.model_id,
        "task_type": result.task_type,
        "suggestion_kind": suggestion.kind,
        "summary": suggestion.summary,
    })
}

pub(super) fn append_core_action_log(
    library_root_path: &Path,
    action_type: &str,
    subject_type: Option<&str>,
    subject_id: Option<String>,
    side_effect_category: &str,
    evidence_ref: Option<String>,
    payload: serde_json::Value,
) -> Result<ActionLogEntry, CoreError> {
    let payload_json = serde_json::to_string(&payload).map_err(|error| {
        CoreError::AppSession(format!("action log payload serialization failed: {error}"))
    })?;
    append_action_log_entry(
        library_root_path,
        NewActionLogEntry {
            actor_type: "core".to_string(),
            actor_id: Some("local-alpha".to_string()),
            action_type: action_type.to_string(),
            subject_type: subject_type.map(str::to_string),
            subject_id,
            side_effect_category: side_effect_category.to_string(),
            evidence_ref,
            payload_json,
        },
    )
}

fn append_permissioned_action_log(
    library_root_path: &Path,
    actor_type: &str,
    actor_id: Option<&str>,
    action_type: &str,
    subject_type: Option<&str>,
    subject_id: Option<&str>,
    side_effect_category: &str,
    evidence_ref: Option<String>,
    payload: serde_json::Value,
) -> Result<ActionLogEntry, CoreError> {
    let payload_json = serde_json::to_string(&payload).map_err(|error| {
        CoreError::AppSession(format!(
            "permission action log payload serialization failed: {error}"
        ))
    })?;
    append_action_log_entry(
        library_root_path,
        NewActionLogEntry {
            actor_type: actor_type.to_string(),
            actor_id: actor_id.map(str::to_string),
            action_type: action_type.to_string(),
            subject_type: subject_type.map(str::to_string),
            subject_id: subject_id.map(str::to_string),
            side_effect_category: side_effect_category.to_string(),
            evidence_ref,
            payload_json,
        },
    )
}

/// Record a future extension permission grant or denial without storing a grant.
pub fn record_permission_decision(
    library_root_path: impl AsRef<Path>,
    actor_type: impl AsRef<str>,
    actor_id: Option<&str>,
    permission: ExtensionPermission,
    granted: bool,
    reason: impl AsRef<str>,
) -> Result<ActionLogEntry, CoreError> {
    let action_type = if granted {
        "permission_grant"
    } else {
        "permission_denial"
    };
    let permission_id = permission.stable_id();
    append_permissioned_action_log(
        library_root_path.as_ref(),
        actor_type.as_ref(),
        actor_id,
        action_type,
        Some("permission"),
        Some(permission_id),
        "permission_decision",
        Some(format!("{action_type}:{permission_id}")),
        serde_json::json!({
            "permission_id": permission_id,
            "granted": granted,
            "reason": reason.as_ref(),
        }),
    )
}

/// Record that a plugin apply path reached permission review.
pub fn record_plugin_apply_attempt(
    library_root_path: impl AsRef<Path>,
    plugin_id: impl AsRef<str>,
    photo_id: Option<&str>,
    permission: ExtensionPermission,
) -> Result<ActionLogEntry, CoreError> {
    let plugin_id = plugin_id.as_ref();
    append_permissioned_action_log(
        library_root_path.as_ref(),
        "plugin",
        Some(plugin_id),
        "plugin_apply",
        Some("photo"),
        photo_id,
        "extension_review",
        Some(format!("plugin:{plugin_id}:apply")),
        serde_json::json!({
            "plugin_id": plugin_id,
            "permission_id": permission.stable_id(),
            "granted": false,
        }),
    )
}

pub fn review_plugin_enable_permission(
    library_root_path: impl AsRef<Path>,
    manifest_json: impl AsRef<str>,
    permission: ExtensionPermission,
    granted: bool,
    reason: impl AsRef<str>,
) -> Result<PluginPermissionReview, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let manifest = silica_plugin::validate_plugin_manifest_json(manifest_json.as_ref())?;
    let permission_id = permission.stable_id();
    ensure_plugin_manifest_requests_permission(&manifest, permission_id)?;
    let action_type = if granted {
        "permission_grant"
    } else {
        "permission_denial"
    };
    let action_log = append_permissioned_action_log(
        library_root_path,
        "plugin",
        Some(&manifest.plugin_id),
        action_type,
        Some("permission"),
        Some(permission_id),
        "permission_decision",
        Some(format!(
            "plugin:{}:enable:{permission_id}",
            manifest.plugin_id
        )),
        serde_json::json!({
            "review_kind": "plugin_enable",
            "plugin_id": manifest.plugin_id,
            "permission_id": permission_id,
            "granted": granted,
            "reason": reason.as_ref(),
            "permission_persisted": false,
            "runtime_started": false,
        }),
    )?;

    Ok(PluginPermissionReview {
        plugin_id: manifest.plugin_id,
        permission_id: permission_id.to_string(),
        granted,
        action_log_id: action_log.id,
        runtime_started: false,
        permission_persisted: false,
        writes_edit_graph: false,
        writes_original: false,
    })
}

pub fn review_plugin_apply_permission(
    library_root_path: impl AsRef<Path>,
    manifest_json: impl AsRef<str>,
    photo_id: Option<&str>,
    preset_id: Option<&str>,
    granted: bool,
    reason: impl AsRef<str>,
) -> Result<PluginPermissionReview, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let manifest = silica_plugin::validate_plugin_manifest_json(manifest_json.as_ref())?;
    let permission_id = ExtensionPermission::EditSuggestionApply.stable_id();
    ensure_plugin_manifest_requests_permission(&manifest, permission_id)?;
    let action_log = append_permissioned_action_log(
        library_root_path,
        "plugin",
        Some(&manifest.plugin_id),
        "plugin_apply",
        Some("photo"),
        photo_id,
        "extension_review",
        Some(format!("plugin:{}:apply-review", manifest.plugin_id)),
        serde_json::json!({
            "review_kind": "plugin_apply",
            "plugin_id": manifest.plugin_id,
            "preset_id": preset_id,
            "permission_id": permission_id,
            "granted": granted,
            "reason": reason.as_ref(),
            "writes_edit_graph": false,
            "writes_original": false,
        }),
    )?;

    Ok(PluginPermissionReview {
        plugin_id: manifest.plugin_id,
        permission_id: permission_id.to_string(),
        granted,
        action_log_id: action_log.id,
        runtime_started: false,
        permission_persisted: false,
        writes_edit_graph: false,
        writes_original: false,
    })
}

/// Apply one data-only plugin preset after explicit approval.
pub fn approve_plugin_preset(
    library_root_path: impl AsRef<Path>,
    photo_id: impl AsRef<str>,
    manifest_json: impl AsRef<str>,
    preset_pack_json: impl AsRef<str>,
    preset_id: impl AsRef<str>,
) -> Result<Option<PluginPresetApproval>, CoreError> {
    let library_root_path = library_root_path.as_ref();
    let photo_id = photo_id.as_ref();
    let preset_id = preset_id.as_ref();
    let manifest = silica_plugin::validate_plugin_manifest_json(manifest_json.as_ref())?;
    if !manifest
        .permissions
        .iter()
        .any(|permission| permission == ExtensionPermission::EditSuggestionApply.stable_id())
    {
        return Err(CoreError::Plugin(
            "plugin preset apply requires edit_suggestion:apply".to_string(),
        ));
    }
    let preset_pack =
        silica_plugin::validate_preset_pack_json(&manifest, preset_pack_json.as_ref())?;
    let preset = preset_pack
        .presets
        .iter()
        .find(|preset| preset.preset_id == preset_id)
        .ok_or_else(|| CoreError::Plugin(format!("preset_id {preset_id} not found")))?;
    if !ensure_supported_develop_source(library_root_path, photo_id)? {
        return Ok(None);
    }
    let graph =
        match silica_storage::load_active_edit_graph_or_default(library_root_path, photo_id)? {
            Some(graph) => graph,
            None => return Ok(None),
        };
    let adjustments = plugin_basic_preset_adjustments(&preset.basic)?;
    let mut edited =
        silica_edit::apply_plugin_basic_preset(&graph, &adjustments, current_timestamp_string())?;
    edited.extensions.insert(
        "silica.plugin_provenance".to_string(),
        serde_json::json!({
            "schema": "silica.plugin_provenance",
            "version": 1,
            "plugin_id": manifest.plugin_id,
            "plugin_version": manifest.plugin_version,
            "preset_id": preset.preset_id,
            "preset_name": preset.name,
            "permission_id": ExtensionPermission::EditSuggestionApply.stable_id(),
        }),
    );
    let persisted = silica_storage::commit_edit_graph(library_root_path, edited)?;
    let action_log = append_permissioned_action_log(
        library_root_path,
        "plugin",
        Some(&manifest.plugin_id),
        "plugin_apply",
        Some("photo"),
        Some(photo_id),
        "edit_graph",
        Some(format!("plugin:{}:preset:{preset_id}", manifest.plugin_id)),
        serde_json::json!({
            "plugin_id": manifest.plugin_id,
            "preset_id": preset.preset_id,
            "preset_name": preset.name,
            "permission_id": ExtensionPermission::EditSuggestionApply.stable_id(),
            "granted": true,
            "writes_edit_graph": true,
            "writes_photo_flags": false,
            "writes_original": false,
        }),
    )?;

    Ok(Some(PluginPresetApproval {
        plugin_id: preset_pack.plugin_id,
        preset_id: preset.preset_id.clone(),
        preset_name: preset.name.clone(),
        action_log_id: action_log.id,
        commit: photo_edit_commit_from_graph(&persisted, "Plugin preset applied."),
        writes_edit_graph: true,
        writes_photo_flags: false,
        writes_original: false,
    }))
}

fn ensure_plugin_manifest_requests_permission(
    manifest: &silica_plugin::PluginManifest,
    permission_id: &str,
) -> Result<(), CoreError> {
    if manifest
        .permissions
        .iter()
        .any(|permission| permission == permission_id)
    {
        return Ok(());
    }
    Err(CoreError::Plugin(format!(
        "plugin manifest does not request {permission_id}"
    )))
}

/// Record an approved AI result boundary without running MLX or mutating edits.
pub fn record_ai_approval(
    library_root_path: impl AsRef<Path>,
    model_id: impl AsRef<str>,
    photo_id: Option<&str>,
    permission: ExtensionPermission,
) -> Result<ActionLogEntry, CoreError> {
    let model_id = model_id.as_ref();
    append_permissioned_action_log(
        library_root_path.as_ref(),
        "ai",
        Some(model_id),
        "ai_approval",
        Some("photo"),
        photo_id,
        "ai_result",
        Some(format!("ai:{model_id}:approval")),
        serde_json::json!({
            "model_id": model_id,
            "permission_id": permission.stable_id(),
        }),
    )
}

fn plugin_basic_preset_adjustments(
    preset: &silica_plugin::PluginBasicPreset,
) -> Result<silica_edit::PluginBasicPresetAdjustments, CoreError> {
    Ok(silica_edit::PluginBasicPresetAdjustments {
        white_balance: plugin_white_balance(&preset.white_balance)?,
        temperature: preset.temperature,
        tint: preset.tint,
        exposure: preset.exposure,
        contrast: preset.contrast,
        highlights: preset.highlights,
        shadows: preset.shadows,
        whites: preset.whites,
        blacks: preset.blacks,
        vibrance: preset.vibrance,
        saturation: preset.saturation,
    })
}

fn plugin_white_balance(value: &str) -> Result<silica_edit::WhiteBalance, CoreError> {
    match value {
        "as_shot" => Ok(silica_edit::WhiteBalance::AsShot),
        "auto" => Ok(silica_edit::WhiteBalance::Auto),
        "daylight" => Ok(silica_edit::WhiteBalance::Daylight),
        "cloudy" => Ok(silica_edit::WhiteBalance::Cloudy),
        "shade" => Ok(silica_edit::WhiteBalance::Shade),
        "tungsten" => Ok(silica_edit::WhiteBalance::Tungsten),
        "fluorescent" => Ok(silica_edit::WhiteBalance::Fluorescent),
        "flash" => Ok(silica_edit::WhiteBalance::Flash),
        "custom" => Ok(silica_edit::WhiteBalance::Custom),
        other => Err(CoreError::Plugin(format!(
            "unsupported plugin white_balance {other}"
        ))),
    }
}

/// Record a future MCP read through Core policy without starting an MCP server.
pub fn record_mcp_read(
    library_root_path: impl AsRef<Path>,
    session_id: impl AsRef<str>,
    subject_type: impl AsRef<str>,
    subject_id: Option<&str>,
    permission: ExtensionPermission,
) -> Result<ActionLogEntry, CoreError> {
    let session_id = session_id.as_ref();
    append_permissioned_action_log(
        library_root_path.as_ref(),
        "mcp",
        Some(session_id),
        "mcp_read",
        Some(subject_type.as_ref()),
        subject_id,
        "catalog_read",
        Some(format!("mcp:{session_id}:read")),
        serde_json::json!({
            "session_id": session_id,
            "permission_id": permission.stable_id(),
        }),
    )
}

/// Record a permissioned export attempt before any future extension export path runs.
pub fn record_permissioned_export_attempt(
    library_root_path: impl AsRef<Path>,
    actor_type: impl AsRef<str>,
    actor_id: Option<&str>,
    photo_id: Option<&str>,
    permission: ExtensionPermission,
    output_path: impl AsRef<str>,
) -> Result<ActionLogEntry, CoreError> {
    let output_path = output_path.as_ref();
    append_permissioned_action_log(
        library_root_path.as_ref(),
        actor_type.as_ref(),
        actor_id,
        "export_attempt",
        Some("photo"),
        photo_id,
        "export_attempt",
        Some(output_path.to_string()),
        serde_json::json!({
            "permission_id": permission.stable_id(),
            "output_path": output_path,
        }),
    )
}
