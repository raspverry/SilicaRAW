use super::backup::restore_staging_path;
use super::common::{modified_at_string, partial_file_hash, path_to_string, stable_catalog_id};
use super::library::ensure_library_directories;
use super::photos::import_candidates_fingerprints_unchanged;
use super::sidecar::{build_photo_sidecar_value, validate_sidecar_json, SIDECAR_FILE_SUFFIX};
use super::*;
use rusqlite::{params, Connection};
use silica_catalog::ImportCandidate;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn exposes_crate_name() {
    assert_eq!(CRATE_NAME, "silica-storage");
}

#[test]
fn records_spike_004_storage_gate() {
    assert_eq!(
        SPIKE_004_STORAGE_GATE.sqlite_binding,
        SqliteBinding::RusqliteBundled
    );
    assert_eq!(
        SPIKE_004_STORAGE_GATE.migration_approach,
        MigrationApproach::EmbeddedSqlMigrations
    );
    assert_eq!(SPIKE_004_STORAGE_GATE.journal_mode, CatalogJournalMode::Wal);
    assert_eq!(
        SPIKE_004_STORAGE_GATE.current_schema_version,
        CURRENT_SCHEMA_VERSION
    );
}

#[test]
fn records_metadata_backfill_policy_without_automatic_open_work() {
    assert!(!METADATA_BACKFILL_ON_OPEN_OR_RESTORE);
    assert!(EXISTING_IMPORTS_WITHOUT_METADATA_STAY_UNKNOWN);

    let jpeg_policy = metadata_extraction_policy_for_path(Path::new("sample.jpg"));
    assert_eq!(
        jpeg_policy.dimension_source,
        MetadataDimensionSource::ExistingRasterPath
    );
    assert!(!jpeg_policy.raw_decode_supported);

    let raw_policy = metadata_extraction_policy_for_path(Path::new("sample.DNG"));
    assert_eq!(
        raw_policy.dimension_source,
        MetadataDimensionSource::Unavailable
    );
    assert!(!raw_policy.raw_decode_supported);
    assert!(!raw_policy.camera_lens_available);

    let workspace = unique_library_root("metadata-policy");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let before_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM photo_metadata", [], |row| row.get(0))
        .expect("count metadata before reopen");
    assert_eq!(before_count, 0);
    drop(connection);

    open_local_library(&library.root_path).expect("reopen library");
    let connection = open_catalog(&library.catalog_path).expect("open reopened catalog");
    let after_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM photo_metadata", [], |row| row.get(0))
        .expect("count metadata after reopen");
    assert_eq!(after_count, 0);

    remove_library_root(&workspace);
}

#[test]
fn metadata_migration_adds_normalized_columns_and_upsert() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");
    run_migrations(&mut connection).expect("run migrations");

    let columns: Vec<String> = {
        let mut statement = connection
            .prepare("SELECT name FROM pragma_table_info('photo_metadata')")
            .expect("prepare metadata column query");
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query metadata columns")
            .map(|row| row.expect("metadata column"))
            .collect()
    };
    for column in ["width", "height", "orientation"] {
        assert!(
            columns.contains(&column.to_string()),
            "missing metadata column {column}"
        );
    }

    let workspace = unique_library_root("metadata-upsert");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&jpeg_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    upsert_photo_metadata_by_path(
        &library.root_path,
        &jpeg_file,
        PhotoMetadataUpdate {
            width: Some(2),
            height: Some(3),
            ..PhotoMetadataUpdate::unavailable()
        },
    )
    .expect("upsert metadata");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let (width, height, camera_make): (Option<i64>, Option<i64>, Option<String>) = connection
        .query_row(
            r#"
            SELECT photo_metadata.width, photo_metadata.height, photo_metadata.camera_make
            FROM photo_metadata
            JOIN photos ON photos.id = photo_metadata.photo_id
            WHERE photos.file_name = 'sample.jpg'
            "#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("read metadata row");
    assert_eq!(width, Some(2));
    assert_eq!(height, Some(3));
    assert_eq!(camera_make, None);

    remove_library_root(&workspace);
}

#[test]
fn metadata_filter_index_and_query_use_stored_dimensions_only() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");
    run_migrations(&mut connection).expect("run migrations");
    assert!(
        catalog_object_exists(&connection, "idx_photo_metadata_dimensions_photo_id")
            .expect("index lookup"),
        "missing metadata dimension filter index"
    );

    let workspace = unique_library_root("metadata-filter");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let known_file = import_root.join("known.jpg");
    let unknown_file = import_root.join("unknown.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&known_file, b"known jpeg bytes").expect("write known");
    std::fs::write(&unknown_file, b"unknown jpeg bytes").expect("write unknown");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    upsert_photo_metadata_by_path(
        &library.root_path,
        &known_file,
        PhotoMetadataUpdate {
            width: Some(24),
            height: Some(36),
            ..PhotoMetadataUpdate::unavailable()
        },
    )
    .expect("upsert known metadata");
    std::fs::remove_file(&known_file).expect("remove original after metadata persist");

    let page = query_library_photos(
        &library.root_path,
        LibraryQueryRequest::new(
            0,
            100,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters {
                metadata: Some(LibraryQueryMetadataFilter::HasDimensions),
                ..LibraryQueryFilters::default()
            },
        ),
    )
    .expect("query metadata-backed filter");

    assert_eq!(page.total_count, 1);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].file_name, "known.jpg");

    remove_library_root(&workspace);
}

#[test]
fn metadata_query_returns_stored_states_without_original_reads() {
    let workspace = unique_library_root("metadata-query");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let known_file = import_root.join("known.jpg");
    let unknown_file = import_root.join("unknown.jpg");
    let unsupported_file = import_root.join("notes.txt");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&known_file, b"known jpeg bytes").expect("write known");
    std::fs::write(&unknown_file, b"unknown jpeg bytes").expect("write unknown");
    std::fs::write(&unsupported_file, b"unsupported").expect("write unsupported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    upsert_photo_metadata_by_path(
        &library.root_path,
        &known_file,
        PhotoMetadataUpdate {
            width: Some(24),
            height: Some(36),
            ..PhotoMetadataUpdate::unavailable()
        },
    )
    .expect("upsert known metadata");

    std::fs::remove_file(&known_file).expect("remove original after metadata persist");

    let known_id = stable_catalog_id("photo", &known_file.display().to_string());
    let known = get_photo_metadata(&library.root_path, &known_id)
        .expect("query known metadata")
        .expect("known photo metadata");
    assert_eq!(known.width.state, PhotoMetadataFieldState::Known);
    assert_eq!(known.width.value, Some(24));
    assert_eq!(known.height.state, PhotoMetadataFieldState::Known);
    assert_eq!(known.height.value, Some(36));
    assert_eq!(
        known.camera_make.state,
        PhotoMetadataFieldState::Unavailable
    );
    assert_eq!(known.camera_make.value, None);
    assert_eq!(known.file_size.state, PhotoMetadataFieldState::Known);

    let unknown_id = stable_catalog_id("photo", &unknown_file.display().to_string());
    let unknown = get_photo_metadata(&library.root_path, &unknown_id)
        .expect("query unknown metadata")
        .expect("unknown photo metadata");
    assert_eq!(unknown.width.state, PhotoMetadataFieldState::Unknown);
    assert_eq!(unknown.width.value, None);
    assert_eq!(unknown.camera_model.state, PhotoMetadataFieldState::Unknown);

    let unsupported_id = stable_catalog_id("photo", &unsupported_file.display().to_string());
    let unsupported = get_photo_metadata(&library.root_path, &unsupported_id)
        .expect("query unsupported metadata")
        .expect("unsupported photo metadata");
    assert!(unsupported.unsupported);
    assert_eq!(
        unsupported.width.state,
        PhotoMetadataFieldState::Unavailable
    );
    assert_eq!(
        unsupported.lens_model.state,
        PhotoMetadataFieldState::Unavailable
    );

    assert!(get_photo_metadata(&library.root_path, "missing-photo")
        .expect("query missing metadata")
        .is_none());

    remove_library_root(&workspace);
}

#[test]
fn creates_empty_catalog_schema_and_required_indexes() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");
    run_migrations(&mut connection).expect("run migrations");

    assert_eq!(
        current_schema_version(&connection).expect("schema version"),
        CURRENT_SCHEMA_VERSION
    );
    assert!(catalog_object_exists(&connection, "libraries").expect("libraries table exists"));
    assert!(catalog_object_exists(&connection, "photos").expect("photos table exists"));
    assert!(catalog_object_exists(&connection, "schema_migrations").expect("migrations table"));

    for table_name in REQUIRED_TABLES {
        assert!(
            catalog_object_exists(&connection, table_name).expect("table lookup"),
            "missing required table {table_name}"
        );
    }

    for index_name in REQUIRED_INDEXES {
        assert!(
            catalog_object_exists(&connection, index_name).expect("index lookup"),
            "missing required index {index_name}"
        );
    }
}

#[test]
fn upgrades_empty_catalog_from_first_migration() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");

    run_migrations_through(&mut connection, 1).expect("run first migration");
    assert_eq!(current_schema_version(&connection).expect("version"), 1);
    assert!(
        !catalog_object_exists(&connection, "idx_photos_library_id").expect("index lookup"),
        "index migration should not have run yet"
    );

    run_migrations(&mut connection).expect("upgrade to latest");
    assert_eq!(
        current_schema_version(&connection).expect("version"),
        CURRENT_SCHEMA_VERSION
    );
    assert!(catalog_object_exists(&connection, "idx_photos_library_id").expect("index lookup"));
    assert!(
        catalog_object_exists(&connection, "idx_edit_history_photo_sequence")
            .expect("history sequence index lookup")
    );
    assert!(
        catalog_object_exists(&connection, "idx_edit_history_photo_state_sequence")
            .expect("history state index lookup")
    );
    let history_columns: Vec<String> = {
        let mut statement = connection
            .prepare("SELECT name FROM pragma_table_info('edit_history')")
            .expect("prepare history column query");
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query history columns")
            .map(|row| row.expect("history column"))
            .collect()
    };
    for column in ["sequence", "action_class", "action_kind", "history_state"] {
        assert!(
            history_columns.contains(&column.to_string()),
            "missing edit_history column {column}"
        );
    }
    let action_log_columns: Vec<String> = {
        let mut statement = connection
            .prepare("SELECT name FROM pragma_table_info('action_log')")
            .expect("prepare action log column query");
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query action log columns")
            .map(|row| row.expect("action log column"))
            .collect()
    };
    for column in ["side_effect_category", "evidence_ref"] {
        assert!(
            action_log_columns.contains(&column.to_string()),
            "missing action_log column {column}"
        );
    }
    assert!(
        catalog_object_exists(&connection, "idx_action_log_action_type_created_at")
            .expect("action log action index lookup")
    );
    assert!(catalog_object_exists(&connection, "idx_action_log_subject")
        .expect("action log subject index lookup"));
    assert!(catalog_object_exists(&connection, "export_settings")
        .expect("export settings table lookup"));
    assert!(
        catalog_object_exists(&connection, "export_presets").expect("export presets table lookup")
    );
    run_migrations(&mut connection).expect("re-run migrations idempotently");
    assert_eq!(
        current_schema_version(&connection).expect("version after rerun"),
        CURRENT_SCHEMA_VERSION
    );
}

#[test]
fn query_index_migration_adds_normalized_file_type_and_indexes() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");
    run_migrations(&mut connection).expect("run migrations");

    let file_type_columns: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM pragma_table_info('photos') WHERE name = 'file_type'",
            [],
            |row| row.get(0),
        )
        .expect("file_type column count");
    assert_eq!(file_type_columns, 1);

    for index_name in [
        "idx_photos_library_imported_id",
        "idx_photos_library_file_name_path_id",
        "idx_photos_library_file_type_id",
        "idx_photo_flags_rating_photo_id",
    ] {
        assert!(
            catalog_object_exists(&connection, index_name).expect("index lookup"),
            "missing paged query index {index_name}"
        );
    }

    run_migrations(&mut connection).expect("rerun migrations");
    assert_eq!(
        current_schema_version(&connection).expect("schema version"),
        CURRENT_SCHEMA_VERSION
    );
}

#[test]
fn query_index_migration_backfills_photo_file_type() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");
    run_migrations_through(&mut connection, 2).expect("run v2 migrations");

    connection
        .execute(
            "INSERT INTO libraries(id, root_path) VALUES ('local', '/tmp/library')",
            [],
        )
        .expect("insert library");
    connection
        .execute(
            "INSERT INTO folders(id, library_id, path) VALUES ('folder', 'local', '/tmp/import')",
            [],
        )
        .expect("insert folder");
    for (id, file_name, unsupported) in [
        ("photo-jpeg", "portrait.JPG", 0_i64),
        ("photo-png", "screen.PNG", 0_i64),
        ("photo-tiff", "scan.TIFF", 0_i64),
        ("photo-tif", "flatbed.tif", 0_i64),
        ("photo-raw", "sample.DNG", 0_i64),
        ("photo-unsupported", "notes.txt", 1_i64),
    ] {
        connection
            .execute(
                r#"
                INSERT INTO photos(
                  id, library_id, folder_id, file_name, path, unsupported
                )
                VALUES (?1, 'local', 'folder', ?2, ?3, ?4)
                "#,
                params![
                    id,
                    file_name,
                    format!("/tmp/import/{file_name}"),
                    unsupported
                ],
            )
            .expect("insert v2 photo");
    }

    run_migrations(&mut connection).expect("upgrade to current");

    for (id, expected_type, expected_unsupported) in [
        ("photo-jpeg", "jpeg", 0_i64),
        ("photo-png", "png", 0_i64),
        ("photo-tiff", "tiff", 0_i64),
        ("photo-tif", "tiff", 0_i64),
        ("photo-raw", "unsupported", 1_i64),
        ("photo-unsupported", "unsupported", 1_i64),
    ] {
        let (actual_type, actual_unsupported): (String, i64) = connection
            .query_row(
                "SELECT file_type, unsupported FROM photos WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("file type");
        assert_eq!(actual_type, expected_type);
        assert_eq!(actual_unsupported, expected_unsupported);
    }
}

