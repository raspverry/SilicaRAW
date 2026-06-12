# 19 — SilicaRAW Schema Reference v1.1

Status: AUTHORITATIVE FOR v0.1 IMPLEMENTATION

## Purpose

This document defines schema sources that Codex / Claude Code must use when implementing storage, sidecars, plugin manifests, MLX model manifests, and MCP tool declarations.

## Authoritative Schema Files

```txt
schemas/edit_graph.schema.json
schemas/edit_graph.example.json
schemas/sidecar.schema.json
schemas/sidecar.example.json
schemas/fixture_manifest.schema.json
schemas/fixture_manifest.example.json
schemas/plugin_manifest.schema.json
schemas/model_manifest.schema.json
schemas/mcp_tool_manifest.schema.json
```

## Codex Rules

```txt
1. Do not invent a different edit graph structure.
2. Implement typed Rust structs equivalent to schemas/edit_graph.schema.json.
3. Edit graph JSON must validate against schemas/edit_graph.schema.json.
4. Sidecar JSON must validate against schemas/sidecar.schema.json.
5. Plugin manifests must validate before enabling plugins.
6. Model manifests must include license/source/hash/preprocessing/output metadata.
7. MCP tools must declare permission, side effects, confirmation behavior, and undo behavior.
8. Experimental data belongs under `extensions`.
```

## Fixture Manifest v1

`schemas/fixture_manifest.schema.json` is the authoritative contract for post-alpha RAW/color fixture manifests.

`schemas/fixture_manifest.example.json` is an example-only external-reference manifest. It does not identify real local sample files and must not be treated as a committed fixture corpus.

The fixture manifest records legal RAW/color fixture provenance, licensing, integrity, expected app behavior, and future probe expectations. It does not prove RAW support or color correctness.

Required fixture guardrails:

```txt
- RAW fixture classes A-E record source, license, privacy, integrity, media metadata, expected app state, expected probe state, RAW metadata, and a blocked decode gate.
- RAW decode gates remain blocked until fixture-backed Core Image probe work records evidence in the later RAW proof phase.
- Color Class F records tagged sRGB, tagged Display P3, and untagged raster expectations with profile policy metadata.
- User photos and unlicensed samples must not be committed.
- Fixture paths must be relative POSIX paths without absolute prefixes, dot segments, backslashes, or double slashes.
```

## Edit Graph v0.1 Required Sections

```txt
source
profile
basic
tone
color
detail
lens
geometry
masks
metadata
extensions
```

## Edit Graph `profile`

The edit graph `profile` object is the authoritative color metadata contract for edit state.

```txt
profile.input_profile -> explicit input profile evidence, or "unknown" when unavailable
profile.working_space -> working color space, currently "linear_display_p3"
profile.camera_profile -> optional camera/profile identifier, null when unavailable
profile.decoder_backend -> core_image_raw | libraw | embedded_preview | raster | null
```

Agents must not invent parser-owned or color-profile fields outside `profile`. Experimental data still belongs under `extensions`, but current profile state belongs in the schema-owned `profile` object.

## Export Record Color Metadata

The existing `exports.export_settings_json` field records export color metadata for local alpha proof work.

Required color metadata keys for JPEG exports:

```txt
color_profile
output_sha256
icc_profile_embedded
icc_profile_sha256
profile_metadata_source
```

These fields are export evidence. They do not prove visual color correctness.

## Undo, History, and Action Trust

Task 16.0 defines action trust before runtime undo/history changes.

Schema ownership:

```txt
edit graph JSON shape -> schemas/edit_graph.schema.json and silica-edit
catalog action tables -> silica-catalog contract and silica-storage migrations
product mutation policy -> silica-core typed commands
desktop UI -> presentation only, no raw SQL or schema ownership
```

Action class rules:

```txt
- Committed edit graph changes and photo flag changes are undoable catalog transactions.
- Redo replays only previously undone undoable catalog actions.
- Export creation, sidecar write, import by reference, backup creation, and restore attempt records are logged-only.
- Cache clear is non-reversible; cache bytes are disposable and must not be reconstructed by undo.
- Original photo mutation, original overwrite export paths, sidecar path escape, and direct extension DB writes are blocked.
```

