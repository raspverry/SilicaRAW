use std::fs;
use std::path::Path;
use std::path::PathBuf;

use rusqlite::{named_params, params, Connection, OptionalExtension};
use silica_catalog::{
    is_supported_photo_extension, CatalogFlagError, ImportCandidate, ImportIssue, ImportIssueKind,
    LibraryQueryFileType, LibraryQueryFilters, LibraryQueryMetadataFilter, LibraryQueryPage,
    LibraryQueryRequest, LibraryQuerySort, PhotoFlags, ALPHA_MAX_RATING,
};

use super::common::{
    bool_to_sql, full_file_sha256, modified_at_string, partial_file_hash, path_to_string,
    sql_to_bool, stable_catalog_id,
};
use super::{
    invalidate_redo_history, mark_clean_sidecar_catalog_newer_after_history_commit,
    next_history_sequence, open_catalog, open_existing_library_for_read,
    open_existing_library_for_read_only_query, photo_flags_action_value,
    restore_photo_flags_in_transaction, FolderImportOptions, FolderImportSummary,
    LibraryPhotoGridItem, LibraryStorageError, MetadataDimensionSource, MetadataExtractionPolicy,
    PhotoMetadata, PhotoMetadataField, PhotoMetadataUpdate, PhotoPreviewCandidate, ACTION_SCHEMA,
    ACTION_VERSION, LOCAL_LIBRARY_ID, THUMBNAIL_CACHE_TYPE,
};

/// Return the metadata extraction policy for one original path.
pub fn metadata_extraction_policy_for_path(path: &Path) -> MetadataExtractionPolicy {
    let is_supported_raster = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(is_supported_photo_extension);

    MetadataExtractionPolicy {
        dimension_source: if is_supported_raster {
            MetadataDimensionSource::ExistingRasterPath
        } else {
            MetadataDimensionSource::Unavailable
        },
        raw_decode_supported: false,
        camera_lens_available: false,
    }
}

/// Scan a selected folder and record file candidates by reference.
pub fn import_folder(
    library_root_path: impl AsRef<Path>,
    folder_path: impl AsRef<Path>,
) -> Result<FolderImportSummary, LibraryStorageError> {
    import_folder_with_options(
        library_root_path,
        folder_path,
        FolderImportOptions::default(),
    )
}

/// Scan a selected folder and record file candidates by reference.
pub fn import_folder_with_options(
    library_root_path: impl AsRef<Path>,
    folder_path: impl AsRef<Path>,
    options: FolderImportOptions,
) -> Result<FolderImportSummary, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let folder_path = folder_path.as_ref();
    let root_policy_issue = import_root_policy_issue(folder_path)?;
    if let Some(issue) = root_policy_issue {
        return Ok(empty_import_summary(folder_path.to_path_buf(), vec![issue]));
    }
    if !folder_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(folder_path.to_path_buf()));
    }

    let (mut candidates, mut issues) = scan_import_candidates(folder_path, options)?;
    candidates.sort_by(|left, right| left.path.cmp(&right.path));
    issues.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.kind.as_str().cmp(right.kind.as_str()))
    });

    let mut connection = open_catalog(&library.catalog_path)?;
    record_import_candidates(&mut connection, folder_path, &candidates)?;

    let unsupported_files = candidates
        .iter()
        .filter(|candidate| candidate.unsupported)
        .count();
    let scanned_files = candidates.len();

    Ok(FolderImportSummary {
        folder_path: folder_path.to_path_buf(),
        scanned_files,
        supported_files: scanned_files - unsupported_files,
        unsupported_files,
        originals_unchanged: import_candidates_fingerprints_unchanged(&candidates),
        candidates,
        issues,
    })
}

fn empty_import_summary(folder_path: PathBuf, issues: Vec<ImportIssue>) -> FolderImportSummary {
    FolderImportSummary {
        folder_path,
        scanned_files: 0,
        supported_files: 0,
        unsupported_files: 0,
        originals_unchanged: true,
        candidates: Vec::new(),
        issues,
    }
}

