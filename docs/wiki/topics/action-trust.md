---
title: Action Trust
status: active
audience: all
updated: 2026-06-17
source_of_truth: docs/wiki/phases/phase-16-undo-history-action-trust.md
---

# Action Trust

## Summary

Action trust defines which SilicaRAW operations are undoable, which are only logged, which are intentionally non-reversible, and which are blocked. The boundary exists so undo/redo cannot mutate originals, delete export files, erase action evidence, or bypass catalog transactions.

## Ownership

- `silica-edit` owns edit graph structure and validation.
- `silica-catalog` owns domain-facing catalog/action contracts and table expectations.
- `silica-storage` owns SQLite migrations, catalog transactions, and physical writes to catalog-owned tables.
- `silica-core` owns product command policy, action classification, and the only mutation API that desktop UI, future plugins, future MCP tools, and future MLX approval paths may use.
- Desktop UI owns presentation and user intent capture only. It must not own action semantics, raw SQL, or schema shape.

## Action Classes

| Class | Meaning | Phase 16 examples |
| --- | --- | --- |
| Undoable | A catalog state transition that can restore the previous catalog state through one Core command and one SQLite transaction. | Committed edit graph changes; rating, pick, reject, and color label changes. |
| Redoable | A previously undone undoable action that can be reapplied through the same transaction boundary. | Reapplying an edit checkpoint or flag change. |
| Logged-only | Durable action evidence that is not part of undo/redo history. | Export creation, sidecar write, import by reference, backup creation, restore attempt. |
| Non-reversible | A valid action whose external effect is intentionally not reconstructed by undo. | Cache clear and regenerated disposable preview/render artifacts. |
| Blocked | An attempted action that violates safety or scope and must not run. | Original-file mutation, export overwrite of an original, sidecar path escape, direct extension DB mutation. |

## Transaction Boundary

Undoable actions must cross Core as typed commands and must commit catalog state atomically through `silica-storage`.

Required boundary:

```txt
validate input
validate edit graph if present
open SQLite transaction
write current state table
write edit_history checkpoint when applicable
commit transaction
return durable result
```

If any step fails before commit, no partial catalog state is trusted. Draft preview updates are outside this boundary and must not create history rows.

External file effects are not inside undo/redo transactions. Export files, sidecar files, backup artifacts, and disposable cache bytes use their own explicit command policies. Their catalog records or status rows may be logged, but undo/redo must not delete external files.

## Operation Policy

| Operation | Class | Policy |
| --- | --- | --- |
| Exposure/contrast commit | Undoable and redoable | Store a validated edit graph checkpoint. Undo/redo changes active catalog edit state only. |
| Batch edit sync | Undoable and redoable per affected photo | Plan all targets first. If any target is blocked, write nothing. If all targets are ready or unchanged, commit ready targets in one catalog transaction with one edit checkpoint per affected photo. |
| P0 Basic reset or built-in preset apply | Undoable and redoable | Commit one validated full edit graph checkpoint through Core and storage. Do not split one preset into multiple history rows. |
| Before/after Develop view | View-only | Switch presentation state only. No edit state, history, action log, sidecar, export, or cache write. |
| Slider draft preview | Not persisted | Render-only request. No `edit_states`, `edit_history`, sidecar, export, or action-log write. |
| Rating, pick, reject, color label | Undoable and redoable | Change `photo_flags` in a catalog transaction and record action semantics. |
| Import by reference | Logged-only for Phase 16 | Adds catalog references and default state. Develop undo must not remove imported photos. A future explicit remove-from-catalog command may define its own policy. |
| Export JPEG sRGB | Logged-only | Export output and `exports` record remain evidence. Undoing an edit must not delete or rewrite export output. |
| Cache clear | Non-reversible and logged-only | Deletes only disposable cache directories and `cache_records`. Undo must not reconstruct cache bytes. |
| Sidecar write | Logged-only | Writes only library-local sidecars under `sidecars/`, then updates `sidecar_status` after successful validation/write. Undo must not delete sidecar files. |
| Sidecar conflict resolution | Blocked until explicit task | No silent overwrite of newer sidecars or unresolved conflicts. |
| AI result proposal | Logged/local result state, unapproved by default | Store local review/suggestion output in `ai_results` only. Do not write edit graph, edit history, or photo flags. Approval remains a later explicit user action. |
| AI blur review panel | View-only | Read stored local blur review rows and model-unavailable state only. Do not approve suggestions, write edit graph, write edit history, change photo flags, or touch originals. |
| Original photo mutation | Blocked | No undo class may touch, rewrite, move, or delete original referenced files. |
| Extension, plugin, or MCP mutation | Blocked until permission/action-log phases | Future mutations must enter through Core APIs and append-only action logging, never direct DB or file writes. |

