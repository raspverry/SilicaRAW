use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use super::current_timestamp_string;
use super::CoreError;

/// App-level desktop session schema identifier.
pub const APP_SESSION_SCHEMA: &str = "silica.desktop_session";
/// App-level desktop session schema version.
pub const APP_SESSION_VERSION: i64 = 1;
/// Default Library grid thumbnail size preference in pixels.
pub const DEFAULT_APP_SESSION_THUMBNAIL_SIZE: u16 = 168;
/// Minimum accepted Library grid thumbnail size preference in pixels.
pub const MIN_APP_SESSION_THUMBNAIL_SIZE: u16 = 132;
/// Maximum accepted Library grid thumbnail size preference in pixels.
pub const MAX_APP_SESSION_THUMBNAIL_SIZE: u16 = 220;
/// Default UI scale preference in percent.
pub const DEFAULT_APP_SESSION_UI_SCALE: u16 = 100;
/// Minimum accepted UI scale preference in percent.
pub const MIN_APP_SESSION_UI_SCALE: u16 = 90;
/// Maximum accepted UI scale preference in percent.
pub const MAX_APP_SESSION_UI_SCALE: u16 = 120;
/// Maximum number of recent libraries retained in app-level session state.
pub const APP_SESSION_RECENTS_LIMIT: usize = 10;

/// Last active desktop mode persisted outside every library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSessionMode {
    Library,
    Develop,
    Export,
}

/// Library grid sort order persisted in app-level session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLibrarySort {
    ImportedAtDesc,
    FileNameAsc,
    RatingDesc,
}

/// Optional file-type filter persisted in app-level session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppFileTypeFilter {
    Jpeg,
    Png,
    Tiff,
    Raw,
    Unsupported,
}

/// Optional metadata-backed filter persisted in app-level session state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMetadataFilter {
    HasDimensions,
}

/// Supported app theme preferences for the current tokenized shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAppearanceTheme {
    Dark,
    Light,
}

/// Supported app density preferences for the current tokenized shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAppearanceDensity {
    Compact,
    Comfortable,
}

/// App-level appearance preferences persisted outside every library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppAppearancePreferences {
    pub theme: AppAppearanceTheme,
    pub density: AppAppearanceDensity,
    pub ui_scale: u16,
}

impl Default for AppAppearancePreferences {
    fn default() -> Self {
        Self {
            theme: AppAppearanceTheme::Dark,
            density: AppAppearanceDensity::Compact,
            ui_scale: DEFAULT_APP_SESSION_UI_SCALE,
        }
    }
}

/// Library storage preferences persisted outside every library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLibraryPreferences {
    pub default_library_root_path: Option<PathBuf>,
}

impl Default for AppLibraryPreferences {
    fn default() -> Self {
        Self {
            default_library_root_path: None,
        }
    }
}

/// Library grid filters persisted in app-level session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionFilters {
    pub min_rating: Option<u8>,
    pub picked: Option<bool>,
    pub rejected: Option<bool>,
    pub file_type: Option<AppFileTypeFilter>,
    pub metadata: Option<AppMetadataFilter>,
    pub search: String,
}

impl Default for AppSessionFilters {
    fn default() -> Self {
        Self {
            min_rating: None,
            picked: None,
            rejected: None,
            file_type: None,
            metadata: None,
            search: String::new(),
        }
    }
}

/// Workspace layout preferences persisted in app-level session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLayoutPreferences {
    pub sidebar_collapsed: bool,
    pub inspector_collapsed: bool,
    pub filmstrip_visible: bool,
    pub thumbnail_size: u16,
    pub sort: AppLibrarySort,
    pub filters: AppSessionFilters,
}

impl Default for AppLayoutPreferences {
    fn default() -> Self {
        Self {
            sidebar_collapsed: false,
            inspector_collapsed: false,
            filmstrip_visible: true,
            thumbnail_size: DEFAULT_APP_SESSION_THUMBNAIL_SIZE,
            sort: AppLibrarySort::ImportedAtDesc,
            filters: AppSessionFilters::default(),
        }
    }
}

/// One recent library entry persisted after successful create/open only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRecentLibrary {
    pub root_path: PathBuf,
    pub display_name: String,
    pub last_opened_at: String,
}

/// Per-library session state keyed by library root path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPerLibrarySession {
    pub selected_photo_id: Option<String>,
    pub last_mode: AppSessionMode,
    pub last_opened_at: String,
}

