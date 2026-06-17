# Color Export Manual QA Checklist

Status: Task 20.5 Display P3 Export Enablement recorded
Updated: 2026-06-17

## Purpose

This checklist records the manual Preview.app or Photos review gate for exported color files.

Task 13.6 proves JPEG ICC embedding by file/profile inspection. This checklist does not claim visual color correctness until a reviewer executes it and records the result.

## Task 20.5 Display P3 Export Enablement

Display P3 export remains explicit. sRGB remains the default. The local alpha exposes Display P3 only as a JPEG profile/ICC capability claim, not a visual color-correctness claim.

Reviewers must verify:

```txt
default export target remains srgb
Display P3 requires explicit selection
Display P3 output embeds inspectable ICC evidence
PNG/TIFF Display P3 remains blocked until separate proof
manual review notes do not claim broad color correctness
```

## Required Evidence

Record these fields for each review:

```txt
reviewer:
date:
git commit:
fixture manifest path:
source fixture id:
export target: srgb | display_p3
export command or app artifact:
output path:
output SHA-256:
embedded ICC profile:
embedded ICC SHA-256:
original source SHA-256 before:
original source SHA-256 after:
macOS version:
viewer: Preview.app | Photos
display model or display profile:
observed issue notes:
pass/fail:
```

## Review Steps

1. Verify the source fixture hash matches the reviewed manifest.
2. Export to a separate output path.
3. Verify the original source hash is unchanged.
4. Verify the exported JPEG embeds the expected ICC profile.
5. Open the exported JPEG in Preview.app or Photos.
6. Record visible clipping, hue shifts, saturation surprises, banding, or profile warnings.
7. Mark pass only if the profile inspection passed and no blocking visual issue was observed.

## Current State

Task 15.6 records one fixture-backed RAW-derived JPEG sRGB review in [Phase 15 RAW Export Manual Color QA](PHASE_15_RAW_EXPORT_MANUAL_QA.md).

Color correctness claims remain blocked until broader review and approved tolerance evidence exist.
