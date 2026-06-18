---
title: Decision Records
status: active
audience: all
updated: 2026-06-18
source_of_truth: docs/wiki/conventions.md
---

# Decision Records

## Summary

Decision records explain durable SilicaRAW choices, their context, and their consequences.

Use ADRs for decisions that future contributors or LLM agents might otherwise reopen accidentally.

## Records

| ADR | Status | Decision |
| --- | --- | --- |
| [ADR 0001](adr-0001-monorepo-foundation.md) | accepted | Establish a Rust workspace with the documented app and crate boundaries. |
| [ADR 0002](adr-0002-local-dmg-distribution.md) | accepted | Define local distribution as a GitHub Release DMG carrying `SilicaRAW.app`. |
| [ADR 0003](adr-0003-app-shell-packaging-path.md) | accepted | Use Tauri v2 for the first shell/packaging spike while preserving the Metal fallback gate. |
| [ADR 0004](adr-0004-local-alpha-scope-and-license-gates.md) | accepted | Limit local alpha scope and keep license choice as a release gate. |
| [ADR 0005](adr-0005-mlx-deferral-for-local-alpha.md) | accepted | Defer MLX from local alpha while keeping `silica-mlx` as a boundary crate. |
| [ADR 0006](adr-0006-unsigned-developer-preview-dmg.md) | accepted | Use unsigned developer-preview DMGs while Developer ID funding is blocked. |
| [ADR 0007](adr-0007-homebrew-and-auto-update-deferral.md) | accepted | Defer Homebrew Cask and auto-update until local DMG alpha trust gates are met. |
| [ADR 0008](adr-0008-project-license.md) | accepted | License SilicaRAW source code and project documentation under MIT. |
| [ADR 0009](adr-0009-mlx-runtime-spike.md) | accepted | Record the Phase 24 MLX runtime spike without enabling a runtime or bundling models. |
| [ADR 0010](adr-0010-mcp-transport-session.md) | accepted | Use disabled-by-default stdio-first MCP with process-lifetime sessions. |

## ADR Format

Decision records should include:

- Context
- Decision
- Consequences
- Alternatives considered
- Links

## Notes for LLM Agents

Before changing a recorded decision, add a new ADR that supersedes the previous one. Do not silently rewrite accepted history.
