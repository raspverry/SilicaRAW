---
title: "ADR 0005: Defer MLX from Local Alpha"
status: accepted
audience: all
updated: 2026-06-08
source_of_truth: docs/wiki/roadmaps/local-dmg-distribution-plan.md
---

# ADR 0005: Defer MLX from Local Alpha

## Context

SilicaRAW's first local distribution goal is a GitHub Release DMG containing a signed and notarized `.app` that can complete a narrow local alpha workflow.

MLX is important to the long-term Apple Silicon direction, but it is not required for the first useful RAW editor workflow. Adding MLX early would require runtime decisions, model licenses, model manifests, preprocessing specs, memory-pressure handling, background scheduling, cache behavior, and user approval flows before the local alpha can prove its core editor loop.

ADR 0004 already defers broad local-alpha scope. This ADR records the specific MLX decision so agents do not reopen it while implementing storage, catalog, rendering, export, or packaging tasks.

## Decision

MLX is not required for local DMG alpha.

For local alpha:

- `crates/silica-mlx` remains a boundary crate only.
- No MLX dependency is added.
- No model loader, model registry, model download, inference queue, mask generation, denoise, upscale, auto tone, culling, or quality scoring behavior is added.
- No bundled model files are included.
- No model manifest is required unless a maintainer explicitly adds a model-related task.

MLX can resume after local alpha foundations are working and after a separate MLX runtime spike records:

- selected MLX package or binding
- model license policy
- model source and hash policy
- preprocessing and output contracts
- memory-pressure and cancellation strategy
- user approval flow for edit-state changes

## Consequences

- Local alpha can focus on launch, library creation, import by reference, grid, culling flags, preview, basic edits, persistence, and JPEG sRGB export.
- Packaging and notarization work stays smaller because no ML runtime or model assets are bundled.
- The MLX crate remains available as an architectural boundary without implying implementation readiness.
- Any future MLX task must update `docs/DEPENDENCIES.md` before adding dependencies.

## Alternatives Considered

- Include MLX auto tone in local alpha: rejected because it would require model and approval infrastructure before the editor core is proven.
- Include only model manager UI: rejected because model management without useful editor behavior is release noise.
- Remove the MLX crate until later: rejected because the boundary is useful for architecture clarity.

## Links

- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [ADR 0004: Local Alpha Scope and License Gates](adr-0004-local-alpha-scope-and-license-gates.md)
- [MLX Feature Specification](../../11_MLX_Feature_Specification.md)
- [MLX Topic](../topics/mlx.md)
- [Dependencies Policy](../../DEPENDENCIES.md)

## Notes for LLM Agents

Do not add MLX code, dependencies, model downloads, model loaders, inference paths, model assets, or MLX UI for local alpha. Treat `silica-mlx` as a boundary crate until a later explicit task changes this decision.