#[test]
fn library_query_migrates_legacy_png_rows_before_read_only_grid() {
    let workspace = unique_library_root("legacy-png-query");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let catalog_path = library_root.join(CATALOG_DATABASE_FILE);

    std::fs::create_dir_all(&import_root).expect("create import root");
    ensure_library_directories(&library_root).expect("create library support dirs");

    {
        let mut connection = Connection::open(&catalog_path).expect("open legacy catalog");
        configure_connection(&connection).expect("configure legacy catalog");
        run_migrations_through(&mut connection, 11).expect("run legacy migrations");
        connection
            .execute(
                "INSERT INTO libraries(id, root_path) VALUES ('local', ?1)",
                params![library_root.display().to_string()],
            )
            .expect("insert library");
        connection
            .execute(
                "INSERT INTO folders(id, library_id, path) VALUES ('folder', 'local', ?1)",
                params![import_root.display().to_string()],
            )
            .expect("insert folder");
        connection
            .execute(
                r#"
                INSERT INTO photos(
                  id, library_id, folder_id, file_name, path, unsupported, file_type
                )
                VALUES (
                  'photo-png',
                  'local',
                  'folder',
                  'sample.PNG',
                  ?1,
                  1,
                  'unsupported'
                )
                "#,
                params![import_root.join("sample.PNG").display().to_string()],
            )
            .expect("insert legacy png row");
    }

    let page = query_library_photos(
        &library_root,
        LibraryQueryRequest::new(
            0,
            10,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters::default(),
        ),
    )
    .expect("query migrates legacy catalog");

    assert_eq!(page.total_count, 1);
    assert_eq!(page.items[0].file_type, "PNG");
    assert!(!page.items[0].unsupported);

    let connection = open_catalog(&catalog_path).expect("open migrated catalog");
    assert_eq!(
        current_schema_version(&connection).expect("schema version"),
        CURRENT_SCHEMA_VERSION
    );
    let (file_type, unsupported): (String, i64) = connection
        .query_row(
            "SELECT file_type, unsupported FROM photos WHERE id = 'photo-png'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("migrated png row");
    assert_eq!(file_type, "png");
    assert_eq!(unsupported, 0);

    remove_library_root(&workspace);
}

#[test]
fn library_query_returns_bounded_pages_and_normalized_filters() {
    let workspace = unique_library_root("library-query");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let jpeg_file = import_root.join("portrait.jpg");
    let unsupported_source_file = import_root.join("sample.DNG");
    let unsupported_file = import_root.join("notes.txt");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&jpeg_file, b"jpeg candidate").expect("write jpeg");
    std::fs::write(&unsupported_source_file, b"raw candidate").expect("write raw");
    std::fs::write(&unsupported_file, b"unsupported").expect("write unsupported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let unsupported_source_id =
        stable_catalog_id("photo", &unsupported_source_file.display().to_string());
    {
        let connection = open_catalog(&library.catalog_path).expect("open catalog");
        connection
            .execute(
                "UPDATE photos SET file_type = 'raw', unsupported = 0 WHERE id = ?1",
                params![unsupported_source_id],
            )
            .expect("simulate legacy raw-supported row");
    }
    set_photo_flags(
        &library.root_path,
        unsupported_source_id.clone(),
        4,
        true,
        false,
        None,
    )
    .expect("set unsupported source flags");

    let first_page = query_library_photos(
        &library.root_path,
        LibraryQueryRequest::new(
            0,
            2,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters::default(),
        ),
    )
    .expect("query first page");

    assert_eq!(first_page.offset, 0);
    assert_eq!(first_page.limit, 2);
    assert_eq!(first_page.total_count, 3);
    assert!(first_page.has_next_page);
    assert_eq!(
        first_page.order_fields,
        LibraryQuerySort::FileNameAsc.order_fields()
    );
    assert_eq!(first_page.items.len(), 2);
    assert_eq!(first_page.items[0].file_name, "notes.txt");
    assert_eq!(first_page.items[1].file_name, "portrait.jpg");

    let unsupported_source_page = query_library_photos(
        &library.root_path,
        LibraryQueryRequest::new(
            0,
            10,
            LibraryQuerySort::RatingDesc,
            LibraryQueryFilters {
                min_rating: Some(4),
                picked: Some(true),
                rejected: Some(false),
                file_type: Some(LibraryQueryFileType::Unsupported),
                search: "sample".to_string(),
                ..LibraryQueryFilters::default()
            },
        ),
    )
    .expect("query filtered page");
    assert_eq!(unsupported_source_page.total_count, 1);
    assert_eq!(unsupported_source_page.items.len(), 1);
    assert_eq!(
        unsupported_source_page.items[0].photo_id,
        unsupported_source_id
    );
    assert_eq!(unsupported_source_page.items[0].rating, 4);
    assert!(unsupported_source_page.items[0].picked);
    assert!(unsupported_source_page.items[0].unsupported);
    assert!(!unsupported_source_page.has_next_page);

    let empty_page = query_library_photos(
        &library.root_path,
        LibraryQueryRequest::new(
            99,
            10,
            LibraryQuerySort::FileNameAsc,
            LibraryQueryFilters::default(),
        ),
    )
    .expect("query empty page");
    assert!(empty_page.items.is_empty());
    assert_eq!(empty_page.offset, 99);
    assert_eq!(empty_page.total_count, 3);
    assert!(!empty_page.has_next_page);

    remove_library_root(&workspace);
}

#[test]
fn enforces_foreign_keys_for_catalog_rows() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");
    run_migrations(&mut connection).expect("run migrations");

    let result = connection.execute(
        r#"
        INSERT INTO photos(id, library_id, folder_id, file_name, path)
        VALUES ('photo-1', 'missing-library', 'missing-folder', 'sample.dng', '/tmp/sample.dng')
        "#,
        [],
    );

    assert!(result.is_err(), "foreign key violation should fail");
}

#[test]
fn opens_file_backed_catalog_with_wal_and_foreign_keys() {
    let path = unique_catalog_path("wal");

    {
        let connection = open_catalog(&path).expect("open file-backed catalog");
        assert_eq!(
            current_schema_version(&connection).expect("version"),
            CURRENT_SCHEMA_VERSION
        );

        let journal_mode: String = connection
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal mode");
        assert_eq!(journal_mode.to_ascii_lowercase(), "wal");

        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign keys");
        assert_eq!(foreign_keys, 1);
    }

    remove_catalog_files(&path);
}

#[test]
fn creates_library_folder_with_catalog_and_support_directories() {
    let root = unique_library_root("create");

    {
        let library = create_local_library(&root).expect("create local library");
        assert_eq!(library.root_path, root);
        assert_eq!(library.catalog_path, root.join(CATALOG_DATABASE_FILE));
        assert_eq!(library.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(library.catalog_path.is_file());

        for directory in REQUIRED_LIBRARY_DIRECTORIES {
            assert!(
                root.join(directory).is_dir(),
                "missing library directory {directory}"
            );
        }
    }

    remove_library_root(&root);
}

#[test]
fn resolves_sidecar_paths_under_library_sidecars_only() {
    let root = unique_library_root("sidecar-path");
    let path = sidecar_path_for_photo(&root, "photo_ABC-123.ok").expect("valid sidecar path");

    assert_eq!(
        path,
        root.join("sidecars")
            .join("photo_ABC-123.ok.silicaraw.sidecar.json")
    );
    assert!(path.starts_with(root.join("sidecars")));
}

#[test]
fn rejects_unsafe_sidecar_photo_ids() {
    let root = unique_library_root("sidecar-invalid-id");
    for invalid in [
        "",
        "../photo",
        "folder/photo",
        "folder\\photo",
        "/tmp/photo",
        "photo\nid",
    ] {
        assert!(
            sidecar_path_for_photo(&root, invalid).is_err(),
            "invalid photo id should be rejected: {invalid:?}"
        );
    }
}

#[test]
fn builds_valid_sidecar_payload_with_flags_and_metadata_mirror() {
    let workspace = unique_library_root("sidecar-payload");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        5,
        true,
        false,
        Some("purple".to_string()),
    )
    .expect("set flags");

    let sidecar = build_photo_sidecar_value(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("build sidecar");

    validate_sidecar_json(&sidecar).expect("validate sidecar");
    assert_eq!(sidecar["schema"], SIDECAR_SCHEMA);
    assert_eq!(sidecar["version"], SIDECAR_VERSION);
    assert_eq!(sidecar["photo"]["photo_id"], photo_id);
    assert_eq!(sidecar["flags"]["rating"], 5);
    assert_eq!(sidecar["flags"]["picked"], true);
    assert_eq!(sidecar["flags"]["rejected"], false);
    assert_eq!(sidecar["flags"]["color_label"], "purple");
    assert_eq!(sidecar["edit_graph"]["metadata"]["rating"], 5);
    assert_eq!(sidecar["edit_graph"]["metadata"]["picked"], true);
    assert_eq!(sidecar["edit_graph"]["metadata"]["rejected"], false);
    assert_eq!(sidecar["edit_graph"]["metadata"]["color_label"], "purple");
    assert!(sidecar["sync"]["sidecar_hash"].is_null());
    assert!(sidecar["flags"].get("edited").is_none());
    assert!(sidecar["flags"].get("exported").is_none());

    let connection = open_catalog(library.catalog_path).expect("open catalog");
    assert_eq!(
        count_edit_states(&connection),
        0,
        "building a sidecar for an unedited photo must not write edit_states"
    );

    remove_library_root(&workspace);
}

#[test]
fn rejects_sidecar_payload_for_invalid_color_label() {
    let workspace = unique_library_root("sidecar-invalid-label");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    connection
        .execute(
            "UPDATE photo_flags SET color_label = 'cyan' WHERE photo_id = ?1",
            params![photo_id],
        )
        .expect("force invalid catalog label");
    drop(connection);

    let error = build_photo_sidecar_value(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect_err("invalid sidecar label should fail");
    assert!(error
        .to_string()
        .contains("unsupported sidecar color label"));

    remove_library_root(&workspace);
}

#[test]
fn writes_sidecar_under_library_and_updates_status_after_success() {
    let workspace = unique_library_root("sidecar-write");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");
    let original_before = std::fs::read(&supported_file).expect("read original before");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        3,
        false,
        true,
        Some("red".to_string()),
    )
    .expect("set flags");

    let result =
        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");

    assert_eq!(result.photo_id, photo_id);
    assert_eq!(
        result.sidecar_relative_path,
        format!("sidecars/{photo_id}.silicaraw.sidecar.json")
    );
    assert!(result.sidecar_path.is_file());
    assert!(result
        .sidecar_path
        .starts_with(library.root_path.join(SIDECAR_DIRECTORY)));
    assert!(result.bytes_written > 0);
    assert_eq!(
        std::fs::read(&supported_file).expect("read original after"),
        original_before
    );

    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&result.sidecar_path).expect("read sidecar"))
            .expect("parse sidecar");
    validate_sidecar_json(&json).expect("validate written sidecar");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let (sidecar_path, conflict_state): (String, String) = connection
        .query_row(
            "SELECT sidecar_path, conflict_state FROM sidecar_status WHERE photo_id = ?1",
            params![&result.photo_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("sidecar status");
    assert_eq!(sidecar_path, result.sidecar_relative_path);
    assert_eq!(conflict_state, "clean");

    remove_library_root(&workspace);
}

#[test]
fn history_commits_mark_clean_sidecar_catalog_newer_without_rewriting_sidecar() {
    let workspace = unique_library_root("sidecar-history-commit");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

    let first_sidecar = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write first sidecar");
    let first_bytes = std::fs::read(&first_sidecar.sidecar_path).expect("read first sidecar");
    let first_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read first sidecar status")
        .expect("first sidecar status");
    assert_eq!(first_status.conflict_state, "clean");

    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        5,
        true,
        false,
        Some("purple".to_string()),
    )
    .expect("commit flag history");
    let flag_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read flag sidecar status")
        .expect("flag sidecar status");
    assert_eq!(flag_status.conflict_state, "catalog_newer");
    assert_eq!(
        std::fs::read(&first_sidecar.sidecar_path).expect("read sidecar after flags"),
        first_bytes,
        "history commit must not rewrite the sidecar file"
    );

    let second_sidecar = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write refreshed sidecar");
    let second_bytes = std::fs::read(&second_sidecar.sidecar_path).expect("read refreshed sidecar");
    let clean_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read clean sidecar status")
        .expect("clean sidecar status");
    assert_eq!(clean_status.conflict_state, "clean");

    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft")
        .expect("draft graph");
    let edited =
        silica_edit::apply_exposure_contrast(&draft, 0.5, -8.0, "unix:3").expect("apply edit");
    commit_edit_graph(&library.root_path, edited).expect("commit edit history");
    let edit_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read edit sidecar status")
        .expect("edit sidecar status");
    assert_eq!(edit_status.conflict_state, "catalog_newer");
    assert_eq!(
        std::fs::read(&second_sidecar.sidecar_path).expect("read sidecar after edit"),
        second_bytes,
        "edit history commit must not rewrite the sidecar file"
    );

    let reopened = open_local_library(&library_root).expect("reopen library");
    let reopened_status = get_photo_sidecar_status(&reopened.root_path, &photo_id)
        .expect("read reopened sidecar status")
        .expect("reopened sidecar status");
    assert_eq!(reopened_status.conflict_state, "catalog_newer");

    remove_library_root(&workspace);
}

#[test]
fn undo_redo_history_marks_clean_sidecar_catalog_newer_without_file_effects() {
    let workspace = unique_library_root("sidecar-history-undo-redo");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft")
        .expect("draft graph");
    let edited =
        silica_edit::apply_exposure_contrast(&draft, 0.5, -8.0, "unix:3").expect("apply edit");
    commit_edit_graph(&library.root_path, edited).expect("commit edit");

    let undo_sidecar = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write undo baseline sidecar");
    let undo_bytes = std::fs::read(&undo_sidecar.sidecar_path).expect("read undo sidecar");
    undo_last_history_action(&library.root_path, &photo_id).expect("undo history");
    let undo_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read undo sidecar status")
        .expect("undo sidecar status");
    assert_eq!(undo_status.conflict_state, "catalog_newer");
    assert_eq!(
        std::fs::read(&undo_sidecar.sidecar_path).expect("read sidecar after undo"),
        undo_bytes,
        "undo must not rewrite or delete the sidecar file"
    );

    let redo_sidecar = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write redo baseline sidecar");
    let redo_bytes = std::fs::read(&redo_sidecar.sidecar_path).expect("read redo sidecar");
    redo_last_history_action(&library.root_path, &photo_id).expect("redo history");
    let redo_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read redo sidecar status")
        .expect("redo sidecar status");
    assert_eq!(redo_status.conflict_state, "catalog_newer");
    assert_eq!(
        std::fs::read(&redo_sidecar.sidecar_path).expect("read sidecar after redo"),
        redo_bytes,
        "redo must not rewrite or delete the sidecar file"
    );

    remove_library_root(&workspace);
}

#[test]
fn history_updates_preserve_conflict_and_sidecar_newer_statuses() {
    let workspace = unique_library_root("sidecar-history-preserve-conflict");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    connection
        .execute(
            "UPDATE sidecar_status SET conflict_state = 'conflict' WHERE photo_id = ?1",
            params![photo_id.clone()],
        )
        .expect("mark conflict");
    drop(connection);
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        3,
        false,
        true,
        Some("red".to_string()),
    )
    .expect("commit flags over conflict");
    let conflict_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read conflict status")
        .expect("conflict status");
    assert_eq!(conflict_status.conflict_state, "conflict");

    let connection = open_catalog(&library.catalog_path).expect("reopen catalog");
    connection
        .execute(
            "UPDATE sidecar_status SET conflict_state = 'sidecar_newer' WHERE photo_id = ?1",
            params![photo_id.clone()],
        )
        .expect("mark sidecar newer");
    drop(connection);
    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft")
        .expect("draft graph");
    let edited =
        silica_edit::apply_exposure_contrast(&draft, 1.0, 3.0, "unix:4").expect("apply edit");
    commit_edit_graph(&library.root_path, edited).expect("commit edit over sidecar_newer");
    let sidecar_newer_status = get_photo_sidecar_status(&library.root_path, &photo_id)
        .expect("read sidecar newer status")
        .expect("sidecar newer status");
    assert_eq!(sidecar_newer_status.conflict_state, "sidecar_newer");

    remove_library_root(&workspace);
}

#[test]
fn failed_sidecar_write_does_not_replace_existing_valid_sidecar() {
    let workspace = unique_library_root("sidecar-write-failure");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let first =
        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("first write");
    let first_bytes = std::fs::read(&first.sidecar_path).expect("read first sidecar");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    connection
        .execute(
            "UPDATE photo_flags SET color_label = 'cyan' WHERE photo_id = ?1",
            params![photo_id],
        )
        .expect("force invalid catalog label");
    drop(connection);

    let error = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect_err("invalid write should fail");
    assert!(error
        .to_string()
        .contains("unsupported sidecar color label"));
    assert_eq!(
        std::fs::read(&first.sidecar_path).expect("read preserved sidecar"),
        first_bytes
    );

    remove_library_root(&workspace);
}

