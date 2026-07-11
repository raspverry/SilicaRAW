use std::fs;
use std::path::Path;
use std::path::PathBuf;

use rusqlite::{params, Connection, OptionalExtension};

use super::common::{current_timestamp_string, validate_sidecar_photo_id};
use super::{
    active_edit_state_id, default_rebuild_flags, get_photo_flags, get_photo_flags_from_connection,
    load_active_edit_graph_or_default, open_catalog, open_existing_library_for_read,
    open_existing_library_for_read_only_query, open_local_library, CatalogRebuildDryRunAction,
    CatalogRebuildDryRunEntry, CatalogRebuildDryRunIssue, CatalogRebuildDryRunIssueKind,
    CatalogRebuildDryRunReport, CatalogRebuildFlagSource, LibraryStorageError, PhotoFlags,
    PhotoSidecarStatus, SidecarWriteResult, ValidatedSidecar, SIDECAR_DIRECTORY, SIDECAR_SCHEMA,
    SIDECAR_VERSION,
};

pub(super) const SIDECAR_FILE_SUFFIX: &str = ".silicaraw.sidecar.json";

#[derive(Debug, Clone, PartialEq, Eq)]
struct SidecarPhotoRow {
    photo_id: String,
    original_path: String,
    file_name: String,
    file_size: i64,
    modified_at: Option<String>,
    partial_hash: Option<String>,
    full_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SidecarPhotoSnapshot {
    original_path: String,
    file_name: String,
    file_size: Option<i64>,
    modified_at: Option<String>,
    partial_hash: Option<String>,
    full_hash: Option<String>,
}

/// Resolve the library-local sidecar path for a catalog photo id.
pub fn sidecar_path_for_photo(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<PathBuf, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    Ok(library_root_path
        .as_ref()
        .join(SIDECAR_DIRECTORY)
        .join(format!("{photo_id}{SIDECAR_FILE_SUFFIX}")))
}

/// Write a validated sidecar into the library-local sidecars directory.
pub fn write_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<SidecarWriteResult, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_local_library(library_root_path)?;
    let sidecar_path = sidecar_path_for_photo(&library.root_path, photo_id)?;
    let sidecar_relative_path = format!("{SIDECAR_DIRECTORY}/{photo_id}{SIDECAR_FILE_SUFFIX}");
    fs::create_dir_all(library.root_path.join(SIDECAR_DIRECTORY))?;

    let value = build_photo_sidecar_value(&library.root_path, photo_id, app_version)?;
    validate_sidecar_json(&value)?;
    let bytes = serde_json::to_vec_pretty(&value)?;
    let temp_path = sidecar_path.with_extension("json.tmp");
    fs::write(&temp_path, &bytes)?;
    let temp_value: serde_json::Value = serde_json::from_slice(&fs::read(&temp_path)?)?;
    validate_sidecar_json(&temp_value)?;
    fs::rename(&temp_path, &sidecar_path)?;

    let written_at = value
        .get("written_at")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_string();
    update_sidecar_status(
        &library.catalog_path,
        photo_id,
        &sidecar_relative_path,
        &written_at,
    )?;

    Ok(SidecarWriteResult {
        photo_id: photo_id.to_string(),
        sidecar_path,
        sidecar_relative_path,
        written_at,
        bytes_written: bytes.len() as u64,
    })
}

/// Read and validate a library-local sidecar without mutating catalog state.
pub fn read_photo_sidecar(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<ValidatedSidecar>, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let sidecar_path = sidecar_path_for_photo(&library.root_path, photo_id)?;
    if !sidecar_path.is_file() {
        return Ok(None);
    }

    let json: serde_json::Value = serde_json::from_slice(&fs::read(&sidecar_path)?)?;
    validate_sidecar_json(&json)?;
    let sidecar_photo_id = json["photo"]["photo_id"].as_str().ok_or_else(|| {
        LibraryStorageError::SidecarValidation(
            "sidecar.photo.photo_id must be a string".to_string(),
        )
    })?;
    if sidecar_photo_id != photo_id {
        return Err(LibraryStorageError::SidecarValidation(format!(
            "sidecar photo id mismatch: expected {photo_id}, found {sidecar_photo_id}"
        )));
    }

    let flags = parse_sidecar_flags(&json)?;
    let edit_graph: silica_edit::EditGraph = serde_json::from_value(json["edit_graph"].clone())?;
    silica_edit::validate_edit_graph(&edit_graph)?;
    let written_at = json["written_at"].as_str().unwrap_or_default().to_string();

    Ok(Some(ValidatedSidecar {
        photo_id: photo_id.to_string(),
        sidecar_path,
        written_at,
        flags,
        edit_graph,
        json,
    }))
}

