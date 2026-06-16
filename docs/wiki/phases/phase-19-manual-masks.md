---
title: Phase 19 Manual Masks
status: active
audience: all
updated: 2026-06-16
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Phase 19: Manual Masks

## Goal

Add manual masking in narrow slices without inventing hidden mask formats, weakening edit graph validation, mixing durable mask data with disposable render caches, or implying AI mask behavior before provenance and permissions exist.

Phase 19 is manual-first. AI masks remain deferred until mask provenance, action trust, permissions, and later MLX scope are explicitly ready.

## Entry State

- Phase 18 is complete.
- Develop controls already validate graph-owned edits, preserve originals, commit undoable history, and block unsupported runtime states honestly.
- `schemas/edit_graph.schema.json` already contains a `masks` array and a `mask` definition.
- Task 19.1 audited the mask schema and added explicit `masks[].geometry` for manual linear/radial masks before runtime mask writes exist.

## Task Order

| Task | Name | Gate |
| --- | --- | --- |
| 19.1 | Mask Schema and Edit Graph Audit | Complete: manual gradient geometry and provenance boundaries are schema-owned before behavior is added |
| 19.2 | Linear and Radial Manual Masks | Complete: manual gradient masks persist in the edit graph, preview on supported JPEG/JPG paths, and block export until compositing is implemented |
| 19.3 | Brush Mask Storage and Rasterization | Complete: brush data is durable in `masks[].brush`; raster artifacts are disposable `render-cache/masks/` cache files |
| 19.4 | Mask Compositing in Preview and Export | Active: committed masks apply consistently in preview and export or block honestly |
| 19.5 | Mask Editor Visual QA | Pending: mask editor UI matches design system and visual QA covers the active mask screen |

## Non-Goals

- No AI mask generation, subject/sky/background auto-selection, MLX model loading, plugin runtime, MCP runtime, cloud sync, telemetry, or auto-update.
- No broad RAW support claim beyond fixture-backed paths.
- No mask sidecar auto-write or hidden collection-wide mutation.
- No original photo mutation, original metadata mutation, or original overwrite export path.
- No disposable raster cache stored as the durable mask source of truth.

## Common Acceptance Rules

- Manual mask data must validate against `schemas/edit_graph.schema.json`.
- Manual mask provenance must remain distinct from future AI/MLX provenance.
- Manual gradient geometry must live in `masks[].geometry`, not `mask.source`.
- Manual brush strokes must live in `masks[].brush`, not `mask.source` or disposable cache files.
- Manual `mask.source` must remain provenance-only `{ "kind": "manual" }`.
- Durable edit graph mask data and disposable mask/render caches must remain separate.
- Draft mask preview must not write durable catalog state, sidecars, exports, action-log rows, or originals.
- Committed mask edits must use undoable catalog history.
- Preview and export semantics must agree before mask support is claimed.
- Unsupported mask combinations must be visibly blocked, not silently ignored or approximated.

## Validation Strategy

- Schema and graph audit: `cargo test -p silica-edit`.
- Runtime parity tasks: relevant `silica-render`, `silica-core`, `silica-export`, and storage tests.
- UI tasks: static UI smoke, visual QA, and full harness when completing a slice.
- Before phase completion: `scripts/harness/check.sh`.

## Stop Gates

Stop if:

- A task requires a mask schema change without documenting whether it is backward-compatible.
- Manual mask data cannot be represented without inventing undocumented fields.
- A preview path writes durable catalog, sidecar, export, or original state.
- Preview and export diverge for the same committed mask state.
- Disposable cache bytes become the durable source of truth for a mask.
- AI or MLX mask behavior becomes necessary to complete a manual-mask task.
- Any implementation can mutate, overwrite, move, or delete original photo files.
- A new dependency appears without `docs/DEPENDENCIES.md`.