impl Default for AppPerLibrarySession {
    fn default() -> Self {
        Self {
            selected_photo_id: None,
            last_mode: AppSessionMode::Library,
            last_opened_at: String::new(),
        }
    }
}

/// Versioned app-level desktop session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSession {
    pub schema: String,
    pub version: i64,
    pub last_library_root_path: Option<PathBuf>,
    pub last_mode: AppSessionMode,
    pub recents: Vec<AppRecentLibrary>,
    pub appearance: AppAppearancePreferences,
    pub library: AppLibraryPreferences,
    pub layout: AppLayoutPreferences,
    pub per_library: BTreeMap<String, AppPerLibrarySession>,
}

impl Default for AppSession {
    fn default() -> Self {
        Self {
            schema: APP_SESSION_SCHEMA.to_string(),
            version: APP_SESSION_VERSION,
            last_library_root_path: None,
            last_mode: AppSessionMode::Library,
            recents: Vec::new(),
            appearance: AppAppearancePreferences::default(),
            library: AppLibraryPreferences::default(),
            layout: AppLayoutPreferences::default(),
            per_library: BTreeMap::new(),
        }
    }
}

/// Non-fatal app-session load warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppSessionWarning {
    Missing,
    Corrupt,
    UnsupportedVersion,
    InvalidValues,
}

/// Result of loading app-level desktop session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionLoadResult {
    pub session: AppSession,
    pub warnings: Vec<AppSessionWarning>,
}

/// Result of atomically writing app-level desktop session state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionWriteResult {
    pub session_path: PathBuf,
    pub bytes_written: u64,
}

/// Relaunch restore state after validating the last app-session library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSessionRestoreStatus {
    NoLastLibrary,
    MissingLibrary,
    MissingCatalog,
    InvalidCatalog,
    Restored,
}

/// Selected-photo restore outcome for the last library.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppSessionSelectedPhotoStatus {
    None,
    Missing,
    Restored,
}

/// Relaunch restore plan that does not create, migrate, import, or repair libraries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSessionRestorePlan {
    pub session: AppSession,
    pub warnings: Vec<AppSessionWarning>,
    pub status: AppSessionRestoreStatus,
    pub library_root_path: Option<PathBuf>,
    pub catalog_path: Option<PathBuf>,
    pub schema_version: Option<i64>,
    pub selected_photo_id: Option<String>,
    pub selected_photo_status: AppSessionSelectedPhotoStatus,
    pub requested_mode: AppSessionMode,
    pub resolved_mode: AppSessionMode,
}

/// Local library session returned by core commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibrarySession {
    pub root_path: PathBuf,
    pub catalog_path: PathBuf,
    pub schema_version: i64,
}

impl LibrarySession {
    /// Compact status string for the minimal desktop shell entry point.
    pub fn status_text(&self) -> String {
        format!(
            "Library: {}\nCatalog: {}\nSchema: {}",
            self.root_path.display(),
            self.catalog_path.display(),
            self.schema_version
        )
    }
}

impl From<silica_storage::LocalLibrary> for LibrarySession {
    fn from(library: silica_storage::LocalLibrary) -> Self {
        Self {
            root_path: library.root_path,
            catalog_path: library.catalog_path,
            schema_version: library.schema_version,
        }
    }
}

