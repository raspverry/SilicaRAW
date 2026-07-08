---
title: Data Safety
status: active
audience: all
updated: 2026-06-17
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
- Phase 4.3 records imported folder files by reference and verifies mixed-file import does not copy or mutate originals. The desktop import response is backed by a post-import full SHA-256 source check rather than a hard-coded success claim.
- Phase 4.4 persists culling flags in SQLite `photo_flags` and verifies they survive library reopen without touching originals or sidecars.
- Phase 5.3 persists exposure/contrast edit graphs in SQLite `edit_states` on commit and verifies draft preview updates do not write edit state rows.
- Phase 6.2 adds a core workflow hash QA that verifies one original fixture hash stays unchanged across import by reference, culling flags, preview, draft edit, committed edit, JPEG sRGB export, simulated cache-directory clearing, and library reopen.
- Task 10.3 writes sidecars only under the library `sidecars/` directory, validates sidecar and nested edit graph JSON, updates `sidecar_status` only after a successful write, and verifies original referenced files remain unchanged.
- Task 10.4 adds a rebuild dry-run report that reads library-local sidecars as untrusted input and reports rebuild actions/issues without mutating the live catalog or original files.
- Task 10.5.1 records the [Backup and Restore](backup-restore.md) policy: checkpoint-before-copy, no original referenced photo files in backups, disposable cache exclusion, rollback-aware restore targets, and explicit migration failure behavior.
- Task 10.5.2 adds checkpointed backup boundary creation for `catalog.db`, `sidecars/`, and `backup-manifest.json` under `backups/` while excluding originals, caches, export outputs, logs, and nested backups.
- Task 10.5.3 adds staged restore from backup artifacts with existing-target rollback copies, restored catalog/sidecar preservation, newer-schema rejection before target mutation, and original-file safety tests.
- Task 14.7 records the native viewer disposable texture lifecycle boundary: viewer texture identity is rebuildable runtime state and cleanup on photo change, drawable resize, library close, or app close does not write catalog rows, sidecars, originals, export outputs, or persistent GPU cache state.
- Task 15.2 adds fixture-backed RAW preview artifacts as disposable cache files under library `previews/`; source/output canonical matches, stale source hashes, and preview cache path escapes are rejected before trust claims are made.
- Task 15.5 adds RAW-derived JPEG sRGB export through a full-resolution source artifact under `render-cache/raw-export-sources/`; final export rejects original overwrite, records source/output/ICC hashes, records original hash unchanged evidence, and does not depend on viewer texture cache. Raster exports also record post-export source hash evidence in `exports.export_settings_json`.
- Task 16.0 records the [Action Trust](action-trust.md) boundary: undo/redo covers catalog state only, exports and cache bytes are not deleted or reconstructed by undo, sidecar writes are explicit, and original-file mutation remains blocked.
- Task 16.1 records exact action semantics so edit commits and culling flags can be undone through catalog state, while export output files, cache bytes, sidecars, backups, imports, restore attempts, and originals remain outside undo mutation.
- Task 16.3 adds transaction-safe undo/redo for edit checkpoints and culling flags. Tests verify export output files survive undo/redo and original files remain outside the command path.
- Task 16.4 adds the Develop history panel as a read-only view of real `edit_history` checkpoints plus buttons that call existing undo/redo commands. It does not add raw SQL to the UI, arbitrary state jumps, export deletion, sidecar writes, cache restoration, or original-file access.
- Task 16.5 adds append-only action log evidence for sensitive local actions through Core and storage APIs. It records import by reference, sidecar write, JPEG export, RAW-derived JPEG export, and disposable cache clear without allowing original mutation claims, plugin/MCP raw DB writes, or hidden reversibility.
- Task 23.3 records future permission and extension-sensitive events through Core action-log wrappers and rejects extension raw-SQL/direct-database bypass claims in storage. These rows are evidence only and do not enable plugin, MCP, AI, or agent runtime.
- Task 24.3 stores AI result rows under `ai_results` only. Results default to unapproved, carry `local_only = true` and `ai_result:propose`, and reject direct edit graph or photo flag mutation payloads. Storing or reading an AI result does not load a model, run inference, write edit history, or change catalog flags.
- Task 24.4 reads stored blur review rows into a review-only panel. Missing models or missing stored results keep the editor usable and do not write edit graph, edit history, catalog flags, action log approvals, sidecars, caches, exports, or original files.
- Task 24.5 approves only scoped stored AI suggestions after explicit user action. Approval commits through the existing edit graph/history path, marks the AI result approved, records provenance, and appends action-log evidence; rejection appends evidence and leaves edit state/history unchanged. Neither path writes photo flags, sidecars, caches, exports, or originals.
- Task 25.2 applies data-only plugin preset packs only after explicit Core approval. Approval commits one edit graph/history checkpoint, records plugin provenance, appends `plugin_apply` evidence, and writes no photo flags, sidecars, caches, exports, plugin runtime state, or originals.
- Task 25.3 records plugin permission review grants and denials as action-log evidence only. Reviews do not persist grants, start plugin runtime, write edit state/history, or mutate originals.
- Task 16.6 marks already-written sidecars as `catalog_newer` after edit commits, flag commits, undo, and redo. It preserves `conflict` and `sidecar_newer`, writes no sidecar files, expands no `sidecar.flags`, and keeps original files untouched.
- Task 20.1 stores export defaults and named presets in catalog-owned `export_settings` and `export_presets` tables only. Updating export preferences must not write `edit_states`, `edit_history`, sidecars, export output files, or original photo files.
- Task 21.3 exposes disposable cache status and cache clear inside Preferences. Status is read-only over disposable cache directories, and clear remains limited to `thumbnails/`, `previews/`, `render-cache/`, and `ai-cache/`.
- Task 21.4 routes Preferences Color/Export defaults through the existing export settings commands only. Defaults remain validated before save, Display P3 remains explicit and JPEG-only, and no original, sidecar, edit history, or export output files are written when defaults change.
- Task 22.3 hardens restore failure safety: corrupt backup catalogs fail with backup/target context, staging is cleaned, existing targets stay recoverable, rollback is not created before staging validation, and disposable cache records can be cleared and recorded again after restore.
- Sidecars provide portable recovery state.
- Caches may be deleted without losing originals, edits, ratings, collections, presets, or sidecars.

