---
title: MLX
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/11_MLX_Feature_Specification.md
---

# MLX

## Summary

MLX is planned as a local Apple Silicon enhancement layer, not the core editor. MLX features should produce suggestions or masks that require explicit user approval before changing edit state.

## Current Stance

- ADR 0005 defers MLX from local DMG alpha.
- MLX work is later-stage and gated by a runtime spike.
- `silica-mlx` remains a boundary crate only.
- Model licenses, sources, hashes, preprocessing, and output metadata must be recorded.
- MLX should not own final image state.
- MLX should not directly mutate the edit graph.

## Blocked Work

- Subject mask.
- Sky mask.
- Auto tone.
- Blur detection.
- Quality score.
- MLX denoise or upscale structures.

## Links

- [ADR 0005: Defer MLX from Local Alpha](../decisions/adr-0005-mlx-deferral-for-local-alpha.md)
- [MLX Feature Specification](../../11_MLX_Feature_Specification.md)
- [Model Manifest Schema](../../../schemas/model_manifest.schema.json)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [Plugin and MCP Topic](plugins-and-mcp.md)

## Notes for LLM Agents

Do not add MLX dependencies, model downloads, model loaders, inference code, model assets, or MLX UI for local alpha. A future explicit MLX runtime spike must change this scope first.
