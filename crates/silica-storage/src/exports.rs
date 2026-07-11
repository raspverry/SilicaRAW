use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};
use silica_catalog::CatalogFlagError;

use crate::common::{path_to_string, stable_catalog_id};
use crate::{
    open_catalog, open_existing_library_for_read, open_local_library, ExportPreset, ExportRecord,
    ExportSettings, ExportSettingsCatalog, LibraryStorageError, RecentExportRecord,
    DEFAULT_EXPORT_PRESET_ID, DEFAULT_EXPORT_SETTINGS_ID,
};

/// Read the library-wide export settings and named presets.
pub fn get_export_settings_catalog(
    library_root_path: impl AsRef<Path>,
) -> Result<ExportSettingsCatalog, LibraryStorageError> {
    let library = open_local_library(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    export_settings_catalog_from_connection(&connection)
}

/// Create or update a named export preset.
pub fn upsert_export_preset(
    library_root_path: impl AsRef<Path>,
    name: impl AsRef<str>,
    settings: ExportSettings,
) -> Result<ExportPreset, LibraryStorageError> {
    let name = name.as_ref().trim();
    if name.is_empty() {
        return Err(LibraryStorageError::ExportSettingsValidation(
            "export preset name must not be empty".to_string(),
        ));
    }
    validate_export_settings(&settings)?;

    let library = open_local_library(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let preset_id = export_preset_id_for_name(name);
    connection.execute(
        r#"
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
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
          name = excluded.name,
          format = excluded.format,
          color_profile = excluded.color_profile,
          quality = excluded.quality,
          metadata_policy = excluded.metadata_policy,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            preset_id,
            name,
            &settings.format,
            &settings.color_profile,
            settings.quality,
            &settings.metadata_policy
        ],
    )?;

    Ok(ExportPreset {
        id: export_preset_id_for_name(name),
        name: name.to_string(),
        settings,
    })
}

/// Persist the current default export settings.
pub fn set_default_export_settings(
    library_root_path: impl AsRef<Path>,
    preset_id: Option<&str>,
    settings: ExportSettings,
) -> Result<ExportSettingsCatalog, LibraryStorageError> {
    validate_export_settings(&settings)?;
    let preset_id = match preset_id.map(str::trim) {
        Some("") => {
            return Err(LibraryStorageError::ExportSettingsValidation(
                "default export preset id must not be empty".to_string(),
            ))
        }
        Some(value) => Some(value.to_string()),
        None => None,
    };

    let library = open_local_library(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    if let Some(preset_id) = preset_id.as_deref() {
        let preset_count: i64 = connection.query_row(
            "SELECT COUNT(*) FROM export_presets WHERE id = ?1",
            params![preset_id],
            |row| row.get(0),
        )?;
        if preset_count == 0 {
            return Err(LibraryStorageError::ExportSettingsValidation(format!(
                "unknown export preset: {preset_id}"
            )));
        }
    }

    connection.execute(
        r#"
        INSERT INTO export_settings(
          id,
          preset_id,
          format,
          color_profile,
          quality,
          metadata_policy,
          updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
          preset_id = excluded.preset_id,
          format = excluded.format,
          color_profile = excluded.color_profile,
          quality = excluded.quality,
          metadata_policy = excluded.metadata_policy,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            DEFAULT_EXPORT_SETTINGS_ID,
            preset_id,
            &settings.format,
            &settings.color_profile,
            settings.quality,
            &settings.metadata_policy
        ],
    )?;

    export_settings_catalog_from_connection(&connection)
}

/// Record a completed export and mark the source photo as exported.
pub fn record_export(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    output_path: impl AsRef<Path>,
    export_settings_json: impl AsRef<str>,
) -> Result<ExportRecord, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let output_path = path_to_string(output_path.as_ref())?;
    let export_settings_json = export_settings_json.as_ref().to_string();
    serde_json::from_str::<serde_json::Value>(&export_settings_json)?;

    let library = open_local_library(library_root_path)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    let export_id = stable_catalog_id("export", &format!("{photo_id}\n{output_path}"));

    let transaction = connection.transaction()?;
    transaction.execute(
        r#"
        INSERT INTO exports(id, photo_id, output_path, export_settings_json, created_at)
        VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
          output_path = excluded.output_path,
          export_settings_json = excluded.export_settings_json,
          created_at = CURRENT_TIMESTAMP
        "#,
        params![export_id, photo_id, output_path, export_settings_json],
    )?;
    transaction.execute(
        r#"
        INSERT INTO photo_flags(photo_id, exported, updated_at)
        VALUES (?1, 1, CURRENT_TIMESTAMP)
        ON CONFLICT(photo_id) DO UPDATE SET
          exported = 1,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![photo_id],
    )?;
    transaction.commit()?;

    Ok(ExportRecord {
        id: export_id,
        photo_id: photo_id.to_string(),
        output_path,
        export_settings_json,
    })
}

/// Read the most recent export record for one photo.
pub fn get_latest_export_record(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<ExportRecord>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    connection
        .query_row(
            r#"
            SELECT id, photo_id, output_path, export_settings_json
            FROM exports
            WHERE photo_id = ?1
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
            params![photo_id],
            export_record_from_row,
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

/// Read recent export records for the library.
pub fn list_recent_export_records(
    library_root_path: impl AsRef<Path>,
    limit: usize,
) -> Result<Vec<RecentExportRecord>, LibraryStorageError> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let mut statement = connection.prepare(
        r#"
        SELECT id, photo_id, output_path, export_settings_json, created_at
        FROM exports
        ORDER BY datetime(created_at) DESC, rowid DESC
        LIMIT ?1
        "#,
    )?;
    let rows = statement.query_map(params![limit as i64], recent_export_record_from_row)?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(LibraryStorageError::from)
}

fn export_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportRecord> {
    Ok(ExportRecord {
        id: row.get(0)?,
        photo_id: row.get(1)?,
        output_path: row.get(2)?,
        export_settings_json: row.get(3)?,
    })
}

fn recent_export_record_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<RecentExportRecord> {
    Ok(RecentExportRecord {
        id: row.get(0)?,
        photo_id: row.get(1)?,
        output_path: row.get(2)?,
        export_settings_json: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn export_settings_catalog_from_connection(
    connection: &Connection,
) -> Result<ExportSettingsCatalog, LibraryStorageError> {
    let (default_preset_id, default_settings) = connection.query_row(
        r#"
        SELECT preset_id, format, color_profile, quality, metadata_policy
        FROM export_settings
        WHERE id = ?1
        "#,
        params![DEFAULT_EXPORT_SETTINGS_ID],
        |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                export_settings_from_row(row, 1)?,
            ))
        },
    )?;

    let mut statement = connection.prepare(
        r#"
        SELECT id, name, format, color_profile, quality, metadata_policy
        FROM export_presets
        ORDER BY
          CASE WHEN id = 'jpeg-srgb-90' THEN 0 ELSE 1 END,
          name ASC,
          id ASC
        "#,
    )?;
    let presets = statement
        .query_map([], export_preset_from_row)?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ExportSettingsCatalog {
        default_preset_id,
        default_settings,
        presets,
    })
}

fn export_preset_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<ExportPreset> {
    Ok(ExportPreset {
        id: row.get(0)?,
        name: row.get(1)?,
        settings: export_settings_from_row(row, 2)?,
    })
}

fn export_settings_from_row(
    row: &rusqlite::Row<'_>,
    offset: usize,
) -> rusqlite::Result<ExportSettings> {
    Ok(ExportSettings {
        format: row.get(offset)?,
        color_profile: row.get(offset + 1)?,
        quality: row.get(offset + 2)?,
        metadata_policy: row.get(offset + 3)?,
    })
}

fn export_preset_id_for_name(name: &str) -> String {
    if name == "JPEG sRGB 90" {
        DEFAULT_EXPORT_PRESET_ID.to_string()
    } else {
        stable_catalog_id("export-preset", name)
    }
}

fn validate_export_settings(settings: &ExportSettings) -> Result<(), LibraryStorageError> {
    if !matches!(settings.format.as_str(), "jpeg" | "png" | "tiff") {
        return Err(LibraryStorageError::ExportSettingsValidation(format!(
            "unsupported export format: {}",
            settings.format
        )));
    }
    if !matches!(settings.color_profile.as_str(), "srgb" | "display_p3") {
        return Err(LibraryStorageError::ExportSettingsValidation(format!(
            "unsupported export color profile: {}",
            settings.color_profile
        )));
    }
    if settings.format != "jpeg" && settings.color_profile != "srgb" {
        return Err(LibraryStorageError::ExportSettingsValidation(
            "PNG and TIFF export settings currently require sRGB color profile".to_string(),
        ));
    }
    if !(1..=100).contains(&settings.quality) {
        return Err(LibraryStorageError::ExportSettingsValidation(format!(
            "export quality must be between 1 and 100, got {}",
            settings.quality
        )));
    }
    if !matches!(
        settings.metadata_policy.as_str(),
        "minimal" | "preserve" | "remove_gps" | "remove_all"
    ) {
        return Err(LibraryStorageError::ExportSettingsValidation(format!(
            "unsupported export metadata policy: {}",
            settings.metadata_policy
        )));
    }
    Ok(())
}
