---
title: RAW Decoding
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/20_v1_1_Architecture_Patch.md
---

# RAW Decoding

## Summary

RAW decoding is one of SilicaRAW's highest-risk technical areas. Spike 002 selected the first decoder path, but real decoding remains blocked by legal fixture coverage.

## Current Stance

- RAW decoding must be abstracted behind `silica-decode`.
- Spike 002 selected Core Image RAW primary.
- LibRaw remains a deferred fallback until legal fixtures prove a camera-support gap.
- Phase 5.1 adds preview decode readiness routing, not RAW pixels.
- Full decoder-dependent features remain blocked until real fixture-backed decoding exists.

## Blocked Work

- Camera profile assumptions.
- Broad camera support claims.
- Fuji RAF support commitments.
- Lens metadata and correction behavior.
- Final color pipeline assumptions tied to decoder output.

## Possible Outcomes

- Core Image RAW primary: selected by Spike 002.
- LibRaw primary: deferred.
- Hybrid Core Image and LibRaw path: deferred.

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

## Fixture Status

The repository currently has no legally usable RAW fixtures. Spike 002 scanned for common RAW file extensions and found none, so decoder confidence is still blocked by a fixture manifest.

## Phase 5.1 Preview Contract

`silica-decode` can now classify preview readiness:

```txt
raster candidate -> ready by reference
unsupported catalog entry -> unsupported
RAW candidate -> Core Image RAW blocked until fixture-backed probe
```

This preserves the Spike 002 decision without pretending RAW decoding exists.

## Links

- [Spike 002 Report](../../spikes/002-raw-decoder.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [RAW Editing Feature Specification](../../07_RAW_Editing_Feature_Specification.md)
- [System Architecture](../../03_System_Architecture.md)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [Open Questions](../questions/open-questions.md)

## Notes for LLM Agents

Do not implement RAW decoding merely because Spike 002 selected Core Image primary. The next decoder work must be fixture-backed and explicitly scoped. Do not treat Phase 5.1 preview readiness as decoded pixels. Do not add LibRaw or a Rust binding without updating `docs/DEPENDENCIES.md`.
