use std::fs;
use std::path::Path;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use silica_catalog::CatalogFlagError;

use crate::common::{path_to_string, stable_catalog_id};
use crate::{
    inspect_local_library_for_restore, open_catalog, open_existing_library_for_read,
    open_local_library, CacheClearSummary, CacheDirectoryStatus, CacheRecord, CacheStatusSummary,
    LibraryStorageError, DISPOSABLE_CACHE_DIRECTORIES, HISTOGRAM_CACHE_TYPE,
    MASK_RASTER_CACHE_TYPE, PREVIEW_CACHE_TYPE, THUMBNAIL_CACHE_TYPE,
};

/// Record a disposable JPEG thumbnail cache file for a catalog photo.
pub fn record_thumbnail_cache(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    record_photo_cache(
        library_root_path,
        "cache-thumbnail",
        photo_id,
        THUMBNAIL_CACHE_TYPE,
        cache_key,
        path,
        byte_size,
    )
}

/// Record a disposable JPEG Loupe preview cache file for a catalog photo.
pub fn record_preview_cache(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    record_photo_cache(
        library_root_path,
        "cache-preview",
        photo_id,
        PREVIEW_CACHE_TYPE,
        cache_key,
        path,
        byte_size,
    )
}

/// Record disposable histogram cache data for a catalog photo.
pub fn record_histogram_cache(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    record_photo_cache(
        library_root_path,
        "cache-histogram",
        photo_id,
        HISTOGRAM_CACHE_TYPE,
        cache_key,
        path,
        byte_size,
    )
}

/// Record disposable manual brush alpha raster cache data for a catalog photo.
pub fn record_mask_raster_cache(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    let cache_key = cache_key.as_ref();
    let cache_id_namespace = format!("cache-mask-raster-{cache_key}");
    record_photo_cache(
        library_root_path,
        &cache_id_namespace,
        photo_id,
        MASK_RASTER_CACHE_TYPE,
        cache_key,
        path,
        byte_size,
    )
}

