# Native Viewer Lifecycle QA Checklist

Status: Task 14.4 lifecycle proof recorded
Updated: 2026-06-12

## Purpose

This checklist records the feature-gated native viewer lifecycle proof for Phase 14.

Task 14.4 proves product-module lifecycle bookkeeping for install, resize, Retina drawable sizing, neutral render timing, and cleanup. It does not install the final product image viewer, render RAW pixels, render exposure or contrast in Metal, allocate product textures, or claim visual correctness.

## Automated Smoke

Run on macOS:

```sh
cargo test -p silica-desktop --features native-metal-viewer lifecycle_smoke_evidence_is_neutral_and_reviewable -- --nocapture
```

Expected evidence shape:

```txt
[SilicaRAW Native Viewer] surface=develop state=uninstalled drawable=1200x675px backing_scale=1.50 resize_events=1 frames=1 last_frame_ms=16.667 preferred_fps=60 cleanup=window-closed neutral_clear_only=true frame_timing_ms=16.667 cleanup_supported=app-closed,window-closed
```

## Recorded Evidence

```txt
date: 2026-06-12
machine: local macOS development machine
command: cargo test -p silica-desktop --features native-metal-viewer lifecycle_smoke_evidence_is_neutral_and_reviewable -- --nocapture
result: passed
evidence:
[SilicaRAW Native Viewer] surface=develop state=uninstalled drawable=1200x675px backing_scale=1.50 resize_events=1 frames=1 last_frame_ms=16.667 preferred_fps=60 cleanup=window-closed neutral_clear_only=true frame_timing_ms=16.667 cleanup_supported=app-closed,window-closed
```

## Review Fields

Record these fields when rerunning the proof:

```txt
reviewer:
date:
git commit:
macOS version:
command:
surface:
initial logical size:
resized logical size:
backing scale:
drawable size:
resize event count:
frame count:
last frame timing:
cleanup reason:
neutral proof only: yes | no
pass/fail:
notes:
```

## Pass Criteria

1. The command runs with `--features native-metal-viewer`.
2. Evidence includes a reserved surface, drawable pixel size, backing scale, resize count, frame count, render timing, cleanup reason, and `neutral_clear_only=true`.
3. The default desktop build still passes without the feature.
4. No original files, catalog rows, sidecars, export files, RAW pixels, or product textures are written by this proof.
