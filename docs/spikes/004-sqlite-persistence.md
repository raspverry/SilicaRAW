# Spike 004: SQLite Catalog Persistence

Status: completed  
Date: 2026-06-08  
Result: Path A - rusqlite with bundled SQLite and embedded SQL migrations

## Question

Which SQLite binding and migration approach should SilicaRAW use first, and can an empty catalog database be created and upgraded with the required indexes?

## Result

Path A:

```txt
Use rusqlite with bundled SQLite.
Use embedded SQL migrations in silica-storage.
Create schema_migrations before migration execution.
Use migration 1 for initial catalog tables.
Use migration 2 for required indexes from docs/10.
Configure writable local catalog connections with foreign_keys=ON, journal_mode=WAL, synchronous=NORMAL, and a busy timeout.
```

This is not a broad catalog implementation. It does not create/open library folders, scan photo folders, import assets, write sidecars, manage caches, or expose storage commands to the desktop app.

## External Evidence

- `rusqlite 0.40.1` is an MIT-licensed SQLite wrapper and supports bundled SQLite through `libsqlite3-sys`.
- `libsqlite3-sys 0.38.1` is MIT-licensed and provides the native SQLite binding used by rusqlite.
- SQLite WAL mode is the official write-ahead logging mode for writable databases.
- SQLite foreign key enforcement is connection-local and must be enabled.
- SQLite supports `CREATE INDEX IF NOT EXISTS`, which is useful for idempotent migration SQL.

## Implementation

- Added `rusqlite = 0.40.1` to `crates/silica-storage` with `default-features = false` and `features = ["bundled"]`.
- Added a minimal migration runner in `crates/silica-storage`.
- Added `schema_migrations(version, name, applied_at)`.
- Added migration 1: initial catalog tables.
- Added migration 2: required indexes from `docs/10_Data_Model_and_Storage_Specification.md`.
- Added tests for fresh empty catalog creation, upgrade from migration 1 to latest, required index existence, foreign key enforcement, file-backed WAL configuration, and recorded spike metadata.

## Required Indexes

Spike 004 creates the required indexes listed in `docs/10_Data_Model_and_Storage_Specification.md`, including:

```txt
folders, photos, photo_flags, collections, collection_photos,
edit_states, edit_history, cache_records, ai_results, exports, action_log
```

The Rust tests check every required index by name.

## Migration Approach

Embedded SQL migrations were selected over adding a migration framework dependency.

Reasons:

- The first schema is small and local to `silica-storage`.
- The repository already has a dependency guard, so avoiding an extra migration crate keeps Phase 3 narrow.
- Tests can verify empty DB creation and upgrade behavior without a runtime migration registry.

Future migrations can still move to a dedicated migration crate if migration count, tooling needs, or release rollback requirements justify it.

## Validation

Commands:

```sh
cargo fmt --all --check
git diff --check
cargo test -p silica-storage
cargo clippy -p silica-storage -- -D warnings
scripts/harness/check.sh
```

Expected storage-specific result:

```txt
6 silica-storage tests pass.
Required indexes exist after migration 2.
Foreign key enforcement rejects invalid photo rows.
File-backed catalog connections report journal_mode=wal and foreign_keys=1.
```

## Follow-Up

Before product catalog work:

- Add file-backed library create/open APIs.
- Add backup/checkpoint policy for WAL-mode catalogs.
- Add original-file fingerprint and hash tests.
- Add sidecar read/write and conflict tests.
- Add cache clear safety tests.
- Keep plugin/MCP paths behind typed Core APIs, never direct DB access.

## Sources

- rusqlite docs: https://docs.rs/rusqlite/
- rusqlite repository: https://github.com/rusqlite/rusqlite
- SQLite WAL documentation: https://www.sqlite.org/wal.html
- SQLite PRAGMA documentation: https://www.sqlite.org/pragma.html
- SQLite foreign key documentation: https://www.sqlite.org/foreignkeys.html
- SQLite CREATE INDEX documentation: https://www.sqlite.org/lang_createindex.html

## Guardrails

Do not use this spike as catalog import or library UI behavior. It only proves binding selection, migration shape, initial schema creation, required index creation, and basic relational safety.
