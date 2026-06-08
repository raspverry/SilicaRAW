---
title: Color Management
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/09_Color_Management_Specification.md
---

# Color Management

## Summary

Color management is a release-trust issue. A RAW editor can feel polished and still fail if preview or export color is silently wrong.

## Current Stance

- Color-managed preview and export require their own feasibility spike.
- Decoder-specific color assumptions must be documented.
- Benchmark fixture classes should be used for color and render tests.
- sRGB and Display P3 behavior must be verified explicitly.

## Blocked Work

- Final working color space decision.
- Export ICC behavior.
- Camera profile behavior.
- Golden image tolerance policy.
- Broad user-facing color claims.

## Links

- [Color Management Specification](../../09_Color_Management_Specification.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [RAW Decoding](raw-decoding.md)

## Notes for LLM Agents

Do not claim color correctness from compile success. Color work needs fixture-based verification and explicit assumptions.

