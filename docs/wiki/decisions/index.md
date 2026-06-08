---
title: Decision Records
status: active
audience: all
updated: 2026-06-08
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

## ADR Format

Decision records should include:

- Context
- Decision
- Consequences
- Alternatives considered
- Links

## Notes for LLM Agents

Before changing a recorded decision, add a new ADR that supersedes the previous one. Do not silently rewrite accepted history.
