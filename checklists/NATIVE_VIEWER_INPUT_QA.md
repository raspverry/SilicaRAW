# Native Viewer Input QA Checklist

Status: Task 14.5 input ownership proof recorded
Updated: 2026-06-12

## Purpose

This checklist records the feature-gated native viewer input ownership proof for Phase 14.

Task 14.5 proves that viewer-surface mouse down, mouse drag, scroll, and magnify samples are native-owned only when they fall inside the reserved viewer rectangle. Events outside that rectangle remain web-owned. This checklist does not enable telemetry, analytics, persistent input logging, product image rendering, or global shortcut capture.

## Automated Smoke

Run on macOS:

```sh
cargo test -p silica-desktop --features native-metal-viewer input_smoke_evidence_is_manual_review_ready -- --nocapture
```

Expected evidence shape:

```txt
[SilicaRAW Native Viewer] surface=develop native_events=4 web_events=1 mouse_down=true mouse_drag=true scroll=true magnify=true web_controls_external=true remote_reporting=false persistent_input_log=false
```

## Recorded Evidence

```txt
date: 2026-06-12
machine: local macOS development machine
command: cargo test -p silica-desktop --features native-metal-viewer input_smoke_evidence_is_manual_review_ready -- --nocapture
result: passed
evidence:
[SilicaRAW Native Viewer] surface=develop native_events=4 web_events=1 mouse_down=true mouse_drag=true scroll=true magnify=true web_controls_external=true remote_reporting=false persistent_input_log=false
```

## Review Fields

Record these fields when rerunning or extending the proof:

```txt
reviewer:
date:
git commit:
macOS version:
command:
surface:
reserved viewer rectangle:
mouse down inside viewer: native-owned | failed
mouse drag inside viewer: native-owned | failed
scroll inside viewer: native-owned | failed
magnify inside viewer: native-owned | failed
click outside viewer: web-owned | failed
modifier key notes:
focus behavior notes:
web button/slider/input interaction outside viewer:
remote reporting enabled: no | yes
persistent input log enabled: no | yes
pass/fail:
notes:
```

## Pass Criteria

1. The command runs with `--features native-metal-viewer`.
2. Evidence includes native-owned mouse down, mouse drag, scroll, and magnify samples inside the reserved viewer rectangle.
3. Evidence includes at least one web-owned event outside the reserved viewer rectangle.
4. `web_controls_external=true`, `remote_reporting=false`, and `persistent_input_log=false`.
5. No catalog rows, sidecars, original files, export files, analytics, telemetry, or persistent input logs are written by this proof.
