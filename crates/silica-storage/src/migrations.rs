use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};

use super::CURRENT_SCHEMA_VERSION;

#[derive(Debug, Clone, Copy)]
struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "initial_catalog_schema",
        sql: INITIAL_CATALOG_SCHEMA_SQL,
    },
    Migration {
        version: 2,
        name: "required_catalog_indexes",
        sql: REQUIRED_INDEXES_SQL,
    },
    Migration {
        version: 3,
        name: "paged_library_query_indexes",
        sql: PAGED_LIBRARY_QUERY_INDEXES_SQL,
    },
    Migration {
        version: 4,
        name: "photo_metadata_normalized_fields",
        sql: PHOTO_METADATA_NORMALIZED_FIELDS_SQL,
    },
    Migration {
        version: 5,
        name: "photo_metadata_query_indexes",
        sql: PHOTO_METADATA_QUERY_INDEXES_SQL,
    },
    Migration {
        version: 6,
        name: "edit_history_checkpoint_columns",
        sql: EDIT_HISTORY_CHECKPOINT_COLUMNS_SQL,
    },
    Migration {
        version: 7,
        name: "edit_history_state_columns",
        sql: EDIT_HISTORY_STATE_COLUMNS_SQL,
    },
    Migration {
        version: 8,
        name: "action_log_side_effect_columns",
        sql: ACTION_LOG_SIDE_EFFECT_COLUMNS_SQL,
    },
    Migration {
        version: 9,
        name: "export_settings_presets",
        sql: EXPORT_SETTINGS_PRESETS_SQL,
    },
    Migration {
        version: 10,
        name: "export_settings_png_tiff_formats",
        sql: EXPORT_SETTINGS_PNG_TIFF_FORMATS_SQL,
    },
    Migration {
        version: 11,
        name: "export_settings_metadata_policy",
        sql: EXPORT_SETTINGS_METADATA_POLICY_SQL,
    },
    Migration {
        version: 12,
        name: "raster_source_file_types",
        sql: RASTER_SOURCE_FILE_TYPES_SQL,
    },
];

/// Apply connection-local safety and durability settings.
pub fn configure_connection(connection: &Connection) -> rusqlite::Result<()> {
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

/// Apply every embedded migration that has not already run.
pub fn run_migrations(connection: &mut Connection) -> rusqlite::Result<()> {
    run_migrations_through(connection, CURRENT_SCHEMA_VERSION)
}

/// Apply embedded migrations up to a target version.
pub fn run_migrations_through(
    connection: &mut Connection,
    target_version: i64,
) -> rusqlite::Result<()> {
    ensure_migration_table(connection)?;
    let applied_version = applied_schema_version(connection)?;

    for migration in MIGRATIONS.iter().filter(|migration| {
        migration.version > applied_version && migration.version <= target_version
    }) {
        let disable_foreign_keys = migration.version == 12;
        if disable_foreign_keys {
            connection.pragma_update(None, "foreign_keys", "OFF")?;
        }
        let result = (|| {
            let transaction = connection.transaction()?;
            transaction.execute_batch(migration.sql)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, name) VALUES (?1, ?2)",
                params![migration.version, migration.name],
            )?;
            transaction.commit()
        })();
        if disable_foreign_keys {
            connection.pragma_update(None, "foreign_keys", "ON")?;
        }
        result?;
    }

    Ok(())
}

/// Return the highest migration version applied to the catalog.
pub fn current_schema_version(connection: &Connection) -> rusqlite::Result<i64> {
    ensure_migration_table(connection)?;
    applied_schema_version(connection)
}

/// Check whether a required table or index exists in `sqlite_master`.
pub fn catalog_object_exists(connection: &Connection, name: &str) -> rusqlite::Result<bool> {
    connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE name = ?1 LIMIT 1",
            params![name],
            |_| Ok(()),
        )
        .optional()
        .map(|result| result.is_some())
}