#[test]
fn reads_valid_sidecar_without_mutating_catalog_flags() {
    let workspace = unique_library_root("sidecar-read");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        4,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set sidecar flags");
    write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");
    set_photo_flags(&library.root_path, photo_id.clone(), 1, false, true, None)
        .expect("change catalog flags after write");

    let sidecar = read_photo_sidecar(&library.root_path, &photo_id)
        .expect("read sidecar")
        .expect("sidecar exists");
    assert_eq!(sidecar.photo_id, photo_id);
    assert_eq!(sidecar.flags.rating, 4);
    assert!(sidecar.flags.picked);
    assert!(!sidecar.flags.rejected);
    assert_eq!(sidecar.flags.color_label.as_deref(), Some("green"));
    assert_eq!(sidecar.edit_graph.metadata.rating, 4);

    let live_flags = get_photo_flags(&library.root_path, &sidecar.photo_id)
        .expect("read live flags")
        .expect("live flags");
    assert_eq!(live_flags.rating, 1);
    assert!(!live_flags.picked);
    assert!(live_flags.rejected);
    assert_eq!(live_flags.color_label, None);

    remove_library_root(&workspace);
}

#[test]
fn sidecar_read_rejects_malformed_and_mismatched_payloads() {
    let workspace = unique_library_root("sidecar-read-invalid");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let sidecar_path = sidecar_path_for_photo(&library.root_path, &photo_id).expect("sidecar path");
    std::fs::write(&sidecar_path, b"{not json").expect("write malformed");
    assert!(
        read_photo_sidecar(&library.root_path, &photo_id).is_err(),
        "malformed sidecar must fail"
    );

    write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write valid sidecar");
    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&sidecar_path).expect("read sidecar"))
            .expect("parse sidecar");
    value["photo"]["photo_id"] = serde_json::Value::String("other-photo".to_string());
    std::fs::write(
        &sidecar_path,
        serde_json::to_vec_pretty(&value).expect("serialize mismatch"),
    )
    .expect("write mismatch");
    let error =
        read_photo_sidecar(&library.root_path, &photo_id).expect_err("photo id mismatch must fail");
    assert!(error.to_string().contains("sidecar photo id mismatch"));

    remove_library_root(&workspace);
}

#[test]
fn sidecar_read_rejects_schema_invalid_fields() {
    let workspace = unique_library_root("sidecar-read-schema-invalid");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let result = write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1")
        .expect("write valid sidecar");
    let mut value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&result.sidecar_path).expect("read sidecar"))
            .expect("parse sidecar");

    value["unexpected"] = serde_json::Value::String("not allowed".to_string());
    std::fs::write(
        &result.sidecar_path,
        serde_json::to_vec_pretty(&value).expect("serialize top-level extra"),
    )
    .expect("write top-level extra");
    let error = read_photo_sidecar(&library.root_path, &photo_id)
        .expect_err("top-level extra field must fail");
    assert!(error.to_string().contains("unsupported top-level field"));

    value
        .as_object_mut()
        .expect("sidecar object")
        .remove("unexpected");
    value["sync"]["status"] = serde_json::Value::String("proof_claim".to_string());
    std::fs::write(
        &result.sidecar_path,
        serde_json::to_vec_pretty(&value).expect("serialize bad sync"),
    )
    .expect("write bad sync");
    let error =
        read_photo_sidecar(&library.root_path, &photo_id).expect_err("bad sync status must fail");
    assert!(error.to_string().contains("sidecar.sync.status"));

    remove_library_root(&workspace);
}

#[test]
fn rebuild_dry_run_reports_deterministic_updates_without_mutating_catalog() {
    let workspace = unique_library_root("sidecar-rebuild-dry-run");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        5,
        true,
        false,
        Some("purple".to_string()),
    )
    .expect("set sidecar flags");
    write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");
    set_photo_flags(&library.root_path, photo_id.clone(), 1, false, true, None)
        .expect("change live catalog flags");

    let report =
        dry_run_catalog_rebuild_from_sidecars(&library.root_path).expect("dry-run sidecar rebuild");
    let second_report = dry_run_catalog_rebuild_from_sidecars(&library.root_path)
        .expect("repeat dry-run sidecar rebuild");

    assert_eq!(report, second_report, "dry-run output must be stable");
    assert_eq!(report.sidecars_scanned, 1);
    assert!(report.issues.is_empty());
    assert_eq!(report.entries.len(), 1);
    let entry = &report.entries[0];
    assert_eq!(entry.photo_id, photo_id);
    assert_eq!(entry.action, CatalogRebuildDryRunAction::UpdatePhotoFlags);
    assert_eq!(entry.flag_source, CatalogRebuildFlagSource::SidecarFlags);
    assert_eq!(entry.resolved_flags.rating, 5);
    assert!(entry.resolved_flags.picked);
    assert!(!entry.resolved_flags.rejected);
    assert_eq!(entry.resolved_flags.color_label.as_deref(), Some("purple"));

    let live_flags = get_photo_flags(&library.root_path, &photo_id)
        .expect("read live flags")
        .expect("live flags");
    assert_eq!(live_flags.rating, 1);
    assert!(!live_flags.picked);
    assert!(live_flags.rejected);
    assert_eq!(live_flags.color_label, None);

    remove_library_root(&workspace);
}

#[test]
fn rebuild_dry_run_reports_schema_invalid_sidecars_without_entries() {
    let workspace = unique_library_root("sidecar-rebuild-schema-invalid");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let metadata_file = import_root.join("metadata.jpg");
    let defaults_file = import_root.join("defaults.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&metadata_file, b"metadata jpeg bytes").expect("write metadata original");
    std::fs::write(&defaults_file, b"defaults jpeg bytes").expect("write defaults original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let metadata_photo_id = stable_catalog_id("photo", &metadata_file.display().to_string());
    let defaults_photo_id = stable_catalog_id("photo", &defaults_file.display().to_string());

    let metadata_sidecar =
        write_photo_sidecar(&library.root_path, &metadata_photo_id, "0.1.0-alpha.1")
            .expect("write metadata sidecar");
    let mut metadata_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&metadata_sidecar.sidecar_path).expect("read sidecar"),
    )
    .expect("parse metadata sidecar");
    metadata_value
        .as_object_mut()
        .expect("sidecar object")
        .remove("flags");
    metadata_value["edit_graph"]["metadata"]["rating"] = serde_json::json!(2);
    metadata_value["edit_graph"]["metadata"]["picked"] = serde_json::json!(true);
    metadata_value["edit_graph"]["metadata"]["rejected"] = serde_json::json!(false);
    metadata_value["edit_graph"]["metadata"]["color_label"] = serde_json::json!("blue");
    std::fs::write(
        &metadata_sidecar.sidecar_path,
        serde_json::to_vec_pretty(&metadata_value).expect("serialize metadata fallback"),
    )
    .expect("write metadata fallback sidecar");

    let defaults_sidecar =
        write_photo_sidecar(&library.root_path, &defaults_photo_id, "0.1.0-alpha.1")
            .expect("write defaults sidecar");
    let mut defaults_value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&defaults_sidecar.sidecar_path).expect("read sidecar"),
    )
    .expect("parse defaults sidecar");
    defaults_value["flags"]["rating"] = serde_json::json!(99);
    defaults_value["edit_graph"]["metadata"]["rating"] = serde_json::json!(9);
    std::fs::write(
        &defaults_sidecar.sidecar_path,
        serde_json::to_vec_pretty(&defaults_value).expect("serialize defaults fallback"),
    )
    .expect("write defaults fallback sidecar");

    let report =
        dry_run_catalog_rebuild_from_sidecars(&library.root_path).expect("dry-run sidecar rebuild");

    assert_eq!(report.sidecars_scanned, 2);
    assert_eq!(
        report
            .issues
            .iter()
            .filter(|issue| issue.kind == CatalogRebuildDryRunIssueKind::SchemaInvalid)
            .count(),
        2,
        "schema-invalid sidecars must be reported"
    );
    assert!(report.entries.is_empty());
    assert!(report.issues.iter().any(|issue| {
        issue.photo_id.as_deref() == Some(metadata_photo_id.as_str())
            && issue.sidecar_relative_path
                == format!("{SIDECAR_DIRECTORY}/{metadata_photo_id}{SIDECAR_FILE_SUFFIX}")
    }));
    assert!(report.issues.iter().any(|issue| {
        issue.photo_id.as_deref() == Some(defaults_photo_id.as_str())
            && issue.sidecar_relative_path
                == format!("{SIDECAR_DIRECTORY}/{defaults_photo_id}{SIDECAR_FILE_SUFFIX}")
    }));

    remove_library_root(&workspace);
}

#[test]
fn rebuild_dry_run_reports_conflicts_and_identity_mismatch() {
    let workspace = unique_library_root("sidecar-rebuild-conflicts");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        4,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set flags");
    let sidecar =
        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");
    let mut value: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&sidecar.sidecar_path).expect("read sidecar"),
    )
    .expect("parse sidecar");
    value["flags"]["rating"] = serde_json::json!(3);
    value["photo"]["original_path"] = serde_json::json!("/tmp/elsewhere/sample.jpg");
    std::fs::write(
        &sidecar.sidecar_path,
        serde_json::to_vec_pretty(&value).expect("serialize conflict sidecar"),
    )
    .expect("write conflict sidecar");

    let mismatch_path =
        sidecar_path_for_photo(&library.root_path, "mismatch-photo").expect("mismatch path");
    std::fs::write(
        &mismatch_path,
        serde_json::to_vec_pretty(&value).expect("serialize mismatch sidecar"),
    )
    .expect("write mismatch sidecar");

    let report =
        dry_run_catalog_rebuild_from_sidecars(&library.root_path).expect("dry-run sidecar rebuild");

    assert_eq!(report.sidecars_scanned, 2);
    let entry = report
        .entries
        .iter()
        .find(|entry| entry.photo_id == photo_id)
        .expect("entry");
    assert_eq!(entry.flag_source, CatalogRebuildFlagSource::SidecarFlags);
    assert_eq!(entry.resolved_flags.rating, 3);
    assert!(report.issues.iter().any(|issue| {
        issue.kind == CatalogRebuildDryRunIssueKind::FlagsMetadataConflict
            && issue.photo_id.as_deref() == Some(&photo_id)
    }));
    assert!(report.issues.iter().any(|issue| {
        issue.kind == CatalogRebuildDryRunIssueKind::CatalogReconcileConflict
            && issue.photo_id.as_deref() == Some(&photo_id)
    }));
    assert!(report.issues.iter().any(|issue| {
        issue.kind == CatalogRebuildDryRunIssueKind::PhotoIdMismatch
            && issue.sidecar_relative_path
                == format!("{SIDECAR_DIRECTORY}/mismatch-photo.silicaraw.sidecar.json")
    }));

    remove_library_root(&workspace);
}

#[test]
fn rebuild_dry_run_reports_malformed_sidecars_without_entries() {
    let workspace = unique_library_root("sidecar-rebuild-malformed");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let sidecar_path = sidecar_path_for_photo(&library.root_path, &photo_id).expect("sidecar path");
    std::fs::write(&sidecar_path, b"{not json").expect("write malformed sidecar");

    let report =
        dry_run_catalog_rebuild_from_sidecars(&library.root_path).expect("dry-run sidecar rebuild");

    assert_eq!(report.sidecars_scanned, 1);
    assert!(report.entries.is_empty());
    assert_eq!(report.issues.len(), 1);
    assert_eq!(
        report.issues[0].kind,
        CatalogRebuildDryRunIssueKind::MalformedJson
    );
    assert_eq!(
        report.issues[0].photo_id.as_deref(),
        Some(photo_id.as_str())
    );

    remove_library_root(&workspace);
}

#[test]
fn creates_backup_with_catalog_sidecars_manifest_and_excludes_disposable_data() {
    let workspace = unique_library_root("backup-boundaries");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("External Exports");
    let supported_file = import_root.join("sample.jpg");
    let export_file = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");
    std::fs::write(&export_file, b"exported jpeg bytes").expect("write export output");
    let original_bytes = std::fs::read(&supported_file).expect("read original before");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        4,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set flags");
    let sidecar =
        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");
    record_export(
        &library.root_path,
        &photo_id,
        &export_file,
        r#"{"format":"jpeg","color_profile":"srgb"}"#,
    )
    .expect("record export");

    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        let path = library.root_path.join(directory);
        std::fs::write(path.join("sentinel.cache"), b"cache bytes").expect("write cache");
    }
    std::fs::write(
        library.root_path.join("backups").join("old-backup.txt"),
        b"old backup bytes",
    )
    .expect("write old backup sentinel");
    std::fs::write(
        library.root_path.join("logs").join("runtime.log"),
        b"runtime log",
    )
    .expect("write log sentinel");
    let sidecar_temp = library
        .root_path
        .join(SIDECAR_DIRECTORY)
        .join("partial.silicaraw.sidecar.json.tmp");
    std::fs::write(&sidecar_temp, b"partial sidecar temp").expect("write sidecar temp");

    let backup =
        create_library_backup(&library.root_path, "0.1.0-alpha.1").expect("create library backup");

    assert!(backup
        .backup_path
        .starts_with(library.root_path.join("backups")));
    assert!(backup.backup_path.join(CATALOG_DATABASE_FILE).is_file());
    assert!(backup.manifest_path.is_file());
    assert!(backup
        .backup_path
        .join(&sidecar.sidecar_relative_path)
        .is_file());
    assert!(
        !backup
            .backup_path
            .join(SIDECAR_DIRECTORY)
            .join("partial.silicaraw.sidecar.json.tmp")
            .exists(),
        "sidecar temp files must not be copied into backup"
    );
    for directory in [
        "thumbnails",
        "previews",
        "render-cache",
        "ai-cache",
        "exports",
        "logs",
        "backups",
    ] {
        assert!(
            !backup.backup_path.join(directory).exists(),
            "{directory} must not be copied into backup"
        );
    }
    assert!(
        !backup
            .backup_path
            .join(export_file.file_name().expect("export file name"))
            .exists(),
        "export output files must not be followed into backup"
    );
    assert_eq!(
        std::fs::read(&supported_file).expect("read original after"),
        original_bytes
    );

    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&backup.manifest_path).expect("read manifest"),
    )
    .expect("parse manifest");
    assert_eq!(manifest["schema"], BACKUP_SCHEMA);
    assert_eq!(manifest["version"], BACKUP_VERSION);
    assert_eq!(manifest["app_version"], "0.1.0-alpha.1");
    assert_eq!(manifest["catalog_schema_version"], CURRENT_SCHEMA_VERSION);
    assert!(manifest["files"]
        .as_array()
        .expect("manifest file list")
        .contains(&serde_json::Value::String(
            CATALOG_DATABASE_FILE.to_string()
        )));
    assert!(manifest["files"]
        .as_array()
        .expect("manifest file list")
        .contains(&serde_json::Value::String(sidecar.sidecar_relative_path)));

    let backup_connection =
        open_catalog(backup.backup_path.join(CATALOG_DATABASE_FILE)).expect("open backup db");
    let export_count: i64 = backup_connection
        .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
        .expect("count backup exports");
    let sidecar_status_count: i64 = backup_connection
        .query_row("SELECT COUNT(*) FROM sidecar_status", [], |row| row.get(0))
        .expect("count backup sidecar status");
    assert_eq!(export_count, 1);
    assert_eq!(sidecar_status_count, 1);

    remove_library_root(&workspace);
}

