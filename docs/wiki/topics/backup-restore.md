---
title: Backup and Restore
status: active
audience: all
updated: 2026-06-11
source_of_truth: docs/superpowers/specs/2026-06-11-phase-10-evidence-recovery-design.md
---

# Backup and Restore

## Summary

This page records the Task 10.5 recovery policy and the current backup boundary implementation.

The policy goal is narrow: preserve a local SilicaRAW library's recoverable state without copying original referenced photo files, disposable caches, or future cloud/plugin/AI state.

## Backup Policy

Task 10.5 uses a checkpoint-before-copy backup policy.

Before copying catalog data, the app must:

- Open the library through `silica-storage`.
- Block or avoid concurrent library writes for the backup operation.
- Run a SQLite WAL checkpoint for `catalog.db`.
- Abort the backup if the checkpoint cannot complete cleanly.

The initial backup artifact should contain:

- `catalog.db` after the checkpoint.
- `sidecars/`.
- A backup manifest with app version, catalog schema version, creation time, and relative file list.

The initial backup artifact must not contain:

- Original referenced photo files.
- `thumbnails/`.
- `previews/`.
- `render-cache/`.
- `ai-cache/`.
- Existing `backups/` artifacts.
- Transient `logs/`, unless a later support-bundle task explicitly includes them.
- Files reached by following arbitrary export output paths.

If WAL or SHM files still contain required state after checkpoint, the backup must either include them explicitly with a manifest note or fail. It must not silently copy only part of a live SQLite state.

## Current Backup Implementation

Task 10.5.2 adds `silica-storage::create_library_backup`.

The current backup artifact is a library-local directory:

```txt
<library_root>/backups/<backup_id>/
├─ catalog.db
├─ sidecars/
└─ backup-manifest.json
```

The backup manifest uses:

```txt
schema = silica.backup
version = 1
```

It records app version, catalog schema version, creation time, checkpoint mode, relative file list, and excluded classes. It does not include cache files, original referenced photo files, export output files, logs, nested backup artifacts, or temporary sidecar write files.

The current implementation runs `PRAGMA wal_checkpoint(TRUNCATE)` before copying `catalog.db`. If the checkpoint reports a busy state, backup creation fails instead of copying a partial catalog state.

## Restore Policy

Restore must target either:

- An empty destination directory.
- An existing library root after first creating an internal rollback copy of the current catalog and sidecar state.

Restore must not:

- Write into original referenced photo folders.
- Move, delete, or relink original files.
- Merge sidecars into a live catalog without an explicit later conflict-resolution task.
- Recreate disposable caches as required data.
- Trust backup JSON or sidecar JSON without validation.

After copying backup data into the destination, the app must reopen the restored catalog through normal migration code. If migration fails, restore must abort and keep the destination recoverable through the rollback copy or untouched empty destination.

## Durable Data

These are durable recovery inputs:

- `catalog.db`.
- SQLite WAL/SHM state only when required by the checkpoint result.
- `sidecars/`.
- `edit_states` rows inside the catalog.
- `photo_flags` rows inside the catalog.
- `sidecar_status` rows inside the catalog, with library-relative sidecar paths.
- `exports` rows inside the catalog.
- `schema_migrations` rows inside the catalog.

## Disposable Data

These are disposable and must not be required for restore correctness:

- Thumbnails.
- Preview JPEGs.
- Render caches.
- AI caches.
- Temporary files.
- Runtime logs, unless a later support bundle includes them separately.

## Migration Failure Policy

Backup restore must handle migration failures explicitly.

Rules:

- A backup from a newer catalog schema must be rejected by older app builds.
- A backup from an older catalog schema may be restored only through normal migration code.
- If migration fails, the restored catalog must not replace a known-good existing library.
- The error report must keep the backup artifact path and target path visible to maintainers.
- The app must not attempt partial table-by-table repair in Task 10.5.

## QA Policy

Task 10.5 implementation tests must prove:

- Backup excludes disposable caches. Covered by Task 10.5.2.
- Backup includes catalog and sidecars. Covered by Task 10.5.2.
- Backup does not include original referenced files. Covered by Task 10.5.2.
- Restore preserves edit states, flags, sidecar status, export records, and migration metadata.
- Restore does not write into original photo folders.
- Migration failure leaves a recoverable target state.

## Links

- [Data Safety](data-safety.md)
- [Catalog](catalog.md)
- [Post-Alpha Product Roadmap](../roadmaps/post-alpha-product-roadmap.md)
- [Phase 10 Evidence and Recovery Design](../../superpowers/specs/2026-06-11-phase-10-evidence-recovery-design.md)
- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)

## Notes for LLM Agents

Do not implement restore by copying over a live library in place. Do not chase original photo paths or export output paths. Start with explicit backup boundaries and tested rollback behavior before adding any user-facing restore control.
