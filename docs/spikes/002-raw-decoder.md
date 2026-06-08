# Spike 002: RAW Decoder Path

Status: completed  
Date: 2026-06-08  
Result: Path A - Core Image RAW primary, LibRaw fallback deferred

## Question

Which RAW decoder path should SilicaRAW use first: Core Image RAW, LibRaw, or a hybrid backend?

## Result

Path A:

```txt
Use Core Image RAW as the first implementation target.
Defer LibRaw until legal fixtures prove a concrete camera-support gap.
```

This is not Path B because adding LibRaw first would add native C/C++ build, FFI, license, packaging, and color-pipeline complexity before SilicaRAW has legal fixture evidence. It is not Path C because maintaining two first-class decoder backends would multiply test and color-management work too early.

## External Evidence

- Apple `CIRAWFilter` is the macOS-native RAW filter path and exposes APIs to create RAW filters from data or URLs, inspect supported camera models, and query supported decoder versions.
- Apple documentation states Core Image can display and edit RAW and Apple ProRAW files.
- LibRaw is a broad RAW decoder library for formats including CRW/CR2, NEF, RAF, DNG, MOS, KDC, and DCR.
- LibRaw is distributed under LGPL 2.1 or CDDL 1.0; a Rust `libraw-sys` path would still link the native LibRaw C library.

## Local Fixture Evidence

Commands:

```sh
rg --files | rg -i '\.(cr2|cr3|nef|arw|raf|dng|rw2|orf|raw)$'
```

Result:

```txt
No legally usable RAW fixture files are committed in the repository.
```

This means the decoder path is selected as an implementation direction, but camera-support confidence remains blocked by the fixture manifest task.

## Implementation

- Added decoder gate metadata to `crates/silica-decode`.
- Did not add Core Image bindings.
- Did not add LibRaw, `libraw-sys`, or any other decoder dependency.
- Did not decode image bytes.
- Tagged decoder-dependent work with `decoder-blocking`.

## Path Comparison

| Path | Decision | Why |
| --- | --- | --- |
| Core Image RAW primary | Selected | Best fit for Apple Silicon/macOS focus, Apple ProRAW direction, and minimal early distribution burden. |
| LibRaw primary | Deferred | Strong broad format fallback, but adds native dependency, FFI, license, and packaging risk before fixture evidence. |
| Hybrid | Deferred | Plausible later if Core Image coverage fails, but too much early test/color matrix complexity. |

## Decoder-Dependent Tags

```txt
RAW Decode: decoder-blocking
Camera Profile: decoder-dependent
Lens Correction: decoder-dependent
Color Baseline: decoder-dependent
Fuji RAF support: high-risk decoder-dependent
Apple ProRAW: CoreImage-preferred
Broad camera support: LibRaw-preferred
```

## Follow-Up

Before implementing real decoding:

- Create a legal RAW fixture manifest.
- Include at least DNG or Apple ProRAW, one mainstream 24MP RAW, one higher-resolution RAW, one high-ISO RAW, and one risky Fuji RAF class if licensing allows.
- Add a tiny Core Image probe behind a macOS-only non-default feature.
- Record fixture machine, macOS version, decoder version, native size, success/failure, and error category.
- Add LibRaw only after fixture results justify the fallback and `docs/DEPENDENCIES.md` is updated with the selected binding.

## Sources

- Apple CIRAWFilter documentation: https://developer.apple.com/documentation/coreimage/cirawfilter
- Apple RAW and Apple ProRAW documentation: https://developer.apple.com/documentation/avfoundation/capturing-photos-in-raw-and-apple-proraw-formats
- LibRaw documentation: https://www.libraw.org/docs
- libraw-sys docs.rs page: https://docs.rs/libraw-sys/

## Guardrails

Do not use this spike as a RAW decoder implementation. It does not read, decode, demosaic, color-manage, cache, display, or export RAW images.
