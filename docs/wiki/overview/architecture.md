---
title: Architecture Overview
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/03_System_Architecture.md
---

# Architecture Overview

## Summary

SilicaRAW uses a modular architecture centered on a Rust core. The UI shell, catalog, edit graph, storage, rendering, export, MLX, plugin, and MCP layers must communicate through explicit boundaries.

## High-Level Shape

```txt
App Shell
  -> Rust Core
    -> Catalog
    -> Storage
    -> Edit Graph
    -> RAW Decode
    -> Render
    -> Export
    -> Permission Layer
    -> MLX
    -> Plugin
    -> MCP
```

## Current Workspace Boundaries

- `apps/desktop`: minimal Tauri shell and packaging skeleton.
- `crates/silica-core`: high-level coordination boundary.
- `crates/silica-catalog`: catalog domain boundary.
- `crates/silica-storage`: persistence boundary.
- `crates/silica-decode`: RAW decode abstraction boundary.
- `crates/silica-render`: render request and renderer boundary.
- `crates/silica-edit`: edit graph boundary.
- `crates/silica-export`: export coordination boundary.
- `crates/silica-mlx`: MLX feature boundary.
- `crates/silica-plugin`: plugin boundary.
- `crates/silica-mcp`: MCP boundary.

## Guardrails

- UI must not directly process pixels.
- MLX must not directly mutate the edit graph.
- Plugins and MCP must not directly write SQLite.
- Nothing may modify original photo files.
- New dependencies require documentation in `docs/DEPENDENCIES.md`.

## Links

- [System Architecture](../../03_System_Architecture.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Codex Handoff](../../../codex/CODEX_HANDOFF.md)
- [Agent Rules](../../../codex/AGENT_RULES.md)
- [ADR 0001: Monorepo Foundation](../decisions/adr-0001-monorepo-foundation.md)

## Notes for LLM Agents

Do not collapse crate boundaries to simplify an early task. If a task needs a cross-boundary shortcut, stop and document the architecture question instead of silently changing the design.
