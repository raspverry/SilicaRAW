# Phase 15 Vertical Slice Evidence Gate

Status: complete for Task 15.0 on 2026-06-12.

This checklist records the exact evidence that allows Phase 15 RAW, color, and Metal implementation to begin. It is not a broad RAW support claim, a camera support claim, or a visual color correctness claim.

## Gate Decision

Phase 15 may proceed to Task 15.1 with a fixture-limited product vertical slice.

Allowed RAW fixture scope:

| Class | Fixture id | Format | Evidence state | Phase 15 use |
| --- | --- | --- | --- | --- |
| A | `raw_pixls_canon_eos_7d_cr2_raw_3_2` | CR2 | Core Image probe success, original hash unchanged | Required minimum path |
| B | `raw_pixls_canon_eos_r6_mark_iii_cr3_full_frame` | CR3 | Core Image probe success, original hash unchanged | Optional edge-case coverage |
| C | `raw_pixls_fujifilm_x_t30_iii_raf_compressed` | RAF | Core Image probe success, original hash unchanged | Recommended higher-risk path |
| D | `raw_pixls_apple_iphone_12_pro_dng` | DNG | Core Image probe success, original hash unchanged | Recommended higher-risk path |
| E | `pending_legal_fixture` | unknown | blocked pending evidence | Not allowed |

Color fixture scope:

| Class | Subclass | Fixture id | Evidence state | Phase 15 use |
| --- | --- | --- | --- | --- |
| F | `srgb_jpeg` | `silicaraw_synthetic_srgb_jpeg` | profile probe success, embedded ICC true, original hash unchanged | Required ICC/profile regression |
| F | `display_p3_jpeg` | `silicaraw_synthetic_display_p3_jpeg` | profile probe success, embedded ICC true, original hash unchanged | Required ICC/profile regression |
| F | `untagged_jpeg` | `silicaraw_synthetic_untagged_jpeg` | profile probe success, assume-sRGB path, original hash unchanged | Required untagged policy regression |

## Local Evidence Commands

RAW fixture evidence was rechecked with:

```bash
SILICARAW_RAW_FIXTURE_MANIFEST=/Users/hansol/dev/personal/SilicaRAW/.tmp/legal-raw-fixtures/raw-fixtures.json scripts/harness/check-raw-probe-fixtures.py
```

Result summary:

```txt
A CR2 -> success, 5184 x 3456, original_hash_unchanged=true
B CR3 -> success, 6960 x 4640, original_hash_unchanged=true
C RAF -> success, 6240 x 4160, original_hash_unchanged=true
D DNG -> success, 4032 x 3024, original_hash_unchanged=true
```

Color fixture evidence was rechecked with:

```bash
SILICARAW_COLOR_FIXTURE_MANIFEST=/Users/hansol/dev/personal/SilicaRAW/.tmp/legal-color-fixtures/color-fixtures.json scripts/harness/check-color-probe-fixtures.py
```

Result summary:

```txt
srgb_jpeg -> success, srgb, embedded ICC, original hash unchanged
display_p3_jpeg -> success, display_p3, embedded ICC, original hash unchanged
untagged_jpeg -> success, none, assume_srgb path, original hash unchanged
```

The fixture media and manifests are local ignored artifacts. They are valid for this maintainer-machine proof path but must not be committed.

## Required Boundaries

- Do not infer RAW support from file extension alone.
- Do not allow RAW class E or unknown RAW classes into Phase 15 product behavior.
- Do not claim broad camera support from classes A-D.
- Do not claim visual color correctness from Class F profile-probe or ICC evidence.
- Do not use viewer textures as export source of truth.
- Do not overwrite or mutate original fixture files.
- Do not add LibRaw, MLX, MCP, plugin runtime, telemetry, auto-update, Homebrew, or Mac App Store scope.

## Next Task

Proceed to [Task 15.1: Decoded Image Handoff Contract](../docs/wiki/tasks/15.1-decoded-image-handoff-contract.md).