## Early Required Tests

- SQLite migration safety. Spike 004 covers empty catalog creation, upgrade from migration 1 to latest, required index creation, and foreign key enforcement.
- Library create/open safety. Phase 4.2 covers library support directory creation, catalog reopen, and sibling original-directory preservation.
- Folder import safety. Phase 4.3 covers mixed supported/unsupported fixture import by reference, partial hash recording, and original byte preservation.
- Flag persistence safety. Phase 4.4 covers default `photo_flags` rows, catalog-authoritative updates, and restart persistence.
- Exposure/contrast edit safety. Phase 5.3 covers draft preview updates without `edit_states` writes and commit/release persistence after library reopen.
- Original hash protection. Phase 6.2 covers the connected local alpha workflow with an automated generated-fixture hash test in `silica-core`.
- Edit graph serialization.
- Sidecar read/write safety. Task 10.3 covers library-local paths, schema-aware validation, status update after success, malformed/mismatched read rejection, and original hash preservation.
- Sidecar rebuild dry-run safety. Task 10.4 covers deterministic output, catalog non-mutation, precedence from `sidecar.flags` to `edit_graph.metadata` to defaults for schema-valid sidecars, and conflict/malformed/schema-invalid sidecar reporting.
- Recovery policy safety. Task 10.5.1 covers backup boundaries, WAL checkpoint policy, restore target rules, disposable cache exclusion, and migration failure behavior.
- Backup boundary safety. Task 10.5.2 covers checkpointed backup artifacts, manifest creation, cache/original/export-output exclusion, and latest WAL state preservation in the copied `catalog.db`.
- Restore boundary safety. Task 10.5.3 covers restore into empty targets, rollback-protected restore into existing targets, restored edit/flag/sidecar/export/migration state, newer-schema rejection before target mutation, and original-file preservation.
- Cache clear safety. Phase 6.2 simulates the currently scoped cache safety surface by deleting disposable library cache directories because no product cache-clear command exists yet.
- Native viewer texture lifecycle safety. Task 14.7 covers disposable viewer texture identity and cleanup without catalog writes, sidecar writes, original write destinations, or persistent GPU cache state.
- RAW preview artifact safety. Task 15.2 covers canonical source/output overwrite rejection, stale probe hash rejection, library `previews/` cache bounding, `..` escape rejection, unsupported-class no-write behavior, and original hash preservation for legal local RAW fixture classes.
- RAW-derived export safety. Task 15.5 covers canonical RAW source/output overwrite rejection, full-resolution export source artifact separation from preview cache, source/output/ICC hash recording, no preview cache dependency, unsupported-class no-write behavior, and original hash preservation for legal local RAW fixture classes.
- Action trust safety. Task 16.0 covers undoable vs logged-only vs non-reversible vs blocked action classes before runtime undo/history changes.
- Action semantics safety. Task 16.1 covers checkpoint units, redo invalidation, disabled states, and slider drafts creating no history entries.
- Undo/redo safety. Task 16.3 covers edit and flag undo/redo transactions, redo invalidation after a new undoable action, and export-output preservation.
- History panel safety. Task 16.4 covers real-checkpoint-only UI data, empty/loading/error/disabled states, and row selection through core undo/redo commands only.
- Action log safety. Task 16.5 covers append-only action log rows, required side-effect/evidence fields, Core logging for sensitive local actions, and rejection of original mutation claims.
- Sidecar status safety. Task 16.6 covers catalog-side stale status after history commits, conflict/newer preservation, reopen persistence, no hidden sidecar file writes, and no `sidecar.flags` schema expansion.
- Export settings safety. Task 20.1 covers schema migration to version 9, conservative JPEG sRGB defaults, named preset reload, no edit history writes, and no original-file or export-output writes when preferences change.
- Preferences cache safety. Task 21.3 covers read-only cache status, Preferences cache clear using the existing disposable boundary, and app-session Library default path persistence without launch behavior changes.
- Preferences export defaults safety. Task 21.4 covers Preferences-driven default format, quality, and color-space updates through the existing export settings path, including unsupported-combination blocking and no second preferences store.

## Links

- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Schema Reference](../../19_Schema_Reference.md)
- [Agent Rules](../../../codex/AGENT_RULES.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [Catalog](catalog.md)
- [Action Trust](action-trust.md)
- [Backup and Restore](backup-restore.md)

## Notes for LLM Agents

Any task that touches files, catalog state, sidecars, exports, or caches must preserve original-file safety. Do not add convenience file operations that can mutate or delete originals.