/// Load app-level desktop session state from a caller-provided path.
pub fn load_app_session(session_path: impl AsRef<Path>) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    if !session_path.exists() {
        return Ok(AppSessionLoadResult {
            session: AppSession::default(),
            warnings: vec![AppSessionWarning::Missing],
        });
    }

    let bytes = std::fs::read(session_path).map_err(|error| {
        CoreError::AppSession(format!("read {}: {error}", session_path.display()))
    })?;
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(_) => {
            return Ok(AppSessionLoadResult {
                session: AppSession::default(),
                warnings: vec![AppSessionWarning::Corrupt],
            });
        }
    };
    let Some(object) = value.as_object() else {
        return Ok(AppSessionLoadResult {
            session: AppSession::default(),
            warnings: vec![AppSessionWarning::Corrupt],
        });
    };

    if object.get("schema").and_then(serde_json::Value::as_str) != Some(APP_SESSION_SCHEMA)
        || object.get("version").and_then(serde_json::Value::as_i64) != Some(APP_SESSION_VERSION)
    {
        return Ok(AppSessionLoadResult {
            session: AppSession::default(),
            warnings: vec![AppSessionWarning::UnsupportedVersion],
        });
    }

    let mut invalid_values = false;
    let session = AppSession {
        schema: APP_SESSION_SCHEMA.to_string(),
        version: APP_SESSION_VERSION,
        last_library_root_path: parse_optional_path(
            object.get("last_library_root_path"),
            &mut invalid_values,
        ),
        last_mode: parse_app_session_mode(object.get("last_mode"), &mut invalid_values),
        recents: parse_app_session_recents(object.get("recents"), &mut invalid_values),
        appearance: parse_app_appearance(object.get("appearance"), &mut invalid_values),
        library: parse_app_library_preferences(object.get("library"), &mut invalid_values),
        layout: parse_app_layout(object.get("layout"), &mut invalid_values),
        per_library: parse_app_per_library(object.get("per_library"), &mut invalid_values),
    };

    let warnings = if invalid_values {
        vec![AppSessionWarning::InvalidValues]
    } else {
        Vec::new()
    };

    Ok(AppSessionLoadResult { session, warnings })
}

/// Atomically write app-level desktop session state to a caller-provided path.
pub fn write_app_session(
    session_path: impl AsRef<Path>,
    session: &AppSession,
) -> Result<AppSessionWriteResult, CoreError> {
    let session_path = session_path.as_ref();
    if let Some(parent) = session_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            CoreError::AppSession(format!("create {}: {error}", parent.display()))
        })?;
    }

    let bytes = serde_json::to_vec_pretty(&app_session_to_json(session))
        .map_err(|error| CoreError::AppSession(format!("serialize app session: {error}")))?;
    let temp_path = session_path.with_extension("tmp");
    std::fs::write(&temp_path, &bytes).map_err(|error| {
        CoreError::AppSession(format!("write {}: {error}", temp_path.display()))
    })?;
    std::fs::rename(&temp_path, session_path).map_err(|error| {
        CoreError::AppSession(format!(
            "rename {} to {}: {error}",
            temp_path.display(),
            session_path.display()
        ))
    })?;

    Ok(AppSessionWriteResult {
        session_path: session_path.to_path_buf(),
        bytes_written: bytes.len() as u64,
    })
}

/// Return the documented default workspace layout preferences.
pub fn default_app_layout_preferences() -> AppLayoutPreferences {
    AppLayoutPreferences::default()
}

/// Return the documented default appearance preferences.
pub fn default_app_appearance_preferences() -> AppAppearancePreferences {
    AppAppearancePreferences::default()
}

/// Return the documented default library preferences.
pub fn default_app_library_preferences() -> AppLibraryPreferences {
    AppLibraryPreferences::default()
}

