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

The RAW fixture probe harness exists and now emits a JSON report after running the ignored fixture test.

Current local fixture state:

```txt
SILICARAW_RAW_FIXTURE_MANIFEST -> .tmp/legal-raw-fixtures/raw-fixtures.json in this maintainer workspace
legal local RAW fixture manifest -> available locally, ignored by git
committed legal RAW fixture corpus -> not present
.tmp blocked placeholder RAW files -> not valid RAW proof evidence
```

The manual harness command is:

```bash
SILICARAW_RAW_FIXTURE_MANIFEST=/absolute/path/to/local/raw-fixtures.json scripts/harness/check-raw-probe-fixtures.py
```

Do not fabricate RAW samples or use user photos without a reviewed legal fixture manifest.

## Phase 12.5 Legal Fixture Source Review

[raw.pixls.us Source Review](../sources/raw-pixls-us.md) accepts raw.pixls.us as the first external source for local ignored RAW probe fixtures.

Accepted local-only candidates:

| Fixture class | Candidate id | Format | License | SHA-256 | Status |
| --- | --- | --- | --- | --- | --- |
| A | `raw_pixls_canon_eos_7d_cr2_raw_3_2` | cr2 | CC0 1.0 Universal | `b5e47c5fcf7332ac03e0134926f17a338a42e68c1fd7f83e16f45f4b767544e8` | accepted_for_local_ignored_probe |
| B | `raw_pixls_canon_eos_r6_mark_iii_cr3_full_frame` | cr3 | CC0 1.0 Universal | `e491e4bb960961b5fa299361bf698310a80cbe7b15d30d8dad3bb21bc5457dab` | accepted_for_local_ignored_probe |
| C | `raw_pixls_fujifilm_x_t30_iii_raf_compressed` | raf | CC0 1.0 Universal | `49f77d6162abfa5c94d2d8b90e4e926b7386c42bcf7e84a152c9ffe1ebd584da` | accepted_for_local_ignored_probe |
| D | `raw_pixls_apple_iphone_12_pro_dng` | dng | CC0 1.0 Universal | `e91e77a4533ed7cce551d83330676ea5c47dd5e55fb38adda7819366afdbdfc2` | accepted_for_local_ignored_probe |

Fixture class E remains pending source review. No candidate is probe evidence until the file is downloaded into an ignored local path, hash-verified, declared in a local fixture manifest, and run through `scripts/harness/check-raw-probe-fixtures.py`.

## Phase 12.5 Local Probe Evidence

The local ignored fixture manifest ran successfully on macOS 26.4.

Evidence command:

```bash
SILICARAW_RAW_FIXTURE_MANIFEST=/Users/hansol/dev/personal/SilicaRAW/.tmp/legal-raw-fixtures/raw-fixtures.json scripts/harness/check-raw-probe-fixtures.py
```

Evidence summary:

| Fixture class | Fixture id | Format | Probe status | Dimensions | Orientation | Original hash unchanged |
| --- | --- | --- | --- | --- | --- | --- |
| A | `raw_pixls_canon_eos_7d_cr2_raw_3_2` | cr2 | success | 5184 x 3456 | unknown | true |
| B | `raw_pixls_canon_eos_r6_mark_iii_cr3_full_frame` | cr3 | success | 6960 x 4640 | unknown | true |
| C | `raw_pixls_fujifilm_x_t30_iii_raf_compressed` | raf | success | 6240 x 4160 | unknown | true |
| D | `raw_pixls_apple_iphone_12_pro_dng` | dng | success | 4032 x 3024 | unknown | true |

This evidence proves Core Image can open these local legal fixtures and report dimensions. It does not prove final color correctness, camera profiles, lens correction, product UI RAW display, or export behavior.

## Phase 12 Core Image Support Matrix

The matrix records fixture-backed Core Image support for classes A-D and keeps class E blocked pending a legal source decision.

| Fixture class | Fixture role | Fixture id | Format | Backend | Probe status | Dimensions | Orientation | Product status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | Ordinary Core Image candidate RAW | `raw_pixls_canon_eos_7d_cr2_raw_3_2` | cr2 | core_image_raw | success | 5184 x 3456 | unknown | core_image_supported | Task 12.5 local ignored manifest probe on macOS 26.4 |
| B | High-risk or edge-case RAW | `raw_pixls_canon_eos_r6_mark_iii_cr3_full_frame` | cr3 | core_image_raw | success | 6960 x 4640 | unknown | core_image_supported | Task 12.5 local ignored manifest probe on macOS 26.4 |
| C | Fuji RAF candidate | `raw_pixls_fujifilm_x_t30_iii_raf_compressed` | raf | core_image_raw | success | 6240 x 4160 | unknown | core_image_supported | Task 12.5 local ignored manifest probe on macOS 26.4 |
| D | Apple ProRAW DNG candidate | `raw_pixls_apple_iphone_12_pro_dng` | dng | core_image_raw | success | 4032 x 3024 | unknown | core_image_supported | Task 12.5 local ignored manifest probe on macOS 26.4 |
| E | RAW-like file expected to stay unsupported or blocked | pending_legal_fixture | unknown | core_image_raw | blocked_pending_evidence | unknown | unknown | blocked_pending_evidence | Source decision still pending; no legal manifest entry supplied |

LibRaw remains deferred. No fixture-backed Core Image gap has been recorded, and no decoder dependency should be added from this matrix.

## Phase 12.4 Product RAW Decode API Contract

`silica-decode` and `silica-core` now expose a product RAW decode planning contract.

Current behavior:

```txt
RAW candidate -> BlockedPendingEvidence
non-RAW candidate -> BlockedUnsupportedClass
successful probe evidence for fixture classes A-D -> Supported metadata-only plan
class E, unknown class, failed probe, or missing dimensions -> blocked state
```

The API returns backend, status, optional dimensions, optional orientation, and a UI-suitable message. It does not return decoded pixels and does not trigger rendering, cache writes, export, or original-file mutation.

The path-only `plan_product_raw_decode` stays conservative and does not infer support from file extension alone. The evidence-driven `plan_product_raw_decode_from_probe` maps only fixture-backed Core Image probe results to metadata-only product plans.

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