## Action Semantics Contract

Task 16.1 defines the first action payload contract. Runtime tasks may add storage columns for query performance, but the semantic payload must stay typed and versioned.

Every undoable action checkpoint records:

```txt
schema: silica.action
version: 1
class: undoable
kind: edit_commit | flag_change
photo_id
label
before
after
created_by: core
```

`before` and `after` must be explicit enough to restore catalog state without reading external files. For edit commits, they reference schema-valid edit graph snapshots. For culling state, they contain rating, picked, rejected, and color_label values.

Logged-only action payloads record:

```txt
schema: silica.action
version: 1
class: logged_only
kind: export | import_reference | sidecar_write | backup | restore_attempt | cache_clear
subject
side_effect
evidence_ref
created_by: core
```

Logged-only entries prove what happened; they do not enter the undo/redo stack.

### Checkpoint Unit

One checkpoint is the smallest user-meaningful committed action:

- one release/commit of exposure and contrast together after a Develop slider gesture;
- one rating update;
- one picked update;
- one rejected update;
- one color label update.

Draft slider movement is not a checkpoint. It may render previews but writes no `edit_states`, `edit_history`, sidecar, export, or action-log row.

### Undo and Redo Rules

Undo selects the latest applied undoable checkpoint for the active photo and restores its `before` state in one catalog transaction.

Redo selects the earliest undone checkpoint still valid for the active photo and reapplies its `after` state in one catalog transaction.

A new undoable checkpoint after undo invalidates redo checkpoints for the same photo that were ahead of the restored state. Logged-only entries do not invalidate redo, but they also cannot be undone.

Undo/redo disabled states must be explicit:

- no library open;
- no active photo;
- no matching applied checkpoint for undo;
- no matching undone checkpoint for redo;
- checkpoint payload fails validation;
- checkpoint would touch an original, export output, sidecar conflict, or cache bytes.

Disabled undo/redo is a normal product state, not an error.

Task 16.3 implements the first runtime form of this contract for edit commits and culling flag changes. `edit_history.history_state` records `applied`, `undone`, or `invalidated`; undo selects the latest applied row for the photo, redo selects the earliest undone row for the photo, and both restore state in a SQLite transaction. Export records and export output files are not touched by these commands.

Task 16.4 exposes this state to the Develop history panel without adding a second mutation path. The panel lists only runtime `edit_history` checkpoints for the selected photo, hides invalidated redo rows, and enables selection only for the next valid undo or redo row. Row selection calls the same core undo/redo commands; it does not jump directly to arbitrary catalog states.

Task 16.5 adds the append-only action log runtime surface for sensitive local actions. Core and storage expose append/read APIs that require actor, action type, subject, timestamp, side-effect category, evidence reference, and JSON object payload context. Current Core flows log import by reference, sidecar write, JPEG export, RAW-derived JPEG export, and disposable cache clear. Task 23.3 extends this with permission grants, denials, plugin apply reviews, AI approvals, MCP reads, and permissioned export attempts through Core wrappers only. The log is evidence only: it does not make logged-only actions undoable, does not store active permission grants, and does not allow plugins, MCP, MLX, or UI code to write raw SQLite rows.