#[test]
fn backup_checkpoint_copies_latest_wal_state_without_wal_files() {
    let workspace = unique_library_root("backup-wal");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

    let connection = open_catalog(&library.catalog_path).expect("open writer connection");
    connection
        .execute(
            "UPDATE photo_flags SET rating = 5, picked = 1 WHERE photo_id = ?1",
            params![photo_id],
        )
        .expect("write uncheckpointed flag state");

    let backup =
        create_library_backup(&library.root_path, "0.1.0-alpha.1").expect("create library backup");

    assert!(backup.backup_path.join(CATALOG_DATABASE_FILE).is_file());
    assert!(!backup.backup_path.join("catalog.db-wal").exists());
    assert!(!backup.backup_path.join("catalog.db-shm").exists());

    let backup_connection =
        open_catalog(backup.backup_path.join(CATALOG_DATABASE_FILE)).expect("open backup db");
    let (rating, picked): (i64, i64) = backup_connection
        .query_row(
            "SELECT rating, picked FROM photo_flags WHERE photo_id = ?1",
            params![stable_catalog_id(
                "photo",
                &supported_file.display().to_string()
            )],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("read backup flags");
    assert_eq!(rating, 5);
    assert_eq!(picked, 1);

    drop(connection);
    remove_library_root(&workspace);
}

#[test]
fn restores_backup_to_empty_directory_preserving_catalog_state_and_originals() {
    let workspace = unique_library_root("restore-empty");
    let library_root = workspace.join("SilicaRAW Library");
    let restore_root = workspace.join("Restored Library");
    let import_root = workspace.join("Originals");
    let export_root = workspace.join("External Exports");
    let supported_file = import_root.join("sample.jpg");
    let export_file = export_root.join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(&export_root).expect("create export directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write original");
    std::fs::write(&export_file, b"exported jpeg bytes").expect("write export output");
    let original_bytes = std::fs::read(&supported_file).expect("read original before");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        5,
        true,
        false,
        Some("purple".to_string()),
    )
    .expect("set flags");
    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft")
        .expect("draft");
    let edited = silica_edit::apply_exposure_contrast(&draft, 0.75, -12.0, "unix:7").expect("edit");
    commit_edit_graph(&library.root_path, edited).expect("commit edit");
    let sidecar =
        write_photo_sidecar(&library.root_path, &photo_id, "0.1.0-alpha.1").expect("write sidecar");
    record_export(
        &library.root_path,
        &photo_id,
        &export_file,
        r#"{"format":"jpeg","color_profile":"srgb"}"#,
    )
    .expect("record export");
    let thumbnail_path = library
        .root_path
        .join("thumbnails")
        .join("sample-thumb.jpg");
    let preview_path = library
        .root_path
        .join("previews")
        .join("sample-preview.jpg");
    std::fs::write(&thumbnail_path, b"thumbnail cache bytes").expect("write thumbnail cache");
    std::fs::write(&preview_path, b"preview cache bytes").expect("write preview cache");
    record_thumbnail_cache(
        &library.root_path,
        &photo_id,
        "restore-thumbnail-key",
        &thumbnail_path,
        21,
    )
    .expect("record thumbnail cache");
    record_preview_cache(
        &library.root_path,
        &photo_id,
        "restore-preview-key",
        &preview_path,
        19,
    )
    .expect("record preview cache");
    let backup = create_library_backup(&library.root_path, "0.1.0-alpha.1").expect("create backup");

    let restored =
        restore_library_backup(&backup.backup_path, &restore_root).expect("restore backup");

    assert_eq!(restored.restored_library.root_path, restore_root);
    assert!(restored.rollback_path.is_none());
    assert!(restore_root.join(CATALOG_DATABASE_FILE).is_file());
    assert!(restore_root.join(&sidecar.sidecar_relative_path).is_file());
    assert_eq!(
        std::fs::read(&supported_file).expect("read original after"),
        original_bytes
    );

    let restored_library = open_local_library(&restore_root).expect("open restored library");
    let restored_flags = get_photo_flags(&restored_library.root_path, &photo_id)
        .expect("read restored flags")
        .expect("restored flags");
    assert_eq!(restored_flags.rating, 5);
    assert!(restored_flags.picked);
    assert_eq!(restored_flags.color_label.as_deref(), Some("purple"));

    let restored_graph = load_active_edit_graph_or_default(&restored_library.root_path, &photo_id)
        .expect("read restored edit")
        .expect("restored edit");
    assert_eq!(restored_graph.basic.exposure.as_f64(), Some(0.75));
    assert_eq!(restored_graph.basic.contrast.as_f64(), Some(-12.0));

    let connection = open_catalog(&restored_library.catalog_path).expect("open restored db");
    assert_eq!(count_edit_states(&connection), 1);
    let export_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
        .expect("count exports");
    let sidecar_status_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM sidecar_status", [], |row| row.get(0))
        .expect("count sidecar status");
    let schema_version: i64 = connection
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
            row.get(0)
        })
        .expect("schema version");
    let (edited_flag, exported_flag): (i64, i64) = connection
        .query_row(
            "SELECT edited, exported FROM photo_flags WHERE photo_id = ?1",
            params![photo_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("edited/exported flags");
    assert_eq!(export_count, 1);
    assert_eq!(sidecar_status_count, 1);
    assert_eq!(schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(edited_flag, 1);
    assert_eq!(exported_flag, 1);
    assert!(
        !restore_root
            .join("thumbnails")
            .join("sample-thumb.jpg")
            .exists(),
        "restore must not copy thumbnail cache bytes"
    );
    assert!(
        !restore_root
            .join("previews")
            .join("sample-preview.jpg")
            .exists(),
        "restore must not copy preview cache bytes"
    );
    let cache_status =
        get_disposable_cache_status(&restored_library.root_path).expect("cache status");
    assert_eq!(cache_status.cache_record_count, 2);
    let cache_clear =
        clear_disposable_cache(&restored_library.root_path).expect("clear restored caches");
    assert_eq!(cache_clear.removed_cache_records, 2);
    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        assert!(
            restored_library.root_path.join(directory).is_dir(),
            "{directory} should be recreated after restored cache clear"
        );
    }
    let regenerated_thumbnail_path = restored_library
        .root_path
        .join("thumbnails")
        .join("regenerated-thumb.jpg");
    let regenerated_preview_path = restored_library
        .root_path
        .join("previews")
        .join("regenerated-preview.jpg");
    std::fs::write(&regenerated_thumbnail_path, b"regenerated thumbnail")
        .expect("write regenerated thumbnail");
    std::fs::write(&regenerated_preview_path, b"regenerated preview")
        .expect("write regenerated preview");
    let regenerated_thumbnail = record_thumbnail_cache(
        &restored_library.root_path,
        &photo_id,
        "restore-regenerated-thumbnail-key",
        &regenerated_thumbnail_path,
        23,
    )
    .expect("record regenerated thumbnail cache");
    let regenerated_preview = record_preview_cache(
        &restored_library.root_path,
        &photo_id,
        "restore-regenerated-preview-key",
        &regenerated_preview_path,
        21,
    )
    .expect("record regenerated preview cache");
    assert_eq!(
        regenerated_thumbnail.path,
        path_to_string(&regenerated_thumbnail_path).expect("thumbnail path string")
    );
    assert_eq!(
        regenerated_preview.path,
        path_to_string(&regenerated_preview_path).expect("preview path string")
    );
    assert_eq!(
        get_photo_cache_record(&restored_library.root_path, &photo_id, THUMBNAIL_CACHE_TYPE)
            .expect("read regenerated thumbnail")
            .expect("regenerated thumbnail record")
            .cache_key,
        "restore-regenerated-thumbnail-key"
    );
    assert_eq!(
        get_photo_cache_record(&restored_library.root_path, &photo_id, PREVIEW_CACHE_TYPE)
            .expect("read regenerated preview")
            .expect("regenerated preview record")
            .cache_key,
        "restore-regenerated-preview-key"
    );
    let regenerated_cache_status =
        get_disposable_cache_status(&restored_library.root_path).expect("regenerated cache status");
    assert_eq!(regenerated_cache_status.cache_record_count, 2);

    remove_library_root(&workspace);
}

#[test]
fn restore_into_existing_library_creates_rollback_before_replacing_state() {
    let workspace = unique_library_root("restore-existing");
    let source_root = workspace.join("Source Library");
    let target_root = workspace.join("Target Library");
    let source_import = workspace.join("Source Originals");
    let target_import = workspace.join("Target Originals");
    let source_file = source_import.join("source.jpg");
    let target_file = target_import.join("target.jpg");

    std::fs::create_dir_all(&source_import).expect("create source import");
    std::fs::create_dir_all(&target_import).expect("create target import");
    std::fs::write(&source_file, b"source jpeg bytes").expect("write source");
    std::fs::write(&target_file, b"target jpeg bytes").expect("write target");

    let source_library = create_local_library(&source_root).expect("create source library");
    import_folder(&source_library.root_path, &source_import).expect("import source");
    let source_photo_id = stable_catalog_id("photo", &source_file.display().to_string());
    set_photo_flags(
        &source_library.root_path,
        source_photo_id.clone(),
        4,
        true,
        false,
        None,
    )
    .expect("set source flags");
    write_photo_sidecar(&source_library.root_path, &source_photo_id, "0.1.0-alpha.1")
        .expect("write source sidecar");
    let backup = create_library_backup(&source_library.root_path, "0.1.0-alpha.1").expect("backup");

    let target_library = create_local_library(&target_root).expect("create target library");
    import_folder(&target_library.root_path, &target_import).expect("import target");
    let target_photo_id = stable_catalog_id("photo", &target_file.display().to_string());
    set_photo_flags(
        &target_library.root_path,
        target_photo_id.clone(),
        1,
        false,
        true,
        None,
    )
    .expect("set target flags");
    write_photo_sidecar(&target_library.root_path, &target_photo_id, "0.1.0-alpha.1")
        .expect("write target sidecar");

    let restored = restore_library_backup(&backup.backup_path, &target_root)
        .expect("restore over existing target");
    let rollback_path = restored.rollback_path.expect("rollback path");
    assert!(rollback_path.starts_with(target_root.join(BACKUPS_DIRECTORY)));
    assert!(rollback_path.join(CATALOG_DATABASE_FILE).is_file());
    assert!(
        rollback_path
            .join(SIDECAR_DIRECTORY)
            .join(format!("{target_photo_id}{SIDECAR_FILE_SUFFIX}"))
            .is_file(),
        "rollback must preserve previous target sidecar"
    );

    let restored_items = list_library_photos(&target_root).expect("list restored target");
    assert!(restored_items
        .iter()
        .any(|item| item.photo_id == source_photo_id));
    assert!(!restored_items
        .iter()
        .any(|item| item.photo_id == target_photo_id));

    let rollback_connection =
        open_catalog(rollback_path.join(CATALOG_DATABASE_FILE)).expect("open rollback db");
    let rollback_photo_count: i64 = rollback_connection
        .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))
        .expect("count rollback photos");
    let rollback_target_count: i64 = rollback_connection
        .query_row(
            "SELECT COUNT(*) FROM photos WHERE id = ?1",
            params![target_photo_id],
            |row| row.get(0),
        )
        .expect("count rollback target photo");
    assert_eq!(rollback_photo_count, 1);
    assert_eq!(rollback_target_count, 1);

    remove_library_root(&workspace);
}

#[test]
fn restore_rejects_newer_schema_backup_without_mutating_existing_library() {
    let workspace = unique_library_root("restore-newer-schema");
    let source_root = workspace.join("Source Library");
    let target_root = workspace.join("Target Library");
    let source_import = workspace.join("Source Originals");
    let target_import = workspace.join("Target Originals");
    let source_file = source_import.join("source.jpg");
    let target_file = target_import.join("target.jpg");

    std::fs::create_dir_all(&source_import).expect("create source import");
    std::fs::create_dir_all(&target_import).expect("create target import");
    std::fs::write(&source_file, b"source jpeg bytes").expect("write source");
    std::fs::write(&target_file, b"target jpeg bytes").expect("write target");

    let source_library = create_local_library(&source_root).expect("create source library");
    import_folder(&source_library.root_path, &source_import).expect("import source");
    let backup = create_library_backup(&source_library.root_path, "0.1.0-alpha.1").expect("backup");
    let manifest_path = backup.backup_path.join(BACKUP_MANIFEST_FILE);
    let mut manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&manifest_path).expect("read manifest"))
            .expect("parse manifest");
    manifest["catalog_schema_version"] = serde_json::json!(CURRENT_SCHEMA_VERSION + 1);
    std::fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&manifest).expect("serialize newer manifest"),
    )
    .expect("write newer manifest");

    let target_library = create_local_library(&target_root).expect("create target library");
    import_folder(&target_library.root_path, &target_import).expect("import target");
    let target_photo_id = stable_catalog_id("photo", &target_file.display().to_string());
    set_photo_flags(
        &target_library.root_path,
        target_photo_id.clone(),
        2,
        false,
        true,
        None,
    )
    .expect("set target flags");

    let error = restore_library_backup(&backup.backup_path, &target_root)
        .expect_err("newer schema restore must fail");
    assert!(error.to_string().contains("newer catalog schema"));

    let target_flags = get_photo_flags(&target_root, &target_photo_id)
        .expect("read target flags")
        .expect("target flags still present");
    assert_eq!(target_flags.rating, 2);
    assert!(target_flags.rejected);
    assert!(!target_root
        .join(BACKUPS_DIRECTORY)
        .join("restore-rollback")
        .exists());

    remove_library_root(&workspace);
}

#[test]
fn restore_corrupt_backup_catalog_preserves_existing_target_with_context_error() {
    let workspace = unique_library_root("restore-corrupt-backup");
    let source_root = workspace.join("Source Library");
    let target_root = workspace.join("Target Library");
    let source_import = workspace.join("Source Originals");
    let target_import = workspace.join("Target Originals");
    let source_file = source_import.join("source.jpg");
    let target_file = target_import.join("target.jpg");

    std::fs::create_dir_all(&source_import).expect("create source import");
    std::fs::create_dir_all(&target_import).expect("create target import");
    std::fs::write(&source_file, b"source jpeg bytes").expect("write source");
    std::fs::write(&target_file, b"target jpeg bytes").expect("write target");

    let source_library = create_local_library(&source_root).expect("create source library");
    import_folder(&source_library.root_path, &source_import).expect("import source");
    let backup = create_library_backup(&source_library.root_path, "0.1.0-alpha.1").expect("backup");
    std::fs::write(
        backup.backup_path.join(CATALOG_DATABASE_FILE),
        b"not a sqlite catalog",
    )
    .expect("corrupt backup catalog");

    let target_library = create_local_library(&target_root).expect("create target library");
    import_folder(&target_library.root_path, &target_import).expect("import target");
    let target_photo_id = stable_catalog_id("photo", &target_file.display().to_string());
    set_photo_flags(
        &target_library.root_path,
        target_photo_id.clone(),
        3,
        false,
        true,
        Some("red".to_string()),
    )
    .expect("set target flags");

    let error = restore_library_backup(&backup.backup_path, &target_root)
        .expect_err("corrupt backup catalog restore must fail");
    let message = error.to_string();
    assert!(message.contains(&backup.backup_path.display().to_string()));
    assert!(message.contains(&target_root.display().to_string()));

    let target_flags = get_photo_flags(&target_root, &target_photo_id)
        .expect("read target flags")
        .expect("target flags still present");
    assert_eq!(target_flags.rating, 3);
    assert!(target_flags.rejected);
    assert_eq!(target_flags.color_label.as_deref(), Some("red"));
    assert!(
        !restore_staging_path(&target_root).exists(),
        "failed staging restore should be cleaned up"
    );
    let rollback_entries = std::fs::read_dir(target_root.join(BACKUPS_DIRECTORY))
        .expect("read target backups")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("restore-rollback-")
        })
        .count();
    assert_eq!(
        rollback_entries, 0,
        "rollback should not be created before staging validation succeeds"
    );

    remove_library_root(&workspace);
}

#[test]
fn reopens_existing_library_without_recreating_original_photo_directory() {
    let workspace = unique_library_root("workspace");
    let original_dir = workspace.join("originals");
    let original_file = original_dir.join("sample.dng");
    let library_root = workspace.join("SilicaRAW Library");

    std::fs::create_dir_all(&original_dir).expect("create original directory");
    std::fs::write(&original_file, b"original raw bytes").expect("write original sentinel");
    let original_before = std::fs::read(&original_file).expect("read original before");

    let created = create_local_library(&library_root).expect("create library");
    let reopened = open_local_library(&library_root).expect("reopen library");

    assert_eq!(reopened.root_path, created.root_path);
    assert_eq!(reopened.catalog_path, created.catalog_path);
    assert_eq!(reopened.schema_version, CURRENT_SCHEMA_VERSION);
    assert_eq!(
        std::fs::read(&original_file).expect("read original after"),
        original_before
    );
    assert!(original_dir.is_dir());

    remove_library_root(&workspace);
}