/// Read catalog-side sidecar sync status without touching sidecar files.
pub fn get_photo_sidecar_status(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoSidecarStatus>, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let (_library, connection) = open_existing_library_for_read_only_query(library_root_path)?;
    connection
        .query_row(
            r#"
            SELECT photo_id, sidecar_path, last_written_at, conflict_state
            FROM sidecar_status
            WHERE photo_id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(PhotoSidecarStatus {
                    photo_id: row.get(0)?,
                    sidecar_path: row.get(1)?,
                    last_written_at: row.get(2)?,
                    conflict_state: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

/// Preview how the live catalog would rebuild portable flag state from sidecars.
pub fn dry_run_catalog_rebuild_from_sidecars(
    library_root_path: impl AsRef<Path>,
) -> Result<CatalogRebuildDryRunReport, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let sidecars_directory = library.root_path.join(SIDECAR_DIRECTORY);
    let mut report = CatalogRebuildDryRunReport {
        sidecars_scanned: 0,
        entries: Vec::new(),
        issues: Vec::new(),
    };

    if !sidecars_directory.is_dir() {
        return Ok(report);
    }

    let mut sidecar_paths = Vec::new();
    for entry in fs::read_dir(&sidecars_directory)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        if file_name.ends_with(SIDECAR_FILE_SUFFIX) {
            sidecar_paths.push(entry.path());
        }
    }
    sidecar_paths.sort_by(|left, right| {
        left.file_name()
            .cmp(&right.file_name())
            .then_with(|| left.cmp(right))
    });

    let connection = open_catalog(&library.catalog_path)?;
    for sidecar_path in sidecar_paths {
        process_rebuild_dry_run_sidecar(&connection, &sidecar_path, &mut report)?;
    }

    Ok(report)
}

fn load_sidecar_photo_row(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<SidecarPhotoRow>, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT id, path, file_name, file_size, modified_at, partial_hash, full_hash
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok(SidecarPhotoRow {
                    photo_id: row.get(0)?,
                    original_path: row.get(1)?,
                    file_name: row.get(2)?,
                    file_size: row.get(3)?,
                    modified_at: row.get(4)?,
                    partial_hash: row.get(5)?,
                    full_hash: row.get(6)?,
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

fn edit_color_label_from_catalog(
    label: Option<&str>,
) -> Result<Option<silica_edit::ColorLabel>, LibraryStorageError> {
    match label {
        None => Ok(None),
        Some("red") => Ok(Some(silica_edit::ColorLabel::Red)),
        Some("orange") => Ok(Some(silica_edit::ColorLabel::Orange)),
        Some("yellow") => Ok(Some(silica_edit::ColorLabel::Yellow)),
        Some("green") => Ok(Some(silica_edit::ColorLabel::Green)),
        Some("blue") => Ok(Some(silica_edit::ColorLabel::Blue)),
        Some("purple") => Ok(Some(silica_edit::ColorLabel::Purple)),
        Some(other) => Err(LibraryStorageError::SidecarValidation(format!(
            "unsupported sidecar color label: {other}"
        ))),
    }
}

fn color_label_value(label: Option<&str>) -> serde_json::Value {
    match label {
        Some(label) => serde_json::Value::String(label.to_string()),
        None => serde_json::Value::Null,
    }
}

pub(super) fn build_photo_sidecar_value(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    app_version: &str,
) -> Result<serde_json::Value, LibraryStorageError> {
    validate_sidecar_photo_id(photo_id)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let photo = load_sidecar_photo_row(&connection, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;
    let flags = get_photo_flags(&library.root_path, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;
    let mut graph = load_active_edit_graph_or_default(&library.root_path, photo_id)?
        .ok_or_else(|| LibraryStorageError::MissingPhoto(photo_id.to_string()))?;

    graph.app_version = Some(app_version.to_string());
    graph.metadata.rating = i64::from(flags.rating);
    graph.metadata.picked = flags.picked;
    graph.metadata.rejected = flags.rejected;
    graph.metadata.color_label = edit_color_label_from_catalog(flags.color_label.as_deref())?;
    silica_edit::validate_edit_graph(&graph)?;
    let edit_graph_json = serde_json::to_value(&graph)?;
    let written_at = current_timestamp_string();
    let catalog_edit_state_id = active_edit_state_id(&connection, photo_id)?;

    let value = serde_json::json!({
        "schema": SIDECAR_SCHEMA,
        "version": SIDECAR_VERSION,
        "app_version": app_version,
        "photo": {
            "photo_id": photo.photo_id,
            "original_path": photo.original_path,
            "file_name": photo.file_name,
            "fingerprint": {
                "file_size": photo.file_size,
                "modified_at": photo.modified_at.unwrap_or_else(|| "unknown".to_string()),
                "partial_hash": photo.partial_hash.unwrap_or_default(),
                "full_hash": photo.full_hash
            }
        },
        "edit_graph": edit_graph_json,
        "flags": {
            "rating": flags.rating,
            "picked": flags.picked,
            "rejected": flags.rejected,
            "color_label": color_label_value(flags.color_label.as_deref())
        },
        "sync": {
            "status": "in_sync",
            "catalog_edit_state_id": catalog_edit_state_id,
            "sidecar_hash": serde_json::Value::Null
        },
        "written_at": written_at
    });

    validate_sidecar_json(&value)?;
    Ok(value)
}

pub(super) fn validate_sidecar_json(value: &serde_json::Value) -> Result<(), LibraryStorageError> {
    let object = value.as_object().ok_or_else(|| {
        LibraryStorageError::SidecarValidation("sidecar root must be an object".to_string())
    })?;
    if object.get("schema").and_then(serde_json::Value::as_str) != Some(SIDECAR_SCHEMA) {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar schema marker must be silica.sidecar".to_string(),
        ));
    }
    if object.get("version").and_then(serde_json::Value::as_i64) != Some(SIDECAR_VERSION) {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar version must be 1".to_string(),
        ));
    }
    for required in [
        "app_version",
        "photo",
        "edit_graph",
        "flags",
        "sync",
        "written_at",
    ] {
        if !object.contains_key(required) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar missing required field: {required}"
            )));
        }
    }
    let allowed_top_level = [
        "schema",
        "version",
        "app_version",
        "photo",
        "edit_graph",
        "flags",
        "sync",
        "written_at",
    ];
    for key in object.keys() {
        if !allowed_top_level.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "unsupported top-level field: {key}"
            )));
        }
    }
    if !object
        .get("app_version")
        .is_some_and(serde_json::Value::is_string)
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.app_version must be a string".to_string(),
        ));
    }
    if !object
        .get("written_at")
        .is_some_and(serde_json::Value::is_string)
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.written_at must be a string".to_string(),
        ));
    }

    let photo = object
        .get("photo")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.photo must be an object".to_string())
        })?;
    let allowed_photo = ["photo_id", "original_path", "file_name", "fingerprint"];
    for key in photo.keys() {
        if !allowed_photo.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.photo contains unsupported field: {key}"
            )));
        }
    }
    for required in allowed_photo {
        if !photo.contains_key(required) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.photo missing required field: {required}"
            )));
        }
    }
    for key in ["photo_id", "original_path", "file_name"] {
        if !photo.get(key).is_some_and(serde_json::Value::is_string) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.photo.{key} must be a string"
            )));
        }
    }
    if !photo
        .get("fingerprint")
        .is_some_and(serde_json::Value::is_object)
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.photo.fingerprint must be an object".to_string(),
        ));
    }

    let flags = object
        .get("flags")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.flags must be an object".to_string())
        })?;
    let allowed_flags = ["rating", "picked", "rejected", "color_label"];
    for key in flags.keys() {
        if !allowed_flags.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.flags contains unsupported field: {key}"
            )));
        }
    }
    if flags
        .get("rating")
        .and_then(serde_json::Value::as_i64)
        .map_or(true, |rating| !(0..=5).contains(&rating))
    {
        return Err(LibraryStorageError::SidecarValidation(
            "sidecar.flags.rating must be 0..=5".to_string(),
        ));
    }
    for key in ["picked", "rejected"] {
        if !flags.get(key).is_some_and(serde_json::Value::is_boolean) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.flags.{key} must be boolean"
            )));
        }
    }
    match flags.get("color_label") {
        Some(value) if value.is_null() => {}
        Some(value) => {
            let label = value.as_str().ok_or_else(|| {
                LibraryStorageError::SidecarValidation(
                    "sidecar.flags.color_label must be string or null".to_string(),
                )
            })?;
            edit_color_label_from_catalog(Some(label))?;
        }
        None => {
            return Err(LibraryStorageError::SidecarValidation(
                "sidecar.flags.color_label is required".to_string(),
            ));
        }
    }

    let edit_graph = object.get("edit_graph").ok_or_else(|| {
        LibraryStorageError::SidecarValidation("sidecar.edit_graph is required".to_string())
    })?;
    silica_edit::validate_edit_graph_json(edit_graph)?;

    let sync = object
        .get("sync")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.sync must be an object".to_string())
        })?;
    let allowed_sync = ["status", "catalog_edit_state_id", "sidecar_hash"];
    for key in sync.keys() {
        if !allowed_sync.contains(&key.as_str()) {
            return Err(LibraryStorageError::SidecarValidation(format!(
                "sidecar.sync contains unsupported field: {key}"
            )));
        }
    }
    let status = sync
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.sync.status must be a string".to_string(),
            )
        })?;
    if ![
        "in_sync",
        "catalog_newer",
        "sidecar_newer",
        "conflict",
        "missing",
        "disabled",
    ]
    .contains(&status)
    {
        return Err(LibraryStorageError::SidecarValidation(format!(
            "sidecar.sync.status is unsupported: {status}"
        )));
    }

    Ok(())
}

