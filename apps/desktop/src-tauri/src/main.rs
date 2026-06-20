#[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
mod metal_host_spike;
#[cfg(all(target_os = "macos", feature = "native-metal-viewer"))]
mod native_metal_viewer;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tauri::{path::BaseDirectory, Manager};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCommandResponse {
    ok: bool,
    command: &'static str,
    message: String,
    data: Option<DesktopCommandData>,
    error: Option<DesktopCommandError>,
}

impl DesktopCommandResponse {
    fn ok(command: &'static str, message: impl Into<String>, data: DesktopCommandData) -> Self {
        Self {
            ok: true,
            command,
            message: message.into(),
            data: Some(data),
            error: None,
        }
    }

    fn empty(command: &'static str, message: impl Into<String>) -> Self {
        Self {
            ok: true,
            command,
            message: message.into(),
            data: None,
            error: None,
        }
    }

    fn error(
        command: &'static str,
        error: silica_core::CoreError,
        context: DesktopCommandContext,
    ) -> Self {
        let kind = core_error_kind(&error).to_string();
        let message = error.to_string();
        Self {
            ok: false,
            command,
            message: message.clone(),
            data: None,
            error: Some(DesktopCommandError {
                kind,
                message,
                context,
            }),
        }
    }

    fn error_message(
        command: &'static str,
        message: impl Into<String>,
        kind: impl Into<String>,
        context: DesktopCommandContext,
    ) -> Self {
        let message = message.into();
        Self {
            ok: false,
            command,
            message: message.clone(),
            data: None,
            error: Some(DesktopCommandError {
                kind: kind.into(),
                message,
                context,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopToneCurveState {
    curve_mode: &'static str,
    rgb_curve: Vec<DesktopToneCurvePoint>,
    red_curve: Vec<DesktopToneCurvePoint>,
    green_curve: Vec<DesktopToneCurvePoint>,
    blue_curve: Vec<DesktopToneCurvePoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHslColorChannelState {
    hue: f64,
    saturation: f64,
    luminance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHslColorMixerState {
    red: DesktopHslColorChannelState,
    orange: DesktopHslColorChannelState,
    yellow: DesktopHslColorChannelState,
    green: DesktopHslColorChannelState,
    aqua: DesktopHslColorChannelState,
    blue: DesktopHslColorChannelState,
    purple: DesktopHslColorChannelState,
    magenta: DesktopHslColorChannelState,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopDetailSharpeningState {
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopDetailNoiseReductionState {
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopDetailState {
    sharpening: DesktopDetailSharpeningState,
    noise_reduction: DesktopDetailNoiseReductionState,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopGeometryCropState {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopGeometryTransformState {
    vertical: f64,
    horizontal: f64,
    aspect: f64,
    scale: f64,
    x_offset: f64,
    y_offset: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopGeometryState {
    crop: Option<DesktopGeometryCropState>,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
    transform: DesktopGeometryTransformState,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    tag = "kind",
    rename_all_fields = "camelCase"
)]
enum DesktopManualMaskGeometryState {
    LinearGradient {
        start_x: f64,
        start_y: f64,
        end_x: f64,
        end_y: f64,
    },
    RadialGradient {
        center_x: f64,
        center_y: f64,
        radius_x: f64,
        radius_y: f64,
        rotation: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopManualMaskState {
    id: String,
    kind: String,
    name: String,
    enabled: bool,
    invert: bool,
    opacity: f64,
    feather: f64,
    geometry: Option<DesktopManualMaskGeometryState>,
    exposure: f64,
    contrast: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopToneCurvePoint {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopEditClipboardTarget {
    photo_id: String,
    status: String,
    code: Option<String>,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopEditClipboardCommit {
    photo_id: String,
    history_id: String,
    sequence: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopEditClipboardFailure {
    photo_id: String,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopExportSettings {
    format: String,
    color_profile: String,
    quality: u8,
    metadata_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopExportPreset {
    id: String,
    name: String,
    settings: DesktopExportSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopRecentExport {
    export_record_id: String,
    photo_id: String,
    output_path: String,
    export_settings_json: String,
    created_at: String,
    output_exists: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(
    rename_all = "camelCase",
    tag = "kind",
    rename_all_fields = "camelCase"
)]
enum DesktopCommandData {
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
    fn kind(&self) -> &'static str {
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
    fn root_path(&self) -> Option<String> {
        match self {
            Self::LibrarySession { root_path, .. } => Some(root_path.clone()),
            _ => None,
        }
    }

    #[cfg(test)]
    fn catalog_path(&self) -> Option<String> {
        match self {
            Self::LibrarySession { catalog_path, .. } => Some(catalog_path.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCacheDirectoryStatus {
    name: String,
    path: String,
    exists: bool,
    byte_size: u64,
    file_count: u64,
}

impl From<silica_core::LibraryCacheDirectoryStatus> for DesktopCacheDirectoryStatus {
    fn from(directory: silica_core::LibraryCacheDirectoryStatus) -> Self {
        Self {
            name: directory.name,
            path: directory.path.display().to_string(),
            exists: directory.exists,
            byte_size: directory.byte_size,
            file_count: directory.file_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAppSession {
    schema: String,
    version: i64,
    last_library_root_path: Option<String>,
    last_mode: String,
    recents: Vec<DesktopRecentLibrary>,
    #[serde(default)]
    appearance: DesktopAppearancePreferences,
    #[serde(default)]
    library: DesktopLibraryPreferences,
    layout: DesktopLayoutPreferences,
    per_library: BTreeMap<String, DesktopPerLibrarySession>,
}

impl Default for DesktopAppSession {
    fn default() -> Self {
        Self::from_core(silica_core::AppSession::default())
    }
}

impl DesktopAppSession {
    fn from_core(session: silica_core::AppSession) -> Self {
        Self {
            schema: session.schema,
            version: session.version,
            last_library_root_path: session
                .last_library_root_path
                .map(|path| path.display().to_string()),
            last_mode: app_session_mode_string(session.last_mode).to_string(),
            recents: session
                .recents
                .into_iter()
                .map(DesktopRecentLibrary::from_core)
                .collect(),
            appearance: DesktopAppearancePreferences::from_core(session.appearance),
            library: DesktopLibraryPreferences::from_core(session.library),
            layout: DesktopLayoutPreferences::from_core(session.layout),
            per_library: session
                .per_library
                .into_iter()
                .map(|(key, value)| (key, DesktopPerLibrarySession::from_core(value)))
                .collect(),
        }
    }

    fn into_core(self) -> Result<silica_core::AppSession, silica_core::CoreError> {
        if self.schema != silica_core::APP_SESSION_SCHEMA {
            return Err(silica_core::CoreError::AppSession(format!(
                "invalid app session schema: {}",
                self.schema
            )));
        }
        if self.version != silica_core::APP_SESSION_VERSION {
            return Err(silica_core::CoreError::AppSession(format!(
                "invalid app session version: {}",
                self.version
            )));
        }

        Ok(silica_core::AppSession {
            schema: self.schema,
            version: self.version,
            last_library_root_path: self.last_library_root_path.map(PathBuf::from),
            last_mode: parse_desktop_app_session_mode(&self.last_mode)?,
            recents: self
                .recents
                .into_iter()
                .map(DesktopRecentLibrary::into_core)
                .collect(),
            appearance: self.appearance.into_core()?,
            library: self.library.into_core(),
            layout: self.layout.into_core()?,
            per_library: self
                .per_library
                .into_iter()
                .map(|(key, value)| value.into_core().map(|value| (key, value)))
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopRecentLibrary {
    root_path: String,
    display_name: String,
    last_opened_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    available: Option<bool>,
}

impl DesktopRecentLibrary {
    fn from_core(recent: silica_core::AppRecentLibrary) -> Self {
        let available = recent.root_path.join("catalog.db").is_file();
        Self {
            root_path: recent.root_path.display().to_string(),
            display_name: recent.display_name,
            last_opened_at: recent.last_opened_at,
            available: Some(available),
        }
    }

    fn into_core(self) -> silica_core::AppRecentLibrary {
        silica_core::AppRecentLibrary {
            root_path: PathBuf::from(self.root_path),
            display_name: self.display_name,
            last_opened_at: self.last_opened_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopPerLibrarySession {
    selected_photo_id: Option<String>,
    last_mode: String,
    last_opened_at: String,
}

impl DesktopPerLibrarySession {
    fn from_core(session: silica_core::AppPerLibrarySession) -> Self {
        Self {
            selected_photo_id: session.selected_photo_id,
            last_mode: app_session_mode_string(session.last_mode).to_string(),
            last_opened_at: session.last_opened_at,
        }
    }

    fn into_core(self) -> Result<silica_core::AppPerLibrarySession, silica_core::CoreError> {
        Ok(silica_core::AppPerLibrarySession {
            selected_photo_id: self.selected_photo_id,
            last_mode: parse_desktop_app_session_mode(&self.last_mode)?,
            last_opened_at: self.last_opened_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAppearancePreferences {
    theme: String,
    density: String,
    ui_scale: u16,
}

impl Default for DesktopAppearancePreferences {
    fn default() -> Self {
        Self::from_core(silica_core::AppAppearancePreferences::default())
    }
}

impl DesktopAppearancePreferences {
    fn from_core(appearance: silica_core::AppAppearancePreferences) -> Self {
        Self {
            theme: app_appearance_theme_string(appearance.theme).to_string(),
            density: app_appearance_density_string(appearance.density).to_string(),
            ui_scale: appearance.ui_scale,
        }
    }

    fn into_core(self) -> Result<silica_core::AppAppearancePreferences, silica_core::CoreError> {
        if self.ui_scale < silica_core::MIN_APP_SESSION_UI_SCALE
            || self.ui_scale > silica_core::MAX_APP_SESSION_UI_SCALE
        {
            return Err(silica_core::CoreError::AppSession(format!(
                "invalid app session ui scale: {}",
                self.ui_scale
            )));
        }

        Ok(silica_core::AppAppearancePreferences {
            theme: parse_desktop_app_appearance_theme(&self.theme)?,
            density: parse_desktop_app_appearance_density(&self.density)?,
            ui_scale: self.ui_scale,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopLibraryPreferences {
    default_library_root_path: Option<String>,
}

impl Default for DesktopLibraryPreferences {
    fn default() -> Self {
        Self::from_core(silica_core::AppLibraryPreferences::default())
    }
}

impl DesktopLibraryPreferences {
    fn from_core(library: silica_core::AppLibraryPreferences) -> Self {
        Self {
            default_library_root_path: library
                .default_library_root_path
                .map(|path| path.display().to_string()),
        }
    }

    fn into_core(self) -> silica_core::AppLibraryPreferences {
        silica_core::AppLibraryPreferences {
            default_library_root_path: self.default_library_root_path.map(PathBuf::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopLayoutPreferences {
    sidebar_collapsed: bool,
    inspector_collapsed: bool,
    filmstrip_visible: bool,
    thumbnail_size: u16,
    sort: String,
    filters: DesktopSessionFilters,
}

impl DesktopLayoutPreferences {
    fn from_core(layout: silica_core::AppLayoutPreferences) -> Self {
        Self {
            sidebar_collapsed: layout.sidebar_collapsed,
            inspector_collapsed: layout.inspector_collapsed,
            filmstrip_visible: layout.filmstrip_visible,
            thumbnail_size: layout.thumbnail_size,
            sort: app_library_sort_string(layout.sort).to_string(),
            filters: DesktopSessionFilters::from_core(layout.filters),
        }
    }

    fn into_core(self) -> Result<silica_core::AppLayoutPreferences, silica_core::CoreError> {
        if self.thumbnail_size < silica_core::MIN_APP_SESSION_THUMBNAIL_SIZE
            || self.thumbnail_size > silica_core::MAX_APP_SESSION_THUMBNAIL_SIZE
        {
            return Err(silica_core::CoreError::AppSession(format!(
                "invalid app session thumbnail size: {}",
                self.thumbnail_size
            )));
        }

        Ok(silica_core::AppLayoutPreferences {
            sidebar_collapsed: self.sidebar_collapsed,
            inspector_collapsed: self.inspector_collapsed,
            filmstrip_visible: self.filmstrip_visible,
            thumbnail_size: self.thumbnail_size,
            sort: parse_desktop_app_library_sort(&self.sort)?,
            filters: self.filters.into_core()?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DesktopSessionFilters {
    min_rating: Option<u8>,
    picked: Option<bool>,
    rejected: Option<bool>,
    file_type: Option<String>,
    metadata: Option<String>,
    search: String,
}

impl DesktopSessionFilters {
    fn from_core(filters: silica_core::AppSessionFilters) -> Self {
        Self {
            min_rating: filters.min_rating,
            picked: filters.picked,
            rejected: filters.rejected,
            file_type: filters
                .file_type
                .map(app_file_type_filter_string)
                .map(str::to_string),
            metadata: filters
                .metadata
                .map(app_metadata_filter_string)
                .map(str::to_string),
            search: filters.search,
        }
    }

    fn into_core(self) -> Result<silica_core::AppSessionFilters, silica_core::CoreError> {
        if self.min_rating.is_some_and(|rating| rating > 5) {
            return Err(silica_core::CoreError::AppSession(format!(
                "invalid app session min rating: {}",
                self.min_rating.unwrap_or_default()
            )));
        }

        Ok(silica_core::AppSessionFilters {
            min_rating: self.min_rating,
            picked: self.picked,
            rejected: self.rejected,
            file_type: self
                .file_type
                .as_deref()
                .map(parse_desktop_app_file_type_filter)
                .transpose()?,
            metadata: self
                .metadata
                .as_deref()
                .map(parse_desktop_app_metadata_filter)
                .transpose()?,
            search: self.search,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DesktopLibraryQueryRequest {
    offset: u64,
    limit: u16,
    sort: String,
    #[serde(default)]
    filters: DesktopLibraryQueryFilters,
}

impl DesktopLibraryQueryRequest {
    fn into_core(self) -> Result<silica_core::LibraryQueryRequest, silica_core::CoreError> {
        Ok(silica_core::LibraryQueryRequest::new(
            self.offset,
            self.limit,
            parse_desktop_library_query_sort(&self.sort)?,
            self.filters.into_core()?,
        ))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
struct DesktopLibraryQueryFilters {
    min_rating: Option<u8>,
    picked: Option<bool>,
    rejected: Option<bool>,
    file_type: Option<String>,
    metadata: Option<String>,
    search: String,
}

impl DesktopLibraryQueryFilters {
    fn into_core(self) -> Result<silica_core::LibraryQueryFilters, silica_core::CoreError> {
        if self.min_rating.is_some_and(|rating| rating > 5) {
            return Err(silica_core::CoreError::AppSession(format!(
                "invalid library query min rating: {}",
                self.min_rating.unwrap_or_default()
            )));
        }

        Ok(silica_core::LibraryQueryFilters {
            min_rating: self.min_rating,
            picked: self.picked,
            rejected: self.rejected,
            file_type: self
                .file_type
                .as_deref()
                .map(parse_desktop_library_query_file_type)
                .transpose()?,
            metadata: self
                .metadata
                .as_deref()
                .map(parse_desktop_library_query_metadata)
                .transpose()?,
            search: self.search,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopImportIssue {
    kind: &'static str,
    path: String,
    file_name: Option<String>,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopPhotoGridItem {
    photo_id: String,
    file_name: String,
    path: String,
    file_type: String,
    thumbnail_path: Option<String>,
    thumbnail_bytes: Option<Vec<u8>>,
    missing: bool,
    unsupported: bool,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHistoryItem {
    history_id: String,
    photo_id: String,
    sequence: i64,
    action_kind: String,
    label: String,
    history_state: String,
    can_undo: bool,
    can_redo: bool,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAiReviewItem {
    result_id: String,
    model_id: String,
    label: String,
    recommendation: String,
    approvable: bool,
    confidence_percent: Option<u8>,
    approved: bool,
    created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopAiSuggestionCommit {
    photo_id: String,
    exposure: f64,
    contrast: f64,
    persisted: bool,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopMetadataField<T> {
    state: &'static str,
    value: Option<T>,
}

impl From<silica_core::LibraryPhotoGridItem> for DesktopPhotoGridItem {
    fn from(photo: silica_core::LibraryPhotoGridItem) -> Self {
        Self {
            photo_id: photo.photo_id,
            file_name: photo.file_name,
            path: photo.path,
            file_type: photo.file_type,
            thumbnail_bytes: photo
                .thumbnail_path
                .as_ref()
                .and_then(|path| std::fs::read(path).ok()),
            thumbnail_path: photo.thumbnail_path,
            missing: photo.missing,
            unsupported: photo.unsupported,
            rating: photo.rating,
            picked: photo.picked,
            rejected: photo.rejected,
            color_label: photo.color_label,
        }
    }
}

impl From<silica_core::PhotoHistoryItem> for DesktopHistoryItem {
    fn from(item: silica_core::PhotoHistoryItem) -> Self {
        Self {
            history_id: item.history_id,
            photo_id: item.photo_id,
            sequence: item.sequence,
            action_kind: item.action_kind,
            label: item.label,
            history_state: item.history_state,
            can_undo: item.can_undo,
            can_redo: item.can_redo,
            created_at: item.created_at,
        }
    }
}

impl From<silica_core::AiReviewItem> for DesktopAiReviewItem {
    fn from(item: silica_core::AiReviewItem) -> Self {
        Self {
            result_id: item.result_id,
            model_id: item.model_id,
            label: item.label,
            recommendation: item.recommendation,
            approvable: item.approvable,
            confidence_percent: item.confidence_percent,
            approved: item.approved,
            created_at: item.created_at,
        }
    }
}

impl From<silica_core::PhotoEditCommit> for DesktopAiSuggestionCommit {
    fn from(commit: silica_core::PhotoEditCommit) -> Self {
        Self {
            photo_id: commit.photo_id,
            exposure: commit.exposure,
            contrast: commit.contrast,
            persisted: commit.persisted,
            message: commit.message,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCommandContext {
    library_path: Option<String>,
    folder_path: Option<String>,
    output_path: Option<String>,
    photo_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopCommandError {
    kind: String,
    message: String,
    context: DesktopCommandContext,
}

#[tauri::command]
fn read_app_session(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => read_app_session_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "read_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn write_app_session(app: tauri::AppHandle, session: DesktopAppSession) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => write_app_session_at_path(session_path, session),
        Err(error) => DesktopCommandResponse::error(
            "write_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn reset_app_session(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn record_app_session_layout(
    app: tauri::AppHandle,
    layout: DesktopLayoutPreferences,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_layout_at_path(session_path, layout),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_layout",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn reset_app_session_layout(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_layout_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session_layout",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn record_app_session_appearance(
    app: tauri::AppHandle,
    appearance: DesktopAppearancePreferences,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_appearance_at_path(session_path, appearance),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_appearance",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn reset_app_session_appearance(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_appearance_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session_appearance",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn record_app_session_library_preferences(
    app: tauri::AppHandle,
    library: DesktopLibraryPreferences,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_library_preferences_at_path(session_path, library),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_library_preferences",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn reset_app_session_library_preferences(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => reset_app_session_library_preferences_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "reset_app_session_library_preferences",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn inspect_app_session(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => inspect_app_session_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "inspect_app_session",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn resolve_launch_restore(app: tauri::AppHandle) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => resolve_launch_restore_at_path(session_path),
        Err(error) => DesktopCommandResponse::error(
            "resolve_launch_restore",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn record_app_session_selection(
    app: tauri::AppHandle,
    library_path: String,
    selected_photo_id: Option<String>,
    mode: String,
) -> DesktopCommandResponse {
    match resolve_app_session_path(&app) {
        Ok(session_path) => record_app_session_selection_at_path(
            session_path,
            library_path,
            selected_photo_id,
            mode,
        ),
        Err(error) => DesktopCommandResponse::error(
            "record_app_session_selection",
            error,
            DesktopCommandContext::default(),
        ),
    }
}

#[tauri::command]
fn create_library(app: tauri::AppHandle, path: String) -> DesktopCommandResponse {
    let session_path = match resolve_app_session_path(&app) {
        Ok(session_path) => Some(session_path),
        Err(error) => {
            return DesktopCommandResponse::error(
                "create_library",
                error,
                DesktopCommandContext {
                    library_path: Some(path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    create_library_at_path(path, session_path)
}

fn create_library_at_path(path: String, session_path: Option<PathBuf>) -> DesktopCommandResponse {
    let command = "create_library";
    match silica_core::create_library(PathBuf::from(&path)) {
        Ok(session) => {
            if let Some(session_path) = session_path {
                if let Err(error) =
                    silica_core::record_app_session_recent_library(&session_path, &session)
                {
                    return DesktopCommandResponse::error(
                        command,
                        error,
                        DesktopCommandContext {
                            library_path: Some(path),
                            ..DesktopCommandContext::default()
                        },
                    );
                }
            }
            DesktopCommandResponse::ok(
                command,
                format!("Library created: {}", session.root_path.display()),
                library_session_data(session),
            )
        }
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn open_library(app: tauri::AppHandle, path: String) -> DesktopCommandResponse {
    let session_path = match resolve_app_session_path(&app) {
        Ok(session_path) => Some(session_path),
        Err(error) => {
            return DesktopCommandResponse::error(
                "open_library",
                error,
                DesktopCommandContext {
                    library_path: Some(path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    open_library_at_path(path, session_path)
}

fn open_library_at_path(path: String, session_path: Option<PathBuf>) -> DesktopCommandResponse {
    let command = "open_library";
    match silica_core::open_library(PathBuf::from(&path)) {
        Ok(session) => {
            if let Some(session_path) = session_path {
                if let Err(error) =
                    silica_core::record_app_session_recent_library(&session_path, &session)
                {
                    return DesktopCommandResponse::error(
                        command,
                        error,
                        DesktopCommandContext {
                            library_path: Some(path),
                            ..DesktopCommandContext::default()
                        },
                    );
                }
            }
            DesktopCommandResponse::ok(
                command,
                format!("Library opened: {}", session.root_path.display()),
                library_session_data(session),
            )
        }
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn import_folder(
    library_path: String,
    folder_path: String,
    recursive: Option<bool>,
) -> DesktopCommandResponse {
    let command = "import_folder";
    let options = silica_core::FolderImportOptions {
        recursive: recursive.unwrap_or(false),
    };
    match silica_core::import_folder_with_options(
        PathBuf::from(&library_path),
        PathBuf::from(&folder_path),
        options,
    ) {
        Ok(summary) => DesktopCommandResponse::ok(
            command,
            format!(
                "Imported {} supported file(s) by reference; originals unchanged.",
                summary.supported_files
            ),
            DesktopCommandData::ImportSummary {
                folder_path: summary.folder_path.display().to_string(),
                scanned_files: summary.scanned_files,
                supported_files: summary.supported_files,
                unsupported_files: summary.unsupported_files,
                issues: summary
                    .issues
                    .into_iter()
                    .map(desktop_import_issue)
                    .collect(),
                originals_unchanged: true,
            },
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                folder_path: Some(folder_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn list_library_photos(library_path: String) -> DesktopCommandResponse {
    let command = "list_library_photos";
    match silica_core::list_library_photos(PathBuf::from(&library_path)) {
        Ok(photos) => {
            let photos = photos.into_iter().map(DesktopPhotoGridItem::from).collect();
            DesktopCommandResponse::ok(
                command,
                "Library grid loaded.",
                DesktopCommandData::PhotoGrid { photos },
            )
        }
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn query_library_photos(
    library_path: String,
    request: DesktopLibraryQueryRequest,
) -> DesktopCommandResponse {
    let command = "query_library_photos";
    let query = match request.into_core() {
        Ok(query) => query,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            );
        }
    };

    match silica_core::query_library_photos_with_thumbnail_hydration(
        PathBuf::from(&library_path),
        query,
    ) {
        Ok(page) => DesktopCommandResponse::ok(
            command,
            "Library grid page loaded.",
            photo_grid_page_data(page),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn set_photo_flags(
    library_path: String,
    photo_id: String,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
) -> DesktopCommandResponse {
    let command = "set_photo_flags";
    match silica_core::set_photo_flags(
        PathBuf::from(&library_path),
        photo_id.clone(),
        rating,
        picked,
        rejected,
        color_label,
    ) {
        Ok(flags) => {
            DesktopCommandResponse::ok(command, "Photo flags updated.", photo_flags_data(flags))
        }
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_photo_flags(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_flags";
    match silica_core::get_photo_flags(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(flags)) => {
            DesktopCommandResponse::ok(command, "Photo flags loaded.", photo_flags_data(flags))
        }
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_photo_metadata(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_metadata";
    match silica_core::get_photo_metadata(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(metadata)) => DesktopCommandResponse::ok(
            command,
            "Photo metadata loaded.",
            photo_metadata_data(metadata),
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_ai_review_panel(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_ai_review_panel";
    match silica_core::get_ai_review_panel(PathBuf::from(&library_path), &photo_id) {
        Ok(panel) => {
            DesktopCommandResponse::ok(command, panel.message.clone(), ai_review_panel_data(panel))
        }
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn approve_ai_suggestion(
    library_path: String,
    photo_id: String,
    result_id: String,
) -> DesktopCommandResponse {
    let command = "approve_ai_suggestion";
    match silica_core::approve_ai_suggestion(PathBuf::from(&library_path), &photo_id, &result_id) {
        Ok(Some(approval)) => DesktopCommandResponse::ok(
            command,
            "AI suggestion approved as an undoable edit checkpoint.",
            ai_suggestion_approval_data(approval),
        ),
        Ok(None) => DesktopCommandResponse::error_message(
            command,
            "AI suggestion approval skipped because the selected photo is unavailable.",
            "aiReview",
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn reject_ai_suggestion(
    library_path: String,
    photo_id: String,
    result_id: String,
) -> DesktopCommandResponse {
    let command = "reject_ai_suggestion";
    match silica_core::reject_ai_suggestion(PathBuf::from(&library_path), &photo_id, &result_id) {
        Ok(Some(rejection)) => DesktopCommandResponse::ok(
            command,
            "AI suggestion rejected; edit state is unchanged.",
            ai_suggestion_rejection_data(rejection),
        ),
        Ok(None) => DesktopCommandResponse::error_message(
            command,
            "AI suggestion rejection skipped because the selected photo is unavailable.",
            "aiReview",
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn open_photo_preview(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "open_photo_preview";
    match silica_core::open_photo_preview(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::PhotoPreview {
                photo_id: preview.photo_id,
                file_name: preview.file_name,
                source_path: preview.source_path,
                preview_bytes: preview.preview_bytes,
                status: preview_status_text(preview.status),
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_exposure_contrast_edit(
    library_path: String,
    photo_id: String,
    exposure: f64,
    contrast: f64,
) -> DesktopCommandResponse {
    let command = "preview_exposure_contrast_edit";
    match silica_core::preview_exposure_contrast_edit(
        PathBuf::from(&library_path),
        &photo_id,
        exposure,
        contrast,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_white_balance_edit(
    library_path: String,
    photo_id: String,
    white_balance: String,
    temperature: f64,
    tint: f64,
) -> DesktopCommandResponse {
    let command = "preview_white_balance_edit";
    let white_balance_mode = match parse_white_balance(&white_balance) {
        Ok(mode) => mode,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    match silica_core::preview_white_balance_edit(
        PathBuf::from(&library_path),
        &photo_id,
        white_balance_mode,
        temperature,
        tint,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_tone_recovery_edit(
    library_path: String,
    photo_id: String,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> DesktopCommandResponse {
    let command = "preview_tone_recovery_edit";
    match silica_core::preview_tone_recovery_edit(
        PathBuf::from(&library_path),
        &photo_id,
        highlights,
        shadows,
        whites,
        blacks,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_color_presence_edit(
    library_path: String,
    photo_id: String,
    vibrance: f64,
    saturation: f64,
) -> DesktopCommandResponse {
    let command = "preview_color_presence_edit";
    match silica_core::preview_color_presence_edit(
        PathBuf::from(&library_path),
        &photo_id,
        vibrance,
        saturation,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_tone_curve_edit(
    library_path: String,
    photo_id: String,
    rgb_curve: Vec<DesktopToneCurvePoint>,
    red_curve: Vec<DesktopToneCurvePoint>,
    green_curve: Vec<DesktopToneCurvePoint>,
    blue_curve: Vec<DesktopToneCurvePoint>,
) -> DesktopCommandResponse {
    let command = "preview_tone_curve_edit";
    let rgb_curve = tone_curve_pairs(&rgb_curve);
    let red_curve = tone_curve_pairs(&red_curve);
    let green_curve = tone_curve_pairs(&green_curve);
    let blue_curve = tone_curve_pairs(&blue_curve);

    match silica_core::preview_tone_curve_edit(
        PathBuf::from(&library_path),
        &photo_id,
        &rgb_curve,
        &red_curve,
        &green_curve,
        &blue_curve,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_hsl_color_mixer_edit(
    library_path: String,
    photo_id: String,
    channel: String,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> DesktopCommandResponse {
    let command = "preview_hsl_color_mixer_edit";
    let hsl_channel = match parse_hsl_color_channel(&channel) {
        Ok(channel) => channel,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::preview_hsl_color_mixer_edit(
        PathBuf::from(&library_path),
        &photo_id,
        hsl_channel,
        hue,
        saturation,
        luminance,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_detail_sharpening_edit(
    library_path: String,
    photo_id: String,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> DesktopCommandResponse {
    let command = "preview_detail_sharpening_edit";
    match silica_core::preview_detail_sharpening_edit(
        PathBuf::from(&library_path),
        &photo_id,
        amount,
        radius,
        detail,
        masking,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn preview_detail_noise_reduction_edit(
    library_path: String,
    photo_id: String,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> DesktopCommandResponse {
    let command = "preview_detail_noise_reduction_edit";
    match silica_core::preview_detail_noise_reduction_edit(
        PathBuf::from(&library_path),
        &photo_id,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn preview_geometry_crop_edit(
    library_path: String,
    photo_id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<String>,
) -> DesktopCommandResponse {
    let command = "preview_geometry_crop_edit";
    match silica_core::preview_geometry_crop_edit(
        PathBuf::from(&library_path),
        &photo_id,
        x,
        y,
        width,
        height,
        angle,
        aspect.as_deref(),
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_clear_geometry_crop(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "preview_clear_geometry_crop";
    match silica_core::preview_clear_geometry_crop(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn preview_geometry_orientation_edit(
    library_path: String,
    photo_id: String,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> DesktopCommandResponse {
    let command = "preview_geometry_orientation_edit";
    match silica_core::preview_geometry_orientation_edit(
        PathBuf::from(&library_path),
        &photo_id,
        rotation,
        flip_horizontal,
        flip_vertical,
    ) {
        Ok(Some(preview)) => DesktopCommandResponse::ok(
            command,
            preview.message.clone(),
            DesktopCommandData::EditPreview {
                photo_id: preview.photo_id,
                source_path: preview.source_path,
                status: preview_status_text(preview.status),
                exposure: preview.exposure,
                contrast: preview.contrast,
                white_balance: white_balance_text(preview.white_balance),
                temperature: preview.temperature,
                tint: preview.tint,
                highlights: preview.highlights,
                shadows: preview.shadows,
                whites: preview.whites,
                blacks: preview.blacks,
                vibrance: preview.vibrance,
                saturation: preview.saturation,
                tone_curve: tone_curve_data(preview.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(preview.hsl_color_mixer),
                detail: detail_data(preview.detail),
                geometry: geometry_data(preview.geometry),
                masks: manual_mask_data(preview.masks),
                develop_preview_bytes: preview.develop_preview_bytes,
                message: preview.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_exposure_contrast_edit(
    library_path: String,
    photo_id: String,
    exposure: f64,
    contrast: f64,
) -> DesktopCommandResponse {
    let command = "commit_exposure_contrast_edit";
    match silica_core::commit_exposure_contrast_edit(
        PathBuf::from(&library_path),
        &photo_id,
        exposure,
        contrast,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_white_balance_edit(
    library_path: String,
    photo_id: String,
    white_balance: String,
    temperature: f64,
    tint: f64,
) -> DesktopCommandResponse {
    let command = "commit_white_balance_edit";
    let white_balance_mode = match parse_white_balance(&white_balance) {
        Ok(mode) => mode,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    match silica_core::commit_white_balance_edit(
        PathBuf::from(&library_path),
        &photo_id,
        white_balance_mode,
        temperature,
        tint,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_tone_recovery_edit(
    library_path: String,
    photo_id: String,
    highlights: f64,
    shadows: f64,
    whites: f64,
    blacks: f64,
) -> DesktopCommandResponse {
    let command = "commit_tone_recovery_edit";
    match silica_core::commit_tone_recovery_edit(
        PathBuf::from(&library_path),
        &photo_id,
        highlights,
        shadows,
        whites,
        blacks,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_color_presence_edit(
    library_path: String,
    photo_id: String,
    vibrance: f64,
    saturation: f64,
) -> DesktopCommandResponse {
    let command = "commit_color_presence_edit";
    match silica_core::commit_color_presence_edit(
        PathBuf::from(&library_path),
        &photo_id,
        vibrance,
        saturation,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_tone_curve_edit(
    library_path: String,
    photo_id: String,
    rgb_curve: Vec<DesktopToneCurvePoint>,
    red_curve: Vec<DesktopToneCurvePoint>,
    green_curve: Vec<DesktopToneCurvePoint>,
    blue_curve: Vec<DesktopToneCurvePoint>,
) -> DesktopCommandResponse {
    let command = "commit_tone_curve_edit";
    let rgb_curve = tone_curve_pairs(&rgb_curve);
    let red_curve = tone_curve_pairs(&red_curve);
    let green_curve = tone_curve_pairs(&green_curve);
    let blue_curve = tone_curve_pairs(&blue_curve);

    match silica_core::commit_tone_curve_edit(
        PathBuf::from(&library_path),
        &photo_id,
        &rgb_curve,
        &red_curve,
        &green_curve,
        &blue_curve,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_hsl_color_mixer_edit(
    library_path: String,
    photo_id: String,
    channel: String,
    hue: f64,
    saturation: f64,
    luminance: f64,
) -> DesktopCommandResponse {
    let command = "commit_hsl_color_mixer_edit";
    let hsl_channel = match parse_hsl_color_channel(&channel) {
        Ok(channel) => channel,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::commit_hsl_color_mixer_edit(
        PathBuf::from(&library_path),
        &photo_id,
        hsl_channel,
        hue,
        saturation,
        luminance,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_detail_sharpening_edit(
    library_path: String,
    photo_id: String,
    amount: f64,
    radius: f64,
    detail: f64,
    masking: f64,
) -> DesktopCommandResponse {
    let command = "commit_detail_sharpening_edit";
    match silica_core::commit_detail_sharpening_edit(
        PathBuf::from(&library_path),
        &photo_id,
        amount,
        radius,
        detail,
        masking,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn commit_detail_noise_reduction_edit(
    library_path: String,
    photo_id: String,
    luminance: f64,
    detail: f64,
    contrast: f64,
    color: f64,
    color_detail: f64,
) -> DesktopCommandResponse {
    let command = "commit_detail_noise_reduction_edit";
    match silica_core::commit_detail_noise_reduction_edit(
        PathBuf::from(&library_path),
        &photo_id,
        luminance,
        detail,
        contrast,
        color,
        color_detail,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn commit_geometry_crop_edit(
    library_path: String,
    photo_id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    angle: f64,
    aspect: Option<String>,
) -> DesktopCommandResponse {
    let command = "commit_geometry_crop_edit";
    match silica_core::commit_geometry_crop_edit(
        PathBuf::from(&library_path),
        &photo_id,
        x,
        y,
        width,
        height,
        angle,
        aspect.as_deref(),
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_clear_geometry_crop(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "commit_clear_geometry_crop";
    match silica_core::commit_clear_geometry_crop(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_geometry_orientation_edit(
    library_path: String,
    photo_id: String,
    rotation: f64,
    flip_horizontal: bool,
    flip_vertical: bool,
) -> DesktopCommandResponse {
    let command = "commit_geometry_orientation_edit";
    match silica_core::commit_geometry_orientation_edit(
        PathBuf::from(&library_path),
        &photo_id,
        rotation,
        flip_horizontal,
        flip_vertical,
    ) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_p0_basic_reset(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "commit_p0_basic_reset";
    match silica_core::commit_p0_basic_reset(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn commit_basic_preset_edit(
    library_path: String,
    photo_id: String,
    preset: String,
) -> DesktopCommandResponse {
    let command = "commit_basic_preset_edit";
    let preset = match parse_basic_preset(&preset) {
        Ok(preset) => preset,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    match silica_core::commit_basic_preset_edit(PathBuf::from(&library_path), &photo_id, preset) {
        Ok(Some(commit)) => DesktopCommandResponse::ok(
            command,
            commit.message.clone(),
            DesktopCommandData::EditCommit {
                photo_id: commit.photo_id,
                exposure: commit.exposure,
                contrast: commit.contrast,
                white_balance: white_balance_text(commit.white_balance),
                temperature: commit.temperature,
                tint: commit.tint,
                highlights: commit.highlights,
                shadows: commit.shadows,
                whites: commit.whites,
                blacks: commit.blacks,
                vibrance: commit.vibrance,
                saturation: commit.saturation,
                tone_curve: tone_curve_data(commit.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(commit.hsl_color_mixer),
                detail: detail_data(commit.detail),
                geometry: geometry_data(commit.geometry),
                masks: manual_mask_data(commit.masks),
                persisted: commit.persisted,
                message: commit.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_photo_edit_state(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_edit_state";
    match silica_core::get_photo_edit_state(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(state)) => DesktopCommandResponse::ok(
            command,
            state.message.clone(),
            DesktopCommandData::EditState {
                photo_id: state.photo_id,
                exposure: state.exposure,
                contrast: state.contrast,
                white_balance: white_balance_text(state.white_balance),
                temperature: state.temperature,
                tint: state.tint,
                highlights: state.highlights,
                shadows: state.shadows,
                whites: state.whites,
                blacks: state.blacks,
                vibrance: state.vibrance,
                saturation: state.saturation,
                tone_curve: tone_curve_data(state.tone_curve),
                hsl_color_mixer: hsl_color_mixer_data(state.hsl_color_mixer),
                detail: detail_data(state.detail),
                geometry: geometry_data(state.geometry),
                masks: manual_mask_data(state.masks),
                persisted: state.persisted,
                message: state.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn copy_edit_clipboard_payload(
    library_path: String,
    photo_id: String,
    selection: silica_core::EditClipboardSelection,
) -> DesktopCommandResponse {
    let command = "copy_edit_clipboard_payload";
    match silica_core::copy_photo_edit_clipboard_payload(
        PathBuf::from(&library_path),
        &photo_id,
        selection,
    ) {
        Ok(Some(payload)) => DesktopCommandResponse::ok(
            command,
            "Copied selected edit sections.",
            edit_clipboard_data(photo_id, selection, payload),
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn plan_edit_clipboard_sync(
    library_path: String,
    photo_ids: Vec<String>,
    payload: silica_core::EditClipboardPayload,
) -> DesktopCommandResponse {
    let command = "plan_edit_clipboard_sync";
    match silica_core::plan_edit_clipboard_sync(PathBuf::from(&library_path), &photo_ids, &payload)
    {
        Ok(plan) => DesktopCommandResponse::ok(
            command,
            plan.message.clone(),
            edit_clipboard_plan_data(plan),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: photo_ids.first().cloned(),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn apply_edit_clipboard_sync(
    library_path: String,
    photo_ids: Vec<String>,
    payload: silica_core::EditClipboardPayload,
) -> DesktopCommandResponse {
    let command = "apply_edit_clipboard_sync";
    match silica_core::apply_edit_clipboard_sync(PathBuf::from(&library_path), &photo_ids, &payload)
    {
        Ok(result) => DesktopCommandResponse::ok(
            command,
            result.message.clone(),
            edit_clipboard_sync_data(result),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: photo_ids.first().cloned(),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_photo_histogram(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_histogram";
    match silica_core::get_photo_histogram(PathBuf::from(&library_path), &photo_id) {
        Ok(Some(histogram)) => DesktopCommandResponse::ok(
            command,
            histogram.message.clone(),
            DesktopCommandData::Histogram {
                photo_id: histogram.photo_id,
                source_path: histogram.source_path,
                status: histogram_status_text(histogram.status),
                red: histogram.red,
                green: histogram.green,
                blue: histogram.blue,
                luminance: histogram.luminance,
                pixel_count: histogram.pixel_count,
                cache_key: histogram.cache_key,
                cache_path: histogram.cache_path,
                message: histogram.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn undo_last_history_action(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "undo_last_history_action";
    match silica_core::undo_last_history_action(PathBuf::from(&library_path), &photo_id) {
        Ok(result) => DesktopCommandResponse::ok(
            command,
            result.message.clone(),
            history_command_data(result),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn redo_last_history_action(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "redo_last_history_action";
    match silica_core::redo_last_history_action(PathBuf::from(&library_path), &photo_id) {
        Ok(result) => DesktopCommandResponse::ok(
            command,
            result.message.clone(),
            history_command_data(result),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_photo_history(library_path: String, photo_id: String) -> DesktopCommandResponse {
    let command = "get_photo_history";
    match silica_core::list_photo_history(PathBuf::from(&library_path), &photo_id) {
        Ok(panel) => DesktopCommandResponse::ok(
            command,
            panel.message.clone(),
            photo_history_panel_data(panel),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn export_photo_jpeg_srgb(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> DesktopCommandResponse {
    let command = "export_photo_jpeg_srgb";
    desktop_photo_export_response(
        command,
        silica_core::export_photo_jpeg_srgb(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
fn export_photo_jpeg(
    library_path: String,
    photo_id: String,
    output_path: String,
    color_profile: Option<String>,
    metadata_policy: Option<String>,
) -> DesktopCommandResponse {
    let command = "export_photo_jpeg";
    let requested_profile = match parse_export_color_profile(color_profile.as_deref()) {
        Ok(profile) => profile,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    output_path: Some(output_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };
    let requested_metadata_policy = match parse_export_metadata_policy(metadata_policy.as_deref()) {
        Ok(policy) => policy,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    output_path: Some(output_path),
                    photo_id: Some(photo_id),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    desktop_photo_export_response(
        command,
        silica_core::export_photo_jpeg_with_metadata_policy(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
            requested_profile,
            requested_metadata_policy,
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
fn export_photo_png(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> DesktopCommandResponse {
    let command = "export_photo_png";
    desktop_photo_export_response(
        command,
        silica_core::export_photo_png(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
fn export_photo_tiff(
    library_path: String,
    photo_id: String,
    output_path: String,
) -> DesktopCommandResponse {
    let command = "export_photo_tiff";
    desktop_photo_export_response(
        command,
        silica_core::export_photo_tiff(
            PathBuf::from(&library_path),
            &photo_id,
            PathBuf::from(&output_path),
        ),
        library_path,
        output_path,
        photo_id,
    )
}

#[tauri::command]
fn get_export_settings(library_path: String) -> DesktopCommandResponse {
    let command = "get_export_settings";
    match silica_core::get_export_settings_catalog(PathBuf::from(&library_path)) {
        Ok(catalog) => DesktopCommandResponse::ok(
            command,
            "Export settings loaded.",
            export_settings_catalog_data(catalog),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_recent_exports(library_path: String, limit: Option<usize>) -> DesktopCommandResponse {
    let command = "get_recent_exports";
    let limit = limit.unwrap_or(10).min(50);
    match silica_core::list_recent_exports(PathBuf::from(&library_path), limit) {
        Ok(exports) => DesktopCommandResponse::ok(
            command,
            "Recent exports loaded.",
            DesktopCommandData::RecentExports {
                exports: exports.into_iter().map(desktop_recent_export).collect(),
                message: "Recent exports loaded.".to_string(),
            },
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn save_export_settings(
    library_path: String,
    preset_id: Option<String>,
    format: Option<String>,
    color_profile: Option<String>,
    quality: Option<u8>,
    metadata_policy: Option<String>,
) -> DesktopCommandResponse {
    let command = "save_export_settings";
    let settings = match export_settings_from_request(
        format.as_deref(),
        color_profile.as_deref(),
        quality,
        metadata_policy.as_deref(),
    ) {
        Ok(settings) => settings,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::set_default_export_settings(
        PathBuf::from(&library_path),
        preset_id.as_deref(),
        settings,
    ) {
        Ok(catalog) => DesktopCommandResponse::ok(
            command,
            "Export settings saved.",
            export_settings_catalog_data(catalog),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn save_export_preset(
    library_path: String,
    name: String,
    format: Option<String>,
    color_profile: Option<String>,
    quality: Option<u8>,
    metadata_policy: Option<String>,
) -> DesktopCommandResponse {
    let command = "save_export_preset";
    let settings = match export_settings_from_request(
        format.as_deref(),
        color_profile.as_deref(),
        quality,
        metadata_policy.as_deref(),
    ) {
        Ok(settings) => settings,
        Err(error) => {
            return DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            )
        }
    };

    match silica_core::upsert_export_preset(PathBuf::from(&library_path), name, settings.clone()) {
        Ok(preset) => match silica_core::set_default_export_settings(
            PathBuf::from(&library_path),
            Some(&preset.id),
            settings,
        ) {
            Ok(catalog) => DesktopCommandResponse::ok(
                command,
                "Export preset saved.",
                export_settings_catalog_data(catalog),
            ),
            Err(error) => DesktopCommandResponse::error(
                command,
                error,
                DesktopCommandContext {
                    library_path: Some(library_path),
                    ..DesktopCommandContext::default()
                },
            ),
        },
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

fn parse_export_color_profile(
    color_profile: Option<&str>,
) -> Result<silica_core::PhotoExportColorProfile, silica_core::CoreError> {
    match color_profile.unwrap_or("srgb") {
        "srgb" => Ok(silica_core::PhotoExportColorProfile::Srgb),
        "display_p3" => Ok(silica_core::PhotoExportColorProfile::DisplayP3),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported export color profile: {unsupported}. Supported profiles: srgb, display_p3."
        ))),
    }
}

fn parse_export_format(format: Option<&str>) -> Result<&'static str, silica_core::CoreError> {
    match format.unwrap_or("jpeg") {
        "jpeg" => Ok("jpeg"),
        "png" => Ok("png"),
        "tiff" => Ok("tiff"),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported export format: {unsupported}. Supported formats: jpeg, png, tiff."
        ))),
    }
}

fn parse_export_metadata_policy(
    metadata_policy: Option<&str>,
) -> Result<silica_core::PhotoExportMetadataPolicy, silica_core::CoreError> {
    match metadata_policy.unwrap_or("minimal") {
        "minimal" => Ok(silica_core::PhotoExportMetadataPolicy::Minimal),
        "preserve" => Ok(silica_core::PhotoExportMetadataPolicy::Preserve),
        "remove_gps" => Ok(silica_core::PhotoExportMetadataPolicy::RemoveGps),
        "remove_all" => Ok(silica_core::PhotoExportMetadataPolicy::RemoveAll),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported export metadata policy: {unsupported}. Supported policies: minimal, preserve, remove_gps, remove_all."
        ))),
    }
}

fn export_metadata_policy_request_string(
    metadata_policy: silica_core::PhotoExportMetadataPolicy,
) -> &'static str {
    match metadata_policy {
        silica_core::PhotoExportMetadataPolicy::Minimal => "minimal",
        silica_core::PhotoExportMetadataPolicy::Preserve => "preserve",
        silica_core::PhotoExportMetadataPolicy::RemoveGps => "remove_gps",
        silica_core::PhotoExportMetadataPolicy::RemoveAll => "remove_all",
    }
}

fn export_settings_from_request(
    format: Option<&str>,
    color_profile: Option<&str>,
    quality: Option<u8>,
    metadata_policy: Option<&str>,
) -> Result<silica_core::ExportSettings, silica_core::CoreError> {
    let format = parse_export_format(format)?;
    let color_profile = parse_export_color_profile(color_profile)?;
    let metadata_policy = parse_export_metadata_policy(metadata_policy)?;
    if format != "jpeg" && color_profile != silica_core::PhotoExportColorProfile::Srgb {
        return Err(silica_core::CoreError::ExportBlocked(
            "PNG and TIFF export settings currently require sRGB color profile.".to_string(),
        ));
    }
    Ok(silica_core::ExportSettings {
        format: format.to_string(),
        color_profile: export_color_profile_request_string(color_profile).to_string(),
        quality: quality.unwrap_or(90),
        metadata_policy: export_metadata_policy_request_string(metadata_policy).to_string(),
    })
}

fn export_color_profile_request_string(
    color_profile: silica_core::PhotoExportColorProfile,
) -> &'static str {
    match color_profile {
        silica_core::PhotoExportColorProfile::Srgb => "srgb",
        silica_core::PhotoExportColorProfile::DisplayP3 => "display_p3",
    }
}

fn parse_white_balance(
    white_balance: &str,
) -> Result<silica_core::WhiteBalance, silica_core::CoreError> {
    match white_balance {
        "as_shot" => Ok(silica_core::WhiteBalance::AsShot),
        "auto" => Ok(silica_core::WhiteBalance::Auto),
        "daylight" => Ok(silica_core::WhiteBalance::Daylight),
        "cloudy" => Ok(silica_core::WhiteBalance::Cloudy),
        "shade" => Ok(silica_core::WhiteBalance::Shade),
        "tungsten" => Ok(silica_core::WhiteBalance::Tungsten),
        "fluorescent" => Ok(silica_core::WhiteBalance::Fluorescent),
        "flash" => Ok(silica_core::WhiteBalance::Flash),
        "custom" => Ok(silica_core::WhiteBalance::Custom),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported white balance mode: {unsupported}."
        ))),
    }
}

fn parse_basic_preset(preset: &str) -> Result<silica_core::BasicPreset, silica_core::CoreError> {
    match preset {
        "silica_neutral" => Ok(silica_core::BasicPreset::SilicaNeutral),
        "warm_contrast" => Ok(silica_core::BasicPreset::WarmContrast),
        "soft_matte" => Ok(silica_core::BasicPreset::SoftMatte),
        unsupported => Err(silica_core::CoreError::ExportBlocked(format!(
            "Unsupported basic preset: {unsupported}."
        ))),
    }
}

fn white_balance_text(white_balance: silica_core::WhiteBalance) -> &'static str {
    match white_balance {
        silica_core::WhiteBalance::AsShot => "as_shot",
        silica_core::WhiteBalance::Auto => "auto",
        silica_core::WhiteBalance::Daylight => "daylight",
        silica_core::WhiteBalance::Cloudy => "cloudy",
        silica_core::WhiteBalance::Shade => "shade",
        silica_core::WhiteBalance::Tungsten => "tungsten",
        silica_core::WhiteBalance::Fluorescent => "fluorescent",
        silica_core::WhiteBalance::Flash => "flash",
        silica_core::WhiteBalance::Custom => "custom",
    }
}

fn tone_curve_data(tone_curve: silica_core::PhotoToneCurveState) -> DesktopToneCurveState {
    DesktopToneCurveState {
        curve_mode: curve_mode_text(tone_curve.curve_mode),
        rgb_curve: tone_curve_points_data(tone_curve.rgb_curve),
        red_curve: tone_curve_points_data(tone_curve.red_curve),
        green_curve: tone_curve_points_data(tone_curve.green_curve),
        blue_curve: tone_curve_points_data(tone_curve.blue_curve),
    }
}

fn curve_mode_text(curve_mode: silica_core::CurveMode) -> &'static str {
    match curve_mode {
        silica_core::CurveMode::None => "none",
        silica_core::CurveMode::Parametric => "parametric",
        silica_core::CurveMode::Point => "point",
    }
}

fn tone_curve_points_data(
    points: Vec<silica_core::PhotoToneCurvePoint>,
) -> Vec<DesktopToneCurvePoint> {
    points
        .into_iter()
        .map(|point| DesktopToneCurvePoint {
            x: point.x,
            y: point.y,
        })
        .collect()
}

fn tone_curve_pairs(points: &[DesktopToneCurvePoint]) -> Vec<(f64, f64)> {
    points.iter().map(|point| (point.x, point.y)).collect()
}

fn hsl_color_mixer_data(
    hsl_color_mixer: silica_core::PhotoHslColorMixerState,
) -> DesktopHslColorMixerState {
    DesktopHslColorMixerState {
        red: hsl_color_channel_data(hsl_color_mixer.red),
        orange: hsl_color_channel_data(hsl_color_mixer.orange),
        yellow: hsl_color_channel_data(hsl_color_mixer.yellow),
        green: hsl_color_channel_data(hsl_color_mixer.green),
        aqua: hsl_color_channel_data(hsl_color_mixer.aqua),
        blue: hsl_color_channel_data(hsl_color_mixer.blue),
        purple: hsl_color_channel_data(hsl_color_mixer.purple),
        magenta: hsl_color_channel_data(hsl_color_mixer.magenta),
    }
}

fn hsl_color_channel_data(
    channel: silica_core::PhotoHslColorChannelState,
) -> DesktopHslColorChannelState {
    DesktopHslColorChannelState {
        hue: channel.hue,
        saturation: channel.saturation,
        luminance: channel.luminance,
    }
}

fn detail_data(detail: silica_core::PhotoDetailState) -> DesktopDetailState {
    DesktopDetailState {
        sharpening: DesktopDetailSharpeningState {
            amount: detail.sharpening.amount,
            radius: detail.sharpening.radius,
            detail: detail.sharpening.detail,
            masking: detail.sharpening.masking,
        },
        noise_reduction: DesktopDetailNoiseReductionState {
            luminance: detail.noise_reduction.luminance,
            detail: detail.noise_reduction.detail,
            contrast: detail.noise_reduction.contrast,
            color: detail.noise_reduction.color,
            color_detail: detail.noise_reduction.color_detail,
        },
    }
}

fn geometry_data(geometry: silica_core::PhotoGeometryState) -> DesktopGeometryState {
    DesktopGeometryState {
        crop: geometry.crop.map(|crop| DesktopGeometryCropState {
            x: crop.x,
            y: crop.y,
            width: crop.width,
            height: crop.height,
            angle: crop.angle,
            aspect: crop.aspect,
        }),
        rotation: geometry.rotation,
        flip_horizontal: geometry.flip_horizontal,
        flip_vertical: geometry.flip_vertical,
        transform: DesktopGeometryTransformState {
            vertical: geometry.transform.vertical,
            horizontal: geometry.transform.horizontal,
            aspect: geometry.transform.aspect,
            scale: geometry.transform.scale,
            x_offset: geometry.transform.x_offset,
            y_offset: geometry.transform.y_offset,
        },
    }
}

fn manual_mask_geometry_data(
    geometry: Option<silica_core::PhotoManualMaskGeometryState>,
) -> Option<DesktopManualMaskGeometryState> {
    geometry.map(|geometry| match geometry {
        silica_core::PhotoManualMaskGeometryState::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        } => DesktopManualMaskGeometryState::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
        },
        silica_core::PhotoManualMaskGeometryState::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        } => DesktopManualMaskGeometryState::RadialGradient {
            center_x,
            center_y,
            radius_x,
            radius_y,
            rotation,
        },
    })
}

fn manual_mask_data(masks: Vec<silica_core::PhotoManualMaskState>) -> Vec<DesktopManualMaskState> {
    masks
        .into_iter()
        .map(|mask| DesktopManualMaskState {
            id: mask.id,
            kind: mask.kind,
            name: mask.name,
            enabled: mask.enabled,
            invert: mask.invert,
            opacity: mask.opacity,
            feather: mask.feather,
            geometry: manual_mask_geometry_data(mask.geometry),
            exposure: mask.exposure,
            contrast: mask.contrast,
        })
        .collect()
}

fn parse_hsl_color_channel(
    channel: &str,
) -> Result<silica_core::HslColorChannel, silica_core::CoreError> {
    silica_core::HslColorChannel::try_from(channel).map_err(silica_core::CoreError::from)
}

fn export_settings_catalog_data(catalog: silica_core::ExportSettingsCatalog) -> DesktopCommandData {
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

fn desktop_export_preset(preset: silica_core::ExportPreset) -> DesktopExportPreset {
    DesktopExportPreset {
        id: preset.id,
        name: preset.name,
        settings: desktop_export_settings(preset.settings),
    }
}

fn desktop_export_settings(settings: silica_core::ExportSettings) -> DesktopExportSettings {
    DesktopExportSettings {
        format: settings.format,
        color_profile: settings.color_profile,
        quality: settings.quality,
        metadata_policy: settings.metadata_policy,
    }
}

fn desktop_recent_export(export: silica_core::PhotoRecentExport) -> DesktopRecentExport {
    DesktopRecentExport {
        export_record_id: export.export_record_id,
        photo_id: export.photo_id,
        output_path: export.output_path,
        export_settings_json: export.export_settings_json,
        created_at: export.created_at,
        output_exists: export.output_exists,
    }
}

fn desktop_photo_export_response(
    command: &'static str,
    export_result: Result<Option<silica_core::PhotoExportSession>, silica_core::CoreError>,
    library_path: String,
    output_path: String,
    photo_id: String,
) -> DesktopCommandResponse {
    match export_result {
        Ok(Some(export)) => DesktopCommandResponse::ok(
            command,
            export.message.clone(),
            DesktopCommandData::Export {
                photo_id: export.photo_id,
                source_path: export.source_path,
                output_path: export.output_path.display().to_string(),
                format: export.format,
                color_profile: export.color_profile,
                bytes_written: export.bytes_written,
                source_sha256: export.source_sha256,
                output_sha256: export.output_sha256,
                icc_profile_embedded: export.icc_profile_embedded,
                icc_profile_sha256: export.icc_profile_sha256,
                decoder_backend: export.decoder_backend,
                input_profile: export.input_profile,
                working_space: export.working_space,
                export_record_id: export.export_record_id,
                message: export.message,
            },
        ),
        Ok(None) => DesktopCommandResponse::empty(command, "Catalog photo was not found."),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                output_path: Some(output_path),
                photo_id: Some(photo_id),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn get_library_cache_status(library_path: String) -> DesktopCommandResponse {
    let command = "get_library_cache_status";
    match silica_core::get_library_cache_status(PathBuf::from(&library_path)) {
        Ok(status) => DesktopCommandResponse::ok(
            command,
            status.message.clone(),
            DesktopCommandData::CacheStatus {
                library_root_path: status.library_root_path.display().to_string(),
                directories: status
                    .directories
                    .into_iter()
                    .map(DesktopCacheDirectoryStatus::from)
                    .collect(),
                total_bytes: status.total_bytes,
                cache_record_count: status.cache_record_count,
                message: status.message,
            },
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

#[tauri::command]
fn clear_library_cache(library_path: String) -> DesktopCommandResponse {
    let command = "clear_library_cache";
    match silica_core::clear_library_cache(PathBuf::from(&library_path)) {
        Ok(summary) => DesktopCommandResponse::ok(
            command,
            summary.message.clone(),
            DesktopCommandData::CacheClear {
                cleared_directories: summary.cleared_directories,
                recreated_directories: summary.recreated_directories,
                removed_cache_records: summary.removed_cache_records,
                message: summary.message,
            },
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

fn library_session_data(session: silica_core::LibrarySession) -> DesktopCommandData {
    DesktopCommandData::LibrarySession {
        root_path: session.root_path.display().to_string(),
        catalog_path: session.catalog_path.display().to_string(),
        schema_version: session.schema_version,
    }
}

fn resolve_app_session_path(app: &tauri::AppHandle) -> Result<PathBuf, silica_core::CoreError> {
    app.path()
        .resolve("app-session.json", BaseDirectory::AppConfig)
        .map_err(|error| {
            silica_core::CoreError::AppSession(format!("resolve app session path: {error}"))
        })
}

fn read_app_session_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "read_app_session";
    match silica_core::load_app_session(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session loaded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn write_app_session_at_path(
    session_path: PathBuf,
    session: DesktopAppSession,
) -> DesktopCommandResponse {
    let command = "write_app_session";
    let session = match session.into_core() {
        Ok(session) => session,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };

    match silica_core::write_app_session(&session_path, &session) {
        Ok(written) => DesktopCommandResponse::ok(
            command,
            "App session written.",
            DesktopCommandData::AppSessionWrite {
                session_path: written.session_path.display().to_string(),
                bytes_written: written.bytes_written,
            },
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn reset_app_session_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "reset_app_session";
    let session = silica_core::AppSession::default();
    match silica_core::write_app_session(&session_path, &session) {
        Ok(_) => DesktopCommandResponse::ok(
            command,
            "App session reset.",
            DesktopCommandData::AppSession {
                session_path: session_path.display().to_string(),
                session: DesktopAppSession::from_core(session),
                warnings: Vec::new(),
            },
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn record_app_session_layout_at_path(
    session_path: PathBuf,
    layout: DesktopLayoutPreferences,
) -> DesktopCommandResponse {
    let command = "record_app_session_layout";
    let layout = match layout.into_core() {
        Ok(layout) => layout,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };

    match silica_core::record_app_session_layout(&session_path, layout) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session layout recorded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn reset_app_session_layout_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "reset_app_session_layout";
    match silica_core::reset_app_session_layout(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session layout reset.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn record_app_session_appearance_at_path(
    session_path: PathBuf,
    appearance: DesktopAppearancePreferences,
) -> DesktopCommandResponse {
    let command = "record_app_session_appearance";
    let appearance = match appearance.into_core() {
        Ok(appearance) => appearance,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };

    match silica_core::record_app_session_appearance(&session_path, appearance) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session appearance recorded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn reset_app_session_appearance_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "reset_app_session_appearance";
    match silica_core::reset_app_session_appearance(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session appearance reset.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn record_app_session_library_preferences_at_path(
    session_path: PathBuf,
    library: DesktopLibraryPreferences,
) -> DesktopCommandResponse {
    let command = "record_app_session_library_preferences";
    match silica_core::record_app_session_library_preferences(&session_path, library.into_core()) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session library preferences recorded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn reset_app_session_library_preferences_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "reset_app_session_library_preferences";
    match silica_core::reset_app_session_library_preferences(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session library preferences reset.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn inspect_app_session_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "inspect_app_session";
    let exists = session_path.is_file();
    match silica_core::load_app_session(&session_path) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session inspected.",
            DesktopCommandData::AppSessionInspection {
                session_path: session_path.display().to_string(),
                exists,
                warnings: app_session_warning_strings(&loaded.warnings),
            },
        ),
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn resolve_launch_restore_at_path(session_path: PathBuf) -> DesktopCommandResponse {
    let command = "resolve_launch_restore";
    match silica_core::plan_app_session_restore(&session_path) {
        Ok(plan) => {
            let status = app_session_restore_status_string(plan.status).to_string();
            let state = if plan.status == silica_core::AppSessionRestoreStatus::Restored {
                "library".to_string()
            } else {
                "welcome".to_string()
            };
            let fallback_reason = if state == "welcome" {
                Some(status.clone())
            } else {
                None
            };
            DesktopCommandResponse::ok(
                command,
                "Launch restore resolved.",
                DesktopCommandData::LaunchRestore {
                    session_path: session_path.display().to_string(),
                    session: DesktopAppSession::from_core(plan.session),
                    warnings: app_session_warning_strings(&plan.warnings),
                    status,
                    state,
                    fallback_reason,
                    requested_mode: app_session_mode_string(plan.requested_mode).to_string(),
                    resolved_mode: app_session_mode_string(plan.resolved_mode).to_string(),
                    selected_photo_id: plan.selected_photo_id,
                    selected_photo_status: app_session_selected_photo_status_string(
                        plan.selected_photo_status,
                    )
                    .to_string(),
                    library_root_path: plan
                        .library_root_path
                        .map(|path| path.display().to_string()),
                    catalog_path: plan.catalog_path.map(|path| path.display().to_string()),
                    schema_version: plan.schema_version,
                },
            )
        }
        Err(error) => {
            DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    }
}

fn record_app_session_selection_at_path(
    session_path: PathBuf,
    library_path: String,
    selected_photo_id: Option<String>,
    mode: String,
) -> DesktopCommandResponse {
    let command = "record_app_session_selection";
    let mode = match parse_desktop_app_session_mode(&mode) {
        Ok(mode) => mode,
        Err(error) => {
            return DesktopCommandResponse::error(command, error, DesktopCommandContext::default())
        }
    };
    let selected_photo_id =
        selected_photo_id.and_then(|photo_id| (!photo_id.trim().is_empty()).then_some(photo_id));

    match silica_core::record_app_session_library_state(
        &session_path,
        PathBuf::from(&library_path),
        selected_photo_id,
        mode,
    ) {
        Ok(loaded) => DesktopCommandResponse::ok(
            command,
            "App session selection recorded.",
            app_session_data(session_path, loaded),
        ),
        Err(error) => DesktopCommandResponse::error(
            command,
            error,
            DesktopCommandContext {
                library_path: Some(library_path),
                ..DesktopCommandContext::default()
            },
        ),
    }
}

fn app_session_data(
    session_path: PathBuf,
    loaded: silica_core::AppSessionLoadResult,
) -> DesktopCommandData {
    DesktopCommandData::AppSession {
        session_path: session_path.display().to_string(),
        session: DesktopAppSession::from_core(loaded.session),
        warnings: app_session_warning_strings(&loaded.warnings),
    }
}

fn photo_flags_data(flags: silica_core::PhotoFlags) -> DesktopCommandData {
    DesktopCommandData::PhotoFlags {
        photo_id: flags.photo_id,
        rating: flags.rating,
        picked: flags.picked,
        rejected: flags.rejected,
        color_label: flags.color_label,
    }
}

fn edit_clipboard_section_count(selection: &silica_core::EditClipboardSelection) -> usize {
    [
        selection.basic,
        selection.tone,
        selection.color,
        selection.detail,
        selection.lens,
        selection.geometry,
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count()
}

fn edit_clipboard_data(
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

fn edit_clipboard_target_data(
    target: silica_core::BatchEditClipboardSyncTarget,
) -> DesktopEditClipboardTarget {
    DesktopEditClipboardTarget {
        photo_id: target.photo_id,
        status: target.status,
        code: target.code,
        message: target.message,
    }
}

fn edit_clipboard_plan_data(plan: silica_core::BatchEditClipboardSyncPlan) -> DesktopCommandData {
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

fn edit_clipboard_sync_data(
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

fn history_command_data(result: silica_core::HistoryCommandResult) -> DesktopCommandData {
    DesktopCommandData::HistoryCommand {
        photo_id: result.photo_id,
        command: result.command,
        applied: result.applied,
        action_kind: result.action_kind,
        history_id: result.history_id,
        message: result.message,
    }
}

fn photo_history_panel_data(panel: silica_core::PhotoHistoryPanel) -> DesktopCommandData {
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

fn ai_review_panel_data(panel: silica_core::AiReviewPanel) -> DesktopCommandData {
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

fn ai_suggestion_approval_data(approval: silica_core::AiSuggestionApproval) -> DesktopCommandData {
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

fn ai_suggestion_rejection_data(
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

fn ai_review_status_text(status: silica_core::AiReviewPanelStatus) -> &'static str {
    match status {
        silica_core::AiReviewPanelStatus::ModelUnavailable => "modelUnavailable",
        silica_core::AiReviewPanelStatus::ReviewAvailable => "reviewAvailable",
    }
}

fn photo_metadata_data(metadata: silica_core::PhotoMetadata) -> DesktopCommandData {
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

fn desktop_import_issue(issue: silica_core::ImportIssue) -> DesktopImportIssue {
    DesktopImportIssue {
        kind: issue.kind.as_str(),
        path: issue.path,
        file_name: issue.file_name,
        message: issue.message,
    }
}

fn metadata_field<T>(field: silica_core::PhotoMetadataField<T>) -> DesktopMetadataField<T> {
    DesktopMetadataField {
        state: metadata_field_state_string(field.state),
        value: field.value,
    }
}

fn metadata_field_state_string(state: silica_core::PhotoMetadataFieldState) -> &'static str {
    match state {
        silica_core::PhotoMetadataFieldState::Known => "known",
        silica_core::PhotoMetadataFieldState::Unknown => "unknown",
        silica_core::PhotoMetadataFieldState::Unavailable => "unavailable",
    }
}

fn photo_grid_page_data(
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

fn app_session_warning_strings(warnings: &[silica_core::AppSessionWarning]) -> Vec<String> {
    warnings
        .iter()
        .map(|warning| match warning {
            silica_core::AppSessionWarning::Missing => "missing",
            silica_core::AppSessionWarning::Corrupt => "corrupt",
            silica_core::AppSessionWarning::UnsupportedVersion => "unsupportedVersion",
            silica_core::AppSessionWarning::InvalidValues => "invalidValues",
        })
        .map(str::to_string)
        .collect()
}

fn app_session_restore_status_string(status: silica_core::AppSessionRestoreStatus) -> &'static str {
    match status {
        silica_core::AppSessionRestoreStatus::NoLastLibrary => "noLastLibrary",
        silica_core::AppSessionRestoreStatus::MissingLibrary => "missingLibrary",
        silica_core::AppSessionRestoreStatus::MissingCatalog => "missingCatalog",
        silica_core::AppSessionRestoreStatus::InvalidCatalog => "invalidCatalog",
        silica_core::AppSessionRestoreStatus::Restored => "restored",
    }
}

fn app_session_selected_photo_status_string(
    status: silica_core::AppSessionSelectedPhotoStatus,
) -> &'static str {
    match status {
        silica_core::AppSessionSelectedPhotoStatus::None => "none",
        silica_core::AppSessionSelectedPhotoStatus::Missing => "missing",
        silica_core::AppSessionSelectedPhotoStatus::Restored => "restored",
    }
}

fn parse_desktop_app_session_mode(
    mode: &str,
) -> Result<silica_core::AppSessionMode, silica_core::CoreError> {
    match mode {
        "library" => Ok(silica_core::AppSessionMode::Library),
        "develop" => Ok(silica_core::AppSessionMode::Develop),
        "export" => Ok(silica_core::AppSessionMode::Export),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app session mode: {other}"
        ))),
    }
}

fn app_session_mode_string(mode: silica_core::AppSessionMode) -> &'static str {
    match mode {
        silica_core::AppSessionMode::Library => "library",
        silica_core::AppSessionMode::Develop => "develop",
        silica_core::AppSessionMode::Export => "export",
    }
}

fn parse_desktop_app_library_sort(
    sort: &str,
) -> Result<silica_core::AppLibrarySort, silica_core::CoreError> {
    match sort {
        "imported_at_desc" => Ok(silica_core::AppLibrarySort::ImportedAtDesc),
        "file_name_asc" => Ok(silica_core::AppLibrarySort::FileNameAsc),
        "rating_desc" => Ok(silica_core::AppLibrarySort::RatingDesc),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app library sort: {other}"
        ))),
    }
}

fn app_library_sort_string(sort: silica_core::AppLibrarySort) -> &'static str {
    match sort {
        silica_core::AppLibrarySort::ImportedAtDesc => "imported_at_desc",
        silica_core::AppLibrarySort::FileNameAsc => "file_name_asc",
        silica_core::AppLibrarySort::RatingDesc => "rating_desc",
    }
}

fn parse_desktop_app_file_type_filter(
    file_type: &str,
) -> Result<silica_core::AppFileTypeFilter, silica_core::CoreError> {
    match file_type {
        "jpeg" => Ok(silica_core::AppFileTypeFilter::Jpeg),
        "raw" => Ok(silica_core::AppFileTypeFilter::Raw),
        "unsupported" => Ok(silica_core::AppFileTypeFilter::Unsupported),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app file type filter: {other}"
        ))),
    }
}

fn app_file_type_filter_string(filter: silica_core::AppFileTypeFilter) -> &'static str {
    match filter {
        silica_core::AppFileTypeFilter::Jpeg => "jpeg",
        silica_core::AppFileTypeFilter::Raw => "raw",
        silica_core::AppFileTypeFilter::Unsupported => "unsupported",
    }
}

fn parse_desktop_app_metadata_filter(
    metadata: &str,
) -> Result<silica_core::AppMetadataFilter, silica_core::CoreError> {
    match metadata {
        "has_dimensions" => Ok(silica_core::AppMetadataFilter::HasDimensions),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app metadata filter: {other}"
        ))),
    }
}

fn app_metadata_filter_string(filter: silica_core::AppMetadataFilter) -> &'static str {
    match filter {
        silica_core::AppMetadataFilter::HasDimensions => "has_dimensions",
    }
}

fn parse_desktop_app_appearance_theme(
    theme: &str,
) -> Result<silica_core::AppAppearanceTheme, silica_core::CoreError> {
    match theme {
        "dark" => Ok(silica_core::AppAppearanceTheme::Dark),
        "light" => Ok(silica_core::AppAppearanceTheme::Light),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app appearance theme: {other}"
        ))),
    }
}

fn app_appearance_theme_string(theme: silica_core::AppAppearanceTheme) -> &'static str {
    match theme {
        silica_core::AppAppearanceTheme::Dark => "dark",
        silica_core::AppAppearanceTheme::Light => "light",
    }
}

fn parse_desktop_app_appearance_density(
    density: &str,
) -> Result<silica_core::AppAppearanceDensity, silica_core::CoreError> {
    match density {
        "compact" => Ok(silica_core::AppAppearanceDensity::Compact),
        "comfortable" => Ok(silica_core::AppAppearanceDensity::Comfortable),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app appearance density: {other}"
        ))),
    }
}

fn app_appearance_density_string(density: silica_core::AppAppearanceDensity) -> &'static str {
    match density {
        silica_core::AppAppearanceDensity::Compact => "compact",
        silica_core::AppAppearanceDensity::Comfortable => "comfortable",
    }
}

fn parse_desktop_library_query_sort(
    sort: &str,
) -> Result<silica_core::LibraryQuerySort, silica_core::CoreError> {
    match sort {
        "imported_at_desc" => Ok(silica_core::LibraryQuerySort::ImportedAtDesc),
        "file_name_asc" => Ok(silica_core::LibraryQuerySort::FileNameAsc),
        "rating_desc" => Ok(silica_core::LibraryQuerySort::RatingDesc),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid library query sort: {other}"
        ))),
    }
}

fn parse_desktop_library_query_file_type(
    file_type: &str,
) -> Result<silica_core::LibraryQueryFileType, silica_core::CoreError> {
    match file_type {
        "jpeg" => Ok(silica_core::LibraryQueryFileType::Jpeg),
        "raw" => Ok(silica_core::LibraryQueryFileType::Raw),
        "unsupported" => Ok(silica_core::LibraryQueryFileType::Unsupported),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid library query file type: {other}"
        ))),
    }
}

fn parse_desktop_library_query_metadata(
    metadata: &str,
) -> Result<silica_core::LibraryQueryMetadataFilter, silica_core::CoreError> {
    match metadata {
        "has_dimensions" => Ok(silica_core::LibraryQueryMetadataFilter::HasDimensions),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid library query metadata filter: {other}"
        ))),
    }
}

fn library_query_order_field_string(field: silica_core::LibraryQueryOrderField) -> &'static str {
    match field {
        silica_core::LibraryQueryOrderField::ImportedAtDesc => "imported_at_desc",
        silica_core::LibraryQueryOrderField::FileNameAsc => "file_name_asc",
        silica_core::LibraryQueryOrderField::RatingDesc => "rating_desc",
        silica_core::LibraryQueryOrderField::PhotoIdAsc => "photo_id_asc",
        silica_core::LibraryQueryOrderField::PathAsc => "path_asc",
    }
}

fn preview_status_text(status: silica_core::PhotoPreviewStatus) -> &'static str {
    match status {
        silica_core::PhotoPreviewStatus::Ready => "Ready",
        silica_core::PhotoPreviewStatus::BlockedByDecode => "BlockedByDecode",
        silica_core::PhotoPreviewStatus::Unsupported => "Unsupported",
    }
}

fn histogram_status_text(status: silica_core::PhotoHistogramStatus) -> &'static str {
    match status {
        silica_core::PhotoHistogramStatus::Ready => "Ready",
        silica_core::PhotoHistogramStatus::BlockedByDecode => "BlockedByDecode",
        silica_core::PhotoHistogramStatus::Unsupported => "Unsupported",
        silica_core::PhotoHistogramStatus::Missing => "Missing",
    }
}

fn core_error_kind(error: &silica_core::CoreError) -> &'static str {
    match error {
        silica_core::CoreError::Storage(_) => "storage",
        silica_core::CoreError::Decode(_) => "decode",
        silica_core::CoreError::RawExport(_) => "decode",
        silica_core::CoreError::EditGraph(_) => "editGraph",
        silica_core::CoreError::EditClipboard(_) => "editClipboard",
        silica_core::CoreError::UnsupportedEdit(_) => "unsupportedEdit",
        silica_core::CoreError::Export(_) => "export",
        silica_core::CoreError::ExportBlocked(_) => "exportBlocked",
        silica_core::CoreError::AppSession(_) => "appSession",
        silica_core::CoreError::AiReview(_) => "aiReview",
        silica_core::CoreError::Plugin(_) => "plugin",
    }
}

fn main() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());

    #[cfg(all(target_os = "macos", feature = "native-metal-viewer"))]
    {
        let _native_viewer_contract = native_metal_viewer::module_contract();
        if std::env::var_os("SILICA_NATIVE_VIEWER_LIFECYCLE_PROOF").is_some() {
            match native_metal_viewer::lifecycle_smoke_evidence() {
                Ok(evidence) => eprintln!("[SilicaRAW Native Viewer] {evidence}"),
                Err(error) => {
                    eprintln!("[SilicaRAW Native Viewer] lifecycle proof unavailable: {error}")
                }
            }
        }
        if std::env::var_os("SILICA_NATIVE_VIEWER_INPUT_PROOF").is_some() {
            match native_metal_viewer::input_smoke_evidence() {
                Ok(evidence) => eprintln!("[SilicaRAW Native Viewer] {evidence}"),
                Err(error) => {
                    eprintln!("[SilicaRAW Native Viewer] input proof unavailable: {error}")
                }
            }
        }
        if std::env::var_os("SILICA_NATIVE_VIEWER_RENDER_REQUEST_PROOF").is_some() {
            let evidence = native_metal_viewer::render_request_smoke_evidence();
            eprintln!("[SilicaRAW Native Viewer] {evidence}");
        }
        if std::env::var_os("SILICA_NATIVE_VIEWER_TEXTURE_LIFECYCLE_PROOF").is_some() {
            let evidence = native_metal_viewer::texture_lifecycle_smoke_evidence();
            eprintln!("[SilicaRAW Native Viewer] {evidence}");
        }
    }

    #[cfg(all(target_os = "macos", feature = "metal-host-spike"))]
    let builder = builder.setup(metal_host_spike::install);

    builder
        .invoke_handler(tauri::generate_handler![
            read_app_session,
            write_app_session,
            reset_app_session,
            record_app_session_layout,
            reset_app_session_layout,
            record_app_session_appearance,
            reset_app_session_appearance,
            record_app_session_library_preferences,
            reset_app_session_library_preferences,
            inspect_app_session,
            resolve_launch_restore,
            record_app_session_selection,
            create_library,
            open_library,
            import_folder,
            list_library_photos,
            query_library_photos,
            set_photo_flags,
            get_photo_flags,
            get_photo_metadata,
            get_ai_review_panel,
            approve_ai_suggestion,
            reject_ai_suggestion,
            open_photo_preview,
            preview_exposure_contrast_edit,
            preview_white_balance_edit,
            preview_tone_recovery_edit,
            preview_color_presence_edit,
            preview_tone_curve_edit,
            preview_hsl_color_mixer_edit,
            preview_detail_sharpening_edit,
            preview_detail_noise_reduction_edit,
            preview_geometry_crop_edit,
            preview_clear_geometry_crop,
            preview_geometry_orientation_edit,
            commit_exposure_contrast_edit,
            commit_white_balance_edit,
            commit_tone_recovery_edit,
            commit_color_presence_edit,
            commit_tone_curve_edit,
            commit_hsl_color_mixer_edit,
            commit_detail_sharpening_edit,
            commit_detail_noise_reduction_edit,
            commit_geometry_crop_edit,
            commit_clear_geometry_crop,
            commit_geometry_orientation_edit,
            commit_p0_basic_reset,
            commit_basic_preset_edit,
            get_photo_edit_state,
            copy_edit_clipboard_payload,
            plan_edit_clipboard_sync,
            apply_edit_clipboard_sync,
            get_photo_histogram,
            undo_last_history_action,
            redo_last_history_action,
            get_photo_history,
            export_photo_jpeg_srgb,
            export_photo_jpeg,
            export_photo_png,
            export_photo_tiff,
            get_export_settings,
            get_recent_exports,
            save_export_settings,
            save_export_preset,
            get_library_cache_status,
            clear_library_cache
        ])
        .run(tauri::generate_context!())
        .expect("failed to run SilicaRAW desktop shell");
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn desktop_core_error_kind_maps_decode_errors() {
        let error =
            silica_core::CoreError::Decode(silica_core::RawPreviewArtifactError::InvalidRequest(
                "invalid RAW preview request".to_string(),
            ));

        assert_eq!(super::core_error_kind(&error), "decode");
        assert_eq!(
            super::core_error_kind(&silica_core::CoreError::Plugin(
                "invalid plugin".to_string()
            )),
            "plugin"
        );
    }

    #[cfg(all(target_os = "macos", feature = "native-metal-viewer"))]
    #[test]
    fn product_native_viewer_contract_is_separate_from_spike() {
        let contract = super::native_metal_viewer::module_contract();

        assert_eq!(contract.module_name, "native_metal_viewer");
        assert_eq!(contract.feature_name, "native-metal-viewer");
        assert_eq!(contract.phase_task, "14.2");
        assert!(contract.product_module);
        assert!(!contract.uses_spike_module);
        assert!(!contract.installs_in_default_build);
        assert_eq!(contract.reserved_surfaces, ["loupe", "develop"]);
        assert!(contract.consumes_web_host_geometry);
        assert!(contract.controls_must_be_external);
    }

    #[test]
    fn desktop_app_session_commands_round_trip_temp_path() {
        let workspace = unique_library_root("desktop-app-session");
        let session_path = workspace.join("AppConfig").join("app-session.json");
        let library_root = workspace.join("SilicaRAW Library");

        let missing = super::read_app_session_at_path(session_path.clone());
        assert!(missing.ok);
        match response_data(&missing) {
            super::DesktopCommandData::AppSession {
                session_path: returned_path,
                session,
                warnings,
            } => {
                assert_eq!(returned_path, &session_path.display().to_string());
                assert_eq!(session.last_mode, "library");
                assert_eq!(session.layout.thumbnail_size, 168);
                assert_eq!(warnings, &vec!["missing".to_string()]);
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(!session_path.exists());

        let mut expected_session = super::DesktopAppSession::default();
        expected_session.last_library_root_path = Some(library_root.display().to_string());
        expected_session.last_mode = "develop".to_string();
        expected_session.recents.push(super::DesktopRecentLibrary {
            root_path: library_root.display().to_string(),
            display_name: "SilicaRAW Library".to_string(),
            last_opened_at: "unix:42".to_string(),
            available: None,
        });
        expected_session.per_library.insert(
            library_root.display().to_string(),
            super::DesktopPerLibrarySession {
                selected_photo_id: Some("photo-1".to_string()),
                last_mode: "develop".to_string(),
                last_opened_at: "unix:42".to_string(),
            },
        );

        let written =
            super::write_app_session_at_path(session_path.clone(), expected_session.clone());
        assert!(written.ok);
        match response_data(&written) {
            super::DesktopCommandData::AppSessionWrite {
                session_path: returned_path,
                bytes_written,
            } => {
                assert_eq!(returned_path, &session_path.display().to_string());
                assert!(*bytes_written > 0);
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(session_path.is_file());

        let inspected = super::inspect_app_session_at_path(session_path.clone());
        assert!(inspected.ok);
        match response_data(&inspected) {
            super::DesktopCommandData::AppSessionInspection {
                session_path: returned_path,
                exists,
                warnings,
            } => {
                assert_eq!(returned_path, &session_path.display().to_string());
                assert!(*exists);
                assert!(warnings.is_empty());
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let loaded = super::read_app_session_at_path(session_path.clone());
        assert!(loaded.ok);
        match response_data(&loaded) {
            super::DesktopCommandData::AppSession {
                session, warnings, ..
            } => {
                assert_eq!(session.last_mode, "develop");
                assert_eq!(
                    session.last_library_root_path.as_deref(),
                    Some(library_root.display().to_string().as_str())
                );
                assert_eq!(session.recents.len(), 1);
                assert_eq!(
                    session.recents[0].root_path,
                    expected_session.recents[0].root_path
                );
                assert_eq!(session.recents[0].available, Some(false));
                assert!(warnings.is_empty());
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let reset = super::reset_app_session_at_path(session_path.clone());
        assert!(reset.ok);
        match response_data(&reset) {
            super::DesktopCommandData::AppSession {
                session, warnings, ..
            } => {
                assert_eq!(session.last_mode, "library");
                assert!(session.last_library_root_path.is_none());
                assert!(session.recents.is_empty());
                assert!(warnings.is_empty());
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_app_session_write_rejects_invalid_payload() {
        let workspace = unique_library_root("desktop-app-session-invalid");
        let session_path = workspace.join("AppConfig").join("app-session.json");
        let mut session = super::DesktopAppSession::default();
        session.last_mode = "not-real".to_string();

        let response = super::write_app_session_at_path(session_path.clone(), session);

        assert!(!response.ok);
        assert_eq!(response.command, "write_app_session");
        let error = response.error.as_ref().expect("structured error");
        assert_eq!(error.kind, "appSession");
        assert!(error.message.contains("invalid app session mode"));
        assert!(!session_path.exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_layout_commands_round_trip_and_reset() {
        let workspace = unique_library_root("desktop-layout-preferences");
        let session_path = workspace.join("AppConfig").join("app-session.json");

        let mut layout = super::DesktopLayoutPreferences::from_core(
            silica_core::default_app_layout_preferences(),
        );
        layout.sidebar_collapsed = true;
        layout.inspector_collapsed = true;
        layout.filmstrip_visible = false;
        layout.thumbnail_size = 220;
        layout.sort = "rating_desc".to_string();
        layout.filters.min_rating = Some(4);
        layout.filters.file_type = Some("jpeg".to_string());
        layout.filters.metadata = Some("has_dimensions".to_string());
        layout.filters.search = "portrait".to_string();

        let recorded = super::record_app_session_layout_at_path(session_path.clone(), layout);

        assert!(recorded.ok);
        assert_eq!(recorded.command, "record_app_session_layout");
        match response_data(&recorded) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert!(session.layout.sidebar_collapsed);
                assert!(session.layout.inspector_collapsed);
                assert!(!session.layout.filmstrip_visible);
                assert_eq!(session.layout.thumbnail_size, 220);
                assert_eq!(session.layout.sort, "rating_desc");
                assert_eq!(session.layout.filters.min_rating, Some(4));
                assert_eq!(session.layout.filters.file_type.as_deref(), Some("jpeg"));
                assert_eq!(
                    session.layout.filters.metadata.as_deref(),
                    Some("has_dimensions")
                );
                assert_eq!(session.layout.filters.search, "portrait");
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let reset = super::reset_app_session_layout_at_path(session_path);

        assert!(reset.ok);
        assert_eq!(reset.command, "reset_app_session_layout");
        match response_data(&reset) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert!(!session.layout.sidebar_collapsed);
                assert!(!session.layout.inspector_collapsed);
                assert!(session.layout.filmstrip_visible);
                assert_eq!(session.layout.thumbnail_size, 168);
                assert_eq!(session.layout.sort, "imported_at_desc");
                assert_eq!(session.layout.filters.min_rating, None);
                assert_eq!(session.layout.filters.file_type, None);
                assert_eq!(session.layout.filters.metadata, None);
                assert_eq!(session.layout.filters.search, "");
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_appearance_commands_round_trip_and_reset() {
        let workspace = unique_library_root("desktop-appearance-preferences");
        let session_path = workspace.join("AppConfig").join("app-session.json");

        let appearance = super::DesktopAppearancePreferences {
            theme: "light".to_string(),
            density: "comfortable".to_string(),
            ui_scale: 120,
        };

        let recorded =
            super::record_app_session_appearance_at_path(session_path.clone(), appearance);

        assert!(recorded.ok);
        assert_eq!(recorded.command, "record_app_session_appearance");
        match response_data(&recorded) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(session.appearance.theme, "light");
                assert_eq!(session.appearance.density, "comfortable");
                assert_eq!(session.appearance.ui_scale, 120);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let reset = super::reset_app_session_appearance_at_path(session_path);

        assert!(reset.ok);
        assert_eq!(reset.command, "reset_app_session_appearance");
        match response_data(&reset) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(session.appearance.theme, "dark");
                assert_eq!(session.appearance.density, "compact");
                assert_eq!(session.appearance.ui_scale, 100);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_library_cache_preferences_round_trip() {
        let workspace = unique_library_root("desktop-library-cache-preferences");
        let session_path = workspace.join("AppConfig").join("app-session.json");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);
        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        std::fs::write(
            library_root.join("thumbnails").join("status.cache"),
            b"cache",
        )
        .expect("write cache");
        std::fs::create_dir_all(library_root.join("exports")).expect("create exports");
        std::fs::write(library_root.join("exports").join("keep.jpg"), b"not cache")
            .expect("write export");

        let status = super::get_library_cache_status(library_root.display().to_string());

        assert!(status.ok);
        assert_eq!(status.command, "get_library_cache_status");
        match response_data(&status) {
            super::DesktopCommandData::CacheStatus {
                directories,
                total_bytes,
                cache_record_count,
                ..
            } => {
                assert_eq!(*total_bytes, 5);
                assert_eq!(*cache_record_count, 0);
                assert_eq!(directories.len(), 4);
                assert!(directories.iter().any(|directory| {
                    directory.name == "thumbnails"
                        && directory.byte_size == 5
                        && directory.path.contains("thumbnails")
                }));
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(library_root.join("exports").join("keep.jpg").is_file());

        let preferences = super::DesktopLibraryPreferences {
            default_library_root_path: Some(library_root.display().to_string()),
        };
        let recorded = super::record_app_session_library_preferences_at_path(
            session_path.clone(),
            preferences,
        );

        assert!(recorded.ok);
        assert_eq!(recorded.command, "record_app_session_library_preferences");
        match response_data(&recorded) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(
                    session.library.default_library_root_path.as_deref(),
                    Some(library_root.to_string_lossy().as_ref())
                );
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let reset = super::reset_app_session_library_preferences_at_path(session_path);

        assert!(reset.ok);
        assert_eq!(reset.command, "reset_app_session_library_preferences");
        match response_data(&reset) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(session.library.default_library_root_path, None);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_launch_restore_returns_existing_library_without_repair() {
        let workspace = unique_library_root("desktop-app-session-restore");
        let library_root = workspace.join("SilicaRAW Library");
        let session_path = workspace.join("AppConfig").join("app-session.json");

        let created = super::create_library_at_path(
            library_root.display().to_string(),
            Some(session_path.clone()),
        );
        assert!(created.ok);

        let mut session = super::DesktopAppSession::default();
        session.last_library_root_path = Some(library_root.display().to_string());
        session.last_mode = "develop".to_string();
        let written = super::write_app_session_at_path(session_path.clone(), session);
        assert!(written.ok);
        std::fs::remove_dir_all(library_root.join("thumbnails")).expect("remove thumbnails");

        let restored = super::resolve_launch_restore_at_path(session_path);

        assert!(restored.ok);
        assert_eq!(restored.command, "resolve_launch_restore");
        match response_data(&restored) {
            super::DesktopCommandData::LaunchRestore {
                status,
                state,
                requested_mode,
                resolved_mode,
                library_root_path,
                catalog_path,
                ..
            } => {
                assert_eq!(status, "restored");
                assert_eq!(state, "library");
                assert_eq!(requested_mode, "develop");
                assert_eq!(resolved_mode, "library");
                assert_eq!(
                    library_root_path.as_deref(),
                    Some(library_root.display().to_string().as_str())
                );
                assert_eq!(
                    catalog_path.as_deref(),
                    Some(
                        library_root
                            .join("catalog.db")
                            .display()
                            .to_string()
                            .as_str()
                    )
                );
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(!library_root.join("thumbnails").exists());

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_launch_restore_returns_recorded_selected_photo() {
        let workspace = unique_library_root("desktop-selected-photo-restore");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let session_path = workspace.join("AppConfig").join("app-session.json");
        let supported_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");

        let created = super::create_library_at_path(
            library_root.display().to_string(),
            Some(session_path.clone()),
        );
        assert!(created.ok);
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        let photo_id = silica_core::list_library_photos(&library_root)
            .expect("list photos")
            .into_iter()
            .find(|photo| photo.file_name == "sample.DNG")
            .map(|photo| photo.photo_id)
            .expect("photo id");

        let recorded = super::record_app_session_selection_at_path(
            session_path.clone(),
            library_root.display().to_string(),
            Some(photo_id.clone()),
            "develop".to_string(),
        );
        assert!(recorded.ok);

        let restored = super::resolve_launch_restore_at_path(session_path);

        assert!(restored.ok);
        match response_data(&restored) {
            super::DesktopCommandData::LaunchRestore {
                selected_photo_id,
                selected_photo_status,
                requested_mode,
                resolved_mode,
                ..
            } => {
                assert_eq!(selected_photo_id.as_deref(), Some(photo_id.as_str()));
                assert_eq!(selected_photo_status, "restored");
                assert_eq!(requested_mode, "develop");
                assert_eq!(resolved_mode, "develop");
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_create_open_records_real_recent_after_success() {
        let workspace = unique_library_root("desktop-real-recents");
        let library_root = workspace.join("SilicaRAW Library");
        let session_path = workspace.join("AppConfig").join("app-session.json");

        let created = super::create_library_at_path(
            library_root.display().to_string(),
            Some(session_path.clone()),
        );
        assert!(created.ok);

        let loaded = super::read_app_session_at_path(session_path.clone());
        match response_data(&loaded) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(
                    session.last_library_root_path.as_deref(),
                    Some(library_root.display().to_string().as_str())
                );
                assert_eq!(session.recents.len(), 1);
                assert_eq!(
                    session.recents[0].root_path,
                    library_root.display().to_string()
                );
                assert_eq!(session.recents[0].available, Some(true));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let opened = super::open_library_at_path(
            library_root.display().to_string(),
            Some(session_path.clone()),
        );
        assert!(opened.ok);
        let loaded = super::read_app_session_at_path(session_path.clone());
        match response_data(&loaded) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(session.recents.len(), 1);
                assert_eq!(
                    session.recents[0].root_path,
                    library_root.display().to_string()
                );
                assert_eq!(session.recents[0].available, Some(true));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let failed = super::open_library_at_path(
            workspace.join("Missing Library").display().to_string(),
            Some(session_path.clone()),
        );
        assert!(!failed.ok);
        let loaded = super::read_app_session_at_path(session_path);
        match response_data(&loaded) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(session.recents.len(), 1);
                assert_eq!(
                    session.recents[0].root_path,
                    library_root.display().to_string()
                );
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_create_and_open_library() {
        let root = unique_library_root("desktop");

        let created = super::create_library_at_path(root.display().to_string(), None);
        let opened = super::open_library_at_path(root.display().to_string(), None);

        assert!(created.ok);
        assert!(created.error.is_none());
        assert_eq!(created.command, "create_library");
        assert_eq!(response_data(&created).kind(), "librarySession");
        assert_eq!(
            response_data(&created).root_path(),
            Some(root.display().to_string())
        );
        assert!(created.message.contains("Library created"));

        assert!(opened.ok);
        assert_eq!(opened.command, "open_library");
        assert_eq!(response_data(&opened).kind(), "librarySession");
        assert_eq!(
            response_data(&opened).catalog_path(),
            response_data(&created).catalog_path()
        );

        let missing = super::open_library_at_path(root.join("missing").display().to_string(), None);
        assert!(!missing.ok);
        assert_eq!(missing.command, "open_library");
        let error = missing.error.as_ref().expect("structured error");
        assert_eq!(error.kind, "storage");
        assert_eq!(
            error.context.library_path,
            Some(root.join("missing").display().to_string())
        );
        assert!(error.message.contains("not a directory"));

        remove_library_root(&root);
    }

    #[test]
    fn desktop_commands_set_and_get_photo_flags() {
        let workspace = unique_library_root("desktop-flags");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let updated = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            2,
            true,
            false,
            Some("blue".to_string()),
        );
        assert!(updated.ok);
        match response_data(&updated) {
            super::DesktopCommandData::PhotoFlags {
                rating,
                picked,
                rejected,
                color_label,
                ..
            } => {
                assert_eq!(*rating, 2);
                assert!(*picked);
                assert!(!*rejected);
                assert_eq!(color_label.as_deref(), Some("blue"));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let rejected = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            0,
            false,
            true,
            None,
        );
        assert!(rejected.ok);
        match response_data(&rejected) {
            super::DesktopCommandData::PhotoFlags {
                rating,
                picked,
                rejected,
                color_label,
                ..
            } => {
                assert_eq!(*rating, 0);
                assert!(!*picked);
                assert!(*rejected);
                assert!(color_label.is_none());
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let cleared = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            5,
            false,
            false,
            None,
        );
        assert!(cleared.ok);
        match response_data(&cleared) {
            super::DesktopCommandData::PhotoFlags {
                rating,
                picked,
                rejected,
                ..
            } => {
                assert_eq!(*rating, 5);
                assert!(!*picked);
                assert!(!*rejected);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let reopened = super::get_photo_flags(library_root.display().to_string(), photo_id);
        assert!(reopened.ok);
        assert_eq!(response_data(&reopened), response_data(&cleared));

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_imports_folder_by_reference() {
        let workspace = unique_library_root("desktop-import");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
        std::fs::write(&unsupported_file, b"not a photo").expect("write unsupported");

        silica_core::create_library(&library_root).expect("create library");

        let imported = super::import_folder(
            library_root.display().to_string(),
            import_root.display().to_string(),
            None,
        );

        assert!(imported.ok);
        match response_data(&imported) {
            super::DesktopCommandData::ImportSummary {
                scanned_files,
                supported_files,
                unsupported_files,
                issues,
                originals_unchanged,
                ..
            } => {
                assert_eq!(*scanned_files, 2);
                assert_eq!(*supported_files, 1);
                assert_eq!(*unsupported_files, 1);
                assert!(issues.iter().any(|issue| issue.kind == "unsupported_file"
                    && issue.file_name == Some("notes.txt".to_string())));
                assert!(*originals_unchanged);
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(supported_file.is_file());
        assert!(unsupported_file.is_file());

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_lists_library_photos_for_grid() {
        let workspace = unique_library_root("desktop-grid");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let unsupported_raw_file = import_root.join("sample.DNG");
        let jpeg_file = import_root.join("sample.jpg");
        let unsupported_file = import_root.join("notes.txt");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::write(&unsupported_raw_file, b"unsupported raw candidate")
            .expect("write unsupported raw");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &jpeg_file.display().to_string());
        let flags = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id,
            4,
            true,
            false,
            Some("green".to_string()),
        );
        assert!(flags.ok);

        let grid = super::list_library_photos(library_root.display().to_string());
        assert!(grid.ok);
        match response_data(&grid) {
            super::DesktopCommandData::PhotoGrid { photos } => {
                assert_eq!(photos.len(), 3);
                assert!(photos.iter().any(|photo| {
                    photo.file_name == "sample.jpg"
                        && photo.thumbnail_path.is_some()
                        && photo
                            .thumbnail_bytes
                            .as_ref()
                            .is_some_and(|bytes| !bytes.is_empty())
                        && photo.rating == 4
                        && photo.picked
                        && photo.color_label.as_deref() == Some("green")
                }));
                assert!(photos
                    .iter()
                    .any(|photo| photo.file_name == "sample.DNG" && photo.unsupported));
                assert!(photos
                    .iter()
                    .any(|photo| photo.file_name == "notes.txt" && photo.unsupported));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn paged_grid_command_returns_typed_page_and_structured_errors() {
        let workspace = unique_library_root("desktop-paged-grid");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("portrait.jpg");
        let unsupported_source_file = import_root.join("sample.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);
        std::fs::write(&unsupported_source_file, b"raw candidate").expect("write raw");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        let unsupported_source_id =
            stable_catalog_id("photo", &unsupported_source_file.display().to_string());
        silica_core::set_photo_flags(
            &library_root,
            unsupported_source_id.clone(),
            5,
            true,
            false,
            None,
        )
        .expect("set unsupported source flags");

        let page = super::query_library_photos(
            library_root.display().to_string(),
            super::DesktopLibraryQueryRequest {
                offset: 0,
                limit: 1,
                sort: "rating_desc".to_string(),
                filters: super::DesktopLibraryQueryFilters {
                    min_rating: Some(4),
                    picked: Some(true),
                    rejected: None,
                    file_type: Some("unsupported".to_string()),
                    metadata: None,
                    search: "sample".to_string(),
                },
            },
        );

        assert!(page.ok);
        assert_eq!(page.command, "query_library_photos");
        match response_data(&page) {
            super::DesktopCommandData::PhotoGridPage {
                photos,
                offset,
                limit,
                total_count,
                has_next_page,
                order_fields,
            } => {
                assert_eq!(*offset, 0);
                assert_eq!(*limit, 1);
                assert_eq!(*total_count, 1);
                assert!(!has_next_page);
                assert_eq!(order_fields, &["rating_desc", "photo_id_asc"]);
                assert_eq!(photos.len(), 1);
                assert_eq!(photos[0].photo_id, unsupported_source_id);
                assert!(photos[0].unsupported);
                assert_eq!(photos[0].thumbnail_bytes, None);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let invalid = super::query_library_photos(
            library_root.display().to_string(),
            super::DesktopLibraryQueryRequest {
                offset: 0,
                limit: 1,
                sort: "created_at_desc".to_string(),
                filters: super::DesktopLibraryQueryFilters::default(),
            },
        );
        assert!(!invalid.ok);
        assert_eq!(invalid.command, "query_library_photos");
        let error = invalid.error.as_ref().expect("structured error");
        assert_eq!(error.kind, "appSession");
        assert_eq!(
            error.context.library_path.as_deref(),
            Some(library_root.to_string_lossy().as_ref())
        );

        let invalid_metadata = super::query_library_photos(
            library_root.display().to_string(),
            super::DesktopLibraryQueryRequest {
                offset: 0,
                limit: 1,
                sort: "file_name_asc".to_string(),
                filters: super::DesktopLibraryQueryFilters {
                    metadata: Some("camera_make".to_string()),
                    ..super::DesktopLibraryQueryFilters::default()
                },
            },
        );
        assert!(!invalid_metadata.ok);
        assert_eq!(invalid_metadata.command, "query_library_photos");
        assert!(invalid_metadata
            .error
            .as_ref()
            .expect("metadata error")
            .message
            .contains("invalid library query metadata filter"));

        remove_library_root(&workspace);
    }

    #[test]
    fn metadata_command_returns_typed_field_states() {
        let workspace = unique_library_root("desktop-metadata-query");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let jpeg_file = import_root.join("portrait.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&jpeg_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        std::fs::remove_file(&jpeg_file).expect("remove original before desktop metadata query");

        let photo_id = stable_catalog_id("photo", &jpeg_file.display().to_string());
        let response = super::get_photo_metadata(library_root.display().to_string(), photo_id);

        assert!(response.ok);
        assert_eq!(response.command, "get_photo_metadata");
        match response_data(&response) {
            super::DesktopCommandData::PhotoMetadata {
                photo_id,
                width,
                height,
                camera_make,
                ..
            } => {
                assert_eq!(
                    photo_id,
                    &stable_catalog_id("photo", &jpeg_file.display().to_string())
                );
                assert_eq!(width.state, "known");
                assert_eq!(width.value, Some(2));
                assert_eq!(height.state, "known");
                assert_eq!(height.value, Some(2));
                assert_eq!(camera_make.state, "unavailable");
                assert_eq!(camera_make.value, None);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let missing = super::get_photo_metadata(
            library_root.display().to_string(),
            "missing-photo".to_string(),
        );
        assert!(missing.ok);
        assert_eq!(missing.data, None);

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_opens_photo_preview_status() {
        let workspace = unique_library_root("desktop-preview");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::open_photo_preview(library_root.display().to_string(), photo_id);

        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::PhotoPreview {
                file_name,
                status,
                message,
                preview_bytes,
                ..
            } => {
                assert_eq!(file_name, "sample.jpg");
                assert_eq!(*status, "Ready");
                assert!(message.contains("display-profile-aware"));
                assert!(preview_bytes.as_ref().is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_exposure_contrast_edit() {
        let workspace = unique_library_root("desktop-edit-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.5,
            -8.0,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                exposure,
                contrast,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(*exposure, 0.5);
                assert_eq!(*contrast, -8.0);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.5,
            -8.0,
        );
        assert!(committed.ok);
        match response_data(&committed) {
            super::DesktopCommandData::EditCommit {
                exposure,
                contrast,
                persisted,
                ..
            } => {
                assert_eq!(*exposure, 0.5);
                assert_eq!(*contrast, -8.0);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let restored =
            super::get_photo_edit_state(library_root.display().to_string(), photo_id.clone());
        assert!(restored.ok);
        match response_data(&restored) {
            super::DesktopCommandData::EditState {
                exposure,
                contrast,
                persisted,
                ..
            } => {
                assert_eq!(*exposure, 0.5);
                assert_eq!(*contrast, -8.0);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_edit_state_returns_manual_mask_readback() {
        let workspace = unique_library_root("desktop-mask-readback");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        silica_core::commit_manual_linear_gradient_mask(
            &library_root,
            &photo_id,
            "mask-linear-1",
            "Linear Gradient 1",
            82.0,
            24.0,
            false,
            0.0,
            0.0,
            1.0,
            1.0,
            Some(0.75),
            Some(-12.0),
        )
        .expect("commit manual mask")
        .expect("manual mask commit result");

        let restored =
            super::get_photo_edit_state(library_root.display().to_string(), photo_id.clone());
        assert!(restored.ok);
        match response_data(&restored) {
            super::DesktopCommandData::EditState { masks, .. } => {
                assert_eq!(masks.len(), 1);
                assert_eq!(masks[0].kind, "linear_gradient");
                assert_eq!(masks[0].name, "Linear Gradient 1");
                assert_eq!(masks[0].exposure, 0.75);
                assert_eq!(masks[0].contrast, -12.0);
                assert!(matches!(
                    &masks[0].geometry,
                    Some(super::DesktopManualMaskGeometryState::LinearGradient { .. })
                ));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_white_balance_edit() {
        let workspace = unique_library_root("desktop-white-balance-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_white_balance_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            "custom".to_string(),
            6500.0,
            20.0,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                white_balance,
                temperature,
                tint,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(*white_balance, "custom");
                assert_eq!(*temperature, 6500.0);
                assert_eq!(*tint, 20.0);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_white_balance_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            "custom".to_string(),
            6500.0,
            20.0,
        );
        assert!(committed.ok);
        match response_data(&committed) {
            super::DesktopCommandData::EditCommit {
                white_balance,
                temperature,
                tint,
                persisted,
                ..
            } => {
                assert_eq!(*white_balance, "custom");
                assert_eq!(*temperature, 6500.0);
                assert_eq!(*tint, 20.0);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let restored =
            super::get_photo_edit_state(library_root.display().to_string(), photo_id.clone());
        assert!(restored.ok);
        match response_data(&restored) {
            super::DesktopCommandData::EditState {
                white_balance,
                temperature,
                tint,
                persisted,
                ..
            } => {
                assert_eq!(*white_balance, "custom");
                assert_eq!(*temperature, 6500.0);
                assert_eq!(*tint, 20.0);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_tone_recovery_edit() {
        let workspace = unique_library_root("desktop-tone-recovery-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_tone_recovery_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            -35.0,
            42.0,
            10.0,
            -12.0,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                highlights,
                shadows,
                whites,
                blacks,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(*highlights, -35.0);
                assert_eq!(*shadows, 42.0);
                assert_eq!(*whites, 10.0);
                assert_eq!(*blacks, -12.0);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_tone_recovery_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            -35.0,
            42.0,
            10.0,
            -12.0,
        );
        assert!(committed.ok);
        match response_data(&committed) {
            super::DesktopCommandData::EditCommit {
                highlights,
                shadows,
                whites,
                blacks,
                persisted,
                ..
            } => {
                assert_eq!(*highlights, -35.0);
                assert_eq!(*shadows, 42.0);
                assert_eq!(*whites, 10.0);
                assert_eq!(*blacks, -12.0);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_tone_curve_edit() {
        let workspace = unique_library_root("desktop-tone-curve-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let rgb_curve = vec![
            super::DesktopToneCurvePoint { x: 0.0, y: 0.0 },
            super::DesktopToneCurvePoint { x: 0.5, y: 0.28 },
            super::DesktopToneCurvePoint { x: 1.0, y: 1.0 },
        ];
        let preview = super::preview_tone_curve_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            rgb_curve.clone(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                tone_curve,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(tone_curve.curve_mode, "point");
                assert_eq!(tone_curve.rgb_curve.len(), 3);
                assert_eq!(tone_curve.rgb_curve[1].y, 0.28);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_tone_curve_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            rgb_curve,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(committed.ok);
        match response_data(&committed) {
            super::DesktopCommandData::EditCommit {
                tone_curve,
                persisted,
                ..
            } => {
                assert_eq!(tone_curve.curve_mode, "point");
                assert_eq!(tone_curve.rgb_curve[1].x, 0.5);
                assert_eq!(tone_curve.rgb_curve[1].y, 0.28);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_color_presence_edit() {
        let workspace = unique_library_root("desktop-color-presence-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_color_presence_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            24.0,
            -8.5,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                vibrance,
                saturation,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(*vibrance, 24.0);
                assert_eq!(*saturation, -8.5);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_color_presence_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            24.0,
            -8.5,
        );
        assert!(committed.ok);
        match response_data(&committed) {
            super::DesktopCommandData::EditCommit {
                vibrance,
                saturation,
                persisted,
                ..
            } => {
                assert_eq!(*vibrance, 24.0);
                assert_eq!(*saturation, -8.5);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_hsl_color_mixer_edit() {
        let workspace = unique_library_root("desktop-hsl-color-mixer-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_hsl_color_mixer_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            "blue".to_string(),
            -12.0,
            24.0,
            -8.5,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                hsl_color_mixer,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(hsl_color_mixer.blue.hue, -12.0);
                assert_eq!(hsl_color_mixer.blue.saturation, 24.0);
                assert_eq!(hsl_color_mixer.blue.luminance, -8.5);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_hsl_color_mixer_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            "blue".to_string(),
            -12.0,
            24.0,
            -8.5,
        );
        assert!(committed.ok);
        match response_data(&committed) {
            super::DesktopCommandData::EditCommit {
                hsl_color_mixer,
                persisted,
                ..
            } => {
                assert_eq!(hsl_color_mixer.blue.hue, -12.0);
                assert_eq!(hsl_color_mixer.blue.saturation, 24.0);
                assert_eq!(hsl_color_mixer.blue.luminance, -8.5);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_preview_and_commit_geometry_edit() {
        let workspace = unique_library_root("desktop-geometry-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_geometry_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_geometry_crop_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.0,
            0.0,
            0.5,
            1.0,
            0.0,
            None,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                geometry,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(geometry.crop.as_ref().map(|crop| crop.width), Some(0.5));
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let crop_commit = super::commit_geometry_crop_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.0,
            0.0,
            0.5,
            1.0,
            0.0,
            None,
        );
        assert!(crop_commit.ok);
        match response_data(&crop_commit) {
            super::DesktopCommandData::EditCommit {
                geometry,
                persisted,
                ..
            } => {
                assert_eq!(geometry.crop.as_ref().map(|crop| crop.height), Some(1.0));
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let orientation_commit = super::commit_geometry_orientation_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            90.0,
            true,
            false,
        );
        assert!(orientation_commit.ok);
        match response_data(&orientation_commit) {
            super::DesktopCommandData::EditCommit {
                geometry,
                persisted,
                ..
            } => {
                assert_eq!(geometry.rotation, 90.0);
                assert!(geometry.flip_horizontal);
                assert!(*persisted);
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_block_detail_commit_until_renderer_support_exists() {
        let workspace = unique_library_root("desktop-detail-boundary-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preview = super::preview_detail_sharpening_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            42.0,
            1.2,
            35.0,
            10.0,
        );
        assert!(preview.ok);
        match response_data(&preview) {
            super::DesktopCommandData::EditPreview {
                status,
                detail,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Unsupported");
                assert_eq!(detail.sharpening.amount, 42.0);
                assert!(develop_preview_bytes.is_none());
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        let committed = super::commit_detail_sharpening_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            42.0,
            1.2,
            35.0,
            10.0,
        );
        assert!(!committed.ok);
        assert_eq!(
            committed.error.as_ref().map(|error| error.kind.as_str()),
            Some("unsupportedEdit")
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_commit_basic_preset_and_reset() {
        let workspace = unique_library_root("desktop-basic-preset-reset");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let preset = super::commit_basic_preset_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            "warm_contrast".to_string(),
        );
        assert!(preset.ok, "preset failed: {preset:?}");
        match response_data(&preset) {
            super::DesktopCommandData::EditCommit {
                white_balance,
                temperature,
                contrast,
                vibrance,
                persisted,
                ..
            } => {
                assert_eq!(*white_balance, "custom");
                assert_eq!(*temperature, 6200.0);
                assert_eq!(*contrast, 18.0);
                assert_eq!(*vibrance, 12.0);
                assert!(*persisted);
            }
            other => panic!("unexpected preset response data: {other:?}"),
        }

        let reset =
            super::commit_p0_basic_reset(library_root.display().to_string(), photo_id.clone());
        assert!(reset.ok, "reset failed: {reset:?}");
        match response_data(&reset) {
            super::DesktopCommandData::EditCommit {
                white_balance,
                temperature,
                exposure,
                contrast,
                highlights,
                shadows,
                whites,
                blacks,
                vibrance,
                saturation,
                persisted,
                ..
            } => {
                assert_eq!(*white_balance, "as_shot");
                assert_eq!(*temperature, 5200.0);
                assert_eq!(*exposure, 0.0);
                assert_eq!(*contrast, 0.0);
                assert_eq!(*highlights, 0.0);
                assert_eq!(*shadows, 0.0);
                assert_eq!(*whites, 0.0);
                assert_eq!(*blacks, 0.0);
                assert_eq!(*vibrance, 0.0);
                assert_eq!(*saturation, 0.0);
                assert!(*persisted);
            }
            other => panic!("unexpected reset response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_copy_plan_and_apply_edit_clipboard_sync() {
        let workspace = unique_library_root("desktop-edit-clipboard-sync");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let source_file = import_root.join("source.jpg");
        let target_file = import_root.join("target.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&source_file);
        write_source_jpeg(&target_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let source_photo_id = stable_catalog_id("photo", &source_file.display().to_string());
        let target_photo_id = stable_catalog_id("photo", &target_file.display().to_string());
        let source_commit = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            source_photo_id.clone(),
            0.55,
            14.0,
        );
        assert!(source_commit.ok, "source commit failed: {source_commit:?}");

        let selection = silica_core::EditClipboardSelection {
            basic: true,
            ..Default::default()
        };
        let copied = super::copy_edit_clipboard_payload(
            library_root.display().to_string(),
            source_photo_id.clone(),
            selection,
        );
        assert!(copied.ok, "copy failed: {copied:?}");
        let payload = match response_data(&copied) {
            super::DesktopCommandData::EditClipboard {
                photo_id,
                section_count,
                payload,
                ..
            } => {
                assert_eq!(photo_id, &source_photo_id);
                assert_eq!(*section_count, 1);
                assert!(payload.basic.is_some());
                payload.clone()
            }
            other => panic!("unexpected copy response data: {other:?}"),
        };

        let targets = vec![target_photo_id.clone()];
        let plan = super::plan_edit_clipboard_sync(
            library_root.display().to_string(),
            targets.clone(),
            payload.clone(),
        );
        assert!(plan.ok, "plan failed: {plan:?}");
        match response_data(&plan) {
            super::DesktopCommandData::EditClipboardPlan {
                status,
                ready_count,
                blocked_count,
                targets,
                ..
            } => {
                assert_eq!(status, "ready");
                assert_eq!(*ready_count, 1);
                assert_eq!(*blocked_count, 0);
                assert_eq!(targets[0].photo_id, target_photo_id);
                assert_eq!(targets[0].status, "ready");
            }
            other => panic!("unexpected plan response data: {other:?}"),
        }

        let applied =
            super::apply_edit_clipboard_sync(library_root.display().to_string(), targets, payload);
        assert!(applied.ok, "apply failed: {applied:?}");
        match response_data(&applied) {
            super::DesktopCommandData::EditClipboardSync {
                status,
                applied_count,
                blocked_count,
                commits,
                ..
            } => {
                assert_eq!(status, "applied");
                assert_eq!(*applied_count, 1);
                assert_eq!(*blocked_count, 0);
                assert_eq!(commits.len(), 1);
                assert_eq!(commits[0].photo_id, target_photo_id);
            }
            other => panic!("unexpected apply response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_edit_clipboard_blocks_raw_copy_and_sync_target() {
        let workspace = unique_library_root("desktop-edit-clipboard-raw-blocked");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let source_file = import_root.join("source.jpg");
        let raw_file = import_root.join("target.DNG");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&source_file);
        std::fs::write(&raw_file, b"raw target placeholder").expect("write raw target");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let source_photo_id = stable_catalog_id("photo", &source_file.display().to_string());
        let raw_photo_id = stable_catalog_id("photo", &raw_file.display().to_string());
        let selection = silica_core::EditClipboardSelection {
            basic: true,
            ..Default::default()
        };

        let raw_copy = super::copy_edit_clipboard_payload(
            library_root.display().to_string(),
            raw_photo_id.clone(),
            selection,
        );
        assert!(
            !raw_copy.ok,
            "RAW copy unexpectedly succeeded: {raw_copy:?}"
        );
        assert_eq!(
            raw_copy.error.as_ref().map(|error| error.kind.as_str()),
            Some("unsupportedEdit")
        );
        assert!(raw_copy.message.contains("JPEG/JPG"));

        let source_commit = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            source_photo_id.clone(),
            0.55,
            14.0,
        );
        assert!(source_commit.ok, "source commit failed: {source_commit:?}");

        let copied = super::copy_edit_clipboard_payload(
            library_root.display().to_string(),
            source_photo_id,
            silica_core::EditClipboardSelection {
                basic: true,
                ..Default::default()
            },
        );
        assert!(copied.ok, "source copy failed: {copied:?}");
        let payload = match response_data(&copied) {
            super::DesktopCommandData::EditClipboard { payload, .. } => payload.clone(),
            other => panic!("unexpected copy response data: {other:?}"),
        };

        let targets = vec![raw_photo_id.clone()];
        let plan = super::plan_edit_clipboard_sync(
            library_root.display().to_string(),
            targets.clone(),
            payload.clone(),
        );
        assert!(plan.ok, "raw target plan failed: {plan:?}");
        match response_data(&plan) {
            super::DesktopCommandData::EditClipboardPlan {
                status,
                ready_count,
                blocked_count,
                targets,
                ..
            } => {
                assert_eq!(status, "blocked");
                assert_eq!(*ready_count, 0);
                assert_eq!(*blocked_count, 1);
                assert_eq!(targets[0].photo_id, raw_photo_id);
                assert_eq!(targets[0].status, "blocked");
                assert_eq!(targets[0].code.as_deref(), Some("unsupported_target"));
                assert!(targets[0].message.contains("JPEG/JPG"));
            }
            other => panic!("unexpected raw plan response data: {other:?}"),
        }

        let applied =
            super::apply_edit_clipboard_sync(library_root.display().to_string(), targets, payload);
        assert!(applied.ok, "raw target apply failed: {applied:?}");
        match response_data(&applied) {
            super::DesktopCommandData::EditClipboardSync {
                status,
                applied_count,
                blocked_count,
                failed_count,
                commits,
                failures,
                targets,
                ..
            } => {
                assert_eq!(status, "blocked");
                assert_eq!(*applied_count, 0);
                assert_eq!(*blocked_count, 1);
                assert_eq!(*failed_count, 1);
                assert!(commits.is_empty());
                assert_eq!(failures[0].photo_id, raw_photo_id);
                assert_eq!(targets[0].code.as_deref(), Some("unsupported_target"));
            }
            other => panic!("unexpected raw apply response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_returns_histogram_contract() {
        let workspace = unique_library_root("desktop-histogram-flow");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let histogram =
            super::get_photo_histogram(library_root.display().to_string(), photo_id.clone());
        assert!(histogram.ok);
        match response_data(&histogram) {
            super::DesktopCommandData::Histogram {
                status,
                red,
                green,
                blue,
                luminance,
                pixel_count,
                cache_path,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(*pixel_count, 4);
                assert_eq!(red.len(), 256);
                assert_eq!(green.len(), 256);
                assert_eq!(blue.len(), 256);
                assert_eq!(luminance.len(), 256);
                assert!(cache_path.contains("render-cache"));
            }
            other => panic!("unexpected response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_commands_undo_and_redo_history() {
        let workspace = unique_library_root("desktop-undo-redo");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let commit = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.5,
            -8.0,
        );
        assert!(commit.ok, "commit failed: {commit:?}");

        let undo =
            super::undo_last_history_action(library_root.display().to_string(), photo_id.clone());
        assert!(undo.ok, "undo failed: {undo:?}");
        match response_data(&undo) {
            super::DesktopCommandData::HistoryCommand {
                applied,
                action_kind,
                ..
            } => {
                assert!(*applied);
                assert_eq!(action_kind.as_deref(), Some("edit_commit"));
            }
            other => panic!("unexpected undo response data: {other:?}"),
        }

        let redo = super::redo_last_history_action(library_root.display().to_string(), photo_id);
        assert!(redo.ok, "redo failed: {redo:?}");
        match response_data(&redo) {
            super::DesktopCommandData::HistoryCommand {
                applied,
                action_kind,
                ..
            } => {
                assert!(*applied);
                assert_eq!(action_kind.as_deref(), Some("edit_commit"));
            }
            other => panic!("unexpected redo response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_returns_history_panel_contract() {
        let workspace = unique_library_root("desktop-history-panel");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

        let empty = super::get_photo_history(library_root.display().to_string(), photo_id.clone());
        assert!(empty.ok, "empty history failed: {empty:?}");
        match response_data(&empty) {
            super::DesktopCommandData::HistoryPanel {
                items,
                can_undo,
                can_redo,
                status,
                ..
            } => {
                assert!(items.is_empty());
                assert!(!can_undo);
                assert!(!can_redo);
                assert_eq!(status, "empty");
            }
            other => panic!("unexpected empty history response data: {other:?}"),
        }

        let commit = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.5,
            -8.0,
        );
        assert!(commit.ok, "commit failed: {commit:?}");

        let history = super::get_photo_history(library_root.display().to_string(), photo_id);
        assert!(history.ok, "history failed: {history:?}");
        assert_eq!(response_data(&history).kind(), "historyPanel");
        match response_data(&history) {
            super::DesktopCommandData::HistoryPanel {
                items,
                can_undo,
                can_redo,
                status,
                ..
            } => {
                assert_eq!(status, "ready");
                assert!(*can_undo);
                assert!(!can_redo);
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].action_kind, "edit_commit");
                assert_eq!(items[0].label, "Exposure / contrast");
                assert_eq!(items[0].history_state, "applied");
            }
            other => panic!("unexpected history response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_returns_ai_review_panel_without_mutating_edits() {
        let workspace = unique_library_root("desktop-ai-review");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        let created = silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        silica_core::store_ai_result(
            &created.root_path,
            &photo_id,
            "blur_score",
            "silicaraw.blur-review.test",
            r#"{"review":{"label":"Motion blur likely","recommendation":"review","confidence":0.91}}"#,
        )
        .expect("store ai review");

        let response =
            super::get_ai_review_panel(library_root.display().to_string(), photo_id.clone());

        assert!(response.ok, "AI review command failed: {response:?}");
        assert_eq!(response.command, "get_ai_review_panel");
        match response_data(&response) {
            super::DesktopCommandData::AiReviewPanel {
                photo_id: returned_photo_id,
                status,
                items,
                writes_edit_graph,
                writes_photo_flags,
                requires_explicit_approval,
                ..
            } => {
                assert_eq!(returned_photo_id, &photo_id);
                assert_eq!(*status, "reviewAvailable");
                assert_eq!(items.len(), 1);
                assert_eq!(items[0].label, "Motion blur likely");
                assert_eq!(items[0].confidence_percent, Some(91));
                assert!(!*writes_edit_graph);
                assert!(!*writes_photo_flags);
                assert!(*requires_explicit_approval);
            }
            other => panic!("unexpected AI review response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_approves_ai_suggestion_with_undoable_edit() {
        let workspace = unique_library_root("desktop-ai-approval");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);

        let created = silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let result = silica_core::store_ai_result(
            &created.root_path,
            &photo_id,
            "blur_score",
            "silicaraw.blur-review.test",
            r#"{"review":{"label":"Usable detail","recommendation":"keep","confidence":0.74},"approval_suggestion":{"kind":"basic_exposure_contrast","exposure":0.3,"contrast":7.0}}"#,
        )
        .expect("store ai suggestion");

        let response = super::approve_ai_suggestion(
            library_root.display().to_string(),
            photo_id.clone(),
            result.id.clone(),
        );

        assert!(response.ok, "AI approval command failed: {response:?}");
        assert_eq!(response.command, "approve_ai_suggestion");
        match response_data(&response) {
            super::DesktopCommandData::AiSuggestionApproval {
                photo_id: returned_photo_id,
                result_id,
                model_id,
                commit,
                writes_edit_graph,
                writes_photo_flags,
                ..
            } => {
                assert_eq!(returned_photo_id, &photo_id);
                assert_eq!(result_id, &result.id);
                assert_eq!(model_id, "silicaraw.blur-review.test");
                assert!(commit.persisted);
                assert_eq!(commit.exposure, 0.3);
                assert_eq!(commit.contrast, 7.0);
                assert!(*writes_edit_graph);
                assert!(!*writes_photo_flags);
            }
            other => panic!("unexpected AI approval response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_reads_export_settings_defaults() {
        let workspace = unique_library_root("desktop-export-settings");
        let library_root = workspace.join("SilicaRAW Library");

        silica_core::create_library(&library_root).expect("create library");

        let response = super::get_export_settings(library_root.display().to_string());

        assert!(response.ok);
        match response_data(&response) {
            super::DesktopCommandData::ExportSettings {
                default_settings,
                default_preset_id,
                presets,
                ..
            } => {
                assert_eq!(default_preset_id.as_deref(), Some("jpeg-srgb-90"));
                assert_eq!(default_settings.format, "jpeg");
                assert_eq!(default_settings.color_profile, "srgb");
                assert_eq!(default_settings.quality, 90);
                assert_eq!(default_settings.metadata_policy, "minimal");
                assert!(presets.iter().any(|preset| preset.id == "jpeg-srgb-90"));
            }
            other => panic!("unexpected export settings response data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_saves_export_preset_as_default() {
        let workspace = unique_library_root("desktop-export-preset");
        let library_root = workspace.join("SilicaRAW Library");

        silica_core::create_library(&library_root).expect("create library");

        let response = super::save_export_preset(
            library_root.display().to_string(),
            "Desktop Display P3 Review".to_string(),
            Some("jpeg".to_string()),
            Some("display_p3".to_string()),
            Some(90),
            Some("preserve".to_string()),
        );

        assert!(response.ok);
        let saved_preset_id = match response_data(&response) {
            super::DesktopCommandData::ExportSettings {
                default_settings,
                default_preset_id,
                presets,
                ..
            } => {
                assert_eq!(default_settings.format, "jpeg");
                assert_eq!(default_settings.color_profile, "display_p3");
                assert_eq!(default_settings.metadata_policy, "preserve");
                let default_preset_id = default_preset_id
                    .as_deref()
                    .expect("default preset id")
                    .to_string();
                assert!(presets.iter().any(|preset| preset.id == default_preset_id));
                default_preset_id
            }
            other => panic!("unexpected export preset response data: {other:?}"),
        };

        let reloaded = super::get_export_settings(library_root.display().to_string());
        assert!(reloaded.ok);
        match response_data(&reloaded) {
            super::DesktopCommandData::ExportSettings {
                default_settings,
                default_preset_id,
                ..
            } => {
                assert_eq!(default_preset_id.as_deref(), Some(saved_preset_id.as_str()));
                assert_eq!(default_settings.color_profile, "display_p3");
            }
            other => panic!("unexpected reloaded export settings data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_saves_png_export_settings_and_rejects_display_p3() {
        let workspace = unique_library_root("desktop-export-settings-format");
        let library_root = workspace.join("SilicaRAW Library");

        silica_core::create_library(&library_root).expect("create library");

        let response = super::save_export_settings(
            library_root.display().to_string(),
            None,
            Some("png".to_string()),
            None,
            Some(90),
            Some("remove_all".to_string()),
        );
        assert!(response.ok, "save PNG settings failed: {response:?}");
        match response_data(&response) {
            super::DesktopCommandData::ExportSettings {
                default_settings, ..
            } => {
                assert_eq!(default_settings.format, "png");
                assert_eq!(default_settings.color_profile, "srgb");
                assert_eq!(default_settings.quality, 90);
                assert_eq!(default_settings.metadata_policy, "remove_all");
            }
            other => panic!("unexpected export settings response data: {other:?}"),
        }

        let rejected = super::save_export_settings(
            library_root.display().to_string(),
            None,
            Some("png".to_string()),
            Some("display_p3".to_string()),
            Some(90),
            Some("minimal".to_string()),
        );
        assert!(!rejected.ok);
        let error = rejected.error.as_ref().expect("error payload");
        assert_eq!(error.kind, "exportBlocked");
        assert!(error
            .message
            .contains("PNG and TIFF export settings currently require sRGB"));

        let invalid_policy = super::save_export_settings(
            library_root.display().to_string(),
            None,
            Some("jpeg".to_string()),
            None,
            Some(90),
            Some("gps_only".to_string()),
        );
        assert!(!invalid_policy.ok);
        let error = invalid_policy.error.as_ref().expect("error payload");
        assert_eq!(error.kind, "exportBlocked");
        assert!(error
            .message
            .contains("Unsupported export metadata policy: gps_only"));

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_exports_photo_jpeg_srgb() {
        let workspace = unique_library_root("desktop-export");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let supported_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-export.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        silica_core::commit_exposure_contrast_edit(&library_root, &photo_id, 0.5, -8.0)
            .expect("commit edit")
            .expect("committed edit");

        let export = super::export_photo_jpeg_srgb(
            library_root.display().to_string(),
            photo_id,
            output_path.display().to_string(),
        );

        assert!(export.ok);
        match response_data(&export) {
            super::DesktopCommandData::Export {
                format,
                color_profile,
                output_path: actual_output_path,
                ..
            } => {
                assert_eq!(format, "jpeg");
                assert_eq!(color_profile, "srgb");
                assert_eq!(actual_output_path, &output_path.display().to_string());
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(output_path.is_file());

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_exports_photo_png_and_tiff() {
        let workspace = unique_library_root("desktop-export-raster-formats");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let supported_file = import_root.join("sample.jpg");
        let png_output_path = export_root.join("sample-export.png");
        let tiff_output_path = export_root.join("sample-export.tiff");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&supported_file);
        let original_before = std::fs::read(&supported_file).expect("read original before");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let png_export = super::export_photo_png(
            library_root.display().to_string(),
            photo_id.clone(),
            png_output_path.display().to_string(),
        );
        assert!(png_export.ok, "PNG export failed: {png_export:?}");
        match response_data(&png_export) {
            super::DesktopCommandData::Export {
                format,
                color_profile,
                output_path,
                icc_profile_embedded,
                ..
            } => {
                assert_eq!(format, "png");
                assert_eq!(color_profile, "srgb");
                assert_eq!(output_path, &png_output_path.display().to_string());
                assert!(!icc_profile_embedded);
            }
            other => panic!("unexpected PNG response data: {other:?}"),
        }

        let tiff_export = super::export_photo_tiff(
            library_root.display().to_string(),
            photo_id,
            tiff_output_path.display().to_string(),
        );
        assert!(tiff_export.ok, "TIFF export failed: {tiff_export:?}");
        match response_data(&tiff_export) {
            super::DesktopCommandData::Export {
                format,
                color_profile,
                output_path,
                icc_profile_embedded,
                ..
            } => {
                assert_eq!(format, "tiff");
                assert_eq!(color_profile, "srgb");
                assert_eq!(output_path, &tiff_output_path.display().to_string());
                assert!(!icc_profile_embedded);
            }
            other => panic!("unexpected TIFF response data: {other:?}"),
        }

        assert!(png_output_path.is_file());
        assert!(tiff_output_path.is_file());
        assert_eq!(
            std::fs::read(&supported_file).expect("read original after"),
            original_before
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_lists_recent_exports_with_missing_evidence() {
        let workspace = unique_library_root("desktop-recent-exports");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let supported_file = import_root.join("sample.jpg");
        let existing_output = export_root.join("sample-existing.jpg");
        let missing_output = export_root.join("sample-missing.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let first = super::export_photo_jpeg_srgb(
            library_root.display().to_string(),
            photo_id.clone(),
            existing_output.display().to_string(),
        );
        assert!(first.ok, "first export failed: {first:?}");
        let second = super::export_photo_jpeg_srgb(
            library_root.display().to_string(),
            photo_id,
            missing_output.display().to_string(),
        );
        assert!(second.ok, "second export failed: {second:?}");
        std::fs::remove_file(&missing_output).expect("remove exported output");

        let recent = super::get_recent_exports(library_root.display().to_string(), Some(2));

        assert!(recent.ok, "recent exports failed: {recent:?}");
        match response_data(&recent) {
            super::DesktopCommandData::RecentExports { exports, .. } => {
                assert_eq!(exports.len(), 2);
                assert_eq!(exports[0].output_path, missing_output.display().to_string());
                assert!(!exports[0].output_exists);
                assert_eq!(
                    exports[1].output_path,
                    existing_output.display().to_string()
                );
                assert!(exports[1].output_exists);
            }
            other => panic!("unexpected recent exports data: {other:?}"),
        }

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_exports_photo_jpeg_display_p3_explicitly() {
        let workspace = unique_library_root("desktop-export-display-p3");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let export_root = workspace.join("Exports");
        let supported_file = import_root.join("sample.jpg");
        let output_path = export_root.join("sample-display-p3.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        std::fs::create_dir_all(&export_root).expect("create export directory");
        write_source_jpeg(&supported_file);

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");

        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        let export = super::export_photo_jpeg(
            library_root.display().to_string(),
            photo_id,
            output_path.display().to_string(),
            Some("display_p3".to_string()),
            None,
        );

        assert!(export.ok);
        match response_data(&export) {
            super::DesktopCommandData::Export {
                format,
                color_profile,
                output_path: actual_output_path,
                ..
            } => {
                assert_eq!(format, "jpeg");
                assert_eq!(color_profile, "display_p3");
                assert_eq!(actual_output_path, &output_path.display().to_string());
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        assert!(output_path.is_file());

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_command_blocks_unsupported_jpeg_color_profile() {
        let rejected = super::export_photo_jpeg(
            "/tmp/missing-library".to_string(),
            "photo-1".to_string(),
            "/tmp/output.jpg".to_string(),
            Some("adobe_rgb".to_string()),
            None,
        );

        assert!(!rejected.ok);
        let error = rejected.error.as_ref().expect("error payload");
        assert_eq!(error.kind, "exportBlocked");
        assert!(error
            .message
            .contains("Unsupported export color profile: adobe_rgb"));
    }

    #[test]
    fn desktop_command_clears_only_disposable_cache() {
        let workspace = unique_library_root("desktop-cache-clear");
        let library_root = workspace.join("SilicaRAW Library");
        let import_root = workspace.join("Originals");
        let supported_file = import_root.join("sample.jpg");

        std::fs::create_dir_all(&import_root).expect("create import directory");
        write_source_jpeg(&supported_file);
        let original_bytes = std::fs::read(&supported_file).expect("read original before");

        silica_core::create_library(&library_root).expect("create library");
        silica_core::import_folder(&library_root, &import_root).expect("import folder");
        let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
        silica_core::open_photo_preview(&library_root, &photo_id)
            .expect("open preview")
            .expect("preview session");
        for directory in ["render-cache", "ai-cache"] {
            let path = library_root.join(directory);
            std::fs::create_dir_all(&path).expect("create cache directory");
            std::fs::write(path.join("sentinel.cache"), b"cache bytes")
                .expect("write cache sentinel");
        }
        for directory in ["sidecars", "exports", "logs", "backups"] {
            let path = library_root.join(directory);
            std::fs::create_dir_all(&path).expect("create protected directory");
            std::fs::write(path.join("keep.txt"), b"preserve this").expect("write protected file");
        }

        let clear = super::clear_library_cache(library_root.display().to_string());

        assert!(clear.ok);
        match response_data(&clear) {
            super::DesktopCommandData::CacheClear {
                cleared_directories,
                removed_cache_records,
                ..
            } => {
                assert_eq!(
                    cleared_directories,
                    &vec![
                        "thumbnails".to_string(),
                        "previews".to_string(),
                        "render-cache".to_string(),
                        "ai-cache".to_string()
                    ]
                );
                assert_eq!(*removed_cache_records, 1);
            }
            other => panic!("unexpected response data: {other:?}"),
        }
        for directory in ["thumbnails", "previews", "render-cache", "ai-cache"] {
            assert!(library_root.join(directory).is_dir());
            assert!(!library_root.join(directory).join("sentinel.cache").exists());
        }
        for directory in ["sidecars", "exports", "logs", "backups"] {
            assert!(library_root.join(directory).join("keep.txt").is_file());
        }
        assert_eq!(
            std::fs::read(&supported_file).expect("read original after"),
            original_bytes
        );

        remove_library_root(&workspace);
    }

    #[test]
    fn desktop_connected_runtime_smoke() {
        let Some(fixtures_root) =
            std::env::var_os("SILICARAW_RUNTIME_SMOKE_FIXTURES").map(PathBuf::from)
        else {
            eprintln!("skipping desktop_connected_runtime_smoke; fixture env var is not set");
            return;
        };
        let run_root = std::env::var_os("SILICARAW_RUNTIME_SMOKE_OUTPUT")
            .map(PathBuf::from)
            .unwrap_or_else(|| unique_library_root("desktop-connected-runtime-smoke"));
        let library_root = run_root.join("SilicaRAW Library");
        let import_root = run_root.join("Import Originals");
        let export_root = run_root.join("Exports");
        let session_path = run_root.join("AppConfig").join("app-session.json");
        std::fs::create_dir_all(&import_root).expect("create connected smoke import folder");
        std::fs::create_dir_all(&export_root).expect("create connected smoke export folder");

        assert!(
            fixtures_root.join("fixture-manifest.json").is_file(),
            "connected runtime smoke requires generated legal fixture metadata"
        );
        let primary_original = import_root.join("synthetic-gradient.jpg");
        let secondary_original = import_root.join("synthetic-checker.jpeg");
        let raw_placeholder = import_root.join("blocked-raw.DNG");
        let unsupported_original = import_root.join("notes.txt");
        let recursive_root = import_root.join("Recursive");
        let recursive_original = recursive_root.join("recursive-child.jpg");
        let recursive_unsupported = recursive_root.join("recursive-notes.txt");
        let recursive_hidden = recursive_root.join(".hidden.jpg");
        let recursive_package = recursive_root.join("Archive.photoslibrary");
        let recursive_package_child = recursive_package.join("package-child.jpg");
        std::fs::create_dir_all(&recursive_package).expect("create recursive smoke package");
        std::fs::copy(
            fixtures_root.join("supported/synthetic-gradient.jpg"),
            &primary_original,
        )
        .expect("copy primary JPEG fixture");
        std::fs::copy(
            fixtures_root.join("supported/synthetic-checker.jpeg"),
            &secondary_original,
        )
        .expect("copy secondary JPEG fixture");
        std::fs::copy(
            fixtures_root.join("raw-blocked/blocked-raw.DNG"),
            &raw_placeholder,
        )
        .expect("copy RAW-blocked placeholder");
        std::fs::copy(
            fixtures_root.join("unsupported/notes.txt"),
            &unsupported_original,
        )
        .expect("copy unsupported fixture");
        std::fs::copy(
            fixtures_root.join("supported/synthetic-checker.jpeg"),
            &recursive_original,
        )
        .expect("copy recursive JPEG fixture");
        std::fs::copy(
            fixtures_root.join("unsupported/notes.txt"),
            &recursive_unsupported,
        )
        .expect("copy recursive unsupported fixture");
        std::fs::copy(
            fixtures_root.join("supported/synthetic-gradient.jpg"),
            &recursive_hidden,
        )
        .expect("copy recursive hidden fixture");
        std::fs::copy(
            fixtures_root.join("supported/synthetic-gradient.jpg"),
            &recursive_package_child,
        )
        .expect("copy recursive package fixture");
        let originals = tracked_originals(&[
            primary_original.clone(),
            secondary_original.clone(),
            raw_placeholder.clone(),
            unsupported_original.clone(),
            recursive_original.clone(),
            recursive_unsupported.clone(),
            recursive_hidden.clone(),
            recursive_package_child.clone(),
        ]);

        let created = super::create_library_at_path(
            library_root.display().to_string(),
            Some(session_path.clone()),
        );
        assert!(created.ok, "create library failed: {created:?}");
        assert_eq!(response_data(&created).kind(), "librarySession");
        let opened = super::open_library_at_path(
            library_root.display().to_string(),
            Some(session_path.clone()),
        );
        assert!(opened.ok, "open library failed: {opened:?}");
        let app_session = super::read_app_session_at_path(session_path.clone());
        assert!(app_session.ok, "read app session failed: {app_session:?}");
        match response_data(&app_session) {
            super::DesktopCommandData::AppSession { session, .. } => {
                assert_eq!(session.recents.len(), 1);
                assert_eq!(
                    session.recents[0].root_path,
                    library_root.display().to_string()
                );
                assert_eq!(session.recents[0].available, Some(true));
            }
            other => panic!("unexpected app session data: {other:?}"),
        }
        assert_originals_unchanged(&originals, "create/open library");

        let imported = super::import_folder(
            library_root.display().to_string(),
            import_root.display().to_string(),
            None,
        );
        assert!(imported.ok, "import folder failed: {imported:?}");
        match response_data(&imported) {
            super::DesktopCommandData::ImportSummary {
                scanned_files,
                supported_files,
                unsupported_files,
                originals_unchanged,
                ..
            } => {
                assert_eq!(*scanned_files, 4);
                assert_eq!(*supported_files, 2);
                assert_eq!(*unsupported_files, 2);
                assert!(*originals_unchanged);
            }
            other => panic!("unexpected import response data: {other:?}"),
        }
        assert_originals_unchanged(&originals, "import by reference");

        let grid = super::list_library_photos(library_root.display().to_string());
        assert!(grid.ok, "list library photos failed: {grid:?}");
        let (photo_id, raw_photo_id) = match response_data(&grid) {
            super::DesktopCommandData::PhotoGrid { photos } => {
                assert_eq!(photos.len(), 4);
                let primary = photos
                    .iter()
                    .find(|photo| photo.file_name == "synthetic-gradient.jpg")
                    .expect("primary JPEG grid row");
                assert!(!primary.unsupported);
                assert!(primary.thumbnail_path.is_some());
                assert!(primary
                    .thumbnail_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
                let raw = photos
                    .iter()
                    .find(|photo| photo.file_name == "blocked-raw.DNG")
                    .expect("RAW-blocked grid row");
                assert!(raw.unsupported);
                let unsupported = photos
                    .iter()
                    .find(|photo| photo.file_name == "notes.txt")
                    .expect("unsupported grid row");
                assert!(unsupported.unsupported);
                (primary.photo_id.clone(), raw.photo_id.clone())
            }
            other => panic!("unexpected grid response data: {other:?}"),
        };
        assert_originals_unchanged(&originals, "grid thumbnail generation");

        let paged = super::query_library_photos(
            library_root.display().to_string(),
            super::DesktopLibraryQueryRequest {
                offset: 0,
                limit: 2,
                sort: "file_name_asc".to_string(),
                filters: super::DesktopLibraryQueryFilters::default(),
            },
        );
        assert!(paged.ok, "paged grid failed: {paged:?}");
        match response_data(&paged) {
            super::DesktopCommandData::PhotoGridPage {
                photos,
                total_count,
                has_next_page,
                ..
            } => {
                assert_eq!(*total_count, 4);
                assert_eq!(photos.len(), 2);
                assert!(*has_next_page);
            }
            other => panic!("unexpected paged grid data: {other:?}"),
        }

        let metadata =
            super::get_photo_metadata(library_root.display().to_string(), photo_id.clone());
        assert!(metadata.ok, "metadata query failed: {metadata:?}");
        match response_data(&metadata) {
            super::DesktopCommandData::PhotoMetadata {
                width,
                height,
                camera_make,
                ..
            } => {
                assert_eq!(width.state, "known");
                assert!(width.value.is_some_and(|value| value > 0));
                assert_eq!(height.state, "known");
                assert!(height.value.is_some_and(|value| value > 0));
                assert_eq!(camera_make.state, "unavailable");
            }
            other => panic!("unexpected metadata data: {other:?}"),
        }

        let picked = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            5,
            true,
            false,
            None,
        );
        assert!(picked.ok, "pick update failed: {picked:?}");
        let rejected = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            3,
            false,
            true,
            None,
        );
        assert!(rejected.ok, "reject update failed: {rejected:?}");
        let final_flags = super::set_photo_flags(
            library_root.display().to_string(),
            photo_id.clone(),
            4,
            true,
            false,
            Some("green".to_string()),
        );
        assert!(
            final_flags.ok,
            "final culling update failed: {final_flags:?}"
        );
        match response_data(&final_flags) {
            super::DesktopCommandData::PhotoFlags {
                rating,
                picked,
                rejected,
                color_label,
                ..
            } => {
                assert_eq!(*rating, 4);
                assert!(*picked);
                assert!(!*rejected);
                assert_eq!(color_label.as_deref(), Some("green"));
            }
            other => panic!("unexpected final flags response data: {other:?}"),
        }
        assert_originals_unchanged(&originals, "rating pick reject");

        let loupe = super::open_photo_preview(library_root.display().to_string(), photo_id.clone());
        assert!(loupe.ok, "loupe preview failed: {loupe:?}");
        match response_data(&loupe) {
            super::DesktopCommandData::PhotoPreview {
                status,
                source_path,
                preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(source_path, &primary_original.display().to_string());
                assert!(preview_bytes.as_ref().is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected loupe response data: {other:?}"),
        }
        let raw_preview =
            super::open_photo_preview(library_root.display().to_string(), raw_photo_id.clone());
        assert!(
            raw_preview.ok,
            "RAW-blocked preview command failed: {raw_preview:?}"
        );
        match response_data(&raw_preview) {
            super::DesktopCommandData::PhotoPreview {
                status,
                preview_bytes,
                message,
                ..
            } => {
                assert_eq!(*status, "Unsupported");
                assert!(preview_bytes.is_none());
                assert!(message.contains("Unsupported file type"));
            }
            other => panic!("unexpected RAW preview response data: {other:?}"),
        }
        assert_originals_unchanged(&originals, "loupe preview");

        let develop_preview = super::preview_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.4,
            12.0,
        );
        assert!(
            develop_preview.ok,
            "develop preview failed: {develop_preview:?}"
        );
        match response_data(&develop_preview) {
            super::DesktopCommandData::EditPreview {
                status,
                exposure,
                contrast,
                develop_preview_bytes,
                ..
            } => {
                assert_eq!(*status, "Ready");
                assert_eq!(*exposure, 0.4);
                assert_eq!(*contrast, 12.0);
                assert!(develop_preview_bytes
                    .as_ref()
                    .is_some_and(|bytes| bytes.len() > 2));
            }
            other => panic!("unexpected develop preview response data: {other:?}"),
        }
        let committed = super::commit_exposure_contrast_edit(
            library_root.display().to_string(),
            photo_id.clone(),
            0.4,
            12.0,
        );
        assert!(committed.ok, "commit edit failed: {committed:?}");
        assert_originals_unchanged(&originals, "develop edit preview and commit");

        let output_path = export_root.join("synthetic-gradient-export.jpg");
        let exported = super::export_photo_jpeg_srgb(
            library_root.display().to_string(),
            photo_id.clone(),
            output_path.display().to_string(),
        );
        assert!(exported.ok, "JPEG sRGB export failed: {exported:?}");
        match response_data(&exported) {
            super::DesktopCommandData::Export {
                source_path,
                output_path: actual_output_path,
                format,
                color_profile,
                bytes_written,
                ..
            } => {
                assert_eq!(source_path, &primary_original.display().to_string());
                assert_eq!(actual_output_path, &output_path.display().to_string());
                assert_ne!(source_path, actual_output_path);
                assert_eq!(format, "jpeg");
                assert_eq!(color_profile, "srgb");
                assert!(*bytes_written > 0);
            }
            other => panic!("unexpected export response data: {other:?}"),
        }
        assert!(output_path.is_file());
        assert_originals_unchanged(&originals, "JPEG sRGB export");

        let cleared = super::clear_library_cache(library_root.display().to_string());
        assert!(cleared.ok, "cache clear failed: {cleared:?}");
        match response_data(&cleared) {
            super::DesktopCommandData::CacheClear {
                cleared_directories,
                recreated_directories,
                removed_cache_records,
                ..
            } => {
                assert_eq!(
                    cleared_directories,
                    &vec![
                        "thumbnails".to_string(),
                        "previews".to_string(),
                        "render-cache".to_string(),
                        "ai-cache".to_string()
                    ]
                );
                assert_eq!(cleared_directories, recreated_directories);
                assert!(*removed_cache_records > 0);
            }
            other => panic!("unexpected cache clear response data: {other:?}"),
        }
        assert_originals_unchanged(&originals, "cache clear");

        let recorded_selection = super::record_app_session_selection_at_path(
            session_path.clone(),
            library_root.display().to_string(),
            Some(photo_id.clone()),
            "develop".to_string(),
        );
        assert!(
            recorded_selection.ok,
            "record session selection failed: {recorded_selection:?}"
        );
        let restored_launch = super::resolve_launch_restore_at_path(session_path.clone());
        assert!(
            restored_launch.ok,
            "launch restore failed: {restored_launch:?}"
        );
        match response_data(&restored_launch) {
            super::DesktopCommandData::LaunchRestore {
                status,
                state,
                selected_photo_id,
                selected_photo_status,
                requested_mode,
                resolved_mode,
                ..
            } => {
                assert_eq!(status, "restored");
                assert_eq!(state, "library");
                assert_eq!(selected_photo_id.as_deref(), Some(photo_id.as_str()));
                assert_eq!(selected_photo_status, "restored");
                assert_eq!(requested_mode, "develop");
                assert_eq!(resolved_mode, "develop");
            }
            other => panic!("unexpected launch restore data: {other:?}"),
        }

        let missing_session_path = run_root.join("AppConfig").join("missing-session.json");
        let mut missing_session = super::DesktopAppSession::default();
        missing_session.last_library_root_path =
            Some(run_root.join("Missing Library").display().to_string());
        let written_missing =
            super::write_app_session_at_path(missing_session_path.clone(), missing_session);
        assert!(
            written_missing.ok,
            "write missing fallback session failed: {written_missing:?}"
        );
        let missing_restore = super::resolve_launch_restore_at_path(missing_session_path);
        assert!(
            missing_restore.ok,
            "missing restore failed: {missing_restore:?}"
        );
        match response_data(&missing_restore) {
            super::DesktopCommandData::LaunchRestore {
                status,
                state,
                selected_photo_status,
                fallback_reason,
                ..
            } => {
                assert_eq!(status, "missingLibrary");
                assert_eq!(state, "welcome");
                assert_eq!(selected_photo_status, "none");
                assert_eq!(fallback_reason.as_deref(), Some("missingLibrary"));
            }
            other => panic!("unexpected missing restore data: {other:?}"),
        }

        let recursive_import = super::import_folder(
            library_root.display().to_string(),
            import_root.display().to_string(),
            Some(true),
        );
        assert!(
            recursive_import.ok,
            "recursive import failed: {recursive_import:?}"
        );
        match response_data(&recursive_import) {
            super::DesktopCommandData::ImportSummary {
                scanned_files,
                supported_files,
                unsupported_files,
                issues,
                ..
            } => {
                assert_eq!(*scanned_files, 6);
                assert_eq!(*supported_files, 3);
                assert_eq!(*unsupported_files, 3);
                assert!(issues.iter().any(|issue| {
                    issue.kind == "unsupported_file"
                        && issue.file_name == Some("recursive-notes.txt".to_string())
                }));
                assert!(issues.iter().any(|issue| {
                    issue.kind == "hidden_entry_skipped"
                        && issue.file_name == Some(".hidden.jpg".to_string())
                }));
                assert!(issues.iter().any(|issue| {
                    issue.kind == "package_directory_skipped"
                        && issue.file_name == Some("Archive.photoslibrary".to_string())
                }));
            }
            other => panic!("unexpected recursive import data: {other:?}"),
        }
        assert_originals_unchanged(&originals, "recursive import");

        let reopened = super::open_library_at_path(library_root.display().to_string(), None);
        assert!(reopened.ok, "reopen library failed: {reopened:?}");
        let restored_flags =
            super::get_photo_flags(library_root.display().to_string(), photo_id.clone());
        assert!(
            restored_flags.ok,
            "restore flags failed: {restored_flags:?}"
        );
        match response_data(&restored_flags) {
            super::DesktopCommandData::PhotoFlags {
                rating,
                picked,
                rejected,
                color_label,
                ..
            } => {
                assert_eq!(*rating, 4);
                assert!(*picked);
                assert!(!*rejected);
                assert_eq!(color_label.as_deref(), Some("green"));
            }
            other => panic!("unexpected restored flags response data: {other:?}"),
        }
        let restored_edit =
            super::get_photo_edit_state(library_root.display().to_string(), photo_id);
        assert!(
            restored_edit.ok,
            "restore edit state failed: {restored_edit:?}"
        );
        match response_data(&restored_edit) {
            super::DesktopCommandData::EditState {
                exposure,
                contrast,
                persisted,
                ..
            } => {
                assert_eq!(*exposure, 0.4);
                assert_eq!(*contrast, 12.0);
                assert!(*persisted);
            }
            other => panic!("unexpected restored edit response data: {other:?}"),
        }
        assert_originals_unchanged(&originals, "library reopen");
        eprintln!("phase-11 connected runtime smoke complete");
    }

    fn response_data(response: &super::DesktopCommandResponse) -> &super::DesktopCommandData {
        response.data.as_ref().expect("response data")
    }

    fn stable_catalog_id(prefix: &str, value: &str) -> String {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in value.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        format!("{prefix}-{hash:016x}")
    }

    fn unique_library_root(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "silicaraw-desktop-library-{label}-{}-{nanos}",
            std::process::id()
        ))
    }

    fn remove_library_root(path: &Path) {
        let _ = std::fs::remove_dir_all(path);
    }

    fn tracked_originals(paths: &[PathBuf]) -> Vec<(PathBuf, Vec<u8>)> {
        paths
            .iter()
            .map(|path| {
                (
                    path.clone(),
                    std::fs::read(path).expect("read original fixture bytes"),
                )
            })
            .collect()
    }

    fn assert_originals_unchanged(originals: &[(PathBuf, Vec<u8>)], stage: &str) {
        for (path, expected) in originals {
            assert_eq!(
                std::fs::read(path).expect("read original fixture for comparison"),
                *expected,
                "original fixture changed after {stage}: {}",
                path.display()
            );
        }
    }

    fn write_source_jpeg(path: &Path) {
        let image = image::RgbImage::from_fn(2, 2, |x, y| {
            if (x + y) % 2 == 0 {
                image::Rgb([64, 128, 192])
            } else {
                image::Rgb([192, 128, 64])
            }
        });
        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write source jpeg");
    }

    fn write_geometry_source_jpeg(path: &Path) {
        let image = image::RgbImage::from_fn(4, 3, |x, y| {
            image::Rgb([
                (32 + (x * 40)) as u8,
                (48 + (y * 50)) as u8,
                (96 + ((x + y) * 10)) as u8,
            ])
        });
        image
            .save_with_format(path, image::ImageFormat::Jpeg)
            .expect("write geometry source jpeg");
    }
}