/// Insert or update normalized metadata for an imported photo by original path.
pub fn upsert_photo_metadata_by_path(
    library_root_path: impl AsRef<Path>,
    original_path: impl AsRef<Path>,
    metadata: PhotoMetadataUpdate,
) -> Result<(), LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let original_path = path_to_string(original_path.as_ref())?;
    let connection = open_catalog(&library.catalog_path)?;
    connection.execute(
        r#"
        INSERT INTO photo_metadata(
          photo_id,
          camera_make,
          camera_model,
          lens_model,
          capture_time,
          raw_json,
          width,
          height,
          orientation
        )
        SELECT
          photos.id,
          ?2,
          ?3,
          ?4,
          ?5,
          ?6,
          ?7,
          ?8,
          ?9
        FROM photos
        WHERE photos.library_id = ?1
          AND photos.path = ?10
          AND photos.unsupported = 0
        ON CONFLICT(photo_id) DO UPDATE SET
          camera_make = excluded.camera_make,
          camera_model = excluded.camera_model,
          lens_model = excluded.lens_model,
          capture_time = excluded.capture_time,
          raw_json = excluded.raw_json,
          width = excluded.width,
          height = excluded.height,
          orientation = excluded.orientation
        "#,
        params![
            LOCAL_LIBRARY_ID,
            metadata.camera_make,
            metadata.camera_model,
            metadata.lens_model,
            metadata.capture_time,
            metadata.raw_json,
            metadata.width,
            metadata.height,
            metadata.orientation,
            original_path,
        ],
    )?;

    Ok(())
}

