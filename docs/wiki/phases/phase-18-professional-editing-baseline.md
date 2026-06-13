---
title: Phase 18 Professional Editing Baseline
status: active
audience: all
updated: 2026-06-13
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Phase 18: Professional Editing Baseline

## Goal

Add P1 professional Develop tools in narrow vertical slices without weakening edit graph validation, undo/history semantics, preview/export parity, or original-file safety.

Phase 18 turns the schema-owned `tone`, `color`, `detail`, `lens`, and `geometry` sections into product behavior. Each family starts with graph validation, then runtime parity, then UI/QA.

## Entry State

- Phase 17 is complete.
- P0 Basic controls have validated graph mutators, draft preview, committed edit state, undo history, export evidence, histogram display, reset, presets, and visual QA.
- Draft preview updates must remain render-only.
- Committed edits must create undoable catalog checkpoints.
- Sidecar writes remain explicit; edit commits only mark stale sidecar status.
- RAW behavior remains fixture-backed and evidence-limited.

## Task Order

| Task | Name | Gate |
| --- | --- | --- |
| 18.1.1 | Tone Curve Mutators | Complete: edit graph validates RGB/channel curve changes and round-trips schema |
| 18.1.2 | Tone Curve Preview/Commit/Export Parity | Complete: supported JPEG/JPG preview/export paths use point tone curve semantics |
| 18.1.3 | Tone Curve Panel UI and QA | Complete: Develop panel exposes supported RGB point curve control without overlap |
| 18.2.1 | HSL Color Mixer Mutators | Complete: per-channel hue/saturation/luminance graph changes validate and round-trip |
| 18.2.2 | HSL Preview/Commit/Export Parity | Complete: supported JPEG/JPG preview/export paths use committed HSL state |
| 18.2.3 | HSL Panel UI and QA | Complete: compact color mixer UI follows existing Develop patterns and visual QA covers seeded HSL state |
| 18.3.1 | Detail Mutators | Complete: sharpening and non-MLX noise reduction graph values validate; MLX denoise remains untouched |
| 18.3.2 | Detail Preview/Export Boundary | Supported detail behavior is explicit; unsupported preview/export states are honest |
| 18.3.3 | Detail Panel UI and QA | Detail controls fit Develop layouts and do not imply MLX support |
| 18.4.1 | Lens and Geometry Mutators | Lens, transform, crop, rotate, and flip graph values validate |
| 18.4.2 | Geometry Preview/Export Parity | Non-destructive crop/rotate/geometry paths preserve originals and export semantics |
| 18.4.3 | Lens Geometry Panel UI and QA | UI exposes supported geometry controls with blocked states where needed |
| 18.5.1 | Edit Clipboard Contract | Copy/paste subsets are schema-owned and user-selectable |
| 18.5.2 | Batch Sync History | Batch sync writes one undoable checkpoint per affected photo |
| 18.5.3 | Copy Paste Batch UI and QA | UI exposes copy/paste/sync without hidden broad batch mutation |

## Non-Goals

- No masks, AI masks, MLX denoise runtime, plugin runtime, MCP server, cloud sync, telemetry, or auto-update.
- No automatic sidecar writes or sidecar conflict UI.
- No broad RAW support claim beyond fixture-backed paths.
- No camera/lens profile database dependency.
- No large fallback renderer stack.
- No editing of original photo files or original metadata.

## Common Acceptance Rules

- Every control family validates against `schemas/edit_graph.schema.json`.
- Draft preview must not write `edit_states`, `edit_history`, sidecars, exports, action-log rows, or originals.
- Commit must write one schema-valid undoable history checkpoint per edited photo.
- Export must use committed state only, record evidence where the current export model supports it, and block unsupported states honestly.
- UI must use existing Develop design tokens and responsive layout patterns.
- Unsupported renderer/export paths must be visible as unsupported or blocked, not silently approximated.
- Original-file hash preservation must remain covered for any task that touches preview, export, or batch mutation behavior.

## Common Slice Contract

Every Phase 18 task card must keep these categories explicit:

- **Edit graph validation:** use schema-owned fields only and reject invalid, non-finite, or unsupported values before runtime mutation.
- **Preview support:** drafts are render-only and may be disabled when a renderer cannot apply the edit honestly.
- **Export support:** export is supported only when the committed edit semantics are applied; silently ignoring an active committed control is a blocker.
- **Unsupported states:** unsupported combinations must return disabled, blocked, or explicit unsupported states.
- **Undo behavior:** committed edits restore catalog state only; undo must not delete export output, restore cache bytes, write sidecars, or touch originals.
- **Original-file safety:** source/output canonical matches are rejected, source hashes remain unchanged, and caches stay library-local and disposable.

## Validation Strategy

- Graph mutator tasks: `cargo test -p silica-edit`.
- Runtime parity tasks: relevant `silica-render`, `silica-core`, `silica-export`, and desktop tests.
- UI tasks: static UI smoke, visual QA, and full harness when completing the slice.
- Before phase completion: `scripts/harness/check.sh`.

## Stop Gates

Stop if:

- A task needs a schema change that is not explicitly planned.
- A draft preview writes durable catalog or sidecar state.
- Commit bypasses the existing Phase 16 history transaction path.
- Preview and export use divergent semantics for the same committed edit.
- Unsupported paths are represented as successful edits.
- Any implementation can mutate, overwrite, move, or delete original photo files.
- A new dependency appears without `docs/DEPENDENCIES.md`.