fn parse_sidecar_flags(value: &serde_json::Value) -> Result<PhotoFlags, LibraryStorageError> {
    let photo_id = value["photo"]["photo_id"].as_str().ok_or_else(|| {
        LibraryStorageError::SidecarValidation(
            "sidecar.photo.photo_id must be a string".to_string(),
        )
    })?;
    let flags = value
        .get("flags")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation("sidecar.flags must be an object".to_string())
        })?;
    let rating = flags
        .get("rating")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.flags.rating must be an integer".to_string(),
            )
        })?;
    let picked = flags
        .get("picked")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.flags.picked must be boolean".to_string(),
            )
        })?;
    let rejected = flags
        .get("rejected")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.flags.rejected must be boolean".to_string(),
            )
        })?;
    let color_label = match flags.get("color_label") {
        Some(value) if value.is_null() => None,
        Some(value) => Some(
            value
                .as_str()
                .ok_or_else(|| {
                    LibraryStorageError::SidecarValidation(
                        "sidecar.flags.color_label must be string or null".to_string(),
                    )
                })?
                .to_string(),
        ),
        None => None,
    };

    PhotoFlags::new(
        photo_id.to_string(),
        rating as u8,
        picked,
        rejected,
        color_label,
    )
    .map_err(LibraryStorageError::from)
}

