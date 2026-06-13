---
title: SilicaRAW Wiki
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/00_INDEX.md
---

# SilicaRAW Wiki

## Summary

This wiki is the public, LLM-readable knowledge layer for SilicaRAW. It helps people and agents understand the project, navigate the specifications, track decisions, and connect research notes without duplicating the authoritative documents.

## How to Use This Wiki

- Start with [Project Overview](overview/project.md) if you are new to SilicaRAW.
- Start with [LLM Routing Index](llm/index.md) if you are an agent trying to minimize read context.
- Read [Architecture Overview](overview/architecture.md) before changing boundaries between crates or systems.
- Read [Roadmap Overview](overview/roadmap.md) before choosing implementation order.
- For Phase 14 through v1.0 sequencing, use the [Post-Alpha Master Execution Plan](roadmaps/post-alpha-master-execution-plan.md) before creating new phase task cards.
- Check [Decision Records](decisions/index.md) before changing an accepted direction.
- Check [Open Questions](questions/open-questions.md) before inventing answers.
- Use [Conventions](conventions.md) when adding or editing wiki pages.
- Use [Git and PR Workflow](contributing/git-and-pr-workflow.md) before starting a new branch or pull request.

## Sections

### LLM Routing

- [LLM Routing Index](llm/index.md): smallest useful read sets for agents.
- [Task Cards](tasks/index.md): compact task-level instructions for current atomic work, including Phase 16.

### Overview

- [Project](overview/project.md): product identity, current scope, and non-goals.
- [Architecture](overview/architecture.md): high-level system boundaries and guardrails.
- [Roadmap](overview/roadmap.md): current execution order and gate logic.
- [Local DMG Distribution Plan](roadmaps/local-dmg-distribution-plan.md): phased plan for GitHub-hosted macOS DMG distribution.
- [Local DMG Release Runbook](roadmaps/local-dmg-release-runbook.md): maintainer steps for signed local releases and current unsigned preview artifacts.
- [Developer Preview Artifact Runbook](roadmaps/developer-preview-artifact-runbook.md): unpaid unsigned DMG artifact build, download, and verification steps.
- [Post-Alpha Product Roadmap](roadmaps/post-alpha-product-roadmap.md): atomic phases for growing the local alpha into a credible RAW editor.
- [Post-Alpha Master Execution Plan](roadmaps/post-alpha-master-execution-plan.md): execution router, dependency graph, and stop gates for Phase 14 through v1.0.
- [Phase 12 RAW Proof Plan](roadmaps/phase-12-raw-proof-plan.md): current RAW proof execution plan and legal fixture evidence gate.
- [Phase 13 Color Pipeline Proof Plan](roadmaps/phase-13-color-pipeline-proof-plan.md): completed color proof execution plan and fixture evidence gate.
- [Phase 14 Product Metal Viewer Bridge Plan](roadmaps/phase-14-metal-viewer-bridge-plan.md): completed native viewer bridge plan and Path B evidence gate.

### Phase Briefs

- [Phase 11 Summary](phases/phase-11-summary.md): completed session, grid, metadata, and import foundation context.
- [Phase 12 RAW Proof Brief](phases/phase-12-raw-proof.md): completed RAW proof scope and task order.
- [Phase 13 Color Pipeline Proof Brief](phases/phase-13-color-pipeline-proof.md): completed color proof scope and task order.
- [Phase 14 Product Metal Viewer Bridge Brief](phases/phase-14-product-metal-viewer-bridge.md): completed native viewer bridge scope and task order.
- [Phase 15 RAW Color Metal Vertical Slice Brief](phases/phase-15-raw-color-metal-vertical-slice.md): completed RAW/color/Metal vertical slice scope and task order.
- [Phase 16 Undo History Action Trust Brief](phases/phase-16-undo-history-action-trust.md): completed undo, history, and action trust scope and task order.
- [Phase 17 Develop P0 Expansion Brief](phases/phase-17-develop-p0-expansion.md): completed Develop P0 control family scope and task order.

### Decisions

- [Decision Index](decisions/index.md): accepted, proposed, and superseded decisions.
- [ADR 0001: Monorepo Foundation](decisions/adr-0001-monorepo-foundation.md): initial Rust workspace and crate boundary decision.
- [ADR 0005: Defer MLX from Local Alpha](decisions/adr-0005-mlx-deferral-for-local-alpha.md): MLX remains outside local alpha scope.
- [ADR 0006: Unsigned Developer Preview DMG](decisions/adr-0006-unsigned-developer-preview-dmg.md): unsigned preview artifacts are allowed while Developer ID funding is blocked.
- [ADR 0007: Defer Homebrew and Auto-Update](decisions/adr-0007-homebrew-and-auto-update-deferral.md): Homebrew Cask and updater work wait for local DMG alpha trust gates.
- [ADR 0008: Project License](decisions/adr-0008-project-license.md): SilicaRAW source code and project documentation use the MIT License.

### Topics

- [RAW Decoding](topics/raw-decoding.md)
- [Metal Rendering](topics/metal-rendering.md)
- [Color Management](topics/color-management.md)
- [Catalog](topics/catalog.md)
- [Data Safety](topics/data-safety.md)
- [Backup and Restore](topics/backup-restore.md)
- [Public Trust](topics/public-trust.md)
- [UI Mockups](topics/ui-mockups.md)
- [UI MVP Baseline](topics/ui-mvp-baseline.md)
- [UI Visual and Responsive QA](topics/ui-visual-responsive-qa.md)
- [Product Alpha Runtime Completion](topics/product-alpha-runtime-completion.md)
- [Edit Graph](topics/edit-graph.md)
- [Action Trust](topics/action-trust.md)
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
