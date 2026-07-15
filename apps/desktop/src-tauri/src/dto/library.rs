use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopCacheDirectoryStatus {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) byte_size: u64,
    pub(crate) file_count: u64,
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct DesktopLibraryQueryRequest {
    pub(crate) offset: u64,
    pub(crate) limit: u16,
    pub(crate) sort: String,
    #[serde(default)]
    pub(crate) filters: DesktopLibraryQueryFilters,
}

impl DesktopLibraryQueryRequest {
    pub(crate) fn into_core(
        self,
    ) -> Result<silica_core::LibraryQueryRequest, silica_core::CoreError> {
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
pub(crate) struct DesktopLibraryQueryFilters {
    pub(crate) min_rating: Option<u8>,
    pub(crate) picked: Option<bool>,
    pub(crate) rejected: Option<bool>,
    pub(crate) file_type: Option<String>,
    pub(crate) metadata: Option<String>,
    pub(crate) search: String,
}

impl DesktopLibraryQueryFilters {
    pub(crate) fn into_core(
        self,
    ) -> Result<silica_core::LibraryQueryFilters, silica_core::CoreError> {
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
pub(crate) struct DesktopImportIssue {
    pub(crate) kind: &'static str,
    pub(crate) path: String,
    pub(crate) file_name: Option<String>,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopPhotoGridItem {
    pub(crate) photo_id: String,
    pub(crate) file_name: String,
    pub(crate) path: String,
    pub(crate) file_type: String,
    pub(crate) thumbnail_path: Option<String>,
    pub(crate) thumbnail_bytes: Option<Vec<u8>>,
    pub(crate) missing: bool,
    pub(crate) unsupported: bool,
    pub(crate) rating: u8,
    pub(crate) picked: bool,
    pub(crate) rejected: bool,
    pub(crate) color_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DesktopMetadataField<T> {
    pub(crate) state: &'static str,
    pub(crate) value: Option<T>,
}

impl From<silica_core::LibraryPhotoGridItem> for DesktopPhotoGridItem {
    fn from(photo: silica_core::LibraryPhotoGridItem) -> Self {
        let thumbnail_path = if photo.missing || photo.unsupported {
            None
        } else {
            photo.thumbnail_path
        };
        let thumbnail_bytes = thumbnail_path
            .as_ref()
            .and_then(|path| std::fs::read(path).ok());
        Self {
            photo_id: photo.photo_id,
            file_name: photo.file_name,
            path: photo.path,
            file_type: photo.file_type,
            thumbnail_bytes,
            thumbnail_path,
            missing: photo.missing,
            unsupported: photo.unsupported,
            rating: photo.rating,
            picked: photo.picked,
            rejected: photo.rejected,
            color_label: photo.color_label,
        }
    }
}

pub(crate) fn desktop_import_issue(issue: silica_core::ImportIssue) -> DesktopImportIssue {
    DesktopImportIssue {
        kind: issue.kind.as_str(),
        path: issue.path,
        file_name: issue.file_name,
        message: issue.message,
    }
}

pub(crate) fn metadata_field<T>(
    field: silica_core::PhotoMetadataField<T>,
) -> DesktopMetadataField<T> {
    DesktopMetadataField {
        state: metadata_field_state_string(field.state),
        value: field.value,
    }
}

pub(crate) fn metadata_field_state_string(
    state: silica_core::PhotoMetadataFieldState,
) -> &'static str {
    match state {
        silica_core::PhotoMetadataFieldState::Known => "known",
        silica_core::PhotoMetadataFieldState::Unknown => "unknown",
        silica_core::PhotoMetadataFieldState::Unavailable => "unavailable",
    }
}

pub(crate) fn parse_desktop_library_query_sort(
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

pub(crate) fn parse_desktop_library_query_file_type(
    file_type: &str,
) -> Result<silica_core::LibraryQueryFileType, silica_core::CoreError> {
    match file_type {
        "jpeg" => Ok(silica_core::LibraryQueryFileType::Jpeg),
        "png" => Ok(silica_core::LibraryQueryFileType::Png),
        "tiff" => Ok(silica_core::LibraryQueryFileType::Tiff),
        "raw" => Ok(silica_core::LibraryQueryFileType::Raw),
        "unsupported" => Ok(silica_core::LibraryQueryFileType::Unsupported),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid library query file type: {other}"
        ))),
    }
}

pub(crate) fn parse_desktop_library_query_metadata(
    metadata: &str,
) -> Result<silica_core::LibraryQueryMetadataFilter, silica_core::CoreError> {
    match metadata {
        "has_dimensions" => Ok(silica_core::LibraryQueryMetadataFilter::HasDimensions),
        other => Err(silica_core::CoreError::AppSession(format!(
            "invalid library query metadata filter: {other}"
        ))),
    }
}

pub(crate) fn library_query_order_field_string(
    field: silica_core::LibraryQueryOrderField,
) -> &'static str {
    match field {
        silica_core::LibraryQueryOrderField::ImportedAtDesc => "imported_at_desc",
        silica_core::LibraryQueryOrderField::FileNameAsc => "file_name_asc",
        silica_core::LibraryQueryOrderField::RatingDesc => "rating_desc",
        silica_core::LibraryQueryOrderField::PhotoIdAsc => "photo_id_asc",
        silica_core::LibraryQueryOrderField::PathAsc => "path_asc",
    }
}

pub(crate) fn preview_status_text(status: silica_core::PhotoPreviewStatus) -> &'static str {
    match status {
        silica_core::PhotoPreviewStatus::Ready => "Ready",
        silica_core::PhotoPreviewStatus::BlockedByDecode => "BlockedByDecode",
        silica_core::PhotoPreviewStatus::Unsupported => "Unsupported",
    }
}

pub(crate) fn histogram_status_text(status: silica_core::PhotoHistogramStatus) -> &'static str {
    match status {
        silica_core::PhotoHistogramStatus::Ready => "Ready",
        silica_core::PhotoHistogramStatus::BlockedByDecode => "BlockedByDecode",
        silica_core::PhotoHistogramStatus::Unsupported => "Unsupported",
        silica_core::PhotoHistogramStatus::Missing => "Missing",
    }
}