/// Read stored metadata for one photo without touching original files.
pub fn get_photo_metadata(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoMetadata>, LibraryStorageError> {
    let (_library, connection) = open_existing_library_for_read_only_query(library_root_path)?;
    connection
        .query_row(
            r#"
            SELECT
              photos.id,
              photos.file_name,
              photos.path,
              photos.file_type,
              photos.unsupported,
              photos.file_size,
              photos.modified_at,
              photo_metadata.photo_id IS NOT NULL,
              photo_metadata.width,
              photo_metadata.height,
              photo_metadata.orientation,
              photo_metadata.capture_time,
              photo_metadata.camera_make,
              photo_metadata.camera_model,
              photo_metadata.lens_model
            FROM photos
            LEFT JOIN photo_metadata ON photo_metadata.photo_id = photos.id
            WHERE photos.library_id = ?1
              AND photos.id = ?2
            "#,
            params![LOCAL_LIBRARY_ID, photo_id],
            photo_metadata_from_row,
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

/// List imported catalog photos for the Library grid without touching originals.
pub fn list_library_photos(
    library_root_path: impl AsRef<Path>,
) -> Result<Vec<LibraryPhotoGridItem>, LibraryStorageError> {
    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let mut statement = connection.prepare(
        r#"
        SELECT
          photos.id,
          photos.file_name,
          photos.path,
          photos.missing,
          photos.unsupported,
          photos.file_type,
          photo_flags.rating,
          photo_flags.picked,
          photo_flags.rejected,
          photo_flags.color_label,
          thumbnail_cache.path,
          thumbnail_cache.cache_key
        FROM photos
        LEFT JOIN photo_flags ON photo_flags.photo_id = photos.id
        LEFT JOIN cache_records AS thumbnail_cache
          ON thumbnail_cache.photo_id = photos.id
          AND thumbnail_cache.cache_type = ?1
        ORDER BY photos.file_name ASC, photos.path ASC
        "#,
    )?;
    let rows = statement.query_map([THUMBNAIL_CACHE_TYPE], library_photo_grid_item_from_row)?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(LibraryStorageError::from)
}

/// Query imported catalog photos by bounded page without touching original files.
/// Legacy catalogs may be migrated before the read-only page query runs.
pub fn query_library_photos(
    library_root_path: impl AsRef<Path>,
    request: LibraryQueryRequest,
) -> Result<LibraryQueryPage<LibraryPhotoGridItem>, LibraryStorageError> {
    let request = LibraryQueryRequest::new(
        request.offset,
        request.limit,
        request.sort,
        request.filters.clone(),
    );
    let (_library, connection) = open_existing_library_for_read_only_query(library_root_path)?;
    let filter = LibraryQuerySqlFilter::from(&request.filters);
    let order_clause = library_query_order_clause(request.sort);

    let total_count = query_library_photo_count(&connection, &filter)?;
    let limit = i64::from(request.limit);
    let offset = i64::try_from(request.offset).unwrap_or(i64::MAX);
    let query_sql =
        format!("{LIBRARY_QUERY_SELECT_SQL}\n{order_clause}\nLIMIT :limit OFFSET :offset");
    let mut statement = connection.prepare(&query_sql)?;
    let rows = statement.query_map(
        named_params! {
            ":thumbnail_cache_type": THUMBNAIL_CACHE_TYPE,
            ":library_id": LOCAL_LIBRARY_ID,
            ":min_rating": filter.min_rating,
            ":picked": filter.picked,
            ":rejected": filter.rejected,
            ":file_type": filter.file_type,
            ":metadata_filter": filter.metadata,
            ":search": filter.search.as_deref(),
            ":limit": limit,
            ":offset": offset,
        },
        library_photo_grid_item_from_row,
    )?;
    let items = rows.collect::<Result<Vec<_>, _>>()?;
    let has_next_page = request.offset.saturating_add(u64::from(request.limit)) < total_count;

    Ok(LibraryQueryPage {
        items,
        offset: request.offset,
        limit: request.limit,
        total_count,
        has_next_page,
        order_fields: request.order_fields(),
    })
}

/// Persist culling and label flags for a photo in the catalog.
pub fn set_photo_flags(
    library_root_path: impl AsRef<Path>,
    photo_id: impl Into<String>,
    rating: u8,
    picked: bool,
    rejected: bool,
    color_label: Option<String>,
) -> Result<PhotoFlags, LibraryStorageError> {
    let flags = PhotoFlags::new(photo_id, rating, picked, rejected, color_label)?;
    let library = open_existing_library_for_read(library_root_path)?;
    let mut connection = open_catalog(&library.catalog_path)?;
    let before_flags = get_photo_flags_from_connection(&connection, &flags.photo_id)?
        .unwrap_or_else(|| default_rebuild_flags(&flags.photo_id));
    let action_json = serde_json::to_string(&serde_json::json!({
        "schema": ACTION_SCHEMA,
        "version": ACTION_VERSION,
        "class": "undoable",
        "kind": "flag_change",
        "photo_id": flags.photo_id.clone(),
        "label": "Culling flags",
        "before": {
            "flags": photo_flags_action_value(&before_flags),
        },
        "after": {
            "flags": photo_flags_action_value(&flags),
        },
        "created_by": "core",
    }))?;

    let transaction = connection.transaction()?;
    invalidate_redo_history(&transaction, &flags.photo_id)?;
    restore_photo_flags_in_transaction(&transaction, &flags)?;
    let sequence = next_history_sequence(&transaction, &flags.photo_id)?;
    let edit_history_id = stable_catalog_id(
        "edit-history",
        &format!("{}\nflag_change\n{sequence}", flags.photo_id),
    );
    transaction.execute(
        r#"
        INSERT INTO edit_history(
          id,
          photo_id,
          edit_state_id,
          action_json,
          sequence,
          action_class,
          action_kind,
          history_state
        )
        VALUES (?1, ?2, NULL, ?3, ?4, 'undoable', 'flag_change', 'applied')
        "#,
        params![edit_history_id, flags.photo_id, action_json, sequence],
    )?;
    mark_clean_sidecar_catalog_newer_after_history_commit(&transaction, &flags.photo_id)?;
    transaction.commit()?;

    Ok(flags)
}

/// Read culling and label flags for a photo from the authoritative catalog row.
pub fn get_photo_flags(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoFlags>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    let row = connection
        .query_row(
            r#"
            SELECT photo_id, rating, picked, rejected, color_label
            FROM photo_flags
            WHERE photo_id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .optional()?;

    row.map(|(photo_id, rating, picked, rejected, color_label)| {
        photo_flags_from_row(photo_id, rating, picked, rejected, color_label)
    })
    .transpose()
}

/// Read the catalog fields needed to decide whether a photo can open a preview.
pub fn get_photo_preview_candidate(
    library_root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<Option<PhotoPreviewCandidate>, LibraryStorageError> {
    if photo_id.is_empty() {
        return Err(CatalogFlagError::EmptyPhotoId.into());
    }

    let library = open_existing_library_for_read(library_root_path)?;
    let connection = open_catalog(&library.catalog_path)?;
    connection
        .query_row(
            r#"
            SELECT id, file_name, path, unsupported, file_type
            FROM photos
            WHERE id = ?1
            "#,
            params![photo_id],
            |row| {
                let file_type: String = row.get(4)?;
                Ok(PhotoPreviewCandidate {
                    photo_id: row.get(0)?,
                    file_name: row.get(1)?,
                    path: row.get(2)?,
                    unsupported: sql_to_bool(row.get::<_, i64>(3)?)
                        || !is_supported_raster_catalog_file_type(&file_type),
                })
            },
        )
        .optional()
        .map_err(LibraryStorageError::from)
}

const IMPORT_RECURSIVE_MAX_DEPTH: usize = 20;

struct ImportScanState {
    candidates: Vec<ImportCandidate>,
    issues: Vec<ImportIssue>,
    recursive: bool,
}

fn scan_import_candidates(
    folder_path: &Path,
    options: FolderImportOptions,
) -> Result<(Vec<ImportCandidate>, Vec<ImportIssue>), LibraryStorageError> {
    let mut state = ImportScanState {
        candidates: Vec::new(),
        issues: Vec::new(),
        recursive: options.recursive,
    };
    scan_import_directory(folder_path, 0, true, &mut state)?;
    Ok((state.candidates, state.issues))
}

fn scan_import_directory(
    folder_path: &Path,
    depth: usize,
    is_root: bool,
    state: &mut ImportScanState,
) -> Result<(), LibraryStorageError> {
    let entries = match fs::read_dir(folder_path) {
        Ok(entries) => entries,
        Err(error) if is_root => return Err(LibraryStorageError::from(error)),
        Err(error) => {
            state.issues.push(import_issue(
                ImportIssueKind::DirectoryReadFailed,
                folder_path,
                folder_path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .map(ToOwned::to_owned),
                format!("failed to read directory: {error}"),
            ));
            return Ok(());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                state.issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    folder_path,
                    None,
                    format!("failed to read directory entry: {error}"),
                ));
                continue;
            }
        };
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                state.issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    &path,
                    Some(file_name),
                    format!("failed to read entry type: {error}"),
                ));
                continue;
            }
        };

        if file_type.is_symlink() {
            state.issues.push(import_issue(
                ImportIssueKind::SymlinkEntrySkipped,
                &path,
                Some(file_name),
                "symbolic links are skipped by import policy",
            ));
            continue;
        }

        if is_hidden_entry(&file_name) {
            state.issues.push(import_issue(
                ImportIssueKind::HiddenEntrySkipped,
                &path,
                Some(file_name),
                "hidden entries are skipped by import policy",
            ));
            continue;
        }

        if file_type.is_dir() {
            if is_package_directory(&path) {
                state.issues.push(import_issue(
                    ImportIssueKind::PackageDirectorySkipped,
                    &path,
                    Some(file_name),
                    "package directories are skipped by import policy",
                ));
            } else if state.recursive {
                if depth >= IMPORT_RECURSIVE_MAX_DEPTH {
                    state.issues.push(import_issue(
                        ImportIssueKind::MaxDepthExceeded,
                        &path,
                        Some(file_name),
                        "recursive import reached the local alpha depth limit",
                    ));
                } else {
                    scan_import_directory(&path, depth + 1, false, state)?;
                }
            }
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(error) => {
                state.issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    &path,
                    Some(file_name),
                    format!("failed to read file metadata: {error}"),
                ));
                continue;
            }
        };
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let unsupported = !is_supported_photo_extension(extension);
        let partial_hash = match partial_file_hash(&path) {
            Ok(hash) => hash,
            Err(error) => {
                state.issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    &path,
                    Some(file_name),
                    format!("failed to read file fingerprint: {error}"),
                ));
                continue;
            }
        };
        let full_hash = match full_file_sha256(&path) {
            Ok(hash) => hash,
            Err(error) => {
                state.issues.push(import_issue(
                    ImportIssueKind::EntryMetadataFailed,
                    &path,
                    Some(file_name),
                    format!("failed to read file hash: {error}"),
                ));
                continue;
            }
        };

        if unsupported {
            state.issues.push(import_issue(
                ImportIssueKind::UnsupportedFile,
                &path,
                Some(file_name.clone()),
                "file extension is unsupported by the local alpha",
            ));
        }

        state.candidates.push(ImportCandidate {
            file_name,
            path: path_to_string(&path)?,
            file_size: metadata.len() as i64,
            modified_at: modified_at_string(&metadata),
            partial_hash,
            full_hash: Some(full_hash),
            unsupported,
        });
    }

    Ok(())
}

