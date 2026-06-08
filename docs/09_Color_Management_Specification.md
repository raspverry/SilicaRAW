# 09 — SilicaRAW Color Management Specification

Status: GO WITH CONDITIONS

## Principle

Fast but wrong color is failure.

Color defaults must be neutral, predictable, and export-consistent.

## Goals

- RAW opens with reasonable baseline color
- Preview is display-profile aware
- sRGB export is correct and default
- Display P3 export supported
- ICC profile embedding works
- Avoid double conversion
- Preview/export consistency

## Pipeline

```txt
RAW / Image File
↓
Decoder
↓
Input profile / camera profile
↓
Working linear RGB
↓
Edit graph operations
↓
Display transform or Export transform
```

## Required Color Spaces

Input:

- RAW camera space
- DNG profile
- Embedded ICC for JPEG/HEIC/TIFF
- Assume sRGB for untagged raster files

Working:

- Spike 003 recommendation: linear Display P3-compatible wide gamut
- This is an implementation recommendation, not fixture-backed color proof yet
- v0.1 may use linear extended sRGB/Core Image reference behavior for comparison

Display:

- sRGB
- Display P3
- external ICC displays

Export:

- sRGB default
- Display P3
- Adobe RGB
- ProPhoto RGB

## Export Defaults

- Format: JPEG
- Color Space: sRGB
- Embed ICC: ON
- Metadata: Preserve, remove GPS option visible
- Quality: 90

## Soft Proofing

P2. Do not pretend full proofing exists in v1.

## HDR

P3. SDR RAW editing is v1 focus.

## Required Spike

Color-managed preview/export spike:

- Load tagged sRGB image
- Load tagged Display P3 image
- Export sRGB and P3
- Embed ICC
- Compare with Preview.app

Spike 003 result:

```txt
Path B:
Core Image/ColorSync-compatible color management first.
Working-space recommendation: linear Display P3-compatible RGB.
Preview: display-profile aware.
Export default: sRGB with ICC embedding.
Optional export: Display P3 with ICC embedding.
Fixture execution: blocked because tagged color fixtures are missing.
```

## Testing

- Golden image tests
- Human review for skin, food, greens, skies, high ISO, mixed lighting
- Reference compare with Lightroom/Capture One/Apple Photos where useful

## Final Verdict

GO WITH CONDITIONS.

Working color space and camera profile strategy need spike results.

After Spike 003, the working-space recommendation is recorded, but fixture-backed color correctness is still required before product claims.
