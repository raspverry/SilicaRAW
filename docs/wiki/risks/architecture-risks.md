---
title: Architecture Risks
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/20_v1_1_Architecture_Patch.md
---

# Architecture Risks

## Summary

SilicaRAW's architecture is promising but gated by several technical decisions that need evidence before broad implementation.

## Risks

### Tauri + Metal Bridge

Current status: partially resolved by Spike 001 Path B.

Why it matters: Metal-first editing is central to the product identity. If Tauri cannot host or coordinate a native Metal viewer cleanly, the shell strategy must change.

Next evidence needed: dedicated AppKit/Metal viewer bridge contract, including layout reservation, lifecycle ownership, and ownership boundaries between web controls and native viewer input.

### RAW Decoder Path

Current status: partially resolved by Spike 002. Core Image RAW is the first implementation target; LibRaw is deferred until fixture evidence proves a gap.

Why it matters: decoder choice affects camera support, input profile, color pipeline, metadata, Apple ProRAW behavior, and distribution complexity.

Next evidence needed: legal RAW fixture manifest and a macOS-only Core Image probe that records success/failure by fixture class.

### Color Pipeline Correctness

Current status: partially resolved by Spike 003. Core Image/ColorSync-compatible color management and a linear Display P3 working-space recommendation are selected, but fixture proof is missing.

Why it matters: silent preview/export color errors are release-blocking trust failures.

Next evidence needed: tagged raster fixtures, ICC embedding proof, Preview.app comparison, and golden-image tolerance policy.

### Storage and Migration Safety

Current status: partially resolved by Spike 004. `rusqlite` with bundled SQLite and embedded SQL migrations can create and upgrade an empty catalog with required indexes.

Why it matters: catalog corruption, lost edits, or original mutation would undermine trust immediately.

Next evidence needed: file-backed library create/open APIs, WAL backup/checkpoint policy, original hash protection tests, sidecar read/write tests, and cache clear safety tests.

### MLX Runtime and Model Safety

Current status: deferred from local alpha by ADR 0005.

Why it matters: MLX can add model-license, memory-pressure, scheduling, cache, and user-approval risks if added before the editor core is stable.

Next evidence needed: later MLX runtime spike covering selected binding, model licensing, preprocessing/output contracts, memory pressure, cancellation, and user approval flow.

### Extension Permission Safety

Current status: later-stage.

Why it matters: plugins and MCP can create dangerous mutation paths if they bypass Core APIs or permissions.

Next evidence needed: manifest validation, permission layer, action log, and explicit confirmation behavior.

## Links

- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Spike 001: Tauri + Native Metal Viewer](../../spikes/001-tauri-metal-viewer.md)
- [Spike 002: RAW Decoder Path](../../spikes/002-raw-decoder.md)
- [Spike 003: Color-Managed Preview and Export](../../spikes/003-color-managed-preview-export.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [ADR 0005: Defer MLX from Local Alpha](../decisions/adr-0005-mlx-deferral-for-local-alpha.md)
- [System Architecture](../../03_System_Architecture.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Color Management](../topics/color-management.md)
- [Data Safety](../topics/data-safety.md)

## Notes for LLM Agents

When a risk says "unresolved" or "partially resolved," do not write code that assumes it is solved. Add a spike result, ADR, or explicit task-scoped note first.
