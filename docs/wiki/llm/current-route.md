---
title: Current LLM Route
status: active
audience: agents
updated: 2026-06-16
source_of_truth: docs/wiki/llm/index.md
---

# Current LLM Route

## Summary

This is the shortest read path for agents. Read this page after [Agent Rules](../../../codex/AGENT_RULES.md) and the [Wiki Index](../index.md).

## Current Work Area

Phase 18 is complete. The current product area is **Phase 19: Manual Masks**.

Task 19.1 is complete. The current next task is [Task 19.2 Linear and Radial Manual Masks](../tasks/19.2-linear-radial-manual-masks.md).

## Minimal Read Set

For Phase 19 task-card creation or implementation, read:

- [Phase 19 Brief](../phases/phase-19-manual-masks.md)
- The selected Phase 19 task card under [Task Cards](../tasks/index.md)
- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 19](../roadmaps/post-alpha-product-roadmap.md#phase-19-masks-and-local-mask-pipeline)
- [Edit Graph](../topics/edit-graph.md)
- [Catalog](../topics/catalog.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

## Phase 18 Context

Use [Phase 18 Summary](../phases/phase-18-summary.md) for historical context. Do not read every Phase 18 task card unless the task directly changes completed Phase 18 behavior.

## Stop Rules

- Do not invent mask schema fields outside the authoritative edit graph schema.
- Do not add MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support.
- Do not mutate original photo files.
