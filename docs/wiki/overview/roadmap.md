---
title: Roadmap Overview
status: active
audience: all
updated: 2026-06-13
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
19. Complete the product alpha runtime loop before clean-Mac install QA: real JPEG/JPG pixels, native/selectable paths, persisted UI readback, cache clear, fixture generation, and installed/runtime smoke.
20. Complete local DMG install, signing, notarization, GitHub Release, and release hardening.
21. Follow the post-alpha product roadmap and master execution plan for fixture-backed RAW, color, Metal, Library, Develop, masks, export, permissions, MLX, plugins, MCP, public beta, and v1.0.

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
- Phase 5.5 completed the UI MVP vertical slice as screen structure plus command wiring. It is not the final installed-app readiness gate because several surfaces still use placeholder pixels, typed paths, string command parsing, and static/demo state.
- Phase 5.6 is now the required Product Alpha Runtime Completion pass before Phase 6 clean-Mac install QA. Its first task records the runtime gap audit and narrows the installed-alpha guaranteed visible photo path to JPEG/JPG until additional codecs are explicitly implemented and tested.
- The local DMG distribution plan runs through Phase 9. After that, the post-alpha product roadmap continues with evidence and trust gates before broad RAW, Metal, Develop, MLX, plugin, or MCP work.
- Phase 10 has completed fixture manifest, golden tolerance policy, sidecar v1, rebuild dry-run, backup/restore boundaries, project license, contribution/security templates, and public trust regression checks.
- Phase 11 is complete: app session, real recents, relaunch restore, layout persistence, paged grid queries, grid interaction behavior, stored metadata display/filtering, structured import issues, opt-in recursive import, and connected runtime smoke.
- Phase 12 RAW proof and Phase 13 color proof are complete.
- Phase 14 product Metal viewer bridge proof is complete.
- Phase 15 task cards now route the RAW/color/Metal vertical slice.
- Task 15.0, the vertical slice evidence gate, is complete.
- Task 15.1, the decoded image handoff contract, is complete.
- Task 15.2, RAW decode to preview artifact, is complete.
- Task 15.3, Metal preview display, is complete.
- Task 15.4, exposure/contrast Metal draft path, is complete.
- Task 15.5, RAW-derived JPEG sRGB export, is complete.
- Task 15.6, RAW export manual color QA, is complete.
- Phase 15 is complete.
- Phase 16 is complete: action trust, semantics, edit history, undo/redo, Develop history panel data, append-only action log, and sidecar status after history commits are implemented.
- Phase 17 is complete. Tasks 18.1.1 through 18.2.1 are complete. Current task is Task 18.2.2 HSL Preview, Commit, and Export Parity.
- The [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md) is the execution router for Phase 14 through v1.0 so maintainers do not recreate phase-wide plans before each phase.

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
- [Product Alpha Runtime Completion](../topics/product-alpha-runtime-completion.md)
- [Post-Alpha Product Roadmap](../roadmaps/post-alpha-product-roadmap.md)
- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [LLM Routing Index](../llm/index.md)
- [Phase 11 Summary](../phases/phase-11-summary.md)
- [Phase 12 RAW Proof Brief](../phases/phase-12-raw-proof.md)
- [Phase 13 Color Pipeline Proof Brief](../phases/phase-13-color-pipeline-proof.md)
- [Phase 14 Product Metal Viewer Bridge Brief](../phases/phase-14-product-metal-viewer-bridge.md)
- [Phase 15 RAW Color Metal Vertical Slice Brief](../phases/phase-15-raw-color-metal-vertical-slice.md)
- [Public Trust](../topics/public-trust.md)

## Notes for LLM Agents

When choosing the next task, prefer the documented task order. Do not skip ahead to broad UI, RAW, Metal, MLX, plugin, or MCP implementation without explicit task scope.
