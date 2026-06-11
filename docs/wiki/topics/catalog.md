---
title: Catalog
status: active
audience: all
updated: 2026-06-11
source_of_truth: docs/10_Data_Model_and_Storage_Specification.md
---

# Catalog

## Summary

The catalog is the local SQLite-backed record of libraries, folders, photos, metadata, flags, edit states, sidecar status, caches, exports, and action history.

## Current Stance

- `silica-catalog` owns the domain-facing local alpha schema contract.
- `silica-storage` owns SQLite connection configuration, embedded migrations, and migration verification.
- Phase 4.1 aligns `silica-storage` with the `silica-catalog` contract for schema version, required tables, required indexes, and the migration bookkeeping table.
- Phase 4.2 adds local library folder create/open through `silica-core`, `silica-storage`, and the minimal Tauri shell.
- Phase 4.3 adds non-recursive folder import scanning through `silica-catalog` and `silica-storage`.
- Phase 4.4 adds catalog-authoritative rating, picked, rejected, and color label persistence through `photo_flags`.
- Phase 5.3 adds active edit graph persistence through `edit_states` on exposure/contrast commit.
- Task 10.3 adds explicit library-local sidecar write/read behavior through `silica-storage`; `photo_flags` remains the live in-app authority until a later explicit sync task changes that policy.
- Task 10.4 adds a catalog rebuild dry-run report from library-local sidecars. It reports what would happen and does not mutate `photos`, `photo_flags`, `edit_states`, or `sidecar_status`.
- Task 10.5.1 records backup/WAL/checkpoint/restore policy before backup or restore code is added.
- Task 10.5.2 adds checkpointed backup artifacts under `backups/` containing `catalog.db`, `sidecars/`, and a manifest only.
- Task 10.5.3 adds staged restore from backup artifacts with rollback copies for existing target libraries.
- Task 11.5.2 adds catalog schema version 3 for paged-query support: normalized `photos.file_type` values and query indexes for accepted sort/filter fields.
- The catalog remains local-first and referenced-folder by default.
- Original photo files must not be modified by catalog work.

## Implemented Foundation

- Current alpha schema version: `3`.
- Migration table: `schema_migrations`.
- Migration 1 creates the initial catalog tables.
- Migration 2 creates the required initial indexes from the storage specification.
- Migration 3 adds `photos.file_type`, backfills `jpeg`, `raw`, and `unsupported`, and creates the accepted paged-query indexes.
- Tests cover empty catalog creation, migration upgrade from version 1 to latest, required table/index existence, foreign key enforcement, and file-backed WAL/foreign key configuration.
- Library create/open creates the selected library folder, `catalog.db`, and required support directories.
- Reopening a library migrates the same `catalog.db` and returns the active root/catalog/schema status.
- Tests cover sibling original-directory preservation during create/open.
- Folder import records immediate child files by reference in `folders` and `photos`.
- Import records include path, file size, modified time, partial hash, and unsupported state.
- Tests cover mixed supported/unsupported fixture files and confirm originals are not copied or mutated.
- Imported photos receive default `photo_flags` rows.
- Rating, picked, rejected, and color label updates are stored in SQLite `photo_flags`.
- Restart tests cover flag persistence after reopening the local library.
- Exposure/contrast draft preview updates do not write SQLite.
- Exposure/contrast commit/release writes the active edit graph to `edit_states`.
- Sidecar write/read validates sidecar and nested edit graph JSON, mirrors rating/picked/rejected/color-label state only, writes under library `sidecars/`, and updates `sidecar_status` after successful writes.
- Sidecar rebuild dry-run scans `sidecars/` in deterministic order, resolves portable flags by `sidecar.flags`, then `edit_graph.metadata`, then defaults, and reports malformed sidecars, schema issues, photo-id mismatches, flag/metadata disagreements, and catalog reconciliation conflicts without applying changes.
- Backup creation checkpoints WAL state before copying `catalog.db`, copies `sidecars/`, writes `backup-manifest.json`, and excludes originals, disposable caches, export outputs, logs, and nested backups.
- Restore copies only `catalog.db` and `sidecars/` from validated backup artifacts, verifies through normal open/migration flow, and creates rollback copies before replacing existing target state.
- Task 11.5.1 defines the typed paged query contract in `silica-catalog`: bounded offset pagination, whitelisted sort/filter enums, deterministic tie breakers, and no UI-provided SQL, column names, or raw predicates.
- Task 11.5.2 represents the paged-query indexes in the catalog schema contract and applies them through embedded storage migration 3.
- Task 11.5.3 implements read-only storage/core paged library query APIs without thumbnail hydration or catalog/cache mutation.

## Paged Library Query Contract

Task 11.5 starts with a contract before storage implementation:

- Pagination is offset-based with `DEFAULT_LIBRARY_QUERY_LIMIT = 100` and `MAX_LIBRARY_QUERY_LIMIT = 500`.
- Cursor pagination is explicitly deferred until benchmark evidence requires it.
- Accepted sort modes are `imported_at_desc`, `file_name_asc`, and `rating_desc`.
- Deterministic ordering includes explicit tie breakers:
  - imported-at descending, then photo id ascending
  - file name ascending, then path ascending, then photo id ascending
  - rating descending, then photo id ascending
- Accepted filters are minimum rating, picked, rejected, file type, and search text.
- File type is a whitelisted enum: JPEG, RAW, or unsupported. Storage records this as normalized `photos.file_type` values `jpeg`, `raw`, and `unsupported`; in the local alpha, `raw` means supported non-JPEG photo candidates until later metadata extraction expands the taxonomy.
- Required query indexes are:
  - `idx_photos_library_imported_id`
  - `idx_photos_library_file_name_path_id`
  - `idx_photos_library_file_type_id`
  - `idx_photo_flags_rating_photo_id`
- Storage must translate the typed contract internally; UI code must not pass SQL strings, column names, or raw predicates.
- `silica-storage::query_library_photos` opens the catalog read-only, returns page metadata, and leaves compatibility full-list behavior in `list_library_photos` until the desktop grid migration is complete.

## Not Implemented Yet

- Recursive folder scanning.
- Camera metadata extraction.
- Thumbnail or preview generation during import.
- Original full-hash protection behavior.
- Automatic sidecar synchronization.
- Applied catalog rebuild or restore from sidecars.
- Sidecar conflict handling and conflict UI.
- Full edit history and undo/redo persistence.
- Cache clear safety.
- Paged query desktop wiring.
- Broad catalog UI screens beyond the local alpha workflow.
- Plugin or MCP catalog access.

## Links

- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Data Safety](data-safety.md)
- [Backup and Restore](backup-restore.md)
- [`silica-catalog` README](../../../crates/silica-catalog/README.md)
- [`silica-storage` README](../../../crates/silica-storage/README.md)

## Notes for LLM Agents

Do not treat folder import, flag persistence, active edit graph commits, or explicit sidecar writes as RAW decoding, thumbnail generation, automatic sidecar sync, cache behavior, or library grid behavior. `photo_flags` is the live in-app authority until sidecar synchronization is implemented by an explicit later task.
