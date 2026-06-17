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

Phase 20 is complete. The current product area is **Phase 21: Preferences and App Settings**.

Task 21.3 is complete. The next implementation task is **Task 21.4: Color and Export Defaults**.

## Minimal Read Set

For Phase 21 task-card creation or implementation, read:

- The selected Phase 21 task card under [Task Cards](../tasks/index.md).
- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 21](../roadmaps/post-alpha-product-roadmap.md#phase-21-preferences-and-app-settings)
- [UI Mockups](../topics/ui-mockups.md)
- [Catalog](../topics/catalog.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

## Phase 21 Context

- Task 21.1 created the compact Preferences shell and section IA only.
- Task 21.2 implemented supported Appearance preferences in app-level session state.
- Task 21.3 implemented Library default path preferences and disposable Cache status/clear controls.
- Task 21.4 owns color and export default persistence.
- Task 21.5 is gated by Phase 23 permission policy.

## Stop Rules

- Do not enable an unimplemented Preferences control.
- Do not add Preferences persistence outside the active scoped task.
- Do not change export defaults before Task 21.4.
- Do not start agent, MCP, or plugin runtime from Preferences before Task 21.5 and Phase 23 policy.
- Do not add MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support.
- Do not mutate original photo files.