#[test]
fn scans_mixed_folder_and_records_photo_candidates_by_reference() {
    let workspace = unique_library_root("import");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let unsupported_raw_file = import_root.join("sample.DNG");
    let unsupported_file = import_root.join("notes.txt");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"supported jpeg candidate").expect("write supported");
    std::fs::write(&unsupported_raw_file, b"unsupported raw candidate")
        .expect("write unsupported raw");
    std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");
    let supported_before = std::fs::read(&supported_file).expect("read supported before");
    let unsupported_raw_before =
        std::fs::read(&unsupported_raw_file).expect("read unsupported raw before");
    let unsupported_before = std::fs::read(&unsupported_file).expect("read unsupported before");

    let library = create_local_library(&library_root).expect("create library");
    let summary = import_folder(&library.root_path, &import_root).expect("import folder");

    assert_eq!(summary.scanned_files, 3);
    assert_eq!(summary.supported_files, 1);
    assert_eq!(summary.unsupported_files, 2);
    assert!(summary.originals_unchanged);
    assert_eq!(summary.candidates.len(), 3);
    assert!(summary
        .candidates
        .iter()
        .any(|candidate| candidate.file_name == "sample.jpg" && !candidate.unsupported));
    assert!(summary
        .candidates
        .iter()
        .any(|candidate| candidate.file_name == "sample.DNG" && candidate.unsupported));
    assert!(summary
        .candidates
        .iter()
        .any(|candidate| candidate.file_name == "notes.txt" && candidate.unsupported));

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let imported_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))
        .expect("count photos");
    assert_eq!(imported_count, 3);

    let (path, file_size, unsupported, file_type, partial_hash, full_hash): (
        String,
        i64,
        i64,
        String,
        String,
        String,
    ) = connection
        .query_row(
            "SELECT path, file_size, unsupported, file_type, partial_hash, full_hash FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .expect("supported row");
    assert_eq!(path, supported_file.display().to_string());
    assert_eq!(file_size, supported_before.len() as i64);
    assert_eq!(unsupported, 0);
    assert_eq!(file_type, "jpeg");
    assert!(!partial_hash.is_empty());
    assert_eq!(full_hash.len(), 64);

    let (unsupported, file_type): (i64, String) = connection
        .query_row(
            "SELECT unsupported, file_type FROM photos WHERE file_name = 'sample.DNG'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("unsupported raw row");
    assert_eq!(unsupported, 1);
    assert_eq!(file_type, "unsupported");

    let (unsupported, file_type): (i64, String) = connection
        .query_row(
            "SELECT unsupported, file_type FROM photos WHERE file_name = 'notes.txt'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("unsupported row");
    assert_eq!(unsupported, 1);
    assert_eq!(file_type, "unsupported");

    assert_eq!(
        std::fs::read(&supported_file).expect("read supported after"),
        supported_before
    );
    assert_eq!(
        std::fs::read(&unsupported_raw_file).expect("read unsupported raw after"),
        unsupported_raw_before
    );
    assert_eq!(
        std::fs::read(&unsupported_file).expect("read unsupported after"),
        unsupported_before
    );
    assert!(!library_root.join("sample.jpg").exists());
    assert!(!library_root.join("sample.DNG").exists());
    assert!(!library_root.join("notes.txt").exists());

    remove_library_root(&workspace);
}

#[test]
fn import_fingerprint_check_rejects_full_hash_mismatch() {
    let workspace = unique_library_root("import-fingerprint");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"supported jpeg candidate").expect("write supported");
    let metadata = std::fs::metadata(&supported_file).expect("read metadata");
    let candidate = ImportCandidate {
        file_name: "sample.jpg".to_string(),
        path: path_to_string(&supported_file).expect("source path"),
        file_size: metadata.len() as i64,
        modified_at: modified_at_string(&metadata),
        partial_hash: partial_file_hash(&supported_file).expect("partial hash"),
        full_hash: Some("0".repeat(64)),
        unsupported: false,
    };

    assert!(!import_candidates_fingerprints_unchanged(&[candidate]));

    remove_library_root(&workspace);
}

#[test]
fn import_error_summary_reports_recoverable_issues_without_blocking_browse() {
    let workspace = unique_library_root("import-errors");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let unsupported_file = import_root.join("notes.txt");
    let hidden_file = import_root.join(".hidden.jpg");
    let package_dir = import_root.join("Archive.photoslibrary");
    let symlink_path = import_root.join("linked");

    std::fs::create_dir_all(&package_dir).expect("create package directory");
    std::fs::write(&supported_file, b"supported jpeg candidate").expect("write supported");
    std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");
    std::fs::write(&hidden_file, b"hidden jpeg candidate").expect("write hidden");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&supported_file, &symlink_path).expect("create symlink");

    let library = create_local_library(&library_root).expect("create library");
    let summary = import_folder(&library.root_path, &import_root).expect("import folder");

    assert_eq!(summary.scanned_files, 2);
    assert_eq!(summary.supported_files, 1);
    assert_eq!(summary.unsupported_files, 1);
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::UnsupportedFile,
        "notes.txt",
    );
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::HiddenEntrySkipped,
        ".hidden.jpg",
    );
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::PackageDirectorySkipped,
        "Archive.photoslibrary",
    );
    #[cfg(unix)]
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::SymlinkEntrySkipped,
        "linked",
    );

    let items = list_library_photos(&library.root_path).expect("list imported rows");
    assert!(items.iter().any(|item| item.file_name == "sample.jpg"));
    assert!(items
        .iter()
        .any(|item| item.file_name == "notes.txt" && item.unsupported));
    assert!(!items.iter().any(|item| item.file_name == ".hidden.jpg"));

    remove_library_root(&workspace);
}

#[test]
fn recursive_import_opt_in_scans_nested_entries_and_reports_issues() {
    let workspace = unique_library_root("recursive-import");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let root_file = import_root.join("root.jpg");
    let nested_root = import_root.join("Nested");
    let nested_file = nested_root.join("child.jpg");
    let nested_unsupported = nested_root.join("notes.txt");
    let hidden_file = nested_root.join(".hidden.jpg");
    let package_dir = nested_root.join("Archive.photoslibrary");
    let symlink_path = nested_root.join("linked");

    std::fs::create_dir_all(&package_dir).expect("create nested package directory");
    std::fs::write(&root_file, b"root jpeg candidate").expect("write root");
    std::fs::write(&nested_file, b"nested jpeg candidate").expect("write nested");
    std::fs::write(&nested_unsupported, b"unsupported note").expect("write unsupported");
    std::fs::write(&hidden_file, b"hidden jpeg candidate").expect("write hidden");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&nested_file, &symlink_path).expect("create symlink");

    let library = create_local_library(&library_root).expect("create library");
    let default_summary = import_folder(&library.root_path, &import_root).expect("default import");
    assert_eq!(default_summary.scanned_files, 1);
    assert!(!default_summary
        .candidates
        .iter()
        .any(|candidate| candidate.file_name == "child.jpg"));

    let summary = import_folder_with_options(
        &library.root_path,
        &import_root,
        FolderImportOptions { recursive: true },
    )
    .expect("recursive import");

    assert_eq!(summary.scanned_files, 3);
    assert_eq!(summary.supported_files, 2);
    assert_eq!(summary.unsupported_files, 1);
    assert!(summary
        .candidates
        .iter()
        .any(|candidate| candidate.file_name == "child.jpg"));
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::UnsupportedFile,
        "notes.txt",
    );
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::HiddenEntrySkipped,
        ".hidden.jpg",
    );
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::PackageDirectorySkipped,
        "Archive.photoslibrary",
    );
    #[cfg(unix)]
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::SymlinkEntrySkipped,
        "linked",
    );

    let items = list_library_photos(&library.root_path).expect("list recursive rows");
    assert!(items.iter().any(|item| item.file_name == "root.jpg"));
    assert!(items.iter().any(|item| item.file_name == "child.jpg"));
    assert!(items
        .iter()
        .any(|item| item.file_name == "notes.txt" && item.unsupported));
    assert!(!items.iter().any(|item| item.file_name == ".hidden.jpg"));

    remove_library_root(&workspace);
}

#[test]
fn recursive_import_skips_policy_rejected_selected_root_without_descending() {
    let workspace = unique_library_root("recursive-root-policy");
    let library_root = workspace.join("SilicaRAW Library");
    let package_root = workspace.join("Archive.photoslibrary");
    let package_file = package_root.join("sample.jpg");

    std::fs::create_dir_all(&package_root).expect("create package root");
    std::fs::write(&package_file, b"package jpeg candidate").expect("write package file");

    let library = create_local_library(&library_root).expect("create library");
    let summary = import_folder_with_options(
        &library.root_path,
        &package_root,
        FolderImportOptions { recursive: true },
    )
    .expect("skip package root");

    assert_eq!(summary.scanned_files, 0);
    assert_issue_kind(
        &summary.issues,
        ImportIssueKind::PackageDirectorySkipped,
        "Archive.photoslibrary",
    );
    let items = list_library_photos(&library.root_path).expect("list package skip rows");
    assert!(items.is_empty());

    #[cfg(unix)]
    {
        let real_root = workspace.join("RealOriginals");
        let real_file = real_root.join("linked.jpg");
        let symlink_root = workspace.join("LinkedOriginals");
        std::fs::create_dir_all(&real_root).expect("create real root");
        std::fs::write(&real_file, b"linked jpeg candidate").expect("write linked file");
        std::os::unix::fs::symlink(&real_root, &symlink_root).expect("create root symlink");

        let symlink_summary = import_folder_with_options(
            &library.root_path,
            &symlink_root,
            FolderImportOptions { recursive: true },
        )
        .expect("skip symlink root");

        assert_eq!(symlink_summary.scanned_files, 0);
        assert_issue_kind(
            &symlink_summary.issues,
            ImportIssueKind::SymlinkEntrySkipped,
            "LinkedOriginals",
        );
        let items = list_library_photos(&library.root_path).expect("list symlink skip rows");
        assert!(items.is_empty());
    }

    remove_library_root(&workspace);
}

#[test]
fn lists_library_photo_grid_items_with_flags_and_states() {
    let workspace = unique_library_root("grid-items");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let unsupported_file = import_root.join("sample.DNG");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"supported jpeg candidate").expect("write supported");
    std::fs::write(&unsupported_file, b"unsupported raw candidate").expect("write unsupported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let supported_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        supported_id.clone(),
        4,
        true,
        false,
        Some("green".to_string()),
    )
    .expect("set grid flags");

    let items = list_library_photos(&library.root_path).expect("list library photos");

    assert_eq!(items.len(), 2);
    let supported = items
        .iter()
        .find(|item| item.file_name == "sample.jpg")
        .expect("supported grid item");
    assert_eq!(supported.photo_id, supported_id);
    assert_eq!(supported.file_type, "JPG");
    assert_eq!(supported.rating, 4);
    assert!(supported.picked);
    assert!(!supported.rejected);
    assert_eq!(supported.color_label.as_deref(), Some("green"));
    assert!(!supported.missing);
    assert!(!supported.unsupported);

    let unsupported = items
        .iter()
        .find(|item| item.file_name == "sample.DNG")
        .expect("unsupported grid item");
    assert_eq!(unsupported.file_type, "DNG");
    assert!(unsupported.unsupported);
    assert_eq!(unsupported.rating, 0);

    remove_library_root(&workspace);
}

#[test]
fn records_thumbnail_cache_and_exposes_grid_path() {
    let workspace = unique_library_root("thumbnail-cache");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let thumbnail_path = library
        .root_path
        .join("thumbnails")
        .join("sample-thumb.jpg");
    std::fs::write(&thumbnail_path, b"thumbnail bytes").expect("write thumbnail");

    let record = record_thumbnail_cache(
        &library.root_path,
        &photo_id,
        "thumbnail-key",
        &thumbnail_path,
        15,
    )
    .expect("record thumbnail cache");
    assert_eq!(record.photo_id.as_deref(), Some(photo_id.as_str()));
    assert_eq!(record.cache_type, THUMBNAIL_CACHE_TYPE);
    assert_eq!(record.path, thumbnail_path.display().to_string());
    assert_eq!(record.byte_size, 15);

    let items = list_library_photos(&library.root_path).expect("list grid items");
    let item = items
        .iter()
        .find(|item| item.file_name == "sample.jpg")
        .expect("jpeg grid item");
    assert_eq!(
        item.thumbnail_path.as_deref(),
        Some(thumbnail_path.display().to_string().as_str())
    );
    assert_eq!(item.thumbnail_cache_key.as_deref(), Some("thumbnail-key"));

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let cache_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'thumbnail'",
            [],
            |row| row.get(0),
        )
        .expect("count thumbnail cache rows");
    assert_eq!(cache_count, 1);

    remove_library_root(&workspace);
}

#[test]
fn records_preview_cache_and_reads_it_by_photo_type() {
    let workspace = unique_library_root("preview-cache");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let preview_path = library
        .root_path
        .join("previews")
        .join("sample-preview.jpg");
    std::fs::write(&preview_path, b"preview bytes").expect("write preview");

    let record = record_preview_cache(
        &library.root_path,
        &photo_id,
        "preview-key",
        &preview_path,
        13,
    )
    .expect("record preview cache");
    assert_eq!(record.cache_type, PREVIEW_CACHE_TYPE);

    let cached = get_photo_cache_record(&library.root_path, &photo_id, PREVIEW_CACHE_TYPE)
        .expect("read preview cache")
        .expect("preview cache row");
    assert_eq!(cached.path, preview_path.display().to_string());
    assert_eq!(cached.cache_key, "preview-key");
    assert_eq!(cached.byte_size, 13);

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let cache_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM cache_records WHERE cache_type = 'preview'",
            [],
            |row| row.get(0),
        )
        .expect("count preview cache rows");
    assert_eq!(cache_count, 1);

    remove_library_root(&workspace);
}

