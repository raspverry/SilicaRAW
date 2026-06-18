---
title: Current LLM Route
status: active
audience: agents
updated: 2026-06-18
source_of_truth: docs/wiki/llm/index.md
---

# Current LLM Route

## Summary

This is the shortest read path for agents. Read this page after [Agent Rules](../../../codex/AGENT_RULES.md) and the [Wiki Index](../index.md).

## Current Work Area

Phase 20, Phase 21, Task 22.1, Task 22.2, Task 22.3, Task 22.4, Task 22.5, Task 23.1, Task 23.2, Task 23.3, Task 24.1, Task 24.2, Task 24.3, Task 24.4, Task 24.5, Task 25.1, and Task 25.2 are complete. The next roadmap task is **Task 25.3: Plugin Permission Review and Action Log**.

Task 21.5 is complete as a disabled-by-default Preferences surface. Task 23.1 adds the core default-deny permission vocabulary. Task 23.2 adds the static permission prompt UI contract only. Task 23.3 connects permission decisions and future extension-sensitive actions to the append-only action log. Task 24.1 records the MLX runtime spike without enabling a runtime. Task 24.2 validates model manifests without loading models. Task 24.3 stores local AI results without mutating edit state. Task 24.4 adds read-only blur review presentation with model-unavailable behavior. Task 24.5 adds explicit approval/rejection for stored AI suggestions through the undoable edit history boundary. Task 25.1 validates plugin manifests and keeps plugins disabled by default. Task 25.2 adds data-only preset packs and explicit approval apply through edit history. MCP/plugin runtime and agent bridges remain unavailable.

## Minimal Read Set

For Task 25.3, read:

- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 25](../roadmaps/post-alpha-product-roadmap.md#phase-25-plugin-foundation)
- [Plugins and MCP](../topics/plugins-and-mcp.md)
- [Action Trust](../topics/action-trust.md)
- [Data Safety](../topics/data-safety.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Schema Reference](../../19_Schema_Reference.md)
- [Plugin Manifest Schema](../../../schemas/plugin_manifest.schema.json)
- [Plugin Preset Pack Schema](../../../schemas/plugin_preset_pack.schema.json)
- [Edit Graph Schema](../../../schemas/edit_graph.schema.json)
- [Task 25.2: Declarative Preset Plugin](../tasks/25.2-declarative-preset-plugin.md)
- [Task 25.1: Plugin Manifest Validation](../tasks/25.1-plugin-manifest-validation.md)
- [Task 24.5: Explicit AI Suggestion Approval](../tasks/24.5-explicit-ai-suggestion-approval.md)
- [Dependencies Policy](../../DEPENDENCIES.md) if adding or changing a dependency

## Phase 23 Context

- Task 23.1 defines `ExtensionPermission`, `ExtensionPermissionCategory`, default-deny `ExtensionPermissionPolicy`, and `McpMode` in `silica-core`.
- `silica-plugin` and `silica-mcp` record matching core permission IDs for boundary checks only and still start no runtime.
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
- Task 24.4 builds the first AI review panel from stored blur review results only. Missing models or missing stored results produce a model-unavailable/editor-usable state.
- Task 24.4 desktop UI presented AI review as information only; Task 24.5 adds the explicit approval controls for approvable stored suggestions.
- Task 24.5 approves only scoped stored suggestions after explicit user action. Approval creates an undoable edit checkpoint, records provenance, marks the AI result approved, and appends `ai_approval`; rejection appends `ai_rejection` and leaves edit state/history unchanged.
- MLX memory policy is bounded worker use under unified-memory pressure; cancellation is cooperative at task boundaries until proven otherwise.

## Phase 25 Context

- Plugin work starts with manifest validation only.
- Plugins remain disabled by default.
- Task 25.1 validates `silica.plugin` v1 manifests, rejects missing trust fields and unsafe permissions, and returns `enabled_by_default = false`.
- Task 25.2 validates `silica.plugin_preset_pack` v1, supports P0 Basic data-only presets, and applies them only through explicit Core approval, edit graph validation, one undoable history checkpoint, and `plugin_apply` action-log evidence.
- Task 25.3 may add permission review UI/logging around plugin enable/apply decisions, but must not add executable plugin runtime or direct SQLite access.
- Plugin manifests cannot request raw SQL, filesystem write, direct database access, or original-file mutation permission.

## Stop Rules

- Do not treat visual QA screenshots as product feature implementation.
- Do not add broad fallback systems for performance, migration, or profiling work.
- Do not start agent, MCP, or plugin runtime.
- Do not add MLX runtime, model loading, MCP runtime, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support unless the selected roadmap task explicitly requires it.
- Do not mutate original photo files.
