---
title: "ADR 0001: Monorepo Foundation"
status: accepted
audience: all
updated: 2026-06-08
source_of_truth: codex/CODEX_HANDOFF.md
---

# ADR 0001: Monorepo Foundation

## Context

SilicaRAW needs a repository structure that matches the architecture before product implementation begins. The first implementation task was limited to creating the monorepo structure.

The task explicitly excluded RAW decoding, Metal viewer implementation, MLX, MCP, plugin behavior, UI screens, and new dependencies.

## Decision

Create a Rust workspace at the repository root with:

- `apps/desktop` as the desktop application placeholder.
- `crates/silica-core`
- `crates/silica-catalog`
- `crates/silica-storage`
- `crates/silica-decode`
- `crates/silica-render`
- `crates/silica-edit`
- `crates/silica-export`
- `crates/silica-mlx`
- `crates/silica-plugin`
- `crates/silica-mcp`

Each crate starts as a placeholder boundary with a README explaining its responsibility.

## Consequences

- Future implementation tasks have explicit package boundaries.
- The workspace can build and test without adding third-party dependencies.
- The placeholder crates reserve architecture boundaries without prematurely implementing product behavior.
- Later tasks must still respect the spike gates for Tauri + Metal, RAW decoding, color management, SQLite persistence, and MLX runtime.

## Alternatives Considered

- Single crate: simpler initially, but it would blur architecture boundaries and make future separation harder.
- Full app scaffold with Tauri and UI: rejected for Task 0101 because the Tauri + Metal spike has not been completed.
- Native SwiftUI/AppKit shell now: rejected for Task 0101 because the Tauri + Metal fallback decision has not been recorded yet.

## Links

- [Codex Handoff](../../../codex/CODEX_HANDOFF.md)
- [System Architecture](../../03_System_Architecture.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Root Workspace](../../../Cargo.toml)
- [Architecture Overview](../overview/architecture.md)

## Notes for LLM Agents

Do not treat the existence of `silica-decode`, `silica-render`, `silica-mlx`, `silica-plugin`, or `silica-mcp` as permission to implement those systems early. They are boundaries, not completed features.

