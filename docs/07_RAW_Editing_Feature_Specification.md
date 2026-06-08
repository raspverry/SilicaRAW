# 07 — SilicaRAW RAW Editing Feature Specification

Status: GO WITH CONDITIONS

## Principle

SilicaRAW is a RAW editor first. Editing must be non-destructive, fast, and exportable.

## Editing Pipeline Concept

```txt
RAW Decode
↓
Input profile / camera profile
↓
Working linear RGB
↓
White balance
↓
Exposure / tone
↓
Tone curve
↓
HSL / Color Mixer
↓
Detail
↓
Lens
↓
Geometry / Crop
↓
Masks / Local adjustments
↓
Display or Export transform
```

## P0 Required

- RAW loading
- Non-destructive edit graph
- Histogram
- White balance
- Exposure
- Contrast
- Highlights/Shadows
- Whites/Blacks
- Temperature/Tint
- Vibrance/Saturation
- Crop/Rotate
- Before/After
- Reset
- Undo/Redo
- JPEG export
- Basic presets

## P1 Professional Baseline

- Tone curve
- HSL / Color Mixer
- Color grading baseline
- Sharpening
- Basic noise reduction
- Lens correction
- Chromatic aberration correction
- Vignette correction
- Edit history
- Copy/paste edits
- Batch sync edits
- PNG/TIFF/HEIC export
- Color space export
- Metadata handling

## P2 Differentiators

- Manual masks
- Linear/radial gradients
- Brush masks
- MLX Subject Mask
- MLX Sky Mask
- Auto Tone
- MLX Denoise
- MLX Upscale

## P3 Future

- Object removal
- Healing brush
- Generative fill
- Film grain engine
- Soft proofing
- Tethering
- HDR merge
- Panorama merge

## Panels

1. Histogram
2. Basic
3. Tone
4. Color
5. Detail
6. Lens
7. Geometry
8. Mask
9. Metadata
10. Export

## Edit Graph Requirements

- Versioned JSON
- Separate image edits from metadata flags
- Separate export settings from edit state
- Unknown future fields preserved where possible
- Undo checkpoints on committed edits, not every slider tick

## Preview vs Export

Preview prioritizes responsiveness and may use lower resolution during interaction. Export prioritizes quality, full resolution, and color management.

## v1 Cutline

v1 must include RAW decode, Basic, Tone, Color, Detail, Lens, Geometry, Presets, Undo/Redo, Before/After, Copy/Paste, Batch Sync, and Export.

v1 should not include object removal, AI chat, HDR, cloud, marketplace, or dangerous MCP.

## Final Verdict

GO WITH CONDITIONS.

Requires Metal pipeline, color management, and storage specs to implement safely.
