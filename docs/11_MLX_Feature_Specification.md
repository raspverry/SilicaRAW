# 11 — SilicaRAW MLX Feature Specification

Status: GO WITH CONDITIONS

## Principle

MLX suggests. User approves. Edit graph records. Original remains untouched.

## Role

MLX is an intelligent editing/analysis layer, not the editor core.

## Priority

P0: No MLX required. Core editor must work without MLX.

Local alpha status: ADR 0005 defers MLX runtime, model loading, inference, and model assets from the local DMG alpha. `silica-mlx` remains a boundary crate only. Task 24.1 records the post-alpha runtime spike without adding a runtime dependency or model assets.

P1:

- Subject Mask
- Sky Mask
- Blur Detection
- Duplicate Grouping
- Quality Score
- Auto Tone Suggestion

P2:

- MLX Denoise
- MLX Upscale
- Background Mask
- Food/Portrait/Cafe Auto Enhance
- Closed Eyes Detection

P3:

- Object Removal
- Inpainting
- Generative Fill
- Face Retouch
- Style Transfer
- AI Chat Editing

## MLX Engine Modules

- MLXEngine
- ModelManager
- ModelRegistry
- InferenceQueue
- Preprocessor
- Postprocessor
- ResultCache
- TaskRunner
- DeviceMonitor
- ErrorReporter

## Required Rules

- Model license explicit
- Model version tracked
- Input preprocessing specified
- Output format specified
- AI result stored separately
- User approval required for state changes
- Background tasks cannot block editor
- Core editor continues if model missing

## Integration

- Subject/Sky output → mask texture → Metal MaskCompositor
- Auto Tone output → Basic edit values → edit graph
- Denoise output → cached intermediate/reference → preview/export path

## Final Verdict

GO WITH CONDITIONS.

Task 24.1 records the runtime spike: future MLX C API bridge behind a non-default Rust feature gate, no-model behavior as AI unavailable/editor usable, bounded unified-memory posture, cooperative cancellation, and manifest-first model packaging. Still needed: model manifest validation, model licensing records for any shipped model, preprocessing specs, mask coordinate/texture spec, and runtime evidence before inference features ship.

After ADR 0005, these requirements remain later-stage and are not part of local DMG alpha.