fn process_rebuild_dry_run_sidecar(
    connection: &Connection,
    sidecar_path: &Path,
    report: &mut CatalogRebuildDryRunReport,
) -> Result<(), LibraryStorageError> {
    report.sidecars_scanned += 1;

    let file_name = sidecar_path
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sidecar_relative_path = format!("{SIDECAR_DIRECTORY}/{file_name}");
    let expected_photo_id = file_name
        .strip_suffix(SIDECAR_FILE_SUFFIX)
        .unwrap_or_default()
        .to_string();

    if let Err(error) = validate_sidecar_photo_id(&expected_photo_id) {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::InvalidPathIdentity,
            Some(expected_photo_id),
            sidecar_relative_path,
            error.to_string(),
        );
        return Ok(());
    }

    let bytes = fs::read(sidecar_path)?;
    let json: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            push_rebuild_issue(
                report,
                CatalogRebuildDryRunIssueKind::MalformedJson,
                Some(expected_photo_id),
                sidecar_relative_path,
                error.to_string(),
            );
            return Ok(());
        }
    };

    if json.get("schema").and_then(serde_json::Value::as_str) != Some(SIDECAR_SCHEMA)
        || json.get("version").and_then(serde_json::Value::as_i64) != Some(SIDECAR_VERSION)
    {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::SchemaInvalid,
            Some(expected_photo_id),
            sidecar_relative_path,
            "sidecar schema marker or version is unsupported".to_string(),
        );
        return Ok(());
    }

    if let Err(error) = validate_sidecar_json(&json) {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::SchemaInvalid,
            Some(expected_photo_id.clone()),
            sidecar_relative_path.clone(),
            error.to_string(),
        );
        return Ok(());
    }

    let sidecar_photo_id = match json["photo"]["photo_id"].as_str() {
        Some(photo_id) => photo_id,
        None => {
            push_rebuild_issue(
                report,
                CatalogRebuildDryRunIssueKind::PhotoIdMismatch,
                Some(expected_photo_id),
                sidecar_relative_path,
                "sidecar.photo.photo_id is missing".to_string(),
            );
            return Ok(());
        }
    };

    if sidecar_photo_id != expected_photo_id {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::PhotoIdMismatch,
            Some(sidecar_photo_id.to_string()),
            sidecar_relative_path,
            format!(
                "sidecar path identity {expected_photo_id} does not match payload {sidecar_photo_id}"
            ),
        );
        return Ok(());
    }

    let sidecar_flags = parse_valid_rebuild_sidecar_flags(&json).ok();
    let metadata_flags = parse_valid_edit_graph_metadata_flags(&expected_photo_id, &json).ok();
    if let (Some(sidecar_flags), Some(metadata_flags)) = (&sidecar_flags, &metadata_flags) {
        if sidecar_flags != metadata_flags {
            push_rebuild_issue(
                report,
                CatalogRebuildDryRunIssueKind::FlagsMetadataConflict,
                Some(expected_photo_id.clone()),
                sidecar_relative_path.clone(),
                "sidecar.flags and edit_graph.metadata disagree; sidecar.flags would win"
                    .to_string(),
            );
        }
    }

    if let Some(snapshot) = parse_sidecar_photo_snapshot(&json) {
        report_catalog_reconcile_issues(
            connection,
            &expected_photo_id,
            &sidecar_relative_path,
            &snapshot,
            report,
        )?;
    }

    let (flag_source, resolved_flags) = match (sidecar_flags, metadata_flags) {
        (Some(flags), _) => (CatalogRebuildFlagSource::SidecarFlags, flags),
        (None, Some(flags)) => (CatalogRebuildFlagSource::EditGraphMetadata, flags),
        (None, None) => (
            CatalogRebuildFlagSource::Defaults,
            default_rebuild_flags(&expected_photo_id),
        ),
    };
    let catalog_flags = get_photo_flags_from_connection(connection, &expected_photo_id)?;
    let action = match &catalog_flags {
        None => CatalogRebuildDryRunAction::CreatePhotoFlags,
        Some(flags) if flags != &resolved_flags => CatalogRebuildDryRunAction::UpdatePhotoFlags,
        Some(_) => CatalogRebuildDryRunAction::KeepPhotoFlags,
    };

    report.entries.push(CatalogRebuildDryRunEntry {
        photo_id: expected_photo_id,
        sidecar_relative_path,
        action,
        flag_source,
        resolved_flags,
        catalog_flags,
    });

    Ok(())
}

