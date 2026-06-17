---
title: "ADR 0009: MLX Runtime Spike"
status: accepted
audience: all
updated: 2026-06-17
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# ADR 0009: MLX Runtime Spike

## Context

Phase 24 starts local AI work after the editor trust gates. The editor must still work without MLX, no model can ship without a manifest, and AI output must remain suggestion data until explicit user approval.

Current source review:

- MLX is Apple Silicon-oriented and uses unified memory.
- MLX has official Python, Swift, and C-family surfaces.
- SilicaRAW currently has a Rust/Tauri core boundary, not a Swift app shell.
- Model manifests are already required by `schemas/model_manifest.schema.json`.

Primary references:

- MLX project: <https://github.com/ml-explore/mlx>
- MLX C API: <https://github.com/ml-explore/mlx-c>
- MLX Swift: <https://github.com/ml-explore/mlx-swift>
- MLX unified memory documentation: <https://ml-explore.github.io/mlx/build/html/usage/unified_memory.html>

## Decision

Task 24.1 records MLX as feasible only as a future optional runtime. It does not add an MLX dependency, model loader, model bundle, inference path, UI, or background worker.

The provisional future runtime path is:

```txt
Rust Core -> non-default feature-gated FFI boundary -> official MLX C API
```

The Python package is rejected for product runtime packaging because it would add a Python environment to a local desktop editor. The Swift package remains a fallback if the app shell moves to SwiftUI/AppKit later, but it is not the first Rust/Tauri integration path.

No-model behavior:

```txt
No manifest or no model -> AI features unavailable; core editor remains usable.
```

Memory policy:

```txt
Treat MLX unified memory as app-global pressure.
Use one bounded AI worker lane by default.
Do not run inference on the UI thread.
Record future memory observations before enabling more concurrency.
```

Cancellation policy:

```txt
Cancellation is cooperative at queue/task boundaries.
Do not promise a hard kill for an already-running MLX kernel until a future runtime probe proves it.
```

Packaging policy:

```txt
No model weight can be bundled or enabled without a valid model manifest.
No runtime binary or package can be shipped before docs/DEPENDENCIES.md records license, source, version, size, and security notes.
```

## Consequences

- `crates/silica-mlx` remains dependency-free and boundary-only.
- Task 24.2 should implement model manifest validation before any model enablement path.
- Task 24.3 should store AI results separately from edit graph and catalog flags.
- Task 24.4 can expose a non-mutating review surface that degrades cleanly when no model is available.
- Task 24.5 must require explicit approval before converting AI suggestions into edit graph changes.

## Alternatives Considered

- **Python MLX runtime:** rejected for first product path due to desktop packaging weight and environment complexity.
- **MLX Swift first:** deferred because the current app shell is Rust/Tauri; keep as fallback if the shell changes.
- **Add MLX now:** rejected because Task 24.1 is a spike and model manifests must land first.
- **Bundle a starter model now:** rejected because no model should ship without manifest, license, source, hash, preprocessing, and output metadata.

## Links

- [MLX Topic](../topics/mlx.md)
- [Post-Alpha Product Roadmap](../roadmaps/post-alpha-product-roadmap.md#phase-24-mlx-and-ai-preview)
- [Model Manifest Schema](../../../schemas/model_manifest.schema.json)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [ADR 0005: Defer MLX from Local Alpha](adr-0005-mlx-deferral-for-local-alpha.md)
