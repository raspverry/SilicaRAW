---
title: Color Management
status: active
audience: all
updated: 2026-06-12
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
- Phase 13 now has an execution plan, brief, and task cards for fixture-backed color proof.

## Blocked Work

- Fixture-backed color correctness claims.
- ICC embedding proof.
- Camera profile behavior.
- Fixture-backed golden image baseline.
- Broad user-facing color claims.

## Fixture Status

Class F tagged raster fixture source review is complete for local-only synthetic fixtures:

```txt
srgb_jpeg -> synthetic pixels + local macOS sRGB ICC, local-only
display_p3_jpeg -> synthetic pixels + local macOS Display P3 ICC, local-only
untagged_jpeg -> synthetic pixels + removed color-management properties, local-only
```

No committed color-management fixture corpus exists yet. Task 13.2 generated an ignored local corpus and manifest:

```txt
manifest: .tmp/legal-color-fixtures/color-fixtures.json
srgb_jpeg sha256: ba7fed85d6fdd5d2dbf8376fdb4030e2206a541e58efa2c69ec5ba494152b6fe
display_p3_jpeg sha256: 84855ac721fbc8062ef543f5fe95df843e6e4a211be2eac2869e3456c55e1ebb
untagged_jpeg sha256: aff808a1c3625a5de3e249c80c3cb9d7e9ae53d92f6bd95158aa4a0f384a23e9
```

This local corpus proves only that legal local fixture files and hashes exist. It does not prove profile parsing, transforms, export ICC embedding, or color correctness.

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

## Phase 13 Plan

[Phase 13 Color Pipeline Proof Plan](../roadmaps/phase-13-color-pipeline-proof-plan.md) is the active route for turning the Spike 003 recommendation into fixture-backed evidence.

Current planned order:

```txt
13.0 design gate
13.1 source review [complete]
13.2 ignored local fixtures and manifest [complete locally]
13.3 feature-gated color probe [complete]
13.4 probe harness [next]
13.5 support matrix
13.6 ICC export proof
13.7 schema-safe color metadata
13.8 explicit export color options
```

Until those tasks produce evidence, color correctness and Display P3 export claims remain blocked.

Task 13.3 added a non-default `color-probe` feature in `silica-render`. It records JPEG profile metadata and source hashes for fixture proof only. It does not apply a ColorSync transform, write ICC profiles, render pixels, or prove color correctness.

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
- [Phase 13 Color Pipeline Proof Plan](../roadmaps/phase-13-color-pipeline-proof-plan.md)
- [Synthetic Local Color Fixture Source Review](../sources/color-fixtures-synthetic-local.md)
- [Color Management Specification](../../09_Color_Management_Specification.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [RAW Decoding](raw-decoding.md)

## Notes for LLM Agents

Do not claim color correctness from Spike 003 or compile success. Spike 003 records an implementation direction, not fixture-backed color proof.
