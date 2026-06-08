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

Current status: unresolved.

Why it matters: decoder choice affects camera support, input profile, color pipeline, metadata, Apple ProRAW behavior, and distribution complexity.

Next evidence needed: Spike 002 result selecting Core Image RAW primary, LibRaw primary, or hybrid.

### Color Pipeline Correctness

Current status: unresolved.

Why it matters: silent preview/export color errors are release-blocking trust failures.

Next evidence needed: color-managed preview/export spike with fixture class and reporting format.

### Storage and Migration Safety

Current status: planned, not implemented.

Why it matters: catalog corruption, lost edits, or original mutation would undermine trust immediately.

Next evidence needed: migration framework, original hash protection tests, sidecar read/write tests, and cache clear safety tests.

### Extension Permission Safety

Current status: later-stage.

Why it matters: plugins and MCP can create dangerous mutation paths if they bypass Core APIs or permissions.

Next evidence needed: manifest validation, permission layer, action log, and explicit confirmation behavior.

## Links

- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Spike 001: Tauri + Native Metal Viewer](../../spikes/001-tauri-metal-viewer.md)
- [System Architecture](../../03_System_Architecture.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Color Management](../topics/color-management.md)
- [Data Safety](../topics/data-safety.md)

## Notes for LLM Agents

When a risk says "unresolved" or "partially resolved," do not write code that assumes it is solved. Add a spike result, ADR, or explicit task-scoped note first.