Existing tables used by Phase 16:

```txt
edit_states
edit_history
photo_flags
exports
sidecar_status
cache_records
action_log
```

Task 16.0 does not change schemas or migrations. Task 16.1 owns the typed action semantics contract before persistence and command implementation.

Task 16.1 action payload contract:

```txt
schema: silica.action
version: 1
class: undoable | logged_only | non_reversible | blocked
kind: edit_commit | flag_change | export | import_reference | sidecar_write | backup | restore_attempt | cache_clear
photo_id or subject
before and after for undoable actions
side_effect and evidence_ref for logged-only actions
created_by: core
```

Undoable `before` and `after` values must be catalog state snapshots. They must not rely on export files, cache bytes, sidecar files, original files, or live decoder output to restore state.

Task 16.2 storage status:

```txt
catalog schema version -> 6
edit_history.sequence -> per-photo checkpoint order
edit_history.action_class -> undoable for edit checkpoints
edit_history.action_kind -> edit_commit for exposure/contrast commits
idx_edit_history_photo_sequence -> ordered per-photo history lookup
```

Committed exposure/contrast edits write one active `edit_states` row and one `edit_history` row in the same transaction. The history `action_json` stores schema-valid before/after edit graphs. Slider drafts still write no `edit_states` or `edit_history` rows.

Task 16.3 storage status:

```txt
catalog schema version -> 7
edit_history.history_state -> applied | undone | invalidated
idx_edit_history_photo_state_sequence -> per-photo undo/redo lookup
```

Undo restores the latest applied undoable row for a photo. Redo reapplies the earliest undone row for a photo. New undoable checkpoints invalidate undone rows for that photo. Runtime support currently covers `edit_commit` and `flag_change`; logged-only actions remain outside undo/redo mutation.

Task 16.4 runtime query status:

```txt
list_photo_history -> reads edit_history rows for one photo
get_photo_history -> desktop command envelope for the Develop history panel
visible row states -> applied | undone
invalidated rows -> hidden from the product history panel
selectable row -> latest applied row for undo or earliest undone row for redo only
```

The Develop history panel is presentation only. It must not own raw SQL, schema shape, or arbitrary checkpoint jumps. Selecting a row calls the same core undo/redo command path used by toolbar/buttons.

Task 16.5 action log status:

```txt
catalog schema version -> 8
action_log.side_effect_category -> required side-effect class
action_log.evidence_ref -> optional durable evidence pointer
idx_action_log_action_type_created_at -> action timeline lookup
idx_action_log_subject -> target lookup
```

Core now exposes append/read action-log APIs and records local alpha sensitive actions for import by reference, sidecar write, JPEG export, RAW-derived export, and disposable cache clear. The action log is append-only through Core-facing APIs; it is not an undo stack and must not be used to authorize direct plugin/MCP database writes.

## Migration Policy

```txt
Every edit graph has `version`.
v0.1 is version 1.
Breaking schema changes increment version.
Migration code must be deterministic and tested.
Unknown fields must not be silently discarded.
```

## Validation Tests Required

```txt
[x] edit graph serialization validates in crates/silica-edit for Phase 5.2
[ ] sidecar serialization validates
[ ] plugin manifest rejects missing license
[ ] model manifest rejects missing hash/license
[ ] MCP tool schema rejects missing permission/side-effect declarations
```

Phase 5.2 status: `crates/silica-edit` implements typed edit graph structures and validates the v0.1 schema marker, version, closed objects, enums, numeric ranges, mask adjustment numbers, and `extensions` placement. This is the schema/type boundary only; sidecar persistence and edit application are separate tasks.

---

# v1.3 Clarifications

## Sidecar `flags` vs Edit Graph `metadata`

SilicaRAW intentionally stores rating/pick/reject/color-label information in more than one place, but each location has a different role.

### Catalog `photo_flags`

`photo_flags` in SQLite is the authoritative in-app source for current Library state.

It is used for:

