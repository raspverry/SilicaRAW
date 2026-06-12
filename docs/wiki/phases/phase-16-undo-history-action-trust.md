---
title: Phase 16 Undo History Action Trust
status: active
audience: agents
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/post-alpha-master-execution-plan.md
---

# Phase 16: Undo, History, and Action Trust

## Goal

Make non-destructive edit state trustworthy before adding more Develop controls, masks, export breadth, permissions, MLX, plugins, or MCP.

Phase 16 is a trust gate, not UI breadth. It defines and implements undoable state, durable history, transaction-safe undo/redo, real history UI data, append-only action logging, and sidecar sync status after history commits.

## Entry State

- Phase 15 is complete.
- RAW/color/Metal vertical slice evidence exists.
- Exposure/contrast commits already write validated active edit graphs.
- Draft preview updates must remain non-persistent.
- Original photo files remain immutable.

## Task Order

| Task | Name | Gate |
| --- | --- | --- |
| 16.0 | Phase 16 Design Gate | Complete: action classes, schema boundary, transaction policy, sidecar policy |
| 16.1 | Undo, History, and Action Semantics Contract | Complete: undoable vs logged-only vs irreversible classes documented |
| 16.2 | Edit History Persistence | Complete: validated checkpoints survive reopen; draft updates write no history |
| 16.3 | Undo/Redo Core Commands | Complete: undo/redo run as catalog transactions and never delete exports/originals |
| 16.4 | Develop History Panel Contract | UI lists real checkpoints only; no fake/demo rows |
| 16.5 | Append-Only Action Log | Core-only action log API for sensitive actions and future permissions |
| 16.6 | Sidecar Sync Status After History Commits | Sidecar status reflects committed history without silent conflict overwrite |

## Non-Goals

- No broad Develop P0 controls.
- No masks.
- No plugin, MCP, MLX, or extension runtime.
- No permission prompt UI beyond action-log groundwork.
- No export-file deletion through undo.
- No original-file mutation.
- No fake history rows or static demo history UI.

## Design Gate Result

Task 16.0 locks the Phase 16 trust boundary in [Action Trust](../topics/action-trust.md):

- edit graph commits and photo flag changes are undoable catalog transactions;
- export, sidecar write, import by reference, backup, and restore attempt records are logged-only;
- cache clear is non-reversible and must not be reconstructed by undo;
- originals, original-overwrite export paths, sidecar path escapes, and direct extension writes are blocked;
- sidecar status after history commits must not claim stale sidecars are clean.

Task 16.1 adds the action semantics contract in the same topic: `silica.action` payload version 1, per-photo checkpoint units, redo invalidation after new undoable checkpoints, explicit disabled undo/redo states, and draft-preview no-history behavior.

Task 16.2 implements the first durable edit checkpoints: catalog schema version 6, ordered per-photo `edit_history` rows, schema-valid before/after edit graph payloads, and tests proving draft preview updates still write no history rows.

Task 16.3 implements storage/core/desktop undo and redo commands for edit checkpoints and culling flags. Catalog schema version 7 adds `history_state`; redo is invalidated by new undoable actions; tests prove export outputs survive undo/redo.

Task 16.4 implements the first real Develop history panel contract. Storage/core expose photo history checkpoints from `edit_history`; desktop exposes `get_photo_history`; the static UI renders an empty list until runtime data arrives, and checkpoint row actions use only the documented undo/redo commands.

Task 16.5 implements the append-only action log groundwork. Catalog schema version 8 adds side-effect and evidence fields to `action_log`; storage and Core expose append/read APIs; Core logs import by reference, sidecar writes, JPEG exports, RAW-derived exports, and disposable cache clear without adding plugin, MCP, MLX, or permission UI runtime.

## Validation Strategy

- Task 16.0 and 16.1: docs/static checks and schema boundary review.
- Task 16.2: `cargo test -p silica-storage -p silica-core -p silica-edit`.
- Task 16.3: `cargo test -p silica-storage -p silica-core -p silica-desktop`.
- Task 16.4: `cargo test -p silica-storage -p silica-core -p silica-desktop`, `python3 scripts/harness/check-static-ui.py`, and full harness before completion.
- Task 16.5 and 16.6: `cargo test -p silica-storage -p silica-core` plus `scripts/harness/check.sh`.
- Before phase completion: `scripts/harness/check.sh`.

## Stop Gates

Stop if:

- Undo/redo can bypass catalog transactions.
- History rows can reference invalid edit graphs.
- Slider draft updates create history rows.
- Export undo deletes output files.
- Cache clear undo attempts to restore disposable cache bytes.
- Sidecar sync silently overwrites a newer sidecar or unresolved conflict.
- Any path can mutate original photo files.
- Extension-facing action logging bypasses Core APIs.

## Notes for LLM Agents

Create and execute one task card at a time. Do not begin Phase 17 controls until Phase 16 exits with durable undo/history/action-log evidence.