#[test]
fn records_histogram_cache_under_render_cache_only() {
    let workspace = unique_library_root("histogram-cache");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let histogram_path = library
        .root_path
        .join("render-cache")
        .join("sample-histogram.json");
    std::fs::write(&histogram_path, br#"{"pixel_count":2}"#).expect("write histogram");

    let record = record_histogram_cache(
        &library.root_path,
        &photo_id,
        "histogram-key",
        &histogram_path,
        17,
    )
    .expect("record histogram cache");
    assert_eq!(record.cache_type, HISTOGRAM_CACHE_TYPE);

    let cached = get_photo_cache_record(&library.root_path, &photo_id, HISTOGRAM_CACHE_TYPE)
        .expect("read histogram cache")
        .expect("histogram cache row");
    assert_eq!(cached.path, histogram_path.display().to_string());
    assert_eq!(cached.cache_key, "histogram-key");
    assert_eq!(cached.byte_size, 17);

    let outside_path = workspace.join("outside-histogram.json");
    std::fs::write(&outside_path, br#"{"pixel_count":2}"#).expect("write outside histogram");
    let error = record_histogram_cache(
        &library.root_path,
        &photo_id,
        "histogram-key",
        &outside_path,
        17,
    )
    .expect_err("histogram cache path outside render-cache/ must be rejected");
    assert!(matches!(
        error,
        LibraryStorageError::CacheValidation(message)
            if message.contains("render-cache")
    ));

    remove_library_root(&workspace);
}

#[test]
fn records_mask_raster_cache_under_render_cache_masks_only() {
    let workspace = unique_library_root("mask-raster-cache");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let mask_dir = library.root_path.join("render-cache").join("masks");
    std::fs::create_dir_all(&mask_dir).expect("create mask cache directory");
    let mask_path = mask_dir.join("mask-brush-1.mask8");
    std::fs::write(&mask_path, [0_u8, 255, 0, 255]).expect("write mask raster");

    let record = record_mask_raster_cache(
        &library.root_path,
        &photo_id,
        "brush-mask-v1-test",
        &mask_path,
        4,
    )
    .expect("record mask raster cache");
    assert_eq!(record.cache_type, MASK_RASTER_CACHE_TYPE);

    let cached = get_photo_cache_record(&library.root_path, &photo_id, MASK_RASTER_CACHE_TYPE)
        .expect("read mask cache")
        .expect("mask cache row");
    assert_eq!(cached.path, mask_path.display().to_string());

    let outside_masks = library
        .root_path
        .join("render-cache")
        .join("mask-brush-1.mask8");
    std::fs::write(&outside_masks, [255_u8]).expect("write outside masks directory");
    let error = record_mask_raster_cache(
        &library.root_path,
        &photo_id,
        "brush-mask-v1-test",
        &outside_masks,
        1,
    )
    .expect_err("mask raster cache outside render-cache/masks/ must be rejected");
    assert!(matches!(
        error,
        LibraryStorageError::CacheValidation(message)
            if message.contains("render-cache/masks")
    ));

    remove_library_root(&workspace);
}

#[test]
fn clear_disposable_cache_removes_mask_raster_records_and_artifacts() {
    let workspace = unique_library_root("mask-raster-clear");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let mask_dir = library.root_path.join("render-cache").join("masks");
    std::fs::create_dir_all(&mask_dir).expect("create mask cache directory");
    let mask_path = mask_dir.join("mask-brush-1.mask8");
    std::fs::write(&mask_path, [0_u8, 255, 0, 255]).expect("write mask raster");
    record_mask_raster_cache(
        &library.root_path,
        &photo_id,
        "brush-mask-v1-test",
        &mask_path,
        4,
    )
    .expect("record mask raster cache");

    let summary = clear_disposable_cache(&library.root_path).expect("clear cache");

    assert_eq!(summary.removed_cache_records, 1);
    assert!(!mask_path.exists());
    assert!(
        get_photo_cache_record(&library.root_path, &photo_id, MASK_RASTER_CACHE_TYPE)
            .expect("read mask cache after clear")
            .is_none()
    );

    remove_library_root(&workspace);
}

#[test]
fn rejects_preview_cache_records_outside_disposable_preview_directory() {
    let workspace = unique_library_root("preview-cache-outside-root");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let outside_path = workspace.join("outside-preview.jpg");
    std::fs::write(&outside_path, b"preview bytes").expect("write outside preview");

    let error = record_preview_cache(
        &library.root_path,
        &photo_id,
        "preview-key",
        &outside_path,
        13,
    )
    .expect_err("preview cache path outside previews/ must be rejected");

    assert!(matches!(
        error,
        LibraryStorageError::CacheValidation(message)
            if message.contains("previews")
    ));

    remove_library_root(&workspace);
}

#[test]
fn rejects_preview_cache_records_that_escape_preview_directory() {
    let workspace = unique_library_root("preview-cache-parent-escape");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let escaped_path = library
        .root_path
        .join("previews")
        .join("..")
        .join("escaped-preview.jpg");
    std::fs::write(
        library.root_path.join("escaped-preview.jpg"),
        b"preview bytes",
    )
    .expect("write escaped preview");

    let error = record_preview_cache(
        &library.root_path,
        &photo_id,
        "preview-key",
        &escaped_path,
        13,
    )
    .expect_err("preview cache path cannot escape previews/");

    assert!(matches!(
        error,
        LibraryStorageError::CacheValidation(message)
            if message.contains("previews")
    ));

    remove_library_root(&workspace);
}

#[test]
fn clears_only_disposable_cache_directories() {
    let workspace = unique_library_root("clear-cache");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
    let original_bytes = std::fs::read(&supported_file).expect("read original before");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        let path = library.root_path.join(directory);
        std::fs::create_dir_all(&path).expect("create cache directory");
        std::fs::write(path.join("sentinel.cache"), b"disposable cache bytes")
            .expect("write cache sentinel");
    }
    for directory in ["sidecars", "exports", "logs", "backups"] {
        let path = library.root_path.join(directory);
        std::fs::create_dir_all(&path).expect("create protected directory");
        std::fs::write(path.join("keep.txt"), b"preserve this").expect("write protected file");
    }

    let thumbnail_path = library
        .root_path
        .join("thumbnails")
        .join("sample-thumb.jpg");
    let preview_path = library
        .root_path
        .join("previews")
        .join("sample-preview.jpg");
    std::fs::write(&thumbnail_path, b"thumbnail bytes").expect("write thumbnail cache");
    std::fs::write(&preview_path, b"preview bytes").expect("write preview cache");
    record_thumbnail_cache(
        &library.root_path,
        &photo_id,
        "thumbnail-key",
        &thumbnail_path,
        15,
    )
    .expect("record thumbnail cache");
    record_preview_cache(
        &library.root_path,
        &photo_id,
        "preview-key",
        &preview_path,
        13,
    )
    .expect("record preview cache");

    let summary = clear_disposable_cache(&library.root_path).expect("clear cache");

    assert_eq!(
        summary.cleared_directories,
        DISPOSABLE_CACHE_DIRECTORIES
            .iter()
            .map(|directory| directory.to_string())
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.removed_cache_records, 2);
    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        let path = library.root_path.join(directory);
        assert!(path.is_dir(), "{directory} should be recreated");
        assert!(
            !path.join("sentinel.cache").exists(),
            "{directory} sentinel should be removed"
        );
    }
    for directory in ["sidecars", "exports", "logs", "backups"] {
        assert!(
            library.root_path.join(directory).join("keep.txt").is_file(),
            "{directory} should be preserved"
        );
    }

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM photos", [], |row| row.get(0))
        .expect("count photos");
    let cache_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM cache_records", [], |row| row.get(0))
        .expect("count cache records");
    assert_eq!(photo_count, 1);
    assert_eq!(cache_count, 0);
    assert_eq!(
        std::fs::read(&supported_file).expect("read original after"),
        original_bytes
    );

    remove_library_root(&workspace);
}

#[cfg(unix)]
#[test]
fn clear_disposable_cache_removes_symlinks_without_following_targets() {
    let workspace = unique_library_root("clear-cache-symlink-boundary");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"original bytes").expect("write original");
    let original_bytes = std::fs::read(&supported_file).expect("read original before");
    let library = create_local_library(&library_root).expect("create library");

    let protected_targets = [
        ("sidecars", "sidecar keep"),
        ("exports", "export keep"),
        ("backups", "backup keep"),
        ("logs", "log keep"),
    ];
    for (directory, file_name) in protected_targets {
        let path = library.root_path.join(directory);
        std::fs::create_dir_all(&path).expect("create protected directory");
        std::fs::write(path.join(file_name), b"protected").expect("write protected file");
    }
    let protected_bytes = protected_targets
        .iter()
        .map(|(directory, file_name)| {
            (
                (*directory).to_string(),
                (*file_name).to_string(),
                std::fs::read(library.root_path.join(directory).join(file_name))
                    .expect("read protected file before"),
            )
        })
        .collect::<Vec<_>>();

    let top_level_links = [
        ("thumbnails", "sidecars"),
        ("previews", "exports"),
        ("render-cache", "backups"),
        ("ai-cache", "logs"),
    ];
    for (cache_directory, target_directory) in top_level_links {
        let cache_path = library.root_path.join(cache_directory);
        if cache_path.exists() {
            std::fs::remove_dir_all(&cache_path).expect("remove cache directory");
        }
        std::os::unix::fs::symlink(library.root_path.join(target_directory), &cache_path)
            .expect("create top-level cache symlink");
        assert!(std::fs::symlink_metadata(&cache_path)
            .expect("cache symlink metadata")
            .file_type()
            .is_symlink());
    }

    clear_disposable_cache(&library.root_path).expect("clear top-level symlink caches");

    for directory in DISPOSABLE_CACHE_DIRECTORIES {
        let metadata = std::fs::symlink_metadata(library.root_path.join(directory))
            .expect("cache directory metadata");
        assert!(
            metadata.file_type().is_dir(),
            "{directory} should be a directory"
        );
        assert!(
            !metadata.file_type().is_symlink(),
            "{directory} should be recreated as a real directory"
        );
    }
    for (directory, file_name, bytes) in &protected_bytes {
        assert_eq!(
            std::fs::read(library.root_path.join(directory).join(file_name))
                .expect("read protected file after"),
            *bytes,
            "{directory} target bytes should be preserved"
        );
    }

    let top_level_original_link = library.root_path.join("thumbnails");
    std::fs::remove_dir_all(&top_level_original_link)
        .expect("remove recreated thumbnails directory");
    std::os::unix::fs::symlink(&import_root, &top_level_original_link)
        .expect("create top-level original symlink");
    clear_disposable_cache(&library.root_path).expect("clear top-level original symlink cache");
    let top_level_original_metadata =
        std::fs::symlink_metadata(&top_level_original_link).expect("thumbnail metadata");
    assert!(top_level_original_metadata.file_type().is_dir());
    assert!(!top_level_original_metadata.file_type().is_symlink());
    assert_eq!(
        std::fs::read(&supported_file).expect("read original after top-level symlink"),
        original_bytes
    );

    let nested_original_link = library
        .root_path
        .join("thumbnails")
        .join("linked-originals");
    std::os::unix::fs::symlink(&import_root, &nested_original_link)
        .expect("create nested original symlink");
    assert!(std::fs::symlink_metadata(&nested_original_link)
        .expect("nested symlink metadata")
        .file_type()
        .is_symlink());

    clear_disposable_cache(&library.root_path).expect("clear nested symlink cache");

    assert_eq!(
        std::fs::read(&supported_file).expect("read original after nested symlink"),
        original_bytes
    );
    assert!(
        std::fs::symlink_metadata(&nested_original_link).is_err(),
        "nested symlink should be removed with disposable cache directory"
    );

    remove_library_root(&workspace);
}

#[test]
fn disposable_cache_status_reports_real_paths_and_sizes() {
    let workspace = unique_library_root("cache-status");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

    let thumbnail_path = library
        .root_path
        .join("thumbnails")
        .join("sample-thumb.jpg");
    let preview_path = library
        .root_path
        .join("previews")
        .join("sample-preview.jpg");
    let histogram_path = library
        .root_path
        .join("render-cache")
        .join("histogram.json");
    std::fs::write(&thumbnail_path, b"thumb").expect("write thumbnail");
    std::fs::write(&preview_path, b"preview").expect("write preview");
    std::fs::write(&histogram_path, b"histogram").expect("write histogram");
    std::fs::create_dir_all(library.root_path.join("exports")).expect("create exports");
    std::fs::write(
        library.root_path.join("exports").join("keep.jpg"),
        b"not cache",
    )
    .expect("write export");
    record_thumbnail_cache(
        &library.root_path,
        &photo_id,
        "thumbnail-key",
        &thumbnail_path,
        5,
    )
    .expect("record thumbnail cache");
    record_preview_cache(
        &library.root_path,
        &photo_id,
        "preview-key",
        &preview_path,
        7,
    )
    .expect("record preview cache");

    let status = get_disposable_cache_status(&library.root_path).expect("cache status");

    assert_eq!(
        status
            .directories
            .iter()
            .map(|directory| directory.name.as_str())
            .collect::<Vec<_>>(),
        DISPOSABLE_CACHE_DIRECTORIES
    );
    assert_eq!(status.total_bytes, 21);
    assert_eq!(status.cache_record_count, 2);
    assert_eq!(
        status
            .directories
            .iter()
            .find(|directory| directory.name == "thumbnails")
            .expect("thumbnail status")
            .byte_size,
        5
    );
    assert_eq!(
        status
            .directories
            .iter()
            .find(|directory| directory.name == "previews")
            .expect("preview status")
            .byte_size,
        7
    );
    assert_eq!(
        status
            .directories
            .iter()
            .find(|directory| directory.name == "render-cache")
            .expect("render status")
            .byte_size,
        9
    );
    assert!(library.root_path.join("exports").join("keep.jpg").is_file());

    remove_library_root(&workspace);
}

#[test]
fn persists_photo_flags_across_library_reopen() {
    let workspace = unique_library_root("flags");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.DNG");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"supported raw candidate").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.DNG'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");

    let initial_flags = get_photo_flags(&library.root_path, &photo_id)
        .expect("read initial flags")
        .expect("default flags row");
    assert_eq!(
        initial_flags,
        silica_catalog::PhotoFlags::new(photo_id.clone(), 0, false, false, None).unwrap()
    );

    let updated_flags = set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        4,
        true,
        false,
        Some(" green ".to_string()),
    )
    .expect("set flags");
    assert_eq!(
        updated_flags,
        silica_catalog::PhotoFlags::new(
            photo_id.clone(),
            4,
            true,
            false,
            Some("green".to_string())
        )
        .unwrap()
    );

    let (rating, picked, rejected, color_label): (i64, i64, i64, String) = connection
        .query_row(
            "SELECT rating, picked, rejected, color_label FROM photo_flags WHERE photo_id = ?1",
            params![photo_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .expect("authoritative photo_flags row");
    assert_eq!(
        (rating, picked, rejected, color_label.as_str()),
        (4, 1, 0, "green")
    );

    drop(connection);
    let reopened = open_local_library(&library_root).expect("reopen library");
    let persisted_flags = get_photo_flags(&reopened.root_path, &updated_flags.photo_id)
        .expect("read persisted flags")
        .expect("persisted flags row");
    assert_eq!(persisted_flags, updated_flags);

    remove_library_root(&workspace);
}

#[test]
fn reads_photo_preview_candidates_from_catalog() {
    let workspace = unique_library_root("preview-candidate");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let unsupported_file = import_root.join("notes.txt");
    let legacy_raw_file = import_root.join("legacy.DNG");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
    std::fs::write(&unsupported_file, b"unsupported side note").expect("write unsupported");
    std::fs::write(&legacy_raw_file, b"legacy raw placeholder").expect("write legacy raw");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let supported_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("supported photo id");
    let unsupported_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'notes.txt'",
            [],
            |row| row.get(0),
        )
        .expect("unsupported photo id");
    let legacy_raw_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'legacy.DNG'",
            [],
            |row| row.get(0),
        )
        .expect("legacy raw photo id");
    connection
        .execute(
            "UPDATE photos SET file_type = 'raw', unsupported = 0 WHERE id = ?1",
            params![legacy_raw_id],
        )
        .expect("simulate legacy raw-supported preview row");

    let supported = get_photo_preview_candidate(&library.root_path, &supported_id)
        .expect("read supported preview candidate")
        .expect("supported preview candidate");
    assert_eq!(supported.photo_id, supported_id);
    assert_eq!(supported.file_name, "sample.jpg");
    assert_eq!(supported.path, supported_file.display().to_string());
    assert!(!supported.unsupported);

    let unsupported = get_photo_preview_candidate(&library.root_path, &unsupported_id)
        .expect("read unsupported preview candidate")
        .expect("unsupported preview candidate");
    assert_eq!(unsupported.file_name, "notes.txt");
    assert!(unsupported.unsupported);

    let legacy_raw = get_photo_preview_candidate(&library.root_path, &legacy_raw_id)
        .expect("read legacy raw preview candidate")
        .expect("legacy raw preview candidate");
    assert_eq!(legacy_raw.file_name, "legacy.DNG");
    assert!(legacy_raw.unsupported);

    assert!(
        get_photo_preview_candidate(&library.root_path, "missing-photo")
            .expect("missing preview candidate lookup")
            .is_none()
    );

    remove_library_root(&workspace);
}

