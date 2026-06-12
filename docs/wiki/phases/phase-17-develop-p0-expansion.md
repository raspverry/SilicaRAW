---
title: Phase 17 Develop P0 Expansion
status: active
audience: all
updated: 2026-06-13
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Phase 17: Develop P0 Expansion

## Goal

Complete the P0 Develop baseline for supported image paths without weakening Phase 16 trust guarantees.

Phase 17 expands basic controls beyond exposure/contrast. Work must stay vertical and testable by control family so edit graph validation, preview behavior, export behavior, and undo/history semantics can fail narrowly.

## Entry State

- Phase 16 is complete.
- Edit graph commits and culling flags are undoable catalog transactions.
- Draft preview updates must remain render-only.
- Sidecar writes remain explicit; history commits only mark stale sidecar status.
- RAW support remains fixture-limited and evidence-bound.

## Task Order

| Task | Name | Gate |
| --- | --- | --- |
| 17.1.1 | White Balance Mutators | Complete: edit graph validates temperature/tint changes and round-trips schema |
| 17.1.2 | Tone Recovery Mutators | Complete: edit graph validates highlights/shadows/whites/blacks and round-trips schema |
| 17.1.3 | Color Presence Mutators | Complete: edit graph validates vibrance/saturation and round-trips schema |
| 17.2.1 | White Balance Preview/Commit/Export Parity | Complete: WB draft/commit/export use the same validated edit state |
| 17.2.2 | Tone Recovery Preview/Commit/Export Parity | Complete: tone recovery draft/commit/export use the same validated edit state |
| 17.2.3 | Color Presence Preview/Commit/Export Parity | Complete: vibrance/saturation draft/commit/export use the same validated edit state |
| 17.3 | Real Histogram Cache and Display | Complete: histogram cache is disposable and honest about unsupported/missing states |
| 17.4 | Reset, Before/After, and Basic Presets | Complete: reset/preset changes are undoable; before/after is view-only |
| 17.5 | Develop P0 Visual QA | P0 Develop UI fits current visual QA widths without overlap |

## Non-Goals

- No tone curve, HSL, detail, lens, geometry, crop, rotate, masks, batch sync, MLX, plugin, or MCP behavior.
- No broad RAW support claim beyond existing fixture-backed surfaces.
- No sidecar auto-write or conflict UI.
- No hidden original-file mutation.
- No large fallback renderer stack.

## Validation Strategy

- Mutator tasks: `cargo test -p silica-edit`.
- Preview/export parity tasks: relevant `silica-render`, `silica-core`, `silica-export`, and desktop/static checks for the touched family.
- Histogram/reset/UI tasks: focused tests plus full harness before completion.
- Before phase completion: `scripts/harness/check.sh`.

## Stop Gates

Stop if:

- A control accepts values outside `schemas/edit_graph.schema.json`.
- Draft preview writes `edit_states`, `edit_history`, sidecars, exports, or action-log rows.
- Commit bypasses the Phase 16 history transaction path.
- Preview and export use divergent semantics for the same committed control.
- UI state claims support for unsupported RAW paths.
- Sidecar writes happen implicitly after edits.
