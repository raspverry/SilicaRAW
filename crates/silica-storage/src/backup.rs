use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::common::current_timestamp_string;
use crate::{
    open_catalog, open_existing_library_for_read, open_local_library, LibraryBackupResult,
    LibraryRestoreResult, LibraryStorageError, LocalLibrary, BACKUPS_DIRECTORY,
    BACKUP_MANIFEST_FILE, BACKUP_SCHEMA, BACKUP_VERSION, CATALOG_DATABASE_FILE,
    CURRENT_SCHEMA_VERSION, SIDECAR_DIRECTORY,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackupManifest {
    catalog_schema_version: i64,
    files: Vec<String>,
}

/// Create a checkpointed backup artifact with catalog data, sidecars, and a manifest.
pub fn create_library_backup(
    library_root_path: impl AsRef<Path>,
    app_version: &str,
) -> Result<LibraryBackupResult, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    checkpoint_catalog_for_backup(&library.catalog_path)?;

    let backup_id = current_backup_id();
    let backups_root = library.root_path.join(BACKUPS_DIRECTORY);
    fs::create_dir_all(&backups_root)?;
    let backup_path = backups_root.join(&backup_id);
    if backup_path.exists() {
        return Err(LibraryStorageError::BackupValidation(format!(
            "backup artifact already exists: {}",
            backup_path.display()
        )));
    }
    fs::create_dir(&backup_path)?;

    let mut files = Vec::new();
    let mut bytes_copied = 0_u64;
    bytes_copied += copy_backup_file(
        &library.catalog_path,
        &backup_path.join(CATALOG_DATABASE_FILE),
        CATALOG_DATABASE_FILE,
        &mut files,
    )?;
    bytes_copied += copy_backup_directory(
        &library.root_path.join(SIDECAR_DIRECTORY),
        &backup_path.join(SIDECAR_DIRECTORY),
        SIDECAR_DIRECTORY,
        &mut files,
    )?;
    files.sort();

    let created_at = current_timestamp_string();
    let manifest_files = files.clone();
    let manifest = serde_json::json!({
        "schema": BACKUP_SCHEMA,
        "version": BACKUP_VERSION,
        "app_version": app_version,
        "catalog_schema_version": library.schema_version,
        "created_at": created_at,
        "checkpoint": "wal_checkpoint_truncate",
        "files": manifest_files,
        "excluded": [
            "original referenced photo files",
            "thumbnails/",
            "previews/",
            "render-cache/",
            "ai-cache/",
            "exports/",
            "logs/",
            "backups/"
        ]
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let manifest_path = backup_path.join(BACKUP_MANIFEST_FILE);
    fs::write(&manifest_path, &manifest_bytes)?;
    bytes_copied += manifest_bytes.len() as u64;

    Ok(LibraryBackupResult {
        backup_id,
        backup_path,
        manifest_path,
        created_at,
        files,
        bytes_copied,
    })
}

/// Restore a backup artifact into an empty or rollback-protected library root.
pub fn restore_library_backup(
    backup_path: impl AsRef<Path>,
    target_root_path: impl AsRef<Path>,
) -> Result<LibraryRestoreResult, LibraryStorageError> {
    let backup_path = backup_path.as_ref();
    let target_root_path = target_root_path.as_ref();
    let manifest = read_backup_manifest(backup_path)?;
    let staging_root = restore_staging_path(target_root_path);
    if staging_root.exists() {
        return Err(LibraryStorageError::BackupValidation(format!(
            "restore staging path already exists: {}",
            staging_root.display()
        )));
    }

    copy_backup_payload_to_root(backup_path, &staging_root)?;
    let staging_library = match open_local_library(&staging_root) {
        Ok(library) => library,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_root);
            return Err(LibraryStorageError::BackupValidation(format!(
                "restore staging validation failed for backup {} targeting {}: {error}",
                backup_path.display(),
                target_root_path.display()
            )));
        }
    };
    drop(staging_library);

    let rollback_path = if target_has_library_state(target_root_path)? {
        Some(create_restore_rollback(target_root_path)?)
    } else {
        None
    };

    let restored_library = apply_staged_restore(&staging_root, target_root_path)?;

    Ok(LibraryRestoreResult {
        restored_library,
        backup_path: backup_path.to_path_buf(),
        rollback_path,
        restored_files: manifest.files,
    })
}