/// Record a successful library create/open in app-level desktop session state.
pub fn record_app_session_recent_library(
    session_path: impl AsRef<Path>,
    library: &LibrarySession,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    let recent_key = app_session_recent_key(&library.root_path);
    let opened_at = current_timestamp_string();
    session.last_library_root_path = Some(library.root_path.clone());
    session
        .recents
        .retain(|recent| app_session_recent_key(&recent.root_path) != recent_key);
    session.recents.insert(
        0,
        AppRecentLibrary {
            root_path: library.root_path.clone(),
            display_name: app_session_library_display_name(&library.root_path),
            last_opened_at: opened_at,
        },
    );
    session.recents.truncate(APP_SESSION_RECENTS_LIMIT);

    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Reset only workspace layout preferences in app-level desktop session state.
pub fn reset_app_session_layout(
    session_path: impl AsRef<Path>,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.layout = default_app_layout_preferences();
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Reset only app appearance preferences in app-level desktop session state.
pub fn reset_app_session_appearance(
    session_path: impl AsRef<Path>,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.appearance = default_app_appearance_preferences();
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Reset only library storage preferences in app-level desktop session state.
pub fn reset_app_session_library_preferences(
    session_path: impl AsRef<Path>,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.library = default_app_library_preferences();
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Record workspace layout preferences in app-level desktop session state.
pub fn record_app_session_layout(
    session_path: impl AsRef<Path>,
    layout: AppLayoutPreferences,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.layout = layout;
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Record app appearance preferences in app-level desktop session state.
pub fn record_app_session_appearance(
    session_path: impl AsRef<Path>,
    appearance: AppAppearancePreferences,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.appearance = appearance;
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Record library storage preferences in app-level desktop session state.
pub fn record_app_session_library_preferences(
    session_path: impl AsRef<Path>,
    library: AppLibraryPreferences,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    session.library = library;
    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

/// Plan app relaunch restore from app-session state without opening a writable library.
/// Older catalogs are restorable so the first grid query can run the required migration.
pub fn plan_app_session_restore(
    session_path: impl AsRef<Path>,
) -> Result<AppSessionRestorePlan, CoreError> {
    let loaded = load_app_session(session_path)?;
    let requested_mode = loaded.session.last_mode;
    let Some(last_library_root_path) = loaded.session.last_library_root_path.clone() else {
        return Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::NoLastLibrary,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        });
    };

    match silica_storage::inspect_local_library_for_restore(&last_library_root_path) {
        Ok(library) => {
            let status = if library.schema_version <= silica_storage::CURRENT_SCHEMA_VERSION {
                AppSessionRestoreStatus::Restored
            } else {
                AppSessionRestoreStatus::InvalidCatalog
            };
            let restored = status == AppSessionRestoreStatus::Restored;
            let per_library = restored
                .then(|| app_session_recent_key(&library.root_path))
                .and_then(|key| loaded.session.per_library.get(&key));
            let requested_mode = per_library
                .map(|session| session.last_mode)
                .unwrap_or(requested_mode);
            let selected_candidate =
                per_library.and_then(|session| session.selected_photo_id.clone());
            let (selected_photo_id, selected_photo_status) =
                if let Some(photo_id) = selected_candidate {
                    match silica_storage::catalog_photo_exists_for_restore(
                        &library.root_path,
                        &photo_id,
                    ) {
                        Ok(true) => (Some(photo_id), AppSessionSelectedPhotoStatus::Restored),
                        Ok(false) => (None, AppSessionSelectedPhotoStatus::Missing),
                        Err(_) => {
                            return Ok(AppSessionRestorePlan {
                                session: loaded.session,
                                warnings: loaded.warnings,
                                status: AppSessionRestoreStatus::InvalidCatalog,
                                library_root_path: None,
                                catalog_path: None,
                                schema_version: None,
                                selected_photo_id: None,
                                selected_photo_status: AppSessionSelectedPhotoStatus::None,
                                requested_mode,
                                resolved_mode: AppSessionMode::Library,
                            })
                        }
                    }
                } else {
                    (None, AppSessionSelectedPhotoStatus::None)
                };
            let resolved_mode = if requested_mode == AppSessionMode::Library
                || selected_photo_status == AppSessionSelectedPhotoStatus::Restored
            {
                requested_mode
            } else {
                AppSessionMode::Library
            };
            Ok(AppSessionRestorePlan {
                session: loaded.session,
                warnings: loaded.warnings,
                status,
                library_root_path: restored.then_some(library.root_path),
                catalog_path: restored.then_some(library.catalog_path),
                schema_version: restored.then_some(library.schema_version),
                selected_photo_id,
                selected_photo_status,
                requested_mode,
                resolved_mode,
            })
        }
        Err(silica_storage::LibraryStorageError::NotDirectory(_)) => Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::MissingLibrary,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        }),
        Err(silica_storage::LibraryStorageError::MissingCatalog(_)) => Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::MissingCatalog,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        }),
        Err(_) => Ok(AppSessionRestorePlan {
            session: loaded.session,
            warnings: loaded.warnings,
            status: AppSessionRestoreStatus::InvalidCatalog,
            library_root_path: None,
            catalog_path: None,
            schema_version: None,
            selected_photo_id: None,
            selected_photo_status: AppSessionSelectedPhotoStatus::None,
            requested_mode,
            resolved_mode: AppSessionMode::Library,
        }),
    }
}

/// Record the active library selection and mode in app-level desktop session state.
pub fn record_app_session_library_state(
    session_path: impl AsRef<Path>,
    library_root_path: impl AsRef<Path>,
    selected_photo_id: Option<String>,
    mode: AppSessionMode,
) -> Result<AppSessionLoadResult, CoreError> {
    let session_path = session_path.as_ref();
    let loaded = load_app_session(session_path)?;
    let mut warnings = loaded.warnings;
    if warnings.as_slice() == [AppSessionWarning::Missing] {
        warnings.clear();
    }

    let mut session = loaded.session;
    let library_root_path = library_root_path.as_ref().to_path_buf();
    let key = app_session_recent_key(&library_root_path);
    let opened_at = current_timestamp_string();
    session.last_library_root_path = Some(library_root_path);
    session.last_mode = mode;
    session.per_library.insert(
        key,
        AppPerLibrarySession {
            selected_photo_id,
            last_mode: mode,
            last_opened_at: opened_at,
        },
    );

    write_app_session(session_path, &session)?;

    Ok(AppSessionLoadResult { session, warnings })
}

fn app_session_to_json(session: &AppSession) -> serde_json::Value {
    let recents = session
        .recents
        .iter()
        .map(|recent| {
            serde_json::json!({
                "root_path": recent.root_path.display().to_string(),
                "display_name": recent.display_name,
                "last_opened_at": recent.last_opened_at,
            })
        })
        .collect::<Vec<_>>();
    let per_library = session
        .per_library
        .iter()
        .map(|(key, value)| {
            (
                key.clone(),
                serde_json::json!({
                    "selected_photo_id": value.selected_photo_id,
                    "last_mode": app_session_mode_string(value.last_mode),
                    "last_opened_at": value.last_opened_at,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();

    serde_json::json!({
        "schema": APP_SESSION_SCHEMA,
        "version": APP_SESSION_VERSION,
        "last_library_root_path": session.last_library_root_path.as_ref().map(|path| path.display().to_string()),
        "last_mode": app_session_mode_string(session.last_mode),
        "recents": recents,
        "appearance": {
            "theme": app_appearance_theme_string(session.appearance.theme),
            "density": app_appearance_density_string(session.appearance.density),
            "ui_scale": session.appearance.ui_scale,
        },
        "library": {
            "default_library_root_path": session.library.default_library_root_path.as_ref().map(|path| path.display().to_string()),
        },
        "layout": {
            "sidebar_collapsed": session.layout.sidebar_collapsed,
            "inspector_collapsed": session.layout.inspector_collapsed,
            "filmstrip_visible": session.layout.filmstrip_visible,
            "thumbnail_size": session.layout.thumbnail_size,
            "sort": app_library_sort_string(session.layout.sort),
            "filters": {
                "min_rating": session.layout.filters.min_rating,
                "picked": session.layout.filters.picked,
                "rejected": session.layout.filters.rejected,
                "file_type": session.layout.filters.file_type.map(app_file_type_filter_string),
                "metadata": session.layout.filters.metadata.map(app_metadata_filter_string),
                "search": session.layout.filters.search,
            }
        },
        "per_library": per_library,
    })
}

fn parse_optional_path(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<PathBuf> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_str() {
            Some(path) => Some(PathBuf::from(path)),
            None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn parse_app_session_mode(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppSessionMode {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("library") => AppSessionMode::Library,
        Some("develop") => AppSessionMode::Develop,
        Some("export") => AppSessionMode::Export,
        Some(_) => {
            *invalid_values = true;
            AppSessionMode::Library
        }
    }
}

fn app_session_mode_string(mode: AppSessionMode) -> &'static str {
    match mode {
        AppSessionMode::Library => "library",
        AppSessionMode::Develop => "develop",
        AppSessionMode::Export => "export",
    }
}

fn parse_app_library_sort(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppLibrarySort {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("imported_at_desc") => AppLibrarySort::ImportedAtDesc,
        Some("file_name_asc") => AppLibrarySort::FileNameAsc,
        Some("rating_desc") => AppLibrarySort::RatingDesc,
        Some(_) => {
            *invalid_values = true;
            AppLibrarySort::ImportedAtDesc
        }
    }
}

fn app_library_sort_string(sort: AppLibrarySort) -> &'static str {
    match sort {
        AppLibrarySort::ImportedAtDesc => "imported_at_desc",
        AppLibrarySort::FileNameAsc => "file_name_asc",
        AppLibrarySort::RatingDesc => "rating_desc",
    }
}

fn parse_app_file_type_filter(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<AppFileTypeFilter> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_str() {
            Some("jpeg") => Some(AppFileTypeFilter::Jpeg),
            Some("png") => Some(AppFileTypeFilter::Png),
            Some("tiff") => Some(AppFileTypeFilter::Tiff),
            Some("raw") => Some(AppFileTypeFilter::Raw),
            Some("unsupported") => Some(AppFileTypeFilter::Unsupported),
            Some(_) | None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn app_file_type_filter_string(filter: AppFileTypeFilter) -> &'static str {
    match filter {
        AppFileTypeFilter::Jpeg => "jpeg",
        AppFileTypeFilter::Png => "png",
        AppFileTypeFilter::Tiff => "tiff",
        AppFileTypeFilter::Raw => "raw",
        AppFileTypeFilter::Unsupported => "unsupported",
    }
}

fn parse_app_metadata_filter(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<AppMetadataFilter> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_str() {
            Some("has_dimensions") => Some(AppMetadataFilter::HasDimensions),
            Some(_) | None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn app_metadata_filter_string(filter: AppMetadataFilter) -> &'static str {
    match filter {
        AppMetadataFilter::HasDimensions => "has_dimensions",
    }
}

fn parse_app_appearance_theme(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppAppearanceTheme {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("dark") => AppAppearanceTheme::Dark,
        Some("light") => AppAppearanceTheme::Light,
        Some(_) => {
            *invalid_values = true;
            AppAppearanceTheme::Dark
        }
    }
}

fn app_appearance_theme_string(theme: AppAppearanceTheme) -> &'static str {
    match theme {
        AppAppearanceTheme::Dark => "dark",
        AppAppearanceTheme::Light => "light",
    }
}

fn parse_app_appearance_density(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppAppearanceDensity {
    match value.and_then(serde_json::Value::as_str) {
        None | Some("compact") => AppAppearanceDensity::Compact,
        Some("comfortable") => AppAppearanceDensity::Comfortable,
        Some(_) => {
            *invalid_values = true;
            AppAppearanceDensity::Compact
        }
    }
}

fn app_appearance_density_string(density: AppAppearanceDensity) -> &'static str {
    match density {
        AppAppearanceDensity::Compact => "compact",
        AppAppearanceDensity::Comfortable => "comfortable",
    }
}

fn parse_app_session_recents(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Vec<AppRecentLibrary> {
    let Some(value) = value else {
        return Vec::new();
    };
    let Some(entries) = value.as_array() else {
        *invalid_values = true;
        return Vec::new();
    };

    entries
        .iter()
        .filter_map(|entry| {
            let object = match entry.as_object() {
                Some(object) => object,
                None => {
                    *invalid_values = true;
                    return None;
                }
            };
            let root_path = match object.get("root_path").and_then(serde_json::Value::as_str) {
                Some(root_path) => PathBuf::from(root_path),
                None => {
                    *invalid_values = true;
                    return None;
                }
            };
            let display_name = object
                .get("display_name")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            let last_opened_at = object
                .get("last_opened_at")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();

            Some(AppRecentLibrary {
                root_path,
                display_name,
                last_opened_at,
            })
        })
        .collect()
}

fn parse_app_appearance(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppAppearancePreferences {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return AppAppearancePreferences::default();
    };

    AppAppearancePreferences {
        theme: parse_app_appearance_theme(object.get("theme"), invalid_values),
        density: parse_app_appearance_density(object.get("density"), invalid_values),
        ui_scale: parse_ui_scale(object.get("ui_scale"), invalid_values),
    }
}

fn parse_app_library_preferences(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppLibraryPreferences {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return AppLibraryPreferences::default();
    };

    AppLibraryPreferences {
        default_library_root_path: parse_optional_path(
            object.get("default_library_root_path"),
            invalid_values,
        ),
    }
}

fn parse_app_layout(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppLayoutPreferences {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return AppLayoutPreferences::default();
    };
    let defaults = AppLayoutPreferences::default();

    AppLayoutPreferences {
        sidebar_collapsed: parse_bool_or_default(
            object.get("sidebar_collapsed"),
            defaults.sidebar_collapsed,
            invalid_values,
        ),
        inspector_collapsed: parse_bool_or_default(
            object.get("inspector_collapsed"),
            defaults.inspector_collapsed,
            invalid_values,
        ),
        filmstrip_visible: parse_bool_or_default(
            object.get("filmstrip_visible"),
            defaults.filmstrip_visible,
            invalid_values,
        ),
        thumbnail_size: parse_thumbnail_size(object.get("thumbnail_size"), invalid_values),
        sort: parse_app_library_sort(object.get("sort"), invalid_values),
        filters: parse_app_session_filters(object.get("filters"), invalid_values),
    }
}

fn parse_app_session_filters(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> AppSessionFilters {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return AppSessionFilters::default();
    };

    AppSessionFilters {
        min_rating: parse_min_rating(object.get("min_rating"), invalid_values),
        picked: parse_optional_bool(object.get("picked"), invalid_values),
        rejected: parse_optional_bool(object.get("rejected"), invalid_values),
        file_type: parse_app_file_type_filter(object.get("file_type"), invalid_values),
        metadata: parse_app_metadata_filter(object.get("metadata"), invalid_values),
        search: parse_search_string(object.get("search"), invalid_values),
    }
}

fn parse_app_per_library(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> BTreeMap<String, AppPerLibrarySession> {
    let Some(object) = value.and_then(serde_json::Value::as_object) else {
        if value.is_some() {
            *invalid_values = true;
        }
        return BTreeMap::new();
    };

    object
        .iter()
        .filter_map(|(key, value)| {
            let entry = match value.as_object() {
                Some(entry) => entry,
                None => {
                    *invalid_values = true;
                    return None;
                }
            };
            let selected_photo_id = match entry.get("selected_photo_id") {
                None | Some(serde_json::Value::Null) => None,
                Some(value) => match value.as_str() {
                    Some(photo_id) => Some(photo_id.to_string()),
                    None => {
                        *invalid_values = true;
                        None
                    }
                },
            };
            let last_opened_at = entry
                .get("last_opened_at")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_string();
            Some((
                key.clone(),
                AppPerLibrarySession {
                    selected_photo_id,
                    last_mode: parse_app_session_mode(entry.get("last_mode"), invalid_values),
                    last_opened_at,
                },
            ))
        })
        .collect()
}

fn parse_bool_or_default(
    value: Option<&serde_json::Value>,
    default: bool,
    invalid_values: &mut bool,
) -> bool {
    match value {
        None => default,
        Some(value) => match value.as_bool() {
            Some(value) => value,
            None => {
                *invalid_values = true;
                default
            }
        },
    }
}

fn parse_optional_bool(
    value: Option<&serde_json::Value>,
    invalid_values: &mut bool,
) -> Option<bool> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => match value.as_bool() {
            Some(value) => Some(value),
            None => {
                *invalid_values = true;
                None
            }
        },
    }
}

fn parse_min_rating(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> Option<u8> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(value) => {
            if let Some(value) = value.as_i64() {
                if !(0..=5).contains(&value) {
                    *invalid_values = true;
                }
                Some(value.clamp(0, 5) as u8)
            } else {
                *invalid_values = true;
                None
            }
        }
    }
}

fn parse_search_string(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> String {
    match value {
        None => String::new(),
        Some(value) => match value.as_str() {
            Some(value) => value.to_string(),
            None => {
                *invalid_values = true;
                String::new()
            }
        },
    }
}

fn parse_thumbnail_size(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> u16 {
    let Some(value) = value else {
        return DEFAULT_APP_SESSION_THUMBNAIL_SIZE;
    };
    let Some(value) = value.as_i64() else {
        *invalid_values = true;
        return DEFAULT_APP_SESSION_THUMBNAIL_SIZE;
    };
    if value < MIN_APP_SESSION_THUMBNAIL_SIZE as i64
        || value > MAX_APP_SESSION_THUMBNAIL_SIZE as i64
    {
        *invalid_values = true;
    }
    value.clamp(
        MIN_APP_SESSION_THUMBNAIL_SIZE as i64,
        MAX_APP_SESSION_THUMBNAIL_SIZE as i64,
    ) as u16
}

fn parse_ui_scale(value: Option<&serde_json::Value>, invalid_values: &mut bool) -> u16 {
    let Some(value) = value else {
        return DEFAULT_APP_SESSION_UI_SCALE;
    };
    let Some(value) = value.as_i64() else {
        *invalid_values = true;
        return DEFAULT_APP_SESSION_UI_SCALE;
    };
    if value < MIN_APP_SESSION_UI_SCALE as i64 || value > MAX_APP_SESSION_UI_SCALE as i64 {
        *invalid_values = true;
    }
    value.clamp(
        MIN_APP_SESSION_UI_SCALE as i64,
        MAX_APP_SESSION_UI_SCALE as i64,
    ) as u16
}

fn app_session_recent_key(root_path: &Path) -> String {
    std::fs::canonicalize(root_path)
        .unwrap_or_else(|_| root_path.to_path_buf())
        .display()
        .to_string()
}

fn app_session_library_display_name(root_path: &Path) -> String {
    root_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("SilicaRAW Library")
        .to_string()
}
