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
| 16.0 | Phase 16 Design Gate | Action classes, schema boundary, transaction policy, sidecar policy |
| 16.1 | Undo, History, and Action Semantics Contract | Undoable vs logged-only vs irreversible classes documented |
| 16.2 | Edit History Persistence | Validated checkpoints survive reopen; draft updates write no history |
| 16.3 | Undo/Redo Core Commands | Undo/redo run as catalog transactions and never delete exports/originals |
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

## Validation Strategy

- Task 16.0 and 16.1: docs/static checks and schema boundary review.
- Task 16.2: `cargo test -p silica-storage -p silica-core -p silica-edit`.
- Task 16.3: `cargo test -p silica-storage -p silica-core -p silica-desktop`.
- Task 16.4: static UI smoke plus visual QA only after real history data exists.
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