fn checkpoint_catalog_for_backup(catalog_path: &Path) -> Result<(), LibraryStorageError> {
    let connection = open_catalog(catalog_path)?;
    let (busy, _log_frames, _checkpointed_frames): (i64, i64, i64) =
        connection.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
    if busy != 0 {
        return Err(LibraryStorageError::BackupValidation(
            "catalog WAL checkpoint could not complete cleanly".to_string(),
        ));
    }
    Ok(())
}

fn read_backup_manifest(backup_path: &Path) -> Result<BackupManifest, LibraryStorageError> {
    if !backup_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(backup_path.to_path_buf()));
    }

    let manifest_path = backup_path.join(BACKUP_MANIFEST_FILE);
    let manifest: serde_json::Value = serde_json::from_slice(&fs::read(&manifest_path)?)?;

    if manifest.get("schema").and_then(serde_json::Value::as_str) != Some(BACKUP_SCHEMA) {
        return Err(LibraryStorageError::BackupValidation(
            "backup schema marker must be silica.backup".to_string(),
        ));
    }
    if manifest.get("version").and_then(serde_json::Value::as_i64) != Some(BACKUP_VERSION) {
        return Err(LibraryStorageError::BackupValidation(
            "backup version must be 1".to_string(),
        ));
    }
    let catalog_schema_version = manifest
        .get("catalog_schema_version")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| {
            LibraryStorageError::BackupValidation(
                "backup catalog_schema_version must be an integer".to_string(),
            )
        })?;
    if catalog_schema_version > CURRENT_SCHEMA_VERSION {
        return Err(LibraryStorageError::BackupValidation(format!(
            "backup uses newer catalog schema {catalog_schema_version}; app supports {CURRENT_SCHEMA_VERSION}"
        )));
    }

    let files = manifest
        .get("files")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| {
            LibraryStorageError::BackupValidation("backup files must be an array".to_string())
        })?
        .iter()
        .map(|value| {
            let file = value.as_str().ok_or_else(|| {
                LibraryStorageError::BackupValidation(
                    "backup files entries must be strings".to_string(),
                )
            })?;
            validate_backup_relative_path(file)?;
            Ok(file.to_string())
        })
        .collect::<Result<Vec<_>, LibraryStorageError>>()?;

    if !files.iter().any(|file| file == CATALOG_DATABASE_FILE) {
        return Err(LibraryStorageError::BackupValidation(
            "backup files must include catalog.db".to_string(),
        ));
    }

    Ok(BackupManifest {
        catalog_schema_version,
        files,
    })
}

fn validate_backup_relative_path(path: &str) -> Result<(), LibraryStorageError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.split('/').any(|part| part == "." || part == "..")
    {
        return Err(LibraryStorageError::BackupValidation(format!(
            "backup contains unsafe relative path: {path:?}"
        )));
    }
    Ok(())
}

fn copy_backup_file(
    source: &Path,
    destination: &Path,
    relative_path: &str,
    files: &mut Vec<String>,
) -> Result<u64, LibraryStorageError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = fs::copy(source, destination)?;
    files.push(relative_path.to_string());
    Ok(bytes)
}

