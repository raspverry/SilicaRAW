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
- Task 16.0 records the [Action Trust](action-trust.md) taxonomy before undo/history runtime work: edit commits and flag changes are undoable catalog transactions, while export, cache clear, sidecar write, and import-by-reference behavior stay outside Develop undo.
- Task 16.1 records the undo/history action semantics contract: `silica.action` payload version 1, per-photo checkpoint units, redo invalidation after new undoable changes, and explicit disabled undo/redo states.
- The catalog remains local-first and referenced-folder by default.
- Original photo files must not be modified by catalog work.

## Implemented Foundation

- Current alpha schema version: `5`.
- Migration table: `schema_migrations`.
- Migration 1 creates the initial catalog tables.
- Migration 2 creates the required initial indexes from the storage specification.
- Migration 3 adds `photos.file_type`, backfills `jpeg`, `raw`, and `unsupported`, and creates the accepted paged-query indexes.
- Migration 4 adds nullable `photo_metadata.width`, `photo_metadata.height`, and `photo_metadata.orientation` columns for import-time metadata extraction.
- Migration 5 adds `idx_photo_metadata_dimensions_photo_id` for the accepted `has_dimensions` metadata filter.
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
- Task 11.5.4 exposes the paged library query through the desktop command boundary as `query_library_photos`.
- Task 11.5.5 moves product grid thumbnail hydration to the requested page only.
- Task 11.6.1 wires the product grid UI to page metadata, visible loading/empty/error states, and previous/next page controls without claiming hidden rows are rendered.
- Task 11.6.2 renders only a visible page-local grid window plus overscan spacer rows and revokes grid-owned thumbnail object URLs as rows leave the rendered window.
- Task 11.6.3 adds current-page roving-focus keyboard navigation for the product grid without changing the paged query contract.
- Task 11.6.4 keeps product grid multi-selection page-local and UI-only: primary selection stays explicit, range/toggle selection updates visual state and counts, and batch catalog edits remain out of scope.
- Task 11.7.1 records the metadata schema/dependency gate: no EXIF parser is added yet, and unavailable camera/lens/orientation/capture metadata must stay explicit.
- Task 11.7.2 records the backfill/extraction policy: no open/restore backfill, existing unknown metadata stays unknown until explicit import/backfill work, and JPEG/JPG dimensions may use the existing raster path without implying RAW decode support.
- Task 11.7.3 adds migration 4 plus import-time metadata persistence: JPEG/JPG width and height are stored when available, RAW rows stay explicitly unavailable, unsupported files do not get fake metadata rows, and originals remain unchanged.
- Task 11.7.4 adds typed metadata read APIs through storage, core, and desktop command boundaries. Query responses use explicit `known`, `unknown`, and `unavailable` field states and read only the catalog, not original files.
- Task 11.8.1 wires the Library/Loupe inspector to `get_photo_metadata` and keeps multi-selection metadata primary-photo-only.
- Task 11.8.2 adds the first metadata-backed query filter: `has_dimensions`, backed by stored `photo_metadata.width` and `photo_metadata.height`. Camera/lens filters remain unavailable until parser/index support exists.

## Paged Library Query Contract

Task 11.5 starts with a contract before storage implementation:

- Pagination is offset-based with `DEFAULT_LIBRARY_QUERY_LIMIT = 100` and `MAX_LIBRARY_QUERY_LIMIT = 500`.
- Cursor pagination is explicitly deferred until benchmark evidence requires it.
- Accepted sort modes are `imported_at_desc`, `file_name_asc`, and `rating_desc`.
- Deterministic ordering includes explicit tie breakers:
  - imported-at descending, then photo id ascending
  - file name ascending, then path ascending, then photo id ascending
  - rating descending, then photo id ascending
- Accepted filters are minimum rating, picked, rejected, file type, `has_dimensions`, and search text.
- File type is a whitelisted enum: JPEG, RAW, or unsupported. Storage records this as normalized `photos.file_type` values `jpeg`, `raw`, and `unsupported`; in the local alpha, `raw` means supported non-JPEG photo candidates until later metadata extraction expands the taxonomy.
- Required query indexes are:
  - `idx_photos_library_imported_id`
  - `idx_photos_library_file_name_path_id`
  - `idx_photos_library_file_type_id`
  - `idx_photo_metadata_dimensions_photo_id`
  - `idx_photo_flags_rating_photo_id`
- Storage must translate the typed contract internally; UI code must not pass SQL strings, column names, or raw predicates.
- `silica-storage::query_library_photos` opens the catalog read-only, returns page metadata, and leaves compatibility full-list behavior in `list_library_photos` until the desktop grid migration is complete.
- The desktop `query_library_photos` command accepts a typed page/sort/filter DTO, hydrates thumbnails only for rows in the requested page, and returns `photoGridPage` metadata.

## Photo Metadata Contract

Task 11.7 starts with a storage-shape and dependency gate before extraction:

- No EXIF or camera metadata parser dependency is added in Task 11.7.1.
- No automatic metadata backfill runs on app launch, library open, or session restore.
- `photo_metadata` normalized fields are nullable values: `width`, `height`, `orientation`, `capture_time`, `camera_make`, `camera_model`, and `lens_model`.
- `photos.file_size` and `photos.modified_at` remain file-system metadata captured at import time; they are not duplicated into `photo_metadata`.
- Existing imports without `photo_metadata` rows remain unknown until an import-time extractor or explicit scoped backfill populates them.
- JPEG/JPG dimensions may be read through the existing raster path already used for thumbnails/previews.
- Import-time JPEG/JPG extraction stores width and height when `image::image_dimensions` can read them; failed reads leave metadata unavailable instead of inventing values.
- RAW metadata policy does not imply RAW decode support; RAW dimensions and camera/lens metadata stay unavailable until later gates add supported extraction.
- Until a parser is added, camera make, camera model, lens model, orientation, and EXIF capture time are stored and displayed as unavailable rather than inferred.
- `photo_metadata.raw_json` remains parser-owned untrusted data and defaults to `{}`.
- Migration 4 adds the first physical metadata extraction columns. Existing imports are not backfilled on open or session restore.
- `silica-storage::get_photo_metadata`, the core wrapper, and desktop `get_photo_metadata` command expose stored metadata only. A missing metadata row reports nullable extraction fields as `unknown`; a present metadata row with `NULL` values reports them as `unavailable`; stored values report `known`.
- Metadata query APIs use the read-only catalog query path and must not touch original files during inspector display.

## Import Error and Recursive Import Policy

Task 11.9 requires reviewable import errors before recursive import exists:

- Folder import remains non-recursive by default. Recursive import must be an explicit user-selected option and must never be enabled silently by restore, relaunch, recents, or folder drag/open behavior.
- Originals remain referenced by path only. Import scanning must not copy, move, rewrite, hash-whole-file by default, write sidecars next to originals, or modify source folders.
- Browsing must continue after recoverable import issues. A scan may return accepted catalog rows plus a reviewable issue list in the same summary.
- Unsupported files are reviewable entries, not crashes and not fake photo metadata. Existing unsupported catalog rows remain visible in the Library grid.
- Recoverable issue categories for the current and recursive paths:
  - `unsupported_file`: file extension is outside the accepted alpha photo list.
  - `hidden_entry_skipped`: dotfile or dot-directory skipped by policy.
  - `package_directory_skipped`: macOS package-like directory skipped by policy.
  - `symlink_entry_skipped`: symbolic link skipped by policy.
  - `directory_read_failed`: a directory cannot be read, usually permissions or removal during scan.
  - `entry_metadata_failed`: a directory entry exists but file metadata cannot be read.
  - `max_depth_exceeded`: recursive scan reached the alpha depth limit.
- Symlink directories are not followed. Symlink files are also skipped and reported as `symlink_entry_skipped` so recursive scans cannot loop or escape the selected tree.
- Hidden files and hidden directories are skipped and reviewable. Later preferences may change this only with an explicit task and UI control.
- macOS package directories are skipped and reviewable. The alpha treats `.app`, `.photoslibrary`, `.aplibrary`, `.lrdata`, `.library`, and other package-like directories as containers, not folders to descend into.
- Recursive import alpha max depth is `20` directory levels below the selected root. Entries past the limit are not scanned and are reported as `max_depth_exceeded`.
- Permission and file-system race errors are recoverable at entry/directory granularity. The scan continues with siblings when possible.
- Task 11.9.2 implements the structured model for the current non-recursive path. `FolderImportSummary.issues` returns reviewable `ImportIssue` records alongside accepted catalog rows, `silica-core` re-exports the model, and the desktop import command forwards the issue list.
- Task 11.9.3 adds the desktop import issue review surface. The review list displays unsupported files, skipped entries, and failed entries from the latest import while the library grid remains loaded behind the import panel. It does not enable recursive scanning.
- Task 11.9.4 adds opt-in recursive scanning through `FolderImportOptions { recursive: true }` and the desktop `Include subfolders` checkbox. The default import path remains non-recursive. Recursive scans keep the same issue model and skip symlinks rather than following them.

## Not Implemented Yet

- Camera metadata extraction.
- Thumbnail or preview generation during import.
- Original full-hash protection behavior.
- Automatic sidecar synchronization.
- Applied catalog rebuild or restore from sidecars.
- Sidecar conflict handling and conflict UI.
- Full edit history and undo/redo persistence.
- Product undo/redo commands and history panel data.
- Cache clear undo behavior beyond the Phase 16 action-trust policy.
- Full paged grid UI states and pagination controls.
- Broad catalog UI screens beyond the local alpha workflow.
- Plugin or MCP catalog access.

## Links

- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Data Safety](data-safety.md)
- [Action Trust](action-trust.md)
- [Backup and Restore](backup-restore.md)
- [`silica-catalog` README](../../../crates/silica-catalog/README.md)
- [`silica-storage` README](../../../crates/silica-storage/README.md)

## Notes for LLM Agents

Do not treat folder import, flag persistence, active edit graph commits, or explicit sidecar writes as RAW decoding, thumbnail generation, automatic sidecar sync, cache behavior, or library grid behavior. `photo_flags` is the live in-app authority until sidecar synchronization is implemented by an explicit later task.
