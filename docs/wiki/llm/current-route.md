---
title: Current LLM Route
status: active
audience: agents
updated: 2026-07-08
source_of_truth: docs/wiki/llm/index.md
---

# Current LLM Route

## Summary

This is the shortest read path for agents. Read this page after [Agent Rules](../../../codex/AGENT_RULES.md) and the [Wiki Index](../index.md).

## Current Work Area

Phase 20, Phase 21, Task 22.1, Task 22.2, Task 22.3, Task 22.4, Task 22.5, Task 23.1, Task 23.2, Task 23.3, Task 24.1, Task 24.2, Task 24.3, Task 24.4, Task 24.5, Task 25.1, Task 25.2, Task 25.3, Task 26.1, Task 26.2, Task 26.3, Task 27.0, and Task 27.1 are complete. **Task 27.2: Public Beta Release Candidate is blocked** until signing/notarization prerequisites and clean-Mac downloaded-artifact QA are available.

Task 21.5 is complete as a disabled-by-default Preferences surface. Task 23.1 adds the core default-deny permission vocabulary. Task 23.2 adds the static permission prompt UI contract only. Task 23.3 connects permission decisions and future extension-sensitive actions to the append-only action log. Task 24.1 records the MLX runtime spike without enabling a runtime. Task 24.2 validates model manifests without loading models. Task 24.3 stores local AI results without mutating edit state. Task 24.4 adds read-only blur review presentation with model-unavailable behavior. Task 24.5 adds explicit approval/rejection for stored AI suggestions through the undoable edit history boundary. Task 25.1 validates plugin manifests and keeps plugins disabled by default. Task 25.2 adds data-only preset packs and explicit approval apply through edit history. Task 25.3 adds plugin permission review evidence without runtime or grant persistence. Task 26.1 selects disabled-by-default stdio-first MCP without starting a server. Task 26.2 defines read-only MCP tool manifests. Task 26.3 adds a runtime-free read-only MCP adapter function through Core APIs only. Task 27.0 freezes public beta scope and adds the public beta evidence index. Task 27.1 completes the readiness audit with a blocked verdict. MCP server startup, plugin runtime, and agent bridges remain unavailable.

While Task 27.2 is blocked, [Local Alpha Quality Closure Plan](../roadmaps/local-alpha-quality-closure-plan.md) is the active route before more product feature growth. It covers data-trust fixes, photo-first UI polish, interaction QA, installed app QA, and unsigned developer-preview DMG proof. [Blocked Public Beta UI Hardening Plan](../roadmaps/blocked-public-beta-ui-hardening-plan.md) remains an input for the UI shell and keyboard subset, not the top-level route.

## Minimal Read Set

For local alpha quality closure, Task 27.2, or any beta release-candidate work, read:

- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap: Phase 27](../roadmaps/post-alpha-product-roadmap.md#phase-27-public-beta-gate)
- [Local Alpha Quality Closure Plan](../roadmaps/local-alpha-quality-closure-plan.md)
- [Public Beta Evidence Index](../roadmaps/public-beta-evidence-index.md)
- [Public Beta Readiness Audit](../roadmaps/public-beta-readiness-audit.md)
- [Local Alpha Closure Evidence](../reports/local-alpha-closure-evidence.md)
- [Local Alpha Library Import Reference Evidence](../reports/local-alpha-library-import-reference.md)
- [Local Alpha Review and Edit Persistence Evidence](../reports/local-alpha-review-edit-persistence.md)
- [Local Alpha JPEG sRGB Export Evidence](../reports/local-alpha-jpeg-export-evidence.md)
- [Local Alpha Trust-State Evidence](../reports/local-alpha-trust-state-evidence.md)
- [Blocked Public Beta UI Hardening Plan](../roadmaps/blocked-public-beta-ui-hardening-plan.md)
- [Public Beta Scope Freeze Checklist](../../../checklists/PUBLIC_BETA_SCOPE_FREEZE.md)
- [Public Beta Readiness Audit Checklist](../../../checklists/PUBLIC_BETA_READINESS_AUDIT.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Completed LLM Context](completed-context.md)
- [Plugins and MCP](../topics/plugins-and-mcp.md)
- [Action Trust](../topics/action-trust.md)
- [Data Safety](../topics/data-safety.md)
- [Local DMG Release Runbook](../roadmaps/local-dmg-release-runbook.md)
- [Dependencies Policy](../../DEPENDENCIES.md)

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
- Task 25.3 validates plugin manifests for permission review, logs enable/apply decisions, handles denials without state mutation, and keeps runtime/grant persistence unavailable.
- Plugin manifests cannot request raw SQL, filesystem write, direct database access, or original-file mutation permission.

## Phase 26 Context

- Task 26.1 records ADR 0010: disabled-by-default stdio-first MCP with one process lifetime per session.
- Task 26.2 validates `silica.mcp_tool` v1 manifests for read-only tools only. Valid manifests use `mcp:read_only`, require empty side effects, require no confirmation, require no undo, and reject mutating tool IDs, unknown fields, and direct SQLite claims.
- Task 26.3 adds `run_read_only_mcp_tool` in `silica-mcp`. It validates manifests, dispatches read-only tools through `silica-core`, records `mcp_read` evidence, and keeps `silica-mcp` free of `silica-storage`, `rusqlite`, and catalog handles.
- MCP remains off by default. There is no background listener, persisted token, persisted grant, app-start server, or HTTP transport.
- Phase 26 starts read-only. Mutating MCP tools, permission self-escalation, direct SQLite, unrestricted filesystem, original mutation, plugin install, and plugin enable are out of scope.
- Future Streamable HTTP MCP transport requires a separate ADR before implementation.

## Phase 27 Context

- Task 27.0 freezes public beta scope and creates the evidence index.
- Task 27.1 audits readiness and blocks public beta release-candidate work.
- Public beta is blocked until a signed/notarized DMG, checksums, and clean-Mac downloaded-artifact QA exist.
- Blocked-gate UI hardening may fix local developer-preview QA bugs, but must not be treated as public beta release-candidate work.
- Local alpha quality closure is now the active route while Task 27.2 is blocked. Q4.4 closure evidence indexing and Q5.1 through Q5.5 developer-runtime evidence are complete; the next gate is Q6.1 unsigned developer-preview DMG build and inspection.
- Trust issues in source support, export overwrite protection, and missing-original state come before UI polish.
- Unsigned developer-preview DMGs are internal testing artifacts only and must not be called public beta.
- MLX runtime, plugin runtime, MCP server/runtime, broad RAW claims, broad visual color-correctness claims, cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution are excluded from public beta scope.

## Stop Rules

- Do not start feature growth before the active local alpha quality closure route is complete or explicitly superseded.
- Do not treat visual QA screenshots as product feature implementation.
- Do not add broad fallback systems for performance, migration, or profiling work.
- Do not start agent, MCP, or plugin runtime.
- Do not add MLX runtime, model loading, MCP runtime, plugin runtime, cloud sync, telemetry, auto-update, or broad RAW support unless the selected roadmap task explicitly requires it.
- Do not mutate original photo files.
