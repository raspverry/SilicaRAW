//! Catalog domain boundary for SilicaRAW.
//!
//! Phase 4.1 records the local alpha catalog schema contract here so
//! storage, import, and UI work can depend on one domain-facing shape.

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
}
