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
}
