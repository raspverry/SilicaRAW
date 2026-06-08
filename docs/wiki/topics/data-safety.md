---
title: Data Safety
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/10_Data_Model_and_Storage_Specification.md
---

# Data Safety

## Summary

Data safety is a core trust requirement. Originals are sacred, catalog state must be recoverable, edits are versioned, and caches are disposable.

## Current Stance

- Original photo files must never be modified by SilicaRAW.
- Catalog state lives in SQLite through `silica-storage`.
- Spike 004 selected `rusqlite` with bundled SQLite and embedded SQL migrations.
- Phase 4.1 records the local alpha schema contract in `silica-catalog` and verifies `silica-storage` migrations against it.
- Phase 4.2 adds local library create/open without mutating sibling original photo directories.
- Phase 4.3 records imported folder files by reference and verifies mixed-file import does not copy or mutate originals.
- Phase 4.4 persists culling flags in SQLite `photo_flags` and verifies they survive library reopen without touching originals or sidecars.
- Sidecars provide portable recovery state.
- Caches may be deleted without losing originals, edits, ratings, collections, presets, or sidecars.

## Early Required Tests

- SQLite migration safety. Spike 004 covers empty catalog creation, upgrade from migration 1 to latest, required index creation, and foreign key enforcement.
- Library create/open safety. Phase 4.2 covers library support directory creation, catalog reopen, and sibling original-directory preservation.
- Folder import safety. Phase 4.3 covers mixed supported/unsupported fixture import by reference, partial hash recording, and original byte preservation.
- Flag persistence safety. Phase 4.4 covers default `photo_flags` rows, catalog-authoritative updates, and restart persistence.
- Original hash protection.
- Edit graph serialization.
- Sidecar read/write.
- Cache clear safety.

## Links

- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Schema Reference](../../19_Schema_Reference.md)
- [Agent Rules](../../../codex/AGENT_RULES.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [Catalog](catalog.md)

## Notes for LLM Agents

Any task that touches files, catalog state, sidecars, exports, or caches must preserve original-file safety. Do not add convenience file operations that can mutate or delete originals.