fn copy_backup_directory(
    source_root: &Path,
    destination_root: &Path,
    relative_root: &str,
    files: &mut Vec<String>,
) -> Result<u64, LibraryStorageError> {
    fs::create_dir_all(destination_root)?;
    if !source_root.is_dir() {
        return Ok(0);
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(source_root)? {
        entries.push(entry?);
    }
    entries.sort_by(|left, right| left.file_name().cmp(&right.file_name()));

    let mut bytes = 0_u64;
    for entry in entries {
        let file_type = entry.file_type()?;
        let name = entry
            .file_name()
            .to_str()
            .map(ToOwned::to_owned)
            .ok_or_else(|| LibraryStorageError::InvalidPath(entry.path()))?;
        let relative_path = format!("{relative_root}/{name}");
        let destination = destination_root.join(&name);

        if file_type.is_symlink() {
            return Err(LibraryStorageError::BackupValidation(format!(
                "backup refuses symlink in durable data: {relative_path}"
            )));
        }
        if file_type.is_file() && name.ends_with(".tmp") {
            continue;
        }
        if file_type.is_dir() {
            bytes += copy_backup_directory(&entry.path(), &destination, &relative_path, files)?;
        } else if file_type.is_file() {
            bytes += copy_backup_file(&entry.path(), &destination, &relative_path, files)?;
        }
    }

    Ok(bytes)
}

fn copy_backup_payload_to_root(
    backup_path: &Path,
    target_root: &Path,
) -> Result<(), LibraryStorageError> {
    fs::create_dir_all(target_root)?;
    copy_backup_file(
        &backup_path.join(CATALOG_DATABASE_FILE),
        &target_root.join(CATALOG_DATABASE_FILE),
        CATALOG_DATABASE_FILE,
        &mut Vec::new(),
    )?;
    copy_backup_directory(
        &backup_path.join(SIDECAR_DIRECTORY),
        &target_root.join(SIDECAR_DIRECTORY),
        SIDECAR_DIRECTORY,
        &mut Vec::new(),
    )?;
    Ok(())
}

fn target_has_library_state(target_root: &Path) -> Result<bool, LibraryStorageError> {
    if !target_root.exists() {
        return Ok(false);
    }
    if !target_root.is_dir() {
        return Err(LibraryStorageError::NotDirectory(target_root.to_path_buf()));
    }
    Ok(target_root.join(CATALOG_DATABASE_FILE).exists()
        || target_root.join(SIDECAR_DIRECTORY).exists())
}

fn create_restore_rollback(target_root: &Path) -> Result<PathBuf, LibraryStorageError> {
    let rollback_path = target_root
        .join(BACKUPS_DIRECTORY)
        .join(current_restore_rollback_id());
    if rollback_path.exists() {
        return Err(LibraryStorageError::BackupValidation(format!(
            "restore rollback path already exists: {}",
            rollback_path.display()
        )));
    }
    fs::create_dir_all(&rollback_path)?;

    if target_root.join(CATALOG_DATABASE_FILE).is_file() {
        copy_backup_file(
            &target_root.join(CATALOG_DATABASE_FILE),
            &rollback_path.join(CATALOG_DATABASE_FILE),
            CATALOG_DATABASE_FILE,
            &mut Vec::new(),
        )?;
    }
    if target_root.join(SIDECAR_DIRECTORY).is_dir() {
        copy_backup_directory(
            &target_root.join(SIDECAR_DIRECTORY),
            &rollback_path.join(SIDECAR_DIRECTORY),
            SIDECAR_DIRECTORY,
            &mut Vec::new(),
        )?;
    }

    Ok(rollback_path)
}

fn apply_staged_restore(
    staging_root: &Path,
    target_root: &Path,
) -> Result<LocalLibrary, LibraryStorageError> {
    if !target_root.exists() {
        fs::rename(staging_root, target_root)?;
    } else if is_empty_directory(target_root)? {
        fs::remove_dir(target_root)?;
        fs::rename(staging_root, target_root)?;
    } else {
        fs::copy(
            staging_root.join(CATALOG_DATABASE_FILE),
            target_root.join(CATALOG_DATABASE_FILE),
        )?;
        let target_sidecars = target_root.join(SIDECAR_DIRECTORY);
        if target_sidecars.exists() {
            fs::remove_dir_all(&target_sidecars)?;
        }
        copy_backup_directory(
            &staging_root.join(SIDECAR_DIRECTORY),
            &target_sidecars,
            SIDECAR_DIRECTORY,
            &mut Vec::new(),
        )?;
        fs::remove_dir_all(staging_root)?;
    }

    open_local_library(target_root)
}

fn is_empty_directory(path: &Path) -> Result<bool, LibraryStorageError> {
    if !path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(path.to_path_buf()));
    }
    Ok(fs::read_dir(path)?.next().is_none())
}

fn current_backup_id() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0));
    format!(
        "backup-unix-{}-{}",
        duration.as_secs(),
        duration.subsec_nanos()
    )
}

fn current_restore_rollback_id() -> String {
    format!("restore-rollback-{}", current_backup_id())
}

pub(super) fn restore_staging_path(target_root: &Path) -> PathBuf {
    let parent = target_root.parent().unwrap_or_else(|| Path::new("."));
    let name = target_root
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "library".to_string());
    parent.join(format!(".{name}.restore-staging-{}", current_backup_id()))
}
