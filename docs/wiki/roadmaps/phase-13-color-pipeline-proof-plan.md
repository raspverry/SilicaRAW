---
title: Phase 13 Color Pipeline Proof Plan
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Phase 13 Color Pipeline Proof Plan

## Summary

Phase 13 proves the Core Image/ColorSync-compatible color path before SilicaRAW expands preview or export color claims.

This is an evidence phase. It is not a broad color-editor implementation phase.

## Current Status

As of 2026-06-12:

- Spike 003 selected Core Image/ColorSync-compatible color management first.
- The recommended working space is linear Display P3-compatible RGB.
- Export policy is sRGB by default with ICC embedding and Display P3 only when explicitly selected.
- Class F fixture expectations exist in the fixture manifest schema and example.
- The golden image tolerance policy exists as a policy baseline.
- No legal local Color Class F fixture corpus has been selected yet.
- No color probe, ICC proof, tolerance result, manual visual review, or color correctness claim exists yet.
- Task 13.0 design gate is complete.
- Task 13.1 color fixture source review is complete with a local-only synthetic fixture source.
- Task 13.2 local color fixture corpus and manifest is complete locally.
- Task 13.3 feature-gated color profile probe is complete.
- Task 13.4 color probe harness is complete.
- Task 13.5 color support matrix is next.

## Goal

Create fixture-backed evidence for:

- tagged sRGB raster handling
- tagged Display P3 raster handling
- untagged raster `assume_srgb` handling
- export ICC embedding for sRGB and Display P3
- schema-safe color metadata propagation
- explicit export color options after proof

## Non-Goals

- Do not claim color correctness from fixture labels, compile success, or JPEG export success.
- Do not add RAW color-profile behavior in this phase.
- Do not add HDR behavior.
- Do not claim parity with Preview.app, Photos, Lightroom, Capture One, or camera vendor renderers.
- Do not add broad fallback systems or fake fixtures.
- Do not add dependencies without updating `docs/DEPENDENCIES.md`.
- Do not mutate original image files.

## Task Sequence

### Task 13.0: Design Gate

- **Card:** [13.0 Phase 13 Design Gate](../tasks/13.0-phase-13-design-gate.md)
- **Status:** complete
- **Output:** Phase 13 scope, stop gates, dependency gate, and atomic task order are documented before code work.
- **Validation:** `python3 scripts/harness/check-md-links.py`, `scripts/harness/check.sh`

### Task 13.1: Color Fixture Source Review

- **Card:** [13.1 Color Fixture Source Review](../tasks/13.1-color-fixture-source-review.md)
- **Status:** complete
- **Output:** Legal source review for one tagged sRGB JPEG, one tagged Display P3 JPEG, and one untagged JPEG.
- **Validation:** Manual source review recorded in wiki docs; fixture media remains uncommitted unless commit permission is explicitly proven.

### Task 13.2: Local Color Fixture Corpus and Manifest

- **Card:** [13.2 Local Color Fixture Corpus and Manifest](../tasks/13.2-color-fixture-local-manifest.md)
- **Status:** complete locally
- **Output:** Ignored local Class F fixture files and a local fixture manifest compatible with `schemas/fixture_manifest.schema.json`.
- **Validation:** `git status --short` shows no fixture media staged.

### Task 13.3: Feature-Gated Color Profile Probe

- **Card:** [13.3 Feature-Gated Color Profile Probe](../tasks/13.3-feature-gated-color-profile-probe.md)
- **Status:** complete
- **Output:** `silica-render` exposes a non-default `color-probe` path that records profile and transform metadata without changing normal product behavior.
- **Validation:** `cargo test -p silica-render --features color-probe`

### Task 13.4: Color Probe Harness

- **Card:** [13.4 Color Probe Harness](../tasks/13.4-color-probe-harness.md)
- **Status:** complete
- **Output:** A manifest-driven harness can run ignored color probe tests and check original hash preservation.
- **Validation:** `SILICARAW_COLOR_FIXTURE_MANIFEST=... scripts/harness/check-color-probe-fixtures.py`

### Task 13.5: Color Support Matrix

- **Card:** [13.5 Color Support Matrix](../tasks/13.5-color-support-matrix.md)
- **Status:** next
- **Output:** [Color Management](../topics/color-management.md) records fixture-backed states for sRGB, Display P3, and untagged raster classes.
- **Validation:** `python3 scripts/harness/check-md-links.py`, `python3 scripts/harness/check-cargo-deps.py`

### Task 13.6: ICC Export Proof

- **Card:** [13.6 ICC Export Proof](../tasks/13.6-icc-export-proof.md)
- **Status:** planned
- **Output:** Exported sRGB and Display P3 JPEGs have fixture-backed ICC embedding evidence and original-preservation evidence.
- **Validation:** color export tests and manual color QA checklist.

### Task 13.7: Color Metadata Contract

- **Card:** [13.7 Color Metadata Contract](../tasks/13.7-color-metadata-contract.md)
- **Status:** planned
- **Output:** Existing edit graph/profile fields carry color metadata without inventing hidden schema fields.
- **Validation:** `cargo test -p silica-edit -p silica-render -p silica-export`

### Task 13.8: Explicit Export Color Options

- **Card:** [13.8 Explicit Export Color Options](../tasks/13.8-explicit-export-color-options.md)
- **Status:** planned
- **Output:** Export UI/API makes sRGB default and Display P3 explicit only after fixture-backed proof.
- **Validation:** export UI smoke and color export tests.

## Completion Gate

Phase 13 is not complete until all of these are true:

- Legal Color Class F fixture evidence is recorded.
- Profile probe evidence exists for tagged sRGB, tagged Display P3, and untagged raster fixtures.
- Export ICC embedding evidence exists for sRGB and Display P3.
- Original image hashes remain unchanged across probe/export workflows.
- Manual Preview.app or Photos review checklist exists and is ready for execution.
- Color correctness claims remain blocked unless approved tolerance and visual review evidence exist.

## Links

- [Phase 13 Color Pipeline Proof Brief](../phases/phase-13-color-pipeline-proof.md)
- [Color Management](../topics/color-management.md)
- [Spike 003 Color-Managed Preview and Export](../../spikes/003-color-managed-preview-export.md)
- [Golden Image and Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md)
- [Fixture Manifest Schema](../../../schemas/fixture_manifest.schema.json)

## Notes for LLM Agents

Treat this phase as proof work. If a task needs a new dependency, update `docs/DEPENDENCIES.md` in the same atomic change. Do not infer color correctness from file labels, screenshots, or successful JPEG encoding.
