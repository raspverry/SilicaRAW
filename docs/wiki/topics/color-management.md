---
title: Color Management
status: active
audience: all
updated: 2026-06-11
source_of_truth: docs/09_Color_Management_Specification.md
---

# Color Management

## Summary

Color management is a release-trust issue. Spike 003 selected the first implementation recommendation, but color correctness remains unproven until tagged fixtures exist.

## Current Stance

- Spike 003 selected Core Image/ColorSync-compatible color management first.
- The first working-space recommendation is linear Display P3-compatible RGB.
- Preview should be display-profile aware.
- Export should default to sRGB with ICC embedding and support Display P3 when explicitly selected.
- Decoder-specific color assumptions must be documented.
- Phase 5.1 records display-profile-aware preview readiness but does not prove color correctness.
- Tagged color fixtures are still missing, so sRGB and Display P3 behavior is not yet proven.

## Blocked Work

- Fixture-backed color correctness claims.
- ICC embedding proof.
- Camera profile behavior.
- Fixture-backed golden image baseline.
- Broad user-facing color claims.

## Fixture Status

Class F tagged raster fixtures are missing:

```txt
sRGB JPEG
Display P3 HEIC/JPEG
untagged JPEG
```

The repository currently contains app icons and UI mockup PNGs, not color-management fixtures.

## Task 10.1 Color Class F Contract

Color Class F covers tagged sRGB, tagged Display P3, and untagged raster fixture expectations. Hashes, profile declarations, and manifest entries do not prove color correctness. Color correctness claims remain blocked until fixture-backed proof and tolerance policy exist.

Task 10.1 records the future fixture contract only:

```txt
tagged sRGB raster -> embedded ICC expected, sRGB input expectation
tagged Display P3 raster -> embedded ICC expected, Display P3 input expectation
untagged raster -> no embedded ICC expected, assume_srgb policy
```

These entries do not prove color correctness because no fixture-backed transform output, tolerance policy, or visual review evidence exists yet.

## Task 10.2 Golden Image and Tolerance Policy

Task 10.2 records the first [Golden Image and Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md).

The policy separates byte equality, file/profile inspection, pixel or perceptual tolerance, and manual visual review. It does not add golden images, ICC parsing, pixel comparison, or color correctness proof.

Color correctness claims remain blocked until fixture-backed proof, approved tolerance values, automated comparison results, and manual visual review records exist.

## Color-Dependent Tags

```txt
Color Baseline: color-blocking
Preview Transform: color-blocking
Export ICC: color-blocking
Golden Images: color-dependent
RAW Camera Profiles: decoder-dependent color-dependent
```

## Links

- [Spike 003 Report](../../spikes/003-color-managed-preview-export.md)
- [Color Management Specification](../../09_Color_Management_Specification.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [RAW Decoding](raw-decoding.md)

## Notes for LLM Agents

Do not claim color correctness from Spike 003 or compile success. Spike 003 records an implementation direction, not fixture-backed color proof.