#[test]
fn commits_active_edit_graph_without_draft_write() {
    let workspace = unique_library_root("edit-state");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    assert_eq!(count_edit_states(&connection), 0);
    assert_eq!(count_edit_history(&connection), 0);
    drop(connection);

    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft edit graph")
        .expect("draft edit graph");
    let edited =
        silica_edit::apply_exposure_contrast(&draft, 0.5, -8.0, "unix:3").expect("apply edit");

    let connection = open_catalog(&library.catalog_path).expect("reopen catalog");
    assert_eq!(
        count_edit_states(&connection),
        0,
        "draft slider update must not write edit_states"
    );
    assert_eq!(
        count_edit_history(&connection),
        0,
        "draft slider update must not write edit_history"
    );
    drop(connection);

    commit_edit_graph(&library.root_path, edited).expect("commit edit graph");

    let reopened = open_local_library(&library_root).expect("reopen library");
    let persisted = load_active_edit_graph_or_default(&reopened.root_path, &photo_id)
        .expect("read active edit graph")
        .expect("active edit graph");
    assert_eq!(persisted.basic.exposure.as_f64(), Some(0.5));
    assert_eq!(persisted.basic.contrast.as_f64(), Some(-8.0));

    let connection = open_catalog(&reopened.catalog_path).expect("open reopened catalog");
    assert_eq!(count_edit_states(&connection), 1);
    assert_eq!(count_edit_history(&connection), 1);
    let action_json: String = connection
        .query_row(
            "SELECT action_json FROM edit_history WHERE photo_id = ?1 AND sequence = 1",
            params![photo_id],
            |row| row.get(0),
        )
        .expect("history action json");
    let action: serde_json::Value =
        serde_json::from_str(&action_json).expect("parse history action json");
    assert_eq!(action["schema"], "silica.action");
    assert_eq!(action["version"], 1);
    assert_eq!(action["class"], "undoable");
    assert_eq!(action["kind"], "edit_commit");
    assert_eq!(action["photo_id"], photo_id);

    let before_graph: silica_edit::EditGraph =
        serde_json::from_value(action["before"]["edit_graph"].clone()).expect("before edit graph");
    silica_edit::validate_edit_graph(&before_graph).expect("before graph validates");
    assert_eq!(before_graph.basic.exposure.as_f64(), Some(0.0));
    assert_eq!(before_graph.basic.contrast.as_f64(), Some(0.0));

    let after_graph: silica_edit::EditGraph =
        serde_json::from_value(action["after"]["edit_graph"].clone()).expect("after edit graph");
    silica_edit::validate_edit_graph(&after_graph).expect("after graph validates");
    assert_eq!(after_graph.basic.exposure.as_f64(), Some(0.5));
    assert_eq!(after_graph.basic.contrast.as_f64(), Some(-8.0));
    drop(connection);

    let second_edit = silica_edit::apply_exposure_contrast(&persisted, 1.0, 3.0, "unix:4")
        .expect("apply second edit");
    commit_edit_graph(&reopened.root_path, second_edit).expect("commit second edit graph");

    let connection = open_catalog(&reopened.catalog_path).expect("open catalog after second");
    assert_eq!(count_edit_states(&connection), 2);
    assert_eq!(count_edit_history(&connection), 2);
    let active_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM edit_states WHERE photo_id = ?1 AND active = 1",
            params![photo_id],
            |row| row.get(0),
        )
        .expect("count active states");
    assert_eq!(active_count, 1);
    let sequences: Vec<i64> = {
        let mut statement = connection
            .prepare("SELECT sequence FROM edit_history WHERE photo_id = ?1 ORDER BY sequence")
            .expect("prepare history sequence query");
        statement
            .query_map(params![photo_id], |row| row.get::<_, i64>(0))
            .expect("query history sequences")
            .map(|row| row.expect("history sequence"))
            .collect()
    };
    assert_eq!(sequences, vec![1, 2]);

    remove_library_root(&workspace);
}

#[test]
fn batch_commit_edit_graph_writes_one_history_row_per_photo() {
    let workspace = unique_library_root("batch-edit-state");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let first_file = import_root.join("first.jpg");
    let second_file = import_root.join("second.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&first_file, b"first jpeg placeholder").expect("write first");
    std::fs::write(&second_file, b"second jpeg placeholder").expect("write second");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_ids: Vec<String> = {
        let mut statement = connection
            .prepare("SELECT id FROM photos ORDER BY file_name ASC")
            .expect("prepare photo query");
        statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("query photo ids")
            .map(|row| row.expect("photo id"))
            .collect()
    };
    assert_eq!(photo_ids.len(), 2);
    assert_eq!(count_edit_states(&connection), 0);
    assert_eq!(count_edit_history(&connection), 0);
    drop(connection);

    let sidecar = write_photo_sidecar(&library.root_path, &photo_ids[0], "0.1.0-alpha.1")
        .expect("write sidecar before batch");
    let sidecar_bytes = std::fs::read(&sidecar.sidecar_path).expect("read sidecar bytes");

    let first_draft = load_active_edit_graph_or_default(&library.root_path, &photo_ids[0])
        .expect("load first draft")
        .expect("first draft");
    let second_draft = load_active_edit_graph_or_default(&library.root_path, &photo_ids[1])
        .expect("load second draft")
        .expect("second draft");
    let first_edit = silica_edit::apply_exposure_contrast(&first_draft, 0.75, 12.0, "unix:30")
        .expect("apply first edit");
    let second_edit =
        silica_edit::apply_tone_recovery(&second_draft, -20.0, 15.0, 8.0, -10.0, "unix:31")
            .expect("apply second edit");

    let result = commit_edit_graph_batch(&library.root_path, vec![first_edit, second_edit])
        .expect("batch commit edit graphs");

    assert_eq!(result.commits.len(), 2);
    assert_eq!(result.commits[0].photo_id, photo_ids[0]);
    assert_eq!(result.commits[0].sequence, 1);
    assert_eq!(result.commits[1].photo_id, photo_ids[1]);
    assert_eq!(result.commits[1].sequence, 1);

    let first_persisted = load_active_edit_graph(&library.root_path, &photo_ids[0])
        .expect("load first active")
        .expect("first active");
    let second_persisted = load_active_edit_graph(&library.root_path, &photo_ids[1])
        .expect("load second active")
        .expect("second active");
    assert_eq!(first_persisted.basic.exposure.as_f64(), Some(0.75));
    assert_eq!(second_persisted.basic.highlights.as_f64(), Some(-20.0));

    let connection = open_catalog(&library.catalog_path).expect("open catalog after batch");
    assert_eq!(count_edit_states(&connection), 2);
    assert_eq!(count_edit_history(&connection), 2);
    let sidecar_state: String = connection
        .query_row(
            "SELECT conflict_state FROM sidecar_status WHERE photo_id = ?1",
            params![photo_ids[0]],
            |row| row.get(0),
        )
        .expect("sidecar state");
    assert_eq!(sidecar_state, "catalog_newer");
    assert_eq!(
        std::fs::read(&sidecar.sidecar_path).expect("read sidecar after batch"),
        sidecar_bytes,
        "batch sync must not rewrite sidecar bytes"
    );

    remove_library_root(&workspace);
}

#[test]
fn batch_commit_edit_graph_preflight_failure_writes_no_history() {
    let workspace = unique_library_root("batch-edit-preflight");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft")
        .expect("draft");
    let valid_edit =
        silica_edit::apply_exposure_contrast(&draft, 0.25, 6.0, "unix:30").expect("edit");
    let missing_edit = silica_edit::default_edit_graph(
        silica_edit::EditGraphSource {
            photo_id: "missing-photo".to_string(),
            path: "/tmp/missing.jpg".to_string(),
            file_size: 12,
            modified_at: Some("unix:1".to_string()),
            partial_hash: Some("missing".to_string()),
            full_hash: None,
        },
        "unix:31",
    );

    let error = commit_edit_graph_batch(&library.root_path, vec![valid_edit, missing_edit])
        .expect_err("missing target must fail preflight");
    assert!(error.to_string().contains("missing catalog photo"));

    let connection = open_catalog(&library.catalog_path).expect("open catalog after failure");
    assert_eq!(count_edit_states(&connection), 0);
    assert_eq!(count_edit_history(&connection), 0);

    remove_library_root(&workspace);
}

#[test]
fn records_export_and_marks_photo_exported() {
    let workspace = unique_library_root("export-record");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let output_path = workspace.join("Exports").join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create export directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    let record = record_export(
        &library.root_path,
        &photo_id,
        &output_path,
        r#"{"format":"jpeg","color_profile":"srgb"}"#,
    )
    .expect("record export");

    assert_eq!(record.photo_id, photo_id);
    assert_eq!(record.output_path, output_path.display().to_string());
    assert!(record.export_settings_json.contains("\"jpeg\""));

    let connection = open_catalog(&library.catalog_path).expect("reopen catalog");
    let export_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM exports", [], |row| row.get(0))
        .expect("count exports");
    assert_eq!(export_count, 1);

    let exported: i64 = connection
        .query_row(
            "SELECT exported FROM photo_flags WHERE photo_id = ?1",
            params![record.photo_id],
            |row| row.get(0),
        )
        .expect("exported flag");
    assert_eq!(exported, 1);
    drop(connection);

    let latest = get_latest_export_record(&library.root_path, &record.photo_id)
        .expect("read latest export")
        .expect("latest export row");
    assert_eq!(latest, record);

    remove_library_root(&workspace);
}

#[test]
fn recent_export_records_are_limited_and_ordered() {
    let workspace = unique_library_root("recent-export-records");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let first_file = import_root.join("first.jpg");
    let second_file = import_root.join("second.jpg");
    let first_output = workspace.join("Exports").join("first-export.jpg");
    let second_output = workspace.join("Exports").join("second-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(first_output.parent().expect("output parent"))
        .expect("create export directory");
    std::fs::write(&first_file, b"jpeg placeholder bytes").expect("write first");
    std::fs::write(&second_file, b"jpeg placeholder bytes").expect("write second");
    std::fs::write(&first_output, b"first export bytes").expect("write first output");
    std::fs::write(&second_output, b"second export bytes").expect("write second output");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let first_photo_id = stable_catalog_id("photo", &first_file.display().to_string());
    let second_photo_id = stable_catalog_id("photo", &second_file.display().to_string());

    let first = record_export(
        &library.root_path,
        &first_photo_id,
        &first_output,
        r#"{"format":"jpeg"}"#,
    )
    .expect("record first export");
    let second = record_export(
        &library.root_path,
        &second_photo_id,
        &second_output,
        r#"{"format":"png"}"#,
    )
    .expect("record second export");

    let recent = list_recent_export_records(&library.root_path, 1).expect("list recent exports");

    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].id, second.id);
    assert_eq!(recent[0].photo_id, second.photo_id);
    assert_eq!(recent[0].output_path, second.output_path);
    assert_eq!(recent[0].export_settings_json, second.export_settings_json);
    assert!(!recent[0].created_at.is_empty());
    assert_ne!(recent[0].id, first.id);

    remove_library_root(&workspace);
}

#[test]
fn export_settings_migration_upgrades_existing_catalog() {
    let mut connection = Connection::open_in_memory().expect("open in-memory sqlite");
    configure_connection(&connection).expect("configure sqlite");

    run_migrations_through(&mut connection, 8).expect("run through v8");
    assert!(!catalog_object_exists(&connection, "export_settings")
        .expect("pre-v9 export settings table lookup"));

    run_migrations(&mut connection).expect("upgrade to latest");
    assert_eq!(
        current_schema_version(&connection).expect("version"),
        CURRENT_SCHEMA_VERSION
    );
    assert!(catalog_object_exists(&connection, "export_settings")
        .expect("export settings table lookup"));
    assert!(
        catalog_object_exists(&connection, "export_presets").expect("export presets table lookup")
    );

    let (preset_id, format, color_profile, quality, metadata_policy): (
        String,
        String,
        String,
        u8,
        String,
    ) = connection
        .query_row(
            r#"
            SELECT preset_id, format, color_profile, quality, metadata_policy
            FROM export_settings
            WHERE id = 'default'
            "#,
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .expect("default export settings");
    assert_eq!(preset_id, DEFAULT_EXPORT_PRESET_ID);
    assert_eq!(format, "jpeg");
    assert_eq!(color_profile, "srgb");
    assert_eq!(quality, 90);
    assert_eq!(metadata_policy, "minimal");
}

#[test]
fn export_settings_accept_png_and_tiff_after_current_migration() {
    let workspace = unique_library_root("export-settings-formats");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let png_settings = ExportSettings {
        format: "png".to_string(),
        color_profile: "srgb".to_string(),
        quality: 90,
        metadata_policy: "minimal".to_string(),
    };
    let png_catalog = set_default_export_settings(&library.root_path, None, png_settings.clone())
        .expect("save png default settings");
    assert_eq!(png_catalog.default_settings, png_settings);

    let tiff_settings = ExportSettings {
        format: "tiff".to_string(),
        color_profile: "srgb".to_string(),
        quality: 90,
        metadata_policy: "minimal".to_string(),
    };
    let preset = upsert_export_preset(&library.root_path, "TIFF sRGB", tiff_settings.clone())
        .expect("save tiff preset");
    let reloaded =
        set_default_export_settings(&library.root_path, Some(&preset.id), tiff_settings.clone())
            .expect("save tiff default settings");
    assert_eq!(reloaded.default_settings, tiff_settings);
    assert!(reloaded
        .presets
        .iter()
        .any(|candidate| candidate == &preset));

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    assert_eq!(count_edit_states(&connection), 0);
    assert_eq!(count_edit_history(&connection), 0);

    remove_library_root(&workspace);
}

#[test]
fn export_settings_accept_metadata_policy_values_after_current_migration() {
    let workspace = unique_library_root("export-settings-metadata-policy");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    for policy in ["preserve", "remove_gps", "remove_all"] {
        let settings = ExportSettings {
            metadata_policy: policy.to_string(),
            ..ExportSettings::jpeg_srgb_default()
        };
        let catalog = set_default_export_settings(&library.root_path, None, settings.clone())
            .expect("save metadata policy settings");
        assert_eq!(catalog.default_settings, settings);
    }

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    assert_eq!(count_edit_states(&connection), 0);
    assert_eq!(count_edit_history(&connection), 0);

    remove_library_root(&workspace);
}

#[test]
fn persists_export_settings_presets_without_edit_history() {
    let workspace = unique_library_root("export-settings");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let initial_catalog =
        get_export_settings_catalog(&library.root_path).expect("read export settings");
    assert_eq!(
        initial_catalog.default_settings,
        ExportSettings::jpeg_srgb_default()
    );
    assert_eq!(
        initial_catalog.default_preset_id.as_deref(),
        Some(DEFAULT_EXPORT_PRESET_ID)
    );
    assert!(initial_catalog
        .presets
        .iter()
        .any(|preset| preset.id == DEFAULT_EXPORT_PRESET_ID));

    let display_p3_settings = ExportSettings {
        color_profile: "display_p3".to_string(),
        ..ExportSettings::jpeg_srgb_default()
    };
    let preset = upsert_export_preset(
        &library.root_path,
        "Display P3 Review",
        display_p3_settings.clone(),
    )
    .expect("upsert export preset");
    let updated_catalog = set_default_export_settings(
        &library.root_path,
        Some(&preset.id),
        display_p3_settings.clone(),
    )
    .expect("set default export settings");
    assert_eq!(
        updated_catalog.default_preset_id.as_deref(),
        Some(preset.id.as_str())
    );
    assert_eq!(updated_catalog.default_settings, display_p3_settings);

    let reloaded_catalog =
        get_export_settings_catalog(&library.root_path).expect("reload export settings");
    assert_eq!(
        reloaded_catalog.default_preset_id,
        updated_catalog.default_preset_id
    );
    assert_eq!(
        reloaded_catalog.default_settings,
        updated_catalog.default_settings
    );
    assert!(reloaded_catalog
        .presets
        .iter()
        .any(|candidate| candidate == &preset));

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    assert_eq!(count_edit_states(&connection), 0);
    assert_eq!(count_edit_history(&connection), 0);

    remove_library_root(&workspace);
}

#[test]
fn undo_redo_history_restores_edit_state_and_preserves_exports() {
    let workspace = unique_library_root("undo-redo-edit");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");
    let output_path = workspace.join("Exports").join("sample-export.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::create_dir_all(output_path.parent().expect("output parent"))
        .expect("create export directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");
    std::fs::write(&output_path, b"export bytes").expect("write export output");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft")
        .expect("draft graph");
    let edited =
        silica_edit::apply_exposure_contrast(&draft, 0.5, -8.0, "unix:3").expect("apply edit");
    commit_edit_graph(&library.root_path, edited).expect("commit edit");
    record_export(
        &library.root_path,
        &photo_id,
        &output_path,
        r#"{"format":"jpeg","color_profile":"srgb"}"#,
    )
    .expect("record export");

    let undo = undo_last_history_action(&library.root_path, &photo_id).expect("undo edit");
    assert!(undo.applied);
    assert_eq!(undo.action_kind.as_deref(), Some("edit_commit"));
    assert!(output_path.exists(), "undo must not delete export output");
    let undone = load_active_edit_graph(&library.root_path, &photo_id)
        .expect("load undone edit")
        .expect("undone edit graph");
    assert_eq!(undone.basic.exposure.as_f64(), Some(0.0));
    assert_eq!(undone.basic.contrast.as_f64(), Some(0.0));

    let redo = redo_last_history_action(&library.root_path, &photo_id).expect("redo edit");
    assert!(redo.applied);
    let redone = load_active_edit_graph(&library.root_path, &photo_id)
        .expect("load redone edit")
        .expect("redone edit graph");
    assert_eq!(redone.basic.exposure.as_f64(), Some(0.5));
    assert_eq!(redone.basic.contrast.as_f64(), Some(-8.0));
    assert!(output_path.exists(), "redo must not delete export output");

    remove_library_root(&workspace);
}

#[test]
fn undo_redo_history_restores_photo_flags() {
    let workspace = unique_library_root("undo-redo-flags");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        4,
        true,
        false,
        Some("blue".into()),
    )
    .expect("set flags");

    let undo = undo_last_history_action(&library.root_path, &photo_id).expect("undo flags");
    assert!(undo.applied);
    assert_eq!(undo.action_kind.as_deref(), Some("flag_change"));
    let undone = get_photo_flags(&library.root_path, &photo_id)
        .expect("read undone flags")
        .expect("flags row");
    assert_eq!(undone.rating, 0);
    assert!(!undone.picked);
    assert!(!undone.rejected);
    assert_eq!(undone.color_label, None);

    let redo = redo_last_history_action(&library.root_path, &photo_id).expect("redo flags");
    assert!(redo.applied);
    let redone = get_photo_flags(&library.root_path, &photo_id)
        .expect("read redone flags")
        .expect("flags row");
    assert_eq!(redone.rating, 4);
    assert!(redone.picked);
    assert!(!redone.rejected);
    assert_eq!(redone.color_label.as_deref(), Some("blue"));

    remove_library_root(&workspace);
}

#[test]
fn new_undoable_action_invalidates_redo_history() {
    let workspace = unique_library_root("redo-invalidated");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());
    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft")
        .expect("draft graph");
    let first =
        silica_edit::apply_exposure_contrast(&draft, 0.5, -8.0, "unix:3").expect("first edit");
    commit_edit_graph(&library.root_path, first).expect("commit first edit");
    undo_last_history_action(&library.root_path, &photo_id).expect("undo first edit");

    let current = load_active_edit_graph(&library.root_path, &photo_id)
        .expect("load current graph")
        .expect("current graph");
    let second =
        silica_edit::apply_exposure_contrast(&current, 1.0, 3.0, "unix:4").expect("second edit");
    commit_edit_graph(&library.root_path, second).expect("commit second edit");

    let redo = redo_last_history_action(&library.root_path, &photo_id).expect("redo command");
    assert!(!redo.applied);
    assert_eq!(redo.action_kind, None);

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let invalidated_count: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM edit_history WHERE photo_id = ?1 AND history_state = 'invalidated'",
            params![photo_id],
            |row| row.get(0),
        )
        .expect("count invalidated history");
    assert_eq!(invalidated_count, 1);

    remove_library_root(&workspace);
}