fn import_issue(
    kind: ImportIssueKind,
    path: &Path,
    file_name: Option<String>,
    message: impl Into<String>,
) -> ImportIssue {
    ImportIssue {
        kind,
        path: path.display().to_string(),
        file_name,
        message: message.into(),
    }
}

fn import_root_policy_issue(
    folder_path: &Path,
) -> Result<Option<ImportIssue>, LibraryStorageError> {
    let metadata = match fs::symlink_metadata(folder_path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(LibraryStorageError::NotDirectory(folder_path.to_path_buf()));
        }
        Err(error) => return Err(LibraryStorageError::from(error)),
    };
    let file_name = folder_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned);

    if metadata.file_type().is_symlink() {
        return Ok(Some(import_issue(
            ImportIssueKind::SymlinkEntrySkipped,
            folder_path,
            file_name,
            "symbolic links are skipped by import policy",
        )));
    }

    if let Some(name) = file_name.as_deref() {
        if is_hidden_entry(name) {
            return Ok(Some(import_issue(
                ImportIssueKind::HiddenEntrySkipped,
                folder_path,
                file_name,
                "hidden entries are skipped by import policy",
            )));
        }
    }

    if metadata.is_dir() && is_package_directory(folder_path) {
        return Ok(Some(import_issue(
            ImportIssueKind::PackageDirectorySkipped,
            folder_path,
            file_name,
            "package directories are skipped by import policy",
        )));
    }

    Ok(None)
}

