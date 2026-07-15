use serde::Serialize;
use std::path::PathBuf;

use super::{
    ai_review_status_text, app_session_warning_strings, desktop_export_preset,
    desktop_export_settings, edit_clipboard_section_count, edit_clipboard_target_data,
    library_query_order_field_string, metadata_field, DesktopAiReviewItem,
    DesktopAiSuggestionCommit, DesktopAppSession, DesktopCacheDirectoryStatus, DesktopDetailState,
    DesktopEditClipboardCommit, DesktopEditClipboardFailure, DesktopEditClipboardTarget,
    DesktopExportPreset, DesktopExportSettings, DesktopGeometryState, DesktopHistoryItem,
    DesktopHslColorMixerState, DesktopImportIssue, DesktopManualMaskState, DesktopMetadataField,
    DesktopPhotoGridItem, DesktopRecentExport, DesktopToneCurveState,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    tag = "kind",
    rename_all_fields = "camelCase"
)]
pub(crate) enum DesktopCommandData {
    LibrarySession {
        root_path: String,
        catalog_path: String,
        schema_version: i64,
    },
    AppSession {
        session_path: String,
        session: DesktopAppSession,
        warnings: Vec<String>,
    },
    AppSessionWrite {
        session_path: String,
        bytes_written: u64,
    },
    AppSessionInspection {
        session_path: String,
        exists: bool,
        warnings: Vec<String>,
    },
    LaunchRestore {
        session_path: String,
        session: DesktopAppSession,
        warnings: Vec<String>,
        status: String,
        state: String,
        fallback_reason: Option<String>,
        requested_mode: String,
        resolved_mode: String,
        selected_photo_id: Option<String>,
        selected_photo_status: String,
        library_root_path: Option<String>,
        catalog_path: Option<String>,
        schema_version: Option<i64>,
    },
    ImportSummary {
        folder_path: String,
        scanned_files: usize,
        supported_files: usize,
        unsupported_files: usize,
        issues: Vec<DesktopImportIssue>,
        originals_unchanged: bool,
    },
    PhotoGrid {
        photos: Vec<DesktopPhotoGridItem>,
    },
    PhotoGridPage {
        photos: Vec<DesktopPhotoGridItem>,
        offset: u64,
        limit: u16,
        total_count: u64,
        has_next_page: bool,
        order_fields: Vec<&'static str>,
    },
    PhotoFlags {
        photo_id: String,
        rating: u8,
        picked: bool,
        rejected: bool,
        color_label: Option<String>,
    },
    PhotoMetadata {
        photo_id: String,
        file_name: String,
        source_path: String,
        file_type: String,
        unsupported: bool,
        file_size: DesktopMetadataField<i64>,
        modified_at: DesktopMetadataField<String>,
        width: DesktopMetadataField<i64>,
        height: DesktopMetadataField<i64>,
        orientation: DesktopMetadataField<String>,
        capture_time: DesktopMetadataField<String>,
        camera_make: DesktopMetadataField<String>,
        camera_model: DesktopMetadataField<String>,
        lens_model: DesktopMetadataField<String>,
    },
    PhotoPreview {
        photo_id: String,
        file_name: String,
        source_path: String,
        preview_bytes: Option<Vec<u8>>,
        status: &'static str,
        message: String,
    },
    EditPreview {
        photo_id: String,
        source_path: String,
        status: &'static str,
        exposure: f64,
        contrast: f64,
        white_balance: &'static str,
        temperature: f64,
        tint: f64,
        highlights: f64,
        shadows: f64,
        whites: f64,
        blacks: f64,
        vibrance: f64,
        saturation: f64,
        tone_curve: DesktopToneCurveState,
        hsl_color_mixer: DesktopHslColorMixerState,
        detail: DesktopDetailState,
        geometry: DesktopGeometryState,
        masks: Vec<DesktopManualMaskState>,
        develop_preview_bytes: Option<Vec<u8>>,
        message: String,
    },
    EditCommit {
        photo_id: String,
        exposure: f64,
        contrast: f64,
        white_balance: &'static str,
        temperature: f64,
        tint: f64,
        highlights: f64,
        shadows: f64,
        whites: f64,
        blacks: f64,
        vibrance: f64,
        saturation: f64,
        tone_curve: DesktopToneCurveState,
        hsl_color_mixer: DesktopHslColorMixerState,
        detail: DesktopDetailState,
        geometry: DesktopGeometryState,
        masks: Vec<DesktopManualMaskState>,
        persisted: bool,
        message: String,
    },
    EditState {
        photo_id: String,
        exposure: f64,
        contrast: f64,
        white_balance: &'static str,
        temperature: f64,
        tint: f64,
        highlights: f64,
        shadows: f64,
        whites: f64,
        blacks: f64,
        vibrance: f64,
        saturation: f64,
        tone_curve: DesktopToneCurveState,
        hsl_color_mixer: DesktopHslColorMixerState,
        detail: DesktopDetailState,
        geometry: DesktopGeometryState,
        masks: Vec<DesktopManualMaskState>,
        persisted: bool,
        message: String,
    },
    EditClipboard {
        photo_id: String,
        selection: silica_core::EditClipboardSelection,
        payload: silica_core::EditClipboardPayload,
        section_count: usize,
        message: String,
    },
    EditClipboardPlan {
        status: String,
        requested_count: usize,
        ready_count: usize,
        unchanged_count: usize,
        blocked_count: usize,
        targets: Vec<DesktopEditClipboardTarget>,
        message: String,
    },
    EditClipboardSync {
        status: String,
        requested_count: usize,
        applied_count: usize,
        skipped_count: usize,
        blocked_count: usize,
        failed_count: usize,
        commits: Vec<DesktopEditClipboardCommit>,
        targets: Vec<DesktopEditClipboardTarget>,
        failures: Vec<DesktopEditClipboardFailure>,
        message: String,
    },
    HistoryCommand {
        photo_id: String,
        command: String,
        applied: bool,
        action_kind: Option<String>,
        history_id: Option<String>,
        message: String,
    },
    HistoryPanel {
        photo_id: String,
        items: Vec<DesktopHistoryItem>,
        can_undo: bool,
        can_redo: bool,
        status: String,
        message: String,
    },
    AiReviewPanel {
        photo_id: String,
        task_type: String,
        status: &'static str,
        message: String,
        editor_remains_usable: bool,
        requires_explicit_approval: bool,
        writes_edit_graph: bool,
        writes_photo_flags: bool,
        items: Vec<DesktopAiReviewItem>,
    },
    AiSuggestionApproval {
        photo_id: String,
        result_id: String,
        model_id: String,
        task_type: String,
        suggestion_kind: String,
        action_log_id: String,
        commit: DesktopAiSuggestionCommit,
        writes_edit_graph: bool,
        writes_photo_flags: bool,
        writes_original: bool,
        message: String,
    },
    AiSuggestionRejection {
        photo_id: String,
        result_id: String,
        model_id: String,
        task_type: String,
        action_log_id: String,
        edit_state_unchanged: bool,
        message: String,
    },
    Histogram {
        photo_id: String,
        source_path: String,
        status: &'static str,
        red: Vec<u32>,
        green: Vec<u32>,
        blue: Vec<u32>,
        luminance: Vec<u32>,
        pixel_count: u64,
        cache_key: String,
        cache_path: String,
        message: String,
    },
    ExportSettings {
        default_preset_id: Option<String>,
        default_settings: DesktopExportSettings,
        presets: Vec<DesktopExportPreset>,
        message: String,
    },
    RecentExports {
        exports: Vec<DesktopRecentExport>,
        message: String,
    },
    Export {
        photo_id: String,
        source_path: String,
        output_path: String,
        format: String,
        color_profile: String,
        bytes_written: u64,
        source_sha256: Option<String>,
        output_sha256: String,
        icc_profile_embedded: bool,
        icc_profile_sha256: String,
        decoder_backend: Option<String>,
        input_profile: Option<String>,
        working_space: Option<String>,
        export_record_id: String,
        message: String,
    },
    CacheClear {
        cleared_directories: Vec<String>,
        recreated_directories: Vec<String>,
        removed_cache_records: usize,
        message: String,
    },
    CacheStatus {
        library_root_path: String,
        directories: Vec<DesktopCacheDirectoryStatus>,
        total_bytes: u64,
        cache_record_count: usize,
        message: String,
    },
}

