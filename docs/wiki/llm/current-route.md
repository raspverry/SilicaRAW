---
title: Current LLM Route
status: active
audience: agents
updated: 2026-06-17
source_of_truth: docs/wiki/llm/index.md
---

# Current LLM Route

## Summary

This is the shortest read path for agents. Read this page after [Agent Rules](../../../codex/AGENT_RULES.md) and the [Wiki Index](../index.md).

## Current Work Area

Phase 20, Phase 21, Task 22.1, Task 22.2, Task 22.3, Task 22.4, and Task 22.5 are complete. The next product area is **Phase 23: Permission and Audit Foundation**.

Task 21.5 is complete as a disabled-by-default Preferences surface. Runtime permission policy, prompts, MCP/plugin runtime, and agent bridges remain gated by Phase 23.

## Minimal Read Set

For Phase 22 task-card creation or implementation, read:

- The selected Phase 22 task card under [Task Cards](../tasks/index.md), if it exists.
- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 22](../roadmaps/post-alpha-product-roadmap.md#phase-22-performance-migration-and-visual-hardening)
- [UI Visual and Responsive QA](../topics/ui-visual-responsive-qa.md)
- [Catalog](../topics/catalog.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

## Phase 22 Context

- Task 22.1 expanded final visual QA to 22 surfaces across compact, desktop, and large widths.
- Task 22.2 added local 1k, 10k, and 50k catalog benchmark evidence without marketing it as universal performance guarantees.
- Task 22.3 added corrupt backup restore, staging cleanup, existing-target preservation, rollback timing, and restored cache clear/re-record tests.
- Task 22.4 added local RAW/Metal-adjacent profiling evidence across decode, render, UI-latency, and export categories without implementing broad RAW decode or product Metal pixel rendering.
- Task 22.5 added a manual photographer QA checklist using licensed or user-provided local assets and records known limitations instead of expanding product claims.

## Stop Rules

- Do not treat visual QA screenshots as product feature implementation.
- Do not add broad fallback systems for performance, migration, or profiling work.
- Do not start agent, MCP, or plugin runtime before Phase 23 policy.
- Do not add MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support.
- Do not mutate original photo files.
