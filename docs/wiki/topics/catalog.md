---
title: Catalog
status: active
audience: all
updated: 2026-06-08
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
- The catalog remains local-first and referenced-folder by default.
- Original photo files must not be modified by catalog work.

## Implemented Foundation

- Current alpha schema version: `2`.
- Migration table: `schema_migrations`.
- Migration 1 creates the initial catalog tables.
- Migration 2 creates the required initial indexes from the storage specification.
- Tests cover empty catalog creation, migration upgrade from version 1 to latest, required table/index existence, foreign key enforcement, and file-backed WAL/foreign key configuration.
- Library create/open creates the selected library folder, `catalog.db`, and required support directories.
- Reopening a library migrates the same `catalog.db` and returns the active root/catalog/schema status.
- Tests cover sibling original-directory preservation during create/open.
- Folder import records immediate child files by reference in `folders` and `photos`.
- Import records include path, file size, modified time, partial hash, and unsupported state.
- Tests cover mixed supported/unsupported fixture files and confirm originals are not copied or mutated.

## Not Implemented Yet

- Recursive folder scanning.
- Camera metadata extraction.
- Thumbnail or preview generation during import.
- Original full-hash protection behavior.
- Rating, pick, reject, and label commands.
- Sidecar read/write and conflict handling.
- Cache clear safety.
- Broad catalog UI screens.
- Plugin or MCP catalog access.

## Links

- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Data Safety](data-safety.md)
- [`silica-catalog` README](../../../crates/silica-catalog/README.md)
- [`silica-storage` README](../../../crates/silica-storage/README.md)

## Notes for LLM Agents

Do not treat folder import as RAW decoding, thumbnail generation, sidecar, cache, or library grid behavior. Task 4.4 is the first place to add rating, pick, and reject persistence.
