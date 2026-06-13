---
title: Plugins and MCP
status: active
audience: all
updated: 2026-06-08
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
