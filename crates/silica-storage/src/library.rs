use std::fs;
use std::path::Path;
use std::time::Duration;

use rusqlite::{params, Connection, OpenFlags, OptionalExtension};

use super::common::path_to_string;
use super::migrations::{configure_connection, current_schema_version, run_migrations};
use super::{
    LibraryStorageError, LocalLibrary, CATALOG_DATABASE_FILE, CURRENT_SCHEMA_VERSION,
    LOCAL_LIBRARY_ID, REQUIRED_LIBRARY_DIRECTORIES,
};

/// Open a catalog database and apply all embedded migrations.
pub fn open_catalog(path: impl AsRef<Path>) -> rusqlite::Result<Connection> {
    let mut connection = Connection::open(path)?;
    configure_connection(&connection)?;
    run_migrations(&mut connection)?;
    Ok(connection)
}

/// Create a local SilicaRAW library folder and initialize its catalog.
pub fn create_local_library(
    root_path: impl AsRef<Path>,
) -> Result<LocalLibrary, LibraryStorageError> {
    let root_path = root_path.as_ref();
    fs::create_dir_all(root_path)?;
    ensure_library_directories(root_path)?;

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    open_library_catalog(root_path, &catalog_path)
}

/// Open an existing local SilicaRAW library folder and migrate its catalog.
pub fn open_local_library(
    root_path: impl AsRef<Path>,
) -> Result<LocalLibrary, LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    ensure_library_directories(root_path)?;
    open_library_catalog(root_path, &catalog_path)
}

/// Inspect an existing local library for relaunch restore without migrations or repair.
pub fn inspect_local_library_for_restore(
    root_path: impl AsRef<Path>,
) -> Result<LocalLibrary, LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    let connection = Connection::open_with_flags(&catalog_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let schema_version = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    Ok(LocalLibrary {
        root_path: root_path.to_path_buf(),
        catalog_path,
        schema_version,
    })
}

/// Check catalog photo existence for relaunch restore without migrations or repair.
pub fn catalog_photo_exists_for_restore(
    root_path: impl AsRef<Path>,
    photo_id: &str,
) -> Result<bool, LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    let connection = Connection::open_with_flags(&catalog_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    connection
        .query_row(
            "SELECT 1 FROM photos WHERE id = ?1 LIMIT 1",
            [photo_id],
            |_| Ok(()),
        )
        .optional()
        .map(|result| result.is_some())
        .map_err(LibraryStorageError::from)
}

pub(super) fn ensure_library_directories(root_path: &Path) -> Result<(), LibraryStorageError> {
    for directory in REQUIRED_LIBRARY_DIRECTORIES {
        fs::create_dir_all(root_path.join(directory))?;
    }
    Ok(())
}

fn open_library_catalog(
    root_path: &Path,
    catalog_path: &Path,
) -> Result<LocalLibrary, LibraryStorageError> {
    let connection = open_catalog(catalog_path)?;
    upsert_local_library_row(&connection, root_path)?;
    let schema_version = current_schema_version(&connection)?;

    Ok(LocalLibrary {
        root_path: root_path.to_path_buf(),
        catalog_path: catalog_path.to_path_buf(),
        schema_version,
    })
}

pub(super) fn open_existing_library_for_read(
    root_path: impl AsRef<Path>,
) -> Result<LocalLibrary, LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    let connection = open_catalog(&catalog_path)?;
    let schema_version = current_schema_version(&connection)?;

    Ok(LocalLibrary {
        root_path: root_path.to_path_buf(),
        catalog_path,
        schema_version,
    })
}

pub(super) fn open_existing_library_for_read_only_query(
    root_path: impl AsRef<Path>,
) -> Result<(LocalLibrary, Connection), LibraryStorageError> {
    let root_path = root_path.as_ref();
    if !root_path.is_dir() {
        return Err(LibraryStorageError::NotDirectory(root_path.to_path_buf()));
    }

    let catalog_path = root_path.join(CATALOG_DATABASE_FILE);
    if !catalog_path.is_file() {
        return Err(LibraryStorageError::MissingCatalog(catalog_path));
    }

    let connection = open_catalog_for_read_only_query(&catalog_path)?;
    let schema_version = current_schema_version(&connection)?;

    if schema_version > CURRENT_SCHEMA_VERSION {
        return Err(LibraryStorageError::CatalogSchemaVersion {
            expected: CURRENT_SCHEMA_VERSION,
            found: schema_version,
        });
    }

    let connection = if schema_version < CURRENT_SCHEMA_VERSION {
        drop(connection);
        let migrated = open_catalog(&catalog_path)?;
        let migrated_schema_version = current_schema_version(&migrated)?;
        drop(migrated);

        if migrated_schema_version != CURRENT_SCHEMA_VERSION {
            return Err(LibraryStorageError::CatalogSchemaVersion {
                expected: CURRENT_SCHEMA_VERSION,
                found: migrated_schema_version,
            });
        }

        open_catalog_for_read_only_query(&catalog_path)?
    } else {
        connection
    };

    Ok((
        LocalLibrary {
            root_path: root_path.to_path_buf(),
            catalog_path,
            schema_version: CURRENT_SCHEMA_VERSION,
        },
        connection,
    ))
}

fn open_catalog_for_read_only_query(catalog_path: &Path) -> rusqlite::Result<Connection> {
    let connection = Connection::open_with_flags(catalog_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(connection)
}

fn upsert_local_library_row(
    connection: &Connection,
    root_path: &Path,
) -> Result<(), LibraryStorageError> {
    let root_path = path_to_string(root_path)?;

    connection.execute(
        r#"
        INSERT INTO libraries(id, root_path)
        VALUES (?1, ?2)
        ON CONFLICT(id) DO UPDATE SET
          root_path = excluded.root_path,
          updated_at = CURRENT_TIMESTAMP
        "#,
        params![LOCAL_LIBRARY_ID, root_path],
    )?;

    Ok(())
}
