---
title: Plugins and MCP
status: active
audience: all
updated: 2026-06-18
source_of_truth: docs/12_Plugin_and_MCP_Specification.md
---

# Plugins and MCP

## Summary

Plugins and MCP are optional extension layers. They must be permissioned, auditable, and subordinate to the core editor.

## Current Stance

- Plugin and MCP work is later-stage.
- Plugins and MCP must go through Core APIs and the permission layer.
- Direct SQLite writes from plugins or MCP are forbidden.
- Dangerous tools and arbitrary executable plugins are forbidden without explicit approval.
- MCP is off by default.
- Task 16.5 adds append-only action log groundwork for future permissioned extension work, but it does not add plugin runtime, MCP runtime, MLX execution, or extension write permissions.
- Task 21.5 adds disabled Preferences controls for Agent Access, MCP Tools, and Plugin Runtime. The controls explain future permission prompts, Core API boundaries, side effects, and action-log evidence, but they do not start any runtime.
- Task 23.1 adds the core extension permission vocabulary and default-deny policy. `silica-plugin` and `silica-mcp` record matching permission IDs for boundary checks only; no plugin runtime dependency, MCP server, tool execution, or permission prompt is active yet.
- Task 23.2 adds the static permission prompt UI contract in Preferences Advanced. It shows actor identity, requested permission, side effects, confirmation requirement, undo availability, deny behavior, and dangerous-permission unavailability without granting permissions or starting runtime behavior.
- Task 23.3 adds Core action-log wrappers for permission grants, denials, plugin apply reviews, AI approvals, MCP reads, and permissioned export attempts. Storage rejects extension raw-SQL/direct-database bypass claims. This is evidence only; it does not enable plugin, MCP, AI, or agent runtime.
- Task 25.1 validates `silica.plugin` v1 manifests before any plugin can be enabled. Valid manifests stay `enabled_by_default = false`; validation rejects missing trust fields, empty permissions, unknown permissions, raw SQL, direct database access, and write-like permissions.
- Task 25.2 validates data-only `silica.plugin_preset_pack` v1 payloads and applies P0 Basic preset data only through explicit Core approval, edit graph validation, undoable history, and `plugin_apply` action-log evidence. It still starts no plugin runtime.

## Task 23.1 Permission Vocabulary

Core permission IDs currently cover:

- `metadata:read`
- `metadata:write`
- `edit_suggestion:read`
- `edit_suggestion:apply`
- `export:local`
- `filesystem:limited_read`
- `filesystem:limited_write`
- `ai_result:read`
- `ai_result:propose`
- `mcp:read_only`
- `mcp:review`
- `mcp:edit`
- `mcp:export`

The default policy grants none of these permissions. There is no `raw_sql` permission, no permission to mutate original photo files, and no MCP mode that can change permissions.

## Task 23.2 Prompt Contract

Future permission prompts must show:

- Actor: extension name, type, version, and source.
- Requested permission: stable core permission ID plus plain language scope.
- Side effects: catalog, sidecar, export, AI result, filesystem, or action-log effects before confirmation.
- Confirmation: explicit user action with no prechecked controls or promotional copy.
- Undo availability: whether undo exists, and why if it does not.
- Deny behavior: denial keeps the extension disabled and records no grant.
- Dangerous permissions: unavailable unless a future ADR approves them.

This is a UI contract only. It does not add permission persistence, grant storage, plugin loading, MCP tools, background agents, or direct SQLite access.

## Task 23.3 Action Log Contract

Future extension-sensitive actions must enter Core before they can be logged:

- `permission_grant`
- `permission_denial`
- `plugin_apply`
- `ai_approval`
- `mcp_read`
- `export_attempt`

Each row records actor, action type, subject, side-effect category, evidence reference, and JSON payload with the stable permission ID. Logged rows are evidence; they are not undo records and do not store active permission grants.

Extension actors cannot log raw SQL or direct database access claims. Direct SQLite access from plugins, MCP tools, AI paths, and agents remains forbidden.

## Required Manifest Areas

- Plugin manifests must include schema, version, plugin identity, license, plugin version, minimum SilicaRAW version, type, and non-empty safe allowlisted permissions.
- Plugin preset packs must include schema, version, matching plugin ID, and data-only P0 Basic preset entries. Executable fields are rejected.
- Model manifests must include license, source, and hash.
- MCP tool manifests must declare permission, side effects, confirmation behavior, and undo behavior.

## Links

- [Plugin and MCP Specification](../../12_Plugin_and_MCP_Specification.md)
- [Plugin Manifest Schema](../../../schemas/plugin_manifest.schema.json)
- [Plugin Preset Pack Schema](../../../schemas/plugin_preset_pack.schema.json)
- [MCP Tool Manifest Schema](../../../schemas/mcp_tool_manifest.schema.json)
- [Agent Rules](../../../codex/AGENT_RULES.md)

## Notes for LLM Agents

Do not add MCP tools, plugin runtimes, or permission bypasses early. Any future mutation path must go through Core APIs, be explicit, be logged through the action log, and be reversible where possible. Direct SQLite access from plugins or MCP remains forbidden.