fn push_rebuild_issue(
    report: &mut CatalogRebuildDryRunReport,
    kind: CatalogRebuildDryRunIssueKind,
    photo_id: Option<String>,
    sidecar_relative_path: String,
    message: String,
) {
    report.issues.push(CatalogRebuildDryRunIssue {
        kind,
        photo_id,
        sidecar_relative_path,
        message,
    });
}

fn parse_valid_rebuild_sidecar_flags(
    value: &serde_json::Value,
) -> Result<PhotoFlags, LibraryStorageError> {
    let flags = parse_sidecar_flags(value)?;
    if let Some(label) = flags.color_label.as_deref() {
        edit_color_label_from_catalog(Some(label))?;
    }
    Ok(flags)
}

fn parse_valid_edit_graph_metadata_flags(
    photo_id: &str,
    value: &serde_json::Value,
) -> Result<PhotoFlags, LibraryStorageError> {
    let metadata = value
        .get("edit_graph")
        .and_then(|edit_graph| edit_graph.get("metadata"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "sidecar.edit_graph.metadata must be an object".to_string(),
            )
        })?;

    let rating = metadata
        .get("rating")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "edit_graph.metadata.rating must be an integer".to_string(),
            )
        })?;
    let rating = u8::try_from(rating).map_err(|_| {
        LibraryStorageError::SidecarValidation(
            "edit_graph.metadata.rating must be 0..=5".to_string(),
        )
    })?;

    let picked = metadata
        .get("picked")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "edit_graph.metadata.picked must be boolean".to_string(),
            )
        })?;
    let rejected = metadata
        .get("rejected")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            LibraryStorageError::SidecarValidation(
                "edit_graph.metadata.rejected must be boolean".to_string(),
            )
        })?;
    let color_label = match metadata.get("color_label") {
        Some(value) if value.is_null() => None,
        Some(value) => {
            let label = value.as_str().ok_or_else(|| {
                LibraryStorageError::SidecarValidation(
                    "edit_graph.metadata.color_label must be string or null".to_string(),
                )
            })?;
            edit_color_label_from_catalog(Some(label))?;
            Some(label.to_string())
        }
        None => None,
    };

    PhotoFlags::new(photo_id.to_string(), rating, picked, rejected, color_label)
        .map_err(LibraryStorageError::from)
}

