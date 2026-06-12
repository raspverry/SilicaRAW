# 10 — SilicaRAW Data Model & Storage Specification

Status: GO WITH CONDITIONS

## Principle

Original files are sacred. Catalog state is recoverable. Edits are versioned. Caches are disposable. Sidecars are portable.

## Storage Modes

v1 default: referenced-folder mode.

- Photos stay where they are
- SilicaRAW catalogs paths and metadata
- Edits stored in catalog and sidecars

Future: managed-library mode.

## Library Structure

```txt
SilicaRAW Library/
├─ catalog.db
├─ sidecars/
├─ thumbnails/
├─ previews/
├─ render-cache/
├─ ai-cache/
├─ exports/
├─ logs/
└─ backups/
```

## SQLite Tables

- libraries
- folders
- photos
- photo_metadata
- photo_flags
- collections
- collection_photos
- edit_states
- edit_history
- presets
- sidecar_status
- cache_records
- ai_results
- exports
- action_log
- schema_migrations

## Photo Identity

Do not rely only on path. Use:

- photo_id, UUID/ULID
- path
- file size
- modified time
- partial hash
- optional full hash
- capture time
- camera metadata

## Edit Graph Storage

Versioned JSON stored in `edit_states`, exported to sidecar JSON.

Metadata flags are separate from image edits. Export settings are separate from develop edits.

## Sidecar JSON

v1 default: stored in library sidecars folder. Later optional sidecar next to original.

Sidecars include:

- photo identity
- source fingerprint
- edit graph
- ratings/pick/reject
- app/schema version

## XMP

Compatibility layer, not primary format. v1 may support ratings/labels/simple metadata. Do not promise Lightroom edit compatibility.

## Cache

Caches are disposable:

- thumbnails
- previews
- render intermediates
- histogram
- AI results/masks

Deleting cache must not delete originals, edits, ratings, collections, presets, or sidecars.

## AI Results

AI results are separate from edits until approved. Approval converts suggestions to edit graph values or metadata changes.

## MCP/Plugin Safety

All MCP/plugin mutations go through Core APIs and action_log. No direct DB access. No original file mutation.

## Final Verdict

GO WITH CONDITIONS.

Spike 004 added the initial embedded SQLite migrations and required indexes to `silica-storage`.

Phase 4.1 records the local alpha schema contract in `silica-catalog` and makes `silica-storage` verify migration output against that domain-facing contract.

Phase 4.2 adds file-backed local library create/open behavior.

Phase 4.3 adds non-recursive folder import records with path, file size, modified time, partial hash, and unsupported state.

Phase 4.4 adds catalog-authoritative rating, picked, rejected, and color label persistence in `photo_flags`, including default rows for imported photos and restart tests.

Phase 5.2 adds Rust edit graph types and schema-aware validation in `silica-edit`.

Phase 5.3 adds active edit graph storage wiring for exposure/contrast commits. Draft preview updates do not write `edit_states`; commit/release writes the active graph.

Task 10.3 adds library-local sidecar read/write foundations. Task 10.4 adds sidecar rebuild dry-run reports. Task 10.5.1 records backup/checkpoint/restore policy. Task 10.5.2 adds checkpointed backup artifact creation for `catalog.db`, `sidecars/`, and `backup-manifest.json`. Task 10.5.3 adds staged restore with existing-target rollback copies and newer-schema rejection before target mutation.

Still needed: broader edit graph storage workflows, automatic sidecar synchronization, sidecar conflict UX, cache size policy, user-facing backup/restore commands and UI, recursive import policy, and original full-hash protection behavior.

---

# v1.1 Patch — Authoritative Schemas and SQLite Indexes

## Authoritative Schemas

Use:

```txt
schemas/edit_graph.schema.json
schemas/edit_graph.example.json
schemas/sidecar.schema.json
```

Codex must implement Rust types equivalent to these schemas.

## Minimum SQLite Indexes

The initial migration must include these indexes unless benchmark tests prove a better alternative.

Spike 004 implements these indexes in schema version 2 and tests every index by name.