#[test]
fn lists_real_history_checkpoints_with_command_state() {
    let workspace = unique_library_root("history-panel");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");
    let photo_id = stable_catalog_id("photo", &supported_file.display().to_string());

    let empty = list_photo_history(&library.root_path, &photo_id).expect("read empty history");
    assert_eq!(empty.photo_id, photo_id);
    assert_eq!(empty.status, "empty");
    assert_eq!(empty.message, "No committed history yet.");
    assert!(!empty.can_undo);
    assert!(!empty.can_redo);
    assert!(empty.items.is_empty());

    let draft = load_active_edit_graph_or_default(&library.root_path, &photo_id)
        .expect("load draft edit graph")
        .expect("draft edit graph");
    let edited =
        silica_edit::apply_exposure_contrast(&draft, 0.5, -8.0, "unix:3").expect("apply edit");
    commit_edit_graph(&library.root_path, edited).expect("commit edit");
    set_photo_flags(
        &library.root_path,
        photo_id.clone(),
        4,
        true,
        false,
        Some("blue".to_string()),
    )
    .expect("set flags");
    undo_last_history_action(&library.root_path, &photo_id).expect("undo latest flags");

    let history = list_photo_history(&library.root_path, &photo_id).expect("read history");
    assert_eq!(history.status, "ready");
    assert_eq!(history.message, "History checkpoints loaded.");
    assert!(history.can_undo);
    assert!(history.can_redo);
    assert_eq!(history.items.len(), 2);
    assert_eq!(history.items[0].sequence, 2);
    assert_eq!(history.items[0].action_kind, "flag_change");
    assert_eq!(history.items[0].label, "Culling flags");
    assert_eq!(history.items[0].history_state, "undone");
    assert!(history.items[0].can_redo);
    assert!(!history.items[0].can_undo);
    assert_eq!(history.items[1].sequence, 1);
    assert_eq!(history.items[1].action_kind, "edit_commit");
    assert_eq!(history.items[1].label, "Exposure / contrast");
    assert_eq!(history.items[1].history_state, "applied");
    assert!(history.items[1].can_undo);
    assert!(!history.items[1].can_redo);

    remove_library_root(&workspace);
}

#[test]
fn appends_action_log_entries_without_replacing_prior_rows() {
    let workspace = unique_library_root("action-log");
    let library_root = workspace.join("SilicaRAW Library");
    let library = create_local_library(&library_root).expect("create library");

    let first = append_action_log_entry(
        &library.root_path,
        NewActionLogEntry {
            actor_type: "core".to_string(),
            actor_id: Some("local-alpha".to_string()),
            action_type: "export".to_string(),
            subject_type: Some("photo".to_string()),
            subject_id: Some("photo-1".to_string()),
            side_effect_category: "file_write".to_string(),
            evidence_ref: Some("export-1".to_string()),
            payload_json: "{\"ok\":true}".to_string(),
        },
    )
    .expect("append first action");
    let second = append_action_log_entry(
        &library.root_path,
        NewActionLogEntry {
            actor_type: "core".to_string(),
            actor_id: Some("local-alpha".to_string()),
            action_type: "cache_clear".to_string(),
            subject_type: Some("library".to_string()),
            subject_id: Some(library.root_path.display().to_string()),
            side_effect_category: "cache_delete".to_string(),
            evidence_ref: Some("cache-clear".to_string()),
            payload_json: "{\"removedCacheRecords\":0}".to_string(),
        },
    )
    .expect("append second action");

    assert_ne!(first.id, second.id);
    let entries = list_action_log_entries(&library.root_path, 20).expect("list action log");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].action_type, "cache_clear");
    assert_eq!(entries[0].side_effect_category, "cache_delete");
    assert_eq!(entries[0].evidence_ref.as_deref(), Some("cache-clear"));
    assert_eq!(entries[1].action_type, "export");
    assert_eq!(entries[1].side_effect_category, "file_write");
    assert_eq!(entries[1].evidence_ref.as_deref(), Some("export-1"));

    remove_library_root(&workspace);
}

#[test]
fn action_log_rejects_original_mutation_claims() {
    let workspace = unique_library_root("action-log-original-mutation");
    let library_root = workspace.join("SilicaRAW Library");
    let library = create_local_library(&library_root).expect("create library");

    let error = append_action_log_entry(
        &library.root_path,
        NewActionLogEntry {
            actor_type: "core".to_string(),
            actor_id: Some("local-alpha".to_string()),
            action_type: "original_mutation".to_string(),
            subject_type: Some("original".to_string()),
            subject_id: Some("/tmp/original.jpg".to_string()),
            side_effect_category: "original_mutation".to_string(),
            evidence_ref: None,
            payload_json: "{}".to_string(),
        },
    )
    .expect_err("original mutation action log must be blocked");
    assert!(error.to_string().contains("original mutation"));

    remove_library_root(&workspace);
}

#[test]
fn action_log_rejects_extension_raw_sql_bypass_claims() {
    let workspace = unique_library_root("action-log-extension-bypass");
    let library_root = workspace.join("SilicaRAW Library");
    let library = create_local_library(&library_root).expect("create library");

    let error = append_action_log_entry(
        &library.root_path,
        NewActionLogEntry {
            actor_type: "plugin".to_string(),
            actor_id: Some("preset-pack".to_string()),
            action_type: "Raw_SQL".to_string(),
            subject_type: Some("catalog".to_string()),
            subject_id: Some("catalog.db".to_string()),
            side_effect_category: "Direct_Database_Access".to_string(),
            evidence_ref: None,
            payload_json: "{}".to_string(),
        },
    )
    .expect_err("extension raw SQL action log must be blocked");
    assert!(error.to_string().contains("extension database bypass"));

    remove_library_root(&workspace);
}

#[test]
fn ai_results_store_unapproved_local_permissioned_results_without_edit_or_flag_mutation() {
    let workspace = unique_library_root("ai-result-store");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    let flags_before: i64 = connection
        .query_row("SELECT COUNT(*) FROM photo_flags", [], |row| row.get(0))
        .expect("count flags");
    assert_eq!(count_edit_states(&connection), 0);
    drop(connection);

    let result = append_ai_result(
        &library.root_path,
        NewAiResult {
            photo_id: photo_id.clone(),
            task_type: "blur_score".to_string(),
            model_id: "silicaraw.blur-review.test".to_string(),
            permission_id: "ai_result:propose".to_string(),
            output_json: r#"{"review":{"score":0.25,"label":"usable"}}"#.to_string(),
        },
    )
    .expect("append ai result");

    assert_eq!(result.photo_id, photo_id);
    assert_eq!(result.task_type, "blur_score");
    assert_eq!(result.model_id, "silicaraw.blur-review.test");
    assert!(!result.approved);

    let payload: serde_json::Value =
        serde_json::from_str(&result.result_json).expect("parse ai result payload");
    assert_eq!(payload["schema"], "silica.ai_result");
    assert_eq!(payload["permission_id"], "ai_result:propose");
    assert_eq!(payload["local_only"], true);

    let listed =
        list_ai_results_for_photo(&library.root_path, &result.photo_id, 10).expect("list ai");
    assert_eq!(listed, vec![result]);

    let connection = open_catalog(&library.catalog_path).expect("reopen catalog");
    assert_eq!(count_edit_states(&connection), 0);
    let flags_after: i64 = connection
        .query_row("SELECT COUNT(*) FROM photo_flags", [], |row| row.get(0))
        .expect("count flags after");
    assert_eq!(flags_after, flags_before);

    remove_library_root(&workspace);
}

#[test]
fn ai_results_reject_direct_edit_graph_or_flag_payloads() {
    let workspace = unique_library_root("ai-result-reject-mutation");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    drop(connection);

    for output_json in [
        r#"{"edit_graph":{"basic":{"exposure":1.0}}}"#,
        r#"{"photo_flags":{"rating":5}}"#,
    ] {
        let error = append_ai_result(
            &library.root_path,
            NewAiResult {
                photo_id: photo_id.clone(),
                task_type: "blur_score".to_string(),
                model_id: "silicaraw.blur-review.test".to_string(),
                permission_id: "ai_result:propose".to_string(),
                output_json: output_json.to_string(),
            },
        )
        .expect_err("direct mutation payload rejected");
        assert!(error.to_string().contains("direct edit mutation"));
    }

    remove_library_root(&workspace);
}

#[test]
fn ai_result_approval_marks_existing_result_without_edit_history() {
    let workspace = unique_library_root("ai-result-approve");
    let library_root = workspace.join("SilicaRAW Library");
    let import_root = workspace.join("Originals");
    let supported_file = import_root.join("sample.jpg");

    std::fs::create_dir_all(&import_root).expect("create import directory");
    std::fs::write(&supported_file, b"jpeg placeholder bytes").expect("write supported");

    let library = create_local_library(&library_root).expect("create library");
    import_folder(&library.root_path, &import_root).expect("import folder");

    let connection = open_catalog(&library.catalog_path).expect("open catalog");
    let photo_id: String = connection
        .query_row(
            "SELECT id FROM photos WHERE file_name = 'sample.jpg'",
            [],
            |row| row.get(0),
        )
        .expect("photo id");
    let before_history = count_edit_history(&connection);
    drop(connection);

    let result = append_ai_result(
        &library.root_path,
        NewAiResult {
            photo_id: photo_id.clone(),
            task_type: "blur_score".to_string(),
            model_id: "silicaraw.blur-review.test".to_string(),
            permission_id: "ai_result:propose".to_string(),
            output_json: r#"{"review":{"label":"Usable detail"},"approval_suggestion":{"kind":"basic_exposure_contrast","exposure":0.25,"contrast":8.0}}"#.to_string(),
        },
    )
    .expect("append ai result");
    assert!(!result.approved);

    let approved = approve_ai_result(&library.root_path, &result.id).expect("approve result");

    assert_eq!(approved.id, result.id);
    assert!(approved.approved);
    let listed =
        list_ai_results_for_photo(&library.root_path, &photo_id, 10).expect("list approved result");
    assert!(listed[0].approved);
    let connection = open_catalog(&library.catalog_path).expect("reopen catalog");
    assert_eq!(count_edit_history(&connection), before_history);

    remove_library_root(&workspace);
}

fn unique_catalog_path(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "silicaraw-storage-{label}-{}-{nanos}.db",
        std::process::id()
    ))
}

fn unique_library_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "silicaraw-library-{label}-{}-{nanos}",
        std::process::id()
    ))
}

fn remove_catalog_files(path: &Path) {
    for suffix in ["", "-wal", "-shm"] {
        let _ = std::fs::remove_file(format!("{}{}", path.display(), suffix));
    }
}

fn remove_library_root(path: &Path) {
    let _ = std::fs::remove_dir_all(path);
}

fn assert_issue_kind(issues: &[ImportIssue], kind: ImportIssueKind, file_name: &str) {
    assert!(
        issues
            .iter()
            .any(|issue| issue.kind == kind && issue.file_name.as_deref() == Some(file_name)),
        "missing {kind:?} for {file_name}: {issues:?}"
    );
}

fn count_edit_states(connection: &Connection) -> i64 {
    connection
        .query_row("SELECT COUNT(*) FROM edit_states", [], |row| row.get(0))
        .expect("count edit states")
}

fn count_edit_history(connection: &Connection) -> i64 {
    connection
        .query_row("SELECT COUNT(*) FROM edit_history", [], |row| row.get(0))
        .expect("count edit history")
}
