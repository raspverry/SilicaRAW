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

Phase 20 and Phase 21 are complete. The current product area is **Phase 22: Performance, Migration, and Visual Hardening**.

Task 21.5 is complete as a disabled-by-default Preferences surface. Runtime permission policy, prompts, MCP/plugin runtime, and agent bridges remain gated by Phase 23.

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
- Task 21.4 implements Color and Export defaults through the existing catalog export settings path.
- Task 21.5 completed disabled Advanced controls and explanatory permission text only; Phase 23 still owns runtime permission policy.

## Stop Rules

- Do not enable an unimplemented Preferences control.
- Do not add Preferences persistence outside the active scoped task.
- Do not create a second export preferences store; reuse catalog export settings.
- Do not start agent, MCP, or plugin runtime from Preferences before Phase 23 policy.
- Do not add MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support.
- Do not mutate original photo files.