impl DesktopCommandData {
    #[cfg(test)]
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::LibrarySession { .. } => "librarySession",
            Self::AppSession { .. } => "appSession",
            Self::AppSessionWrite { .. } => "appSessionWrite",
            Self::AppSessionInspection { .. } => "appSessionInspection",
            Self::LaunchRestore { .. } => "launchRestore",
            Self::ImportSummary { .. } => "importSummary",
            Self::PhotoGrid { .. } => "photoGrid",
            Self::PhotoGridPage { .. } => "photoGridPage",
            Self::PhotoFlags { .. } => "photoFlags",
            Self::PhotoMetadata { .. } => "photoMetadata",
            Self::PhotoPreview { .. } => "photoPreview",
            Self::EditPreview { .. } => "editPreview",
            Self::EditCommit { .. } => "editCommit",
            Self::EditState { .. } => "editState",
            Self::EditClipboard { .. } => "editClipboard",
            Self::EditClipboardPlan { .. } => "editClipboardPlan",
            Self::EditClipboardSync { .. } => "editClipboardSync",
            Self::HistoryCommand { .. } => "historyCommand",
            Self::HistoryPanel { .. } => "historyPanel",
            Self::AiReviewPanel { .. } => "aiReviewPanel",
            Self::AiSuggestionApproval { .. } => "aiSuggestionApproval",
            Self::AiSuggestionRejection { .. } => "aiSuggestionRejection",
            Self::Histogram { .. } => "histogram",
            Self::ExportSettings { .. } => "exportSettings",
            Self::RecentExports { .. } => "recentExports",
            Self::Export { .. } => "export",
            Self::CacheClear { .. } => "cacheClear",
            Self::CacheStatus { .. } => "cacheStatus",
        }
    }

    #[cfg(test)]
    pub(crate) fn root_path(&self) -> Option<String> {
        match self {
            Self::LibrarySession { root_path, .. } => Some(root_path.clone()),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn catalog_path(&self) -> Option<String> {
        match self {
            Self::LibrarySession { catalog_path, .. } => Some(catalog_path.clone()),
            _ => None,
        }
    }
}

pub(crate) fn export_settings_catalog_data(
    catalog: silica_core::ExportSettingsCatalog,
) -> DesktopCommandData {
    DesktopCommandData::ExportSettings {
        default_preset_id: catalog.default_preset_id,
        default_settings: desktop_export_settings(catalog.default_settings),
        presets: catalog
            .presets
            .into_iter()
            .map(desktop_export_preset)
            .collect(),
        message: "Export settings loaded.".to_string(),
    }
}

pub(crate) fn library_session_data(session: silica_core::LibrarySession) -> DesktopCommandData {
    DesktopCommandData::LibrarySession {
        root_path: session.root_path.display().to_string(),
        catalog_path: session.catalog_path.display().to_string(),
        schema_version: session.schema_version,
    }
}

pub(crate) fn app_session_data(
    session_path: PathBuf,
    loaded: silica_core::AppSessionLoadResult,
) -> DesktopCommandData {
    DesktopCommandData::AppSession {
        session_path: session_path.display().to_string(),
        session: DesktopAppSession::from_core(loaded.session),
        warnings: app_session_warning_strings(&loaded.warnings),
    }
}

pub(crate) fn photo_flags_data(flags: silica_core::PhotoFlags) -> DesktopCommandData {
    DesktopCommandData::PhotoFlags {
        photo_id: flags.photo_id,
        rating: flags.rating,
        picked: flags.picked,
        rejected: flags.rejected,
        color_label: flags.color_label,
    }
}

pub(crate) fn edit_clipboard_data(
    photo_id: String,
    selection: silica_core::EditClipboardSelection,
    payload: silica_core::EditClipboardPayload,
) -> DesktopCommandData {
    let section_count = edit_clipboard_section_count(&selection);
    DesktopCommandData::EditClipboard {
        photo_id,
        selection,
        payload,
        section_count,
        message: format!("Copied {section_count} edit section(s)."),
    }
}

pub(crate) fn edit_clipboard_plan_data(
    plan: silica_core::BatchEditClipboardSyncPlan,
) -> DesktopCommandData {
    DesktopCommandData::EditClipboardPlan {
        status: plan.status,
        requested_count: plan.requested_count,
        ready_count: plan.ready_count,
        unchanged_count: plan.unchanged_count,
        blocked_count: plan.blocked_count,
        targets: plan
            .targets
            .into_iter()
            .map(edit_clipboard_target_data)
            .collect(),
        message: plan.message,
    }
}

pub(crate) fn edit_clipboard_sync_data(
    result: silica_core::BatchEditClipboardSyncResult,
) -> DesktopCommandData {
    DesktopCommandData::EditClipboardSync {
        status: result.status,
        requested_count: result.requested_count,
        applied_count: result.applied_count,
        skipped_count: result.skipped_count,
        blocked_count: result.blocked_count,
        failed_count: result.failed_count,
        commits: result
            .commits
            .into_iter()
            .map(|commit| DesktopEditClipboardCommit {
                photo_id: commit.photo_id,
                history_id: commit.history_id,
                sequence: commit.sequence,
            })
            .collect(),
        targets: result
            .targets
            .into_iter()
            .map(edit_clipboard_target_data)
            .collect(),
        failures: result
            .failures
            .into_iter()
            .map(|failure| DesktopEditClipboardFailure {
                photo_id: failure.photo_id,
                message: failure.message,
            })
            .collect(),
        message: result.message,
    }
}

pub(crate) fn history_command_data(
    result: silica_core::HistoryCommandResult,
) -> DesktopCommandData {
    DesktopCommandData::HistoryCommand {
        photo_id: result.photo_id,
        command: result.command,
        applied: result.applied,
        action_kind: result.action_kind,
        history_id: result.history_id,
        message: result.message,
    }
}

pub(crate) fn photo_history_panel_data(
    panel: silica_core::PhotoHistoryPanel,
) -> DesktopCommandData {
    DesktopCommandData::HistoryPanel {
        photo_id: panel.photo_id,
        items: panel
            .items
            .into_iter()
            .map(DesktopHistoryItem::from)
            .collect(),
        can_undo: panel.can_undo,
        can_redo: panel.can_redo,
        status: panel.status,
        message: panel.message,
    }
}

pub(crate) fn ai_review_panel_data(panel: silica_core::AiReviewPanel) -> DesktopCommandData {
    DesktopCommandData::AiReviewPanel {
        photo_id: panel.photo_id,
        task_type: panel.task_type,
        status: ai_review_status_text(panel.status),
        message: panel.message,
        editor_remains_usable: panel.editor_remains_usable,
        requires_explicit_approval: panel.requires_explicit_approval,
        writes_edit_graph: panel.writes_edit_graph,
        writes_photo_flags: panel.writes_photo_flags,
        items: panel
            .items
            .into_iter()
            .map(DesktopAiReviewItem::from)
            .collect(),
    }
}

pub(crate) fn ai_suggestion_approval_data(
    approval: silica_core::AiSuggestionApproval,
) -> DesktopCommandData {
    DesktopCommandData::AiSuggestionApproval {
        photo_id: approval.photo_id,
        result_id: approval.result_id,
        model_id: approval.model_id,
        task_type: approval.task_type,
        suggestion_kind: approval.suggestion_kind,
        action_log_id: approval.action_log_id,
        commit: approval.commit.into(),
        writes_edit_graph: approval.writes_edit_graph,
        writes_photo_flags: approval.writes_photo_flags,
        writes_original: approval.writes_original,
        message: "AI suggestion approved as an undoable edit checkpoint.".to_string(),
    }
}

pub(crate) fn ai_suggestion_rejection_data(
    rejection: silica_core::AiSuggestionRejection,
) -> DesktopCommandData {
    DesktopCommandData::AiSuggestionRejection {
        photo_id: rejection.photo_id,
        result_id: rejection.result_id,
        model_id: rejection.model_id,
        task_type: rejection.task_type,
        action_log_id: rejection.action_log_id,
        edit_state_unchanged: rejection.edit_state_unchanged,
        message: "AI suggestion rejected; edit state is unchanged.".to_string(),
    }
}

pub(crate) fn photo_metadata_data(metadata: silica_core::PhotoMetadata) -> DesktopCommandData {
    DesktopCommandData::PhotoMetadata {
        photo_id: metadata.photo_id,
        file_name: metadata.file_name,
        source_path: metadata.source_path,
        file_type: metadata.file_type,
        unsupported: metadata.unsupported,
        file_size: metadata_field(metadata.file_size),
        modified_at: metadata_field(metadata.modified_at),
        width: metadata_field(metadata.width),
        height: metadata_field(metadata.height),
        orientation: metadata_field(metadata.orientation),
        capture_time: metadata_field(metadata.capture_time),
        camera_make: metadata_field(metadata.camera_make),
        camera_model: metadata_field(metadata.camera_model),
        lens_model: metadata_field(metadata.lens_model),
    }
}

pub(crate) fn photo_grid_page_data(
    page: silica_core::LibraryQueryPage<silica_core::LibraryPhotoGridItem>,
) -> DesktopCommandData {
    DesktopCommandData::PhotoGridPage {
        photos: page
            .items
            .into_iter()
            .map(DesktopPhotoGridItem::from)
            .collect(),
        offset: page.offset,
        limit: page.limit,
        total_count: page.total_count,
        has_next_page: page.has_next_page,
        order_fields: page
            .order_fields
            .iter()
            .copied()
            .map(library_query_order_field_string)
            .collect(),
    }
}
