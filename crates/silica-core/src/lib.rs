//! Core coordination boundary for SilicaRAW.
//!
//! The crate root keeps the stable public API while the implementation lives in
//! private modules.

use std::path::Path;
use std::path::PathBuf;

mod clipboard;
mod common;
mod develop;
mod error;
mod export;
mod library;
mod permissions;
mod pipeline;
mod raw;
mod session;
mod trust;

pub use silica_decode::RawFullResolutionExportSourceError;
pub use silica_decode::RawPreviewArtifactError;
pub use silica_edit::BasicPreset;
pub use silica_edit::CurveMode;
pub use silica_edit::EditClipboardPayload;
pub use silica_edit::EditClipboardSelection;
pub use silica_edit::HslColorChannel;
pub use silica_edit::PluginBasicPresetAdjustments;
pub use silica_edit::WhiteBalance;
pub use silica_plugin::{PluginBasicPreset, PluginManifest, PluginPreset, PluginPresetPack};
pub use silica_storage::ActionLogEntry;
pub use silica_storage::AiResult;
pub use silica_storage::CatalogRebuildDryRunAction;
pub use silica_storage::CatalogRebuildDryRunEntry;
pub use silica_storage::CatalogRebuildDryRunIssue;
pub use silica_storage::CatalogRebuildDryRunIssueKind;
pub use silica_storage::CatalogRebuildDryRunReport;
pub use silica_storage::CatalogRebuildFlagSource;
pub use silica_storage::ExportPreset;
pub use silica_storage::ExportSettings;
pub use silica_storage::ExportSettingsCatalog;
pub use silica_storage::FolderImportOptions;
pub use silica_storage::HistoryCommandResult;
pub use silica_storage::ImportIssue;
pub use silica_storage::ImportIssueKind;
pub use silica_storage::LibraryPhotoGridItem;
pub use silica_storage::LibraryQueryFileType;
pub use silica_storage::LibraryQueryFilters;
pub use silica_storage::LibraryQueryMetadataFilter;
pub use silica_storage::LibraryQueryOrderField;
pub use silica_storage::LibraryQueryPage;
pub use silica_storage::LibraryQueryRequest;
pub use silica_storage::LibraryQuerySort;
pub use silica_storage::NewActionLogEntry;
pub use silica_storage::PhotoFlags;
pub use silica_storage::PhotoHistoryItem;
pub use silica_storage::PhotoHistoryPanel;
pub use silica_storage::PhotoMetadata;
pub use silica_storage::PhotoMetadataField;
pub use silica_storage::PhotoMetadataFieldState;
pub use silica_storage::PhotoSidecarStatus;
pub use silica_storage::SidecarWriteResult;
pub use silica_storage::ValidatedSidecar;

