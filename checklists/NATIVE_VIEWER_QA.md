# Native Viewer QA Checklist

Status: Task 14.8 QA harness and checklist ready
Updated: 2026-06-12

## Purpose

This checklist separates automated proof from manual proof for the Phase 14 product native viewer bridge.

The default CI harness checks this checklist, the static reserved-layout contract, and wiki routing without requiring `native-metal-viewer`. Feature-gated macOS checks are documented here for maintainers who explicitly run the native viewer proof path.

## Automated Proof

Default CI command:

```sh
scripts/harness/check.sh
```

The default CI path runs `python3 scripts/harness/check-native-viewer-qa.py`. It verifies:

- this checklist exists
- Metal Rendering links to this checklist
- static UI keeps `data-native-viewer-host="reserved"`
- static UI exposes `SilicaRAWViewerHost`
- manual proof commands and viewport targets are present

This is automated proof, not manual proof. It does not run feature-gated AppKit/Metal proof commands by default CI.

## Feature-Gated macOS Commands

Run these manually on macOS when reviewing native viewer bridge work:

```sh
cargo test -p silica-desktop --features native-metal-viewer lifecycle_smoke_evidence_is_neutral_and_reviewable -- --nocapture
cargo test -p silica-desktop --features native-metal-viewer input_smoke_evidence_is_manual_review_ready -- --nocapture
cargo test -p silica-desktop --features native-metal-viewer render_request_smoke_evidence_is_reviewable -- --nocapture
cargo test -p silica-desktop --features native-metal-viewer texture_lifecycle_smoke_evidence_is_reviewable -- --nocapture
```

Optional runtime evidence env vars:

```sh
SILICA_NATIVE_VIEWER_LIFECYCLE_PROOF=1 cargo run -p silica-desktop --features native-metal-viewer
SILICA_NATIVE_VIEWER_INPUT_PROOF=1 cargo run -p silica-desktop --features native-metal-viewer
SILICA_NATIVE_VIEWER_RENDER_REQUEST_PROOF=1 cargo run -p silica-desktop --features native-metal-viewer
SILICA_NATIVE_VIEWER_TEXTURE_LIFECYCLE_PROOF=1 cargo run -p silica-desktop --features native-metal-viewer
```

## Manual Proof Scope

Record manual proof separately from automated proof.

Required manual surfaces:

- mouse down inside viewer
- mouse drag inside viewer
- scroll inside viewer
- magnify inside viewer
- click outside viewer and confirm web controls remain web-owned
- resize the app window and record drawable size behavior
- verify Retina backing scale evidence
- move across an external display if available and record scale/resize notes
- confirm UI responsiveness while the feature-gated proof is active

Required viewport screenshot targets:

```txt
1280x800
1440x900
1728x965
```

## Evidence Form

```txt
reviewer:
date:
git commit:
macOS version:
machine:
display setup:
feature command:
automated proof command:
manual proof: yes | no
mouse down:
mouse drag:
scroll:
magnify:
web controls:
resize:
Retina:
external display:
UI responsiveness:
viewport 1280x800:
viewport 1440x900:
viewport 1728x965:
default CI result:
feature-gated result:
pass/fail:
notes:
```

## Current Recorded Proof

Task 14.4 lifecycle:

```txt
[SilicaRAW Native Viewer] surface=develop state=uninstalled drawable=1200x675px backing_scale=1.50 resize_events=1 frames=1 last_frame_ms=16.667 preferred_fps=60 cleanup=window-closed neutral_clear_only=true frame_timing_ms=16.667 cleanup_supported=app-closed,window-closed
```

Task 14.5 input:

```txt
[SilicaRAW Native Viewer] surface=develop native_events=4 web_events=1 mouse_down=true mouse_drag=true scroll=true magnify=true web_controls_external=true remote_reporting=false persistent_input_log=false
```

Task 14.6 render request:

```txt
[SilicaRAW Native Viewer] latest_request=12 replaced_request=11 latest_wins=true catalog_write_requested=false contains_image_pixels=false future_texture_identity=true
```

Task 14.7 texture lifecycle:

```txt
[SilicaRAW Native Viewer] state=released release_count=4 last_release=app-closed catalog_write=false sidecar_write=false original_write_destination=false persistent_gpu_cache=false rebuildable=true
```

## Pass Criteria

1. Automated proof passes through `scripts/harness/check.sh`.
2. Manual proof records mouse, drag, scroll, magnify, resize, Retina, optional external display movement, and UI responsiveness.
3. Feature-gated proof is clearly marked and never confused with default CI proof.
4. Web controls outside the reserved viewer rectangle remain web-owned.
5. No RAW pixels, shader passes, product texture allocation, catalog writes, sidecar writes, original-file mutation, analytics, or persistent input logs are introduced by Phase 14 QA.