fn is_hidden_entry(file_name: &str) -> bool {
    file_name.starts_with('.') && file_name != "." && file_name != ".."
}

fn is_package_directory(path: &Path) -> bool {
    const PACKAGE_EXTENSIONS: &[&str] = &[
        "app",
        "aplibrary",
        "framework",
        "library",
        "lrdata",
        "photoslibrary",
        "plugin",
    ];

    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| {
            PACKAGE_EXTENSIONS
                .iter()
                .any(|package| extension.eq_ignore_ascii_case(package))
        })
}

fn record_import_candidates(
    connection: &mut Connection,
    folder_path: &Path,
    candidates: &[ImportCandidate],
) -> Result<(), LibraryStorageError> {
    let folder_path = path_to_string(folder_path)?;
    let folder_id = stable_catalog_id("folder", &folder_path);
    let transaction = connection.transaction()?;

    transaction.execute(
        r#"
        INSERT INTO folders(id, library_id, path, scanned_at, missing)
        VALUES (?1, ?2, ?3, CURRENT_TIMESTAMP, 0)
        ON CONFLICT(library_id, path) DO UPDATE SET
          scanned_at = CURRENT_TIMESTAMP,
          missing = 0
        "#,
        params![folder_id, LOCAL_LIBRARY_ID, folder_path],
    )?;

    for candidate in candidates {
        let photo_id = stable_catalog_id("photo", &candidate.path);
        transaction.execute(
            r#"
            INSERT INTO photos(
              id,
              library_id,
              folder_id,
              file_name,
              path,
              file_size,
              modified_at,
              missing,
              unsupported,
              file_type,
              partial_hash,
              full_hash
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9, ?10, ?11)
            ON CONFLICT(library_id, path) DO UPDATE SET
              folder_id = excluded.folder_id,
              file_name = excluded.file_name,
              file_size = excluded.file_size,
              modified_at = excluded.modified_at,
              missing = 0,
              unsupported = excluded.unsupported,
              file_type = excluded.file_type,
              partial_hash = excluded.partial_hash,
              full_hash = excluded.full_hash
            "#,
            params![
                photo_id,
                LOCAL_LIBRARY_ID,
                folder_id,
                candidate.file_name,
                candidate.path,
                candidate.file_size,
                candidate.modified_at,
                bool_to_sql(candidate.unsupported),
                catalog_file_type_for_path(&candidate.path, candidate.unsupported),
                candidate.partial_hash,
                candidate.full_hash,
            ],
        )?;

        transaction.execute(
            r#"
            INSERT INTO photo_flags(photo_id)
            VALUES (?1)
            ON CONFLICT(photo_id) DO NOTHING
            "#,
            params![photo_id],
        )?;
    }

    transaction.commit()?;
    Ok(())
}

