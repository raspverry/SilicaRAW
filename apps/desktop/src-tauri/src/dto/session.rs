use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopAppSession {
    pub(crate) schema: String,
    pub(crate) version: i64,
    pub(crate) last_library_root_path: Option<String>,
    pub(crate) last_mode: String,
    pub(crate) recents: Vec<DesktopRecentLibrary>,
    #[serde(default)]
    pub(crate) appearance: DesktopAppearancePreferences,
    #[serde(default)]
    pub(crate) library: DesktopLibraryPreferences,
    pub(crate) layout: DesktopLayoutPreferences,
    pub(crate) per_library: BTreeMap<String, DesktopPerLibrarySession>,
}

impl Default for DesktopAppSession {
    fn default() -> Self {
        Self::from_core(silica_core::AppSession::default())
    }
}

impl DesktopAppSession {
    pub(crate) fn from_core(session: silica_core::AppSession) -> Self {
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

    pub(crate) fn into_core(self) -> Result<silica_core::AppSession, silica_core::CoreError> {
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
pub(crate) struct DesktopRecentLibrary {
    pub(crate) root_path: String,
    pub(crate) display_name: String,
    pub(crate) last_opened_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) available: Option<bool>,
}

impl DesktopRecentLibrary {
    pub(crate) fn from_core(recent: silica_core::AppRecentLibrary) -> Self {
        let available = recent.root_path.join("catalog.db").is_file();
        Self {
            root_path: recent.root_path.display().to_string(),
            display_name: recent.display_name,
            last_opened_at: recent.last_opened_at,
            available: Some(available),
        }
    }

    pub(crate) fn into_core(self) -> silica_core::AppRecentLibrary {
        silica_core::AppRecentLibrary {
            root_path: PathBuf::from(self.root_path),
            display_name: self.display_name,
            last_opened_at: self.last_opened_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopPerLibrarySession {
    pub(crate) selected_photo_id: Option<String>,
    pub(crate) last_mode: String,
    pub(crate) last_opened_at: String,
}

impl DesktopPerLibrarySession {
    pub(crate) fn from_core(session: silica_core::AppPerLibrarySession) -> Self {
        Self {
            selected_photo_id: session.selected_photo_id,
            last_mode: app_session_mode_string(session.last_mode).to_string(),
            last_opened_at: session.last_opened_at,
        }
    }

    pub(crate) fn into_core(
        self,
    ) -> Result<silica_core::AppPerLibrarySession, silica_core::CoreError> {
        Ok(silica_core::AppPerLibrarySession {
            selected_photo_id: self.selected_photo_id,
            last_mode: parse_desktop_app_session_mode(&self.last_mode)?,
            last_opened_at: self.last_opened_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopAppearancePreferences {
    pub(crate) theme: String,
    pub(crate) density: String,
    pub(crate) ui_scale: u16,
}

impl Default for DesktopAppearancePreferences {
    fn default() -> Self {
        Self::from_core(silica_core::AppAppearancePreferences::default())
    }
}

impl DesktopAppearancePreferences {
    pub(crate) fn from_core(appearance: silica_core::AppAppearancePreferences) -> Self {
        Self {
            theme: app_appearance_theme_string(appearance.theme).to_string(),
            density: app_appearance_density_string(appearance.density).to_string(),
            ui_scale: appearance.ui_scale,
        }
    }

    pub(crate) fn into_core(
        self,
    ) -> Result<silica_core::AppAppearancePreferences, silica_core::CoreError> {
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
pub(crate) struct DesktopLibraryPreferences {
    pub(crate) default_library_root_path: Option<String>,
}

impl Default for DesktopLibraryPreferences {
    fn default() -> Self {
        Self::from_core(silica_core::AppLibraryPreferences::default())
    }
}

impl DesktopLibraryPreferences {
    pub(crate) fn from_core(library: silica_core::AppLibraryPreferences) -> Self {
        Self {
            default_library_root_path: library
                .default_library_root_path
                .map(|path| path.display().to_string()),
        }
    }

    pub(crate) fn into_core(self) -> silica_core::AppLibraryPreferences {
        silica_core::AppLibraryPreferences {
            default_library_root_path: self.default_library_root_path.map(PathBuf::from),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopLayoutPreferences {
    pub(crate) sidebar_collapsed: bool,
    pub(crate) inspector_collapsed: bool,
    pub(crate) filmstrip_visible: bool,
    pub(crate) thumbnail_size: u16,
    pub(crate) sort: String,
    pub(crate) filters: DesktopSessionFilters,
}

impl DesktopLayoutPreferences {
    pub(crate) fn from_core(layout: silica_core::AppLayoutPreferences) -> Self {
        Self {
            sidebar_collapsed: layout.sidebar_collapsed,
            inspector_collapsed: layout.inspector_collapsed,
            filmstrip_visible: layout.filmstrip_visible,
            thumbnail_size: layout.thumbnail_size,
            sort: app_library_sort_string(layout.sort).to_string(),
            filters: DesktopSessionFilters::from_core(layout.filters),
        }
    }

    pub(crate) fn into_core(
        self,
    ) -> Result<silica_core::AppLayoutPreferences, silica_core::CoreError> {
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
pub(crate) struct DesktopSessionFilters {
    pub(crate) min_rating: Option<u8>,
    pub(crate) picked: Option<bool>,
    pub(crate) rejected: Option<bool>,
    pub(crate) file_type: Option<String>,
    pub(crate) metadata: Option<String>,
    pub(crate) search: String,
}

impl DesktopSessionFilters {
    pub(crate) fn from_core(filters: silica_core::AppSessionFilters) -> Self {
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

    pub(crate) fn into_core(
        self,
    ) -> Result<silica_core::AppSessionFilters, silica_core::CoreError> {
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

pub(crate) fn app_session_warning_strings(
    warnings: &[silica_core::AppSessionWarning],
) -> Vec<String> {
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

pub(crate) fn app_session_restore_status_string(
    status: silica_core::AppSessionRestoreStatus,
) -> &'static str {
    match status {
        silica_core::AppSessionRestoreStatus::NoLastLibrary => "noLastLibrary",
        silica_core::AppSessionRestoreStatus::MissingLibrary => "missingLibrary",
        silica_core::AppSessionRestoreStatus::MissingCatalog => "missingCatalog",
        silica_core::AppSessionRestoreStatus::InvalidCatalog => "invalidCatalog",
        silica_core::AppSessionRestoreStatus::Restored => "restored",
    }
}

pub(crate) fn app_session_selected_photo_status_string(
    status: silica_core::AppSessionSelectedPhotoStatus,
) -> &'static str {
    match status {
        silica_core::AppSessionSelectedPhotoStatus::None => "none",
        silica_core::AppSessionSelectedPhotoStatus::Missing => "missing",
        silica_core::AppSessionSelectedPhotoStatus::Restored => "restored",
    }
}

pub(crate) fn parse_desktop_app_session_mode(
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

pub(crate) fn app_session_mode_string(mode: silica_core::AppSessionMode) -> &'static str {
    match mode {
        silica_core::AppSessionMode::Library => "library",
        silica_core::AppSessionMode::Develop => "develop",
        silica_core::AppSessionMode::Export => "export",
    }
}

pub(crate) fn parse_desktop_app_library_sort(
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

pub(crate) fn app_library_sort_string(sort: silica_core::AppLibrarySort) -> &'static str {
    match sort {
        silica_core::AppLibrarySort::ImportedAtDesc => "imported_at_desc",
        silica_core::AppLibrarySort::FileNameAsc => "file_name_asc",
        silica_core::AppLibrarySort::RatingDesc => "rating_desc",
    }
}

pub(crate) fn parse_desktop_app_file_type_filter(
    file_type: &str,
) -> Result<silica_core::AppFileTypeFilter, silica_core::CoreError> {
    match file_type {
        "jpeg" => Ok(silica_core::AppFileTypeFilter::Jpeg),
        "png" => Ok(silica_core::AppFileTypeFilter::Png),
        "tiff" => Ok(silica_core::AppFileTypeFilter::Tiff),
        "raw" => Ok(silica_core::AppFileTypeFilter::Raw),
        "unsupported" => Ok(silica_core::AppFileTypeFilter::Unsupported),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app file type filter: {other}"
        ))),
    }
}

pub(crate) fn app_file_type_filter_string(filter: silica_core::AppFileTypeFilter) -> &'static str {
    match filter {
        silica_core::AppFileTypeFilter::Jpeg => "jpeg",
        silica_core::AppFileTypeFilter::Png => "png",
        silica_core::AppFileTypeFilter::Tiff => "tiff",
        silica_core::AppFileTypeFilter::Raw => "raw",
        silica_core::AppFileTypeFilter::Unsupported => "unsupported",
    }
}

pub(crate) fn parse_desktop_app_metadata_filter(
    metadata: &str,
) -> Result<silica_core::AppMetadataFilter, silica_core::CoreError> {
    match metadata {
        "has_dimensions" => Ok(silica_core::AppMetadataFilter::HasDimensions),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid app metadata filter: {other}"
        ))),
    }
}

pub(crate) fn app_metadata_filter_string(filter: silica_core::AppMetadataFilter) -> &'static str {
    match filter {
        silica_core::AppMetadataFilter::HasDimensions => "has_dimensions",
    }
}

pub(crate) fn parse_desktop_app_appearance_theme(
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

pub(crate) fn app_appearance_theme_string(theme: silica_core::AppAppearanceTheme) -> &'static str {
    match theme {
        silica_core::AppAppearanceTheme::Dark => "dark",
        silica_core::AppAppearanceTheme::Light => "light",
    }
}

pub(crate) fn parse_desktop_app_appearance_density(
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

pub(crate) fn app_appearance_density_string(
    density: silica_core::AppAppearanceDensity,
) -> &'static str {
    match density {
        silica_core::AppAppearanceDensity::Compact => "compact",
        silica_core::AppAppearanceDensity::Comfortable => "comfortable",
    }
}