pub use clipboard::{
    apply_edit_clipboard_payload_to_graph, apply_edit_clipboard_sync, copy_edit_clipboard_payload,
    copy_photo_edit_clipboard_payload, plan_edit_clipboard_sync,
    sync_edit_clipboard_payload_to_photos, BatchEditClipboardSyncCommit,
    BatchEditClipboardSyncFailure, BatchEditClipboardSyncPlan, BatchEditClipboardSyncResult,
    BatchEditClipboardSyncTarget,
};
pub use develop::{
    commit_basic_preset_edit, commit_clear_geometry_crop, commit_color_presence_edit,
    commit_detail_noise_reduction_edit, commit_detail_sharpening_edit,
    commit_exposure_contrast_edit, commit_geometry_crop_edit, commit_geometry_orientation_edit,
    commit_hsl_color_mixer_edit, commit_manual_brush_mask, commit_manual_linear_gradient_mask,
    commit_manual_radial_gradient_mask, commit_p0_basic_reset, commit_tone_curve_edit,
    commit_tone_recovery_edit, commit_white_balance_edit, get_photo_edit_state,
    plan_exposure_contrast_metal_draft, preview_clear_geometry_crop, preview_color_presence_edit,
    preview_detail_noise_reduction_edit, preview_detail_sharpening_edit,
    preview_exposure_contrast_edit, preview_geometry_crop_edit, preview_geometry_orientation_edit,
    preview_hsl_color_mixer_edit, preview_manual_brush_mask, preview_manual_linear_gradient_mask,
    preview_manual_radial_gradient_mask, preview_tone_curve_edit, preview_tone_recovery_edit,
    preview_white_balance_edit, PhotoDetailNoiseReductionState, PhotoDetailSharpeningState,
    PhotoDetailState, PhotoEditCommit, PhotoEditPreviewSession, PhotoEditState,
    PhotoGeometryCropState, PhotoGeometryState, PhotoGeometryTransformState,
    PhotoHslColorChannelState, PhotoHslColorMixerState, PhotoManualBrushPointInput,
    PhotoManualBrushStrokeInput, PhotoManualMaskGeometryState, PhotoManualMaskState,
    PhotoToneCurvePoint, PhotoToneCurveState,
};
pub use error::CoreError;
pub use export::{
    export_photo_jpeg, export_photo_jpeg_srgb, export_photo_jpeg_with_metadata_policy,
    export_photo_png, export_photo_tiff, export_raw_photo_jpeg_srgb_from_probe,
    get_export_settings_catalog, list_recent_exports, set_default_export_settings,
    upsert_export_preset, PhotoExportColorProfile, PhotoExportMetadataPolicy, PhotoExportSession,
    PhotoRecentExport,
};
pub use library::{
    clear_library_cache, create_library, dry_run_catalog_rebuild_from_sidecars,
    get_library_cache_status, get_mcp_photo_read_record, get_photo_flags, get_photo_histogram,
    get_photo_metadata, get_photo_sidecar_status, import_folder, import_folder_with_options,
    list_library_photos, list_library_photos_json, list_photo_history,
    metadata_extraction_policy_for_path, open_library, open_photo_preview, query_library_photos,
    query_library_photos_with_thumbnail_hydration, read_photo_sidecar, redo_last_history_action,
    set_photo_flags, undo_last_history_action, write_photo_sidecar, LibraryCacheClearSession,
    LibraryCacheDirectoryStatus, LibraryCacheStatusSession, PhotoHistogramSession,
    PhotoHistogramStatus, PhotoPreviewSession, PhotoPreviewStatus,
};
pub use permissions::{
    ExtensionPermission, ExtensionPermissionCategory, ExtensionPermissionPolicy, McpMode,
    CRATE_NAME,
};
pub use raw::{
    plan_decoded_image_viewer_handoff, plan_product_raw_decode, plan_product_raw_decode_from_probe,
    write_raw_preview_artifact_for_probe, DecodedImageViewerHandoffPlan, RawPreviewArtifactSession,
};
pub use session::{
    default_app_appearance_preferences, default_app_layout_preferences,
    default_app_library_preferences, load_app_session, plan_app_session_restore,
    record_app_session_appearance, record_app_session_layout,
    record_app_session_library_preferences, record_app_session_library_state,
    record_app_session_recent_library, reset_app_session_appearance, reset_app_session_layout,
    reset_app_session_library_preferences, write_app_session, AppAppearanceDensity,
    AppAppearancePreferences, AppAppearanceTheme, AppFileTypeFilter, AppLayoutPreferences,
    AppLibraryPreferences, AppLibrarySort, AppMetadataFilter, AppPerLibrarySession,
    AppRecentLibrary, AppSession, AppSessionFilters, AppSessionLoadResult, AppSessionMode,
    AppSessionRestorePlan, AppSessionRestoreStatus, AppSessionSelectedPhotoStatus,
    AppSessionWarning, AppSessionWriteResult, LibrarySession, APP_SESSION_RECENTS_LIMIT,
    APP_SESSION_SCHEMA, APP_SESSION_VERSION, DEFAULT_APP_SESSION_THUMBNAIL_SIZE,
    DEFAULT_APP_SESSION_UI_SCALE, MAX_APP_SESSION_THUMBNAIL_SIZE, MAX_APP_SESSION_UI_SCALE,
    MIN_APP_SESSION_THUMBNAIL_SIZE, MIN_APP_SESSION_UI_SCALE,
};
pub use trust::{
    append_action_log_entry, approve_ai_suggestion, approve_plugin_preset, get_ai_review_panel,
    list_action_log_entries, list_ai_results_for_photo, record_ai_approval, record_mcp_read,
    record_permission_decision, record_permissioned_export_attempt, record_plugin_apply_attempt,
    reject_ai_suggestion, review_plugin_apply_permission, review_plugin_enable_permission,
    store_ai_result, AiReviewItem, AiReviewPanel, AiReviewPanelStatus, AiSuggestionApproval,
    AiSuggestionRejection, McpPhotoReadRecord, PluginPermissionReview, PluginPresetApproval,
    AI_REVIEW_BLUR_TASK_TYPE,
};

use common::{
    current_timestamp_string, LOCAL_ALPHA_BRUSH_MASK_RASTER_EDGE,
    LOCAL_ALPHA_DEVELOP_PREVIEW_QUALITY, LOCAL_ALPHA_JPEG_QUALITY,
    LOCAL_ALPHA_LOUPE_PREVIEW_MAX_EDGE, LOCAL_ALPHA_LOUPE_PREVIEW_QUALITY,
    LOCAL_ALPHA_THUMBNAIL_MAX_EDGE, LOCAL_ALPHA_THUMBNAIL_QUALITY,
};
use develop::{mark_runtime_missing_source, photo_edit_commit_from_graph, preview_render_plan};
use export::PhotoExportFormat;
use library::{
    is_supported_raster_source_path, record_brush_mask_raster_caches,
    write_jpeg_develop_preview_bytes,
};
use pipeline::{
    apply_detail_preview_boundary, apply_lens_geometry_preview_boundary,
    apply_manual_mask_preview_boundary, detail_settings_json, detail_state_from_graph,
    detail_unsupported_message, edit_graphs_equal_ignoring_updated_at,
    ensure_no_active_manual_masks_for_export, ensure_supported_develop_source,
    ensure_supported_lens_geometry_commit, ensure_supported_lens_geometry_export,
    ensure_supported_manual_masks_commit, export_color_presence_from_render,
    export_color_profile_string, export_color_profile_to_export, export_detail_from_render,
    export_format_string, export_geometry_from_render, export_hsl_color_mixer_from_render,
    export_manual_masks_from_render, export_metadata_policy_string,
    export_metadata_policy_to_export, export_profile_metadata_source,
    export_raster_format_to_export, export_raster_message, export_tone_curve_from_render,
    export_tone_recovery_from_render, export_white_balance_from_render, geometry_settings_json,
    geometry_state_from_graph, geometry_unsupported_message, has_unsupported_basic_runtime,
    hsl_color_mixer_settings_json, hsl_color_mixer_state_from_graph, is_supported_raster_file_type,
    lens_unsupported_message, manual_mask_settings_json, photo_manual_masks_from_graph,
    preview_status_from_render, render_color_presence_from_graph, render_detail_from_graph,
    render_geometry_from_graph, render_hsl_color_mixer_from_graph, render_manual_masks_from_graph,
    render_tone_curve_from_graph, render_tone_recovery_from_graph, render_white_balance_from_graph,
    tone_curve_settings_json, tone_curve_state_from_graph, white_balance_render_mode_string,
};
use trust::append_core_action_log;

#[cfg(test)]
mod tests;
