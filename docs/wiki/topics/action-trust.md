---
title: Action Trust
status: active
audience: all
updated: 2026-06-12
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
| Slider draft preview | Not persisted | Render-only request. No `edit_states`, `edit_history`, sidecar, export, or action-log write. |
| Rating, pick, reject, color label | Undoable and redoable | Change `photo_flags` in a catalog transaction and record action semantics. |
| Import by reference | Logged-only for Phase 16 | Adds catalog references and default state. Develop undo must not remove imported photos. A future explicit remove-from-catalog command may define its own policy. |
| Export JPEG sRGB | Logged-only | Export output and `exports` record remain evidence. Undoing an edit must not delete or rewrite export output. |
| Cache clear | Non-reversible and logged-only | Deletes only disposable cache directories and `cache_records`. Undo must not reconstruct cache bytes. |
| Sidecar write | Logged-only | Writes only library-local sidecars under `sidecars/`, then updates `sidecar_status` after successful validation/write. Undo must not delete sidecar files. |
| Sidecar conflict resolution | Blocked until explicit task | No silent overwrite of newer sidecars or unresolved conflicts. |
| Original photo mutation | Blocked | No undo class may touch, rewrite, move, or delete original referenced files. |
| Extension, plugin, or MCP mutation | Blocked until permission/action-log phases | Future mutations must enter through Core APIs and append-only action logging, never direct DB or file writes. |

## Schema Boundary

Existing catalog tables already reserve the first Phase 16 surfaces:

- `edit_states`: current and prior validated edit graph snapshots.
- `edit_history`: ordered action/checkpoint records for undoable edit state.
- `photo_flags`: current culling state.
- `exports`: export evidence and output path records.
- `sidecar_status`: mirror/conflict state for library-local sidecars.
- `cache_records`: disposable cache metadata.
- `action_log`: append-only evidence for sensitive actions and future extension permissions.

Task 16.0 does not change migrations. Task 16.1 owns the typed action semantics contract before Task 16.2 and Task 16.3 add or adjust runtime behavior. If a later task needs queryable columns that are not present, it must add them through `silica-storage` migrations and update `docs/DEPENDENCIES.md` only if dependencies change.

## Sidecar Status After History

Committed history changes can make a previously written sidecar stale. Phase 16 policy:

- A successful edit or flag history commit must not leave the system claiming a sidecar is clean if the sidecar no longer mirrors current catalog state.
- Sidecar sync status is catalog mirror state, not undo history.
- Undo/redo may mark sidecar sync status as needing write, but must not write or delete a sidecar as a hidden side effect.
- A sidecar write may mark status clean only after the sidecar payload and nested edit graph validate and the file write succeeds.
- Conflict status must not be cleared by undo, redo, import, export, or cache clear.

Task 16.6 owns the exact runtime update path for these rules.

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
