//! Catalog domain boundary for SilicaRAW.
//!
//! Phase 4.1 records the local alpha catalog schema contract here so
//! storage, import, and UI work can depend on one domain-facing shape.

use std::error::Error;
use std::fmt;

/// Stable crate name used by scaffold verification.
pub const CRATE_NAME: &str = "silica-catalog";

/// Current local alpha catalog schema version.
pub const ALPHA_CATALOG_SCHEMA_VERSION: i64 = 2;

/// Migration bookkeeping table required in every catalog database.
pub const SCHEMA_MIGRATIONS_TABLE: &str = "schema_migrations";

/// Required local alpha catalog tables from the storage specification.
pub const ALPHA_CATALOG_REQUIRED_TABLES: &[&str] = &[
    "libraries",
    "folders",
    "photos",
    "photo_metadata",
    "photo_flags",
    "collections",
    "collection_photos",
    "edit_states",
    "edit_history",
    "presets",
    "sidecar_status",
    "cache_records",
    "ai_results",
    "exports",
    "action_log",
    SCHEMA_MIGRATIONS_TABLE,
];

/// Required local alpha catalog indexes from the storage specification.
pub const ALPHA_CATALOG_REQUIRED_INDEXES: &[&str] = &[
    "idx_folders_library_id",
    "idx_photos_library_id",
    "idx_photos_folder_id",
    "idx_photos_capture_time",
    "idx_photos_imported_at",
    "idx_photos_missing",
    "idx_photos_unsupported",
    "idx_photo_flags_rating",
    "idx_photo_flags_rejected",
    "idx_photo_flags_picked",
    "idx_photo_flags_label",
    "idx_collections_library_id",
    "idx_collection_photos_photo_id",
    "idx_edit_states_photo_id",
    "idx_edit_states_photo_active",
    "idx_edit_history_photo_id",
    "idx_cache_records_photo_type",
    "idx_cache_records_key",
    "idx_ai_results_photo_task",
    "idx_ai_results_model",
    "idx_exports_photo_id",
    "idx_action_log_actor",
    "idx_action_log_created_at",
];

/// Domain-facing catalog schema contract for local alpha storage work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogSchemaContract {
    pub current_version: i64,
    pub migration_table: &'static str,
    pub required_tables: &'static [&'static str],
    pub required_indexes: &'static [&'static str],
}

/// Local alpha catalog schema contract implemented by `silica-storage`.
pub const ALPHA_CATALOG_SCHEMA: CatalogSchemaContract = CatalogSchemaContract {
    current_version: ALPHA_CATALOG_SCHEMA_VERSION,
    migration_table: SCHEMA_MIGRATIONS_TABLE,
    required_tables: ALPHA_CATALOG_REQUIRED_TABLES,
    required_indexes: ALPHA_CATALOG_REQUIRED_INDEXES,
};

/// File extensions accepted as supported photo candidates in the alpha scanner.
pub const ALPHA_SUPPORTED_PHOTO_EXTENSIONS: &[&str] = &[
    "dng", "cr2", "cr3", "nef", "arw", "raf", "orf", "rw2", "pef", "srw", "raw", "jpg", "jpeg",
    "tif", "tiff", "heic",
];

/// Return whether an extension is a supported local alpha photo candidate.
pub fn is_supported_photo_extension(extension: &str) -> bool {
    ALPHA_SUPPORTED_PHOTO_EXTENSIONS
        .iter()
        .any(|supported| extension.eq_ignore_ascii_case(supported))
}

/// Domain-facing import candidate recorded by the folder scanner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportCandidate {
    pub file_name: String,
    pub path: String,
    pub file_size: i64,
    pub modified_at: Option<String>,
    pub partial_hash: String,
    pub unsupported: bool,
}

/// Highest rating value allowed by the local alpha catalog contract.
pub const ALPHA_MAX_RATING: u8 = 5;

/// Default bounded page size for library grid queries.
pub const DEFAULT_LIBRARY_QUERY_LIMIT: u16 = 100;
/// Maximum accepted page size for offset-based library grid queries.
pub const MAX_LIBRARY_QUERY_LIMIT: u16 = 500;
/// Cursor pagination is intentionally deferred until benchmark evidence requires it.
pub const LIBRARY_QUERY_CURSOR_PAGINATION_DEFERRED: bool = true;

/// Whitelisted sort modes for library grid queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryQuerySort {
    ImportedAtDesc,
    FileNameAsc,
    RatingDesc,
}

/// Whitelisted file type filters for library grid queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryQueryFileType {
    Jpeg,
    Raw,
    Unsupported,
}

/// Whitelisted filter set for library grid queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryQueryFilters {
    pub min_rating: Option<u8>,
    pub picked: Option<bool>,
    pub rejected: Option<bool>,
    pub file_type: Option<LibraryQueryFileType>,
    pub search: String,
}