pub(super) fn import_candidates_fingerprints_unchanged(candidates: &[ImportCandidate]) -> bool {
    candidates.iter().all(|candidate| {
        let path = Path::new(&candidate.path);
        let Ok(metadata) = fs::metadata(path) else {
            return false;
        };
        if !metadata.is_file() || metadata.len() as i64 != candidate.file_size {
            return false;
        }
        if modified_at_string(&metadata) != candidate.modified_at {
            return false;
        }
        candidate
            .full_hash
            .as_deref()
            .is_some_and(|expected| full_file_sha256(path).is_ok_and(|hash| hash == expected))
    })
}

fn catalog_file_type_for_path(path: &str, unsupported: bool) -> &'static str {
    if unsupported {
        return "unsupported";
    }

    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");

    if extension.eq_ignore_ascii_case("jpg") || extension.eq_ignore_ascii_case("jpeg") {
        "jpeg"
    } else if extension.eq_ignore_ascii_case("png") {
        "png"
    } else if extension.eq_ignore_ascii_case("tif") || extension.eq_ignore_ascii_case("tiff") {
        "tiff"
    } else {
        "unsupported"
    }
}

fn is_supported_raster_catalog_file_type(file_type: &str) -> bool {
    matches!(file_type, "jpeg" | "png" | "tiff")
}

fn photo_flags_from_row(
    photo_id: String,
    rating: i64,
    picked: i64,
    rejected: i64,
    color_label: Option<String>,
) -> Result<PhotoFlags, LibraryStorageError> {
    let rating = u8::try_from(rating).unwrap_or(u8::MAX);
    PhotoFlags::new(
        photo_id,
        rating,
        sql_to_bool(picked),
        sql_to_bool(rejected),
        color_label,
    )
    .map_err(LibraryStorageError::from)
}