/// Read a disposable cache record for one catalog photo and cache type.
pub fn get_photo_cache_record(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
    cache_type: &str,
) -> Result<Option<CacheRecord>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    connection
        .query_row(
            r#"
            SELECT id, photo_id, cache_type, cache_key, path, byte_size
            FROM cache_records
            WHERE photo_id = ?1 AND cache_type = ?2
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            params![photo_id, cache_type],
            |row| {
                Ok(CacheRecord {
                    id: row.get(0)?,
                    photo_id: row.get(1)?,
                    cache_type: row.get(2)?,
                    cache_key: row.get(3)?,
                    path: row.get(4)?,
                    byte_size: row.get(5)?,
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

/// Read disposable cache directory status without clearing or repairing cache directories.
pub fn get_disposable_cache_status(
    library_root_path: impl AsRef<Path>,
) -> Result<CacheStatusSummary, LibraryStorageError> {
    let library = inspect_local_library_for_restore(library_root_path)?;
    let connection =
        Connection::open_with_flags(&library.catalog_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let cache_record_count =
        connection.query_row("SELECT COUNT(*) FROM cache_records", [], |row| {
            row.get::<_, i64>(0)
        })?;

    let mut total_bytes = 0_u64;
    let mut directories = Vec::with_capacity(DISPOSABLE_CACHE_DIRECTORIES.len());
    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        let path = library.root_path.join(directory);
        let (byte_size, file_count) = cache_directory_size(&path)?;
        total_bytes = total_bytes.saturating_add(byte_size);
        directories.push(CacheDirectoryStatus {
            name: (*directory).to_string(),
            path,
            exists: library.root_path.join(directory).is_dir(),
            byte_size,
            file_count,
        });
    }

    Ok(CacheStatusSummary {
        library_root_path: library.root_path,
        directories,
        total_bytes,
        cache_record_count: usize::try_from(cache_record_count).unwrap_or(usize::MAX),
        message: "Cache status covers disposable library caches only.".to_string(),
    })
}

/// Clear only disposable cache directories and cache record metadata.
pub fn clear_disposable_cache(
    library_root_path: impl AsRef<Path>,
) -> Result<CacheClearSummary, LibraryStorageError> {
    let library = open_local_library(library_root_path)?;
    let mut cleared_directories = Vec::with_capacity(DISPOSABLE_CACHE_DIRECTORIES.len());
    let mut recreated_directories = Vec::with_capacity(DISPOSABLE_CACHE_DIRECTORIES.len());

    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        let path = library.root_path.join(directory);
        clear_disposable_cache_directory(&path)?;
        cleared_directories.push((*directory).to_string());
        recreated_directories.push((*directory).to_string());
    }

    let connection = open_catalog(&library.catalog_path)?;
    let removed_cache_records = connection.execute("DELETE FROM cache_records", [])?;

    Ok(CacheClearSummary {
        cleared_directories,
        recreated_directories,
        removed_cache_records,
        message: "Cache clear removed only disposable library caches.".to_string(),
    })
}

fn clear_disposable_cache_directory(path: &Path) -> Result<(), LibraryStorageError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            let file_type = metadata.file_type();
            if file_type.is_symlink() || file_type.is_file() {
                fs::remove_file(path)?;
            } else if file_type.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                return Err(LibraryStorageError::CacheValidation(format!(
                    "disposable cache path is not removable cache material: {}",
                    path.display()
                )));
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn record_photo_cache(
    library_root_path: impl AsRef<Path>,
    cache_id_namespace: &str,
    photo_id: &str,
    cache_type: &str,
    cache_key: impl AsRef<str>,
    path: impl AsRef<Path>,
    byte_size: i64,
) -> Result<CacheRecord, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    validate_cache_path(&library.root_path, cache_type, path.as_ref())?;
    let connection = open_catalog(&library.catalog_path)?;
    let cache_id = stable_catalog_id(cache_id_namespace, photo_id);
    let cache_key = cache_key.as_ref().to_string();
    let path = path_to_string(path.as_ref())?;
    let cache_type = cache_type.to_string();

    connection.execute(
        r#"
        INSERT INTO cache_records(
          id,
          photo_id,
          cache_type,
          cache_key,
          path,
          byte_size,
          created_at,
          last_accessed_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
          cache_key = excluded.cache_key,
          path = excluded.path,
          byte_size = excluded.byte_size,
          created_at = CURRENT_TIMESTAMP,
          last_accessed_at = CURRENT_TIMESTAMP
        "#,
        params![cache_id, photo_id, cache_type, cache_key, path, byte_size,],
    )?;

    Ok(CacheRecord {
        id: cache_id,
        photo_id: Some(photo_id.to_string()),
        cache_type,
        cache_key,
        path,
        byte_size,
    })
}

fn cache_directory_size(path: &Path) -> Result<(u64, u64), LibraryStorageError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((0, 0)),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_file() {
        return Ok((metadata.len(), 1));
    }
    if !metadata.file_type().is_dir() {
        return Ok((0, 0));
    }

    let mut total_bytes = 0_u64;
    let mut file_count = 0_u64;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let (entry_bytes, entry_files) = cache_directory_size(&entry.path())?;
        total_bytes = total_bytes.saturating_add(entry_bytes);
        file_count = file_count.saturating_add(entry_files);
    }

    Ok((total_bytes, file_count))
}

fn validate_cache_path(
    library_root_path: &Path,
    cache_type: &str,
    path: &Path,
) -> Result<(), LibraryStorageError> {
    let expected_directory = match cache_type {
        THUMBNAIL_CACHE_TYPE => Some("thumbnails"),
        PREVIEW_CACHE_TYPE => Some("previews"),
        HISTOGRAM_CACHE_TYPE => Some("render-cache"),
        MASK_RASTER_CACHE_TYPE => Some("render-cache/masks"),
        _ => None,
    };
    let Some(expected_directory) = expected_directory else {
        return Ok(());
    };
    let expected_root = library_root_path.join(expected_directory);
    let expected_root = std::fs::canonicalize(&expected_root).map_err(|error| {
        LibraryStorageError::CacheValidation(format!(
            "{cache_type} cache root must resolve before recording cache metadata: {} ({error})",
            expected_root.display()
        ))
    })?;
    let resolved_path = std::fs::canonicalize(path).map_err(|error| {
        LibraryStorageError::CacheValidation(format!(
            "{cache_type} cache path must resolve before recording cache metadata: {} ({error})",
            path.display()
        ))
    })?;
    if resolved_path.starts_with(&expected_root) {
        return Ok(());
    }

    Err(LibraryStorageError::CacheValidation(format!(
        "{cache_type} cache path must stay under disposable {expected_directory}/ cache directory: {}",
        path.display()
    )))
}
