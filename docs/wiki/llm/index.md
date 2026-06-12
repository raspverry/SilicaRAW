---
title: LLM Routing Index
status: active
audience: agents
updated: 2026-06-12
source_of_truth: docs/wiki/index.md
---

# LLM Routing Index

## Summary

This page is the short routing layer for LLM agents. Use it to choose the smallest useful read set before working on SilicaRAW.

## Always Read First

- [Agent Rules](../../../codex/AGENT_RULES.md)
- [Wiki Index](../index.md)
- This page

If the task changes schemas, dependencies, architecture, release behavior, or product scope, also read the specific source-of-truth document linked from the relevant route below.

## Current Route

Phase 14 is complete. The current planned product area is Phase 15: RAW, Color, and Metal Vertical Slice.

For Phase 14 through v1.0 sequencing, read [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md) once before choosing work. It prevents repeated phase-wide replanning and records the dependency graph, stop gates, and future task splits.

For Phase 15 work, read:

- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Phase 15 Brief](../phases/phase-15-raw-color-metal-vertical-slice.md)
- The matching task card under [Task Cards](../tasks/index.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Color Management](../topics/color-management.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Metal Render Pipeline Specification](../../08_Metal_Render_Pipeline_Specification.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

For Phase 14 historical native viewer bridge context, read:

- [Phase 14 Product Metal Viewer Bridge Plan](../roadmaps/phase-14-metal-viewer-bridge-plan.md)
- [Phase 14 Brief](../phases/phase-14-product-metal-viewer-bridge.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Spike 001 Tauri + Native Metal Viewer](../../spikes/001-tauri-metal-viewer.md)

For Phase 13 historical color-proof context, read:

- [Phase 13 Color Pipeline Proof Plan](../roadmaps/phase-13-color-pipeline-proof-plan.md)
- [Phase 13 Brief](../phases/phase-13-color-pipeline-proof.md)
- The matching task card under [Task Cards](../tasks/index.md)
- [Color Management](../topics/color-management.md)
- [Golden Image and Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

For Phase 12 historical context, read:

- [Phase 12 RAW Proof Plan](../roadmaps/phase-12-raw-proof-plan.md)
- [Phase 12 Brief](../phases/phase-12-raw-proof.md)
- [RAW Decoding](../topics/raw-decoding.md)

Do not read the full [Post-Alpha Product Roadmap](../roadmaps/post-alpha-product-roadmap.md) unless the current phase brief, task card, or master execution plan is missing required information.

## Completed Context

- Phase 11 is complete. For context, read [Phase 11 Summary](../phases/phase-11-summary.md) instead of the full Phase 11 design spec.
- Phase 10 evidence, recovery, and public trust gates are complete enough for Phase 12 to begin. Read the full Phase 10 spec only when changing fixtures, backup/restore, public trust files, or recovery policy.

## Read Avoidance Rules

- Prefer phase briefs over full roadmaps.
- Prefer task cards over phase briefs when the task is already selected.
- Prefer the master execution plan over new phase-wide planning when choosing Phase 14+ order.
- Prefer topic pages for durable facts.
- Use numbered docs and schemas only when the task touches their source-of-truth area.
- Do not use `docs/archive/` for implementation.

## Stop Gates

Stop and report before proceeding if a task would:

- Modify original photo files.
- Add RAW product pixels before fixture-backed proof exists.
- Add LibRaw or another decoder dependency without updating `docs/DEPENDENCIES.md`.
- Add MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, Homebrew, or Mac App Store scope.
- Treat a wiki summary as permission to bypass schemas or agent rules.

## Notes for LLM Agents

This page is a routing index, not a replacement for source-of-truth files. Read the smallest linked set that can answer the task, then verify with the relevant harness.
