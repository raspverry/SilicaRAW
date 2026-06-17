---
title: MLX
status: active
audience: all
updated: 2026-06-17
source_of_truth: docs/11_MLX_Feature_Specification.md
---

# MLX

## Summary

MLX is planned as a local Apple Silicon enhancement layer, not the core editor. MLX features should produce suggestions or masks that require explicit user approval before changing edit state.

## Current Stance

- ADR 0005 defers MLX from local DMG alpha.
- Task 24.1 records the first post-alpha runtime spike without enabling a runtime.
- `silica-mlx` remains a boundary crate only and still has no MLX dependency.
- The provisional future binding path is the official MLX C API behind a non-default Rust feature gate.
- The Python package is not a product runtime path; MLX Swift remains a fallback if the app shell changes.
- If no model manifest or model is available, AI features stay unavailable and the core editor continues.
- Model licenses, sources, hashes, preprocessing, and output metadata must be recorded.
- No model weight can be bundled or enabled without a valid model manifest.
- Treat MLX unified memory as app-global memory pressure; future runtime work should use a bounded worker lane.
- Cancellation is cooperative at queue/task boundaries until a runtime probe proves stronger behavior.
- MLX should not own final image state.
- MLX should not directly mutate the edit graph.

## Blocked Work

- Model loading.
- Inference runtime.
- Subject mask.
- Sky mask.
- Auto tone.
- Blur detection.
- Quality score.
- MLX denoise or upscale structures.

## Links

- [ADR 0005: Defer MLX from Local Alpha](../decisions/adr-0005-mlx-deferral-for-local-alpha.md)
- [ADR 0009: MLX Runtime Spike](../decisions/adr-0009-mlx-runtime-spike.md)
- [MLX Feature Specification](../../11_MLX_Feature_Specification.md)
- [Model Manifest Schema](../../../schemas/model_manifest.schema.json)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [Plugin and MCP Topic](plugins-and-mcp.md)

## Notes for LLM Agents

Do not add MLX dependencies, model downloads, model loaders, inference code, model assets, or MLX UI unless the selected task explicitly requires that scoped work. Task 24.1 records the runtime direction only; Task 24.2 must validate manifests before any model can be enabled.
