# silica-mlx

MLX feature boundary for SilicaRAW.

This crate will eventually contain local MLX integration boundaries for approved intelligent editing features.

ADR 0005 defers MLX from local alpha. This crate records that decision and remains a boundary only. No MLX dependency, model loading, inference, model asset, or AI behavior is present yet.

Task 24.1 records the first post-alpha runtime spike decision:

- Provisional future binding path: MLX C API behind a non-default Rust feature gate.
- No-model behavior: AI surfaces remain unavailable while the editor continues to work.
- Packaging rule: no model weight can be bundled or enabled without a valid model manifest.
- Runtime status: still boundary-only; no MLX dependency is linked by default.
