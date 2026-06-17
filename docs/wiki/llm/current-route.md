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

Phase 20, Phase 21, Task 22.1, Task 22.2, Task 22.3, Task 22.4, Task 22.5, Task 23.1, Task 23.2, Task 23.3, Task 24.1, Task 24.2, and Task 24.3 are complete. The next roadmap task is **Task 24.4: First Non-Mutating AI Review Feature**.

Task 21.5 is complete as a disabled-by-default Preferences surface. Task 23.1 adds the core default-deny permission vocabulary. Task 23.2 adds the static permission prompt UI contract only. Task 23.3 connects permission decisions and future extension-sensitive actions to the append-only action log. Task 24.1 records the MLX runtime spike without enabling a runtime. Task 24.2 validates model manifests without loading models. Task 24.3 stores local AI results without mutating edit state. MCP/plugin runtime and agent bridges remain unavailable.

## Minimal Read Set

For Task 24.4, read:

- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 24](../roadmaps/post-alpha-product-roadmap.md#phase-24-mlx-and-ai-preview)
- [MLX](../topics/mlx.md)
- [Plugins and MCP](../topics/plugins-and-mcp.md)
- [Action Trust](../topics/action-trust.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Schema Reference](../../19_Schema_Reference.md)
- [Model Manifest Schema](../../../schemas/model_manifest.schema.json)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

## Phase 23 Context

- Task 23.1 defines `ExtensionPermission`, `ExtensionPermissionCategory`, default-deny `ExtensionPermissionPolicy`, and `McpMode` in `silica-core`.
- `silica-plugin` and `silica-mcp` record matching core permission IDs for boundary checks only and still have no runtime dependencies.
- Task 23.2 defines the static prompt contract in Preferences Advanced: actor, requested permission, side effects, confirmation, undo availability, deny behavior, and dangerous-permission unavailability.
- Task 23.3 records permission grants, denials, plugin apply reviews, AI approvals, MCP reads, and permissioned export attempts through Core action-log wrappers. Storage rejects extension raw-SQL/direct-database bypass claims.
- There is no raw SQL permission, no original-mutation permission, no permission persistence, and no runtime server or plugin execution.

## Phase 24 Context

- Task 24.1 records ADR 0009: provisional future MLX C API bridge behind a non-default Rust feature gate.
- Task 24.1 does not add MLX dependency, model loading, inference, model assets, background workers, or AI UI.
- No manifest or no model means AI features are unavailable while the core editor remains usable.
- No model can be bundled or enabled without license, source, hash, preprocessing, and output metadata in a model manifest.
- Task 24.2 validates `silica.model` v1 manifests, rejects missing license/source/hash/preprocessing/output metadata, and compares deterministic `sha256:` model hashes against candidate bytes.
- Model validation does not load a model, run inference, create a worker, or require AI.
- Task 24.3 stores `silica.ai_result` v1 rows in `ai_results`, unapproved by default, with `local_only = true` and `permission_id = ai_result:propose`.
- Task 24.3 rejects AI result payloads that directly carry edit graph or photo flag mutation keys.
- AI result storage/read is local-only through Core and Storage APIs; it does not load models, run inference, approve suggestions, or write edit history.
- MLX memory policy is bounded worker use under unified-memory pressure; cancellation is cooperative at task boundaries until proven otherwise.

## Stop Rules

- Do not treat visual QA screenshots as product feature implementation.
- Do not add broad fallback systems for performance, migration, or profiling work.
- Do not start agent, MCP, or plugin runtime.
- Do not add MLX runtime, model loading, MCP runtime, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support unless the selected roadmap task explicitly requires it. Task 24.4 allows a non-mutating review surface only.
- Do not mutate original photo files.
