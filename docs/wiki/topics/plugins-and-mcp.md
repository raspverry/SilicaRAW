---
title: Plugins and MCP
status: active
audience: all
updated: 2026-06-17
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

## Required Manifest Areas

- Plugin manifests must include license and permissions.
- Model manifests must include license, source, and hash.
- MCP tool manifests must declare permission, side effects, confirmation behavior, and undo behavior.

## Links

- [Plugin and MCP Specification](../../12_Plugin_and_MCP_Specification.md)
- [Plugin Manifest Schema](../../../schemas/plugin_manifest.schema.json)
- [MCP Tool Manifest Schema](../../../schemas/mcp_tool_manifest.schema.json)
- [Agent Rules](../../../codex/AGENT_RULES.md)

## Notes for LLM Agents

Do not add MCP tools, plugin runtimes, or permission bypasses early. Any future mutation path must go through Core APIs, be explicit, be logged through the action log, and be reversible where possible. Direct SQLite access from plugins or MCP remains forbidden.