fn parse_sidecar_photo_snapshot(value: &serde_json::Value) -> Option<SidecarPhotoSnapshot> {
    let photo = value.get("photo")?.as_object()?;
    let fingerprint = photo.get("fingerprint")?.as_object()?;

    Some(SidecarPhotoSnapshot {
        original_path: photo.get("original_path")?.as_str()?.to_string(),
        file_name: photo.get("file_name")?.as_str()?.to_string(),
        file_size: fingerprint
            .get("file_size")
            .and_then(serde_json::Value::as_i64),
        modified_at: fingerprint
            .get("modified_at")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        partial_hash: fingerprint
            .get("partial_hash")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        full_hash: fingerprint.get("full_hash").and_then(|value| {
            if value.is_null() {
                None
            } else {
                value.as_str().map(ToOwned::to_owned)
            }
        }),
    })
}

fn report_catalog_reconcile_issues(
    connection: &Connection,
    photo_id: &str,
    sidecar_relative_path: &str,
    snapshot: &SidecarPhotoSnapshot,
    report: &mut CatalogRebuildDryRunReport,
) -> Result<(), LibraryStorageError> {
    let Some(catalog_photo) = load_sidecar_photo_row(connection, photo_id)? else {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::CatalogReconcileConflict,
            Some(photo_id.to_string()),
            sidecar_relative_path.to_string(),
            "catalog photo is missing; rebuild would depend on sidecar photo data".to_string(),
        );
        return Ok(());
    };

    let mut mismatches = Vec::new();
    if catalog_photo.original_path != snapshot.original_path {
        mismatches.push("original_path");
    }
    if catalog_photo.file_name != snapshot.file_name {
        mismatches.push("file_name");
    }
    if snapshot
        .file_size
        .is_some_and(|file_size| file_size != catalog_photo.file_size)
    {
        mismatches.push("file_size");
    }
    if snapshot
        .modified_at
        .as_ref()
        .is_some_and(|modified_at| Some(modified_at) != catalog_photo.modified_at.as_ref())
    {
        mismatches.push("modified_at");
    }
    if snapshot
        .partial_hash
        .as_ref()
        .is_some_and(|partial_hash| Some(partial_hash) != catalog_photo.partial_hash.as_ref())
    {
        mismatches.push("partial_hash");
    }
    if snapshot
        .full_hash
        .as_ref()
        .is_some_and(|full_hash| Some(full_hash) != catalog_photo.full_hash.as_ref())
    {
        mismatches.push("full_hash");
    }

    if !mismatches.is_empty() {
        push_rebuild_issue(
            report,
            CatalogRebuildDryRunIssueKind::CatalogReconcileConflict,
            Some(photo_id.to_string()),
            sidecar_relative_path.to_string(),
            format!(
                "catalog photo differs from sidecar fields: {}",
                mismatches.join(", ")
            ),
        );
    }

    Ok(())
}

fn update_sidecar_status(
    catalog_path: &Path,
    photo_id: &str,
    sidecar_relative_path: &str,
    written_at: &str,
) -> Result<(), LibraryStorageError> {
    let connection = open_catalog(catalog_path)?;
    connection.execute(
        r#"
        INSERT INTO sidecar_status(photo_id, sidecar_path, last_written_at, conflict_state)
        VALUES (?1, ?2, ?3, 'clean')
        ON CONFLICT(photo_id) DO UPDATE SET
          sidecar_path = excluded.sidecar_path,
          last_written_at = excluded.last_written_at,
          conflict_state = 'clean'
        "#,
        params![photo_id, sidecar_relative_path, written_at],
    )?;
    Ok(())
}
