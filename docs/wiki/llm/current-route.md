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

Phase 20, Phase 21, Task 22.1, Task 22.2, Task 22.3, Task 22.4, Task 22.5, Task 23.1, Task 23.2, and Task 23.3 are complete. Phase 23 is complete. The next roadmap area is **Phase 24: MLX and AI Preview**, but do not start it unless a maintainer explicitly selects that task.

Task 21.5 is complete as a disabled-by-default Preferences surface. Task 23.1 adds the core default-deny permission vocabulary. Task 23.2 adds the static permission prompt UI contract only. Task 23.3 connects permission decisions and future extension-sensitive actions to the append-only action log. MCP/plugin runtime and agent bridges remain unavailable.

## Minimal Read Set

For any Phase 24 planning or implementation, read:

- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 24](../roadmaps/post-alpha-product-roadmap.md#phase-24-mlx-and-ai-preview)
- [MLX](../topics/mlx.md)
- [Plugins and MCP](../topics/plugins-and-mcp.md)
- [Action Trust](../topics/action-trust.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

## Phase 23 Context

- Task 23.1 defines `ExtensionPermission`, `ExtensionPermissionCategory`, default-deny `ExtensionPermissionPolicy`, and `McpMode` in `silica-core`.
- `silica-plugin` and `silica-mcp` record matching core permission IDs for boundary checks only and still have no runtime dependencies.
- Task 23.2 defines the static prompt contract in Preferences Advanced: actor, requested permission, side effects, confirmation, undo availability, deny behavior, and dangerous-permission unavailability.
- Task 23.3 records permission grants, denials, plugin apply reviews, AI approvals, MCP reads, and permissioned export attempts through Core action-log wrappers. Storage rejects extension raw-SQL/direct-database bypass claims.
- There is no raw SQL permission, no original-mutation permission, no permission persistence, and no runtime server or plugin execution.

## Stop Rules

- Do not treat visual QA screenshots as product feature implementation.
- Do not add broad fallback systems for performance, migration, or profiling work.
- Do not start agent, MCP, or plugin runtime.
- Do not add MLX runtime, model loading, MCP runtime, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support unless the selected roadmap task explicitly requires it.
- Do not mutate original photo files.
