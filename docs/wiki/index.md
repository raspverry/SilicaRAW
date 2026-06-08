---
title: SilicaRAW Wiki
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/00_INDEX.md
---

# SilicaRAW Wiki

## Summary

This wiki is the public, LLM-readable knowledge layer for SilicaRAW. It helps people and agents understand the project, navigate the specifications, track decisions, and connect research notes without duplicating the authoritative documents.

## How to Use This Wiki

- Start with [Project Overview](overview/project.md) if you are new to SilicaRAW.
- Read [Architecture Overview](overview/architecture.md) before changing boundaries between crates or systems.
- Read [Roadmap Overview](overview/roadmap.md) before choosing implementation order.
- Check [Decision Records](decisions/index.md) before changing an accepted direction.
- Check [Open Questions](questions/open-questions.md) before inventing answers.
- Use [Conventions](conventions.md) when adding or editing wiki pages.
- Use [Git and PR Workflow](contributing/git-and-pr-workflow.md) before starting a new branch or pull request.

## Sections

### Overview

- [Project](overview/project.md): product identity, current scope, and non-goals.
- [Architecture](overview/architecture.md): high-level system boundaries and guardrails.
- [Roadmap](overview/roadmap.md): current execution order and gate logic.
- [Local DMG Distribution Plan](roadmaps/local-dmg-distribution-plan.md): phased plan for GitHub-hosted macOS DMG distribution.

### Decisions

- [Decision Index](decisions/index.md): accepted, proposed, and superseded decisions.
- [ADR 0001: Monorepo Foundation](decisions/adr-0001-monorepo-foundation.md): initial Rust workspace and crate boundary decision.
- [ADR 0005: Defer MLX from Local Alpha](decisions/adr-0005-mlx-deferral-for-local-alpha.md): MLX remains outside local alpha scope.

### Topics

- [RAW Decoding](topics/raw-decoding.md)
- [Metal Rendering](topics/metal-rendering.md)
- [Color Management](topics/color-management.md)
- [Catalog](topics/catalog.md)
- [Data Safety](topics/data-safety.md)
- [UI Mockups](topics/ui-mockups.md)
- [Edit Graph](topics/edit-graph.md)
- [MLX](topics/mlx.md)
- [Plugins and MCP](topics/plugins-and-mcp.md)

### Contributing

- [Git and PR Workflow](contributing/git-and-pr-workflow.md): branch naming, PR flow, merge policy, and release branch rules.

### Spikes

- [Spike 001: Tauri Metal Viewer](../spikes/001-tauri-metal-viewer.md): Phase 3.1 native `MTKView` proof, input evidence, resize evidence, and shell-path decision.
- [Spike 002: RAW Decoder Path](../spikes/002-raw-decoder.md): Phase 3.2 Core Image primary decision, LibRaw fallback status, and fixture gap.
- [Spike 003: Color-Managed Preview and Export](../spikes/003-color-managed-preview-export.md): Phase 3.3 working-space recommendation, export color stance, and fixture gap.
- [Spike 004: SQLite Catalog Persistence](../spikes/004-sqlite-persistence.md): Phase 3.4 SQLite binding, migration approach, schema, and required index proof.

### Sources

- [Source Index](sources/index.md)
- [Karpathy LLM Wiki](sources/karpathy-llm-wiki.md)
- [karpathy/autoresearch](sources/karpathy-autoresearch.md)
- [huggingface/ml-intern](sources/huggingface-ml-intern.md)

### Risks and Questions

- [Risk Index](risks/index.md)
- [Architecture Risks](risks/architecture-risks.md)
- [Open Questions](questions/open-questions.md)

## Authoritative Sources

The wiki is not the source of truth for schemas, product requirements, or implementation rules. Use these documents first when the details matter:

- [Docs Index](../00_INDEX.md)
- [Codex Handoff](../../codex/CODEX_HANDOFF.md)
- [Agent Rules](../../codex/AGENT_RULES.md)
- [Architecture Patch](../20_v1_1_Architecture_Patch.md)
- [Schema Reference](../19_Schema_Reference.md)
- [Edit Graph Schema](../../schemas/edit_graph.schema.json)

## Notes for LLM Agents

Read this page first, then read the smallest relevant set of linked pages. Do not treat wiki summaries as permission to bypass `codex/AGENT_RULES.md`, schema files, or explicit task scope.