```txt
Library filtering
Rating/reject/pick display
Smart collections
Culling workflows
Fast queries
Metadata-backed `has_dimensions` filtering when indexed
```

### Edit Graph `metadata`

`edit_graph.metadata` is a portable snapshot included with the edit graph.

It exists so an edit graph remains meaningful when exported, copied, or inspected outside the live catalog.

It is not the primary query source inside the app.

### Sidecar `flags`

`sidecar.flags` is the latest portable mirror of catalog flags at the time the sidecar is written.

It exists for:

```txt
Catalog rebuild from sidecars
Recovery if catalog.db is lost
Portable folder workflows
Conflict detection
```

### Rebuild precedence

When rebuilding a catalog from sidecars:

```txt
1. sidecar.flags wins if present and valid.
2. edit_graph.metadata is fallback.
3. missing flags default to rating 0, picked false, rejected false, color_label null.
```

This prevents Codex from inventing a separate meaning for `flags`.

---

## Catalog Photo Metadata

Task 11.7 records the local alpha metadata contract without adding a parser dependency.

- `photo_metadata` normalized fields: `width`, `height`, `orientation`, `capture_time`, `camera_make`, `camera_model`, and `lens_model`.
- `photos.file_size` and `photos.modified_at` are file-system metadata captured during import and are not duplicated into `photo_metadata`.
- Task 11.7.3 adds physical catalog columns for `width`, `height`, and `orientation`; JPEG/JPG import may store width and height from the existing raster path.
- Camera make, camera model, lens model, orientation, and EXIF capture time remain unavailable until a parser dependency is selected and documented in `docs/DEPENDENCIES.md`.
- `photo_metadata.raw_json` is parser-owned untrusted data and defaults to `{}`.
- Unsupported files must not receive fake metadata rows. Existing imports are not backfilled on library open or session restore.
- Metadata read APIs serialize each displayed field with an explicit `known`, `unknown`, or `unavailable` state and must not read original files during query.
- The first metadata-backed grid filter is `has_dimensions`, defined as stored `photo_metadata.width IS NOT NULL AND photo_metadata.height IS NOT NULL`. It must not infer dimensions from original files during query.

---

## Schema Versioning Policy

Current v0.1 schemas use:

```json
"version": { "const": 1 }
```

This is intentional.

When a breaking schema change occurs, create a new schema file instead of modifying v1 in place.

Recommended future naming:

```txt
schemas/edit_graph.v1.schema.json
schemas/edit_graph.v2.schema.json
schemas/sidecar.v1.schema.json
schemas/sidecar.v2.schema.json
```

Current compatibility aliases may remain:

```txt
schemas/edit_graph.schema.json -> current stable schema
schemas/sidecar.schema.json -> current stable schema
```

Migration rules:

```txt
- v1 edit graphs validate against version const 1.
- v2 edit graphs validate against a separate v2 schema.
- Migration code must explicitly convert v1 → v2.
- Never loosen v1 schema silently to accept v2 fields.
- Unknown experimental data belongs under `extensions`.
```

---

## Intentionally Loose Fields

Some schema fields are intentionally loose in v0.1 because their feature areas are later-stage or model-dependent.

These include:

```txt
detail.mlx_denoise
sidecar.edit_graph
plugin/model input/output details
MCP input_schema / output_schema
mask.source extra properties
extensions
```

Rules for Codex:

```txt
Do not fill these loose fields with invented final structures.
Do not hard-code MLX denoise structure before MLX feature implementation.
Do not create plugin/model/MCP sub-schemas unless the corresponding implementation task requires it.
When a loose field becomes implementation-critical, update the relevant schema and docs first.
```

This is intentional flexibility, not an invitation to invent hidden formats.

---

# v1.4 Clarification — sidecar.flags Scope

`sidecar.flags` is intentionally limited to portable culling and label state:

```txt
rating
picked
rejected
color_label
```

It intentionally excludes:

```txt
edited
exported
```

Reason:

```txt
edited is derived from edit graph state.
exported is local catalog/export-history state.
```

If a future schema needs to persist exported workflow history in sidecars, it must be added as a separate versioned section, not silently added to `sidecar.flags`.
