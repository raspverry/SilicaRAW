# Spike 003: Color-Managed Preview and Export

Status: completed  
Date: 2026-06-08  
Result: Path B - recommendation selected, fixture execution blocked

## Question

What color-management path should SilicaRAW use first for preview and export, and can the project verify basic sRGB and Display P3 assumptions now?

## Result

Path B:

```txt
Use Core Image/ColorSync-compatible color management first.
Use a linear Display P3-compatible working space as the first implementation recommendation.
Render preview through a display-profile-aware transform.
Default export to sRGB with ICC embedding.
Support Display P3 export when explicitly selected.
Do not claim color correctness until tagged fixtures exist.
```

This is not full Path A because the repository does not yet contain legal tagged sRGB/Display P3 raster fixtures or RAW color fixtures. It is not Path C because Apple's Core Image and Core Graphics APIs expose the required working, destination, sRGB, and Display P3 color-space concepts.

## External Evidence

- Core Image `CIContext` supports automatic color management through a working color space and destination color space.
- `CIContext.workingColorSpace` defines the color space used while executing filter kernels.
- `CIContextOption.outputColorSpace` defines the default destination color space for render methods and defaults to sRGB when unspecified.
- Core Graphics provides system-defined `CGColorSpace` values for sRGB, Display P3, and Linear Display P3.
- Display P3 uses DCI P3 primaries, D65 white point, and the sRGB transfer function.
- Core Image representation APIs render image data into an explicit output `CGColorSpace`.

## Local Fixture Evidence

Commands:

```sh
rg --files | rg -i '\.(jpg|jpeg|heic|tif|tiff|png)$'
```

Result:

```txt
The repository contains app icons and UI mockup PNGs only.
No tagged sRGB, tagged Display P3, untagged raster, or RAW color fixtures are committed.
```

Fixture class:

```txt
Class F - Tagged raster images
sRGB JPEG, Display P3 HEIC/JPEG, untagged JPEG
Status: missing
```

## Machine Details

Recorded on 2026-06-08:

```txt
macOS: 26.4 (25E246)
Architecture: arm64
Chip: Apple M5 Max
Memory: 128GB
GPU: Apple M5 Max, 40 cores, Metal 4
Display: Built-in Liquid Retina XDR Display
Resolution: 3024 x 1964 Retina
Display class: built-in Display P3-capable panel
```

## Implementation

- Added color gate metadata to `crates/silica-render`.
- Did not add Core Image, ColorSync, ICC, image, or export dependencies.
- Did not render, transform, compare, or export image data.
- Tagged color-dependent work with `color-blocking`.

## Recommendation

| Area | Decision | Notes |
| --- | --- | --- |
| Working space | Linear Display P3-compatible RGB | Good Apple-platform fit and wide-gamut baseline, but needs fixture-backed validation. |
| Preview | Display-profile-aware transform | Preview must convert from working space to the active display profile. |
| Export default | sRGB with ICC embedding | Matches the safest first local-alpha sharing path. |
| Optional export | Display P3 with ICC embedding | Supported after fixture validation. |
| Untagged raster input | Treat as sRGB | Must be explicit and visible in future diagnostics. |

## Known Limitations

- No tagged color fixtures are committed.
- No Preview.app, Apple Photos, Lightroom, or Capture One comparison was run.
- No ICC embedding proof exists yet.
- No RAW camera-profile behavior is verified.
- No golden-image tolerance policy exists.
- No HDR workflow is selected; SDR remains the v1 focus.

## Follow-Up

Before implementing product color behavior:

- Add a legal color fixture manifest.
- Commit or reference tagged sRGB, tagged Display P3, and untagged raster fixtures with license metadata.
- Add a macOS-only non-default Core Image color probe.
- Export sRGB and Display P3 outputs and verify ICC embedding.
- Compare results against Preview.app at minimum.
- Record tolerance policy before adding golden image tests.

## Sources

- Apple CIContext documentation: https://developer.apple.com/documentation/coreimage/cicontext
- Apple CIContext workingColorSpace documentation: https://developer.apple.com/documentation/coreimage/cicontext/workingcolorspace
- Apple CIContextOption outputColorSpace documentation: https://developer.apple.com/documentation/coreimage/cicontextoption/1438052-outputcolorspace
- Apple CGColorSpace documentation: https://developer.apple.com/documentation/coregraphics/cgcolorspace
- Apple CGColorSpace displayP3 documentation: https://developer.apple.com/documentation/coregraphics/cgcolorspace/displayp3
- Apple CGColorSpace sRGB documentation: https://developer.apple.com/documentation/coregraphics/cgcolorspace/srgb

## Guardrails

Do not use this spike as a color pipeline implementation. It does not prove color correctness, export fidelity, ICC embedding, RAW color behavior, or viewer consistency.
