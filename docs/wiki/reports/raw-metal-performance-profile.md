---
title: RAW and Metal Performance Profile
status: active
audience: agents
updated: 2026-06-17
source_of_truth: scripts/harness/run-raw-metal-profile.py
---

# RAW and Metal Performance Profile

## Scope

Local RAW/Metal profiling evidence for this machine only; results are not universal performance guarantees. This report does not implement RAW decoding or Metal viewer behavior.

The measured rows use existing contract and feature-gated smoke paths. They keep unsupported fixture-backed RAW decode/export and full Metal pixel rendering visible instead of silently treating proxies as product throughput.

## Machine

| Field | Value |
|---|---|
| Git Commit | `e4bd0bb` |
| App Version | `0.1.0` |
| OS | `macOS-26.4-arm64-arm-64bit-Mach-O` |
| macOS | `26.4` |
| Arch | `arm64` |
| Machine Model | `Mac17,7` |
| Chip | `Apple M5 Max` |
| Memory Bytes | `137438953472` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |

## Timings

Each row records median and p95 milliseconds over 3 local runs after 1 warm-up run.

| Category | Operation | Status | RAW Fixture Class | Median ms | p95 ms | Notes |
|---|---|---|---|---:|---:|---|
| decode_time | RAW decode contract boundary | `measured_contract` | synthetic supported fixture probe contract | 54.799 | 55.065 | Measures the fixture-backed decode planning contract. It does not run Core Image RAW decode bytes unless a legal fixture manifest is provided to the separate ignored tests. |
| render_time | Viewer render scheduler boundary | `measured_contract` | decoded artifact identity contract | 54.414 | 54.716 | Measures the render request scheduler boundary only. It does not allocate Metal textures or render pixels. |
| ui_latency | Feature-gated native viewer request smoke | `measured_feature_gate` | feature-gated decoded artifact identity | 144.78 | 196.556 | Measures the feature-gated native viewer request smoke path as a UI-latency proxy. The default app path still keeps native Metal viewer behavior disabled. |
| export_time | RAW-derived export safety boundary | `measured_preflight` | synthetic RAW catalog guard | 91.298 | 98.622 | Measures the RAW-derived export safety preflight. Full fixture-backed RAW export timing remains gated on SILICARAW_RAW_FIXTURE_MANIFEST. |

## Memory

Memory pressure is recorded as child-process `max_rss_kb` observed while running each measured command. On macOS the source value is normalized from bytes to KiB.

| Category | Operation | max_rss_kb | Runs ms |
|---|---|---:|---|
| decode_time | RAW decode contract boundary | 75744.0 | `[54.799, 54.193, 55.065]` |
| render_time | Viewer render scheduler boundary | 75568.0 | `[54.159, 54.414, 54.716]` |
| ui_latency | Feature-gated native viewer request smoke | 109440.0 | `[136.9, 144.78, 196.556]` |
| export_time | RAW-derived export safety boundary | 79936.0 | `[90.931, 98.622, 91.298]` |

## Unsupported and Gated Paths

- Full fixture-backed Core Image RAW decode timing requires SILICARAW_RAW_FIXTURE_MANIFEST and legal local RAW files.
- Full fixture-backed RAW-derived JPEG export timing requires SILICARAW_RAW_FIXTURE_MANIFEST and remains outside default harness runs.
- Native Metal viewer pixel rendering remains feature-gated; this profile records request-boundary smoke timing, not GPU pixel throughput.
- UI latency is represented by the native viewer request smoke boundary; full interactive drag latency still requires an installed-app profiling pass.

## Reproduce

```bash
python3 scripts/harness/run-raw-metal-profile.py
python3 scripts/harness/check-raw-metal-profile.py
```
