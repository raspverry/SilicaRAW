# Color Export Manual QA Checklist

Status: ready for Task 13.6 manual review
Updated: 2026-06-12

## Purpose

This checklist records the manual Preview.app or Photos review gate for exported color files.

Task 13.6 proves JPEG ICC embedding by file/profile inspection. This checklist does not claim visual color correctness until a reviewer executes it and records the result.

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

No manual visual review result has been recorded yet.

Color correctness claims remain blocked until this checklist is executed with approved tolerance evidence.
