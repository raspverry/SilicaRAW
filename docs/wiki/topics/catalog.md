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
- The catalog remains local-first and referenced-folder by default.
- Original photo files must not be modified by catalog work.

## Implemented Foundation

- Current alpha schema version: `2`.
- Migration table: `schema_migrations`.
- Migration 1 creates the initial catalog tables.
- Migration 2 creates the required initial indexes from the storage specification.
- Tests cover empty catalog creation, migration upgrade from version 1 to latest, required table/index existence, foreign key enforcement, and file-backed WAL/foreign key configuration.

## Not Implemented Yet

- Library folder create/open workflow.
- Folder scanner and import records.
- Photo fingerprinting and original hash protection.
- Rating, pick, reject, and label commands.
- Sidecar read/write and conflict handling.
- Cache clear safety.
- UI screens or Tauri commands for catalog workflows.
- Plugin or MCP catalog access.

## Links

- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Data Safety](data-safety.md)
- [`silica-catalog` README](../../../crates/silica-catalog/README.md)
- [`silica-storage` README](../../../crates/silica-storage/README.md)

## Notes for LLM Agents

Do not treat the migration foundation as library create/open, import, sidecar, cache, or UI behavior. Task 4.2 is the first place to add local library create/open behavior, and it must preserve original-file safety.
