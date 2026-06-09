---
title: Roadmap Overview
status: active
audience: all
updated: 2026-06-09
source_of_truth: docs/13_Development_Roadmap.md
---

# Roadmap Overview

## Summary

SilicaRAW should kill technical risk before adding product breadth. The early roadmap is dominated by feasibility spikes, repository foundation, guardrails, and schema-backed storage decisions.

## Current Sequence

The current implementation order starts with:

1. Create monorepo structure.
2. Add Tauri desktop shell.
3. Add CI, formatting, linting, and test baseline.
4. Add architecture guardrails.
5. Spike Tauri + Metal viewer.
6. Spike RAW decoder comparison.
7. Spike color-managed preview/export.
8. Spike SQLite catalog persistence.
9. Decide MLX deferral for local DMG alpha.
10. Implement catalog migration foundation.
11. Implement local library create/open.
12. Implement folder import scanner.
13. Implement rating, pick, reject, and color label persistence.
14. Implement minimal preview path contract.
15. Implement edit graph types and validation.
16. Implement exposure and contrast edit flow.
17. Implement JPEG sRGB export.
18. Establish UI MVP baseline and then implement the connected UI vertical slice.

## Gate Logic

- Gate A: architecture viability.
- Gate B: editor viability.
- Gate C: color and trust viability.
- Public beta: final license, dependency inventory, sample asset license manifest, and data-safety confidence.

## Current Status

- Task 0101, monorepo foundation, has been implemented as a placeholder Rust workspace.
- Spike 001 recorded Path B: Tauri remains viable, but native viewer work needs a dedicated AppKit/Metal bridge before product UI depends on it.
- Spike 002 recorded Path A: Core Image RAW primary, with LibRaw deferred until legal fixtures prove a camera-support gap.
- RAW decoder-dependent work remains tagged as `decoder-blocking` until real fixture-backed decoding exists.
- Spike 003 recorded Path B: color-management direction is selected, but tagged color fixtures are still required before color correctness claims.
- Spike 004 recorded Path A: `rusqlite` with bundled SQLite and embedded SQL migrations can create and upgrade an empty catalog with required indexes.
- ADR 0005 defers MLX from local alpha; `silica-mlx` remains a dependency-free boundary crate.
- Phase 4.1 records the local alpha catalog schema contract in `silica-catalog` and verifies `silica-storage` migrations against that contract.
- Phase 4.2 adds local library create/open through `silica-core`, `silica-storage`, and the minimal Tauri shell.
- Phase 4.3 adds non-recursive folder import scanning with path, partial hash, unsupported-state, and original-preservation tests.
- Phase 4.4 adds catalog-authoritative rating, pick, reject, and color label persistence through `photo_flags`, core APIs, and minimal Tauri commands.
- Phase 5.1 adds a minimal preview readiness path: raster candidates can return ready-by-reference, unsupported entries return a clear state, and RAW entries remain blocked until fixture-backed Core Image probe work.
- Phase 5.2 adds typed edit graph structures and schema-aware validation in `silica-edit`; edit application, render integration, sidecar persistence, and UI controls remain later tasks.
- Phase 5.3 adds command/API-level exposure and contrast edit flow: draft preview requests do not write SQLite, while commit persists the active edit graph.
- Phase 5.4 adds command/API-level JPEG sRGB export with original overwrite protection and catalog export records.
- Phase 5.5 starts the UI MVP vertical slice. Task 5.5.1 records the baseline and tokenizes the static shell; product screens begin in Task 5.5.2.

## Links

- [Development Roadmap](../../13_Development_Roadmap.md)
- [Task Breakdown](../../14_Codex_Claude_Task_Breakdown.md)
- [Issue List](../../../github/ISSUE_LIST.md)
- [Architecture Risks](../risks/architecture-risks.md)
- [Spike 001: Tauri + Native Metal Viewer](../../spikes/001-tauri-metal-viewer.md)
- [Spike 002: RAW Decoder Path](../../spikes/002-raw-decoder.md)
- [Spike 003: Color-Managed Preview and Export](../../spikes/003-color-managed-preview-export.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [ADR 0005: Defer MLX from Local Alpha](../decisions/adr-0005-mlx-deferral-for-local-alpha.md)
- [Catalog](../topics/catalog.md)
- [UI MVP Baseline](../topics/ui-mvp-baseline.md)

## Notes for LLM Agents

When choosing the next task, prefer the documented task order. Do not skip ahead to broad UI, RAW, Metal, MLX, plugin, or MCP implementation without explicit task scope.
