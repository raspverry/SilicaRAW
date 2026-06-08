# 08 — SilicaRAW Metal Render Pipeline Specification

Status: GO WITH CONDITIONS

## Principle

Metal is the core interactive render path. It is not optional.

## Goals

- Slider changes feel immediate
- Zoom/pan smooth
- Before/after instant
- Histogram updates quickly
- Preview/export consistency
- Avoid unnecessary CPU/GPU transfers
- Separate preview and export paths

## Architecture

```txt
RAW Decode
↓
Texture Upload / Cache
↓
Metal Render Graph
↓
Adjustment Passes
↓
Mask Composite Passes
↓
Display Transform
↓
Viewer
```

## Modules

- RenderCoordinator
- RenderGraph
- TextureManager
- PipelineRegistry
- ShaderLibrary
- PreviewRenderer
- ExportRenderer
- HistogramRenderer
- MaskCompositor
- RenderCache
- PerformanceProfiler

## Preview vs Export

Preview:

- Fast
- Interactive
- May use downscaled texture
- Latest request wins
- Full quality after drag end

Export:

- Full resolution
- Deterministic
- Color-managed
- Complete edit graph
- ICC-aware output

## Render Graph Passes

- WhiteBalancePass
- ExposurePass
- TonePass
- CurvePass
- ColorMixerPass
- DetailPass
- LensPass
- GeometryPass
- MaskCompositePass
- DisplayTransformPass

## Required Spike

Tauri + native Metal viewer feasibility spike. Must prove:

- Metal output in app window
- Resize works
- Retina scaling works
- Mouse/trackpad events map correctly
- UI responsive
- Render timing available
- Metal render loop can be controlled from Rust/Core

## Performance Targets

- Slider perceived response target: under 50ms where feasible
- Fit preview after slider release: under 150ms target
- 100% detail render: under 1s target
- Batch export must not freeze UI

## Final Verdict

GO WITH CONDITIONS.

Must complete Metal viewer spike before broad UI/Develop implementation.
