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

Phase 19 is complete. The current product area is **Phase 20: Export and Delivery Expansion**.

Tasks 20.1 and 20.2 are complete. The current next task is Task 20.3 Export Metadata Policy.

## Minimal Read Set

For Phase 20 task-card creation or implementation, read:

- The selected Phase 20 task card under [Task Cards](../tasks/index.md).
- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 20](../roadmaps/post-alpha-product-roadmap.md#phase-20-export-and-delivery-expansion)
- [Edit Graph](../topics/edit-graph.md)
- [Catalog](../topics/catalog.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

## Phase 19 Context

Use [Phase 19 Manual Masks](../phases/phase-19-manual-masks.md) for completed mask context. Do not read every Phase 19 task card unless the task directly changes completed mask behavior.

## Stop Rules

- Do not mix export settings into edit graph state.
- Do not add metadata controls, batch export, or Display P3 PNG/TIFF behavior before their scoped tasks.
- Do not add MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support.
- Do not mutate original photo files.