impl Default for LibraryQueryFilters {
    fn default() -> Self {
        Self {
            min_rating: None,
            picked: None,
            rejected: None,
            file_type: None,
            search: String::new(),
        }
    }
}

/// Offset-paginated library query request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryQueryRequest {
    pub offset: u64,
    pub limit: u16,
    pub sort: LibraryQuerySort,
    pub filters: LibraryQueryFilters,
}

impl LibraryQueryRequest {
    /// Create a bounded offset-paginated query request.
    pub fn new(
        offset: u64,
        limit: u16,
        sort: LibraryQuerySort,
        filters: LibraryQueryFilters,
    ) -> Self {
        Self {
            offset,
            limit: limit.clamp(1, MAX_LIBRARY_QUERY_LIMIT),
            sort,
            filters: normalize_library_query_filters(filters),
        }
    }

    /// Return the deterministic order fields storage must implement.
    pub fn order_fields(&self) -> &'static [LibraryQueryOrderField] {
        self.sort.order_fields()
    }
}

impl Default for LibraryQueryRequest {
    fn default() -> Self {
        Self::new(
            0,
            DEFAULT_LIBRARY_QUERY_LIMIT,
            LibraryQuerySort::ImportedAtDesc,
            LibraryQueryFilters::default(),
        )
    }
}

/// Deterministic order fields for accepted query sort modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryQueryOrderField {
    ImportedAtDesc,
    FileNameAsc,
    RatingDesc,
    PhotoIdAsc,
    PathAsc,
}

impl LibraryQuerySort {
    /// Return the primary sort plus explicit tie breakers.
    pub fn order_fields(self) -> &'static [LibraryQueryOrderField] {
        match self {
            Self::ImportedAtDesc => &[
                LibraryQueryOrderField::ImportedAtDesc,
                LibraryQueryOrderField::PhotoIdAsc,
            ],
            Self::FileNameAsc => &[
                LibraryQueryOrderField::FileNameAsc,
                LibraryQueryOrderField::PathAsc,
                LibraryQueryOrderField::PhotoIdAsc,
            ],
            Self::RatingDesc => &[
                LibraryQueryOrderField::RatingDesc,
                LibraryQueryOrderField::PhotoIdAsc,
            ],
        }
    }
}

/// Page response contract for storage/core library queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryQueryPage<T> {
    pub items: Vec<T>,
    pub offset: u64,
    pub limit: u16,
    pub total_count: u64,
    pub has_next_page: bool,
    pub order_fields: &'static [LibraryQueryOrderField],
}

fn normalize_library_query_filters(mut filters: LibraryQueryFilters) -> LibraryQueryFilters {
    filters.min_rating = filters
        .min_rating
        .map(|rating| rating.min(ALPHA_MAX_RATING));
    filters.search = filters.search.trim().to_string();
    filters
}

/// Domain-facing culling and label flags for one photo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhotoFlags {
    pub photo_id: String,
    pub rating: u8,
    pub picked: bool,
    pub rejected: bool,
    pub color_label: Option<String>,
}

impl PhotoFlags {
    /// Create validated photo flags for catalog persistence.
    pub fn new(
        photo_id: impl Into<String>,
        rating: u8,
        picked: bool,
        rejected: bool,
        color_label: Option<String>,
    ) -> Result<Self, CatalogFlagError> {
        let photo_id = photo_id.into();
        if photo_id.is_empty() {
            return Err(CatalogFlagError::EmptyPhotoId);
        }
        if rating > ALPHA_MAX_RATING {
            return Err(CatalogFlagError::InvalidRating(rating));
        }

        let color_label = match color_label {
            Some(label) => {
                let label = label.trim().to_string();
                if label.is_empty() {
                    return Err(CatalogFlagError::EmptyColorLabel);
                }
                Some(label)
            }
            None => None,
        };

        Ok(Self {
            photo_id,
            rating,
            picked,
            rejected,
            color_label,
        })
    }
}

/// Validation errors for catalog photo flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogFlagError {
    EmptyPhotoId,
    InvalidRating(u8),
    EmptyColorLabel,
}

impl fmt::Display for CatalogFlagError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPhotoId => write!(formatter, "photo id must not be empty"),
            Self::InvalidRating(rating) => {
                write!(
                    formatter,
                    "rating must be between 0 and {ALPHA_MAX_RATING}: {rating}"
                )
            }
            Self::EmptyColorLabel => write!(formatter, "color label must not be empty"),
        }
    }
}