fn library_photo_grid_item_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<LibraryPhotoGridItem> {
    let photo_id: String = row.get(0)?;
    let file_name: String = row.get(1)?;
    let path: String = row.get(2)?;
    let file_type = Path::new(&path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_uppercase();
    let catalog_file_type: String = row.get(5)?;
    let rating = u8::try_from(row.get::<_, Option<i64>>(6)?.unwrap_or(0))
        .unwrap_or(0)
        .min(ALPHA_MAX_RATING);
    let unsupported = sql_to_bool(row.get::<_, i64>(4)?)
        || !is_supported_raster_catalog_file_type(&catalog_file_type);

    Ok(LibraryPhotoGridItem {
        photo_id,
        file_name,
        path,
        file_type,
        thumbnail_path: row.get(10)?,
        thumbnail_cache_key: row.get(11)?,
        missing: sql_to_bool(row.get::<_, i64>(3)?),
        unsupported,
        rating,
        picked: sql_to_bool(row.get::<_, Option<i64>>(7)?.unwrap_or(0)),
        rejected: sql_to_bool(row.get::<_, Option<i64>>(8)?.unwrap_or(0)),
        color_label: row.get(9)?,
    })
}

fn photo_metadata_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PhotoMetadata> {
    let file_type: String = row.get(3)?;
    let unsupported =
        sql_to_bool(row.get::<_, i64>(4)?) || !is_supported_raster_catalog_file_type(&file_type);
    let metadata_present = row.get::<_, i64>(7)? != 0;

    Ok(PhotoMetadata {
        photo_id: row.get(0)?,
        file_name: row.get(1)?,
        source_path: row.get(2)?,
        file_type,
        unsupported,
        file_size: PhotoMetadataField::known(row.get(5)?),
        modified_at: match row.get(6)? {
            Some(value) => PhotoMetadataField::known(value),
            None => PhotoMetadataField::unavailable(),
        },
        width: stored_metadata_field(row.get(8)?, metadata_present, unsupported),
        height: stored_metadata_field(row.get(9)?, metadata_present, unsupported),
        orientation: stored_metadata_field(row.get(10)?, metadata_present, unsupported),
        capture_time: stored_metadata_field(row.get(11)?, metadata_present, unsupported),
        camera_make: stored_metadata_field(row.get(12)?, metadata_present, unsupported),
        camera_model: stored_metadata_field(row.get(13)?, metadata_present, unsupported),
        lens_model: stored_metadata_field(row.get(14)?, metadata_present, unsupported),
    })
}

fn stored_metadata_field<T>(
    value: Option<T>,
    metadata_present: bool,
    unsupported: bool,
) -> PhotoMetadataField<T> {
    match value {
        Some(value) => PhotoMetadataField::known(value),
        None if metadata_present || unsupported => PhotoMetadataField::unavailable(),
        None => PhotoMetadataField::unknown(),
    }
}

pub(super) fn get_photo_flags_from_connection(
    connection: &Connection,
    photo_id: &str,
) -> Result<Option<PhotoFlags>, LibraryStorageError> {
    connection
        .query_row(
            r#"
            SELECT photo_id, rating, picked, rejected, color_label
            FROM photo_flags
            WHERE photo_id = ?1
            "#,
            params![photo_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .optional()?
        .map(|(photo_id, rating, picked, rejected, color_label)| {
            photo_flags_from_row(photo_id, rating, picked, rejected, color_label)
        })
        .transpose()
}

pub(super) fn default_rebuild_flags(photo_id: &str) -> PhotoFlags {
    PhotoFlags::new(photo_id.to_string(), 0, false, false, None)
        .expect("default rebuild flags are valid")
}

const LIBRARY_QUERY_COUNT_SQL: &str = r#"
SELECT COUNT(*)
FROM photos
LEFT JOIN photo_flags ON photo_flags.photo_id = photos.id
LEFT JOIN photo_metadata ON photo_metadata.photo_id = photos.id
WHERE photos.library_id = :library_id
  AND (:min_rating IS NULL OR COALESCE(photo_flags.rating, 0) >= :min_rating)
  AND (:picked IS NULL OR COALESCE(photo_flags.picked, 0) = :picked)
  AND (:rejected IS NULL OR COALESCE(photo_flags.rejected, 0) = :rejected)
  AND (
    :file_type IS NULL
    OR (:file_type = 'jpeg' AND photos.file_type = 'jpeg' AND photos.unsupported = 0)
    OR (:file_type = 'png' AND photos.file_type = 'png' AND photos.unsupported = 0)
    OR (:file_type = 'tiff' AND photos.file_type = 'tiff' AND photos.unsupported = 0)
    OR (:file_type = 'raw' AND photos.file_type = 'raw')
    OR (
      :file_type = 'unsupported'
      AND (
        photos.unsupported = 1
        OR photos.file_type NOT IN ('jpeg', 'png', 'tiff')
      )
    )
  )
  AND (
    :metadata_filter IS NULL
    OR (
      :metadata_filter = 'has_dimensions'
      AND photo_metadata.width IS NOT NULL
      AND photo_metadata.height IS NOT NULL
    )
  )
  AND (
    :search IS NULL
    OR lower(photos.file_name) LIKE :search ESCAPE '\'
    OR lower(photos.path) LIKE :search ESCAPE '\'
  )
"#;

const LIBRARY_QUERY_SELECT_SQL: &str = r#"
SELECT
  photos.id,
  photos.file_name,
  photos.path,
  photos.missing,
  photos.unsupported,
  photos.file_type,
  photo_flags.rating,
  photo_flags.picked,
  photo_flags.rejected,
  photo_flags.color_label,
  thumbnail_cache.path,
  thumbnail_cache.cache_key
FROM photos
LEFT JOIN photo_flags ON photo_flags.photo_id = photos.id
LEFT JOIN photo_metadata ON photo_metadata.photo_id = photos.id
LEFT JOIN cache_records AS thumbnail_cache
  ON thumbnail_cache.photo_id = photos.id
  AND thumbnail_cache.cache_type = :thumbnail_cache_type
WHERE photos.library_id = :library_id
  AND (:min_rating IS NULL OR COALESCE(photo_flags.rating, 0) >= :min_rating)
  AND (:picked IS NULL OR COALESCE(photo_flags.picked, 0) = :picked)
  AND (:rejected IS NULL OR COALESCE(photo_flags.rejected, 0) = :rejected)
  AND (
    :file_type IS NULL
    OR (:file_type = 'jpeg' AND photos.file_type = 'jpeg' AND photos.unsupported = 0)
    OR (:file_type = 'png' AND photos.file_type = 'png' AND photos.unsupported = 0)
    OR (:file_type = 'tiff' AND photos.file_type = 'tiff' AND photos.unsupported = 0)
    OR (:file_type = 'raw' AND photos.file_type = 'raw')
    OR (
      :file_type = 'unsupported'
      AND (
        photos.unsupported = 1
        OR photos.file_type NOT IN ('jpeg', 'png', 'tiff')
      )
    )
  )
  AND (
    :metadata_filter IS NULL
    OR (
      :metadata_filter = 'has_dimensions'
      AND photo_metadata.width IS NOT NULL
      AND photo_metadata.height IS NOT NULL
    )
  )
  AND (
    :search IS NULL
    OR lower(photos.file_name) LIKE :search ESCAPE '\'
    OR lower(photos.path) LIKE :search ESCAPE '\'
  )
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LibraryQuerySqlFilter {
    min_rating: Option<i64>,
    picked: Option<i64>,
    rejected: Option<i64>,
    file_type: Option<&'static str>,
    metadata: Option<&'static str>,
    search: Option<String>,
}

impl From<&LibraryQueryFilters> for LibraryQuerySqlFilter {
    fn from(filters: &LibraryQueryFilters) -> Self {
        Self {
            min_rating: filters.min_rating.map(i64::from),
            picked: filters.picked.map(bool_to_sql),
            rejected: filters.rejected.map(bool_to_sql),
            file_type: filters.file_type.map(library_query_file_type_value),
            metadata: filters.metadata.map(library_query_metadata_filter_value),
            search: library_query_search_pattern(&filters.search),
        }
    }
}

fn query_library_photo_count(
    connection: &Connection,
    filter: &LibraryQuerySqlFilter,
) -> rusqlite::Result<u64> {
    let count: i64 = connection.query_row(
        LIBRARY_QUERY_COUNT_SQL,
        named_params! {
            ":library_id": LOCAL_LIBRARY_ID,
            ":min_rating": filter.min_rating,
            ":picked": filter.picked,
            ":rejected": filter.rejected,
            ":file_type": filter.file_type,
            ":metadata_filter": filter.metadata,
            ":search": filter.search.as_deref(),
        },
        |row| row.get(0),
    )?;

    Ok(u64::try_from(count).unwrap_or(0))
}

fn library_query_order_clause(sort: LibraryQuerySort) -> &'static str {
    match sort {
        LibraryQuerySort::ImportedAtDesc => "ORDER BY photos.imported_at DESC, photos.id ASC",
        LibraryQuerySort::FileNameAsc => {
            "ORDER BY photos.file_name ASC, photos.path ASC, photos.id ASC"
        }
        LibraryQuerySort::RatingDesc => {
            "ORDER BY COALESCE(photo_flags.rating, 0) DESC, photos.id ASC"
        }
    }
}

fn library_query_file_type_value(file_type: LibraryQueryFileType) -> &'static str {
    match file_type {
        LibraryQueryFileType::Jpeg => "jpeg",
        LibraryQueryFileType::Png => "png",
        LibraryQueryFileType::Tiff => "tiff",
        LibraryQueryFileType::Raw => "raw",
        LibraryQueryFileType::Unsupported => "unsupported",
    }
}

fn library_query_metadata_filter_value(metadata: LibraryQueryMetadataFilter) -> &'static str {
    match metadata {
        LibraryQueryMetadataFilter::HasDimensions => "has_dimensions",
    }
}

fn library_query_search_pattern(search: &str) -> Option<String> {
    let search = search.trim();
    if search.is_empty() {
        return None;
    }

    let mut pattern = String::from("%");
    for character in search.to_ascii_lowercase().chars() {
        if matches!(character, '\\' | '%' | '_') {
            pattern.push('\\');
        }
        pattern.push(character);
    }
    pattern.push('%');
    Some(pattern)
}
