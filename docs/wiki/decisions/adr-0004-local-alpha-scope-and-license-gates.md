---
title: "ADR 0004: Local Alpha Scope and License Gates"
status: accepted
audience: all
updated: 2026-06-08
source_of_truth: docs/18_Final_Master_Plan.md
---

# ADR 0004: Local Alpha Scope and License Gates

## Context

SilicaRAW needs a narrow first downloadable alpha. The full v1 plan includes many professional editing features, MLX, plugins, MCP, release channels, and community assets, but those are too broad for the first local DMG target.

The project license is not finalized yet.

## Decision

Local alpha scope is limited to:

- Launch app.
- Create or open a local library.
- Import a folder by reference.
- Show a library grid.
- Rate, pick, or reject photos.
- Open a preview.
- Apply exposure/contrast.
- Persist edit state.
- Export JPEG sRGB.
- Verify original files are unchanged.

These are explicitly deferred from local alpha:

- MLX runtime and model loading.
- MCP server and tools.
- Plugin runtime.
- Cloud sync.
- Telemetry or analytics.
- Auto-update.
- Homebrew Cask.
- Mac App Store distribution.

License gate:

- Private local development can continue while license selection is open.
- Any public GitHub release requires at least a provisional license strategy.
- Public beta/stable requires a final project license and dependency/license inventory.

## Consequences

- The first app can be useful without implementing the full v1 feature list.
- Release planning must keep public-license readiness visible.
- Agents must not add MLX/MCP/plugin/cloud/telemetry work while implementing local alpha unless the user explicitly changes scope.
- ADR 0005 records the MLX-specific deferral in more detail.

## Alternatives Considered

- Full v1 before any DMG: too broad and delays validation.
- Packaging-only DMG with no editor workflow: not useful enough to call local distribution.
- Selecting a final license automatically: rejected because license choice is a maintainer decision.

## Links

- [Final Master Plan](../../18_Final_Master_Plan.md)
- [Release and Distribution Plan](../../16_Release_Distribution_Plan.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [ADR 0005: Defer MLX from Local Alpha](adr-0005-mlx-deferral-for-local-alpha.md)
- [ADR 0008: Project License](adr-0008-project-license.md)
- [Open Questions](../questions/open-questions.md)

## Follow-Up

ADR 0008 selected the MIT License for SilicaRAW source code and project documentation on 2026-06-11. Dependency, model, and sample-asset license inventories remain separate release checks.

## Notes for LLM Agents

Keep alpha work narrow. If a task needs a deferred feature, record a scope question instead of implementing it.
