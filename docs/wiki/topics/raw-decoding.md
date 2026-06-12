---
title: RAW Decoding
status: active
audience: all
updated: 2026-06-12
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
- Phase 12.1 adds a feature-gated Core Image RAW probe contract and macOS metadata path, not product RAW pixels.
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

## Task 10.1 Fixture Manifest Contract

The repository has no committed legal RAW fixture corpus. Task 10.1 defines RAW fixture provenance and expected gate states only. RAW support claims remain blocked until fixture-backed Core Image probe work in Phase 12 records evidence. RAW-blocked placeholders are not decodable RAW evidence.

The fixture manifest contract separates fixture metadata from decoder behavior:

```txt
RAW Class A-E entries -> provenance, license, privacy, hash, media metadata, expected app state, expected probe state, RAW metadata, blocked decode gate
Core Image probe result -> absent until Phase 12
RAW support claim -> forbidden until fixture-backed evidence exists
```

## Task 10.2 Golden Image and Tolerance Policy

The [Golden Image and Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md) keeps RAW support claims tied to fixture-backed evidence, not file extensions or marketing language.

RAW support claims require a legal fixture manifest entry, fixture source hash verification, Core Image probe result, decode result record, original file hash preservation proof, and explicit blocked records for unsupported classes.

Task 10.2 does not add RAW decoding, Core Image probing, real RAW fixtures, or RAW support proof.

## Phase 5.1 Preview Contract

`silica-decode` can now classify preview readiness:

```txt
raster candidate -> ready by reference
unsupported catalog entry -> unsupported
RAW candidate -> Core Image RAW blocked until fixture-backed probe
```

This preserves the Spike 002 decision without pretending RAW decoding exists.

## Phase 12.1 Core Image RAW Probe Contract

`silica-decode` now exposes a proof-only Core Image RAW probe behind the non-default `core-image-raw-probe` feature.

The probe result records:

```txt
backend
platform
macos_version
source_path
source_sha256
original_file_size
original_modified_at
status
width
height
orientation
error_category
message
```

On macOS feature builds, the probe:

- reads the source file by path
- records file size and modified time before Core Image work
- computes SHA-256 for fixture evidence
- checks expected SHA-256 when supplied
- opens the source with Core Image and records image dimensions when available
- returns explicit failure categories for missing files, permission failures, source hash mismatch, Core Image open failure, missing metadata, invalid fixtures, and unknown errors

Default builds still return an unavailable probe result and do not compile the Core Image path.

Phase 12.1 does not prove RAW support. Support claims remain blocked until Task 12.2 fixture probe evidence and Task 12.3 support-matrix decisions exist.

## Phase 12.2 Fixture Probe Harness

The RAW fixture probe harness exists, but no fixture-backed RAW evidence has been recorded.

Current blocked state:

```txt
SILICARAW_RAW_FIXTURE_MANIFEST -> unset
legal local RAW fixture manifest -> not available
committed legal RAW fixture corpus -> not present
.tmp blocked placeholder RAW files -> not valid RAW proof evidence
```

The manual harness command is:

```bash
SILICARAW_RAW_FIXTURE_MANIFEST=/absolute/path/to/local/raw-fixtures.json scripts/harness/check-raw-probe-fixtures.py
```

Do not fabricate RAW samples or use user photos without a reviewed legal fixture manifest.

## Phase 12 Core Image Support Matrix

No legal RAW fixture probe evidence is available yet. The matrix records blocked status for each RAW fixture class without inferring support from file extensions.

| Fixture class | Fixture role | Fixture id | Format | Backend | Probe status | Dimensions | Orientation | Product status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | Ordinary Core Image candidate RAW | pending_legal_fixture | unknown | core_image_raw | blocked_pending_evidence | unknown | unknown | blocked_pending_evidence | Task 12.2 harness exists; no legal manifest supplied |
| B | High-risk or edge-case RAW | pending_legal_fixture | unknown | core_image_raw | blocked_pending_evidence | unknown | unknown | blocked_pending_evidence | Task 12.2 harness exists; no legal manifest supplied |
| C | Fuji RAF candidate | pending_legal_fixture | raf | core_image_raw | blocked_pending_evidence | unknown | unknown | blocked_pending_evidence | Task 12.2 harness exists; no legal manifest supplied |
| D | Apple ProRAW DNG candidate | pending_legal_fixture | dng | core_image_raw | blocked_pending_evidence | unknown | unknown | blocked_pending_evidence | Task 12.2 harness exists; no legal manifest supplied |
| E | RAW-like file expected to stay unsupported or blocked | pending_legal_fixture | unknown | core_image_raw | blocked_pending_evidence | unknown | unknown | blocked_pending_evidence | Task 12.2 harness exists; no legal manifest supplied |

LibRaw remains deferred. No fixture-backed Core Image gap has been recorded, and no decoder dependency should be added from this matrix.

## Links

- [Phase 12 RAW Proof Brief](../phases/phase-12-raw-proof.md)
- [Phase 12 Task Cards](../tasks/index.md)
- [Spike 002 Report](../../spikes/002-raw-decoder.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [RAW Editing Feature Specification](../../07_RAW_Editing_Feature_Specification.md)
- [System Architecture](../../03_System_Architecture.md)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [Open Questions](../questions/open-questions.md)

## Notes for LLM Agents

Do not implement RAW decoding merely because Spike 002 selected Core Image primary. The next decoder work must be fixture-backed and explicitly scoped. Do not treat Phase 5.1 preview readiness as decoded pixels. Do not add LibRaw or a Rust binding without updating `docs/DEPENDENCIES.md`.