fn ensure_migration_table(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
          version INTEGER PRIMARY KEY,
          name TEXT NOT NULL,
          applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
}

fn applied_schema_version(connection: &Connection) -> rusqlite::Result<i64> {
    connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )
}

const INITIAL_CATALOG_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS libraries (
  id TEXT PRIMARY KEY,
  root_path TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS folders (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  path TEXT NOT NULL,
  scanned_at TEXT,
  missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
  FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
  UNIQUE (library_id, path)
);

CREATE TABLE IF NOT EXISTS photos (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  folder_id TEXT NOT NULL,
  file_name TEXT NOT NULL,
  path TEXT NOT NULL,
  file_size INTEGER NOT NULL DEFAULT 0,
  modified_at TEXT,
  capture_time TEXT,
  imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
  unsupported INTEGER NOT NULL DEFAULT 0 CHECK (unsupported IN (0, 1)),
  partial_hash TEXT,
  full_hash TEXT,
  FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
  FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE,
  UNIQUE (library_id, path)
);

CREATE TABLE IF NOT EXISTS photo_metadata (
  photo_id TEXT PRIMARY KEY,
  camera_make TEXT,
  camera_model TEXT,
  lens_model TEXT,
  capture_time TEXT,
  raw_json TEXT NOT NULL DEFAULT '{}',
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS photo_flags (
  photo_id TEXT PRIMARY KEY,
  rating INTEGER NOT NULL DEFAULT 0 CHECK (rating BETWEEN 0 AND 5),
  rejected INTEGER NOT NULL DEFAULT 0 CHECK (rejected IN (0, 1)),
  picked INTEGER NOT NULL DEFAULT 0 CHECK (picked IN (0, 1)),
  color_label TEXT,
  edited INTEGER NOT NULL DEFAULT 0 CHECK (edited IN (0, 1)),
  exported INTEGER NOT NULL DEFAULT 0 CHECK (exported IN (0, 1)),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS collections (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
  UNIQUE (library_id, name)
);

CREATE TABLE IF NOT EXISTS collection_photos (
  collection_id TEXT NOT NULL,
  photo_id TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (collection_id, photo_id),
  FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS edit_states (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  active INTEGER NOT NULL DEFAULT 1 CHECK (active IN (0, 1)),
  edit_graph_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS edit_history (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  edit_state_id TEXT,
  action_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE,
  FOREIGN KEY (edit_state_id) REFERENCES edit_states(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  preset_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS sidecar_status (
  photo_id TEXT PRIMARY KEY,
  sidecar_path TEXT,
  last_written_at TEXT,
  conflict_state TEXT NOT NULL DEFAULT 'clean',
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS cache_records (
  id TEXT PRIMARY KEY,
  photo_id TEXT,
  cache_type TEXT NOT NULL,
  cache_key TEXT NOT NULL UNIQUE,
  path TEXT NOT NULL,
  byte_size INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_accessed_at TEXT,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ai_results (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  task_type TEXT NOT NULL,
  model_id TEXT NOT NULL,
  result_json TEXT NOT NULL,
  approved INTEGER NOT NULL DEFAULT 0 CHECK (approved IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS exports (
  id TEXT PRIMARY KEY,
  photo_id TEXT NOT NULL,
  output_path TEXT NOT NULL,
  export_settings_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (photo_id) REFERENCES photos(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS action_log (
  id TEXT PRIMARY KEY,
  actor_type TEXT NOT NULL,
  actor_id TEXT,
  action_type TEXT NOT NULL,
  subject_type TEXT,
  subject_id TEXT,
  payload_json TEXT NOT NULL DEFAULT '{}',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

const REQUIRED_INDEXES_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_folders_library_id
  ON folders(library_id);

CREATE INDEX IF NOT EXISTS idx_photos_library_id
  ON photos(library_id);

CREATE INDEX IF NOT EXISTS idx_photos_folder_id
  ON photos(folder_id);

CREATE INDEX IF NOT EXISTS idx_photos_capture_time
  ON photos(capture_time);

CREATE INDEX IF NOT EXISTS idx_photos_imported_at
  ON photos(imported_at);

CREATE INDEX IF NOT EXISTS idx_photos_missing
  ON photos(missing);

CREATE INDEX IF NOT EXISTS idx_photos_unsupported
  ON photos(unsupported);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rating
  ON photo_flags(rating);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rejected
  ON photo_flags(rejected);

CREATE INDEX IF NOT EXISTS idx_photo_flags_picked
  ON photo_flags(picked);

CREATE INDEX IF NOT EXISTS idx_photo_flags_label
  ON photo_flags(color_label);

CREATE INDEX IF NOT EXISTS idx_collections_library_id
  ON collections(library_id);

CREATE INDEX IF NOT EXISTS idx_collection_photos_photo_id
  ON collection_photos(photo_id);

CREATE INDEX IF NOT EXISTS idx_edit_states_photo_id
  ON edit_states(photo_id);

CREATE INDEX IF NOT EXISTS idx_edit_states_photo_active
  ON edit_states(photo_id, active);

CREATE INDEX IF NOT EXISTS idx_edit_history_photo_id
  ON edit_history(photo_id);

CREATE INDEX IF NOT EXISTS idx_cache_records_photo_type
  ON cache_records(photo_id, cache_type);

CREATE INDEX IF NOT EXISTS idx_cache_records_key
  ON cache_records(cache_key);

CREATE INDEX IF NOT EXISTS idx_ai_results_photo_task
  ON ai_results(photo_id, task_type);

CREATE INDEX IF NOT EXISTS idx_ai_results_model
  ON ai_results(model_id, task_type);

CREATE INDEX IF NOT EXISTS idx_exports_photo_id
  ON exports(photo_id);

CREATE INDEX IF NOT EXISTS idx_action_log_actor
  ON action_log(actor_type, actor_id);

CREATE INDEX IF NOT EXISTS idx_action_log_created_at
  ON action_log(created_at);
"#;

const PAGED_LIBRARY_QUERY_INDEXES_SQL: &str = r#"
ALTER TABLE photos
  ADD COLUMN file_type TEXT NOT NULL DEFAULT 'unsupported'
  CHECK (file_type IN ('jpeg', 'raw', 'unsupported'));

UPDATE photos
SET file_type = CASE
  WHEN unsupported = 1 THEN 'unsupported'
  WHEN lower(file_name) GLOB '*.jpg'
    OR lower(file_name) GLOB '*.jpeg'
    OR lower(path) GLOB '*.jpg'
    OR lower(path) GLOB '*.jpeg'
    THEN 'jpeg'
  ELSE 'unsupported'
END;

UPDATE photos
SET unsupported = 1
WHERE file_type = 'unsupported';

CREATE INDEX IF NOT EXISTS idx_photos_library_imported_id
  ON photos(library_id, imported_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_photos_library_file_name_path_id
  ON photos(library_id, file_name ASC, path ASC, id ASC);

CREATE INDEX IF NOT EXISTS idx_photos_library_file_type_id
  ON photos(library_id, file_type, id ASC);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rating_photo_id
  ON photo_flags(rating DESC, photo_id ASC);
"#;

const PHOTO_METADATA_NORMALIZED_FIELDS_SQL: &str = r#"
ALTER TABLE photo_metadata
  ADD COLUMN width INTEGER;

ALTER TABLE photo_metadata
  ADD COLUMN height INTEGER;

ALTER TABLE photo_metadata
  ADD COLUMN orientation TEXT;
"#;

const PHOTO_METADATA_QUERY_INDEXES_SQL: &str = r#"
CREATE INDEX IF NOT EXISTS idx_photo_metadata_dimensions_photo_id
  ON photo_metadata(width, height, photo_id);
"#;

const EDIT_HISTORY_CHECKPOINT_COLUMNS_SQL: &str = r#"
ALTER TABLE edit_history
  ADD COLUMN sequence INTEGER NOT NULL DEFAULT 0;

ALTER TABLE edit_history
  ADD COLUMN action_class TEXT NOT NULL DEFAULT 'undoable';

ALTER TABLE edit_history
  ADD COLUMN action_kind TEXT NOT NULL DEFAULT 'edit_commit';

CREATE INDEX IF NOT EXISTS idx_edit_history_photo_sequence
  ON edit_history(photo_id, sequence);
"#;

const EDIT_HISTORY_STATE_COLUMNS_SQL: &str = r#"
ALTER TABLE edit_history
  ADD COLUMN history_state TEXT NOT NULL DEFAULT 'applied';

CREATE INDEX IF NOT EXISTS idx_edit_history_photo_state_sequence
  ON edit_history(photo_id, history_state, sequence);
"#;

const ACTION_LOG_SIDE_EFFECT_COLUMNS_SQL: &str = r#"
ALTER TABLE action_log
  ADD COLUMN side_effect_category TEXT NOT NULL DEFAULT 'unspecified';

ALTER TABLE action_log
  ADD COLUMN evidence_ref TEXT;

CREATE INDEX IF NOT EXISTS idx_action_log_action_type_created_at
  ON action_log(action_type, created_at);

CREATE INDEX IF NOT EXISTS idx_action_log_subject
  ON action_log(subject_type, subject_id);
"#;

const EXPORT_SETTINGS_PRESETS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS export_presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  format TEXT NOT NULL DEFAULT 'jpeg' CHECK (format IN ('jpeg')),
  color_profile TEXT NOT NULL DEFAULT 'srgb' CHECK (color_profile IN ('srgb', 'display_p3')),
  quality INTEGER NOT NULL DEFAULT 90 CHECK (quality BETWEEN 1 AND 100),
  metadata_policy TEXT NOT NULL DEFAULT 'minimal' CHECK (metadata_policy IN ('minimal')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS export_settings (
  id TEXT PRIMARY KEY CHECK (id = 'default'),
  preset_id TEXT,
  format TEXT NOT NULL DEFAULT 'jpeg' CHECK (format IN ('jpeg')),
  color_profile TEXT NOT NULL DEFAULT 'srgb' CHECK (color_profile IN ('srgb', 'display_p3')),
  quality INTEGER NOT NULL DEFAULT 90 CHECK (quality BETWEEN 1 AND 100),
  metadata_policy TEXT NOT NULL DEFAULT 'minimal' CHECK (metadata_policy IN ('minimal')),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (preset_id) REFERENCES export_presets(id) ON DELETE SET NULL
);

INSERT OR IGNORE INTO export_presets(
  id,
  name,
  format,
  color_profile,
  quality,
  metadata_policy
) VALUES (
  'jpeg-srgb-90',
  'JPEG sRGB 90',
  'jpeg',
  'srgb',
  90,
  'minimal'
);

INSERT OR IGNORE INTO export_settings(
  id,
  preset_id,
  format,
  color_profile,
  quality,
  metadata_policy
) VALUES (
  'default',
  'jpeg-srgb-90',
  'jpeg',
  'srgb',
  90,
  'minimal'
);
"#;

const EXPORT_SETTINGS_PNG_TIFF_FORMATS_SQL: &str = r#"
ALTER TABLE export_settings RENAME TO export_settings_v9;
ALTER TABLE export_presets RENAME TO export_presets_v9;

CREATE TABLE export_presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  format TEXT NOT NULL DEFAULT 'jpeg' CHECK (format IN ('jpeg', 'png', 'tiff')),
  color_profile TEXT NOT NULL DEFAULT 'srgb' CHECK (color_profile IN ('srgb', 'display_p3')),
  quality INTEGER NOT NULL DEFAULT 90 CHECK (quality BETWEEN 1 AND 100),
  metadata_policy TEXT NOT NULL DEFAULT 'minimal' CHECK (metadata_policy IN ('minimal')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE export_settings (
  id TEXT PRIMARY KEY CHECK (id = 'default'),
  preset_id TEXT,
  format TEXT NOT NULL DEFAULT 'jpeg' CHECK (format IN ('jpeg', 'png', 'tiff')),
  color_profile TEXT NOT NULL DEFAULT 'srgb' CHECK (color_profile IN ('srgb', 'display_p3')),
  quality INTEGER NOT NULL DEFAULT 90 CHECK (quality BETWEEN 1 AND 100),
  metadata_policy TEXT NOT NULL DEFAULT 'minimal' CHECK (metadata_policy IN ('minimal')),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (preset_id) REFERENCES export_presets(id) ON DELETE SET NULL
);

INSERT INTO export_presets(
  id,
  name,
  format,
  color_profile,
  quality,
  metadata_policy,
  created_at,
  updated_at
)
SELECT
  id,
  name,
  format,
  color_profile,
  quality,
  metadata_policy,
  created_at,
  updated_at
FROM export_presets_v9;

INSERT INTO export_settings(
  id,
  preset_id,
  format,
  color_profile,
  quality,
  metadata_policy,
  updated_at
)
SELECT
  id,
  preset_id,
  format,
  color_profile,
  quality,
  metadata_policy,
  updated_at
FROM export_settings_v9;

DROP TABLE export_settings_v9;
DROP TABLE export_presets_v9;
"#;

const EXPORT_SETTINGS_METADATA_POLICY_SQL: &str = r#"
ALTER TABLE export_settings RENAME TO export_settings_v10;
ALTER TABLE export_presets RENAME TO export_presets_v10;

CREATE TABLE export_presets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  format TEXT NOT NULL DEFAULT 'jpeg' CHECK (format IN ('jpeg', 'png', 'tiff')),
  color_profile TEXT NOT NULL DEFAULT 'srgb' CHECK (color_profile IN ('srgb', 'display_p3')),
  quality INTEGER NOT NULL DEFAULT 90 CHECK (quality BETWEEN 1 AND 100),
  metadata_policy TEXT NOT NULL DEFAULT 'minimal' CHECK (metadata_policy IN ('minimal', 'preserve', 'remove_gps', 'remove_all')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE export_settings (
  id TEXT PRIMARY KEY CHECK (id = 'default'),
  preset_id TEXT,
  format TEXT NOT NULL DEFAULT 'jpeg' CHECK (format IN ('jpeg', 'png', 'tiff')),
  color_profile TEXT NOT NULL DEFAULT 'srgb' CHECK (color_profile IN ('srgb', 'display_p3')),
  quality INTEGER NOT NULL DEFAULT 90 CHECK (quality BETWEEN 1 AND 100),
  metadata_policy TEXT NOT NULL DEFAULT 'minimal' CHECK (metadata_policy IN ('minimal', 'preserve', 'remove_gps', 'remove_all')),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (preset_id) REFERENCES export_presets(id) ON DELETE SET NULL
);

INSERT INTO export_presets(
  id,
  name,
  format,
  color_profile,
  quality,
  metadata_policy,
  created_at,
  updated_at
)
SELECT
  id,
  name,
  format,
  color_profile,
  quality,
  metadata_policy,
  created_at,
  updated_at
FROM export_presets_v10;

INSERT INTO export_settings(
  id,
  preset_id,
  format,
  color_profile,
  quality,
  metadata_policy,
  updated_at
)
SELECT
  id,
  preset_id,
  format,
  color_profile,
  quality,
  metadata_policy,
  updated_at
FROM export_settings_v10;

DROP TABLE export_settings_v10;
DROP TABLE export_presets_v10;
"#;

const RASTER_SOURCE_FILE_TYPES_SQL: &str = r#"
PRAGMA legacy_alter_table = ON;

ALTER TABLE photos RENAME TO photos_v11;

CREATE TABLE photos (
  id TEXT PRIMARY KEY,
  library_id TEXT NOT NULL,
  folder_id TEXT NOT NULL,
  file_name TEXT NOT NULL,
  path TEXT NOT NULL,
  file_size INTEGER NOT NULL DEFAULT 0,
  modified_at TEXT,
  capture_time TEXT,
  imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  missing INTEGER NOT NULL DEFAULT 0 CHECK (missing IN (0, 1)),
  unsupported INTEGER NOT NULL DEFAULT 0 CHECK (unsupported IN (0, 1)),
  partial_hash TEXT,
  full_hash TEXT,
  file_type TEXT NOT NULL DEFAULT 'unsupported'
    CHECK (file_type IN ('jpeg', 'png', 'tiff', 'raw', 'unsupported')),
  FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE,
  FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE,
  UNIQUE (library_id, path)
);

INSERT INTO photos(
  id,
  library_id,
  folder_id,
  file_name,
  path,
  file_size,
  modified_at,
  capture_time,
  imported_at,
  missing,
  unsupported,
  partial_hash,
  full_hash,
  file_type
)
SELECT
  id,
  library_id,
  folder_id,
  file_name,
  path,
  file_size,
  modified_at,
  capture_time,
  imported_at,
  missing,
  CASE
    WHEN (
      lower(file_name) GLOB '*.jpg'
      OR lower(file_name) GLOB '*.jpeg'
      OR lower(file_name) GLOB '*.png'
      OR lower(file_name) GLOB '*.tif'
      OR lower(file_name) GLOB '*.tiff'
      OR lower(path) GLOB '*.jpg'
      OR lower(path) GLOB '*.jpeg'
      OR lower(path) GLOB '*.png'
      OR lower(path) GLOB '*.tif'
      OR lower(path) GLOB '*.tiff'
    ) THEN 0
    ELSE 1
  END,
  partial_hash,
  full_hash,
  CASE
    WHEN (
      lower(file_name) GLOB '*.jpg'
      OR lower(file_name) GLOB '*.jpeg'
      OR lower(path) GLOB '*.jpg'
      OR lower(path) GLOB '*.jpeg'
    ) THEN 'jpeg'
    WHEN (
      lower(file_name) GLOB '*.png'
      OR lower(path) GLOB '*.png'
    ) THEN 'png'
    WHEN (
      lower(file_name) GLOB '*.tif'
      OR lower(file_name) GLOB '*.tiff'
      OR lower(path) GLOB '*.tif'
      OR lower(path) GLOB '*.tiff'
    ) THEN 'tiff'
    ELSE 'unsupported'
  END
FROM photos_v11;

DROP TABLE photos_v11;

CREATE INDEX IF NOT EXISTS idx_photos_library_id
  ON photos(library_id);

CREATE INDEX IF NOT EXISTS idx_photos_folder_id
  ON photos(folder_id);

CREATE INDEX IF NOT EXISTS idx_photos_capture_time
  ON photos(capture_time);

CREATE INDEX IF NOT EXISTS idx_photos_imported_at
  ON photos(imported_at);

CREATE INDEX IF NOT EXISTS idx_photos_missing
  ON photos(missing);

CREATE INDEX IF NOT EXISTS idx_photos_unsupported
  ON photos(unsupported);

CREATE INDEX IF NOT EXISTS idx_photos_library_imported_id
  ON photos(library_id, imported_at DESC, id ASC);

CREATE INDEX IF NOT EXISTS idx_photos_library_file_name_path_id
  ON photos(library_id, file_name ASC, path ASC, id ASC);

CREATE INDEX IF NOT EXISTS idx_photos_library_file_type_id
  ON photos(library_id, file_type, id ASC);

PRAGMA legacy_alter_table = OFF;
"#;