impl Error for CatalogFlagError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_crate_name() {
        assert_eq!(CRATE_NAME, "silica-catalog");
    }

    #[test]
    fn records_phase_4_1_catalog_schema_contract() {
        assert_eq!(ALPHA_CATALOG_SCHEMA.current_version, 2);
        assert_eq!(ALPHA_CATALOG_SCHEMA.migration_table, "schema_migrations");
        assert_eq!(
            ALPHA_CATALOG_SCHEMA.required_tables,
            [
                "libraries",
                "folders",
                "photos",
                "photo_metadata",
                "photo_flags",
                "collections",
                "collection_photos",
                "edit_states",
                "edit_history",
                "presets",
                "sidecar_status",
                "cache_records",
                "ai_results",
                "exports",
                "action_log",
                "schema_migrations",
            ]
        );
        assert!(ALPHA_CATALOG_SCHEMA
            .required_indexes
            .contains(&"idx_photos_library_id"));
        assert!(ALPHA_CATALOG_SCHEMA
            .required_indexes
            .contains(&"idx_action_log_created_at"));
    }

    #[test]
    fn classifies_alpha_import_file_extensions() {
        assert!(is_supported_photo_extension("DNG"));
        assert!(is_supported_photo_extension("jpg"));
        assert!(is_supported_photo_extension("RAF"));
        assert!(!is_supported_photo_extension("txt"));
        assert!(!is_supported_photo_extension(""));
    }

    #[test]
    fn records_import_candidate_support_state() {
        let candidate = ImportCandidate {
            file_name: "notes.txt".to_string(),
            path: "/tmp/notes.txt".to_string(),
            file_size: 11,
            modified_at: Some("2026-06-08T10:00:00Z".to_string()),
            partial_hash: "hash".to_string(),
            unsupported: true,
        };

        assert!(candidate.unsupported);
        assert_eq!(candidate.file_name, "notes.txt");
        assert_eq!(candidate.partial_hash, "hash");
    }

    #[test]
    fn records_paged_library_query_contract() {
        let mut filters = LibraryQueryFilters {
            min_rating: Some(9),
            picked: Some(true),
            rejected: None,
            file_type: Some(LibraryQueryFileType::Jpeg),
            search: "  portrait  ".to_string(),
        };

        let request = LibraryQueryRequest::new(
            500,
            MAX_LIBRARY_QUERY_LIMIT + 100,
            LibraryQuerySort::FileNameAsc,
            filters.clone(),
        );

        assert_eq!(request.offset, 500);
        assert_eq!(request.limit, MAX_LIBRARY_QUERY_LIMIT);
        assert_eq!(request.filters.min_rating, Some(ALPHA_MAX_RATING));
        assert_eq!(request.filters.search, "portrait");
        assert_eq!(
            request.order_fields(),
            &[
                LibraryQueryOrderField::FileNameAsc,
                LibraryQueryOrderField::PathAsc,
                LibraryQueryOrderField::PhotoIdAsc,
            ]
        );
        assert!(LIBRARY_QUERY_CURSOR_PAGINATION_DEFERRED);

        filters.min_rating = None;
        let defaulted = LibraryQueryRequest::new(0, 0, LibraryQuerySort::ImportedAtDesc, filters);
        assert_eq!(defaulted.limit, 1);
        assert_eq!(
            defaulted.order_fields(),
            &[
                LibraryQueryOrderField::ImportedAtDesc,
                LibraryQueryOrderField::PhotoIdAsc,
            ]
        );

        let page = LibraryQueryPage {
            items: vec!["photo-1".to_string()],
            offset: defaulted.offset,
            limit: defaulted.limit,
            total_count: 2,
            has_next_page: true,
            order_fields: LibraryQuerySort::RatingDesc.order_fields(),
        };
        assert_eq!(
            page.order_fields,
            &[
                LibraryQueryOrderField::RatingDesc,
                LibraryQueryOrderField::PhotoIdAsc,
            ]
        );
        assert!(page.has_next_page);
    }

    #[test]
    fn validates_photo_flags_contract() {
        let flags =
            PhotoFlags::new("photo-1", 5, true, false, Some(" green ".to_string())).unwrap();

        assert_eq!(flags.photo_id, "photo-1");
        assert_eq!(flags.rating, 5);
        assert!(flags.picked);
        assert!(!flags.rejected);
        assert_eq!(flags.color_label.as_deref(), Some("green"));
        assert_eq!(
            PhotoFlags::new("photo-1", 6, false, false, None),
            Err(CatalogFlagError::InvalidRating(6))
        );
        assert_eq!(
            PhotoFlags::new("", 0, false, false, None),
            Err(CatalogFlagError::EmptyPhotoId)
        );
        assert_eq!(
            PhotoFlags::new("photo-1", 0, false, false, Some(" ".to_string())),
            Err(CatalogFlagError::EmptyColorLabel)
        );
    }
}