## Schema Boundary

Existing catalog tables already reserve the first Phase 16 surfaces:

- `edit_states`: current and prior validated edit graph snapshots.
- `edit_history`: ordered action/checkpoint records for undoable edit state.
- `photo_flags`: current culling state.
- `exports`: export evidence and output path records.
- `sidecar_status`: mirror/conflict state for library-local sidecars.
- `cache_records`: disposable cache metadata.
- `action_log`: append-only evidence for sensitive actions and future extension permissions.

Task 16.0 does not change migrations. Task 16.1 owns the typed action semantics contract before runtime behavior. Task 16.2 adds ordered edit history checkpoints. Task 16.3 adds transaction-safe undo/redo state via `history_state`. Task 16.4 adds a read-only history panel query over existing state and does not require a new migration. Task 16.5 adds schema version 8 for `action_log.side_effect_category`, `action_log.evidence_ref`, `idx_action_log_action_type_created_at`, and `idx_action_log_subject`. If a later task needs queryable columns that are not present, it must add them through `silica-storage` migrations and update `docs/DEPENDENCIES.md` only if dependencies change.

## Sidecar Status After History

Committed history changes can make a previously written sidecar stale. Phase 16 policy:

- A successful edit or flag history commit must not leave the system claiming a sidecar is clean if the sidecar no longer mirrors current catalog state.
- Sidecar sync status is catalog mirror state, not undo history.
- Undo/redo may mark sidecar sync status as needing write, but must not write or delete a sidecar as a hidden side effect.
- A sidecar write may mark status clean only after the sidecar payload and nested edit graph validate and the file write succeeds.
- Conflict status must not be cleared by undo, redo, import, export, or cache clear.

Task 16.6 implements the runtime path: edit commits, flag commits, undo, and redo mark clean sidecars as `catalog_newer` in the same catalog transaction and preserve `conflict` and `sidecar_newer`. Storage/Core expose `get_photo_sidecar_status` for status data. These commands do not write, delete, or overwrite sidecar files.

Task 17.4 applies P0 Basic reset and built-in presets through the same edit commit transaction path. Before/after controls are explicitly presentation state and must not enter the action tables.

Task 18.2.2 applies HSL color mixer commits through the same validated edit graph transaction path. Draft HSL preview remains render-only; committed HSL changes create one undoable catalog checkpoint and do not write sidecars, exports, cache bytes, or originals.

Task 18.5.2 adds batch edit sync for typed edit clipboard payloads. Partial commit is not allowed: blocked targets produce a deterministic plan/result and no catalog writes. Successful sync writes one `edit_commit` checkpoint per changed photo in one SQLite transaction, skips unchanged targets, marks clean sidecars stale, and does not write sidecar files, exports, cache bytes, or originals.

## Stop Gates

Stop before implementation if a design would:

- Let undo/redo bypass Core or SQLite transactions.
- Write history for draft preview updates.
- Store invalid edit graph JSON in a history checkpoint.
- Delete export outputs through undo.
- Rebuild deleted cache bytes through undo.
- Mark sidecars clean after stale history commits.
- Clear sidecar conflicts silently.
- Mutate original photo files.
- Let plugins, MCP tools, or MLX paths mutate catalog or files outside Core APIs.

## Links

- [Phase 16 Undo History Action Trust](../phases/phase-16-undo-history-action-trust.md)
- [Task 16.0 Phase 16 Design Gate](../tasks/16.0-phase-16-design-gate.md)
- [Catalog](catalog.md)
- [Data Safety](data-safety.md)
- [Edit Graph](edit-graph.md)
- [Schema Reference](../../19_Schema_Reference.md)

## Notes for LLM Agents

Do not expand undo into a general file-system rollback system. Phase 16 undo/redo is a catalog transaction feature for trusted product state, with external file effects handled only by explicit commands and append-only evidence.