```sql
CREATE INDEX IF NOT EXISTS idx_folders_library_id
  ON folders(library_id);

CREATE INDEX IF NOT EXISTS idx_photos_library_id
  ON photos(library_id);

CREATE INDEX IF NOT EXISTS idx_photos_folder_id
  ON photos(folder_id);

CREATE INDEX IF NOT EXISTS idx_photos_capture_time
  ON photos(capture_time);

CREATE INDEX IF NOT EXISTS idx_photos_imported_at
  ON photos(imported_at);

CREATE INDEX IF NOT EXISTS idx_photos_missing
  ON photos(missing);

CREATE INDEX IF NOT EXISTS idx_photos_unsupported
  ON photos(unsupported);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rating
  ON photo_flags(rating);

CREATE INDEX IF NOT EXISTS idx_photo_flags_rejected
  ON photo_flags(rejected);

CREATE INDEX IF NOT EXISTS idx_photo_flags_picked
  ON photo_flags(picked);

CREATE INDEX IF NOT EXISTS idx_photo_flags_label
  ON photo_flags(color_label);

CREATE INDEX IF NOT EXISTS idx_collections_library_id
  ON collections(library_id);

CREATE INDEX IF NOT EXISTS idx_collection_photos_photo_id
  ON collection_photos(photo_id);

CREATE INDEX IF NOT EXISTS idx_edit_states_photo_id
  ON edit_states(photo_id);

CREATE INDEX IF NOT EXISTS idx_edit_states_photo_active
  ON edit_states(photo_id, active);

CREATE INDEX IF NOT EXISTS idx_edit_history_photo_id
  ON edit_history(photo_id);

CREATE INDEX IF NOT EXISTS idx_edit_history_photo_sequence
  ON edit_history(photo_id, sequence);

CREATE INDEX IF NOT EXISTS idx_edit_history_photo_state_sequence
  ON edit_history(photo_id, history_state, sequence);

CREATE INDEX IF NOT EXISTS idx_cache_records_photo_type
  ON cache_records(photo_id, cache_type);

CREATE INDEX IF NOT EXISTS idx_cache_records_key
  ON cache_records(cache_key);

CREATE INDEX IF NOT EXISTS idx_ai_results_photo_task
  ON ai_results(photo_id, task_type);

CREATE INDEX IF NOT EXISTS idx_ai_results_model
  ON ai_results(model_id, task_type);

CREATE INDEX IF NOT EXISTS idx_exports_photo_id
  ON exports(photo_id);

CREATE INDEX IF NOT EXISTS idx_action_log_actor
  ON action_log(actor_type, actor_id);

CREATE INDEX IF NOT EXISTS idx_action_log_created_at
  ON action_log(created_at);
```

## Index Rules

```txt
Add indexes through migrations.
Benchmark common Library queries with 10k and 50k photos.
Do not store queryable UI state only in JSON.
Queryable UI state should have normalized columns.
```

---

# v1.3 Patch — Flags Authority and Sidecar Rebuild Semantics

## Flags Authority

Inside a live SilicaRAW catalog, `photo_flags` is authoritative for:

```txt
rating
picked
rejected
color_label
edited
exported
```

`edit_graph.metadata` is a portable snapshot, not the primary query source.

`sidecar.flags` mirrors the latest catalog flags at the time of sidecar write.

## Catalog Rebuild from Sidecars

When rebuilding from sidecars:

```txt
1. Use sidecar.flags when present and valid.
2. Fall back to edit_graph.metadata.
3. Fall back to defaults:
   rating = 0
   picked = false
   rejected = false
   color_label = null
```

## Conflict Rules

If catalog, sidecar.flags, and edit_graph.metadata disagree:

```txt
- catalog wins during normal app operation
- sidecar conflict state is recorded
- user-facing conflict resolution is P2
- do not silently overwrite newer sidecar data
```

---

# v1.4 Patch — Catalog-only Flags Clarification

`photo_flags` in SQLite remains authoritative for all live Library state:

```txt
rating
picked
rejected
color_label
edited
exported
```

However, `sidecar.flags` intentionally includes only the portable user culling/label flags:

```txt
rating
picked
rejected
color_label
```

The following are catalog-only and must not be added to `sidecar.flags` in v0.1:

```txt
edited
exported
```

Reason:

```txt
edited:
Derived from active edit state / edit graph presence and may be recomputed during catalog rebuild.

exported:
Represents local export history and output paths, which are catalog/session/workflow state rather than portable photo intent.
```

Catalog rebuild rule:

```txt
- Rebuild rating/picked/rejected/color_label from sidecar.flags.
- Recompute edited from edit graph presence.
- Do not restore exported from sidecar.flags.
- Export history must come from catalog exports table, not sidecar flags.
```

Codex rule:

```txt
Do not add edited/exported to sidecar.flags unless a future schema version explicitly changes this decision.
```
